use rrd_core::{Claim, Predicate, Producer, ScopeId, Subject};
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
    assert!(engine.open_evidence().created);
    assert_eq!(engine.open_evidence().segment_validation.segment_count, 0);
    assert_eq!(engine.open_evidence().reconciliation.page_requests, 0);
    assert_eq!(engine.open_evidence().reconciliation.read_operations, 0);
    engine.claims().append_batch(&[claim("rrflow-kv")]).unwrap();
    drop(engine);

    let reopened = RrflowKvStore::open(&path).unwrap();
    assert!(!reopened.open_evidence().created);
    assert_eq!(reopened.open_evidence().segment_validation.segment_count, 0);
    assert_eq!(reopened.open_evidence().reconciliation.page_requests, 0);
    assert_eq!(reopened.open_evidence().reconciliation.read_operations, 0);
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

#[test]
fn rrflow_kv_open_separates_segment_validation_and_reconciliation_io() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("open-evidence");
    let scope = ScopeId::new("instance:open-evidence").unwrap();
    {
        let store = RrflowKvStore::open(&path).unwrap();
        store
            .runtime()
            .open_snapshot(&scope, "agent:open-evidence", 10, 100)
            .unwrap();
        store.projections().put("filter-a", b"first").unwrap();
        store.projections().put("filter-c", b"third").unwrap();
        store.flush(20).unwrap();
    }

    let reopened = RrflowKvStore::open(&path).unwrap();
    let open = reopened.open_evidence().clone();
    assert!(open.segment_validation.segment_count > 0);
    assert_eq!(
        open.segment_validation.full_checksum_operations,
        open.segment_validation.segment_count
    );
    assert!(open.segment_validation.format_probe_bytes > 0);
    assert!(open.segment_validation.full_checksum_bytes > 0);
    assert!(open.segment_validation.metadata_bytes > 0);
    assert!(open.segment_validation.persisted_filter_count > 0);
    assert!(open.segment_validation.persisted_filter_bytes > 0);
    assert_eq!(open.segment_validation.semantic_page_operations, 0);
    assert_eq!(open.segment_validation.semantic_page_bytes, 0);
    assert!(open.reconciliation.page_requests > 0);
    assert!(open.reconciliation.page_cache_loads > 0);
    assert!(open.reconciliation.read_operations > 0);
    assert!(open.reconciliation.bytes_read > 0);
    assert_eq!(
        open.reconciliation.page_cache_loads,
        open.reconciliation.read_operations
    );
    assert_eq!(
        open.reconciliation.page_bytes_read,
        open.reconciliation.bytes_read
    );

    let before_absent = reopened.physical_store_evidence().unwrap();
    assert_eq!(reopened.projections().get("filter-b").unwrap(), None);
    let after_absent = reopened.physical_store_evidence().unwrap();
    assert!(
        after_absent.filter_negatives.unwrap() > before_absent.filter_negatives.unwrap(),
        "an authenticated persisted filter should reject the in-range absent key"
    );
    assert_eq!(
        reopened.projections().get("filter-a").unwrap(),
        Some(b"first".to_vec())
    );

    assert_eq!(reopened.runtime().snapshots(20).unwrap().len(), 1);
    assert_eq!(reopened.open_evidence(), &open);
}
