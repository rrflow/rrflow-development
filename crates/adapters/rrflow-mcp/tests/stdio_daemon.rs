use rrd_contract::{CanonicalId, ResourceId, ResourceKind, ResourcePath};
use rrd_core::{
    digest, RuntimeCommit, RuntimeMutation, RuntimeProperties, RuntimePropertySchema,
    RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeSchemaRegistry, RuntimeType,
    RuntimeValue, RuntimeValueType, ScopeId,
};
use rrd_engine::{InstanceBinding, InstanceManifest, RrdEngine};
use rrd_security::{
    Action, Principal, PrincipalKind, ResourceGrant, SecurityRepository, SecurityState,
    SECURITY_FORMAT,
};
use rrd_server::RrdHttpServer;
use rrd_store::{RrflowKvStore, StorageEngine};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::Write as _;
use std::process::{Command, Stdio};

#[test]
fn daemon_mode_uses_one_authenticated_project_bound_authority() {
    let temporary = tempfile::tempdir().unwrap();
    let project = temporary.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    InstanceManifest::ensure_dedicated_as(&project, "mcp-daemon-test").unwrap();
    let binding = InstanceBinding::discover(&project).unwrap();
    let store = binding.expected_store();
    let instance = CanonicalId::new("mcp-daemon-test").unwrap();
    let resource = ResourcePath {
        segments: vec![ResourceId::new(ResourceKind::Instance, instance.as_str()).unwrap()],
    };
    let api_key = "mcp-daemon-api-key";
    let principal = Principal {
        id: CanonicalId::new("mcp-daemon-client").unwrap(),
        kind: PrincipalKind::Service,
        credential_sha256: digest::sha256_hex(api_key.as_bytes()),
        credential_revision: 1,
        not_before_unix_ms: 1,
        expires_at_unix_ms: u64::MAX,
        disabled: false,
        role_ids: Default::default(),
        grants: [
            Action::SessionCreate,
            Action::SessionClose,
            Action::MemoryContextRead,
        ]
        .into_iter()
        .map(|action| ResourceGrant {
            action,
            resource_prefix: resource.clone(),
            data_policy: None,
        })
        .collect(),
    };
    let storage = RrflowKvStore::open(&store).unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "Daemon context fixture");
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
            scope: ScopeId::new("instance:mcp-daemon-test").unwrap(),
            at: 100,
            actor: "daemon-context-fixture".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry },
                RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: RuntimeRef::new("note", "daemon-alpha").unwrap(),
                        valid_from: 100,
                        valid_to: None,
                        properties: RuntimeProperties::from([(
                            "body".into(),
                            RuntimeValue::String("Authenticated daemon context flow".into()),
                        )]),
                    },
                },
            ],
        })
        .unwrap();
    SecurityRepository::new(&storage, instance.clone())
        .initialize(
            SecurityState {
                format_version: SECURITY_FORMAT,
                revision: 1,
                principals: BTreeMap::from([(principal.id.clone(), principal)]),
                roles: BTreeMap::new(),
                identity_bindings: BTreeMap::new(),
                jwt_issuers: BTreeMap::new(),
            },
            1,
            "bootstrap",
            "request-bootstrap",
            "operation-bootstrap",
        )
        .unwrap();
    drop(storage);

    let engine = RrdEngine::open_bound_with_token_key_file(
        &binding,
        instance.clone(),
        &store.join("RRD.SERVER.SECRET"),
        2,
    )
    .unwrap();
    let server = RrdHttpServer::bind_project(
        engine,
        binding.authority_binding().unwrap(),
        "127.0.0.1:0".parse().unwrap(),
    )
    .unwrap();
    let address = server.local_addr();
    let (shutdown, receiver) = tokio::sync::oneshot::channel();
    let server_thread = std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.serve_until(async move {
                let _ = receiver.await;
            }))
    });

    let api_key_file = temporary.path().join("mcp-api-key");
    std::fs::write(&api_key_file, api_key).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&api_key_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_rrflow-mcp"))
        .arg("daemon")
        .arg("--url")
        .arg(format!("http://{address}"))
        .arg("--instance")
        .arg(instance.as_str())
        .arg("--principal")
        .arg("mcp-daemon-client")
        .arg("--api-key-file")
        .arg(&api_key_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    for request in [
        json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"rrflow_context","arguments":{"query":"authenticated daemon","valid_at":200}}}),
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
    let responses = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    let listed = responses[0]["result"]["tools"].as_array().unwrap();
    let profile = &responses[0]["result"]["_meta"]["io.rrflow/runtimeProfile"];
    assert_eq!(profile["mode"], "daemon");
    assert_eq!(profile["execution_authority"], "rrd_server_via_rrd_client");
    assert_eq!(profile["storage_access"], "none");
    assert_eq!(
        profile["caller_authentication"],
        "principal_api_key_session"
    );
    assert_eq!(profile["context_engine"], "temporal_text_vector_graph");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["name"], "rrflow_context");
    let packet: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(responses[1]["result"]["isError"], false);
    assert_eq!(packet["items"][0]["identity"], "record:note:daemon-alpha");
    assert!(packet["items"][0]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .any(|evidence| evidence["kind"] == "text"));

    shutdown.send(()).unwrap();
    server_thread.join().unwrap().unwrap();
    let storage = RrflowKvStore::open(&store).unwrap();
    let audit = SecurityRepository::new(&storage, instance)
        .audit_since(0, 256)
        .unwrap();
    assert!(audit.records.iter().any(|(_, record)| {
        record.principal_id.as_ref().map(CanonicalId::as_str) == Some("mcp-daemon-client")
            && record.action == Action::MemoryContextRead
            && record.phase == rrd_contract::AuditPhase::Completed
            && record.decision == rrd_contract::AuditDecision::Allowed
    }));
}

#[test]
fn daemon_mode_refuses_an_unsecured_server() {
    let temporary = tempfile::tempdir().unwrap();
    let project = temporary.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    InstanceManifest::ensure_dedicated_as(&project, "mcp-unsecured-test").unwrap();
    let binding = InstanceBinding::discover(&project).unwrap();
    let store = binding.expected_store();
    let instance = CanonicalId::new("mcp-unsecured-test").unwrap();
    let engine = RrdEngine::open_bound_with_token_key_file(
        &binding,
        instance.clone(),
        &store.join("RRD.SERVER.SECRET"),
        1,
    )
    .unwrap();
    let server = RrdHttpServer::bind_project(
        engine,
        binding.authority_binding().unwrap(),
        "127.0.0.1:0".parse().unwrap(),
    )
    .unwrap();
    let address = server.local_addr();
    let (shutdown, receiver) = tokio::sync::oneshot::channel();
    let server_thread = std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.serve_until(async move {
                let _ = receiver.await;
            }))
    });

    let api_key_file = temporary.path().join("mcp-api-key");
    std::fs::write(&api_key_file, "not-an-authenticated-key").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&api_key_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow-mcp"))
        .arg("daemon")
        .arg("--url")
        .arg(format!("http://{address}"))
        .arg("--instance")
        .arg(instance.as_str())
        .arg("--principal")
        .arg("untrusted")
        .arg("--api-key-file")
        .arg(&api_key_file)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("daemon MCP requires an initialized RRD security policy"));

    shutdown.send(()).unwrap();
    server_thread.join().unwrap().unwrap();
}
