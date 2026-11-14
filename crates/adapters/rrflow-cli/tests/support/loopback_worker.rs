//! Loopback model endpoints, raw public HTTP sessions and the installed worker driver for lifecycle tests.

use super::*;

pub(super) fn unix_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .unwrap()
}

pub(super) fn loopback_worker_profile(
    listener: &TcpListener,
    allow_context_export: bool,
    allow_source_export: bool,
) -> rrd_contract::EndpointProfileInput {
    let mut configuration: rrd_contract::EndpointConfiguration =
        serde_json::from_slice(LOOPBACK_CONFIGURATION_BYTES).unwrap();
    let allowance: rrd_contract::EndpointSelectionAllowance =
        serde_json::from_slice(LOOPBACK_ALLOWANCE_BYTES).unwrap();
    let profile = configuration.profile.take().unwrap();
    let mut bindings = profile.bindings;
    let controller_service = bindings[0].service_id.clone();
    let controller_binding_id = bindings
        .iter()
        .find(|binding| binding.service_id == controller_service)
        .unwrap()
        .id
        .clone();
    let maximum_concurrent_proposals = bindings
        .iter()
        .find(|binding| binding.id == controller_binding_id)
        .unwrap()
        .limits
        .max_concurrency;
    let service_id = rrd_contract::CanonicalId::new("controlled-loopback-worker").unwrap();
    let binding_id = rrd_contract::CanonicalId::new("controlled-loopback-binding").unwrap();
    let manifest_sha256 = rrd_core::digest::sha256_hex(b"controlled loopback endpoint fixture");
    for binding in &mut bindings {
        binding.id = binding_id.clone();
        binding.endpoint = rrd_contract::EndpointLocator::Url {
            url: format!("http://{}/v1", listener.local_addr().unwrap()),
        };
        binding.credential = None;
        binding.manifest_sha256 = Some(manifest_sha256.clone());
        binding.service_id = service_id.clone();
        binding.limits.timeout_ms = 5_000;
    }
    let mut candidates = allowance.candidates;
    for candidate in &mut candidates {
        candidate.binding_id = binding_id.clone();
        candidate.manifest_sha256 = Some(manifest_sha256.clone());
        candidate.service_id = service_id.clone();
    }
    let selection_policy = rrd_contract::EndpointSelectionPolicy {
        contract_version: allowance.contract_version,
        purpose: rrd_contract::EndpointSelectionPurpose::ExperimentalProposal,
        allow_unverified_identity: true,
        allow_network: true,
        maximum_concurrent_proposals,
        controller_binding_id: binding_id,
        workload_class: allowance.workload_class,
        limits: allowance.limits,
        candidates,
        system_prompt: "Select the single explicitly allowed controlled loopback test binding."
            .into(),
        maximum_output_tokens: 64,
        temperature_milli: 0,
        lfg: None,
    };
    let worker_binding = selection_policy.candidates[0].binding_id.clone();
    rrd_contract::EndpointProfileInput {
        contract_version: profile.contract_version,
        name: rrd_contract::CanonicalId::new("controlled-installed-source-worker").unwrap(),
        bindings,
        selection_policy: Some(selection_policy),
        observation_policy: None,
        assessment_policy: None,
        worker_policy: Some(rrd_contract::EndpointWorkerPolicy {
            contract_version: rrd_contract::ENDPOINT_WORKER_VERSION,
            allow_network: true,
            allow_context_export,
            allow_source_export,
            allow_unverified_identity: true,
            maximum_concurrent_calls: 1,
            bindings: vec![rrd_contract::EndpointWorkerBinding {
                binding_id: worker_binding,
                protocol_evidence_sha256: rrd_core::digest::sha256_hex(
                    b"controlled loopback protocol fixture; no model or quality claim",
                ),
                adapter: rrd_contract::EndpointWorkerAdapter::default(),
                system_prompt: "Return a bounded draft. Do not mutate project files.".into(),
                generation: rrd_contract::EndpointGenerationSettings {
                    maximum_output_tokens: 128,
                    token_parameter: rrd_contract::EndpointTokenParameter::MaxTokens,
                    temperature_milli: None,
                    response_format: rrd_contract::EndpointResponseFormat::Text,
                },
                maximum_response_bytes: 16_384,
                effort_values: Default::default(),
                allow_default_effort: true,
            }],
        }),
    }
}

pub(super) fn accept_loopback_chat(listener: &TcpListener) -> (TcpStream, serde_json::Value) {
    listener.set_nonblocking(true).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let (mut socket, _) = loop {
        match listener.accept() {
            Ok(socket) => break socket,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("bounded loopback provider accept failed: {error}"),
        }
    };
    socket.set_nonblocking(false).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        header.push(byte[0]);
        assert!(header.len() < 8_192);
    }
    let header = String::from_utf8(header).unwrap();
    assert!(header.starts_with("POST /v1/chat/completions HTTP/1.1"));
    let length: usize = header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().unwrap())
        })
        .unwrap();
    assert!(length < 524_288);
    let mut body = vec![0; length];
    socket.read_exact(&mut body).unwrap();
    (socket, serde_json::from_slice(&body).unwrap())
}

pub(super) fn reply_loopback_chat(socket: &mut TcpStream, model: &str, content: &str) {
    let body = serde_json::to_vec(&serde_json::json!({
        "model": model,
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": content}}]
    }))
    .unwrap();
    write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
    socket.write_all(&body).unwrap();
}

pub(super) fn exercise_installed_source_worker(
    project: &Path,
    source_read_enabled: bool,
    job_id: &str,
) {
    let mut lifecycle_output = Vec::new();
    let (mut service, address) = RunningEngine::start(project, &mut lifecycle_output);
    let locator = RrdEngine::read_project_locator(project).unwrap();
    let credential = RrdEngine::read_installed_api_key(project).unwrap();
    let client = RrdClient::connect_local(
        address.parse().unwrap(),
        locator.identity.instance_id.clone(),
        ClientConfig::default(),
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let session = runtime
        .block_on(
            client.create_session(
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
                    "source-choice-session-request",
                    "source-choice-session-operation",
                    "source-choice-session-idempotency",
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let read = runtime
        .block_on(
            client.read_attunement(
                &session,
                rrd_contract::ReadAttunement {
                    contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                    job_id: rrd_contract::CanonicalId::new(job_id).unwrap(),
                },
                RequestOptions::read(
                    "source-choice-attunement-read",
                    "source-choice-attunement-read",
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let job = runtime.block_on(inventory_through_client(
        &client,
        &session,
        &read.status.job,
        &read.plan,
        "source-choice-inventory",
    ));
    let now = unix_ms().max(job.updated_at_unix_ms + 2);
    let scope = format!("instance:{}", client.instance_id());
    let source_query = runtime
        .block_on(client.execute_query(
            &session,
            ExecuteQuery {
                scope: scope.clone(),
                query: format!("FROM record:source-tree-entry AT VALID {now} KNOWN HEAD WHERE path = \"source.txt\" PROJECT identity, path, source_file"),
                parameters: BTreeMap::new(),
                budget: QueryBudget::default(),
            },
            RequestOptions::read("source-choice-inventory-query", "source-choice-inventory-query")
                .unwrap(),
        ))
        .unwrap();
    assert_eq!(source_query.rows.len(), 1);
    let values = &source_query.rows[0].values;
    let source_identity = match values.get("identity").unwrap() {
        QueryValue::String(value) | QueryValue::Digest(value) => value.clone(),
        value => panic!("unexpected source identity value {value:?}"),
    };
    let source_path = match values.get("path").unwrap() {
        QueryValue::String(value) => value.clone(),
        value => panic!("unexpected source path value {value:?}"),
    };
    let content_sha256 = match values.get("source_file").unwrap() {
        QueryValue::Map(values) => match values.get("content_sha256").unwrap() {
            QueryValue::String(value) | QueryValue::Digest(value) => value.clone(),
            value => panic!("unexpected source content digest {value:?}"),
        },
        value => panic!("unexpected source_file value {value:?}"),
    };
    let source_content = std::fs::read_to_string(project.join(&source_path)).unwrap();
    let task = runtime
        .block_on(
            client.open_task(
                &session,
                rrd_contract::OpenTask {
                    scope: scope.clone(),
                    title: "Inspect selected source".into(),
                    goal: "Return one bounded source-grounded draft.".into(),
                },
                RequestOptions::mutation(
                    "source-choice-open-task",
                    "source-choice-open-task",
                    "source-choice-open-task",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .task;
    let task_read = runtime
        .block_on(client.read_task(
            &session,
            rrd_contract::ReadTask {
                scope: scope.clone(),
                task_id: task.task_id.clone(),
                max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
            },
            RequestOptions::read("source-choice-read-task", "source-choice-read-task").unwrap(),
        ))
        .unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let worker = InstalledWorker {
        runtime: &runtime,
        client: &client,
        session: &session,
        listener: &listener,
    };
    let selection_configuration = install_loopback_profile(
        &runtime, &client, &session, &listener, true, true, "initial",
    );
    let selection_request = rrd_contract::EndpointSelectionRequest {
        scope: scope.clone(),
        task_id: task.task_id.clone(),
        attempt_id: rrd_contract::CorrelationId::new("source-choice-select-initial").unwrap(),
        max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
        expected_task_sha256: rrd_contract::EndpointSelectionTask::from_task(&task_read.task)
            .unwrap()
            .task_sha256,
        expected_configuration_sha256: selection_configuration.sha256().unwrap(),
        flight: None,
        routing: None,
    };

    if !source_read_enabled {
        let proposal = propose_over_loopback(
            &runtime,
            &client,
            &session,
            &listener,
            selection_request,
            "default-denial",
        );
        worker.assert_denied(
            WorkerAttempt {
                proposal: &proposal,
                scope: &scope,
                task_id: &task.task_id,
                source: Some((&source_identity, &content_sha256)),
                context: None,
                suffix: "default-denial",
            },
            rrd_contract::ErrorCode::PermissionDenied,
        );
        runtime
            .block_on(
                client.close_session(
                    &session,
                    rrd_contract::CloseSession {},
                    RequestOptions::mutation(
                        "source-choice-default-close-session",
                        "source-choice-default-close-session",
                        "source-choice-default-close-session",
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        service.stop(&mut lifecycle_output);
        return;
    }

    let source_denial_profile = install_loopback_profile(
        &runtime,
        &client,
        &session,
        &listener,
        true,
        false,
        "source-export-denial",
    );
    let source_denial_request = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &source_denial_profile,
        "source-choice-select-source-denial",
    );
    let proposal = propose_over_loopback(
        &runtime,
        &client,
        &session,
        &listener,
        source_denial_request,
        "source-export-denial",
    );
    worker.assert_denied(
        WorkerAttempt {
            proposal: &proposal,
            scope: &scope,
            task_id: &task.task_id,
            source: Some((&source_identity, &content_sha256)),
            context: None,
            suffix: "source-export-denial",
        },
        rrd_contract::ErrorCode::PermissionDenied,
    );

    let context_denial_profile = install_loopback_profile(
        &runtime,
        &client,
        &session,
        &listener,
        false,
        true,
        "context-export-denial",
    );
    let context_denial_request = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &context_denial_profile,
        "source-choice-select-context-denial",
    );
    let proposal = propose_over_loopback(
        &runtime,
        &client,
        &session,
        &listener,
        context_denial_request,
        "context-export-denial",
    );
    let context = rrd_contract::AssembleContext {
        scope: scope.clone(),
        query: "selected source".into(),
        valid_at: now,
        seeds: Vec::new(),
        max_graph_depth: 1,
        max_items: 8,
        max_output_bytes: 8192,
        max_storage_keys: 10_000,
    };
    worker.assert_denied(
        WorkerAttempt {
            proposal: &proposal,
            scope: &scope,
            task_id: &task.task_id,
            source: Some((&source_identity, &content_sha256)),
            context: Some(context.clone()),
            suffix: "context-export-denial",
        },
        rrd_contract::ErrorCode::PermissionDenied,
    );

    let ready_profile =
        install_loopback_profile(&runtime, &client, &session, &listener, true, true, "ready");
    let ignored_path_selection = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &ready_profile,
        "source-choice-select-ignored-path",
    );
    let proposal = propose_over_loopback(
        &runtime,
        &client,
        &session,
        &listener,
        ignored_path_selection,
        "ignored-source-path",
    );
    std::fs::write(project.join(".gitignore"), b"source.txt\n").unwrap();
    assert_eq!(
        std::fs::read(project.join(&source_path)).unwrap(),
        source_content.as_bytes()
    );
    worker.assert_denied(
        WorkerAttempt {
            proposal: &proposal,
            scope: &scope,
            task_id: &task.task_id,
            source: Some((&source_identity, &content_sha256)),
            context: None,
            suffix: "ignored-source-path",
        },
        rrd_contract::ErrorCode::FailedPrecondition,
    );
    std::fs::remove_file(project.join(".gitignore")).unwrap();
    let ready_selection = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &ready_profile,
        "source-choice-select-ready",
    );
    let (proposal, wire_source) = worker.propose_and_execute(
        ready_selection,
        &scope,
        &task.task_id,
        Some((&source_identity, &content_sha256)),
        Some(context.clone()),
    );
    let output = wire_source.expect("successful loopback worker received source packet");
    assert_eq!(output, source_content);
    assert_eq!(proposal.content, "Reviewed one selected file.");
    assert!(proposal.source.is_some());

    let assessment_task = runtime
        .block_on(
            client.open_task(
                &session,
                rrd_contract::OpenTask {
                    scope: scope.clone(),
                    title: "Assess selected source separately".into(),
                    goal: "Evaluate explicit typed questions over one accepted source file.".into(),
                },
                RequestOptions::mutation(
                    "source-choice-assessment-open-task",
                    "source-choice-assessment-open-task",
                    "source-choice-assessment-open-task",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .task;
    let assessment_task_read = runtime
        .block_on(
            client.read_task(
                &session,
                rrd_contract::ReadTask {
                    scope: scope.clone(),
                    task_id: assessment_task.task_id.clone(),
                    max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
                },
                RequestOptions::read(
                    "source-choice-assessment-read-task",
                    "source-choice-assessment-read-task",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .task;
    assert_ne!(assessment_task_read.task_id, task.task_id);

    let budget_request = rrd_contract::ConfigureTaskBudget {
        scope: scope.clone(),
        task_id: assessment_task_read.task_id.clone(),
        limits: rrd_contract::ReasoningLimits {
            max_run_elapsed_ms: 120_000,
            max_steps: 8,
            max_step_elapsed_ms: 30_000,
        },
        repetition_policy: None,
    };
    let budget_before = runtime
        .block_on(
            client.configure_task_budget(
                &session,
                budget_request.clone(),
                RequestOptions::mutation(
                    "source-choice-assessment-budget-configure",
                    "source-choice-assessment-budget-configure",
                    "source-choice-assessment-budget-configure",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .budget
        .expect("public assessment task budget is configured");
    assert_eq!(budget_before.reserved_steps, 0);

    let assessment_configuration = install_loopback_assessment_profile(
        &runtime,
        &client,
        &session,
        &listener,
        true,
        true,
        "assessment-enabled",
    );
    let assessment_policy = assessment_configuration
        .profile
        .as_ref()
        .unwrap()
        .assessment_policy
        .as_ref()
        .expect("assessment requires explicit endpoint policy");
    assert!(assessment_policy.allow_network);
    assert!(assessment_policy.allow_source_export);
    assert!(assessment_policy.allow_context_export);
    let task_sha256 = rrd_contract::EndpointSelectionTask::from_task(&assessment_task_read)
        .unwrap()
        .task_sha256;
    let source_request = rrd_contract::TaskWorkerSourceRequest {
        contract_version: rrd_contract::TASK_WORKER_SOURCE_VERSION,
        entry_identity: source_identity.clone(),
        expected_content_sha256: content_sha256.clone(),
        maximum_input_bytes: 16_384,
    };
    let practice_identity = AssessmentPracticeIdentity {
        question_set_id: CanonicalId::new("installed-assessment-set").unwrap(),
        question_id: CanonicalId::new("installed-assessment-question").unwrap(),
        template_id: CanonicalId::new("installed-assessment-template").unwrap(),
        template_revision: 1,
        template_sha256: rrd_core::digest::sha256_hex(b"controlled installed practice declaration"),
        requirement_id: CanonicalId::new("installed-assessment-requirement").unwrap(),
    };
    let practice_text = "Review only the selected source bytes against this declared practice.";
    let descriptors = [
        TaskAssessmentDescriptor::Binary {
            proposition: "The selected source contains the stated fixture phrase.".into(),
        },
        TaskAssessmentDescriptor::Choice {
            question: "Which declared relation best matches the selected source?".into(),
            options: vec![
                AssessmentOption {
                    id: CanonicalId::new("supported").unwrap(),
                    description: "The selected bytes contain direct support.".into(),
                },
                AssessmentOption {
                    id: CanonicalId::new("uncertain").unwrap(),
                    description: "The selected bytes do not settle the question.".into(),
                },
            ],
        },
        TaskAssessmentDescriptor::Score {
            rubric: "Score how clearly the selected bytes state the fixture phrase.".into(),
            minimum: 0.0,
            maximum: 5.0,
            direction: AssessmentScoreDirection::HigherIsBetter,
        },
        TaskAssessmentDescriptor::Binary {
            proposition: "The selected source supports the declared practice.".into(),
        },
    ];
    let decisions = [
        (
            serde_json::json!({
                "outcome": "answered",
                "value": { "kind": "binary", "value": true }
            }),
            Some("source"),
        ),
        (
            serde_json::json!({
                "outcome": "answered",
                "value": { "kind": "choice", "option_id": "supported" }
            }),
            Some("source"),
        ),
        (
            serde_json::json!({
                "outcome": "answered",
                "value": { "kind": "score", "value": 3.0 }
            }),
            Some("practice"),
        ),
        (
            serde_json::json!({
                "outcome": "abstained",
                "reason": "The controlled packet does not contain the requested evidence."
            }),
            None,
        ),
    ];
    let mut assessment_outputs = Vec::new();
    let mut observed_budget = budget_before.clone();
    for (index, (descriptor, (decision, reference_input))) in
        descriptors.into_iter().zip(decisions).enumerate()
    {
        let suffix = ["binary", "choice", "score", "abstention"][index];
        let request = TaskAssessmentRequest {
            contract_version: TASK_ASSESSMENT_VERSION,
            scope: scope.clone(),
            task_id: assessment_task_read.task_id.clone(),
            attempt_id: rrd_contract::CorrelationId::new(format!(
                "source-choice-assessment-{suffix}"
            ))
            .unwrap(),
            expected_task_sha256: task_sha256.clone(),
            expected_configuration_sha256: assessment_configuration.sha256().unwrap(),
            max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
            source: source_request.clone(),
            descriptor,
            practice_identity: practice_identity.clone(),
            practice_text: practice_text.into(),
        };
        let (assessment, packet) = execute_installed_assessment(
            &runtime,
            &client,
            &session,
            &listener,
            &request,
            decision,
            reference_input,
        );
        assert_eq!(packet["scope"], request.scope);
        assert_eq!(packet["task_id"], request.task_id.as_str());
        assert_eq!(packet["task_sha256"], task_sha256);
        assert_eq!(
            packet["configuration_sha256"],
            request.expected_configuration_sha256
        );
        assert_eq!(
            packet["descriptor"],
            serde_json::to_value(&request.descriptor).unwrap()
        );
        assert_eq!(
            packet["practice_identity"],
            serde_json::to_value(&practice_identity).unwrap()
        );
        assert_eq!(packet["practice_text"], practice_text);
        assert_eq!(
            packet["practice_sha256"],
            rrd_core::digest::sha256_hex(practice_text.as_bytes())
        );
        assert_eq!(packet["source_content"], source_content);
        assert_eq!(packet["source"]["entry_identity"], source_identity);
        assert_eq!(packet["source"]["content_sha256"], content_sha256);
        assert_eq!(assessment.source.entry_identity, source_identity);
        assert_eq!(assessment.source.content_sha256, content_sha256);
        assert_eq!(
            assessment.practice_sha256,
            rrd_core::digest::sha256_hex(practice_text.as_bytes())
        );
        match (&request.descriptor, &assessment.decision, reference_input) {
            (
                TaskAssessmentDescriptor::Binary { .. },
                TaskAssessmentDecision::Answered {
                    value: TaskAssessmentValue::Binary { value: true },
                },
                Some("source"),
            ) if index == 0 => {}
            (
                TaskAssessmentDescriptor::Choice { .. },
                TaskAssessmentDecision::Answered {
                    value: TaskAssessmentValue::Choice { option_id },
                },
                Some("source"),
            ) if index == 1 && option_id.as_str() == "supported" => {}
            (
                TaskAssessmentDescriptor::Score { .. },
                TaskAssessmentDecision::Answered {
                    value: TaskAssessmentValue::Score { value: 3.0 },
                },
                Some("practice"),
            ) if index == 2 => {}
            (
                TaskAssessmentDescriptor::Binary { .. },
                TaskAssessmentDecision::Abstained { reason },
                None,
            ) if index == 3 && !reason.is_empty() => {}
            other => panic!("unexpected controlled assessment result: {other:?}"),
        }
        match (reference_input, assessment.references.as_slice()) {
            (Some("source"), [reference]) => {
                assert_eq!(reference.input, AssessmentInputKind::Source);
                assert_eq!(reference.input_sha256, content_sha256);
                assert_eq!((reference.start_byte, reference.end_byte), (0, 7));
                assert_eq!(&source_content[0..7], "bounded");
            }
            (Some("practice"), [reference]) => {
                assert_eq!(reference.input, AssessmentInputKind::Practice);
                assert_eq!(
                    reference.input_sha256,
                    rrd_core::digest::sha256_hex(practice_text.as_bytes())
                );
                assert_eq!((reference.start_byte, reference.end_byte), (0, 6));
                assert_eq!(&practice_text[0..6], "Review");
            }
            (None, []) => {}
            other => panic!("unexpected assessment reference binding: {other:?}"),
        }
        let next_budget = runtime
            .block_on(
                client.read_task_budget(
                    &session,
                    rrd_contract::ReadTaskBudget {
                        scope: scope.clone(),
                        task_id: assessment_task_read.task_id.clone(),
                    },
                    RequestOptions::read(
                        &format!("source-choice-assessment-budget-read-{suffix}"),
                        &format!("source-choice-assessment-budget-read-{suffix}"),
                    )
                    .unwrap(),
                ),
            )
            .unwrap()
            .budget
            .expect("public task budget remains readable");
        assert_eq!(
            next_budget.reserved_steps,
            observed_budget.reserved_steps + 1,
            "one explicit attempt reserves exactly one measured task-budget step"
        );
        assert!(next_budget.active_attempt.is_none());
        assert!(!next_budget.execution_may_continue);
        observed_budget = next_budget;
        assessment_outputs.push((request, assessment));
    }
    assert_eq!(observed_budget.reserved_steps, 4);

    let raw_session = open_raw_public_session(
        &address,
        &locator.identity.instance_id,
        &credential.principal_id,
        credential.credential(),
    );
    let mut malformed_request = assessment_outputs[0].0.clone();
    malformed_request.attempt_id =
        rrd_contract::CorrelationId::new("source-choice-assessment-unsupported-context").unwrap();
    let mut payload = serde_json::to_value(malformed_request).unwrap();
    payload["context"] = serde_json::to_value(&context).unwrap();
    let malformed_envelope = raw_envelope(
        &locator.identity.instance_id,
        payload,
        "source-choice-assessment-unsupported-context-request",
        "source-choice-assessment-unsupported-context-operation",
        Some("source-choice-assessment-unsupported-context-idempotency"),
    );
    let assessment_catalogue = rrd_contract::endpoint_catalogue();
    let assessment_execute_path = assessment_catalogue
        .endpoints
        .iter()
        .find(|endpoint| endpoint.operation.as_str() == "task-assessment-execute")
        .expect("public assessment execute endpoint is catalogued")
        .path
        .as_str();
    let (status, response) = raw_public_http(
        &address,
        "POST",
        assessment_execute_path,
        &malformed_envelope,
        &[
            ("X-RRD-Session", raw_session.session_id.as_str().to_owned()),
            (
                "Authorization",
                format!("Bearer {}", raw_session.token.as_str()),
            ),
        ],
    );
    assert_eq!(status, 400);
    assert_eq!(response["outcome"]["status"], "error");
    assert_eq!(response["outcome"]["error"]["code"], "invalid_argument");
    assert!(response["outcome"]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown field `context`"));
    assert_listener_empty(&listener);
    let after_unsupported_context = runtime
        .block_on(
            client.read_task_budget(
                &session,
                rrd_contract::ReadTaskBudget {
                    scope: scope.clone(),
                    task_id: assessment_task_read.task_id.clone(),
                },
                RequestOptions::read(
                    "source-choice-assessment-budget-after-unsupported-context",
                    "source-choice-assessment-budget-after-unsupported-context",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .budget
        .unwrap();
    assert_eq!(after_unsupported_context, observed_budget);

    for (suffix, allow_context_export, allow_source_export) in [
        ("source-export-denied", true, false),
        ("context-export-denied", false, true),
    ] {
        install_loopback_assessment_profile(
            &runtime,
            &client,
            &session,
            &listener,
            allow_context_export,
            allow_source_export,
            suffix,
        );
        let read = runtime.block_on(
            client.call_operation::<rrd_client::operations::TaskAssessmentRead>(
                &session,
                rrd_contract::ReadTaskAssessment {
                    contract_version: TASK_ASSESSMENT_VERSION,
                    scope: scope.clone(),
                    task_id: assessment_task_read.task_id.clone(),
                    attempt_id: assessment_outputs[0].0.attempt_id.clone(),
                },
                RequestOptions::read(
                    &format!("source-choice-assessment-read-{suffix}"),
                    &format!("source-choice-assessment-read-{suffix}"),
                )
                .unwrap(),
            ),
        );
        assert!(matches!(
            read,
            Err(rrd_client::Error::Api { error, .. })
                if error.code == ErrorCode::PermissionDenied
        ));
        assert_listener_empty(&listener);
    }
    install_loopback_assessment_profile(
        &runtime,
        &client,
        &session,
        &listener,
        true,
        true,
        "assessment-export-restored",
    );
    for (request, expected) in &assessment_outputs {
        let retained = runtime
            .block_on(
                client.call_operation::<rrd_client::operations::TaskAssessmentRead>(
                    &session,
                    rrd_contract::ReadTaskAssessment {
                        contract_version: TASK_ASSESSMENT_VERSION,
                        scope: scope.clone(),
                        task_id: assessment_task_read.task_id.clone(),
                        attempt_id: request.attempt_id.clone(),
                    },
                    RequestOptions::read(
                        &format!(
                            "source-choice-assessment-read-{}-restored",
                            request.attempt_id.as_str()
                        ),
                        &format!(
                            "source-choice-assessment-read-{}-restored",
                            request.attempt_id.as_str()
                        ),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(retained.assessment, Some(expected.clone()));
    }

    raw_close_public_session(&address, &locator.identity.instance_id, &raw_session);
    let attempt_id = proposal.attempt_id.clone();

    let close = runtime.block_on(
        client.close_session(
            &session,
            rrd_contract::CloseSession {},
            RequestOptions::mutation(
                "source-choice-close-session",
                "source-choice-close-session",
                "source-choice-close-session",
            )
            .unwrap(),
        ),
    );
    close.unwrap();
    service.stop(&mut lifecycle_output);
    let (mut reopened_service, reopened_address) =
        RunningEngine::start(project, &mut lifecycle_output);
    let reopened_locator = RrdEngine::read_project_locator(project).unwrap();
    let reopened_client = RrdClient::connect_local(
        reopened_address.parse().unwrap(),
        reopened_locator.identity.instance_id,
        ClientConfig::default(),
    )
    .unwrap();
    let fresh_session = runtime
        .block_on(
            reopened_client.create_session(
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
                    "source-choice-reauth-session-request",
                    "source-choice-reauth-session-operation",
                    "source-choice-reauth-session-idempotency",
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let readback = runtime
        .block_on(reopened_client.read_task_attempt(
            &fresh_session,
            rrd_contract::ReadTaskAttempt {
                scope: scope.clone(),
                task_id: task.task_id,
                attempt_id,
            },
            RequestOptions::read("source-choice-readback", "source-choice-readback").unwrap(),
        ))
        .unwrap();
    let retained = readback.attempt.unwrap().worker_output.unwrap();
    assert_eq!(retained.content, "Reviewed one selected file.");
    assert_eq!(retained.source.unwrap().content_sha256, content_sha256);
    assert_eq!(
        std::fs::read_to_string(project.join(source_path)).unwrap(),
        source_content
    );
    for (request, expected) in &assessment_outputs {
        let assessment_read = runtime
            .block_on(
                reopened_client.call_operation::<rrd_client::operations::TaskAssessmentRead>(
                    &fresh_session,
                    rrd_contract::ReadTaskAssessment {
                        contract_version: TASK_ASSESSMENT_VERSION,
                        scope: scope.clone(),
                        task_id: assessment_task_read.task_id.clone(),
                        attempt_id: request.attempt_id.clone(),
                    },
                    RequestOptions::read(
                        &format!(
                            "source-choice-assessment-reopen-read-{}",
                            request.attempt_id.as_str()
                        ),
                        &format!(
                            "source-choice-assessment-reopen-read-{}",
                            request.attempt_id.as_str()
                        ),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(assessment_read.assessment, Some(expected.clone()));
    }
    let reopened_budget = runtime
        .block_on(
            reopened_client.read_task_budget(
                &fresh_session,
                rrd_contract::ReadTaskBudget {
                    scope: scope.clone(),
                    task_id: assessment_task_read.task_id.clone(),
                },
                RequestOptions::read(
                    "source-choice-assessment-budget-reopen-read",
                    "source-choice-assessment-budget-reopen-read",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .budget
        .expect("assessment budget survives authenticated reopen");
    assert_eq!(reopened_budget.reserved_steps, 4);
    assert_eq!(reopened_budget.limits, budget_request.limits);
    assert_eq!(
        reopened_budget.deadline_unix_ms,
        observed_budget.deadline_unix_ms
    );
    assert_eq!(
        reopened_budget.charged_elapsed_ms,
        observed_budget.charged_elapsed_ms
    );
    assert!(reopened_budget.active_attempt.is_none());
    assert!(!reopened_budget.execution_may_continue);
    reopened_service.stop(&mut lifecycle_output);
    let verify = invoke(&[
        "--json",
        "verify",
        "--project",
        project.to_str().unwrap(),
        "--level",
        "quick",
    ]);
    assert_success(&verify, "verify source-read grant binding after reopen");
}

pub(super) fn execute_installed_assessment(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    request: &TaskAssessmentRequest,
    decision: serde_json::Value,
    reference_input: Option<&str>,
) -> (rrd_contract::TaskAssessmentOutput, serde_json::Value) {
    std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, wire) = accept_loopback_chat(listener);
            assert_eq!(wire["messages"][0]["role"], "system");
            assert_eq!(
                wire["messages"][0]["content"],
                "Assess only the supplied packet; this controlled fixture is not a model result."
            );
            let packet: serde_json::Value =
                serde_json::from_str(wire["messages"][1]["content"].as_str().unwrap()).unwrap();
            assert!(packet.get("context").is_none());
            let response = assessment_reply_for_packet(&packet, decision, reference_input);
            reply_loopback_chat(&mut socket, wire["model"].as_str().unwrap(), &response);
            packet
        });
        let output = runtime
            .block_on(
                client.call_operation::<rrd_client::operations::TaskAssessmentExecute>(
                    session,
                    request.clone(),
                    RequestOptions::mutation(
                        &format!("assessment-execute-{}", request.attempt_id.as_str()),
                        &format!("assessment-execute-{}", request.attempt_id.as_str()),
                        &format!("assessment-execute-{}", request.attempt_id.as_str()),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        let packet = server.join().unwrap();
        assert_listener_empty(listener);
        (output, packet)
    })
}

pub(super) fn assessment_reply_for_packet(
    packet: &serde_json::Value,
    decision: serde_json::Value,
    reference_input: Option<&str>,
) -> String {
    let references = reference_input
        .map(|input| {
            let (sha256, end_byte) = match input {
                "source" => (packet["source"]["content_sha256"].as_str().unwrap(), 7),
                "practice" => (packet["practice_sha256"].as_str().unwrap(), 6),
                other => panic!("unsupported controlled assessment reference input {other}"),
            };
            serde_json::json!([{
                "input": input,
                "input_sha256": sha256,
                "start_byte": 0,
                "end_byte": end_byte
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

pub(super) fn assert_listener_empty(listener: &TcpListener) {
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

pub(super) fn raw_envelope<T: serde::Serialize>(
    instance_id: &CanonicalId,
    payload: T,
    request_id: &str,
    operation_id: &str,
    idempotency_key: Option<&str>,
) -> rrd_contract::RequestEnvelope<T> {
    rrd_contract::RequestEnvelope {
        protocol: rrd_contract::PROTOCOL.into(),
        protocol_version: rrd_contract::PROTOCOL_VERSION,
        context: rrd_contract::RequestContext {
            request_id: rrd_contract::CorrelationId::new(request_id).unwrap(),
            operation_id: rrd_contract::CorrelationId::new(operation_id).unwrap(),
            idempotency_key: idempotency_key
                .map(|key| rrd_contract::CorrelationId::new(key).unwrap()),
            deadline_unix_ms: None,
        },
        resource: rrd_contract::ResourcePath {
            segments: vec![rrd_contract::ResourceId::new(
                rrd_contract::ResourceKind::Instance,
                instance_id.as_str(),
            )
            .unwrap()],
        },
        payload,
    }
}

pub(super) fn raw_public_http(
    address: &str,
    method: &str,
    path: &str,
    envelope: &impl serde::Serialize,
    headers: &[(&str, String)],
) -> (u16, serde_json::Value) {
    let body = serde_json::to_vec(envelope).unwrap();
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n", body.len()).unwrap();
    for (name, value) in headers {
        write!(stream, "{name}: {value}\r\n").unwrap();
    }
    stream.write_all(b"\r\n").unwrap();
    stream.write_all(&body).unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    let split = response
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .unwrap();
    let status = String::from_utf8_lossy(&response[..split])
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    (
        status,
        serde_json::from_slice(&response[split + 4..]).unwrap(),
    )
}

pub(super) fn open_raw_public_session(
    address: &str,
    instance_id: &CanonicalId,
    principal_id: &CanonicalId,
    api_key: &str,
) -> rrd_contract::SessionLease {
    let envelope = raw_envelope(
        instance_id,
        CreateSession {
            limits: SessionLimits {
                idle_timeout_ms: 60_000,
                absolute_timeout_ms: 300_000,
                max_open_transactions: 1,
            },
        },
        "assessment-raw-session-request",
        "assessment-raw-session-operation",
        Some("assessment-raw-session-idempotency"),
    );
    let (status, response) = raw_public_http(
        address,
        "POST",
        "/v1/sessions",
        &envelope,
        &[
            ("X-RRD-Principal", principal_id.as_str().to_owned()),
            ("Authorization", format!("ApiKey {api_key}")),
        ],
    );
    assert_eq!(status, 200);
    assert_eq!(response["outcome"]["status"], "ok");
    serde_json::from_value(response["outcome"]["payload"].clone()).unwrap()
}

pub(super) fn raw_close_public_session(
    address: &str,
    instance_id: &CanonicalId,
    lease: &rrd_contract::SessionLease,
) {
    let envelope = raw_envelope(
        instance_id,
        CloseSession {},
        "assessment-raw-session-close-request",
        "assessment-raw-session-close-operation",
        Some("assessment-raw-session-close-idempotency"),
    );
    let (status, response) = raw_public_http(
        address,
        "DELETE",
        &format!("/v1/sessions/{}", lease.session_id.as_str()),
        &envelope,
        &[
            ("X-RRD-Session", lease.session_id.as_str().to_owned()),
            ("Authorization", format!("Bearer {}", lease.token.as_str())),
        ],
    );
    assert_eq!(status, 200);
    assert_eq!(response["outcome"]["status"], "ok");
}

pub(super) fn install_loopback_profile(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    allow_context_export: bool,
    allow_source_export: bool,
    suffix: &str,
) -> rrd_contract::EndpointConfiguration {
    let profile = loopback_worker_profile(listener, allow_context_export, allow_source_export);
    apply_loopback_profile(runtime, client, session, suffix, profile)
}

pub(super) fn install_loopback_assessment_profile(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    allow_context_export: bool,
    allow_source_export: bool,
    suffix: &str,
) -> rrd_contract::EndpointConfiguration {
    let mut profile = loopback_worker_profile(listener, allow_context_export, allow_source_export);
    profile.assessment_policy = Some(EndpointAssessmentPolicy {
        contract_version: TASK_ASSESSMENT_VERSION,
        allow_network: true,
        allow_source_export,
        allow_context_export,
        system_prompt:
            "Assess only the supplied packet; this controlled fixture is not a model result.".into(),
        generation: EndpointGenerationSettings {
            maximum_output_tokens: 128,
            token_parameter: EndpointTokenParameter::MaxTokens,
            temperature_milli: None,
            response_format: EndpointResponseFormat::JsonObject,
        },
        maximum_input_bytes: 16_384,
        maximum_response_bytes: 16_384,
    });
    apply_loopback_profile(runtime, client, session, suffix, profile)
}

pub(super) fn apply_loopback_profile(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    suffix: &str,
    profile: rrd_contract::EndpointProfileInput,
) -> rrd_contract::EndpointConfiguration {
    let current = runtime
        .block_on(
            client.read_endpoint_configuration(
                session,
                rrd_contract::ReadEndpointConfiguration {},
                RequestOptions::read(
                    &format!("source-choice-profile-read-{suffix}"),
                    &format!("source-choice-profile-read-{suffix}"),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let plan = runtime
        .block_on(
            client.preview_endpoint_configuration(
                session,
                rrd_contract::PreviewEndpointConfiguration {
                    expected_revision: current.revision,
                    profile: Some(profile),
                },
                RequestOptions::mutation(
                    &format!("source-choice-profile-preview-{suffix}"),
                    &format!("source-choice-profile-preview-{suffix}"),
                    &format!("source-choice-profile-preview-{suffix}"),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    runtime
        .block_on(
            client.apply_endpoint_configuration(
                session,
                rrd_contract::ApplyEndpointConfiguration {
                    approved_plan_sha256: plan.plan_sha256.clone(),
                    plan,
                },
                RequestOptions::mutation(
                    &format!("source-choice-profile-apply-{suffix}"),
                    &format!("source-choice-profile-apply-{suffix}"),
                    &format!("source-choice-profile-apply-{suffix}"),
                )
                .unwrap(),
            ),
        )
        .unwrap()
}

pub(super) fn endpoint_selection_request(
    scope: &str,
    task_id: &rrd_contract::CanonicalId,
    task: &rrd_contract::TaskRecord,
    configuration: &rrd_contract::EndpointConfiguration,
    attempt_id: &str,
) -> rrd_contract::EndpointSelectionRequest {
    rrd_contract::EndpointSelectionRequest {
        scope: scope.to_owned(),
        task_id: task_id.clone(),
        attempt_id: rrd_contract::CorrelationId::new(attempt_id).unwrap(),
        max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
        expected_task_sha256: rrd_contract::EndpointSelectionTask::from_task(task)
            .unwrap()
            .task_sha256,
        expected_configuration_sha256: configuration.sha256().unwrap(),
        flight: None,
        routing: None,
    }
}

pub(super) fn propose_over_loopback(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    request: rrd_contract::EndpointSelectionRequest,
    suffix: &str,
) -> rrd_contract::EndpointSelectionProposal {
    std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, wire) = accept_loopback_chat(listener);
            let binding_id = rrd_contract::CanonicalId::new("controlled-loopback-binding").unwrap();
            let choice = rrd_contract::EndpointSelectionChoice::Select {
                binding_id,
                effort: None,
            };
            let model = wire["model"].as_str().unwrap();
            reply_loopback_chat(&mut socket, model, &serde_json::to_string(&choice).unwrap());
        });
        let result = runtime
            .block_on(
                client.call_operation::<rrd_client::operations::TaskEndpointPropose>(
                    session,
                    request,
                    RequestOptions::mutation(
                        &format!("source-choice-proposal-{suffix}"),
                        &format!("source-choice-proposal-{suffix}"),
                        &format!("source-choice-proposal-{suffix}"),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        server.join().unwrap();
        result
    })
}

pub(super) fn worker_request(
    proposal: &rrd_contract::EndpointSelectionProposal,
    scope: &str,
    task_id: &rrd_contract::CanonicalId,
    source: Option<(&str, &str)>,
    context: Option<rrd_contract::AssembleContext>,
    suffix: &str,
) -> rrd_contract::TaskWorkerRequest {
    rrd_contract::TaskWorkerRequest {
        scope: scope.to_owned(),
        task_id: task_id.clone(),
        attempt_id: rrd_contract::CorrelationId::new(format!("source-choice-worker-{suffix}"))
            .unwrap(),
        selection_attempt_id: proposal.attempt_id.clone(),
        expected_selection_sha256: rrd_core::digest::sha256_hex(
            &serde_json::to_vec(proposal).unwrap(),
        ),
        expected_task_sha256: proposal.task_sha256.clone(),
        expected_configuration_sha256: proposal.configuration_sha256.clone(),
        max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
        context,
        source: source.map(|(entry_identity, expected_content_sha256)| {
            rrd_contract::TaskWorkerSourceRequest {
                contract_version: rrd_contract::TASK_WORKER_SOURCE_VERSION,
                entry_identity: entry_identity.to_owned(),
                expected_content_sha256: expected_content_sha256.to_owned(),
                maximum_input_bytes: 64 * 1024,
            }
        }),
    }
}

pub(super) struct InstalledWorker<'a> {
    runtime: &'a tokio::runtime::Runtime,
    client: &'a RrdClient,
    session: &'a rrd_client::Session,
    listener: &'a TcpListener,
}

pub(super) struct WorkerAttempt<'a> {
    proposal: &'a rrd_contract::EndpointSelectionProposal,
    scope: &'a str,
    task_id: &'a rrd_contract::CanonicalId,
    source: Option<(&'a str, &'a str)>,
    context: Option<rrd_contract::AssembleContext>,
    suffix: &'a str,
}

impl InstalledWorker<'_> {
    fn assert_denied(&self, attempt: WorkerAttempt<'_>, expected_error: rrd_contract::ErrorCode) {
        let request = worker_request(
            attempt.proposal,
            attempt.scope,
            attempt.task_id,
            attempt.source,
            attempt.context,
            attempt.suffix,
        );
        let result = self.runtime.block_on(
            self.client.execute_task_worker(
                self.session,
                request,
                RequestOptions::mutation(
                    &format!("source-choice-worker-deny-{}", attempt.suffix),
                    &format!("source-choice-worker-deny-{}", attempt.suffix),
                    &format!("source-choice-worker-deny-{}", attempt.suffix),
                )
                .unwrap(),
            ),
        );
        assert!(matches!(
            result,
            Err(rrd_client::Error::Api { error, .. }) if error.code == expected_error
        ));
        self.listener.set_nonblocking(true).unwrap();
        assert!(matches!(
            self.listener.accept(),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
        ));
    }

    fn propose_and_execute(
        &self,
        selection_request: rrd_contract::EndpointSelectionRequest,
        scope: &str,
        task_id: &rrd_contract::CanonicalId,
        source: Option<(&str, &str)>,
        context: Option<rrd_contract::AssembleContext>,
    ) -> (rrd_contract::TaskWorkerOutput, Option<String>) {
        std::thread::scope(|scope_thread| {
            let server = scope_thread.spawn(|| {
                let (mut selection_socket, selection_wire) = accept_loopback_chat(self.listener);
                let choice = rrd_contract::EndpointSelectionChoice::Select {
                    binding_id: rrd_contract::CanonicalId::new("controlled-loopback-binding")
                        .unwrap(),
                    effort: None,
                };
                let model = selection_wire["model"].as_str().unwrap();
                reply_loopback_chat(
                    &mut selection_socket,
                    model,
                    &serde_json::to_string(&choice).unwrap(),
                );
                let (mut worker_socket, worker_wire) = accept_loopback_chat(self.listener);
                let packet: serde_json::Value =
                    serde_json::from_str(worker_wire["messages"][1]["content"].as_str().unwrap())
                        .unwrap();
                let source_text = packet["source"]["content"].as_str().map(str::to_owned);
                let model = worker_wire["model"].as_str().unwrap();
                reply_loopback_chat(&mut worker_socket, model, "Reviewed one selected file.");
                source_text
            });
            let proposal = self
                .runtime
                .block_on(
                    self.client
                        .call_operation::<rrd_client::operations::TaskEndpointPropose>(
                            self.session,
                            selection_request,
                            RequestOptions::mutation(
                                "source-choice-ready-proposal",
                                "source-choice-ready-proposal",
                                "source-choice-ready-proposal",
                            )
                            .unwrap(),
                        ),
                )
                .unwrap();
            let request = worker_request(&proposal, scope, task_id, source, context, "ready");
            let output = self
                .runtime
                .block_on(
                    self.client.execute_task_worker(
                        self.session,
                        request,
                        RequestOptions::mutation(
                            "source-choice-ready-worker",
                            "source-choice-ready-worker",
                            "source-choice-ready-worker",
                        )
                        .unwrap(),
                    ),
                )
                .unwrap();
            let source_text = server.join().unwrap();
            (output, source_text)
        })
    }
}
