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

fn physical_rows(store: &dyn StorageEngine) -> Vec<(Vec<u8>, Vec<u8>)> {
    let transaction = store.begin_transaction().unwrap();
    let rows = transaction.scan(&[0], &[0xff], 4096).unwrap();
    assert!(rows.len() < 4096, "oracle must include every fixture row");
    rows
}

fn checkpoint_commit(store: &dyn StorageEngine) -> rrd_core::ReadStamp {
    use rrd_core::{RuntimeLogicalModel, RuntimeRecordSchema, RuntimeSchemaRegistry, RuntimeType};
    let mut commit = bootstrap_commit();
    let mut registry = RuntimeSchemaRegistry::empty(1, "checkpoint composition fixture");
    registry
        .define_record_table(
            RuntimeType::new("file").unwrap(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema::default(),
        )
        .unwrap();
    commit
        .mutations
        .insert(0, RuntimeMutation::Schema { registry });
    commit.mutations.push(RuntimeMutation::Record {
        record: rrd_core::RuntimeRecord {
            reference: rrd_core::RuntimeRef::new("file", "src/lib.rs").unwrap(),
            valid_from: 100,
            valid_to: None,
            properties: rrd_core::RuntimeProperties::new(),
        },
    });
    let prior = store.runtime().read_stamp(&commit.scope).unwrap();
    let mut calls = 0;
    let (outcome, entries, staged) = store
        .runtime()
        .commit_with_control_builder(&commit, |read, outcome| {
            calls += 1;
            read.validate().unwrap();
            assert_eq!(read.commit_cursor, outcome.last_cursor);
            assert_eq!(read.schema_revision, Some(1));
            assert_eq!(outcome.commit_id, commit.digest());
            assert_ne!(read.manifest_id, prior.manifest_id);
            let mut transition = installed_transition(None);
            transition.replacement = Some(serde_json::to_vec(read)?);
            Ok::<_, rrd_store::Error>((vec![transition], read.clone()))
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(outcome.last_cursor, commit.mutations.len() as u64);
    assert_eq!(entries.len(), 1);
    assert_eq!(staged, store.runtime().read_stamp(&commit.scope).unwrap());
    let stored: rrd_core::ReadStamp = serde_json::from_slice(
        &store
            .control()
            .get("server/state/install/record")
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(stored, staged);
    assert_eq!(
        store
            .runtime()
            .data_snapshot(&commit.scope, 100, 4096)
            .unwrap()
            .snapshot
            .records
            .len(),
        1
    );
    staged
}

fn rejected_checkpoint_builds_preserve_all_rows(store: &dyn StorageEngine) {
    let before = physical_rows(store);
    let commit = bootstrap_commit();
    let error = store
        .runtime()
        .commit_with_control_builder(&commit, |read, _| {
            assert_eq!(read.commit_cursor, 1, "runtime was staged before rejection");
            Err::<(Vec<ControlTransition>, ()), _>(rrd_store::Error::Codec(
                "checkpoint validation rejected".into(),
            ))
        })
        .unwrap_err();
    assert!(matches!(error, rrd_store::Error::Codec(_)));
    assert_eq!(physical_rows(store), before);
    let error = store
        .runtime()
        .commit_with_control_builder(&commit, |_, _| {
            Ok::<_, rrd_store::Error>((
                vec![installed_transition(Some(b"stale-control".to_vec()))],
                (),
            ))
        })
        .unwrap_err();
    assert!(matches!(error, rrd_store::Error::ControlConflict(_)));
    assert_eq!(physical_rows(store), before);
    assert!(store
        .runtime()
        .commit_outcome(&commit.digest())
        .unwrap()
        .is_none());
    assert!(store.runtime().audit(&commit.digest()).unwrap().is_none());
}

#[test]
fn checkpoint_uses_resulting_runtime_stamp_on_mx() {
    let store = rrd_store::RrflowMxStore::new();
    rejected_checkpoint_builds_preserve_all_rows(&store);
    checkpoint_commit(&store);
}

#[test]
fn checkpoint_and_resulting_runtime_stamp_share_one_wal_frame_and_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("checkpoint");
    let expected = {
        let store = RrflowKvStore::create_new(&root).unwrap();
        rejected_checkpoint_builds_preserve_all_rows(&store);
        checkpoint_commit(&store)
    };
    let recovery = rrd_lsm::recover(&root.join("wal/00000000000000000001.wal")).unwrap();
    assert_eq!(
        recovery.batches.len(),
        1,
        "rejections publish nothing; success is one frame"
    );
    let reopened = RrflowKvStore::open_existing(&root).unwrap();
    let actual = reopened.runtime().read_stamp(&expected.scope).unwrap();
    assert_eq!(actual, expected);
    let checkpoint: rrd_core::ReadStamp = serde_json::from_slice(
        &reopened
            .control()
            .get("server/state/install/record")
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(checkpoint, actual);
    assert_eq!(reopened.control().sequence().unwrap(), 1);
}

// Characterizes the transaction and inclusion primitives considered for
// inventory selection. This does not prove that an event range is complete or
// that the latest event is a valid inventory selection.
#[test]
fn selection_event_and_checkpoint_are_stamped_together_across_reopen() {
    use rrd_core::{
        RuntimeEvent, RuntimeEventSchema, RuntimeLogicalModel, RuntimeProperties,
        RuntimeSchemaRegistry, RuntimeType, RuntimeValue,
    };
    use rrd_store::{Error, RuntimeReadBudget, RuntimeVersionedSource};

    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("selection");
    let scope = ScopeId::new("instance:selection-fixture").unwrap();
    let event_kind = RuntimeType::new("selection-fixture").unwrap();
    let event = |generation: &str| RuntimeMutation::Event {
        event: RuntimeEvent {
            kind: event_kind.clone(),
            subject: None,
            properties: RuntimeProperties::from([(
                "generation".into(),
                RuntimeValue::String(generation.into()),
            )]),
        },
    };
    let mut schema = RuntimeSchemaRegistry::empty(1, "selection fixture");
    schema
        .define_event_table(
            event_kind.clone(),
            RuntimeLogicalModel::Event,
            RuntimeEventSchema {
                allow_additional_properties: true,
                ..Default::default()
            },
        )
        .unwrap();
    let store = RrflowKvStore::create_new(&root).unwrap();
    let first = RuntimeCommit {
        scope: scope.clone(),
        at: 100,
        actor: "test:selection".into(),
        expected_cursor: 0,
        mutations: vec![RuntimeMutation::Schema { registry: schema }, event("first")],
    };
    let (_, _, first_read) = store
        .runtime()
        .commit_with_control_builder(&first, |read, _| {
            let mut transition = installed_transition(None);
            transition.replacement = Some(serde_json::to_vec(read)?);
            Ok::<_, Error>((vec![transition], read.clone()))
        })
        .unwrap();
    let first_bytes = serde_json::to_vec(&first_read).unwrap();
    let second = RuntimeCommit {
        scope: scope.clone(),
        at: 200,
        actor: "test:selection".into(),
        expected_cursor: first_read.commit_cursor,
        mutations: vec![event("second")],
    };
    let rejected = store
        .runtime()
        .commit_with_control_builder(&second, |read, _| {
            let mut transition = installed_transition(Some(b"stale".to_vec()));
            transition.replacement = Some(serde_json::to_vec(read)?);
            Ok::<_, Error>((vec![transition], ()))
        });
    assert!(matches!(rejected, Err(Error::ControlConflict(_))));
    assert_eq!(store.runtime().read_stamp(&scope).unwrap(), first_read);
    assert_eq!(
        store.control().get("server/state/install/record").unwrap(),
        Some(first_bytes.clone())
    );

    let (_, _, second_read) = store
        .runtime()
        .commit_with_control_builder(&second, |read, _| {
            let mut transition = installed_transition(Some(first_bytes));
            transition.replacement = Some(serde_json::to_vec(read)?);
            Ok::<_, Error>((vec![transition], read.clone()))
        })
        .unwrap();
    let later = RuntimeCommit {
        scope: scope.clone(),
        at: 300,
        actor: "test:unrelated".into(),
        expected_cursor: second_read.commit_cursor,
        mutations: vec![RuntimeMutation::Claim {
            claim: Claim::new(
                Subject::new("unrelated:subject").unwrap(),
                Predicate::new("unrelated-status").unwrap(),
                "newer",
                300,
                300,
                Producer {
                    actor: "test:unrelated".into(),
                    on_behalf_of: None,
                    session: None,
                },
            ),
        }],
    };
    store.runtime().commit(&later).unwrap();
    let later_read = store.runtime().read_stamp(&scope).unwrap();
    drop(store);

    let reopened = RrflowKvStore::open_existing(&root).unwrap();
    let saved: rrd_core::ReadStamp = serde_json::from_slice(
        &reopened
            .control()
            .get("server/state/install/record")
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(saved, second_read);
    assert_eq!(reopened.runtime().read_stamp(&scope).unwrap(), later_read);
    let source = RuntimeVersionedSource::Events {
        kind: Some(event_kind),
    };
    let values = |read: &rrd_core::ReadStamp| {
        reopened
            .runtime()
            .read_versioned(
                read,
                std::slice::from_ref(&source),
                RuntimeReadBudget::new(4096).unwrap(),
            )
            .unwrap()
            .changes
            .into_iter()
            .filter_map(|change| match change.mutation {
                RuntimeMutation::Event { event } => event.properties.get("generation").cloned(),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        values(&first_read),
        vec![RuntimeValue::String("first".into())]
    );
    assert_eq!(
        values(&second_read),
        vec![
            RuntimeValue::String("first".into()),
            RuntimeValue::String("second".into())
        ]
    );
    assert_eq!(values(&later_read), values(&second_read));
    let unrelated = reopened
        .runtime()
        .read_versioned(
            &later_read,
            &[RuntimeVersionedSource::Claims {
                predicate: Some(Predicate::new("unrelated-status").unwrap()),
            }],
            RuntimeReadBudget::new(4096).unwrap(),
        )
        .unwrap();
    assert!(unrelated.changes.iter().any(|change| matches!(
        &change.mutation,
        RuntimeMutation::Claim { claim } if claim.subject.as_str() == "unrelated:subject"
    )));
    assert!(matches!(
        reopened.runtime().read_versioned(
            &second_read,
            &[source],
            RuntimeReadBudget::new(1).unwrap()
        ),
        Err(Error::RuntimeReadBudgetExceeded { .. })
    ));
}
