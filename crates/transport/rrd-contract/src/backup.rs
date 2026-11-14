//! Instance backup and restore.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackupCoverageSnapshot {
    Included,
    ReferencedOnly,
    RebuildRequired,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LogicalArchiveSnapshot {
    pub format_version: u16,
    pub contract_version: u16,
    pub archive_sha256: String,
    pub action_count: u64,
    pub standalone_claims: u64,
    pub runtime_commits: u64,
    pub runtime_mutations: u64,
    pub payload_bytes: u64,
    pub claim_sequence: u64,
    pub runtime_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_audit_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InstanceBackupSnapshot {
    pub backup_sha256: String,
    pub label: String,
    pub created_at_unix_ms: u64,
    pub archive: LogicalArchiveSnapshot,
    pub claims: BackupCoverageSnapshot,
    pub typed_runtime: BackupCoverageSnapshot,
    pub catalogues: BackupCoverageSnapshot,
    /// The engram's control state outside the catalogues: installed record
    /// and checkpoint, active configuration, security policy and audit chain,
    /// attunement plan and job state, and engram documents. `included` means
    /// the backup carries them as a replayable control-journal archive; the
    /// token key and operator credential are never carried.
    pub installed_control: BackupCoverageSnapshot,
    pub object_payloads: BackupCoverageSnapshot,
    pub projections: BackupCoverageSnapshot,
    pub invocation_telemetry: BackupCoverageSnapshot,
    pub snapshot_leases: BackupCoverageSnapshot,
    pub application_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InstanceBackupCatalogueSnapshot {
    pub format_version: u16,
    pub revision: u64,
    pub catalogue_sha256: String,
    pub archives_verified: bool,
    pub backups: Vec<InstanceBackupSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateInstanceBackup {
    pub label: String,
    pub created_at_unix_ms: u64,
}

impl CreateInstanceBackup {
    pub fn validate(&self) -> Result<()> {
        if self.label.is_empty()
            || self.label.len() > 96
            || self.label.trim() != self.label
            || !self
                .label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return invalid(
                "backup label must be 1-96 ASCII alphanumeric, '.', '-', or '_' characters",
            );
        }
        if self.created_at_unix_ms == 0 {
            return invalid("backup created_at_unix_ms must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateInstanceBackupResult {
    pub backup: InstanceBackupSnapshot,
    pub catalogue_revision: u64,
    pub catalogue_sha256: String,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListInstanceBackups {
    #[serde(default)]
    pub verify_archives: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RestoreInstanceBackup {
    pub backup_sha256: String,
    pub restore_id: CanonicalId,
    pub restored_at_unix_ms: u64,
}

impl RestoreInstanceBackup {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.backup_sha256, "backup_sha256")?;
        if self.restored_at_unix_ms == 0 {
            return invalid("restore restored_at_unix_ms must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RestoreInstanceBackupResult {
    pub backup_sha256: String,
    pub restore_id: CanonicalId,
    pub inventory: LogicalArchiveSnapshot,
    pub reopened: bool,
    pub idempotent_replay: bool,
}
