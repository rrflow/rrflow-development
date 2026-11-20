//! Clearing the database and dropping whole segments by key range.

use rrd_lsm::{Database, DatabaseOptions, Durability, Error, Mutation, Snapshot, WriteBatch};

fn key(index: usize) -> Vec<u8> {
    format!("k{index:04}").into_bytes()
}

fn put_range(database: &mut Database, range: std::ops::Range<usize>, value: &str) -> Snapshot {
    let mutations = range
        .map(|index| Mutation::Put {
            key: key(index),
            value: value.as_bytes().to_vec(),
        })
        .collect();
    database
        .write_owned(
            WriteBatch::new(mutations).unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.snapshot()
}

fn create(root: &std::path::Path) -> Database {
    Database::create_with_options(root, DatabaseOptions::default()).unwrap()
}

fn keys(database: &Database) -> Vec<Vec<u8>> {
    database
        .scan(&[], None, database.snapshot())
        .unwrap()
        .into_iter()
        .map(|(key, _)| key)
        .collect()
}

#[test]
fn clear_removes_flushed_and_unflushed_keys_and_keeps_the_sequence() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("db");
    let mut database = create(&root);
    put_range(&mut database, 0..50, "flushed");
    database.flush_memtable(1).unwrap();
    let before = put_range(&mut database, 50..80, "unflushed");
    assert_eq!(keys(&database).len(), 80);

    let manifest = database.clear(2).unwrap();

    assert!(manifest.segments.is_empty());
    assert!(database.is_empty(database.snapshot()).unwrap());
    assert_eq!(database.get(&key(10), database.snapshot()).unwrap(), None);
    assert_eq!(database.get(&key(60), database.snapshot()).unwrap(), None);
    // The sequence continues; a write after the clear is visible and newer.
    assert_eq!(database.snapshot().sequence, before.sequence);
    let after = put_range(&mut database, 5..6, "after");
    assert!(after.sequence > before.sequence);
    assert_eq!(keys(&database), vec![key(5)]);

    // The cleared state, and the write after it, survive a reopen: no log
    // frame from before the clear is replayed.
    drop(database);
    let reopened = Database::open(&root).unwrap();
    assert_eq!(keys(&reopened), vec![key(5)]);
    assert_eq!(reopened.snapshot().sequence, after.sequence);
}

#[test]
fn clear_of_an_empty_database_and_a_second_clear_are_harmless() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("db");
    let mut database = create(&root);
    database.clear(1).unwrap();
    put_range(&mut database, 0..3, "v");
    database.clear(2).unwrap();
    database.clear(3).unwrap();
    assert!(database.is_empty(database.snapshot()).unwrap());
    drop(database);
    assert!(Database::open(&root)
        .unwrap()
        .manifest()
        .segments
        .is_empty());
}

#[test]
fn clear_and_drop_range_refuse_while_a_transaction_is_open() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = create(&directory.path().join("db"));
    put_range(&mut database, 0..10, "v");
    database.flush_memtable(1).unwrap();
    let transaction = database.begin_transaction().unwrap();

    assert!(matches!(
        database.clear(2),
        Err(Error::ActiveTransactions { operation: "clear" })
    ));
    assert!(matches!(
        database.drop_range(&key(0), None, 2),
        Err(Error::ActiveTransactions {
            operation: "drop_range"
        })
    ));
    // Nothing was removed, and the transaction still reads its snapshot.
    assert_eq!(keys(&database).len(), 10);
    assert_eq!(
        transaction.get(&database, &key(3)).unwrap(),
        Some(b"v".to_vec())
    );

    transaction.rollback().unwrap();
    assert_eq!(database.drop_range(&key(0), None, 3).unwrap(), 1);
    assert!(keys(&database).is_empty());
}

#[test]
fn drop_range_drops_only_segments_wholly_inside_the_range() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("db");
    let mut database = create(&root);
    // Three flushed segments over disjoint key ranges, and unflushed keys.
    for (round, range) in [0..100, 100..200, 200..300].into_iter().enumerate() {
        put_range(&mut database, range, "flushed");
        database.flush_memtable(round as u64 + 1).unwrap();
    }
    put_range(&mut database, 150..160, "memtable");
    assert_eq!(database.manifest().segments.len(), 3);

    // [50, 250) wholly contains only the middle segment.
    assert_eq!(
        database.drop_range(&key(50), Some(&key(250)), 9).unwrap(),
        1
    );
    assert_eq!(database.manifest().segments.len(), 2);

    let remaining = keys(&database);
    // The overlapping segments are kept whole, and the memtable is untouched.
    let expected = (0..100)
        .chain(150..160)
        .chain(200..300)
        .map(key)
        .collect::<Vec<_>>();
    assert_eq!(remaining, expected);
    assert_eq!(
        database.get(&key(155), database.snapshot()).unwrap(),
        Some(b"memtable".to_vec())
    );
    assert_eq!(database.get(&key(120), database.snapshot()).unwrap(), None);

    // A range that contains no whole segment, and an empty range, drop nothing.
    assert_eq!(
        database.drop_range(&key(10), Some(&key(20)), 10).unwrap(),
        0
    );
    assert_eq!(
        database.drop_range(&key(20), Some(&key(20)), 10).unwrap(),
        0
    );
    assert_eq!(database.manifest().segments.len(), 2);

    // The drop survives a reopen.
    drop(database);
    let reopened = Database::open(&root).unwrap();
    assert_eq!(keys(&reopened), expected);
}

#[test]
fn drop_range_brings_back_an_older_version_kept_in_another_segment() {
    // The documented cost of a physical drop: it is not a versioned delete.
    let directory = tempfile::tempdir().unwrap();
    let mut database = create(&directory.path().join("db"));
    put_range(&mut database, 0..300, "old");
    database.flush_memtable(1).unwrap();
    put_range(&mut database, 100..110, "new");
    database.flush_memtable(2).unwrap();
    assert_eq!(
        database.get(&key(105), database.snapshot()).unwrap(),
        Some(b"new".to_vec())
    );

    assert_eq!(
        database.drop_range(&key(100), Some(&key(110)), 3).unwrap(),
        1
    );

    assert_eq!(
        database.get(&key(105), database.snapshot()).unwrap(),
        Some(b"old".to_vec())
    );
}

fn segment_files(root: &std::path::Path) -> usize {
    std::fs::read_dir(root.join("segments")).unwrap().count()
}

#[test]
fn cleared_segments_are_reclaimed_unless_a_checkpoint_pins_them() {
    for pinned in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("db");
        let mut database = create(&root);
        put_range(&mut database, 0..20, "v");
        database.flush_memtable(1).unwrap();
        assert_eq!(segment_files(&root), 1);
        let before = database.manifest().digest.clone();
        if pinned {
            let checkpoint = database.checkpoint("before-clear", 2).unwrap();
            assert_eq!(checkpoint.manifest, before);
        }

        database.clear(3).unwrap();
        database.garbage_collect().unwrap();

        assert_ne!(database.manifest().digest, before);
        assert!(database.is_empty(database.snapshot()).unwrap());
        assert_eq!(
            segment_files(&root),
            usize::from(pinned),
            "pinned={pinned}: segment files left after clear and collection"
        );
    }
}
