use super::*;
use crate::engine::tests::attunement::{
    call as attunement_call, context as attunement_context,
    first_inventory_for_worker_with_setup_at, step as attunement_step,
};
use crate::AttunementCommand;
use rrd_contract::{
    ApplyEndpointConfiguration, CredentialReference, EndpointGenerationSettings, EndpointLocator,
    EndpointProfileInput, EndpointResponseFormat, EndpointTokenParameter, EndpointWorkerBinding,
    EndpointWorkerPolicy, ExecuteQuery, ObservationExportPolicy, ObservationSource, OpenTask,
    PrepareObservationDelivery, PreviewEndpointConfiguration, QueryBudget, QueryValue,
    ReadObservationDelivery, ReadTask, ReadTaskAttempt, TaskAttemptInvalidation,
    TaskWorkerSourceRequest,
};
use rrd_security::ResourceGrant;
use std::path::Path;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

struct SourceWorkerFixture {
    installed: crate::engine::tests::attunement::Installed,
    engine: RrdEngine,
    lease: SessionLease,
    job: rrd_contract::AttunementJob,
    principal_id: CanonicalId,
    worker_token: CorrelationId,
    selection: EndpointSelectionRequest,
    source: TaskWorkerSourceRequest,
    source_path: String,
    source_content: String,
    now: u64,
}

fn host_unix_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .unwrap()
}

fn worker_packet_from_chat_wire(wire: &serde_json::Value) -> serde_json::Value {
    serde_json::from_str(
        wire["messages"][1]["content"]
            .as_str()
            .expect("worker packet is encoded in the user message"),
    )
    .expect("worker packet JSON is valid")
}

fn set_query_execute_policy(
    engine: &RrdEngine,
    lease: &SessionLease,
    at: u64,
    data_policy: Option<rrd_security::DataPolicy>,
    suffix: &str,
) {
    let repository =
        rrd_security::SecurityRepository::new(&engine.storage, engine.instance.clone());
    let mut authority = repository.load().unwrap().unwrap();
    let expected_revision = authority.revision;
    let mut count = 0;
    for principal in authority.principals.values_mut() {
        for grant in &mut principal.grants {
            if grant.action == SecurityAction::QueryExecute {
                grant.data_policy = data_policy.clone();
                count += 1;
            }
        }
    }
    for role in authority.roles.values_mut() {
        for grant in &mut role.grants {
            if grant.action == SecurityAction::QueryExecute {
                grant.data_policy = data_policy.clone();
                count += 1;
            }
        }
    }
    assert!(count > 0, "fixture has a QueryExecute authorization grant");
    authority.revision += 1;
    engine
        .replace_security_authority(
            &lease.session_id,
            &lease.token,
            expected_revision,
            authority,
            at,
            &format!("source-worker-query-policy-{suffix}"),
            &format!("source-worker-query-policy-{suffix}"),
        )
        .unwrap();
}

fn verify_source_file_query_policy(
    engine: &RrdEngine,
    lease: &SessionLease,
    scope: &str,
    entry_identity: &str,
    at: u64,
) {
    let query = |projection: &str| {
        ExecuteQuery {
        scope: scope.to_owned(),
        query: format!(
            "FROM record:source-tree-entry AT VALID {at} KNOWN HEAD WHERE identity = \"{entry_identity}\" PROJECT {projection}"
        ),
        parameters: Default::default(),
        budget: QueryBudget::default(),
    }
    };
    set_query_execute_policy(
        engine,
        lease,
        at,
        Some(rrd_security::DataPolicy {
            tenant: None,
            rows: Vec::new(),
            allowed_fields: Some(std::collections::BTreeSet::from([
                "identity".into(),
                "path".into(),
                "source_file".into(),
            ])),
        }),
        "field-allowed",
    );
    let selected = engine
        .execute_query(
            &lease.session_id,
            &lease.token,
            &query("identity, path, source_file"),
            at + 1,
            "source-worker-source-file-query-allowed",
            "source-worker-source-file-query-allowed",
        )
        .unwrap();
    assert_eq!(selected.rows.len(), 1);

    set_query_execute_policy(
        engine,
        lease,
        at + 2,
        Some(rrd_security::DataPolicy {
            tenant: None,
            rows: Vec::new(),
            allowed_fields: Some(std::collections::BTreeSet::from([
                "identity".into(),
                "path".into(),
            ])),
        }),
        "field-denied",
    );
    assert!(matches!(
        engine.execute_query(
            &lease.session_id,
            &lease.token,
            &query("identity, path, source_file"),
            at + 3,
            "source-worker-source-file-query-denied",
            "source-worker-source-file-query-denied",
        ),
        Err(ServiceError::PermissionDenied)
    ));

    set_query_execute_policy(
        engine,
        lease,
        at + 4,
        Some(rrd_security::DataPolicy {
            tenant: None,
            rows: vec![rrd_security::PolicyPredicate {
                field: "identity".into(),
                value: rrd_core::RuntimeValue::String("0".repeat(64)),
            }],
            allowed_fields: Some(std::collections::BTreeSet::from([
                "identity".into(),
                "path".into(),
                "source_file".into(),
            ])),
        }),
        "row-denied",
    );
    let filtered = engine
        .execute_query(
            &lease.session_id,
            &lease.token,
            &query("identity, path, source_file"),
            at + 5,
            "source-worker-source-file-row-denied",
            "source-worker-source-file-row-denied",
        )
        .unwrap();
    assert!(filtered.rows.is_empty());
    set_query_execute_policy(engine, lease, at + 6, None, "restored");
}

fn source_worker_fixture(listener: &TcpListener, allow_source_export: bool) -> SourceWorkerFixture {
    source_worker_fixture_for_path(listener, allow_source_export, false, "alpha.txt")
}

fn source_worker_fixture_with_git(
    listener: &TcpListener,
    allow_source_export: bool,
    git_project: bool,
) -> SourceWorkerFixture {
    source_worker_fixture_for_path(
        listener,
        allow_source_export,
        git_project,
        if git_project {
            "nested/alpha.txt"
        } else {
            "alpha.txt"
        },
    )
}

fn source_worker_fixture_for_path(
    listener: &TcpListener,
    allow_source_export: bool,
    git_project: bool,
    source_path: &str,
) -> SourceWorkerFixture {
    source_worker_fixture_for_path_at(
        listener,
        allow_source_export,
        git_project,
        source_path,
        crate::engine::tests::attunement::BASE,
    )
}

fn source_worker_fixture_for_path_at(
    listener: &TcpListener,
    allow_source_export: bool,
    git_project: bool,
    source_path: &str,
    base: u64,
) -> SourceWorkerFixture {
    let (installed, engine, bootstrap_lease, job, _, _) = if git_project {
        first_inventory_for_worker_with_setup_at(
            |root| {
                std::fs::create_dir_all(root.join(".git")).unwrap();
                std::fs::write(
                    root.join(".git/config"),
                    b"[core]\nrepositoryFormatVersion=0\nignoreCase=false\n",
                )
                .unwrap();
                std::fs::write(
                root.join(".git/index"),
                include_bytes!("../../../../../../../compute/rrd-attunement/tests/fixtures/git-index/sha1-v2.index"),
            )
            .unwrap();
                std::fs::write(root.join(".gitignore"), b"*.log\nignored/\n").unwrap();
                std::fs::create_dir_all(root.join("nested")).unwrap();
                std::fs::write(root.join("nested/.gitignore"), b"*.tmp\n").unwrap();
                std::fs::write(root.join("nested/alpha.txt"), "source nested/alpha.txt\n").unwrap();
                std::fs::create_dir_all(root.join("ignored")).unwrap();
                std::fs::write(root.join("ignored/tracked.txt"), "tracked source\n").unwrap();
            },
            base,
        )
    } else {
        first_inventory_for_worker_with_setup_at(|_| {}, base)
    };
    let installed_record = engine.attunement_installed_record().unwrap();
    let project_id = installed_record.installation.target.project_id.clone();
    let accepted = crate::engine::attunement::load_accepted_source(
        &engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let source_content = std::fs::read_to_string(installed.root.path().join(source_path)).unwrap();
    let entry = accepted
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == source_path)
        .unwrap();
    let scope = format!("instance:{}", engine.instance_id());
    let selected_at = job.updated_at_unix_ms + 1;
    let snapshot_rows = engine
        .execute_query(
            &bootstrap_lease.session_id,
            &bootstrap_lease.token,
            &ExecuteQuery {
                scope: scope.clone(),
                query: format!(
                    "FROM record:source-tree-snapshot AT VALID {selected_at} KNOWN HEAD WHERE proposal_sha256 = \"{}\" PROJECT proposal_sha256, revisions",
                    accepted.proposal_sha256
                ),
                parameters: Default::default(),
                budget: QueryBudget::default(),
            },
            selected_at,
            "source-worker-public-snapshot-query",
            "source-worker-public-snapshot-query",
        )
        .unwrap();
    assert!(!snapshot_rows.execution.truncated);
    assert_eq!(snapshot_rows.rows.len(), 1);
    let snapshot = &snapshot_rows.rows[0];
    let queried_snapshot = match snapshot.values.get("proposal_sha256") {
        Some(QueryValue::String(value) | QueryValue::Digest(value)) => value,
        value => panic!("unexpected typed snapshot digest: {value:?}"),
    };
    assert_eq!(queried_snapshot, &accepted.proposal_sha256);
    let policy_sha256 = match snapshot.values.get("revisions") {
        Some(QueryValue::Map(values)) => match values.get("policy_sha256") {
            Some(QueryValue::String(value) | QueryValue::Digest(value)) => value.clone(),
            value => panic!("unexpected typed inventory policy digest: {value:?}"),
        },
        value => panic!("unexpected typed snapshot revisions: {value:?}"),
    };
    assert_eq!(policy_sha256.len(), 64);
    assert!(policy_sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let entry_rows = engine
        .execute_query(
            &bootstrap_lease.session_id,
            &bootstrap_lease.token,
            &ExecuteQuery {
                scope,
                query: format!(
                    "FROM record:source-tree-entry AT VALID {selected_at} KNOWN HEAD WHERE identity = \"{}\" PROJECT identity, path, source_file",
                    entry.identity
                ),
                parameters: Default::default(),
                budget: QueryBudget::default(),
            },
            selected_at,
            "source-worker-public-entry-query",
            "source-worker-public-entry-query",
        )
        .unwrap();
    assert!(!entry_rows.execution.truncated);
    assert_eq!(entry_rows.rows.len(), 1);
    let queried_entry = &entry_rows.rows[0];
    let entry_identity = match queried_entry.values.get("identity") {
        Some(QueryValue::String(value) | QueryValue::Digest(value)) => value.clone(),
        value => panic!("unexpected typed entry identity: {value:?}"),
    };
    assert_eq!(entry_identity, entry.identity);
    let queried_path = match queried_entry.values.get("path") {
        Some(QueryValue::String(value)) => value,
        value => panic!("unexpected typed entry path: {value:?}"),
    };
    assert_eq!(queried_path, source_path);
    let (content_sha256, queried_bytes) = match queried_entry.values.get("source_file") {
        Some(QueryValue::Map(values)) => match values.get("content_sha256") {
            Some(QueryValue::String(value) | QueryValue::Digest(value)) => {
                let bytes = match values.get("bytes") {
                    Some(QueryValue::Unsigned(bytes)) => *bytes,
                    value => panic!("unexpected typed file byte count: {value:?}"),
                };
                (value.clone(), bytes)
            }
            value => panic!("unexpected typed file content digest: {value:?}"),
        },
        value => panic!("unexpected typed source_file projection: {value:?}"),
    };
    assert_eq!(queried_bytes, source_content.len() as u64);
    assert_eq!(
        content_sha256,
        digest::sha256_hex(source_content.as_bytes())
    );
    let source = TaskWorkerSourceRequest {
        contract_version: rrd_contract::TASK_WORKER_SOURCE_VERSION,
        entry_identity,
        expected_content_sha256: content_sha256,
        maximum_input_bytes: 64 * 1024,
    };

    let repository =
        rrd_security::SecurityRepository::new(&engine.storage, engine.instance.clone());
    let mut authority = repository.load().unwrap().unwrap();
    let expected_revision = authority.revision;
    let bootstrap_principal = authority
        .principals
        .get(&installed.preview.installation.initial_principal_id)
        .unwrap();
    assert!(!bootstrap_principal
        .grants
        .iter()
        .any(|grant| grant.action == SecurityAction::ProjectSourceRead));
    let principal_id = CanonicalId::new("source-worker-principal").unwrap();
    let token = id("source-worker-principal-token");
    let mut principal = bootstrap_principal.clone();
    principal.id = principal_id.clone();
    principal.credential_sha256 = digest::sha256_hex(token.as_str().as_bytes());
    principal.grants.push(ResourceGrant {
        action: SecurityAction::ProjectSourceRead,
        resource_prefix: ResourcePath {
            segments: vec![
                ResourceId::new(ResourceKind::Instance, engine.instance.as_str()).unwrap(),
                ResourceId::new(ResourceKind::Project, project_id.as_str()).unwrap(),
                ResourceId::new(ResourceKind::Record, entry.identity.clone()).unwrap(),
            ],
        },
        data_policy: None,
    });
    authority.principals.insert(principal_id.clone(), principal);
    authority.revision += 1;
    let grant_at = host_unix_ms().max(job.updated_at_unix_ms + 1);
    engine
        .replace_security_authority(
            &bootstrap_lease.session_id,
            &bootstrap_lease.token,
            expected_revision,
            authority,
            grant_at,
            "source-worker-grant",
            "source-worker-grant",
        )
        .unwrap();
    let lease = engine
        .create_authenticated_session(
            &principal_id,
            token.as_str().as_bytes(),
            &crate::engine::tests::session_request(10_000, 2),
            &id("source-worker-principal-session"),
            grant_at + 1,
            "source-worker-principal-session",
            "source-worker-principal-session",
        )
        .unwrap();

    let scope = format!("instance:{}", engine.instance_id());
    let query_policy_at = grant_at + 2;
    verify_source_file_query_policy(&engine, &lease, &scope, &entry.identity, query_policy_at);

    let profile = proposal::configured().profile.unwrap();
    let selector = profile.selection_policy.unwrap();
    let candidate = selector.candidates[0].clone();
    let mut bindings = profile.bindings;
    for binding in &mut bindings {
        binding.endpoint = EndpointLocator::Url {
            url: format!("http://{}/v1", listener.local_addr().unwrap()),
        };
        binding.credential = None;
        binding.limits.timeout_ms = 5_000;
    }
    let input = EndpointProfileInput {
        contract_version: profile.contract_version,
        name: profile.name,
        bindings,
        selection_policy: Some(selector),
        observation_policy: None,
        worker_policy: Some(EndpointWorkerPolicy {
            contract_version: rrd_contract::ENDPOINT_WORKER_VERSION,
            allow_network: true,
            allow_context_export: true,
            allow_source_export,
            allow_unverified_identity: true,
            maximum_concurrent_calls: 1,
            bindings: vec![EndpointWorkerBinding {
                binding_id: candidate.binding_id,
                protocol_evidence_sha256: digest::sha256_hex(b"controlled source worker fixture"),
                system_prompt: "Return a bounded draft. Do not mutate the project.".into(),
                generation: EndpointGenerationSettings {
                    maximum_output_tokens: 256,
                    token_parameter: EndpointTokenParameter::MaxTokens,
                    temperature_milli: None,
                    response_format: EndpointResponseFormat::Text,
                },
                maximum_response_bytes: 16_384,
                effort_values: Default::default(),
                allow_default_effort: true,
            }],
        }),
    };
    let at = query_policy_at + 7;
    let plan = engine
        .preview_endpoint_configuration(
            &lease.session_id,
            &lease.token,
            &PreviewEndpointConfiguration {
                expected_revision: 0,
                profile: Some(input),
            },
            at,
            "source-worker-config-preview",
            "source-worker-config-preview",
        )
        .unwrap();
    let configuration = engine
        .apply_endpoint_configuration(
            &lease.session_id,
            &lease.token,
            &ApplyEndpointConfiguration {
                approved_plan_sha256: plan.plan_sha256.clone(),
                plan,
            },
            at + 1,
            "source-worker-config-apply",
            "source-worker-config-apply",
        )
        .unwrap();
    let scope = format!("instance:{}", engine.instance_id());
    let task_id = engine
        .open_task(
            &lease.session_id,
            &lease.token,
            &id("source-worker-open"),
            &OpenTask {
                scope: scope.clone(),
                title: "Inspect one selected file".into(),
                goal: "Return a bounded draft from selected source.".into(),
            },
            at + 2,
            "source-worker-open",
            "source-worker-open",
        )
        .unwrap()
        .task
        .task_id;
    let task = engine
        .read_task(
            &lease.session_id,
            &lease.token,
            &ReadTask {
                scope: scope.clone(),
                task_id: task_id.clone(),
                max_storage_keys: MAX_TASK_STORAGE_KEYS,
            },
            at + 3,
            "source-worker-read-task",
            "source-worker-read-task",
        )
        .unwrap();
    let selection = EndpointSelectionRequest {
        scope,
        task_id,
        attempt_id: id("source-worker-selection"),
        max_storage_keys: MAX_TASK_STORAGE_KEYS,
        expected_task_sha256: EndpointSelectionTask::from_task(&task.task)
            .unwrap()
            .task_sha256,
        expected_configuration_sha256: configuration.sha256().unwrap(),
    };
    SourceWorkerFixture {
        installed,
        engine,
        lease,
        job,
        principal_id,
        worker_token: token,
        selection,
        source,
        source_path: source_path.into(),
        source_content,
        now: at + 4,
    }
}

fn selected_request_host(
    engine: &RrdEngine,
    lease: &SessionLease,
    listener: &TcpListener,
    selection: &EndpointSelectionRequest,
) -> TaskWorkerRequest {
    let at = host_unix_ms();
    let configuration = engine
        .read_endpoint_configuration(&lease.session_id, &lease.token, at, "read", "read")
        .unwrap();
    let candidate = configuration
        .profile
        .unwrap()
        .selection_policy
        .unwrap()
        .candidates[0]
        .clone();
    let proposal = std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, body) = read_wire(listener);
            let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let choice = EndpointSelectionChoice::Select {
                binding_id: candidate.binding_id,
                effort: None,
            };
            reply(
                &mut socket,
                wire["model"].as_str().unwrap(),
                &serde_json::to_string(&choice).unwrap(),
            );
        });
        let result = engine.propose_task_endpoint(
            &lease.session_id,
            &lease.token,
            selection,
            "source-worker-host-selection",
            "source-worker-host-selection",
        );
        server.join().unwrap();
        result.unwrap()
    });
    TaskWorkerRequest {
        scope: proposal.scope.clone(),
        task_id: proposal.task_id.clone(),
        attempt_id: id("source-worker-host-attempt"),
        selection_attempt_id: proposal.attempt_id.clone(),
        expected_selection_sha256: operation_digest(&proposal).unwrap(),
        expected_task_sha256: proposal.task_sha256,
        expected_configuration_sha256: proposal.configuration_sha256,
        max_storage_keys: MAX_TASK_STORAGE_KEYS,
        context: None,
        source: None,
    }
}

fn refresh_inventory_without_source_edits(
    engine: &RrdEngine,
    lease: &SessionLease,
    job: &mut rrd_contract::AttunementJob,
    minimum_at: u64,
) {
    let preview_at = (job.updated_at_unix_ms + 1).max(minimum_at);
    let preview = engine
        .preview_attunement_replan(
            &lease.session_id,
            &lease.token,
            &job.id,
            job.revision,
            preview_at,
            "source-worker-legacy-replan-preview",
            "source-worker-legacy-replan-preview",
        )
        .unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &lease.session_id,
            &lease.token,
            &preview,
            &preview.proposal_sha256,
            &attunement_context("source-worker-legacy-replan-accept"),
            preview_at + 1,
        )
        .unwrap();
    let successor = &accepted.successor.job;
    let started = attunement_call(
        engine,
        lease,
        &AttunementCommand::Start {
            job_id: successor.id.clone(),
            plan_sha256: successor.plan_sha256.clone(),
            expected_revision: successor.revision,
            lease_ms: 30_000,
        },
        "source-worker-legacy-replan-start",
        (successor.updated_at_unix_ms + 1).max(preview_at + 2),
    )
    .unwrap();
    let connected = attunement_call(
        engine,
        lease,
        &attunement_step(&started.status.job),
        "source-worker-legacy-replan-connect",
        (started.status.job.updated_at_unix_ms + 1).max(preview_at + 3),
    )
    .unwrap();
    let inventory = attunement_call(
        engine,
        lease,
        &attunement_step(&connected.status.job),
        "source-worker-legacy-replan-inventory",
        (connected.status.job.updated_at_unix_ms + 1).max(preview_at + 4),
    )
    .unwrap();
    *job = inventory.status.job;
}

fn refresh_unrelated(
    root: &Path,
    engine: &RrdEngine,
    lease: &SessionLease,
    project_id: &CanonicalId,
    job: &mut rrd_contract::AttunementJob,
    name: &str,
) -> String {
    refresh_unrelated_with_host_floor(root, engine, lease, project_id, job, name, None)
}

fn refresh_unrelated_with_host_floor(
    root: &Path,
    engine: &RrdEngine,
    lease: &SessionLease,
    project_id: &CanonicalId,
    job: &mut rrd_contract::AttunementJob,
    name: &str,
    host_floor: Option<u64>,
) -> String {
    let next_at = |minimum: u64| host_floor.unwrap_or_default().max(minimum);
    std::fs::write(root.join(name), format!("unrelated edit {name}\n")).unwrap();
    let preview_at = next_at(job.updated_at_unix_ms + 1);
    let preview = engine
        .preview_attunement_replan(
            &lease.session_id,
            &lease.token,
            &job.id,
            job.revision,
            preview_at,
            &format!("source-worker-replan-preview-{name}"),
            &format!("source-worker-replan-preview-{name}"),
        )
        .unwrap();
    let accept_at = next_at(job.updated_at_unix_ms + 2);
    let accepted = engine
        .accept_attunement_replan(
            &lease.session_id,
            &lease.token,
            &preview,
            &preview.proposal_sha256,
            &attunement_context(&format!("source-worker-replan-accept-{name}")),
            accept_at,
        )
        .unwrap();
    let successor = &accepted.successor.job;
    let start_at = next_at(successor.updated_at_unix_ms + 1);
    let started = attunement_call(
        engine,
        lease,
        &AttunementCommand::Start {
            job_id: successor.id.clone(),
            plan_sha256: successor.plan_sha256.clone(),
            expected_revision: successor.revision,
            lease_ms: 30_000,
        },
        &format!("source-worker-replan-start-{name}"),
        start_at,
    )
    .unwrap();
    let connect_at = next_at(started.status.job.updated_at_unix_ms + 1);
    let connected = attunement_call(
        engine,
        lease,
        &attunement_step(&started.status.job),
        &format!("source-worker-replan-connect-{name}"),
        connect_at,
    )
    .unwrap();
    let inventory_at = next_at(connected.status.job.updated_at_unix_ms + 1);
    let refreshed = attunement_call(
        engine,
        lease,
        &attunement_step(&connected.status.job),
        &format!("source-worker-replan-inventory-{name}"),
        inventory_at,
    )
    .unwrap();
    *job = refreshed.status.job;
    crate::engine::attunement::load_accepted_source(engine, project_id, 8 * 1024 * 1024, 10_000)
        .unwrap()
        .proposal_sha256
}

fn revoke_source_read(
    engine: &RrdEngine,
    lease: &SessionLease,
    principal_id: &CanonicalId,
    at: u64,
) {
    let repository =
        rrd_security::SecurityRepository::new(&engine.storage, engine.instance.clone());
    let mut authority = repository.load().unwrap().unwrap();
    let expected_revision = authority.revision;
    let principal = authority.principals.get_mut(principal_id).unwrap();
    let prior = principal.grants.len();
    principal
        .grants
        .retain(|grant| grant.action != SecurityAction::ProjectSourceRead);
    assert!(principal.grants.len() < prior);
    authority.revision += 1;
    engine
        .replace_security_authority(
            &lease.session_id,
            &lease.token,
            expected_revision,
            authority,
            at,
            "source-worker-revoke",
            "source-worker-revoke",
        )
        .unwrap();
}

fn restore_source_read(
    engine: &RrdEngine,
    lease: &SessionLease,
    principal_id: &CanonicalId,
    project_id: &CanonicalId,
    entry_identity: &str,
    at: u64,
) {
    let repository =
        rrd_security::SecurityRepository::new(&engine.storage, engine.instance.clone());
    let mut authority = repository.load().unwrap().unwrap();
    let expected_revision = authority.revision;
    let principal = authority.principals.get_mut(principal_id).unwrap();
    assert!(!principal
        .grants
        .iter()
        .any(|grant| grant.action == SecurityAction::ProjectSourceRead));
    principal.grants.push(ResourceGrant {
        action: SecurityAction::ProjectSourceRead,
        resource_prefix: ResourcePath {
            segments: vec![
                ResourceId::new(ResourceKind::Instance, engine.instance.as_str()).unwrap(),
                ResourceId::new(ResourceKind::Project, project_id.as_str()).unwrap(),
                ResourceId::new(ResourceKind::Record, entry_identity.to_owned()).unwrap(),
            ],
        },
        data_policy: None,
    });
    authority.revision += 1;
    engine
        .replace_security_authority(
            &lease.session_id,
            &lease.token,
            expected_revision,
            authority,
            at,
            "source-worker-source-read-restore",
            "source-worker-source-read-restore",
        )
        .unwrap();
}

fn set_worker_export_flags(
    engine: &RrdEngine,
    lease: &SessionLease,
    at: u64,
    allow_context_export: bool,
    allow_source_export: bool,
    suffix: &str,
) {
    let current = engine.load_endpoint_configuration().unwrap().1;
    let profile = current.profile.unwrap();
    let mut worker_policy = profile.worker_policy.unwrap();
    worker_policy.allow_context_export = allow_context_export;
    worker_policy.allow_source_export = allow_source_export;
    let input = EndpointProfileInput {
        contract_version: profile.contract_version,
        name: profile.name,
        bindings: profile.bindings,
        selection_policy: profile.selection_policy,
        worker_policy: Some(worker_policy),
        observation_policy: profile.observation_policy,
    };
    let preview = engine
        .preview_endpoint_configuration(
            &lease.session_id,
            &lease.token,
            &PreviewEndpointConfiguration {
                expected_revision: current.revision,
                profile: Some(input),
            },
            at,
            &format!("source-worker-export-preview-{suffix}"),
            &format!("source-worker-export-preview-{suffix}"),
        )
        .unwrap();
    engine
        .apply_endpoint_configuration(
            &lease.session_id,
            &lease.token,
            &rrd_contract::ApplyEndpointConfiguration {
                approved_plan_sha256: preview.plan_sha256.clone(),
                plan: preview,
            },
            at + 1,
            &format!("source-worker-export-apply-{suffix}"),
            &format!("source-worker-export-apply-{suffix}"),
        )
        .unwrap();
}

fn attach_source_data_policy(fixture: &SourceWorkerFixture, at: u64) {
    let repository = rrd_security::SecurityRepository::new(
        &fixture.engine.storage,
        fixture.engine.instance.clone(),
    );
    let mut authority = repository.load().unwrap().unwrap();
    let expected_revision = authority.revision;
    let principal = authority.principals.get_mut(&fixture.principal_id).unwrap();
    let grant = principal
        .grants
        .iter_mut()
        .find(|grant| grant.action == SecurityAction::ProjectSourceRead)
        .unwrap();
    grant.data_policy = Some(rrd_security::DataPolicy {
        tenant: None,
        rows: Vec::new(),
        allowed_fields: Some(std::collections::BTreeSet::from(["content".into()])),
    });
    authority.revision += 1;
    fixture
        .engine
        .replace_security_authority(
            &fixture.lease.session_id,
            &fixture.lease.token,
            expected_revision,
            authority,
            at,
            "source-worker-data-policy",
            "source-worker-data-policy",
        )
        .unwrap();
}

fn inspect_budget(fixture: &SourceWorkerFixture, at: u64) -> TaskReasoningBudget {
    fixture
        .engine
        .inspect_task_reasoning_budget(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &fixture.selection.scope,
            &fixture.selection.task_id,
            at,
            "source-worker-budget-inspect",
            "source-worker-budget-inspect",
        )
        .unwrap()
        .unwrap()
}

#[test]
fn unrelated_accepted_head_advance_before_dispatch_keeps_selected_worker_source() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_fixture(&listener, true);
    let original_snapshot = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &fixture.installed.preview.installation.target.project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap()
    .proposal_sha256;
    let current_snapshot = refresh_unrelated(
        fixture.installed.root.path(),
        &fixture.engine,
        &fixture.lease,
        &fixture.installed.preview.installation.target.project_id,
        &mut fixture.job,
        "beta.txt",
    );
    fixture.now = fixture.job.updated_at_unix_ms + 1;
    assert_ne!(current_snapshot, original_snapshot);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let budget_before = inspect_budget(&fixture, fixture.now + 1);
    let (output, (body, _response)) = std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, body) = read_wire(&listener);
            let worker_wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                worker_packet_from_chat_wire(&worker_wire)["source"]["content"],
                fixture.source_content
            );
            assert_eq!(
                worker_packet_from_chat_wire(&worker_wire)["source"]["provenance"]
                    ["captured_snapshot_sha256"],
                current_snapshot
            );
            let response = reply(
                &mut socket,
                worker_wire["model"].as_str().unwrap(),
                "Reviewed one file.",
            );
            (body, response)
        });
        let output = fixture
            .engine
            .execute_task_worker_with_clock(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &request,
                "source-worker-execute",
                "source-worker-execute",
                &FixedClock::at(fixture.now + 1),
            )
            .unwrap();
        (output, server.join().unwrap())
    });
    let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        worker_packet_from_chat_wire(&wire)["source"]["content"],
        fixture.source_content
    );
    let provenance = output.source.as_ref().unwrap();
    assert_eq!(provenance.captured_snapshot_sha256, current_snapshot);
    assert_eq!(provenance.validated_snapshot_sha256, current_snapshot);
    assert_eq!(output.content, "Reviewed one file.");
    let budget_after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(
        budget_after.reserved_steps,
        budget_before.reserved_steps + 1
    );
    assert_eq!(
        budget_after.deadline_unix_ms,
        budget_before.deadline_unix_ms
    );
    assert!(!budget_after.execution_may_continue);
    assert!(budget_after.active_attempt.is_none());
    drop(fixture.engine);
    let reopened = fixture.installed.reopen(fixture.now + 3);
    let readback = reopened
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 3,
            "source-worker-readback",
            "source-worker-readback",
        )
        .unwrap();
    assert_eq!(readback.attempt.unwrap().worker_output.unwrap(), output);
    let reauthenticated = reopened
        .create_authenticated_session(
            &fixture.principal_id,
            fixture.worker_token.as_str().as_bytes(),
            &crate::engine::tests::session_request(10_000, 2),
            &id("source-worker-reauthenticated-session"),
            fixture.now + 4,
            "source-worker-reauthenticated-session",
            "source-worker-reauthenticated-session",
        )
        .unwrap();
    let reauthenticated_readback = reopened
        .read_task_attempt(
            &reauthenticated.session_id,
            &reauthenticated.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 5,
            "source-worker-readback-reauthenticated",
            "source-worker-readback-reauthenticated",
        )
        .unwrap();
    assert_eq!(
        reauthenticated_readback
            .attempt
            .unwrap()
            .worker_output
            .unwrap(),
        output
    );
    assert_eq!(
        std::fs::read_to_string(fixture.installed.root.path().join(&fixture.source_path)).unwrap(),
        fixture.source_content
    );
    std::fs::write(
        fixture.installed.root.path().join(&fixture.source_path),
        "later authorized source edit\n",
    )
    .unwrap();
    let historical = reopened
        .read_task_attempt(
            &reauthenticated.session_id,
            &reauthenticated.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 6,
            "source-worker-readback-after-content-change",
            "source-worker-readback-after-content-change",
        )
        .unwrap();
    let historical_output = historical.attempt.unwrap().worker_output.unwrap();
    assert_eq!(historical_output.content, "Reviewed one file.");
    assert_eq!(
        historical_output.source.unwrap().content_sha256,
        output.source.as_ref().unwrap().content_sha256
    );
    assert_ne!(
        digest::sha256_hex(b"later authorized source edit\n"),
        output.source.as_ref().unwrap().content_sha256
    );
    set_worker_export_flags(
        &reopened,
        &reauthenticated,
        fixture.now + 7,
        true,
        false,
        "source-disabled",
    );
    let source_export_hidden = reopened
        .read_task_attempt(
            &reauthenticated.session_id,
            &reauthenticated.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 8,
            "source-worker-readback-source-export-disabled",
            "source-worker-readback-source-export-disabled",
        )
        .unwrap();
    assert!(source_export_hidden
        .attempt
        .unwrap()
        .worker_output
        .is_none());
    set_worker_export_flags(
        &reopened,
        &reauthenticated,
        fixture.now + 9,
        false,
        true,
        "context-disabled",
    );
    let context_export_hidden = reopened
        .read_task_attempt(
            &reauthenticated.session_id,
            &reauthenticated.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 10,
            "source-worker-readback-context-export-disabled",
            "source-worker-readback-context-export-disabled",
        )
        .unwrap();
    assert!(context_export_hidden
        .attempt
        .unwrap()
        .worker_output
        .is_none());
    set_worker_export_flags(
        &reopened,
        &reauthenticated,
        fixture.now + 11,
        true,
        true,
        "exports-restored",
    );
    revoke_source_read(
        &reopened,
        &reauthenticated,
        &fixture.principal_id,
        fixture.now + 12,
    );
    let revoked_readback = reopened
        .read_task_attempt(
            &reauthenticated.session_id,
            &reauthenticated.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 13,
            "source-worker-readback-revoked",
            "source-worker-readback-revoked",
        )
        .unwrap();
    let revoked_attempt = revoked_readback.attempt.unwrap();
    assert_eq!(revoked_attempt.status, TaskAttemptStatus::Accepted);
    assert!(revoked_attempt.worker_output.is_none());
}

#[test]
fn unrelated_git_index_and_nonmatching_ignore_changes_keep_selected_worker_source() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_fixture_with_git(&listener, true, true);
    let original_snapshot = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &fixture.installed.preview.installation.target.project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap()
    .proposal_sha256;
    std::fs::write(
        fixture.installed.root.path().join(".git/index"),
        include_bytes!(
            "../../../../../../../compute/rrd-attunement/tests/fixtures/git-index/sha1-v3.index"
        ),
    )
    .unwrap();
    std::fs::remove_file(fixture.installed.root.path().join(".gitignore")).unwrap();
    std::fs::remove_file(fixture.installed.root.path().join("nested/.gitignore")).unwrap();
    let current_snapshot = refresh_unrelated(
        fixture.installed.root.path(),
        &fixture.engine,
        &fixture.lease,
        &fixture.installed.preview.installation.target.project_id,
        &mut fixture.job,
        "beta.txt",
    );
    fixture.now = fixture.job.updated_at_unix_ms + 1;
    assert_ne!(current_snapshot, original_snapshot);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let (output, (body, response)) = std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, body) = read_wire(&listener);
            let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                worker_packet_from_chat_wire(&wire)["source"]["content"],
                fixture.source_content
            );
            let response = reply(
                &mut socket,
                wire["model"].as_str().unwrap(),
                "Reviewed one file.",
            );
            (body, response)
        });
        let output = fixture
            .engine
            .execute_task_worker_with_clock(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &request,
                "source-worker-git-before",
                "source-worker-git-before",
                &FixedClock::at(fixture.now + 1),
            )
            .unwrap();
        (output, server.join().unwrap())
    });
    let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        worker_packet_from_chat_wire(&wire)["source"]["content"],
        fixture.source_content
    );
    assert_eq!(output.content, "Reviewed one file.");
    assert_eq!(
        output.source.unwrap().captured_snapshot_sha256,
        current_snapshot
    );
    assert!(!response.is_empty());
}

#[test]
fn persisted_legacy_inventory_reopens_and_eligible_replan_publishes_source_file() {
    let _legacy =
        crate::engine::attunement::LegacySourceFileProjectionGuard::omit_for_current_thread();
    let (installed, engine, lease, mut job, scope, _) =
        first_inventory_for_worker_with_setup_at(|_| {}, crate::engine::tests::attunement::BASE);
    let project_id = installed.preview.installation.target.project_id.clone();
    let legacy = crate::engine::attunement::load_accepted_source(
        &engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let source_entry = legacy
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == "alpha.txt")
        .unwrap()
        .clone();
    let source_sha256 = match &source_entry.kind {
        rrd_attunement::inventory::EntryKind::File { content_sha256, .. } => content_sha256.clone(),
        _ => panic!("fixture source entry is a regular file"),
    };
    let legacy_digest = legacy.proposal_sha256.clone();
    let entry_type = rrd_core::RuntimeType::new("source-tree-entry").unwrap();
    let legacy_schema = engine.storage.runtime().schema(&scope).unwrap().unwrap();
    assert!(!legacy_schema.records[&entry_type]
        .properties
        .contains_key("source_file"));
    let unavailable = engine.execute_query(
        &lease.session_id,
        &lease.token,
        &ExecuteQuery {
            scope: format!("instance:{}", engine.instance_id()),
            query: format!(
                "FROM record:source-tree-entry AT VALID {} KNOWN HEAD WHERE identity = \"{}\" PROJECT source_file",
                job.updated_at_unix_ms + 1,
                source_entry.identity
            ),
            parameters: Default::default(),
            budget: QueryBudget::default(),
        },
        job.updated_at_unix_ms + 2,
        "source-worker-legacy-file-unavailable",
        "source-worker-legacy-file-unavailable",
    );
    assert!(unavailable.is_err());

    drop(_legacy);
    drop(engine);
    let reopened = installed.reopen(job.updated_at_unix_ms + 3);
    let reopened_legacy = crate::engine::attunement::load_accepted_source(
        &reopened,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    assert_eq!(reopened_legacy.proposal_sha256, legacy_digest);
    assert_eq!(
        reopened_legacy.proposal.entries(),
        legacy.proposal.entries()
    );

    let replan_floor = job.updated_at_unix_ms + 3;
    refresh_inventory_without_source_edits(&reopened, &lease, &mut job, replan_floor);
    let refreshed = crate::engine::attunement::load_accepted_source(
        &reopened,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    let legacy_value = serde_json::to_value(&legacy.proposal).unwrap();
    let refreshed_value = serde_json::to_value(&refreshed.proposal).unwrap();
    let legacy_fields = legacy_value.as_object().unwrap();
    let refreshed_fields = refreshed_value.as_object().unwrap();
    let changed_fields = legacy_fields
        .iter()
        .filter_map(|(name, value)| {
            (refreshed_fields.get(name) != Some(value)).then_some(name.clone())
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        changed_fields,
        std::collections::BTreeSet::from([
            "previous_sha256".to_owned(),
            "changes".to_owned(),
            "proposal_sha256".to_owned(),
        ])
    );
    assert_eq!(legacy.proposal.entries(), refreshed.proposal.entries());
    assert_eq!(
        refreshed
            .proposal
            .entries()
            .iter()
            .find(|entry| entry.path == "alpha.txt")
            .unwrap()
            .identity,
        source_entry.identity
    );
    let refreshed_schema = reopened.storage.runtime().schema(&scope).unwrap().unwrap();
    assert!(refreshed_schema.records[&entry_type]
        .properties
        .contains_key("source_file"));
    let available = reopened
        .execute_query(
            &lease.session_id,
            &lease.token,
            &ExecuteQuery {
                scope: format!("instance:{}", reopened.instance_id()),
                query: format!(
                    "FROM record:source-tree-entry AT VALID {} KNOWN HEAD WHERE identity = \"{}\" PROJECT identity, path, source_file",
                    job.updated_at_unix_ms + 1,
                    source_entry.identity
                ),
                parameters: Default::default(),
                budget: QueryBudget::default(),
            },
            job.updated_at_unix_ms + 2,
            "source-worker-legacy-file-republished",
            "source-worker-legacy-file-republished",
        )
        .unwrap();
    assert_eq!(available.rows.len(), 1);
    let projection = match available.rows[0].values.get("source_file") {
        Some(QueryValue::Map(values)) => values,
        value => panic!("unexpected typed source_file projection: {value:?}"),
    };
    assert!(matches!(
        projection.get("content_sha256"),
        Some(QueryValue::String(value) | QueryValue::Digest(value)) if value == &source_sha256
    ));
    assert!(matches!(
        projection.get("bytes"),
        Some(QueryValue::Unsigned(value)) if *value == "source alpha.txt\n".len() as u64
    ));
}

#[test]
fn newly_applicable_ignore_rule_denies_source_before_provider_dispatch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let fixture = source_worker_fixture_with_git(&listener, true, true);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    std::fs::write(
        fixture.installed.root.path().join("nested/.gitignore"),
        b"alpha.txt\n",
    )
    .unwrap();
    let before = inspect_budget(&fixture, fixture.now + 1);
    assert!(matches!(
        fixture.engine.execute_task_worker_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request,
            "source-worker-git-excluded",
            "source-worker-git-excluded",
            &FixedClock::at(fixture.now + 1),
        ),
        Err(ServiceError::ProjectBindingMismatch)
    ));
    let after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(after.reserved_steps, before.reserved_steps);
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
}

#[test]
fn tracked_index_entry_overrides_ignore_for_selected_source() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let fixture = source_worker_fixture_for_path(&listener, true, true, "ignored/tracked.txt");
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let (output, body) = std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, body) = read_wire(&listener);
            let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                worker_packet_from_chat_wire(&wire)["source"]["content"],
                fixture.source_content
            );
            let response = reply(
                &mut socket,
                wire["model"].as_str().unwrap(),
                "Reviewed tracked file.",
            );
            (body, response)
        });
        let output = fixture
            .engine
            .execute_task_worker_with_clock(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &request,
                "source-worker-tracked-override",
                "source-worker-tracked-override",
                &FixedClock::at(fixture.now + 1),
            )
            .unwrap();
        (output, server.join().unwrap())
    });
    let wire: serde_json::Value = serde_json::from_slice(&body.0).unwrap();
    assert_eq!(
        worker_packet_from_chat_wire(&wire)["source"]["content"],
        fixture.source_content
    );
    assert_eq!(output.content, "Reviewed tracked file.");
    assert!(!body.1.is_empty());
}

#[test]
fn unrelated_git_stage_does_not_reset_source_repetition_ceiling() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = host_unix_ms().saturating_sub(30_000);
    let mut fixture =
        source_worker_fixture_for_path_at(&listener, true, true, "nested/alpha.txt", base);
    fixture
        .engine
        .configure_task_budget(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ConfigureTaskBudget {
                scope: fixture.selection.scope.clone(),
                task_id: fixture.selection.task_id.clone(),
                limits: fixture.engine.estate_configuration.reasoning.clone(),
                repetition_policy: Some(rrd_contract::TaskRepetitionPolicy {
                    max_equivalent_attempts: 1,
                }),
            },
            host_unix_ms(),
            "source-worker-repetition-configure",
            "source-worker-repetition-configure",
        )
        .unwrap();
    let mut request = selected_request_host(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
    );
    request.source = Some(fixture.source.clone());
    let (first_output, (body, _response)) = std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, body) = read_wire(&listener);
            let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                worker_packet_from_chat_wire(&wire)["source"]["content"],
                fixture.source_content
            );
            let response = reply(
                &mut socket,
                wire["model"].as_str().unwrap(),
                "One source draft.",
            );
            (body, response)
        });
        let output = fixture
            .engine
            .execute_task_worker(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &request,
                "source-worker-repetition-first",
                "source-worker-repetition-first",
            )
            .unwrap();
        (output, server.join().unwrap())
    });
    let first_wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(first_output.content, "One source draft.");
    assert!(!first_output
        .source
        .as_ref()
        .unwrap()
        .captured_snapshot_sha256
        .is_empty());
    assert!(!first_wire["model"].as_str().unwrap().is_empty());
    let budget_after_first = inspect_budget(&fixture, host_unix_ms());
    assert!(budget_after_first.reserved_steps > 0);
    assert!(budget_after_first.stopped.is_none());
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
    let original_entry = original
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap()
        .clone();
    std::fs::write(
        fixture.installed.root.path().join(".git/index"),
        include_bytes!(
            "../../../../../../../compute/rrd-attunement/tests/fixtures/git-index/sha1-v3.index"
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
    let current = refresh_unrelated_with_host_floor(
        fixture.installed.root.path(),
        &fixture.engine,
        &fixture.lease,
        &fixture.installed.preview.installation.target.project_id,
        &mut fixture.job,
        "beta.txt",
        Some(host_unix_ms()),
    );
    fixture.now = host_unix_ms().max(fixture.job.updated_at_unix_ms + 1);
    assert_ne!(current, original.proposal_sha256);
    let refreshed = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap();
    assert_eq!(current, refreshed.proposal_sha256);
    assert_ne!(
        original.proposal.revisions().policy_sha256,
        refreshed.proposal.revisions().policy_sha256
    );
    let refreshed_entry = refreshed
        .proposal
        .entries()
        .iter()
        .find(|entry| entry.path == fixture.source_path)
        .unwrap();
    assert_eq!(refreshed_entry.identity, original_entry.identity);
    assert_eq!(refreshed_entry.kind, original_entry.kind);
    assert_eq!(
        first_output.source.as_ref().unwrap().entry_identity,
        fixture.source.entry_identity
    );
    let mut equivalent = request.clone();
    equivalent.attempt_id = id("source-worker-repetition-second");
    let before_second = inspect_budget(&fixture, host_unix_ms());
    assert_eq!(
        before_second.reserved_steps,
        budget_after_first.reserved_steps
    );
    assert_eq!(
        before_second.charged_elapsed_ms,
        budget_after_first.charged_elapsed_ms
    );
    assert!(matches!(
        fixture.engine.execute_task_worker(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &equivalent,
            "source-worker-repetition-second",
            "source-worker-repetition-second",
        ),
        Err(ServiceError::ReasoningBudgetStopped(
            rrd_contract::ReasoningBudgetStop::RepeatedWork
        ))
    ));
    let after_second = inspect_budget(&fixture, host_unix_ms());
    assert_eq!(after_second.reserved_steps, before_second.reserved_steps);
    assert_eq!(
        after_second.charged_elapsed_ms,
        before_second.charged_elapsed_ms
    );
    assert_eq!(
        after_second.stopped,
        Some(rrd_contract::ReasoningBudgetStop::RepeatedWork)
    );
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn retained_source_observation_is_denied_on_exclusion_and_available_after_restore() {
    let worker = TcpListener::bind("127.0.0.1:0").unwrap();
    let sink = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_fixture_with_git(&worker, true, true);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &worker,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let (output, _) = complete_worker(
        &fixture.engine,
        &fixture.lease,
        &worker,
        &request,
        fixture.now + 1,
    );
    assert!(output.source.is_some());
    let read_attempt = ReadTaskAttempt {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
    };
    let retained = fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read_attempt,
            fixture.now + 2,
            "source-observation-retained-attempt",
            "source-observation-retained-attempt",
        )
        .unwrap();
    let retained_output = retained
        .attempt
        .as_ref()
        .unwrap()
        .worker_output
        .as_ref()
        .unwrap();
    assert_eq!(retained_output.content, output.content);
    assert!(retained_output.source.is_some());

    let credential = CredentialReference {
        provider: CanonicalId::new("test-process-registry").unwrap(),
        name: "source-observation-sink".into(),
    };
    fixture
        .engine
        .install_endpoint_credentials(
            crate::EndpointCredentials::new(vec![(
                credential.clone(),
                "controlled-observation-secret".into(),
            )])
            .unwrap(),
        )
        .unwrap();
    let current = fixture.engine.load_endpoint_configuration().unwrap().1;
    let profile = current.profile.unwrap();
    let mut bindings = profile.bindings;
    let mut binding = bindings[0].clone();
    binding.id = CanonicalId::new("source-observation-sink").unwrap();
    binding.class = rrd_contract::WorkloadClass::BehaviorUpdates;
    binding.kind = rrd_contract::BindingKind::Api;
    binding.placement = None;
    binding.endpoint = EndpointLocator::Url {
        url: format!("http://{}", sink.local_addr().unwrap()),
    };
    binding.credential = Some(credential);
    binding.limits.max_concurrency = 1;
    binding.limits.max_input_bytes = 16_384;
    let binding_id = binding.id.clone();
    bindings.push(binding);
    let input = EndpointProfileInput {
        contract_version: profile.contract_version,
        name: profile.name,
        bindings,
        selection_policy: profile.selection_policy,
        worker_policy: profile.worker_policy,
        observation_policy: Some(ObservationExportPolicy {
            contract_version: 1,
            binding_id,
            expected_devspace: "test-devspace".into(),
            protocol_evidence_sha256: digest::sha256_hex(b"controlled protocol"),
            allow_network: true,
            allow_metadata_export: true,
            allowed_scopes: vec![request.scope.clone()],
            max_retained_deliveries: 8,
            retention_ms: 60_000,
            retire_expired_uncertain: false,
            max_attempts: 1,
            retry_delay_ms: 1_000,
            max_response_bytes: 16_384,
        }),
    };
    let configured_at = fixture.now + 3;
    let plan = fixture
        .engine
        .preview_endpoint_configuration(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &PreviewEndpointConfiguration {
                expected_revision: current.revision,
                profile: Some(input),
            },
            configured_at,
            "source-observation-config-preview",
            "source-observation-config-preview",
        )
        .unwrap();
    let configuration = fixture
        .engine
        .apply_endpoint_configuration(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &rrd_contract::ApplyEndpointConfiguration {
                approved_plan_sha256: plan.plan_sha256.clone(),
                plan,
            },
            configured_at + 1,
            "source-observation-config-apply",
            "source-observation-config-apply",
        )
        .unwrap();
    let delivery_id = id("source-observation-retained");
    let prepared = fixture
        .engine
        .prepare_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &PrepareObservationDelivery {
                contract_version: 1,
                delivery_id: delivery_id.clone(),
                expected_configuration_sha256: configuration.sha256().unwrap(),
                source: ObservationSource::TaskAttempt {
                    request: read_attempt,
                    expected_status: rrd_contract::TaskAttemptStatus::Accepted,
                    expected_outcome_sha256: retained.attempt.unwrap().outcome_sha256,
                },
            },
            configured_at + 2,
            "source-observation-prepare",
            "source-observation-prepare",
        )
        .unwrap();
    let read = ReadObservationDelivery {
        contract_version: 1,
        delivery_id,
    };
    let before_exclusion = fixture
        .engine
        .read_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read,
            configured_at + 3,
            "source-observation-read-authorized",
            "source-observation-read-authorized",
        )
        .unwrap()
        .source_response_json
        .unwrap();
    let prepared_source: rrd_contract::TaskAttemptView =
        serde_json::from_str(&before_exclusion).unwrap();
    assert_eq!(
        prepared_source
            .attempt
            .unwrap()
            .worker_output
            .unwrap()
            .content,
        output.content
    );
    assert_eq!(prepared.delivery_id, read.delivery_id);

    set_worker_export_flags(
        &fixture.engine,
        &fixture.lease,
        configured_at + 7,
        true,
        false,
        "observation-source-export-revoked",
    );
    assert!(matches!(
        fixture.engine.read_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read,
            configured_at + 9,
            "source-observation-read-source-export-revoked",
            "source-observation-read-source-export-revoked",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    set_worker_export_flags(
        &fixture.engine,
        &fixture.lease,
        configured_at + 10,
        true,
        true,
        "observation-source-export-restored",
    );
    assert_eq!(
        fixture
            .engine
            .read_observation_delivery_at(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &read,
                configured_at + 12,
                "source-observation-read-source-export-restored",
                "source-observation-read-source-export-restored",
            )
            .unwrap()
            .source_response_json
            .as_deref(),
        Some(before_exclusion.as_str())
    );
    set_worker_export_flags(
        &fixture.engine,
        &fixture.lease,
        configured_at + 13,
        false,
        true,
        "observation-context-export-revoked",
    );
    assert!(matches!(
        fixture.engine.read_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read,
            configured_at + 15,
            "source-observation-read-context-export-revoked",
            "source-observation-read-context-export-revoked",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    set_worker_export_flags(
        &fixture.engine,
        &fixture.lease,
        configured_at + 16,
        true,
        true,
        "observation-context-export-restored",
    );
    assert_eq!(
        fixture
            .engine
            .read_observation_delivery_at(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &read,
                configured_at + 18,
                "source-observation-read-context-export-restored",
                "source-observation-read-context-export-restored",
            )
            .unwrap()
            .source_response_json
            .as_deref(),
        Some(before_exclusion.as_str())
    );
    let principal_id = fixture.principal_id.clone();
    let project_id = fixture
        .installed
        .preview
        .installation
        .target
        .project_id
        .clone();
    revoke_source_read(
        &fixture.engine,
        &fixture.lease,
        &principal_id,
        configured_at + 19,
    );
    assert!(matches!(
        fixture.engine.read_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read,
            configured_at + 20,
            "source-observation-read-grant-revoked",
            "source-observation-read-grant-revoked",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    restore_source_read(
        &fixture.engine,
        &fixture.lease,
        &principal_id,
        &project_id,
        &fixture.source.entry_identity,
        configured_at + 21,
    );
    assert_eq!(
        fixture
            .engine
            .read_observation_delivery_at(
                &fixture.lease.session_id,
                &fixture.lease.token,
                &read,
                configured_at + 22,
                "source-observation-read-grant-restored",
                "source-observation-read-grant-restored",
            )
            .unwrap()
            .source_response_json
            .as_deref(),
        Some(before_exclusion.as_str())
    );

    std::fs::write(
        fixture.installed.root.path().join("nested/.gitignore"),
        b"alpha.txt\n",
    )
    .unwrap();
    assert!(matches!(
        fixture.engine.read_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read,
            configured_at + 23,
            "source-observation-read-excluded",
            "source-observation-read-excluded",
        ),
        Err(ServiceError::PermissionDenied)
    ));

    std::fs::write(
        fixture.installed.root.path().join("nested/.gitignore"),
        b"*.tmp\n",
    )
    .unwrap();
    let restored = fixture
        .engine
        .read_observation_delivery_at(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &read,
            configured_at + 24,
            "source-observation-read-restored",
            "source-observation-read-restored",
        )
        .unwrap();
    assert_eq!(
        restored.source_response_json.as_deref(),
        Some(before_exclusion.as_str())
    );
    assert_eq!(restored.delivery.unwrap().delivery_id, prepared.delivery_id);
    sink.set_nonblocking(true).unwrap();
    assert!(matches!(sink.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock));
}

#[test]
fn unrelated_accepted_head_advance_during_dispatch_revalidates_without_redispatch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut fixture = source_worker_fixture_with_git(&listener, true, true);
    let captured_snapshot = crate::engine::attunement::load_accepted_source(
        &fixture.engine,
        &fixture.installed.preview.installation.target.project_id,
        8 * 1024 * 1024,
        10_000,
    )
    .unwrap()
    .proposal_sha256;
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let budget_before = inspect_budget(&fixture, fixture.now + 1);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (output, (worker_body, worker_response)) = std::thread::scope(|scope| {
        let source_content = fixture.source_content.clone();
        let server = scope.spawn(move || {
            let (mut socket, body) = read_wire(&listener);
            let worker_wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                worker_packet_from_chat_wire(&worker_wire)["source"]["content"],
                source_content
            );
            entered_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            let response = reply(
                &mut socket,
                worker_wire["model"].as_str().unwrap(),
                "Reviewed one file.",
            );
            (body, response)
        });
        let worker_engine = &fixture.engine;
        let worker_lease = &fixture.lease;
        let worker_now = fixture.now;
        let worker_request = &request;
        let worker = scope.spawn(move || {
            worker_engine.execute_task_worker_with_clock(
                &worker_lease.session_id,
                &worker_lease.token,
                worker_request,
                "source-worker-execute-during",
                "source-worker-execute-during",
                &FixedClock::at(worker_now + 1),
            )
        });
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        std::fs::write(
            fixture.installed.root.path().join(".git/index"),
            include_bytes!(
                "../../../../../../../compute/rrd-attunement/tests/fixtures/git-index/sha1-v3.index"
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
        let validated_snapshot = refresh_unrelated(
            fixture.installed.root.path(),
            &fixture.engine,
            &fixture.lease,
            &fixture.installed.preview.installation.target.project_id,
            &mut fixture.job,
            "gamma.txt",
        );
        fixture.now = fixture.job.updated_at_unix_ms + 1;
        assert_ne!(captured_snapshot, validated_snapshot);
        release_tx.send(()).unwrap();
        let output = worker.join().unwrap().unwrap();
        let server = server.join().unwrap();
        (output, server)
    });
    let wire: serde_json::Value = serde_json::from_slice(&worker_body).unwrap();
    assert_eq!(
        worker_packet_from_chat_wire(&wire)["source"]["content"],
        fixture.source_content
    );
    assert_eq!(output.content, "Reviewed one file.");
    let provenance = output.source.as_ref().unwrap();
    assert_eq!(provenance.captured_snapshot_sha256, captured_snapshot);
    assert_ne!(provenance.validated_snapshot_sha256, captured_snapshot);
    let budget_after = inspect_budget(&fixture, fixture.now + 2);
    assert_eq!(
        budget_after.reserved_steps,
        budget_before.reserved_steps + 1
    );
    assert_eq!(
        budget_after.deadline_unix_ms,
        budget_before.deadline_unix_ms
    );
    assert!(!budget_after.execution_may_continue);
    assert!(budget_after.active_attempt.is_none());
    let readback = fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 2,
            "source-worker-readback-during",
            "source-worker-readback-during",
        )
        .unwrap();
    assert_eq!(readback.attempt.unwrap().worker_output.unwrap(), output);
    assert!(!worker_response.is_empty());
    assert_eq!(
        std::fs::read_to_string(fixture.installed.root.path().join(&fixture.source_path)).unwrap(),
        fixture.source_content
    );
    std::fs::write(
        fixture.installed.root.path().join("nested/.gitignore"),
        b"alpha.txt\n",
    )
    .unwrap();
    let hidden = fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 3,
            "source-worker-readback-excluded",
            "source-worker-readback-excluded",
        )
        .unwrap();
    let hidden_attempt = hidden.attempt.unwrap();
    assert_eq!(hidden_attempt.status, TaskAttemptStatus::Accepted);
    assert!(hidden_attempt.worker_output.is_none());
    let (_, durable) = fixture
        .engine
        .read_task_attempt_record(&ReadTaskAttempt {
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            attempt_id: request.attempt_id.clone(),
        })
        .unwrap();
    assert!(durable.attempt.unwrap().worker_output.is_some());
}

#[test]
fn worker_source_export_is_denied_by_default_before_provider_dispatch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let fixture = source_worker_fixture(&listener, false);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let before = fixture
        .engine
        .inspect_task_reasoning_budget(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request.scope,
            &request.task_id,
            fixture.now + 1,
            "source-worker-budget-before-denial",
            "source-worker-budget-before-denial",
        )
        .unwrap();
    let result = fixture.engine.execute_task_worker_with_clock(
        &fixture.lease.session_id,
        &fixture.lease.token,
        &request,
        "source-worker-denied",
        "source-worker-denied",
        &FixedClock::at(fixture.now + 1),
    );
    assert!(matches!(result, Err(ServiceError::PermissionDenied)));
    let after = fixture
        .engine
        .inspect_task_reasoning_budget(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request.scope,
            &request.task_id,
            fixture.now + 2,
            "source-worker-budget-after-denial",
            "source-worker-budget-after-denial",
        )
        .unwrap();
    assert_eq!(after, before);
    let mut socket = None;
    listener.set_nonblocking(true).unwrap();
    match listener.accept() {
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
        Ok((accepted, _)) => socket = Some(accepted),
        Err(error) => panic!("unexpected listener error: {error}"),
    }
    assert!(
        socket.is_none(),
        "source denial must not make a provider request"
    );
}

#[test]
fn worker_denies_unenforceable_source_data_policy_without_provider_dispatch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let fixture = source_worker_fixture(&listener, true);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    attach_source_data_policy(&fixture, fixture.now + 1);
    assert!(matches!(
        fixture.engine.execute_task_worker_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request,
            "source-worker-policy-denied",
            "source-worker-policy-denied",
            &FixedClock::at(fixture.now + 2),
        ),
        Err(ServiceError::PermissionDenied)
    ));
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn changed_selected_file_is_rejected_before_dispatch_without_reserving_worker_attempt() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let fixture = source_worker_fixture(&listener, true);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let before = fixture
        .engine
        .inspect_task_reasoning_budget(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request.scope,
            &request.task_id,
            fixture.now + 1,
            "source-worker-drift-budget-before",
            "source-worker-drift-budget-before",
        )
        .unwrap();
    std::fs::write(
        fixture.installed.root.path().join(&fixture.source_path),
        "selected source changed\n",
    )
    .unwrap();
    assert!(matches!(
        fixture.engine.execute_task_worker_with_clock(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request,
            "source-worker-drift-before",
            "source-worker-drift-before",
            &FixedClock::at(fixture.now + 2),
        ),
        Err(ServiceError::ProjectBindingMismatch)
    ));
    let after = fixture
        .engine
        .inspect_task_reasoning_budget(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &request.scope,
            &request.task_id,
            fixture.now + 3,
            "source-worker-drift-budget-after",
            "source-worker-drift-budget-after",
        )
        .unwrap();
    assert_eq!(before, after);
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn changed_selected_file_during_dispatch_withholds_output_without_redispatch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let fixture = source_worker_fixture(&listener, true);
    let mut request = selected_request(
        &fixture.engine,
        &fixture.lease,
        &listener,
        &fixture.selection,
        fixture.now,
        false,
    );
    request.source = Some(fixture.source.clone());
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (result, (worker_body, response)) = std::thread::scope(|scope| {
        let source_content = fixture.source_content.clone();
        let server = scope.spawn(move || {
            let (mut socket, body) = read_wire(&listener);
            let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                worker_packet_from_chat_wire(&wire)["source"]["content"],
                source_content
            );
            entered_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            let response = reply(
                &mut socket,
                wire["model"].as_str().unwrap(),
                "Stale source draft.",
            );
            (body, response)
        });
        let worker_engine = &fixture.engine;
        let worker_lease = &fixture.lease;
        let worker_request = &request;
        let worker_now = fixture.now;
        let worker = scope.spawn(move || {
            worker_engine.execute_task_worker_with_clock(
                &worker_lease.session_id,
                &worker_lease.token,
                worker_request,
                "source-worker-drift-during",
                "source-worker-drift-during",
                &FixedClock::at(worker_now + 1),
            )
        });
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        std::fs::write(
            fixture.installed.root.path().join(&fixture.source_path),
            "selected source changed during request\n",
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
    assert!(!response.is_empty());
    let retained = fixture
        .engine
        .read_task_attempt(
            &fixture.lease.session_id,
            &fixture.lease.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            fixture.now + 2,
            "source-worker-drift-readback",
            "source-worker-drift-readback",
        )
        .unwrap();
    let retained = retained.attempt.unwrap();
    assert_eq!(retained.status, TaskAttemptStatus::Invalidated);
    assert!(retained.worker_output.is_none());
    let wire: serde_json::Value = serde_json::from_slice(&worker_body).unwrap();
    assert_eq!(
        worker_packet_from_chat_wire(&wire)["source"]["content"],
        fixture.source_content
    );
}
