//! Storage limits and their validation.

use super::*;

/// Installed native storage limits. These bound separately accounted owners,
/// not total process RSS. Changing them requires a new governed configuration
/// and reopening storage; they are never hot-applied by a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StorageLimits {
    /// Explicitly configured background cold-history work; absent means disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cold_history: Option<ColdHistoryPolicy>,
    /// Named checkpoint-directory entry ceiling; restart required.
    pub checkpoint_entries: u64,
    /// Native transaction owner count and logical buffer capacity; restart required.
    pub native_transactions: u64,
    pub native_transaction_bytes: u64,
    /// Aggregate live physical read owners and charged pinned bytes; restart required.
    pub active_read_views: u64,
    pub pinned_read_bytes: u64,
    /// Bytes to keep available beyond each native WAL/immutable allocation.
    /// Requires restart; racy admission cannot replace actual I/O error handling.
    pub recovery_reserve_bytes: u64,
    pub mutable_accounted_bytes: u64,
    pub mutable_versions: u64,
    pub projected_output_bytes: u64,
    pub source_page_bytes: u64,
    pub page_cache_bytes: u64,
    /// Immutable segment file ceiling per native store; restart required.
    /// Omission preserves the native default of 128 and existing sealed records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_open_files: Option<u64>,
    pub wal_flush_target_bytes: u64,
    pub memtable_flush_target_versions: u64,
    pub compaction_input_bytes: u64,
    pub compaction_input_logical_bytes: u64,
    pub compaction_input_versions: u64,
    pub compaction_output_segments: u64,
    pub compaction_elapsed_ms: u64,
    pub compaction_l0_segments: u64,
    pub retained_mutable_bytes: u64,
    pub retained_mutable_versions: u64,
    pub retained_mutable_generations: u64,
    pub reclamation_inventory_entries: u64,
    pub reclamation_metadata_bytes: u64,
    pub reclamation_delete_files: u64,
    pub reclamation_delete_bytes: u64,
    pub reclamation_elapsed_ms: u64,
    pub transaction_admission_requests: u64,
    pub transaction_admission_bytes: u64,
    pub transaction_admission_wait_ms: u64,
    pub recovery_metadata_bytes: u64,
    pub recovery_files: u64,
    pub recovery_read_bytes: u64,
    pub recovery_wal_bytes: u64,
    pub recovery_wal_frames: u64,
    pub recovery_elapsed_ms: u64,
}

impl Default for StorageLimits {
    fn default() -> Self {
        Self {
            cold_history: None,
            checkpoint_entries: 1024,
            native_transactions: 64,
            native_transaction_bytes: 64 * 1024 * 1024,
            active_read_views: 1024,
            pinned_read_bytes: 4 * 1024 * 1024 * 1024,
            mutable_accounted_bytes: 128 * 1024 * 1024,
            recovery_reserve_bytes: 16 * 1024 * 1024,
            mutable_versions: 1_048_576,
            projected_output_bytes: 128 * 1024 * 1024,
            source_page_bytes: 256 * 1024 * 1024,
            page_cache_bytes: 64 * 1024 * 1024,
            segment_open_files: None,
            wal_flush_target_bytes: 64 * 1024 * 1024,
            memtable_flush_target_versions: 524_288,
            compaction_input_bytes: 1024 * 1024 * 1024,
            compaction_input_logical_bytes: 256 * 1024 * 1024,
            compaction_input_versions: 4_194_304,
            compaction_output_segments: 256,
            compaction_elapsed_ms: 30_000,
            compaction_l0_segments: 64,
            retained_mutable_bytes: 536_870_912,
            retained_mutable_versions: 4_194_304,
            retained_mutable_generations: 64,
            reclamation_inventory_entries: 262_144,
            reclamation_metadata_bytes: 67_108_864,
            reclamation_delete_files: 1_024,
            reclamation_delete_bytes: 1_073_741_824,
            reclamation_elapsed_ms: 30_000,
            transaction_admission_requests: 64,
            transaction_admission_bytes: 67_108_864,
            transaction_admission_wait_ms: 30_000,
            recovery_metadata_bytes: 67_108_864,
            recovery_files: 262_144,
            recovery_read_bytes: 1_099_511_627_776,
            recovery_wal_bytes: 268_435_456,
            recovery_wal_frames: 1_048_576,
            recovery_elapsed_ms: 120_000,
        }
    }
}

impl StorageLimits {
    pub fn validate(&self) -> Result<()> {
        if let Some(policy) = &self.cold_history {
            policy.validate()?;
        }
        // Fixed protocol ceilings keep admission finite across platforms;
        // the native adapter additionally checks host-size conversion.
        if let Some(limit) = self.segment_open_files {
            validate_nonzero_max("storage segment_open_files", limit, 4096)?;
        }
        const MAX_BYTES: u64 = 1 << 40;
        const MAX_VERSIONS: u64 = 1 << 32;
        validate_nonzero_max(
            "storage checkpoint_entries",
            self.checkpoint_entries,
            65_536,
        )?;
        validate_nonzero_max(
            "storage native_transactions",
            self.native_transactions,
            65_536,
        )?;
        validate_nonzero_max(
            "storage native_transaction_bytes",
            self.native_transaction_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max("storage active_read_views", self.active_read_views, 65_536)?;
        validate_nonzero_max(
            "storage pinned_read_bytes",
            self.pinned_read_bytes,
            MAX_BYTES,
        )?;
        for (name, value) in [
            ("storage source_page_bytes", self.source_page_bytes),
            (
                "storage recovery_reserve_bytes",
                self.recovery_reserve_bytes,
            ),
            (
                "storage mutable_accounted_bytes",
                self.mutable_accounted_bytes,
            ),
            (
                "storage projected_output_bytes",
                self.projected_output_bytes,
            ),
            (
                "storage wal_flush_target_bytes",
                self.wal_flush_target_bytes,
            ),
        ] {
            validate_nonzero_max(name, value, MAX_BYTES)?;
        }
        validate_nonzero_max(
            "storage mutable_versions",
            self.mutable_versions,
            MAX_VERSIONS,
        )?;
        validate_nonzero_max(
            "storage memtable_flush_target_versions",
            self.memtable_flush_target_versions,
            self.mutable_versions,
        )?;

        validate_nonzero_max(
            "storage compaction_input_bytes",
            self.compaction_input_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max(
            "storage compaction_input_logical_bytes",
            self.compaction_input_logical_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max(
            "storage compaction_input_versions",
            self.compaction_input_versions,
            MAX_VERSIONS,
        )?;
        validate_nonzero_max(
            "storage compaction_output_segments",
            self.compaction_output_segments,
            65_536,
        )?;
        validate_nonzero_max(
            "storage compaction_elapsed_ms",
            self.compaction_elapsed_ms,
            3_600_000,
        )?;
        validate_nonzero_max(
            "storage compaction_l0_segments",
            self.compaction_l0_segments,
            65_536,
        )?;
        validate_nonzero_max(
            "storage retained_mutable_bytes",
            self.retained_mutable_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max(
            "storage retained_mutable_versions",
            self.retained_mutable_versions,
            MAX_VERSIONS,
        )?;
        validate_nonzero_max(
            "storage retained_mutable_generations",
            self.retained_mutable_generations,
            65_536,
        )?;
        validate_nonzero_max(
            "storage reclamation_inventory_entries",
            self.reclamation_inventory_entries,
            1 << 24,
        )?;
        validate_nonzero_max(
            "storage reclamation_metadata_bytes",
            self.reclamation_metadata_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max(
            "storage reclamation_delete_files",
            self.reclamation_delete_files,
            1 << 20,
        )?;
        validate_nonzero_max(
            "storage reclamation_delete_bytes",
            self.reclamation_delete_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max(
            "storage reclamation_elapsed_ms",
            self.reclamation_elapsed_ms,
            3_600_000,
        )?;
        if self.retained_mutable_bytes < self.mutable_accounted_bytes
            || self.retained_mutable_versions < self.mutable_versions
        {
            return invalid("retained mutable limits must accommodate one full active generation");
        }
        validate_nonzero_max(
            "storage transaction_admission_requests",
            self.transaction_admission_requests,
            65_536,
        )?;
        validate_nonzero_max(
            "storage transaction_admission_bytes",
            self.transaction_admission_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max(
            "storage transaction_admission_wait_ms",
            self.transaction_admission_wait_ms,
            3_600_000,
        )?;
        validate_nonzero_max(
            "storage recovery_metadata_bytes",
            self.recovery_metadata_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max("storage recovery_files", self.recovery_files, 1 << 24)?;
        validate_nonzero_max(
            "storage recovery_read_bytes",
            self.recovery_read_bytes,
            1 << 50,
        )?;
        validate_nonzero_max(
            "storage recovery_wal_bytes",
            self.recovery_wal_bytes,
            MAX_BYTES,
        )?;
        validate_nonzero_max(
            "storage recovery_wal_frames",
            self.recovery_wal_frames,
            1 << 32,
        )?;
        validate_nonzero_max(
            "storage recovery_elapsed_ms",
            self.recovery_elapsed_ms,
            3_600_000,
        )?;
        if self.compaction_l0_segments < 8 {
            return invalid(
                "storage compaction_l0_segments must accommodate the native L0 trigger of eight",
            );
        }
        if self.page_cache_bytes > MAX_BYTES {
            return invalid("storage page_cache_bytes exceeds the protocol ceiling");
        }
        Ok(())
    }
}
