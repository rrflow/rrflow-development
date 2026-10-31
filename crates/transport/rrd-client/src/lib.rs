//! Supported asynchronous Rust client for the public RRD v1 protocol.
//!
//! This crate depends on `rrd-contract`, never on RRD storage or query
//! internals. Loopback HTTP remains available for local development. Remote
//! endpoints use an explicit mutually authenticated TLS configuration.

mod attunement;
mod client;
mod endpoint;
mod error;
mod operation;
mod retry;
mod session;
mod subscription;
mod transport;
mod worker;

#[allow(dead_code)] // The complete generated projection precedes full H-04 client migration.
mod generated_operations {
    pub(crate) trait GeneratedOperation {
        type Request: serde::Serialize;
        type Response: serde::de::DeserializeOwned;
        const ID: &'static str;
    }
    include!(concat!(env!("OUT_DIR"), "/operation_bindings.rs"));

    #[cfg(test)]
    mod tests {
        use super::GENERATED_IDS;

        #[test]
        fn compiled_projection_covers_the_executable_catalogue_exactly() {
            let catalogue = rrd_contract::endpoint_catalogue();
            let mut actual = GENERATED_IDS.to_vec();
            let mut expected = catalogue
                .endpoints
                .iter()
                .map(|entry| entry.operation.as_str())
                .collect::<Vec<_>>();
            actual.sort_unstable();
            expected.sort_unstable();
            assert_eq!(actual, expected);
        }
    }
}

pub use client::RrdClient;
pub use error::{is_unauthenticated, Error, Result};
pub use operation::RequestOptions;
pub use retry::ClientConfig;
pub use session::Session;
pub use subscription::RrdWebSocket;
pub use transport::MAX_RESPONSE_BYTES;
