use rrd_core::{
    Claim, Producer, RuntimeCommit, RuntimeMutation, RuntimeProperties, RuntimeRecord,
    RuntimeRecordSchema, RuntimeRef, RuntimeSchemaRegistry, RuntimeType, ScopeId,
};
use rrd_store::{ControlTransition, RrflowKvStore, StorageEngine, StorageProfile};

fn exercise(storage: &dyn StorageEngine) -> (u64, u64, String, usize) {
    let claim = Claim::new(
        rrd_core::Subject::new("file:src/lib.rs").unwrap(),
        rrd_core::Predicate::new("language").unwrap(),
        "rust",
        10,
        10,
        Producer {
            actor: "agent:repository-conformance".into(),
            on_behalf_of: None,
            session: Some("session:repository-conformance".into()),
        },
    );
    let appended = storage.claims().append_batch(&[claim]).unwrap();

    let transition = ControlTransition {
        key: "server/state/repository/conformance".into(),
        expected: None,
        replacement: Some(b"ready".to_vec()),
        at: 10,
        actor: "agent:repository-conformance".into(),
        action: "repository.verify".into(),
        request_id: "request:repository-conformance".into(),
        operation_id: "operation:repository-conformance".into(),
    };
    let journal = storage.control().commit(&transition).unwrap();

    let scope = ScopeId::new("project:repository-conformance").unwrap();
    let item_kind = RuntimeType::new("item").unwrap();
    let item = RuntimeRef::new("item", "one").unwrap();
    let mut schema = RuntimeSchemaRegistry::empty(1, "repository conformance");
    schema
        .records
        .insert(item_kind, RuntimeRecordSchema::default());
    let outcome = storage
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope.clone(),
            at: 11,
            actor: "agent:repository-conformance".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry: schema },
                RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: item,
                        valid_from: 11,
                        valid_to: None,
                        properties: RuntimeProperties::new(),
                    },
                },
            ],
        })
        .unwrap();
    let (_, snapshot) = storage.runtime().data_snapshot(&scope, 11, 16).unwrap();
    storage.projections().rebuild_current().unwrap();

    (
        appended.last_sequence,
        journal.sequence,
        outcome.commit_id,
        snapshot.records.len(),
    )
}

#[test]
fn all_semantic_repositories_share_one_mx_kv_transaction_implementation() {
    let directory = tempfile::tempdir().unwrap();
    let mx = StorageProfile::rrflow_mx();
    let kv = StorageProfile::rrflow_kv(
        RrflowKvStore::open(&directory.path().join("rrflow-kv")).unwrap(),
    );
    assert_eq!(exercise(&mx), exercise(&kv));
}

#[test]
fn repository_state_reopens_through_rrflow_kv() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv-reopen");
    let expected = {
        let storage = RrflowKvStore::open(&path).unwrap();
        exercise(&storage)
    };
    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_eq!(reopened.claims().sequence().unwrap(), expected.0);
    assert_eq!(reopened.control().sequence().unwrap(), expected.1);
    assert_eq!(
        reopened
            .runtime()
            .commit_outcome(&expected.2)
            .unwrap()
            .unwrap()
            .commit_id,
        expected.2
    );
}
