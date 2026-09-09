//! Canonical encoded access paths shared by semantic repositories.
//!
//! This boundary owns physical key selection, stamped reads, and preparation
//! of canonical semantic write plans. It does not authenticate,
//! authorize, select a storage profile, or publish a transaction.

pub(crate) mod runtime_state;
mod semantic_commit;

pub(crate) use semantic_commit::prepare_semantic_commit;
pub use semantic_commit::SemanticCommitPlan;
