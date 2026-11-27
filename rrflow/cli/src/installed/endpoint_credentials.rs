//! Explicit process-start configuration; never exported to the public profile.
use super::*;
use serde::Deserialize;

const MAX_CREDENTIAL_MAPPING_BYTES: u64 = 65_536;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mapping {
    format_version: u16,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    reference: rrd_contract::CredentialReference,
    environment_variable: String,
}

pub(super) fn load(path: &Path) -> Result<rrd_engine::EndpointCredentials, BoxError> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_CREDENTIAL_MAPPING_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CREDENTIAL_MAPPING_BYTES {
        return Err("endpoint credential mapping is oversized".into());
    }
    let mapping: Mapping =
        serde_json::from_slice(&bytes).map_err(|_| "invalid endpoint credential mapping")?;
    if mapping.format_version != 1 || mapping.entries.len() > rrd_contract::MAX_PROFILE_BINDINGS {
        return Err("unsupported endpoint credential mapping version or size".into());
    }
    let mut entries = Vec::with_capacity(mapping.entries.len());
    for entry in mapping.entries {
        if entry.environment_variable.is_empty()
            || entry.environment_variable.len() > 256
            || !entry
                .environment_variable
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("invalid endpoint credential environment name".into());
        }
        let value = std::env::var(&entry.environment_variable)
            .map_err(|_| "configured endpoint credential environment value is unavailable")?;
        entries.push((entry.reference, value));
    }
    Ok(rrd_engine::EndpointCredentials::new(entries)?)
}
