//! Operator invocation audit records.

use rrd_core::Millis;
use serde::{Deserialize, Serialize};

/// What caused an invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    Manual,
    Event,
    Interval,
    Threshold,
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Trigger::Manual => "manual",
            Trigger::Event => "event",
            Trigger::Interval => "interval",
            Trigger::Threshold => "threshold",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Ok,
    Error,
}

/// One recorded invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invocation {
    /// Monotonic ordinal, allocated inside the recording transaction.
    pub ordinal: u64,
    pub at: Millis,
    pub trigger: Trigger,
    pub command: String,
    pub arguments: Vec<String>,
    pub outcome: Outcome,
    pub duration_ms: u64,
    /// Failure reason, or a short result summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Invocation {
    /// Rendered as one line, for an operator reading the log directly.
    pub fn render(&self) -> String {
        format!(
            "{:>6}  {}  {:<9} {:<10} {:>7}ms  {}{}",
            self.ordinal,
            self.at,
            self.trigger.to_string(),
            self.command,
            self.duration_ms,
            match self.outcome {
                Outcome::Ok => "ok",
                Outcome::Error => "error",
            },
            self.detail
                .as_ref()
                .map(|d| format!("  {d}"))
                .unwrap_or_default(),
        )
    }
}

/// Fields supplied when recording an invocation.
///
/// Grouped rather than passed positionally: the ordinal is allocated by the
/// store, so a caller supplies everything else as one value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationInput<'a> {
    pub at: Millis,
    pub trigger: Trigger,
    pub command: &'a str,
    pub arguments: &'a [String],
    pub outcome: Outcome,
    pub duration_ms: u64,
    pub detail: Option<String>,
}
