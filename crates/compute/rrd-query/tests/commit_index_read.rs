//! J3: rrflowQL reads commit-maintained index entries and postings. With the
//! same data, a store whose indexes are defined answers every query exactly as
//! a store without them, through edits and retirements, reads fewer keys, and
//! ranks MATCH with the same scores.
use rrd_core::{
    ProjectionId, RuntimeCommit, RuntimeLogicalModel, RuntimeMutation, RuntimePropertySchema,
    RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeRetirement, RuntimeSchemaRegistry,
    RuntimeType, RuntimeValue, RuntimeValueType, ScopeId,
};
use rrd_query::{
    bind, execute, parse, plan, Bm25Config, Catalog, ExecutionBudget, IndexCatalogueRepository,
    IndexDefinition, IndexKind, IndexMutationContext, Parameters, QueryRow, Source,
};
use rrd_store::{RrflowKvStore, RrflowMxStore, RuntimeReadBudget, StorageEngine};
use std::collections::BTreeMap;

fn scope() -> ScopeId {
    ScopeId::new("instance:commit-index").unwrap()
}

fn source() -> Source {
    Source::Record {
        kind: RuntimeType::new("symbol").unwrap(),
    }
}

fn catalog<E: StorageEngine>(engine: &E) -> Catalog {
    Catalog::capture_for_sources(
        engine,
        &scope(),
        &[source()],
        RuntimeReadBudget::new(ExecutionBudget::default().max_storage_keys).unwrap(),
    )
    .unwrap()
}

fn symbol(id: &str, name: &str, text: &str, valid_from: u64) -> RuntimeMutation {
    RuntimeMutation::Record {
        record: RuntimeRecord {
            reference: RuntimeRef::new("symbol", id).unwrap(),
            valid_from,
            valid_to: None,
            properties: BTreeMap::from([
                ("name".into(), RuntimeValue::String(name.into())),
                ("text".into(), RuntimeValue::String(text.into())),
            ]),
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
            actor: "test:commit-index".into(),
            expected_cursor: cursor,
            mutations,
        })
        .unwrap();
}

fn seed(engine: &dyn StorageEngine) {
    let mut schema = RuntimeSchemaRegistry::empty(1, "commit index fixture");
    schema
        .define_record_table(
            RuntimeType::new("symbol").unwrap(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema {
                properties: BTreeMap::from([
                    (
                        "name".into(),
                        RuntimePropertySchema::required(RuntimeValueType::String),
                    ),
                    (
                        "text".into(),
                        RuntimePropertySchema::required(RuntimeValueType::String),
                    ),
                ]),
                ..RuntimeRecordSchema::default()
            },
        )
        .unwrap();
    let mut mutations = vec![RuntimeMutation::Schema { registry: schema }];
    for index in 0..200 {
        mutations.push(symbol(
            &format!("s{index:03}"),
            &format!("helper_{}", index % 50),
            &format!("fn helper_{}() {{ call_{}(); }}", index % 50, index % 7),
            10,
        ));
    }
    mutations.push(symbol(
        "control",
        "control_attunement",
        "fn control_attunement(engine: &RrdEngine) { commit the parse checkpoint }",
        10,
    ));
    commit(engine, 1, mutations);
}

fn define<E: StorageEngine>(engine: &E) {
    let repository = IndexCatalogueRepository::new(engine, scope());
    let context = |at, action: &str| IndexMutationContext {
        at,
        actor: "operator:test".into(),
        request_id: format!("request-{action}"),
        operation_id: format!("operation-{action}"),
    };
    repository
        .create(
            &context(2, "name"),
            &catalog(engine),
            IndexDefinition {
                id: ProjectionId::new("symbol-name").unwrap(),
                source: source(),
                fields: vec!["name".into()],
                unique: false,
                kind: IndexKind::Scalar,
                filters: Vec::new(),
            },
        )
        .unwrap();
    repository
        .create(
            &context(3, "text"),
            &catalog(engine),
            IndexDefinition {
                id: ProjectionId::new("symbol-text").unwrap(),
                source: source(),
                fields: vec!["text".into()],
                unique: false,
                kind: IndexKind::Bm25 {
                    config: Bm25Config::default(),
                },
                filters: Vec::new(),
            },
        )
        .unwrap();
}

struct Answer {
    selected: String,
    rows: Vec<QueryRow>,
    keys: usize,
}

fn answer<E: StorageEngine>(engine: &E, text: &str) -> Answer {
    let query = parse(text).unwrap();
    let physical = plan(&bind(&query, &Parameters::new(), &catalog(engine)).unwrap()).unwrap();
    let selected = physical
        .explanation
        .candidates
        .iter()
        .find(|candidate| candidate.selected)
        .unwrap()
        .name
        .clone();
    let execution = execute(engine, &physical, &ExecutionBudget::default()).unwrap();
    Answer {
        selected,
        keys: execution.selected_versions,
        rows: execution
            .batches
            .into_iter()
            .flat_map(|batch| batch.rows)
            .collect(),
    }
}

fn queries(valid_at: u64) -> [String; 4] {
    [
        format!("FROM record:symbol AT VALID {valid_at} KNOWN HEAD WHERE name = \"control_attunement\" PROJECT name, text"),
        format!("FROM record:symbol AT VALID {valid_at} KNOWN HEAD WHERE name = \"helper_7\" PROJECT name"),
        format!("FROM record:symbol AT VALID {valid_at} KNOWN HEAD WHERE text MATCH \"parse checkpoint\" LIMIT 5"),
        format!("FROM record:symbol AT VALID {valid_at} KNOWN HEAD WHERE text MATCH \"call_3 helper_7\" LIMIT 10"),
    ]
}

fn compare<E: StorageEngine>(indexed: &E, plain: &E, valid_at: u64, expect_index: bool) {
    for query in queries(valid_at) {
        let with = answer(indexed, &query);
        let without = answer(plain, &query);
        assert_eq!(with.rows, without.rows, "{query}");
        assert!(!with.rows.is_empty(), "{query} found nothing");
        if query.contains("MATCH") {
            // Scores, matched terms and highlights are compared with the rows.
            assert!(with.rows[0].values.contains_key("_score"), "{query}");
            assert!(with.rows[0].values.contains_key("_highlight"), "{query}");
        }
        if expect_index {
            assert!(
                with.selected.starts_with("commit_index:"),
                "{query}: {}",
                with.selected
            );
            assert!(
                with.keys < without.keys,
                "{query}: the index read {} versions, the scan {}",
                with.keys,
                without.keys
            );
        }
        assert_eq!(without.selected, "versioned_source_read", "{query}");
    }
}

fn exercise<E: StorageEngine>(indexed: &E, plain: &E) {
    for engine in [indexed, plain] {
        seed(engine);
    }
    define(indexed);
    compare(indexed, plain, 100, true);

    let edits = || {
        vec![
            symbol(
                "s007",
                "renamed_helper",
                "fn renamed_helper() { call_3(); }",
                20,
            ),
            symbol(
                "new",
                "helper_7",
                "fn helper_7() { parse checkpoint again }",
                20,
            ),
        ]
    };
    commit(indexed, 20, edits());
    commit(plain, 20, edits());
    compare(indexed, plain, 100, true);

    let retire = || {
        vec![RuntimeMutation::Retire {
            retirement: RuntimeRetirement {
                model: RuntimeLogicalModel::Relational,
                reference: RuntimeRef::new("symbol", "s057").unwrap(),
                effective_at: 30,
            },
        }]
    };
    commit(indexed, 30, retire());
    commit(plain, 30, retire());
    compare(indexed, plain, 100, true);
    // Before the latest change the indexes cannot answer, so the planner's
    // fallback reads history and the answers still agree.
    compare(indexed, plain, 25, false);
}

#[test]
fn commit_index_reads_equal_a_full_scan_on_every_engine() {
    exercise(&RrflowMxStore::new(), &RrflowMxStore::new());
    let directory = tempfile::tempdir().unwrap();
    exercise(
        &RrflowKvStore::create_new(&directory.path().join("indexed")).unwrap(),
        &RrflowKvStore::create_new(&directory.path().join("plain")).unwrap(),
    );
}
