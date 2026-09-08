//! Current closed public client error vocabulary and classifications.

use hyper::StatusCode;
use rrd_contract::{ErrorBody, ErrorCode};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Contract(String),
    Transport(String),
    Timeout,
    ResponseTooLarge,
    Decode(String),
    Subscription {
        error: ErrorBody,
        acknowledged_cursor: u64,
    },
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
            Self::Subscription {
                error,
                acknowledged_cursor,
            } => {
                write!(
                    formatter,
                    "RRD subscription {:?} after cursor {acknowledged_cursor}: {}",
                    error.code, error.message,
                )
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

pub fn is_unauthenticated(error: &Error) -> bool {
    matches!(
        error,
        Error::Api {
            error: ErrorBody {
                code: ErrorCode::Unauthenticated,
                ..
            },
            ..
        } | Error::Subscription {
            error: ErrorBody {
                code: ErrorCode::Unauthenticated,
                ..
            },
            ..
        }
    )
}
