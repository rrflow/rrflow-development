use rrd_lsm::{
    Database, Durability, Error, Mutation, ProjectedReadBudget, ProjectedReadProjection,
    ProjectedReadRange, ProjectedReadRequest, ProjectedReadResource, ProjectedReadStream,
    WriteBatch,
};

fn view(database: &Database) -> ProjectedReadStream {
    database.begin_projected_read(request(database)).unwrap()
}

fn request(database: &Database) -> ProjectedReadRequest {
    ProjectedReadRequest {
        ranges: vec![ProjectedReadRange::all()],
        projection: ProjectedReadProjection::KeyValue,
        snapshot: database.snapshot(),
        budget: ProjectedReadBudget {
            max_active_views: 2,
            ..ProjectedReadBudget::default()
        },
    }
}

fn empty_views(release: &str) {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::create(&directory.path().join("empty")).unwrap();
    let mut held = view(&database);
    for _ in 0..32 {
        let mut transient = view(&database);
        assert!(matches!(
            database.begin_projected_read(request(&database)),
            Err(Error::ProjectedReadLimit {
                resource: ProjectedReadResource::ActiveViews,
                ..
            })
        ));
        match release {
            "complete" => assert!(transient.next_batch().unwrap().is_none()),
            "cancel" => transient.cancel().unwrap(),
            "drop" => {}
            _ => unreachable!(),
        }
        drop(transient);
        // Admission must be reusable while the other zero-byte view is live.
        let mut replacement = view(&database);
        replacement.cancel().unwrap();
    }
    assert!(held.next_batch().unwrap().is_none());
    drop(view(&database));
}

#[test]
fn overlapping_empty_views_complete_and_release_capacity() {
    empty_views("complete");
}

#[test]
fn overlapping_empty_views_cancel_and_release_capacity() {
    empty_views("cancel");
}

#[test]
fn overlapping_empty_views_drop_and_release_capacity() {
    empty_views("drop");
}

#[test]
fn nonempty_view_can_release_while_an_empty_generation_remains_live() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("mixed");
    let mut database = Database::create(&root).unwrap();
    let empty_generation = database.memtable() as *const _ as usize;
    let mut empty = view(&database);
    let manifest = database.manifest().digest.clone();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record".to_vec(),
                value: b"durable".to_vec(),
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    assert_eq!(database.manifest().digest, manifest);
    assert_eq!(
        database.memtable() as *const _ as usize,
        empty_generation,
        "an empty read must not force copy-on-write"
    );
    let mut populated = view(&database);
    assert!(populated.evidence().pinned_bytes > 0);
    let batch = populated.next_batch().unwrap().unwrap();
    assert_eq!(batch.value(0), Some(b"durable".as_slice()));
    populated.cancel().unwrap();
    assert!(empty.next_batch().unwrap().is_none());
    drop(view(&database));
    drop(database);
    let reopened = Database::open(&root).unwrap();
    let mut restored = view(&reopened);
    let restored_batch = restored.next_batch().unwrap().unwrap();
    assert_eq!(restored_batch.key(0), Some(b"record".as_slice()));
    assert_eq!(restored_batch.value(0), Some(b"durable".as_slice()));
    assert!(restored.next_batch().unwrap().is_none());
    assert_eq!(batch.value(0), Some(b"durable".as_slice()));
}

#[test]
fn database_ceilings_cannot_be_raised_by_a_later_read_request() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("owner");
    let options = rrd_lsm::DatabaseOptions {
        active_read_views: 1,
        ..Default::default()
    };
    let database = Database::create_with_options(&root, options).unwrap();
    let held = view(&database);
    let mut raised = request(&database);
    raised.budget.max_active_views = usize::MAX;
    assert!(matches!(
        database.begin_projected_read(raised),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::ActiveViews,
            limit: 1,
            observed: 2
        })
    ));
    assert_eq!(database.read_view_ownership().unwrap(), (1, 0));
    drop(held);
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
    drop(view(&database));
    drop(database);
    let reopened = Database::open_with_options(&root, options).unwrap();
    let held = view(&reopened);
    assert!(reopened.begin_projected_read(request(&reopened)).is_err());
    drop(held);
}

#[test]
fn read_view_and_independent_batch_charges_release_with_their_actual_owners() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("pins");
    let mut database = Database::create(&root).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record".to_vec(),
                value: vec![7; 4096],
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let mut stream = view(&database);
    let pinned = stream.evidence().pinned_bytes;
    assert!(pinned > 1);
    let batch = stream.next_batch().unwrap().unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (1, pinned));
    drop(stream);
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
    assert!(database.projected_output_memory().stats().reserved_bytes > 0);
    assert_eq!(batch.value(0).unwrap(), vec![7; 4096]);
    drop(batch);
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
    assert_eq!(database.projected_output_memory().stats().reserved_bytes, 0);
    drop(database);
    let database = Database::open_with_options(
        &root,
        rrd_lsm::DatabaseOptions {
            pinned_read_bytes: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let mut raised = request(&database);
    raised.budget.max_pinned_bytes = u64::MAX;
    assert!(matches!(
        database.begin_projected_read(raised),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PinnedBytes,
            limit: 1,
            ..
        })
    ));
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
}

#[test]
fn shared_generation_is_charged_once_until_its_last_view_releases() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::create(&directory.path().join("shared-pins")).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record".to_vec(),
                value: vec![7; 4096],
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();
    let first = view(&database);
    let pinned = first.evidence().pinned_bytes;
    assert!(pinned > 0);
    let mut bounded = request(&database);
    bounded.budget.max_pinned_bytes = pinned;
    let mut second = database.begin_projected_read(bounded).unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (2, pinned));
    assert_eq!(
        second.next_batch().unwrap().unwrap().value(0),
        Some(vec![7; 4096].as_slice())
    );
    second.cancel().unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (1, pinned));
    drop(first);
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
}

#[test]
fn shared_mutable_generation_is_charged_once_and_survives_owner_release() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::create(&directory.path().join("shared-mutable")).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record".to_vec(),
                value: vec![7; 4096],
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let mut first = view(&database);
    let pinned = first.evidence().pinned_bytes;
    assert!(pinned > 0);
    let mut bounded = request(&database);
    bounded.budget.max_pinned_bytes = pinned;
    let mut second = database.begin_projected_read(bounded).unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (2, pinned));
    assert_eq!(
        second.evidence().pinned_bytes,
        pinned,
        "each stream retains its own logical footprint evidence"
    );
    assert_eq!(
        first.next_batch().unwrap().unwrap().value(0),
        Some(vec![7; 4096].as_slice())
    );
    assert_eq!(
        second.next_batch().unwrap().unwrap().value(0),
        Some(vec![7; 4096].as_slice())
    );
    first.cancel().unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (1, pinned));
    drop(second);
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
}

#[test]
fn concurrent_views_share_one_mutable_charge_without_losing_values() {
    use std::sync::mpsc;
    use std::time::Duration;
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::create(&directory.path().join("concurrent-mutable")).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record".to_vec(),
                value: vec![9; 4096],
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let first = view(&database);
    let pinned = first.evidence().pinned_bytes;
    drop(first);
    std::thread::scope(|scope| {
        let (ready_tx, ready_rx) = mpsc::channel();
        let mut releases = Vec::new();
        let mut threads = Vec::new();
        for _ in 0..4 {
            let (release_tx, release_rx) = mpsc::channel();
            releases.push(release_tx);
            let ready = ready_tx.clone();
            let db = &database;
            threads.push(scope.spawn(move || {
                let mut candidate = request(db);
                candidate.budget.max_active_views = 4;
                candidate.budget.max_pinned_bytes = pinned;
                match db.begin_projected_read(candidate) {
                    Ok(mut stream) => {
                        ready.send(true).unwrap();
                        release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
                        assert_eq!(
                            stream.next_batch().unwrap().unwrap().value(0),
                            Some(vec![9; 4096].as_slice())
                        );
                        stream.cancel().unwrap();
                    }
                    Err(_) => {
                        ready.send(false).unwrap();
                    }
                }
            }));
        }
        drop(ready_tx);
        let admitted = (0..4)
            .filter(|_| ready_rx.recv_timeout(Duration::from_secs(10)).unwrap())
            .count();
        let ownership = database.read_view_ownership().unwrap();
        for release in releases {
            let _ = release.send(());
        }
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(admitted, 4);
        assert_eq!(ownership, (4, pinned));
    });
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
}

#[test]
fn copy_on_write_mutable_generations_are_independently_charged() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::create(&directory.path().join("cow-mutable")).unwrap();
    let write = |database: &mut Database, byte| {
        database
            .write_owned(
                WriteBatch::new(vec![Mutation::Put {
                    key: b"record".to_vec(),
                    value: vec![byte; 4096],
                }])
                .unwrap(),
                Durability::Authoritative,
            )
            .unwrap();
    };
    let memory = database.mutable_memory();
    write(&mut database, 1);
    let mut old = view(&database);
    let old_pinned = old.evidence().pinned_bytes;
    write(&mut database, 2);
    assert_eq!(memory.stats().generations, 2);
    let mut denied = request(&database);
    denied.budget.max_pinned_bytes = old_pinned;
    assert!(matches!(
        database.begin_projected_read(denied),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PinnedBytes,
            ..
        })
    ));
    assert_eq!(database.read_view_ownership().unwrap(), (1, old_pinned));
    let mut new = view(&database);
    let new_pinned = new.evidence().pinned_bytes;
    assert_eq!(
        database.read_view_ownership().unwrap(),
        (2, old_pinned + new_pinned)
    );
    assert_eq!(
        old.next_batch().unwrap().unwrap().value(0),
        Some(vec![1; 4096].as_slice())
    );
    assert_eq!(
        new.next_batch().unwrap().unwrap().value(0),
        Some(vec![2; 4096].as_slice())
    );
    new.cancel().unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (1, old_pinned));
    drop(old);
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
    assert_eq!(memory.stats().generations, 1);
    drop(database);
    assert_eq!(memory.stats().generations, 0);
}

#[test]
fn shared_segments_do_not_hide_memtable_pins_or_new_generations() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::create(&directory.path().join("distinct-pins")).unwrap();
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record".to_vec(),
                value: b"old".to_vec(),
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();
    let mut old = view(&database);
    let old_pinned = old.evidence().pinned_bytes;
    database
        .write_owned(
            WriteBatch::new(vec![Mutation::Put {
                key: b"record".to_vec(),
                value: vec![3; 4096],
            }])
            .unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    let mut too_small = request(&database);
    too_small.budget.max_pinned_bytes = old_pinned;
    assert!(matches!(
        database.begin_projected_read(too_small),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PinnedBytes,
            ..
        })
    ));
    assert_eq!(database.read_view_ownership().unwrap(), (1, old_pinned));
    let with_memtable = view(&database);
    let full = with_memtable.evidence().pinned_bytes;
    assert!(full > old_pinned);
    assert_eq!(database.read_view_ownership().unwrap(), (2, full));
    let mut third_request = request(&database);
    third_request.budget.max_active_views = 3;
    let second_memtable = database.begin_projected_read(third_request).unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (3, full));
    drop(with_memtable);
    assert_eq!(database.read_view_ownership().unwrap(), (2, full));
    drop(second_memtable);
    assert_eq!(database.read_view_ownership().unwrap(), (1, old_pinned));
    database.flush_memtable(2).unwrap();
    let new_generation = view(&database);
    let new_pinned = new_generation.evidence().pinned_bytes;
    assert_eq!(database.read_view_ownership().unwrap(), (2, new_pinned));
    drop(new_generation);
    let mut bounded = request(&database);
    bounded.budget.max_pinned_bytes = new_pinned - 1;
    assert!(matches!(
        database.begin_projected_read(bounded),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PinnedBytes,
            ..
        })
    ));
    assert_eq!(database.read_view_ownership().unwrap(), (1, old_pinned));
    assert_eq!(
        old.next_batch().unwrap().unwrap().value(0),
        Some(b"old".as_slice())
    );
    old.cancel().unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
}

#[test]
fn narrow_view_admission_counts_only_selected_segments() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::create(&directory.path().join("narrow-view")).unwrap();
    for (key, byte) in [(b"a".as_slice(), 1), (b"z".as_slice(), 2)] {
        database
            .write_owned(
                WriteBatch::new(vec![Mutation::Put {
                    key: key.to_vec(),
                    value: vec![byte; 4096],
                }])
                .unwrap(),
                Durability::Authoritative,
            )
            .unwrap();
        database.flush_memtable(u64::from(byte)).unwrap();
    }
    let segments = &database.manifest().segments;
    assert_eq!(segments.len(), 2);
    let first = segments
        .iter()
        .find(|s| s.first_key.as_slice() == b"a")
        .unwrap()
        .bytes;
    let second = segments
        .iter()
        .find(|s| s.first_key.as_slice() == b"z")
        .unwrap()
        .bytes;
    assert!(first + second > first);
    let mut narrow = request(&database);
    narrow.ranges = vec![ProjectedReadRange::new(b"a".to_vec(), Some(b"b".to_vec())).unwrap()];
    narrow.budget.max_pinned_bytes = first;
    let mut stream = database
        .begin_projected_read(narrow)
        .expect("unrelated segment must not exhaust narrow read budget");
    assert_eq!(database.read_view_ownership().unwrap(), (1, first));
    assert_eq!(
        stream.next_batch().unwrap().unwrap().value(0),
        Some(vec![1; 4096].as_slice())
    );
    stream.cancel().unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
}

#[test]
fn selected_segment_pins_charge_shared_union_and_release_independently() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::create(&directory.path().join("segment-union")).unwrap();
    for (key, byte) in [(b"a".as_slice(), 1), (b"z".as_slice(), 2)] {
        database
            .write_owned(
                WriteBatch::new(vec![Mutation::Put {
                    key: key.to_vec(),
                    value: vec![byte; 4096],
                }])
                .unwrap(),
                Durability::Authoritative,
            )
            .unwrap();
        database.flush_memtable(u64::from(byte)).unwrap();
    }
    let segments = &database.manifest().segments;
    let first = segments.iter().find(|s| s.first_key == b"a").unwrap().bytes;
    let second = segments.iter().find(|s| s.first_key == b"z").unwrap().bytes;
    let narrow = |start: &[u8], end: &[u8], limit| {
        let mut candidate = request(&database);
        candidate.ranges =
            vec![ProjectedReadRange::new(start.to_vec(), Some(end.to_vec())).unwrap()];
        candidate.budget.max_active_views = 3;
        candidate.budget.max_pinned_bytes = limit;
        candidate
    };
    let first_view = database
        .begin_projected_read(narrow(b"a", b"b", first))
        .unwrap();
    let second_view = database
        .begin_projected_read(narrow(b"a", b"b", first))
        .unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (2, first));
    assert!(matches!(
        database.begin_projected_read(narrow(b"z", b"zz", first)),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PinnedBytes,
            ..
        })
    ));
    assert_eq!(database.read_view_ownership().unwrap(), (2, first));
    let third = database
        .begin_projected_read(narrow(b"z", b"zz", first + second))
        .unwrap();
    assert_eq!(database.read_view_ownership().unwrap(), (3, first + second));
    drop(first_view);
    assert_eq!(database.read_view_ownership().unwrap(), (2, first + second));
    drop(second_view);
    assert_eq!(database.read_view_ownership().unwrap(), (1, second));
    drop(third);
    assert_eq!(database.read_view_ownership().unwrap(), (0, 0));
}
