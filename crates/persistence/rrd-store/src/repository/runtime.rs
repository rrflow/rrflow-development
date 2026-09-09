use crate::access::prepare_semantic_commit;
use crate::access::runtime_state::{
    authenticated_point_page, change_page, checked_key, get, get_json, read_sequence,
    read_stamp_with, scan_space, scan_space_from, validate_read_stamp,
};
use crate::keyspaces::{self, Durability};
use crate::{Error, Result, StorageEngine};
use rrd_core::{
    AuditEnvelope, DataTransaction, DataTransactionView, Millis, ProjectionWork, ReadStamp,
    RetentionPin, RuntimeChange, RuntimeChangePage, RuntimeCommit, RuntimeCommitOutcome,
    RuntimeDataSnapshot, RuntimeGraphSnapshot, RuntimeMutation, RuntimeSchemaRegistry, ScopeId,
    SnapshotHandle, SnapshotId,
};

/// Canonical multi-model runtime repository shared by rrflowMX and rrflowKV.
/// It plans semantic effects against one captured physical snapshot and emits
/// the complete prepared plan through that transaction's single commit.
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
        let mut transaction = self.storage.begin_transaction()?;
        let plan = prepare_semantic_commit(&*transaction, commit, read, archived_audit)?;
        let outcome = plan.apply(&mut *transaction)?;
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

    /// Durable projection-work identities committed with their canonical
    /// mutations. These are distinct from outbox delivery records even though
    /// both initially carry the same validated work identity; C-03b adds the
    /// family-specific old/new index deltas behind these identities.
    pub fn projection_deltas_since(&self, after: u64, limit: usize) -> Result<Vec<ProjectionWork>> {
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime projection-delta page limit must be greater than zero".into(),
            ));
        }
        let transaction = self.storage.begin_transaction()?;
        let start = keyspaces::runtime_projection_delta_start(
            after.checked_add(1).ok_or(Error::SequenceOverflow)?,
        );
        scan_space_from(&*transaction, keyspaces::RUNTIME_PROJECTION_DELTAS, &start)?
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
