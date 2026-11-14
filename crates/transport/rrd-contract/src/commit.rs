//! Commit, preview and forward rollback.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CommitTransaction {
    pub operation_sha256: String,
    pub mutations: Vec<TransactionMutation>,
}

impl CommitTransaction {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.operation_sha256, "operation_sha256")?;
        if self.mutations.is_empty() || self.mutations.len() > MAX_TRANSACTION_CLAIMS {
            return invalid(format!(
                "transaction mutation count must be in 1..={MAX_TRANSACTION_CLAIMS}"
            ));
        }
        for mutation in &self.mutations {
            mutation.validate()?;
        }
        Ok(())
    }

    pub fn computed_operation_sha256(&self) -> String {
        transaction_operation_sha256(&self.mutations)
    }
}

/// SHA-256 over the stable typed JSON representation of the ordered mutation
/// list. Clients must use this function's cross-language equivalent rather
/// than hashing arbitrary input-object key order.
pub fn transaction_operation_sha256(mutations: &[TransactionMutation]) -> String {
    let bytes = serde_json::to_vec(mutations).expect("public transaction mutations serialize");
    sha256_bytes(&bytes)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreviewTransaction {
    pub mutations: Vec<TransactionMutation>,
    /// Valid-time coordinate for the prospective read-your-writes snapshot.
    /// Omission selects the server's single request time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_at: Option<u64>,
    #[serde(default = "default_transaction_preview_changes")]
    pub max_storage_keys: u32,
}

impl PreviewTransaction {
    pub fn validate(&self) -> Result<()> {
        if self.mutations.is_empty() || self.mutations.len() > MAX_TRANSACTION_CLAIMS {
            return invalid(format!(
                "transaction mutation count must be in 1..={MAX_TRANSACTION_CLAIMS}"
            ));
        }
        for mutation in &self.mutations {
            mutation.validate()?;
        }
        if self.valid_at == Some(0)
            || self.max_storage_keys == 0
            || self.max_storage_keys > MAX_DATA_SNAPSHOT_STORAGE_KEYS
        {
            return invalid(format!(
                "transaction preview requires a non-zero valid_at when present and max_storage_keys in 1..={MAX_DATA_SNAPSHOT_STORAGE_KEYS}"
            ));
        }
        Ok(())
    }
}

pub(crate) fn default_transaction_preview_changes() -> u32 {
    10_000
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransactionPreview {
    pub transaction_id: CorrelationId,
    pub read_cursor: u64,
    pub operation_sha256: String,
    pub mutations: Vec<TransactionMutation>,
    pub prospective: DataSnapshot,
    pub idempotent_replay: bool,
}

impl TransactionPreview {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.operation_sha256, "operation_sha256")?;
        PreviewTransaction {
            mutations: self.mutations.clone(),
            valid_at: Some(self.prospective.valid_at),
            max_storage_keys: MAX_DATA_SNAPSHOT_STORAGE_KEYS,
        }
        .validate()?;
        if self.prospective.known_at_cursor
            != self
                .read_cursor
                .checked_add(self.mutations.len() as u64)
                .ok_or_else(|| ContractError("transaction preview cursor overflowed".into()))?
        {
            return invalid(
                "transaction prospective cursor must cover the read cursor and every mutation",
            );
        }
        validate_sha256(
            &self.prospective.read_manifest_sha256,
            "prospective read manifest",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CommitReceipt {
    pub transaction_id: CorrelationId,
    pub operation_sha256: String,
    pub first_claim_sequence: u64,
    pub last_claim_sequence: u64,
    pub mutation_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_commit_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_runtime_cursor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_runtime_cursor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_mutation_count: Option<u64>,
    pub idempotent_replay: bool,
}

/// Selects one retained historical structural state and the later instant at
/// which compensating versions become effective. The target cursor is
/// transaction time; `target_valid_at` is modeled valid time. They remain
/// independent so rollback never collapses RRFlow's two timelines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForwardRollbackRequest {
    pub target_valid_at: u64,
    pub target_known_at_cursor: u64,
    pub effective_at: u64,
    pub reason: String,
}

impl ForwardRollbackRequest {
    pub fn validate(&self) -> Result<()> {
        if self.target_valid_at == 0 || self.effective_at <= self.target_valid_at {
            return invalid(
                "rollback effective time must be greater than its positive target valid time",
            );
        }
        if self.reason.trim().is_empty() || self.reason.len() > MAX_MESSAGE_BYTES {
            return invalid(format!(
                "rollback reason length must be in 1..={MAX_MESSAGE_BYTES} bytes"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForwardRollbackCounts {
    pub restored_records: u64,
    pub retired_records: u64,
    pub restored_relations: u64,
    pub retired_relations: u64,
}

impl ForwardRollbackCounts {
    pub fn checked_mutation_count(self) -> Option<u64> {
        self.restored_records
            .checked_add(self.retired_records)?
            .checked_add(self.restored_relations)?
            .checked_add(self.retired_relations)
    }
}

/// Deterministic compensation plan bound to the transaction's original read
/// cursor. The final mutation is an evidence claim; `operation_sha256` covers
/// the complete ordered list and feeds the existing transaction idempotency
/// authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForwardRollbackPlan {
    pub request: ForwardRollbackRequest,
    pub read_cursor: u64,
    pub target_state_sha256: String,
    pub counts: ForwardRollbackCounts,
    pub operation_sha256: String,
    pub mutations: Vec<TransactionMutation>,
}

impl ForwardRollbackPlan {
    pub fn validate(&self) -> Result<()> {
        self.request.validate()?;
        if self.request.target_known_at_cursor >= self.read_cursor {
            return invalid(
                "rollback target cursor must be older than the transaction read cursor",
            );
        }
        validate_sha256(&self.target_state_sha256, "target_state_sha256")?;
        validate_sha256(&self.operation_sha256, "operation_sha256")?;
        let expected_mutations = self
            .counts
            .checked_mutation_count()
            .and_then(|count| count.checked_add(1));
        if u64::try_from(self.mutations.len()).ok() != expected_mutations {
            return invalid("rollback mutation count does not match its structural counts");
        }
        for mutation in &self.mutations {
            mutation.validate()?;
        }
        if transaction_operation_sha256(&self.mutations) != self.operation_sha256 {
            return invalid("rollback operation digest does not match its mutations");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForwardRollbackReceipt {
    pub request: ForwardRollbackRequest,
    pub read_cursor: u64,
    pub target_state_sha256: String,
    pub counts: ForwardRollbackCounts,
    pub commit: CommitReceipt,
}

impl ForwardRollbackReceipt {
    pub fn validate(&self) -> Result<()> {
        self.request.validate()?;
        if self.request.target_known_at_cursor >= self.read_cursor {
            return invalid("rollback receipt target must be older than its read cursor");
        }
        validate_sha256(&self.target_state_sha256, "target_state_sha256")?;
        self.commit.validate()?;
        let expected_mutations = self
            .counts
            .checked_mutation_count()
            .and_then(|count| count.checked_add(1));
        if Some(self.commit.mutation_count) != expected_mutations {
            return invalid("rollback receipt does not match its committed mutation counts");
        }
        Ok(())
    }
}

impl CommitReceipt {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.operation_sha256, "operation_sha256")?;
        match (
            &self.runtime_commit_sha256,
            self.first_runtime_cursor,
            self.last_runtime_cursor,
            self.claim_mutation_count,
        ) {
            (None, None, None, None) => {
                if self.first_claim_sequence == 0
                    || self.last_claim_sequence < self.first_claim_sequence
                    || self.mutation_count
                        != self.last_claim_sequence - self.first_claim_sequence + 1
                {
                    return invalid("commit receipt sequence interval is inconsistent");
                }
            }
            (Some(commit), Some(first), Some(last), Some(claims)) => {
                validate_sha256(commit, "runtime_commit_sha256")?;
                if first == 0 || last < first || self.mutation_count != last - first + 1 {
                    return invalid("runtime commit receipt cursor interval is inconsistent");
                }
                if (claims == 0
                    && (self.first_claim_sequence != 0 || self.last_claim_sequence != 0))
                    || (claims > 0
                        && (self.first_claim_sequence == 0
                            || self.last_claim_sequence - self.first_claim_sequence + 1 != claims))
                {
                    return invalid("runtime commit receipt claim interval is inconsistent");
                }
            }
            _ => return invalid("runtime commit receipt fields must be supplied together"),
        }
        Ok(())
    }
}
