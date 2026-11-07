//! Typed read-only status and generation readback for derived cold history.

use crate::{
    invalid, QueryBudget, Result, RuntimeChangeSnapshot, MAX_ID_BYTES, MAX_QUERY_MEMORY_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadColdHistory {
    pub scope: String,
    /// Omit to read status only. Generations begin at one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    pub max_artifact_bytes: u64,
    #[serde(default)]
    pub budget: QueryBudget,
}

impl ReadColdHistory {
    pub fn validate(&self) -> Result<()> {
        if self.scope.is_empty() || self.scope.len() > MAX_ID_BYTES || self.scope.contains('\0') {
            return invalid("cold history scope is empty, too long, or contains NUL");
        }
        if self.generation == Some(0) {
            return invalid("cold history generation must be greater than zero");
        }
        self.budget.validate()?;
        if self.max_artifact_bytes == 0
            || self.max_artifact_bytes > MAX_QUERY_MEMORY_BYTES
            || self.max_artifact_bytes > self.budget.max_memory_bytes
        {
            return invalid("cold history artifact limit must fit the query memory budget");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColdHistoryGenerationSnapshot {
    pub generation: u64,
    pub requested_after_cursor: u64,
    pub through_cursor: u64,
    pub source_head_cursor: u64,
    pub source_manifest_sha256: String,
    pub policy_sha256: String,
    pub artifact_sha256: String,
    pub artifact_bytes: u64,
    pub rows: u64,
    pub row_groups: u64,
    pub observed_arrow_bytes: u64,
    pub declared_uncompressed_bytes: u64,
}

/// A payload-free classification of unresolved background work failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColdHistoryFailureCode {
    DeadlineExceeded,
    ResourceLimit,
    SourceConflict,
    StorageOrOther,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColdHistoryFailure {
    pub code: ColdHistoryFailureCode,
    pub since_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColdHistoryReadResult {
    pub scope: String,
    pub enabled: bool,
    pub current_generation: u64,
    pub through_cursor: u64,
    pub pending: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<ColdHistoryFailure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<ColdHistoryGenerationSnapshot>,
    pub changes: Vec<RuntimeChangeSnapshot>,
}

impl ColdHistoryReadResult {
    /// Bind a response to the requested scope and generation before a client
    /// treats it as the outcome of its request. This does not prove coverage.
    pub fn validate_request_identity(&self, request: &ReadColdHistory) -> Result<()> {
        request.validate()?;
        if self.scope != request.scope || (self.current_generation == 0 && self.through_cursor != 0)
        {
            return invalid("cold history status does not match request or head");
        }
        match (request.generation, &self.generation) {
            (None, None) if self.changes.is_empty() => Ok(()),
            (Some(expected), Some(generation))
                if generation.generation == expected
                    && expected <= self.current_generation
                    && generation.requested_after_cursor < generation.through_cursor
                    && generation.through_cursor <= self.through_cursor
                    && generation.through_cursor <= generation.source_head_cursor
                    && generation.artifact_bytes > 0
                    && generation.artifact_bytes <= request.max_artifact_bytes
                    && generation.rows == self.changes.len() as u64
                    && generation.rows <= request.budget.max_rows
                    && generation.row_groups <= generation.rows =>
            {
                crate::validate_sha256(
                    &generation.source_manifest_sha256,
                    "cold history source manifest",
                )?;
                crate::validate_sha256(&generation.policy_sha256, "cold history policy")?;
                crate::validate_sha256(&generation.artifact_sha256, "cold history artifact")?;
                let mut previous = generation.requested_after_cursor;
                for change in &self.changes {
                    if change.scope != self.scope
                        || change.cursor <= previous
                        || change.cursor > generation.through_cursor
                    {
                        return invalid("cold history change identity or order is invalid");
                    }
                    previous = change.cursor;
                }
                Ok(())
            }
            _ => invalid("cold history generation does not match request"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColdHistoryCompression, ColdHistoryPolicy, StorageLimits};

    #[test]
    fn read_contract_is_strict_and_generation_and_byte_limits_are_enforced() {
        let request = ReadColdHistory {
            scope: "instance:example".into(),
            generation: Some(1),
            max_artifact_bytes: 1024,
            budget: QueryBudget::default(),
        };
        request.validate().unwrap();
        let mut changed = request.clone();
        changed.generation = Some(0);
        assert!(changed.validate().is_err());
        changed = request.clone();
        changed.max_artifact_bytes = changed.budget.max_memory_bytes + 1;
        assert!(changed.validate().is_err());
        changed = request.clone();
        changed.scope = "instance:\0bad".into();
        assert!(changed.validate().is_err());
        let mut encoded = serde_json::to_value(request).unwrap();
        encoded["unexpected"] = true.into();
        assert!(serde_json::from_value::<ReadColdHistory>(encoded).is_err());
        let mut status = ColdHistoryReadResult {
            scope: "instance:example".into(),
            enabled: true,
            current_generation: 0,
            through_cursor: 0,
            pending: false,
            failure: None,
            generation: None,
            changes: Vec::new(),
        };
        let mut status_request = ReadColdHistory {
            scope: status.scope.clone(),
            generation: None,
            max_artifact_bytes: 1024,
            budget: QueryBudget::default(),
        };
        status.validate_request_identity(&status_request).unwrap();
        status.scope = "instance:other".into();
        assert!(status.validate_request_identity(&status_request).is_err());
        status.scope = status_request.scope.clone();
        status_request.generation = Some(1);
        assert!(status.validate_request_identity(&status_request).is_err());
    }

    #[test]
    fn policy_is_optional_and_rejects_invalid_rows_compression_and_memory() {
        let mut storage = StorageLimits::default();
        assert!(serde_json::to_value(&storage)
            .unwrap()
            .get("cold_history")
            .is_none());
        let policy = ColdHistoryPolicy {
            interval_ms: 1000,
            max_changes: 8,
            max_input_bytes: 1024,
            max_output_bytes: 2048,
            max_memory_bytes: 4096,
            max_elapsed_ms: 1000,
            row_group_rows: 4,
            compression: ColdHistoryCompression::Zstd { level: 3 },
            dictionary: true,
            statistics: true,
        };
        storage.cold_history = Some(policy.clone());
        storage.validate().unwrap();
        let mut invalid = policy.clone();
        invalid.interval_ms = 0;
        assert!(invalid.validate().is_err());
        invalid = policy.clone();
        invalid.row_group_rows = 9;
        assert!(invalid.validate().is_err());
        invalid = policy.clone();
        invalid.compression = ColdHistoryCompression::Zstd { level: 0 };
        assert!(invalid.validate().is_err());
        invalid = policy;
        invalid.max_input_bytes = invalid.max_memory_bytes + 1;
        assert!(invalid.validate().is_err());
    }
}
