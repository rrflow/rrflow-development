//! Engram-facing names for the established, single durable memory contract.
//!
//! The aliases keep one persisted representation and one validator while the
//! implementation terminology converges on engram.

/// Engram-facing name for the established provider-neutral memory mutation.
pub type PersistMemoryengram = crate::PersistMemoryEstate;

/// Engram-facing name for the established deterministic mutation plan.
pub type MemoryengramPlan = crate::MemoryEstatePlan;
