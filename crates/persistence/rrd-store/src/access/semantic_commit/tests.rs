use super::super::runtime_state::{get, get_json};
use super::*;
use crate::{RrflowKvStore, RrflowMxStore, StorageEngine};
use rrd_core::{
    RuntimeLogicalModel, RuntimeProperties, RuntimeRecord, RuntimeRecordSchema, RuntimeRef,
    RuntimeRelationSchema, RuntimeRetirement, RuntimeSchemaRegistry, RuntimeType,
};
use std::collections::BTreeSet;

struct CommittedFixture {
    scope: rrd_core::ScopeId,
    original: RuntimeRelation,
    replacement: RuntimeRelation,
    commit_ids: Vec<String>,
}

fn record(id: &str) -> RuntimeRecord {
    RuntimeRecord {
        reference: RuntimeRef::new("file", id).unwrap(),
        valid_from: 100,
        valid_to: None,
        properties: RuntimeProperties::new(),
    }
}

fn relation(id: &str, to: &str, valid_from: u64) -> RuntimeRelation {
    RuntimeRelation {
        reference: RuntimeRef::new("imports", id).unwrap(),
        from: RuntimeRef::new("file", "a").unwrap(),
        to: RuntimeRef::new("file", to).unwrap(),
        valid_from,
        valid_to: None,
        properties: RuntimeProperties::new(),
    }
}

fn schema() -> RuntimeSchemaRegistry {
    let file = RuntimeType::new("file").unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "semantic commit fan-out");
    registry
        .define_record_table(
            file.clone(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema::default(),
        )
        .unwrap();
    registry
        .define_relation_table(
            RuntimeType::new("imports").unwrap(),
            RuntimeRelationSchema {
                from: BTreeSet::from([file.clone()]),
                to: BTreeSet::from([file]),
                ..RuntimeRelationSchema::default()
            },
        )
        .unwrap();
    registry
}

fn commit_fixture(storage: &dyn StorageEngine) -> CommittedFixture {
    let scope = rrd_core::ScopeId::new("project:semantic-atomicity").unwrap();
    let original = relation("a-target", "b", 100);
    let replacement = relation("a-target", "c", 200);
    let first = storage
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope.clone(),
            at: 100,
            actor: "agent:semantic-atomicity".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry: schema() },
                RuntimeMutation::Record {
                    record: record("a"),
                },
                RuntimeMutation::Record {
                    record: record("b"),
                },
                RuntimeMutation::Record {
                    record: record("c"),
                },
                RuntimeMutation::Relation {
                    relation: original.clone(),
                },
            ],
        })
        .unwrap();
    let second = storage
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope.clone(),
            at: 200,
            actor: "agent:semantic-atomicity".into(),
            expected_cursor: first.last_cursor,
            mutations: vec![
                RuntimeMutation::Relation {
                    relation: replacement.clone(),
                },
                RuntimeMutation::Retire {
                    retirement: RuntimeRetirement {
                        model: RuntimeLogicalModel::GraphRelation,
                        reference: replacement.reference.clone(),
                        effective_at: 300,
                    },
                },
            ],
        })
        .unwrap();
    assert_eq!((first.last_cursor, second.last_cursor), (5, 7));
    CommittedFixture {
        scope,
        original,
        replacement,
        commit_ids: vec![first.commit_id, second.commit_id],
    }
}

fn assert_physical_fan_out(storage: &dyn StorageEngine, fixture: &CommittedFixture) {
    let transaction = storage.begin_transaction().unwrap();
    for id in ["a", "b", "c"] {
        let reference = RuntimeRef::new("file", id).unwrap();
        let current: RuntimeRecord = get_json(
            &*transaction,
            keyspaces::RUNTIME_RECORDS,
            &keyspaces::runtime_identity_key(
                keyspaces::RUNTIME_RECORDS,
                &fixture.scope,
                &reference,
            ),
        )
        .unwrap()
        .unwrap();
        assert_eq!(current.reference, reference);
    }
    let record_history: RuntimeChange = get_json(
        &*transaction,
        keyspaces::RUNTIME_RECORD_VERSIONS,
        &keyspaces::runtime_version_key(
            keyspaces::RUNTIME_RECORD_VERSIONS,
            &fixture.scope,
            &RuntimeRef::new("file", "a").unwrap(),
            100,
            2,
        ),
    )
    .unwrap()
    .unwrap();
    assert!(matches!(
        record_history.mutation,
        RuntimeMutation::Record { .. }
    ));

    let current_relation_key = keyspaces::runtime_identity_key(
        keyspaces::RUNTIME_RELATIONS,
        &fixture.scope,
        &fixture.replacement.reference,
    );
    assert!(get(
        &*transaction,
        keyspaces::RUNTIME_RELATIONS,
        &current_relation_key,
    )
    .unwrap()
    .is_none());

    for (relation, effective_at, cursor, retired) in [
        (&fixture.original, 100, 5, false),
        (&fixture.replacement, 200, 6, false),
        (&fixture.replacement, 300, 7, true),
    ] {
        let version: RuntimeChange = get_json(
            &*transaction,
            keyspaces::RUNTIME_RELATION_VERSIONS,
            &keyspaces::runtime_version_key(
                keyspaces::RUNTIME_RELATION_VERSIONS,
                &fixture.scope,
                &relation.reference,
                effective_at,
                cursor,
            ),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            matches!(version.mutation, RuntimeMutation::Retire { .. }),
            retired
        );
        for space in [
            keyspaces::RUNTIME_OUTGOING_EDGE_VERSIONS,
            keyspaces::RUNTIME_INCOMING_EDGE_VERSIONS,
        ] {
            let adjacency_version: RuntimeChange = get_json(
                &*transaction,
                space,
                &keyspaces::runtime_adjacency_version_key(
                    space,
                    &fixture.scope,
                    relation,
                    effective_at,
                    cursor,
                ),
            )
            .unwrap()
            .unwrap();
            assert_eq!(adjacency_version.digest, version.digest);
        }
    }
    for relation in [&fixture.original, &fixture.replacement] {
        for space in [
            keyspaces::RUNTIME_OUTGOING_EDGES,
            keyspaces::RUNTIME_INCOMING_EDGES,
        ] {
            assert!(get(
                &*transaction,
                space,
                &keyspaces::runtime_adjacency_key(space, &fixture.scope, relation),
            )
            .unwrap()
            .is_none());
        }
    }

    let deltas = storage.runtime().projection_deltas_since(0, 16).unwrap();
    let outbox = storage.runtime().outbox_since(0, 16).unwrap();
    assert_eq!(deltas, outbox);
    assert_eq!(deltas.len(), 6);
    assert_eq!(deltas.last().unwrap().source_cursor, 7);
    for commit_id in &fixture.commit_ids {
        assert!(storage.runtime().audit(commit_id).unwrap().is_some());
        assert!(storage
            .runtime()
            .commit_outcome(commit_id)
            .unwrap()
            .is_some());
    }
}

fn remove_runtime_accumulator(storage: &dyn StorageEngine) {
    let mut transaction = storage.begin_transaction().unwrap();
    transaction
        .delete(checked_key(keyspaces::META, &keyspaces::runtime_accumulator_state_key()).unwrap())
        .unwrap();
    transaction
        .commit(crate::keyspaces::Durability::Authoritative)
        .unwrap();
}

fn assert_missing_accumulator_is_rejected(storage: &dyn StorageEngine, cursor: u64) {
    let error = storage
        .runtime()
        .commit(&RuntimeCommit {
            scope: rrd_core::ScopeId::new("project:semantic-atomicity").unwrap(),
            at: 400,
            actor: "agent:semantic-atomicity".into(),
            expected_cursor: cursor,
            mutations: vec![RuntimeMutation::Record {
                record: RuntimeRecord {
                    valid_from: 400,
                    ..record("d")
                },
            }],
        })
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Substrate(reason)
            if reason == format!("runtime accumulator is absent at non-empty cursor {cursor}")
    ));
}

#[test]
fn graph_temporal_and_delivery_effects_share_one_plan_on_mx_kv_and_reopen() {
    let mx = RrflowMxStore::new();
    let mx_fixture = commit_fixture(&mx);
    assert_physical_fan_out(&mx, &mx_fixture);

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv");
    let kv_fixture = {
        let kv = RrflowKvStore::open(&path).unwrap();
        let fixture = commit_fixture(&kv);
        assert_physical_fan_out(&kv, &fixture);
        fixture
    };
    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_physical_fan_out(&reopened, &kv_fixture);
    assert_eq!(mx_fixture.commit_ids, kv_fixture.commit_ids);
}

#[test]
fn nonempty_runtime_never_rebuilds_a_missing_accumulator_from_the_change_log() {
    let mx = RrflowMxStore::new();
    let mx_fixture = commit_fixture(&mx);
    remove_runtime_accumulator(&mx);
    assert_missing_accumulator_is_rejected(&mx, 7);

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv");
    {
        let kv = RrflowKvStore::open(&path).unwrap();
        let kv_fixture = commit_fixture(&kv);
        assert_eq!(kv_fixture.commit_ids, mx_fixture.commit_ids);
        remove_runtime_accumulator(&kv);
    }
    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_missing_accumulator_is_rejected(&reopened, 7);
}
