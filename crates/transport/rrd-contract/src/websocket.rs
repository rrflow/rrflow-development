//! Closed, transport-neutral application protocol carried by one RRD WebSocket.
//!
//! RFC 6455 framing remains a carriage concern: one RRFlow application message
//! is one encoded [`WebSocketFrame`], independent of fragmentation performed by
//! a WebSocket implementation. Durable subscription cursors and generations
//! remain engine semantics; connection sequence and correlation state remain
//! transport-local.

use super::*;
use std::collections::BTreeSet;

pub const WEBSOCKET_CONTRACT_VERSION: u16 = 1;
pub const MAX_WEBSOCKET_MESSAGE_BYTES: usize = 1024 * 1024;
pub const MAX_WEBSOCKET_FRAME_BYTES: usize = 1024 * 1024;
pub const MAX_WEBSOCKET_BUFFER_BYTES: usize = 1024 * 1024;
pub const MAX_WEBSOCKET_WRITE_BUFFER_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_WEBSOCKET_IN_FLIGHT_REQUESTS: u16 = 128;
pub const MAX_WEBSOCKET_SUBSCRIPTIONS: u16 = 64;
pub const MAX_WEBSOCKET_ERROR_BYTES: usize = 16 * 1024;
pub const MAX_WEBSOCKET_ERROR_DETAILS: usize = 32;
pub const MAX_WEBSOCKET_CANCEL_REASON_BYTES: usize = 1024;
pub const MAX_WEBSOCKET_JSON_DEPTH: usize = 32;
pub const MAX_WEBSOCKET_JSON_ITEMS: usize = 16_384;
pub const MIN_WEBSOCKET_HEARTBEAT_MS: u64 = 100;
pub const MAX_WEBSOCKET_TIMEOUT_MS: u64 = 300_000;
pub const MAX_WEBSOCKET_RETRY_AFTER_MS: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketPeer {
    Client,
    Server,
}

/// Negotiated availability limits for one authenticated connection. These
/// values constrain carriage and multiplexing; they never grant engine access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketLimits {
    pub max_message_bytes: u32,
    pub max_frame_bytes: u32,
    pub read_buffer_bytes: u32,
    pub write_buffer_bytes: u32,
    pub max_write_buffer_bytes: u32,
    pub max_in_flight_requests: u16,
    pub max_subscriptions: u16,
    pub send_timeout_ms: u64,
    pub receive_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
}

impl Default for WebSocketLimits {
    fn default() -> Self {
        Self {
            max_message_bytes: MAX_WEBSOCKET_MESSAGE_BYTES as u32,
            max_frame_bytes: MAX_WEBSOCKET_FRAME_BYTES as u32,
            read_buffer_bytes: 64 * 1024,
            write_buffer_bytes: 64 * 1024,
            max_write_buffer_bytes: MAX_WEBSOCKET_WRITE_BUFFER_BYTES as u32,
            max_in_flight_requests: 64,
            max_subscriptions: 32,
            send_timeout_ms: 5_000,
            receive_timeout_ms: 30_000,
            heartbeat_interval_ms: 5_000,
        }
    }
}

impl WebSocketLimits {
    pub fn validate(&self) -> Result<()> {
        let message = self.max_message_bytes as usize;
        let frame = self.max_frame_bytes as usize;
        let read = self.read_buffer_bytes as usize;
        let write = self.write_buffer_bytes as usize;
        let max_write = self.max_write_buffer_bytes as usize;
        if message == 0
            || message > MAX_WEBSOCKET_MESSAGE_BYTES
            || frame == 0
            || frame > MAX_WEBSOCKET_FRAME_BYTES
            || frame > message
            || read == 0
            || read > MAX_WEBSOCKET_BUFFER_BYTES
            || write == 0
            || write > MAX_WEBSOCKET_BUFFER_BYTES
            || max_write <= write
            || max_write > MAX_WEBSOCKET_WRITE_BUFFER_BYTES
            || max_write < write.saturating_add(message)
        {
            return invalid("WebSocket byte and buffer limits are inconsistent");
        }
        if self.max_in_flight_requests == 0
            || self.max_in_flight_requests > MAX_WEBSOCKET_IN_FLIGHT_REQUESTS
            || self.max_subscriptions == 0
            || self.max_subscriptions > MAX_WEBSOCKET_SUBSCRIPTIONS
        {
            return invalid("WebSocket multiplexing limits are outside supported bounds");
        }
        if self.send_timeout_ms == 0
            || self.send_timeout_ms > MAX_WEBSOCKET_TIMEOUT_MS
            || self.receive_timeout_ms == 0
            || self.receive_timeout_ms > MAX_WEBSOCKET_TIMEOUT_MS
            || self.heartbeat_interval_ms < MIN_WEBSOCKET_HEARTBEAT_MS
            || self.heartbeat_interval_ms >= self.receive_timeout_ms
        {
            return invalid("WebSocket timeout and heartbeat limits are inconsistent");
        }
        Ok(())
    }

    fn admitted_by(&self, local: &Self) -> Result<()> {
        self.validate()?;
        local.validate()?;
        if self.max_message_bytes > local.max_message_bytes
            || self.max_frame_bytes > local.max_frame_bytes
            || self.read_buffer_bytes > local.read_buffer_bytes
            || self.write_buffer_bytes > local.write_buffer_bytes
            || self.max_write_buffer_bytes > local.max_write_buffer_bytes
            || self.max_in_flight_requests > local.max_in_flight_requests
            || self.max_subscriptions > local.max_subscriptions
            || self.send_timeout_ms > local.send_timeout_ms
            || self.receive_timeout_ms > local.receive_timeout_ms
            || self.heartbeat_interval_ms > local.heartbeat_interval_ms
        {
            return invalid("peer WebSocket limits exceed the local admission limits");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketFrame {
    pub protocol: String,
    pub protocol_version: u16,
    pub connection_id: CorrelationId,
    pub sequence: u64,
    pub payload: WebSocketPayload,
}

impl WebSocketFrame {
    pub fn validate(&self, sender: WebSocketPeer, limits: &WebSocketLimits) -> Result<()> {
        validate_protocol(&self.protocol, self.protocol_version)?;
        limits.validate()?;
        if self.sequence == 0 {
            return invalid("WebSocket frame sequence must be greater than zero");
        }
        self.payload.validate(sender, limits)?;
        if matches!(self.payload, WebSocketPayload::Connected(_)) && self.sequence != 1 {
            return invalid("the connected frame must be the first server message");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "body",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WebSocketPayload {
    Connected(WebSocketConnected),
    Request(WebSocketRequest),
    Response(WebSocketResponse),
    Cancel(WebSocketCancel),
    Cancellation(WebSocketCancellation),
    Subscribe(WebSocketSubscribe),
    Subscribed(WebSocketSubscribed),
    Delivery(WebSocketDelivery),
    Ack(SubscriptionAcknowledgement),
    Acknowledged(WebSocketAcknowledged),
    Heartbeat(WebSocketHeartbeat),
    Unsubscribe(WebSocketUnsubscribe),
    Unsubscribed(WebSocketUnsubscribed),
    Error(WebSocketError),
    Backpressure(WebSocketBackpressure),
}

impl WebSocketPayload {
    fn validate(&self, sender: WebSocketPeer, limits: &WebSocketLimits) -> Result<()> {
        let direction_is_valid = matches!(
            (sender, self),
            (WebSocketPeer::Server, Self::Connected(_))
                | (WebSocketPeer::Client, Self::Request(_))
                | (WebSocketPeer::Server, Self::Response(_))
                | (WebSocketPeer::Client, Self::Cancel(_))
                | (WebSocketPeer::Server, Self::Cancellation(_))
                | (WebSocketPeer::Client, Self::Subscribe(_))
                | (WebSocketPeer::Server, Self::Subscribed(_))
                | (WebSocketPeer::Server, Self::Delivery(_))
                | (WebSocketPeer::Client, Self::Ack(_))
                | (WebSocketPeer::Server, Self::Acknowledged(_))
                | (_, Self::Heartbeat(_))
                | (WebSocketPeer::Client, Self::Unsubscribe(_))
                | (WebSocketPeer::Server, Self::Unsubscribed(_))
                | (WebSocketPeer::Server, Self::Error(_))
                | (WebSocketPeer::Server, Self::Backpressure(_))
        );
        if !direction_is_valid {
            return invalid("WebSocket payload is not valid in this sender direction");
        }
        match self {
            Self::Connected(value) => value.validate(limits),
            Self::Request(value) => value.validate(),
            Self::Response(value) => value.validate(),
            Self::Cancel(value) => value.validate(),
            Self::Cancellation(value) => value.validate(),
            Self::Subscribe(value) => value.resume.validate(),
            Self::Subscribed(value) => validate_subscription_snapshot(&value.subscription, true),
            Self::Delivery(value) => value.validate(),
            Self::Ack(value) => value.validate(),
            Self::Acknowledged(value) => value.validate(),
            Self::Heartbeat(value) => value.validate(limits),
            Self::Unsubscribe(value) => value.validate(),
            Self::Unsubscribed(value) => validate_subscription_snapshot(&value.subscription, true),
            Self::Error(value) => value.validate(),
            Self::Backpressure(value) => value.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketConnected {
    pub contract_version: u16,
    pub session_id: CorrelationId,
    pub limits: WebSocketLimits,
}

impl WebSocketConnected {
    fn validate(&self, admission: &WebSocketLimits) -> Result<()> {
        if self.contract_version != WEBSOCKET_CONTRACT_VERSION {
            return invalid(format!(
                "unsupported WebSocket contract version {}; expected {WEBSOCKET_CONTRACT_VERSION}",
                self.contract_version
            ));
        }
        self.limits.admitted_by(admission)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketRequestTarget {
    pub operation: CanonicalId,
    pub request_id: CorrelationId,
    pub operation_id: CorrelationId,
}

impl WebSocketRequestTarget {
    fn validate(&self) -> Result<()> {
        if !endpoint_catalogue()
            .endpoints
            .iter()
            .any(|descriptor| descriptor.operation == self.operation)
        {
            return invalid("WebSocket request target operation is not catalogued");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketRequest {
    pub operation: CanonicalId,
    pub request: RequestEnvelope<serde_json::Value>,
}

impl WebSocketRequest {
    pub fn target(&self) -> WebSocketRequestTarget {
        WebSocketRequestTarget {
            operation: self.operation.clone(),
            request_id: self.request.context.request_id.clone(),
            operation_id: self.request.context.operation_id.clone(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        let catalogue = endpoint_catalogue();
        let descriptor = catalogue
            .endpoints
            .iter()
            .find(|descriptor| descriptor.operation == self.operation)
            .ok_or_else(|| ContractError("WebSocket request operation is not catalogued".into()))?;
        self.request.validate(descriptor.mutation)?;
        validate_json_value(&self.request.payload)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketResponse {
    pub operation: CanonicalId,
    pub response: ResponseEnvelope<serde_json::Value>,
}

impl WebSocketResponse {
    pub fn target(&self) -> WebSocketRequestTarget {
        WebSocketRequestTarget {
            operation: self.operation.clone(),
            request_id: self.response.request_id.clone(),
            operation_id: self.response.operation_id.clone(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !endpoint_catalogue()
            .endpoints
            .iter()
            .any(|descriptor| descriptor.operation == self.operation)
        {
            return invalid("WebSocket response operation is not catalogued");
        }
        self.response.validate()?;
        if let ResponseOutcome::Ok { payload } = &self.response.outcome {
            validate_json_value(payload)?;
        }
        Ok(())
    }
}

pub fn validate_websocket_response_correlation(
    request: &WebSocketRequest,
    response: &WebSocketResponse,
) -> Result<()> {
    request.validate()?;
    response.validate()?;
    if request.target() != response.target() {
        return invalid("WebSocket response does not match its request coordinates");
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketCancel {
    pub cancellation_id: CorrelationId,
    pub target: WebSocketRequestTarget,
    pub reason: String,
}

impl WebSocketCancel {
    pub fn validate(&self) -> Result<()> {
        self.target.validate()?;
        if self.reason.trim().is_empty() || self.reason.len() > MAX_WEBSOCKET_CANCEL_REASON_BYTES {
            return invalid(format!(
                "WebSocket cancellation reason must contain 1..={MAX_WEBSOCKET_CANCEL_REASON_BYTES} bytes"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketCancellationDisposition {
    Cancelled,
    AlreadyCompleted,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketCancellation {
    pub cancellation_id: CorrelationId,
    pub target: WebSocketRequestTarget,
    pub disposition: WebSocketCancellationDisposition,
}

impl WebSocketCancellation {
    pub fn validate(&self) -> Result<()> {
        self.target.validate()
    }
}

pub fn validate_websocket_cancellation_correlation(
    request: &WebSocketCancel,
    result: &WebSocketCancellation,
) -> Result<()> {
    request.validate()?;
    result.validate()?;
    if request.cancellation_id != result.cancellation_id || request.target != result.target {
        return invalid("WebSocket cancellation result does not match its request coordinates");
    }
    Ok(())
}

/// Exact durable state a reconnecting client last observed. The engine checks
/// every coordinate before fencing the previous connection generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionResume {
    pub subscription_id: CorrelationId,
    pub stream_sha256: String,
    pub connection_generation: u64,
    pub acknowledged_cursor: u64,
}

impl SubscriptionResume {
    pub fn from_snapshot(snapshot: &SubscriptionSnapshot) -> Self {
        Self {
            subscription_id: snapshot.subscription_id.clone(),
            stream_sha256: snapshot.stream_sha256.clone(),
            connection_generation: snapshot.connection_generation,
            acknowledged_cursor: snapshot.acknowledged_cursor,
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.stream_sha256, "subscription resume stream_sha256")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketSubscribe {
    pub resume: SubscriptionResume,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketSubscribed {
    pub subscription: SubscriptionSnapshot,
}

pub fn validate_websocket_subscription_correlation(
    request: &SubscriptionResume,
    subscription: &SubscriptionSnapshot,
) -> Result<()> {
    request.validate()?;
    validate_subscription_snapshot(subscription, true)?;
    if request.subscription_id != subscription.subscription_id
        || request.stream_sha256 != subscription.stream_sha256
        || request.acknowledged_cursor != subscription.acknowledged_cursor
        || request
            .connection_generation
            .checked_add(1)
            .is_none_or(|next| next != subscription.connection_generation)
    {
        return invalid("subscribed result does not match the requested resume coordinates");
    }
    Ok(())
}

/// Engine-produced semantic delivery. It contains no WebSocket connection
/// sequence, socket identity, or carriage state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "stream", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubscriptionDelivery {
    Changefeed { page: ChangefeedPage },
    LiveQuery { delta: LiveQueryDeltaResult },
}

/// One delivery interval durably reserved by the engine for an exact
/// subscription generation. This is semantic delivery state, not a socket
/// frame or connection sequence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionDeliveryBatch {
    pub subscription_id: CorrelationId,
    pub connection_generation: u64,
    pub delivery_sequence: u64,
    pub from_cursor: u64,
    pub through_cursor: u64,
    pub delivery: SubscriptionDelivery,
}

impl SubscriptionDeliveryBatch {
    pub fn validate(&self) -> Result<()> {
        WebSocketDelivery::from(self.clone()).validate()
    }
}

/// Result of polling the engine-owned durable delivery window. Idle state is
/// explicit so the engine never manufactures a transport heartbeat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubscriptionPoll {
    Idle { subscription: SubscriptionSnapshot },
    Delivery { batch: SubscriptionDeliveryBatch },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketDelivery {
    pub subscription_id: CorrelationId,
    pub connection_generation: u64,
    pub delivery_sequence: u64,
    pub from_cursor: u64,
    pub through_cursor: u64,
    pub delivery: SubscriptionDelivery,
}

impl WebSocketDelivery {
    pub fn validate(&self) -> Result<()> {
        if self.connection_generation == 0
            || self.delivery_sequence == 0
            || self.through_cursor <= self.from_cursor
        {
            return invalid("WebSocket delivery coordinates are invalid");
        }
        match &self.delivery {
            SubscriptionDelivery::Changefeed { page } => {
                if page.requested_after_cursor != self.from_cursor
                    || page.through_cursor != self.through_cursor
                    || page.head_cursor < page.through_cursor
                    || page.changes.len() > MAX_CHANGEFEED_PAGE as usize
                    || page
                        .changes
                        .windows(2)
                        .any(|pair| pair[0].cursor >= pair[1].cursor)
                    || page
                        .changes
                        .first()
                        .is_some_and(|change| change.cursor <= self.from_cursor)
                    || page
                        .changes
                        .last()
                        .is_some_and(|change| change.cursor > self.through_cursor)
                {
                    return invalid("changefeed delivery does not match its cursor envelope");
                }
            }
            SubscriptionDelivery::LiveQuery { delta } => {
                let rows = delta
                    .added
                    .len()
                    .saturating_add(delta.updated.len())
                    .saturating_add(delta.removed.len());
                if delta.from_cursor != self.from_cursor
                    || delta.through_cursor != self.through_cursor
                    || delta.head_cursor < delta.through_cursor
                    || delta.timed_out
                    || rows > MAX_LIVE_QUERY_DELTA_ROWS as usize
                {
                    return invalid("live-query delivery does not match its cursor envelope");
                }
            }
        }
        Ok(())
    }
}

impl From<SubscriptionDeliveryBatch> for WebSocketDelivery {
    fn from(value: SubscriptionDeliveryBatch) -> Self {
        Self {
            subscription_id: value.subscription_id,
            connection_generation: value.connection_generation,
            delivery_sequence: value.delivery_sequence,
            from_cursor: value.from_cursor,
            through_cursor: value.through_cursor,
            delivery: value.delivery,
        }
    }
}

/// Exact cumulative acknowledgement applied by `RrdEngine`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionAcknowledgement {
    pub subscription_id: CorrelationId,
    pub connection_generation: u64,
    pub delivery_sequence: u64,
    pub through_cursor: u64,
}

impl SubscriptionAcknowledgement {
    pub fn validate(&self) -> Result<()> {
        if self.connection_generation == 0 || self.delivery_sequence == 0 {
            return invalid(
                "subscription acknowledgement generation and sequence must be positive",
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketAcknowledged {
    pub acknowledgement: SubscriptionAcknowledgement,
    pub subscription: SubscriptionSnapshot,
}

impl WebSocketAcknowledged {
    fn validate(&self) -> Result<()> {
        self.acknowledgement.validate()?;
        validate_subscription_snapshot(&self.subscription, true)?;
        if self.acknowledgement.subscription_id != self.subscription.subscription_id
            || self.acknowledgement.connection_generation != self.subscription.connection_generation
            || self.acknowledgement.through_cursor != self.subscription.acknowledged_cursor
        {
            return invalid("acknowledged result does not match its subscription coordinates");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionLeaseCoordinate {
    pub subscription_id: CorrelationId,
    pub connection_generation: u64,
    pub acknowledged_cursor: u64,
}

impl SubscriptionLeaseCoordinate {
    pub fn from_snapshot(snapshot: &SubscriptionSnapshot) -> Self {
        Self {
            subscription_id: snapshot.subscription_id.clone(),
            connection_generation: snapshot.connection_generation,
            acknowledged_cursor: snapshot.acknowledged_cursor,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.connection_generation == 0 {
            return invalid("subscription lease generation must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketHeartbeat {
    pub heartbeat_id: CorrelationId,
    pub observed_sequence: u64,
    pub subscriptions: Vec<SubscriptionLeaseCoordinate>,
}

impl WebSocketHeartbeat {
    fn validate(&self, limits: &WebSocketLimits) -> Result<()> {
        if self.observed_sequence == 0
            || self.subscriptions.len() > usize::from(limits.max_subscriptions)
        {
            return invalid("WebSocket heartbeat coordinates exceed their bounds");
        }
        let mut ids = BTreeSet::new();
        for subscription in &self.subscriptions {
            subscription.validate()?;
            if !ids.insert(&subscription.subscription_id) {
                return invalid("WebSocket heartbeat subscription identities must be unique");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketUnsubscribe {
    pub subscription_id: CorrelationId,
    pub connection_generation: u64,
}

impl WebSocketUnsubscribe {
    fn validate(&self) -> Result<()> {
        if self.connection_generation == 0 {
            return invalid("unsubscribe connection generation must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketUnsubscribed {
    pub subscription: SubscriptionSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WebSocketErrorTarget {
    Connection,
    Request {
        target: WebSocketRequestTarget,
    },
    Cancellation {
        cancellation_id: CorrelationId,
        target: WebSocketRequestTarget,
    },
    Subscription {
        subscription_id: CorrelationId,
        connection_generation: u64,
    },
}

impl WebSocketErrorTarget {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Request { target } | Self::Cancellation { target, .. } => target.validate()?,
            Self::Connection | Self::Subscription { .. } => {}
        }
        if matches!(
            self,
            Self::Subscription {
                connection_generation: 0,
                ..
            }
        ) {
            return invalid("subscription error generation must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketError {
    pub target: WebSocketErrorTarget,
    pub error: ErrorBody,
}

impl WebSocketError {
    fn validate(&self) -> Result<()> {
        self.target.validate()?;
        self.error.validate()?;
        if self.error.details.len() > MAX_WEBSOCKET_ERROR_DETAILS
            || serde_json::to_vec(&self.error)
                .map_err(|error| ContractError(error.to_string()))?
                .len()
                > MAX_WEBSOCKET_ERROR_BYTES
        {
            return invalid("WebSocket error exceeds its detail or encoded-byte bound");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WebSocketBackpressureTarget {
    Request {
        target: WebSocketRequestTarget,
    },
    Subscription {
        subscription_id: CorrelationId,
        connection_generation: u64,
    },
}

impl WebSocketBackpressureTarget {
    fn validate(&self) -> Result<()> {
        if let Self::Request { target } = self {
            target.validate()?;
        }
        if matches!(
            self,
            Self::Subscription {
                connection_generation: 0,
                ..
            }
        ) {
            return invalid("subscription backpressure generation must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSocketBackpressure {
    pub target: WebSocketBackpressureTarget,
    pub in_flight: u32,
    pub limit: u32,
    pub retry_after_ms: u64,
}

impl WebSocketBackpressure {
    fn validate(&self) -> Result<()> {
        self.target.validate()?;
        if self.limit == 0
            || self.in_flight < self.limit
            || self.retry_after_ms == 0
            || self.retry_after_ms > MAX_WEBSOCKET_RETRY_AFTER_MS
        {
            return invalid("WebSocket backpressure coordinates are invalid");
        }
        Ok(())
    }
}

pub fn encode_websocket_frame(
    frame: &WebSocketFrame,
    sender: WebSocketPeer,
    limits: &WebSocketLimits,
) -> Result<Vec<u8>> {
    frame.validate(sender, limits)?;
    let bytes = serde_json::to_vec(frame).map_err(|error| ContractError(error.to_string()))?;
    if bytes.len() > limits.max_message_bytes as usize {
        return invalid("encoded WebSocket message exceeds the negotiated byte limit");
    }
    Ok(bytes)
}

pub fn decode_websocket_frame(
    bytes: &[u8],
    sender: WebSocketPeer,
    limits: &WebSocketLimits,
) -> Result<WebSocketFrame> {
    limits.validate()?;
    if bytes.len() > limits.max_message_bytes as usize {
        return invalid("WebSocket message exceeds the negotiated byte limit");
    }
    let frame = serde_json::from_slice::<WebSocketFrame>(bytes)
        .map_err(|error| ContractError(format!("invalid WebSocket frame: {error}")))?;
    frame.validate(sender, limits)?;
    Ok(frame)
}

/// Transport-local monotonically sequenced encoder. It is deliberately not
/// serializable and must never be persisted by `RrdEngine`.
#[derive(Debug, Clone)]
pub struct WebSocketSendState {
    sender: WebSocketPeer,
    connection_id: CorrelationId,
    limits: WebSocketLimits,
    last_sequence: u64,
}

impl WebSocketSendState {
    pub fn new(
        sender: WebSocketPeer,
        connection_id: CorrelationId,
        limits: WebSocketLimits,
    ) -> Result<Self> {
        limits.validate()?;
        Ok(Self {
            sender,
            connection_id,
            limits,
            last_sequence: 0,
        })
    }

    pub fn encode(&mut self, payload: WebSocketPayload) -> Result<Vec<u8>> {
        let sequence = self
            .last_sequence
            .checked_add(1)
            .ok_or_else(|| ContractError("WebSocket send sequence overflowed".into()))?;
        if self.sender == WebSocketPeer::Server
            && sequence == 1
            && !matches!(payload, WebSocketPayload::Connected(_))
        {
            return invalid("the first server application message must be connected");
        }
        let frame = WebSocketFrame {
            protocol: PROTOCOL.into(),
            protocol_version: PROTOCOL_VERSION,
            connection_id: self.connection_id.clone(),
            sequence,
            payload,
        };
        let bytes = encode_websocket_frame(&frame, self.sender, &self.limits)?;
        self.last_sequence = sequence;
        Ok(bytes)
    }

    pub fn last_sequence(&self) -> u64 {
        self.last_sequence
    }
}

/// Transport-local decoder that rejects replay, gaps, cross-connection
/// substitution, and a server stream that does not begin with `connected`.
#[derive(Debug, Clone)]
pub struct WebSocketReceiveState {
    sender: WebSocketPeer,
    connection_id: Option<CorrelationId>,
    limits: WebSocketLimits,
    last_sequence: u64,
}

impl WebSocketReceiveState {
    pub fn awaiting_server(limits: WebSocketLimits) -> Result<Self> {
        limits.validate()?;
        Ok(Self {
            sender: WebSocketPeer::Server,
            connection_id: None,
            limits,
            last_sequence: 0,
        })
    }

    pub fn for_connection(
        sender: WebSocketPeer,
        connection_id: CorrelationId,
        limits: WebSocketLimits,
    ) -> Result<Self> {
        limits.validate()?;
        Ok(Self {
            sender,
            connection_id: Some(connection_id),
            limits,
            last_sequence: 0,
        })
    }

    pub fn accept(&mut self, bytes: &[u8]) -> Result<WebSocketFrame> {
        let frame = decode_websocket_frame(bytes, self.sender, &self.limits)?;
        let expected = self
            .last_sequence
            .checked_add(1)
            .ok_or_else(|| ContractError("WebSocket receive sequence overflowed".into()))?;
        if frame.sequence != expected {
            return invalid(format!(
                "WebSocket sequence {} is not the expected {expected}",
                frame.sequence
            ));
        }
        match &self.connection_id {
            Some(connection_id) if *connection_id != frame.connection_id => {
                return invalid("WebSocket frame carries another connection identity");
            }
            Some(_) if self.sender == WebSocketPeer::Server && expected == 1 => {
                if !matches!(frame.payload, WebSocketPayload::Connected(_)) {
                    return invalid("the first server application message must be connected");
                }
            }
            Some(_) => {}
            None => {
                let WebSocketPayload::Connected(connected) = &frame.payload else {
                    return invalid("the server stream did not begin with connected");
                };
                connected.limits.admitted_by(&self.limits)?;
                self.limits = connected.limits.clone();
                self.connection_id = Some(frame.connection_id.clone());
            }
        }
        self.last_sequence = frame.sequence;
        Ok(frame)
    }

    pub fn connection_id(&self) -> Option<&CorrelationId> {
        self.connection_id.as_ref()
    }

    pub fn last_sequence(&self) -> u64 {
        self.last_sequence
    }
}

pub fn validate_subscription_snapshot(
    subscription: &SubscriptionSnapshot,
    connected: bool,
) -> Result<()> {
    validate_sha256(&subscription.stream_sha256, "subscription stream_sha256")?;
    if subscription.acknowledged_cursor > subscription.head_cursor
        || subscription.retention_floor_cursor > subscription.acknowledged_cursor
        || subscription.batch_size == 0
        || subscription.batch_size > MAX_CHANGEFEED_PAGE
        || subscription.max_in_flight == 0
        || subscription.max_in_flight > MAX_SUBSCRIPTION_IN_FLIGHT
        || subscription.heartbeat_interval_ms < MIN_SUBSCRIPTION_HEARTBEAT_MS
        || subscription.heartbeat_interval_ms > MAX_SUBSCRIPTION_HEARTBEAT_MS
        || (connected && subscription.connection_generation == 0)
        || (!connected && subscription.connection_generation != 0)
        || (subscription.status == SubscriptionStatus::Open
            && subscription.lease_expires_at_unix_ms == 0)
    {
        return invalid("subscription snapshot coordinates are inconsistent");
    }
    Ok(())
}

fn validate_json_value(value: &serde_json::Value) -> Result<()> {
    fn walk(value: &serde_json::Value, depth: usize, items: &mut usize) -> Result<()> {
        *items = items.saturating_add(1);
        if depth > MAX_WEBSOCKET_JSON_DEPTH || *items > MAX_WEBSOCKET_JSON_ITEMS {
            return invalid("WebSocket JSON payload exceeds its depth or item bound");
        }
        match value {
            serde_json::Value::String(value) if value.len() > MAX_WEBSOCKET_MESSAGE_BYTES => {
                invalid("WebSocket JSON string exceeds the message byte bound")
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    walk(value, depth + 1, items)?;
                }
                Ok(())
            }
            serde_json::Value::Object(values) => {
                for (name, value) in values {
                    if name.len() > MAX_ID_BYTES {
                        return invalid("WebSocket JSON object key exceeds the identifier bound");
                    }
                    walk(value, depth + 1, items)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    walk(value, 0, &mut 0)
}
