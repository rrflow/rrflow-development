//! Bounded decoding of an LFG advisory response. This does not call a service,
//! authorize an effect, prove producer identity, or grant routing authority.

use serde::{Deserialize, Serialize};

/// Only identities already admitted by the caller can match a response.
pub struct ExpectedObservation<'a> {
    pub request_id: &'a str,
    pub revision: u64,
    pub input_sha256: &'a str,
    pub assessment_sha256: &'a str,
    pub rules_sha256: &'a str,
    pub maximum_response_bytes: usize,
}

/// Version-one projection. Future unrecognized fields are ignored rather than
/// becoming authority or being forwarded into a controller request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfgObservation {
    pub schema_version: String,
    pub request_id: String,
    pub revision: u64,
    pub input_sha256: String,
    pub analyzer: String,
    pub assessment_revision: String,
    pub assessment_sha256: String,
    pub rules_revision: String,
    pub rules_sha256: String,
    pub status: String,
    pub intent: LfgIntent,
    pub complexity: LfgComplexity,
    pub agents: LfgAgents,
    pub suggested_role: String,
    pub suggested_effort: String,
    pub reason_ids: Vec<String>,
    pub ignored_extension_count: u64,
    pub advisory: bool,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfgIntent {
    pub status: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LfgComplexity {
    pub bracket: String,
    pub score: f64,
    pub score_unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LfgAgents {
    pub suggested: u64,
    pub min: u64,
    pub max: u64,
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn label(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(char::is_control)
}

/// Decode and compare response identities. The returned values are advisory
/// data; validation cannot verify who produced the response or its quality.
pub fn decode_lfg_observation(
    bytes: &[u8],
    expected: &ExpectedObservation<'_>,
) -> Result<LfgObservation, &'static str> {
    if expected.maximum_response_bytes == 0 || bytes.len() > expected.maximum_response_bytes {
        return Err("LFG response exceeds configured bound");
    }
    if !label(expected.request_id)
        || !digest(expected.input_sha256)
        || !digest(expected.assessment_sha256)
        || !digest(expected.rules_sha256)
    {
        return Err("invalid expected LFG identities");
    }
    let value: LfgObservation =
        serde_json::from_slice(bytes).map_err(|_| "invalid LFG response")?;
    if value.schema_version != "lfg.observation.v1"
        || value.request_id != expected.request_id
        || value.revision != expected.revision
        || value.input_sha256 != expected.input_sha256
        || value.assessment_sha256 != expected.assessment_sha256
        || value.rules_sha256 != expected.rules_sha256
    {
        return Err("LFG response identity mismatch");
    }
    if !matches!(value.status.as_str(), "available" | "partial")
        || !matches!(value.intent.status.as_str(), "available" | "inconclusive")
        || value.complexity.score_unit != "configured-rule-points"
        || !value.complexity.score.is_finite()
        || value.agents.min == 0
        || value.agents.min > value.agents.suggested
        || value.agents.suggested > value.agents.max
        || !value.advisory
        || value.confidence.is_some()
        || ![
            &value.analyzer,
            &value.assessment_revision,
            &value.rules_revision,
            &value.intent.category,
            &value.complexity.bracket,
            &value.suggested_role,
            &value.suggested_effort,
        ]
        .into_iter()
        .all(|item| label(item))
        || !value.reason_ids.iter().all(|item| label(item))
    {
        return Err("unsupported or invalid LFG advisory observation");
    }
    Ok(value)
}
