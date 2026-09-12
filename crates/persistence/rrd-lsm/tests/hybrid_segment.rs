use rrd_lsm::{
    CompactionPolicy, Database, DatabaseOptions, Durability, Error, Manifest, ManifestStore,
    Memtable, Mutation, ProjectedReadBudget, ProjectedReadEvidence, ProjectedReadProjection,
    ProjectedReadRange, ProjectedReadRequest, ProjectedReadResource, RecoveredBatch, Segment,
    SegmentIoMode, SegmentIoPolicy, SegmentRowGroupBudget, Snapshot, WriteBatch,
};
use std::collections::BTreeMap;

const CASE_SEEDS: [u64; 4] = [
    0x4d59_5df4_d0f3_3173,
    0x94d0_49bb_1331_11eb,
    0xda94_2042_e4dd_58b5,
    0x9e37_79b9_7f4a_7c15,
];
const GENERATED_OPERATIONS: usize = 96;
const KEYS_PER_FAMILY: usize = 9;
const FAMILIES: [&[u8]; 5] = [b"control", b"edge", b"record", b"term", b"vector"];
type ProjectedRows = Vec<(Vec<u8>, Option<Vec<u8>>)>;

#[derive(Debug)]
struct Generator(u64);

impl Generator {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn index(&mut self, upper: usize) -> usize {
        (self.next_u64() as usize) % upper
    }

    fn value(&mut self, operation: usize) -> Vec<u8> {
        let length = 1 + self.index(384);
        (0..length)
            .map(|offset| {
                (self.next_u64() as u8)
                    ^ (operation as u8).wrapping_mul(17)
                    ^ (offset as u8).rotate_left(3)
            })
            .collect()
    }
}

#[derive(Debug)]
struct ModelVersion {
    sequence: u64,
    value: Option<Vec<u8>>,
}

#[derive(Debug, Default)]
struct Model {
    histories: BTreeMap<Vec<u8>, Vec<ModelVersion>>,
}

impl Model {
    fn apply(&mut self, mutations: &[Mutation], first_sequence: u64) {
        for (offset, mutation) in mutations.iter().enumerate() {
            let sequence = first_sequence + offset as u64;
            let (key, value) = match mutation {
                Mutation::Put { key, value } => (key, Some(value.clone())),
                Mutation::Delete { key } => (key, None),
            };
            self.histories
                .entry(key.clone())
                .or_default()
                .push(ModelVersion { sequence, value });
        }
    }

    fn get(&self, key: &[u8], snapshot: Snapshot) -> Option<Vec<u8>> {
        self.histories.get(key).and_then(|history| {
            history
                .iter()
                .rev()
                .find(|version| version.sequence <= snapshot.sequence)
                .and_then(|version| version.value.clone())
        })
    }

    fn scan(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        snapshot: Snapshot,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.histories
            .iter()
            .filter(|(key, _)| {
                key.as_slice() >= start && end.is_none_or(|end| key.as_slice() < end)
            })
            .filter_map(|(key, _)| self.get(key, snapshot).map(|value| (key.clone(), value)))
            .collect()
    }

    fn scan_ranges(
        &self,
        ranges: &[(Vec<u8>, Vec<u8>)],
        snapshot: Snapshot,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.histories
            .iter()
            .filter(|(key, _)| {
                ranges.iter().any(|(start, end)| {
                    key.as_slice() >= start.as_slice() && key.as_slice() < end.as_slice()
                })
            })
            .filter_map(|(key, _)| self.get(key, snapshot).map(|value| (key.clone(), value)))
            .collect()
    }
}

fn key(family: &[u8], index: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(family.len() + 3);
    key.extend_from_slice(family);
    key.push(b'/');
    key.push(index as u8);
    key.push(match index % 3 {
        0 => 0x00,
        1 => 0xff,
        _ => (index as u8).wrapping_mul(29),
    });
    key
}

fn mutation_keys() -> Vec<Vec<u8>> {
    let mut keys = FAMILIES
        .iter()
        .flat_map(|family| (0..KEYS_PER_FAMILY).map(move |index| key(family, index)))
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

fn verification_keys(mutation_keys: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut keys = mutation_keys.to_vec();
    keys.push(b"absent/before".to_vec());
    keys.push(b"zz-absent/after".to_vec());
    keys.sort();
    keys
}

fn row_group_budget(case: usize) -> SegmentRowGroupBudget {
    const BUDGETS: [SegmentRowGroupBudget; 4] = [
        SegmentRowGroupBudget {
            max_rows: 1,
            target_bytes: 96,
        },
        SegmentRowGroupBudget {
            max_rows: 2,
            target_bytes: 192,
        },
        SegmentRowGroupBudget {
            max_rows: 5,
            target_bytes: 448,
        },
        SegmentRowGroupBudget {
            max_rows: 9,
            target_bytes: 896,
        },
    ];
    BUDGETS[case]
}

fn database_options(case: usize) -> DatabaseOptions {
    DatabaseOptions {
        page_cache_bytes: 16 * 1024,
        segment_io: SegmentIoPolicy {
            mode: SegmentIoMode::Bounded,
            ..SegmentIoPolicy::default()
        },
        segment_row_group_budget: row_group_budget(case),
        compaction: CompactionPolicy {
            l0_compaction_trigger: 2,
            max_input_segments: 64,
            target_segment_bytes: 2 * 1024,
            ..CompactionPolicy::default()
        },
        ..DatabaseOptions::default()
    }
}

fn generated_mutation(generator: &mut Generator, operation: usize, keys: &[Vec<u8>]) -> Mutation {
    let key = keys[generator.index(keys.len())].clone();
    if generator.index(4) == 0 {
        Mutation::Delete { key }
    } else {
        Mutation::Put {
            key,
            value: generator.value(operation),
        }
    }
}

fn verification_ranges() -> Vec<(Vec<u8>, Vec<u8>)> {
    vec![
        (b"control/".to_vec(), b"control0".to_vec()),
        (b"record/".to_vec(), b"record0".to_vec()),
        (b"vector/".to_vec(), b"vector0".to_vec()),
    ]
}

fn verify_snapshot(
    database: &Database,
    model: &Model,
    universe: &[Vec<u8>],
    snapshot: Snapshot,
    context: &str,
) {
    for key in universe {
        assert_eq!(
            database.get(key, snapshot).unwrap(),
            model.get(key, snapshot),
            "{context}: point read differs at snapshot {} for key {key:?}",
            snapshot.sequence
        );
    }

    let mut batched_keys = universe.to_vec();
    batched_keys.extend_from_slice(&universe[..3]);
    let expected_many = batched_keys
        .iter()
        .map(|key| model.get(key, snapshot))
        .collect::<Vec<_>>();
    assert_eq!(
        database.get_many(&batched_keys, snapshot).unwrap(),
        expected_many,
        "{context}: batched point read differs at snapshot {}",
        snapshot.sequence
    );

    assert_eq!(
        database.scan(&[], None, snapshot).unwrap(),
        model.scan(&[], None, snapshot),
        "{context}: full range differs at snapshot {}",
        snapshot.sequence
    );
    assert_eq!(
        database.scan(b"edge/", Some(b"vector0"), snapshot).unwrap(),
        model.scan(b"edge/", Some(b"vector0"), snapshot),
        "{context}: bounded range differs at snapshot {}",
        snapshot.sequence
    );

    let ranges = verification_ranges();
    assert_eq!(
        database.scan_ranges(&ranges, snapshot).unwrap(),
        model.scan_ranges(&ranges, snapshot),
        "{context}: multi-range differs at snapshot {}",
        snapshot.sequence
    );

    verify_projected_snapshot(database, model, snapshot, context);
}

fn verify_projected_snapshot(
    database: &Database,
    model: &Model,
    snapshot: Snapshot,
    context: &str,
) {
    let expected_full = model
        .scan(&[], None, snapshot)
        .into_iter()
        .map(|(key, value)| (key, Some(value)))
        .collect::<Vec<_>>();
    let model_ranges = verification_ranges();
    let projected_ranges = model_ranges
        .iter()
        .map(|(start, end)| ProjectedReadRange::new(start.clone(), Some(end.clone())).unwrap())
        .collect::<Vec<_>>();
    let expected_keys = model
        .scan_ranges(&model_ranges, snapshot)
        .into_iter()
        .map(|(key, _)| (key, None))
        .collect::<Vec<_>>();

    for max_batch_rows in [1, 3] {
        let budget = ProjectedReadBudget {
            max_batch_rows,
            ..ProjectedReadBudget::default()
        };
        let (actual_full, full_evidence) = collect_projected(
            database,
            request(
                vec![ProjectedReadRange::all()],
                ProjectedReadProjection::KeyValue,
                snapshot,
                budget,
            ),
        );
        assert_eq!(
            actual_full, expected_full,
            "{context}: projected key/value stream differs at snapshot {} with batch rows {max_batch_rows}",
            snapshot.sequence
        );
        assert!(full_evidence.peak_batch_rows <= max_batch_rows as u64);

        let (actual_keys, key_evidence) = collect_projected(
            database,
            request(
                projected_ranges.clone(),
                ProjectedReadProjection::Key,
                snapshot,
                budget,
            ),
        );
        assert_eq!(
            actual_keys, expected_keys,
            "{context}: projected key stream differs at snapshot {} with batch rows {max_batch_rows}",
            snapshot.sequence
        );
        assert_eq!(key_evidence.value_offset_page_requests, 0);
        assert_eq!(key_evidence.value_data_page_requests, 0);
    }
}

fn verify_snapshots(
    database: &Database,
    model: &Model,
    universe: &[Vec<u8>],
    snapshots: &[Snapshot],
    context: &str,
) {
    for snapshot in snapshots {
        verify_snapshot(database, model, universe, *snapshot, context);
    }
}

fn collect_projected(
    database: &Database,
    request: ProjectedReadRequest,
) -> (ProjectedRows, ProjectedReadEvidence) {
    let mut stream = database.begin_projected_read(request).unwrap();
    let mut rows = Vec::new();
    while let Some(batch) = stream.next_batch().unwrap() {
        for row in 0..batch.len() {
            rows.push((
                batch.key(row).unwrap().to_vec(),
                batch.value(row).map(<[u8]>::to_vec),
            ));
        }
    }
    assert!(stream.next_batch().unwrap().is_none());
    (rows, stream.evidence().clone())
}

fn request(
    ranges: Vec<ProjectedReadRange>,
    projection: ProjectedReadProjection,
    snapshot: Snapshot,
    budget: ProjectedReadBudget,
) -> ProjectedReadRequest {
    ProjectedReadRequest {
        ranges,
        projection,
        snapshot,
        budget,
    }
}

#[test]
fn generated_mixed_family_histories_match_model_across_reopen_and_compaction() {
    let directory = tempfile::tempdir().unwrap();
    let mutation_keys = mutation_keys();
    let verification_keys = verification_keys(&mutation_keys);

    for (case, seed) in CASE_SEEDS.into_iter().enumerate() {
        let root = directory.path().join(format!("case-{case}"));
        let options = database_options(case);
        let mut database = Database::create_with_options(&root, options).unwrap();
        let mut generator = Generator::new(seed);
        let mut model = Model::default();
        let mut snapshots = vec![Snapshot { sequence: 0 }];
        let mut operation = 0usize;
        let mut batch_index = 0usize;

        while operation < GENERATED_OPERATIONS {
            let batch_size = (1 + generator.index(7)).min(GENERATED_OPERATIONS - operation);
            let mutations = (0..batch_size)
                .map(|offset| {
                    generated_mutation(&mut generator, operation + offset, mutation_keys.as_slice())
                })
                .collect::<Vec<_>>();
            let batch = WriteBatch::new(mutations.clone()).unwrap();
            let receipt = database
                .write_owned(batch, Durability::Authoritative)
                .unwrap();
            model.apply(&mutations, receipt.first_sequence);
            snapshots.push(Snapshot {
                sequence: receipt.last_sequence,
            });
            operation += batch_size;
            batch_index += 1;

            if batch_index.is_multiple_of(3) {
                database.flush_memtable(1_000 + batch_index as u64).unwrap();
            }
        }
        database.flush_memtable(2_000).unwrap();
        for family in FAMILIES {
            assert!(
                model
                    .histories
                    .keys()
                    .any(|candidate| candidate.starts_with(family)),
                "seed {seed:#018x}: generated corpus did not exercise family {family:?}"
            );
        }
        assert!(
            model.histories.values().any(|history| history.len() > 1),
            "seed {seed:#018x}: generated corpus did not update any key"
        );
        assert!(
            model
                .histories
                .values()
                .flatten()
                .any(|version| version.value.is_none()),
            "seed {seed:#018x}: generated corpus did not create a tombstone"
        );
        assert!(
            database.manifest().segments.len() >= 2,
            "seed {seed:#018x}: generated corpus did not create multiple segments"
        );

        let before_reopen = format!(
            "seed {seed:#018x}, operation {operation}, budget {:?}, before reopen",
            options.segment_row_group_budget
        );
        verify_snapshots(
            &database,
            &model,
            &verification_keys,
            &snapshots,
            &before_reopen,
        );
        for descriptor in &database.manifest().segments {
            let segment =
                Segment::open(&root.join("segments").join(format!("{}.seg", descriptor.id)))
                    .unwrap();
            assert_eq!(
                segment.row_group_budget(),
                options.segment_row_group_budget,
                "{before_reopen}: segment {} lost its authenticated budget",
                descriptor.id
            );
        }

        drop(database);
        let mut database = Database::open_with_options(&root, options).unwrap();
        let open_cache = database.page_cache_stats();
        let open_io = database.segment_io_stats();
        assert_eq!(
            open_cache.loads, 0,
            "seed {seed:#018x}: manifest open unexpectedly loaded semantic pages"
        );
        assert_eq!(
            open_io.read_operations, 0,
            "seed {seed:#018x}: manifest open unexpectedly performed page I/O"
        );
        let after_reopen = format!(
            "seed {seed:#018x}, operation {operation}, budget {:?}, after reopen",
            options.segment_row_group_budget
        );
        verify_snapshots(
            &database,
            &model,
            &verification_keys,
            &snapshots,
            &after_reopen,
        );
        assert!(
            database.page_cache_stats().loads > open_cache.loads,
            "{after_reopen}: immutable queries did not load pages"
        );
        assert!(
            database.segment_io_stats().read_operations > open_io.read_operations,
            "{after_reopen}: immutable queries did not report physical I/O"
        );

        let protected = [
            snapshots[1],
            snapshots[snapshots.len() / 2],
            *snapshots.last().unwrap(),
        ];
        let outcome = database.compact(&protected, 3_000).unwrap().unwrap();
        assert!(
            outcome.input_segments >= 2,
            "seed {seed:#018x}: compaction did not consume multiple segments"
        );
        assert!(outcome.history_pruned);
        assert_eq!(
            outcome.protected_sequences,
            protected
                .iter()
                .map(|snapshot| snapshot.sequence)
                .collect::<Vec<_>>()
        );

        let after_compaction = format!(
            "seed {seed:#018x}, operation {operation}, budget {:?}, after compaction",
            options.segment_row_group_budget
        );
        verify_snapshots(
            &database,
            &model,
            &verification_keys,
            &protected,
            &after_compaction,
        );
        for descriptor in &database.manifest().segments {
            let segment =
                Segment::open(&root.join("segments").join(format!("{}.seg", descriptor.id)))
                    .unwrap();
            assert_eq!(segment.row_group_budget(), options.segment_row_group_budget);
        }

        drop(database);
        let database = Database::open_with_options(&root, options).unwrap();
        let after_compaction_reopen = format!(
            "seed {seed:#018x}, operation {operation}, budget {:?}, after compaction reopen",
            options.segment_row_group_budget
        );
        verify_snapshots(
            &database,
            &model,
            &verification_keys,
            &protected,
            &after_compaction_reopen,
        );
    }
}

#[test]
fn projected_stream_matches_model_across_ranges_snapshots_and_projections() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("projected");
    let options = DatabaseOptions {
        segment_io: SegmentIoPolicy {
            mode: SegmentIoMode::Bounded,
            ..SegmentIoPolicy::default()
        },
        segment_row_group_budget: SegmentRowGroupBudget {
            max_rows: 2,
            target_bytes: 128,
        },
        ..DatabaseOptions::default()
    };
    let mut database = Database::create_with_options(&root, options).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![
                Mutation::Put {
                    key: b"a/one".to_vec(),
                    value: b"a1".to_vec(),
                },
                Mutation::Put {
                    key: b"b/one".to_vec(),
                    value: b"b1".to_vec(),
                },
                Mutation::Put {
                    key: b"c/one".to_vec(),
                    value: b"c1".to_vec(),
                },
            ])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let historical = database.snapshot();
    database.flush_memtable(1).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![
                Mutation::Put {
                    key: b"b/one".to_vec(),
                    value: b"b2".to_vec(),
                },
                Mutation::Delete {
                    key: b"c/one".to_vec(),
                },
                Mutation::Put {
                    key: b"d/one".to_vec(),
                    value: b"d2".to_vec(),
                },
            ])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(2).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"e/hot".to_vec(),
                value: b"e3".to_vec(),
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let current = database.snapshot();

    let mut captured_memtable = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            current,
            ProjectedReadBudget {
                max_batch_rows: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![
                Mutation::Put {
                    key: b"e/hot".to_vec(),
                    value: b"e4".to_vec(),
                },
                Mutation::Put {
                    key: b"f/hot".to_vec(),
                    value: b"f4".to_vec(),
                },
            ])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let mut captured_rows = Vec::new();
    while let Some(batch) = captured_memtable.next_batch().unwrap() {
        for row in 0..batch.len() {
            captured_rows.push((
                batch.key(row).unwrap().to_vec(),
                batch.value(row).unwrap().to_vec(),
            ));
        }
    }
    assert_eq!(
        captured_rows,
        vec![
            (b"a/one".to_vec(), b"a1".to_vec()),
            (b"b/one".to_vec(), b"b2".to_vec()),
            (b"d/one".to_vec(), b"d2".to_vec()),
            (b"e/hot".to_vec(), b"e3".to_vec()),
        ]
    );
    let after_write = database.snapshot();
    assert_eq!(
        database.get(b"e/hot", after_write).unwrap(),
        Some(b"e4".to_vec())
    );
    assert_eq!(
        database.get(b"f/hot", after_write).unwrap(),
        Some(b"f4".to_vec())
    );

    let tiny_batches = ProjectedReadBudget {
        max_batch_rows: 2,
        ..ProjectedReadBudget::default()
    };
    let (historical_rows, historical_evidence) = collect_projected(
        &database,
        request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            historical,
            tiny_batches,
        ),
    );
    assert_eq!(
        historical_rows,
        vec![
            (b"a/one".to_vec(), Some(b"a1".to_vec())),
            (b"b/one".to_vec(), Some(b"b1".to_vec())),
            (b"c/one".to_vec(), Some(b"c1".to_vec())),
        ]
    );
    assert_eq!(
        historical_evidence.outcome,
        rrd_lsm::ProjectedReadOutcome::Completed
    );
    assert!(historical_evidence.peak_batch_rows <= 2);

    let ranges = vec![
        ProjectedReadRange::new(b"a/".to_vec(), Some(b"b/".to_vec())).unwrap(),
        ProjectedReadRange::new(b"d/".to_vec(), Some(b"f/".to_vec())).unwrap(),
    ];
    let (keys, key_evidence) = collect_projected(
        &database,
        request(ranges, ProjectedReadProjection::Key, current, tiny_batches),
    );
    assert_eq!(
        keys,
        vec![
            (b"a/one".to_vec(), None),
            (b"d/one".to_vec(), None),
            (b"e/hot".to_vec(), None),
        ]
    );
    assert_eq!(key_evidence.value_offset_page_requests, 0);
    assert_eq!(key_evidence.value_data_page_requests, 0);
    assert!(key_evidence.validity_page_requests > 0);
    assert_eq!(key_evidence.emitted_rows, 3);

    let (empty, _) = collect_projected(
        &database,
        request(
            vec![ProjectedReadRange::new(b"x/".to_vec(), Some(b"y/".to_vec())).unwrap()],
            ProjectedReadProjection::KeyValue,
            current,
            ProjectedReadBudget::default(),
        ),
    );
    assert!(empty.is_empty());
}

#[test]
fn projected_stream_enforces_bounds_and_reports_selected_pages() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("bounded");
    let mut database = Database::create_with_options(
        &root,
        DatabaseOptions {
            segment_row_group_budget: SegmentRowGroupBudget {
                max_rows: 1,
                target_bytes: 64,
            },
            ..DatabaseOptions::default()
        },
    )
    .unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![
                Mutation::Put {
                    key: b"a".to_vec(),
                    value: b"one".to_vec(),
                },
                Mutation::Put {
                    key: b"b".to_vec(),
                    value: b"two".to_vec(),
                },
            ])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"c".to_vec(),
                value: b"three".to_vec(),
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let snapshot = database.snapshot();

    assert!(ProjectedReadRange::new(b"z".to_vec(), Some(b"a".to_vec())).is_err());
    assert!(database
        .begin_projected_read(request(
            Vec::new(),
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget::default(),
        ))
        .is_err());
    assert!(database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_batch_rows: 0,
                ..ProjectedReadBudget::default()
            },
        ))
        .is_err());

    macro_rules! assert_zero_budget_rejected {
        ($field:ident, $resource:expr) => {{
            let mut budget = ProjectedReadBudget::default();
            budget.$field = 0;
            let error = database
                .begin_projected_read(request(
                    vec![ProjectedReadRange::all()],
                    ProjectedReadProjection::Key,
                    snapshot,
                    budget,
                ))
                .unwrap_err();
            assert!(matches!(
                error,
                Error::InvalidProjectedReadBudget { resource }
                    if resource == $resource
            ));
        }};
    }
    assert_zero_budget_rejected!(max_active_views, ProjectedReadResource::ActiveViews);
    assert_zero_budget_rejected!(max_ranges, ProjectedReadResource::Ranges);
    assert_zero_budget_rejected!(max_runs, ProjectedReadResource::Runs);
    assert_zero_budget_rejected!(max_pinned_bytes, ProjectedReadResource::PinnedBytes);
    assert_zero_budget_rejected!(
        max_versions_examined,
        ProjectedReadResource::VersionsExamined
    );
    assert_zero_budget_rejected!(max_page_requests, ProjectedReadResource::PageRequests);
    assert_zero_budget_rejected!(
        max_page_logical_bytes,
        ProjectedReadResource::PageLogicalBytes
    );
    assert_zero_budget_rejected!(max_output_rows, ProjectedReadResource::OutputRows);
    assert_zero_budget_rejected!(
        max_output_buffer_bytes,
        ProjectedReadResource::OutputBufferBytes
    );
    assert_zero_budget_rejected!(max_batch_rows, ProjectedReadResource::BatchRows);
    assert_zero_budget_rejected!(
        max_batch_allocated_bytes,
        ProjectedReadResource::BatchAllocatedBytes
    );

    for ranges in [
        vec![
            ProjectedReadRange::new(b"a".to_vec(), Some(b"c".to_vec())).unwrap(),
            ProjectedReadRange::new(b"b".to_vec(), Some(b"d".to_vec())).unwrap(),
        ],
        vec![
            ProjectedReadRange::new(b"b".to_vec(), Some(b"c".to_vec())).unwrap(),
            ProjectedReadRange::new(b"a".to_vec(), Some(b"b".to_vec())).unwrap(),
        ],
        vec![
            ProjectedReadRange::all(),
            ProjectedReadRange::new(b"z".to_vec(), None).unwrap(),
        ],
    ] {
        assert!(matches!(
            database.begin_projected_read(request(
                ranges,
                ProjectedReadProjection::Key,
                snapshot,
                ProjectedReadBudget::default(),
            )),
            Err(Error::InvalidProjectedRead(_))
        ));
    }
    assert!(matches!(
        database.begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::Key,
            Snapshot {
                sequence: snapshot.sequence + 1,
            },
            ProjectedReadBudget::default(),
        )),
        Err(Error::InvalidProjectedRead(_))
    ));

    let range_limit = database
        .begin_projected_read(request(
            vec![
                ProjectedReadRange::new(b"a".to_vec(), Some(b"b".to_vec())).unwrap(),
                ProjectedReadRange::new(b"b".to_vec(), Some(b"d".to_vec())).unwrap(),
            ],
            ProjectedReadProjection::Key,
            snapshot,
            ProjectedReadBudget {
                max_ranges: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap_err();
    assert!(matches!(
        range_limit,
        Error::ProjectedReadLimit {
            resource: ProjectedReadResource::Ranges,
            ..
        }
    ));

    let run_limit = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::Key,
            snapshot,
            ProjectedReadBudget {
                max_runs: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap_err();
    assert!(matches!(
        run_limit,
        Error::ProjectedReadLimit {
            resource: ProjectedReadResource::Runs,
            ..
        }
    ));

    let held = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget::default(),
        ))
        .unwrap();
    let active_limit = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_active_views: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap_err();
    assert!(matches!(
        active_limit,
        Error::ProjectedReadLimit {
            resource: ProjectedReadResource::ActiveViews,
            ..
        }
    ));
    drop(held);
    drop(
        database
            .begin_projected_read(request(
                vec![ProjectedReadRange::all()],
                ProjectedReadProjection::Key,
                snapshot,
                ProjectedReadBudget {
                    max_active_views: 1,
                    ..ProjectedReadBudget::default()
                },
            ))
            .unwrap(),
    );

    let pinned_limit = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_pinned_bytes: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap_err();
    assert!(matches!(
        pinned_limit,
        Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PinnedBytes,
            ..
        }
    ));

    let mut page_limited = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_page_requests: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    assert!(matches!(
        page_limited.next_batch(),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PageRequests,
            ..
        })
    ));
    assert!(page_limited.next_batch().unwrap().is_none());
    drop(
        database
            .begin_projected_read(request(
                vec![ProjectedReadRange::all()],
                ProjectedReadProjection::Key,
                snapshot,
                ProjectedReadBudget {
                    max_active_views: 1,
                    ..ProjectedReadBudget::default()
                },
            ))
            .unwrap(),
    );

    let mut page_bytes_limited = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::Key,
            snapshot,
            ProjectedReadBudget {
                max_page_logical_bytes: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    assert!(matches!(
        page_bytes_limited.next_batch(),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PageLogicalBytes,
            ..
        })
    ));
    assert_eq!(page_bytes_limited.evidence().cache_loads, 0);

    let mut versions_limited = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::Key,
            snapshot,
            ProjectedReadBudget {
                max_versions_examined: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    assert!(matches!(
        versions_limited.next_batch(),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::VersionsExamined,
            ..
        })
    ));

    let mut batch_bytes_limited = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_batch_allocated_bytes: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    assert!(matches!(
        batch_bytes_limited.next_batch(),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::BatchAllocatedBytes,
            ..
        })
    ));
    assert_eq!(batch_bytes_limited.evidence().peak_batch_allocated_bytes, 0);
    assert_eq!(batch_bytes_limited.evidence().emitted_rows, 0);

    let mut output_bytes_limited = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_output_buffer_bytes: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    assert!(matches!(
        output_bytes_limited.next_batch(),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::OutputBufferBytes,
            ..
        })
    ));
    assert_eq!(
        output_bytes_limited.evidence().peak_batch_allocated_bytes,
        0
    );
    assert_eq!(output_bytes_limited.evidence().emitted_rows, 0);

    let mut row_limited = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_output_rows: 1,
                max_batch_rows: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    assert_eq!(row_limited.next_batch().unwrap().unwrap().len(), 1);
    assert!(matches!(
        row_limited.next_batch(),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::OutputRows,
            ..
        })
    ));

    let mut aligned = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            snapshot,
            ProjectedReadBudget {
                max_batch_rows: 1,
                ..ProjectedReadBudget::default()
            },
        ))
        .unwrap();
    let batch = aligned.next_batch().unwrap().unwrap();
    assert_eq!(batch.key_offsets_buffer().as_ptr() as usize % 64, 0);
    assert_eq!(batch.key_data_buffer().as_ptr() as usize % 64, 0);
    assert_eq!(
        batch.value_offsets_buffer().unwrap().as_ptr() as usize % 64,
        0
    );
    assert_eq!(batch.value_data_buffer().unwrap().as_ptr() as usize % 64, 0);
    aligned.cancel().unwrap();

    let mut cancelled = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::Key,
            snapshot,
            ProjectedReadBudget::default(),
        ))
        .unwrap();
    cancelled.cancel().unwrap();
    assert_eq!(
        cancelled.evidence().outcome,
        rrd_lsm::ProjectedReadOutcome::Cancelled
    );
    assert!(cancelled.next_batch().unwrap().is_none());
    drop(
        database
            .begin_projected_read(request(
                vec![ProjectedReadRange::all()],
                ProjectedReadProjection::Key,
                snapshot,
                ProjectedReadBudget {
                    max_active_views: 1,
                    ..ProjectedReadBudget::default()
                },
            ))
            .unwrap(),
    );
}

#[test]
fn projected_stream_rejects_equal_key_sequence_across_live_runs() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("duplicate-sequence");
    let database = Database::create(&root).unwrap();
    let current = database.manifest().clone();
    drop(database);

    let first_batch = WriteBatch::new(vec![Mutation::Put {
        key: b"shared".to_vec(),
        value: b"one".to_vec(),
    }])
    .unwrap();
    let second_batch = WriteBatch::new(vec![
        Mutation::Put {
            key: b"shared".to_vec(),
            value: b"two".to_vec(),
        },
        Mutation::Put {
            key: b"other".to_vec(),
            value: b"value".to_vec(),
        },
    ])
    .unwrap();
    let first = Memtable::recover(&[RecoveredBatch {
        offset: 0,
        first_sequence: 1,
        last_sequence: 1,
        checksum: 0,
        payload: first_batch.encode().unwrap(),
    }])
    .unwrap();
    let second = Memtable::recover(&[RecoveredBatch {
        offset: 0,
        first_sequence: 1,
        last_sequence: 2,
        checksum: 0,
        payload: second_batch.encode().unwrap(),
    }])
    .unwrap();
    let first_dir = directory.path().join("first");
    let second_dir = directory.path().join("second");
    let (first_segment, first_path) = Segment::write_from_memtable(&first_dir, &first).unwrap();
    let (second_segment, second_path) = Segment::write_from_memtable(&second_dir, &second).unwrap();
    std::fs::copy(
        first_path,
        root.join("segments")
            .join(format!("{}.seg", first_segment.descriptor.id)),
    )
    .unwrap();
    std::fs::copy(
        second_path,
        root.join("segments")
            .join(format!("{}.seg", second_segment.descriptor.id)),
    )
    .unwrap();
    let manifest = Manifest::new(
        2,
        Some(current.digest.clone()),
        1,
        2,
        1,
        vec![first_segment.descriptor, second_segment.descriptor],
    )
    .unwrap();
    let manifests = ManifestStore::open(&root).unwrap();
    manifests.publish(&manifest, Some(&current.digest)).unwrap();
    drop(manifests);

    let database = Database::open(&root).unwrap();
    let mut stream = database
        .begin_projected_read(request(
            vec![ProjectedReadRange::all()],
            ProjectedReadProjection::KeyValue,
            database.snapshot(),
            ProjectedReadBudget::default(),
        ))
        .unwrap();
    assert!(matches!(
        stream.next_batch(),
        Err(Error::DuplicateProjectedSequence { sequence: 1, .. })
    ));
}

#[test]
fn deterministic_malformed_segment_bytes_are_rejected_without_panics() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("source");
    let mut database = Database::create(&root).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record/valid".to_vec(),
                value: b"authenticated payload".to_vec(),
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();
    let segment_id = database.manifest().segments[0].id.clone();
    drop(database);

    let valid_path = root.join("segments").join(format!("{segment_id}.seg"));
    let valid_bytes = std::fs::read(&valid_path).unwrap();
    let malformed_root = directory.path().join("malformed");
    std::fs::create_dir(&malformed_root).unwrap();

    let mut truncation_lengths = vec![
        0,
        1,
        7,
        8,
        9,
        63,
        64,
        255,
        256,
        valid_bytes.len() / 2,
        valid_bytes.len() - 1,
    ];
    truncation_lengths.sort_unstable();
    truncation_lengths.dedup();
    for length in truncation_lengths {
        let path = malformed_root.join(format!("truncated-{length}.seg"));
        std::fs::write(&path, &valid_bytes[..length]).unwrap();
        assert!(
            Segment::open(&path).is_err(),
            "truncated segment unexpectedly opened at length {length}"
        );
    }

    let mut generator = Generator::new(0xc6f0_c6f0_c6f0_c6f0);
    for case in 0..64usize {
        let length = 257 + generator.index(1_792);
        let mut bytes = (0..length)
            .map(|_| generator.next_u64() as u8)
            .collect::<Vec<_>>();
        if case % 3 == 0 {
            bytes[..8].copy_from_slice(b"RRDSEG05");
            bytes[8..10].copy_from_slice(&5u16.to_le_bytes());
        }
        let path = malformed_root.join(format!("generated-{case:02}.seg"));
        std::fs::write(&path, bytes).unwrap();
        assert!(
            Segment::open(&path).is_err(),
            "malformed segment case {case} unexpectedly opened"
        );
    }
}
