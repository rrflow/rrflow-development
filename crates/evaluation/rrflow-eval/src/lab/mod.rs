//! A comparison lab: one typed workload run against RRFlow and other systems.
//!
//! Results are diagnostic evidence for optimization. They configure nothing,
//! and they are not J-04 acceptance, parity or superiority claims. The harness
//! grades every system with its own exact oracle, so no system grades itself.
pub mod adapter;
pub mod corpus;
pub mod custom;
pub mod driver;
pub mod manifest;
pub mod milvus;
pub mod qdrant;
pub mod report;
pub mod rrflow;
pub mod surrealdb;

use serde::Serialize;
use sha2::{Digest as _, Sha256};

pub type Result<T> = std::result::Result<T, String>;

pub(crate) fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// JSON with sorted object keys, so a digest does not depend on field order.
pub fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut canonical = serde_json::to_value(value).map_err(|error| error.to_string())?;
    canonical.sort_all_objects();
    let mut bytes = serde_json::to_vec_pretty(&canonical).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
