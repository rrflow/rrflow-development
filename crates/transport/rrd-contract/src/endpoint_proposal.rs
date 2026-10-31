//! Task-bound proposal semantics. Validation grants neither dispatch nor worker authority.
use crate::{
    endpoint_selection::bounded, invalid, sha256_bytes, validate_sha256, CanonicalId,
    CorrelationId, EndpointConfiguration, EndpointSelectionAllowance, EndpointSelectionChoice,
    OpenTask, Result, TaskRecord, ENDPOINT_SELECTION_VERSION, MAX_ROUTE_REQUEST_BYTES,
    MAX_ROUTE_RESPONSE_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A proposal can describe a choice without qualifying that choice for execution.
/// Adding another purpose requires its own admission semantics and evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointSelectionPurpose {
    ExperimentalProposal,
}

/// No endpoint, credential, prompt or client-authored candidate facts enter this request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionRequest {
    pub scope: String,
    pub task_id: CanonicalId,
    pub attempt_id: CorrelationId,
    pub max_storage_keys: u64,
    pub expected_task_sha256: String,
    pub expected_configuration_sha256: String,
}

impl EndpointSelectionRequest {
    pub fn validate(&self) -> Result<()> {
        crate::ReadTask {
            scope: self.scope.clone(),
            task_id: self.task_id.clone(),
            max_storage_keys: self.max_storage_keys,
        }
        .validate()?;
        validate_sha256(&self.expected_task_sha256, "expected task digest")?;
        validate_sha256(
            &self.expected_configuration_sha256,
            "expected endpoint configuration digest",
        )
    }
}

/// The bounded task-text projection intentionally excludes its creation replay key.
/// Repository context and dependency freshness are not implied by this projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionTask {
    pub scope: String,
    pub task_id: CanonicalId,
    pub title: String,
    pub goal: String,
    pub task_sha256: String,
    pub known_at_cursor: u64,
}

impl EndpointSelectionTask {
    pub fn from_task(task: &TaskRecord) -> Result<Self> {
        task.validate()?;
        if task.status.is_closed() {
            return invalid("a closed task cannot request an endpoint proposal");
        }
        Ok(Self {
            scope: task.scope.clone(),
            task_id: task.task_id.clone(),
            title: task.title.clone(),
            goal: task.goal.clone(),
            task_sha256: digest(task)?,
            known_at_cursor: task.known_at_cursor,
        })
    }

    fn validate(&self) -> Result<()> {
        OpenTask {
            scope: self.scope.clone(),
            title: self.title.clone(),
            goal: self.goal.clone(),
        }
        .validate()?;
        validate_sha256(&self.task_sha256, "selection task digest")?;
        if self.known_at_cursor == 0 {
            return invalid("selection task needs a committed cursor");
        }
        Ok(())
    }
}

/// Engine-built task text and declared candidates. Digests identify inputs; they
/// do not attest capability, entitlement, loaded weights or data-sharing authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionPacket {
    pub contract_version: u16,
    pub purpose: EndpointSelectionPurpose,
    pub task: EndpointSelectionTask,
    pub read_manifest_sha256: String,
    pub read_cursor: u64,
    pub controller_binding_id: CanonicalId,
    pub allowance: EndpointSelectionAllowance,
}

impl EndpointSelectionPacket {
    pub fn validate_for(&self, configuration: &EndpointConfiguration) -> Result<()> {
        configuration.validate()?;
        if self.contract_version != ENDPOINT_SELECTION_VERSION {
            return invalid("unsupported endpoint proposal packet version");
        }
        self.task.validate()?;
        validate_sha256(&self.read_manifest_sha256, "selection read manifest digest")?;
        if self.read_cursor < self.task.known_at_cursor {
            return invalid("selection read precedes its task version");
        }
        let profile = configuration
            .profile
            .as_ref()
            .ok_or_else(|| crate::ContractError("endpoint selection is not configured".into()))?;
        let policy = profile
            .selection_policy
            .as_ref()
            .ok_or_else(|| crate::ContractError("endpoint selector policy is absent".into()))?;
        if self.purpose != policy.purpose
            || self.controller_binding_id != policy.controller_binding_id
            || self.allowance != policy.allowance(configuration)?
        {
            return invalid("selection packet differs from the configured policy");
        }
        let controller = profile
            .bindings
            .iter()
            .find(|binding| binding.id == self.controller_binding_id)
            .ok_or_else(|| crate::ContractError("controller binding is absent".into()))?;
        bounded(
            self,
            controller
                .limits
                .max_input_bytes
                .min(MAX_ROUTE_REQUEST_BYTES),
        )
    }

    pub fn validate_task(&self, task: &TaskRecord) -> Result<()> {
        if self.task != EndpointSelectionTask::from_task(task)? {
            return invalid("selection packet task changed");
        }
        Ok(())
    }

    pub fn sha256(&self, configuration: &EndpointConfiguration) -> Result<String> {
        self.validate_for(configuration)?;
        digest(self)
    }
}

/// Provider-reported counts only. Absence is unknown, not zero or an estimate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointTokenUsage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl EndpointTokenUsage {
    pub fn validate(&self) -> Result<()> {
        let sum = match (self.prompt_tokens, self.completion_tokens) {
            (Some(prompt), Some(completion)) => Some(
                prompt
                    .checked_add(completion)
                    .ok_or_else(|| crate::ContractError("reported token usage overflows".into()))?,
            ),
            _ => None,
        };
        if let Some(total) = self.total_tokens {
            if sum.is_some_and(|sum| sum != total)
                || self.prompt_tokens.is_some_and(|count| count > total)
                || self.completion_tokens.is_some_and(|count| count > total)
            {
                return invalid("inconsistent reported token usage");
            }
        }
        Ok(())
    }
}

/// A typed proposal receipt, not a selected-worker admission or artifact attestation.
/// Construction alone is not evidence that an engine admitted or observed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointSelectionProposal {
    pub contract_version: u16,
    pub purpose: EndpointSelectionPurpose,
    pub scope: String,
    pub task_id: CanonicalId,
    pub attempt_id: CorrelationId,
    pub task_sha256: String,
    pub packet_sha256: String,
    pub configuration_sha256: String,
    pub allowance_sha256: String,
    pub controller_binding_id: CanonicalId,
    /// Observed service alias; it does not prove the bytes loaded by that service.
    pub service_id: CanonicalId,
    pub request_sha256: String,
    pub response_sha256: String,
    pub content_sha256: String,
    pub choice: EndpointSelectionChoice,
    pub usage: Option<EndpointTokenUsage>,
    pub elapsed_ms: Option<u64>,
    pub http_status: Option<u16>,
}

impl EndpointSelectionProposal {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != ENDPOINT_SELECTION_VERSION {
            return invalid("unsupported endpoint proposal version");
        }
        crate::task::validate_scope(&self.scope)?;
        for (value, field) in [
            (&self.task_sha256, "selection task digest"),
            (&self.configuration_sha256, "selection configuration digest"),
            (&self.packet_sha256, "selection packet digest"),
            (&self.allowance_sha256, "selection allowance digest"),
            (&self.request_sha256, "selector request digest"),
            (&self.response_sha256, "selector response digest"),
            (&self.content_sha256, "selector content digest"),
        ] {
            validate_sha256(value, field)?;
        }
        if self
            .http_status
            .is_some_and(|status| !(200..300).contains(&status))
        {
            return invalid("an accepted HTTP proposal requires a successful status");
        }
        if let Some(usage) = &self.usage {
            usage.validate()?;
        }
        bounded(self, MAX_ROUTE_RESPONSE_BYTES)
    }

    pub fn validate_for(
        &self,
        request: &EndpointSelectionRequest,
        packet: &EndpointSelectionPacket,
        configuration: &EndpointConfiguration,
    ) -> Result<()> {
        self.validate()?;
        request.validate()?;
        packet.validate_for(configuration)?;
        if self.scope != request.scope
            || self.scope != packet.task.scope
            || self.task_id != request.task_id
            || self.task_id != packet.task.task_id
            || self.attempt_id != request.attempt_id
            || self.task_sha256 != request.expected_task_sha256
            || self.task_sha256 != packet.task.task_sha256
            || self.configuration_sha256 != request.expected_configuration_sha256
            || self.configuration_sha256 != configuration.sha256()?
            || self.packet_sha256 != packet.sha256(configuration)?
            || self.allowance_sha256 != packet.allowance.sha256(configuration)?
            || self.controller_binding_id != packet.controller_binding_id
            || self.purpose != packet.purpose
        {
            return invalid("endpoint proposal input identity mismatch");
        }
        let controller = configuration
            .profile
            .as_ref()
            .and_then(|profile| {
                profile
                    .bindings
                    .iter()
                    .find(|binding| binding.id == self.controller_binding_id)
            })
            .ok_or_else(|| crate::ContractError("proposal controller is absent".into()))?;
        if self.service_id != controller.service_id {
            return invalid("endpoint proposal service identity mismatch");
        }
        // Reuse the same bounded choice decoder as the actual native transport.
        let encoded =
            serde_json::to_vec(&self.choice).map_err(|e| crate::ContractError(e.to_string()))?;
        EndpointSelectionChoice::decode(&encoded, &packet.allowance, configuration)?;
        Ok(())
    }
}

fn digest(value: &impl Serialize) -> Result<String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(value).map_err(|e| crate::ContractError(e.to_string()))?,
    ))
}
