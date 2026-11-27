//! Current closed public client error vocabulary and classifications.

use hyper::StatusCode;
use rrd_contract::{ErrorBody, ErrorCode, WebSocketError};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Contract(String),
    Transport(String),
    Timeout,
    ResponseTooLarge,
    Decode(String),
    WebSocket(Box<WebSocketError>),
    WebSocketProtocol(String),
    Api {
        status: StatusCode,
        error: ErrorBody,
    },
    ResponseIdentityMismatch,
    UnsupportedProtocol {
        protocol: String,
        version: u16,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract(message) => write!(formatter, "RRD client contract error: {message}"),
            Self::Transport(message) => write!(formatter, "RRD transport error: {message}"),
            Self::Timeout => formatter.write_str("RRD request timed out"),
            Self::ResponseTooLarge => formatter.write_str("RRD response exceeded four MiB"),
            Self::Decode(message) => write!(formatter, "RRD response decode failed: {message}"),
            Self::WebSocket(error) => {
                write!(
                    formatter,
                    "RRD WebSocket {:?}: {:?}: {}",
                    error.target, error.error.code, error.error.message
                )
            }
            Self::WebSocketProtocol(message) => {
                write!(formatter, "RRD WebSocket protocol error: {message}")
            }
            Self::Api { status, error } => {
                write!(
                    formatter,
                    "RRD API {}: {:?}: {}",
                    status, error.code, error.message
                )
            }
            Self::ResponseIdentityMismatch => {
                formatter.write_str("RRD response request/operation identity differs")
            }
            Self::UnsupportedProtocol { protocol, version } => {
                write!(
                    formatter,
                    "unsupported RRD protocol {protocol:?} version {version}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

pub(crate) fn contract(error: impl fmt::Display) -> Error {
    Error::Contract(error.to_string())
}

pub(crate) fn decode(error: impl fmt::Display) -> Error {
    Error::Decode(error.to_string())
}

pub(crate) fn websocket_connect_error(error: tokio_tungstenite::tungstenite::Error) -> Error {
    match error {
        tokio_tungstenite::tungstenite::Error::Http(response) => {
            let status = response.status();
            let body = response
                .body()
                .as_deref()
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
                .unwrap_or("empty response body");
            Error::Transport(format!("WebSocket upgrade failed with {status}: {body}"))
        }
        error => Error::Transport(error.to_string()),
    }
}

pub(crate) fn websocket_stream_error(error: tokio_tungstenite::tungstenite::Error) -> Error {
    use tokio_tungstenite::tungstenite::Error as WebSocketError;

    match error {
        WebSocketError::Capacity(_)
        | WebSocketError::Protocol(_)
        | WebSocketError::Utf8(_)
        | WebSocketError::AttackAttempt => Error::WebSocketProtocol(error.to_string()),
        error => Error::Transport(error.to_string()),
    }
}

pub fn is_unauthenticated(error: &Error) -> bool {
    match error {
        Error::Api {
            error:
                ErrorBody {
                    code: ErrorCode::Unauthenticated,
                    ..
                },
            ..
        } => true,
        Error::WebSocket(error) => error.error.code == ErrorCode::Unauthenticated,
        _ => false,
    }
}
