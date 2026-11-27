//! Point and prefix operations that answer without reading value data, the
//! first visible row, emptiness, and the transaction's take and update forms.

use rrflow_lsm::{Database, DatabaseOptions, Durability, Mutation, Snapshot, WriteBatch};

fn put(key: &str, value: &str) -> Mutation {
    Mutation::Put {
        key: key.as_bytes().to_vec(),
        value: value.as_bytes().to_vec(),
    }
}

fn delete(key: &str) -> Mutation {
    Mutation::Delete {
        key: key.as_bytes().to_vec(),
    }
}

fn write(database: &mut Database, mutations: Vec<Mutation>) -> Snapshot {
    database
        .write_owned(WriteBatch::new(mutations).unwrap(), Durability::Buffered)
        .unwrap();
    database.snapshot()
}

fn rows(pairs: &[(&str, &str)]) -> Vec<(Vec<u8>, Vec<u8>)> {
    pairs
        .iter()
        .map(|(key, value)| (key.as_bytes().to_vec(), value.as_bytes().to_vec()))
        .collect()
}

/// One history, read from the memtable and again from flushed segments:
/// `a` and `b` are written, then `b` is deleted and `c` is written.
fn history(root: &std::path::Path, flushed: bool) -> (Database, Snapshot, Snapshot) {
    let mut database = Database::create_with_options(root, DatabaseOptions::default()).unwrap();
    let first = write(
        &mut database,
        vec![put("user/a", "alpha"), put("user/b", "bravo-longer")],
    );
    if flushed {
        database.flush_memtable(1).unwrap();
    }
    let second = write(&mut database, vec![delete("user/b"), put("zone/c", "")]);
    if flushed {
        database.flush_memtable(2).unwrap();
    }
    (database, first, second)
}

#[test]
fn contains_and_size_answer_for_values_tombstones_and_absent_keys_at_each_snapshot() {
    for flushed in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let (database, first, second) = history(&directory.path().join("db"), flushed);

        assert!(database.contains_key(b"user/a", first).unwrap());
        assert!(database.contains_key(b"user/b", first).unwrap());
        assert!(!database.contains_key(b"zone/c", first).unwrap());
        assert_eq!(database.size_of(b"user/b", first).unwrap(), Some(12));

        assert!(database.contains_key(b"user/a", second).unwrap());
        assert!(
            !database.contains_key(b"user/b", second).unwrap(),
            "flushed={flushed}: a deleted key is not contained"
        );
        assert_eq!(database.size_of(b"user/a", second).unwrap(), Some(5));
        assert_eq!(database.size_of(b"user/b", second).unwrap(), None);
        // An empty value is a value: contained, with length zero.
        assert!(database.contains_key(b"zone/c", second).unwrap());
        assert_eq!(database.size_of(b"zone/c", second).unwrap(), Some(0));

        assert!(!database.contains_key(b"user/missing", second).unwrap());
        assert_eq!(database.size_of(b"user/missing", second).unwrap(), None);
    }
}

#[test]
fn contains_and_size_read_no_value_data_from_a_flushed_segment() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("cold");
    let (database, _, second) = history(&root, true);
    drop(database);
    // A fresh open starts with an empty page cache, so every page read is a load.
    let database = Database::open(&root).unwrap();
    let loads = |database: &Database| database.page_cache_stats().loads;

    let before = loads(&database);
    assert!(database.contains_key(b"user/a", second).unwrap());
    let after_contains = loads(&database);
    assert_eq!(database.size_of(b"user/a", second).unwrap(), Some(5));
    let after_size = loads(&database);
    assert_eq!(
        database.get(b"user/a", second).unwrap(),
        Some(b"alpha".to_vec())
    );
    let after_get = loads(&database);

    // Key offsets, key data, sequences and validity; then value offsets; then value data.
    assert_eq!(after_contains - before, 4, "contains_key loads");
    assert_eq!(after_size - after_contains, 1, "size_of adds the offsets");
    assert_eq!(after_get - after_size, 1, "get adds the value data");
}

#[test]
fn prefix_scan_first_row_and_emptiness_follow_the_snapshot() {
    for flushed in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let (database, first, second) = history(&directory.path().join("db"), flushed);
        let empty = Snapshot { sequence: 0 };

        assert_eq!(
            database.scan_prefix(b"user/", first).unwrap(),
            rows(&[("user/a", "alpha"), ("user/b", "bravo-longer")])
        );
        assert_eq!(
            database.scan_prefix(b"user/", second).unwrap(),
            rows(&[("user/a", "alpha")])
        );
        assert_eq!(database.scan_prefix(b"zone/", first).unwrap(), rows(&[]));
        assert_eq!(
            database.scan_prefix(b"", second).unwrap(),
            rows(&[("user/a", "alpha"), ("zone/c", "")])
        );

        assert_eq!(
            database.first_key_value(second).unwrap(),
            Some((b"user/a".to_vec(), b"alpha".to_vec()))
        );
        assert_eq!(database.first_key_value(empty).unwrap(), None);
        assert!(database.is_empty(empty).unwrap());
        assert!(!database.is_empty(first).unwrap());
    }
}

#[test]
fn a_prefix_of_only_high_bytes_scans_to_the_end() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        Database::create_with_options(&directory.path().join("db"), DatabaseOptions::default())
            .unwrap();
    let mutations = [vec![0xfe], vec![0xff], vec![0xff, 0x00], vec![0xff, 0xff]]
        .into_iter()
        .map(|key| Mutation::Put {
            key,
            value: b"v".to_vec(),
        })
        .collect();
    let snapshot = write(&mut database, mutations);

    let keys = |prefix: &[u8]| {
        database
            .scan_prefix(prefix, snapshot)
            .unwrap()
            .into_iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        keys(&[0xff]),
        vec![vec![0xff], vec![0xff, 0x00], vec![0xff, 0xff]]
    );
    assert_eq!(keys(&[0xff, 0xff]), vec![vec![0xff, 0xff]]);
    assert_eq!(keys(&[0xfe]), vec![vec![0xfe]]);
}

#[test]
fn a_first_row_hidden_by_a_tombstone_is_skipped() {
    for flushed in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let mut database =
            Database::create_with_options(&directory.path().join("db"), DatabaseOptions::default())
                .unwrap();
        write(&mut database, vec![put("a", "one"), put("b", "two")]);
        if flushed {
            database.flush_memtable(1).unwrap();
        }
        let after_delete = write(&mut database, vec![delete("a")]);
        assert_eq!(
            database.first_key_value(after_delete).unwrap(),
            Some((b"b".to_vec(), b"two".to_vec())),
            "flushed={flushed}"
        );
        let all_deleted = write(&mut database, vec![delete("b")]);
        assert!(database.is_empty(all_deleted).unwrap(), "flushed={flushed}");
        assert_eq!(database.first_key_value(all_deleted).unwrap(), None);
    }
}

#[test]
fn a_transaction_takes_and_updates_through_its_own_writes() {
    let directory = tempfile::tempdir().unwrap();
    let (mut database, _, _) = history(&directory.path().join("db"), true);
    let mut transaction = database.begin_transaction().unwrap();

    assert!(transaction.contains_key(&database, b"user/a").unwrap());
    assert_eq!(transaction.size_of(&database, b"user/a").unwrap(), Some(5));

    // take returns the value and removes the key for this transaction.
    assert_eq!(
        transaction.take(&database, b"user/a").unwrap(),
        Some(b"alpha".to_vec())
    );
    assert!(!transaction.contains_key(&database, b"user/a").unwrap());
    assert_eq!(transaction.size_of(&database, b"user/a").unwrap(), None);
    assert_eq!(transaction.take(&database, b"user/a").unwrap(), None);

    // Taking a key that never held a value writes nothing.
    let writes = transaction.write_count();
    assert_eq!(transaction.take(&database, b"user/never").unwrap(), None);
    assert_eq!(transaction.write_count(), writes);

    // fetch_update returns the previous value; update_fetch the new one.
    let previous = transaction
        .fetch_update(&database, b"counter", |current| {
            assert_eq!(current, None);
            Some(b"1".to_vec())
        })
        .unwrap();
    assert_eq!(previous, None);
    let updated = transaction
        .update_fetch(&database, b"counter", |current| {
            assert_eq!(current, Some(b"1".as_slice()));
            Some(b"2".to_vec())
        })
        .unwrap();
    assert_eq!(updated, Some(b"2".to_vec()));
    assert_eq!(transaction.size_of(&database, b"counter").unwrap(), Some(1));

    // Returning None removes the key; on an absent key it writes nothing.
    assert_eq!(
        transaction
            .fetch_update(&database, b"zone/c", |_| None)
            .unwrap(),
        Some(Vec::new())
    );
    let writes = transaction.write_count();
    assert_eq!(
        transaction
            .update_fetch(&database, b"user/never", |_| None)
            .unwrap(),
        None
    );
    assert_eq!(transaction.write_count(), writes);

    database
        .commit_transaction(transaction, Durability::Buffered)
        .unwrap();
    let committed = database.snapshot();
    assert!(!database.contains_key(b"user/a", committed).unwrap());
    assert!(!database.contains_key(b"zone/c", committed).unwrap());
    assert_eq!(
        database.get(b"counter", committed).unwrap(),
        Some(b"2".to_vec())
    );
}
