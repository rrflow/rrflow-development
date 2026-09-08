use futures_util::{SinkExt, StreamExt};
use rrd_client::{ClientConfig, Error, RequestOptions, RrdClient, Session};
use rrd_contract::{
    encode_websocket_frame, CanonicalId, CorrelationId, CreateSession, ErrorBody, ErrorCode,
    RequestContext, RequestEnvelope, ResourceId, ResourceKind, ResourcePath, ResponseEnvelope,
    ResponseOutcome, SessionLease, SessionLimits, WebSocketConnected, WebSocketError,
    WebSocketErrorTarget, WebSocketFrame, WebSocketLimits, WebSocketPayload, WebSocketPeer,
    WebSocketReceiveState, WebSocketRequest, WebSocketResponse, WebSocketSendState, PROTOCOL,
    PROTOCOL_VERSION, WEBSOCKET_CONTRACT_VERSION,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone, Copy)]
enum ServerFault {
    ForeignConnection,
    SequenceGap,
    UnknownPayloadMember,
    BinaryApplicationMessage,
    OversizedMessage,
    MismatchedResponse,
}

fn id(value: &str) -> CorrelationId {
    CorrelationId::new(value).unwrap()
}

fn instance() -> CanonicalId {
    CanonicalId::new("websocket-fault-instance").unwrap()
}

async fn mock_session() -> Session {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let response = ResponseEnvelope {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        request_id: id("fault-session-request"),
        operation_id: id("fault-session-operation"),
        outcome: ResponseOutcome::Ok {
            payload: SessionLease {
                session_id: id("fault-session"),
                token: id("fault-session-token"),
                issued_at_unix_ms: 1,
                idle_expires_at_unix_ms: u64::MAX - 1,
                absolute_expires_at_unix_ms: u64::MAX,
                limits: SessionLimits {
                    idle_timeout_ms: 60_000,
                    absolute_timeout_ms: 300_000,
                    max_open_transactions: 1,
                },
            },
        },
    };
    let body = serde_json::to_vec(&response).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = vec![0; 8 * 1024];
        let _ = stream.read(&mut request).await.unwrap();
        let headers = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(headers.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        stream.shutdown().await.unwrap();
    });
    let client = RrdClient::connect_local(
        address,
        instance(),
        ClientConfig {
            request_timeout: Duration::from_secs(2),
            max_attempts: 1,
            websocket_limits: WebSocketLimits::default(),
        },
    )
    .unwrap();
    let session = client
        .create_session(
            CanonicalId::new("fault-principal").unwrap(),
            "fault-key",
            CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 60_000,
                    absolute_timeout_ms: 300_000,
                    max_open_transactions: 1,
                },
            },
            RequestOptions::mutation(
                "fault-session-request",
                "fault-session-operation",
                "fault-session-key",
            )
            .unwrap(),
        )
        .await
        .unwrap();
    server.await.unwrap();
    session
}

fn request() -> WebSocketRequest {
    WebSocketRequest {
        operation: CanonicalId::new("query-execute").unwrap(),
        request: RequestEnvelope {
            protocol: PROTOCOL.into(),
            protocol_version: PROTOCOL_VERSION,
            context: RequestContext {
                request_id: id("fault-query-request"),
                operation_id: id("fault-query-operation"),
                idempotency_key: None,
                deadline_unix_ms: None,
            },
            resource: ResourcePath {
                segments: vec![
                    ResourceId::new(ResourceKind::Instance, "websocket-fault-instance").unwrap(),
                ],
            },
            payload: json!({}),
        },
    }
}

fn error_frame(connection_id: CorrelationId, sequence: u64) -> WebSocketFrame {
    WebSocketFrame {
        protocol: PROTOCOL.into(),
        protocol_version: PROTOCOL_VERSION,
        connection_id,
        sequence,
        payload: WebSocketPayload::Error(WebSocketError {
            target: WebSocketErrorTarget::Connection,
            error: ErrorBody {
                code: ErrorCode::Internal,
                message: "injected server fault".into(),
                retryable: false,
                details: BTreeMap::new(),
            },
        }),
    }
}

async fn run_fault(fault: ServerFault) -> Error {
    let session = mock_session().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let limits = WebSocketLimits::default();
    let connection_id = id("fault-connection");
    let server_limits = limits.clone();
    let server_connection = connection_id.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut sender = WebSocketSendState::new(
            WebSocketPeer::Server,
            server_connection.clone(),
            server_limits.clone(),
        )
        .unwrap();
        let connected = sender
            .encode(WebSocketPayload::Connected(WebSocketConnected {
                contract_version: WEBSOCKET_CONTRACT_VERSION,
                session_id: id("fault-session"),
                limits: server_limits.clone(),
            }))
            .unwrap();
        socket
            .send(Message::Text(String::from_utf8(connected).unwrap().into()))
            .await
            .unwrap();

        match fault {
            ServerFault::ForeignConnection => {
                let bytes = encode_websocket_frame(
                    &error_frame(id("foreign-connection"), 2),
                    WebSocketPeer::Server,
                    &server_limits,
                )
                .unwrap();
                socket
                    .send(Message::Text(String::from_utf8(bytes).unwrap().into()))
                    .await
                    .unwrap();
            }
            ServerFault::SequenceGap => {
                let bytes = encode_websocket_frame(
                    &error_frame(server_connection, 3),
                    WebSocketPeer::Server,
                    &server_limits,
                )
                .unwrap();
                socket
                    .send(Message::Text(String::from_utf8(bytes).unwrap().into()))
                    .await
                    .unwrap();
            }
            ServerFault::UnknownPayloadMember => {
                let frame = error_frame(server_connection, 2);
                let mut value = serde_json::to_value(frame).unwrap();
                value["payload"]["unknown"] = json!(true);
                socket
                    .send(Message::Text(value.to_string().into()))
                    .await
                    .unwrap();
            }
            ServerFault::BinaryApplicationMessage => {
                socket
                    .send(Message::Binary(vec![0, 1, 2, 3].into()))
                    .await
                    .unwrap();
            }
            ServerFault::OversizedMessage => {
                socket
                    .send(Message::Text(
                        "x".repeat(server_limits.max_message_bytes as usize + 1)
                            .into(),
                    ))
                    .await
                    .unwrap();
            }
            ServerFault::MismatchedResponse => {
                let message = socket.next().await.unwrap().unwrap();
                let Message::Text(text) = message else {
                    panic!("client request was not a text application message");
                };
                let mut receiver = WebSocketReceiveState::for_connection(
                    WebSocketPeer::Client,
                    server_connection.clone(),
                    server_limits.clone(),
                )
                .unwrap();
                let client_frame = receiver.accept(text.as_bytes()).unwrap();
                assert!(matches!(client_frame.payload, WebSocketPayload::Request(_)));
                let response = WebSocketFrame {
                    protocol: PROTOCOL.into(),
                    protocol_version: PROTOCOL_VERSION,
                    connection_id: server_connection,
                    sequence: 2,
                    payload: WebSocketPayload::Response(WebSocketResponse {
                        operation: CanonicalId::new("query-execute").unwrap(),
                        response: ResponseEnvelope {
                            protocol: PROTOCOL.into(),
                            protocol_version: PROTOCOL_VERSION,
                            request_id: id("substituted-request"),
                            operation_id: id("fault-query-operation"),
                            outcome: ResponseOutcome::Ok { payload: json!({}) },
                        },
                    }),
                };
                let bytes =
                    encode_websocket_frame(&response, WebSocketPeer::Server, &server_limits)
                        .unwrap();
                socket
                    .send(Message::Text(String::from_utf8(bytes).unwrap().into()))
                    .await
                    .unwrap();
            }
        }
    });

    let client = RrdClient::connect_local(
        address,
        instance(),
        ClientConfig {
            request_timeout: Duration::from_secs(2),
            max_attempts: 1,
            websocket_limits: limits,
        },
    )
    .unwrap();
    let mut socket = client.connect_websocket(&session).await.unwrap();
    if matches!(fault, ServerFault::MismatchedResponse) {
        socket.send_request(request()).await.unwrap();
    }
    let error = socket.receive().await.unwrap_err();
    server.await.unwrap();
    error
}

#[tokio::test]
async fn client_rejects_sequence_and_connection_substitution() {
    assert!(matches!(
        run_fault(ServerFault::ForeignConnection).await,
        Error::WebSocketProtocol(_)
    ));
    assert!(matches!(
        run_fault(ServerFault::SequenceGap).await,
        Error::WebSocketProtocol(_)
    ));
}

#[tokio::test]
async fn client_rejects_unknown_binary_and_oversized_messages() {
    assert!(matches!(
        run_fault(ServerFault::UnknownPayloadMember).await,
        Error::WebSocketProtocol(_)
    ));
    assert!(matches!(
        run_fault(ServerFault::BinaryApplicationMessage).await,
        Error::WebSocketProtocol(_)
    ));
    assert!(matches!(
        run_fault(ServerFault::OversizedMessage).await,
        Error::WebSocketProtocol(_)
    ));
}

#[tokio::test]
async fn client_rejects_response_correlation_substitution() {
    assert!(matches!(
        run_fault(ServerFault::MismatchedResponse).await,
        Error::WebSocketProtocol(_)
    ));
}
