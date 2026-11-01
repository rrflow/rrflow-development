//! Actual engine authentication/denials plus retained-response transport probes.
//! Replaying/corrupting an archived receipt is not a fresh model observation,
//! an accepted engine effect, a training example or a quality evaluation.
use rrd_client::{
    operations::TaskEndpointPropose, ClientConfig, Error, OperationMetadata, RequestOptions,
    RrdClient, Session,
};
use rrd_contract::*;
use rrd_core::digest;
use rrd_engine::RrdEngine;
use rrd_security::{
    Action, Principal, PrincipalKind, ResourceGrant, SecurityRepository, SecurityState,
    SECURITY_FORMAT,
};
use rrd_server::RrdHttpServer;
use rrd_store::RrflowKvStore;
use serde_json::Value;
use std::{collections::BTreeMap, net::SocketAddr, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn observation() -> Value {
    serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../../docs/evidence/change-journals/gate-g/controller-installed-receipts/baseline/authorized-proposal.json"))).unwrap()
}
fn request() -> EndpointSelectionRequest {
    serde_json::from_value(observation()["request"]["payload"].clone()).unwrap()
}
fn options() -> RequestOptions {
    let context: RequestContext =
        serde_json::from_value(observation()["request"]["context"].clone()).unwrap();
    RequestOptions::new(
        context.request_id.as_str(),
        context.operation_id.as_str(),
        context.idempotency_key.as_ref().map(CorrelationId::as_str),
        None,
    )
    .unwrap()
}
fn instance() -> CanonicalId {
    serde_json::from_value(observation()["request"]["resource"]["segments"][0]["id"].clone())
        .unwrap()
}
fn client(address: SocketAddr) -> RrdClient {
    RrdClient::connect_local(
        address,
        instance(),
        ClientConfig {
            request_timeout: Duration::from_secs(2),
            max_attempts: 3,
            websocket_limits: WebSocketLimits::default(),
        },
    )
    .unwrap()
}
struct Engine {
    _root: tempfile::TempDir,
    client: RrdClient,
    shutdown: tokio::sync::oneshot::Sender<()>,
    running: tokio::task::JoinHandle<std::result::Result<(), rrd_server::HttpError>>,
}
impl Engine {
    async fn start() -> Self {
        let root = tempfile::tempdir().unwrap();
        let storage = RrflowKvStore::open(root.path()).unwrap();
        let resource = ResourcePath {
            segments: vec![ResourceId::new(ResourceKind::Instance, instance().as_str()).unwrap()],
        };
        let principals = [
            (
                "allowed",
                vec![
                    Action::SessionCreate,
                    Action::EndpointPropose,
                    Action::TaskRead,
                    Action::ReasoningWrite,
                ],
            ),
            ("denied", vec![Action::SessionCreate]),
        ]
        .into_iter()
        .map(|(name, actions)| {
            let id = CanonicalId::new(name).unwrap();
            (
                id.clone(),
                Principal {
                    id,
                    kind: PrincipalKind::Service,
                    credential_sha256: digest::sha256_hex(name.as_bytes()),
                    credential_revision: 1,
                    not_before_unix_ms: 1,
                    expires_at_unix_ms: u64::MAX,
                    disabled: false,
                    role_ids: Default::default(),
                    grants: actions
                        .into_iter()
                        .map(|action| ResourceGrant {
                            action,
                            resource_prefix: resource.clone(),
                            data_policy: None,
                        })
                        .collect(),
                },
            )
        })
        .collect();
        SecurityRepository::new(&storage, instance())
            .initialize(
                SecurityState {
                    format_version: SECURITY_FORMAT,
                    revision: 1,
                    principals,
                    roles: BTreeMap::new(),
                    identity_bindings: BTreeMap::new(),
                    jwt_issuers: BTreeMap::new(),
                },
                1,
                "bootstrap",
                "bootstrap",
                "bootstrap",
            )
            .unwrap();
        drop(storage);
        let engine = RrdEngine::open(root.path(), instance(), [0x5b; 32]).unwrap();
        let server = RrdHttpServer::bind(engine, "127.0.0.1:0".parse().unwrap()).unwrap();
        let client = client(server.local_addr());
        let (shutdown, receiver) = tokio::sync::oneshot::channel();
        let running = tokio::spawn(server.serve_until(async move {
            let _ = receiver.await;
        }));
        Self {
            _root: root,
            client,
            shutdown,
            running,
        }
    }
    async fn session(&self, name: &str) -> Session {
        self.client
            .create_session(
                CanonicalId::new(name).unwrap(),
                name,
                CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 60_000,
                        absolute_timeout_ms: 300_000,
                        max_open_transactions: 1,
                    },
                },
                RequestOptions::mutation(name, name, name).unwrap(),
            )
            .await
            .unwrap()
    }
    async fn finish(self) {
        self.shutdown.send(()).unwrap();
        self.running.await.unwrap().unwrap();
    }
}

#[test]
fn retained_proposal_requires_every_request_identity() {
    let request = request();
    let value = observation()["response"]["outcome"]["payload"].clone();
    let proposal: EndpointSelectionProposal = serde_json::from_value(value.clone()).unwrap();
    proposal.validate_request_identity(&request).unwrap();
    for field in [
        "scope",
        "task_id",
        "attempt_id",
        "task_sha256",
        "configuration_sha256",
    ] {
        let mut changed = value.clone();
        changed[field] = if field.ends_with("sha256") {
            "0".repeat(64).into()
        } else if field == "scope" {
            "instance:foreign".into()
        } else {
            "foreign".into()
        };
        let changed: EndpointSelectionProposal = serde_json::from_value(changed).unwrap();
        assert!(changed.validate().is_ok());
        assert!(
            changed.validate_request_identity(&request).is_err(),
            "{field}"
        );
    }
}

#[tokio::test]
async fn generated_call_preserves_real_engine_authorization_and_installation_denial() {
    let engine = Engine::start().await;
    let denied = engine.session("denied").await;
    let error = engine
        .client
        .call_operation::<TaskEndpointPropose>(&denied, request(), options())
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Api { error, .. } if error.code == ErrorCode::PermissionDenied));
    let allowed = engine.session("allowed").await;
    let error = engine
        .client
        .call_operation::<TaskEndpointPropose>(&allowed, request(), options())
        .await
        .unwrap_err();
    assert!(
        matches!(&error, Error::Api { error, .. } if error.message.contains("requires an installed project identity")),
        "{error:?}"
    );
    engine.finish().await;
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut buffer = [0; 4096];
        let count = stream.read(&mut buffer).await.unwrap();
        assert!(count > 0);
        bytes.extend_from_slice(&buffer[..count]);
        assert!(bytes.len() < 64 * 1024);
        if let Some(end) = bytes.windows(4).position(|chunk| chunk == b"\r\n\r\n") {
            let header = String::from_utf8_lossy(&bytes[..end]);
            let length: usize = header
                .lines()
                .find_map(|line| {
                    line.to_lowercase()
                        .strip_prefix("content-length: ")
                        .map(str::to_owned)
                })
                .unwrap()
                .parse()
                .unwrap();
            if bytes.len() >= end + 4 + length {
                return bytes;
            }
        }
    }
}

async fn replay_probe(
    session: &Session,
    body: Option<Value>,
    status: u16,
    media: &str,
    request: EndpointSelectionRequest,
    options: RequestOptions,
) -> (rrd_client::Result<EndpointSelectionProposal>, usize) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let client = client(listener.local_addr().unwrap());
    let media = media.to_owned();
    let server = tokio::spawn(async move {
        let mut calls = 0;
        while let Ok(Ok((mut stream, _))) =
            tokio::time::timeout(Duration::from_millis(300), listener.accept()).await
        {
            let bytes = read_request(&mut stream).await;
            let first = String::from_utf8_lossy(&bytes)
                .lines()
                .next()
                .unwrap()
                .to_owned();
            assert_eq!(
                first,
                format!("POST {} HTTP/1.1", TaskEndpointPropose::PATH)
            );
            calls += 1;
            if let Some(body) = &body {
                let bytes = serde_json::to_vec(body).unwrap();
                let header = format!("HTTP/1.1 {status} Probe\r\ncontent-type: {media}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", bytes.len());
                // Rejected headers can close the peer before body delivery.
                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(&bytes).await;
            }
            let _ = stream.shutdown().await;
        }
        calls
    });
    let result = client
        .call_operation::<TaskEndpointPropose>(session, request, options)
        .await;
    (result, server.await.unwrap())
}

#[tokio::test]
async fn retained_response_transport_checks_identity_status_media_and_never_retries() {
    let engine = Engine::start().await;
    let session = engine.session("allowed").await;
    let body = observation()["response"].clone();
    let (result, count) = replay_probe(
        &session,
        Some(body.clone()),
        200,
        "application/json",
        request(),
        options(),
    )
    .await;
    assert_eq!(
        result.unwrap(),
        serde_json::from_value::<EndpointSelectionProposal>(body["outcome"]["payload"].clone())
            .unwrap()
    );
    assert_eq!(count, 1);
    for (status, media) in [
        (201, "application/json"),
        (200, "text/plain"),
        (200, "application/json\r\ncontent-type: application/json"),
        (302, "application/json"),
    ] {
        let (result, count) = replay_probe(
            &session,
            Some(body.clone()),
            status,
            media,
            request(),
            options(),
        )
        .await;
        assert!(
            matches!(result, Err(Error::Decode(_))),
            "{status} {media}: {result:?}"
        );
        assert_eq!(count, 1);
    }
    let mut wrong = body.clone();
    wrong["request_id"] = "foreign".into();
    let (result, count) = replay_probe(
        &session,
        Some(wrong),
        200,
        "application/json",
        request(),
        options(),
    )
    .await;
    assert!(matches!(result, Err(Error::ResponseIdentityMismatch)));
    assert_eq!(count, 1);
    let mut wrong = body.clone();
    wrong["outcome"]["payload"]["attempt_id"] = "foreign".into();
    let (result, count) = replay_probe(
        &session,
        Some(wrong),
        200,
        "application/json",
        request(),
        options(),
    )
    .await;
    assert!(matches!(result, Err(Error::Contract(_))));
    assert_eq!(count, 1);
    let (result, count) = replay_probe(
        &session,
        None,
        200,
        "application/json",
        request(),
        options(),
    )
    .await;
    assert!(matches!(result, Err(Error::Transport(_))));
    assert_eq!(count, 1, "lost ACK must not redispatch");
    let mut invalid = request();
    invalid.expected_task_sha256.clear();
    let (result, count) = replay_probe(
        &session,
        Some(body.clone()),
        200,
        "application/json",
        invalid,
        options(),
    )
    .await;
    assert!(matches!(result, Err(Error::Contract(_))));
    assert_eq!(count, 0);
    let expired = RequestOptions::new("expired", "expired", Some("expired"), Some(1)).unwrap();
    let (result, count) = replay_probe(
        &session,
        Some(body.clone()),
        200,
        "application/json",
        request(),
        expired,
    )
    .await;
    assert!(matches!(result, Err(Error::Timeout)));
    assert_eq!(count, 0);
    let invalid = RequestOptions::read("read-only", "read-only").unwrap();
    let (result, count) = replay_probe(
        &session,
        Some(body),
        200,
        "application/json",
        request(),
        invalid,
    )
    .await;
    assert!(matches!(result, Err(Error::Contract(_))));
    assert_eq!(count, 0);
    engine.finish().await;
}

#[tokio::test]
async fn installed_engine_without_endpoint_policy_denies_generated_proposal() {
    let temporary = tempfile::tempdir().unwrap();
    let project = temporary.path().join("project");
    std::fs::create_dir(&project).unwrap();
    // Use the actual test executable as the embedded fixture's source identity.
    // This tests installation APIs, not qualification of the primary rrflow binary.
    let executable = std::env::current_exe().unwrap();
    let installation = RrdEngine::plan_installation(
        &project,
        InstallationTargetKind::ExistingProject,
        &RrdEngine::generate_installation_identity().unwrap(),
        "default",
        None,
        &executable,
    )
    .unwrap();
    RrdEngine::apply_installation(
        &project,
        InstallationTargetKind::ExistingProject,
        &installation,
        &installation.installation.plan_sha256,
        &executable,
    )
    .unwrap();
    let instance = installation.installation.target.instance_id.clone();
    let engine = RrdEngine::open_installed(&project, &executable).unwrap();
    let server = RrdHttpServer::bind(engine, "127.0.0.1:0".parse().unwrap()).unwrap();
    let client = RrdClient::connect_local(
        server.local_addr(),
        instance.clone(),
        ClientConfig::default(),
    )
    .unwrap();
    let (shutdown, receiver) = tokio::sync::oneshot::channel();
    let running = tokio::spawn(server.serve_until(async move {
        let _ = receiver.await;
    }));
    let credential = RrdEngine::read_installed_api_key(&project).unwrap();
    let session = client
        .create_session(
            credential.principal_id.clone(),
            credential.credential(),
            CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 60_000,
                    absolute_timeout_ms: 300_000,
                    max_open_transactions: 1,
                },
            },
            RequestOptions::mutation(
                "installed-session",
                "installed-session",
                "installed-session",
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let mut request = request();
    request.scope = format!("instance:{instance}");
    let error = client
        .call_operation::<TaskEndpointPropose>(&session, request, options())
        .await
        .unwrap_err();
    assert!(
        matches!(&error, Error::Api { status, error } if *status == hyper::StatusCode::BAD_REQUEST
        && error.code == ErrorCode::InvalidArgument && error.message.contains("endpoint configuration is absent")),
        "{error:?}"
    );
    shutdown.send(()).unwrap();
    running.await.unwrap().unwrap();
}
