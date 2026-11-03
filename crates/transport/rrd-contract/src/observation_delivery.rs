//! Explicit metadata export and its engine-owned transport receipts.
//! These records do not authorize training or complete the observed task.
use crate::{
    invalid, validate_sha256, CanonicalId, CorrelationId, EndpointBinding, EndpointLocator,
    ReadAttunement, ReadTaskAttempt, Result, TaskAttemptStatus, WorkloadClass,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const OBSERVATION_DELIVERY_VERSION: u16 = 1;
pub const MAX_OBSERVATION_SOURCE_BYTES: usize = 262_144;
pub const MAX_OBSERVATION_EVENT_BYTES: usize = 16_384;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationExportPolicy {
    pub contract_version: u16,
    pub binding_id: CanonicalId,
    pub expected_devspace: String,
    pub protocol_evidence_sha256: String,
    pub allow_network: bool,
    pub allow_metadata_export: bool,
    /// Explicit scopes; absence is not an all-project grant.
    pub allowed_scopes: Vec<String>,
    pub max_retained_deliveries: u32,
    pub retention_ms: u64,
    /// Explicitly permit retiring unresolved records after their original expiry.
    /// Retirement is audited and never means delivery or task success.
    #[serde(default)]
    pub retire_expired_uncertain: bool,
    pub max_attempts: u32,
    pub retry_delay_ms: u64,
    pub max_response_bytes: u64,
}

fn identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.:/".contains(&b))
}

impl ObservationExportPolicy {
    pub fn validate_for(&self, bindings: &[EndpointBinding]) -> Result<()> {
        validate_sha256(
            &self.protocol_evidence_sha256,
            "observation protocol evidence",
        )?;
        if self.contract_version != OBSERVATION_DELIVERY_VERSION
            || !identity(&self.expected_devspace)
            || self.allowed_scopes.is_empty()
            || self.allowed_scopes.len() > 32
            || self.allowed_scopes.iter().any(|scope| !identity(scope))
            || self
                .allowed_scopes
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || !(1..=128).contains(&self.max_retained_deliveries)
            || !(1..=2_592_000_000).contains(&self.retention_ms)
            || !(1..=16).contains(&self.max_attempts)
            || !(1..=60_000).contains(&self.retry_delay_ms)
            || !(256..=65_536).contains(&self.max_response_bytes)
        {
            return invalid("invalid observation export policy identity or limits");
        }
        let binding = bindings
            .iter()
            .find(|binding| binding.id == self.binding_id)
            .ok_or_else(|| crate::ContractError("observation binding is absent".into()))?;
        binding.validate()?;
        if binding.class != WorkloadClass::BehaviorUpdates
            || !matches!(binding.endpoint, EndpointLocator::Url { .. })
            || binding.credential.is_none()
            || binding.limits.timeout_ms > 60_000
            || binding.limits.max_concurrency != 1
        {
            return invalid(
                "observation export requires a bounded authenticated behavior-update URL binding",
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ObservationSource {
    TaskAttempt {
        request: ReadTaskAttempt,
        expected_status: TaskAttemptStatus,
        expected_outcome_sha256: Option<String>,
    },
    AttunementJob {
        request: ReadAttunement,
        expected_revision: u64,
        expected_plan_sha256: String,
    },
}
impl ObservationSource {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::TaskAttempt {
                request,
                expected_status,
                expected_outcome_sha256,
            } => {
                request.validate()?;
                if *expected_status == TaskAttemptStatus::Reserved
                    || (*expected_status == TaskAttemptStatus::Accepted)
                        != expected_outcome_sha256.is_some()
                {
                    return invalid("observation source must identify a terminal attempt");
                }
                if let Some(digest) = expected_outcome_sha256 {
                    validate_sha256(digest, "observed attempt outcome")?;
                }
                Ok(())
            }
            Self::AttunementJob {
                request,
                expected_revision,
                expected_plan_sha256,
            } => {
                request.validate()?;
                if *expected_revision == 0 {
                    return invalid("observed job revision must be positive");
                }
                validate_sha256(expected_plan_sha256, "observed job plan")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrepareObservationDelivery {
    pub contract_version: u16,
    pub delivery_id: CorrelationId,
    pub source: ObservationSource,
    pub expected_configuration_sha256: String,
}
impl PrepareObservationDelivery {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != OBSERVATION_DELIVERY_VERSION {
            return invalid("unsupported observation delivery version");
        }
        self.source.validate()?;
        validate_sha256(
            &self.expected_configuration_sha256,
            "observation configuration",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliverObservation {
    pub contract_version: u16,
    pub delivery_id: CorrelationId,
    pub expected_payload_sha256: String,
    pub expected_configuration_sha256: String,
}
impl DeliverObservation {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != OBSERVATION_DELIVERY_VERSION {
            return invalid("unsupported observation delivery version");
        }
        validate_sha256(&self.expected_payload_sha256, "observation payload")?;
        validate_sha256(
            &self.expected_configuration_sha256,
            "observation configuration",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadObservationDelivery {
    pub contract_version: u16,
    pub delivery_id: CorrelationId,
}
impl ReadObservationDelivery {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != OBSERVATION_DELIVERY_VERSION {
            return invalid("unsupported observation delivery version");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationDeliveryState {
    Prepared,
    InFlight,
    Uncertain,
    Acknowledged,
    NotDispatched,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationDeliverySnapshot {
    pub contract_version: u16,
    pub delivery_id: CorrelationId,
    pub source: ObservationSource,
    pub configuration_sha256: String,
    pub source_response_sha256: String,
    pub payload_sha256: String,
    pub event_id: String,
    pub scope: String,
    pub state: ObservationDeliveryState,
    pub attempts: u32,
    pub prepared_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub next_attempt_at_unix_ms: u64,
    pub lease_expires_at_unix_ms: Option<u64>,
    pub http_status: Option<u16>,
    /// Digest of the bounded actual sink reply, not a task outcome.
    pub acknowledgement_sha256: Option<String>,
    /// Public bounded classification only; never transport error text or secrets.
    pub disposition: Option<ObservationDeliveryDisposition>,
}

impl ObservationDeliverySnapshot {
    pub fn validate_prepare_identity(&self, request: &PrepareObservationDelivery) -> Result<()> {
        request.validate()?;
        self.validate()?;
        if self.delivery_id != request.delivery_id
            || self.source != request.source
            || self.configuration_sha256 != request.expected_configuration_sha256
        {
            return invalid("observation preparation response identity mismatch");
        }
        Ok(())
    }
    pub fn validate_delivery_identity(&self, request: &DeliverObservation) -> Result<()> {
        request.validate()?;
        self.validate()?;
        if self.delivery_id != request.delivery_id
            || self.payload_sha256 != request.expected_payload_sha256
            || self.configuration_sha256 != request.expected_configuration_sha256
        {
            return invalid("observation delivery response identity mismatch");
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.source.validate()?;
        for digest in [
            &self.configuration_sha256,
            &self.source_response_sha256,
            &self.payload_sha256,
        ] {
            validate_sha256(digest, "observation delivery identity")?;
        }
        if let Some(digest) = &self.acknowledgement_sha256 {
            validate_sha256(digest, "observation acknowledgement")?;
        }
        let event_id_valid = self.event_id.len() == 36
            && self.event_id.bytes().enumerate().all(|(index, b)| {
                if [8, 13, 18, 23].contains(&index) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            });
        if self.contract_version != OBSERVATION_DELIVERY_VERSION
            || !event_id_valid
            || !identity(&self.scope)
            || self.prepared_at_unix_ms == 0
            || self.expires_at_unix_ms <= self.prepared_at_unix_ms
            || self.attempts > 16
            || self.next_attempt_at_unix_ms < self.prepared_at_unix_ms
            || self
                .http_status
                .is_some_and(|status| !(100..=599).contains(&status))
        {
            return invalid("invalid observation delivery snapshot");
        }
        let in_flight = self.state == ObservationDeliveryState::InFlight;
        let acknowledged = self.state == ObservationDeliveryState::Acknowledged;
        if in_flight != self.lease_expires_at_unix_ms.is_some()
            || self
                .lease_expires_at_unix_ms
                .is_some_and(|at| at <= self.prepared_at_unix_ms)
            || acknowledged != self.acknowledgement_sha256.is_some()
            || acknowledged
                != (self.disposition == Some(ObservationDeliveryDisposition::Acknowledged))
            || acknowledged && self.http_status != Some(200)
            || (self.state == ObservationDeliveryState::Prepared) != (self.attempts == 0)
            || self.state == ObservationDeliveryState::Prepared
                && (self.http_status.is_some() || self.disposition.is_some())
        {
            return invalid("inconsistent observation delivery receipt state");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationDeliveryDisposition {
    CredentialUnavailable,
    TransportUnavailable,
    ReplyRejected,
    ReplyTooLarge,
    DeadlineExceeded,
    Acknowledged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationDeliveryReadResult {
    /// None means absent or outside retained history, never successful delivery.
    pub delivery: Option<ObservationDeliverySnapshot>,
    /// Exact local public source response captured at preparation. Returned only
    /// after current source-read authorization; never included in sink exports.
    pub source_response_json: Option<String>,
}
impl ObservationDeliveryReadResult {
    pub fn validate_request_identity(&self, request: &ReadObservationDelivery) -> Result<()> {
        request.validate()?;
        self.validate()?;
        if self
            .delivery
            .as_ref()
            .is_some_and(|delivery| delivery.delivery_id != request.delivery_id)
        {
            return invalid("observation read response identity mismatch");
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        if self.delivery.is_some() != self.source_response_json.is_some() {
            return invalid("observation read source/receipt presence mismatch");
        }
        if let Some(delivery) = &self.delivery {
            delivery.validate()?;
            let source = self.source_response_json.as_deref().unwrap_or_default();
            if source.len() > MAX_OBSERVATION_SOURCE_BYTES
                || crate::sha256_bytes(source.as_bytes()) != delivery.source_response_sha256
            {
                return invalid("observation read source digest or size mismatch");
            }
            match &delivery.source {
                ObservationSource::TaskAttempt {
                    request,
                    expected_status,
                    expected_outcome_sha256,
                } => {
                    let result: crate::TaskAttemptView =
                        serde_json::from_str(source).map_err(|_| {
                            crate::ContractError("invalid retained attempt response".into())
                        })?;
                    result.validate()?;
                    if !result.attempt.as_ref().is_some_and(|attempt| {
                        attempt.attempt_id == request.attempt_id
                            && attempt.status == *expected_status
                            && attempt.outcome_sha256 == *expected_outcome_sha256
                    }) {
                        return invalid("retained attempt source identity mismatch");
                    }
                }
                ObservationSource::AttunementJob {
                    request,
                    expected_revision,
                    expected_plan_sha256,
                } => {
                    let result: crate::AttunementReadResult = serde_json::from_str(source)
                        .map_err(|_| {
                            crate::ContractError("invalid retained job response".into())
                        })?;
                    result.validate()?;
                    if result.status.job.id != request.job_id
                        || result.status.job.revision != *expected_revision
                        || result.status.job.plan_sha256 != *expected_plan_sha256
                    {
                        return invalid("retained job source identity mismatch");
                    }
                }
            }
        }
        Ok(())
    }
}
