//! Bounded endpoint choices, distinct from reasoning-tree route decisions.
//!
//! This is validation of supplied configuration/candidate facts, not evidence
//! that a candidate is authorized, healthy, affordable, or qualified. The engine
//! must construct the allowance and re-admit every provider effect separately.
use crate::{
    invalid, sha256_bytes, validate_sha256, CanonicalId, EndpointConfiguration, Result,
    WorkloadClass, MAX_PROFILE_BINDINGS, MAX_ROUTE_REQUEST_BYTES, MAX_ROUTE_RESPONSE_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

pub const ENDPOINT_SELECTION_VERSION: u16 = 1;

/// Explicit per-adapter limits. Protocol ceilings are resource bounds, not
/// provider rankings or implicit defaults for a configured installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionLimits {
    pub maximum_allowance_bytes: u64,
    pub maximum_response_bytes: u64,
    pub maximum_candidates: u32,
    pub maximum_effort_values: u32,
    pub maximum_effort_bytes: u32,
}

impl EndpointSelectionLimits {
    pub fn validate(&self) -> Result<()> {
        if self.maximum_allowance_bytes == 0
            || self.maximum_allowance_bytes > MAX_ROUTE_REQUEST_BYTES
            || self.maximum_response_bytes == 0
            || self.maximum_response_bytes > MAX_ROUTE_RESPONSE_BYTES
            || self.maximum_candidates == 0
            || self.maximum_candidates as usize > MAX_PROFILE_BINDINGS
            || self.maximum_effort_values == 0
            || u64::from(self.maximum_effort_values) > self.maximum_allowance_bytes
            || self.maximum_effort_bytes == 0
            || u64::from(self.maximum_effort_bytes) > self.maximum_response_bytes
        {
            return invalid("invalid endpoint selection limits");
        }
        Ok(())
    }
}

/// Unknown is distinct from observed support for omission of an effort field.
/// Evidence digests identify declared supporting records, not trusted attestations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EndpointEffortSupport {
    Unknown,
    Observed {
        evidence_sha256: String,
        allow_default: bool,
        values: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionCandidate {
    pub binding_id: CanonicalId,
    pub service_id: CanonicalId,
    pub manifest_sha256: Option<String>,
    pub effort: EndpointEffortSupport,
}

/// A bounded projection for a future engine-built selection packet. Public
/// fields permit serialization, not client-issued eligibility or permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionAllowance {
    pub contract_version: u16,
    pub configuration_sha256: String,
    pub workload_class: WorkloadClass,
    pub limits: EndpointSelectionLimits,
    pub candidates: Vec<EndpointSelectionCandidate>,
}

impl EndpointSelectionAllowance {
    pub fn validate_for(&self, configuration: &EndpointConfiguration) -> Result<()> {
        self.limits.validate()?;
        bounded(self, self.limits.maximum_allowance_bytes)?;
        if self.contract_version != ENDPOINT_SELECTION_VERSION {
            return invalid("unsupported endpoint selection version");
        }
        validate_sha256(&self.configuration_sha256, "selection configuration digest")?;
        if self.configuration_sha256 != configuration.sha256()? {
            return invalid("endpoint selection configuration changed");
        }
        validate_selection_candidates(
            &self.candidates,
            &self.limits,
            self.workload_class,
            configuration
                .profile
                .as_ref()
                .map_or(&[], |profile| profile.bindings.as_slice()),
        )
    }

    pub fn sha256(&self, configuration: &EndpointConfiguration) -> Result<String> {
        self.validate_for(configuration)?;
        Ok(sha256_bytes(&serde_json::to_vec(self).map_err(|_| {
            crate::ContractError("cannot encode endpoint selection allowance".into())
        })?))
    }
}

pub(crate) fn validate_selection_candidates(
    candidates: &[EndpointSelectionCandidate],
    limits: &EndpointSelectionLimits,
    workload_class: WorkloadClass,
    bindings: &[crate::EndpointBinding],
) -> Result<()> {
    if candidates.len() > limits.maximum_candidates as usize {
        return invalid("endpoint selection candidate limit exceeded");
    }
    let mut previous: Option<&CanonicalId> = None;
    for candidate in candidates {
        if previous.is_some_and(|id| id >= &candidate.binding_id) {
            return invalid("endpoint selection candidates must be sorted and unique");
        }
        previous = Some(&candidate.binding_id);
        let binding = bindings
            .iter()
            .find(|binding| binding.id == candidate.binding_id);
        let Some(binding) = binding else {
            return invalid("endpoint selection candidate is not configured");
        };
        if binding.class != workload_class
            || binding.service_id != candidate.service_id
            || binding.manifest_sha256 != candidate.manifest_sha256
        {
            return invalid("endpoint selection candidate differs from configured binding");
        }
        if let EndpointEffortSupport::Observed {
            evidence_sha256,
            allow_default,
            values,
        } = &candidate.effort
        {
            validate_sha256(evidence_sha256, "effort evidence digest")?;
            if values.len() > limits.maximum_effort_values as usize
                || (!allow_default && values.is_empty())
                || values.windows(2).any(|pair| pair[0] >= pair[1])
                || values.iter().any(|value| {
                    value.is_empty()
                        || value.len() > limits.maximum_effort_bytes as usize
                        || value.trim() != value
                        || value.chars().any(char::is_control)
                })
            {
                return invalid("invalid observed endpoint effort support");
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointAbstentionReason {
    NoEligibleCandidate,
    InsufficientEvidence,
    CannotDecide,
}

/// Only the model's observable choice. The engine owns request/source identity,
/// authorization, timestamps, raw-output digest, usage and any accepted receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EndpointSelectionChoice {
    Select {
        binding_id: CanonicalId,
        effort: Option<String>,
    },
    Abstain {
        reason: EndpointAbstentionReason,
    },
}

impl EndpointSelectionChoice {
    pub fn validate_for(
        &self,
        allowance: &EndpointSelectionAllowance,
        configuration: &EndpointConfiguration,
    ) -> Result<()> {
        allowance.validate_for(configuration)?;
        bounded(self, allowance.limits.maximum_response_bytes)?;
        match self {
            Self::Select { binding_id, effort } => {
                let Some(candidate) = allowance
                    .candidates
                    .iter()
                    .find(|c| &c.binding_id == binding_id)
                else {
                    return invalid("selected endpoint is outside the allowance");
                };
                let EndpointEffortSupport::Observed {
                    allow_default,
                    values,
                    ..
                } = &candidate.effort
                else {
                    return invalid("selected endpoint effort support is unknown");
                };
                let supported = match effort {
                    None => *allow_default,
                    Some(value) => values.binary_search(value).is_ok(),
                };
                if !supported {
                    return invalid("selected endpoint effort is unsupported");
                }
            }
            Self::Abstain {
                reason: EndpointAbstentionReason::NoEligibleCandidate,
            } if !allowance.candidates.is_empty() => {
                return invalid("no-candidate abstention contradicts a nonempty allowance");
            }
            Self::Abstain { .. } => {}
        }
        Ok(())
    }

    /// Reject oversized/invalid bytes before returning a usable choice. No raw
    /// model text is included in errors and no external effect occurs here.
    pub fn decode(
        bytes: &[u8],
        allowance: &EndpointSelectionAllowance,
        configuration: &EndpointConfiguration,
    ) -> Result<Self> {
        allowance.validate_for(configuration)?;
        if bytes.len() as u64 > allowance.limits.maximum_response_bytes {
            return invalid("endpoint selection response exceeds byte limit");
        }
        let choice: Self = serde_json::from_slice(bytes)
            .map_err(|_| crate::ContractError("invalid endpoint selection JSON".into()))?;
        choice.validate_for(allowance, configuration)?;
        Ok(choice)
    }
}

/// Bound encoding without allocating a second unbounded serialized copy.
pub(crate) fn bounded(value: &impl Serialize, limit: u64) -> Result<()> {
    struct Counter(u64);
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(bytes.len() as u64)
                .ok_or_else(|| io::Error::other("selection byte limit exceeded"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter(limit), value)
        .map_err(|_| crate::ContractError("endpoint selection encoding exceeds byte limit".into()))
}
