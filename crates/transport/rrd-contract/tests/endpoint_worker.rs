//! Explicit configuration perturbations, never provider qualification records.
use rrd_contract::*;
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/endpoint-selection-observed/available")
            .join(name),
    )
    .unwrap()
}

fn proposed() -> (EndpointConfiguration, EndpointWorkerPolicy) {
    let mut configuration: EndpointConfiguration =
        serde_json::from_slice(&fixture("configuration.json")).unwrap();
    let allowance: EndpointSelectionAllowance =
        serde_json::from_slice(&fixture("allowance.json")).unwrap();
    let request: Value = serde_json::from_slice(&fixture("choice-request.json")).unwrap();
    let profile = configuration.profile.as_mut().unwrap();
    let controller = profile
        .bindings
        .iter()
        .find(|binding| binding.service_id.as_str() == request["model"].as_str().unwrap())
        .unwrap();
    let candidate = &allowance.candidates[0];
    let EndpointEffortSupport::Observed {
        evidence_sha256,
        allow_default,
        values,
    } = &candidate.effort
    else {
        panic!("fixture needs observed effort")
    };
    let worker = EndpointWorkerPolicy {
        contract_version: ENDPOINT_WORKER_VERSION,
        allow_network: true,
        allow_context_export: true,
        allow_unverified_identity: true,
        maximum_concurrent_calls: 1,
        bindings: vec![EndpointWorkerBinding {
            binding_id: candidate.binding_id.clone(),
            // Digest-shaped protocol probe, not evidence of worker compatibility.
            protocol_evidence_sha256: evidence_sha256.clone(),
            system_prompt: "Return a draft; do not claim project effects.".into(),
            generation: EndpointGenerationSettings {
                maximum_output_tokens: 256,
                token_parameter: EndpointTokenParameter::MaxCompletionTokens,
                temperature_milli: None,
                response_format: EndpointResponseFormat::Text,
            },
            maximum_response_bytes: 16_384,
            effort_values: values
                .iter()
                .map(|value| (value.clone(), value.clone()))
                .collect::<BTreeMap<_, _>>(),
            allow_default_effort: *allow_default,
        }],
    };
    profile.selection_policy = Some(EndpointSelectionPolicy {
        contract_version: allowance.contract_version,
        purpose: EndpointSelectionPurpose::ExperimentalProposal,
        allow_unverified_identity: true,
        allow_network: true,
        maximum_concurrent_proposals: controller.limits.max_concurrency,
        controller_binding_id: controller.id.clone(),
        workload_class: allowance.workload_class,
        limits: allowance.limits,
        candidates: allowance.candidates,
        system_prompt: request["messages"][0]["content"].as_str().unwrap().into(),
        maximum_output_tokens: request["max_tokens"].as_u64().unwrap().try_into().unwrap(),
        temperature_milli: (request["temperature"].as_f64().unwrap() * 1000.0) as u16,
    });
    profile.profile_sha256 = endpoint_profile_sha256(profile).unwrap();
    configuration.validate().unwrap();
    (configuration, worker)
}

#[test]
fn worker_absence_preserves_identity_and_present_settings_require_a_new_approval() {
    let original: EndpointConfiguration =
        serde_json::from_slice(&fixture("configuration.json")).unwrap();
    let receipt: Value = serde_json::from_slice(&fixture("receipt.json")).unwrap();
    assert_eq!(original.sha256().unwrap(), receipt["configurationSha256"]);
    assert!(original.profile.as_ref().unwrap().worker_policy.is_none());
    assert!(serde_json::to_value(original).unwrap()["profile"]
        .get("worker_policy")
        .is_none());
    let (mut configuration, worker) = proposed();
    let before = configuration.sha256().unwrap();
    let profile = configuration.profile.as_mut().unwrap();
    profile.worker_policy = Some(worker);
    assert!(profile.validate().is_err());
    profile.profile_sha256 = endpoint_profile_sha256(profile).unwrap();
    configuration.validate().unwrap();
    assert_ne!(configuration.sha256().unwrap(), before);
    let profile = configuration.profile.as_mut().unwrap();
    profile.worker_policy.as_mut().unwrap().bindings[0]
        .generation
        .maximum_output_tokens += 1;
    assert!(profile.validate().is_err());
}

#[test]
fn worker_binding_settings_and_effort_cannot_invent_selector_support() {
    let (configuration, policy) = proposed();
    let profile = configuration.profile.unwrap();
    policy
        .validate_for(&profile.bindings, profile.selection_policy.as_ref())
        .unwrap();
    assert!(policy.validate_for(&profile.bindings, None).is_err());
    assert!(policy.bindings[0]
        .wire_effort(Some("unobserved-effort"))
        .is_err());
    let mut changed = policy.clone();
    changed.bindings[0]
        .effort_values
        .insert("unobserved-effort".into(), "medium".into());
    assert!(changed
        .validate_for(&profile.bindings, profile.selection_policy.as_ref())
        .is_err());
    for field in ["contract_version", "maximum_concurrent_calls"] {
        let mut value = serde_json::to_value(&policy).unwrap();
        value[field] = 0.into();
        let changed: EndpointWorkerPolicy = serde_json::from_value(value).unwrap();
        assert!(changed
            .validate_for(&profile.bindings, profile.selection_policy.as_ref())
            .is_err());
    }
    let mut changed = policy.clone();
    changed.bindings.push(changed.bindings[0].clone());
    assert!(changed
        .validate_for(&profile.bindings, profile.selection_policy.as_ref())
        .is_err());
    let mut changed = policy.clone();
    changed.bindings[0].maximum_response_bytes = 0;
    assert!(changed
        .validate_for(&profile.bindings, profile.selection_policy.as_ref())
        .is_err());
    let mut value = serde_json::to_value(policy).unwrap();
    value["authorize_project_write"] = true.into();
    assert!(serde_json::from_value::<EndpointWorkerPolicy>(value).is_err());
}

#[test]
fn configured_effort_mapping_is_exact_and_default_is_never_inferred() {
    let (_, policy) = proposed();
    let mut binding = policy.bindings[0].clone();
    binding.allow_default_effort = false;
    binding.effort_values = BTreeMap::from([("selected".into(), "wire-value".into())]);
    assert_eq!(
        binding.wire_effort(Some("selected")).unwrap(),
        Some("wire-value")
    );
    assert!(binding.wire_effort(None).is_err());
    assert!(binding.wire_effort(Some("wire-value")).is_err());
    binding.allow_default_effort = true;
    assert_eq!(binding.wire_effort(None).unwrap(), None);
}

#[test]
fn worker_operation_has_explicit_authority_and_closed_input_identity() {
    let catalogue = endpoint_catalogue();
    let operation = catalogue
        .endpoints
        .iter()
        .find(|endpoint| endpoint.operation.as_str() == "task-worker-execute")
        .unwrap();
    assert_eq!(operation.path, "/v1/tasks/worker/execute");
    assert!(operation.mutation);
    assert_eq!(
        operation.action.fixed_action(),
        Some(SecurityAction::WorkerExecute)
    );
    let request = TaskWorkerRequest {
        scope: "instance:worker-test".into(),
        task_id: CanonicalId::new("worker-test-task").unwrap(),
        attempt_id: CorrelationId::new("worker-test-attempt").unwrap(),
        selection_attempt_id: CorrelationId::new("worker-test-selection").unwrap(),
        expected_selection_sha256: "1".repeat(64),
        expected_task_sha256: "2".repeat(64),
        expected_configuration_sha256: "3".repeat(64),
        max_storage_keys: 10_000,
        context: None,
    };
    request.validate().unwrap();
    let mut value = serde_json::to_value(&request).unwrap();
    value["endpoint"] = "http://127.0.0.1:1".into();
    assert!(serde_json::from_value::<TaskWorkerRequest>(value).is_err());
    let mut same_attempt = request.clone();
    same_attempt.selection_attempt_id = same_attempt.attempt_id.clone();
    assert!(same_attempt.validate().is_err());
    let mut foreign = request;
    foreign.context = Some(AssembleContext {
        scope: "instance:foreign".into(),
        query: "test".into(),
        valid_at: 1,
        seeds: Vec::new(),
        max_graph_depth: 0,
        max_items: 1,
        max_output_bytes: 1024,
        max_storage_keys: 100,
    });
    assert!(foreign.validate().is_err());
}
