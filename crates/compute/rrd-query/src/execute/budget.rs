//! Execution budgets and the rows, batches and executions a query returns.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBudget {
    pub max_storage_keys: u64,
    pub max_input_rows: usize,
    pub max_rows: usize,
    pub max_output_bytes: usize,
    pub max_batch_rows: usize,
    pub max_memory_bytes: usize,
    pub max_spill_bytes: usize,
    pub max_elapsed_ms: u64,
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self {
            max_storage_keys: 100_000,
            max_input_rows: 100_000,
            max_rows: 10_000,
            max_output_bytes: 8 * 1024 * 1024,
            max_batch_rows: 256,
            max_memory_bytes: 64 * 1024 * 1024,
            max_spill_bytes: 256 * 1024 * 1024,
            max_elapsed_ms: 30_000,
        }
    }
}

impl ExecutionBudget {
    pub(super) fn validate(&self) -> Result<()> {
        if self.max_storage_keys == 0
            || self.max_input_rows == 0
            || self.max_rows == 0
            || self.max_output_bytes == 0
            || self.max_batch_rows == 0
            || self.max_memory_bytes == 0
            || self.max_spill_bytes == 0
            || self.max_elapsed_ms == 0
        {
            return Err(Error::Budget(
                "all execution budgets must be greater than zero".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryRow {
    pub identity: String,
    pub values: BTreeMap<String, RuntimeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryBatch {
    pub ordinal: usize,
    pub rows: Vec<QueryRow>,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryExecution {
    pub plan_digest: String,
    pub read_manifest: String,
    pub valid_at: u64,
    pub known_at_cursor: u64,
    /// Number of authenticated semantic versions selected before temporal
    /// reduction. Index-backed paths report zero because their source rows are
    /// carried by the separately verified immutable artifact.
    pub selected_versions: usize,
    pub read_evidence: RuntimeReadEvidence,
    pub returned_rows: usize,
    pub output_bytes: usize,
    pub truncated: bool,
    pub batches: Vec<QueryBatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<FusionAnalysis>,
}
