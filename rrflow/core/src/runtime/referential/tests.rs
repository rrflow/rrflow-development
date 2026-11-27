use super::*;
use crate::{RuntimeRecordSchema, RuntimeRelationSchema};

fn record(id: &str, from: u64, to: Option<u64>) -> RuntimeMutation {
    RuntimeMutation::Record {
        record: RuntimeRecord {
            reference: RuntimeRef::new("node", id).unwrap(),
            valid_from: from,
            valid_to: to,
            properties: RuntimeProperties::new(),
        },
    }
}

fn edge(from: u64) -> RuntimeMutation {
    RuntimeMutation::Relation {
        relation: RuntimeRelation {
            reference: RuntimeRef::new("link", "ab").unwrap(),
            from: RuntimeRef::new("node", "a").unwrap(),
            to: RuntimeRef::new("node", "b").unwrap(),
            valid_from: from,
            valid_to: None,
            properties: RuntimeProperties::new(),
        },
    }
}

fn retire(model: RuntimeLogicalModel, kind: &str, id: &str, at: u64) -> RuntimeMutation {
    RuntimeMutation::Retire {
        retirement: RuntimeRetirement {
            model,
            reference: RuntimeRef::new(kind, id).unwrap(),
            effective_at: at,
        },
    }
}

fn commit(mutations: Vec<RuntimeMutation>, cursor: u64) -> RuntimeCommit {
    RuntimeCommit {
        scope: crate::ScopeId::new("project:references").unwrap(),
        at: 1000,
        actor: "agent:references".into(),
        expected_cursor: cursor,
        mutations,
    }
}

fn history(mutations: Vec<RuntimeMutation>) -> Vec<RuntimeChange> {
    let base = commit(mutations, 0);
    base.mutations
        .iter()
        .enumerate()
        .map(|(i, mutation)| {
            RuntimeChange::committed(
                i as u64 + 1,
                &base,
                &base.digest(),
                i as u64,
                mutation.clone(),
                None,
            )
        })
        .collect()
}

fn schema(action: Option<RuntimeReferentialAction>) -> RuntimeSchemaRegistry {
    let mut schema = RuntimeSchemaRegistry::empty(1, "temporal references");
    schema
        .define_record_table(
            RuntimeType::new("node").unwrap(),
            RuntimeLogicalModel::GraphNode,
            RuntimeRecordSchema::default(),
        )
        .unwrap();
    schema
        .define_relation_table(
            RuntimeType::new("link").unwrap(),
            RuntimeRelationSchema {
                on_delete_from: action,
                on_delete_to: action,
                ..Default::default()
            },
        )
        .unwrap();
    schema
}

#[test]
fn interval_sweep_matches_existing_bitemporal_snapshot_reducer() {
    let changes = history(vec![
        record("a", 10, None),
        record("a", 30, Some(50)),
        record("a", 20, Some(25)),
        record("a", 80, None),
        retire(RuntimeLogicalModel::GraphNode, "node", "a", 90),
        record("a", 95, Some(100)),
        record("a", 60, Some(70)),
    ]);
    for known in 1..=changes.len() as u64 {
        let mut records = BTreeMap::new();
        let mut relations = BTreeMap::new();
        for change in changes.iter().filter(|change| change.cursor <= known) {
            add_version(
                &mut records,
                &mut relations,
                &change.mutation,
                change.cursor,
            );
        }
        let intervals = records
            .remove(&RuntimeRef::new("node", "a").unwrap())
            .unwrap()
            .finish();
        for valid in 0..=110 {
            let snapshot =
                RuntimeGraphSnapshot::from_changes(&changes, commit(vec![], 0).scope, valid, known);
            let present = intervals
                .iter()
                .any(|part| part.from <= valid && part.to.is_none_or(|end| valid < end));
            assert_eq!(
                present,
                !snapshot.records.is_empty(),
                "valid={valid} known={known}"
            );
        }
    }
}

#[test]
fn reject_cascade_and_explicit_retirement_preserve_historical_truth() {
    let changes = history(vec![record("a", 10, None), record("b", 10, None), edge(10)]);
    let mut deletion = commit(
        vec![retire(RuntimeLogicalModel::GraphNode, "node", "b", 20)],
        3,
    );
    assert!(relation_referential_actions(&schema(None), &changes, &deletion).is_err());
    let cascade = schema(Some(RuntimeReferentialAction::Cascade));
    let actions = relation_referential_actions(&cascade, &changes, &deletion).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].reference, RuntimeRef::new("link", "ab").unwrap());
    assert_eq!(actions[0].effective_at, 20);
    deletion.mutations.push(RuntimeMutation::Retire {
        retirement: actions[0].clone(),
    });
    assert!(relation_referential_actions(&cascade, &changes, &deletion)
        .unwrap()
        .is_empty());
    let old = RuntimeGraphSnapshot::from_changes(&changes, deletion.scope.clone(), 30, 3);
    assert_eq!((old.records.len(), old.relations.len()), (2, 1));
    let mut accepted = changes;
    for (i, mutation) in deletion.mutations.iter().enumerate() {
        accepted.push(RuntimeChange::committed(
            4 + i as u64,
            &deletion,
            &deletion.digest(),
            i as u64,
            mutation.clone(),
            None,
        ));
    }
    let before = RuntimeGraphSnapshot::from_changes(&accepted, deletion.scope.clone(), 19, 5);
    let after = RuntimeGraphSnapshot::from_changes(&accepted, deletion.scope.clone(), 20, 5);
    assert_eq!((before.records.len(), before.relations.len()), (2, 1));
    assert_eq!((after.records.len(), after.relations.len()), (1, 0));
    assert_eq!(
        RuntimeGraphSnapshot::from_changes(&accepted, deletion.scope, 30, 3),
        old
    );
}

#[test]
fn future_edges_and_self_loop_rejection_are_not_hidden_by_cascade() {
    let changes = history(vec![record("a", 10, None), record("b", 10, None), edge(30)]);
    let deletion = commit(
        vec![retire(RuntimeLogicalModel::GraphNode, "node", "b", 20)],
        3,
    );
    let actions = relation_referential_actions(
        &schema(Some(RuntimeReferentialAction::Cascade)),
        &changes,
        &deletion,
    )
    .unwrap();
    assert_eq!(actions[0].effective_at, 30);
    let mut self_loop = edge(10);
    if let RuntimeMutation::Relation { relation } = &mut self_loop {
        relation.to = relation.from.clone();
    }
    let changes = history(vec![record("a", 10, None), self_loop]);
    let mut mixed = schema(Some(RuntimeReferentialAction::Cascade));
    mixed
        .relations
        .get_mut(&RuntimeType::new("link").unwrap())
        .unwrap()
        .on_delete_to = Some(RuntimeReferentialAction::Reject);
    assert!(relation_referential_actions(
        &mixed,
        &changes,
        &commit(
            vec![retire(RuntimeLogicalModel::GraphNode, "node", "a", 20)],
            2
        )
    )
    .is_err());
}

#[test]
fn finite_endpoint_window_does_not_allow_an_unbounded_relation() {
    let proposed = commit(
        vec![record("a", 10, None), record("b", 10, Some(30)), edge(10)],
        0,
    );
    assert!(relation_referential_actions(
        &schema(Some(RuntimeReferentialAction::Cascade)),
        &[],
        &proposed
    )
    .is_err());
}

#[test]
fn explicit_endpoint_rules_are_part_of_schema_identity() {
    let implicit = schema(None);
    let explicit = schema(Some(RuntimeReferentialAction::Reject));
    let cascade = schema(Some(RuntimeReferentialAction::Cascade));
    let digest = |registry| commit(vec![RuntimeMutation::Schema { registry }], 0).digest();
    assert_ne!(digest(implicit), digest(explicit.clone()));
    assert_ne!(digest(explicit), digest(cascade));
}
