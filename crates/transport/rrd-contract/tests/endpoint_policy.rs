//! Configuration proposals derived from actual retained settings. Mutated
//! values are validation probes, not installed observations or training data.
use rrd_contract::{
    endpoint_profile_sha256, EndpointConfiguration, EndpointSelectionAllowance,
    EndpointSelectionPolicy,
};
use serde_json::Value;
use std::path::PathBuf;

fn bytes(file: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/endpoint-selection-observed/available")
            .join(file),
    )
    .unwrap()
}
fn proposed() -> (EndpointConfiguration, EndpointSelectionPolicy) {
    let configuration: EndpointConfiguration =
        serde_json::from_slice(&bytes("configuration.json")).unwrap();
    let allowance: EndpointSelectionAllowance =
        serde_json::from_slice(&bytes("allowance.json")).unwrap();
    let request: Value = serde_json::from_slice(&bytes("choice-request.json")).unwrap();
    let controller = configuration
        .profile
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|b| b.service_id.as_str() == request["model"].as_str().unwrap())
        .unwrap();
    let policy = EndpointSelectionPolicy {
        contract_version: allowance.contract_version,
        controller_binding_id: controller.id.clone(),
        workload_class: allowance.workload_class,
        limits: allowance.limits,
        candidates: allowance.candidates,
        system_prompt: request["messages"][0]["content"].as_str().unwrap().into(),
        maximum_output_tokens: request["max_tokens"].as_u64().unwrap().try_into().unwrap(),
        temperature_milli: (request["temperature"].as_f64().unwrap() * 1000.0) as u16,
    };
    (configuration, policy)
}

#[test]
fn policy_is_explicit_and_absence_preserves_actual_configuration_identity() {
    let (configuration, policy) = proposed();
    let profile = configuration.profile.as_ref().unwrap();
    assert!(profile.selection_policy.is_none());
    assert!(!serde_json::to_value(profile)
        .unwrap()
        .as_object()
        .unwrap()
        .contains_key("selection_policy"));
    policy.validate_for(&profile.bindings).unwrap();
    assert!(policy.allowance(&configuration).is_err());
    let receipt: Value = serde_json::from_slice(&bytes("receipt.json")).unwrap();
    assert_eq!(
        configuration.sha256().unwrap(),
        receipt["configurationSha256"]
    );
}

#[test]
fn proposed_policy_changes_digest_and_only_its_exact_configuration_can_project_it() {
    let (mut proposal, policy) = proposed();
    let previous = proposal.sha256().unwrap();
    let profile = proposal.profile.as_mut().unwrap();
    profile.selection_policy = Some(policy.clone());
    assert!(
        profile.validate().is_err(),
        "old digest cannot approve a new policy"
    );
    profile.profile_sha256 = endpoint_profile_sha256(profile).unwrap();
    proposal.validate().unwrap();
    let allowance = policy.allowance(&proposal).unwrap();
    assert_ne!(previous, allowance.configuration_sha256);
    assert_eq!(allowance.configuration_sha256, proposal.sha256().unwrap());
    let mut output_change = policy.clone();
    output_change.maximum_output_tokens += 1;
    let mut prompt_change = policy.clone();
    prompt_change.system_prompt.push('\n');
    let mut candidate_change = policy.clone();
    candidate_change.candidates.clear();
    let mut limit_change = policy.clone();
    limit_change.limits.maximum_response_bytes -= 1;
    for changed in [output_change, prompt_change, candidate_change, limit_change] {
        assert!(changed.allowance(&proposal).is_err());
        let mut profile = proposal.profile.clone().unwrap();
        profile.selection_policy = Some(changed);
        assert!(profile.validate().is_err());
        assert_ne!(
            endpoint_profile_sha256(&profile).unwrap(),
            profile.profile_sha256
        );
    }
}

#[test]
fn unknown_bindings_mismatched_facts_and_invalid_bounds_are_rejected() {
    let (configuration, policy) = proposed();
    let bindings = &configuration.profile.as_ref().unwrap().bindings;
    let mut changed = policy.clone();
    changed.controller_binding_id = bindings[0].service_id.clone();
    assert!(changed.validate_for(bindings).is_err());
    let mut changed = policy.clone();
    changed.candidates[0].service_id = bindings[0].id.clone();
    assert!(changed.validate_for(bindings).is_err());
    let mut changed = policy.clone();
    changed.maximum_output_tokens = 0;
    assert!(changed.validate_for(bindings).is_err());
    let mut changed = policy.clone();
    changed.temperature_milli = u16::MAX;
    assert!(changed.validate_for(bindings).is_err());
    let mut changed = policy.clone();
    changed.system_prompt = policy
        .system_prompt
        .repeat((bindings[0].limits.max_input_bytes as usize / policy.system_prompt.len()) + 1);
    assert!(changed.validate_for(bindings).is_err());
    let mut changed = policy;
    changed.system_prompt.clear();
    assert!(changed.validate_for(bindings).is_err());
}

#[test]
fn policy_decoding_is_closed_and_empty_candidates_remain_explicit() {
    let (configuration, mut policy) = proposed();
    let mut value = serde_json::to_value(&policy).unwrap();
    value["authorization_granted"] = Value::Bool(true);
    assert!(serde_json::from_value::<EndpointSelectionPolicy>(value).is_err());
    policy.candidates.clear();
    policy
        .validate_for(&configuration.profile.unwrap().bindings)
        .unwrap();
}
