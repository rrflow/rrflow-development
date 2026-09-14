use rrd_core::{Claim, Predicate, Producer, RuntimeCommit, RuntimeMutation, ScopeId, Subject};
use rrd_store::{ControlTransition, RrflowKvInspector, RrflowKvStore, StorageEngine};

fn bootstrap_commit() -> RuntimeCommit {
    RuntimeCommit {
        scope: ScopeId::new("instance:installed-test").unwrap(),
        at: 100,
        actor: "rrflow:installer".into(),
        expected_cursor: 0,
        mutations: vec![RuntimeMutation::Claim {
            claim: Claim::new(
                Subject::new("runtime:installed-test").unwrap(),
                Predicate::new("status").unwrap(),
                "installed",
                100,
                100,
                Producer {
                    actor: "rrflow:installer".into(),
                    on_behalf_of: None,
                    session: None,
                },
            ),
        }],
    }
}

fn installed_transition(expected: Option<Vec<u8>>) -> ControlTransition {
    ControlTransition {
        key: "server/state/install/record".into(),
        expected,
        replacement: Some(br#"{"installed":true}"#.to_vec()),
        at: 100,
        actor: "rrflow:installer".into(),
        action: "install.initialize".into(),
        request_id: "install-request".into(),
        operation_id: "install-operation".into(),
    }
}

#[test]
fn explicit_create_open_and_inspect_never_collapse_into_create_on_open() {
    let parent = tempfile::tempdir().unwrap();
    let missing = parent.path().join("missing");
    assert!(RrflowKvStore::open_existing(&missing).is_err());
    assert!(RrflowKvInspector::open(&missing).is_err());
    assert!(!missing.exists());

    let root = parent.path().join("installed");
    let store = RrflowKvStore::create_new(&root).unwrap();
    assert!(store.open_evidence().created);
    assert!(RrflowKvStore::create_new(&root).is_err());
    drop(store);

    let reopened = RrflowKvStore::open_existing(&root).unwrap();
    assert!(!reopened.open_evidence().created);
    drop(reopened);
    let inspected = RrflowKvInspector::open(&root).unwrap();
    assert_eq!(inspected.inspection().runtime_cursor, 0);
    assert_eq!(inspected.inspection().control_journal_sequence, 0);
}

#[test]
fn bootstrap_runtime_and_control_effects_share_one_physical_commit() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("installed");
    let store = RrflowKvStore::create_new(&root).unwrap();
    let (runtime, entries) = store
        .runtime()
        .commit_with_control_transitions(&bootstrap_commit(), &[installed_transition(None)])
        .unwrap();
    assert_eq!(runtime.first_cursor, 1);
    assert_eq!(runtime.last_cursor, 1);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].sequence, 1);
    assert_eq!(store.runtime().cursor().unwrap(), 1);
    assert_eq!(store.control().sequence().unwrap(), 1);
    drop(store);

    let recovery = rrd_lsm::recover(&root.join("wal/00000000000000000001.wal")).unwrap();
    assert_eq!(recovery.batches.len(), 1, "bootstrap is one WAL frame");
    let inspected = RrflowKvInspector::open(&root).unwrap();
    assert_eq!(inspected.inspection().runtime_cursor, 1);
    assert_eq!(inspected.inspection().control_journal_sequence, 1);
    assert_eq!(
        inspected
            .control_record("server/state/install/record")
            .unwrap(),
        Some(br#"{"installed":true}"#.to_vec())
    );
    let read = inspected
        .runtime_read_stamp(&ScopeId::new("instance:installed-test").unwrap())
        .unwrap();
    read.validate().unwrap();
    assert_eq!(read.commit_cursor, 1);
}

#[test]
fn rejected_control_precondition_publishes_no_runtime_or_control_state() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("installed");
    let store = RrflowKvStore::create_new(&root).unwrap();
    assert!(store
        .runtime()
        .commit_with_control_transitions(
            &bootstrap_commit(),
            &[installed_transition(Some(b"not-current".to_vec()))],
        )
        .is_err());
    assert_eq!(store.runtime().cursor().unwrap(), 0);
    assert_eq!(store.control().sequence().unwrap(), 0);
    assert_eq!(
        store.control().get("server/state/install/record").unwrap(),
        None
    );
}
