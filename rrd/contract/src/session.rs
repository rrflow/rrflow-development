//! Sessions, leases and transaction leases.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Resource limits requested for one loopback transport session. These are
/// availability boundaries, not authentication or authorization policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SessionLimits {
    pub idle_timeout_ms: u64,
    pub absolute_timeout_ms: u64,
    pub max_open_transactions: u16,
}

impl SessionLimits {
    pub fn validate(&self) -> Result<()> {
        if !(MIN_LEASE_MS..=MAX_LEASE_MS).contains(&self.idle_timeout_ms)
            || !(MIN_LEASE_MS..=MAX_LEASE_MS).contains(&self.absolute_timeout_ms)
            || self.idle_timeout_ms > self.absolute_timeout_ms
        {
            return invalid("session timeouts are outside bounds or idle exceeds absolute");
        }
        if self.max_open_transactions == 0 || self.max_open_transactions > 256 {
            return invalid("max_open_transactions must be in 1..=256");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateSession {
    pub limits: SessionLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SessionLease {
    pub session_id: CorrelationId,
    pub token: CorrelationId,
    pub issued_at_unix_ms: u64,
    pub idle_expires_at_unix_ms: u64,
    pub absolute_expires_at_unix_ms: u64,
    pub limits: SessionLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenewSession {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloseSession {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AbortTransaction {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionEndState {
    Closed,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SessionTermination {
    pub session_id: CorrelationId,
    pub state: SessionEndState,
    pub ended_at_unix_ms: u64,
    pub affected_open_transactions: u16,
    pub idempotent_replay: bool,
}

impl SessionTermination {
    pub fn validate(&self) -> Result<()> {
        if self.ended_at_unix_ms == 0 {
            return invalid("session termination time must be greater than zero");
        }
        Ok(())
    }
}

impl SessionLease {
    pub fn validate(&self) -> Result<()> {
        self.limits.validate()?;
        if self.issued_at_unix_ms == 0
            || self.idle_expires_at_unix_ms <= self.issued_at_unix_ms
            || self.absolute_expires_at_unix_ms < self.idle_expires_at_unix_ms
        {
            return invalid("session lease timestamps are inconsistent");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BeginTransaction {
    pub scope: CanonicalId,
    pub timeout_ms: u64,
}

impl BeginTransaction {
    pub fn validate(&self) -> Result<()> {
        if !(MIN_LEASE_MS..=MAX_LEASE_MS).contains(&self.timeout_ms) {
            return invalid(format!(
                "transaction timeout must be in {MIN_LEASE_MS}..={MAX_LEASE_MS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransactionState {
    Open,
    Committed,
    Aborted,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransactionLease {
    pub transaction_id: CorrelationId,
    pub session_id: CorrelationId,
    pub scope: CanonicalId,
    pub read_cursor: u64,
    pub expires_at_unix_ms: u64,
    pub state: TransactionState,
}

impl TransactionLease {
    pub fn validate(&self) -> Result<()> {
        if self.expires_at_unix_ms == 0 {
            return invalid("transaction expiry must be greater than zero");
        }
        Ok(())
    }
}
