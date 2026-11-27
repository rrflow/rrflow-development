//! Shared test-evidence interpretation owned by rrd-contract.
#![allow(dead_code)] // each test crate uses only its required fixture operations
#[path = "../../../../contract/tests/support/observed_configuration.rs"]
mod recorded;
pub use recorded::*;
