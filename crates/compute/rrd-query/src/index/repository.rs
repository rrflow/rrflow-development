//! The index catalogue repository: ensure, build, publish and read.

use super::*;

impl<'a, E: StorageEngine> IndexCatalogueRepository<'a, E> {
    pub fn new(engine: &'a E, scope: ScopeId) -> Self {
        let key = format!("server/state/index-catalogue/{scope}");
        Self { engine, scope, key }
    }

    pub fn load(&self) -> Result<IndexCatalogue> {
        let Some(bytes) = self.engine.control().get(&self.key)? else {
            return Ok(IndexCatalogue::empty(self.scope.clone()));
        };
        let catalogue: IndexCatalogue = serde_json::from_slice(&bytes)?;
        if catalogue.scope != self.scope {
            return Err(Error::Integrity(
                "index catalogue scope differs from its control key".into(),
            ));
        }
        catalogue.validate()?;
        Ok(catalogue)
    }

    pub fn operation_receipt(
        &self,
        idempotency_key: &str,
        operation_digest: &str,
    ) -> Result<Option<IndexOperationReceipt>> {
        let catalogue = self.load()?;
        let Some(receipt) = catalogue.operations.get(idempotency_key) else {
            return Ok(None);
        };
        if receipt.operation_digest != operation_digest {
            return Err(Error::Catalog(
                "index idempotency key is bound to a different operation".into(),
            ));
        }
        Ok(Some(receipt.clone()))
    }

    pub fn record_operation(
        &self,
        context: &IndexMutationContext,
        idempotency_key: String,
        operation_digest: String,
        entry: IndexEntry,
    ) -> Result<IndexCatalogue> {
        context.validate()?;
        if idempotency_key.is_empty()
            || operation_digest.len() != 64
            || !operation_digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(Error::Catalog(
                "index operation requires a canonical idempotency key and SHA-256".into(),
            ));
        }
        validate_index_entry(&entry.definition.id, &entry)?;
        self.update(context, "index.operation_recorded", move |catalogue| {
            if let Some(existing) = catalogue.operations.get(&idempotency_key) {
                if existing.operation_digest == operation_digest && existing.entry == entry {
                    return Ok(());
                }
                return Err(Error::Catalog(
                    "index idempotency key is bound to a different operation".into(),
                ));
            }
            if catalogue.operations.len() >= MAX_INDEX_OPERATION_RECEIPTS {
                return Err(Error::Budget(
                    "index operation receipt limit exceeded".into(),
                ));
            }
            catalogue.operations.insert(
                idempotency_key,
                IndexOperationReceipt {
                    operation_digest,
                    entry,
                },
            );
            Ok(())
        })
    }

    pub fn create(
        &self,
        context: &IndexMutationContext,
        query_catalogue: &Catalog,
        definition: IndexDefinition,
    ) -> Result<IndexCatalogue> {
        context.validate()?;
        definition.validate()?;
        if query_catalogue.read.scope != self.scope {
            return Err(Error::Catalog(
                "query and index catalogues belong to different scopes".into(),
            ));
        }
        validate_fields(query_catalogue, &definition)?;
        self.update(context, "index.created", move |catalogue| {
            if catalogue.entries.contains_key(&definition.id) {
                return Err(Error::Catalog(format!(
                    "index {} already exists",
                    definition.id
                )));
            }
            let config_digest = definition.config_digest()?;
            let id = definition.id.clone();
            let unique_validated = definition.unique;
            catalogue.entries.insert(
                id.clone(),
                IndexEntry {
                    definition,
                    stamp: ProjectionStamp {
                        contract_version: DATA_RUNTIME_CONTRACT_VERSION,
                        id,
                        generation: 1,
                        source_cursor: 0,
                        config_digest,
                        artifact_digest: digest::sha256_hex(&[]),
                        state: ProjectionState::Building,
                    },
                    built_valid_at: None,
                    artifact_rows: None,
                    result_identity_version: None,
                    prior_artifact: None,
                    maintenance: None,
                    analytics_total_count: None,
                    analytics_group_count: None,
                    unique_validated,
                },
            );
            Ok(())
        })
    }

    pub fn begin_rebuild(
        &self,
        context: &IndexMutationContext,
        id: &ProjectionId,
    ) -> Result<IndexCatalogue> {
        context.validate()?;
        let id = id.clone();
        self.update(context, "index.rebuild_started", move |catalogue| {
            let entry = catalogue
                .entries
                .get_mut(&id)
                .ok_or_else(|| Error::Catalog(format!("unknown index {id}")))?;
            entry.stamp.generation = entry
                .stamp
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::Integrity("index generation overflow".into()))?;
            entry.prior_artifact = (entry.stamp.state == ProjectionState::Ready
                && entry.maintenance.is_some()
                && entry.result_identity_version == Some(RESULT_IDENTITY_VERSION))
            .then(|| IndexArtifactReference {
                generation: entry.stamp.generation - 1,
                source_cursor: entry.stamp.source_cursor,
                valid_at: entry
                    .built_valid_at
                    .expect("ready index has valid-time coverage"),
                artifact_digest: entry.stamp.artifact_digest.clone(),
            });
            entry.stamp.source_cursor = 0;
            entry.stamp.artifact_digest = digest::sha256_hex(&[]);
            entry.stamp.state = ProjectionState::Building;
            entry.built_valid_at = None;
            entry.artifact_rows = None;
            entry.result_identity_version = None;
            entry.maintenance = None;
            entry.analytics_total_count = None;
            entry.analytics_group_count = None;
            Ok(())
        })
    }

    pub fn build(
        &self,
        context: &IndexMutationContext,
        id: &ProjectionId,
        valid_at: u64,
        budget: &ExecutionBudget,
    ) -> Result<IndexCatalogue> {
        context.validate()?;
        let catalogue = self.load()?;
        let entry = catalogue
            .entries
            .get(id)
            .ok_or_else(|| Error::Catalog(format!("unknown index {id}")))?;
        if entry.stamp.state != ProjectionState::Building {
            return Err(Error::Catalog(format!("index {id} is not building")));
        }
        let generation = entry.stamp.generation;
        let definition = entry.definition.clone();
        let mut query = Query::new(
            definition.source.clone(),
            TemporalSelector {
                valid_at: TimeExpr::Literal(valid_at),
                known_at: CursorExpr::Head,
            },
        );
        query.filters = definition.filters.clone();
        query.projection = Projection::All;
        let query_catalogue = Catalog::capture_for_query(
            self.engine,
            &self.scope,
            &query,
            RuntimeReadBudget::new(budget.max_storage_keys)?,
        )?;
        let bound = bind(&query, &Parameters::new(), &query_catalogue)?;
        let physical = plan(&bound)?;
        let execution = execute(self.engine, &physical, budget)?;
        if execution.truncated {
            return Err(Error::Budget(
                "index build execution was truncated by its budget".into(),
            ));
        }
        let rows = execution
            .batches
            .into_iter()
            .flat_map(|batch| batch.rows)
            .collect::<Vec<_>>();
        validate_unique_rows(&definition, &rows)?;
        let bm25 = match &definition.kind {
            IndexKind::Scalar
            | IndexKind::Count
            | IndexKind::Geo
            | IndexKind::MaterializedView
            | IndexKind::AggregateCount => None,
            IndexKind::Bm25 { config } => {
                let field = &definition.fields[0];
                let documents = rows
                    .iter()
                    .filter_map(|row| match row.values.get(field) {
                        Some(rrd_core::RuntimeValue::String(text)) => {
                            Some(Ok((row.identity.clone(), text.clone())))
                        }
                        Some(rrd_core::RuntimeValue::Null) | None => None,
                        Some(_) => Some(Err(Error::Catalog(format!(
                            "BM25 field {field:?} produced a non-string value"
                        )))),
                    })
                    .collect::<Result<Vec<_>>>()?;
                Some(Bm25Artifact::build(
                    config.clone(),
                    bound.source_cursor,
                    bound.schema_revision,
                    valid_at,
                    documents,
                )?)
            }
        };
        let analytics = match definition.kind {
            IndexKind::Count | IndexKind::AggregateCount => {
                Some(build_analytics(&definition, &rows)?)
            }
            _ => None,
        };
        let prior = entry.prior_artifact.clone();
        let maintenance = maintenance_evidence(
            self.engine,
            &self.scope,
            id,
            prior.as_ref(),
            bound.source_cursor,
            &rows,
        )?;
        let artifact = IndexArtifact {
            contract_version: INDEX_ARTIFACT_CONTRACT_VERSION,
            result_identity_version: RESULT_IDENTITY_VERSION,
            scope: self.scope.clone(),
            definition,
            generation,
            source_cursor: bound.source_cursor,
            read_cursor: query_catalogue.read.commit_cursor,
            schema_revision: bound.schema_revision,
            valid_at,
            rows,
            bm25,
            analytics,
            maintenance,
        };
        let bytes = artifact.encode()?;
        let artifact_digest = digest::sha256_hex(&bytes);
        let name = index_artifact_name(&self.scope, id, generation, &artifact_digest);
        if let Some(existing) = self.engine.projections().get(&name)? {
            if existing != bytes {
                return Err(Error::Integrity(
                    "content-addressed index artifact name contains different bytes".into(),
                ));
            }
        } else {
            self.engine
                .projections()
                .put_with(&name, &bytes, Durability::Authoritative)?;
        }
        let stored = self
            .engine
            .projections()
            .get(&name)?
            .ok_or_else(|| Error::Integrity("published index artifact is unreadable".into()))?;
        if digest::sha256_hex(&stored) != artifact_digest {
            return Err(Error::Integrity(
                "published index artifact digest does not verify".into(),
            ));
        }
        let stored_artifact = IndexArtifact::decode(&stored)?;
        if stored_artifact != artifact {
            return Err(Error::Integrity(
                "published index artifact does not decode to the built artifact".into(),
            ));
        }
        self.publish_ready(
            context,
            id,
            &IndexArtifactPublication {
                generation,
                source_cursor: artifact.source_cursor,
                valid_at,
                artifact_rows: u64::try_from(artifact.rows.len())
                    .map_err(|_| Error::Budget("index artifact row count exceeds u64".into()))?,
                artifact_digest,
                maintenance: artifact.maintenance.clone(),
                analytics_total_count: artifact.analytics.as_ref().map(|value| value.total_count),
                analytics_group_count: artifact
                    .analytics
                    .as_ref()
                    .map(|value| value.groups.len())
                    .map(u64::try_from)
                    .transpose()
                    .map_err(|_| Error::Budget("analytics group count exceeds u64".into()))?,
            },
        )
    }

    pub fn publish_ready(
        &self,
        context: &IndexMutationContext,
        id: &ProjectionId,
        publication: &IndexArtifactPublication,
    ) -> Result<IndexCatalogue> {
        context.validate()?;
        let scope_head = self.engine.runtime().read_stamp(&self.scope)?.commit_cursor;
        if publication.source_cursor == 0 || publication.source_cursor > scope_head {
            return Err(Error::Catalog(
                "index source cursor must name an existing authoritative change".into(),
            ));
        }
        let current = self.load()?;
        let entry = current
            .entries
            .get(id)
            .ok_or_else(|| Error::Catalog(format!("unknown index {id}")))?;
        if entry.stamp.state != ProjectionState::Building
            || entry.stamp.generation != publication.generation
        {
            return Err(Error::Catalog(
                "index publication is stale or not building".into(),
            ));
        }
        let name = index_artifact_name(
            &self.scope,
            id,
            publication.generation,
            &publication.artifact_digest,
        );
        let bytes = self
            .engine
            .projections()
            .get(&name)?
            .ok_or_else(|| Error::Integrity("published index artifact is missing".into()))?;
        if digest::sha256_hex(&bytes) != publication.artifact_digest {
            return Err(Error::Integrity(
                "published index artifact digest does not verify".into(),
            ));
        }
        let artifact = IndexArtifact::decode(&bytes)?;
        let analytics_group_count = artifact
            .analytics
            .as_ref()
            .map(|value| u64::try_from(value.groups.len()))
            .transpose()
            .map_err(|_| Error::Budget("analytics group count exceeds u64".into()))?;
        if artifact.scope != self.scope
            || artifact.definition != entry.definition
            || artifact.generation != publication.generation
            || artifact.source_cursor != publication.source_cursor
            || artifact.valid_at != publication.valid_at
            || u64::try_from(artifact.rows.len()).ok() != Some(publication.artifact_rows)
            || artifact.maintenance != publication.maintenance
            || artifact.analytics.as_ref().map(|value| value.total_count)
                != publication.analytics_total_count
            || analytics_group_count != publication.analytics_group_count
        {
            return Err(Error::Integrity(
                "index publication does not match its artifact".into(),
            ));
        }
        let id = id.clone();
        let publication = publication.clone();
        self.update(context, "index.ready", move |catalogue| {
            let entry = catalogue
                .entries
                .get_mut(&id)
                .ok_or_else(|| Error::Catalog(format!("unknown index {id}")))?;
            if entry.stamp.generation != publication.generation
                || entry.stamp.state != ProjectionState::Building
                || entry.definition != artifact.definition
            {
                return Err(Error::Catalog(
                    "index publication is stale or not building".into(),
                ));
            }
            entry.stamp.source_cursor = publication.source_cursor;
            entry.stamp.artifact_digest = publication.artifact_digest;
            entry.stamp.state = ProjectionState::Ready;
            entry.built_valid_at = Some(publication.valid_at);
            entry.artifact_rows = Some(publication.artifact_rows);
            entry.result_identity_version = Some(RESULT_IDENTITY_VERSION);
            entry.prior_artifact = None;
            entry.maintenance = Some(publication.maintenance);
            entry.analytics_total_count = publication.analytics_total_count;
            entry.analytics_group_count = publication.analytics_group_count;
            if entry.definition.unique {
                entry.unique_validated = true;
            }
            entry
                .stamp
                .validate()
                .map_err(|error| Error::Catalog(format!("invalid ready index stamp: {error}")))?;
            Ok(())
        })
    }

    pub fn quarantine(
        &self,
        context: &IndexMutationContext,
        id: &ProjectionId,
    ) -> Result<IndexCatalogue> {
        self.set_state(
            context,
            id,
            ProjectionState::Quarantined,
            "index.quarantined",
        )
    }

    pub fn retire(
        &self,
        context: &IndexMutationContext,
        id: &ProjectionId,
    ) -> Result<IndexCatalogue> {
        self.set_state(context, id, ProjectionState::Retiring, "index.retiring")
    }

    fn set_state(
        &self,
        context: &IndexMutationContext,
        id: &ProjectionId,
        state: ProjectionState,
        action: &str,
    ) -> Result<IndexCatalogue> {
        context.validate()?;
        let id = id.clone();
        self.update(context, action, move |catalogue| {
            let entry = catalogue
                .entries
                .get_mut(&id)
                .ok_or_else(|| Error::Catalog(format!("unknown index {id}")))?;
            entry.stamp.state = state;
            Ok(())
        })
    }

    fn update(
        &self,
        context: &IndexMutationContext,
        action: &str,
        mutate: impl FnOnce(&mut IndexCatalogue) -> Result<()>,
    ) -> Result<IndexCatalogue> {
        let expected = self.engine.control().get(&self.key)?;
        let mut catalogue = match &expected {
            Some(bytes) => serde_json::from_slice(bytes)?,
            None => IndexCatalogue::empty(self.scope.clone()),
        };
        catalogue.validate()?;
        mutate(&mut catalogue)?;
        catalogue.revision = catalogue
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::Integrity("index catalogue revision overflow".into()))?;
        catalogue.validate()?;
        let schema_revision = self
            .engine
            .runtime()
            .read_stamp(&self.scope)?
            .schema_revision
            .ok_or_else(|| Error::Catalog("index catalogue requires an installed schema".into()))?;
        let bindings = commit_binding_definitions(&catalogue)?;
        let replacement = serde_json::to_vec(&catalogue)?;
        self.engine.control().commit_catalog_with_index_bindings(
            &self.scope,
            schema_revision,
            &ControlTransition {
                key: self.key.clone(),
                expected,
                replacement: Some(replacement),
                at: context.at,
                actor: context.actor.clone(),
                action: action.into(),
                request_id: context.request_id.clone(),
                operation_id: context.operation_id.clone(),
            },
            &bindings,
        )?;
        Ok(catalogue)
    }
}
