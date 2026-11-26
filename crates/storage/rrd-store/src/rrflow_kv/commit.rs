//! Snapshot reads, database creation and commit preparation.

use super::*;

pub(super) struct RrflowKvSnapshotRead<'a> {
    pub(super) database: &'a Database,
    pub(super) snapshot: Snapshot,
}

impl crate::access::runtime_state::AccessRead for RrflowKvSnapshotRead<'_> {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.database.get(key, self.snapshot).map_err(Error::from)
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: &[u8],
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        Ok(self
            .database
            .scan(start, Some(end), self.snapshot)
            .map_err(Error::from)?
            .into_iter()
            .take(limit)
            .collect())
    }
}

/// Creates an empty database in the rrflowKV application format for an owner
/// that applies rrflowKV commits through the `&Database` functions below,
/// such as the Raft application bridge's canonical state.
pub fn create_rrflow_kv_database(
    root: &(impl crate::filesystem::AnchorInput + ?Sized),
) -> Result<Database> {
    Ok(Database::create_anchored(
        &root.anchor()?,
        DatabaseOptions::default(),
        Some(keyspaces::RRFLOW_KV_FORMAT),
    )?)
}

/// Whether `database` carries the rrflowKV application format that the
/// `&Database` functions below require.
pub fn is_rrflow_kv_database(database: &Database) -> bool {
    database_codec(database).is_ok()
}

/// Reads the exact rrflowKV cursor/schema pair used by the temporary Raft
/// application bridge before it submits the complete semantic plan.
pub fn rrflow_kv_commit_context(
    database: &Database,
    scope: &ScopeId,
) -> Result<(ReadStamp, Option<RuntimeSchemaRegistry>)> {
    let reader = RrflowKvSnapshotRead {
        database,
        snapshot: database.snapshot(),
    };
    let read = crate::access::runtime_state::read_stamp_with(&reader, scope)?;
    let schema = crate::access::runtime_state::get_json(
        &reader,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(scope),
    )?;
    if schema
        .as_ref()
        .map(|value: &RuntimeSchemaRegistry| value.revision)
        != read.schema_revision
    {
        return Err(Error::Substrate(
            "rrflowKV runtime schema differs from its read stamp".into(),
        ));
    }
    Ok((read, schema))
}

/// Validates and lowers one canonical [`RuntimeCommit`] into the same encoded
/// semantic plan used by rrflowMX and ordinary rrflowKV repositories. This
/// remains public only for the already-inventoried Raft application path.
pub fn prepare_rrflow_kv_commit(
    database: &Database,
    commit: &RuntimeCommit,
) -> Result<SemanticCommitPlan> {
    prepare_rrflow_kv_commit_at_read(database, commit, None, None, None)
}

pub(super) fn prepare_rrflow_kv_commit_at_read(
    database: &Database,
    commit: &RuntimeCommit,
    read: Option<&ReadStamp>,
    archived_audit: Option<&AuditEnvelope>,
    function_receipts: Option<(&str, &[crate::FunctionInvocationReceiptRecord])>,
) -> Result<SemanticCommitPlan> {
    let reader = RrflowKvSnapshotRead {
        database,
        snapshot: database.snapshot(),
    };
    prepare_semantic_commit(&reader, commit, read, archived_audit, function_receipts)
}

/// Reads a previously accepted rrflowKV runtime outcome from a caller-held
/// database snapshot. Coordinators use this before planning so content-addressed
/// retries remain idempotent even when the transport request id changes.
pub fn rrflow_kv_commit_outcome(
    database: &Database,
    commit_id: &str,
) -> Result<Option<RuntimeCommitOutcome>> {
    let outcome: Option<RuntimeCommitOutcome> = get_json(
        database,
        database.snapshot(),
        keyspaces::RUNTIME_COMMITS,
        &keyspaces::runtime_commit_key(commit_id),
    )?;
    if outcome
        .as_ref()
        .is_some_and(|outcome| outcome.commit_id != commit_id)
    {
        return Err(Error::Substrate(
            "runtime commit outcome key does not match its content identity".into(),
        ));
    }
    Ok(outcome)
}
