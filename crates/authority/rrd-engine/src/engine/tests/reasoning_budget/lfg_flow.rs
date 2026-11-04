//! Synthetic HTTP services exercise the real engine composition, not model quality.
use super::*;
use rrd_contract::{
    ApplyEndpointConfiguration, EndpointLocator, EndpointProfileInput, InstalledEstateIdentity,
    LfgAnalysisPolicy, PreviewEndpointConfiguration, ReadTaskAttempt,
};
use std::io::{Read, Write};
use std::net::TcpListener;

fn serve_once(listener: TcpListener, respond: impl FnOnce(serde_json::Value) -> serde_json::Value) {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(value) => break value,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(error) => panic!("listener did not receive request: {error}"),
        }
    };
    // On Windows the accepted socket can inherit the listener's nonblocking mode.
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let request = loop {
        let count = stream.read(&mut chunk).unwrap();
        assert!(count > 0);
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(split) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&bytes[..split]);
            let length: usize = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse().unwrap())
                })
                .unwrap();
            if bytes.len() >= split + 4 + length {
                break serde_json::from_slice(&bytes[split + 4..split + 4 + length]).unwrap();
            }
        }
    };
    let body = serde_json::to_vec(&respond(request)).unwrap();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(&body).unwrap();
}

#[test]
fn real_engine_composes_bounded_lfg_then_controller_and_reopens_receipt() {
    run_flow(false, false, None);
}

#[test]
fn changed_configuration_during_lfg_records_result_without_controller_dispatch() {
    run_flow(true, false, None);
}

#[test]
fn controller_response_failure_keeps_lfg_checkpoint_and_uncertain_capacity() {
    run_flow(false, true, None);
}

#[test]
#[ignore = "requires an explicitly configured local LFG service with the fixture policy"]
fn real_lfg_service_flows_through_engine_controller_and_reopen() {
    let url = std::env::var("RRFLOW_TEST_LFG_URL").expect("configure the local LFG service origin");
    run_flow(false, false, Some(url));
}

fn run_flow(change_configuration: bool, reject_controller: bool, real_lfg: Option<String>) {
    let (root, mut engine, bootstrap, mut configuration, mut request, _, _) =
        proposal::tuple_fixture();
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../compute/rrd-inference/tests/fixtures/lfg-synthetic-observation.json");
    let fixture: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture_path).unwrap()).unwrap();
    let analyzer = if real_lfg.is_none() {
        Some(TcpListener::bind("127.0.0.1:0").unwrap())
    } else {
        None
    };
    let controller = TcpListener::bind("127.0.0.1:0").unwrap();
    engine.installed_estate = Some(InstalledEstateIdentity {
        project_id: CanonicalId::new("lfg-flow-project").unwrap(),
        estate_id: CanonicalId::new("lfg-flow-estate").unwrap(),
        instance_id: instance(),
    });
    let profile = configuration.profile.as_mut().unwrap();
    let policy = profile.selection_policy.as_mut().unwrap();
    policy.lfg = Some(LfgAnalysisPolicy {
        base_url: real_lfg.unwrap_or_else(|| {
            format!(
                "http://{}",
                analyzer.as_ref().unwrap().local_addr().unwrap()
            )
        }),
        assessment_sha256: fixture["request"]["expectedAssessmentSha256"]
            .as_str()
            .unwrap()
            .into(),
        rules_sha256: fixture["request"]["expectedRulesSha256"]
            .as_str()
            .unwrap()
            .into(),
        timeout_ms: 3_000,
        maximum_input_bytes: 65_536,
        maximum_request_bytes: 131_072,
        maximum_response_bytes: 65_536,
    });
    let binding = profile
        .bindings
        .iter_mut()
        .find(|item| item.id == policy.controller_binding_id)
        .unwrap();
    binding.endpoint = EndpointLocator::Url {
        url: format!("http://{}", controller.local_addr().unwrap()),
    };
    binding.credential = None;
    let input = EndpointProfileInput {
        contract_version: profile.contract_version,
        name: profile.name.clone(),
        bindings: profile.bindings.clone(),
        selection_policy: profile.selection_policy.clone(),
        observation_policy: profile.observation_policy.clone(),
    };
    let mut next_input = input.clone();
    next_input
        .selection_policy
        .as_mut()
        .unwrap()
        .system_prompt
        .push_str(" updated");
    let plan = engine
        .preview_endpoint_configuration(
            &bootstrap.session_id,
            &bootstrap.token,
            &PreviewEndpointConfiguration {
                expected_revision: 0,
                profile: Some(input),
            },
            60,
            "preview",
            "preview",
        )
        .unwrap();
    configuration = engine
        .apply_endpoint_configuration(
            &bootstrap.session_id,
            &bootstrap.token,
            &ApplyEndpointConfiguration {
                approved_plan_sha256: plan.plan_sha256.clone(),
                plan,
            },
            61,
            "apply",
            "apply",
        )
        .unwrap();
    request.expected_configuration_sha256 = configuration.sha256().unwrap();

    let principal_id = engine.instance.clone();
    let principal = rrd_security::Principal {
        id: principal_id.clone(),
        kind: rrd_security::PrincipalKind::Service,
        credential_sha256: digest::sha256_hex(bootstrap.token.as_str().as_bytes()),
        credential_revision: 1,
        not_before_unix_ms: 1,
        expires_at_unix_ms: u64::MAX,
        disabled: false,
        role_ids: Default::default(),
        grants: [
            SecurityAction::SessionCreate,
            SecurityAction::EndpointPropose,
            SecurityAction::TaskRead,
            SecurityAction::ReasoningWrite,
            SecurityAction::ReasoningRead,
            SecurityAction::EstateAdmin,
        ]
        .into_iter()
        .map(|action| rrd_security::ResourceGrant {
            action,
            resource_prefix: engine.instance_resource(),
            data_policy: None,
        })
        .collect(),
    };
    rrd_security::SecurityRepository::new(&engine.storage, engine.instance.clone())
        .initialize(
            rrd_security::SecurityState {
                format_version: rrd_security::SECURITY_FORMAT,
                revision: 1,
                principals: [(principal_id.clone(), principal)].into_iter().collect(),
                roles: Default::default(),
                identity_bindings: Default::default(),
                jwt_issuers: Default::default(),
            },
            70,
            "test",
            "initialize",
            "initialize",
        )
        .unwrap();
    let engine = Arc::new(engine);
    let edit_engine = Arc::clone(&engine);
    let lease = engine
        .create_authenticated_session(
            &principal_id,
            bootstrap.token.as_str().as_bytes(),
            &session_request(9_000, 4),
            &id("lfg-flow-session"),
            100,
            "session",
            "session",
        )
        .unwrap();
    let edit_lease = lease.clone();

    let analyzer_thread = analyzer.map(|analyzer| {
        std::thread::spawn(move || {
            serve_once(analyzer, |input| {
                assert_eq!(
                    digest::sha256_hex(input["text"].as_str().unwrap().as_bytes()),
                    input["inputSha256"]
                );
                if change_configuration {
                    let plan = edit_engine
                        .preview_endpoint_configuration(
                            &edit_lease.session_id,
                            &edit_lease.token,
                            &PreviewEndpointConfiguration {
                                expected_revision: 1,
                                profile: Some(next_input),
                            },
                            900,
                            "change-preview",
                            "change-preview",
                        )
                        .unwrap();
                    edit_engine
                        .apply_endpoint_configuration(
                            &edit_lease.session_id,
                            &edit_lease.token,
                            &ApplyEndpointConfiguration {
                                approved_plan_sha256: plan.plan_sha256.clone(),
                                plan,
                            },
                            901,
                            "change-apply",
                            "change-apply",
                        )
                        .unwrap();
                }
                let mut output = fixture["observation"].clone();
                for field in ["requestId", "revision", "inputSha256"] {
                    output[field] = input[field].clone();
                }
                output["assessmentSha256"] = input["expectedAssessmentSha256"].clone();
                output["rulesSha256"] = input["expectedRulesSha256"].clone();
                output
            })
        })
    });
    let response_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../transport/rrd-contract/tests/fixtures/endpoint-selection-observed/available/choice-response.json");
    let mut response: serde_json::Value =
        serde_json::from_slice(&std::fs::read(response_path).unwrap()).unwrap();
    if reject_controller {
        response = serde_json::json!({"synthetic": "invalid controller response"});
    }
    let (tx, rx) = mpsc::channel();
    let (controller_thread, idle_controller) = if change_configuration {
        controller.set_nonblocking(true).unwrap();
        (None, Some(controller))
    } else {
        (
            Some(std::thread::spawn(move || {
                serve_once(controller, |input| {
                    let packet: serde_json::Value =
                        serde_json::from_str(input["messages"][1]["content"].as_str().unwrap())
                            .unwrap();
                    tx.send(packet).unwrap();
                    response
                })
            })),
            None,
        )
    };
    let proposal = engine.propose_task_endpoint_with_clock(
        &lease.session_id,
        &lease.token,
        &request,
        "lfg-flow",
        "lfg-flow",
        &FixedClock::at(1_000),
    );
    if let Some(analyzer_thread) = analyzer_thread {
        analyzer_thread.join().unwrap();
    }
    let proposal = if change_configuration {
        assert!(matches!(
            proposal,
            Err(ServiceError::TaskAttemptInvalidated(
                rrd_contract::TaskAttemptInvalidation::InputsChanged
            ))
        ));
        assert!(
            matches!(idle_controller.unwrap().accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
        None
    } else {
        controller_thread.unwrap().join().unwrap();
        let packet = rx.recv().unwrap();
        assert_eq!(packet["lfg"]["request_id"], request.attempt_id.as_str());
        if reject_controller {
            assert!(matches!(
                proposal,
                Err(ServiceError::EndpointCallFailed {
                    execution_may_continue: true,
                    ..
                })
            ));
            None
        } else {
            let proposal = proposal.unwrap();
            assert_eq!(
                packet["lfg"]["response_sha256"],
                proposal.lfg.as_ref().unwrap().response_sha256
            );
            Some(proposal)
        }
    };
    let read_request = ReadTaskAttempt {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
    };
    let before = engine
        .read_task_attempt(
            &lease.session_id,
            &lease.token,
            &read_request,
            1_001,
            "read",
            "read",
        )
        .unwrap();
    assert!(before.attempt.as_ref().unwrap().lfg.is_some());
    assert_eq!(before.attempt.as_ref().unwrap().endpoint_proposal, proposal);
    if change_configuration {
        assert_eq!(
            before.attempt.as_ref().unwrap().status,
            rrd_contract::TaskAttemptStatus::Invalidated
        );
        let budget = state(&engine, &lease, &request.task_id);
        assert!(budget.active_attempt.is_none() && budget.stopped.is_none());
    }
    if reject_controller {
        assert_eq!(
            before.attempt.as_ref().unwrap().status,
            rrd_contract::TaskAttemptStatus::Stopped
        );
        assert!(state(&engine, &lease, &request.task_id).execution_may_continue);
    }
    drop(engine);
    let mut reopened = RrdEngine::open_existing(root.path(), instance(), TOKEN_KEY).unwrap();
    configure(&mut reopened, 3);
    assert_eq!(
        reopened
            .read_task_attempt(
                &lease.session_id,
                &lease.token,
                &read_request,
                1_002,
                "read",
                "read"
            )
            .unwrap(),
        before
    );
    assert!(reopened
        .propose_task_endpoint_with_clock(
            &lease.session_id,
            &lease.token,
            &request,
            "replay",
            "replay",
            &FixedClock::at(1_100)
        )
        .is_err());
}
