//! Transactions, projected reads and the single-writer write path.

use super::*;

impl Database {
    /// Captures one immutable logical/physical read view and returns a bounded
    /// projected stream that owns its memtable and segment generations. The
    /// caller can consume the stream without retaining a borrow of `Database`.
    pub fn begin_projected_read(
        &self,
        mut request: ProjectedReadRequest,
    ) -> Result<ProjectedReadStream> {
        let current_sequence = self.snapshot().sequence;
        request.validate(current_sequence)?;
        request.budget.max_active_views = request
            .budget
            .max_active_views
            .min(self.active_read_view_limit);
        request.budget.max_pinned_bytes = request
            .budget
            .max_pinned_bytes
            .min(self.pinned_read_byte_limit);
        let segments = self
            .segments
            .iter()
            .filter(|segment| {
                request.snapshot.sequence >= segment.descriptor.minimum_sequence
                    && request.ranges.iter().any(|range| {
                        range.overlaps(
                            segment.descriptor.first_key.as_slice(),
                            segment.descriptor.last_key.as_slice(),
                        )
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        let run_count = segments
            .len()
            .checked_add(usize::from(self.memtable.version_count() != 0))
            .ok_or_else(|| Error::InvalidProjectedRead("projected run count overflow".into()))?;
        if run_count > request.budget.max_runs {
            return Err(Error::ProjectedReadLimit {
                resource: ProjectedReadResource::Runs,
                limit: request.budget.max_runs as u64,
                observed: run_count as u64,
            });
        }
        let memtable_bytes = u64::try_from(self.memtable.approximate_bytes()).map_err(|_| {
            Error::InvalidProjectedRead("captured memtable bytes exceed u64".into())
        })?;
        let segment_bytes = segments.iter().try_fold(0_u64, |total, segment| {
            total.checked_add(segment.descriptor.bytes).ok_or_else(|| {
                Error::InvalidProjectedRead("pinned read-view bytes overflow".into())
            })
        })?;
        let pinned_bytes = memtable_bytes
            .checked_add(segment_bytes)
            .ok_or_else(|| Error::InvalidProjectedRead("pinned read-view bytes overflow".into()))?;
        let cache_access = next_projected_cache_access()?;
        let lease = ActiveReadViews::acquire(
            &self.active_read_views,
            Arc::clone(&self.manifest),
            Some(&self.memtable),
            memtable_bytes,
            &segments,
            request.budget,
        )?;
        let view = ReadView {
            sequence: request.snapshot.sequence,
            manifest: self.manifest.digest.clone(),
            memtable: Arc::clone(&self.memtable),
            segments,
            pinned_bytes,
        };
        Ok(ProjectedReadStream::new(
            view,
            request.ranges,
            request.projection,
            request.budget,
            cache_access,
            lease,
            self.projected_output.clone(),
        ))
    }

    /// Begins one snapshot-isolated physical transaction and pins its read
    /// sequence against history-pruning compaction until commit, rollback, or
    /// drop.
    pub fn begin_transaction(&self) -> Result<crate::Transaction> {
        let transaction =
            crate::Transaction::begin(self.snapshot(), Arc::clone(&self.transaction_snapshots))?;
        tracing::debug!(
            target: "rrd_lsm::transaction",
            snapshot_sequence = transaction.snapshot().sequence,
            outcome = "begun",
            "physical transaction began"
        );
        Ok(transaction)
    }

    /// Atomically conflict-checks and publishes one transaction write set.
    /// Only keys written by the transaction participate in conflict detection.
    pub fn commit_transaction(
        &mut self,
        transaction: crate::Transaction,
        durability: Durability,
    ) -> Result<crate::TransactionCommit> {
        self.commit_transaction_inner(
            transaction,
            durability,
            None,
            WriteLockWait::default(),
            None,
        )
    }

    /// Commits through the ordinary transaction path while attaching
    /// caller-measured adapter mutex waits to an explicitly enabled diagnostic
    /// sample. The context is ignored when collection is disabled.
    pub fn commit_transaction_with_write_context(
        &mut self,
        transaction: crate::Transaction,
        durability: Durability,
        lock_wait: WriteLockWait,
    ) -> Result<crate::TransactionCommit> {
        self.commit_transaction_inner(transaction, durability, None, lock_wait, None)
    }

    /// Commits with explicit snapshot retention for any foreground compaction.
    /// As with [`Self::compact`], callers must supply all untracked snapshots
    /// they need. Registered transactions are retained automatically; owned
    /// projected views and named checkpoints preserve their own generations.
    /// The lock-wait context only records an explicitly enabled diagnostic.
    pub fn commit_transaction_with_protected_snapshots(
        &mut self,
        transaction: crate::Transaction,
        durability: Durability,
        protected: &[Snapshot],
        lock_wait: WriteLockWait,
    ) -> Result<crate::TransactionCommit> {
        self.commit_transaction_inner(transaction, durability, None, lock_wait, Some(protected))
    }

    /// Commits through one deterministic write-boundary failure. This is the
    /// transaction-level fault surface used to prove that a semantic write
    /// set never becomes a collection of independently visible families.
    pub fn commit_transaction_with_failure(
        &mut self,
        transaction: crate::Transaction,
        durability: Durability,
        boundary: WriteBoundary,
        mode: FailureMode,
    ) -> Result<crate::TransactionCommit> {
        self.commit_transaction_inner(
            transaction,
            durability,
            Some((boundary, mode)),
            WriteLockWait::default(),
            None,
        )
    }

    pub(super) fn commit_transaction_inner(
        &mut self,
        mut transaction: crate::Transaction,
        durability: Durability,
        failure: Option<(WriteBoundary, FailureMode)>,
        lock_wait: WriteLockWait,
        protected: Option<&[Snapshot]>,
    ) -> Result<crate::TransactionCommit> {
        let mut diagnostics = self.write_path_diagnostics.as_ref().map(|_| {
            PendingWriteDiagnostics::start(
                lock_wait,
                self.maintenance_stats,
                self.memtable.approximate_bytes(),
            )
        });
        let validation_started = diagnostics.is_some().then(PhaseClock::start);
        if transaction.is_measuring() {
            return Err(Error::InvalidTransaction(
                "a measuring transaction cannot commit".into(),
            ));
        }
        self.ensure_writer_ready()?;
        self.validate_transaction_owner(&transaction)?;
        for key in transaction.pending().keys() {
            if let Some(conflicting_sequence) = self.latest_sequence_for_key(key)? {
                if conflicting_sequence > transaction.snapshot().sequence {
                    tracing::warn!(
                        target: "rrd_lsm::transaction",
                        snapshot_sequence = transaction.snapshot().sequence,
                        conflicting_sequence,
                        mutation_count = transaction.write_count(),
                        outcome = "conflict",
                        "physical transaction commit denied"
                    );
                    return Err(Error::TransactionConflict {
                        snapshot_sequence: transaction.snapshot().sequence,
                        conflicting_sequence,
                    });
                }
            }
        }
        if let Some(started) = validation_started {
            diagnostics
                .as_mut()
                .expect("diagnostic recorder checked")
                .phases
                .transaction_validation = started.finish();
        }
        let mutation_count = transaction.write_count();
        if mutation_count == 0 {
            tracing::debug!(
                target: "rrd_lsm::transaction",
                snapshot_sequence = transaction.snapshot().sequence,
                mutation_count,
                outcome = "read_only",
                "physical transaction committed without a write"
            );
            return Ok(crate::TransactionCommit {
                snapshot_sequence: transaction.snapshot().sequence,
                mutation_count,
                receipt: None,
            });
        }
        let snapshot_sequence = transaction.snapshot().sequence;
        let batch_started = diagnostics.is_some().then(PhaseClock::start);
        let batch = WriteBatch::new(transaction.take_mutations())?;
        if let Some(started) = batch_started {
            diagnostics
                .as_mut()
                .expect("diagnostic recorder checked")
                .phases
                .batch_prepare = started.finish();
        }
        let receipt = self.write_owned_inner(batch, durability, failure, diagnostics, protected)?;
        tracing::debug!(
            target: "rrd_lsm::transaction",
            snapshot_sequence,
            mutation_count,
            first_sequence = receipt.first_sequence,
            last_sequence = receipt.last_sequence,
            durable = receipt.durable,
            outcome = "committed",
            "physical transaction committed"
        );
        Ok(crate::TransactionCommit {
            snapshot_sequence,
            mutation_count,
            receipt: Some(receipt),
        })
    }

    pub(crate) fn validate_transaction_owner(
        &self,
        transaction: &crate::Transaction,
    ) -> Result<()> {
        if transaction.belongs_to(&self.transaction_snapshots) {
            Ok(())
        } else {
            Err(Error::TransactionDatabaseMismatch)
        }
    }

    pub(super) fn latest_sequence_for_key(&self, key: &[u8]) -> Result<Option<u64>> {
        let read_sequence = self.snapshot().sequence;
        // As in get(), every active-memtable version is newer than all
        // published segments. A tombstone is still an authoritative version.
        if let Some(version) = self.memtable.get_version(key, read_sequence) {
            return Ok(Some(version.sequence));
        }
        let mut latest = None;
        for segment in &self.segments {
            if let Some(sequence) = segment.get_sequence(key, read_sequence)? {
                if latest.is_none_or(|latest| sequence > latest) {
                    latest = Some(sequence);
                }
            }
        }
        Ok(latest)
    }

    pub fn write(&mut self, batch: &WriteBatch, durability: Durability) -> Result<AppendReceipt> {
        self.write_inner(batch, durability, None)
    }

    pub fn write_with_failure(
        &mut self,
        batch: &WriteBatch,
        durability: Durability,
        boundary: WriteBoundary,
        mode: FailureMode,
    ) -> Result<AppendReceipt> {
        self.write_inner(batch, durability, Some((boundary, mode)))
    }

    pub(super) fn write_inner(
        &mut self,
        batch: &WriteBatch,
        durability: Durability,
        failure: Option<(WriteBoundary, FailureMode)>,
    ) -> Result<AppendReceipt> {
        self.ensure_writer_ready()?;
        let charge = MutableCharge::batch(batch)?;
        self.mutable_limits.check(charge)?;
        let payload = batch.encode()?;
        self.prepare_write(batch, payload.len(), charge, None)?;
        let prepared_mutable = self.mutable_memory.prepare(&self.memtable, charge)?;
        inject_write_failure(failure, WriteBoundary::Prepared)?;
        let append_durability = append_durability(durability, failure);
        let receipt = self
            .wal
            .append_encoded_write_batch(batch, &payload, append_durability)?;
        self.inject_post_append_failure(failure)?;
        self.inject_post_wal_failure(failure, &receipt)?;
        if prepared_mutable
            .apply(
                &mut self.memtable,
                batch,
                receipt.first_sequence,
                receipt.last_sequence,
            )
            .is_err()
        {
            // WAL may already be durable. Never report this as an absent write.
            self.writer_requires_reopen = Some("mutable-publication");
            return Err(Error::RecoveryRequired {
                boundary: "mutable-publication",
            });
        }
        self.wal_payload_bytes = self.wal_payload_bytes.saturating_add(payload.len());
        self.record_memtable_peak();
        self.inject_post_visibility_failure(failure)?;
        Ok(receipt)
    }

    /// Owned fast path used by adapters that build a batch for immediate
    /// commit. Keys and values move into the memtable after WAL encoding rather
    /// than being cloned a second time.
    pub fn write_owned(
        &mut self,
        batch: WriteBatch,
        durability: Durability,
    ) -> Result<AppendReceipt> {
        let diagnostics = self.write_path_diagnostics.as_ref().map(|_| {
            PendingWriteDiagnostics::start(
                WriteLockWait::default(),
                self.maintenance_stats,
                self.memtable.approximate_bytes(),
            )
        });
        self.write_owned_inner(batch, durability, None, diagnostics, None)
    }

    pub fn write_owned_with_failure(
        &mut self,
        batch: WriteBatch,
        durability: Durability,
        boundary: WriteBoundary,
        mode: FailureMode,
    ) -> Result<AppendReceipt> {
        let diagnostics = self.write_path_diagnostics.as_ref().map(|_| {
            PendingWriteDiagnostics::start(
                WriteLockWait::default(),
                self.maintenance_stats,
                self.memtable.approximate_bytes(),
            )
        });
        self.write_owned_inner(batch, durability, Some((boundary, mode)), diagnostics, None)
    }

    pub(super) fn write_owned_inner(
        &mut self,
        batch: WriteBatch,
        durability: Durability,
        failure: Option<(WriteBoundary, FailureMode)>,
        mut diagnostics: Option<PendingWriteDiagnostics>,
        protected: Option<&[Snapshot]>,
    ) -> Result<AppendReceipt> {
        self.ensure_writer_ready()?;
        let charge = MutableCharge::batch(&batch)?;
        self.mutable_limits.check(charge)?;
        let mutation_count = batch.len();
        let encode_started = diagnostics.is_some().then(PhaseClock::start);
        let payload = batch.encode()?;
        if let Some(started) = encode_started {
            diagnostics
                .as_mut()
                .expect("diagnostic recorder checked")
                .phases
                .batch_encode = started.finish();
        }
        let payload_bytes = payload.len();
        let maintenance_started = diagnostics.is_some().then(PhaseClock::start);
        self.prepare_write(&batch, payload.len(), charge, protected)?;
        if let Some(started) = maintenance_started {
            diagnostics
                .as_mut()
                .expect("diagnostic recorder checked")
                .phases
                .maintenance = started.finish();
        }
        let prepared_mutable = self.mutable_memory.prepare(&self.memtable, charge)?;
        inject_write_failure(failure, WriteBoundary::Prepared)?;
        let append_durability = append_durability(durability, failure);
        let receipt = if let Some(diagnostics) = diagnostics.as_mut() {
            let (receipt, wal) = self.wal.append_encoded_write_batch_profiled(
                &batch,
                &payload,
                append_durability,
            )?;
            diagnostics.phases.wal_record_prepare = wal.record_prepare;
            diagnostics.phases.wal_initial_reservation = wal.initial_reservation;
            diagnostics.phases.wal_write = wal.write;
            diagnostics.phases.wal_sync = wal.sync;
            receipt
        } else {
            self.wal
                .append_encoded_write_batch(&batch, &payload, append_durability)?
        };
        self.inject_post_append_failure(failure)?;
        self.inject_post_wal_failure(failure, &receipt)?;
        let apply_started = diagnostics.is_some().then(PhaseClock::start);
        if prepared_mutable
            .apply_owned(
                &mut self.memtable,
                batch,
                receipt.first_sequence,
                receipt.last_sequence,
            )
            .is_err()
        {
            // WAL may already be durable. Never report this as an absent write.
            self.writer_requires_reopen = Some("mutable-publication");
            return Err(Error::RecoveryRequired {
                boundary: "mutable-publication",
            });
        }
        if let Some(started) = apply_started {
            diagnostics
                .as_mut()
                .expect("diagnostic recorder checked")
                .phases
                .memtable_apply = started.finish();
        }
        let bookkeeping_started = diagnostics.is_some().then(PhaseClock::start);
        self.wal_payload_bytes = self.wal_payload_bytes.saturating_add(payload.len());
        self.record_memtable_peak();
        self.inject_post_visibility_failure(failure)?;
        if let Some(started) = bookkeeping_started {
            diagnostics
                .as_mut()
                .expect("diagnostic recorder checked")
                .phases
                .bookkeeping = started.finish();
        }
        if let Some(diagnostics) = diagnostics {
            let sample = diagnostics.finish(
                durability,
                mutation_count,
                payload_bytes,
                &receipt,
                self.maintenance_stats,
                self.memtable.approximate_bytes(),
            );
            self.write_path_diagnostics
                .as_mut()
                .expect("collector remains enabled for one mutable write")
                .record(sample);
        }
        Ok(receipt)
    }

    pub(super) fn ensure_writer_ready(&self) -> Result<()> {
        self.manifests.ensure_publication_ready()?;
        match self.writer_requires_reopen {
            Some(boundary) => Err(Error::RecoveryRequired { boundary }),
            None => Ok(()),
        }
    }

    pub(super) fn prepare_write(
        &mut self,
        batch: &WriteBatch,
        payload_bytes: usize,
        charge: MutableCharge,
        protected: Option<&[Snapshot]>,
    ) -> Result<()> {
        self.reclaim_if_pending()?;
        // A busy writer must not depend on the idle worker winning this lock.
        // Offer one immutable quantum once a full configured source quantum
        // accumulates (or hard capacity is reached). Smaller debt remains idle-
        // eligible. Never flush the mutable tail or loop to drain housekeeping.
        let foreground_threshold =
            (self.compaction.max_input_segments as u64).min(self.compaction_limits.l0_segments);
        let maintenance_offered = self.compaction_debt_segments() > 0
            && self.l0_segment_count() as u64 >= foreground_threshold;
        if maintenance_offered {
            self.offer_write_compaction(protected)?;
        }
        let projected_bytes = self.wal_payload_bytes.saturating_add(payload_bytes);
        let projected_versions = self.memtable.version_count().saturating_add(batch.len());
        let active = self.memtable.version_count() != 0;
        if active
            && (projected_bytes > self.maintenance.wal_payload_max_bytes
                || projected_versions > self.maintenance.memtable_max_versions
                || self
                    .mutable_limits
                    .needs_flush(self.memtable.charge(), charge))
        {
            self.maintenance_stats.write_stalls =
                self.maintenance_stats.write_stalls.saturating_add(1);
            match self.flush_memtable(self.manifest.created_at) {
                Ok(Some(_)) => {
                    self.maintenance_stats.automatic_flushes =
                        self.maintenance_stats.automatic_flushes.saturating_add(1);
                    if !maintenance_offered {
                        self.offer_write_compaction(protected)?;
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    self.maintenance_stats.failed_flushes =
                        self.maintenance_stats.failed_flushes.saturating_add(1);
                    return Err(error);
                }
            }
        }
        let l0_segments = self
            .segments
            .iter()
            .filter(|segment| segment.descriptor.level == 0)
            .count() as u64;
        if l0_segments >= self.compaction_limits.l0_segments {
            // The single offered quantum above has already had its chance.
            // Preserve rejection before WAL append rather than retrying work.
            self.compaction_limits.check(
                crate::MaintenanceResource::L0Segments,
                l0_segments.saturating_add(1),
            )?;
        }
        self.reclaim_if_pending()?;
        self.mutable_limits
            .check(self.memtable.charge().checked_add(charge)?)?;
        if payload_bytes > self.maintenance.wal_payload_max_bytes
            || batch.len() > self.maintenance.memtable_max_versions
        {
            self.maintenance_stats.oversized_batches =
                self.maintenance_stats.oversized_batches.saturating_add(1);
        }
        Ok(())
    }

    pub(super) fn compact_for_write(
        &mut self,
        protected: Option<&[Snapshot]>,
    ) -> Result<Option<CompactionOutcome>> {
        self.compact_segments_inner(
            protected.unwrap_or(&[]),
            self.manifest.created_at,
            None,
            false,
            protected.is_none(),
        )
    }

    pub(super) fn offer_write_compaction(&mut self, protected: Option<&[Snapshot]>) -> Result<()> {
        let result = self.compact_for_write(protected);
        match self.record_automatic_compaction(result) {
            Ok(_) => Ok(()),
            // A denied optional merge leaves debt visible. The hard L0,
            // reclamation and mutable admission still decide if the write fits.
            Err(Error::MaintenanceLimit {
                resource:
                    crate::MaintenanceResource::InputBytes
                    | crate::MaintenanceResource::InputLogicalBytes
                    | crate::MaintenanceResource::InputVersions
                    | crate::MaintenanceResource::OutputSegments
                    | crate::MaintenanceResource::ElapsedMs,
                ..
            }) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub(super) fn record_memtable_peak(&mut self) {
        self.maintenance_stats.peak_accounted_memtable_bytes = self
            .maintenance_stats
            .peak_accounted_memtable_bytes
            .max(self.memtable.approximate_bytes());
        self.maintenance_stats.peak_wal_payload_bytes = self
            .maintenance_stats
            .peak_wal_payload_bytes
            .max(self.wal_payload_bytes);
        self.maintenance_stats.peak_memtable_versions = self
            .maintenance_stats
            .peak_memtable_versions
            .max(self.memtable.version_count());
    }

    pub fn sync(&mut self) -> Result<u64> {
        self.ensure_writer_ready()?;
        self.wal.sync()
    }

    pub(super) fn write_compaction_partition(
        &self,
        versions: BTreeMap<Vec<u8>, Vec<VersionedValue>>,
        durable: u64,
        target_level: u8,
    ) -> Result<Segment> {
        let table = Memtable::from_versions(versions, durable)?;
        let (mut segment, _) = Segment::write_from_memtable_with_cache(
            &self.root.join(SEGMENT_DIRECTORY),
            &table,
            self.segment_row_group_budget,
            self.segment_compression,
            Arc::clone(&self.page_cache),
            Arc::clone(&self.segment_io),
        )?;
        segment.descriptor.level = target_level;
        Ok(segment)
    }
}
