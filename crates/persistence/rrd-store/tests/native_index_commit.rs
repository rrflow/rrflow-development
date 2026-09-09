use rrd_core::{
    digest, DataTransaction, ProjectionId, RuntimeCommit, RuntimeLogicalModel, RuntimeMutation,
    RuntimePropertySchema, RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeRetirement,
    RuntimeSchemaRegistry, RuntimeTableSchema, RuntimeType, RuntimeValue, RuntimeValueType,
    RuntimeVector, ScopeId, VectorCollectionAddress, VectorValue,
};
use rrd_store::{
    ControlTransition, Error, IndexCommitBindingDefinition, IndexCommitBindingKind,
    IndexSourceDelta, RrflowKvStore, RrflowMxStore, StorageEngine, VectorSourceAddress,
    VectorSourceDelta,
};
use std::collections::BTreeMap;

fn scope() -> ScopeId {
    ScopeId::new("project:native-index-commit").unwrap()
}

fn reference(kind: &str, id: &str) -> RuntimeRef {
    RuntimeRef::new(kind, id).unwrap()
}

fn schema() -> RuntimeSchemaRegistry {
    let mut schema = RuntimeSchemaRegistry::empty(1, "install native index fixture");
    schema.records.insert(
        RuntimeType::new("document").unwrap(),
        RuntimeRecordSchema {
            properties: BTreeMap::from([
                (
                    "status".into(),
                    RuntimePropertySchema::required(RuntimeValueType::String),
                ),
                (
                    "title".into(),
                    RuntimePropertySchema::required(RuntimeValueType::String),
                ),
            ]),
            ..RuntimeRecordSchema::default()
        },
    );
    schema.tables.insert(
        RuntimeType::new("embedding").unwrap(),
        RuntimeTableSchema::schemaless(RuntimeLogicalModel::Vector),
    );
    schema
}

fn record(id: &str, status: &str, title: &str, valid_from: u64) -> RuntimeRecord {
    RuntimeRecord {
        reference: reference("document", id),
        valid_from,
        valid_to: None,
        properties: BTreeMap::from([
            ("status".into(), RuntimeValue::String(status.into())),
            ("title".into(), RuntimeValue::String(title.into())),
        ]),
    }
}

fn bootstrap(engine: &dyn StorageEngine, duplicate: bool) {
    engine
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope(),
            at: 1,
            actor: "test:native-index".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry: schema() },
                RuntimeMutation::Record {
                    record: record("alpha", "open", "Alpha", 10),
                },
                RuntimeMutation::Record {
                    record: record(
                        "beta",
                        if duplicate { "open" } else { "closed" },
                        "Beta",
                        10,
                    ),
                },
            ],
        })
        .unwrap();
}

fn binding(id: &str, field: &str, kind: IndexCommitBindingKind) -> IndexCommitBindingDefinition {
    IndexCommitBindingDefinition {
        id: ProjectionId::new(id).unwrap(),
        record_kind: RuntimeType::new("document").unwrap(),
        fields: vec![field.into()],
        kind,
        configuration_sha256: digest::sha256_hex(format!("{id}:{field}:{kind:?}").as_bytes()),
    }
}

fn bindings() -> Vec<IndexCommitBindingDefinition> {
    vec![
        binding(
            "document-status",
            "status",
            IndexCommitBindingKind::Scalar { unique: false },
        ),
        binding(
            "document-status-unique",
            "status",
            IndexCommitBindingKind::Scalar { unique: true },
        ),
        binding("document-title-bm25", "title", IndexCommitBindingKind::Bm25),
    ]
}

fn install_bindings(engine: &dyn StorageEngine) -> rrd_store::Result<()> {
    let key = format!("server/state/index-catalogue/{}", scope());
    engine
        .control()
        .commit_catalog_with_index_bindings(
            &scope(),
            1,
            &ControlTransition {
                key,
                expected: None,
                replacement: Some(br#"{"contract":"native-index-fixture-v1"}"#.to_vec()),
                at: 2,
                actor: "test:native-index".into(),
                action: "index_catalogue.installed".into(),
                request_id: "request-native-index".into(),
                operation_id: "operation-native-index".into(),
            },
            &bindings(),
        )
        .map(|_| ())
}

fn commit_at_read(
    engine: &dyn StorageEngine,
    at: u64,
    mutations: Vec<RuntimeMutation>,
) -> rrd_store::Result<rrd_core::RuntimeCommitOutcome> {
    let read = engine.runtime().read_stamp(&scope())?;
    engine
        .runtime()
        .commit_data_transaction(&DataTransaction::new(
            read.clone(),
            RuntimeCommit {
                scope: scope(),
                at,
                actor: "test:native-index".into(),
                expected_cursor: read.commit_cursor,
                mutations,
            },
        )?)
}

fn vector(collection: &str, value: [f32; 2], valid_from: u64) -> RuntimeVector {
    RuntimeVector {
        reference: reference("embedding", "alpha-title"),
        subject: reference("document", "alpha"),
        collection: Some(VectorCollectionAddress {
            collection_id: collection.into(),
            vector_name: "semantic".into(),
        }),
        field: "title".into(),
        valid_from,
        valid_to: None,
        value: VectorValue::Dense {
            values: value.into(),
        },
        provenance: None,
        properties: BTreeMap::new(),
    }
}

#[derive(Debug, PartialEq)]
struct ExerciseResult {
    scalar: Vec<IndexSourceDelta>,
    unique: Vec<IndexSourceDelta>,
    bm25: Vec<IndexSourceDelta>,
    original_vector: Vec<VectorSourceDelta>,
    moved_vector: Vec<VectorSourceDelta>,
    final_cursor: u64,
}

fn exercise(engine: &dyn StorageEngine) -> ExerciseResult {
    bootstrap(engine, false);
    install_bindings(engine).unwrap();
    assert_eq!(
        engine
            .runtime()
            .read_stamp(&scope())
            .unwrap()
            .catalog_revision,
        1
    );
    let mut revised_schema = schema();
    revised_schema.revision = 2;
    revised_schema.migration = "revalidate native index bindings".into();
    commit_at_read(
        engine,
        3,
        vec![RuntimeMutation::Schema {
            registry: revised_schema,
        }],
    )
    .unwrap();

    commit_at_read(
        engine,
        10,
        vec![RuntimeMutation::Record {
            record: record("alpha", "pending", "Alpha", 20),
        }],
    )
    .unwrap();
    let cursor_before_conflict = engine.runtime().cursor().unwrap();
    assert!(matches!(
        commit_at_read(
            engine,
            11,
            vec![RuntimeMutation::Record {
                record: record("gamma", "pending", "Gamma", 20),
            }],
        ),
        Err(Error::IndexConstraint(message))
            if message.contains("rejects overlapping duplicate records")
    ));
    assert_eq!(engine.runtime().cursor().unwrap(), cursor_before_conflict);

    // Transaction-final validation permits an atomic value swap without
    // exposing either mutation's intermediate duplicate state.
    commit_at_read(
        engine,
        12,
        vec![
            RuntimeMutation::Record {
                record: record("alpha", "closed", "Alpha", 30),
            },
            RuntimeMutation::Record {
                record: record("beta", "pending", "Beta", 30),
            },
        ],
    )
    .unwrap();
    commit_at_read(
        engine,
        13,
        vec![RuntimeMutation::Retire {
            retirement: RuntimeRetirement {
                model: RuntimeLogicalModel::Relational,
                reference: reference("document", "beta"),
                effective_at: 40,
            },
        }],
    )
    .unwrap();

    commit_at_read(
        engine,
        14,
        vec![RuntimeMutation::Vector {
            vector: vector("documents", [1.0, 0.0], 50),
        }],
    )
    .unwrap();
    commit_at_read(
        engine,
        15,
        vec![RuntimeMutation::Vector {
            vector: vector("archive", [0.0, 1.0], 60),
        }],
    )
    .unwrap();
    commit_at_read(
        engine,
        16,
        vec![RuntimeMutation::Retire {
            retirement: RuntimeRetirement {
                model: RuntimeLogicalModel::Vector,
                reference: reference("embedding", "alpha-title"),
                effective_at: 70,
            },
        }],
    )
    .unwrap();

    let original_source = VectorSourceAddress {
        collection: Some(VectorCollectionAddress {
            collection_id: "documents".into(),
            vector_name: "semantic".into(),
        }),
        field: "title".into(),
    };
    let moved_source = VectorSourceAddress {
        collection: Some(VectorCollectionAddress {
            collection_id: "archive".into(),
            vector_name: "semantic".into(),
        }),
        field: "title".into(),
    };
    ExerciseResult {
        scalar: engine
            .runtime()
            .index_source_deltas_since(
                &scope(),
                &ProjectionId::new("document-status").unwrap(),
                0,
                16,
            )
            .unwrap(),
        unique: engine
            .runtime()
            .index_source_deltas_since(
                &scope(),
                &ProjectionId::new("document-status-unique").unwrap(),
                0,
                16,
            )
            .unwrap(),
        bm25: engine
            .runtime()
            .index_source_deltas_since(
                &scope(),
                &ProjectionId::new("document-title-bm25").unwrap(),
                0,
                16,
            )
            .unwrap(),
        original_vector: engine
            .runtime()
            .vector_source_deltas_since(&scope(), &original_source, 0, 16)
            .unwrap(),
        moved_vector: engine
            .runtime()
            .vector_source_deltas_since(&scope(), &moved_source, 0, 16)
            .unwrap(),
        final_cursor: engine.runtime().cursor().unwrap(),
    }
}

#[test]
fn rrflow_mx_and_rrflow_kv_commit_identical_native_index_and_vector_deltas() {
    let directory = tempfile::tempdir().unwrap();
    let mx = exercise(&RrflowMxStore::new());
    let kv = exercise(&RrflowKvStore::open(&directory.path().join("rrflow-kv")).unwrap());
    assert_eq!(mx, kv);
    assert_eq!(mx.scalar.len(), 4);
    assert_eq!(mx.unique.len(), 4);
    assert_eq!(mx.bm25.len(), 4);
    assert_eq!(mx.original_vector.len(), 2);
    assert!(mx.original_vector[0].before.is_none());
    assert!(mx.original_vector[0].after.is_some());
    assert!(mx.original_vector[1].before.is_some());
    assert!(mx.original_vector[1].after.is_none());
    assert_eq!(mx.moved_vector.len(), 2);
    assert!(mx.moved_vector[0].before.is_none());
    assert!(mx.moved_vector[0].after.is_some());
    assert!(mx.moved_vector[1].before.is_some());
    assert!(mx.moved_vector[1].after.is_none());
    assert!(mx.scalar.iter().all(|delta| delta.schema_revision == 2));
}

#[test]
fn unique_backfill_failure_is_atomic_on_every_storage_profile() {
    fn assert_profile(engine: &dyn StorageEngine) {
        bootstrap(engine, true);
        let cursor = engine.runtime().cursor().unwrap();
        assert!(matches!(
            install_bindings(engine),
            Err(Error::IndexConstraint(message))
                if message.contains("rejects overlapping duplicate records")
        ));
        assert_eq!(engine.runtime().cursor().unwrap(), cursor);
        assert_eq!(
            engine
                .runtime()
                .read_stamp(&scope())
                .unwrap()
                .catalog_revision,
            0
        );
        assert!(engine
            .control()
            .get(&format!("server/state/index-catalogue/{}", scope()))
            .unwrap()
            .is_none());
    }

    let directory = tempfile::tempdir().unwrap();
    assert_profile(&RrflowMxStore::new());
    assert_profile(&RrflowKvStore::open(&directory.path().join("rrflow-kv")).unwrap());
}

#[test]
fn rrflow_kv_reopens_commit_bindings_constraints_and_source_deltas() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv");
    let expected = {
        let engine = RrflowKvStore::open(&path).unwrap();
        exercise(&engine)
    };
    let reopened = RrflowKvStore::open(&path).unwrap();
    let scalar = reopened
        .runtime()
        .index_source_deltas_since(
            &scope(),
            &ProjectionId::new("document-status").unwrap(),
            0,
            16,
        )
        .unwrap();
    assert_eq!(scalar, expected.scalar);
    assert_eq!(reopened.runtime().cursor().unwrap(), expected.final_cursor);
    assert!(matches!(
        commit_at_read(
            &reopened,
            100,
            vec![RuntimeMutation::Record {
                record: record("gamma", "closed", "Gamma", 30),
            }],
        ),
        Err(Error::IndexConstraint(_))
    ));
}

#[test]
fn generic_catalogue_replacement_cannot_bypass_commit_bound_indexes() {
    fn assert_profile(engine: &dyn StorageEngine) {
        bootstrap(engine, false);
        install_bindings(engine).unwrap();
        let key = format!("server/state/index-catalogue/{}", scope());
        let original = br#"{"contract":"native-index-fixture-v1"}"#.to_vec();
        engine
            .control()
            .commit_catalog(
                &scope(),
                &ControlTransition {
                    key,
                    expected: Some(original),
                    replacement: Some(br#"{"contract":"generic-bypass-attempt"}"#.to_vec()),
                    at: 3,
                    actor: "test:native-index".into(),
                    action: "index_catalogue.generic_replacement".into(),
                    request_id: "request-generic-replacement".into(),
                    operation_id: "operation-generic-replacement".into(),
                },
            )
            .unwrap();
        let cursor = engine.runtime().cursor().unwrap();
        assert!(matches!(
            commit_at_read(
                engine,
                4,
                vec![RuntimeMutation::Record {
                    record: record("alpha", "pending", "Alpha", 20),
                }],
            ),
            Err(Error::IndexConstraint(message))
                if message.contains("does not match its accepted catalogue and schema")
        ));
        assert_eq!(engine.runtime().cursor().unwrap(), cursor);
    }

    let directory = tempfile::tempdir().unwrap();
    assert_profile(&RrflowMxStore::new());
    assert_profile(&RrflowKvStore::open(&directory.path().join("rrflow-kv")).unwrap());
}
