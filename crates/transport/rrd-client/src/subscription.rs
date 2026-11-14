//! Multiplexed, bounded RRFlow WebSocket transport.
//!
//! Durable subscription state remains owned by `RrdEngine`. This client keeps
//! only connection-local correlation, ordering, and carriage state.

use crate::error::{contract, websocket_connect_error, websocket_stream_error};
use crate::{Error, RequestOptions, Result, RrdClient, Session};
use futures_util::{SinkExt, StreamExt};
use hyper::Method;
use rrd_contract::{
    validate_websocket_cancellation_correlation, validate_websocket_response_correlation,
    validate_websocket_subscription_correlation, CloseSubscription, CloseSubscriptionResult,
    CorrelationId, OpenSubscription, OpenSubscriptionResult, SubscriptionAcknowledgement,
    SubscriptionLeaseCoordinate, SubscriptionResume, SubscriptionSnapshot,
    WebSocketBackpressureTarget, WebSocketCancel, WebSocketCancellationDisposition,
    WebSocketErrorTarget, WebSocketFrame, WebSocketHeartbeat, WebSocketLimits, WebSocketPayload,
    WebSocketPeer, WebSocketReceiveState, WebSocketRequest, WebSocketRequestTarget,
    WebSocketSendState, WebSocketSubscribe, WebSocketUnsubscribe,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type SubscriptionKey = (CorrelationId, u64);
type AcknowledgementKey = (CorrelationId, u64, u64);

struct ActiveSubscription {
    snapshot: SubscriptionSnapshot,
    last_delivery_sequence: Option<u64>,
    last_through_cursor: u64,
    outstanding_deliveries: BTreeMap<u64, u64>,
}

impl ActiveSubscription {
    fn new(snapshot: SubscriptionSnapshot) -> Self {
        Self {
            last_through_cursor: snapshot.acknowledged_cursor,
            snapshot,
            last_delivery_sequence: None,
            outstanding_deliveries: BTreeMap::new(),
        }
    }
}

/// One authenticated RRFlow WebSocket carrying requests, cancellations,
/// subscriptions, acknowledgements, heartbeat state, and backpressure.
pub struct RrdWebSocket {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    connection_id: CorrelationId,
    session_id: CorrelationId,
    limits: WebSocketLimits,
    sender: WebSocketSendState,
    receiver: WebSocketReceiveState,
    pending_requests: BTreeMap<WebSocketRequestTarget, WebSocketRequest>,
    pending_cancellations: BTreeMap<CorrelationId, WebSocketCancel>,
    pending_subscriptions: BTreeMap<CorrelationId, SubscriptionResume>,
    pending_acknowledgements: BTreeMap<AcknowledgementKey, SubscriptionAcknowledgement>,
    pending_unsubscribes: BTreeSet<SubscriptionKey>,
    subscriptions: BTreeMap<CorrelationId, ActiveSubscription>,
    last_server_observed_sequence: u64,
}

impl RrdClient {
    pub async fn open_subscription(
        &self,
        session: &Session,
        request: OpenSubscription,
        options: RequestOptions,
    ) -> Result<OpenSubscriptionResult> {
        self.session_call(
            Method::POST,
            "/v1/subscriptions/open",
            session,
            request,
            options,
            true,
        )
        .await
    }

    pub async fn close_subscription(
        &self,
        session: &Session,
        request: CloseSubscription,
        options: RequestOptions,
    ) -> Result<CloseSubscriptionResult> {
        self.session_call(
            Method::POST,
            "/v1/subscriptions/close",
            session,
            request,
            options,
            true,
        )
        .await
    }

    /// Establishes the single authenticated RRFlow WebSocket endpoint. The
    /// first application message must be `connected`; all later messages use
    /// the negotiated limits and exact connection identity.
    pub async fn connect_websocket(&self, session: &Session) -> Result<RrdWebSocket> {
        let origin = if let Some(rest) = self.endpoint.strip_prefix("http://") {
            format!("ws://{rest}")
        } else if let Some(rest) = self.endpoint.strip_prefix("https://") {
            format!("wss://{rest}")
        } else {
            return Err(Error::Contract(
                "RRD endpoint has no WebSocket scheme".into(),
            ));
        };
        let mut request = format!("{origin}/v1/ws")
            .into_client_request()
            .map_err(contract)?;
        request.headers_mut().insert(
            "x-rrd-session",
            session.session_id().as_str().parse().map_err(contract)?,
        );
        request.headers_mut().insert(
            "authorization",
            format!("Bearer {}", session.token().as_str())
                .parse()
                .map_err(contract)?,
        );

        let admission = self.config.websocket_limits.clone();
        admission.validate().map_err(contract)?;
        let carriage = WebSocketConfig::default()
            .read_buffer_size(admission.read_buffer_bytes as usize)
            .write_buffer_size(admission.write_buffer_bytes as usize)
            .max_write_buffer_size(admission.max_write_buffer_bytes as usize)
            .max_message_size(Some(admission.max_message_bytes as usize))
            .max_frame_size(Some(admission.max_frame_bytes as usize));
        let connector = match &self.websocket_tls {
            Some(tls) => tokio_tungstenite::Connector::Rustls(tls.clone()),
            None => tokio_tungstenite::Connector::Plain,
        };
        let connected = tokio::time::timeout(
            self.config.request_timeout,
            tokio_tungstenite::connect_async_tls_with_config(
                request,
                Some(carriage),
                false,
                Some(connector),
            ),
        )
        .await
        .map_err(|_| Error::Timeout)?
        .map_err(websocket_connect_error)?;
        let mut stream = connected.0;
        let mut receiver = WebSocketReceiveState::awaiting_server(admission).map_err(contract)?;
        let first = receive_initial(
            &mut stream,
            &mut receiver,
            Instant::now() + self.config.request_timeout,
        )
        .await?;
        let WebSocketPayload::Connected(connected) = first.payload else {
            return Err(Error::WebSocketProtocol(
                "server did not begin with connected".into(),
            ));
        };
        if connected.session_id != *session.session_id() {
            return Err(Error::ResponseIdentityMismatch);
        }
        let connection_id = first.connection_id;
        let limits = connected.limits;
        let sender =
            WebSocketSendState::new(WebSocketPeer::Client, connection_id.clone(), limits.clone())
                .map_err(contract)?;
        Ok(RrdWebSocket {
            stream,
            connection_id,
            session_id: connected.session_id,
            limits,
            sender,
            receiver,
            pending_requests: BTreeMap::new(),
            pending_cancellations: BTreeMap::new(),
            pending_subscriptions: BTreeMap::new(),
            pending_acknowledgements: BTreeMap::new(),
            pending_unsubscribes: BTreeSet::new(),
            subscriptions: BTreeMap::new(),
            last_server_observed_sequence: 0,
        })
    }
}

impl RrdWebSocket {
    pub fn connection_id(&self) -> &CorrelationId {
        &self.connection_id
    }

    pub fn session_id(&self) -> &CorrelationId {
        &self.session_id
    }

    pub fn limits(&self) -> &WebSocketLimits {
        &self.limits
    }

    pub fn subscription(&self, subscription_id: &CorrelationId) -> Option<&SubscriptionSnapshot> {
        self.subscriptions
            .get(subscription_id)
            .map(|active| &active.snapshot)
    }

    pub async fn send_request(
        &mut self,
        request: WebSocketRequest,
    ) -> Result<WebSocketRequestTarget> {
        if self.pending_requests.len() >= usize::from(self.limits.max_in_flight_requests) {
            return Err(Error::WebSocketProtocol(
                "local WebSocket request limit is exhausted".into(),
            ));
        }
        let target = request.target();
        if self.pending_requests.contains_key(&target) {
            return Err(Error::WebSocketProtocol(
                "request coordinates are already in flight".into(),
            ));
        }
        self.send(WebSocketPayload::Request(request.clone()))
            .await?;
        self.pending_requests.insert(target.clone(), request);
        Ok(target)
    }

    pub async fn cancel(&mut self, request: WebSocketCancel) -> Result<()> {
        if !self.pending_requests.contains_key(&request.target) {
            return Err(Error::WebSocketProtocol(
                "cancellation does not target an in-flight request".into(),
            ));
        }
        if self
            .pending_cancellations
            .contains_key(&request.cancellation_id)
        {
            return Err(Error::WebSocketProtocol(
                "cancellation identity is already in flight".into(),
            ));
        }
        self.send(WebSocketPayload::Cancel(request.clone())).await?;
        self.pending_cancellations
            .insert(request.cancellation_id.clone(), request);
        Ok(())
    }

    pub async fn subscribe(&mut self, resume: SubscriptionResume) -> Result<()> {
        let count = self
            .subscriptions
            .len()
            .saturating_add(self.pending_subscriptions.len());
        if count >= usize::from(self.limits.max_subscriptions) {
            return Err(Error::WebSocketProtocol(
                "local WebSocket subscription limit is exhausted".into(),
            ));
        }
        if self.subscriptions.contains_key(&resume.subscription_id)
            || self
                .pending_subscriptions
                .contains_key(&resume.subscription_id)
        {
            return Err(Error::WebSocketProtocol(
                "subscription is already attached or pending".into(),
            ));
        }
        self.send(WebSocketPayload::Subscribe(WebSocketSubscribe {
            resume: resume.clone(),
        }))
        .await?;
        self.pending_subscriptions
            .insert(resume.subscription_id.clone(), resume);
        Ok(())
    }

    pub async fn acknowledge(
        &mut self,
        acknowledgement: SubscriptionAcknowledgement,
    ) -> Result<()> {
        let Some(active) = self.subscriptions.get(&acknowledgement.subscription_id) else {
            return Err(Error::WebSocketProtocol(
                "ACK does not target an active subscription".into(),
            ));
        };
        if active.snapshot.connection_generation != acknowledgement.connection_generation
            || active
                .outstanding_deliveries
                .get(&acknowledgement.delivery_sequence)
                != Some(&acknowledgement.through_cursor)
        {
            return Err(Error::WebSocketProtocol(
                "ACK does not match an outstanding delivery".into(),
            ));
        }
        if self
            .pending_acknowledgements
            .keys()
            .any(|(id, _, _)| id == &acknowledgement.subscription_id)
        {
            return Err(Error::WebSocketProtocol(
                "subscription already has an ACK in flight".into(),
            ));
        }
        let key = acknowledgement_key(&acknowledgement);
        self.send(WebSocketPayload::Ack(acknowledgement.clone()))
            .await?;
        self.pending_acknowledgements.insert(key, acknowledgement);
        Ok(())
    }

    pub async fn heartbeat(&mut self, heartbeat_id: CorrelationId) -> Result<()> {
        let subscriptions = self
            .subscriptions
            .values()
            .map(|active| SubscriptionLeaseCoordinate::from_snapshot(&active.snapshot))
            .collect();
        self.send(WebSocketPayload::Heartbeat(WebSocketHeartbeat {
            heartbeat_id,
            observed_sequence: self.receiver.last_sequence(),
            subscriptions,
        }))
        .await
    }

    pub async fn unsubscribe(&mut self, subscription_id: &CorrelationId) -> Result<()> {
        let Some(active) = self.subscriptions.get(subscription_id) else {
            return Err(Error::WebSocketProtocol(
                "unsubscribe does not target an active subscription".into(),
            ));
        };
        let key = (
            subscription_id.clone(),
            active.snapshot.connection_generation,
        );
        if self.pending_unsubscribes.contains(&key) {
            return Err(Error::WebSocketProtocol(
                "unsubscribe is already in flight".into(),
            ));
        }
        self.send(WebSocketPayload::Unsubscribe(WebSocketUnsubscribe {
            subscription_id: key.0.clone(),
            connection_generation: key.1,
        }))
        .await?;
        self.pending_unsubscribes.insert(key);
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<WebSocketFrame> {
        self.receive_until(Instant::now() + Duration::from_millis(self.limits.receive_timeout_ms))
            .await
    }

    pub async fn receive_until(&mut self, deadline: Instant) -> Result<WebSocketFrame> {
        loop {
            let message = tokio::time::timeout_at(deadline, self.stream.next())
                .await
                .map_err(|_| Error::Timeout)?;
            match message {
                Some(Ok(Message::Text(text))) => {
                    let frame = self
                        .receiver
                        .accept(text.as_bytes())
                        .map_err(|error| Error::WebSocketProtocol(error.to_string()))?;
                    self.accept_server_frame(&frame)?;
                    return Ok(frame);
                }
                Some(Ok(Message::Ping(bytes))) => {
                    self.send_transport(Message::Pong(bytes)).await?;
                }
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Binary(_))) => {
                    return Err(Error::WebSocketProtocol(
                        "server sent an unsupported binary application message".into(),
                    ));
                }
                Some(Ok(Message::Close(frame))) => {
                    return Err(Error::Transport(format!(
                        "RRFlow WebSocket closed: {frame:?}"
                    )));
                }
                Some(Err(error)) => return Err(websocket_stream_error(error)),
                None => return Err(Error::Transport("RRFlow WebSocket ended".into())),
                Some(Ok(Message::Frame(_))) => {}
            }
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        self.send_transport(Message::Close(None)).await
    }

    async fn send(&mut self, payload: WebSocketPayload) -> Result<()> {
        let bytes = self.sender.encode(payload).map_err(contract)?;
        let text = String::from_utf8(bytes).map_err(contract)?;
        self.send_transport(Message::Text(text.into())).await
    }

    async fn send_transport(&mut self, message: Message) -> Result<()> {
        tokio::time::timeout(
            Duration::from_millis(self.limits.send_timeout_ms),
            self.stream.send(message),
        )
        .await
        .map_err(|_| Error::Timeout)?
        .map_err(|error| Error::Transport(error.to_string()))
    }

    fn accept_server_frame(&mut self, frame: &WebSocketFrame) -> Result<()> {
        match &frame.payload {
            WebSocketPayload::Response(response) => {
                let target = response.target();
                let request = self.pending_requests.get(&target).ok_or_else(|| {
                    Error::WebSocketProtocol("response has no matching in-flight request".into())
                })?;
                validate_websocket_response_correlation(request, response)
                    .map_err(|error| Error::WebSocketProtocol(error.to_string()))?;
                self.pending_requests.remove(&target);
            }
            WebSocketPayload::Cancellation(cancellation) => {
                let request = self
                    .pending_cancellations
                    .get(&cancellation.cancellation_id)
                    .ok_or_else(|| {
                        Error::WebSocketProtocol(
                            "cancellation result has no matching in-flight cancellation".into(),
                        )
                    })?;
                validate_websocket_cancellation_correlation(request, cancellation)
                    .map_err(|error| Error::WebSocketProtocol(error.to_string()))?;
                self.pending_cancellations
                    .remove(&cancellation.cancellation_id);
                if cancellation.disposition == WebSocketCancellationDisposition::Cancelled {
                    self.pending_requests.remove(&cancellation.target);
                }
            }
            WebSocketPayload::Subscribed(subscribed) => {
                let resume = self
                    .pending_subscriptions
                    .get(&subscribed.subscription.subscription_id)
                    .ok_or_else(|| {
                        Error::WebSocketProtocol(
                            "subscribed result has no matching pending subscription".into(),
                        )
                    })?;
                validate_websocket_subscription_correlation(resume, &subscribed.subscription)
                    .map_err(|error| Error::WebSocketProtocol(error.to_string()))?;
                self.pending_subscriptions
                    .remove(&subscribed.subscription.subscription_id);
                self.subscriptions.insert(
                    subscribed.subscription.subscription_id.clone(),
                    ActiveSubscription::new(subscribed.subscription.clone()),
                );
            }
            WebSocketPayload::Delivery(delivery) => {
                let active = self
                    .subscriptions
                    .get_mut(&delivery.subscription_id)
                    .ok_or_else(|| {
                        Error::WebSocketProtocol(
                            "delivery does not target an active subscription".into(),
                        )
                    })?;
                if active.snapshot.connection_generation != delivery.connection_generation
                    || delivery.from_cursor != active.last_through_cursor
                    || active
                        .last_delivery_sequence
                        .is_some_and(|last| last.checked_add(1) != Some(delivery.delivery_sequence))
                {
                    return Err(Error::WebSocketProtocol(
                        "delivery generation, sequence, or cursor is discontinuous".into(),
                    ));
                }
                active.last_delivery_sequence = Some(delivery.delivery_sequence);
                active.last_through_cursor = delivery.through_cursor;
                active
                    .outstanding_deliveries
                    .insert(delivery.delivery_sequence, delivery.through_cursor);
            }
            WebSocketPayload::Acknowledged(acknowledged) => {
                let key = acknowledgement_key(&acknowledged.acknowledgement);
                if self.pending_acknowledgements.get(&key) != Some(&acknowledged.acknowledgement) {
                    return Err(Error::WebSocketProtocol(
                        "acknowledged result has no exact pending ACK".into(),
                    ));
                }
                let active = self
                    .subscriptions
                    .get_mut(&acknowledged.subscription.subscription_id)
                    .ok_or_else(|| {
                        Error::WebSocketProtocol(
                            "acknowledged result has no active subscription".into(),
                        )
                    })?;
                if active.snapshot.connection_generation
                    != acknowledged.subscription.connection_generation
                {
                    return Err(Error::WebSocketProtocol(
                        "acknowledged result changed subscription generation".into(),
                    ));
                }
                self.pending_acknowledgements.remove(&key);
                active.snapshot = acknowledged.subscription.clone();
                active
                    .outstanding_deliveries
                    .retain(|_, through| *through > acknowledged.acknowledgement.through_cursor);
            }
            WebSocketPayload::Heartbeat(heartbeat) => {
                if heartbeat.observed_sequence > self.sender.last_sequence()
                    || heartbeat.observed_sequence < self.last_server_observed_sequence
                {
                    return Err(Error::WebSocketProtocol(
                        "heartbeat peer sequence is ahead of the client or moved backward".into(),
                    ));
                }
                let expected = self
                    .subscriptions
                    .values()
                    .map(|active| SubscriptionLeaseCoordinate::from_snapshot(&active.snapshot))
                    .collect::<BTreeSet<_>>();
                let actual = heartbeat
                    .subscriptions
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if actual != expected {
                    return Err(Error::WebSocketProtocol(
                        "heartbeat subscription coordinates differ from active state".into(),
                    ));
                }
                self.last_server_observed_sequence = heartbeat.observed_sequence;
            }
            WebSocketPayload::Unsubscribed(unsubscribed) => {
                let key = (
                    unsubscribed.subscription.subscription_id.clone(),
                    unsubscribed.subscription.connection_generation,
                );
                if !self.pending_unsubscribes.remove(&key) {
                    return Err(Error::WebSocketProtocol(
                        "unsubscribed result has no exact pending request".into(),
                    ));
                }
                let active = self
                    .subscriptions
                    .get(&unsubscribed.subscription.subscription_id)
                    .ok_or_else(|| {
                        Error::WebSocketProtocol(
                            "unsubscribed result has no active subscription".into(),
                        )
                    })?;
                if active.snapshot != unsubscribed.subscription {
                    return Err(Error::WebSocketProtocol(
                        "unsubscribed snapshot differs from active state".into(),
                    ));
                }
                self.subscriptions
                    .remove(&unsubscribed.subscription.subscription_id);
            }
            WebSocketPayload::Error(error) => {
                self.accept_server_error(&error.target)?;
                return Err(Error::WebSocket(Box::new(error.clone())));
            }
            WebSocketPayload::Backpressure(backpressure) => {
                self.validate_backpressure_target(&backpressure.target)?;
            }
            WebSocketPayload::Connected(_) => {
                return Err(Error::WebSocketProtocol(
                    "server repeated the connected payload".into(),
                ));
            }
            WebSocketPayload::Request(_)
            | WebSocketPayload::Cancel(_)
            | WebSocketPayload::Subscribe(_)
            | WebSocketPayload::Ack(_)
            | WebSocketPayload::Unsubscribe(_) => {
                return Err(Error::WebSocketProtocol(
                    "server sent a client-only payload".into(),
                ));
            }
        }
        Ok(())
    }

    fn accept_server_error(&mut self, target: &WebSocketErrorTarget) -> Result<()> {
        match target {
            WebSocketErrorTarget::Connection => Ok(()),
            WebSocketErrorTarget::Request { target } => {
                self.pending_requests.remove(target).ok_or_else(|| {
                    Error::WebSocketProtocol("request error has no pending request".into())
                })?;
                Ok(())
            }
            WebSocketErrorTarget::Cancellation {
                cancellation_id,
                target,
            } => {
                let cancellation = self
                    .pending_cancellations
                    .remove(cancellation_id)
                    .ok_or_else(|| {
                        Error::WebSocketProtocol(
                            "cancellation error has no pending cancellation".into(),
                        )
                    })?;
                if cancellation.target != *target {
                    return Err(Error::WebSocketProtocol(
                        "cancellation error target differs from pending request".into(),
                    ));
                }
                Ok(())
            }
            WebSocketErrorTarget::Subscription {
                subscription_id,
                connection_generation,
            } => {
                let pending_matches =
                    self.pending_subscriptions
                        .get(subscription_id)
                        .is_some_and(|resume| {
                            resume.connection_generation.checked_add(1)
                                == Some(*connection_generation)
                        });
                let active_matches =
                    self.subscriptions
                        .get(subscription_id)
                        .is_some_and(|active| {
                            active.snapshot.connection_generation == *connection_generation
                        });
                if !pending_matches && !active_matches {
                    return Err(Error::WebSocketProtocol(
                        "subscription error has no matching pending or active generation".into(),
                    ));
                }
                if pending_matches {
                    self.pending_subscriptions.remove(subscription_id);
                }
                self.pending_acknowledgements
                    .retain(|(id, generation, _), _| {
                        id != subscription_id || generation != connection_generation
                    });
                self.pending_unsubscribes
                    .remove(&(subscription_id.clone(), *connection_generation));
                Ok(())
            }
        }
    }

    fn validate_backpressure_target(&self, target: &WebSocketBackpressureTarget) -> Result<()> {
        let valid = match target {
            WebSocketBackpressureTarget::Request { target } => {
                self.pending_requests.contains_key(target)
            }
            WebSocketBackpressureTarget::Subscription {
                subscription_id,
                connection_generation,
            } => {
                self.subscriptions
                    .get(subscription_id)
                    .is_some_and(|active| {
                        active.snapshot.connection_generation == *connection_generation
                    })
                    || self
                        .pending_subscriptions
                        .get(subscription_id)
                        .is_some_and(|resume| {
                            resume.connection_generation.checked_add(1)
                                == Some(*connection_generation)
                        })
            }
        };
        if valid {
            Ok(())
        } else {
            Err(Error::WebSocketProtocol(
                "backpressure has no matching request or subscription".into(),
            ))
        }
    }
}

fn acknowledgement_key(acknowledgement: &SubscriptionAcknowledgement) -> AcknowledgementKey {
    (
        acknowledgement.subscription_id.clone(),
        acknowledgement.connection_generation,
        acknowledgement.delivery_sequence,
    )
}

async fn receive_initial(
    stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    receiver: &mut WebSocketReceiveState,
    deadline: Instant,
) -> Result<WebSocketFrame> {
    loop {
        let message = tokio::time::timeout_at(deadline, stream.next())
            .await
            .map_err(|_| Error::Timeout)?;
        match message {
            Some(Ok(Message::Text(text))) => {
                return receiver
                    .accept(text.as_bytes())
                    .map_err(|error| Error::WebSocketProtocol(error.to_string()));
            }
            Some(Ok(Message::Ping(bytes))) => stream
                .send(Message::Pong(bytes))
                .await
                .map_err(|error| Error::Transport(error.to_string()))?,
            Some(Ok(Message::Pong(_))) => {}
            Some(Ok(Message::Binary(_))) => {
                return Err(Error::WebSocketProtocol(
                    "server sent binary data before connected".into(),
                ));
            }
            Some(Ok(Message::Close(frame))) => {
                return Err(Error::Transport(format!(
                    "RRFlow WebSocket closed before connected: {frame:?}"
                )));
            }
            Some(Err(error)) => return Err(websocket_stream_error(error)),
            None => {
                return Err(Error::Transport(
                    "RRFlow WebSocket ended before connected".into(),
                ));
            }
            Some(Ok(Message::Frame(_))) => {}
        }
    }
}
