//! The StorageEngine port over rrflowKV and its physical-store evidence.

use super::*;

impl StorageEngine for RrflowKvStore {
    fn begin_transaction(&self) -> Result<Box<dyn StorageTransaction + '_>> {
        let diagnostics_enabled = self
            .native_write_path_diagnostics_enabled
            .load(Ordering::Acquire);
        let begin_lock_started = diagnostics_enabled.then(Instant::now);
        let database = self.read_lock()?;
        let begin_mutex_wait_ns = begin_lock_started.map_or(0, |started| {
            u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
        });
        let transaction = database.begin_transaction()?;
        drop(database);
        tracing::debug!(
            target: "rrd_store::transaction",
            storage_profile = "rrflow_kv",
            snapshot_sequence = transaction.snapshot().sequence,
            outcome = "begun",
            "storage transaction began"
        );
        Ok(Box::new(RrflowKvTransaction {
            store: self,
            transaction,
            begin_mutex_wait_ns,
        }))
    }

    fn claims(&self) -> crate::ClaimRepository<'_> {
        crate::ClaimRepository::new(self)
    }

    fn control(&self) -> crate::ControlRepository<'_> {
        crate::ControlRepository::new(self)
    }

    fn function_catalogue(&self) -> crate::FunctionCatalogueRepository<'_> {
        crate::FunctionCatalogueRepository::new(self)
    }

    fn projections(&self) -> crate::ProjectionRepository<'_> {
        crate::ProjectionRepository::new(self)
    }

    fn runtime(&self) -> crate::RuntimeRepository<'_> {
        crate::RuntimeRepository::new(self)
    }

    fn invocations(&self) -> crate::InvocationRepository<'_> {
        crate::InvocationRepository::new(self)
    }

    fn physical_store_evidence(&self) -> Result<PhysicalStoreEvidence> {
        // Clock reads and events are paid only for explicitly enabled developer
        // traces. These measurements are not storage state or a delivery SLA.
        let timed = tracing::event_enabled!(
            target: "rrd_store::diagnostic", tracing::Level::DEBUG,
            mutex_wait_ns, mutex_held_assembly_ns, memtable_profile_ns,
            memtable_keys, memtable_versions, segment_count, message
        );
        let wait_start = timed.then(Instant::now);
        let database = self.read_lock()?;
        let acquired = timed.then(Instant::now);
        let manifest = database.manifest();
        let cache = database.page_cache_stats();
        let source_pages = database.source_page_memory().stats();
        let segment_io = database.segment_io_stats();
        let maintenance = database.maintenance_policy();
        let compaction = database.compaction_policy();
        let maintenance_stats = database.maintenance_stats();
        let profile_start = timed.then(Instant::now);
        let memtable = database.memtable().profile();
        let profile_end = timed.then(Instant::now);
        let (checkpoint_entries, checkpoint_limit) = database.checkpoint_ownership()?;
        let (native_transactions, native_transaction_bytes) = database.transaction_ownership()?;
        let (active_read_views, pinned_read_bytes) = database.read_view_ownership()?;
        let segment_count = manifest.segments.len() as u64;
        let evidence = PhysicalStoreEvidence {
            backend: "rrflow_kv".into(),
            evidence_level: RRFLOW_KV_COUNTER_EVIDENCE.into(),
            physical_sequence: Some(database.snapshot().sequence),
            manifest_generation: Some(manifest.generation),
            durable_sequence: Some(manifest.durable_sequence),
            checkpoint_entries: Some(checkpoint_entries),
            checkpoint_limit: Some(checkpoint_limit),
            native_transactions: Some(native_transactions),
            native_transaction_bytes: Some(native_transaction_bytes),
            active_read_views: Some(active_read_views),
            pinned_read_bytes: Some(pinned_read_bytes),
            retained_mutable_bytes: Some(database.mutable_memory().stats().accounted_bytes as u64),
            retained_source_bytes: Some(source_pages.reserved_bytes as u64),
            peak_source_bytes: Some(source_pages.peak_reserved_bytes as u64),
            denied_source_allocations: Some(source_pages.denied_allocations),
            retained_projected_bytes: Some(
                database.projected_output_memory().stats().reserved_bytes as u64,
            ),
            memtable_versions: Some(database.memtable().version_count() as u64),
            memtable_bytes: Some(database.memtable().approximate_bytes() as u64),
            memtable_keys: Some(memtable.key_count as u64),
            memtable_key_payload_bytes: Some(memtable.key_payload_bytes as u64),
            memtable_value_payload_bytes: Some(memtable.value_payload_bytes as u64),
            memtable_tombstones: Some(memtable.tombstones as u64),
            memtable_spilled_chains: Some(memtable.spilled_chains as u64),
            memtable_spilled_version_capacity: Some(memtable.spilled_version_capacity as u64),
            memtable_version_record_bytes: Some(memtable.version_record_bytes as u64),
            memtable_owned_bytes_lower_bound: Some(memtable.owned_bytes_lower_bound as u64),
            memtable_max_versions: Some(maintenance.memtable_max_versions as u64),
            wal_payload_bytes: Some(database.wal_payload_bytes() as u64),
            wal_payload_max_bytes: Some(maintenance.wal_payload_max_bytes as u64),
            automatic_flushes: Some(maintenance_stats.automatic_flushes),
            maintenance_write_stalls: Some(maintenance_stats.write_stalls),
            failed_maintenance_flushes: Some(maintenance_stats.failed_flushes),
            oversized_batches: Some(maintenance_stats.oversized_batches),
            automatic_compactions: Some(maintenance_stats.automatic_compactions),
            failed_compactions: Some(maintenance_stats.failed_compactions),
            compaction_input_bytes: Some(maintenance_stats.compaction_input_bytes),
            compaction_output_bytes: Some(maintenance_stats.compaction_output_bytes),
            peak_compaction_buffer_bytes: Some(
                maintenance_stats.peak_compaction_buffer_bytes as u64,
            ),
            l0_segment_count: Some(database.l0_segment_count() as u64),
            l0_compaction_trigger: Some(compaction.l0_compaction_trigger as u64),
            compaction_debt_segments: Some(database.compaction_debt_segments() as u64),
            compaction_target_segment_bytes: Some(compaction.target_segment_bytes as u64),
            segment_count: Some(segment_count),
            segment_bytes: Some(manifest.segments.iter().map(|segment| segment.bytes).sum()),
            cache_capacity_bytes: Some(cache.capacity_bytes as u64),
            cache_resident_bytes: Some(cache.resident_bytes as u64),
            cache_entries: Some(cache.entries as u64),
            cache_hits: Some(cache.hits),
            cache_misses: Some(cache.misses),
            cache_evictions: Some(cache.evictions),
            page_loads: Some(cache.loads),
            page_bytes_read: Some(cache.bytes_read),
            page_bytes_decoded: Some(cache.bytes_decoded),
            page_bytes_borrowed: Some(cache.bytes_borrowed),
            page_bytes_allocated: Some(cache.bytes_allocated),
            page_bytes_copied: Some(cache.bytes_copied),
            page_bytes_decompressed: Some(cache.bytes_decompressed),
            filter_checks: Some(cache.filter_checks),
            filter_negatives: Some(cache.filter_negatives),
            segment_io_requested_mode: Some(segment_io.requested_mode.as_str().into()),
            segment_io_mmap_segments: Some(segment_io.mmap_segments),
            segment_io_uring_segments: Some(segment_io.io_uring_segments),
            segment_io_bounded_segments: Some(segment_io.bounded_segments),
            segment_io_fallbacks: Some(segment_io.fallback_count),
            segment_io_last_fallback: segment_io.last_fallback_reason,
            segment_io_read_operations: Some(segment_io.read_operations),
            segment_io_mmap_reads: Some(segment_io.mmap_read_operations),
            segment_io_uring_reads: Some(segment_io.io_uring_read_operations),
            segment_io_bounded_reads: Some(segment_io.bounded_read_operations),
            segment_io_bytes_read: Some(segment_io.bytes_read),
            segment_io_max_request_bytes: Some(segment_io.configured_max_request_bytes as u64),
            segment_io_peak_request_bytes: Some(segment_io.peak_request_bytes as u64),
            segment_io_max_open_files: Some(segment_io.configured_max_open_files as u64),
            segment_io_open_files: Some(segment_io.open_files as u64),
            segment_io_peak_open_files: Some(segment_io.peak_open_files as u64),
            segment_io_handle_evictions: Some(segment_io.handle_evictions),
            segment_io_handle_reopens: Some(segment_io.handle_reopens),
            segment_io_handle_denials: Some(segment_io.handle_denials),
        };
        let released_at = timed.then(Instant::now);
        // Do not hold the writer's database mutex while formatting or emitting
        // the trace: subscribers may block or perform arbitrary I/O.
        drop(database);
        if let (
            Some(wait_start),
            Some(acquired),
            Some(profile_start),
            Some(profile_end),
            Some(released_at),
        ) = (
            wait_start,
            acquired,
            profile_start,
            profile_end,
            released_at,
        ) {
            let nanos = |duration: std::time::Duration| {
                u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
            };
            tracing::debug!(
                target: "rrd_store::diagnostic",
                mutex_wait_ns = nanos(acquired.duration_since(wait_start)),
                mutex_held_assembly_ns = nanos(released_at.duration_since(acquired)),
                memtable_profile_ns = nanos(profile_end.duration_since(profile_start)),
                memtable_keys = memtable.key_count,
                memtable_versions = memtable.version_count,
                segment_count,
                "physical storage diagnostic assembled"
            );
        }
        Ok(evidence)
    }
}

/// Converts the one profile-neutral encoded semantic plan into the complete
/// rrflowKV mutation vector used by the temporary Raft application bridge.
///
/// The bridge appends coordinator metadata to this complete vector. Its raw
/// access remains an inventoried distributed-authority gap; centralizing the
/// semantic plan here removes the second planner but does not close that gap.
impl SemanticCommitPlan {
    pub fn into_parts_for(
        self,
        database: &Database,
    ) -> Result<(RuntimeCommitOutcome, Vec<Mutation>)> {
        database_codec(database)?;
        let (outcome, writes) = self.into_encoded_writes();
        let operations = writes
            .into_iter()
            .map(|(key, value)| match value {
                Some(value) => Mutation::Put { key, value },
                None => Mutation::Delete { key },
            })
            .collect();
        Ok((outcome, operations))
    }
}
