//! A downward walk over one segment's keys, for reverse iteration.

use super::*;

/// Yields the keys of one segment inside `[start, end)` from the greatest
/// down, each with the newest version visible at one read sequence. A key
/// whose every version is newer than the read sequence is skipped. Tombstones
/// are yielded, so a caller merging segments cannot resurrect an older value.
///
/// A row group's rows are in key order and a key's versions are contiguous, so
/// the walk needs one key spine at a time and reads no value page.
pub(crate) struct SegmentReverseCursor<'a> {
    segment: &'a Segment,
    start: &'a [u8],
    end: Option<&'a [u8]>,
    read_sequence: u64,
    /// Row groups `[0, remaining_groups)` are still to be walked.
    remaining_groups: usize,
    /// Rows `[0, remaining_rows)` of the loaded row group are still to be walked.
    remaining_rows: usize,
    spine: Option<KeySpine>,
}

impl<'a> SegmentReverseCursor<'a> {
    pub(crate) fn new(
        segment: &'a Segment,
        start: &'a [u8],
        end: Option<&'a [u8]>,
        read_sequence: u64,
    ) -> Self {
        let outside = end.is_some_and(|end| segment.descriptor.first_key.as_slice() >= end)
            || segment.descriptor.last_key.as_slice() < start
            || read_sequence < segment.descriptor.minimum_sequence;
        let remaining_groups = if outside {
            0
        } else {
            // Row groups that start below `end` are the only ones that can hold a key in range.
            segment
                .row_groups
                .partition_point(|group| end.is_none_or(|end| group.first_key.as_slice() < end))
        };
        Self {
            segment,
            start,
            end,
            read_sequence,
            remaining_groups,
            remaining_rows: 0,
            spine: None,
        }
    }

    pub(crate) fn next(&mut self) -> Result<Option<(Vec<u8>, SelectedVersion<'a>)>> {
        loop {
            if self.spine.is_none() {
                let Some(row_group) = self.remaining_groups.checked_sub(1) else {
                    return Ok(None);
                };
                let descriptor = &self.segment.row_groups[row_group];
                if descriptor.last_key.as_slice() < self.start {
                    // Every earlier row group is lower still.
                    self.remaining_groups = 0;
                    return Ok(None);
                }
                if descriptor.page(PageKind::SequenceValues).statistic_min > self.read_sequence {
                    self.remaining_groups = row_group;
                    continue;
                }
                let spine = self.segment.load_key_spine(row_group)?;
                let rows = descriptor.row_count as usize;
                self.remaining_rows = match self.end {
                    Some(end) => lower_bound(&spine, rows, end)?,
                    None => rows,
                };
                self.spine = Some(spine);
            }
            let row_group = self.remaining_groups - 1;
            if self.remaining_rows == 0 {
                self.remaining_groups = row_group;
                self.spine = None;
                continue;
            }
            let spine = self.spine.as_ref().expect("spine was loaded");
            let key = key_at(spine, self.remaining_rows - 1)?.to_vec();
            if key.as_slice() < self.start {
                self.remaining_groups = 0;
                self.spine = None;
                return Ok(None);
            }
            // Walk this key's versions downward and keep the newest one at or
            // below the read sequence, whatever order the versions are stored in.
            let mut selected: Option<(usize, usize, u64)> = None;
            let mut group = row_group;
            let mut row = self.remaining_rows;
            loop {
                let spine = self.spine.as_ref().expect("spine was loaded");
                while row > 0 && key_at(spine, row - 1)? == key.as_slice() {
                    let sequence = sequence_at(spine, row - 1)?;
                    if sequence <= self.read_sequence
                        && selected.is_none_or(|(_, _, best)| sequence > best)
                    {
                        selected = Some((group, row - 1, sequence));
                    }
                    row -= 1;
                }
                // A version chain is written into one row group. Should one ever
                // continue in the group below, follow it there.
                let continues = row == 0
                    && group > 0
                    && self.segment.row_groups[group - 1].last_key.as_slice() == key.as_slice();
                if !continues {
                    break;
                }
                group -= 1;
                self.spine = Some(self.segment.load_key_spine(group)?);
                row = self.segment.row_groups[group].row_count as usize;
            }
            self.remaining_groups = group + 1;
            self.remaining_rows = row;
            if let Some((row_group, row, sequence)) = selected {
                return Ok(Some((
                    key,
                    SelectedVersion {
                        segment: self.segment,
                        row_group,
                        row,
                        sequence,
                    },
                )));
            }
        }
    }
}
