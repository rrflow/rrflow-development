use rrd_core::{Claim, Predicate, Producer, Subject};
use rrd_store::{RrflowKvStore, StorageEngine};

fn claim(object: &str) -> Claim {
    Claim::new(
        Subject::new("default-engine").unwrap(),
        Predicate::new("status").unwrap(),
        object,
        1,
        1,
        Producer {
            actor: "test".into(),
            on_behalf_of: None,
            session: None,
        },
    )
}

#[test]
fn missing_paths_create_rrflow_kv_and_reopen_by_authenticated_marker() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("nested").join("store");
    let engine = RrflowKvStore::open(&path).unwrap();
    assert!(path.join("CURRENT").is_file());
    engine.claims().append_batch(&[claim("rrflow-kv")]).unwrap();
    drop(engine);

    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_eq!(reopened.claims().sequence().unwrap(), 1);
}

#[test]
fn existing_empty_directories_initialize_as_rrflow_kv() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("empty-store");
    std::fs::create_dir(&path).unwrap();
    let _engine = RrflowKvStore::open(&path).unwrap();
    assert!(path.join("CURRENT").is_file());
}

#[test]
fn partial_rrflow_kv_identity_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("partial-rrflow-kv");
    std::fs::create_dir(&path).unwrap();
    std::fs::write(path.join("MANIFEST.LOCK"), []).unwrap();
    assert!(RrflowKvStore::open(&path).is_err());
}
