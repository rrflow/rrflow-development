//! rrflowKV store, transaction, inspection and reconciliation-evidence types.

use super::*;

pub(super) const RUNTIME_CHECKPOINT_PREFIX: &str = "runtime-";
pub const RRFLOW_KV_OPEN_EVIDENCE_VERSION: u16 = 2;

/// Work performed by startup checkpoint reconciliation after the physical
/// database has opened but before the store is published to callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RrflowKvReconciliationEvidence {
    pub contract_version: u16,
    pub page_requests: u64,
    pub page_cache_hits: u64,
    pub page_cache_misses: u64,
    pub page_cache_loads: u64,
    pub page_cache_evictions: u64,
    pub page_bytes_read: u64,
    pub page_bytes_decoded: u64,
    pub page_bytes_borrowed: u64,
    pub page_bytes_allocated: u64,
    pub page_bytes_copied: u64,
    pub page_bytes_decompressed: u64,
    pub filter_checks: u64,
    pub filter_negatives: u64,
    pub io_fallbacks: u64,
    pub read_operations: u64,
    pub mmap_read_operations: u64,
    pub io_uring_read_operations: u64,
    pub bounded_read_operations: u64,
    pub bytes_read: u64,
}

impl Default for RrflowKvReconciliationEvidence {
    fn default() -> Self {
        Self {
            contract_version: RRFLOW_KV_OPEN_EVIDENCE_VERSION,
            page_requests: 0,
            page_cache_hits: 0,
            page_cache_misses: 0,
            page_cache_loads: 0,
            page_cache_evictions: 0,
            page_bytes_read: 0,
            page_bytes_decoded: 0,
            page_bytes_borrowed: 0,
            page_bytes_allocated: 0,
            page_bytes_copied: 0,
            page_bytes_decompressed: 0,
            filter_checks: 0,
            filter_negatives: 0,
            io_fallbacks: 0,
            read_operations: 0,
            mmap_read_operations: 0,
            io_uring_read_operations: 0,
            bounded_read_operations: 0,
            bytes_read: 0,
        }
    }
}

/// Immutable evidence for the two physically distinct read phases performed
/// by [`RrflowKvStore::open_with_options`]. Query evidence is owned by each
/// projected read and never inferred from these process-wide counters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RrflowKvOpenEvidence {
    pub contract_version: u16,
    pub created: bool,
    pub segment_validation: SegmentOpenEvidence,
    pub reconciliation: RrflowKvReconciliationEvidence,
}

pub struct RrflowKvStore {
    pub(super) path: AnchoredPath,
    /// Reads take the shared side, because they need only `&Database`. Anything that needs
    /// `&mut Database` takes the exclusive side. On Linux a waiting writer blocks new readers, so a
    /// stream of reads cannot starve a commit.
    pub(super) database: RwLock<Database>,
    pub(super) open_evidence: RrflowKvOpenEvidence,
    pub(super) native_write_path_diagnostics_enabled: AtomicBool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RrflowKvInspection {
    pub physical: DatabaseInspection,
    pub claim_sequence: u64,
    pub runtime_cursor: u64,
    pub control_journal_sequence: u64,
}

/// Read-only rrflowKV semantic inspection over an authenticated physical
/// snapshot. It intentionally exposes no storage transaction or repair path.
pub struct RrflowKvInspector {
    pub(super) database: ReadOnlyDatabase,
    pub(super) inspection: RrflowKvInspection,
}

pub(super) struct RrflowKvTransaction<'a> {
    pub(super) store: &'a RrflowKvStore,
    pub(super) transaction: rrd_lsm::Transaction,
    pub(super) begin_mutex_wait_ns: u64,
}
