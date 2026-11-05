use rrd_contract::{CanonicalId, CorrelationId, EndpointSelectionFlightIdentity, EndpointSelectionRequest};

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
    };
    assert!(request.validate().is_err());
    request.flight.as_mut().unwrap().context_request_sha256 = Some(digest('4'));
    request.validate().unwrap();
}
