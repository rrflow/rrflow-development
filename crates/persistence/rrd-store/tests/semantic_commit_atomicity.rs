use rrd_core::{
    RuntimeCommit, RuntimeMutation, RuntimeProperties, RuntimeRecord, RuntimeRecordSchema,
    RuntimeRef, RuntimeRelation, RuntimeRelationSchema, RuntimeSchemaRegistry, RuntimeType,
    ScopeId,
};
use rrd_store::{RrflowKvStore, RrflowMxStore, StorageEngine};
use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq)]
struct PublicEvidence {
    commit_id: String,
    cursor: u64,
    changes: usize,
    records: usize,
    relations: usize,
    delta_cursors: Vec<u64>,
    audit_cursor: u64,
}

fn commit(storage: &dyn StorageEngine) -> PublicEvidence {
    let scope = ScopeId::new("project:semantic-commit-public").unwrap();
    let file = RuntimeType::new("file").unwrap();
    let imports = RuntimeType::new("imports").unwrap();
    let mut schema = RuntimeSchemaRegistry::empty(1, "public semantic commit corpus");
    schema
        .records
        .insert(file.clone(), RuntimeRecordSchema::default());
    schema.relations.insert(
        imports,
        RuntimeRelationSchema {
            from: BTreeSet::from([file.clone()]),
            to: BTreeSet::from([file]),
            ..RuntimeRelationSchema::default()
        },
    );
    let source = RuntimeRef::new("file", "src/lib.rs").unwrap();
    let target = RuntimeRef::new("file", "src/store.rs").unwrap();
    let outcome = storage
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope.clone(),
            at: 100,
            actor: "agent:semantic-commit-public".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry: schema },
                RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: source.clone(),
                        valid_from: 100,
                        valid_to: None,
                        properties: RuntimeProperties::new(),
                    },
                },
                RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: target.clone(),
                        valid_from: 100,
                        valid_to: None,
                        properties: RuntimeProperties::new(),
                    },
                },
                RuntimeMutation::Relation {
                    relation: RuntimeRelation {
                        reference: RuntimeRef::new("imports", "lib-store").unwrap(),
                        from: source,
                        to: target,
                        valid_from: 100,
                        valid_to: None,
                        properties: RuntimeProperties::new(),
                    },
                },
            ],
        })
        .unwrap();
    let page = storage
        .runtime()
        .changes_since(0, 16, Some(&scope))
        .unwrap();
    let (_, snapshot) = storage.runtime().data_snapshot(&scope, 100, 16).unwrap();
    let deltas = storage.runtime().projection_deltas_since(0, 16).unwrap();
    let outbox = storage.runtime().outbox_since(0, 16).unwrap();
    assert_eq!(deltas, outbox);
    assert_eq!(outcome.outbox_count, 3);
    assert_eq!(
        storage
            .runtime()
            .commit_outcome(&outcome.commit_id)
            .unwrap(),
        Some(outcome.clone())
    );
    let audit = storage
        .runtime()
        .audit(&outcome.commit_id)
        .unwrap()
        .unwrap();
    audit.validate().unwrap();

    PublicEvidence {
        commit_id: outcome.commit_id,
        cursor: outcome.last_cursor,
        changes: page.changes.len(),
        records: snapshot.records.len(),
        relations: snapshot.relations.len(),
        delta_cursors: deltas
            .into_iter()
            .map(|delta| delta.source_cursor)
            .collect(),
        audit_cursor: audit.outcome_cursor.unwrap(),
    }
}

#[test]
fn semantic_commit_has_identical_public_evidence_on_mx_kv_and_kv_reopen() {
    let mx = RrflowMxStore::new();
    let mx_evidence = commit(&mx);

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv");
    let kv_evidence = {
        let kv = RrflowKvStore::open(&path).unwrap();
        commit(&kv)
    };
    assert_eq!(mx_evidence, kv_evidence);

    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_eq!(reopened.runtime().cursor().unwrap(), kv_evidence.cursor);
    assert_eq!(
        reopened
            .runtime()
            .projection_deltas_since(0, 16)
            .unwrap()
            .into_iter()
            .map(|delta| delta.source_cursor)
            .collect::<Vec<_>>(),
        kv_evidence.delta_cursors
    );
    assert_eq!(
        reopened
            .runtime()
            .audit(&kv_evidence.commit_id)
            .unwrap()
            .unwrap()
            .outcome_cursor,
        Some(kv_evidence.audit_cursor)
    );
}
