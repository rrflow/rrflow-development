use rrd_contract::{
    decode_websocket_frame, encode_websocket_frame, validate_websocket_cancellation_correlation,
    validate_websocket_response_correlation, validate_websocket_subscription_correlation,
    CanonicalId, ChangeMutationSnapshot, ChangefeedPage, ChangefeedValidation, ClaimChangeSnapshot,
    ClaimPromotionSnapshot, ClaimTierSnapshot, CorrelationId, ErrorBody, ErrorCode, QueryBudget,
    RequestContext, RequestEnvelope, ResourceId, ResourceKind, ResourcePath, ResponseEnvelope,
    ResponseOutcome, RuntimeChangeSnapshot, SubscriptionAcknowledgement, SubscriptionDelivery,
    SubscriptionLeaseCoordinate, SubscriptionResume, SubscriptionSnapshot, SubscriptionStatus,
    WebSocketAcknowledged, WebSocketBackpressure, WebSocketBackpressureTarget, WebSocketCancel,
    WebSocketCancellation, WebSocketCancellationDisposition, WebSocketConnected, WebSocketDelivery,
    WebSocketError, WebSocketErrorTarget, WebSocketFrame, WebSocketHeartbeat, WebSocketLimits,
    WebSocketPayload, WebSocketPeer, WebSocketReceiveState, WebSocketRequest,
    WebSocketRequestTarget, WebSocketResponse, WebSocketSendState, WebSocketSubscribe,
    WebSocketSubscribed, WebSocketUnsubscribe, WebSocketUnsubscribed, MAX_WEBSOCKET_MESSAGE_BYTES,
    MIN_WEBSOCKET_HEARTBEAT_MS, PROTOCOL, PROTOCOL_VERSION, WEBSOCKET_CONTRACT_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WebSocketFixture {
    contract_version: u16,
    limits: WebSocketLimits,
    frames: Vec<DirectedFrame>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectedFrame {
    sender: WebSocketPeer,
    frame: WebSocketFrame,
}

fn id(value: &str) -> CorrelationId {
    CorrelationId::new(value).unwrap()
}

fn request_target() -> WebSocketRequestTarget {
    WebSocketRequestTarget {
        operation: CanonicalId::new("query-execute").unwrap(),
        request_id: id("request-query-01"),
        operation_id: id("operation-query-01"),
    }
}

fn request() -> WebSocketRequest {
    let target = request_target();
    WebSocketRequest {
        operation: target.operation,
        request: RequestEnvelope {
            protocol: PROTOCOL.into(),
            protocol_version: PROTOCOL_VERSION,
            context: RequestContext {
                request_id: target.request_id,
                operation_id: target.operation_id,
                idempotency_key: None,
                deadline_unix_ms: Some(2_000_000_000_000),
            },
            resource: ResourcePath {
                segments: vec![ResourceId::new(ResourceKind::Instance, "project-alpha").unwrap()],
            },
            payload: serde_json::json!({
                "scope": "instance:project-alpha",
                "query": "FROM record:document AT VALID 42 KNOWN HEAD PROJECT title",
                "parameters": {},
                "budget": QueryBudget::default()
            }),
        },
    }
}

fn response() -> WebSocketResponse {
    let target = request_target();
    WebSocketResponse {
        operation: target.operation,
        response: ResponseEnvelope {
            protocol: PROTOCOL.into(),
            protocol_version: PROTOCOL_VERSION,
            request_id: target.request_id,
            operation_id: target.operation_id,
            outcome: ResponseOutcome::Ok {
                payload: serde_json::json!({"known_at_cursor": 9, "rows": []}),
            },
        },
    }
}

fn snapshot(acknowledged_cursor: u64) -> SubscriptionSnapshot {
    SubscriptionSnapshot {
        subscription_id: id("subscription-alpha"),
        stream_sha256: "a".repeat(64),
        acknowledged_cursor,
        retention_floor_cursor: 2,
        head_cursor: 9,
        batch_size: 16,
        max_in_flight: 4,
        connection_generation: 4,
        lease_expires_at_unix_ms: 2_000_000_010_000,
        heartbeat_interval_ms: 500,
        status: SubscriptionStatus::Open,
    }
}

fn resume() -> SubscriptionResume {
    SubscriptionResume {
        subscription_id: id("subscription-alpha"),
        stream_sha256: "a".repeat(64),
        connection_generation: 3,
        acknowledged_cursor: 7,
    }
}

fn delivery() -> WebSocketDelivery {
    WebSocketDelivery {
        subscription_id: id("subscription-alpha"),
        connection_generation: 4,
        delivery_sequence: 11,
        from_cursor: 7,
        through_cursor: 8,
        delivery: SubscriptionDelivery::Changefeed {
            page: ChangefeedPage {
                requested_after_cursor: 7,
                through_cursor: 8,
                head_cursor: 9,
                has_more: true,
                validation: ChangefeedValidation {
                    method: "full_hash_chain_replay".into(),
                    change_reads: 1,
                    proof_nodes: 0,
                },
                changes: vec![RuntimeChangeSnapshot {
                    cursor: 8,
                    commit_sha256: "b".repeat(64),
                    commit_ordinal: 0,
                    scope: "instance:project-alpha".into(),
                    at_unix_ms: 1_900_000_000_000,
                    actor: "session:session-alpha".into(),
                    mutation: ChangeMutationSnapshot::Claim {
                        claim: ClaimChangeSnapshot {
                            subject: "document-alpha".into(),
                            predicate: "status".into(),
                            object: "indexed".into(),
                            valid_from: 42,
                            valid_to: None,
                            tx_time: 43,
                            producer: "agent-alpha".into(),
                            on_behalf_of: None,
                            session: Some("session-alpha".into()),
                            confidence: None,
                            supersedes_sha256: None,
                            signature: None,
                            tier: ClaimTierSnapshot::Local,
                            promotion: ClaimPromotionSnapshot::Unpromoted,
                        },
                    },
                    previous_change_sha256: Some("c".repeat(64)),
                    change_sha256: "d".repeat(64),
                }],
            },
        },
    }
}

fn acknowledgement() -> SubscriptionAcknowledgement {
    SubscriptionAcknowledgement {
        subscription_id: id("subscription-alpha"),
        connection_generation: 4,
        delivery_sequence: 11,
        through_cursor: 8,
    }
}

fn frame(
    connection_id: &CorrelationId,
    sender: WebSocketPeer,
    sequence: u64,
    payload: WebSocketPayload,
) -> DirectedFrame {
    DirectedFrame {
        sender,
        frame: WebSocketFrame {
            protocol: PROTOCOL.into(),
            protocol_version: PROTOCOL_VERSION,
            connection_id: connection_id.clone(),
            sequence,
            payload,
        },
    }
}

fn fixture() -> WebSocketFixture {
    let limits = WebSocketLimits::default();
    let connection = id("connection-alpha");
    let request = request();
    let target = request.target();
    let cancellation = WebSocketCancel {
        cancellation_id: id("cancellation-query-01"),
        target: target.clone(),
        reason: "caller deadline elapsed".into(),
    };
    WebSocketFixture {
        contract_version: WEBSOCKET_CONTRACT_VERSION,
        limits: limits.clone(),
        frames: vec![
            frame(
                &connection,
                WebSocketPeer::Server,
                1,
                WebSocketPayload::Connected(WebSocketConnected {
                    contract_version: WEBSOCKET_CONTRACT_VERSION,
                    session_id: id("session-alpha"),
                    limits,
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Client,
                1,
                WebSocketPayload::Request(request),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                2,
                WebSocketPayload::Response(response()),
            ),
            frame(
                &connection,
                WebSocketPeer::Client,
                2,
                WebSocketPayload::Cancel(cancellation.clone()),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                3,
                WebSocketPayload::Cancellation(WebSocketCancellation {
                    cancellation_id: cancellation.cancellation_id,
                    target: target.clone(),
                    disposition: WebSocketCancellationDisposition::AlreadyCompleted,
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Client,
                3,
                WebSocketPayload::Subscribe(WebSocketSubscribe { resume: resume() }),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                4,
                WebSocketPayload::Subscribed(WebSocketSubscribed {
                    subscription: snapshot(7),
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                5,
                WebSocketPayload::Delivery(delivery()),
            ),
            frame(
                &connection,
                WebSocketPeer::Client,
                4,
                WebSocketPayload::Ack(acknowledgement()),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                6,
                WebSocketPayload::Acknowledged(WebSocketAcknowledged {
                    acknowledgement: acknowledgement(),
                    subscription: snapshot(8),
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Client,
                5,
                WebSocketPayload::Heartbeat(WebSocketHeartbeat {
                    heartbeat_id: id("heartbeat-client-01"),
                    observed_sequence: 6,
                    subscriptions: vec![SubscriptionLeaseCoordinate {
                        subscription_id: id("subscription-alpha"),
                        connection_generation: 4,
                        acknowledged_cursor: 8,
                    }],
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                7,
                WebSocketPayload::Heartbeat(WebSocketHeartbeat {
                    heartbeat_id: id("heartbeat-server-01"),
                    observed_sequence: 5,
                    subscriptions: vec![SubscriptionLeaseCoordinate {
                        subscription_id: id("subscription-alpha"),
                        connection_generation: 4,
                        acknowledged_cursor: 8,
                    }],
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                8,
                WebSocketPayload::Backpressure(WebSocketBackpressure {
                    target: WebSocketBackpressureTarget::Subscription {
                        subscription_id: id("subscription-alpha"),
                        connection_generation: 4,
                    },
                    in_flight: 4,
                    limit: 4,
                    retry_after_ms: 100,
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                9,
                WebSocketPayload::Error(WebSocketError {
                    target: WebSocketErrorTarget::Request { target },
                    error: ErrorBody {
                        code: ErrorCode::FailedPrecondition,
                        message: "request execution is not enabled on this endpoint".into(),
                        retryable: false,
                        details: BTreeMap::new(),
                    },
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Client,
                6,
                WebSocketPayload::Unsubscribe(WebSocketUnsubscribe {
                    subscription_id: id("subscription-alpha"),
                    connection_generation: 4,
                }),
            ),
            frame(
                &connection,
                WebSocketPeer::Server,
                10,
                WebSocketPayload::Unsubscribed(WebSocketUnsubscribed {
                    subscription: snapshot(8),
                }),
            ),
        ],
    }
}

#[test]
fn multiplex_protocol_matches_the_language_neutral_golden() {
    let actual = fixture();
    let expected: WebSocketFixture =
        serde_json::from_str(include_str!("../fixtures/websocket-protocol-v1.json")).unwrap();
    assert_eq!(actual, expected);

    let connection = id("connection-alpha");
    let mut client_receiver = WebSocketReceiveState::for_connection(
        WebSocketPeer::Client,
        connection.clone(),
        actual.limits.clone(),
    )
    .unwrap();
    let mut server_receiver =
        WebSocketReceiveState::awaiting_server(actual.limits.clone()).unwrap();
    for directed in actual.frames {
        let bytes =
            encode_websocket_frame(&directed.frame, directed.sender, &actual.limits).unwrap();
        assert_eq!(
            decode_websocket_frame(&bytes, directed.sender, &actual.limits).unwrap(),
            directed.frame
        );
        match directed.sender {
            WebSocketPeer::Client => {
                assert_eq!(client_receiver.accept(&bytes).unwrap(), directed.frame)
            }
            WebSocketPeer::Server => {
                assert_eq!(server_receiver.accept(&bytes).unwrap(), directed.frame)
            }
        }
    }
    assert_eq!(client_receiver.last_sequence(), 6);
    assert_eq!(server_receiver.last_sequence(), 10);
    assert_eq!(server_receiver.connection_id(), Some(&connection));
}

#[test]
fn sequence_connection_direction_and_unknown_fields_fail_closed() {
    let fixture = fixture();
    let first = &fixture.frames[0].frame;
    let bytes = encode_websocket_frame(first, WebSocketPeer::Server, &fixture.limits).unwrap();
    assert!(decode_websocket_frame(&bytes, WebSocketPeer::Client, &fixture.limits).is_err());

    let mut receiver = WebSocketReceiveState::awaiting_server(fixture.limits.clone()).unwrap();
    receiver.accept(&bytes).unwrap();
    assert!(receiver.accept(&bytes).is_err());

    let mut gap = fixture.frames[2].frame.clone();
    gap.sequence = 3;
    assert!(receiver
        .accept(&encode_websocket_frame(&gap, WebSocketPeer::Server, &fixture.limits).unwrap())
        .is_err());

    let mut foreign = fixture.frames[2].frame.clone();
    foreign.connection_id = id("connection-foreign");
    assert!(receiver
        .accept(&encode_websocket_frame(&foreign, WebSocketPeer::Server, &fixture.limits).unwrap())
        .is_err());

    let mut unknown = serde_json::to_value(first).unwrap();
    unknown["payload"]["body"]["unknown"] = serde_json::json!(true);
    assert!(decode_websocket_frame(
        &serde_json::to_vec(&unknown).unwrap(),
        WebSocketPeer::Server,
        &fixture.limits
    )
    .is_err());

    let mut unknown_payload_member = serde_json::to_value(first).unwrap();
    unknown_payload_member["payload"]["unknown"] = serde_json::json!(true);
    assert!(decode_websocket_frame(
        &serde_json::to_vec(&unknown_payload_member).unwrap(),
        WebSocketPeer::Server,
        &fixture.limits
    )
    .is_err());
}

#[test]
fn oversized_and_unbounded_inputs_are_rejected_before_use() {
    let limits = WebSocketLimits::default();
    assert!(decode_websocket_frame(
        &vec![b' '; limits.max_message_bytes as usize + 1],
        WebSocketPeer::Client,
        &limits
    )
    .is_err());

    let mut invalid = limits.clone();
    invalid.max_write_buffer_bytes = invalid.write_buffer_bytes;
    assert!(invalid.validate().is_err());
    invalid = limits.clone();
    invalid.heartbeat_interval_ms = invalid.receive_timeout_ms;
    assert!(invalid.validate().is_err());
    invalid = limits.clone();
    invalid.heartbeat_interval_ms = MIN_WEBSOCKET_HEARTBEAT_MS - 1;
    assert!(invalid.validate().is_err());

    let mut request = request();
    let mut value = serde_json::Value::Null;
    for _ in 0..=rrd_contract::MAX_WEBSOCKET_JSON_DEPTH {
        value = serde_json::json!([value]);
    }
    request.request.payload = value;
    let frame = WebSocketFrame {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        connection_id: id("connection-alpha"),
        sequence: 1,
        payload: WebSocketPayload::Request(request),
    };
    assert!(encode_websocket_frame(&frame, WebSocketPeer::Client, &limits).is_err());

    let mut error_frame = fixture().frames[13].frame.clone();
    let WebSocketPayload::Error(error) = &mut error_frame.payload else {
        unreachable!()
    };
    for index in 0..=rrd_contract::MAX_WEBSOCKET_ERROR_DETAILS {
        error.error.details.insert(
            CanonicalId::new(format!("detail-{index}")).unwrap(),
            "bounded".into(),
        );
    }
    assert!(encode_websocket_frame(&error_frame, WebSocketPeer::Server, &limits).is_err());
}

#[test]
fn cursor_only_subscription_progress_is_a_valid_delivery() {
    let limits = WebSocketLimits::default();
    let mut changefeed = delivery();
    let SubscriptionDelivery::Changefeed { page } = &mut changefeed.delivery else {
        unreachable!()
    };
    page.changes.clear();
    let changefeed_frame = WebSocketFrame {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        connection_id: id("connection-cursor-only-changefeed"),
        sequence: 1,
        payload: WebSocketPayload::Delivery(changefeed),
    };
    encode_websocket_frame(&changefeed_frame, WebSocketPeer::Server, &limits).unwrap();

    let live_query = WebSocketDelivery {
        subscription_id: id("subscription-cursor-only-live-query"),
        connection_generation: 2,
        delivery_sequence: 3,
        from_cursor: 7,
        through_cursor: 8,
        delivery: SubscriptionDelivery::LiveQuery {
            delta: rrd_contract::LiveQueryDeltaResult {
                timed_out: false,
                waited_ms: 0,
                query_sha256: "e".repeat(64),
                from_cursor: 7,
                through_cursor: 8,
                head_cursor: 8,
                added: Vec::new(),
                updated: Vec::new(),
                removed: Vec::new(),
            },
        },
    };
    let live_query_frame = WebSocketFrame {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        connection_id: id("connection-cursor-only-live-query"),
        sequence: 1,
        payload: WebSocketPayload::Delivery(live_query),
    };
    encode_websocket_frame(&live_query_frame, WebSocketPeer::Server, &limits).unwrap();
}

#[test]
fn request_cancellation_and_resume_correlations_reject_substitution() {
    let request = request();
    let response = response();
    validate_websocket_response_correlation(&request, &response).unwrap();
    let mut wrong_response = response;
    wrong_response.response.request_id = id("request-substituted");
    assert!(validate_websocket_response_correlation(&request, &wrong_response).is_err());

    let cancel = WebSocketCancel {
        cancellation_id: id("cancellation-query-01"),
        target: request.target(),
        reason: "caller stopped waiting".into(),
    };
    let result = WebSocketCancellation {
        cancellation_id: cancel.cancellation_id.clone(),
        target: cancel.target.clone(),
        disposition: WebSocketCancellationDisposition::Cancelled,
    };
    validate_websocket_cancellation_correlation(&cancel, &result).unwrap();
    let mut wrong_result = result;
    wrong_result.target.operation_id = id("operation-substituted");
    assert!(validate_websocket_cancellation_correlation(&cancel, &wrong_result).is_err());

    let resume = resume();
    validate_websocket_subscription_correlation(&resume, &snapshot(7)).unwrap();
    let mut stale = snapshot(7);
    stale.connection_generation = 3;
    assert!(validate_websocket_subscription_correlation(&resume, &stale).is_err());
    let mut drifted = snapshot(7);
    drifted.stream_sha256 = "e".repeat(64);
    assert!(validate_websocket_subscription_correlation(&resume, &drifted).is_err());

    let mut forged_cancel = cancel;
    forged_cancel.target.operation = CanonicalId::new("uncatalogued-operation").unwrap();
    let forged_frame = WebSocketFrame {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        connection_id: id("connection-alpha"),
        sequence: 1,
        payload: WebSocketPayload::Cancel(forged_cancel),
    };
    assert!(encode_websocket_frame(
        &forged_frame,
        WebSocketPeer::Client,
        &WebSocketLimits::default()
    )
    .is_err());
}

#[test]
fn negotiated_lower_limits_govern_every_later_message() {
    let admission = WebSocketLimits::default();
    let negotiated = WebSocketLimits {
        max_message_bytes: 1_024,
        max_frame_bytes: 1_024,
        read_buffer_bytes: 512,
        write_buffer_bytes: 512,
        max_write_buffer_bytes: 2_048,
        max_in_flight_requests: 4,
        max_subscriptions: 2,
        send_timeout_ms: 1_000,
        receive_timeout_ms: 2_000,
        heartbeat_interval_ms: 500,
    };
    negotiated.validate().unwrap();
    let connection = id("connection-lower-limits");
    let connected = WebSocketFrame {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        connection_id: connection.clone(),
        sequence: 1,
        payload: WebSocketPayload::Connected(WebSocketConnected {
            contract_version: WEBSOCKET_CONTRACT_VERSION,
            session_id: id("session-lower-limits"),
            limits: negotiated,
        }),
    };
    let mut receiver = WebSocketReceiveState::awaiting_server(admission.clone()).unwrap();
    receiver
        .accept(&encode_websocket_frame(&connected, WebSocketPeer::Server, &admission).unwrap())
        .unwrap();

    let oversized_after_negotiation = WebSocketFrame {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        connection_id: connection,
        sequence: 2,
        payload: WebSocketPayload::Error(WebSocketError {
            target: WebSocketErrorTarget::Connection,
            error: ErrorBody {
                code: ErrorCode::Internal,
                message: "x".repeat(2_048),
                retryable: false,
                details: BTreeMap::new(),
            },
        }),
    };
    let bytes = encode_websocket_frame(
        &oversized_after_negotiation,
        WebSocketPeer::Server,
        &admission,
    )
    .unwrap();
    assert!(receiver.accept(&bytes).is_err());

    for mutate in [
        |limits: &mut WebSocketLimits| limits.send_timeout_ms += 1,
        |limits: &mut WebSocketLimits| limits.receive_timeout_ms += 1,
        |limits: &mut WebSocketLimits| limits.heartbeat_interval_ms += 1,
    ] {
        let mut enlarged = admission.clone();
        mutate(&mut enlarged);
        let connected = WebSocketFrame {
            protocol: PROTOCOL.into(),
            protocol_version: PROTOCOL_VERSION,
            connection_id: id("connection-enlarged-time-limit"),
            sequence: 1,
            payload: WebSocketPayload::Connected(WebSocketConnected {
                contract_version: WEBSOCKET_CONTRACT_VERSION,
                session_id: id("session-enlarged-time-limit"),
                limits: enlarged,
            }),
        };
        let mut receiver = WebSocketReceiveState::awaiting_server(admission.clone()).unwrap();
        let bytes = serde_json::to_vec(&connected).unwrap();
        assert!(receiver.accept(&bytes).is_err());
    }
}

#[test]
fn send_state_assigns_sequences_only_after_valid_encoding() {
    let limits = WebSocketLimits::default();
    let mut state = WebSocketSendState::new(
        WebSocketPeer::Server,
        id("connection-alpha"),
        limits.clone(),
    )
    .unwrap();
    assert!(state
        .encode(WebSocketPayload::Heartbeat(WebSocketHeartbeat {
            heartbeat_id: id("heartbeat-invalid-first"),
            observed_sequence: 1,
            subscriptions: Vec::new(),
        }))
        .is_err());
    assert_eq!(state.last_sequence(), 0);
    state
        .encode(WebSocketPayload::Connected(WebSocketConnected {
            contract_version: WEBSOCKET_CONTRACT_VERSION,
            session_id: id("session-alpha"),
            limits,
        }))
        .unwrap();
    assert_eq!(state.last_sequence(), 1);
}

#[test]
fn golden_message_ceiling_is_explicit() {
    assert_eq!(MAX_WEBSOCKET_MESSAGE_BYTES, 1024 * 1024);
}
