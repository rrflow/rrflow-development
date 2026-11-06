use rrd_contract::{
    CanonicalId, CorrelationId, EndpointSelectionFlightIdentity, EndpointSelectionRequest,
    EndpointSelectionRoutingInput,
};

fn id(value: &str) -> CanonicalId {
    CanonicalId::new(value).unwrap()
}

fn digest(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

#[test]
fn selection_request_allows_unknown_fields_for_forward_extensions() {
    let value = serde_json::json!({
        "scope": "instance:test-instance",
        "task_id": "task",
        "attempt_id": "attempt",
        "max_storage_keys": 4096,
        "expected_task_sha256": digest('1'),
        "expected_configuration_sha256": digest('2'),
        "future_extension": {
            "controller_hint": "latency_floor"
        }
    });
    let request: EndpointSelectionRequest = serde_json::from_value(value).unwrap();
    assert!(request.flight.is_none());
    assert!(request.routing.is_none());
    request.validate().unwrap();
}

#[test]
fn selection_request_validates_optional_flight_identity_digests() {
    let mut request = EndpointSelectionRequest {
        scope: "instance:test-instance".into(),
        task_id: id("task"),
        attempt_id: CorrelationId::new("attempt").unwrap(),
        max_storage_keys: 4096,
        expected_task_sha256: digest('1'),
        expected_configuration_sha256: digest('2'),
        flight: Some(EndpointSelectionFlightIdentity {
            flight_id: id("flight"),
            prompt_cohort_sha256: digest('3'),
            context_request_sha256: Some("bad".into()),
        }),
        routing: None,
    };
    assert!(request.validate().is_err());
    request.flight.as_mut().unwrap().context_request_sha256 = Some(digest('4'));
    request.validate().unwrap();
}

#[test]
fn routing_input_narrows_allowance_to_ranked_subset() {
    let allowance: rrd_contract::EndpointSelectionAllowance = serde_json::from_slice(
        &std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/endpoint-selection-observed/available/allowance.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let selected = allowance.candidates[0].binding_id.as_str().to_owned();
    let routing: EndpointSelectionRoutingInput = serde_json::from_value(serde_json::json!({
        "context_extraction": {
            "modality": "text",
            "signals": [
                {"name": "intent", "value": "search"}
            ]
        },
        "semantic_router": {
            "policy_id": "semantic-router-v1",
            "ranked_candidates": [
                {"binding_id": selected, "score_milli": 900}
            ],
            "narrowed_candidate_count": 1
        }
    }))
    .unwrap();
    let narrowed = routing.narrow_allowance(&allowance).unwrap();
    assert_eq!(narrowed.candidates.len(), 1);
    assert_eq!(
        narrowed.candidates[0].binding_id.as_str(),
        allowance.candidates[0].binding_id.as_str()
    );
}
