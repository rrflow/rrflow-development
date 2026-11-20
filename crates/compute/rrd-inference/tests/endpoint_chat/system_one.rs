//! Controlled protocol fixtures, not live model quality evidence.
use super::*;

fn answer(prepared: &PreparedEndpointController<'_>) -> Value {
    let request: Value = serde_json::from_slice(prepared.request_bytes()).unwrap();
    assert_eq!(request["questions"]["endpoint"]["type"], "choice");
    assert!(request.get("max_tokens").is_none());
    let criteria = request["questions"]["endpoint"]["criteria"]
        .as_object()
        .unwrap();
    let chosen = criteria
        .iter()
        .find(|(_, v)| v["kind"] == "select")
        .or_else(|| criteria.iter().next())
        .unwrap()
        .0;
    let probabilities: serde_json::Map<String, Value> = criteria
        .keys()
        .map(|key| {
            (
                key.clone(),
                serde_json::json!(if key == chosen { 1.0 } else { 0.0 }),
            )
        })
        .collect();
    serde_json::json!({"model":request["model"],"answers":{"endpoint":{
        "type":"choice","choice":chosen,"confidence":1.0,"probabilities":probabilities}},
        "usage":{"input_tokens":120,"output_tokens":12}})
}

#[test]
fn system_one_projects_only_allowed_pairs_and_maps_usage_without_fake_measurements() {
    let (configuration, allowance, _) = inputs("available");
    let binding = &configuration.profile.as_ref().unwrap().bindings[0].id;
    let prepared = PreparedEndpointController::system_one(
        &configuration,
        binding,
        &allowance,
        "Choose the suitable endpoint and effort, or abstain.",
        "task",
    )
    .unwrap();
    let response = answer(&prepared);
    let observed = prepared
        .decode_response(serde_json::to_vec(&response).unwrap())
        .unwrap();
    observed
        .choice
        .validate_for(&allowance, &configuration)
        .unwrap();
    let content = serde_json::to_vec(&observed.choice).unwrap();
    assert_eq!(
        observed.content_sha256,
        rrd_core::digest::sha256_hex(&content)
    );
    assert_eq!(observed.usage.unwrap().total_tokens, Some(132));
    assert_eq!(observed.elapsed_ms, None);
    assert_eq!(observed.http_status, None);
    let wire: Value = serde_json::from_slice(prepared.request_bytes()).unwrap();
    for criterion in wire["questions"]["endpoint"]["criteria"]
        .as_object()
        .unwrap()
        .values()
    {
        let choice: rrd_contract::EndpointSelectionChoice =
            serde_json::from_value(criterion.clone()).unwrap();
        choice.validate_for(&allowance, &configuration).unwrap();
    }
    assert!(!prepared
        .execute(NetworkPolicy::Deny, &control(), |_, _| panic!(
            "denied credential lookup"
        ))
        .unwrap_err()
        .dispatch_may_have_occurred());
}

#[test]
fn system_one_rejects_unknown_choices_invalid_distributions_and_unknown_effects() {
    let (configuration, allowance, _) = inputs("available");
    let binding = &configuration.profile.as_ref().unwrap().bindings[0].id;
    let prepared = PreparedEndpointController::system_one(
        &configuration,
        binding,
        &allowance,
        "Select",
        "task",
    )
    .unwrap();
    let original = answer(&prepared);
    let selected = original["answers"]["endpoint"]["choice"].as_str().unwrap();
    for (pointer, replacement) in [
        (
            "/answers/endpoint/type".to_owned(),
            serde_json::json!("noul"),
        ),
        (
            "/answers/endpoint/choice".to_owned(),
            serde_json::json!("foreign"),
        ),
        (
            "/answers/endpoint/confidence".to_owned(),
            serde_json::json!(1.1),
        ),
        (
            format!("/answers/endpoint/probabilities/{selected}"),
            serde_json::json!(0.0),
        ),
        (
            format!("/answers/endpoint/probabilities/{selected}"),
            serde_json::json!(-1.0),
        ),
        ("/model".to_owned(), serde_json::json!("different-model")),
        (
            "/usage/input_tokens".to_owned(),
            serde_json::json!(u64::MAX),
        ),
    ] {
        let mut response = original.clone();
        *response.pointer_mut(&pointer).unwrap() = replacement;
        assert!(
            prepared
                .decode_response(serde_json::to_vec(&response).unwrap())
                .is_err(),
            "{pointer}"
        );
    }
    let mut extra = original.clone();
    extra["answers"]["endpoint"]["probabilities"]["foreign"] = serde_json::json!(0.0);
    assert!(prepared
        .decode_response(serde_json::to_vec(&extra).unwrap())
        .is_err());
    let mut effect = original.clone();
    effect["tool_calls"] = serde_json::json!([]);
    assert!(prepared
        .decode_response(serde_json::to_vec(&effect).unwrap())
        .is_err());
    let raw = serde_json::to_string(&original).unwrap();
    let duplicate = raw.replace(
        &format!("\"{selected}\":1.0"),
        &format!("\"{selected}\":1.0,\"{selected}\":1.0"),
    );
    assert_ne!(raw, duplicate);
    assert!(prepared.decode_response(duplicate.into_bytes()).is_err());
    let mut narrowed = allowance.clone();
    narrowed.candidates.clear();
    let narrowed = PreparedEndpointController::system_one(
        &configuration,
        binding,
        &narrowed,
        "Select",
        "task",
    )
    .unwrap();
    assert!(narrowed
        .decode_response(serde_json::to_vec(&original).unwrap())
        .is_err());
}

#[test]
fn system_one_fails_before_dispatch_when_joint_options_exceed_protocol_limit() {
    let (configuration, mut allowance, _) = inputs("available");
    let binding = &configuration.profile.as_ref().unwrap().bindings[0].id;
    allowance.limits.maximum_effort_values = 255;
    let rrd_contract::EndpointEffortSupport::Observed {
        values,
        allow_default,
        ..
    } = &mut allowance.candidates[0].effort
    else {
        panic!("observed fixture")
    };
    *values = (0..254).map(|i| format!("effort-{i:03}")).collect();
    *allow_default = false;
    allowance.validate_for(&configuration).unwrap();
    let error = PreparedEndpointController::system_one(
        &configuration,
        binding,
        &allowance,
        "Select",
        "task",
    )
    .err()
    .unwrap();
    assert_eq!(error.reason, "System One choice limit exceeded");
    assert!(!error.dispatch_may_have_occurred());
}
