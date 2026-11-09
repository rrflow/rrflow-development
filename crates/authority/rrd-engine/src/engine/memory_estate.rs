use super::memory_engram::{canonical, required_digest, required_string};
use super::*;
use rrd_contract::{
    AssembleContext, ContextReadStamp, MemoryEstatePlan, MemoryWarp, PersistMemoryEstate,
    ProviderRepresentation, ResolveMemoryWarp, ResolveSeatIdentity, ResolvedMemoryWarp,
    SeatIdentity, MEMORY_PROVIDER_IDENTITY_KIND, MEMORY_REPRESENTS_KIND, MEMORY_SEAT_KIND,
};

impl RrdEngine {
    /// Builds canonical schema/record/relation mutations for one durable seat.
    /// The returned plan is committed through the existing authenticated data
    /// transaction API; this method creates no side registry or write path.
    #[allow(clippy::too_many_arguments)]
    pub fn plan_memory_estate(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &PersistMemoryEstate,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<MemoryEstatePlan> {
        self.authorize(
            session_id,
            token,
            SecurityAction::TransactionPreview,
            now,
            request_id,
            operation_id,
        )?;
        let scope = self.query_scope(&request.scope)?;
        let current = self
            .storage
            .runtime()
            .schema(&scope)?
            .as_ref()
            .map(public_schema)
            .transpose()?;
        build_memory_estate_plan(request, current)
    }

    /// Resolves a stable `rrflow://` coordinate through the same context
    /// planner used by every other engine surface.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_memory_warp(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &ResolveMemoryWarp,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ResolvedMemoryWarp> {
        request
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let (_, _, authorization) = self.authorize_resource_without_data_policy(
            session_id,
            token,
            SecurityAction::MemoryContextRead,
            &self.instance_resource(),
            now,
            request_id,
            operation_id,
        )?;
        self.query_scope(&request.scope)?;
        let warp = MemoryWarp::parse(&request.uri)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        if warp.instance != self.instance {
            return Err(ServiceError::WrongScope);
        }
        let context = self.assemble_context_at(
            &AssembleContext {
                scope: request.scope.clone(),
                query: request.query.clone(),
                valid_at: request.valid_at,
                seeds: vec![warp.target.clone()],
                max_graph_depth: request.max_graph_depth,
                max_items: request.max_items,
                max_output_bytes: request.max_output_bytes,
                max_storage_keys: request.max_storage_keys,
            },
            authorization.as_ref(),
        )?;
        let expected = rrd_core::result_identity::typed(
            rrd_core::result_identity::ResultIdentityFamily::Record,
            &runtime_ref(&warp.target)?,
        );
        if !context.items.iter().any(|item| item.identity == expected) {
            return Err(ServiceError::MemoryTargetNotFound);
        }
        let resolved = ResolvedMemoryWarp {
            uri: warp.uri(),
            target: warp.target,
            context,
        };
        resolved
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        Ok(resolved)
    }

    /// Resolves a provider-independent seat as self only while at least one
    /// current provider identity represents it in the same temporal snapshot.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_seat_identity(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &ResolveSeatIdentity,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<SeatIdentity> {
        request
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        self.authorize(
            session_id,
            token,
            SecurityAction::MemoryContextRead,
            now,
            request_id,
            operation_id,
        )?;
        let scope = self.query_scope(&request.scope)?;
        let key_budget = usize::try_from(request.max_storage_keys)
            .map_err(|_| ServiceError::Query("seat key budget exceeds usize".into()))?;
        let RuntimeDataSnapshotRead {
            read,
            selected_versions,
            read_evidence,
            snapshot,
        } = self
            .storage
            .runtime()
            .data_snapshot(&scope, request.valid_at, key_budget)?;
        if snapshot.known_at_cursor != read.commit_cursor
            || snapshot.schema_revision != read.schema_revision.unwrap_or(0)
        {
            return Err(ServiceError::StorageConflict(
                "seat snapshot differs from its captured read stamp".into(),
            ));
        }

        let seat_reference =
            RuntimeRef::new(MEMORY_SEAT_KIND, request.seat_id.as_str()).map_err(core_contract)?;
        let seat = snapshot
            .records
            .iter()
            .find(|entry| entry.value.reference == seat_reference)
            .ok_or(ServiceError::MemoryTargetNotFound)?;
        let display_name = required_string(&seat.value.properties, "display_name")?;
        let purpose = required_string(&seat.value.properties, "purpose")?;
        let provider_records = snapshot
            .records
            .iter()
            .filter(|entry| entry.value.reference.kind.as_str() == MEMORY_PROVIDER_IDENTITY_KIND)
            .map(|entry| (entry.value.reference.clone(), &entry.value))
            .collect::<BTreeMap<_, _>>();
        let mut representations = Vec::new();
        for relation in snapshot.relations.iter().filter(|entry| {
            entry.value.reference.kind.as_str() == MEMORY_REPRESENTS_KIND
                && entry.value.to == seat_reference
        }) {
            let Some(provider_identity) = provider_records.get(&relation.value.from) else {
                return Err(ServiceError::Storage(
                    "seat representation references a missing provider identity".into(),
                ));
            };
            let provider =
                CanonicalId::new(required_string(&provider_identity.properties, "provider")?)
                    .map_err(|error| ServiceError::Storage(error.to_string()))?;
            let subject_sha256 = required_digest(&provider_identity.properties, "subject_sha256")?;
            representations.push(ProviderRepresentation {
                id: canonical(relation.value.reference.id.as_str())?,
                provider_identity: canonical(provider_identity.reference.id.as_str())?,
                provider,
                subject_sha256,
            });
        }
        representations.sort_by(|left, right| left.id.cmp(&right.id));
        if representations.is_empty() {
            return Err(ServiceError::SeatNotRepresented);
        }
        let identity = SeatIdentity {
            uri: MemoryWarp::new(
                self.instance.clone(),
                DataReference {
                    kind: canonical(MEMORY_SEAT_KIND)?,
                    id: request.seat_id.clone(),
                },
            )
            .uri(),
            seat_id: request.seat_id.clone(),
            display_name,
            purpose,
            representations,
            read: ContextReadStamp {
                runtime_manifest_sha256: read.manifest_id,
                runtime_cursor: read.commit_cursor,
                schema_revision: read.schema_revision,
                catalogue_revision: read.catalog_revision,
            },
            selected_versions,
            read_evidence: crate::runtime::public_read_evidence(read_evidence)?,
        };
        identity
            .validate()
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        Ok(identity)
    }
}

/// Existing source-level entry point delegates to the canonical engram planner.
pub(crate) fn build_memory_estate_plan(
    request: &PersistMemoryEstate,
    current: Option<DataSchemaRegistry>,
) -> Result<MemoryEstatePlan> {
    super::memory_engram::build_memory_engram_plan(request, current)
}
