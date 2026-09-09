use rrd_core::{
    RuntimeCommit, RuntimeMutation, RuntimeProperties, RuntimePropertySchema, RuntimeRecord,
    RuntimeRecordSchema, RuntimeRef, RuntimeSchemaRegistry, RuntimeType, RuntimeValue,
    RuntimeValueType, ScopeId,
};
use rrd_store::{RrflowKvStore, StorageEngine};
use std::collections::BTreeMap;
use std::io::Write as _;
use std::process::{Command, Stdio};

#[test]
fn stdio_context_flows_through_the_single_engine_operation() {
    let root = tempfile::tempdir().unwrap();
    rrd_engine::InstanceManifest::ensure_dedicated_as(root.path(), "mcp-context-test").unwrap();
    let db = root.path().join(".rrflow/rrd");
    let storage = RrflowKvStore::open(&db).unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "MCP context fixture");
    registry.records.insert(
        RuntimeType::new("note").unwrap(),
        RuntimeRecordSchema {
            properties: BTreeMap::from([(
                "body".into(),
                RuntimePropertySchema::required(RuntimeValueType::String),
            )]),
            ..RuntimeRecordSchema::default()
        },
    );
    storage
        .runtime()
        .commit(&RuntimeCommit {
            scope: ScopeId::new("instance:mcp-context-test").unwrap(),
            at: 100,
            actor: "mcp-context-fixture".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry },
                RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: RuntimeRef::new("note", "alpha").unwrap(),
                        valid_from: 100,
                        valid_to: None,
                        properties: RuntimeProperties::from([(
                            "body".into(),
                            RuntimeValue::String("Durable authentication session recovery".into()),
                        )]),
                    },
                },
            ],
        })
        .unwrap();
    drop(storage);

    let mut child = Command::new(env!("CARGO_BIN_EXE_rrflow-mcp"))
        .arg("embedded")
        .arg("--db")
        .arg(&db)
        .arg("--root")
        .arg(root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    for request in [
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"rrflow_context","arguments":{"query":"authentication recovery","valid_at":200,"max_items":8,"max_output_bytes":16384,"max_scanned_changes":100}}}),
        serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"obsolete_tool","arguments":{}}}),
    ] {
        serde_json::to_writer(child.stdin.as_mut().unwrap(), &request).unwrap();
        child.stdin.as_mut().unwrap().write_all(b"\n").unwrap();
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let responses: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(responses.len(), 4);
    assert_eq!(responses[0]["result"]["protocolVersion"], "2025-11-25");
    let profile = &responses[0]["result"]["_meta"]["io.rrflow/runtimeProfile"];
    assert_eq!(profile["execution_authority"], "rrd_engine");
    assert_eq!(profile["context_engine"], "temporal_text_vector_graph");
    let tools = responses[1]["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "rrflow_context");
    assert_eq!(responses[2]["result"]["isError"], false);
    let packet: serde_json::Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(packet["read"]["runtime_cursor"], 2);
    assert_eq!(packet["items"][0]["identity"], "record:note:alpha");
    assert!(packet["items"][0]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|evidence| evidence["kind"] == "text"));
    assert_eq!(responses[3]["error"]["code"], -32602);
}

#[test]
fn stdio_server_supports_the_stateless_2026_discovery_era() {
    let root = tempfile::tempdir().unwrap();
    rrd_engine::InstanceManifest::ensure_dedicated(root.path()).unwrap();
    let db = root.path().join(".rrflow/rrd");
    let input = [
        serde_json::json!({"jsonrpc":"2.0","id":"d","method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"test","version":"1"}}}}),
        serde_json::json!({"jsonrpc":"2.0","id":"l","method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"test","version":"1"}}}}),
    ];
    let mut child = Command::new(env!("CARGO_BIN_EXE_rrflow-mcp"))
        .arg("embedded")
        .arg("--db")
        .arg(&db)
        .arg("--root")
        .arg(root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    for message in input {
        serde_json::to_writer(child.stdin.as_mut().unwrap(), &message).unwrap();
        child.stdin.as_mut().unwrap().write_all(b"\n").unwrap();
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let responses: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(responses[0]["result"]["supportedVersions"][0], "2026-07-28");
    assert_eq!(
        responses[1]["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "rrflow-mcp"
    );
    assert_eq!(responses[1]["result"]["tools"].as_array().unwrap().len(), 1);
}

#[test]
fn stdio_server_refuses_a_foreign_instance_store_before_protocol_start() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    rrd_engine::InstanceManifest::ensure_dedicated(first.path()).unwrap();
    rrd_engine::InstanceManifest::ensure_dedicated(second.path()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow-mcp"))
        .arg("embedded")
        .arg("--db")
        .arg(first.path().join(".rrflow/rrd"))
        .arg("--root")
        .arg(second.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("does not belong"));
}
