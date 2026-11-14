//! Compaction candidate selection, merging and reclamation.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CompactionCandidate {
    indices: Vec<usize>,
    source_level: u8,
    target_level: u8,
}

pub(super) struct CompactionMerge {
    segments: Vec<Segment>,
    output_segments: usize,
    input_versions: u64,
    output_versions: u64,
    output_bytes: u64,
    peak_buffer_bytes: usize,
}

pub(super) fn ranges_overlap(
    left_first: &[u8],
    left_last: &[u8],
    right_first: &[u8],
    right_last: &[u8],
) -> bool {
    left_first <= right_last && right_first <= left_last
}

pub(super) fn version_group_bytes(key: &[u8], versions: &[VersionedValue]) -> usize {
    versions.iter().fold(0usize, |bytes, version| {
        bytes
            .saturating_add(key.len())
            .saturating_add(version.value.as_ref().map_or(0, |value| value.len()))
            .saturating_add(std::mem::size_of::<VersionedValue>())
    })
}

pub(super) fn retain_compacted_key(
    retained: &[VersionedValue],
    retain_all_versions: bool,
    shadows_unselected: impl FnOnce() -> bool,
) -> bool {
    // Only a tombstone that would otherwise be discarded needs reachability
    // outside this merge. Live/protected values must survive either way.
    !retained.is_empty()
        && (retain_all_versions
            || retained.iter().any(|version| version.value.is_some())
            || shadows_unselected())
}

pub(super) fn remove_candidates(
    directory: &AnchoredPath,
    candidates: Vec<(AnchoredPath, String, u64)>,
    removed: &mut Vec<String>,
    budget: &mut crate::reclamation::ReclamationBudget,
) -> Result<bool> {
    let result = (|| {
        for (path, file_name, bytes) in candidates {
            if !budget.admit_delete(bytes)? {
                return Ok(true);
            }
            path.remove_file()?;
            removed.push(file_name);
        }
        Ok(false)
    })();
    // Even a partial or failed deletion cycle synchronizes completed removals.
    let sync = if removed.is_empty() {
        Ok(())
    } else {
        directory.sync_directory().map_err(Error::Io)
    };
    match result {
        Err(error) => {
            let _ = sync;
            Err(error)
        }
        Ok(more) => {
            sync?;
            Ok(more)
        }
    }
}

impl Database {
    /// Executes one deterministic leveled compaction step while retaining
    /// exactly the versions observable at `protected` snapshots and at the
    /// current durable head. Input selection is bounded and records are merged
    /// through forward-only segment cursors. Output is partitioned at key
    /// boundaries. Hard input byte/version limits include every version of a
    /// hot key before decoding; the output target alone is not a hard bound.
    ///
    /// Named checkpoints keep their original manifests and segments and are
    /// handled independently by garbage collection.
    pub fn compact(
        &mut self,
        protected: &[Snapshot],
        at: u64,
    ) -> Result<Option<CompactionOutcome>> {
        self.compact_inner(protected, at, None, true, false)
    }

    pub fn compact_with_failure(
        &mut self,
        protected: &[Snapshot],
        at: u64,
        boundary: CompactionBoundary,
        mode: FailureMode,
    ) -> Result<Option<CompactionOutcome>> {
        self.compact_inner(protected, at, Some((boundary, mode)), true, false)
    }

    /// Runs one leveled step only when the configured L0 threshold or a lower
    /// level debt is present. Automatic maintenance conservatively retains all
    /// MVCC versions because a plain [`Snapshot`] is intentionally copyable and
    /// cannot act as a lifetime-tracked reclamation lease. Explicit compaction
    /// supplies the precise protected snapshot set and may prune history.
    pub fn compact_if_needed(&mut self, at: u64) -> Result<Option<CompactionOutcome>> {
        self.compact_inner(&[], at, None, false, true)
    }

    /// One bounded physical housekeeping quantum. No logical write or flush is
    /// introduced: the mutable tail stays in its acknowledged WAL generation.
    /// Reclamation and compaction use the same publication owner and configured
    /// limits as foreground maintenance. A caller decides when to offer work.
    pub fn maintenance_step(&mut self) -> Result<bool> {
        self.maintenance_step_inner(&[], true)
    }

    /// One idle quantum with the same pruning contract as [`Self::compact`].
    /// The owner must supply every untracked snapshot it needs to retain.
    /// Registered transactions are protected automatically; projected views
    /// and named checkpoints retain their original physical generations.
    /// Unlike explicit compaction, this never flushes the mutable tail.
    pub fn maintenance_step_with_protected_snapshots(
        &mut self,
        protected: &[Snapshot],
    ) -> Result<bool> {
        self.maintenance_step_inner(protected, false)
    }

    pub(super) fn maintenance_step_inner(
        &mut self,
        protected: &[Snapshot],
        retain_all_versions: bool,
    ) -> Result<bool> {
        self.ensure_writer_ready()?;
        if self.reclamation_pending {
            let report = self.garbage_collect()?;
            self.reclamation_pending = report.has_more;
            return Ok(true);
        }
        if self.compaction_debt_segments() == 0 {
            return Ok(false);
        }
        let result = self.compact_segments_inner(
            protected,
            self.manifest.created_at,
            None,
            false,
            retain_all_versions,
        );
        self.record_automatic_compaction(result)
    }

    pub(super) fn record_automatic_compaction(
        &mut self,
        result: Result<Option<CompactionOutcome>>,
    ) -> Result<bool> {
        match result {
            Ok(Some(outcome)) => {
                self.maintenance_stats.automatic_compactions = self
                    .maintenance_stats
                    .automatic_compactions
                    .saturating_add(1);
                self.maintenance_stats.compaction_input_bytes = self
                    .maintenance_stats
                    .compaction_input_bytes
                    .saturating_add(outcome.input_bytes);
                self.maintenance_stats.compaction_output_bytes = self
                    .maintenance_stats
                    .compaction_output_bytes
                    .saturating_add(outcome.output_bytes);
                self.maintenance_stats.peak_compaction_buffer_bytes = self
                    .maintenance_stats
                    .peak_compaction_buffer_bytes
                    .max(outcome.peak_buffer_bytes);
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(error) => {
                self.maintenance_stats.failed_compactions =
                    self.maintenance_stats.failed_compactions.saturating_add(1);
                Err(error)
            }
        }
    }

    pub(super) fn compact_inner(
        &mut self,
        protected: &[Snapshot],
        at: u64,
        failure: Option<(CompactionBoundary, FailureMode)>,
        force: bool,
        retain_all_versions: bool,
    ) -> Result<Option<CompactionOutcome>> {
        self.flush_memtable_inner(at, None)?;
        self.compact_segments_inner(protected, at, failure, force, retain_all_versions)
    }

    pub(super) fn compact_segments_inner(
        &mut self,
        protected: &[Snapshot],
        at: u64,
        failure: Option<(CompactionBoundary, FailureMode)>,
        force: bool,
        retain_all_versions: bool,
    ) -> Result<Option<CompactionOutcome>> {
        self.ensure_writer_ready()?;
        let started = Instant::now();
        let Some(candidate) = self.select_compaction_candidate(force, started)? else {
            return Ok(None);
        };
        let input_bytes = self.admit_compaction_input(&candidate, started)?;
        let durable = self.manifest.durable_sequence;
        let mut protected_sequences = protected
            .iter()
            .map(|snapshot| snapshot.sequence)
            .filter(|sequence| *sequence <= durable)
            .collect::<BTreeSet<_>>();
        protected_sequences.extend(self.transaction_snapshots.sequences()?);
        protected_sequences.insert(durable);
        let previous_manifest = self.manifest.digest.clone();
        let input_segments = candidate.indices.len();
        // Partition creation can fail before a replacement manifest exists.
        // Keep cleanup owned even when an idle quantum began without a flush.
        self.reclamation_pending = true;
        let merge = self.merge_compaction_candidate(
            &candidate,
            durable,
            &protected_sequences,
            retain_all_versions,
            started,
        )?;
        let CompactionMerge {
            segments: compacted_segments,
            output_segments,
            input_versions,
            output_versions,
            output_bytes,
            peak_buffer_bytes,
        } = merge;
        self.compaction_limits.check_elapsed(started)?;
        inject_compaction_failure(failure, CompactionBoundary::SegmentSynced)?;
        let selected = candidate.indices.iter().copied().collect::<BTreeSet<_>>();
        let mut descriptors = self
            .segments
            .iter()
            .enumerate()
            .filter(|(index, _)| !selected.contains(index))
            .map(|(_, segment)| segment.descriptor.clone())
            .collect::<Vec<_>>();
        descriptors.extend(
            compacted_segments
                .iter()
                .map(|segment| segment.descriptor.clone()),
        );
        let point_index = PointIndex::new(&descriptors);
        let next = new_manifest(
            self.manifest
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::InvalidManifest("manifest generation overflow".into()))?,
            Some(previous_manifest.clone()),
            at,
            durable,
            self.manifest.wal_start_sequence,
            descriptors,
            self.manifest.application_format,
        )?;
        self.compaction_limits.check_elapsed(started)?;
        self.manifests
            .publish(&next, Some(previous_manifest.as_str()))?;
        let mut retained_segments = std::mem::take(&mut self.segments)
            .into_iter()
            .enumerate()
            .filter_map(|(index, segment)| (!selected.contains(&index)).then_some(segment))
            .collect::<Vec<_>>();
        retained_segments.extend(compacted_segments.into_iter().map(Arc::new));
        self.segments = retained_segments;
        self.point_index = point_index;
        self.manifest = Arc::new(next.clone());
        inject_compaction_failure(failure, CompactionBoundary::ManifestPublished)?;
        Ok(Some(CompactionOutcome {
            previous_manifest,
            manifest: next.digest,
            input_segments,
            output_segments,
            input_versions,
            output_versions,
            protected_sequences: protected_sequences.into_iter().collect(),
            source_level: candidate.source_level,
            target_level: candidate.target_level,
            input_bytes,
            output_bytes,
            peak_buffer_bytes,
            history_pruned: !retain_all_versions,
        }))
    }

    /// Select a complete overlap closure that fits one maintenance quantum.
    /// A blocked upper level must not hide feasible work that clears its target.
    pub(super) fn select_compaction_candidate(
        &self,
        force: bool,
        started: Instant,
    ) -> Result<Option<CompactionCandidate>> {
        let mut denied = None;
        let mut overlap_blocked = false;
        for source_level in 0..self.compaction.max_level {
            self.compaction_limits.check_elapsed(started)?;
            let mut sources = self
                .segments
                .iter()
                .enumerate()
                .filter(|(_, segment)| segment.descriptor.level == source_level)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            let blocked = denied.is_some() || overlap_blocked;
            let threshold = if source_level == 0 {
                self.compaction.l0_compaction_trigger
            } else {
                2
            };
            if sources.is_empty() || (!force && !blocked && sources.len() < threshold) {
                continue;
            }
            sources.sort_by(|left, right| {
                let left = &self.segments[*left].descriptor;
                let right = &self.segments[*right].descriptor;
                if source_level == 0 {
                    left.maximum_sequence
                        .cmp(&right.maximum_sequence)
                        .then_with(|| left.first_key.cmp(&right.first_key))
                        .then_with(|| left.id.cmp(&right.id))
                } else {
                    left.first_key
                        .cmp(&right.first_key)
                        .then_with(|| left.id.cmp(&right.id))
                }
            });
            // Preserve oldest/key-first preference, but a blocked prefix must
            // not hide independent ranges that fit the same admitted quantum.
            // Enumeration remains subject to the existing elapsed bound; every
            // candidate includes its complete target-level overlap closure.
            for start in 0..sources.len() {
                for take in
                    (1..=(sources.len() - start).min(self.compaction.max_input_segments)).rev()
                {
                    self.compaction_limits.check_elapsed(started)?;
                    let Some(candidate) = self.compaction_overlap_candidate(
                        &sources[start..start + take],
                        source_level,
                        started,
                    )?
                    else {
                        overlap_blocked = true;
                        continue;
                    };
                    // Do not disguise a rejected merge of multiple source segments
                    // as a successful move of just one. A sole blocking target may
                    // move down to free its parent; this is finite in max_level.
                    if candidate.indices.len() < 2 && !(sources.len() == 1 && (force || blocked)) {
                        continue;
                    }
                    match self.admit_compaction_input(&candidate, started) {
                        Ok(_) => return Ok(Some(candidate)),
                        Err(
                            error @ Error::MaintenanceLimit {
                                resource:
                                    crate::MaintenanceResource::InputBytes
                                    | crate::MaintenanceResource::InputLogicalBytes
                                    | crate::MaintenanceResource::InputVersions,
                                ..
                            },
                        ) => {
                            if denied.is_none() {
                                denied = Some(error);
                            }
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
        }
        match denied {
            Some(error) => Err(error),
            None => Ok(None),
        }
    }

    pub(super) fn compaction_overlap_candidate(
        &self,
        sources: &[usize],
        source_level: u8,
        started: Instant,
    ) -> Result<Option<CompactionCandidate>> {
        let first_key = sources
            .iter()
            .map(|index| self.segments[*index].descriptor.first_key.as_slice())
            .min()
            .expect("nonempty source selection");
        let last_key = sources
            .iter()
            .map(|index| self.segments[*index].descriptor.last_key.as_slice())
            .max()
            .expect("nonempty source selection");
        let target_level = source_level + 1;
        let mut indices = sources.to_vec();
        for (index, segment) in self.segments.iter().enumerate() {
            self.compaction_limits.check_elapsed(started)?;
            if segment.descriptor.level == target_level
                && ranges_overlap(
                    first_key,
                    last_key,
                    &segment.descriptor.first_key,
                    &segment.descriptor.last_key,
                )
            {
                indices.push(index);
                if indices.len() > self.compaction.max_input_segments {
                    return Ok(None);
                }
            }
        }
        indices.sort_unstable();
        Ok(Some(CompactionCandidate {
            indices,
            source_level,
            target_level,
        }))
    }

    pub(super) fn admit_compaction_input(
        &self,
        candidate: &CompactionCandidate,
        started: Instant,
    ) -> Result<u64> {
        let mut logical_bytes = 0u64;
        let mut versions = 0u64;
        let mut physical_bytes = 0u64;
        for index in &candidate.indices {
            let segment = &self.segments[*index];
            physical_bytes = physical_bytes
                .checked_add(segment.descriptor.bytes)
                .ok_or_else(|| Error::InvalidSegment("compaction physical byte overflow".into()))?;
            logical_bytes = logical_bytes
                .checked_add(segment.compaction_logical_bytes()?)
                .ok_or_else(|| Error::InvalidSegment("compaction logical byte overflow".into()))?;
            versions = versions
                .checked_add(segment.descriptor.entries)
                .ok_or_else(|| Error::InvalidSegment("compaction version count overflow".into()))?;
            self.compaction_limits
                .check(crate::MaintenanceResource::InputBytes, physical_bytes)?;
            self.compaction_limits
                .check(crate::MaintenanceResource::InputLogicalBytes, logical_bytes)?;
            self.compaction_limits
                .check(crate::MaintenanceResource::InputVersions, versions)?;
            self.compaction_limits.check_elapsed(started)?;
        }
        Ok(physical_bytes)
    }

    pub(super) fn merge_compaction_candidate(
        &self,
        candidate: &CompactionCandidate,
        durable: u64,
        protected_sequences: &BTreeSet<u64>,
        retain_all_versions: bool,
        started: Instant,
    ) -> Result<CompactionMerge> {
        let selected = candidate.indices.iter().copied().collect::<BTreeSet<_>>();
        let mut cursors = candidate
            .indices
            .iter()
            .map(|index| self.segments[*index].record_cursor())
            .collect::<Vec<_>>();
        let mut current = cursors
            .iter_mut()
            .map(|cursor| cursor.next_record())
            .collect::<Result<Vec<_>>>()?;
        let mut buffered = BTreeMap::<Vec<u8>, Vec<VersionedValue>>::new();
        let mut buffered_bytes = 0usize;
        let mut peak_buffer_bytes = 0usize;
        let mut compacted_segments = Vec::new();
        let mut input_versions = 0u64;
        let mut output_versions = 0u64;
        let mut output_bytes = 0u64;

        while let Some(key) = current
            .iter()
            .filter_map(|record| record.as_ref().map(|record| record.key.as_slice()))
            .min()
            .map(<[u8]>::to_vec)
        {
            self.compaction_limits.check_elapsed(started)?;
            let mut versions = BTreeMap::<u64, Option<Box<[u8]>>>::new();
            for (cursor, record) in cursors.iter_mut().zip(current.iter_mut()) {
                while record.as_ref().is_some_and(|record| record.key == key) {
                    self.compaction_limits.check_elapsed(started)?;
                    let observed = record.take().expect("record was checked");
                    match versions.get(&observed.version.sequence) {
                        Some(existing) if existing != &observed.version.value => {
                            return Err(Error::InvalidSegment(format!(
                                "segments disagree for sequence {}",
                                observed.version.sequence
                            )));
                        }
                        Some(_) => {}
                        None => {
                            input_versions = input_versions.saturating_add(1);
                            versions.insert(observed.version.sequence, observed.version.value);
                        }
                    }
                    *record = cursor.next_record()?;
                }
            }
            let retained = if retain_all_versions {
                versions
                    .into_iter()
                    .map(|(sequence, value)| VersionedValue { sequence, value })
                    .collect::<Vec<_>>()
            } else {
                let retained_sequences = protected_sequences
                    .iter()
                    .filter_map(|sequence| {
                        versions
                            .range(..=*sequence)
                            .next_back()
                            .map(|(sequence, _)| *sequence)
                    })
                    .collect::<BTreeSet<_>>();
                versions
                    .into_iter()
                    .filter(|(sequence, _)| retained_sequences.contains(sequence))
                    .map(|(sequence, value)| VersionedValue { sequence, value })
                    .collect::<Vec<_>>()
            };
            if !retain_compacted_key(&retained, retain_all_versions, || {
                self.segments.iter().enumerate().any(|(index, segment)| {
                    !selected.contains(&index)
                        && key >= segment.descriptor.first_key
                        && key <= segment.descriptor.last_key
                })
            }) {
                continue;
            }
            let group_bytes = version_group_bytes(&key, &retained);
            if !buffered.is_empty()
                && buffered_bytes.saturating_add(group_bytes) > self.compaction.target_segment_bytes
            {
                self.compaction_limits.check_elapsed(started)?;
                self.compaction_limits.check(
                    crate::MaintenanceResource::OutputSegments,
                    compacted_segments.len() as u64 + 1,
                )?;
                let segment = self.write_compaction_partition(
                    std::mem::take(&mut buffered),
                    durable,
                    candidate.target_level,
                )?;
                output_bytes = output_bytes.saturating_add(segment.descriptor.bytes);
                compacted_segments.push(segment);
                buffered_bytes = 0;
            }
            output_versions = output_versions.saturating_add(retained.len() as u64);
            buffered_bytes = buffered_bytes.saturating_add(group_bytes);
            peak_buffer_bytes = peak_buffer_bytes.max(buffered_bytes);
            buffered.insert(key, retained);
        }
        if !buffered.is_empty() {
            self.compaction_limits.check_elapsed(started)?;
            self.compaction_limits.check(
                crate::MaintenanceResource::OutputSegments,
                compacted_segments.len() as u64 + 1,
            )?;
            let segment =
                self.write_compaction_partition(buffered, durable, candidate.target_level)?;
            output_bytes = output_bytes.saturating_add(segment.descriptor.bytes);
            compacted_segments.push(segment);
        }
        Ok(CompactionMerge {
            output_segments: compacted_segments.len(),
            segments: compacted_segments,
            input_versions,
            output_versions,
            output_bytes,
            peak_buffer_bytes,
        })
    }

    pub(super) fn reclaim_if_pending(&mut self) -> Result<()> {
        if self.reclamation_pending {
            let report = self.garbage_collect()?;
            self.reclamation_pending = report.has_more;
            if report.has_more {
                // Finish finite cleanup across retries before creating more debt.
                return Err(Error::ReclamationPending);
            }
        }
        Ok(())
    }

    pub fn compaction_policy(&self) -> CompactionPolicy {
        self.compaction
    }

    pub fn compaction_debt_segments(&self) -> usize {
        self.l0_segment_count()
            .saturating_sub(self.compaction.l0_compaction_trigger.saturating_sub(1))
    }
}
