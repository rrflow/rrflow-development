//! Retained evidence stays byte-for-byte recorded. Current-contract test inputs
//! are explicitly derived and do not inherit the captured authorization receipt.

use rrd_contract::{CanonicalId, EndpointProfile};
use rrflow_kernel::digest::sha256_hex;
use serde::{Deserialize, Serialize};
use std::path::Path;

// Immutable test evidence only, never a runtime compatibility API.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalIdentity {
    project_id: CanonicalId,
    estate_id: CanonicalId,
    instance_id: CanonicalId,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalConfiguration {
    format_version: u16,
    identity: HistoricalIdentity,
    revision: u64,
    profile: Option<EndpointProfile>,
    last_applied_plan_sha256: Option<String>,
}

/// Derive an offline test input; this alone does not attest a capture.
pub fn project(bytes: &[u8]) -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let identity = value["identity"].as_object_mut().unwrap();
    let recorded = identity.remove("estate_id").expect("recorded identity");
    assert!(identity.insert("engram_id".into(), recorded).is_none());
    serde_json::to_vec(&value).unwrap()
}

/// Original archived assertions: verify both file bytes and canonical identity.
pub fn project_verified(bytes: &[u8], file_digest: &str, captured_digest: &str) -> Vec<u8> {
    assert_eq!(sha256_hex(bytes), file_digest, "captured file digest");
    let historical: HistoricalConfiguration = serde_json::from_slice(bytes).unwrap();
    assert_eq!(
        sha256_hex(&serde_json::to_vec(&historical).unwrap()),
        captured_digest,
        "historical canonical configuration digest"
    );
    let projected = project(bytes);
    let configuration: rrd_contract::EndpointConfiguration =
        serde_json::from_slice(&projected).unwrap();
    configuration.validate().unwrap();
    assert_ne!(configuration.sha256().unwrap(), captured_digest);
    projected
}

/// Read original bytes and validate their recorded file digest before use.
pub fn read_recorded(directory: &Path, file: &str) -> Vec<u8> {
    let bytes = std::fs::read(directory.join(file)).unwrap();
    if file != "files.json" {
        let files: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("files.json")).unwrap()).unwrap();
        assert_eq!(
            sha256_hex(&bytes),
            files[file].as_str().expect("recorded file digest")
        );
    }
    bytes
}

/// Derive only configuration, allowance and request inputs. Receipts, responses,
/// observations and digest manifests are always returned exactly as recorded.
pub fn read_projected(directory: &Path, file: &str) -> Vec<u8> {
    let bytes = read_recorded(directory, file);
    if file != "configuration.json"
        && !file.ends_with("allowance.json")
        && !file.ends_with("request.json")
        && !file.ends_with("request-input.json")
    {
        return bytes;
    }
    let configuration = read_recorded(directory, "configuration.json");
    let files: serde_json::Value =
        serde_json::from_slice(&read_recorded(directory, "files.json")).unwrap();
    let mut allowances: Vec<_> = files
        .as_object()
        .unwrap()
        .keys()
        .filter(|name| name.ends_with("allowance.json"))
        .collect();
    allowances.sort();
    let allowance: serde_json::Value = serde_json::from_slice(&read_recorded(
        directory,
        allowances.first().expect("recorded allowance"),
    ))
    .unwrap();
    let old = allowance["configuration_sha256"].as_str().unwrap();
    let projected = project_verified(
        &configuration,
        files["configuration.json"].as_str().unwrap(),
        old,
    );
    if file == "configuration.json" {
        return projected;
    }
    let configuration: rrd_contract::EndpointConfiguration =
        serde_json::from_slice(&projected).unwrap();
    String::from_utf8(bytes)
        .unwrap()
        .replace(old, &configuration.sha256().unwrap())
        .into_bytes()
}
