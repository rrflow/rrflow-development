//! Database options, maintenance and compaction policies, and the reports they produce.

use super::*;

/// Synchronous soft write-path maintenance targets. Crossing either target
/// stalls the next writer while the existing WAL-backed memtable is published.
/// One batch may exceed a soft target, but must fit the separate hard mutable
/// limits. Neither policy bounds retained generations or whole-process memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenancePolicy {
    pub wal_payload_max_bytes: usize,
    pub memtable_max_versions: usize,
}

/// Deterministic bounds for one immutable maintenance step. The input bound
/// prevents a growing database from turning one compaction into an unbounded
/// all-segment rewrite; the output target is enforced at key boundaries so all
/// MVCC versions for one key remain together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionPolicy {
    pub l0_compaction_trigger: usize,
    pub max_input_segments: usize,
    /// Soft logical output target shared by flush and compaction. A complete
    /// MVCC key group may exceed it; hard mutable/maintenance limits still apply.
    pub target_segment_bytes: usize,
    pub max_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseOptions {
    /// Maximum named checkpoint-directory entries, including interrupted temporary files.
    pub checkpoint_entries: usize,
    pub transaction_limits: crate::TransactionLimits,
    /// Per-open aggregate ownership ceilings, independent of caller budgets.
    pub active_read_views: usize,
    pub pinned_read_bytes: u64,
    /// Available native bytes retained beyond each proposed WAL/immutable
    /// allocation. This is admission, not reserved disk space or a total quota.
    pub recovery_reserve_bytes: u64,
    /// Output allocation cap shared by projected streams of this open instance.
    pub projected_output_bytes: usize,
    /// Live owned source allocations across cached and uncached readers.
    pub source_page_bytes: usize,
    pub page_cache_bytes: usize,
    pub page_cache_policy: PageCachePolicy,
    pub segment_io: SegmentIoPolicy,
    pub segment_row_group_budget: SegmentRowGroupBudget,
    pub segment_compression: SegmentCompressionPolicy,
    pub maintenance: MaintenancePolicy,
    pub mutable_limits: crate::MutableLimits,
    pub mutable_retention: crate::MutableRetentionLimits,
    pub compaction: CompactionPolicy,
    pub compaction_limits: crate::CompactionLimits,
    pub reclamation: crate::ReclamationLimits,
    pub recovery: crate::RecoveryLimits,
}

/// Process-local maintenance counters. Canonical state lives in the WAL,
/// segments, and manifests; these counters intentionally reset on reopen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceStats {
    pub automatic_flushes: u64,
    pub write_stalls: u64,
    pub failed_flushes: u64,
    pub oversized_batches: u64,
    pub peak_wal_payload_bytes: usize,
    pub peak_memtable_versions: usize,
    pub peak_accounted_memtable_bytes: usize,
    pub automatic_compactions: u64,
    pub failed_compactions: u64,
    pub compaction_input_bytes: u64,
    pub compaction_output_bytes: u64,
    pub peak_compaction_buffer_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionOutcome {
    pub previous_manifest: String,
    pub manifest: String,
    pub input_segments: usize,
    pub output_segments: usize,
    pub input_versions: u64,
    pub output_versions: u64,
    pub protected_sequences: Vec<u64>,
    pub source_level: u8,
    pub target_level: u8,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub peak_buffer_bytes: usize,
    pub history_pruned: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GarbageCollectionReport {
    pub retained_manifests: Vec<String>,
    pub retained_segments: Vec<String>,
    pub retained_wals: Vec<String>,
    pub removed_manifests: Vec<String>,
    pub removed_segments: Vec<String>,
    pub removed_wals: Vec<String>,
    pub examined_entries: u64,
    pub metadata_bytes: u64,
    pub removed_bytes: u64,
    pub has_more: bool,
}
