use rrd_lsm::{
    Database, DatabaseOptions, Durability, MaintenancePolicy, Mutation, WriteBatch, WriteLockWait,
    MAX_WRITE_PATH_DIAGNOSTIC_SAMPLES, WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION,
};

fn put(key: &str, value: &str) -> Mutation {
    Mutation::Put {
        key: key.as_bytes().to_vec(),
        value: value.as_bytes().to_vec(),
    }
}

fn write(database: &mut Database, key: &str, durability: Durability) {
    database
        .write_owned(
            WriteBatch::new(vec![put(key, "value")]).unwrap(),
            durability,
        )
        .unwrap();
}

#[test]
fn diagnostics_are_disabled_by_default_and_bounded_when_enabled() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("database");
    let mut database = Database::create(&root).unwrap();

    write(&mut database, "unprofiled", Durability::Authoritative);
    assert!(database.take_write_path_diagnostics().is_none());
    assert!(database.enable_write_path_diagnostics(0).is_err());
    assert!(database
        .enable_write_path_diagnostics(MAX_WRITE_PATH_DIAGNOSTIC_SAMPLES + 1)
        .is_err());

    database.enable_write_path_diagnostics(1).unwrap();
    assert!(database.enable_write_path_diagnostics(1).is_err());
    write(&mut database, "profiled-a", Durability::Authoritative);
    write(&mut database, "profiled-b", Durability::Authoritative);
    let diagnostics = database.take_write_path_diagnostics().unwrap();

    assert_eq!(
        diagnostics.contract_version,
        WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION
    );
    assert_eq!(diagnostics.capacity, 1);
    assert_eq!(diagnostics.observed_samples, 2);
    assert_eq!(diagnostics.dropped_samples, 1);
    assert_eq!(diagnostics.samples.len(), 1);
    assert_eq!(diagnostics.samples[0].ordinal, 0);
    assert_eq!(diagnostics.samples[0].first_sequence, 2);
    assert_eq!(diagnostics.samples[0].last_sequence, 2);
    assert!(database.take_write_path_diagnostics().is_none());

    drop(database);
    let reopened = Database::open(&root).unwrap();
    let snapshot = reopened.snapshot();
    for key in ["unprofiled", "profiled-a", "profiled-b"] {
        assert_eq!(
            reopened.get(key.as_bytes(), snapshot).unwrap().as_deref(),
            Some(b"value".as_slice())
        );
    }
}

#[test]
fn raw_phases_distinguish_buffered_and_authoritative_wal_sync() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("database");
    let mut database = Database::create(&root).unwrap();
    database.enable_write_path_diagnostics(2).unwrap();

    write(&mut database, "buffered", Durability::Buffered);
    write(&mut database, "authoritative", Durability::Authoritative);
    let diagnostics = database.take_write_path_diagnostics().unwrap();
    assert_eq!(diagnostics.dropped_samples, 0);
    assert_eq!(diagnostics.samples.len(), 2);

    let buffered = &diagnostics.samples[0];
    let authoritative = &diagnostics.samples[1];
    assert_eq!(buffered.durability, Durability::Buffered);
    assert_eq!(buffered.phases.wal_sync.wall_ns, 0);
    assert_eq!(authoritative.durability, Durability::Authoritative);
    assert!(authoritative.phases.wal_sync.wall_ns > 0);
    for sample in &diagnostics.samples {
        assert_eq!(sample.mutation_count, 1);
        assert!(sample.encoded_payload_bytes > 0);
        assert!(sample.wal_frame_bytes > sample.encoded_payload_bytes);
        assert!(sample.memtable_bytes_added > 0);
        assert!(sample.phases.measured_wall_ns() <= sample.phases.physical_total.wall_ns);
        #[cfg(target_os = "linux")]
        assert!(sample.thread_resources.is_some());
        #[cfg(not(target_os = "linux"))]
        assert!(sample.thread_resources.is_none());
    }
}

#[test]
fn transaction_lock_context_and_automatic_maintenance_are_attributed() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("database");
    let options = DatabaseOptions {
        maintenance: MaintenancePolicy {
            wal_payload_max_bytes: usize::MAX,
            memtable_max_versions: 1,
        },
        ..DatabaseOptions::default()
    };
    let mut database = Database::create_with_options(&root, options).unwrap();
    database.enable_write_path_diagnostics(2).unwrap();

    for (key, lock_wait) in [
        (
            "first",
            WriteLockWait {
                begin_mutex_wait_ns: 11,
                commit_mutex_wait_ns: 13,
            },
        ),
        ("second", WriteLockWait::default()),
    ] {
        let mut transaction = database.begin_transaction().unwrap();
        transaction
            .put(key.as_bytes().to_vec(), b"value".to_vec())
            .unwrap();
        database
            .commit_transaction_with_write_context(
                transaction,
                Durability::Authoritative,
                lock_wait,
            )
            .unwrap();
    }

    let diagnostics = database.take_write_path_diagnostics().unwrap();
    assert_eq!(diagnostics.samples.len(), 2);
    assert_eq!(diagnostics.samples[0].lock_wait.begin_mutex_wait_ns, 11);
    assert_eq!(diagnostics.samples[0].lock_wait.commit_mutex_wait_ns, 13);
    assert_eq!(diagnostics.samples[0].maintenance.write_stalls, 0);
    assert_eq!(diagnostics.samples[1].maintenance.write_stalls, 1);
    assert_eq!(diagnostics.samples[1].maintenance.automatic_flushes, 1);
    assert_eq!(diagnostics.samples[1].maintenance.failed_flushes, 0);
    assert!(diagnostics.samples[1].phases.maintenance.wall_ns > 0);
}
