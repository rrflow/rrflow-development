use rrd_core::ScopeId;
use rrd_store::{ControlTransition, Error, RrflowKvStore, RrflowMxStore, StorageEngine};

fn scope(name: &str) -> ScopeId {
    ScopeId::new(name).unwrap()
}

fn catalogue_transition(key_suffix: &str) -> ControlTransition {
    ControlTransition {
        key: format!("server/state/catalogue/{key_suffix}"),
        expected: None,
        replacement: Some(br#"{"state":"ready"}"#.to_vec()),
        at: 1,
        actor: "test:catalogue-revision".into(),
        action: "catalogue.ensure".into(),
        request_id: format!("request-{key_suffix}"),
        operation_id: format!("operation-{key_suffix}"),
    }
}

fn assert_catalogue_revision_contract(engine: &dyn StorageEngine, key_suffix: &str) {
    let target = scope("instance:catalogue-target");
    let unrelated = scope("instance:catalogue-unrelated");
    let stale = engine.runtime().read_stamp(&target).unwrap();
    let unrelated_before = engine.runtime().read_stamp(&unrelated).unwrap();

    assert_eq!(stale.catalog_revision, 0);
    assert_eq!(unrelated_before.catalog_revision, 0);

    let cursor_before = stale.commit_cursor;
    let manifest_before = stale.manifest_id.clone();
    let (revision, entry) = engine
        .control()
        .commit_catalog(&target, &catalogue_transition(key_suffix))
        .unwrap();

    assert_eq!(revision, 1);
    assert!(entry.verify());

    let fresh = engine.runtime().read_stamp(&target).unwrap();
    assert_eq!(fresh.catalog_revision, 1);
    assert_eq!(fresh.commit_cursor, cursor_before);
    assert_ne!(fresh.manifest_id, manifest_before);

    assert!(matches!(
        engine
            .runtime()
            .read_changes(&stale, stale.commit_cursor, 1),
        Err(Error::ReadStampMismatch(_))
    ));
    engine
        .runtime()
        .read_changes(&fresh, fresh.commit_cursor, 1)
        .unwrap();

    let unrelated_after = engine.runtime().read_stamp(&unrelated).unwrap();
    assert_eq!(unrelated_after.catalog_revision, 0);
    assert_eq!(unrelated_after.manifest_id, unrelated_before.manifest_id);
}

#[test]
fn every_engine_binds_catalogue_transitions_to_the_scope_read_stamp() {
    assert_catalogue_revision_contract(&RrflowMxStore::new(), "rrflow_mx");

    let rrflow_kv_root = tempfile::tempdir().unwrap();
    assert_catalogue_revision_contract(
        &RrflowKvStore::open(rrflow_kv_root.path()).unwrap(),
        "rrflow_kv",
    );
}

#[test]
fn rrflow_kv_catalogue_revision_survives_reopen() {
    let root = tempfile::tempdir().unwrap();
    let target = scope("instance:catalogue-reopen-rrflow-kv");
    let stale = {
        let engine = RrflowKvStore::open(root.path()).unwrap();
        let stale = engine.runtime().read_stamp(&target).unwrap();
        let (revision, _) = engine
            .control()
            .commit_catalog(&target, &catalogue_transition("reopen-rrflow-kv"))
            .unwrap();
        assert_eq!(revision, 1);
        stale
    };

    let reopened = RrflowKvStore::open(root.path()).unwrap();
    let fresh = reopened.runtime().read_stamp(&target).unwrap();
    assert_eq!(fresh.catalog_revision, 1);
    assert_eq!(fresh.commit_cursor, stale.commit_cursor);
    assert_ne!(fresh.manifest_id, stale.manifest_id);
    assert!(matches!(
        reopened
            .runtime()
            .read_changes(&stale, stale.commit_cursor, 1),
        Err(Error::ReadStampMismatch(_))
    ));
}
