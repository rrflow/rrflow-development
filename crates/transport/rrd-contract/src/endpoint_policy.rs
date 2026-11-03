//! Optional governed selector settings. Configuration is not effect authority.
use crate::{
    endpoint_selection::{bounded, validate_selection_candidates},
    invalid, CanonicalId, EndpointBinding, EndpointConfiguration, EndpointSelectionAllowance,
    EndpointSelectionCandidate, EndpointSelectionLimits, Result, WorkloadClass,
    ENDPOINT_SELECTION_VERSION, MAX_ROUTE_REQUEST_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Explicit opt-in for the local LFG advisory protocol. A loopback address is
/// a transport restriction, not evidence of the analyzer's identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LfgAnalysisPolicy {
    pub base_url: String,
    pub assessment_sha256: String,
    pub rules_sha256: String,
    pub timeout_ms: u64,
    pub maximum_input_bytes: u64,
    pub maximum_request_bytes: u64,
    pub maximum_response_bytes: u64,
}

impl LfgAnalysisPolicy {
    pub fn validate(&self) -> Result<()> {
        let address = self
            .base_url
            .strip_prefix("http://")
            .map(|value| value.strip_suffix('/').unwrap_or(value))
            .and_then(|value| value.parse::<std::net::SocketAddr>().ok());
        if !address.is_some_and(|value| value.ip().is_loopback() && value.port() != 0)
            || self.timeout_ms == 0
            || self.timeout_ms > crate::MAX_BINDING_TIMEOUT_MS
            || self.maximum_input_bytes == 0
            || self.maximum_request_bytes == 0
            || self.maximum_response_bytes == 0
            || self.maximum_input_bytes > self.maximum_request_bytes
            || self.maximum_request_bytes > crate::MAX_ROUTE_REQUEST_BYTES
            || self.maximum_response_bytes > crate::MAX_ROUTE_RESPONSE_BYTES
        {
            return invalid("invalid LFG advisory origin or resource limits");
        }
        crate::validate_sha256(&self.assessment_sha256, "LFG assessment digest")?;
        crate::validate_sha256(&self.rules_sha256, "LFG rules digest")
    }
}

/// Operator-declared settings and evidence references, bound by the same
/// endpoint-profile preview/apply digest. Dispatch must still verify required
/// capability evidence, intersect permission and reserve the task's budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionPolicy {
    pub contract_version: u16,
    pub purpose: crate::EndpointSelectionPurpose,
    /// Explicit experimental permission; a reported alias is not weight attestation.
    pub allow_unverified_identity: bool,
    pub allow_network: bool,
    /// Aggregate outstanding controller calls, including unresolved remote work.
    pub maximum_concurrent_proposals: u32,
    pub controller_binding_id: CanonicalId,
    pub workload_class: WorkloadClass,
    pub limits: EndpointSelectionLimits,
    pub candidates: Vec<EndpointSelectionCandidate>,
    pub system_prompt: String,
    pub maximum_output_tokens: u32,
    /// Exact thousandths avoid floating-point configuration digest ambiguity.
    pub temperature_milli: u16,
    /// Absent means no analyzer dispatch and preserves prior configuration bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfg: Option<LfgAnalysisPolicy>,
}

impl EndpointSelectionPolicy {
    pub fn validate_for(&self, bindings: &[EndpointBinding]) -> Result<()> {
        self.limits.validate()?;
        if let Some(lfg) = &self.lfg {
            lfg.validate()?;
            if !self.allow_network || !self.allow_unverified_identity {
                return invalid(
                    "LFG advisory requires explicit network and unverified-identity permission",
                );
            }
        }
        if self.contract_version != ENDPOINT_SELECTION_VERSION {
            return invalid("unsupported selector policy version");
        }
        if self.maximum_concurrent_proposals == 0
            || self.maximum_concurrent_proposals as usize > crate::MAX_PROFILE_BINDINGS
        {
            return invalid("invalid controller concurrency limit");
        }
        let controller = bindings
            .iter()
            .find(|b| b.id == self.controller_binding_id)
            .ok_or_else(|| {
                crate::ContractError("selector controller binding is not configured".into())
            })?;
        if !matches!(
            controller.class,
            WorkloadClass::LargeModelServing | WorkloadClass::AlwaysWarmSmallModels
        ) || self.system_prompt.trim().is_empty()
            || self.maximum_output_tokens == 0
            || self.maximum_output_tokens > 65_536
            || self.temperature_milli > 2000
        {
            return invalid("invalid selector controller or generation settings");
        }
        bounded(
            self,
            controller
                .limits
                .max_input_bytes
                .min(MAX_ROUTE_REQUEST_BYTES),
        )?;
        validate_selection_candidates(
            &self.candidates,
            &self.limits,
            self.workload_class,
            bindings,
        )
    }

    /// Project only the policy installed in this exact configuration. This is
    /// declared candidate consistency, not runtime eligibility or permission.
    pub fn allowance(
        &self,
        configuration: &EndpointConfiguration,
    ) -> Result<EndpointSelectionAllowance> {
        configuration.validate()?;
        if configuration
            .profile
            .as_ref()
            .and_then(|p| p.selection_policy.as_ref())
            != Some(self)
        {
            return invalid("selector policy differs from installed configuration");
        }
        let allowance = EndpointSelectionAllowance {
            contract_version: self.contract_version,
            configuration_sha256: configuration.sha256()?,
            workload_class: self.workload_class,
            limits: self.limits.clone(),
            candidates: self.candidates.clone(),
        };
        allowance.validate_for(configuration)?;
        Ok(allowance)
    }
}
