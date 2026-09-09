use super::common::{
    checked_key, get, get_json, put_sequence, read_sequence, scan_space, scan_space_from,
    RepositoryRead,
};
use crate::keyspaces::{self, Durability};
use crate::{Error, Result, StorageEngine, StorageTransaction};
use rrd_core::{
    projection_family, AuditEnvelope, DataTransaction, DataTransactionView, Millis, ProjectionWork,
    ReadStamp, RetentionPin, RuntimeChange, RuntimeChangePage, RuntimeCommit, RuntimeCommitOutcome,
    RuntimeDataSnapshot, RuntimeGraphSnapshot, RuntimeLogAccumulator, RuntimeMerkleNode,
    RuntimeMutation, RuntimeRecord, RuntimeRef, RuntimeRelation, RuntimeSchemaRegistry, ScopeId,
    SnapshotHandle, SnapshotId,
};
use serde::de::DeserializeOwned;
use std::collections::BTreeSet;

/// Canonical multi-model runtime repository shared by rrflowMX and rrflowKV.
/// It plans semantic effects against one captured physical snapshot and emits
/// every accepted effect through that transaction's single commit.
pub struct RuntimeRepository<'a> {
    storage: &'a dyn StorageEngine,
}

impl<'a> RuntimeRepository<'a> {
    pub(crate) const fn new(storage: &'a dyn StorageEngine) -> Self {
        Self { storage }
    }

    pub fn cursor(&self) -> Result<u64> {
        let transaction = self.storage.begin_transaction()?;
        read_sequence(&*transaction, &keyspaces::runtime_cursor_key())
    }

    pub fn schema(&self, scope: &ScopeId) -> Result<Option<RuntimeSchemaRegistry>> {
        let transaction = self.storage.begin_transaction()?;
        get_json(
            &*transaction,
            keyspaces::RUNTIME_SCHEMAS,
            &keyspaces::runtime_schema_key(scope),
        )
    }

    pub fn read_stamp(&self, scope: &ScopeId) -> Result<ReadStamp> {
        let transaction = self.storage.begin_transaction()?;
        read_stamp_with(&*transaction, scope)
    }

    pub fn open_snapshot(
        &self,
        scope: &ScopeId,
        owner: &str,
        now: Millis,
        ttl: Millis,
    ) -> Result<SnapshotHandle> {
        let mut transaction = self.storage.begin_transaction()?;
        let handle = SnapshotHandle::new(read_stamp_with(&*transaction, scope)?, owner, now, ttl)?;
        let key = keyspaces::runtime_snapshot_key(handle.id.as_str());
        if let Some(persisted) =
            get_json::<SnapshotHandle>(&*transaction, keyspaces::RUNTIME_SNAPSHOTS, &key)?
        {
            if persisted != handle {
                return Err(Error::SnapshotMismatch(handle.id.to_string()));
            }
            return Ok(handle);
        }
        transaction.put(
            checked_key(keyspaces::RUNTIME_SNAPSHOTS, &key)?,
            serde_json::to_vec(&handle)?,
        )?;
        transaction.commit(Durability::Authoritative)?;
        Ok(handle)
    }

    pub fn snapshot_changes(
        &self,
        handle: &SnapshotHandle,
        after: u64,
        limit: usize,
        now: Millis,
    ) -> Result<RuntimeChangePage> {
        handle.validate()?;
        let transaction = self.storage.begin_transaction()?;
        let persisted: SnapshotHandle = get_json(
            &*transaction,
            keyspaces::RUNTIME_SNAPSHOTS,
            &keyspaces::runtime_snapshot_key(handle.id.as_str()),
        )?
        .ok_or_else(|| Error::SnapshotNotFound(handle.id.to_string()))?;
        if &persisted != handle {
            return Err(Error::SnapshotMismatch(handle.id.to_string()));
        }
        if handle.is_expired(now) {
            return Err(Error::SnapshotExpired {
                id: handle.id.to_string(),
                expired_at: handle.expires_at,
            });
        }
        change_page(
            &*transaction,
            handle.read.commit_cursor,
            after,
            limit,
            Some(&handle.read.scope),
        )
    }

    pub fn release_snapshot(&self, id: &SnapshotId) -> Result<bool> {
        let mut transaction = self.storage.begin_transaction()?;
        let key = keyspaces::runtime_snapshot_key(id.as_str());
        if get(&*transaction, keyspaces::RUNTIME_SNAPSHOTS, &key)?.is_none() {
            return Ok(false);
        }
        transaction.delete(checked_key(keyspaces::RUNTIME_SNAPSHOTS, &key)?)?;
        transaction.commit(Durability::Authoritative)?;
        Ok(true)
    }

    pub fn snapshots(&self, now: Millis) -> Result<Vec<SnapshotHandle>> {
        let transaction = self.storage.begin_transaction()?;
        let mut handles = scan_space(&*transaction, keyspaces::RUNTIME_SNAPSHOTS, &[])?
            .into_iter()
            .map(|(_, bytes)| serde_json::from_slice::<SnapshotHandle>(&bytes).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        for handle in &handles {
            handle.validate()?;
        }
        handles.retain(|handle| !handle.is_expired(now));
        handles.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(handles)
    }

    pub fn retention_pins(&self, now: Millis) -> Result<Vec<RetentionPin>> {
        self.snapshots(now)?
            .iter()
            .map(RetentionPin::from_snapshot)
            .collect::<rrd_core::Result<Vec<_>>>()
            .map_err(Error::from)
    }

    pub fn read_changes(
        &self,
        read: &ReadStamp,
        after: u64,
        limit: usize,
    ) -> Result<RuntimeChangePage> {
        let transaction = self.storage.begin_transaction()?;
        let validation = validate_read_stamp(&*transaction, read)?;
        if limit == 1 && after < read.commit_cursor && read.accumulator_root.is_some() {
            let mut page = authenticated_point_page(&*transaction, read, after + 1)?;
            if validation.method == "full_hash_chain_replay" {
                page.validation.method = "full_hash_chain_replay_then_rfc9162_inclusion".into();
                page.validation.change_reads = page
                    .validation
                    .change_reads
                    .saturating_add(validation.change_reads);
            }
            return Ok(page);
        }
        let mut page = change_page(
            &*transaction,
            read.commit_cursor,
            after,
            limit,
            Some(&read.scope),
        )?;
        page.validation = validation;
        Ok(page)
    }

    pub fn data_snapshot(
        &self,
        scope: &ScopeId,
        valid_at: Millis,
        replay_limit: usize,
    ) -> Result<(ReadStamp, RuntimeDataSnapshot)> {
        if replay_limit == 0 {
            return Err(Error::Substrate(
                "runtime data snapshot replay limit must be greater than zero".into(),
            ));
        }
        let read = self.read_stamp(scope)?;
        let page = self.read_changes(&read, 0, replay_limit)?;
        if page.through_cursor != read.commit_cursor || page.has_more() {
            return Err(Error::Substrate(format!(
                "runtime data snapshot requires more than {replay_limit} retained changes"
            )));
        }
        let snapshot = if read.schema_revision.is_some() {
            let schema = schema_at_read(&read, &page)?;
            RuntimeDataSnapshot::from_changes(
                &page.changes,
                &schema,
                scope.clone(),
                valid_at,
                read.commit_cursor,
            )?
        } else {
            RuntimeDataSnapshot::from_claim_changes(
                &page.changes,
                scope.clone(),
                valid_at,
                read.commit_cursor,
            )?
        };
        Ok((read, snapshot))
    }

    pub fn commit(&self, commit: &RuntimeCommit) -> Result<RuntimeCommitOutcome> {
        self.commit_at_read(commit, None, None)
    }

    pub fn commit_data_transaction(
        &self,
        transaction: &DataTransaction,
    ) -> Result<RuntimeCommitOutcome> {
        transaction.validate()?;
        self.commit_at_read(&transaction.commit, Some(&transaction.read), None)
    }

    pub(crate) fn restore_commit(
        &self,
        commit: &RuntimeCommit,
        audit: &AuditEnvelope,
    ) -> Result<RuntimeCommitOutcome> {
        self.commit_at_read(commit, None, Some(audit))
    }

    fn commit_at_read(
        &self,
        commit: &RuntimeCommit,
        read: Option<&ReadStamp>,
        archived_audit: Option<&AuditEnvelope>,
    ) -> Result<RuntimeCommitOutcome> {
        commit.validate()?;
        self.validate_retirement_targets(commit)?;
        let mut transaction = self.storage.begin_transaction()?;
        let outcome = prepare_commit(&mut *transaction, commit, read, archived_audit)?;
        match transaction.commit(Durability::Authoritative) {
            Ok(_) => Ok(outcome),
            Err(Error::TransactionConflict { .. }) => Err(Error::RuntimeConflict {
                expected: commit.expected_cursor,
                actual: self.cursor()?,
            }),
            Err(error) => Err(error),
        }
    }

    pub fn changes_since(
        &self,
        after: u64,
        limit: usize,
        scope: Option<&ScopeId>,
    ) -> Result<RuntimeChangePage> {
        let transaction = self.storage.begin_transaction()?;
        let head = read_sequence(&*transaction, &keyspaces::runtime_cursor_key())?;
        change_page(&*transaction, head, after, limit, scope)
    }

    pub fn outbox_since(&self, after: u64, limit: usize) -> Result<Vec<ProjectionWork>> {
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime outbox page limit must be greater than zero".into(),
            ));
        }
        let transaction = self.storage.begin_transaction()?;
        let start =
            keyspaces::runtime_outbox_key(after.checked_add(1).ok_or(Error::SequenceOverflow)?);
        scan_space_from(&*transaction, keyspaces::RUNTIME_OUTBOX, &start)?
            .into_iter()
            .take(limit)
            .map(|(_, bytes)| {
                let work: ProjectionWork = serde_json::from_slice(&bytes)?;
                work.validate()?;
                Ok(work)
            })
            .collect()
    }

    pub fn audit(&self, commit_id: &str) -> Result<Option<AuditEnvelope>> {
        let transaction = self.storage.begin_transaction()?;
        let audit: Option<AuditEnvelope> = get_json(
            &*transaction,
            keyspaces::RUNTIME_AUDIT,
            &keyspaces::runtime_audit_key(commit_id),
        )?;
        if let Some(value) = &audit {
            value.validate()?;
        }
        Ok(audit)
    }

    pub fn commit_outcome(&self, commit_id: &str) -> Result<Option<RuntimeCommitOutcome>> {
        let transaction = self.storage.begin_transaction()?;
        get_json(
            &*transaction,
            keyspaces::RUNTIME_COMMITS,
            &keyspaces::runtime_commit_key(commit_id),
        )
    }

    pub fn preview_transaction(
        &self,
        transaction: &DataTransaction,
        valid_at: Millis,
    ) -> Result<DataTransactionView> {
        transaction.validate()?;
        let page = self.read_changes(&transaction.read, 0, usize::MAX)?;
        if page.through_cursor != transaction.read.commit_cursor {
            return Err(Error::Substrate(format!(
                "stamped runtime replay ended at {}, expected {}",
                page.through_cursor, transaction.read.commit_cursor
            )));
        }
        let base = RuntimeGraphSnapshot::from_changes(
            &page.changes,
            transaction.read.scope.clone(),
            valid_at,
            transaction.read.commit_cursor,
        );
        transaction.preview(&base).map_err(Error::from)
    }

    pub fn preview_data_snapshot(
        &self,
        transaction: &DataTransaction,
        valid_at: Millis,
        replay_limit: usize,
    ) -> Result<RuntimeDataSnapshot> {
        transaction.validate()?;
        if valid_at == 0 || replay_limit == 0 {
            return Err(Error::Substrate(
                "transaction data preview requires non-zero valid time and replay limit".into(),
            ));
        }
        let page = self.read_changes(&transaction.read, 0, replay_limit)?;
        if page.through_cursor != transaction.read.commit_cursor || page.has_more() {
            return Err(Error::Substrate(format!(
                "transaction data preview requires more than {replay_limit} retained changes"
            )));
        }
        let schema = transaction
            .commit
            .mutations
            .iter()
            .filter_map(|mutation| match mutation {
                RuntimeMutation::Schema { registry } => Some(registry.clone()),
                _ => None,
            })
            .next_back()
            .map(Ok)
            .unwrap_or_else(|| schema_at_read(&transaction.read, &page))?;
        schema.validate()?;
        let prospective_cursor = transaction
            .read
            .commit_cursor
            .checked_add(transaction.commit.mutations.len() as u64)
            .ok_or_else(|| Error::Substrate("transaction preview cursor overflowed".into()))?;
        let commit_id = transaction.commit.digest();
        let mut changes = page.changes;
        let mut previous_digest = changes.last().map(|change| change.digest.clone());
        for (ordinal, mutation) in transaction.commit.mutations.iter().cloned().enumerate() {
            let cursor = transaction.read.commit_cursor + ordinal as u64 + 1;
            let change = RuntimeChange::committed(
                cursor,
                &transaction.commit,
                &commit_id,
                ordinal as u64,
                mutation,
                previous_digest,
            );
            previous_digest = Some(change.digest.clone());
            changes.push(change);
        }
        RuntimeDataSnapshot::from_changes(
            &changes,
            &schema,
            transaction.read.scope.clone(),
            valid_at,
            prospective_cursor,
        )
        .map_err(Error::from)
    }

    fn validate_retirement_targets(&self, commit: &RuntimeCommit) -> Result<()> {
        let retirements = commit
            .mutations
            .iter()
            .filter_map(|mutation| match mutation {
                RuntimeMutation::Retire { retirement } => Some(retirement),
                _ => None,
            })
            .collect::<Vec<_>>();
        if retirements.is_empty() {
            return Ok(());
        }
        let read = self.read_stamp(&commit.scope)?;
        if read.commit_cursor != commit.expected_cursor {
            return Err(Error::RuntimeConflict {
                expected: commit.expected_cursor,
                actual: read.commit_cursor,
            });
        }
        let page = self.read_changes(&read, 0, usize::MAX)?;
        if page.through_cursor != read.commit_cursor || page.has_more() {
            return Err(Error::ReadStampUnavailable(format!(
                "retirement validation for {}",
                commit.scope
            )));
        }
        let schema = schema_at_read(&read, &page)?;
        let mut snapshots = std::collections::BTreeMap::new();
        for retirement in retirements {
            let snapshot = match snapshots.entry(retirement.effective_at) {
                std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::btree_map::Entry::Vacant(entry) => entry.insert(
                    RuntimeDataSnapshot::from_changes(
                        &page.changes,
                        &schema,
                        commit.scope.clone(),
                        retirement.effective_at,
                        read.commit_cursor,
                    )
                    .map_err(Error::from)?,
                ),
            };
            if !snapshot.contains(retirement.model, &retirement.reference) {
                return Err(Error::RuntimeTargetNotFound(format!(
                    "{:?} {}/{} at {} in scope {}",
                    retirement.model,
                    retirement.reference.kind,
                    retirement.reference.id,
                    retirement.effective_at,
                    commit.scope
                )));
            }
        }
        Ok(())
    }
}

fn schema_at_read(read: &ReadStamp, page: &RuntimeChangePage) -> Result<RuntimeSchemaRegistry> {
    let expected_revision = read
        .schema_revision
        .ok_or_else(|| Error::RuntimeSchemaMissing(read.scope.to_string()))?;
    let schema = page
        .changes
        .iter()
        .filter(|change| change.scope == read.scope && change.cursor <= read.commit_cursor)
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Schema { registry } => Some(registry),
            _ => None,
        })
        .next_back()
        .cloned()
        .ok_or_else(|| Error::RuntimeSchemaMissing(read.scope.to_string()))?;
    if schema.revision != expected_revision {
        return Err(Error::ReadStampUnavailable(format!(
            "schema revision {} for scope {} at cursor {}",
            expected_revision, read.scope, read.commit_cursor
        )));
    }
    Ok(schema)
}

fn prepare_commit(
    transaction: &mut dyn StorageTransaction,
    commit: &RuntimeCommit,
    read: Option<&ReadStamp>,
    archived_audit: Option<&AuditEnvelope>,
) -> Result<RuntimeCommitOutcome> {
    if read.is_some() && archived_audit.is_some() {
        return Err(Error::Archive(
            "archive replay cannot also supply a live read stamp".into(),
        ));
    }
    if let Some(audit) = archived_audit {
        audit.validate()?;
        if let Some(read) = &audit.read {
            read.validate()?;
        }
    } else if let Some(read) = read {
        validate_read_stamp(&*transaction, read)?;
    }
    let commit_id = commit.digest();
    let start = read_sequence(&*transaction, &keyspaces::runtime_cursor_key())?;
    if start != commit.expected_cursor {
        return Err(Error::RuntimeConflict {
            expected: commit.expected_cursor,
            actual: start,
        });
    }
    let (mut accumulator, bootstrap_nodes) = accumulator_with(&*transaction, start)?;
    let previous_schema: Option<RuntimeSchemaRegistry> = get_json(
        &*transaction,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(&commit.scope),
    )?;
    let proposed_schema = commit.mutations.iter().find_map(|mutation| match mutation {
        RuntimeMutation::Schema { registry } => Some(registry),
        _ => None,
    });
    let schema_free_claims = previous_schema.is_none()
        && proposed_schema.is_none()
        && commit
            .mutations
            .iter()
            .all(|mutation| matches!(mutation, RuntimeMutation::Claim { .. }));
    if !schema_free_claims {
        let effective_schema = match (previous_schema.as_ref(), proposed_schema) {
            (None, Some(registry)) if registry.revision == 1 => registry,
            (None, Some(registry)) => {
                return Err(Error::RuntimeSchemaConflict {
                    expected: 1,
                    actual: registry.revision,
                });
            }
            (Some(previous), Some(registry))
                if registry.revision == previous.revision.saturating_add(1) =>
            {
                registry
            }
            (Some(previous), Some(registry)) => {
                return Err(Error::RuntimeSchemaConflict {
                    expected: previous.revision.saturating_add(1),
                    actual: registry.revision,
                });
            }
            (Some(previous), None) => previous,
            (None, None) => {
                return Err(Error::RuntimeSchemaMissing(commit.scope.to_string()));
            }
        };
        let existing_records = if effective_schema
            .records
            .values()
            .any(|schema| !schema.unique_properties.is_empty())
        {
            values_for_scope::<RuntimeRecord>(
                &*transaction,
                keyspaces::RUNTIME_RECORDS,
                &commit.scope,
            )?
        } else {
            Vec::new()
        };
        let existing_relations = if effective_schema.relations.values().any(|schema| {
            schema.unique_pair || schema.max_outgoing.is_some() || schema.max_incoming.is_some()
        }) {
            values_for_scope::<RuntimeRelation>(
                &*transaction,
                keyspaces::RUNTIME_RELATIONS,
                &commit.scope,
            )?
        } else {
            Vec::new()
        };
        effective_schema.validate_objects(
            &commit.mutations,
            &existing_records,
            &existing_relations,
        )?;
    }

    let new_records = commit
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Record { record } => Some(record.reference.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for mutation in &commit.mutations {
        let references: Vec<&RuntimeRef> = match mutation {
            RuntimeMutation::Relation { relation } => vec![&relation.from, &relation.to],
            RuntimeMutation::Event { event } => event.subject.iter().collect(),
            RuntimeMutation::Vector { vector } => vec![&vector.subject],
            RuntimeMutation::SeriesSample { sample } => vec![&sample.series],
            RuntimeMutation::Geo { geo } => vec![&geo.subject],
            RuntimeMutation::Object { object } => object.subject.iter().collect(),
            RuntimeMutation::Claim { .. }
            | RuntimeMutation::Schema { .. }
            | RuntimeMutation::Record { .. }
            | RuntimeMutation::Retire { .. } => Vec::new(),
        };
        for reference in references {
            if !new_records.contains(reference)
                && get(
                    &*transaction,
                    keyspaces::RUNTIME_RECORDS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RECORDS,
                        &commit.scope,
                        reference,
                    ),
                )?
                .is_none()
            {
                return Err(Error::DanglingRuntimeReference(format!(
                    "{}/{} in scope {}",
                    reference.kind, reference.id, commit.scope
                )));
            }
        }
    }

    let claim_count = commit
        .mutations
        .iter()
        .filter(|mutation| matches!(mutation, RuntimeMutation::Claim { .. }))
        .count();
    let claim_start = read_sequence(&*transaction, &keyspaces::sequence_watermark_key())?;
    let mut claim_sequence = claim_start;
    let mut cursor = start;
    let mut previous_digest = get(
        &*transaction,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    let previous_audit_digest = get(
        &*transaction,
        keyspaces::META,
        &keyspaces::runtime_last_audit_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    for node in bootstrap_nodes {
        put_bytes(
            transaction,
            keyspaces::META,
            &keyspaces::runtime_accumulator_node_key(node.level, node.index),
            node.digest.into_bytes(),
        )?;
    }
    let mut outbox_count = 0;
    for (ordinal, mutation) in commit.mutations.iter().cloned().enumerate() {
        if let RuntimeMutation::Claim { claim } = &mutation {
            claim.validate()?;
            claim_sequence = claim_sequence
                .checked_add(1)
                .ok_or(Error::SequenceOverflow)?;
            let claim_key = keyspaces::claim_key(
                &claim.subject,
                &claim.predicate,
                claim.valid_from,
                claim.tx_time,
            );
            put_json(transaction, keyspaces::CLAIMS, &claim_key, claim)?;
            put_bytes(
                transaction,
                keyspaces::SEQUENCE_INDEX,
                &keyspaces::sequence_key(claim_sequence),
                claim_key,
            )?;
        }
        cursor = cursor.checked_add(1).ok_or(Error::SequenceOverflow)?;
        let change = RuntimeChange::committed(
            cursor,
            commit,
            &commit_id,
            ordinal as u64,
            mutation.clone(),
            previous_digest.clone(),
        );
        put_json(
            transaction,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(cursor),
            &change,
        )?;
        for node in accumulator.append_change(&change)? {
            put_bytes(
                transaction,
                keyspaces::META,
                &keyspaces::runtime_accumulator_node_key(node.level, node.index),
                node.digest.into_bytes(),
            )?;
        }
        if let Some(family) = projection_family(&mutation) {
            let work = ProjectionWork::for_change(
                commit.scope.clone(),
                cursor,
                commit_id.clone(),
                ordinal as u64,
                family,
            )?;
            put_json(
                transaction,
                keyspaces::RUNTIME_OUTBOX,
                &keyspaces::runtime_outbox_key(cursor),
                &work,
            )?;
            outbox_count += 1;
        }
        match mutation {
            RuntimeMutation::Schema { registry } => put_json(
                transaction,
                keyspaces::RUNTIME_SCHEMAS,
                &keyspaces::runtime_schema_key(&commit.scope),
                &registry,
            )?,
            RuntimeMutation::Record { record } => put_json(
                transaction,
                keyspaces::RUNTIME_RECORDS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_RECORDS,
                    &commit.scope,
                    &record.reference,
                ),
                &record,
            )?,
            RuntimeMutation::Relation { relation } => put_json(
                transaction,
                keyspaces::RUNTIME_RELATIONS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_RELATIONS,
                    &commit.scope,
                    &relation.reference,
                ),
                &relation,
            )?,
            RuntimeMutation::Vector { vector } => put_json(
                transaction,
                keyspaces::RUNTIME_VECTORS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_VECTORS,
                    &commit.scope,
                    &vector.reference,
                ),
                &vector,
            )?,
            RuntimeMutation::SeriesSample { sample } => put_json(
                transaction,
                keyspaces::RUNTIME_SERIES,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_SERIES,
                    &commit.scope,
                    &sample.reference,
                ),
                &sample,
            )?,
            RuntimeMutation::Geo { geo } => put_json(
                transaction,
                keyspaces::RUNTIME_GEO,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_GEO,
                    &commit.scope,
                    &geo.reference,
                ),
                &geo,
            )?,
            RuntimeMutation::Object { object } => put_json(
                transaction,
                keyspaces::RUNTIME_OBJECTS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_OBJECTS,
                    &commit.scope,
                    &object.reference,
                ),
                &object,
            )?,
            RuntimeMutation::Retire { retirement } => {
                let space = if retirement.model.is_record_like() {
                    Some(keyspaces::RUNTIME_RECORDS)
                } else {
                    match retirement.model {
                        rrd_core::RuntimeLogicalModel::GraphRelation => {
                            Some(keyspaces::RUNTIME_RELATIONS)
                        }
                        rrd_core::RuntimeLogicalModel::Vector => Some(keyspaces::RUNTIME_VECTORS),
                        rrd_core::RuntimeLogicalModel::TimeSeries => {
                            Some(keyspaces::RUNTIME_SERIES)
                        }
                        rrd_core::RuntimeLogicalModel::Geo => Some(keyspaces::RUNTIME_GEO),
                        rrd_core::RuntimeLogicalModel::Object => Some(keyspaces::RUNTIME_OBJECTS),
                        rrd_core::RuntimeLogicalModel::ReasoningClaim
                        | rrd_core::RuntimeLogicalModel::Document
                        | rrd_core::RuntimeLogicalModel::Relational
                        | rrd_core::RuntimeLogicalModel::GraphNode
                        | rrd_core::RuntimeLogicalModel::KeyValue
                        | rrd_core::RuntimeLogicalModel::Event
                        | rrd_core::RuntimeLogicalModel::ReasoningRecord
                        | rrd_core::RuntimeLogicalModel::ReasoningEvent
                        | rrd_core::RuntimeLogicalModel::LifecycleRecord
                        | rrd_core::RuntimeLogicalModel::LifecycleEvent => None,
                    }
                };
                if let Some(space) = space {
                    let key = keyspaces::runtime_identity_key(
                        space,
                        &commit.scope,
                        &retirement.reference,
                    );
                    transaction.delete(checked_key(space, &key)?)?;
                }
            }
            RuntimeMutation::Claim { .. } | RuntimeMutation::Event { .. } => {}
        }
        previous_digest = Some(change.digest);
    }
    if claim_count > 0 {
        put_sequence(
            transaction,
            &keyspaces::sequence_watermark_key(),
            claim_sequence,
        )?;
    }
    put_sequence(transaction, &keyspaces::runtime_cursor_key(), cursor)?;
    put_bytes(
        transaction,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
        previous_digest.as_deref().unwrap_or("").as_bytes().to_vec(),
    )?;
    put_json(
        transaction,
        keyspaces::META,
        &keyspaces::runtime_accumulator_state_key(),
        &accumulator,
    )?;
    if let Some(read) = read {
        put_sequence(
            transaction,
            &keyspaces::catalog_revision_key(&read.scope),
            read.catalog_revision,
        )?;
    }
    let audit_read = archived_audit
        .and_then(|audit| audit.read.as_ref())
        .or(read);
    let audit = AuditEnvelope::accepted_commit_at_read(
        commit,
        audit_read,
        &commit_id,
        cursor,
        previous_audit_digest,
    )?;
    if archived_audit.is_some_and(|expected| expected != &audit) {
        return Err(Error::Archive(format!(
            "runtime commit {commit_id} audit envelope differs from its archive"
        )));
    }
    put_json(
        transaction,
        keyspaces::RUNTIME_AUDIT,
        &keyspaces::runtime_audit_key(&commit_id),
        &audit,
    )?;
    put_bytes(
        transaction,
        keyspaces::META,
        &keyspaces::runtime_last_audit_digest_key(),
        audit.digest.as_bytes().to_vec(),
    )?;
    let outcome = RuntimeCommitOutcome {
        commit_id,
        first_cursor: start + 1,
        last_cursor: cursor,
        count: commit.mutations.len(),
        first_claim_sequence: (claim_count > 0).then_some(claim_start + 1),
        last_claim_sequence: (claim_count > 0).then_some(claim_sequence),
        outbox_count,
    };
    put_json(
        transaction,
        keyspaces::RUNTIME_COMMITS,
        &keyspaces::runtime_commit_key(&outcome.commit_id),
        &outcome,
    )?;
    Ok(outcome)
}

fn read_stamp_with(
    transaction: &(impl RepositoryRead + ?Sized),
    scope: &ScopeId,
) -> Result<ReadStamp> {
    let commit_cursor = read_sequence(transaction, &keyspaces::runtime_cursor_key())?;
    let schema_revision = get_json::<RuntimeSchemaRegistry>(
        transaction,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(scope),
    )?
    .map(|schema| schema.revision);
    let catalog_revision = read_sequence(transaction, &keyspaces::catalog_revision_key(scope))?;
    let head_digest = get(
        transaction,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    match load_accumulator(transaction, commit_cursor)? {
        Some(accumulator) => ReadStamp::authenticated(
            scope.clone(),
            schema_revision,
            catalog_revision,
            commit_cursor,
            head_digest,
            accumulator.root,
        ),
        None => ReadStamp::new(
            scope.clone(),
            schema_revision,
            catalog_revision,
            commit_cursor,
            head_digest,
        ),
    }
    .map_err(Error::from)
}

fn validate_read_stamp(
    transaction: &(impl RepositoryRead + ?Sized),
    read: &ReadStamp,
) -> Result<rrd_core::RuntimeReadValidation> {
    read.validate()?;
    let current = read_sequence(transaction, &keyspaces::runtime_cursor_key())?;
    if read.commit_cursor > current {
        return Err(Error::ReadStampUnavailable(read.manifest_id.clone()));
    }
    if read.commit_cursor == current && read.accumulator_root.is_some() {
        let accumulator = load_accumulator(transaction, current)?
            .ok_or_else(|| Error::ReadStampMismatch(read.manifest_id.clone()))?;
        let head_digest = get(
            transaction,
            keyspaces::META,
            &keyspaces::runtime_last_digest_key(),
        )?
        .map(String::from_utf8)
        .transpose()
        .map_err(|error| Error::CorruptWatermark(error.to_string()))?
        .filter(|digest| !digest.is_empty());
        let schema_revision = get_json::<RuntimeSchemaRegistry>(
            transaction,
            keyspaces::RUNTIME_SCHEMAS,
            &keyspaces::runtime_schema_key(&read.scope),
        )?
        .map(|schema| schema.revision);
        let catalog_revision =
            read_sequence(transaction, &keyspaces::catalog_revision_key(&read.scope))?;
        if read.catalog_revision != catalog_revision
            || read.head_digest != head_digest
            || read.schema_revision != schema_revision
            || read.accumulator_root.as_deref() != Some(accumulator.root.as_str())
        {
            return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
        }
        return Ok(rrd_core::RuntimeReadValidation::new(
            "authenticated_current_head",
            0,
            0,
        ));
    }
    let retained_head = if read.commit_cursor == 0 {
        None
    } else {
        let change: RuntimeChange = get_json(
            transaction,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(read.commit_cursor),
        )?
        .ok_or_else(|| Error::ReadStampUnavailable(read.manifest_id.clone()))?;
        if !change.verify_digest() {
            return Err(Error::Substrate(format!(
                "runtime change {} failed digest verification",
                read.commit_cursor
            )));
        }
        Some(change.digest)
    };
    let page = change_page(
        transaction,
        read.commit_cursor,
        0,
        usize::MAX,
        Some(&read.scope),
    )?;
    let schema_revision = page
        .changes
        .iter()
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Schema { registry } => Some(registry.revision),
            _ => None,
        })
        .next_back();
    let catalog_revision =
        read_sequence(transaction, &keyspaces::catalog_revision_key(&read.scope))?;
    if let Some(root) = read.accumulator_root.as_deref() {
        RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
            read_accumulator_node(transaction, level, index)
        })?;
    }
    if read.catalog_revision != catalog_revision
        || read.head_digest != retained_head
        || read.schema_revision != schema_revision
    {
        return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
    }
    Ok(rrd_core::RuntimeReadValidation::new(
        "full_hash_chain_replay",
        read.commit_cursor,
        0,
    ))
}

fn load_accumulator(
    transaction: &(impl RepositoryRead + ?Sized),
    expected_size: u64,
) -> Result<Option<RuntimeLogAccumulator>> {
    let stored: Option<RuntimeLogAccumulator> = get_json(
        transaction,
        keyspaces::META,
        &keyspaces::runtime_accumulator_state_key(),
    )?;
    match stored {
        Some(accumulator) => {
            accumulator.validate()?;
            if accumulator.tree_size != expected_size {
                return Err(Error::Substrate(format!(
                    "runtime accumulator size {} differs from cursor {expected_size}",
                    accumulator.tree_size
                )));
            }
            Ok(Some(accumulator))
        }
        None if expected_size == 0 => Ok(Some(RuntimeLogAccumulator::new())),
        None => Ok(None),
    }
}

fn accumulator_with(
    transaction: &(impl RepositoryRead + ?Sized),
    expected_size: u64,
) -> Result<(RuntimeLogAccumulator, Vec<RuntimeMerkleNode>)> {
    if let Some(accumulator) = load_accumulator(transaction, expected_size)? {
        return Ok((accumulator, Vec::new()));
    }
    let page = change_page(transaction, expected_size, 0, usize::MAX, None)?;
    let mut accumulator = RuntimeLogAccumulator::new();
    let mut nodes = Vec::new();
    for change in &page.changes {
        nodes.extend(accumulator.append_change(change)?);
    }
    if accumulator.tree_size != expected_size {
        return Err(Error::Substrate(
            "runtime accumulator bootstrap did not cover the full log".into(),
        ));
    }
    Ok((accumulator, nodes))
}

fn authenticated_point_page(
    transaction: &(impl RepositoryRead + ?Sized),
    read: &ReadStamp,
    cursor: u64,
) -> Result<RuntimeChangePage> {
    let root = read
        .accumulator_root
        .as_deref()
        .ok_or_else(|| Error::ReadStampMismatch(read.manifest_id.clone()))?;
    let accumulator = match load_accumulator(transaction, read.commit_cursor) {
        Ok(Some(accumulator)) if accumulator.root == root => accumulator,
        Ok(_) | Err(_) => {
            RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
                read_accumulator_node(transaction, level, index)
            })?
        }
    };
    let change: RuntimeChange = get_json(
        transaction,
        keyspaces::RUNTIME_CHANGES,
        &keyspaces::runtime_change_key(cursor),
    )?
    .ok_or_else(|| Error::ReadStampUnavailable(read.manifest_id.clone()))?;
    let proof = accumulator.inclusion_proof(cursor - 1, |level, index| {
        read_accumulator_node(transaction, level, index)
    })?;
    let proof_nodes = proof.path.len();
    proof.verify_change(&change, root)?;
    let selected = (change.scope == read.scope).then_some(change);
    Ok(RuntimeChangePage {
        requested_after: cursor - 1,
        through_cursor: cursor,
        head_cursor: read.commit_cursor,
        validation: rrd_core::RuntimeReadValidation::new("rfc9162_inclusion_proof", 1, proof_nodes),
        changes: selected.into_iter().collect(),
    })
}

fn read_accumulator_node(
    transaction: &(impl RepositoryRead + ?Sized),
    level: u8,
    index: u64,
) -> rrd_core::Result<Option<String>> {
    get(
        transaction,
        keyspaces::META,
        &keyspaces::runtime_accumulator_node_key(level, index),
    )
    .map_err(|error| rrd_core::Error::InvalidRuntime {
        reason: format!("cannot read runtime accumulator node: {error}"),
    })?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| rrd_core::Error::InvalidRuntime {
        reason: format!("runtime accumulator node is not UTF-8: {error}"),
    })
}

fn change_page(
    transaction: &(impl RepositoryRead + ?Sized),
    head: u64,
    after: u64,
    limit: usize,
    scope: Option<&ScopeId>,
) -> Result<RuntimeChangePage> {
    if limit == 0 {
        return Err(Error::Substrate(
            "runtime change page limit must be greater than zero".into(),
        ));
    }
    if after == u64::MAX || after >= head {
        return Ok(RuntimeChangePage {
            requested_after: after,
            through_cursor: after,
            head_cursor: head,
            validation: rrd_core::RuntimeReadValidation::new("bounded_hash_chain_page", 0, 0),
            changes: Vec::new(),
        });
    }
    let mut previous_digest = if after == 0 {
        None
    } else {
        let prior: RuntimeChange = get_json(
            transaction,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(after),
        )?
        .ok_or_else(|| Error::Substrate(format!("runtime log is missing cursor {after}")))?;
        if !prior.verify_digest() {
            return Err(Error::Substrate(format!(
                "runtime change {after} failed digest verification"
            )));
        }
        Some(prior.digest)
    };
    let mut through = after;
    let mut selected = Vec::new();
    for expected in after + 1..=head {
        if through.saturating_sub(after) as usize >= limit {
            break;
        }
        let change: RuntimeChange = get_json(
            transaction,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(expected),
        )?
        .ok_or_else(|| Error::Substrate(format!("runtime log is missing cursor {expected}")))?;
        if change.cursor != expected
            || change.previous_digest != previous_digest
            || !change.verify_digest()
        {
            return Err(Error::Substrate(format!(
                "runtime change {expected} failed cursor/hash-chain verification"
            )));
        }
        through = expected;
        previous_digest = Some(change.digest.clone());
        if scope.is_none_or(|scope| scope == &change.scope) {
            selected.push(change);
        }
    }
    Ok(RuntimeChangePage {
        requested_after: after,
        through_cursor: through,
        head_cursor: head,
        validation: rrd_core::RuntimeReadValidation::new(
            "bounded_hash_chain_page",
            through.saturating_sub(after),
            0,
        ),
        changes: selected,
    })
}

fn values_for_scope<T: DeserializeOwned>(
    transaction: &(impl RepositoryRead + ?Sized),
    space: keyspaces::Space,
    scope: &ScopeId,
) -> Result<Vec<T>> {
    let prefix = keyspaces::runtime_scope_prefix(space, scope);
    scan_space(transaction, space, &prefix)?
        .into_iter()
        .map(|(_, value)| serde_json::from_slice(&value).map_err(Error::from))
        .collect()
}

fn put_bytes(
    transaction: &mut dyn StorageTransaction,
    space: keyspaces::Space,
    key: &[u8],
    value: Vec<u8>,
) -> Result<()> {
    transaction.put(checked_key(space, key)?, value)
}

fn put_json<T: serde::Serialize>(
    transaction: &mut dyn StorageTransaction,
    space: keyspaces::Space,
    key: &[u8],
    value: &T,
) -> Result<()> {
    put_bytes(transaction, space, key, serde_json::to_vec(value)?)
}
