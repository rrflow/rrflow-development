//! A comparison lab: one typed workload run against RRFlow and other systems.
//! Its shared vocabulary and report format are `rrflow_evaluation_core`.
//!
//! Results are diagnostic evidence for optimization. They configure nothing,
//! and they are not J-04 acceptance, parity or superiority claims. The harness
//! grades every system with its own exact oracle, so no system grades itself.
pub mod catalog;
pub mod corpus;
pub mod definitions;
pub mod driver;
pub mod import;
pub mod knowledge;
pub mod manifest;
pub mod provider;
pub mod references;
pub mod settings;
pub mod sources;
pub mod study;
pub mod tasks;

pub(crate) use rrflow_evaluation_core::{canonical_json, require, sha256_hex, Result};
