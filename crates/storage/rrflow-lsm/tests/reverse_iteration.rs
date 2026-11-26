//! Reverse iteration and the last key, checked against an independent model
//! across the memtable, flushed segments and compaction.

use rrflow_lsm::{
    Database, DatabaseOptions, Durability, Mutation, SegmentRowGroupBudget, Snapshot, WriteBatch,
};
use std::collections::BTreeMap;

/// A sequence and the value written at it. `None` is a tombstone.
type Version = (u64, Option<Vec<u8>>);

/// Every version of every key, in sequence order.
#[derive(Default)]
struct Model(BTreeMap<Vec<u8>, Vec<Version>>);

impl Model {
    fn apply(&mut self, mutations: &[Mutation], sequence: u64) {
        for mutation in mutations {
            let (key, value) = match mutation {
                Mutation::Put { key, value } => (key.clone(), Some(value.clone())),
                Mutation::Delete { key } => (key.clone(), None),
            };
            self.0.entry(key).or_default().push((sequence, value));
        }
    }

    fn reverse(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        snapshot: Snapshot,
        limit: usize,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.0
            .iter()
            .rev()
            .filter(|(key, _)| {
                key.as_slice() >= start && end.is_none_or(|end| key.as_slice() < end)
            })
            .filter_map(|(key, versions)| {
                versions
                    .iter()
                    .rev()
                    .find(|(sequence, _)| *sequence <= snapshot.sequence)
                    .and_then(|(_, value)| value.clone())
                    .map(|value| (key.clone(), value))
            })
            .take(limit)
            .collect()
    }
}

struct Generator(u64);

impl Generator {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, upper: usize) -> usize {
        (self.next() % upper as u64) as usize
    }
}

fn key(index: usize) -> Vec<u8> {
    format!("k{index:03}").into_bytes()
}

fn check(database: &Database, model: &Model, snapshots: &[Snapshot], label: &str) {
    let bounds: [(Vec<u8>, Option<Vec<u8>>); 6] = [
        (Vec::new(), None),
        (key(10), Some(key(30))),
        (key(0), Some(key(1))),
        (key(25), None),
        (key(39), Some(key(40))),
        (b"zzz".to_vec(), None),
    ];
    for snapshot in snapshots {
        for (start, end) in &bounds {
            for limit in [1, 3, usize::MAX] {
                assert_eq!(
                    database
                        .scan_reverse(start, end.as_deref(), *snapshot, limit)
                        .unwrap(),
                    model.reverse(start, end.as_deref(), *snapshot, limit),
                    "{label}: [{start:?}, {end:?}) at {} limit {limit}",
                    snapshot.sequence
                );
            }
        }
        assert_eq!(
            database.last_key_value(*snapshot).unwrap(),
            model.reverse(&[], None, *snapshot, 1).pop(),
            "{label}: last key at {}",
            snapshot.sequence
        );
        // Reverse iteration is the forward scan read backward.
        let mut forward = database.scan(&[], None, *snapshot).unwrap();
        forward.reverse();
        assert_eq!(
            database
                .scan_reverse(&[], None, *snapshot, usize::MAX)
                .unwrap(),
            forward,
            "{label}: reverse of the forward scan at {}",
            snapshot.sequence
        );
    }
}

#[test]
fn reverse_scans_match_the_model_across_memtable_segments_and_compaction() {
    for (case, seed) in [
        0x9e37_79b9_7f4a_7c15u64,
        0x4d59_5df4_d0f3_3173,
        0xda94_2042_e4dd_58b5,
    ]
    .into_iter()
    .enumerate()
    {
        let directory = tempfile::tempdir().unwrap();
        // Small row groups, so a range crosses many of them.
        let options = DatabaseOptions {
            segment_row_group_budget: SegmentRowGroupBudget {
                max_rows: [3, 7, 64][case],
                target_bytes: 4_096,
            },
            ..DatabaseOptions::default()
        };
        let mut database =
            Database::create_with_options(&directory.path().join("db"), options).unwrap();
        let mut generator = Generator(seed);
        let mut model = Model::default();
        let mut snapshots = vec![Snapshot { sequence: 0 }];

        for round in 0..6 {
            for _ in 0..12 {
                let mutations = (0..1 + generator.below(4))
                    .map(|_| {
                        let key = key(generator.below(40));
                        if generator.below(4) == 0 {
                            Mutation::Delete { key }
                        } else {
                            Mutation::Put {
                                key,
                                value: format!("v{}", generator.next() % 1_000).into_bytes(),
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                // A batch holds one final mutation per key.
                let mut last = BTreeMap::new();
                for mutation in mutations {
                    let key = match &mutation {
                        Mutation::Put { key, .. } | Mutation::Delete { key } => key.clone(),
                    };
                    last.insert(key, mutation);
                }
                let mutations = last.into_values().collect::<Vec<_>>();
                database
                    .write_owned(
                        WriteBatch::new(mutations.clone()).unwrap(),
                        Durability::Buffered,
                    )
                    .unwrap();
                let snapshot = database.snapshot();
                model.apply(&mutations, snapshot.sequence);
                snapshots.push(snapshot);
            }
            check(
                &database,
                &model,
                &snapshots,
                &format!("case {case} round {round} memtable"),
            );
            if round % 2 == 0 {
                database.flush_memtable(round as u64 + 1).unwrap();
                check(
                    &database,
                    &model,
                    &snapshots,
                    &format!("case {case} round {round} flushed"),
                );
            }
        }
        database.flush_memtable(99).unwrap();
        // Every snapshot is protected, so compaction must keep each one readable.
        let mut compactions = 0;
        while database.compact(&snapshots, 100).unwrap().is_some() {
            compactions += 1;
        }
        assert!(compactions > 0, "case {case}: no compaction ran");
        check(
            &database,
            &model,
            &snapshots,
            &format!("case {case} compacted"),
        );
    }
}

#[test]
fn a_transaction_scans_in_reverse_through_its_pending_writes() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        Database::create_with_options(&directory.path().join("db"), DatabaseOptions::default())
            .unwrap();
    let put = |index: usize, value: &str| Mutation::Put {
        key: key(index),
        value: value.as_bytes().to_vec(),
    };
    database
        .write_owned(
            WriteBatch::new((0..10).map(|index| put(index, "stored")).collect()).unwrap(),
            Durability::Buffered,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();

    let mut transaction = database.begin_transaction().unwrap();
    transaction.delete(key(9)).unwrap();
    transaction.delete(key(8)).unwrap();
    transaction.put(key(7), b"pending".to_vec()).unwrap();
    transaction.put(key(20), b"new".to_vec()).unwrap();

    let row = |index: usize, value: &str| (key(index), value.as_bytes().to_vec());
    assert_eq!(
        transaction
            .scan_reverse(&database, &key(0), &key(99), 4)
            .unwrap(),
        vec![
            row(20, "new"),
            row(7, "pending"),
            row(6, "stored"),
            row(5, "stored")
        ]
    );
    // The two hidden rows do not shorten the result.
    assert_eq!(
        transaction
            .scan_reverse(&database, &key(5), &key(10), 10)
            .unwrap(),
        vec![row(7, "pending"), row(6, "stored"), row(5, "stored")]
    );
    assert!(transaction
        .scan_reverse(&database, &key(5), &key(5), 10)
        .unwrap()
        .is_empty());
}
