use rrd_core::{
    RuntimeLogicalModel, RuntimeMutation, RuntimeProperties, RuntimeSchemaRegistry,
    RuntimeTraceEvent, RuntimeType, ScopeId, TraceBoundary, TraceDataClass, TraceOperation,
    TraceOutcome,
};
use rrd_engine::{install_runtime_trace_contract, record_runtime_trace, TraceIdentity};
use rrd_store::{RrflowKvStore, RrflowMxStore, StorageEngine};
use std::sync::{Arc, Barrier};

fn scope() -> ScopeId {
    ScopeId::new("instance:runtime-trace-test").unwrap()
}

fn identity(seed: &str) -> TraceIdentity {
    TraceIdentity::derive(&[seed.as_bytes()]).unwrap()
}

fn phases<E: StorageEngine>(store: &E) -> Vec<(String, String)> {
    store
        .runtime()
        .changes_since(0, usize::MAX, Some(&scope()))
        .unwrap()
        .changes
        .into_iter()
        .filter_map(|change| match change.mutation {
            RuntimeMutation::Event { event } if event.kind.as_str() == "runtime_trace" => {
                let rrd_core::RuntimeValue::String(phase) = &event.properties["phase"] else {
                    panic!("trace phase has the wrong type")
                };
                let rrd_core::RuntimeValue::String(outcome) = &event.properties["outcome"] else {
                    panic!("trace outcome has the wrong type")
                };
                Some((phase.clone(), outcome.clone()))
            }
            _ => None,
        })
        .collect()
}

fn exercise<E: StorageEngine>(store: &E) -> Vec<Vec<u8>> {
    let scope = scope();
    assert!(
        install_runtime_trace_contract(store, &scope, 1, "test:bootstrap")
            .unwrap()
            .is_some()
    );
    assert!(
        install_runtime_trace_contract(store, &scope, 2, "test:bootstrap")
            .unwrap()
            .is_none()
    );

    let identity = identity("paired-lifecycle");
    record_runtime_trace(
        store,
        &scope,
        "engine:test",
        RuntimeTraceEvent::start(
            identity.trace_id.clone(),
            identity.span_id.clone(),
            None,
            TraceBoundary::Engine,
            TraceOperation::EngineOperation,
            10,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::new(),
        )
        .unwrap(),
    )
    .unwrap();
    record_runtime_trace(
        store,
        &scope,
        "engine:test",
        RuntimeTraceEvent::finish(
            identity.trace_id,
            identity.span_id,
            None,
            TraceBoundary::Engine,
            TraceOperation::EngineOperation,
            11,
            250,
            TraceOutcome::Ok,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::new(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        phases(store),
        [
            ("start".into(), "running".into()),
            ("finish".into(), "ok".into())
        ]
    );
    let schema = store.runtime().schema(&scope).unwrap().unwrap();
    assert_eq!(schema.revision, 1);
    assert_eq!(
        schema.events[&RuntimeTraceEvent::event_type().unwrap()],
        RuntimeTraceEvent::event_schema()
    );
    store
        .runtime()
        .changes_since(0, usize::MAX, Some(&scope))
        .unwrap()
        .changes
        .into_iter()
        .map(|change| serde_json::to_vec(&change.mutation).unwrap())
        .collect()
}

#[test]
fn trace_schema_and_events_match_rrflow_mx_and_rrflow_kv() {
    let memory = RrflowMxStore::new();
    let rrflow_kv_root = tempfile::tempdir().unwrap();
    let rrflow_kv = RrflowKvStore::open(rrflow_kv_root.path()).unwrap();
    let expected = exercise(&memory);
    assert_eq!(exercise(&rrflow_kv), expected);
}

#[test]
fn conflicting_trace_schema_is_repaired_atomically_with_the_first_event() {
    let store = RrflowMxStore::new();
    let scope = scope();
    let mut wrong = RuntimeSchemaRegistry::empty(1, "deliberately incomplete trace schema");
    wrong
        .define_event_table(
            RuntimeType::new("runtime_trace").unwrap(),
            RuntimeLogicalModel::Event,
            rrd_core::RuntimeEventSchema::default(),
        )
        .unwrap();
    store
        .runtime()
        .commit(&rrd_core::RuntimeCommit {
            scope: scope.clone(),
            at: 1,
            actor: "test:fixture".into(),
            expected_cursor: 0,
            mutations: vec![RuntimeMutation::Schema { registry: wrong }],
        })
        .unwrap();

    let identity = identity("schema-repair");
    let outcome = record_runtime_trace(
        &store,
        &scope,
        "test:repair",
        RuntimeTraceEvent::annotation(
            identity.trace_id,
            identity.span_id,
            None,
            TraceBoundary::Engine,
            TraceOperation::EngineCommit,
            2,
            TraceOutcome::Ok,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::new(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.count, 2, "schema and event must share one commit");
    assert_eq!(store.runtime().schema(&scope).unwrap().unwrap().revision, 2);
    assert_eq!(phases(&store), [("annotation".into(), "ok".into())]);
}

#[test]
fn authoritative_start_survives_reopen_as_an_honest_incomplete_span() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("rrflow-kv");
    let scope = scope();
    let identity = identity("crash-visible-start");
    {
        let store = RrflowKvStore::open(&path).unwrap();
        record_runtime_trace(
            &store,
            &scope,
            "engine:test",
            RuntimeTraceEvent::start(
                identity.trace_id,
                identity.span_id,
                None,
                TraceBoundary::Engine,
                TraceOperation::EngineOperation,
                10,
                TraceDataClass::Control,
                Vec::new(),
                RuntimeProperties::new(),
            )
            .unwrap(),
        )
        .unwrap();
    }
    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_eq!(phases(&reopened), [("start".into(), "running".into())]);
}

#[test]
fn concurrent_trace_writers_rebase_without_losing_events() {
    let store = Arc::new(RrflowMxStore::new());
    let barrier = Arc::new(Barrier::new(8));
    let mut threads = Vec::new();
    for index in 0..8_u64 {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            let seed = format!("concurrent-{index}");
            let identity = identity(&seed);
            barrier.wait();
            record_runtime_trace(
                store.as_ref(),
                &scope(),
                "test:concurrent",
                RuntimeTraceEvent::annotation(
                    identity.trace_id,
                    identity.span_id,
                    None,
                    TraceBoundary::Kv,
                    TraceOperation::KvWriteBatch,
                    index + 1,
                    TraceOutcome::Ok,
                    TraceDataClass::Control,
                    Vec::new(),
                    RuntimeProperties::new(),
                )
                .unwrap(),
            )
            .unwrap();
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(phases(store.as_ref()).len(), 8);
    assert_eq!(store.runtime().cursor().unwrap(), 9);
}
