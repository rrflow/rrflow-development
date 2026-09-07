//! Stable outcomes returned by canonical storage mutations.

/// Claim sequences assigned by one atomic append.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppendOutcome {
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub count: usize,
}

/// Result of a content-bound, idempotent claim append.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdempotentAppendOutcome {
    pub operation_sha256: String,
    pub append: AppendOutcome,
    pub idempotent_replay: bool,
}
