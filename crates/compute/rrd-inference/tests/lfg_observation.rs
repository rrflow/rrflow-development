use rrd_inference::lfg_observation::{decode_lfg_observation, ExpectedObservation};
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/lfg-synthetic-observation.json")).unwrap()
}

fn decode(
    fixture: &Value,
    maximum_response_bytes: usize,
) -> Result<rrd_inference::lfg_observation::LfgObservation, &'static str> {
    let request = &fixture["request"];
    let expected = ExpectedObservation {
        request_id: request["requestId"].as_str().unwrap(),
        revision: request["revision"].as_u64().unwrap(),
        input_sha256: request["inputSha256"].as_str().unwrap(),
        assessment_sha256: request["expectedAssessmentSha256"].as_str().unwrap(),
        rules_sha256: request["expectedRulesSha256"].as_str().unwrap(),
        maximum_response_bytes,
    };
    decode_lfg_observation(
        &serde_json::to_vec(&fixture["observation"]).unwrap(),
        &expected,
    )
}

#[test]
fn captured_lfg_response_preserves_partial_and_unknown_semantics() {
    let mut value = fixture();
    let observation = decode(&value, 65_536).unwrap();
    assert_eq!(observation.schema_version, "lfg.observation.v1");
    assert_eq!(observation.intent.status, "inconclusive");
    assert!(observation.advisory);
    assert_eq!(observation.confidence, None);
    value["observation"]["status"] = json!("partial");
    value["observation"]["unknownFutureField"] = json!({"private": "not forwarded"});
    let projected = decode(&value, 65_536).unwrap();
    assert_eq!(projected.status, "partial");
    assert!(serde_json::to_value(projected)
        .unwrap()
        .get("unknownFutureField")
        .is_none());
}

#[test]
fn rejects_stale_identity_unbounded_or_unsupported_observations() {
    let original = fixture();
    assert!(decode(&original, 10).is_err());
    for (field, replacement) in [
        ("requestId", json!("other-request")),
        ("revision", json!(2)),
        ("inputSha256", json!("a".repeat(64))),
        ("assessmentSha256", json!("a".repeat(64))),
        ("rulesSha256", json!("a".repeat(64))),
        ("schemaVersion", json!("lfg.observation.v2")),
        ("advisory", json!(false)),
        ("confidence", json!(0.9)),
    ] {
        let mut changed = original.clone();
        changed["observation"][field] = replacement;
        assert!(decode(&changed, 65_536).is_err(), "field {field}");
    }
    let mut changed = original.clone();
    changed["observation"]["agents"]["min"] = json!(2);
    assert!(decode(&changed, 65_536).is_err());
    let mut changed = original.clone();
    changed["observation"]["complexity"]["scoreUnit"] = json!("unrecognized");
    assert!(decode(&changed, 65_536).is_err());
}
