use crate::mutable::{version_charge, MutableCharge};
use crate::{Error, Mutation, ProjectedReadRange, RecoveredBatch, Result, WriteBatch};
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included, Unbounded};

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionChain {
    One(VersionedValue),
    Many(Vec<VersionedValue>),
}

impl VersionChain {
    fn from_vec(mut versions: Vec<VersionedValue>) -> Self {
        if versions.len() == 1 {
            Self::One(versions.pop().expect("one checked version"))
        } else {
            Self::Many(versions)
        }
    }

    fn push(&mut self, version: VersionedValue) {
        match self {
            Self::Many(versions) => versions.push(version),
            Self::One(_) => {
                let Self::One(previous) =
                    std::mem::replace(self, Self::Many(Vec::with_capacity(2)))
                else {
                    unreachable!("one-version branch changed during replacement")
                };
                let Self::Many(versions) = self else {
                    unreachable!("replacement created a multi-version chain")
                };
                versions.push(previous);
                versions.push(version);
            }
        }
    }

    fn as_slice(&self) -> &[VersionedValue] {
        match self {
            Self::One(version) => std::slice::from_ref(version),
            Self::Many(versions) => versions,
        }
    }

    fn iter(&self) -> std::slice::Iter<'_, VersionedValue> {
        self.as_slice().iter()
    }

    fn spilled_capacity(&self) -> Option<usize> {
        match self {
            Self::One(_) => None,
            Self::Many(versions) => Some(versions.capacity()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedValue {
    pub sequence: u64,
    /// Exact-length payload storage keeps the value descriptor two machine
    /// words wide instead of carrying `Vec`'s unused capacity word. The full
    /// record is three words on 64-bit targets; WAL/segment encodings are
    /// unchanged.
    pub value: Option<Box<[u8]>>,
}

/// Attributable mutable-memory components. `owned_bytes_lower_bound` excludes
/// B-tree node metadata, allocator headers/fragmentation, and process pages;
/// it is intentionally not presented as an RSS estimate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemtableProfile {
    pub key_count: usize,
    pub version_count: usize,
    pub key_payload_bytes: usize,
    pub value_payload_bytes: usize,
    pub tombstones: usize,
    pub spilled_chains: usize,
    pub spilled_version_capacity: usize,
    pub key_handle_bytes: usize,
    pub version_chain_bytes: usize,
    pub version_record_bytes: usize,
    pub owned_bytes_lower_bound: usize,
}

pub(crate) struct MemtableProjectedSeek {
    pub next: Option<(Vec<u8>, u64)>,
    pub versions_examined: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct ProfileCounters {
    key_payload_bytes: usize,
    value_payload_bytes: usize,
    tombstones: usize,
    spilled_chains: usize,
    spilled_version_capacity: usize,
}

impl ProfileCounters {
    fn add_version(&mut self, version: &VersionedValue) {
        match &version.value {
            Some(value) => {
                self.value_payload_bytes = self.value_payload_bytes.saturating_add(value.len())
            }
            None => self.tombstones = self.tombstones.saturating_add(1),
        }
    }
}

/// Ordered MVCC reference memtable. Versions remain until snapshot-aware
/// flush/compaction proves they are unreachable.
#[derive(Debug, Default)]
pub struct Memtable {
    // AI storage workloads overwhelmingly create one live version per logical
    // key. Keep that version inline in the tree node and spill only actual
    // MVCC history, avoiding one heap allocation for every new key.
    versions: BTreeMap<Box<[u8]>, VersionChain>,
    maximum_sequence: u64,
    version_count: usize,
    approximate_bytes: usize,
    profile_counters: ProfileCounters,
}

// Vec::clone may reduce a spilled chain's capacity. Recompute that physical
// property from the actual clone rather than copying the original count.
impl Clone for Memtable {
    fn clone(&self) -> Self {
        let versions: BTreeMap<_, _> = self.versions.clone();
        let mut profile_counters = self.profile_counters;
        profile_counters.spilled_version_capacity = versions
            .values()
            .filter_map(VersionChain::spilled_capacity)
            .fold(0usize, usize::saturating_add);
        Self {
            versions,
            maximum_sequence: self.maximum_sequence,
            version_count: self.version_count,
            approximate_bytes: self.approximate_bytes,
            profile_counters,
        }
    }
}

// Physical counters are derived, and vector capacity was never part of the
// logical equality of the table.
impl PartialEq for Memtable {
    fn eq(&self, other: &Self) -> bool {
        self.versions == other.versions
            && self.maximum_sequence == other.maximum_sequence
            && self.version_count == other.version_count
            && self.approximate_bytes == other.approximate_bytes
    }
}
impl Eq for Memtable {}

impl Memtable {
    pub fn recover(batches: &[RecoveredBatch]) -> Result<Self> {
        Self::recover_from(batches, 0)
    }

    pub fn recover_from(batches: &[RecoveredBatch], previous_sequence: u64) -> Result<Self> {
        let mut table = Self::at_sequence(previous_sequence);
        for recovered in batches {
            let batch = WriteBatch::decode(&recovered.payload)?;
            table.apply_owned_write_batch(
                batch,
                recovered.first_sequence,
                recovered.last_sequence,
            )?;
        }
        Ok(table)
    }

    pub(crate) fn at_sequence(sequence: u64) -> Self {
        Self {
            maximum_sequence: sequence,
            ..Self::default()
        }
    }

    pub(crate) fn from_versions(
        versions: BTreeMap<Vec<u8>, Vec<VersionedValue>>,
        maximum_sequence: u64,
    ) -> Result<Self> {
        let mut approximate_bytes = 0usize;
        let mut version_count = 0usize;
        let mut profile_counters = ProfileCounters::default();
        for (key, values) in &versions {
            if key.is_empty() || values.is_empty() {
                return Err(Error::InvalidSegment(
                    "compacted memtable contains an empty key/version set".into(),
                ));
            }
            let mut previous = 0;
            profile_counters.key_payload_bytes =
                profile_counters.key_payload_bytes.saturating_add(key.len());
            if values.len() > 1 {
                profile_counters.spilled_chains = profile_counters.spilled_chains.saturating_add(1);
                profile_counters.spilled_version_capacity = profile_counters
                    .spilled_version_capacity
                    .saturating_add(values.capacity());
            }
            for value in values {
                if value.sequence == 0
                    || value.sequence <= previous
                    || value.sequence > maximum_sequence
                {
                    return Err(Error::InvalidSegment(
                        "compacted versions are not strictly ordered".into(),
                    ));
                }
                previous = value.sequence;
                profile_counters.add_version(value);
                approximate_bytes = approximate_bytes
                    .checked_add(version_charge(
                        key.len(),
                        value.value.as_ref().map_or(0, |value| value.len()),
                    )?)
                    .ok_or_else(|| {
                        Error::InvalidSegment("compacted mutable byte accounting overflow".into())
                    })?;
            }
            version_count = version_count
                .checked_add(values.len())
                .ok_or_else(|| Error::InvalidSegment("compacted version count overflow".into()))?;
        }
        Ok(Self {
            versions: versions
                .into_iter()
                .map(|(key, values)| (key.into_boxed_slice(), VersionChain::from_vec(values)))
                .collect(),
            maximum_sequence,
            version_count,
            approximate_bytes,
            profile_counters,
        })
    }

    pub fn apply(&mut self, recovered: &RecoveredBatch) -> Result<()> {
        let batch = WriteBatch::decode(&recovered.payload)?;
        self.apply_write_batch(&batch, recovered.first_sequence, recovered.last_sequence)
    }

    pub(crate) fn apply_owned_recovered(
        &mut self,
        recovered: RecoveredBatch,
        limits: crate::MutableLimits,
    ) -> Result<()> {
        let batch = WriteBatch::decode(&recovered.payload)?;
        limits.check(self.charge().checked_add(MutableCharge::batch(&batch)?)?)?;
        self.apply_owned_write_batch(batch, recovered.first_sequence, recovered.last_sequence)
    }

    pub(crate) fn apply_write_batch(
        &mut self,
        batch: &WriteBatch,
        first_sequence: u64,
        last_sequence: u64,
    ) -> Result<()> {
        let operation_count = u64::try_from(batch.len())
            .map_err(|_| Error::InvalidBatch("operation count exceeds u64".into()))?;
        let expected_last = first_sequence
            .checked_add(operation_count - 1)
            .ok_or_else(|| Error::InvalidBatch("batch sequence range overflow".into()))?;
        if first_sequence != self.maximum_sequence.saturating_add(1)
            || last_sequence != expected_last
        {
            return Err(Error::InvalidBatch(format!(
                "batch sequence range {}..={} does not match {} operation(s) after sequence {}",
                first_sequence,
                last_sequence,
                batch.len(),
                self.maximum_sequence
            )));
        }
        let next_charge = self.charge().checked_add(MutableCharge::batch(batch)?)?;
        for (index, operation) in batch.operations.iter().enumerate() {
            let sequence = first_sequence + index as u64;
            let (key, value) = match operation {
                Mutation::Put { key, value } => {
                    (key.clone(), Some(value.clone().into_boxed_slice()))
                }
                Mutation::Delete { key } => (key.clone(), None),
            };
            self.insert_version(key.into_boxed_slice(), VersionedValue { sequence, value });
        }
        self.maximum_sequence = last_sequence;
        self.version_count = next_charge.versions;
        self.approximate_bytes = next_charge.bytes;
        Ok(())
    }

    pub(crate) fn apply_owned_write_batch(
        &mut self,
        batch: WriteBatch,
        first_sequence: u64,
        last_sequence: u64,
    ) -> Result<()> {
        let operation_count = u64::try_from(batch.len())
            .map_err(|_| Error::InvalidBatch("operation count exceeds u64".into()))?;
        let expected_last = first_sequence
            .checked_add(operation_count - 1)
            .ok_or_else(|| Error::InvalidBatch("batch sequence range overflow".into()))?;
        if first_sequence != self.maximum_sequence.saturating_add(1)
            || last_sequence != expected_last
        {
            return Err(Error::InvalidBatch(format!(
                "batch sequence range {first_sequence}..={last_sequence} does not match {} operation(s) after sequence {}",
                batch.len(), self.maximum_sequence
            )));
        }
        let next_charge = self.charge().checked_add(MutableCharge::batch(&batch)?)?;
        for (index, operation) in batch.operations.into_iter().enumerate() {
            let sequence = first_sequence + index as u64;
            let (key, value) = match operation {
                Mutation::Put { key, value } => (key, Some(value.into_boxed_slice())),
                Mutation::Delete { key } => (key, None),
            };
            self.insert_version(key.into_boxed_slice(), VersionedValue { sequence, value });
        }
        self.maximum_sequence = last_sequence;
        self.version_count = next_charge.versions;
        self.approximate_bytes = next_charge.bytes;
        Ok(())
    }

    pub fn get(&self, key: &[u8], read_sequence: u64) -> Option<&[u8]> {
        self.get_version(key, read_sequence)?.value.as_deref()
    }

    pub fn get_version(&self, key: &[u8], read_sequence: u64) -> Option<&VersionedValue> {
        self.versions
            .get(key)?
            .iter()
            .rev()
            .find(|version| version.sequence <= read_sequence)
    }

    pub fn scan(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        read_sequence: u64,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        let bounds = (Included(start), end.map_or(Unbounded, Excluded));
        self.versions
            .range::<[u8], _>(bounds)
            .filter_map(|(key, versions)| {
                versions
                    .iter()
                    .rev()
                    .find(|version| version.sequence <= read_sequence)?
                    .value
                    .as_deref()
                    .map(|value| (key.to_vec(), value.to_vec()))
            })
            .collect()
    }

    pub(crate) fn scan_each<E, F>(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        read_sequence: u64,
        mut visit: F,
    ) -> std::result::Result<(), E>
    where
        F: FnMut(&[u8], &[u8]) -> std::result::Result<(), E>,
    {
        let bounds = (Included(start), end.map_or(Unbounded, Excluded));
        for (key, versions) in self.versions.range::<[u8], _>(bounds) {
            let Some(value) = versions
                .iter()
                .rev()
                .find(|version| version.sequence <= read_sequence)
                .and_then(|version| version.value.as_deref())
            else {
                continue;
            };
            visit(key, value)?;
        }
        Ok(())
    }

    /// Returns only visible versions inside the requested ordered interval.
    /// Tombstones are retained so callers merging immutable and mutable layers
    /// cannot resurrect an older value.
    pub(crate) fn visible_from(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        read_sequence: u64,
    ) -> Vec<(Vec<u8>, VersionedValue)> {
        let bounds = (Included(start), end.map_or(Unbounded, Excluded));
        self.versions
            .range::<[u8], _>(bounds)
            .filter_map(|(key, versions)| {
                versions
                    .iter()
                    .rev()
                    .find(|version| version.sequence <= read_sequence)
                    .cloned()
                    .map(|version| (key.to_vec(), version))
            })
            .collect()
    }

    /// Seeks to the first key after `after` in the requested disjoint ranges
    /// and returns only its greatest version visible at `read_sequence`.
    /// Repeated calls make monotonic B-tree progress without materializing the
    /// remaining result set.
    pub(crate) fn seek_visible_after(
        &self,
        ranges: &[ProjectedReadRange],
        after: Option<&[u8]>,
        read_sequence: u64,
    ) -> Result<MemtableProjectedSeek> {
        let mut examined = 0u64;
        for range in ranges {
            if after.is_some_and(|after| {
                range
                    .end
                    .as_ref()
                    .is_some_and(|end| end.as_slice() <= after)
            }) {
                continue;
            }
            let lower = match after {
                Some(after) if after >= range.start.as_slice() => Excluded(after),
                _ => Included(range.start.as_slice()),
            };
            let upper = range.end.as_deref().map_or(Unbounded, Excluded);
            for (key, versions) in self.versions.range::<[u8], _>((lower, upper)) {
                let mut visible = None;
                for version in versions.iter().rev() {
                    examined = examined.checked_add(1).ok_or_else(|| {
                        Error::InvalidProjectedRead(
                            "memtable projected version count overflow".into(),
                        )
                    })?;
                    if version.sequence <= read_sequence {
                        visible = Some(version.sequence);
                        break;
                    }
                }
                if let Some(sequence) = visible {
                    return Ok(MemtableProjectedSeek {
                        next: Some((key.to_vec(), sequence)),
                        versions_examined: examined,
                    });
                }
            }
        }
        Ok(MemtableProjectedSeek {
            next: None,
            versions_examined: examined,
        })
    }

    pub fn maximum_sequence(&self) -> u64 {
        self.maximum_sequence
    }

    pub fn key_count(&self) -> usize {
        self.versions.len()
    }

    pub fn version_count(&self) -> usize {
        self.version_count
    }

    pub(crate) fn charge(&self) -> MutableCharge {
        MutableCharge {
            bytes: self.approximate_bytes,
            versions: self.version_count,
        }
    }

    /// Per-version key/value/record charge; not allocator usage or RSS.
    pub fn approximate_bytes(&self) -> usize {
        self.approximate_bytes
    }

    pub fn profile(&self) -> MemtableProfile {
        let mut profile = MemtableProfile {
            key_count: self.key_count(),
            version_count: self.version_count(),
            key_handle_bytes: std::mem::size_of::<Box<[u8]>>(),
            version_chain_bytes: std::mem::size_of::<VersionChain>(),
            version_record_bytes: std::mem::size_of::<VersionedValue>(),
            key_payload_bytes: self.profile_counters.key_payload_bytes,
            value_payload_bytes: self.profile_counters.value_payload_bytes,
            tombstones: self.profile_counters.tombstones,
            spilled_chains: self.profile_counters.spilled_chains,
            spilled_version_capacity: self.profile_counters.spilled_version_capacity,
            ..MemtableProfile::default()
        };
        profile.owned_bytes_lower_bound = profile
            .key_count
            .saturating_mul(
                profile
                    .key_handle_bytes
                    .saturating_add(profile.version_chain_bytes),
            )
            .saturating_add(profile.key_payload_bytes)
            .saturating_add(profile.value_payload_bytes)
            .saturating_add(
                profile
                    .spilled_version_capacity
                    .saturating_mul(profile.version_record_bytes),
            );
        profile
    }

    pub fn all_versions(&self) -> impl Iterator<Item = (&[u8], &[VersionedValue])> + Clone {
        self.versions
            .iter()
            .map(|(key, versions)| (key.as_ref(), versions.as_slice()))
    }

    /// Borrow a half-open immutable-output partition without copying its keys
    /// or version chains. BTreeMap seeks directly to the lower boundary.
    pub(crate) fn range_versions(
        &self,
        first: Option<&[u8]>,
        end: Option<&[u8]>,
    ) -> impl Iterator<Item = (&[u8], &[VersionedValue])> + Clone {
        self.versions
            .range::<[u8], _>((
                first.map_or(Unbounded, Included),
                end.map_or(Unbounded, Excluded),
            ))
            .map(|(key, versions)| (key.as_ref(), versions.as_slice()))
    }

    pub fn visible_versions(&self, read_sequence: u64) -> Vec<(Vec<u8>, VersionedValue)> {
        self.visible_from(&[], None, read_sequence)
    }

    fn insert_version(&mut self, key: Box<[u8]>, version: VersionedValue) {
        self.profile_counters.add_version(&version);
        match self.versions.entry(key) {
            Entry::Vacant(entry) => {
                self.profile_counters.key_payload_bytes = self
                    .profile_counters
                    .key_payload_bytes
                    .saturating_add(entry.key().len());
                entry.insert(VersionChain::One(version));
            }
            Entry::Occupied(mut entry) => {
                let chain = entry.get_mut();
                let before = chain.spilled_capacity();
                chain.push(version);
                let after = chain.spilled_capacity().expect("multiple versions spill");
                if before.is_none() {
                    self.profile_counters.spilled_chains =
                        self.profile_counters.spilled_chains.saturating_add(1);
                }
                self.profile_counters.spilled_version_capacity = self
                    .profile_counters
                    .spilled_version_capacity
                    .saturating_add(after.saturating_sub(before.unwrap_or(0)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Deliberately traverse the physical representation rather than using
    // the maintained counters. This also observes Vec capacity after cloning.
    fn traversed_profile(table: &Memtable) -> MemtableProfile {
        let mut result = MemtableProfile {
            key_count: table.versions.len(),
            version_count: table.versions.values().map(|v| v.as_slice().len()).sum(),
            key_handle_bytes: std::mem::size_of::<Box<[u8]>>(),
            version_chain_bytes: std::mem::size_of::<VersionChain>(),
            version_record_bytes: std::mem::size_of::<VersionedValue>(),
            ..MemtableProfile::default()
        };
        for (key, chain) in &table.versions {
            result.key_payload_bytes = result.key_payload_bytes.saturating_add(key.len());
            if let Some(capacity) = chain.spilled_capacity() {
                result.spilled_chains = result.spilled_chains.saturating_add(1);
                result.spilled_version_capacity =
                    result.spilled_version_capacity.saturating_add(capacity);
            }
            for item in chain.as_slice() {
                if let Some(value) = &item.value {
                    result.value_payload_bytes =
                        result.value_payload_bytes.saturating_add(value.len());
                } else {
                    result.tombstones = result.tombstones.saturating_add(1);
                }
            }
        }
        result.owned_bytes_lower_bound = result
            .key_count
            .saturating_mul(
                result
                    .key_handle_bytes
                    .saturating_add(result.version_chain_bytes),
            )
            .saturating_add(result.key_payload_bytes)
            .saturating_add(result.value_payload_bytes)
            .saturating_add(
                result
                    .spilled_version_capacity
                    .saturating_mul(result.version_record_bytes),
            );
        result
    }

    #[test]
    fn profile_matches_physical_traversal_through_mutation_recovery_and_clone() {
        let mut table = Memtable::default();
        assert_eq!(table.profile(), traversed_profile(&table));
        let mut recovered = Vec::new();
        for sequence in 1..=67u64 {
            let key = format!("key-{}", sequence % 4).into_bytes();
            let mutation = if sequence % 3 == 0 {
                Mutation::Delete { key }
            } else {
                Mutation::Put {
                    key,
                    value: vec![sequence as u8; sequence as usize % 13],
                }
            };
            let batch = WriteBatch::new(vec![mutation]).unwrap();
            recovered.push(RecoveredBatch {
                offset: 0,
                checksum: 0,
                first_sequence: sequence,
                last_sequence: sequence,
                payload: batch.encode().unwrap(),
            });
            if sequence % 2 == 0 {
                table.apply_write_batch(&batch, sequence, sequence).unwrap();
            } else {
                table
                    .apply_owned_write_batch(batch, sequence, sequence)
                    .unwrap();
            }
            assert_eq!(
                table.profile(),
                traversed_profile(&table),
                "sequence {sequence}"
            );
            let cloned = table.clone();
            assert_eq!(
                cloned.profile(),
                traversed_profile(&cloned),
                "clone at sequence {sequence}"
            );
            assert_eq!(cloned, table);
        }
        let replayed = Memtable::recover(&recovered).unwrap();
        assert_eq!(replayed.profile(), traversed_profile(&replayed));
        let compacted = table
            .versions
            .iter()
            .map(|(key, chain)| (key.to_vec(), chain.as_slice().to_vec()))
            .collect();
        let rebuilt = Memtable::from_versions(compacted, 67).unwrap();
        assert_eq!(rebuilt.profile(), traversed_profile(&rebuilt));
        let mut spare = Vec::with_capacity(64);
        spare.push(VersionedValue {
            sequence: 1,
            value: None,
        });
        spare.push(VersionedValue {
            sequence: 2,
            value: Some(vec![7; 3].into_boxed_slice()),
        });
        let rebuilt_with_spare =
            Memtable::from_versions(BTreeMap::from([(b"spare".to_vec(), spare)]), 2).unwrap();
        assert_eq!(
            rebuilt_with_spare.profile(),
            traversed_profile(&rebuilt_with_spare)
        );
        let cloned_spare = rebuilt_with_spare.clone();
        assert_eq!(cloned_spare.profile(), traversed_profile(&cloned_spare));
        assert_eq!(rebuilt_with_spare, cloned_spare);
        let before = table.profile();
        let invalid = WriteBatch::new(vec![Mutation::Delete {
            key: b"bad".to_vec(),
        }])
        .unwrap();
        assert!(table.apply_write_batch(&invalid, 99, 99).is_err());
        assert_eq!(table.profile(), before);
    }

    #[test]
    fn profile_separates_attributable_bytes_from_rss() {
        let mut table = Memtable::default();
        let first = WriteBatch::new(vec![
            Mutation::Put {
                key: b"a".to_vec(),
                value: b"one".to_vec(),
            },
            Mutation::Put {
                key: b"a".to_vec(),
                value: b"two".to_vec(),
            },
            Mutation::Delete { key: b"b".to_vec() },
        ])
        .unwrap();
        table.apply_owned_write_batch(first, 1, 3).unwrap();

        let profile = table.profile();
        assert_eq!(profile.key_count, 2);
        assert_eq!(profile.version_count, 3);
        assert_eq!(profile.key_payload_bytes, 2);
        assert_eq!(profile.value_payload_bytes, 6);
        assert_eq!(profile.tombstones, 1);
        assert_eq!(profile.spilled_chains, 1);
        assert!(profile.spilled_version_capacity >= 2);
        assert!(profile.owned_bytes_lower_bound >= 8);
        #[cfg(target_pointer_width = "64")]
        assert_eq!(profile.version_record_bytes, 24);
    }

    #[test]
    fn exact_length_value_keeps_the_existing_serde_shape() {
        let value = VersionedValue {
            sequence: 7,
            value: Some(vec![1, 2, 3].into_boxed_slice()),
        };
        let encoded = serde_json::to_string(&value).unwrap();
        assert_eq!(encoded, r#"{"sequence":7,"value":[1,2,3]}"#);
        assert_eq!(
            serde_json::from_str::<VersionedValue>(&encoded).unwrap(),
            value
        );
    }
}
