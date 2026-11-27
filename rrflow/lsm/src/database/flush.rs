//! Memtable flush, sync and named checkpoints.

use super::*;

impl Database {
    /// Flushes the WAL-backed memtable into key-range immutable partitions,
    /// published together by one manifest and one successor WAL.
    /// Publication order makes either the old or new manifest fully recoverable
    /// at every crash boundary.
    pub fn flush_memtable(&mut self, at: u64) -> Result<Option<Manifest>> {
        self.flush_memtable_inner(at, None)
    }

    /// Deterministic fault-injection entry point for recovery matrices. A
    /// `ManifestPublished` failure updates in-memory state before returning so
    /// callers that do not immediately simulate process death still fail safe.
    pub fn flush_memtable_with_failure(
        &mut self,
        at: u64,
        boundary: FlushBoundary,
        mode: FailureMode,
    ) -> Result<Option<Manifest>> {
        self.flush_memtable_inner(at, Some((boundary, mode)))
    }

    pub(super) fn flush_memtable_inner(
        &mut self,
        at: u64,
        failure: Option<(FlushBoundary, FailureMode)>,
    ) -> Result<Option<Manifest>> {
        self.ensure_writer_ready()?;
        if self.memtable.version_count() == 0 {
            self.wal.sync()?;
            return Ok(None);
        }
        let started = Instant::now();
        let l0_segments = self
            .segments
            .iter()
            .filter(|segment| segment.descriptor.level == 0)
            .count() as u64;
        // Retain only borrowed boundary keys, bounded before each push. The
        // encoder borrows one range at a time; no second mutable table exists.
        let mut boundaries = Vec::new();
        let mut partition_bytes = 0usize;
        for (key, versions) in self.memtable.all_versions() {
            self.compaction_limits.check_elapsed(started)?;
            let group_bytes = versions.iter().try_fold(0usize, |bytes, version| {
                bytes
                    .checked_add(key.len())
                    .and_then(|bytes| {
                        bytes.checked_add(version.value.as_ref().map_or(0, |value| value.len()))
                    })
                    .and_then(|bytes| bytes.checked_add(24))
                    .ok_or_else(|| {
                        Error::InvalidSegment("flush key-group estimate overflow".into())
                    })
            })?;
            if boundaries.is_empty()
                || partition_bytes.saturating_add(group_bytes)
                    > self.compaction.target_segment_bytes
            {
                let outputs = (boundaries.len() as u64).saturating_add(1);
                self.compaction_limits
                    .check(crate::MaintenanceResource::OutputSegments, outputs)?;
                self.compaction_limits.check(
                    crate::MaintenanceResource::L0Segments,
                    l0_segments.saturating_add(outputs),
                )?;
                boundaries.push(key);
                partition_bytes = 0;
            }
            partition_bytes = partition_bytes.saturating_add(group_bytes);
        }
        let sequence = self.wal.sync()?;
        inject_failure(failure, FlushBoundary::WalSynced)?;
        // Failed partial output is unreachable from the old manifest and must
        // be collected even when the caller stays in this process.
        self.reclamation_pending = true;
        let mut flushed = Vec::with_capacity(boundaries.len());
        for (index, first) in boundaries.iter().enumerate() {
            self.compaction_limits.check_elapsed(started)?;
            let (segment, _) = Segment::write_from_versions_with_cache(
                &self.root.join(SEGMENT_DIRECTORY),
                self.memtable
                    .range_versions(Some(first), boundaries.get(index + 1).copied()),
                self.segment_row_group_budget,
                self.segment_compression,
                Arc::clone(&self.page_cache),
                Arc::clone(&self.segment_io),
            )?;
            flushed.push(segment);
        }
        inject_failure(failure, FlushBoundary::SegmentSynced)?;
        let next_sequence = sequence
            .checked_add(1)
            .ok_or_else(|| Error::InvalidBatch("sequence overflow at flush".into()))?;
        let next_path = wal_path(&self.root, next_sequence);
        let successor = if next_path.try_exists()? {
            WalWriter::open_empty_at_anchored(&next_path, next_sequence)?
        } else {
            WalWriter::create_at_anchored(&next_path, next_sequence)?
        };
        inject_failure(failure, FlushBoundary::SuccessorWalSynced)?;
        // The live vector may differ from the sorted manifest after compaction.
        // Build point-index ordinals in exactly the order retained below.
        let mut descriptors = self
            .segments
            .iter()
            .map(|segment| segment.descriptor.clone())
            .collect::<Vec<_>>();
        descriptors.extend(flushed.iter().map(|segment| segment.descriptor.clone()));
        let point_index = PointIndex::new(&descriptors);
        let next = new_manifest(
            self.manifest
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::InvalidManifest("manifest generation overflow".into()))?,
            Some(self.manifest.digest.clone()),
            at,
            sequence,
            next_sequence,
            descriptors,
            self.manifest.application_format,
        )?;
        self.compaction_limits.check_elapsed(started)?;
        self.manifests.publish(&next, Some(&self.manifest.digest))?;
        self.wal = successor;
        self.memtable = self.mutable_memory.empty(sequence);
        self.wal_payload_bytes = 0;
        self.segments.extend(flushed.into_iter().map(Arc::new));
        self.point_index = point_index;
        self.manifest = Arc::new(next.clone());
        self.reclamation_pending = true;
        inject_failure(failure, FlushBoundary::ManifestPublished)?;
        Ok(Some(next))
    }

    pub fn checkpoint(&self, name: &str, at: u64) -> Result<Checkpoint> {
        self.manifests.checkpoint(name, &self.manifest.digest, at)
    }

    pub fn checkpoint_ownership(&self) -> Result<(u64, u64)> {
        self.manifests.checkpoint_ownership()
    }

    pub fn checkpoints(&self) -> Result<Vec<Checkpoint>> {
        self.manifests.checkpoints()
    }

    pub fn release_checkpoint(&self, name: &str) -> Result<bool> {
        self.manifests.release_checkpoint(name)
    }
}
