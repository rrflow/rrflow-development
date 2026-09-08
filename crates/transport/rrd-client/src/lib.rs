//! Supported asynchronous Rust client for the public RRD v1 protocol.
//!
//! This crate depends on `rrd-contract`, never on RRD storage or query
//! internals. Loopback HTTP remains available for local development. Remote
//! endpoints use an explicit mutually authenticated TLS configuration.

mod client;
mod endpoint;
mod error;
mod operation;
mod retry;
mod session;
mod subscription;
mod transport;

pub use client::RrdClient;
pub use error::{is_unauthenticated, Error, Result};
pub use operation::RequestOptions;
pub use retry::ClientConfig;
pub use session::Session;
pub use subscription::SubscriptionSocket;
pub use transport::MAX_RESPONSE_BYTES;
