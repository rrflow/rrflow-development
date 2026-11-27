//! Request and response envelopes and error bodies.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope<T> {
    pub protocol: String,
    pub protocol_version: u16,
    pub context: RequestContext,
    pub resource: ResourcePath,
    pub payload: T,
}

impl<T> RequestEnvelope<T> {
    pub fn validate(&self, mutation: bool) -> Result<()> {
        validate_protocol(&self.protocol, self.protocol_version)?;
        self.context.validate(mutation)?;
        self.resource.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidArgument,
    NotFound,
    AlreadyExists,
    Conflict,
    FailedPrecondition,
    Unauthenticated,
    PermissionDenied,
    ResourceExhausted,
    DeadlineExceeded,
    Cancelled,
    Unavailable,
    Corruption,
    UnsupportedVersion,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<CanonicalId, String>,
}

impl ErrorBody {
    pub fn validate(&self) -> Result<()> {
        if self.message.is_empty() || self.message.len() > MAX_MESSAGE_BYTES {
            return invalid(format!(
                "error message length must be in 1..={MAX_MESSAGE_BYTES} bytes"
            ));
        }
        if self
            .details
            .values()
            .any(|value| value.len() > MAX_MESSAGE_BYTES)
        {
            return invalid(format!(
                "error detail values must be at most {MAX_MESSAGE_BYTES} bytes"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ResponseOutcome<T> {
    Ok { payload: T },
    Error { error: ErrorBody },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResponseEnvelope<T> {
    pub protocol: String,
    pub protocol_version: u16,
    pub request_id: CorrelationId,
    pub operation_id: CorrelationId,
    pub outcome: ResponseOutcome<T>,
}

impl<T> ResponseEnvelope<T> {
    pub fn validate(&self) -> Result<()> {
        validate_protocol(&self.protocol, self.protocol_version)?;
        if let ResponseOutcome::Error { error } = &self.outcome {
            error.validate()?;
        }
        Ok(())
    }
}
