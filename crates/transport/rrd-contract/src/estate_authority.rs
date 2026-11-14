//! Estate desired and observed state, authority, leases, backups and recovery.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadEstate {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateDesiredPhase {
    Running,
    Stopped,
    Absent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateObservedPhase {
    Unknown,
    Provisioning,
    Starting,
    Running,
    Stopping,
    Stopped,
    Deleting,
    Absent,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateActivityClass {
    Unknown,
    Active,
    Idle,
    Stale,
    Neglected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateOperationKind {
    Provision,
    Start,
    Stop,
    Restart,
    Upgrade,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateOperationState {
    Pending,
    Leased,
    Prepared,
    Applied,
    Succeeded,
    Failed,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateReceiptBoundary {
    Prepared,
    Applied,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateActivityPolicySnapshot {
    pub idle_after_ms: u64,
    pub stale_after_ms: u64,
    pub neglected_after_ms: u64,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EstateAuthorityResourceKind {
    Organisation,
    Account,
    Entitlement,
    Project,
    Environment,
    Instance,
    Node,
    Shard,
    Job,
    Assignment,
    Health,
    SecretReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateAuthorityStatus {
    Pending,
    Ready,
    Degraded,
    Failed,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateAuthorityReceiptBoundary {
    DesiredAccepted,
    Assigned,
    Applied,
    Observed,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateAuthorityDesiredSnapshot {
    pub generation: u64,
    pub spec_sha256: String,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateAuthorityObservedSnapshot {
    pub generation: u64,
    pub status: EstateAuthorityStatus,
    pub observed_at_unix_ms: u64,
    pub evidence_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateAuthorityResourceSnapshot {
    pub id: CanonicalId,
    pub kind: EstateAuthorityResourceKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_ids: Vec<CanonicalId>,
    pub name: String,
    pub spec_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desired: Option<EstateAuthorityDesiredSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<EstateAuthorityObservedSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secret_reference_ids: Vec<CanonicalId>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateAuthorityReceiptSnapshot {
    pub id: CanonicalId,
    pub resource_id: CanonicalId,
    pub operation_id: CanonicalId,
    pub lease_epoch: u64,
    pub boundary: EstateAuthorityReceiptBoundary,
    pub at_unix_ms: u64,
    pub evidence_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateAuthoritySnapshot {
    pub catalogue_sha256: String,
    pub resources: Vec<EstateAuthorityResourceSnapshot>,
    pub receipts: Vec<EstateAuthorityReceiptSnapshot>,
    pub idempotency_binding_count: u32,
}

impl EstateAuthoritySnapshot {
    pub fn empty() -> Self {
        Self {
            catalogue_sha256: sha256_bytes(
                br#"{"format":1,"resources":{},"receipts":{},"idempotency":{}}"#,
            ),
            resources: Vec::new(),
            receipts: Vec::new(),
            idempotency_binding_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateDesiredSnapshot {
    pub generation: u64,
    pub phase: EstateDesiredPhase,
    pub deployment_ref: CanonicalId,
    pub version: String,
    pub configuration_sha256: String,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateObservedSnapshot {
    pub generation: u64,
    pub phase: EstateObservedPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u32>,
    pub observed_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateActivitySnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_meaningful_runtime_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at_unix_ms: Option<u64>,
    pub class: EstateActivityClass,
    pub evaluated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateInstanceSnapshot {
    pub id: CanonicalId,
    pub desired: EstateDesiredSnapshot,
    pub observed: EstateObservedSnapshot,
    pub activity: EstateActivitySnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateLeaseSnapshot {
    pub owner: CanonicalId,
    pub epoch: u64,
    pub acquired_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateReceiptSnapshot {
    pub boundary: EstateReceiptBoundary,
    pub lease_epoch: u64,
    pub at_unix_ms: u64,
    pub evidence_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateOperationSnapshot {
    pub id: CanonicalId,
    pub instance_id: CanonicalId,
    pub kind: EstateOperationKind,
    pub desired_generation: u64,
    pub request_sha256: String,
    pub state: EstateOperationState,
    pub attempts: u32,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease: Option<EstateLeaseSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<EstateReceiptSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateSnapshot {
    pub format_version: u16,
    pub id: CanonicalId,
    pub revision: u64,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub activity_policy: EstateActivityPolicySnapshot,
    pub authority: EstateAuthoritySnapshot,
    pub instances: Vec<EstateInstanceSnapshot>,
    pub operations: Vec<EstateOperationSnapshot>,
    pub idempotency_binding_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateMutationResult {
    pub estate: EstateSnapshot,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateBackupJobState {
    Pending,
    Leased,
    Prepared,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateBackupReceiptBoundary {
    Prepared,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateBackupReceiptSnapshot {
    pub boundary: EstateBackupReceiptBoundary,
    pub lease_epoch: u64,
    pub at_unix_ms: u64,
    pub evidence_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateBackupRecoveryPolicySnapshot {
    pub revision: u64,
    pub max_rpo_ms: u64,
    pub max_rto_ms: u64,
    pub minimum_recovery_points: u16,
    pub retention_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateBackupJobSnapshot {
    pub id: CanonicalId,
    pub instance_id: CanonicalId,
    pub source_generation: u64,
    pub label: String,
    pub request_sha256: String,
    pub state: EstateBackupJobState,
    pub attempts: u32,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub recovery_policy: EstateBackupRecoveryPolicySnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease: Option<EstateLeaseSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<EstateBackupReceiptSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalogue_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateBackupJobsSnapshot {
    pub estate_id: CanonicalId,
    pub estate_revision: u64,
    pub jobs: Vec<EstateBackupJobSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateBackupMutationResult {
    pub estate: EstateSnapshot,
    pub job: EstateBackupJobSnapshot,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRecoveryPolicySnapshot {
    pub instance_id: CanonicalId,
    pub revision: u64,
    pub max_rpo_ms: u64,
    pub max_rto_ms: u64,
    pub minimum_recovery_points: u16,
    pub retention_ms: u64,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRecoveryPointSnapshot {
    pub backup_sha256: String,
    pub instance_id: CanonicalId,
    pub backup_job_id: CanonicalId,
    pub source_generation: u64,
    pub archive_sha256: String,
    pub catalogue_sha256: String,
    pub source_cut_at_unix_ms: u64,
    pub completed_at_unix_ms: u64,
    pub policy_revision: u64,
    pub max_rpo_ms: u64,
    pub max_rto_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pruned_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EstateRetentionPinKindSnapshot {
    Policy,
    ExplicitHold,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRetentionPinSnapshot {
    pub id: CanonicalId,
    pub backup_sha256: String,
    pub kind: EstateRetentionPinKindSnapshot,
    pub created_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRestoreEvidenceSnapshot {
    pub restore_id: CanonicalId,
    pub instance_id: CanonicalId,
    pub backup_sha256: String,
    pub started_at_unix_ms: u64,
    pub completed_at_unix_ms: u64,
    pub duration_ms: u64,
    pub recovery_point_age_ms: u64,
    pub restored_claim_sequence: u64,
    pub restored_runtime_cursor: u64,
    pub closure_sha256: String,
    pub policy_revision: u64,
    pub rpo_within_objective: bool,
    pub rto_within_objective: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRecoveryPruneIntentSnapshot {
    pub id: CanonicalId,
    pub instance_id: CanonicalId,
    pub based_on_estate_revision: u64,
    pub evaluated_at_unix_ms: u64,
    pub expected_catalogue_sha256: String,
    pub retained_backup_ids: Vec<String>,
    pub prune_candidate_backup_ids: Vec<String>,
    pub created_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRecoverySnapshot {
    pub estate_id: CanonicalId,
    pub estate_revision: u64,
    pub policies: Vec<EstateRecoveryPolicySnapshot>,
    pub recovery_points: Vec<EstateRecoveryPointSnapshot>,
    pub retention_pins: Vec<EstateRetentionPinSnapshot>,
    pub restore_evidence: Vec<EstateRestoreEvidenceSnapshot>,
    pub prune_intents: Vec<EstateRecoveryPruneIntentSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRetentionDecisionSnapshot {
    pub estate_revision: u64,
    pub instance_id: CanonicalId,
    pub evaluated_at_unix_ms: u64,
    pub retained_backup_ids: Vec<String>,
    pub prune_candidate_backup_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRecoveryMutationResult {
    pub recovery: EstateRecoverySnapshot,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRecoveryPruneResult {
    pub recovery: EstateRecoverySnapshot,
    pub decision: EstateRetentionDecisionSnapshot,
    pub catalogue_revision: u64,
    pub catalogue_sha256: String,
    pub pruned_backup_ids: Vec<String>,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstateRecoveryRestoreResult {
    pub recovery: EstateRecoverySnapshot,
    pub restore_id: CanonicalId,
    pub backup_sha256: String,
    pub inventory: LogicalArchiveSnapshot,
    pub reopened: bool,
    pub idempotent_replay: bool,
}
