//! A bounded range read over flushed data examines the rows it returns, not
//! the row group they sit in.

use rrflow_lsm::{
    Database, DatabaseOptions, Durability, Mutation, ProjectedReadBudget, ProjectedReadEvidence,
    ProjectedReadProjection, ProjectedReadRange, ProjectedReadRequest, WriteBatch,
};

const KEYS: usize = 4_000;

fn key(index: usize) -> Vec<u8> {
    format!("k{index:06}").into_bytes()
}

/// `KEYS` keys in flushed segments, with nothing left in the memtable.
fn flushed_database(root: &std::path::Path) -> Database {
    let mut database = Database::create_with_options(root, DatabaseOptions::default()).unwrap();
    for chunk in (0..KEYS).collect::<Vec<_>>().chunks(500) {
        let mutations = chunk
            .iter()
            .map(|index| Mutation::Put {
                key: key(*index),
                value: format!("value-{index:06}").into_bytes(),
            })
            .collect();
        database
            .write_owned(WriteBatch::new(mutations).unwrap(), Durability::Buffered)
            .unwrap();
    }
    database.flush_memtable(1).unwrap();
    database
}

fn read(database: &Database, ranges: Vec<(usize, usize)>) -> (Vec<Vec<u8>>, ProjectedReadEvidence) {
    let ranges = ranges
        .into_iter()
        .map(|(start, end)| ProjectedReadRange::new(key(start), Some(key(end))).unwrap())
        .collect();
    let mut stream = database
        .begin_projected_read(ProjectedReadRequest {
            ranges,
            projection: ProjectedReadProjection::KeyValue,
            snapshot: database.snapshot(),
            budget: ProjectedReadBudget::default(),
        })
        .unwrap();
    let mut keys = Vec::new();
    while let Some(batch) = stream.next_batch().unwrap() {
        for row in 0..batch.len() {
            keys.push(batch.key(row).unwrap().to_vec());
        }
    }
    (keys, stream.evidence().clone())
}

#[test]
fn a_narrow_range_examines_its_own_rows_not_the_row_group() {
    let directory = tempfile::tempdir().unwrap();
    let database = flushed_database(&directory.path().join("narrow"));

    let (keys, evidence) = read(&database, vec![(1_000, 1_016)]);

    assert_eq!(keys, (1_000..1_016).map(key).collect::<Vec<_>>());
    // Sixteen rows returned, and the one row past the end that shows the range is over.
    assert!(
        evidence.versions_examined <= 17,
        "a 16-row range examined {} versions",
        evidence.versions_examined
    );
}

#[test]
fn two_ranges_in_one_row_group_skip_the_rows_between_them() {
    let directory = tempfile::tempdir().unwrap();
    let database = flushed_database(&directory.path().join("gap"));

    let (keys, evidence) = read(&database, vec![(100, 104), (1_500, 1_504)]);

    let expected = (100..104).chain(1_500..1_504).map(key).collect::<Vec<_>>();
    assert_eq!(keys, expected);
    // Eight rows returned, one row past each range.
    assert!(
        evidence.versions_examined <= 10,
        "two 4-row ranges examined {} versions",
        evidence.versions_examined
    );
}

#[test]
fn a_range_ending_at_a_row_group_boundary_does_not_open_the_next_group() {
    let directory = tempfile::tempdir().unwrap();
    let database = flushed_database(&directory.path().join("boundary"));

    let (keys, evidence) = read(&database, vec![(0, 8)]);

    assert_eq!(keys, (0..8).map(key).collect::<Vec<_>>());
    assert_eq!(evidence.selected_row_groups, 1);
    assert!(
        evidence.versions_examined <= 9,
        "an 8-row range at the start examined {} versions",
        evidence.versions_examined
    );
}
