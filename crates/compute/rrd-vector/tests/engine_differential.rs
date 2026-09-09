use rrd_core::{
    ReadStamp, RuntimeCommit, RuntimeLogicalModel, RuntimeMutation, RuntimeProperties,
    RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeSchemaRegistry, RuntimeTableSchema,
    RuntimeType, RuntimeValue, RuntimeVector, ScopeId, VectorValue,
};
use rrd_store::{RrflowKvStore, RrflowMxStore, StorageEngine};
use rrd_vector::{search_changes_exact, ScoreMetric, SearchMode, SearchRequest, VectorQuery};
use tempfile::tempdir;

fn commit(engine: &dyn StorageEngine, scope: &ScopeId) -> ReadStamp {
    let mut schema = RuntimeSchemaRegistry::empty(1, "vector differential");
    schema
        .define_record_table(
            RuntimeType::new("document").unwrap(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema::default(),
        )
        .unwrap();
    schema.tables.insert(
        RuntimeType::new("embedding").unwrap(),
        RuntimeTableSchema::schemaless(RuntimeLogicalModel::Vector),
    );
    let documents = [
        ("a", vec![1.0, 0.0], "red"),
        ("b", vec![0.8, 0.2], "red"),
        ("c", vec![0.0, 1.0], "blue"),
    ];
    let mut mutations = vec![RuntimeMutation::Schema { registry: schema }];
    for (id, _, _) in &documents {
        mutations.push(RuntimeMutation::Record {
            record: RuntimeRecord {
                reference: RuntimeRef::new("document", *id).unwrap(),
                valid_from: 1,
                valid_to: None,
                properties: RuntimeProperties::new(),
            },
        });
    }
    for (id, values, color) in documents {
        let mut properties = RuntimeProperties::new();
        properties.insert("color".into(), RuntimeValue::String(color.into()));
        mutations.push(RuntimeMutation::Vector {
            vector: RuntimeVector {
                reference: RuntimeRef::new("embedding", id).unwrap(),
                subject: RuntimeRef::new("document", id).unwrap(),
                collection: None,
                field: "body".into(),
                valid_from: 1,
                valid_to: None,
                value: VectorValue::Dense { values },
                provenance: None,
                properties,
            },
        });
    }
    engine
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope.clone(),
            at: 1,
            actor: "agent:vector-differential".into(),
            expected_cursor: 0,
            mutations,
        })
        .unwrap();
    engine.runtime().read_stamp(scope).unwrap()
}

fn search(engine: &dyn StorageEngine) -> Vec<(String, f64)> {
    let scope = ScopeId::new("instance:vector-differential").unwrap();
    let read = commit(engine, &scope);
    let page = engine.runtime().read_changes(&read, 0, usize::MAX).unwrap();
    let hits = search_changes_exact(
        &SearchRequest {
            scope,
            read,
            valid_at: 2,
            field: "body".into(),
            query: VectorQuery::Dense {
                values: vec![1.0, 0.0],
            },
            metric: ScoreMetric::Cosine,
            embedding_model: None,
            top_k: 3,
            mode: SearchMode::Exact,
            filter: None,
        },
        &page.changes,
    )
    .unwrap();
    hits.into_iter()
        .map(|hit| (hit.reference.id.as_str().to_owned(), hit.score))
        .collect()
}

#[test]
fn exact_search_is_identical_across_rrflow_mx_and_rrflow_kv() {
    let memory = search(&RrflowMxStore::new());

    let rrflow_kv_directory = tempdir().unwrap();
    let rrflow_kv = search(&RrflowKvStore::open(rrflow_kv_directory.path()).unwrap());

    assert_eq!(memory, rrflow_kv);
    assert_eq!(memory[0].0, "a");
}
