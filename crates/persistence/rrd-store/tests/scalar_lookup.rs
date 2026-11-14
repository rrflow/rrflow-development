//! A scalar commit-bound index narrows a read to exactly the current records
//! whose indexed values match, through edit and retirement, and refuses a
//! valid time its current entries cannot answer.
use rrd_core::{
    digest, ProjectionId, RuntimeCommit, RuntimeLogicalModel, RuntimeMutation,
    RuntimePropertySchema, RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeRetirement,
    RuntimeSchemaRegistry, RuntimeType, RuntimeValue, RuntimeValueType, ScopeId,
};
use rrd_store::{
    ControlTransition, IndexCommitBindingDefinition, IndexCommitBindingKind, IndexLookup,
    RrflowKvStore, RrflowMxStore, StorageEngine,
};
use std::collections::BTreeMap;

fn scope() -> ScopeId {
    ScopeId::new("project:scalar-lookup").unwrap()
}

fn index() -> ProjectionId {
    ProjectionId::new("symbol-name").unwrap()
}

fn symbol(id: &str, name: &str, valid_from: u64) -> RuntimeMutation {
    RuntimeMutation::Record {
        record: RuntimeRecord {
            reference: RuntimeRef::new("symbol", id).unwrap(),
            valid_from,
            valid_to: None,
            properties: BTreeMap::from([("name".into(), RuntimeValue::String(name.into()))]),
        },
    }
}

fn commit(engine: &dyn StorageEngine, at: u64, mutations: Vec<RuntimeMutation>) {
    let cursor = engine.runtime().read_stamp(&scope()).unwrap().commit_cursor;
    engine
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope(),
            at,
            actor: "test:scalar".into(),
            expected_cursor: cursor,
            mutations,
        })
        .unwrap();
}

fn lookup(engine: &dyn StorageEngine, name: &str, valid_at: u64) -> IndexLookup {
    let read = engine.runtime().read_stamp(&scope()).unwrap();
    engine
        .runtime()
        .index_lookup(
            &read,
            &index(),
            &[RuntimeValue::String(name.into())],
            valid_at,
            100,
        )
        .unwrap()
}

fn ids(lookup: IndexLookup) -> Vec<String> {
    match lookup {
        IndexLookup::Records { references, .. } => references
            .into_iter()
            .map(|reference| reference.id.to_string())
            .collect(),
        IndexLookup::NotApplicable(reason) => panic!("not applicable: {reason}"),
    }
}

fn exercise(engine: &dyn StorageEngine) {
    let mut schema = RuntimeSchemaRegistry::empty(1, "scalar lookup fixture");
    schema
        .define_record_table(
            RuntimeType::new("symbol").unwrap(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema {
                properties: BTreeMap::from([(
                    "name".into(),
                    RuntimePropertySchema::required(RuntimeValueType::String),
                )]),
                ..RuntimeRecordSchema::default()
            },
        )
        .unwrap();
    commit(
        engine,
        1,
        vec![
            RuntimeMutation::Schema { registry: schema },
            symbol("a", "control_attunement", 10),
            symbol("b", "control_attunement", 10),
            symbol("c", "step", 10),
        ],
    );
    engine
        .control()
        .commit_catalog_with_index_bindings(
            &scope(),
            1,
            &ControlTransition {
                key: format!("server/state/index-catalogue/{}", scope()),
                expected: None,
                replacement: Some(br#"{"contract":"scalar-fixture-v1"}"#.to_vec()),
                at: 2,
                actor: "test:scalar".into(),
                action: "index_catalogue.installed".into(),
                request_id: "request-scalar".into(),
                operation_id: "operation-scalar".into(),
            },
            &[IndexCommitBindingDefinition {
                id: index(),
                record_kind: RuntimeType::new("symbol").unwrap(),
                fields: vec!["name".into()],
                kind: IndexCommitBindingKind::Scalar { unique: false },
                configuration_sha256: digest::sha256_hex(b"symbol-name:scalar"),
                lexical: None,
            }],
        )
        .unwrap();
    // Backfilled entries answer before any later commit.
    assert_eq!(ids(lookup(engine, "control_attunement", 100)), ["a", "b"]);
    assert_eq!(ids(lookup(engine, "step", 100)), ["c"]);
    assert!(ids(lookup(engine, "absent", 100)).is_empty());

    // A rename moves the record between values in the same commit.
    commit(engine, 20, vec![symbol("b", "step", 20)]);
    assert_eq!(ids(lookup(engine, "control_attunement", 100)), ["a"]);
    assert_eq!(ids(lookup(engine, "step", 100)), ["b", "c"]);
    // Before the rename the old name was visible, which current entries do
    // not hold, so that valid time is refused rather than answered wrongly.
    assert!(matches!(
        lookup(engine, "control_attunement", 15),
        IndexLookup::NotApplicable(_)
    ));

    // A retirement removes the entry and raises the watermark.
    commit(
        engine,
        30,
        vec![RuntimeMutation::Retire {
            retirement: RuntimeRetirement {
                model: RuntimeLogicalModel::Relational,
                reference: RuntimeRef::new("symbol", "a").unwrap(),
                effective_at: 30,
            },
        }],
    );
    assert!(ids(lookup(engine, "control_attunement", 100)).is_empty());
    assert!(matches!(
        lookup(engine, "step", 25),
        IndexLookup::NotApplicable(_)
    ));

    // A stamp behind the head is not answered from current entries.
    let stale = engine.runtime().read_stamp(&scope()).unwrap();
    commit(engine, 40, vec![symbol("d", "step", 40)]);
    assert!(matches!(
        engine
            .runtime()
            .index_lookup(
                &stale,
                &index(),
                &[RuntimeValue::String("step".into())],
                100,
                100
            )
            .unwrap(),
        IndexLookup::NotApplicable(_)
    ));
    assert_eq!(ids(lookup(engine, "step", 100)), ["b", "c", "d"]);
    // Before the newest version became valid the visible set differed.
    assert!(matches!(
        lookup(engine, "step", 35),
        IndexLookup::NotApplicable(_)
    ));
}

#[test]
fn scalar_lookup_selects_exactly_the_current_matching_records() {
    exercise(&RrflowMxStore::new());
    let directory = tempfile::tempdir().unwrap();
    exercise(&RrflowKvStore::create_new(&directory.path().join("kv")).unwrap());
}
