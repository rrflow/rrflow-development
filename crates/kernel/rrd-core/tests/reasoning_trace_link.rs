use rrd_core::{
    ReadStamp, ReasoningActiveCursor, RuntimeId, RuntimeProperties, RuntimeTraceEvent,
    RuntimeValue, ScopeId, SpanId, TraceBoundary, TraceDataClass, TraceId, TraceLink,
    TraceOperation, REASONING_TREE_CONTRACT_VERSION,
};

#[test]
fn trace_link_retains_the_generic_reasoning_cursor_coordinate() {
    let read = ReadStamp::new(
        ScopeId::new("instance:trace").unwrap(),
        Some(1),
        2,
        7,
        Some("1".repeat(64)),
    )
    .unwrap();
    let cursor = ReasoningActiveCursor {
        contract_version: REASONING_TREE_CONTRACT_VERSION,
        id: RuntimeId::new("cursor-01").unwrap(),
        tree_id: RuntimeId::new("workspace-context").unwrap(),
        tree_revision: 3,
        node_id: RuntimeId::new("route-context").unwrap(),
        step: 4,
        read: read.clone(),
    };
    let trace = RuntimeTraceEvent::start(
        TraceId::new("0123456789abcdef0123456789abcdef").unwrap(),
        SpanId::new("0123456789abcdef").unwrap(),
        None,
        TraceBoundary::Inference,
        TraceOperation::InferenceRoute,
        1_000,
        TraceDataClass::Control,
        vec![TraceLink::ReasoningCursor {
            cursor: cursor.clone(),
        }],
        RuntimeProperties::new(),
    )
    .unwrap();
    trace.validate().unwrap();

    let RuntimeValue::List(links) = trace.into_runtime_event().unwrap().properties["links"].clone()
    else {
        panic!("trace links must be persisted as a list");
    };
    let RuntimeValue::Map(link) = &links[0] else {
        panic!("reasoning cursor link must be persisted as a map");
    };
    assert_eq!(
        link["kind"],
        RuntimeValue::String("reasoning_cursor".into())
    );
    assert_eq!(
        link["cursor_id"],
        RuntimeValue::String(cursor.id.to_string())
    );
    assert_eq!(
        link["tree_id"],
        RuntimeValue::String(cursor.tree_id.to_string())
    );
    assert_eq!(link["tree_revision"], RuntimeValue::Unsigned(3));
    assert_eq!(
        link["node_id"],
        RuntimeValue::String("route-context".into())
    );
    assert_eq!(link["step"], RuntimeValue::Unsigned(4));
    assert_eq!(
        link["read_manifest_sha256"],
        RuntimeValue::Digest(read.manifest_id)
    );
}
