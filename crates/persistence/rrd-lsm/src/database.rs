mod compaction;
mod failpoints;
mod flush;
mod gc;
mod open;
mod options;
mod point_index;
mod reads;
mod snapshot;
mod transactions;

pub use failpoints::*;
pub use options::*;

use compaction::*;
pub(crate) use open::*;

use point_index::PointIndex;

use crate::filesystem::{AnchoredPath, OpenOptions};

use crate::io::{IoContext, SharedIoContext};

use crate::mutable::MutableCharge;

use crate::segment::{
    new_page_cache_with_limits, next_projected_cache_access, page_cache_stats, ActiveReadViews,
    ReadView, SelectedVersion, SharedPageCache,
};

use crate::write_diagnostics::{PendingWriteDiagnostics, PhaseClock, WritePathCollector};

use crate::{
    AppendReceipt, Checkpoint, Durability, Error, Manifest, ManifestStore, Memtable,
    PageCachePolicy, ProjectedReadRequest, ProjectedReadResource, ProjectedReadStream, Result,
    Segment, SegmentCompressionPolicy, SegmentIoPolicy, SegmentIoStats, SegmentOpenEvidence,
    SegmentRowGroupBudget, SnapshotBundle, SnapshotBundleFile, SnapshotExportBoundary,
    SnapshotSegment, VersionedValue, WalWriter, WriteBatch, WriteLockWait, WritePathDiagnostics,
};

use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, BTreeSet};

use std::path::Path;

use std::sync::Arc;

use std::time::Instant;

pub(crate) const WAL_DIRECTORY: &str = "wal";

pub(crate) const SEGMENT_DIRECTORY: &str = "segments";

pub const DEFAULT_WAL_PAYLOAD_MAX_BYTES: usize = 64 * 1024 * 1024;

pub const DEFAULT_MEMTABLE_MAX_VERSIONS: usize = 524_288;

pub const DEFAULT_L0_COMPACTION_TRIGGER: usize = 8;

pub const DEFAULT_COMPACTION_MAX_INPUT_SEGMENTS: usize = 16;

pub const DEFAULT_COMPACTION_TARGET_SEGMENT_BYTES: usize = 8 * 1024 * 1024;

pub const DEFAULT_MAX_COMPACTION_LEVEL: u8 = 6;

impl Default for MaintenancePolicy {
    fn default() -> Self {
        Self {
            wal_payload_max_bytes: DEFAULT_WAL_PAYLOAD_MAX_BYTES,
            memtable_max_versions: DEFAULT_MEMTABLE_MAX_VERSIONS,
        }
    }
}

impl MaintenancePolicy {
    fn validate(self) -> Result<Self> {
        if self.wal_payload_max_bytes == 0 || self.memtable_max_versions == 0 {
            return Err(Error::InvalidConfiguration(
                "WAL payload and memtable version limits must be greater than zero".into(),
            ));
        }
        Ok(self)
    }
}

impl Default for CompactionPolicy {
    fn default() -> Self {
        Self {
            l0_compaction_trigger: DEFAULT_L0_COMPACTION_TRIGGER,
            max_input_segments: DEFAULT_COMPACTION_MAX_INPUT_SEGMENTS,
            target_segment_bytes: DEFAULT_COMPACTION_TARGET_SEGMENT_BYTES,
            max_level: DEFAULT_MAX_COMPACTION_LEVEL,
        }
    }
}

impl CompactionPolicy {
    fn validate(self) -> Result<Self> {
        if self.l0_compaction_trigger < 2
            || self.max_input_segments < 2
            || self.target_segment_bytes == 0
            || self.max_level == 0
        {
            return Err(Error::InvalidConfiguration(
                "compaction requires an L0 trigger and input bound of at least two, a non-zero output target, and at least one level".into(),
            ));
        }
        Ok(self)
    }
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            checkpoint_entries: crate::manifest::DEFAULT_CHECKPOINT_ENTRIES,
            transaction_limits: crate::TransactionLimits::default(),
            active_read_views: crate::DEFAULT_MAX_ACTIVE_READ_VIEWS,
            pinned_read_bytes: crate::DEFAULT_MAX_PINNED_READ_BYTES,
            projected_output_bytes: crate::DEFAULT_PROJECTED_OUTPUT_BYTES,
            source_page_bytes: crate::DEFAULT_SOURCE_PAGE_BYTES,
            recovery_reserve_bytes: 16 * 1024 * 1024,
            page_cache_bytes: crate::DEFAULT_PAGE_CACHE_BYTES,
            page_cache_policy: PageCachePolicy::default(),
            segment_io: SegmentIoPolicy::default(),
            segment_row_group_budget: SegmentRowGroupBudget::default(),
            segment_compression: SegmentCompressionPolicy::default(),
            maintenance: MaintenancePolicy::default(),
            mutable_limits: crate::MutableLimits::default(),
            mutable_retention: crate::MutableRetentionLimits::default(),
            compaction: CompactionPolicy::default(),
            compaction_limits: crate::CompactionLimits::default(),
            reclamation: crate::ReclamationLimits::default(),
            recovery: crate::RecoveryLimits::default(),
        }
    }
}

impl DatabaseOptions {
    pub(crate) fn validate(self) -> Result<Self> {
        if self.checkpoint_entries == 0 || self.checkpoint_entries > 65_536 {
            return Err(Error::InvalidConfiguration(
                "checkpoint entries must be in 1..=65536".into(),
            ));
        }
        if self.source_page_bytes == 0
            || self.projected_output_bytes == 0
            || self.recovery_reserve_bytes == 0
            || self.active_read_views == 0
            || self.pinned_read_bytes == 0
        {
            return Err(Error::InvalidConfiguration(
                "source pages, projected output, read retention and recovery reserve must be positive".into(),
            ));
        }
        self.page_cache_policy
            .protected_capacity_bytes(self.page_cache_bytes)?;
        self.segment_io.validate()?;
        self.segment_row_group_budget.validate()?;
        self.segment_compression.validate()?;
        self.transaction_limits.validate()?;
        self.maintenance.validate()?;
        self.mutable_limits.validate()?;
        if self.mutable_retention.accounted_bytes < self.mutable_limits.accounted_bytes
            || self.mutable_retention.versions < self.mutable_limits.versions
            || self.mutable_retention.generations == 0
        {
            return Err(Error::InvalidConfiguration(
                "mutable retention must cover active limits and at least one generation".into(),
            ));
        }
        self.compaction.validate()?;
        self.compaction_limits.validate()?;
        self.reclamation.validate()?;
        self.recovery.validate()?;
        if self.compaction_limits.l0_segments < self.compaction.l0_compaction_trigger as u64 {
            return Err(Error::InvalidConfiguration(
                "L0 hard capacity is below its compaction trigger".into(),
            ));
        }
        Ok(self)
    }
}

impl WriteBoundary {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "write.prepared",
            Self::WalAppended => "write.wal_appended",
            Self::WalSynced => "write.wal_synced",
            Self::Visible => "write.visible",
        }
    }
}

impl FailureMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Crash => "crash",
            Self::StorageFull => "storage-full",
        }
    }
}

impl FlushBoundary {
    fn as_str(self) -> &'static str {
        match self {
            Self::WalSynced => "flush.wal_synced",
            Self::SegmentSynced => "flush.segment_synced",
            Self::SuccessorWalSynced => "flush.successor_wal_synced",
            Self::ManifestPublished => "flush.manifest_published",
        }
    }
}

impl SnapshotInstallBoundary {
    fn as_str(self) -> &'static str {
        match self {
            Self::SegmentsSynced => "snapshot.segments_synced",
            Self::SuccessorWalSynced => "snapshot.successor_wal_synced",
            Self::ManifestPublished => "snapshot.manifest_published",
        }
    }
}

impl CompactionBoundary {
    fn as_str(self) -> &'static str {
        match self {
            Self::SegmentSynced => "compaction.segment_synced",
            Self::ManifestPublished => "compaction.manifest_published",
        }
    }
}

/// Single-writer RRFlow LSM database with crash-ordered WAL → segment →
/// manifest publication. RRFlow's model, differential, recovery, and benchmark
/// gates define its correctness and performance requirements.
pub struct Database {
    root: AnchoredPath,
    manifests: ManifestStore,
    // Shared immutable generation: reads retain it without copying every descriptor.
    manifest: Arc<Manifest>,
    wal: WalWriter,
    memtable: Arc<crate::mutable_memory::MutableGeneration>,
    mutable_memory: crate::MutableMemory,
    segments: Vec<Arc<Segment>>,
    point_index: PointIndex,
    page_cache: SharedPageCache,
    segment_io: SharedIoContext,
    segment_open_evidence: SegmentOpenEvidence,
    segment_row_group_budget: SegmentRowGroupBudget,
    segment_compression: SegmentCompressionPolicy,
    maintenance: MaintenancePolicy,
    mutable_limits: crate::MutableLimits,
    compaction: CompactionPolicy,
    compaction_limits: crate::CompactionLimits,
    reclamation: crate::ReclamationLimits,
    reclamation_pending: bool,
    maintenance_stats: MaintenanceStats,
    wal_payload_bytes: usize,
    writer_requires_reopen: Option<&'static str>,
    transaction_snapshots: Arc<crate::transaction::TransactionSnapshots>,
    active_read_views: Arc<std::sync::Mutex<ActiveReadViews>>,
    active_read_view_limit: usize,
    pinned_read_byte_limit: u64,
    projected_output: crate::ProjectedOutputMemory,
    write_path_diagnostics: Option<WritePathCollector>,
}

impl Database {
    pub fn root(&self) -> &Path {
        self.root.diagnostic_path()
    }

    pub fn anchored_root(&self) -> &AnchoredPath {
        &self.root
    }

    /// Check native space admission for one new engine-owned immutable file.
    /// Free space can change immediately; callers must still handle I/O errors.
    pub fn admit_immutable_file(&self, bytes: u64) -> Result<()> {
        self.root.admit_disk(bytes, true)
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            sequence: self.memtable.maximum_sequence(),
        }
    }

    pub fn mutable_memory(&self) -> crate::MutableMemory {
        self.mutable_memory.clone()
    }

    /// Coherent aggregate native transaction owner/buffer charge observation.
    pub fn transaction_ownership(&self) -> Result<(u64, u64)> {
        self.transaction_snapshots.ownership()
    }

    /// Aggregate live stream ownership: each retained segment or mutable
    /// generation counts once. Returned batches own separate output reservations.
    pub fn read_view_ownership(&self) -> Result<(u64, u64)> {
        ActiveReadViews::ownership(&self.active_read_views)
    }

    pub fn source_page_memory(&self) -> crate::SourcePageMemory {
        crate::segment::source_page_memory(&self.page_cache)
    }

    pub fn projected_output_memory(&self) -> crate::ProjectedOutputMemory {
        self.projected_output.clone()
    }

    pub fn memtable(&self) -> &Memtable {
        &self.memtable
    }

    pub fn mutable_limits(&self) -> crate::MutableLimits {
        self.mutable_limits
    }

    pub fn maintenance_policy(&self) -> MaintenancePolicy {
        self.maintenance
    }

    pub fn segment_row_group_budget(&self) -> SegmentRowGroupBudget {
        self.segment_row_group_budget
    }

    pub fn segment_compression_policy(&self) -> SegmentCompressionPolicy {
        self.segment_compression
    }

    pub fn l0_segment_count(&self) -> usize {
        self.segments
            .iter()
            .filter(|segment| segment.descriptor.level == 0)
            .count()
    }

    pub fn maintenance_stats(&self) -> MaintenanceStats {
        self.maintenance_stats
    }

    /// Enables bounded, process-local measurement for subsequently accepted
    /// owned write batches. An active buffer must be drained before another
    /// one is enabled so evidence cannot be discarded implicitly.
    pub fn enable_write_path_diagnostics(&mut self, capacity: usize) -> Result<()> {
        if self.write_path_diagnostics.is_some() {
            return Err(Error::InvalidConfiguration(
                "write path diagnostics are already enabled".into(),
            ));
        }
        self.write_path_diagnostics = Some(WritePathCollector::new(capacity)?);
        Ok(())
    }

    /// Disables and drains the current process-local write diagnostic buffer.
    pub fn take_write_path_diagnostics(&mut self) -> Option<WritePathDiagnostics> {
        self.write_path_diagnostics
            .take()
            .map(WritePathCollector::finish)
    }

    pub fn wal_payload_bytes(&self) -> usize {
        self.wal_payload_bytes
    }

    /// Current shared immutable-page residency and effectiveness counters.
    pub fn page_cache_stats(&self) -> crate::PageCacheStats {
        page_cache_stats(&self.page_cache)
    }

    pub fn segment_io_stats(&self) -> SegmentIoStats {
        self.segment_io.stats()
    }

    /// Immutable phase evidence captured while authenticating the segments
    /// referenced by `CURRENT`. Later page reads cannot mutate this record.
    pub fn segment_open_evidence(&self) -> &SegmentOpenEvidence {
        &self.segment_open_evidence
    }

    fn ensure_no_active_transactions(&self, operation: &'static str) -> Result<()> {
        if self.transaction_snapshots.is_empty()? {
            Ok(())
        } else {
            Err(Error::ActiveTransactions { operation })
        }
    }
}

#[cfg(test)]
mod compaction_shadow_tests {
    use super::*;
    use std::cell::Cell;

    fn live(sequence: u64) -> VersionedValue {
        VersionedValue {
            sequence,
            value: Some(b"value".to_vec().into_boxed_slice()),
        }
    }

    fn deleted(sequence: u64) -> VersionedValue {
        VersionedValue {
            sequence,
            value: None,
        }
    }

    #[test]
    fn live_or_protected_history_never_probes_unselected_segments() {
        for versions in [
            vec![live(1)],
            vec![live(1), deleted(2)],
            vec![deleted(1), live(2)],
        ] {
            for retain_all in [false, true] {
                assert!(retain_compacted_key(&versions, retain_all, || {
                    panic!("live history must not scan unrelated segments")
                }));
            }
        }
    }

    #[test]
    fn empty_or_unpruned_tombstones_do_not_probe_shadows() {
        assert!(!retain_compacted_key(&[], false, || panic!("empty output")));
        assert!(!retain_compacted_key(&[], true, || panic!("empty output")));
        assert!(retain_compacted_key(&[deleted(1)], true, || panic!(
            "no pruning"
        )));
    }

    #[test]
    fn tombstone_pruning_requires_exactly_one_shadow_decision() {
        for shadowed in [false, true] {
            let probes = Cell::new(0);
            assert_eq!(
                retain_compacted_key(&[deleted(1), deleted(2)], false, || {
                    probes.set(probes.get() + 1);
                    shadowed
                }),
                shadowed
            );
            assert_eq!(probes.get(), 1);
        }
    }
}
