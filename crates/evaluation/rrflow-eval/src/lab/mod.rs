//! A comparison lab: one typed workload run against RRFlow and other systems.
//! Its shared vocabulary and report format are `rrflow_evaluation_core`.
//!
//! Results are diagnostic evidence for optimization. They configure nothing,
//! and they are not J-04 acceptance, parity or superiority claims. The harness
//! grades every system with its own exact oracle, so no system grades itself.
pub mod definitions;
pub mod settings;

pub(crate) use rrflow_evaluation_core::{canonical_json, require, sha256_hex, Result};
