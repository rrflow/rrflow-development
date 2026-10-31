//! Configured worker-call boundary. A policy is not execution authorization.
use crate::{
    endpoint_selection::bounded, invalid, validate_sha256, CanonicalId, EndpointBinding,
    EndpointEffortSupport, EndpointGenerationSettings, EndpointSelectionPolicy, Result,
    WorkloadClass, MAX_PROFILE_BINDINGS, MAX_ROUTE_REQUEST_BYTES, MAX_ROUTE_RESPONSE_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ENDPOINT_WORKER_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointWorkerBinding {
    pub binding_id: CanonicalId,
    /// Reference to observed wire support; its presence is not an attestation.
    pub protocol_evidence_sha256: String,
    pub system_prompt: String,
    pub generation: EndpointGenerationSettings,
    pub maximum_response_bytes: u64,
    /// Explicit mapping from selected effort to the adapter's wire parameter.
    /// No ranking, model-name inference or silent dropping of effort is allowed.
    pub effort_values: BTreeMap<String, String>,
    pub allow_default_effort: bool,
}

impl EndpointWorkerBinding {
    pub fn wire_effort(&self, selected: Option<&str>) -> Result<Option<&str>> {
        match selected {
            None if self.allow_default_effort => Ok(None),
            Some(value) => self
                .effort_values
                .get(value)
                .map(|value| Some(value.as_str()))
                .ok_or_else(|| {
                    crate::ContractError("selected effort has no configured worker mapping".into())
                }),
            None => invalid("default worker effort is not configured"),
        }
    }
}

/// Installed through the existing endpoint profile preview/apply transaction.
/// Absence disables worker calls. Every call still needs task, context, session,
/// budget and current-input admission by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointWorkerPolicy {
    pub contract_version: u16,
    pub allow_network: bool,
    /// Explicit permission to export admitted task text and selected context.
    pub allow_context_export: bool,
    pub allow_unverified_identity: bool,
    pub maximum_concurrent_calls: u32,
    pub bindings: Vec<EndpointWorkerBinding>,
}

impl EndpointWorkerPolicy {
    pub fn validate_for(
        &self,
        bindings: &[EndpointBinding],
        selector: Option<&EndpointSelectionPolicy>,
    ) -> Result<()> {
        if self.contract_version != ENDPOINT_WORKER_VERSION
            || self.maximum_concurrent_calls == 0
            || self.maximum_concurrent_calls as usize > MAX_PROFILE_BINDINGS
            || self.bindings.is_empty()
            || self.bindings.len() > MAX_PROFILE_BINDINGS
            || self
                .bindings
                .windows(2)
                .any(|pair| pair[0].binding_id >= pair[1].binding_id)
        {
            return invalid("invalid worker policy version, limits or binding order");
        }
        let selector = selector.ok_or_else(|| {
            crate::ContractError("worker policy requires a configured selector".into())
        })?;
        selector.validate_for(bindings)?;
        bounded(self, MAX_ROUTE_REQUEST_BYTES)?;
        for worker in &self.bindings {
            worker.generation.validate()?;
            validate_sha256(
                &worker.protocol_evidence_sha256,
                "worker protocol evidence digest",
            )?;
            let binding = bindings
                .iter()
                .find(|binding| binding.id == worker.binding_id)
                .ok_or_else(|| crate::ContractError("worker binding is not configured".into()))?;
            if !matches!(
                binding.class,
                WorkloadClass::LargeModelServing | WorkloadClass::AlwaysWarmSmallModels
            ) || worker.system_prompt.trim().is_empty()
                || worker.maximum_response_bytes == 0
                || worker.maximum_response_bytes > MAX_ROUTE_RESPONSE_BYTES
            {
                return invalid("invalid worker binding, prompt or response limit");
            }
            bounded(
                worker,
                binding.limits.max_input_bytes.min(MAX_ROUTE_REQUEST_BYTES),
            )?;
            let candidate = selector
                .candidates
                .iter()
                .find(|candidate| candidate.binding_id == worker.binding_id)
                .ok_or_else(|| {
                    crate::ContractError("worker binding is outside the selector candidates".into())
                })?;
            let EndpointEffortSupport::Observed {
                allow_default,
                values,
                ..
            } = &candidate.effort
            else {
                return invalid("worker effort support is unknown");
            };
            if (worker.allow_default_effort && !allow_default)
                || (!worker.allow_default_effort && worker.effort_values.is_empty())
                || worker.effort_values.iter().any(|(selected, wire)| {
                    values.binary_search(selected).is_err()
                        || wire.is_empty()
                        || wire.trim() != wire
                        || wire.chars().any(char::is_control)
                })
            {
                return invalid("worker effort mapping differs from declared support");
            }
        }
        Ok(())
    }
}
