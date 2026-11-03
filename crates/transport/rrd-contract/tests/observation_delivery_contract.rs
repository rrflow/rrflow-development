//! Protocol fixtures, not collected observations or training examples.
use rrd_contract::*;
use serde_json::json;

fn input() -> EndpointProfileInput {
    serde_json::from_value(json!({
        "contract_version":1,"name":"fixture-profile","bindings":[{
            "id":"fixture-sink","class":"behavior_updates","kind":"api",
            "endpoint":{"kind":"url","url":"https://sink.invalid"},
            "service_id":"fixture-service","manifest_sha256":null,
            "credential":{"provider":"environment","name":"FIXTURE_INGEST_KEY"},
            "placement":null,"footprint":{"memory_bytes":1024,"gpu":{"kind":"none"}},
            "limits":{"max_concurrency":1,"timeout_ms":1000,"max_input_bytes":16384}
        }],"observation_policy":{
            "contract_version":1,"binding_id":"fixture-sink","expected_devspace":"fixture-space",
            "protocol_evidence_sha256":"a".repeat(64),"allow_network":true,
            "allow_metadata_export":true,"allowed_scopes":["default"],
            "max_retained_deliveries":8,"retention_ms":60000,"max_attempts":3,
            "retry_delay_ms":1000,"max_response_bytes":4096
        }
    }))
    .unwrap()
}

#[test]
fn export_policy_changes_require_a_new_profile_identity() {
    let original = input();
    original.validate().unwrap();
    let profile = EndpointProfile::from_input(1, original.clone()).unwrap();
    let mut changed = original.clone();
    changed
        .observation_policy
        .as_mut()
        .unwrap()
        .expected_devspace = "another-space".into();
    assert_ne!(
        profile.profile_sha256,
        EndpointProfile::from_input(1, changed)
            .unwrap()
            .profile_sha256
    );
    let mut tampered = profile;
    tampered
        .observation_policy
        .as_mut()
        .unwrap()
        .allow_metadata_export = false;
    assert!(tampered.validate().is_err());
    let mut absent = original;
    absent.observation_policy = None;
    let serialized = serde_json::to_value(&absent).unwrap();
    assert!(serialized.get("observation_policy").is_none());
    let restored: EndpointProfileInput = serde_json::from_value(serialized).unwrap();
    assert!(restored.observation_policy.is_none());
    assert_eq!(
        EndpointProfile::from_input(1, absent).unwrap(),
        EndpointProfile::from_input(1, restored).unwrap()
    );
}

#[test]
fn export_policy_is_explicit_scoped_and_bounded() {
    let original = input();
    for (field, value) in [
        ("contract_version", json!(2)),
        ("binding_id", json!("absent")),
        ("expected_devspace", json!("")),
        ("protocol_evidence_sha256", json!("bad")),
        ("allowed_scopes", json!([])),
        ("allowed_scopes", json!(["default", "default"])),
        ("max_retained_deliveries", json!(129)),
        ("retention_ms", json!(0)),
        ("max_attempts", json!(0)),
        ("retry_delay_ms", json!(0)),
        ("max_response_bytes", json!(65537)),
    ] {
        let mut wire = serde_json::to_value(&original).unwrap();
        wire["observation_policy"][field] = value;
        let changed: EndpointProfileInput = serde_json::from_value(wire).unwrap();
        assert!(changed.validate().is_err(), "accepted {field}");
    }
    let mut wrong = original.clone();
    wrong.bindings[0].class = WorkloadClass::LargeModelServing;
    assert!(wrong.validate().is_err());
    let mut no_secret_reference = original;
    no_secret_reference.bindings[0].credential = None;
    assert!(no_secret_reference.validate().is_err());
}

#[test]
fn preparation_requires_known_source_and_exact_configuration() {
    let request = json!({"contract_version":1,"delivery_id":"fixture-delivery",
        "source":{"kind":"task_attempt","request":{"scope":"default","task_id":"fixture-task","attempt_id":"fixture-attempt"},"expected_status":"accepted","expected_outcome_sha256":"b".repeat(64)},
        "expected_configuration_sha256":"a".repeat(64)});
    serde_json::from_value::<PrepareObservationDelivery>(request.clone())
        .unwrap()
        .validate()
        .unwrap();
    let mut changed = request.clone();
    changed["expected_configuration_sha256"] = json!("bad");
    assert!(
        serde_json::from_value::<PrepareObservationDelivery>(changed)
            .unwrap()
            .validate()
            .is_err()
    );
    let mut active = request.clone();
    active["source"]["expected_status"] = json!("reserved");
    assert!(serde_json::from_value::<PrepareObservationDelivery>(active)
        .unwrap()
        .validate()
        .is_err());
    let mut invented = request;
    invented["source"]["content"] = json!("caller cannot upload arbitrary content");
    assert!(serde_json::from_value::<PrepareObservationDelivery>(invented).is_err());
}

#[test]
fn job_preconditions_use_durable_revision_not_a_readback_clock() {
    let source = json!({"kind":"attunement_job","request":{"contract_version":ATTUNEMENT_CONTRACT_VERSION,"job_id":"fixture-job"},
        "expected_revision":7,"expected_plan_sha256":"c".repeat(64)});
    serde_json::from_value::<ObservationSource>(source.clone())
        .unwrap()
        .validate()
        .unwrap();
    let mut invalid = source.clone();
    invalid["expected_revision"] = json!(0);
    assert!(serde_json::from_value::<ObservationSource>(invalid)
        .unwrap()
        .validate()
        .is_err());
    let mut caller_clock = source;
    caller_clock["observed_at_unix_ms"] = json!(1000);
    assert!(serde_json::from_value::<ObservationSource>(caller_clock).is_err());
}
