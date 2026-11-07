//! One explicit, source-bound controller assessment under a task budget.
//!
//! A retained assessment is a controller report about bounded supplied inputs;
//! it does not establish correctness, project permission, or a project effect.

use crate::{
    endpoint_selection::bounded, invalid, sha256_bytes, validate_sha256, CanonicalId,
    ContextReadStamp, CorrelationId, Result, TaskWorkerSourceProvenance, TaskWorkerSourceRequest,
    MAX_ROUTE_REQUEST_BYTES, MAX_ROUTE_RESPONSE_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const TASK_ASSESSMENT_VERSION: u16 = 1;
pub const MAX_ASSESSMENT_PRACTICE_BYTES: u64 = 64 * 1024;
pub const MAX_ASSESSMENT_OPTIONS: usize = 100;
pub const MAX_ASSESSMENT_REFERENCE_COUNT: usize = 64;

/// Assessment is disabled unless an operator explicitly reviews these flags.
/// The controller is always the selector's configured controller binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointAssessmentPolicy {
    pub contract_version: u16,
    pub allow_network: bool,
    pub allow_source_export: bool,
    pub allow_context_export: bool,
    pub system_prompt: String,
    pub generation: crate::EndpointGenerationSettings,
    pub maximum_input_bytes: u64,
    pub maximum_response_bytes: u64,
}

impl EndpointAssessmentPolicy {
    pub fn validate_for(
        &self,
        bindings: &[crate::EndpointBinding],
        selection: Option<&crate::EndpointSelectionPolicy>,
    ) -> Result<()> {
        if self.contract_version != TASK_ASSESSMENT_VERSION
            || self.system_prompt.trim().is_empty()
            || self.system_prompt.len() as u64 > MAX_ASSESSMENT_PRACTICE_BYTES
            || self.maximum_input_bytes == 0
            || self.maximum_input_bytes > MAX_ROUTE_REQUEST_BYTES
            || self.maximum_response_bytes == 0
            || self.maximum_response_bytes > MAX_ROUTE_RESPONSE_BYTES
        {
            return invalid("invalid or incomplete assessment policy");
        }
        self.generation.validate()?;
        if self.generation.response_format != crate::EndpointResponseFormat::JsonObject {
            return invalid("assessment controller response format must be JSON object");
        }
        let selector = selection.ok_or_else(|| {
            crate::ContractError("assessment requires the configured selector controller".into())
        })?;
        let controller = bindings
            .iter()
            .find(|binding| binding.id == selector.controller_binding_id)
            .ok_or_else(|| crate::ContractError("assessment controller is absent".into()))?;
        if !matches!(
            controller.class,
            crate::WorkloadClass::LargeModelServing | crate::WorkloadClass::AlwaysWarmSmallModels
        ) || !matches!(&controller.endpoint, crate::EndpointLocator::Url { .. })
            || self.maximum_input_bytes > controller.limits.max_input_bytes
        {
            return invalid("assessment controller must be the configured selector URL binding");
        }
        bounded(
            self,
            controller
                .limits
                .max_input_bytes
                .min(MAX_ROUTE_REQUEST_BYTES),
        )?;
        Ok(())
    }
}

/// Caller-declared practice provenance. Its digest describes the declaration,
/// not a separately authenticated template repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssessmentPracticeIdentity {
    pub question_set_id: CanonicalId,
    pub question_id: CanonicalId,
    pub template_id: CanonicalId,
    pub template_revision: u64,
    pub template_sha256: String,
    pub requirement_id: CanonicalId,
}

impl AssessmentPracticeIdentity {
    pub fn validate(&self) -> Result<()> {
        if self.template_revision == 0 {
            return invalid("assessment template revision must be nonzero");
        }
        validate_sha256(&self.template_sha256, "assessment template digest")
    }
}

/// Closed mode descriptor for a single assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum TaskAssessmentDescriptor {
    Binary {
        proposition: String,
    },
    Choice {
        question: String,
        options: Vec<AssessmentOption>,
    },
    Score {
        rubric: String,
        minimum: f64,
        maximum: f64,
        direction: AssessmentScoreDirection,
    },
}

impl TaskAssessmentDescriptor {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Binary { proposition } => validate_assessment_text(proposition, "proposition"),
            Self::Choice { question, options } => {
                validate_assessment_text(question, "question")?;
                if !(2..=MAX_ASSESSMENT_OPTIONS).contains(&options.len()) {
                    return invalid("assessment choice option count is out of bounds");
                }
                let mut ids = std::collections::BTreeSet::new();
                for option in options {
                    option.validate()?;
                    if !ids.insert(option.id.as_str()) {
                        return invalid("assessment choice option identity is duplicated");
                    }
                }
                Ok(())
            }
            Self::Score {
                rubric,
                minimum,
                maximum,
                ..
            } => {
                validate_assessment_text(rubric, "rubric")?;
                if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
                    return invalid("assessment score bounds must be finite and ordered");
                }
                Ok(())
            }
        }
    }
}

fn validate_assessment_text(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty()
        || value.len() as u64 > MAX_ASSESSMENT_PRACTICE_BYTES
        || value.contains('\0')
    {
        return invalid(format!("assessment {label} is empty or out of bounds"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssessmentOption {
    pub id: CanonicalId,
    pub description: String,
}

impl AssessmentOption {
    fn validate(&self) -> Result<()> {
        validate_assessment_text(&self.description, "option description")
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentScoreDirection {
    HigherIsBetter,
    LowerIsBetter,
}

/// Strict source and practice selection. Paths and source bytes are resolved
/// only by the engine from its accepted inventory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAssessmentRequest {
    pub contract_version: u16,
    pub scope: String,
    pub task_id: CanonicalId,
    pub attempt_id: CorrelationId,
    pub expected_task_sha256: String,
    pub expected_configuration_sha256: String,
    pub max_storage_keys: u64,
    pub source: TaskWorkerSourceRequest,
    pub descriptor: TaskAssessmentDescriptor,
    pub practice_identity: AssessmentPracticeIdentity,
    pub practice_text: String,
}

impl TaskAssessmentRequest {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != TASK_ASSESSMENT_VERSION {
            return invalid("unsupported task assessment request version");
        }
        crate::ReadTask {
            scope: self.scope.clone(),
            task_id: self.task_id.clone(),
            max_storage_keys: self.max_storage_keys,
        }
        .validate()?;
        for (value, label) in [
            (&self.expected_task_sha256, "task digest"),
            (&self.expected_configuration_sha256, "configuration digest"),
        ] {
            validate_sha256(value, label)?;
        }
        self.source.validate()?;
        self.descriptor.validate()?;
        self.practice_identity.validate()?;
        if self.practice_text.trim().is_empty()
            || self.practice_text.len() as u64 > MAX_ASSESSMENT_PRACTICE_BYTES
            || self.practice_text.as_bytes().contains(&0)
        {
            return invalid("assessment practice text is empty or exceeds its byte bound");
        }
        bounded(self, MAX_ROUTE_REQUEST_BYTES)
    }

    pub fn practice_sha256(&self) -> Result<String> {
        self.validate()?;
        Ok(sha256_bytes(self.practice_text.as_bytes()))
    }
}

/// The exact source/practice payload supplied to the configured controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAssessmentPacket {
    pub contract_version: u16,
    pub scope: String,
    pub task_id: CanonicalId,
    pub task_sha256: String,
    pub configuration_sha256: String,
    pub descriptor: TaskAssessmentDescriptor,
    pub practice_identity: AssessmentPracticeIdentity,
    pub practice_text: String,
    pub practice_sha256: String,
    pub source_content: String,
    pub source: TaskWorkerSourceProvenance,
    pub read: ContextReadStamp,
}

impl TaskAssessmentPacket {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != TASK_ASSESSMENT_VERSION {
            return invalid("unsupported task assessment packet version");
        }
        crate::task::validate_scope(&self.scope)?;
        for (value, label) in [
            (&self.task_sha256, "assessment packet task digest"),
            (
                &self.configuration_sha256,
                "assessment packet configuration digest",
            ),
            (&self.practice_sha256, "assessment packet practice digest"),
        ] {
            validate_sha256(value, label)?;
        }
        self.descriptor.validate()?;
        self.practice_identity.validate()?;
        self.source.validate()?;
        self.read.validate()?;
        if self.practice_text.trim().is_empty()
            || self.practice_text.len() as u64 > MAX_ASSESSMENT_PRACTICE_BYTES
            || sha256_bytes(self.practice_text.as_bytes()) != self.practice_sha256
            || self.source_content.len() as u64 > MAX_ROUTE_REQUEST_BYTES
            || sha256_bytes(self.source_content.as_bytes()) != self.source.content_sha256
        {
            return invalid("assessment packet input digest or bound mismatch");
        }
        bounded(self, MAX_ROUTE_REQUEST_BYTES)
    }
}

/// A controller answer has exactly the type required by its declared mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TaskAssessmentValue {
    Binary { value: bool },
    Choice { option_id: CanonicalId },
    Score { value: f64 },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentInputKind {
    Source,
    Practice,
}

/// Byte offsets into exactly one of the two packet inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAssessmentReference {
    pub input: AssessmentInputKind,
    pub input_sha256: String,
    pub start_byte: u64,
    pub end_byte: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum TaskAssessmentDecision {
    Answered { value: TaskAssessmentValue },
    Abstained { reason: String },
}

/// Parsed provider content. Unknown keys are rejected and every cited byte
/// range is bound to the exact source or practice bytes in this packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAssessmentReply {
    pub contract_version: u16,
    pub decision: TaskAssessmentDecision,
    pub references: Vec<TaskAssessmentReference>,
}

impl TaskAssessmentReply {
    pub fn decode(
        content: &[u8],
        descriptor: &TaskAssessmentDescriptor,
        source: &str,
        practice: &str,
    ) -> Result<Self> {
        descriptor.validate()?;
        if content.len() as u64 > MAX_ROUTE_RESPONSE_BYTES {
            return invalid("assessment response exceeds its byte bound");
        }
        let reply: Self = serde_json::from_slice(content)
            .map_err(|error| crate::ContractError(format!("invalid assessment reply: {error}")))?;
        reply.validate_for(
            descriptor,
            &sha256_bytes(source.as_bytes()),
            source.len() as u64,
            &sha256_bytes(practice.as_bytes()),
            practice.len() as u64,
        )?;
        bounded(&reply, MAX_ROUTE_RESPONSE_BYTES)?;
        Ok(reply)
    }

    pub fn validate_for(
        &self,
        descriptor: &TaskAssessmentDescriptor,
        source_sha256: &str,
        source_bytes: u64,
        practice_sha256: &str,
        practice_bytes: u64,
    ) -> Result<()> {
        if self.contract_version != TASK_ASSESSMENT_VERSION {
            return invalid("unsupported task assessment reply version");
        }
        descriptor.validate()?;
        validate_sha256(source_sha256, "assessment source digest")?;
        validate_sha256(practice_sha256, "assessment practice digest")?;
        if self.references.len() > MAX_ASSESSMENT_REFERENCE_COUNT {
            return invalid("assessment references exceed their bound");
        }
        if let TaskAssessmentDecision::Answered { value } = &self.decision {
            if self.references.is_empty() {
                return invalid("answered assessment must include evidence references");
            }
            match (descriptor, value) {
                (TaskAssessmentDescriptor::Binary { .. }, TaskAssessmentValue::Binary { .. }) => {}
                (
                    TaskAssessmentDescriptor::Choice { options, .. },
                    TaskAssessmentValue::Choice { option_id },
                ) if options.iter().any(|option| &option.id == option_id) => {}
                (
                    TaskAssessmentDescriptor::Score {
                        minimum, maximum, ..
                    },
                    TaskAssessmentValue::Score { value },
                ) if value.is_finite() && value >= minimum && value <= maximum => {}
                _ => return invalid("assessment value does not match its descriptor"),
            }
        } else if let TaskAssessmentDecision::Abstained { reason } = &self.decision {
            validate_assessment_text(reason, "abstention reason")?;
        }
        let mut unique = std::collections::BTreeSet::new();
        for reference in &self.references {
            validate_sha256(&reference.input_sha256, "assessment reference digest")?;
            let (digest, length) = match reference.input {
                AssessmentInputKind::Source => (source_sha256, source_bytes),
                AssessmentInputKind::Practice => (practice_sha256, practice_bytes),
            };
            if reference.input_sha256 != digest
                || reference.start_byte >= reference.end_byte
                || reference.end_byte > length
            {
                return invalid("assessment reference is outside the supplied input");
            }
            if !unique.insert((reference.input, reference.start_byte, reference.end_byte)) {
                return invalid("assessment reference is duplicated");
            }
        }
        bounded(self, MAX_ROUTE_RESPONSE_BYTES)
    }
}

/// Durable assessment projection is read separately from task-attempt v1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAssessmentOutput {
    pub contract_version: u16,
    pub scope: String,
    pub task_id: CanonicalId,
    pub attempt_id: CorrelationId,
    pub task_sha256: String,
    pub configuration_sha256: String,
    pub packet_sha256: String,
    pub request_sha256: String,
    pub response_sha256: String,
    pub content_sha256: String,
    pub controller_binding_id: CanonicalId,
    /// Provider alias observed at the configured endpoint; not loaded-model proof.
    pub service_id: CanonicalId,
    pub descriptor: TaskAssessmentDescriptor,
    pub practice_identity: AssessmentPracticeIdentity,
    pub practice_sha256: String,
    pub source_bytes: u64,
    pub practice_bytes: u64,
    pub source: TaskWorkerSourceProvenance,
    pub read: ContextReadStamp,
    pub decision: TaskAssessmentDecision,
    pub references: Vec<TaskAssessmentReference>,
    pub usage: Option<crate::EndpointTokenUsage>,
    pub elapsed_ms: Option<u64>,
    pub http_status: Option<u16>,
}

impl TaskAssessmentOutput {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != TASK_ASSESSMENT_VERSION {
            return invalid("unsupported task assessment output version");
        }
        crate::task::validate_scope(&self.scope)?;
        for (digest, label) in [
            (&self.task_sha256, "assessment task digest"),
            (
                &self.configuration_sha256,
                "assessment configuration digest",
            ),
            (&self.packet_sha256, "assessment packet digest"),
            (&self.request_sha256, "assessment request digest"),
            (&self.response_sha256, "assessment response digest"),
            (&self.content_sha256, "assessment content digest"),
            (&self.practice_sha256, "assessment practice digest"),
        ] {
            validate_sha256(digest, label)?;
        }
        self.descriptor.validate()?;
        self.practice_identity.validate()?;
        self.source.validate()?;
        self.read.validate()?;
        if self.practice_bytes == 0
            || self.source_bytes > MAX_ROUTE_REQUEST_BYTES
            || self.practice_bytes > MAX_ASSESSMENT_PRACTICE_BYTES
        {
            return invalid("assessment input byte count is out of bounds");
        }
        TaskAssessmentReply {
            contract_version: TASK_ASSESSMENT_VERSION,
            decision: self.decision.clone(),
            references: self.references.clone(),
        }
        .validate_for(
            &self.descriptor,
            &self.source.content_sha256,
            self.source_bytes,
            &self.practice_sha256,
            self.practice_bytes,
        )?;
        if self.http_status.is_some_and(|status| status != 200) {
            return invalid("accepted assessment HTTP status must be 200");
        }
        if let Some(usage) = &self.usage {
            usage.validate()?;
        }
        bounded(self, MAX_ROUTE_RESPONSE_BYTES)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadTaskAssessment {
    pub contract_version: u16,
    pub scope: String,
    pub task_id: CanonicalId,
    pub attempt_id: CorrelationId,
}

impl ReadTaskAssessment {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != TASK_ASSESSMENT_VERSION {
            return invalid("unsupported task assessment read version");
        }
        crate::ReadTaskAttempt {
            scope: self.scope.clone(),
            task_id: self.task_id.clone(),
            attempt_id: self.attempt_id.clone(),
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAssessmentView {
    pub contract_version: u16,
    pub assessment: Option<TaskAssessmentOutput>,
    pub outcome_sha256: Option<String>,
}

impl TaskAssessmentView {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != TASK_ASSESSMENT_VERSION {
            return invalid("unsupported task assessment read response version");
        }
        match (&self.assessment, &self.outcome_sha256) {
            (None, None) => Ok(()),
            (Some(output), Some(digest)) => {
                output.validate()?;
                validate_sha256(digest, "assessment outcome digest")?;
                let bytes = serde_json::to_vec(output)
                    .map_err(|error| crate::ContractError(error.to_string()))?;
                if sha256_bytes(&bytes) != *digest {
                    return invalid("assessment read digest mismatch");
                }
                Ok(())
            }
            _ => invalid("assessment output and outcome digest must be present together"),
        }
    }
}
