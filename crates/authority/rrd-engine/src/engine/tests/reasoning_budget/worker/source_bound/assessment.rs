use super::*;
use rrd_contract::{
    AssessmentInputKind, AssessmentPracticeIdentity, AssessmentScoreDirection,
    EndpointAssessmentPolicy, EndpointConfiguration, EndpointGenerationSettings,
    EndpointProfileInput, EndpointResponseFormat, EndpointTokenParameter,
    PreviewEndpointConfiguration, ReadTaskAssessment, TaskAssessmentDecision,
    TaskAssessmentDescriptor, TaskAssessmentRequest, TaskAssessmentValue, TASK_ASSESSMENT_VERSION,
};
use std::sync::atomic::{AtomicU64, Ordering};

struct AssessmentClock(AtomicU64);

impl AssessmentClock {
    fn at(at: u64) -> Self {
        Self(AtomicU64::new(at))
    }

    fn advance_to(&self, at: u64) {
        self.0.fetch_max(at, Ordering::SeqCst);
    }
}

impl crate::engine::clock::ClockSource for AssessmentClock {
    fn observe(
        &self,
    ) -> std::result::Result<
        crate::engine::clock::ClockObservation,
        crate::engine::clock::ClockObservationFailure,
    > {
        Ok(crate::engine::clock::ClockObservation {
            format_version: crate::engine::clock::CLOCK_OBSERVATION_FORMAT_VERSION,
            source: crate::engine::clock::ClockSourceKind::HostSystemTime,
            trust: crate::engine::clock::ClockTrustLevel::Unverified,
            observed_at_unix_ms: self.0.load(Ordering::SeqCst),
        })
    }
}

fn publish_assessment_policy(
    fixture: &mut SourceWorkerFixture,
    allow_network: bool,
    allow_source_export: bool,
    allow_context_export: bool,
) -> EndpointConfiguration {
    publish_assessment_policy_with_prompt(
        fixture,
        allow_network,
        allow_source_export,
        allow_context_export,
        "Return a typed advisory assessment of the supplied packet.",
    )
}

fn publish_assessment_policy_with_prompt(
    fixture: &mut SourceWorkerFixture,
    allow_network: bool,
    allow_source_export: bool,
    allow_context_export: bool,
    system_prompt: &str,
) -> EndpointConfiguration {
    publish_assessment_policy_with_prompt_for(
        &fixture.engine,
        &fixture.lease,
        &mut fixture.now,
        &mut fixture.selection,
        allow_network,
        allow_source_export,
        allow_context_export,
        system_prompt,
    )
}

fn publish_assessment_policy_with_prompt_for(
    engine: &RrdEngine,
    lease: &SessionLease,
    fixture_now: &mut u64,
    selection: &mut EndpointSelectionRequest,
    allow_network: bool,
    allow_source_export: bool,
    allow_context_export: bool,
    system_prompt: &str,
) -> EndpointConfiguration {
    let read_at = *fixture_now + 1;
    let current = engine
        .read_endpoint_configuration(
            &lease.session_id,
            &lease.token,
            read_at,
            &format!("assessment-config-read-{read_at}"),
            &format!("assessment-config-read-{read_at}"),
        )
        .unwrap();
    let profile = current.profile.unwrap();
    let mut selection_policy = profile.selection_policy.clone().unwrap();
    selection_policy.maximum_concurrent_proposals = 1;
    let controller = selection_policy.controller_binding_id.clone();
    let mut bindings = profile.bindings;
    for binding in &mut bindings {
        binding.limits.max_concurrency = 1;
    }
    let mut worker_policy = profile.worker_policy.clone().unwrap();
    worker_policy.maximum_concurrent_calls = 1;
    let binding_limit = bindings
        .iter()
        .find(|binding| binding.id == controller)
        .unwrap()
        .limits
        .max_input_bytes;
    let assessment_policy = EndpointAssessmentPolicy {
        contract_version: TASK_ASSESSMENT_VERSION,
        allow_network,
        allow_source_export,
        allow_context_export,
        system_prompt: system_prompt.into(),
        generation: EndpointGenerationSettings {
            maximum_output_tokens: 256,
            token_parameter: EndpointTokenParameter::MaxTokens,
            temperature_milli: None,
            response_format: EndpointResponseFormat::JsonObject,
        },
        maximum_input_bytes: binding_limit.min(64 * 1024),
        maximum_response_bytes: 16 * 1024,
    };
    let input = EndpointProfileInput {
        contract_version: profile.contract_version,
        name: profile.name,
        bindings,
        selection_policy: Some(selection_policy),
        worker_policy: Some(worker_policy),
        observation_policy: profile.observation_policy,
        assessment_policy: Some(assessment_policy),
    };
    let preview_at = read_at + 1;
    let plan = engine
        .preview_endpoint_configuration(
            &lease.session_id,
            &lease.token,
            &PreviewEndpointConfiguration {
                expected_revision: current.revision,
                profile: Some(input),
            },
            preview_at,
            &format!("assessment-config-preview-{preview_at}"),
            &format!("assessment-config-preview-{preview_at}"),
        )
        .unwrap();
    let applied = engine
        .apply_endpoint_configuration(
            &lease.session_id,
            &lease.token,
            &rrd_contract::ApplyEndpointConfiguration {
                approved_plan_sha256: plan.plan_sha256.clone(),
                plan,
            },
            preview_at + 1,
            &format!("assessment-config-apply-{}", preview_at + 1),
            &format!("assessment-config-apply-{}", preview_at + 1),
        )
        .unwrap();
    *fixture_now = preview_at + 2;
    selection.expected_configuration_sha256 = applied.sha256().unwrap();
    applied
}

fn source_worker_assessment_fixture(listener: &TcpListener) -> SourceWorkerFixture {
    let base = host_unix_ms().saturating_sub(30_000);
    source_worker_fixture_for_path_at(listener, true, false, "alpha.txt", base)
}

fn configure_assessment_budget(fixture: &mut SourceWorkerFixture) {
    configure_assessment_budget_with_repetition(fixture, None);
}

fn configure_assessment_budget_with_repetition(
    fixture: &mut SourceWorkerFixture,
    max_equivalent_attempts: Option<u64>,
) {
    let at = fixture.now + 1;
    let configured = fixture
        .engine
        .configure_task_budget(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ConfigureTaskBudget {
                scope: fixture.selection.scope.clone(),
                task_id: fixture.selection.task_id.clone(),
                limits: fixture.engine.estate_configuration.reasoning.clone(),
                repetition_policy: max_equivalent_attempts.map(|max_equivalent_attempts| {
                    rrd_contract::TaskRepetitionPolicy {
                        max_equivalent_attempts,
                    }
                }),
            },
            at,
            "assessment-budget-configure",
            "assessment-budget-configure",
        )
        .unwrap();
    let budget = configured.budget.expect("public budget was configured");
    fixture.now = at.max(budget.last_observed_at_unix_ms);
}

fn publish_metadata_observation_policy(fixture: &mut SourceWorkerFixture) -> EndpointConfiguration {
    let read_at = fixture.now + 1;
    let current = fixture
        .engine
        .read_endpoint_configuration(
            &fixture.lease.session_id,
            &fixture.lease.token,
            read_at,
            "assessment-observation-config-read",
            "assessment-observation-config-read",
        )
        .unwrap();
    let profile = current.profile.unwrap();
    let binding_id = CanonicalId::new("assessment-metadata-observer").unwrap();
    let credential = rrd_contract::CredentialReference {
        provider: CanonicalId::new("assessment-test-registry").unwrap(),
        name: "metadata-observation".into(),
    };
    let mut bindings = profile.bindings;
    bindings.push(rrd_contract::EndpointBinding {
        id: binding_id.clone(),
        class: rrd_contract::WorkloadClass::BehaviorUpdates,
        kind: rrd_contract::BindingKind::Api,
        endpoint: rrd_contract::EndpointLocator::Url {
            url: "http://127.0.0.1:9/".into(),
        },
        service_id: CanonicalId::new("assessment-metadata-sink").unwrap(),
        manifest_sha256: None,
        credential: Some(credential),
        placement: None,
        footprint: rrd_contract::DeclaredFootprint {
            memory_bytes: 1,
            gpu: rrd_contract::GpuRequirement::None,
        },
        limits: rrd_contract::BindingLimits {
            max_concurrency: 1,
            timeout_ms: 1_000,
            max_input_bytes: 16_384,
        },
    });
    let input = EndpointProfileInput {
        contract_version: profile.contract_version,
        name: profile.name,
        bindings,
        selection_policy: profile.selection_policy,
        worker_policy: profile.worker_policy,
        observation_policy: Some(rrd_contract::ObservationExportPolicy {
            contract_version: rrd_contract::OBSERVATION_DELIVERY_VERSION,
            binding_id,
            expected_devspace: "test-devspace".into(),
            protocol_evidence_sha256: digest::sha256_hex(
                b"controlled metadata-only observation preparation; no receiver call",
            ),
            allow_network: true,
            allow_metadata_export: true,
            allowed_scopes: vec![fixture.selection.scope.clone()],
            max_retained_deliveries: 1,
            retention_ms: 10_000,
            retire_expired_uncertain: false,
            max_attempts: 1,
            retry_delay_ms: 10,
            max_response_bytes: 1_024,
        }),
        assessment_policy: profile.assessment_policy,
    };
    let preview_at = read_at + 1;
    let plan = fixture
        .engine
        .preview_endpoint_configuration(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &PreviewEndpointConfiguration {
                expected_revision: current.revision,
                profile: Some(input),
            },
            preview_at,
            "assessment-observation-config-preview",
            "assessment-observation-config-preview",
        )
        .unwrap();
    let applied = fixture
        .engine
        .apply_endpoint_configuration(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ApplyEndpointConfiguration {
                approved_plan_sha256: plan.plan_sha256.clone(),
                plan,
            },
            preview_at + 1,
            "assessment-observation-config-apply",
            "assessment-observation-config-apply",
        )
        .unwrap();
    fixture.now = preview_at + 2;
    applied
}

fn assessment_request(
    fixture: &SourceWorkerFixture,
    suffix: &str,
    descriptor: TaskAssessmentDescriptor,
) -> TaskAssessmentRequest {
    TaskAssessmentRequest {
        contract_version: TASK_ASSESSMENT_VERSION,
        scope: fixture.selection.scope.clone(),
        task_id: fixture.selection.task_id.clone(),
        attempt_id: id(&format!("assessment-{suffix}")),
        expected_task_sha256: fixture.selection.expected_task_sha256.clone(),
        expected_configuration_sha256: fixture.selection.expected_configuration_sha256.clone(),
        max_storage_keys: MAX_TASK_STORAGE_KEYS,
        source: fixture.source.clone(),
        descriptor,
        practice_identity: AssessmentPracticeIdentity {
            question_set_id: CanonicalId::new("assessment-set").unwrap(),
            question_id: CanonicalId::new("assessment-question").unwrap(),
            template_id: CanonicalId::new("assessment-template").unwrap(),
            template_revision: 1,
            template_sha256: digest::sha256_hex(b"controlled practice declaration"),
            requirement_id: CanonicalId::new("assessment-requirement").unwrap(),
        },
        practice_text: "Check only the selected source and the declared rubric.".into(),
    }
}

fn binary_descriptor() -> TaskAssessmentDescriptor {
    TaskAssessmentDescriptor::Binary {
        proposition: "The selected source contains the named declaration".into(),
    }
}

fn assessment_reply(packet: &serde_json::Value, value: bool) -> String {
    assessment_content(
        packet,
        serde_json::json!({ "outcome": "answered", "value": { "kind": "binary", "value": value } }),
        Some("source"),
    )
}

fn assessment_content(
    packet: &serde_json::Value,
    decision: serde_json::Value,
    evidence_input: Option<&str>,
) -> String {
    let references = evidence_input
        .map(|input| {
            let sha256 = match input {
                "source" => packet["source"]["content_sha256"].as_str().unwrap(),
                "practice" => packet["practice_sha256"].as_str().unwrap(),
                _ => panic!("test fixture uses only declared packet inputs"),
            };
            serde_json::json!([{
                "input": input,
                "input_sha256": sha256,
                "start_byte": 0,
                "end_byte": 1
            }])
        })
        .unwrap_or_else(|| serde_json::json!([]));
    serde_json::json!({
        "contract_version": TASK_ASSESSMENT_VERSION,
        "decision": decision,
        "references": references
    })
    .to_string()
}

fn receive_assessment(listener: &TcpListener) -> (std::net::TcpStream, serde_json::Value, String) {
    let (socket, body) = read_wire(listener);
    let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let model = wire["model"].as_str().unwrap().to_owned();
    let packet: serde_json::Value =
        serde_json::from_str(wire["messages"][1]["content"].as_str().unwrap()).unwrap();
    (socket, packet, model)
}

fn run_assessment(
    fixture: &SourceWorkerFixture,
    listener: &TcpListener,
    request: &TaskAssessmentRequest,
    decision: serde_json::Value,
    evidence_input: Option<&str>,
) -> (rrd_contract::TaskAssessmentOutput, serde_json::Value) {
    std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, packet, model) = receive_assessment(listener);
            let body = assessment_content(&packet, decision, evidence_input);
            let response = reply(&mut socket, &model, &body);
            assert!(!response.is_empty());
            packet
        });
        let output = fixture
            .engine
            .execute_task_assessment_with_clock(
                &fixture.lease.session_id,
                &fixture.lease.token,
                request,
                request.attempt_id.as_str(),
                request.attempt_id.as_str(),
                &crate::engine::clock::FixedClock::at(fixture.now + 1),
            )
            .unwrap();
        (output, server.join().unwrap())
    })
}

#[test]
fn controlled_assessment_is_typed_retained_and_currently_authorized_on_readback() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "happy", binary_descriptor());
    assert!(
        fixture
            .engine
            .load_endpoint_configuration()
            .unwrap()
            .1
            .profile
            .unwrap()
            .assessment_policy
            .unwrap()
            .allow_network
    );
    let budget_before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(budget_before.reserved_steps, 0);
    let assessment_at = fixture.now + 1;
    assert!(assessment_at > budget_before.started_at_unix_ms);
    assert!(assessment_at < budget_before.deadline_unix_ms);
    assert!(assessment_at < fixture.lease.absolute_expires_at_unix_ms);
    let (output, (packet, response)) = std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, packet, model) = receive_assessment(&listener);
            assert_eq!(packet["source_content"], fixture.source_content);
            assert!(packet.get("context").is_none());
            let body = assessment_reply(&packet, true);
            let response = reply(&mut socket, &model, &body);
            (packet, response)
        });
        let output = fixture
            .engine
            .execute_task_assessment_with_clock(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &request,
                "assessment-execute-happy",
                "assessment-execute-happy",
                &crate::engine::clock::FixedClock::at(assessment_at),
            )
            .unwrap();
        let (packet, response) = server.join().unwrap();
        (output, (packet, response))
    });
    assert_eq!(packet["source_content"], fixture.source_content);
    assert_eq!(output.source.entry_identity, fixture.source.entry_identity);
    assert_eq!(
        output.source.content_sha256,
        fixture.source.expected_content_sha256
    );
    assert!(matches!(
        &output.decision,
        TaskAssessmentDecision::Answered {
            value: TaskAssessmentValue::Binary { value: true }
        }
    ));
    assert_eq!(output.references.len(), 1);
    assert_eq!(output.references[0].input, AssessmentInputKind::Source);
    assert!(!response.is_empty());
    let budget_after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(
        budget_after.reserved_steps,
        budget_before.reserved_steps + 1
    );
    assert_eq!(budget_after.reserved_steps, 1);
    assert!(budget_after.active_attempt.is_none());
    assert!(!budget_after.execution_may_continue);
    assert_eq!(
        budget_after.deadline_unix_ms,
        budget_before.deadline_unix_ms
    );
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
    listener.set_nonblocking(false).unwrap();

    let read_request = ReadTaskAssessment {
        contract_version: TASK_ASSESSMENT_VERSION,
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
    };
    std::fs::write(
        fixture.installed.root.path().join(&fixture.source_path),
        "same selected path, changed ordinary contents\n",
    )
    .unwrap();
    let retained = fixture
        .engine
        .read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_request,
            fixture.now + 3,
            "assessment-read-after-edit",
            "assessment-read-after-edit",
        )
        .unwrap();
    assert_eq!(retained.assessment, Some(output.clone()));

    publish_assessment_policy(&mut fixture, true, false, true);
    assert!(matches!(
        fixture.engine.read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_request,
            fixture.now + 1,
            "assessment-read-source-export-disabled",
            "assessment-read-source-export-disabled",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    publish_assessment_policy(&mut fixture, true, true, false);
    assert!(matches!(
        fixture.engine.read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_request,
            fixture.now + 1,
            "assessment-read-context-export-disabled",
            "assessment-read-context-export-disabled",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    publish_assessment_policy(&mut fixture, true, true, true);
    assert_eq!(
        fixture
            .engine
            .read_task_assessment(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &read_request,
                fixture.now + 1,
                "assessment-read-export-restored",
                "assessment-read-export-restored",
            )
            .unwrap()
            .assessment,
        Some(output.clone())
    );

    revoke_source_read(
        &fixture.engine,
        &fixture.lease,
        &fixture.principal_id,
        fixture.now + 2,
    );
    assert!(matches!(
        fixture.engine.read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_request,
            fixture.now + 3,
            "assessment-read-source-revoked",
            "assessment-read-source-revoked",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    restore_source_read(
        &fixture.engine,
        &fixture.lease,
        &fixture.principal_id,
        &fixture.installed.preview.installation.target.project_id,
        &fixture.source.entry_identity,
        fixture.now + 4,
    );
    let restored = fixture
        .engine
        .read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_request,
            fixture.now + 5,
            "assessment-read-restored",
            "assessment-read-restored",
        )
        .unwrap();
    assert_eq!(restored.assessment, Some(output));

    let metadata_configuration = publish_metadata_observation_policy(&mut fixture);
    let attempt_request = rrd_contract::ReadTaskAttempt {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
    };
    let attempt_view = fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &attempt_request,
            fixture.now + 1,
            "assessment-attempt-metadata-read",
            "assessment-attempt-metadata-read",
        )
        .unwrap();
    let attempt = attempt_view.attempt.as_ref().unwrap();
    assert_eq!(attempt.status, rrd_contract::TaskAttemptStatus::Accepted);
    assert!(attempt.outcome_sha256.is_some());
    assert!(attempt.endpoint_proposal.is_none() && attempt.worker_output.is_none());
    let attempt_json = serde_json::to_value(&attempt_view).unwrap();
    assert!(attempt_json.pointer("/attempt/assessment_output").is_none());
    assert!(attempt_json.pointer("/attempt/decision").is_none());

    let delivery_id = id("assessment-metadata-only-observation");
    let prepare_request = rrd_contract::PrepareObservationDelivery {
        contract_version: rrd_contract::OBSERVATION_DELIVERY_VERSION,
        delivery_id: delivery_id.clone(),
        source: rrd_contract::ObservationSource::TaskAttempt {
            request: attempt_request.clone(),
            expected_status: rrd_contract::TaskAttemptStatus::Accepted,
            expected_outcome_sha256: attempt.outcome_sha256.clone(),
        },
        expected_configuration_sha256: metadata_configuration.sha256().unwrap(),
    };
    let prepared = fixture
        .engine
        .prepare_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &prepare_request,
            fixture.now + 2,
            "assessment-metadata-observation-prepare",
            "assessment-metadata-observation-prepare",
        )
        .unwrap();
    let observation_key = format!(
        "server/state/{}/observation-delivery/record/{}",
        fixture.engine.instance,
        operation_digest(&(&fixture.principal_id, &delivery_id)).unwrap()
    );
    let observation_bytes = fixture
        .engine
        .storage
        .control()
        .get(&observation_key)
        .unwrap()
        .unwrap();
    let observation_record: serde_json::Value = serde_json::from_slice(&observation_bytes).unwrap();
    for field in ["source_json", "payload_json"] {
        let encoded = observation_record[field].as_str().unwrap();
        assert!(!encoded.contains("decision"));
        assert!(!encoded.contains("references"));
        assert!(!encoded.contains(&fixture.source_content));
        assert!(!encoded.contains(&request.practice_text));
    }
    assert_eq!(
        observation_record["snapshot"]["payload_sha256"],
        prepared.payload_sha256
    );

    revoke_source_read(
        &fixture.engine,
        &fixture.lease,
        &fixture.principal_id,
        fixture.now + 3,
    );
    assert!(matches!(
        fixture.engine.read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_request,
            fixture.now + 4,
            "assessment-direct-output-denied-after-revoke",
            "assessment-direct-output-denied-after-revoke",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    let metadata_after_revoke = fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &attempt_request,
            fixture.now + 5,
            "assessment-attempt-metadata-after-revoke",
            "assessment-attempt-metadata-after-revoke",
        )
        .unwrap()
        .attempt
        .unwrap();
    assert_eq!(
        metadata_after_revoke.status,
        rrd_contract::TaskAttemptStatus::Accepted
    );
    assert_eq!(metadata_after_revoke.outcome_sha256, attempt.outcome_sha256);
    assert!(metadata_after_revoke.worker_output.is_none());
    let observed_metadata = fixture
        .engine
        .read_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ReadObservationDelivery {
                contract_version: rrd_contract::OBSERVATION_DELIVERY_VERSION,
                delivery_id,
            },
            fixture.now + 6,
            "assessment-metadata-observation-read-after-revoke",
            "assessment-metadata-observation-read-after-revoke",
        )
        .unwrap()
        .delivery
        .unwrap();
    assert_eq!(observed_metadata, prepared);
}

#[test]
fn unrelated_accepted_inventory_advance_preserves_selected_assessment_input() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_fixture_for_path_at(
        &listener,
        true,
        true,
        "nested/alpha.txt",
        host_unix_ms().saturating_sub(30_000),
    );
    let project_id = fixture
        .installed
        .preview
        .installation
        .target
        .project_id
        .clone();
    let original = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let original_selected = original
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap()
        .clone();

    let refreshed_sha256 = refresh_unrelated(
        fixture.installed.root.path(),
        &fixture.engine,
        &fixture.lease,
        &project_id,
        &mut fixture.job,
        "unrelated-assessment.txt",
    );
    assert_ne!(refreshed_sha256, original.proposal_sha256);
    let refreshed = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let refreshed_selected = refreshed
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap();
    assert_eq!(refreshed_selected.identity, original_selected.identity);
    assert_eq!(refreshed_selected.kind, original_selected.kind);
    fixture.now = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);

    let request = assessment_request(&fixture, "unrelated-head", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);
    let (output, packet) = run_assessment(
        &fixture,
        &listener,
        &request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "binary", "value": true }
        }),
        Some("source"),
    );
    assert_eq!(
        packet["source"]["entry_identity"],
        fixture.source.entry_identity
    );
    assert_eq!(packet["source_content"], fixture.source_content);
    assert_eq!(output.source.entry_identity, original_selected.identity);
    assert_eq!(
        output.source.content_sha256,
        fixture.source.expected_content_sha256
    );
    let after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(after.reserved_steps, before.reserved_steps + 1);
    assert_eq!(after.deadline_unix_ms, before.deadline_unix_ms);
}

#[test]
fn unrelated_accepted_head_advance_during_assessment_preserves_output_and_single_dispatch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_fixture_for_path_at(
        &listener,
        true,
        true,
        "nested/alpha.txt",
        host_unix_ms().saturating_sub(30_000),
    );
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "unrelated-head-during", binary_descriptor());
    let before_budget = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before_budget.reserved_steps, 0);
    let before = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &fixture.installed.preview.installation.target.project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let before_entry = before
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap()
        .clone();
    let start_at = fixture.now + 1;
    assert!(start_at > before_budget.started_at_unix_ms);
    assert!(start_at < before_budget.deadline_unix_ms);
    assert!(start_at < fixture.lease.absolute_expires_at_unix_ms);
    let clock = AssessmentClock::at(start_at);
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let engine = &fixture.engine;
    let session = &fixture.lease.session_id;
    let token = &fixture.lease.token;
    let root = fixture.installed.root.path();
    let project_id = fixture
        .installed
        .preview
        .installation
        .target
        .project_id
        .clone();
    let selected_path = fixture.source_path.clone();
    let (result, (packet, response)) = std::thread::scope(|scope| {
        let expected_content = fixture.source_content.clone();
        let listener_ref = &listener;
        let server = scope.spawn(move || {
            let (mut socket, packet, model) = receive_assessment(listener_ref);
            assert_eq!(packet["source_content"], expected_content);
            entered_tx.send(()).unwrap();
            release_rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            let response = reply(&mut socket, &model, &assessment_reply(&packet, true));
            (packet, response)
        });
        let clock_ref = &clock;
        let worker = scope.spawn(|| {
            engine.execute_task_assessment_with_clock(
                session,
                token,
                &request,
                "assessment-unrelated-head-during",
                "assessment-unrelated-head-during",
                clock_ref,
            )
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let profile = engine
            .load_endpoint_configuration()
            .unwrap()
            .1
            .profile
            .unwrap();
        let selector = profile.selection_policy.as_ref().unwrap();
        let controller = selector.controller_binding_id.clone();
        let binding_limit = profile
            .bindings
            .iter()
            .find(|binding| binding.id == controller)
            .unwrap()
            .limits
            .max_concurrency;
        assert_eq!(selector.maximum_concurrent_proposals, 1);
        assert_eq!(binding_limit, 1);
        let worker_binding = profile.worker_policy.as_ref().unwrap().bindings[0]
            .binding_id
            .clone();
        assert_eq!(worker_binding, controller);
        let capacity_key = format!(
            "server/state/{}/endpoint-proposal/capacity",
            engine.instance
        );
        let capacity_bytes = engine
            .storage
            .control()
            .get(&capacity_key)
            .unwrap()
            .unwrap();
        let capacity_value: serde_json::Value = serde_json::from_slice(&capacity_bytes).unwrap();
        assert_eq!(capacity_value["assessments"].as_object().unwrap().len(), 1);

        let mut competing_selection = fixture.selection.clone();
        competing_selection.attempt_id = id("assessment-capacity-competing-proposal");
        let competing_proposal = crate::engine::endpoint_proposal::capacity::CapacityLease {
            request: competing_selection,
            controller_binding_id: controller.clone(),
        };
        assert!(matches!(
            engine.endpoint_capacity_transition(
                &competing_proposal,
                Some((selector.maximum_concurrent_proposals, binding_limit))
            ),
            Err(ServiceError::InferenceBusy)
        ));
        let competing_worker = crate::engine::endpoint_proposal::capacity::WorkerCapacityLease {
            scope: fixture.selection.scope.clone(),
            task_id: fixture.selection.task_id.clone(),
            attempt_id: id("assessment-capacity-competing-worker"),
            binding_id: worker_binding,
        };
        assert!(matches!(
            engine.worker_capacity_transition(&competing_worker, Some((1, binding_limit))),
            Err(ServiceError::InferenceBusy)
        ));
        let new_head = refresh_unrelated_with_host_floor(
            root,
            engine,
            &fixture.lease,
            &project_id,
            &mut fixture.job,
            "unrelated-during-assessment.txt",
            Some(start_at),
        );
        assert_ne!(new_head, before.proposal_sha256);
        let after = crate::engine::attunement::load_accepted_source(
            &fixture.engine,
            &project_id,
            8 * 1024 * 1024,
            10_000,
        )
        .unwrap();
        let after_entry = after
            .proposal
            .entries()
            .iter()
            .find(|entry| entry.path == selected_path)
            .unwrap();
        assert_eq!(after_entry.identity, before_entry.identity);
        assert_eq!(after_entry.kind, before_entry.kind);
        let completion_at = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
        assert!(completion_at < before_budget.deadline_unix_ms);
        assert!(completion_at < fixture.lease.absolute_expires_at_unix_ms);
        clock.advance_to(completion_at);
        release_tx.send(()).unwrap();
        (worker.join().unwrap(), server.join().unwrap())
    });
    let output = result.unwrap();
    assert_eq!(packet["source_content"], fixture.source_content);
    assert_ne!(
        output.source.captured_snapshot_sha256,
        output.source.validated_snapshot_sha256
    );
    assert_eq!(output.source.entry_identity, before_entry.identity);
    assert_eq!(
        output.source.content_sha256,
        fixture.source.expected_content_sha256
    );
    assert!(!response.is_empty());
    fixture.now = clock.0.load(Ordering::SeqCst);
    let after_budget = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(after_budget.reserved_steps, 1);
    assert_eq!(
        after_budget.deadline_unix_ms,
        before_budget.deadline_unix_ms
    );
    assert!(after_budget.active_attempt.is_none());
    assert!(!after_budget.execution_may_continue);
    let capacity_bytes = fixture
        .engine
        .storage
        .control()
        .get(&format!(
            "server/state/{}/endpoint-proposal/capacity",
            fixture.engine.instance
        ))
        .unwrap()
        .unwrap();
    let capacity_value: serde_json::Value = serde_json::from_slice(&capacity_bytes).unwrap();
    assert!(capacity_value.get("assessments").is_none());
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn lost_assessment_response_retains_uncertain_attempt_capacity_and_budget_across_reopen() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "lost-response", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);
    let assessment_at = fixture.now + 1;
    assert!(assessment_at < before.deadline_unix_ms);
    assert!(assessment_at < fixture.lease.absolute_expires_at_unix_ms);

    let (result, packet) = std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (_socket, packet, _model) = receive_assessment(&listener);
            packet
        });
        let result = fixture.engine.execute_task_assessment_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request,
            "assessment-lost-response",
            "assessment-lost-response",
            &crate::engine::clock::FixedClock::at(assessment_at),
        );
        (result, server.join().unwrap())
    });
    assert_eq!(packet["source_content"], fixture.source_content);
    assert!(matches!(
        result,
        Err(ServiceError::EndpointCallFailed {
            execution_may_continue: true,
            ..
        })
    ));
    let after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(after.reserved_steps, before.reserved_steps + 1);
    assert_eq!(after.deadline_unix_ms, before.deadline_unix_ms);
    assert!(after.active_attempt.is_none());
    assert!(after.execution_may_continue);
    assert_eq!(
        after.stopped,
        Some(rrd_contract::ReasoningBudgetStop::AttemptFailed)
    );
    let read_attempt = rrd_contract::ReadTaskAttempt {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
    };
    let attempt = fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_attempt,
            fixture.now + 3,
            "assessment-lost-attempt-read",
            "assessment-lost-attempt-read",
        )
        .unwrap()
        .attempt
        .unwrap();
    assert_eq!(attempt.status, rrd_contract::TaskAttemptStatus::Stopped);
    assert!(attempt.outcome_sha256.is_none());
    assert!(attempt.worker_output.is_none() && attempt.endpoint_proposal.is_none());
    assert!(fixture
        .engine
        .read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAssessment {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 4,
            "assessment-lost-output-read",
            "assessment-lost-output-read",
        )
        .unwrap()
        .assessment
        .is_none());
    let capacity_key = format!(
        "server/state/{}/endpoint-proposal/capacity",
        fixture.engine.instance
    );
    let capacity = fixture
        .engine
        .storage
        .control()
        .get(&capacity_key)
        .unwrap()
        .unwrap();
    let capacity_value: serde_json::Value = serde_json::from_slice(&capacity).unwrap();
    assert_eq!(capacity_value["assessments"].as_object().unwrap().len(), 1);

    let reopen_at = host_unix_ms();
    assert!(reopen_at < fixture.lease.absolute_expires_at_unix_ms);
    let SourceWorkerFixture {
        installed,
        engine,
        lease,
        job,
        principal_id,
        worker_token,
        selection,
        source,
        source_path,
        source_content,
        now: _,
    } = fixture;
    drop(engine);
    let reopened = installed.reopen(reopen_at);
    let fixture = SourceWorkerFixture {
        installed,
        engine: reopened,
        lease,
        job,
        principal_id,
        worker_token,
        selection,
        source,
        source_path,
        source_content,
        now: reopen_at,
    };
    assert_eq!(inspect_budget(&fixture, reopen_at + 1), after);
    assert_eq!(
        fixture
            .engine
            .read_task_attempt(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &read_attempt,
                reopen_at + 2,
                "assessment-lost-attempt-reopened",
                "assessment-lost-attempt-reopened",
            )
            .unwrap()
            .attempt,
        Some(attempt)
    );
    assert!(fixture
        .engine
        .read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAssessment {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            reopen_at + 3,
            "assessment-lost-output-reopened",
            "assessment-lost-output-reopened",
        )
        .unwrap()
        .assessment
        .is_none());
    assert_eq!(
        fixture.engine.storage.control().get(&capacity_key).unwrap(),
        Some(capacity.clone())
    );
    assert!(matches!(
        fixture.engine.execute_task_assessment_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request,
            "assessment-lost-response-no-replay",
            "assessment-lost-response-no-replay",
            &crate::engine::clock::FixedClock::at(reopen_at + 4),
        ),
        Err(ServiceError::ReasoningAttemptRecorded {
            outcome_sha256: None
        })
    ));
    assert_eq!(
        fixture.engine.storage.control().get(&capacity_key).unwrap(),
        Some(capacity)
    );
    assert!(matches!(
        fixture.engine.resume_task_budget_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ResumeTaskBudget {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                expected_budget_sha256: operation_digest(&after).unwrap(),
            },
            "assessment-lost-budget-resume",
            "assessment-lost-budget-resume",
            &crate::engine::clock::FixedClock::at(reopen_at + 4),
        ),
        Err(ServiceError::ReasoningBudgetStopped(
            rrd_contract::ReasoningBudgetStop::UnresolvedAttempt
        ))
    ));
    assert_eq!(inspect_budget(&fixture, reopen_at + 5), after);
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn engine_executes_choice_score_and_evidence_limited_abstention() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);

    let choice_request = assessment_request(
        &fixture,
        "engine-choice",
        TaskAssessmentDescriptor::Choice {
            question: "Which declared mode matches the supplied evidence?".into(),
            options: vec![
                rrd_contract::AssessmentOption {
                    id: CanonicalId::new("supported").unwrap(),
                    description: "Directly supported".into(),
                },
                rrd_contract::AssessmentOption {
                    id: CanonicalId::new("uncertain").unwrap(),
                    description: "Insufficient evidence".into(),
                },
            ],
        },
    );
    let (choice_output, choice_packet) = run_assessment(
        &fixture,
        &listener,
        &choice_request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "choice", "option_id": "supported" }
        }),
        Some("source"),
    );
    assert_eq!(choice_packet["source_content"], fixture.source_content);
    assert!(matches!(
        &choice_output.decision,
        TaskAssessmentDecision::Answered {
            value: TaskAssessmentValue::Choice { option_id }
        } if option_id.as_str() == "supported"
    ));

    let score_request = assessment_request(
        &fixture,
        "engine-score",
        TaskAssessmentDescriptor::Score {
            rubric: "Use a bounded score for the supplied rubric".into(),
            minimum: 0.0,
            maximum: 5.0,
            direction: AssessmentScoreDirection::HigherIsBetter,
        },
    );
    let (score_output, score_packet) = run_assessment(
        &fixture,
        &listener,
        &score_request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "score", "value": 3.0 }
        }),
        Some("practice"),
    );
    assert_eq!(
        score_packet["practice_sha256"],
        score_output.practice_sha256
    );
    assert!(matches!(
        score_output.decision,
        TaskAssessmentDecision::Answered {
            value: TaskAssessmentValue::Score { value: 3.0 }
        }
    ));
    assert_eq!(
        score_output.references[0].input,
        AssessmentInputKind::Practice
    );

    let abstention_request = assessment_request(&fixture, "engine-abstention", binary_descriptor());
    let (abstention_output, _) = run_assessment(
        &fixture,
        &listener,
        &abstention_request,
        serde_json::json!({
            "outcome": "abstained",
            "reason": "The required evidence is absent from the selected bytes"
        }),
        None,
    );
    assert!(matches!(
        abstention_output.decision,
        TaskAssessmentDecision::Abstained { .. }
    ));
    assert!(abstention_output.references.is_empty());
}

#[test]
fn disabled_assessment_flags_deny_before_budget_reservation_or_provider_dispatch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    for (suffix, network, source, context) in [
        ("network", false, true, true),
        ("source", true, false, true),
        ("context", true, true, false),
    ] {
        publish_assessment_policy(&mut fixture, network, source, context);
        let request = assessment_request(&fixture, suffix, binary_descriptor());
        let before = inspect_budget(&fixture, fixture.now + 1);
        assert!(matches!(
            fixture.engine.execute_task_assessment_with_clock(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &request,
                &format!("assessment-denied-{suffix}"),
                &format!("assessment-denied-{suffix}"),
                &crate::engine::clock::FixedClock::at(fixture.now + 1),
            ),
            Err(ServiceError::PermissionDenied)
        ));
        let after = inspect_budget(&fixture, fixture.now + 2);
        assert_eq!(after, before);
        listener.set_nonblocking(true).unwrap();
        assert!(matches!(
            listener.accept(),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
        ));
        listener.set_nonblocking(false).unwrap();
    }
}

#[test]
fn configuration_change_before_assessment_dispatch_is_rejected_without_budget_or_provider_use() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "stale-configuration", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);

    let changed_configuration = publish_assessment_policy_with_prompt(
        &mut fixture,
        true,
        true,
        true,
        "Use the currently approved typed-assessment rubric.",
    );
    assert_ne!(
        changed_configuration.sha256().unwrap(),
        request.expected_configuration_sha256
    );
    assert!(matches!(
        fixture.engine.execute_task_assessment_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request,
            "assessment-stale-configuration",
            "assessment-stale-configuration",
            &crate::engine::clock::FixedClock::at(fixture.now + 1),
        ),
        Err(ServiceError::TaskAttemptInvalidated(
            TaskAttemptInvalidation::InputsChanged
        ))
    ));
    let after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(after, before);
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn closed_task_before_assessment_dispatch_is_rejected_without_budget_or_provider_use() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "closed-task", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);

    fixture.now += 2;
    fixture
        .engine
        .close_task(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::CloseTask {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                status: rrd_contract::TaskStatus::Abandoned,
                outcome: None,
            },
            fixture.now,
            "assessment-close-task-before-dispatch",
            "assessment-close-task-before-dispatch",
        )
        .unwrap();
    assert!(matches!(
        fixture.engine.execute_task_assessment_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request,
            "assessment-closed-task",
            "assessment-closed-task",
            &crate::engine::clock::FixedClock::at(fixture.now + 3),
        ),
        Err(ServiceError::Contract(_))
    ));
    let after = inspect_budget(&fixture, fixture.now + 4);
    assert_eq!(after, before);
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn selected_source_change_during_assessment_dispatch_withholds_output_without_retry() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "source-drift", binary_descriptor());
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (result, (packet, response)) = std::thread::scope(|scope| {
        let expected_source = fixture.source_content.clone();
        let listener_ref = &listener;
        let server = scope.spawn(move || {
            let (mut socket, packet, model) = receive_assessment(listener_ref);
            assert_eq!(packet["source_content"], expected_source);
            entered_tx.send(()).unwrap();
            release_rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            let response = reply(&mut socket, &model, &assessment_reply(&packet, true));
            (packet, response)
        });
        let worker = scope.spawn(|| {
            fixture.engine.execute_task_assessment_with_clock(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &request,
                "assessment-source-drift-during",
                "assessment-source-drift-during",
                &crate::engine::clock::FixedClock::at(fixture.now + 1),
            )
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        std::fs::write(
            fixture.installed.root.path().join(&fixture.source_path),
            "changed while the single controlled request was pending\n",
        )
        .unwrap();
        release_tx.send(()).unwrap();
        (worker.join().unwrap(), server.join().unwrap())
    });
    assert!(matches!(
        result,
        Err(ServiceError::TaskAttemptInvalidated(
            TaskAttemptInvalidation::InputsChanged
        ))
    ));
    assert_eq!(packet["source_content"], fixture.source_content);
    assert!(!response.is_empty());
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
    let read = fixture
        .engine
        .read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAssessment {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 5,
            "assessment-source-drift-read",
            "assessment-source-drift-read",
        )
        .unwrap();
    assert!(read.assessment.is_none());
}

#[test]
fn current_task_close_during_assessment_withholds_output_without_retry() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "task-close-during", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);
    let start_at = fixture.now + 1;
    assert!(start_at > before.started_at_unix_ms);
    assert!(start_at < before.deadline_unix_ms);
    assert!(start_at < fixture.lease.absolute_expires_at_unix_ms);
    let clock = AssessmentClock::at(start_at);
    let engine = &fixture.engine;
    let session = fixture.lease.session_id.clone();
    let token = fixture.lease.token.clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (result, (packet, response)) = std::thread::scope(|scope| {
        let expected_content = fixture.source_content.clone();
        let listener_ref = &listener;
        let server = scope.spawn(move || {
            let (mut socket, packet, model) = receive_assessment(listener_ref);
            assert_eq!(packet["source_content"], expected_content);
            entered_tx.send(()).unwrap();
            release_rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            let response = reply(&mut socket, &model, &assessment_reply(&packet, true));
            (packet, response)
        });
        let clock_ref = &clock;
        let worker = scope.spawn(|| {
            engine.execute_task_assessment_with_clock(
                &session,
                &token,
                &request,
                "assessment-task-close-during",
                "assessment-task-close-during",
                clock_ref,
            )
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let close_at = host_unix_ms().max(start_at + 1);
        assert!(close_at < before.deadline_unix_ms);
        assert!(close_at < fixture.lease.absolute_expires_at_unix_ms);
        engine
            .close_task(
                &session,
                &token,
                &rrd_contract::CloseTask {
                    scope: request.scope.clone(),
                    task_id: request.task_id.clone(),
                    status: rrd_contract::TaskStatus::Abandoned,
                    outcome: None,
                },
                close_at,
                "assessment-task-close-during-state-change",
                "assessment-task-close-during-state-change",
            )
            .unwrap();
        clock.advance_to(close_at + 1);
        release_tx.send(()).unwrap();
        (worker.join().unwrap(), server.join().unwrap())
    });
    assert!(matches!(
        result,
        Err(ServiceError::TaskAttemptInvalidated(
            TaskAttemptInvalidation::InputsChanged
        ))
    ));
    assert_eq!(packet["source_content"], fixture.source_content);
    assert!(!response.is_empty());
    let after = inspect_budget(&fixture, clock.0.load(Ordering::SeqCst) + 1);
    assert_eq!(after.reserved_steps, 1);
    assert_eq!(after.deadline_unix_ms, before.deadline_unix_ms);
    assert!(after.active_attempt.is_none());
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
    assert!(fixture
        .engine
        .read_task_assessment(
            &session,
            &token,
            &ReadTaskAssessment {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            clock.0.load(Ordering::SeqCst) + 2,
            "assessment-task-close-during-read",
            "assessment-task-close-during-read",
        )
        .unwrap()
        .assessment
        .is_none());
}

#[test]
fn current_configuration_change_during_assessment_withholds_output_without_retry() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "configuration-change-during", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);
    let start_at = fixture.now + 1;
    assert!(start_at > before.started_at_unix_ms);
    assert!(start_at < before.deadline_unix_ms);
    assert!(start_at < fixture.lease.absolute_expires_at_unix_ms);
    let clock = AssessmentClock::at(start_at);
    let engine = &fixture.engine;
    let session = fixture.lease.session_id.clone();
    let token = fixture.lease.token.clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (result, (packet, response)) = std::thread::scope(|scope| {
        let expected_content = fixture.source_content.clone();
        let listener_ref = &listener;
        let server = scope.spawn(move || {
            let (mut socket, packet, model) = receive_assessment(listener_ref);
            assert_eq!(packet["source_content"], expected_content);
            entered_tx.send(()).unwrap();
            release_rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            let response = reply(&mut socket, &model, &assessment_reply(&packet, true));
            (packet, response)
        });
        let clock_ref = &clock;
        let worker = scope.spawn(|| {
            engine.execute_task_assessment_with_clock(
                &session,
                &token,
                &request,
                "assessment-configuration-change-during",
                "assessment-configuration-change-during",
                clock_ref,
            )
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let changed = publish_assessment_policy_with_prompt_for(
            engine,
            &fixture.lease,
            &mut fixture.now,
            &mut fixture.selection,
            true,
            true,
            true,
            "Use the currently approved alternate advisory rubric.",
        );
        assert_ne!(
            changed.sha256().unwrap(),
            request.expected_configuration_sha256
        );
        let completion_at = host_unix_ms().max(fixture.now + 1);
        assert!(completion_at < before.deadline_unix_ms);
        assert!(completion_at < fixture.lease.absolute_expires_at_unix_ms);
        clock.advance_to(completion_at);
        release_tx.send(()).unwrap();
        (worker.join().unwrap(), server.join().unwrap())
    });
    assert!(matches!(
        result,
        Err(ServiceError::TaskAttemptInvalidated(
            TaskAttemptInvalidation::InputsChanged
        ))
    ));
    assert_eq!(packet["source_content"], fixture.source_content);
    assert!(!response.is_empty());
    let after = inspect_budget(&fixture, clock.0.load(Ordering::SeqCst) + 1);
    assert_eq!(after.reserved_steps, 1);
    assert_eq!(after.deadline_unix_ms, before.deadline_unix_ms);
    assert!(after.active_attempt.is_none());
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
    assert!(fixture
        .engine
        .read_task_assessment(
            &session,
            &token,
            &ReadTaskAssessment {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            clock.0.load(Ordering::SeqCst) + 2,
            "assessment-configuration-change-during-read",
            "assessment-configuration-change-during-read",
        )
        .unwrap()
        .assessment
        .is_none());
}

#[test]
fn public_budget_stop_during_pending_assessment_withholds_output_and_retains_capacity() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_assessment_fixture(&listener);
    configure_assessment_budget(&mut fixture);
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "explicit-stop", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);
    let start_at = fixture.now + 1;
    assert!(start_at > before.started_at_unix_ms);
    assert!(start_at < before.deadline_unix_ms);
    assert!(start_at < fixture.lease.absolute_expires_at_unix_ms);
    let clock = AssessmentClock::at(start_at);
    let engine = &fixture.engine;
    let session = fixture.lease.session_id.clone();
    let token = fixture.lease.token.clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (result, (packet, response)) = std::thread::scope(|scope| {
        let expected_content = fixture.source_content.clone();
        let listener_ref = &listener;
        let server = scope.spawn(move || {
            let (mut socket, packet, model) = receive_assessment(listener_ref);
            assert_eq!(packet["source_content"], expected_content);
            entered_tx.send(()).unwrap();
            release_rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            let response = reply(&mut socket, &model, &assessment_reply(&packet, true));
            (packet, response)
        });
        let clock_ref = &clock;
        let worker = scope.spawn(|| {
            engine.execute_task_assessment_with_clock(
                &session,
                &token,
                &request,
                "assessment-explicit-stop",
                "assessment-explicit-stop",
                clock_ref,
            )
        });
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let pending = inspect_budget(&fixture, start_at + 1);
        assert_eq!(pending.reserved_steps, 1);
        assert!(pending.active_attempt.is_some());
        let stop_at = host_unix_ms().max(start_at + 1);
        assert!(stop_at < before.deadline_unix_ms);
        assert!(stop_at < fixture.lease.absolute_expires_at_unix_ms);
        engine
            .stop_task_budget(
                &session,
                &token,
                &rrd_contract::StopTaskBudget {
                    scope: request.scope.clone(),
                    task_id: request.task_id.clone(),
                    expected_budget_sha256: operation_digest(&pending).unwrap(),
                },
                stop_at,
                "assessment-explicit-stop-budget",
                "assessment-explicit-stop-budget",
            )
            .unwrap();
        clock.advance_to(stop_at + 1);
        release_tx.send(()).unwrap();
        (worker.join().unwrap(), server.join().unwrap())
    });
    assert!(
        matches!(
            &result,
            Err(ServiceError::EndpointCallFailed {
                reason: "chat cancelled",
                execution_may_continue: true,
            })
        ),
        "unexpected explicit-stop result: {result:?}"
    );
    assert_eq!(packet["source_content"], fixture.source_content);
    assert!(!response.is_empty());
    let after = inspect_budget(&fixture, clock.0.load(Ordering::SeqCst) + 1);
    assert_eq!(after.reserved_steps, 1);
    assert_eq!(after.deadline_unix_ms, before.deadline_unix_ms);
    assert_eq!(
        after.stopped,
        Some(rrd_contract::ReasoningBudgetStop::Cancelled)
    );
    assert!(after.active_attempt.is_none());
    assert!(after.execution_may_continue);
    let attempt = fixture
        .engine
        .read_task_attempt(
            &session,
            &token,
            &rrd_contract::ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            clock.0.load(Ordering::SeqCst) + 2,
            "assessment-explicit-stop-read-attempt",
            "assessment-explicit-stop-read-attempt",
        )
        .unwrap()
        .attempt
        .unwrap();
    assert_eq!(attempt.status, rrd_contract::TaskAttemptStatus::Stopped);
    assert!(attempt.outcome_sha256.is_none());
    assert!(attempt.worker_output.is_none() && attempt.endpoint_proposal.is_none());
    assert!(fixture
        .engine
        .read_task_assessment(
            &session,
            &token,
            &ReadTaskAssessment {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            clock.0.load(Ordering::SeqCst) + 3,
            "assessment-explicit-stop-read-output",
            "assessment-explicit-stop-read-output",
        )
        .unwrap()
        .assessment
        .is_none());
    let capacity_bytes = fixture
        .engine
        .storage
        .control()
        .get(&format!(
            "server/state/{}/endpoint-proposal/capacity",
            fixture.engine.instance
        ))
        .unwrap()
        .unwrap();
    let capacity: serde_json::Value = serde_json::from_slice(&capacity_bytes).unwrap();
    assert_eq!(capacity["assessments"].as_object().unwrap().len(), 1);
    assert!(matches!(
        fixture.engine.resume_task_budget_with_clock(
            &session,
            &token,
            &rrd_contract::ResumeTaskBudget {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                expected_budget_sha256: operation_digest(&after).unwrap(),
            },
            "assessment-explicit-stop-budget-resume",
            "assessment-explicit-stop-budget-resume",
            &crate::engine::clock::FixedClock::at(clock.0.load(Ordering::SeqCst) + 4),
        ),
        Err(ServiceError::ReasoningBudgetStopped(
            rrd_contract::ReasoningBudgetStop::UnresolvedAttempt
        ))
    ));
    assert!(matches!(
        fixture.engine.execute_task_assessment_with_clock(
            &session,
            &token,
            &request,
            "assessment-explicit-stop-no-replay",
            "assessment-explicit-stop-no-replay",
            &crate::engine::clock::FixedClock::at(clock.0.load(Ordering::SeqCst) + 5),
        ),
        Err(ServiceError::ReasoningAttemptRecorded {
            outcome_sha256: None
        })
    ));
    assert_eq!(
        fixture
            .engine
            .storage
            .control()
            .get(&format!(
                "server/state/{}/endpoint-proposal/capacity",
                fixture.engine.instance
            ))
            .unwrap(),
        Some(capacity_bytes)
    );
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn repeated_assessment_is_blocked_after_unrelated_git_policy_and_head_advance() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = host_unix_ms().saturating_sub(30_000);
    let mut fixture =
        source_worker_fixture_for_path_at(&listener, true, true, "nested/alpha.txt", base);
    configure_assessment_budget_with_repetition(&mut fixture, Some(1));
    publish_assessment_policy(&mut fixture, true, true, true);
    let request = assessment_request(&fixture, "repetition-first", binary_descriptor());
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before.reserved_steps, 0);

    let (first_output, first_packet) = run_assessment(
        &fixture,
        &listener,
        &request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "binary", "value": true }
        }),
        Some("source"),
    );
    assert_eq!(
        first_output.decision,
        TaskAssessmentDecision::Answered {
            value: TaskAssessmentValue::Binary { value: true }
        }
    );
    assert_eq!(
        first_packet["source"]["entry_identity"],
        fixture.source.entry_identity
    );
    let after_first = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(after_first.reserved_steps, 1);
    assert_eq!(after_first.stopped, None);

    let project_id = fixture
        .installed
        .preview
        .installation
        .target
        .project_id
        .clone();
    let before_head = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let before_entry = before_head
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap()
        .clone();
    std::fs::write(
        fixture.installed.root.path().join(".git/index"),
        include_bytes!(
            "../../../../../../../../compute/rrd-attunement/tests/fixtures/git-index/sha1-v3.index"
        ),
    )
    .unwrap();
    std::fs::write(
        fixture.installed.root.path().join(".gitignore"),
        b"*.tmp\nignored/\n",
    )
    .unwrap();
    std::fs::write(
        fixture.installed.root.path().join("nested/.gitignore"),
        b"*.bak\n",
    )
    .unwrap();
    let host_floor = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
    let changed_head = refresh_unrelated_with_host_floor(
        fixture.installed.root.path(),
        &fixture.engine,
        &fixture.lease,
        &project_id,
        &mut fixture.job,
        "assessment-unrelated-beta.txt",
        Some(host_floor),
    );
    fixture.now = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
    let after_head = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let after_entry = after_head
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap();
    assert_ne!(changed_head, before_head.proposal_sha256);
    assert_ne!(
        before_head.proposal.revisions().policy_sha256,
        after_head.proposal.revisions().policy_sha256
    );
    assert_eq!(after_head.proposal_sha256, changed_head);
    assert_eq!(after_entry.identity, before_entry.identity);
    assert_eq!(after_entry.kind, before_entry.kind);

    let mut repeated = request.clone();
    repeated.attempt_id = id("assessment-repetition-distinct-attempt");
    let before_repeat = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(before_repeat, after_first);
    assert!(matches!(
        fixture.engine.execute_task_assessment_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &repeated,
            "assessment-repetition-distinct-attempt",
            "assessment-repetition-distinct-attempt",
            &crate::engine::clock::FixedClock::at(fixture.now + 1),
        ),
        Err(ServiceError::ReasoningBudgetStopped(
            rrd_contract::ReasoningBudgetStop::RepeatedWork
        ))
    ));
    let after_repeat = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(after_repeat.reserved_steps, before_repeat.reserved_steps);
    assert_eq!(
        after_repeat.charged_elapsed_ms,
        before_repeat.charged_elapsed_ms
    );
    assert_eq!(
        after_repeat.stopped,
        Some(rrd_contract::ReasoningBudgetStop::RepeatedWork)
    );
    assert!(first_output.validate().is_ok());
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn changed_selected_source_gets_new_assessment_identity_without_budget_reset() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = host_unix_ms().saturating_sub(30_000);
    let mut fixture =
        source_worker_fixture_for_path_at(&listener, true, true, "nested/alpha.txt", base);
    configure_assessment_budget_with_repetition(&mut fixture, Some(1));
    publish_assessment_policy(&mut fixture, true, true, true);
    let first_request = assessment_request(&fixture, "changed-source-first", binary_descriptor());
    let first_before = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(first_before.reserved_steps, 0);
    let (first_output, first_packet) = run_assessment(
        &fixture,
        &listener,
        &first_request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "binary", "value": true }
        }),
        Some("source"),
    );
    assert_eq!(
        first_packet["source"]["content_sha256"],
        fixture.source.expected_content_sha256
    );
    let first_after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(first_after.reserved_steps, 1);
    assert_eq!(first_after.stopped, None);

    let changed_content = "selected source changed to a distinct admitted input\n";
    std::fs::write(
        fixture.installed.root.path().join(&fixture.source_path),
        changed_content,
    )
    .unwrap();
    let refresh_at = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
    refresh_inventory_without_source_edits(
        &fixture.engine,
        &fixture.lease,
        &mut fixture.job,
        refresh_at,
    );
    fixture.now = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
    let project_id = fixture
        .installed
        .preview
        .installation
        .target
        .project_id
        .clone();
    let current = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let selected = current
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap();
    let content_sha256 = match &selected.kind {
        rrd_attunement::inventory::EntryKind::File {
            observation,
            content_sha256,
        } => {
            assert_eq!(observation.bytes, changed_content.len() as u64);
            assert_eq!(
                content_sha256,
                &digest::sha256_hex(changed_content.as_bytes())
            );
            content_sha256.clone()
        }
        other => panic!("selected source is not an admitted file: {other:?}"),
    };
    let second_source = TaskWorkerSourceRequest {
        contract_version: rrd_contract::TASK_WORKER_SOURCE_VERSION,
        entry_identity: selected.identity.clone(),
        expected_content_sha256: content_sha256,
        maximum_input_bytes: fixture.source.maximum_input_bytes,
    };
    fixture.source_content = changed_content.to_owned();

    let historical = fixture
        .engine
        .read_task_assessment(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAssessment {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: first_request.scope.clone(),
                task_id: first_request.task_id.clone(),
                attempt_id: first_request.attempt_id.clone(),
            },
            fixture.now + 1,
            "assessment-read-old-authorized-output-after-source-edit",
            "assessment-read-old-authorized-output-after-source-edit",
        )
        .unwrap();
    assert_eq!(historical.assessment, Some(first_output.clone()));

    let mut second_request =
        assessment_request(&fixture, "changed-source-second", binary_descriptor());
    second_request.source = second_source;
    let before_second = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(before_second.reserved_steps, 1);
    assert_eq!(before_second.stopped, None);
    assert_eq!(before_second.deadline_unix_ms, first_after.deadline_unix_ms);
    let (second_output, second_packet) = run_assessment(
        &fixture,
        &listener,
        &second_request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "binary", "value": false }
        }),
        Some("source"),
    );
    assert_ne!(
        first_output.source.content_sha256,
        second_output.source.content_sha256
    );
    assert_eq!(
        second_output.source.content_sha256,
        digest::sha256_hex(changed_content.as_bytes())
    );
    assert_eq!(second_packet["source_content"], changed_content);
    let after_second = inspect_budget(&fixture, fixture.now + 3);
    assert_eq!(after_second.reserved_steps, 2);
    assert_eq!(after_second.stopped, None);
    assert_eq!(after_second.deadline_unix_ms, first_after.deadline_unix_ms);
    assert!(after_second.charged_elapsed_ms >= first_after.charged_elapsed_ms);
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn repeated_work_resume_preserves_ceiling_and_admits_changed_source() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = host_unix_ms().saturating_sub(30_000);
    let mut fixture =
        source_worker_fixture_for_path_at(&listener, true, true, "nested/alpha.txt", base);
    configure_assessment_budget_with_repetition(&mut fixture, Some(1));
    publish_assessment_policy(&mut fixture, true, true, true);

    let first_request =
        assessment_request(&fixture, "resume-after-repeat-first", binary_descriptor());
    let (first_output, _) = run_assessment(
        &fixture,
        &listener,
        &first_request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "binary", "value": true }
        }),
        Some("source"),
    );
    assert!(first_output.validate().is_ok());
    let after_first = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(after_first.reserved_steps, 1);
    assert_eq!(after_first.stopped, None);

    let mut repeated_request = first_request.clone();
    repeated_request.attempt_id = id("assessment-resume-after-repeat-second");
    assert!(matches!(
        fixture.engine.execute_task_assessment_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &repeated_request,
            "assessment-resume-after-repeat-second",
            "assessment-resume-after-repeat-second",
            &crate::engine::clock::FixedClock::at(fixture.now + 1),
        ),
        Err(ServiceError::ReasoningBudgetStopped(
            rrd_contract::ReasoningBudgetStop::RepeatedWork
        ))
    ));
    let stopped_budget = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(stopped_budget.reserved_steps, after_first.reserved_steps);
    assert_eq!(
        stopped_budget.deadline_unix_ms,
        after_first.deadline_unix_ms
    );
    assert_eq!(
        stopped_budget.stopped,
        Some(rrd_contract::ReasoningBudgetStop::RepeatedWork)
    );

    let first_resume = fixture
        .engine
        .resume_task_budget_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ResumeTaskBudget {
                scope: first_request.scope.clone(),
                task_id: first_request.task_id.clone(),
                expected_budget_sha256: operation_digest(&stopped_budget).unwrap(),
            },
            "assessment-resume-after-repeat-first-resume",
            "assessment-resume-after-repeat-first-resume",
            &crate::engine::clock::FixedClock::at(fixture.now + 3),
        )
        .unwrap()
        .budget
        .unwrap();
    assert_eq!(first_resume.stopped, None);
    assert_eq!(first_resume.reserved_steps, stopped_budget.reserved_steps);
    assert_eq!(
        first_resume.deadline_unix_ms,
        stopped_budget.deadline_unix_ms
    );
    assert_eq!(first_resume.active_attempt, None);
    assert!(!first_resume.execution_may_continue);
    assert_eq!(first_resume.stop_expected_sha256, None);
    assert_eq!(
        first_resume.charged_elapsed_ms,
        stopped_budget.charged_elapsed_ms
    );
    assert_eq!(
        first_resume.equivalent_attempts,
        stopped_budget.equivalent_attempts
    );

    let mut repeated_again = first_request.clone();
    repeated_again.attempt_id = id("assessment-resume-after-repeat-third");
    assert!(matches!(
        fixture.engine.execute_task_assessment_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &repeated_again,
            "assessment-resume-after-repeat-third",
            "assessment-resume-after-repeat-third",
            &crate::engine::clock::FixedClock::at(fixture.now + 4),
        ),
        Err(ServiceError::ReasoningBudgetStopped(
            rrd_contract::ReasoningBudgetStop::RepeatedWork
        ))
    ));
    let repeated_again_budget = inspect_budget(&fixture, fixture.now + 5);
    assert_eq!(
        repeated_again_budget.reserved_steps,
        first_resume.reserved_steps
    );
    assert_eq!(
        repeated_again_budget.deadline_unix_ms,
        first_resume.deadline_unix_ms
    );
    assert_eq!(
        repeated_again_budget.charged_elapsed_ms,
        first_resume.charged_elapsed_ms
    );
    assert_eq!(
        repeated_again_budget.equivalent_attempts,
        first_resume.equivalent_attempts
    );
    assert_eq!(
        repeated_again_budget.stopped,
        Some(rrd_contract::ReasoningBudgetStop::RepeatedWork)
    );
    assert!(fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAttempt {
                scope: first_request.scope.clone(),
                task_id: first_request.task_id.clone(),
                attempt_id: repeated_again.attempt_id.clone(),
            },
            fixture.now + 5,
            "assessment-repeated-work-rejection-has-no-attempt",
            "assessment-repeated-work-rejection-has-no-attempt",
        )
        .unwrap()
        .attempt
        .is_none());

    let second_resume = fixture
        .engine
        .resume_task_budget_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ResumeTaskBudget {
                scope: first_request.scope.clone(),
                task_id: first_request.task_id.clone(),
                expected_budget_sha256: operation_digest(&repeated_again_budget).unwrap(),
            },
            "assessment-resume-after-repeat-second-resume",
            "assessment-resume-after-repeat-second-resume",
            &crate::engine::clock::FixedClock::at(fixture.now + 6),
        )
        .unwrap()
        .budget
        .unwrap();
    assert_eq!(second_resume.stopped, None);
    assert_eq!(
        second_resume.reserved_steps,
        repeated_again_budget.reserved_steps
    );
    assert_eq!(
        second_resume.deadline_unix_ms,
        repeated_again_budget.deadline_unix_ms
    );
    assert_eq!(second_resume.active_attempt, None);
    assert!(!second_resume.execution_may_continue);
    assert_eq!(second_resume.stop_expected_sha256, None);
    assert_eq!(
        second_resume.charged_elapsed_ms,
        repeated_again_budget.charged_elapsed_ms
    );
    assert_eq!(
        second_resume.equivalent_attempts,
        repeated_again_budget.equivalent_attempts
    );

    let changed_content = "selected source changed after repeated work was stopped\n";
    std::fs::write(
        fixture.installed.root.path().join(&fixture.source_path),
        changed_content,
    )
    .unwrap();
    let refresh_at = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
    refresh_inventory_without_source_edits(
        &fixture.engine,
        &fixture.lease,
        &mut fixture.job,
        refresh_at,
    );
    fixture.now = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);

    let project_id = fixture
        .installed
        .preview
        .installation
        .target
        .project_id
        .clone();
    let accepted = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let selected = accepted
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap();
    assert_eq!(selected.identity, fixture.source.entry_identity);
    let current_content_sha256 = match &selected.kind {
        rrd_attunement::inventory::EntryKind::File {
            observation,
            content_sha256,
        } => {
            assert_eq!(observation.bytes, changed_content.len() as u64);
            assert_eq!(
                content_sha256,
                &digest::sha256_hex(changed_content.as_bytes())
            );
            content_sha256.clone()
        }
        other => panic!("selected source is not an admitted file: {other:?}"),
    };
    fixture.source_content = changed_content.to_owned();

    let before_changed_assessment = inspect_budget(&fixture, fixture.now + 1);
    assert_eq!(
        before_changed_assessment.reserved_steps,
        first_resume.reserved_steps
    );
    assert_eq!(before_changed_assessment.stopped, None);
    assert_eq!(
        before_changed_assessment.deadline_unix_ms,
        first_resume.deadline_unix_ms
    );
    assert_eq!(
        before_changed_assessment.equivalent_attempts,
        first_resume.equivalent_attempts
    );
    let mut changed_request = assessment_request(
        &fixture,
        "resume-after-repeat-changed-source",
        binary_descriptor(),
    );
    changed_request.source = TaskWorkerSourceRequest {
        contract_version: rrd_contract::TASK_WORKER_SOURCE_VERSION,
        entry_identity: selected.identity.clone(),
        expected_content_sha256: current_content_sha256,
        maximum_input_bytes: fixture.source.maximum_input_bytes,
    };
    let (changed_output, changed_packet) = run_assessment(
        &fixture,
        &listener,
        &changed_request,
        serde_json::json!({
            "outcome": "answered",
            "value": { "kind": "binary", "value": false }
        }),
        Some("source"),
    );
    assert_eq!(changed_packet["source_content"], changed_content);
    assert_eq!(
        changed_output.source.content_sha256,
        digest::sha256_hex(changed_content.as_bytes())
    );
    let completed_budget = inspect_budget(&fixture, fixture.now + 3);
    assert_eq!(
        completed_budget.reserved_steps,
        first_resume.reserved_steps + 1
    );
    assert_eq!(
        completed_budget.deadline_unix_ms,
        first_resume.deadline_unix_ms
    );
    assert_eq!(completed_budget.stopped, None);
    assert!(completed_budget.charged_elapsed_ms >= first_resume.charged_elapsed_ms);
    assert_eq!(completed_budget.equivalent_attempts.len(), 2);
    assert_eq!(
        completed_budget.equivalent_attempts.values().sum::<u64>(),
        2
    );
    assert_eq!(
        completed_budget.deadline_unix_ms,
        before_changed_assessment.deadline_unix_ms
    );

    let reopen_at = host_unix_ms();
    assert!(reopen_at < fixture.lease.absolute_expires_at_unix_ms);
    assert!(reopen_at < completed_budget.deadline_unix_ms);
    let SourceWorkerFixture {
        installed,
        engine,
        lease,
        job,
        principal_id,
        worker_token,
        selection,
        source,
        source_path,
        source_content,
        now: _,
    } = fixture;
    drop(engine);
    let reopened = installed.reopen(reopen_at);
    let reopened_fixture = SourceWorkerFixture {
        installed,
        engine: reopened,
        lease,
        job,
        principal_id,
        worker_token,
        selection,
        source,
        source_path,
        source_content,
        now: reopen_at,
    };
    assert_eq!(
        inspect_budget(&reopened_fixture, reopen_at + 1),
        completed_budget
    );
    assert_eq!(
        reopened_fixture
            .engine
            .read_task_assessment(
                &reopened_fixture.lease.session_id,
                &reopened_fixture.lease.token,
                &ReadTaskAssessment {
                    contract_version: TASK_ASSESSMENT_VERSION,
                    scope: changed_request.scope.clone(),
                    task_id: changed_request.task_id.clone(),
                    attempt_id: changed_request.attempt_id.clone(),
                },
                reopen_at + 2,
                "assessment-repeated-work-recovery-read-after-reopen",
                "assessment-repeated-work-recovery-read-after-reopen",
            )
            .unwrap()
            .assessment,
        Some(changed_output)
    );
    assert_eq!(
        reopened_fixture
            .engine
            .read_task_assessment(
                &reopened_fixture.lease.session_id,
                &reopened_fixture.lease.token,
                &ReadTaskAssessment {
                    contract_version: TASK_ASSESSMENT_VERSION,
                    scope: first_request.scope.clone(),
                    task_id: first_request.task_id.clone(),
                    attempt_id: first_request.attempt_id.clone(),
                },
                reopen_at + 3,
                "assessment-repeated-work-first-output-read-after-reopen",
                "assessment-repeated-work-first-output-read-after-reopen",
            )
            .unwrap()
            .assessment,
        Some(first_output)
    );
    for rejected in [&repeated_request, &repeated_again] {
        assert!(reopened_fixture
            .engine
            .read_task_attempt(
                &reopened_fixture.lease.session_id,
                &reopened_fixture.lease.token,
                &ReadTaskAttempt {
                    scope: rejected.scope.clone(),
                    task_id: rejected.task_id.clone(),
                    attempt_id: rejected.attempt_id.clone(),
                },
                reopen_at + 4,
                "assessment-repeated-work-rejected-attempt-read-after-reopen",
                "assessment-repeated-work-rejected-attempt-read-after-reopen",
            )
            .unwrap()
            .attempt
            .is_none());
    }
    assert!(matches!(
        reopened_fixture.engine.execute_task_assessment_with_clock(
            &reopened_fixture.lease.session_id,
            &reopened_fixture.lease.token,
            &changed_request,
            "assessment-repeated-work-recovery-no-replay",
            "assessment-repeated-work-recovery-no-replay",
            &crate::engine::clock::FixedClock::at(reopen_at + 5),
        ),
        Err(ServiceError::ReasoningAttemptRecorded {
            outcome_sha256: Some(_)
        })
    ));
    assert_eq!(
        inspect_budget(&reopened_fixture, reopen_at + 6),
        completed_budget
    );
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}
