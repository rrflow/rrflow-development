use connectome_ui::{ConnectomeBackend, ConnectomeConfig, ContextRequest};
use rrd_contract::{CanonicalId, ContextEvidenceKind, ResourceId, ResourceKind, ResourcePath};
use rrd_core::{
    digest, RuntimeCommit, RuntimeMutation, RuntimeProperties, RuntimePropertySchema,
    RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeSchemaRegistry, RuntimeType,
    RuntimeValue, RuntimeValueType, ScopeId,
};
use rrd_engine::{load_or_create_token_key, InstanceBinding, InstanceManifest, RrdEngine};
use rrd_security::{
    Action, Principal, PrincipalKind, ResourceGrant, SecurityRepository, SecurityState,
    SECURITY_FORMAT,
};
use rrd_server::RrdHttpServer;
use rrd_store::{Engine, PersistentEngine};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

struct RunningRrd {
    address: std::net::SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for RunningRrd {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn start_rrd() -> (tempfile::TempDir, RunningRrd) {
    let temporary = tempfile::tempdir().unwrap();
    let project = temporary.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    InstanceManifest::ensure_dedicated_as(&project, "connectome-test").unwrap();
    let binding = InstanceBinding::discover(&project).unwrap();
    let database = binding.expected_store();
    let storage = PersistentEngine::open(&database).unwrap();

    let mut registry = RuntimeSchemaRegistry::empty(1, "Connectome client fixture");
    registry.records.insert(
        RuntimeType::new("document").unwrap(),
        RuntimeRecordSchema {
            properties: BTreeMap::from([(
                "title".into(),
                RuntimePropertySchema::required(RuntimeValueType::String),
            )]),
            ..RuntimeRecordSchema::default()
        },
    );
    storage
        .commit_runtime(&RuntimeCommit {
            scope: ScopeId::new("instance:connectome-test").unwrap(),
            at: 100,
            actor: "connectome-fixture".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry },
                RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: RuntimeRef::new("document", "alpha").unwrap(),
                        valid_from: 100,
                        valid_to: None,
                        properties: RuntimeProperties::from([(
                            "title".into(),
                            RuntimeValue::String("Alpha".into()),
                        )]),
                    },
                },
            ],
        })
        .unwrap();
    let instance = CanonicalId::new("connectome-test").unwrap();
    let resource = ResourcePath {
        segments: vec![ResourceId::new(ResourceKind::Instance, instance.as_str()).unwrap()],
    };
    let principal = Principal {
        id: CanonicalId::new("connectome-client").unwrap(),
        kind: PrincipalKind::Service,
        credential_sha256: digest::sha256_hex(b"connectome-api-key"),
        credential_revision: 1,
        not_before_unix_ms: 1,
        expires_at_unix_ms: u64::MAX,
        disabled: false,
        role_ids: Default::default(),
        grants: [
            Action::SessionCreate,
            Action::SessionClose,
            Action::DiagnosticsRead,
            Action::ServiceInspect,
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

    let token_key = load_or_create_token_key(&database.join("RRD.SERVER.SECRET")).unwrap();
    let engine = RrdEngine::open_bound_with_token_key(&binding, instance, token_key, 2).unwrap();
    let authority = binding.authority_binding().unwrap();
    let server =
        RrdHttpServer::bind_project(engine, authority, "127.0.0.1:0".parse().unwrap()).unwrap();
    let address = server.local_addr();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_thread = Arc::clone(&stop);
    let thread = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        runtime
            .block_on(server.serve_until(async move {
                while !stop_for_thread.load(Ordering::Acquire) {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }))
            .unwrap();
    });
    (
        temporary,
        RunningRrd {
            address,
            stop,
            thread: Some(thread),
        },
    )
}

#[test]
fn connectome_uses_authenticated_public_rrd_contracts_end_to_end() {
    let (_temporary, rrd) = start_rrd();
    let backend = ConnectomeBackend::connect(&ConnectomeConfig {
        rrd_address: rrd.address,
        instance: CanonicalId::new("connectome-test").unwrap(),
        principal: CanonicalId::new("connectome-client").unwrap(),
        api_key: "connectome-api-key".into(),
        scope: "instance:connectome-test".into(),
        bind: "127.0.0.1:4387".parse().unwrap(),
        shutdown: None,
    })
    .unwrap();

    let service = backend.service_capabilities().unwrap();
    assert_eq!(service.instance.id.as_str(), "connectome-test");
    assert_eq!(
        service.product_capabilities,
        rrd_engine::product_capability_catalogue()
    );
    let snapshot = backend.diagnostic_snapshot().unwrap();
    assert_eq!(snapshot.scope, "instance:connectome-test");
    let diagnostic_cursor = snapshot.read.runtime_cursor;
    assert_eq!(diagnostic_cursor, 2);
    assert_eq!(snapshot.graph.records.len(), 1);
    assert_eq!(snapshot.graph.records[0].reference.id, "alpha");
    assert!(snapshot
        .models
        .models
        .iter()
        .any(|model| model.id.as_str() == "document"));
    snapshot.validate().unwrap();

    let packet = backend
        .assemble_context(ContextRequest {
            query: "Alpha".into(),
            seeds: Vec::new(),
            valid_at: Some(100),
            max_graph_depth: 2,
            max_items: 10,
            max_output_bytes: 4_096,
            max_scanned_changes: 100,
        })
        .unwrap();
    packet.validate().unwrap();
    assert_eq!(packet.read.runtime_cursor, diagnostic_cursor);
    assert_eq!(packet.items.len(), 1);
    assert_eq!(packet.items[0].identity, "record:document:alpha");
    assert!(packet.items[0]
        .evidence
        .iter()
        .any(|evidence| evidence.kind == ContextEvidenceKind::Text));
}
