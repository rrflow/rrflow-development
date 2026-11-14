//! Test-only interpretation of an immutable pre-rename observation.
//! The derived current configuration is not the captured configuration and
//! does not inherit its receipt, authorization or digest.
use rrd_contract::{CanonicalId, EndpointConfiguration, EndpointProfile, InstalledEngramIdentity};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

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

pub fn project(bytes: &[u8], file_digest: &str, captured_digest: &str) -> EndpointConfiguration {
    assert_eq!(sha256(bytes), file_digest, "captured file digest");
    let historical: HistoricalConfiguration = serde_json::from_slice(bytes).unwrap();
    let digest = sha256(&serde_json::to_vec(&historical).unwrap());
    assert_eq!(
        digest, captured_digest,
        "historical canonical configuration digest"
    );
    let projected = EndpointConfiguration {
        format_version: historical.format_version,
        identity: InstalledEngramIdentity {
            project_id: historical.identity.project_id,
            engram_id: historical.identity.estate_id,
            instance_id: historical.identity.instance_id,
        },
        revision: historical.revision,
        profile: historical.profile,
        last_applied_plan_sha256: historical.last_applied_plan_sha256,
    };
    projected.validate().unwrap();
    assert_ne!(projected.sha256().unwrap(), captured_digest);
    projected
}
