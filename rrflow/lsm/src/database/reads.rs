//! Point, batch and range reads.

use super::*;
use crate::{
    ProjectedReadBudget, ProjectedReadProjection, ProjectedReadRange, ProjectedReadRequest,
};

impl Database {
    pub fn get(&self, key: &[u8], snapshot: Snapshot) -> Result<Option<Vec<u8>>> {
        // The active memtable always contains sequences newer than every
        // published segment. A visible value (including a tombstone) is
        // therefore authoritative for this snapshot and lets hot runtime keys
        // avoid immutable-page I/O entirely.
        if let Some(version) = self.memtable.get_version(key, snapshot.sequence) {
            return Ok(version.value.as_deref().map(<[u8]>::to_vec));
        }
        // Selection loads authenticated key/sequence spines, not value pages.
        // Materialize only the winner, including a winning tombstone.
        match self.select_segment_version(key, snapshot)? {
            Some(version) => version.value(),
            None => Ok(None),
        }
    }

    /// Whether `key` holds a value at `snapshot`. Reads no value data: a flushed
    /// key costs its key spine and one validity page.
    pub fn contains_key(&self, key: &[u8], snapshot: Snapshot) -> Result<bool> {
        if let Some(version) = self.memtable.get_version(key, snapshot.sequence) {
            return Ok(version.value.is_some());
        }
        match self.select_segment_version(key, snapshot)? {
            Some(version) => version.is_present(),
            None => Ok(false),
        }
    }

    /// The length in bytes of the value `key` holds at `snapshot`, or `None`
    /// when it holds none. Reads the value's offsets, never its data.
    pub fn size_of(&self, key: &[u8], snapshot: Snapshot) -> Result<Option<u64>> {
        if let Some(version) = self.memtable.get_version(key, snapshot.sequence) {
            return Ok(version.value.as_ref().map(|value| value.len() as u64));
        }
        match self.select_segment_version(key, snapshot)? {
            Some(version) => version.value_len(),
            None => Ok(None),
        }
    }

    /// Every visible row whose key starts with `prefix`, in key order.
    pub fn scan_prefix(
        &self,
        prefix: &[u8],
        snapshot: Snapshot,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.scan(prefix, prefix_end(prefix).as_deref(), snapshot)
    }

    /// The visible row with the smallest key, or `None` when nothing is visible.
    /// Reads forward from the first key and stops at the first visible row.
    pub fn first_key_value(&self, snapshot: Snapshot) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        let mut stream = self.begin_projected_read(ProjectedReadRequest {
            ranges: vec![ProjectedReadRange::all()],
            projection: ProjectedReadProjection::KeyValue,
            snapshot,
            budget: ProjectedReadBudget {
                max_batch_rows: 1,
                ..ProjectedReadBudget::default()
            },
        })?;
        let Some(batch) = stream.next_batch()? else {
            return Ok(None);
        };
        let key = batch.key(0).ok_or_else(|| {
            Error::InvalidProjectedRead("a non-empty batch has no first row".into())
        })?;
        let value = batch.value(0).ok_or_else(|| {
            Error::InvalidProjectedRead("a key-value batch row has no value".into())
        })?;
        Ok(Some((key.to_vec(), value.to_vec())))
    }

    /// The visible row with the greatest key, or `None` when nothing is visible.
    pub fn last_key_value(&self, snapshot: Snapshot) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        Ok(self.scan_reverse(&[], None, snapshot, 1)?.pop())
    }

    /// At most `limit` visible rows of `[start, end)` from the greatest key
    /// down. Each layer is walked downward from the end of the range, so the
    /// work follows the rows returned, not the size of the range. Only the
    /// winning version of a key has its value read.
    pub fn scan_reverse(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        snapshot: Snapshot,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut rows = Vec::new();
        if limit == 0 || end.is_some_and(|end| start >= end) {
            return Ok(rows);
        }
        let mut cursors = self
            .segments
            .iter()
            .map(|segment| SegmentReverseCursor::new(segment, start, end, snapshot.sequence))
            .collect::<Vec<_>>();
        let mut heads = Vec::with_capacity(cursors.len());
        for cursor in &mut cursors {
            heads.push(cursor.next()?);
        }
        // The memtable is walked by moving its upper bound down past each key.
        let mut memtable_before = end.map(<[u8]>::to_vec);
        let mut memtable_head = self
            .memtable
            .seek_visible_before(start, memtable_before.as_deref(), snapshot.sequence)
            .map(|(key, version)| (key.to_vec(), version.value.as_deref().map(<[u8]>::to_vec)));

        while rows.len() < limit {
            let greatest = heads
                .iter()
                .flatten()
                .map(|(key, _)| key.as_slice())
                .chain(memtable_head.as_ref().map(|(key, _)| key.as_slice()))
                .max()
                .map(<[u8]>::to_vec);
            let Some(key) = greatest else {
                break;
            };
            // The active memtable holds sequences newer than every published
            // segment, so its version of a key wins. Among segments the highest
            // sequence wins, and the first segment wins an equal one, as in `get`.
            let mut winner: Option<SelectedVersion<'_>> = None;
            for (head, cursor) in heads.iter_mut().zip(&mut cursors) {
                if head
                    .as_ref()
                    .is_some_and(|(candidate, _)| *candidate == key)
                {
                    let (_, version) = head.take().expect("head was checked");
                    if winner
                        .as_ref()
                        .is_none_or(|current| version.sequence() > current.sequence())
                    {
                        winner = Some(version);
                    }
                    *head = cursor.next()?;
                }
            }
            let value = match memtable_head.take_if(|(candidate, _)| *candidate == key) {
                Some((_, value)) => {
                    memtable_before = Some(key.clone());
                    memtable_head = self
                        .memtable
                        .seek_visible_before(start, memtable_before.as_deref(), snapshot.sequence)
                        .map(|(key, version)| {
                            (key.to_vec(), version.value.as_deref().map(<[u8]>::to_vec))
                        });
                    value
                }
                None => match winner {
                    Some(version) => version.value()?,
                    None => None,
                },
            };
            if let Some(value) = value {
                rows.push((key, value));
            }
        }
        Ok(rows)
    }

    /// Whether no row is visible at `snapshot`.
    pub fn is_empty(&self, snapshot: Snapshot) -> Result<bool> {
        let mut stream = self.begin_projected_read(ProjectedReadRequest {
            ranges: vec![ProjectedReadRange::all()],
            projection: ProjectedReadProjection::Key,
            snapshot,
            budget: ProjectedReadBudget {
                max_batch_rows: 1,
                ..ProjectedReadBudget::default()
            },
        })?;
        Ok(stream.next_batch()?.is_none())
    }

    /// The newest version of `key` visible at `snapshot` among the flushed segments.
    fn select_segment_version(
        &self,
        key: &[u8],
        snapshot: Snapshot,
    ) -> Result<Option<SelectedVersion<'_>>> {
        let mut best_sequence = 0;
        let mut best_ordinal = usize::MAX;
        let mut selected = None;
        self.point_index.visit(
            |i| &self.segments[i].descriptor,
            key,
            |ordinal| {
                if let Some(version) =
                    self.segments[ordinal].select_version(key, snapshot.sequence)?
                {
                    // Tree traversal is key-ordered, not manifest-ordered. Preserve
                    // the original first-segment winner for equal sequences.
                    if version.sequence() > best_sequence
                        || (version.sequence() == best_sequence && ordinal < best_ordinal)
                    {
                        best_sequence = version.sequence();
                        best_ordinal = ordinal;
                        selected = Some(version);
                    }
                }
                Ok(())
            },
        )?;
        Ok(selected)
    }

    pub fn get_many(&self, keys: &[Vec<u8>], snapshot: Snapshot) -> Result<Vec<Option<Vec<u8>>>> {
        if self.segments.len() <= 1 {
            return keys.iter().map(|key| self.get(key, snapshot)).collect();
        }
        let mut values = vec![None; keys.len()];
        let mut unresolved = Vec::with_capacity(keys.len());
        for (index, key) in keys.iter().enumerate() {
            if let Some(version) = self.memtable.get_version(key, snapshot.sequence) {
                values[index] = version.value.as_deref().map(<[u8]>::to_vec);
            } else {
                unresolved.push(index);
            }
        }
        if unresolved.is_empty() {
            return Ok(values);
        }
        // Visit keys within each segment so a small cache can reuse its spine.
        // Keep only borrowed row identities, never a batch of losing payloads
        // or retained pages. The first segment still wins equal sequences.
        let mut selected: Vec<Option<SelectedVersion<'_>>> = std::iter::repeat_with(|| None)
            .take(unresolved.len())
            .collect();
        for segment in &self.segments {
            for (position, index) in unresolved.iter().enumerate() {
                if let Some(version) = segment.select_version(&keys[*index], snapshot.sequence)? {
                    if selected[position]
                        .as_ref()
                        .is_none_or(|current| version.sequence() > current.sequence())
                    {
                        selected[position] = Some(version);
                    }
                }
            }
        }
        // Preserve payload locality too when requested keys alternate between
        // segments. Caller order and duplicates remain in the result vector.
        for segment in &self.segments {
            for (index, selected) in unresolved.iter().zip(&mut selected) {
                if let Some(version) = selected.take_if(|version| version.belongs_to(segment)) {
                    values[*index] = version.value()?;
                }
            }
        }
        Ok(values)
    }

    pub fn scan(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        snapshot: Snapshot,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        if self.segments.is_empty() {
            return Ok(self.memtable.scan(start, end, snapshot.sequence));
        }
        if self.segments.len() == 1 && self.memtable.version_count() == 0 {
            return self.segments[0].scan(start, end, snapshot.sequence);
        }
        let mut visible = BTreeMap::<Vec<u8>, VersionedValue>::new();
        for segment in &self.segments {
            for (key, version) in segment.visible_from(start, end, snapshot.sequence)? {
                if visible
                    .get(&key)
                    .is_none_or(|current| version.sequence > current.sequence)
                {
                    visible.insert(key, version);
                }
            }
        }
        for (key, version) in self.memtable.visible_from(start, end, snapshot.sequence) {
            if visible
                .get(&key)
                .is_none_or(|current| version.sequence > current.sequence)
            {
                visible.insert(key, version);
            }
        }
        Ok(visible
            .into_iter()
            .filter_map(|(key, version)| version.value.map(|value| (key, value.into_vec())))
            .collect())
    }

    /// Scans sorted, disjoint half-open key ranges at one snapshot. Immutable
    /// segments are visited once so ranges sharing a row group share
    /// one authenticated read and decode.
    pub fn scan_ranges(
        &self,
        ranges: &[(Vec<u8>, Vec<u8>)],
        snapshot: Snapshot,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        if ranges.iter().any(|(start, end)| start >= end)
            || ranges.windows(2).any(|ranges| ranges[0].1 > ranges[1].0)
        {
            return Err(Error::InvalidConfiguration(
                "multi-range scan requires sorted, disjoint, non-empty half-open ranges".into(),
            ));
        }
        if ranges.is_empty() {
            return Ok(Vec::new());
        }
        let mut visible = BTreeMap::<Vec<u8>, VersionedValue>::new();
        for segment in &self.segments {
            for (key, version) in segment.visible_ranges(ranges, snapshot.sequence)? {
                if visible
                    .get(&key)
                    .is_none_or(|current| version.sequence > current.sequence)
                {
                    visible.insert(key, version);
                }
            }
        }
        for (start, end) in ranges {
            for (key, version) in self
                .memtable
                .visible_from(start, Some(end), snapshot.sequence)
            {
                if visible
                    .get(&key)
                    .is_none_or(|current| version.sequence > current.sequence)
                {
                    visible.insert(key, version);
                }
            }
        }
        Ok(visible
            .into_iter()
            .filter_map(|(key, version)| version.value.map(|value| (key, value.into_vec())))
            .collect())
    }

    /// Visits visible rows in key order without requiring callers to retain a
    /// second materialized copy of the result. The mutable-only path borrows
    /// directly from the memtable; mixed immutable/mutable state retains the
    /// canonical merge behavior used by [`Self::scan`].
    pub fn scan_each<E, F>(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        snapshot: Snapshot,
        mut visit: F,
    ) -> std::result::Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&[u8], &[u8]) -> std::result::Result<(), E>,
    {
        if self.segments.is_empty() {
            return self
                .memtable
                .scan_each(start, end, snapshot.sequence, visit);
        }
        for (key, value) in self.scan(start, end, snapshot).map_err(E::from)? {
            visit(&key, &value)?;
        }
        Ok(())
    }
}

/// The smallest key greater than every key that starts with `prefix`, or `None`
/// when no such key exists (an empty prefix, or one of only `0xff` bytes).
pub(crate) fn prefix_end(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut end = prefix.to_vec();
    while let Some(last) = end.pop() {
        if last < u8::MAX {
            end.push(last + 1);
            return Some(end);
        }
    }
    None
}
