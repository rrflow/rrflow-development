use super::*;
use axum::extract::ws::{close_code, CloseFrame, Message, WebSocket, WebSocketUpgrade};
use rrd_contract::{
    SubscriptionAcknowledgement, SubscriptionLeaseCoordinate, SubscriptionPoll, SubscriptionResume,
    SubscriptionSnapshot, WebSocketAcknowledged, WebSocketBackpressure,
    WebSocketBackpressureTarget, WebSocketCancellation, WebSocketCancellationDisposition,
    WebSocketConnected, WebSocketDelivery, WebSocketError, WebSocketErrorTarget,
    WebSocketHeartbeat, WebSocketLimits, WebSocketPayload, WebSocketPeer, WebSocketReceiveState,
    WebSocketSendState, WebSocketSubscribed, WebSocketUnsubscribed, WEBSOCKET_CONTRACT_VERSION,
};
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::Instant;

const SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(100);

struct ActiveSubscription {
    snapshot: SubscriptionSnapshot,
    next_poll_at: Instant,
}

enum SubscriptionInvocationError {
    Service(ServiceError),
    Worker(String),
}

pub(super) async fn websocket_upgrade(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> HttpResponse {
    let now = unix_time_ms();
    let mut context = generated_context(now, "websocket-connect");
    context.idempotency_key = Some(context.operation_id.clone());
    let (session_id, token) = match authenticated_session(&headers, None) {
        Ok(session) => session,
        Err(error) => return failure(&context, error),
    };
    let connection_id = context.request_id.clone();
    let limits = WebSocketLimits::default();
    let authorized = match state.service.begin_invocation(
        Invocation {
            context: context.clone(),
            resource: instance_resource(state.service.instance_id()),
            observed_at_unix_ms: now,
            attempt: HTTP_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            request_sha256: sha256_hex(b"rrd-websocket-connect-v1"),
        },
        RrdOperation::WebSocketConnect,
        InvocationCredential::Session {
            session_id: &session_id,
            token: &token,
        },
    ) {
        Ok(authorized) => authorized,
        Err(error) => return failure(&context, api_error(error)),
    };
    let response_sha256 = sha256_hex(
        &serde_json::to_vec(&(&connection_id, &limits))
            .expect("WebSocket connection coordinates serialize"),
    );
    if let Err(error) = state.service.complete_invocation(
        &authorized,
        InvocationCompletion {
            decision: AuditDecision::Allowed,
            // The audit contract records completed logical outcomes (2xx+),
            // while the wire handshake itself returns 101 Switching Protocols.
            status_code: StatusCode::OK.as_u16(),
            response_sha256,
        },
    ) {
        return failure(&context, api_error(error));
    }

    websocket
        .read_buffer_size(limits.read_buffer_bytes as usize)
        .write_buffer_size(limits.write_buffer_bytes as usize)
        .max_write_buffer_size(limits.max_write_buffer_bytes as usize)
        .max_message_size(limits.max_message_bytes as usize)
        .max_frame_size(limits.max_frame_bytes as usize)
        .on_upgrade(move |socket| {
            serve_websocket(socket, state, session_id, token, connection_id, limits)
        })
}

async fn serve_websocket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    session_id: CorrelationId,
    token: CorrelationId,
    connection_id: CorrelationId,
    limits: WebSocketLimits,
) {
    let mut sender =
        match WebSocketSendState::new(WebSocketPeer::Server, connection_id.clone(), limits.clone())
        {
            Ok(sender) => sender,
            Err(_) => return,
        };
    let mut receiver = match WebSocketReceiveState::for_connection(
        WebSocketPeer::Client,
        connection_id,
        limits.clone(),
    ) {
        Ok(receiver) => receiver,
        Err(_) => return,
    };
    if send_payload(
        &mut socket,
        &mut sender,
        WebSocketPayload::Connected(WebSocketConnected {
            contract_version: WEBSOCKET_CONTRACT_VERSION,
            session_id: session_id.clone(),
            limits: limits.clone(),
        }),
        &limits,
    )
    .await
    .is_err()
    {
        return;
    }

    let mut subscriptions = BTreeMap::<CorrelationId, ActiveSubscription>::new();
    let mut poll = tokio::time::interval(SOCKET_POLL_INTERVAL);
    poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    poll.tick().await;
    let mut last_inbound = Instant::now();
    let mut next_heartbeat = last_inbound + Duration::from_millis(limits.heartbeat_interval_ms);
    let mut last_client_observed_sequence = 0;

    loop {
        let receive_deadline = last_inbound + Duration::from_millis(limits.receive_timeout_ms);
        tokio::select! {
            inbound = socket.recv() => {
                match inbound {
                    Some(Ok(Message::Text(text))) => {
                        last_inbound = Instant::now();
                        let frame = match receiver.accept(text.as_bytes()) {
                            Ok(frame) => frame,
                            Err(error) => {
                                let _ = send_error(
                                    &mut socket,
                                    &mut sender,
                                    WebSocketErrorTarget::Connection,
                                    ErrorCode::InvalidArgument,
                                    error.to_string(),
                                    &limits,
                                ).await;
                                close_socket(&mut socket, close_code::PROTOCOL, "invalid RRFlow frame").await;
                                return;
                            }
                        };
                        if handle_client_payload(
                            &mut socket,
                            &mut sender,
                            &mut subscriptions,
                            &state,
                            &session_id,
                            &token,
                            frame.payload,
                            receiver.last_sequence(),
                            &mut last_client_observed_sequence,
                            &limits,
                        ).await.is_err() {
                            close_socket(&mut socket, close_code::PROTOCOL, "RRFlow protocol failure").await;
                            return;
                        }
                    }
                    Some(Ok(Message::Ping(bytes))) => {
                        last_inbound = Instant::now();
                        if send_transport(&mut socket, Message::Pong(bytes), &limits).await.is_err() {
                            return;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => last_inbound = Instant::now(),
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
                    Some(Ok(Message::Binary(_))) => {
                        close_socket(&mut socket, close_code::UNSUPPORTED, "text application messages required").await;
                        return;
                    }
                }
            }
            _ = poll.tick() => {
                if poll_subscriptions(
                    &mut socket,
                    &mut sender,
                    &mut subscriptions,
                    &state,
                    &session_id,
                    &token,
                    &limits,
                ).await.is_err() {
                    return;
                }
                if Instant::now() >= next_heartbeat && receiver.last_sequence() > 0 {
                    let heartbeat_id = generated_context(unix_time_ms(), "websocket-heartbeat").request_id;
                    let coordinates = subscriptions
                        .values()
                        .map(|active| SubscriptionLeaseCoordinate::from_snapshot(&active.snapshot))
                        .collect();
                    if send_payload(
                        &mut socket,
                        &mut sender,
                        WebSocketPayload::Heartbeat(WebSocketHeartbeat {
                            heartbeat_id,
                            observed_sequence: receiver.last_sequence(),
                            subscriptions: coordinates,
                        }),
                        &limits,
                    ).await.is_err() {
                        return;
                    }
                    next_heartbeat = Instant::now() + Duration::from_millis(limits.heartbeat_interval_ms);
                }
            }
            _ = tokio::time::sleep_until(receive_deadline) => {
                let _ = send_error(
                    &mut socket,
                    &mut sender,
                    WebSocketErrorTarget::Connection,
                    ErrorCode::DeadlineExceeded,
                    "WebSocket receive deadline elapsed",
                    &limits,
                ).await;
                close_socket(&mut socket, close_code::POLICY, "receive deadline elapsed").await;
                return;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_client_payload(
    socket: &mut WebSocket,
    sender: &mut WebSocketSendState,
    subscriptions: &mut BTreeMap<CorrelationId, ActiveSubscription>,
    state: &Arc<AppState>,
    session_id: &CorrelationId,
    token: &CorrelationId,
    payload: WebSocketPayload,
    observed_sequence: u64,
    last_client_observed_sequence: &mut u64,
    limits: &WebSocketLimits,
) -> std::result::Result<(), ()> {
    match payload {
        WebSocketPayload::Request(request) => {
            send_error(
                socket,
                sender,
                WebSocketErrorTarget::Request {
                    target: request.target(),
                },
                ErrorCode::FailedPrecondition,
                "WebSocket operation execution is not enabled until H-04 binds the shared dispatch",
                limits,
            )
            .await
        }
        WebSocketPayload::Cancel(cancel) => {
            send_payload(
                socket,
                sender,
                WebSocketPayload::Cancellation(WebSocketCancellation {
                    cancellation_id: cancel.cancellation_id,
                    target: cancel.target,
                    disposition: WebSocketCancellationDisposition::NotFound,
                }),
                limits,
            )
            .await
        }
        WebSocketPayload::Subscribe(request) => {
            let expected_generation = request
                .resume
                .connection_generation
                .saturating_add(1)
                .max(1);
            if subscriptions.contains_key(&request.resume.subscription_id) {
                return send_error(
                    socket,
                    sender,
                    WebSocketErrorTarget::Subscription {
                        subscription_id: request.resume.subscription_id,
                        connection_generation: expected_generation,
                    },
                    ErrorCode::Conflict,
                    "subscription is already attached to this connection",
                    limits,
                )
                .await;
            }
            if subscriptions.len() >= usize::from(limits.max_subscriptions) {
                return send_payload(
                    socket,
                    sender,
                    WebSocketPayload::Backpressure(WebSocketBackpressure {
                        target: WebSocketBackpressureTarget::Subscription {
                            subscription_id: request.resume.subscription_id,
                            connection_generation: expected_generation,
                        },
                        in_flight: subscriptions.len() as u32,
                        limit: u32::from(limits.max_subscriptions),
                        retry_after_ms: limits.heartbeat_interval_ms,
                    }),
                    limits,
                )
                .await;
            }
            let resume = request.resume;
            let result = invoke_subscription_connect(
                Arc::clone(state),
                session_id.clone(),
                token.clone(),
                resume.clone(),
            )
            .await;
            match result {
                Ok(snapshot) => {
                    subscriptions.insert(
                        snapshot.subscription_id.clone(),
                        ActiveSubscription {
                            snapshot: snapshot.clone(),
                            next_poll_at: Instant::now(),
                        },
                    );
                    send_payload(
                        socket,
                        sender,
                        WebSocketPayload::Subscribed(WebSocketSubscribed {
                            subscription: snapshot,
                        }),
                        limits,
                    )
                    .await
                }
                Err(error) => {
                    send_subscription_invocation_error(
                        socket,
                        sender,
                        WebSocketErrorTarget::Subscription {
                            subscription_id: resume.subscription_id,
                            connection_generation: expected_generation,
                        },
                        error,
                        limits,
                    )
                    .await
                }
            }
        }
        WebSocketPayload::Ack(acknowledgement) => {
            if !active_matches(subscriptions, &acknowledgement) {
                return send_error(
                    socket,
                    sender,
                    WebSocketErrorTarget::Subscription {
                        subscription_id: acknowledgement.subscription_id,
                        connection_generation: acknowledgement.connection_generation,
                    },
                    ErrorCode::FailedPrecondition,
                    "ACK does not target an active subscription generation",
                    limits,
                )
                .await;
            }
            let result = invoke_subscription_ack(
                Arc::clone(state),
                session_id.clone(),
                token.clone(),
                acknowledgement.clone(),
            )
            .await;
            match result {
                Ok(snapshot) => {
                    if let Some(active) = subscriptions.get_mut(&snapshot.subscription_id) {
                        active.snapshot = snapshot.clone();
                    }
                    send_payload(
                        socket,
                        sender,
                        WebSocketPayload::Acknowledged(WebSocketAcknowledged {
                            acknowledgement,
                            subscription: snapshot,
                        }),
                        limits,
                    )
                    .await
                }
                Err(error) => {
                    let target = WebSocketErrorTarget::Subscription {
                        subscription_id: acknowledgement.subscription_id,
                        connection_generation: acknowledgement.connection_generation,
                    };
                    send_subscription_invocation_error(socket, sender, target, error, limits).await
                }
            }
        }
        WebSocketPayload::Heartbeat(heartbeat) => {
            if heartbeat.observed_sequence > sender.last_sequence()
                || heartbeat.observed_sequence < *last_client_observed_sequence
            {
                return send_error(
                    socket,
                    sender,
                    WebSocketErrorTarget::Connection,
                    ErrorCode::FailedPrecondition,
                    "heartbeat peer sequence is ahead of the server or moved backward",
                    limits,
                )
                .await;
            }
            for coordinate in &heartbeat.subscriptions {
                let active = subscriptions.get(&coordinate.subscription_id);
                if active.is_none_or(|active| {
                    active.snapshot.connection_generation != coordinate.connection_generation
                        || active.snapshot.acknowledged_cursor != coordinate.acknowledged_cursor
                }) {
                    return send_error(
                        socket,
                        sender,
                        WebSocketErrorTarget::Subscription {
                            subscription_id: coordinate.subscription_id.clone(),
                            connection_generation: coordinate.connection_generation,
                        },
                        ErrorCode::FailedPrecondition,
                        "heartbeat does not target active durable coordinates",
                        limits,
                    )
                    .await;
                }
            }
            *last_client_observed_sequence = heartbeat.observed_sequence;
            let mut renewed = Vec::with_capacity(heartbeat.subscriptions.len());
            for coordinate in heartbeat.subscriptions {
                match invoke_subscription_heartbeat(
                    Arc::clone(state),
                    session_id.clone(),
                    token.clone(),
                    coordinate.clone(),
                )
                .await
                {
                    Ok(snapshot) => {
                        subscriptions
                            .get_mut(&snapshot.subscription_id)
                            .expect("active subscription was checked")
                            .snapshot = snapshot.clone();
                        renewed.push(SubscriptionLeaseCoordinate::from_snapshot(&snapshot));
                    }
                    Err(error) => {
                        let target = WebSocketErrorTarget::Subscription {
                            subscription_id: coordinate.subscription_id,
                            connection_generation: coordinate.connection_generation,
                        };
                        return send_subscription_invocation_error(
                            socket, sender, target, error, limits,
                        )
                        .await;
                    }
                }
            }
            send_payload(
                socket,
                sender,
                WebSocketPayload::Heartbeat(WebSocketHeartbeat {
                    heartbeat_id: heartbeat.heartbeat_id,
                    observed_sequence,
                    subscriptions: renewed,
                }),
                limits,
            )
            .await
        }
        WebSocketPayload::Unsubscribe(request) => {
            let active = subscriptions.get(&request.subscription_id);
            if active.is_none_or(|active| {
                active.snapshot.connection_generation != request.connection_generation
            }) {
                return send_error(
                    socket,
                    sender,
                    WebSocketErrorTarget::Subscription {
                        subscription_id: request.subscription_id,
                        connection_generation: request.connection_generation,
                    },
                    ErrorCode::FailedPrecondition,
                    "unsubscribe does not target an active subscription generation",
                    limits,
                )
                .await;
            }
            let snapshot = subscriptions
                .remove(&request.subscription_id)
                .expect("active subscription was checked")
                .snapshot;
            send_payload(
                socket,
                sender,
                WebSocketPayload::Unsubscribed(WebSocketUnsubscribed {
                    subscription: snapshot,
                }),
                limits,
            )
            .await
        }
        _ => {
            send_error(
                socket,
                sender,
                WebSocketErrorTarget::Connection,
                ErrorCode::InvalidArgument,
                "server-only WebSocket payload received from client",
                limits,
            )
            .await
        }
    }
}

fn active_matches(
    subscriptions: &BTreeMap<CorrelationId, ActiveSubscription>,
    acknowledgement: &SubscriptionAcknowledgement,
) -> bool {
    subscriptions
        .get(&acknowledgement.subscription_id)
        .is_some_and(|active| {
            active.snapshot.connection_generation == acknowledgement.connection_generation
        })
}

#[allow(clippy::too_many_arguments)]
async fn poll_subscriptions(
    socket: &mut WebSocket,
    sender: &mut WebSocketSendState,
    subscriptions: &mut BTreeMap<CorrelationId, ActiveSubscription>,
    state: &Arc<AppState>,
    session_id: &CorrelationId,
    token: &CorrelationId,
    limits: &WebSocketLimits,
) -> std::result::Result<(), ()> {
    let now_instant = Instant::now();
    let due = subscriptions
        .iter()
        .filter(|(_, active)| active.next_poll_at <= now_instant)
        .map(|(id, active)| (id.clone(), active.snapshot.clone()))
        .collect::<Vec<_>>();
    for (subscription_id, snapshot) in due {
        if let Some(active) = subscriptions.get_mut(&subscription_id) {
            active.next_poll_at =
                now_instant + Duration::from_millis(active.snapshot.heartbeat_interval_ms.max(1));
        }
        let now = unix_time_ms();
        let context = generated_context(now, "subscription-delivery");
        let state_for_call = Arc::clone(state);
        let session_for_call = session_id.clone();
        let token_for_call = token.clone();
        let subscription_for_call = subscription_id.clone();
        let result = tokio::task::spawn_blocking(move || {
            state_for_call.service.next_subscription_delivery(
                &session_for_call,
                &token_for_call,
                &subscription_for_call,
                snapshot.connection_generation,
                now,
                context.request_id.as_str(),
                context.operation_id.as_str(),
            )
        })
        .await;
        match result {
            Ok(Ok(SubscriptionPoll::Idle { subscription })) => {
                if let Some(active) = subscriptions.get_mut(&subscription_id) {
                    active.snapshot = subscription;
                }
            }
            Ok(Ok(SubscriptionPoll::Delivery { batch })) => {
                if send_payload(
                    socket,
                    sender,
                    WebSocketPayload::Delivery(WebSocketDelivery::from(batch)),
                    limits,
                )
                .await
                .is_err()
                {
                    return Err(());
                }
            }
            Ok(Err(ServiceError::SubscriptionBackpressure)) => {
                if send_payload(
                    socket,
                    sender,
                    WebSocketPayload::Backpressure(WebSocketBackpressure {
                        target: WebSocketBackpressureTarget::Subscription {
                            subscription_id: subscription_id.clone(),
                            connection_generation: snapshot.connection_generation,
                        },
                        in_flight: u32::from(snapshot.max_in_flight),
                        limit: u32::from(snapshot.max_in_flight),
                        retry_after_ms: snapshot.heartbeat_interval_ms,
                    }),
                    limits,
                )
                .await
                .is_err()
                {
                    return Err(());
                }
            }
            Ok(Err(error)) => {
                let target = WebSocketErrorTarget::Subscription {
                    subscription_id: subscription_id.clone(),
                    connection_generation: snapshot.connection_generation,
                };
                send_service_error(socket, sender, target, error, limits).await?;
                subscriptions.remove(&subscription_id);
            }
            Err(error) => {
                send_error(
                    socket,
                    sender,
                    WebSocketErrorTarget::Subscription {
                        subscription_id: subscription_id.clone(),
                        connection_generation: snapshot.connection_generation,
                    },
                    ErrorCode::Internal,
                    error.to_string(),
                    limits,
                )
                .await?;
                subscriptions.remove(&subscription_id);
            }
        }
    }
    Ok(())
}

async fn invoke_subscription_connect(
    state: Arc<AppState>,
    session_id: CorrelationId,
    token: CorrelationId,
    resume: SubscriptionResume,
) -> std::result::Result<SubscriptionSnapshot, SubscriptionInvocationError> {
    tokio::task::spawn_blocking(move || {
        invoke_subscription_operation(
            &state,
            &session_id,
            &token,
            RrdOperation::SubscriptionConnect,
            &resume,
            |context| {
                state.service.connect_subscription(
                    &session_id,
                    &token,
                    &resume,
                    context.0,
                    context.1.request_id.as_str(),
                    context.1.operation_id.as_str(),
                )
            },
        )
    })
    .await
    .map_err(|error| SubscriptionInvocationError::Worker(error.to_string()))?
    .map_err(SubscriptionInvocationError::Service)
}

async fn invoke_subscription_ack(
    state: Arc<AppState>,
    session_id: CorrelationId,
    token: CorrelationId,
    acknowledgement: SubscriptionAcknowledgement,
) -> std::result::Result<SubscriptionSnapshot, SubscriptionInvocationError> {
    tokio::task::spawn_blocking(move || {
        invoke_subscription_operation(
            &state,
            &session_id,
            &token,
            RrdOperation::SubscriptionAck,
            &acknowledgement,
            |context| {
                state.service.acknowledge_subscription(
                    &session_id,
                    &token,
                    &acknowledgement,
                    context.0,
                    context.1.request_id.as_str(),
                    context.1.operation_id.as_str(),
                )
            },
        )
    })
    .await
    .map_err(|error| SubscriptionInvocationError::Worker(error.to_string()))?
    .map_err(SubscriptionInvocationError::Service)
}

async fn invoke_subscription_heartbeat(
    state: Arc<AppState>,
    session_id: CorrelationId,
    token: CorrelationId,
    coordinate: SubscriptionLeaseCoordinate,
) -> std::result::Result<SubscriptionSnapshot, SubscriptionInvocationError> {
    tokio::task::spawn_blocking(move || {
        invoke_subscription_operation(
            &state,
            &session_id,
            &token,
            RrdOperation::SubscriptionAck,
            &coordinate,
            |context| {
                state.service.renew_subscription_lease(
                    &session_id,
                    &token,
                    &coordinate,
                    context.0,
                    context.1.request_id.as_str(),
                    context.1.operation_id.as_str(),
                )
            },
        )
    })
    .await
    .map_err(|error| SubscriptionInvocationError::Worker(error.to_string()))?
    .map_err(SubscriptionInvocationError::Service)
}

fn invoke_subscription_operation<T: serde::Serialize>(
    state: &AppState,
    session_id: &CorrelationId,
    token: &CorrelationId,
    operation: RrdOperation,
    request: &T,
    call: impl FnOnce((u64, RequestContext)) -> std::result::Result<SubscriptionSnapshot, ServiceError>,
) -> std::result::Result<SubscriptionSnapshot, ServiceError> {
    let now = unix_time_ms();
    let mut context = generated_context(now, "websocket-subscription-operation");
    context.idempotency_key = Some(context.operation_id.clone());
    let authorized = state.service.begin_invocation(
        Invocation {
            context: context.clone(),
            resource: instance_resource(state.service.instance_id()),
            observed_at_unix_ms: now,
            attempt: HTTP_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            request_sha256: sha256_hex(
                &serde_json::to_vec(request).expect("subscription operation serializes"),
            ),
        },
        operation,
        InvocationCredential::Session { session_id, token },
    )?;
    let result = call((now, context));
    let completion = match &result {
        Ok(snapshot) => InvocationCompletion {
            decision: AuditDecision::Allowed,
            status_code: StatusCode::OK.as_u16(),
            response_sha256: sha256_hex(
                &serde_json::to_vec(snapshot).expect("subscription snapshot serializes"),
            ),
        },
        Err(error) => InvocationCompletion {
            decision: if matches!(
                error.kind(),
                ServiceErrorKind::Unauthenticated | ServiceErrorKind::PermissionDenied
            ) {
                AuditDecision::Denied
            } else {
                AuditDecision::Failed
            },
            status_code: service_status(error.kind()).as_u16(),
            response_sha256: sha256_hex(error.to_string().as_bytes()),
        },
    };
    state.service.complete_invocation(&authorized, completion)?;
    result
}

async fn send_service_error(
    socket: &mut WebSocket,
    sender: &mut WebSocketSendState,
    target: WebSocketErrorTarget,
    error: ServiceError,
    limits: &WebSocketLimits,
) -> std::result::Result<(), ()> {
    let error = websocket_error(error);
    send_error(socket, sender, target, error.code, error.message, limits).await
}

async fn send_subscription_invocation_error(
    socket: &mut WebSocket,
    sender: &mut WebSocketSendState,
    target: WebSocketErrorTarget,
    error: SubscriptionInvocationError,
    limits: &WebSocketLimits,
) -> std::result::Result<(), ()> {
    match error {
        SubscriptionInvocationError::Service(error) => {
            send_service_error(socket, sender, target, error, limits).await
        }
        SubscriptionInvocationError::Worker(message) => {
            send_error(socket, sender, target, ErrorCode::Internal, message, limits).await
        }
    }
}

async fn send_error(
    socket: &mut WebSocket,
    sender: &mut WebSocketSendState,
    target: WebSocketErrorTarget,
    code: ErrorCode,
    message: impl Into<String>,
    limits: &WebSocketLimits,
) -> std::result::Result<(), ()> {
    let message = bounded_message(message.into());
    send_payload(
        socket,
        sender,
        WebSocketPayload::Error(WebSocketError {
            target,
            error: ErrorBody {
                code,
                message,
                retryable: false,
                details: BTreeMap::new(),
            },
        }),
        limits,
    )
    .await
}

fn bounded_message(mut message: String) -> String {
    if message.is_empty() {
        return "RRFlow WebSocket error".into();
    }
    while message.len() > rrd_contract::MAX_MESSAGE_BYTES {
        message.pop();
    }
    message
}

async fn send_payload(
    socket: &mut WebSocket,
    sender: &mut WebSocketSendState,
    payload: WebSocketPayload,
    limits: &WebSocketLimits,
) -> std::result::Result<(), ()> {
    let bytes = sender.encode(payload).map_err(|_| ())?;
    let text = String::from_utf8(bytes).map_err(|_| ())?;
    send_transport(socket, Message::Text(text.into()), limits).await
}

async fn send_transport(
    socket: &mut WebSocket,
    message: Message,
    limits: &WebSocketLimits,
) -> std::result::Result<(), ()> {
    tokio::time::timeout(
        Duration::from_millis(limits.send_timeout_ms),
        socket.send(message),
    )
    .await
    .map_err(|_| ())?
    .map_err(|_| ())
}

async fn close_socket(socket: &mut WebSocket, code: u16, reason: &'static str) {
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })))
        .await;
}

fn service_status(kind: ServiceErrorKind) -> StatusCode {
    match kind {
        ServiceErrorKind::InvalidArgument => StatusCode::BAD_REQUEST,
        ServiceErrorKind::NotFound => StatusCode::NOT_FOUND,
        ServiceErrorKind::Unauthenticated => StatusCode::UNAUTHORIZED,
        ServiceErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
        ServiceErrorKind::Conflict => StatusCode::CONFLICT,
        ServiceErrorKind::FailedPrecondition => StatusCode::PRECONDITION_FAILED,
        ServiceErrorKind::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
        ServiceErrorKind::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
        ServiceErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
