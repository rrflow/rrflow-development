//! Installed engine + native client + HTTP exporter integration. The loopback
//! receiver is a protocol fixture, not RROFLOW deployment or training evidence.
use rrd_client::{
    operations::{ObservationDeliver, ObservationPrepare, ObservationRead},
    ClientConfig, RequestOptions, RrdClient, Session,
};
use rrd_contract::*;
use rrd_core::digest::sha256_hex;
use rrd_engine::{EndpointCredentials, RrdEngine};
use rrd_server::RrdHttpServer;
use serde_json::{json, Value};
use std::{path::Path, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn id(value: &str) -> CanonicalId {
    CanonicalId::new(value).unwrap()
}
fn mutation(value: &str) -> RequestOptions {
    RequestOptions::mutation(value, value, value).unwrap()
}
fn read(value: &str) -> RequestOptions {
    RequestOptions::read(value, value).unwrap()
}
fn credential() -> CredentialReference {
    CredentialReference {
        provider: id("protocol-fixture"),
        name: "sink-key".into(),
    }
}

struct Running {
    client: RrdClient,
    session: Session,
    shutdown: tokio::sync::oneshot::Sender<()>,
    running: tokio::task::JoinHandle<std::result::Result<(), rrd_server::HttpError>>,
}
impl Running {
    async fn start(project: &Path, executable: &Path, instance: CanonicalId, label: &str) -> Self {
        let mut engine = RrdEngine::open_installed(project, executable).unwrap();
        engine
            .install_endpoint_credentials(
                EndpointCredentials::new(vec![(credential(), "protocol-test-secret".into())])
                    .unwrap(),
            )
            .unwrap();
        let server = RrdHttpServer::bind(engine, "127.0.0.1:0".parse().unwrap()).unwrap();
        let client = RrdClient::connect_local(
            server.local_addr(),
            instance,
            ClientConfig {
                request_timeout: Duration::from_secs(10),
                ..ClientConfig::default()
            },
        )
        .unwrap();
        let (shutdown, receiver) = tokio::sync::oneshot::channel();
        let running = tokio::spawn(server.serve_until(async move {
            let _ = receiver.await;
        }));
        let key = RrdEngine::read_installed_api_key(project).unwrap();
        let session = client
            .create_session(
                key.principal_id.clone(),
                key.credential(),
                CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 60_000,
                        absolute_timeout_ms: 300_000,
                        max_open_transactions: 1,
                    },
                },
                mutation(label),
            )
            .await
            .unwrap();
        Self {
            client,
            session,
            shutdown,
            running,
        }
    }
    async fn stop(self) {
        self.shutdown.send(()).unwrap();
        self.running.await.unwrap().unwrap();
    }
}

async fn receive(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        header.push(stream.read_u8().await.unwrap());
        assert!(header.len() < 16384);
    }
    let header = String::from_utf8(header).unwrap();
    assert!(header.starts_with("POST /v1/events HTTP/1.1\r\n"));
    let headers: Vec<_> = header
        .lines()
        .filter_map(|line| line.split_once(':'))
        .collect();
    assert!(headers
        .iter()
        .any(|(name, value)| name.eq_ignore_ascii_case("authorization")
            && value.trim() == "Bearer protocol-test-secret"));
    let length: usize = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .unwrap()
        .1
        .trim()
        .parse()
        .unwrap();
    assert!(length <= MAX_OBSERVATION_EVENT_BYTES);
    let mut body = vec![0; length];
    stream.read_exact(&mut body).await.unwrap();
    body
}

async fn task_source(engine: &Running, scope: String, project: &Path) -> ObservationSource {
    // Execute the bundled deterministic embedding adapter on actual local bytes.
    // This proves task-attempt transport; it is not an LLM quality observation.
    let task = engine
        .client
        .open_task(
            &engine.session,
            OpenTask {
                scope: scope.clone(),
                title: "Verify observation transport".into(),
                goal: "Retain and export an actual bounded attempt through public operations"
                    .into(),
            },
            mutation("open-task"),
        )
        .await
        .unwrap()
        .task
        .task_id;
    engine
        .client
        .configure_task_budget(
            &engine.session,
            ConfigureTaskBudget {
                scope: scope.clone(),
                task_id: task.clone(),
                repetition_policy: None,
                limits: ReasoningLimits {
                    max_run_elapsed_ms: 60000,
                    max_steps: 2,
                    max_step_elapsed_ms: 5000,
                },
            },
            mutation("configure-budget"),
        )
        .await
        .unwrap();
    let models = engine
        .client
        .activate_builtin_embedding(
            &engine.session,
            ActivateBuiltinEmbedding {
                scope: scope.clone(),
                dimensions: 32,
                seed: 42,
            },
            mutation("activate-builtin"),
        )
        .await
        .unwrap();
    let attempt_id = CorrelationId::new("bounded-embedding").unwrap();
    let output = engine
        .client
        .execute_task_embedding(
            &engine.session,
            ExecuteTaskEmbedding {
                task_id: task.clone(),
                attempt_id: attempt_id.clone(),
                request: GenerateEmbeddings {
                    scope: scope.clone(),
                    backend_id: models.backends[0].id.clone(),
                    network_policy: EmbeddingNetworkPolicy::Deny,
                    inputs: vec![EmbeddingInput {
                        id: id("project-readme"),
                        media_type: "text/plain".into(),
                        bytes: std::fs::read(project.join("README.md")).unwrap(),
                    }],
                },
            },
            mutation("execute-builtin"),
        )
        .await
        .unwrap();
    let request = ReadTaskAttempt {
        scope,
        task_id: task,
        attempt_id,
    };
    let receipt = engine
        .client
        .read_task_attempt(&engine.session, request.clone(), read("attempt-source"))
        .await
        .unwrap();
    let attempt = receipt.attempt.unwrap();
    assert_eq!(attempt.status, TaskAttemptStatus::Accepted);
    assert_eq!(attempt.result.as_ref(), Some(&output));
    ObservationSource::TaskAttempt {
        request,
        expected_status: attempt.status,
        expected_outcome_sha256: attempt.outcome_sha256,
    }
}

async fn round_trip(lose_first_reply: bool, observe_task: bool) {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(
        project.join("README.md"),
        "Local transport integration fixture.\n",
    )
    .unwrap();
    // Actual installation API, with the test executable as its source identity.
    // This does not qualify the separately shipped rrflow executable.
    let executable = std::env::current_exe().unwrap();
    let preview = RrdEngine::plan_installation(
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
        &preview,
        &preview.installation.plan_sha256,
        &executable,
    )
    .unwrap();
    let instance = preview.installation.target.instance_id.clone();
    let job = id(&format!("job-{}", &preview.attunement.plan_sha256[..24]));
    let engine = Running::start(&project, &executable, instance.clone(), "initial-session").await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sink_stop, mut sink_done) = tokio::sync::oneshot::channel();
    let sink = tokio::spawn(async move {
        let mut requests = Vec::new();
        let mut replies = Vec::new();
        loop {
            let (mut stream, _) = tokio::select! {
                _ = &mut sink_done => break,
                connection = listener.accept() => connection.unwrap(),
            };
            let body = tokio::time::timeout(Duration::from_secs(5), receive(&mut stream))
                .await
                .unwrap();
            let event: Value = serde_json::from_slice(&body).unwrap();
            requests.push(body);
            if lose_first_reply && requests.len() == 1 {
                continue;
            }
            let reply = json!({"devspace":"protocol-space", "accepted_ids":[event["events"][0]["event_id"]]}).to_string();
            let header = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", reply.len());
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(reply.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
            replies.push(reply);
        }
        (requests, replies)
    });
    let current = engine
        .client
        .read_endpoint_configuration(
            &engine.session,
            ReadEndpointConfiguration {},
            read("configuration"),
        )
        .await
        .unwrap();
    let profile: EndpointProfileInput = serde_json::from_value(json!({
        "contract_version":1,"name":"protocol-integration", "selection_policy":null,
        "bindings":[{"id":"sink","class":"behavior_updates","kind":"api",
            "endpoint":{"kind":"url","url":format!("http://{address}")},"service_id":"protocol-fixture",
            "manifest_sha256":null,"credential":credential(),"placement":null,
            "footprint":{"memory_bytes":1024,"gpu":{"kind":"none"}},
            "limits":{"max_concurrency":1,"timeout_ms":5000,"max_input_bytes":16384}}],
        "observation_policy":{"contract_version":1,"binding_id":"sink","expected_devspace":"protocol-space",
            "protocol_evidence_sha256":sha256_hex(include_bytes!("observation_delivery.rs")),
            "allow_network":true,"allow_metadata_export":true,"allowed_scopes":[format!("instance:{instance}")],
            "max_retained_deliveries":8,"retention_ms":60000,"retire_expired_uncertain":false,
            "max_attempts":3,"retry_delay_ms":1,"max_response_bytes":4096}
    })).unwrap();
    let plan = engine
        .client
        .preview_endpoint_configuration(
            &engine.session,
            PreviewEndpointConfiguration {
                expected_revision: current.revision,
                profile: Some(profile),
            },
            mutation("preview-configuration"),
        )
        .await
        .unwrap();
    let configuration = engine
        .client
        .apply_endpoint_configuration(
            &engine.session,
            ApplyEndpointConfiguration {
                approved_plan_sha256: plan.plan_sha256.clone(),
                plan,
            },
            mutation("approve-configuration"),
        )
        .await
        .unwrap();
    let source = engine
        .client
        .read_attunement(
            &engine.session,
            ReadAttunement {
                contract_version: ATTUNEMENT_CONTRACT_VERSION,
                job_id: job.clone(),
            },
            read("source"),
        )
        .await
        .unwrap();
    let observation_source = if observe_task {
        task_source(&engine, format!("instance:{instance}"), &project).await
    } else {
        ObservationSource::AttunementJob {
            request: ReadAttunement {
                contract_version: ATTUNEMENT_CONTRACT_VERSION,
                job_id: job.clone(),
            },
            expected_revision: source.status.job.revision,
            expected_plan_sha256: source.plan.plan_sha256.clone(),
        }
    };
    let request = PrepareObservationDelivery {
        contract_version: 1,
        delivery_id: CorrelationId::new("public-delivery").unwrap(),
        expected_configuration_sha256: configuration.sha256().unwrap(),
        source: observation_source.clone(),
    };
    let mut stale_configuration = request.clone();
    stale_configuration.expected_configuration_sha256 = "0".repeat(64);
    assert!(matches!(
        engine
            .client
            .call_operation::<ObservationPrepare>(
                &engine.session,
                stale_configuration,
                mutation("stale-configuration")
            )
            .await,
        Err(rrd_client::Error::Api { .. })
    ));
    let mut stale_source = request.clone();
    match &mut stale_source.source {
        ObservationSource::AttunementJob {
            expected_revision, ..
        } => *expected_revision += 1,
        ObservationSource::TaskAttempt {
            expected_outcome_sha256,
            ..
        } => *expected_outcome_sha256 = Some("0".repeat(64)),
    }
    assert!(matches!(
        engine
            .client
            .call_operation::<ObservationPrepare>(
                &engine.session,
                stale_source,
                mutation("stale-source")
            )
            .await,
        Err(rrd_client::Error::Api { .. })
    ));
    let absent = engine
        .client
        .call_operation::<ObservationRead>(
            &engine.session,
            ReadObservationDelivery {
                contract_version: 1,
                delivery_id: request.delivery_id.clone(),
            },
            read("failed-preparation-readback"),
        )
        .await
        .unwrap();
    assert!(absent.delivery.is_none());
    assert!(absent.source_response_json.is_none());
    let prepared = engine
        .client
        .call_operation::<ObservationPrepare>(&engine.session, request.clone(), mutation("prepare"))
        .await
        .unwrap();
    assert_eq!(prepared.state, ObservationDeliveryState::Prepared);
    assert_eq!(
        engine
            .client
            .call_operation::<ObservationPrepare>(
                &engine.session,
                request,
                mutation("prepare-again")
            )
            .await
            .unwrap(),
        prepared
    );
    let deliver = DeliverObservation {
        contract_version: 1,
        delivery_id: prepared.delivery_id.clone(),
        expected_payload_sha256: prepared.payload_sha256.clone(),
        expected_configuration_sha256: prepared.configuration_sha256.clone(),
    };
    let mut wrong_payload = deliver.clone();
    wrong_payload.expected_payload_sha256 = "0".repeat(64);
    assert!(matches!(
        engine
            .client
            .call_operation::<ObservationDeliver>(
                &engine.session,
                wrong_payload,
                mutation("wrong-payload")
            )
            .await,
        Err(rrd_client::Error::Api { .. })
    ));
    let first = engine
        .client
        .call_operation::<ObservationDeliver>(&engine.session, deliver.clone(), mutation("deliver"))
        .await
        .unwrap();
    assert_eq!(first.attempts, 1);
    assert_eq!(
        first.state,
        if lose_first_reply {
            ObservationDeliveryState::Uncertain
        } else {
            ObservationDeliveryState::Acknowledged
        }
    );
    engine.stop().await;
    let engine = Running::start(&project, &executable, instance.clone(), "reopened-session").await;
    let read_request = ReadObservationDelivery {
        contract_version: 1,
        delivery_id: prepared.delivery_id.clone(),
    };
    let retained = engine
        .client
        .call_operation::<ObservationRead>(&engine.session, read_request.clone(), read("retained"))
        .await
        .unwrap();
    assert_eq!(retained.delivery.as_ref().unwrap(), &first);
    let source_json = retained.source_response_json.as_ref().unwrap();
    assert_eq!(
        sha256_hex(source_json.as_bytes()),
        prepared.source_response_sha256
    );
    match &observation_source {
        ObservationSource::AttunementJob { .. } => {
            let historical: AttunementReadResult = serde_json::from_str(source_json).unwrap();
            assert_eq!(historical.status.job.id, job);
            assert_eq!(historical.plan, source.plan);
        }
        ObservationSource::TaskAttempt {
            request,
            expected_status,
            expected_outcome_sha256,
        } => {
            let historical: TaskAttemptView = serde_json::from_str(source_json).unwrap();
            let attempt = historical.attempt.unwrap();
            assert_eq!(&attempt.attempt_id, &request.attempt_id);
            assert_eq!(&attempt.status, expected_status);
            assert_eq!(&attempt.outcome_sha256, expected_outcome_sha256);
            assert!(attempt.result.is_some());
        }
    }
    // Explicit retry after persisted cooldown; never an SDK retry.
    tokio::time::sleep(Duration::from_millis(2)).await;
    let acknowledged = engine
        .client
        .call_operation::<ObservationDeliver>(
            &engine.session,
            deliver.clone(),
            mutation("explicit-retry"),
        )
        .await
        .unwrap();
    assert_eq!(acknowledged.state, ObservationDeliveryState::Acknowledged);
    assert_eq!(acknowledged.attempts, if lose_first_reply { 2 } else { 1 });
    assert_eq!(
        engine
            .client
            .call_operation::<ObservationDeliver>(
                &engine.session,
                deliver,
                mutation("acknowledged-replay")
            )
            .await
            .unwrap(),
        acknowledged
    );
    let readback = engine
        .client
        .call_operation::<ObservationRead>(&engine.session, read_request, read("final-read"))
        .await
        .unwrap();
    assert_eq!(readback.delivery.as_ref(), Some(&acknowledged));
    assert_eq!(readback.source_response_json, retained.source_response_json);
    engine.stop().await;
    sink_stop.send(()).unwrap();
    let (requests, replies) = sink.await.unwrap();
    assert_eq!(requests.len(), acknowledged.attempts as usize);
    assert!(requests.iter().all(|body| body == &requests[0]));
    assert_eq!(sha256_hex(&requests[0]), prepared.payload_sha256);
    assert_eq!(
        acknowledged.acknowledgement_sha256.as_deref(),
        Some(sha256_hex(replies.last().unwrap().as_bytes()).as_str())
    );
    let exported: Value = serde_json::from_slice(&requests[0]).unwrap();
    let event = &exported["events"][0];
    assert_eq!(event["event_id"], prepared.event_id);
    assert_eq!(event["engine"]["instance_id"], instance.as_str());
    match &observation_source {
        ObservationSource::AttunementJob { .. } => {
            assert_eq!(event["engine"]["job_id"], job.as_str())
        }
        ObservationSource::TaskAttempt { request, .. } => {
            assert_eq!(event["engine"]["task_id"], request.task_id.as_str());
            assert_eq!(event["engine"]["attempt_id"], request.attempt_id.as_str());
            assert!(event["engine"]["job_id"].is_null());
        }
    }
    assert_eq!(
        event["engine"]["response_sha256"],
        prepared.source_response_sha256
    );
    assert!(event.get("payload").is_none());
    assert!(!String::from_utf8(requests[0].clone())
        .unwrap()
        .contains("Local transport integration fixture"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_delivery_acknowledgement_survives_restart_without_redispatch() {
    round_trip(false, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_delivery_lost_reply_retries_identical_event_after_restart() {
    round_trip(true, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_delivery_retains_actual_task_attempt_and_exports_only_references() {
    round_trip(false, true).await;
}
