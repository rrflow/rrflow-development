//! Physical removal: clearing the database and dropping whole segments by key range.
//!
//! Both publish one manifest that no longer lists the removed segments. Neither
//! writes a tombstone, so neither is a versioned delete: a snapshot taken
//! before the call reads the data as gone once the call returns. An open
//! transaction would see that, so both refuse while one is open. A live read
//! stream keeps the segments it already holds, and a named checkpoint keeps
//! the manifest it pinned.

use super::*;

impl Database {
    /// Removes every key. The memtable is discarded, a successor log starts at
    /// the next sequence, and the published manifest lists no segment.
    /// Sequences are not reset: the next write continues from where the
    /// database stood. Either the old or the new manifest is fully recoverable
    /// at every crash boundary, as for a flush.
    pub fn clear(&mut self, at: u64) -> Result<Manifest> {
        self.ensure_writer_ready()?;
        self.ensure_no_active_transactions("clear")?;
        let sequence = self.wal.sync()?;
        let next_sequence = sequence
            .checked_add(1)
            .ok_or_else(|| Error::InvalidBatch("sequence overflow at clear".into()))?;
        let next_path = wal_path(&self.root, next_sequence);
        let successor = if next_path.try_exists()? {
            WalWriter::open_empty_at_anchored(&next_path, next_sequence)?
        } else {
            WalWriter::create_at_anchored(&next_path, next_sequence)?
        };
        let next = new_manifest(
            self.manifest
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::InvalidManifest("manifest generation overflow".into()))?,
            Some(self.manifest.digest.clone()),
            at,
            sequence,
            next_sequence,
            Vec::new(),
            self.manifest.application_format,
        )?;
        self.manifests.publish(&next, Some(&self.manifest.digest))?;
        self.wal = successor;
        self.memtable = self.mutable_memory.empty(sequence);
        self.wal_payload_bytes = 0;
        self.segments.clear();
        self.point_index = PointIndex::new(&[]);
        self.manifest = Arc::new(next.clone());
        self.reclamation_pending = true;
        Ok(next)
    }

    /// Drops every segment whose keys all lie in `[start, end)`, and returns
    /// how many it dropped. `end` of `None` is unbounded above.
    ///
    /// This is a physical drop of whole segments, for removing a key range
    /// that was written together. A segment that only overlaps the range is
    /// kept whole, and the memtable is untouched, so keys of the range can
    /// remain. An older version of a dropped key that lives in a kept segment
    /// becomes visible again. Use deletes where every key must go.
    pub fn drop_range(&mut self, start: &[u8], end: Option<&[u8]>, at: u64) -> Result<usize> {
        self.ensure_writer_ready()?;
        self.ensure_no_active_transactions("drop_range")?;
        if end.is_some_and(|end| start >= end) {
            return Ok(0);
        }
        let contained = |segment: &Segment| {
            segment.descriptor.first_key.as_slice() >= start
                && end.is_none_or(|end| segment.descriptor.last_key.as_slice() < end)
        };
        let dropped = self
            .segments
            .iter()
            .filter(|segment| contained(segment))
            .count();
        if dropped == 0 {
            return Ok(0);
        }
        let descriptors = self
            .segments
            .iter()
            .filter(|segment| !contained(segment))
            .map(|segment| segment.descriptor.clone())
            .collect::<Vec<_>>();
        let point_index = PointIndex::new(&descriptors);
        let next = new_manifest(
            self.manifest
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::InvalidManifest("manifest generation overflow".into()))?,
            Some(self.manifest.digest.clone()),
            at,
            self.manifest.durable_sequence,
            self.manifest.wal_start_sequence,
            descriptors,
            self.manifest.application_format,
        )?;
        self.manifests.publish(&next, Some(&self.manifest.digest))?;
        self.segments.retain(|segment| !contained(segment));
        self.point_index = point_index;
        self.manifest = Arc::new(next);
        self.reclamation_pending = true;
        Ok(dropped)
    }
}
