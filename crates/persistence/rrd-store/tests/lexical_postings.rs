//! BM25 postings maintained at commit (E-03) answer exactly what a full
//! rebuild over the current documents answers, through backfill, update,
//! retirement and reopen, on both storage profiles.
use rrd_core::{
    digest, ProjectionId, RuntimeCommit, RuntimeLogicalModel, RuntimeMutation,
    RuntimePropertySchema, RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeRetirement,
    RuntimeSchemaRegistry, RuntimeType, RuntimeValue, RuntimeValueType, ScopeId,
};
use rrd_store::lexical::{analyze, bm25_contribution, Bm25Config, Bm25Identifiers};
use rrd_store::{
    ControlTransition, IndexCommitBindingDefinition, IndexCommitBindingKind, LexicalQuery,
    LexicalSearch, RrflowKvStore, RrflowMxStore, StorageEngine,
};
use std::collections::{BTreeMap, BTreeSet};

fn scope() -> ScopeId {
    ScopeId::new("project:lexical-postings").unwrap()
}

fn index() -> ProjectionId {
    ProjectionId::new("chunk-text").unwrap()
}

fn config() -> Bm25Config {
    Bm25Config {
        identifiers: Bm25Identifiers::WholeAndParts,
        ..Bm25Config::default()
    }
}

fn schema() -> RuntimeSchemaRegistry {
    let mut schema = RuntimeSchemaRegistry::empty(1, "lexical postings fixture");
    schema
        .define_record_table(
            RuntimeType::new("chunk").unwrap(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema {
                properties: BTreeMap::from([(
                    "text".into(),
                    RuntimePropertySchema::required(RuntimeValueType::String),
                )]),
                ..RuntimeRecordSchema::default()
            },
        )
        .unwrap();
    schema
}

fn chunk(id: &str, text: &str, valid_from: u64) -> RuntimeMutation {
    RuntimeMutation::Record {
        record: RuntimeRecord {
            reference: RuntimeRef::new("chunk", id).unwrap(),
            valid_from,
            valid_to: None,
            properties: BTreeMap::from([("text".into(), RuntimeValue::String(text.into()))]),
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
            actor: "test:lexical".into(),
            expected_cursor: cursor,
            mutations,
        })
        .unwrap();
}

fn install(engine: &dyn StorageEngine) {
    engine
        .control()
        .commit_catalog_with_index_bindings(
            &scope(),
            1,
            &ControlTransition {
                key: format!("server/state/index-catalogue/{}", scope()),
                expected: None,
                replacement: Some(br#"{"contract":"lexical-fixture-v1"}"#.to_vec()),
                at: 2,
                actor: "test:lexical".into(),
                action: "index_catalogue.installed".into(),
                request_id: "request-lexical".into(),
                operation_id: "operation-lexical".into(),
            },
            &[IndexCommitBindingDefinition {
                id: index(),
                record_kind: RuntimeType::new("chunk").unwrap(),
                fields: vec!["text".into()],
                kind: IndexCommitBindingKind::Bm25,
                configuration_sha256: digest::sha256_hex(b"chunk-text:bm25"),
                lexical: Some(config()),
            }],
        )
        .unwrap();
}

/// BM25 over `documents` computed from scratch: the oracle.
fn rebuilt(documents: &BTreeMap<&str, &str>, query: &str) -> Vec<(String, f64)> {
    let config = config();
    let analyzed = documents
        .iter()
        .map(|(id, text)| (*id, analyze(&config, text)))
        .collect::<Vec<_>>();
    let total = analyzed.len() as f64;
    let average = analyzed.iter().map(|(_, terms)| terms.len()).sum::<usize>() as f64 / total;
    let mut scores = BTreeMap::<String, f64>::new();
    for term in analyze(&config, query).into_iter().collect::<BTreeSet<_>>() {
        let frequency = analyzed
            .iter()
            .filter(|(_, terms)| terms.contains(&term))
            .count() as f64;
        for (id, terms) in &analyzed {
            let count = terms.iter().filter(|candidate| **candidate == term).count();
            if count > 0 {
                *scores.entry(format!("chunk:{id}")).or_default() += bm25_contribution(
                    &config,
                    total,
                    frequency,
                    count as f64,
                    terms.len() as f64,
                    average,
                );
            }
        }
    }
    let mut ranked = scores.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked
}

fn searched(engine: &dyn StorageEngine, query: &str, valid_at: u64) -> Vec<(String, f64)> {
    let read = engine.runtime().read_stamp(&scope()).unwrap();
    match engine
        .runtime()
        .lexical_search(
            &read,
            LexicalQuery {
                index: &index(),
                text: query,
                valid_at,
                top_k: 10,
                max_keys: 10_000,
            },
        )
        .unwrap()
    {
        LexicalSearch::Hits { hits, .. } => hits
            .into_iter()
            .map(|hit| (format!("{}:{}", hit.record.kind, hit.record.id), hit.score))
            .collect(),
        LexicalSearch::NotApplicable(reason) => panic!("not applicable: {reason}"),
    }
}

fn assert_same(engine: &dyn StorageEngine, documents: &BTreeMap<&str, &str>, query: &str) {
    let expected = rebuilt(documents, query);
    let actual = searched(engine, query, 1_000);
    assert_eq!(
        actual.iter().map(|(id, _)| id).collect::<Vec<_>>(),
        expected.iter().map(|(id, _)| id).collect::<Vec<_>>(),
        "ranking for {query:?}"
    );
    for ((_, left), (_, right)) in actual.iter().zip(&expected) {
        assert!(
            (left - right).abs() < 1e-9,
            "{left} != {right} for {query:?}"
        );
    }
}

const QUERIES: [&str; 4] = [
    "control_attunement",
    "how does the planner choose the exact scan",
    "RrdEngine",
    "commit checkpoint",
];

fn exercise(engine: &dyn StorageEngine) {
    // Records that exist before the index are backfilled when it is bound.
    commit(
        engine,
        1,
        vec![
            RuntimeMutation::Schema { registry: schema() },
            chunk(
                "a",
                "fn control_attunement(engine: &RrdEngine) { step(); }",
                10,
            ),
            chunk("b", "the planner chooses the exact scan or TurboQuant", 10),
        ],
    );
    install(engine);
    let mut documents = BTreeMap::from([
        ("a", "fn control_attunement(engine: &RrdEngine) { step(); }"),
        ("b", "the planner chooses the exact scan or TurboQuant"),
    ]);
    for query in QUERIES {
        assert_same(engine, &documents, query);
    }

    // New documents and an edit are maintained in the commit that makes them.
    commit(
        engine,
        20,
        vec![
            chunk(
                "c",
                "commit the parse checkpoint with the job transition",
                20,
            ),
            chunk("b", "how the planner picks an exact scan", 20),
        ],
    );
    documents.insert("c", "commit the parse checkpoint with the job transition");
    documents.insert("b", "how the planner picks an exact scan");
    for query in QUERIES {
        assert_same(engine, &documents, query);
    }

    // A retired document leaves the postings; an earlier valid time needs
    // history and is refused rather than answered from current postings.
    commit(
        engine,
        30,
        vec![RuntimeMutation::Retire {
            retirement: RuntimeRetirement {
                model: RuntimeLogicalModel::Relational,
                reference: RuntimeRef::new("chunk", "a").unwrap(),
                effective_at: 30,
            },
        }],
    );
    documents.remove("a");
    for query in QUERIES {
        assert_same(engine, &documents, query);
    }
    let read = engine.runtime().read_stamp(&scope()).unwrap();
    assert!(matches!(
        engine
            .runtime()
            .lexical_search(
                &read,
                LexicalQuery {
                    index: &index(),
                    text: "planner",
                    valid_at: 25,
                    top_k: 10,
                    max_keys: 10_000
                }
            )
            .unwrap(),
        LexicalSearch::NotApplicable(_)
    ));

    // A stamp behind the head is not answered from current postings.
    commit(engine, 40, vec![chunk("d", "planner notes", 40)]);
    assert!(matches!(
        engine
            .runtime()
            .lexical_search(
                &read,
                LexicalQuery {
                    index: &index(),
                    text: "planner",
                    valid_at: 1_000,
                    top_k: 10,
                    max_keys: 10_000
                }
            )
            .unwrap(),
        LexicalSearch::NotApplicable(_)
    ));
    documents.insert("d", "planner notes");
    for query in QUERIES {
        assert_same(engine, &documents, query);
    }

    // The key bound refuses rather than rank partially.
    let head = engine.runtime().read_stamp(&scope()).unwrap();
    assert!(engine
        .runtime()
        .lexical_search(
            &head,
            LexicalQuery {
                index: &index(),
                text: "the planner",
                valid_at: 1_000,
                top_k: 10,
                max_keys: 2
            }
        )
        .is_err());
}

#[test]
fn postings_equal_a_full_rebuild_through_backfill_update_and_retirement() {
    exercise(&RrflowMxStore::new());
    let directory = tempfile::tempdir().unwrap();
    exercise(&RrflowKvStore::create_new(&directory.path().join("kv")).unwrap());
}

#[test]
fn postings_survive_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("kv");
    let store = RrflowKvStore::create_new(&root).unwrap();
    exercise(&store);
    let before = QUERIES.map(|query| searched(&store, query, 1_000));
    drop(store);
    let reopened = RrflowKvStore::open(&root).unwrap();
    assert_eq!(
        QUERIES.map(|query| searched(&reopened, query, 1_000)),
        before
    );
}
