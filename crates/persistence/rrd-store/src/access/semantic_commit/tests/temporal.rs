use super::*;
use crate::{RuntimeReadAccessPath, RuntimeReadBudget, RuntimeVersionedSource};
use rrflow_kernel::{DataTransaction, ReadStamp, RuntimeDataSnapshot, RuntimeReferentialAction};

fn resolved_cascade(storage: &dyn StorageEngine) -> (ReadStamp, ReadStamp) {
    let fixture = commit_fixture(storage);
    let mut rules = schema();
    rules.revision = 2;
    rules
        .relations
        .get_mut(&RuntimeType::new("imports").unwrap())
        .unwrap()
        .on_delete_to = Some(RuntimeReferentialAction::Cascade);
    storage
        .runtime()
        .commit(&RuntimeCommit {
            scope: fixture.scope.clone(),
            at: 400,
            actor: "agent:temporal-proof".into(),
            expected_cursor: 7,
            mutations: vec![RuntimeMutation::Schema { registry: rules }],
        })
        .unwrap();
    let read = storage.runtime().read_stamp(&fixture.scope).unwrap();
    let commit = RuntimeCommit {
        scope: fixture.scope,
        at: 500,
        actor: "agent:temporal-proof".into(),
        expected_cursor: read.commit_cursor,
        mutations: vec![RuntimeMutation::Retire {
            retirement: RuntimeRetirement {
                model: RuntimeLogicalModel::Relational,
                reference: record("b").reference,
                effective_at: 150,
            },
        }],
    };
    let transaction = DataTransaction::new(read.clone(), commit.clone()).unwrap();
    assert!(matches!(
        storage
            .runtime()
            .resolve_relation_actions(&transaction, RuntimeReadBudget::new(1).unwrap()),
        Err(Error::RuntimeReadBudgetExceeded { .. })
    ));
    // Low-level callers cannot bypass the canonical expanded write set.
    assert!(storage.runtime().commit(&commit).is_err());
    assert_eq!(storage.runtime().read_stamp(&read.scope).unwrap(), read);
    let (resolved, evidence) = storage
        .runtime()
        .resolve_relation_actions(&transaction, RuntimeReadBudget::new(1024).unwrap())
        .unwrap();
    assert_eq!(resolved.mutations.len(), 2);
    assert_ne!(resolved.digest(), commit.digest());
    let evidence = evidence.unwrap();
    evidence.validate().unwrap();
    assert_eq!(
        evidence
            .paths
            .iter()
            .find(|path| path.path == RuntimeReadAccessPath::AdjacencyVersions)
            .unwrap()
            .range_scans,
        2
    );
    let outcome = storage
        .runtime()
        .commit_data_transaction(&DataTransaction::new(read.clone(), resolved).unwrap())
        .unwrap();
    assert_eq!(outcome.last_cursor, read.commit_cursor + 2);
    assert_eq!(outcome.count, 2);
    let after = storage.runtime().read_stamp(&read.scope).unwrap();
    assert_temporal_cascade(storage, &read, &after);
    (read, after)
}

fn snapshot(storage: &dyn StorageEngine, read: &ReadStamp, valid: u64) -> RuntimeDataSnapshot {
    let selected = storage
        .runtime()
        .read_versioned(
            read,
            &RuntimeVersionedSource::all(),
            RuntimeReadBudget::new(1024).unwrap(),
        )
        .unwrap();
    let schema = crate::access::schema_at_read(read, &selected.changes).unwrap();
    RuntimeDataSnapshot::from_changes(
        &selected.changes,
        &schema,
        read.scope.clone(),
        valid,
        read.commit_cursor,
    )
    .unwrap()
}

fn assert_temporal_cascade(storage: &dyn StorageEngine, before: &ReadStamp, after: &ReadStamp) {
    for (read, valid, endpoint, edge) in [
        (before, 175, true, true),
        (before, 250, true, true),
        (after, 149, true, true),
        (after, 150, false, false),
        (after, 250, false, false),
    ] {
        let state = snapshot(storage, read, valid);
        assert_eq!(
            state.contains(RuntimeLogicalModel::Relational, &record("b").reference),
            endpoint
        );
        assert_eq!(
            state.contains(
                RuntimeLogicalModel::GraphRelation,
                &relation("a-target", "b", 100).reference
            ),
            edge,
            "cursor={} valid={valid} state={state:?}",
            read.commit_cursor
        );
    }
}

#[test]
fn cascade_is_canonical_bounded_atomic_and_preserves_both_times_after_reopen() {
    resolved_cascade(&RrflowMxStore::new());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv");
    let (before, after) = {
        let kv = RrflowKvStore::open(&path).unwrap();
        resolved_cascade(&kv)
    };
    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_temporal_cascade(&reopened, &before, &after);
}

#[test]
fn same_batch_schema_future_relation_and_endpoint_retirement_resolve_once() {
    let storage = RrflowMxStore::new();
    let scope = rrflow_kernel::ScopeId::new("project:batch-cascade").unwrap();
    let read = storage.runtime().read_stamp(&scope).unwrap();
    let mut registry = schema();
    registry
        .relations
        .get_mut(&RuntimeType::new("imports").unwrap())
        .unwrap()
        .on_delete_to = Some(RuntimeReferentialAction::Cascade);
    let commit = RuntimeCommit {
        scope: scope.clone(),
        at: 500,
        actor: "agent:temporal-proof".into(),
        expected_cursor: 0,
        mutations: vec![
            RuntimeMutation::Schema { registry },
            RuntimeMutation::Record {
                record: record("a"),
            },
            RuntimeMutation::Record {
                record: record("b"),
            },
            RuntimeMutation::Relation {
                relation: relation("future", "b", 300),
            },
            RuntimeMutation::Retire {
                retirement: RuntimeRetirement {
                    model: RuntimeLogicalModel::Relational,
                    reference: record("b").reference,
                    effective_at: 200,
                },
            },
        ],
    };
    let (resolved, _) = storage
        .runtime()
        .resolve_relation_actions(
            &DataTransaction::new(read.clone(), commit).unwrap(),
            RuntimeReadBudget::new(1024).unwrap(),
        )
        .unwrap();
    assert_eq!(resolved.mutations.len(), 6);
    assert!(
        matches!(&resolved.mutations[5], RuntimeMutation::Retire { retirement }
        if retirement.effective_at == 300)
    );
    storage
        .runtime()
        .commit_data_transaction(&DataTransaction::new(read, resolved).unwrap())
        .unwrap();
    let after = storage.runtime().read_stamp(&scope).unwrap();
    assert!(!snapshot(&storage, &after, 300).contains(
        RuntimeLogicalModel::GraphRelation,
        &relation("future", "b", 300).reference
    ));
}

#[test]
fn open_ended_record_updates_add_coverage_but_finite_corrections_check_incident_history() {
    let storage = RrflowMxStore::new();
    let fixture = commit_fixture(&storage);
    let read = storage.runtime().read_stamp(&fixture.scope).unwrap();
    let mut commit = RuntimeCommit {
        scope: fixture.scope,
        at: 500,
        actor: "agent:coverage-proof".into(),
        expected_cursor: read.commit_cursor,
        mutations: vec![RuntimeMutation::Record {
            record: RuntimeRecord {
                valid_from: 150,
                ..record("b")
            },
        }],
    };
    // Adding an unbounded version cannot uncover an endpoint interval. No
    // graph selection is needed even with an operator key budget of one.
    let (resolved, evidence) = storage
        .runtime()
        .resolve_relation_actions(
            &DataTransaction::new(read.clone(), commit.clone()).unwrap(),
            RuntimeReadBudget::new(1).unwrap(),
        )
        .unwrap();
    assert_eq!(resolved, commit);
    assert!(evidence.is_none());
    if let RuntimeMutation::Record { record } = &mut commit.mutations[0] {
        record.valid_to = Some(180);
    }
    assert!(matches!(
        storage.runtime().commit(&commit),
        Err(Error::DanglingRuntimeReference(_))
    ));
    assert_eq!(storage.runtime().read_stamp(&read.scope).unwrap(), read);
}
