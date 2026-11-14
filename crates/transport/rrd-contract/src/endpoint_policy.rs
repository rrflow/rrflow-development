//! Optional governed selector settings. Configuration is not effect authority.
use crate::{
    endpoint_selection::{bounded, validate_selection_candidates},
    invalid, CanonicalId, EndpointBinding, EndpointConfiguration, EndpointSelectionAllowance,
    EndpointSelectionCandidate, EndpointSelectionLimits, Result, WorkloadClass,
    ENDPOINT_SELECTION_VERSION, MAX_ROUTE_REQUEST_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
}

impl EndpointSelectionPolicy {
    pub fn validate_for(&self, bindings: &[EndpointBinding]) -> Result<()> {
        self.limits.validate()?;
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
