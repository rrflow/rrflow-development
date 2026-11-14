//! Actual retained worker receipt; mutations below are rejection probes, not observations.
use rrd_contract::{TaskWorkerContext, TaskWorkerOutput, TaskWorkerRequest};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn captured() -> (TaskWorkerRequest, TaskWorkerOutput) {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/worker-observed");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let read = |name: &str| {
        let bytes = std::fs::read(root.join(name)).unwrap();
        let digest: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(digest, manifest["files"][name].as_str().unwrap());
        bytes
    };
    let request = serde_json::from_slice(&read("worker-request.json")).unwrap();
    let response: Value = serde_json::from_slice(&read("worker-response.json")).unwrap();
    assert_eq!(response["status"], 200);
    let output =
        serde_json::from_value(response["response"]["outcome"]["payload"].clone()).unwrap();
    (request, output)
}

#[test]
fn captured_worker_response_binds_every_request_known_identity() {
    let (request, response) = captured();
    response.validate_request_identity(&request).unwrap();
    for field in [
        "scope",
        "task_id",
        "attempt_id",
        "selection_attempt_id",
        "selection_sha256",
        "task_sha256",
        "configuration_sha256",
    ] {
        let mut changed = serde_json::to_value(&response).unwrap();
        changed[field] = if field.ends_with("sha256") {
            let original = changed[field].as_str().unwrap();
            format!(
                "{}{}",
                if original.starts_with('0') { '1' } else { '0' },
                &original[1..]
            )
            .into()
        } else if field == "scope" {
            "instance:identity-rejection-probe".into()
        } else {
            "identity-rejection-probe".into()
        };
        let changed: TaskWorkerOutput = serde_json::from_value(changed).unwrap();
        changed.validate().unwrap();
        assert!(
            changed.validate_request_identity(&request).is_err(),
            "{field}"
        );
    }
    let mut malformed = response.clone();
    malformed.content.push('!');
    assert!(malformed.validate_request_identity(&request).is_err());
    let mut invalid_request = request.clone();
    invalid_request.attempt_id = invalid_request.selection_attempt_id.clone();
    assert!(response
        .validate_request_identity(&invalid_request)
        .is_err());
}

#[test]
fn unsolicited_context_provenance_is_rejected() {
    let (request, mut response) = captured();
    assert!(request.context.is_none() && response.context.is_none());
    // Reuse observed digest/read fields only to construct an explicit invalid response.
    response.context = Some(TaskWorkerContext {
        request_sha256: response.request_sha256.clone(),
        packet_sha256: response.packet_sha256.clone(),
        read: response.read.clone(),
    });
    response.validate().unwrap();
    assert!(response.validate_request_identity(&request).is_err());
}
