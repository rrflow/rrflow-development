//! Changefeed reads, follows and subscriptions.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadChangefeed {
    pub scope: String,
    pub after_cursor: u64,
    pub limit: u64,
}

impl ReadChangefeed {
    pub fn validate(&self) -> Result<()> {
        if self.scope.is_empty()
            || self.scope.len() > MAX_ID_BYTES
            || self.scope.as_bytes().contains(&0)
        {
            return invalid("changefeed scope is invalid");
        }
        if self.limit == 0 || self.limit > MAX_CHANGEFEED_PAGE {
            return invalid(format!(
                "changefeed limit must be in 1..={MAX_CHANGEFEED_PAGE}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimTierSnapshot {
    Local,
    Primary,
    Tenant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimPromotionSnapshot {
    Unpromoted,
    Pending,
    Promoted,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClaimChangeSnapshot {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u64>,
    pub tx_time: u64,
    pub producer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub tier: ClaimTierSnapshot,
    pub promotion: ClaimPromotionSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "family", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChangeMutationSnapshot {
    Claim { claim: ClaimChangeSnapshot },
    Data { mutation: TransactionMutation },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeChangeSnapshot {
    pub cursor: u64,
    pub commit_sha256: String,
    pub commit_ordinal: u64,
    pub scope: String,
    pub at_unix_ms: u64,
    pub actor: String,
    pub mutation: ChangeMutationSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_change_sha256: Option<String>,
    pub change_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangefeedValidation {
    pub method: String,
    pub change_reads: u64,
    pub proof_nodes: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangefeedPage {
    pub requested_after_cursor: u64,
    pub through_cursor: u64,
    pub head_cursor: u64,
    pub has_more: bool,
    pub validation: ChangefeedValidation,
    pub changes: Vec<RuntimeChangeSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FollowChangefeed {
    pub read: ReadChangefeed,
    pub wait_timeout_ms: u64,
}

impl FollowChangefeed {
    pub fn validate(&self) -> Result<()> {
        self.read.validate()?;
        if self.wait_timeout_ms == 0 || self.wait_timeout_ms > MAX_CHANGEFEED_WAIT_MS {
            return invalid(format!(
                "changefeed wait_timeout_ms must be in 1..={MAX_CHANGEFEED_WAIT_MS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangefeedFollowResult {
    pub timed_out: bool,
    pub waited_ms: u64,
    pub page: ChangefeedPage,
}

/// The immutable source definition owned by a durable push subscription.
/// Delivery state is deliberately separate: reconnecting never changes which
/// data a subscription means.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "stream", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubscriptionStream {
    Changefeed {
        scope: String,
    },
    LiveQuery {
        scope: String,
        query: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        parameters: BTreeMap<String, QueryValue>,
        #[serde(default)]
        budget: QueryBudget,
        max_delta_rows: u64,
        /// Version of an existing caller snapshot when opening from a cursor.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result_identity_version: Option<u16>,
    },
}

impl SubscriptionStream {
    pub fn validate(&self, after_cursor: u64, batch_size: u64) -> Result<()> {
        match self {
            Self::Changefeed { scope } => ReadChangefeed {
                scope: scope.clone(),
                after_cursor,
                limit: batch_size,
            }
            .validate(),
            Self::LiveQuery {
                scope,
                query,
                parameters,
                budget,
                max_delta_rows,
                result_identity_version,
            } => PollLiveQuery {
                scope: scope.clone(),
                query: query.clone(),
                parameters: parameters.clone(),
                after_cursor,
                result_identity_version: *result_identity_version,
                budget: budget.clone(),
                max_delta_rows: *max_delta_rows,
                wait_timeout_ms: 0,
            }
            .validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenSubscription {
    pub subscription_id: CorrelationId,
    pub stream: SubscriptionStream,
    pub after_cursor: u64,
    pub batch_size: u64,
    pub max_in_flight: u16,
    pub retention_cursor_window: u64,
    pub lease_ms: u64,
    pub heartbeat_interval_ms: u64,
}

impl OpenSubscription {
    pub fn validate(&self) -> Result<()> {
        if self.batch_size == 0 || self.batch_size > MAX_CHANGEFEED_PAGE {
            return invalid(format!(
                "subscription batch_size must be in 1..={MAX_CHANGEFEED_PAGE}"
            ));
        }
        self.stream.validate(self.after_cursor, self.batch_size)?;
        if self.max_in_flight == 0 || self.max_in_flight > MAX_SUBSCRIPTION_IN_FLIGHT {
            return invalid(format!(
                "subscription max_in_flight must be in 1..={MAX_SUBSCRIPTION_IN_FLIGHT}"
            ));
        }
        if self.retention_cursor_window < self.batch_size
            || self.retention_cursor_window > MAX_SUBSCRIPTION_RETENTION_CURSORS
        {
            return invalid(format!(
                "subscription retention_cursor_window must be between batch_size and {MAX_SUBSCRIPTION_RETENTION_CURSORS}"
            ));
        }
        if !(MIN_LEASE_MS..=MAX_LEASE_MS).contains(&self.lease_ms) {
            return invalid(format!(
                "subscription lease_ms must be in {MIN_LEASE_MS}..={MAX_LEASE_MS}"
            ));
        }
        if !(MIN_SUBSCRIPTION_HEARTBEAT_MS..=MAX_SUBSCRIPTION_HEARTBEAT_MS)
            .contains(&self.heartbeat_interval_ms)
        {
            return invalid(format!(
                "subscription heartbeat_interval_ms must be in {MIN_SUBSCRIPTION_HEARTBEAT_MS}..={MAX_SUBSCRIPTION_HEARTBEAT_MS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionSnapshot {
    pub subscription_id: CorrelationId,
    pub stream_sha256: String,
    pub acknowledged_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_identity_version: Option<u16>,
    pub retention_floor_cursor: u64,
    pub head_cursor: u64,
    pub batch_size: u64,
    pub max_in_flight: u16,
    pub connection_generation: u64,
    pub lease_expires_at_unix_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub status: SubscriptionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenSubscriptionResult {
    pub subscription: SubscriptionSnapshot,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloseSubscription {
    pub subscription_id: CorrelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloseSubscriptionResult {
    pub subscription: SubscriptionSnapshot,
    pub idempotent_replay: bool,
}
