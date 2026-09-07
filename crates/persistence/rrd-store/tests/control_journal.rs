use rrd_store::{ControlTransition, Error, RrflowKvStore, RrflowMxStore, StorageEngine};

fn transition(expected: Option<&[u8]>, replacement: Option<&[u8]>, at: u64) -> ControlTransition {
    keyed_transition("server/state/session/session-1", expected, replacement, at)
}

fn keyed_transition(
    key: &str,
    expected: Option<&[u8]>,
    replacement: Option<&[u8]>,
    at: u64,
) -> ControlTransition {
    ControlTransition {
        key: key.into(),
        expected: expected.map(<[u8]>::to_vec),
        replacement: replacement.map(<[u8]>::to_vec),
        at,
        actor: "rrd-server".into(),
        action: "session.transition".into(),
        request_id: format!("request-{at}"),
        operation_id: format!("operation-{at}"),
    }
}

fn assert_atomic_batch(engine: &dyn StorageEngine) {
    let entries = engine
        .commit_control_batch(&[
            keyed_transition("server/state/audit/record-1", None, Some(b"record"), 20),
            keyed_transition("server/state/audit/head", None, Some(b"head"), 20),
        ])
        .unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].sequence + 1, entries[1].sequence);
    assert_eq!(
        entries[1].previous_digest.as_deref(),
        Some(entries[0].digest.as_str())
    );
    assert_eq!(
        engine
            .control_record("server/state/audit/record-1")
            .unwrap(),
        Some(b"record".to_vec())
    );

    let before = engine.control_sequence().unwrap();
    let denied = engine.commit_control_batch(&[
        keyed_transition("server/state/audit/record-2", None, Some(b"record-2"), 21),
        keyed_transition("server/state/audit/head", None, Some(b"wrong-head"), 21),
    ]);
    assert!(matches!(denied, Err(Error::ControlConflict(_))));
    assert_eq!(engine.control_sequence().unwrap(), before);
    assert_eq!(
        engine
            .control_record("server/state/audit/record-2")
            .unwrap(),
        None
    );
    assert_eq!(
        engine.control_record("server/state/audit/head").unwrap(),
        Some(b"head".to_vec())
    );
}

fn assert_journal(engine: &dyn StorageEngine) {
    assert_eq!(engine.control_sequence().unwrap(), 0);
    let created = engine
        .commit_control_transition(&transition(None, Some(b"open"), 10))
        .unwrap();
    assert_eq!(created.sequence, 1);
    assert!(created.verify());
    assert_eq!(
        engine.control_record(&created.key).unwrap(),
        Some(b"open".to_vec())
    );

    assert!(matches!(
        engine.commit_control_transition(&transition(None, Some(b"collision"), 11)),
        Err(Error::ControlConflict(_))
    ));
    let renewed = engine
        .commit_control_transition(&transition(Some(b"open"), Some(b"renewed"), 12))
        .unwrap();
    assert_eq!(renewed.sequence, 2);
    assert_eq!(
        renewed.previous_digest.as_deref(),
        Some(created.digest.as_str())
    );
    let deleted = engine
        .commit_control_transition(&transition(Some(b"renewed"), None, 13))
        .unwrap();
    assert_eq!(deleted.sequence, 3);
    assert_eq!(engine.control_record(&deleted.key).unwrap(), None);
    let journal = engine.control_journal_since(0, 10).unwrap();
    assert_eq!(journal, vec![created, renewed, deleted]);
    assert!(journal.iter().all(|entry| entry.verify()));
    assert_eq!(engine.control_sequence().unwrap(), 3);
}

#[test]
fn every_engine_materializes_and_journals_the_same_cas_transitions() {
    let memory = RrflowMxStore::new();
    assert_journal(&memory);
    assert_atomic_batch(&memory);
    let rrflow_kv = tempfile::tempdir().unwrap();
    let rrflow_kv = RrflowKvStore::open(rrflow_kv.path()).unwrap();
    assert_journal(&rrflow_kv);
    assert_atomic_batch(&rrflow_kv);
}

#[test]
fn materialized_state_and_hash_chain_survive_restart() {
    let root = tempfile::tempdir().unwrap();
    let engine = RrflowKvStore::open(root.path()).unwrap();
    engine
        .commit_control_transition(&transition(None, Some(b"open"), 10))
        .unwrap();
    drop(engine);
    let reopened = RrflowKvStore::open(root.path()).unwrap();
    assert_eq!(
        reopened
            .control_record("server/state/session/session-1")
            .unwrap(),
        Some(b"open".to_vec())
    );
    assert_eq!(reopened.control_journal_since(0, 10).unwrap().len(), 1);
    assert_eq!(reopened.control_sequence().unwrap(), 1);
}
