mod commits;
mod feeds;
mod indexes;
mod snapshots;

use crate::access::runtime_state::{
    authenticated_point_page, change_page, change_page_bounded_bytes, checked_key, get, get_json,
    read_sequence, read_stamp_with, scan_space, scan_space_from,
    validate_read_stamp as validate_runtime_read_stamp,
};

use crate::access::{
    index_source_deltas, prepare_semantic_commit, read_vector_deltas as read_vector_deltas_access,
    read_versioned as read_versioned_access,
    read_versioned_expanding as read_versioned_expanding_access, schema_at_read,
};

use crate::keyspaces::{self, Durability};

use crate::{
    ControlJournalEntry, ControlTransition, Error, FunctionInvocationReceiptRecord,
    IndexSourceDelta, Result, RuntimeReadBudget, RuntimeReadEvidence, RuntimeVectorDeltaRead,
    RuntimeVersionedRead, RuntimeVersionedSource, StorageEngine, VectorSourceAddress,
};

use rrd_core::{
    AuditEnvelope, DataTransaction, DataTransactionView, Millis, ProjectionWork, ReadStamp,
    RetentionPin, RuntimeChange, RuntimeChangePage, RuntimeCommit, RuntimeCommitOutcome,
    RuntimeDataSnapshot, RuntimeGraphSnapshot, RuntimeMutation, RuntimeReadValidation,
    RuntimeSchemaRegistry, ScopeId, SnapshotHandle, SnapshotId,
};

/// Canonical multi-model runtime repository shared by rrflowMX and rrflowKV.
/// It plans semantic effects against one captured physical snapshot and emits
/// the complete prepared plan through that transaction's single commit.
pub struct RuntimeRepository<'a> {
    storage: &'a dyn StorageEngine,
}

/// One all-model snapshot and the complete direct-read evidence that produced
/// its transaction-visible base. For a prospective transaction preview,
/// `selected_versions` and `read_evidence` describe the authenticated base;
/// the proposed mutations remain explicit in the caller's transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeDataSnapshotRead {
    pub read: ReadStamp,
    pub selected_versions: u64,
    pub read_evidence: RuntimeReadEvidence,
    pub snapshot: RuntimeDataSnapshot,
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

    /// Authenticates one previously captured read stamp without opening the
    /// causal log as a range source. Current heads use only current-state point
    /// reads; historical stamps use bounded accumulator and semantic-version
    /// point proofs.
    pub fn validate_read_stamp(&self, read: &ReadStamp) -> Result<RuntimeReadValidation> {
        let transaction = self.storage.begin_transaction()?;
        validate_runtime_read_stamp(&*transaction, read)
    }
}

/// Reduces directly selected, authenticated versions into the data snapshot
/// visible at `valid_at` under `read`.
fn reduce_data_snapshot(
    read: &ReadStamp,
    changes: &[RuntimeChange],
    valid_at: Millis,
) -> Result<RuntimeDataSnapshot> {
    Ok(if read.schema_revision.is_some() {
        let schema = schema_at_read(read, changes)?;
        RuntimeDataSnapshot::from_changes(
            changes,
            &schema,
            read.scope.clone(),
            valid_at,
            read.commit_cursor,
        )?
    } else {
        RuntimeDataSnapshot::from_claim_changes(
            changes,
            read.scope.clone(),
            valid_at,
            read.commit_cursor,
        )?
    })
}

fn selected_version_count(changes: &[RuntimeChange]) -> Result<u64> {
    u64::try_from(changes.len())
        .map_err(|_| Error::Substrate("selected semantic-version count exceeds u64".into()))
}

fn read_budget(max_keys: usize, operation: &str) -> Result<RuntimeReadBudget> {
    let max_keys = u64::try_from(max_keys)
        .map_err(|_| Error::Substrate(format!("{operation} key budget exceeds u64")))?;
    RuntimeReadBudget::new(max_keys)
}
