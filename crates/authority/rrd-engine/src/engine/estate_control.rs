use super::engram_layout::{
    instance_operation_root, BACKUPS_DIRECTORY_NAME, RESTORES_DIRECTORY_NAME,
};
use super::installation::read_locator;
use super::restore_publication::require_distribution_binding;
use super::security::AuditEvent;
use super::*;
use rrd_engram::{engramBackupDriver, engramDriver};
use std::fs;
#[cfg(debug_assertions)]
use std::io::Write as _;

const engram_RECOVERY_OPERATION_FORMAT: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct engramRecoveryPruneOperationState {
    format_version: u16,
    operation_sha256: String,
    instance_id: CanonicalId,
    operation_id: CanonicalId,
    evaluated_at: u64,
    expected_engram_revision: u64,
    expected_catalogue_sha256: String,
    retained_backup_ids: Vec<String>,
    prune_candidate_backup_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result: Option<rrd_contract::engramRecoveryPruneResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct engramRecoveryRestoreOperationState {
    format_version: u16,
    operation_sha256: String,
    instance_id: CanonicalId,
    operation_id: CanonicalId,
    backup_sha256: String,
    restore_id: CanonicalId,
    started_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result: Option<rrd_contract::engramRecoveryRestoreResult>,
}

/// A bounded engram mutation accepted by the RRD composition root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum engramAdminAction {
    Create,
    SetDesired {
        instance: CanonicalId,
        idempotency_key: String,
        phase: rrd_contract::engramDesiredPhase,
        deployment: CanonicalId,
        version: String,
        configuration_sha256: String,
    },
    ScheduleBackup {
        instance: CanonicalId,
        idempotency_key: String,
        label: String,
    },
    SetRecoveryPolicy {
        instance: CanonicalId,
        idempotency_key: String,
        max_rpo_ms: u64,
        max_rto_ms: u64,
        minimum_recovery_points: u16,
        retention_ms: u64,
    },
    PinRecoveryPoint {
        idempotency_key: String,
        backup_sha256: String,
        expires_at_unix_ms: Option<u64>,
    },
    ReleaseRecoveryPin {
        idempotency_key: String,
        pin_id: CanonicalId,
    },
}

/// Public result from an engine-owned engram administration operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum engramAdminResult {
    Mutation(Box<rrd_contract::engramMutationResult>),
    Backup(Box<rrd_contract::engramBackupMutationResult>),
    Recovery(Box<rrd_contract::engramRecoveryMutationResult>),
}

pub type engramReconcileOutcome = rrd_engram::ReconcileOutcome;
pub type engramBackupReconcileOutcome = rrd_engram::BackupReconcileOutcome;

fn engram_audit_resource(instance: &CanonicalId, engram: &CanonicalId) -> ResourcePath {
    ResourcePath {
        segments: vec![
            ResourceId::new(ResourceKind::Instance, instance.as_str())
                .expect("engine instance identity is canonical"),
            ResourceId::new(ResourceKind::engram, engram.as_str())
                .expect("engram identity is canonical"),
        ],
    }
}

#[allow(clippy::too_many_arguments)]
fn audit_engram_operation<T, F>(
    engine: &RrdEngine,
    at: u64,
    principal_id: CanonicalId,
    resource: ResourcePath,
    request_id: String,
    operation_id: String,
    request_sha256: String,
    operation: F,
) -> Result<T>
where
    T: Serialize,
    F: FnOnce() -> Result<T>,
{
    engine.append_audit(AuditEvent {
        at_unix_ms: at,
        attempt: 1,
        principal_id: Some(principal_id.clone()),
        action: SecurityAction::engramAdmin,
        resource: resource.clone(),
        request_id: request_id.clone(),
        operation_id: operation_id.clone(),
        phase: AuditPhase::Authorized,
        decision: AuditDecision::Allowed,
        status_code: 100,
        request_sha256: request_sha256.clone(),
        response_sha256: digest::sha256_hex(b"rrd-audit-completion-pending"),
    })?;
    let result = operation();
    let (decision, status_code, response_sha256) = match &result {
        Ok(response) => (
            AuditDecision::Allowed,
            200,
            digest::sha256_hex(&serde_json::to_vec(response).map_err(contract_json)?),
        ),
        Err(error) => {
            let (decision, status_code) = super::invocation::audit_failure(error);
            (
                decision,
                status_code,
                digest::sha256_hex(error.to_string().as_bytes()),
            )
        }
    };
    engine.append_audit(AuditEvent {
        at_unix_ms: at,
        attempt: 1,
        principal_id: Some(principal_id),
        action: SecurityAction::engramAdmin,
        resource,
        request_id,
        operation_id,
        phase: AuditPhase::Completed,
        decision,
        status_code,
        request_sha256,
        response_sha256,
    })?;
    result
}

impl RrdEngine {
    /// Authorizes and applies one engram mutation through a single local RRD
    /// authority. Policy denial happens before the database can be created.
    #[allow(clippy::too_many_arguments)]
    pub fn administer_engram_store(
        database: &Path,
        authority_instance: CanonicalId,
        policy_path: &Path,
        key_path: &Path,
        engram_id: CanonicalId,
        at: u64,
        request_id: String,
        operation_id: CanonicalId,
        action: engramAdminAction,
    ) -> Result<engramAdminResult> {
        let permission = match &action {
            engramAdminAction::Create => rrd_engram::LocalengramPermission::Create,
            engramAdminAction::SetDesired { .. } => rrd_engram::LocalengramPermission::SetDesired,
            engramAdminAction::ScheduleBackup { .. } => {
                rrd_engram::LocalengramPermission::ScheduleBackup
            }
            engramAdminAction::SetRecoveryPolicy { .. } => {
                rrd_engram::LocalengramPermission::ManageRecoveryPolicy
            }
            engramAdminAction::PinRecoveryPoint { .. }
            | engramAdminAction::ReleaseRecoveryPin { .. } => {
                rrd_engram::LocalengramPermission::ManageRecoveryHolds
            }
        };
        let policy = rrd_engram::LocalOperatorPolicy::load_json(policy_path)
            .map_err(ServiceError::Contract)?;
        let authorization = policy
            .authorize_key_file(key_path, engram_id, permission, at)
            .map_err(ServiceError::Contract)?;
        let audit_request_id = request_id.clone();
        let audit_operation_id = operation_id.to_string();
        let audit_request_sha256 =
            digest::sha256_hex(&serde_json::to_vec(&action).map_err(contract_json)?);
        let audit_resource = engram_audit_resource(&authority_instance, &authorization.engram_id);
        let audit_principal = authorization.operator_id.clone();
        let engine = Self::open_local_authority(database, authority_instance)?;
        let repository =
            rrd_engram::engramRepository::new(&engine.storage, authorization.engram_id);
        engine.append_audit(AuditEvent {
            at_unix_ms: at,
            attempt: 1,
            principal_id: Some(audit_principal.clone()),
            action: SecurityAction::engramAdmin,
            resource: audit_resource.clone(),
            request_id: audit_request_id.clone(),
            operation_id: audit_operation_id.clone(),
            phase: AuditPhase::Authorized,
            decision: AuditDecision::Allowed,
            status_code: 100,
            request_sha256: audit_request_sha256.clone(),
            response_sha256: digest::sha256_hex(b"rrd-audit-completion-pending"),
        })?;
        let context = rrd_engram::MutationContext {
            at,
            actor: authorization.operator_id.to_string(),
            request_id,
            operation_id,
        };
        let result = (|| match action {
            engramAdminAction::Create => {
                let outcome = repository.create_idempotent(&context)?;
                Ok(engramAdminResult::Mutation(Box::new(
                    rrd_contract::engramMutationResult {
                        engram: rrd_engram::public_snapshot(&outcome.document),
                        idempotent_replay: outcome.idempotent_replay,
                    },
                )))
            }
            engramAdminAction::SetDesired {
                instance,
                idempotency_key,
                phase,
                deployment,
                version,
                configuration_sha256,
            } => {
                let phase = match phase {
                    rrd_contract::engramDesiredPhase::Running => rrd_engram::DesiredPhase::Running,
                    rrd_contract::engramDesiredPhase::Stopped => rrd_engram::DesiredPhase::Stopped,
                    rrd_contract::engramDesiredPhase::Absent => rrd_engram::DesiredPhase::Absent,
                };
                let outcome = repository.set_desired(&rrd_engram::SetDesired {
                    context,
                    instance_id: instance,
                    idempotency_key,
                    target: rrd_engram::DesiredTarget {
                        phase,
                        deployment_ref: deployment,
                        version,
                        configuration_sha256,
                    },
                })?;
                Ok(engramAdminResult::Mutation(Box::new(
                    rrd_contract::engramMutationResult {
                        engram: rrd_engram::public_snapshot(&outcome.document),
                        idempotent_replay: outcome.idempotent_replay,
                    },
                )))
            }
            engramAdminAction::ScheduleBackup {
                instance,
                idempotency_key,
                label,
            } => {
                let outcome = repository.schedule_backup(&rrd_engram::ScheduleBackup {
                    context,
                    instance_id: instance,
                    idempotency_key,
                    label,
                })?;
                Ok(engramAdminResult::Backup(Box::new(
                    rrd_contract::engramBackupMutationResult {
                        engram: rrd_engram::public_snapshot(&outcome.document),
                        job: rrd_engram::public_backup_job(&outcome.job),
                        idempotent_replay: outcome.idempotent_replay,
                    },
                )))
            }
            engramAdminAction::SetRecoveryPolicy {
                instance,
                idempotency_key,
                max_rpo_ms,
                max_rto_ms,
                minimum_recovery_points,
                retention_ms,
            } => {
                let outcome = repository.set_recovery_policy(&rrd_engram::SetRecoveryPolicy {
                    context,
                    instance_id: instance,
                    idempotency_key,
                    max_rpo_ms,
                    max_rto_ms,
                    minimum_recovery_points,
                    retention_ms,
                })?;
                Ok(engramAdminResult::Recovery(Box::new(
                    rrd_contract::engramRecoveryMutationResult {
                        recovery: rrd_engram::public_recovery_snapshot(&outcome.document),
                        idempotent_replay: outcome.idempotent_replay,
                    },
                )))
            }
            engramAdminAction::PinRecoveryPoint {
                idempotency_key,
                backup_sha256,
                expires_at_unix_ms,
            } => {
                let outcome = repository.pin_recovery_point(&rrd_engram::PinRecoveryPoint {
                    context,
                    idempotency_key,
                    backup_id: backup_sha256,
                    expires_at: expires_at_unix_ms,
                })?;
                Ok(engramAdminResult::Recovery(Box::new(
                    rrd_contract::engramRecoveryMutationResult {
                        recovery: rrd_engram::public_recovery_snapshot(&outcome.document),
                        idempotent_replay: outcome.idempotent_replay,
                    },
                )))
            }
            engramAdminAction::ReleaseRecoveryPin {
                idempotency_key,
                pin_id,
            } => {
                let outcome = repository.release_recovery_pin(&rrd_engram::ReleaseRecoveryPin {
                    context,
                    idempotency_key,
                    pin_id,
                })?;
                Ok(engramAdminResult::Recovery(Box::new(
                    rrd_contract::engramRecoveryMutationResult {
                        recovery: rrd_engram::public_recovery_snapshot(&outcome.document),
                        idempotent_replay: outcome.idempotent_replay,
                    },
                )))
            }
        })();
        let (decision, status_code, response_sha256) = match &result {
            Ok(response) => (
                AuditDecision::Allowed,
                200,
                digest::sha256_hex(&serde_json::to_vec(response).map_err(contract_json)?),
            ),
            Err(error) => {
                let (decision, status_code) = super::invocation::audit_failure(error);
                (
                    decision,
                    status_code,
                    digest::sha256_hex(error.to_string().as_bytes()),
                )
            }
        };
        engine.append_audit(AuditEvent {
            at_unix_ms: at,
            attempt: 1,
            principal_id: Some(audit_principal),
            action: SecurityAction::engramAdmin,
            resource: audit_resource,
            request_id: audit_request_id,
            operation_id: audit_operation_id,
            phase: AuditPhase::Completed,
            decision,
            status_code,
            request_sha256: audit_request_sha256,
            response_sha256,
        })?;
        result
    }

    /// Advances one engram reconciliation boundary. The outward executable
    /// supplies coordinates only; catalog, driver, repository, and store
    /// construction remain inside `RrdEngine`.
    #[allow(clippy::too_many_arguments)]
    pub fn reconcile_engram_store(
        database: &Path,
        authority_instance: CanonicalId,
        state_root: &Path,
        catalog_path: &Path,
        engram_id: CanonicalId,
        worker: CanonicalId,
        lease_ms: u64,
        at: u64,
        hold_after_effect: Option<&Path>,
    ) -> Result<engramReconcileOutcome> {
        let audit_resource = engram_audit_resource(&authority_instance, &engram_id);
        let audit_request_id = format!("engram-reconcile-request-{engram_id}-{at}");
        let audit_operation_id = format!("engram-reconcile-{engram_id}-{worker}-{at}");
        let audit_request_sha256 = operation_digest(&(
            "engram-reconcile-v1",
            &engram_id,
            &worker,
            lease_ms,
            at,
            state_root.to_string_lossy(),
            catalog_path.to_string_lossy(),
        ))?;
        let engine = Self::open_local_authority(database, authority_instance)?;
        audit_engram_operation(
            &engine,
            at,
            worker.clone(),
            audit_resource,
            audit_request_id,
            audit_operation_id,
            audit_request_sha256,
            || {
                let catalog = rrd_engram::LocalDeploymentCatalog::load_json(catalog_path)
                    .map_err(ServiceError::Contract)?;
                let driver = rrd_engram::LocalProcessDriver::new(state_root, catalog)
                    .map_err(ServiceError::Contract)?;
                let mut reconciler = rrd_engram::Reconciler::new(
                    &engine.storage,
                    engram_id,
                    worker.clone(),
                    lease_ms,
                    EffectHoldDriver {
                        inner: driver,
                        marker: hold_after_effect.map(Path::to_path_buf),
                    },
                )?;
                reconciler.step(at).map_err(Into::into)
            },
        )
    }

    /// Advances one engram backup boundary with an engine-owned local backup
    /// driver. The driver opens a managed instance only as another installed
    /// `RrdEngine`, bound to the engram's distribution executable; no physical
    /// store handle escapes the composition root.
    #[allow(clippy::too_many_arguments)]
    pub fn reconcile_engram_backup_store(
        database: &Path,
        authority_instance: CanonicalId,
        state_root: &Path,
        distribution_executable: &Path,
        engram_id: CanonicalId,
        worker: CanonicalId,
        lease_ms: u64,
        at: u64,
        hold_after_effect: Option<&Path>,
    ) -> Result<engramBackupReconcileOutcome> {
        let audit_resource = engram_audit_resource(&authority_instance, &engram_id);
        let audit_request_id = format!("engram-backup-reconcile-request-{engram_id}-{at}");
        let audit_operation_id = format!("engram-backup-reconcile-{engram_id}-{worker}-{at}");
        let audit_request_sha256 = operation_digest(&(
            "engram-backup-reconcile-v1",
            &engram_id,
            &worker,
            lease_ms,
            at,
            state_root.to_string_lossy(),
            distribution_executable.to_string_lossy(),
        ))?;
        let engine = Self::open_local_authority(database, authority_instance)?;
        audit_engram_operation(
            &engine,
            at,
            worker.clone(),
            audit_resource,
            audit_request_id,
            audit_operation_id,
            audit_request_sha256,
            || {
                let driver = EngineLocalBackupDriver::new(state_root, distribution_executable)
                    .map_err(ServiceError::Contract)?;
                let mut reconciler = rrd_engram::BackupReconciler::new(
                    &engine.storage,
                    engram_id,
                    worker.clone(),
                    lease_ms,
                    BackupEffectHoldDriver {
                        inner: driver,
                        marker: hold_after_effect.map(Path::to_path_buf),
                    },
                )?;
                reconciler.step(at).map_err(Into::into)
            },
        )
    }

    /// Applies one engram-authorized retention decision through the existing
    /// engram and authenticated backup catalogue authorities. The durable
    /// engram intent fences policy, hold, restore, and backup completion
    /// mutations until physical pruning is verified and recorded.
    #[allow(clippy::too_many_arguments)]
    pub fn prune_engram_backups_store(
        database: &Path,
        authority_instance: CanonicalId,
        state_root: &Path,
        distribution_executable: &Path,
        policy_path: &Path,
        key_path: &Path,
        engram_id: CanonicalId,
        instance_id: CanonicalId,
        evaluated_at: u64,
        at: u64,
        request_id: String,
        operation_id: CanonicalId,
        idempotency_key: String,
    ) -> Result<rrd_contract::engramRecoveryPruneResult> {
        validate_local_recovery_request(at, &idempotency_key)?;
        if evaluated_at == 0 || evaluated_at > at {
            return Err(ServiceError::Contract(
                "recovery prune evaluation time is invalid".into(),
            ));
        }
        let authorization = authorize_local_recovery(
            policy_path,
            key_path,
            engram_id,
            rrd_engram::LocalengramPermission::PruneRecovery,
            at,
        )?;
        let state_root = canonical_local_state_root(state_root)?;
        let managed = resolve_managed_instance(&state_root, &instance_id, distribution_executable)?;
        let catalogue_root = managed.catalogue_root;
        require_direct_directory(&catalogue_root, "backup catalogue")?;
        let audit_resource = engram_audit_resource(&authority_instance, &authorization.engram_id);
        let audit_request_sha256 = operation_digest(&(
            "engram-recovery-prune-audit-v1",
            &authorization.engram_id,
            &instance_id,
            evaluated_at,
            &operation_id,
            &idempotency_key,
        ))?;
        let engine = Self::open_local_authority(database, authority_instance)?;
        audit_engram_operation(
            &engine,
            at,
            authorization.operator_id.clone(),
            audit_resource,
            request_id.clone(),
            operation_id.to_string(),
            audit_request_sha256,
            || {
                let repository = rrd_engram::engramRepository::new(
                    &engine.storage,
                    authorization.engram_id.clone(),
                );
                let operation_sha256 = operation_digest(&(
                    "engram-recovery-prune-v1",
                    &authorization.engram_id,
                    &instance_id,
                    evaluated_at,
                    &operation_id,
                ))?;
                let state_key = local_recovery_operation_key(
                    "prune",
                    &authorization.engram_id,
                    &instance_id,
                    &idempotency_key,
                );
                let existing = engine.storage.control().get(&state_key)?;
                let recovered_operation = existing.is_some();
                let (prepared_bytes, mut state) = if let Some(bytes) = existing {
                    let state: engramRecoveryPruneOperationState =
                        serde_json::from_slice(&bytes).map_err(contract_json)?;
                    if state.format_version != engram_RECOVERY_OPERATION_FORMAT
                        || state.operation_sha256 != operation_sha256
                        || state.instance_id != instance_id
                        || state.operation_id != operation_id
                    {
                        return Err(ServiceError::IdempotencyConflict);
                    }
                    if let Some(mut result) = state.result {
                        result.idempotent_replay = true;
                        return Ok(result);
                    }
                    (bytes, state)
                } else {
                    let document = repository
                        .load()?
                        .ok_or_else(|| ServiceError::engram("engram is not initialized".into()))?;
                    let decision =
                        rrd_engram::retention_decision(&document, &instance_id, evaluated_at)?;
                    if decision.prune_candidate_backup_ids.is_empty() {
                        return Err(ServiceError::engram(
                            "recovery retention decision has no prune candidates".into(),
                        ));
                    }
                    let catalogue = rrd_store::verify_backup_catalogue(&catalogue_root)?;
                    let mut engram_inventory = decision.retained_backup_ids.clone();
                    engram_inventory.extend(decision.prune_candidate_backup_ids.clone());
                    engram_inventory.sort();
                    let mut physical_inventory = catalogue
                        .backups
                        .iter()
                        .map(|entry| entry.backup_id.clone())
                        .collect::<Vec<_>>();
                    physical_inventory.sort();
                    if engram_inventory != physical_inventory {
                        return Err(ServiceError::Backup(
                    "engram recovery points do not exactly match the physical backup catalogue"
                        .into(),
                ));
                    }
                    let state = engramRecoveryPruneOperationState {
                        format_version: engram_RECOVERY_OPERATION_FORMAT,
                        operation_sha256: operation_sha256.clone(),
                        instance_id: instance_id.clone(),
                        operation_id: operation_id.clone(),
                        evaluated_at,
                        expected_engram_revision: decision.engram_revision,
                        expected_catalogue_sha256: catalogue.catalogue_sha256,
                        retained_backup_ids: decision.retained_backup_ids,
                        prune_candidate_backup_ids: decision.prune_candidate_backup_ids,
                        result: None,
                    };
                    let bytes = serde_json::to_vec(&state).map_err(contract_json)?;
                    commit_local_recovery_control(
                        &engine.storage,
                        state_key.clone(),
                        None,
                        &state,
                        at,
                        authorization.operator_id.as_str(),
                        "engram.recovery.prune.operation.prepared",
                        &request_id,
                        operation_id.as_str(),
                    )?;
                    (bytes, state)
                };

                let prepare_key = local_recovery_idempotency_key("prune-prepare", &idempotency_key);
                let prepared =
                    repository.prepare_recovery_prune(&rrd_engram::PrepareRecoveryPrune {
                        context: rrd_engram::MutationContext {
                            at,
                            actor: authorization.operator_id.to_string(),
                            request_id: request_id.clone(),
                            operation_id: operation_id.clone(),
                        },
                        idempotency_key: prepare_key,
                        instance_id: instance_id.clone(),
                        expected_engram_revision: state.expected_engram_revision,
                        evaluated_at: state.evaluated_at,
                        expected_catalogue_sha256: state.expected_catalogue_sha256.clone(),
                        retained_backup_ids: state.retained_backup_ids.clone(),
                        prune_candidate_backup_ids: state.prune_candidate_backup_ids.clone(),
                    })?;
                let physical = rrd_store::prune_backup_catalogue(
                    &catalogue_root,
                    &rrd_store::BackupPrunePlan {
                        expected_catalogue_sha256: state.expected_catalogue_sha256.clone(),
                        retained_backup_ids: state.retained_backup_ids.clone(),
                        prune_candidate_backup_ids: state.prune_candidate_backup_ids.clone(),
                    },
                )?;
                let complete_operation = derived_recovery_operation_id(
                    "prune-complete",
                    &authorization.engram_id,
                    &instance_id,
                    &idempotency_key,
                )?;
                let completed =
                    repository.complete_recovery_prune(&rrd_engram::CompleteRecoveryPrune {
                        context: rrd_engram::MutationContext {
                            at,
                            actor: authorization.operator_id.to_string(),
                            request_id: request_id.clone(),
                            operation_id: complete_operation,
                        },
                        idempotency_key: local_recovery_idempotency_key(
                            "prune-complete",
                            &idempotency_key,
                        ),
                        intent_id: operation_id.clone(),
                        resulting_catalogue_sha256: physical.catalogue.catalogue_sha256.clone(),
                    })?;
                let decision = rrd_contract::engramRetentionDecisionSnapshot {
                    engram_revision: state.expected_engram_revision,
                    instance_id: instance_id.clone(),
                    evaluated_at_unix_ms: state.evaluated_at,
                    retained_backup_ids: state.retained_backup_ids.clone(),
                    prune_candidate_backup_ids: state.prune_candidate_backup_ids.clone(),
                };
                let result = rrd_contract::engramRecoveryPruneResult {
                    recovery: rrd_engram::public_recovery_snapshot(&completed.document),
                    decision,
                    catalogue_revision: physical.catalogue.revision,
                    catalogue_sha256: physical.catalogue.catalogue_sha256,
                    pruned_backup_ids: physical.pruned_backup_ids,
                    idempotent_replay: recovered_operation
                        || prepared.idempotent_replay
                        || physical.idempotent_replay
                        || completed.idempotent_replay,
                };
                state.result = Some(result.clone());
                commit_local_recovery_control(
                    &engine.storage,
                    state_key,
                    Some(prepared_bytes),
                    &state,
                    at,
                    authorization.operator_id.as_str(),
                    "engram.recovery.prune.operation.completed",
                    &request_id,
                    operation_id.as_str(),
                )?;
                Ok(result)
            },
        )
    }

    /// Restores one engram-owned recovery point only to the absent
    /// `.rrflow/rrd/restores/<instance>/<restore-id>` staging root of the
    /// managed instance, where the offline `restore plan|apply` publication
    /// finds it; verifies the reopened closure; and records measured RPO/RTO
    /// evidence before releasing its in-flight retention hold.
    #[allow(clippy::too_many_arguments)]
    pub fn restore_engram_backup_store(
        database: &Path,
        authority_instance: CanonicalId,
        state_root: &Path,
        distribution_executable: &Path,
        policy_path: &Path,
        key_path: &Path,
        engram_id: CanonicalId,
        instance_id: CanonicalId,
        backup_sha256: String,
        restore_id: CanonicalId,
        started_at: u64,
        at: u64,
        request_id: String,
        operation_id: CanonicalId,
        idempotency_key: String,
    ) -> Result<rrd_contract::engramRecoveryRestoreResult> {
        validate_local_recovery_request(at, &idempotency_key)?;
        if started_at == 0 || started_at > at {
            return Err(ServiceError::Contract(
                "recovery restore start time is invalid".into(),
            ));
        }
        validate_sha256_contract(&backup_sha256, "backup_sha256")?;
        let authorization = authorize_local_recovery(
            policy_path,
            key_path,
            engram_id,
            rrd_engram::LocalengramPermission::RestoreRecovery,
            at,
        )?;
        let state_root = canonical_local_state_root(state_root)?;
        let managed = resolve_managed_instance(&state_root, &instance_id, distribution_executable)?;
        let catalogue_root = managed.catalogue_root;
        require_direct_directory(&catalogue_root, "backup catalogue")?;
        let restore_parent = managed.restore_parent;
        restore_parent
            .create_dir_all()
            .map_err(|e| ServiceError::Backup(e.to_string()))?;
        let restore_parent = restore_parent
            .pin_directory()
            .map_err(|e| ServiceError::Backup(e.to_string()))?;
        let target = restore_parent.join(restore_id.as_str());
        let audit_resource = engram_audit_resource(&authority_instance, &authorization.engram_id);
        let audit_request_sha256 = operation_digest(&(
            "engram-recovery-restore-audit-v1",
            &authorization.engram_id,
            &instance_id,
            &backup_sha256,
            &restore_id,
            started_at,
            &operation_id,
            &idempotency_key,
        ))?;
        let engine = Self::open_local_authority(database, authority_instance)?;
        audit_engram_operation(
            &engine,
            at,
            authorization.operator_id.clone(),
            audit_resource,
            request_id.clone(),
            operation_id.to_string(),
            audit_request_sha256,
            || {
                let repository = rrd_engram::engramRepository::new(
                    &engine.storage,
                    authorization.engram_id.clone(),
                );
                let operation_sha256 = operation_digest(&(
                    "engram-recovery-restore-v1",
                    &authorization.engram_id,
                    &instance_id,
                    &backup_sha256,
                    &restore_id,
                    started_at,
                    &operation_id,
                ))?;
                let state_key = local_recovery_operation_key(
                    "restore",
                    &authorization.engram_id,
                    &instance_id,
                    &idempotency_key,
                );
                let existing = engine.storage.control().get(&state_key)?;
                let recovered_operation = existing.is_some();
                let (prepared_bytes, mut state) = if let Some(bytes) = existing {
                    let state: engramRecoveryRestoreOperationState =
                        serde_json::from_slice(&bytes).map_err(contract_json)?;
                    if state.format_version != engram_RECOVERY_OPERATION_FORMAT
                        || state.operation_sha256 != operation_sha256
                        || state.instance_id != instance_id
                        || state.operation_id != operation_id
                        || state.backup_sha256 != backup_sha256
                        || state.restore_id != restore_id
                    {
                        return Err(ServiceError::IdempotencyConflict);
                    }
                    if let Some(mut result) = state.result {
                        result.idempotent_replay = true;
                        return Ok(result);
                    }
                    (bytes, state)
                } else {
                    let document = repository
                        .load()?
                        .ok_or_else(|| ServiceError::engram("engram is not initialized".into()))?;
                    let point = document
                        .recovery_points
                        .get(&backup_sha256)
                        .ok_or_else(|| ServiceError::engram("recovery point is unknown".into()))?;
                    if point.instance_id != instance_id || point.pruned_at.is_some() {
                        return Err(ServiceError::engram(
                            "recovery point is unavailable for this instance".into(),
                        ));
                    }
                    let catalogue = rrd_store::verify_backup_catalogue(&catalogue_root)?;
                    let entry = catalogue
                        .backups
                        .iter()
                        .find(|entry| entry.backup_id == backup_sha256)
                        .ok_or_else(|| {
                            ServiceError::Backup("recovery point is not catalogued".into())
                        })?;
                    if entry.archive.archive_sha256 != point.archive_sha256 {
                        return Err(ServiceError::Backup(
                            "recovery point archive identity diverges from the catalogue".into(),
                        ));
                    }
                    let state = engramRecoveryRestoreOperationState {
                        format_version: engram_RECOVERY_OPERATION_FORMAT,
                        operation_sha256: operation_sha256.clone(),
                        instance_id: instance_id.clone(),
                        operation_id: operation_id.clone(),
                        backup_sha256: backup_sha256.clone(),
                        restore_id: restore_id.clone(),
                        started_at,
                        result: None,
                    };
                    let bytes = serde_json::to_vec(&state).map_err(contract_json)?;
                    commit_local_recovery_control(
                        &engine.storage,
                        state_key.clone(),
                        None,
                        &state,
                        at,
                        authorization.operator_id.as_str(),
                        "engram.recovery.restore.operation.prepared",
                        &request_id,
                        operation_id.as_str(),
                    )?;
                    (bytes, state)
                };

                let hold_operation = derived_recovery_operation_id(
                    "restore-hold",
                    &authorization.engram_id,
                    &instance_id,
                    &idempotency_key,
                )?;
                let held = repository.pin_recovery_point(&rrd_engram::PinRecoveryPoint {
                    context: rrd_engram::MutationContext {
                        at,
                        actor: authorization.operator_id.to_string(),
                        request_id: request_id.clone(),
                        operation_id: hold_operation.clone(),
                    },
                    idempotency_key: local_recovery_idempotency_key(
                        "restore-hold",
                        &idempotency_key,
                    ),
                    backup_id: backup_sha256.clone(),
                    expires_at: None,
                })?;
                let catalogue = rrd_store::verify_backup_catalogue(&catalogue_root)?;
                let entry = catalogue
                    .backups
                    .iter()
                    .find(|entry| entry.backup_id == backup_sha256)
                    .ok_or_else(|| {
                        ServiceError::Backup("recovery point is not catalogued".into())
                    })?;
                let (inventory, reopened, physical_replay) = if target
                    .try_exists()
                    .map_err(|e| ServiceError::Backup(e.to_string()))?
                {
                    let metadata = target.metadata().map_err(|error| {
                        ServiceError::Backup(format!("cannot inspect restore target: {error}"))
                    })?;
                    if metadata.file_type().is_symlink() || !metadata.is_dir() {
                        return Err(ServiceError::Backup(
                            "existing restore target is not a direct canonical directory".into(),
                        ));
                    }
                    let restored =
                        RrflowKvStore::open_existing_anchored(&target, Default::default())?;
                    if restored.claims().sequence()? != entry.archive.claim_sequence
                        || restored.runtime().cursor()? != entry.archive.runtime_cursor
                    {
                        return Err(ServiceError::Backup(
                            "existing restore target watermarks differ from the recovery point"
                                .into(),
                        ));
                    }
                    drop(restored);
                    rrd_store::verify_restored_backup_objects(
                        &catalogue_root,
                        &backup_sha256,
                        &target,
                    )?;
                    (entry.archive.clone(), true, true)
                } else {
                    let report = rrd_store::restore_catalogued_backup(
                        &catalogue_root,
                        &backup_sha256,
                        &target,
                        at,
                    )?;
                    let restored =
                        RrflowKvStore::open_existing_anchored(&target, Default::default())?;
                    if restored.claims().sequence()? != report.inventory.claim_sequence
                        || restored.runtime().cursor()? != report.inventory.runtime_cursor
                    {
                        return Err(ServiceError::Backup(
                            "published restore watermarks failed verification".into(),
                        ));
                    }
                    drop(restored);
                    rrd_store::verify_restored_backup_objects(
                        &catalogue_root,
                        &backup_sha256,
                        &target,
                    )?;
                    (report.inventory, report.reopened, false)
                };
                let closure_sha256 = digest::sha256_hex(
                    &serde_json::to_vec(&("engram-recovery-closure-v1", entry, &inventory))
                        .map_err(contract_json)?,
                );
                let evidence =
                    repository.record_restore_evidence(&rrd_engram::RecordRestoreEvidence {
                        context: rrd_engram::MutationContext {
                            at,
                            actor: authorization.operator_id.to_string(),
                            request_id: request_id.clone(),
                            operation_id: operation_id.clone(),
                        },
                        idempotency_key: local_recovery_idempotency_key(
                            "restore-evidence",
                            &idempotency_key,
                        ),
                        restore_id: restore_id.clone(),
                        instance_id: instance_id.clone(),
                        backup_id: backup_sha256.clone(),
                        started_at: state.started_at,
                        completed_at: at,
                        restored_claim_sequence: inventory.claim_sequence,
                        restored_runtime_cursor: inventory.runtime_cursor,
                        closure_sha256,
                    })?;
                let release_operation = derived_recovery_operation_id(
                    "restore-release",
                    &authorization.engram_id,
                    &instance_id,
                    &idempotency_key,
                )?;
                let released =
                    repository.release_recovery_pin(&rrd_engram::ReleaseRecoveryPin {
                        context: rrd_engram::MutationContext {
                            at,
                            actor: authorization.operator_id.to_string(),
                            request_id: request_id.clone(),
                            operation_id: release_operation,
                        },
                        idempotency_key: local_recovery_idempotency_key(
                            "restore-release",
                            &idempotency_key,
                        ),
                        pin_id: hold_operation,
                    })?;
                let result = rrd_contract::engramRecoveryRestoreResult {
                    recovery: rrd_engram::public_recovery_snapshot(&released.document),
                    restore_id: restore_id.clone(),
                    backup_sha256: backup_sha256.clone(),
                    inventory: super::backup::public_archive(&inventory),
                    reopened,
                    idempotent_replay: recovered_operation
                        || held.idempotent_replay
                        || physical_replay
                        || evidence.idempotent_replay
                        || released.idempotent_replay,
                };
                state.result = Some(result.clone());
                commit_local_recovery_control(
                    &engine.storage,
                    state_key,
                    Some(prepared_bytes),
                    &state,
                    at,
                    authorization.operator_id.as_str(),
                    "engram.recovery.restore.operation.completed",
                    &request_id,
                    operation_id.as_str(),
                )?;
                Ok(result)
            },
        )
    }
}

struct EffectHoldDriver {
    inner: rrd_engram::LocalProcessDriver,
    marker: Option<PathBuf>,
}

impl engramDriver for EffectHoldDriver {
    fn apply(
        &mut self,
        request: &rrd_engram::DriverRequest,
    ) -> std::result::Result<rrd_engram::DriverEffect, rrd_engram::DriverError> {
        let effect = self.inner.apply(request)?;
        maybe_test_hold(self.marker.as_deref(), &effect.evidence_sha256).map_err(|error| {
            rrd_engram::DriverError::retryable(error.to_string(), effect.evidence_sha256.clone())
        })?;
        Ok(effect)
    }

    fn observe(
        &mut self,
        request: &rrd_engram::DriverRequest,
    ) -> std::result::Result<rrd_engram::DriverObservation, rrd_engram::DriverError> {
        self.inner.observe(request)
    }
}

struct BackupEffectHoldDriver {
    inner: EngineLocalBackupDriver,
    marker: Option<PathBuf>,
}

impl engramBackupDriver for BackupEffectHoldDriver {
    fn create(
        &mut self,
        request: &rrd_engram::BackupDriverRequest,
    ) -> std::result::Result<rrd_engram::BackupResult, rrd_engram::DriverError> {
        let result = self.inner.create(request)?;
        maybe_test_hold(self.marker.as_deref(), &result.backup_id).map_err(|error| {
            rrd_engram::DriverError::retryable(error.to_string(), result.evidence_sha256.clone())
        })?;
        Ok(result)
    }
}

/// Engine-owned backup driver over the engram's managed instances.
///
/// A managed instance is the installed project `instances/<instance>` below
/// the engram's state root. The driver resolves it through its own locator,
/// binds the engram's distribution executable, opens it through
/// `RrdEngine::open_installed` (the same path `serve` and `verify` take, so
/// the real token key and the installed record are checked), and writes the
/// backup into the instance's own catalogue, `.rrflow/rrd/backups/<id>`,
/// which the served engine lists and the engram prunes. There is no second
/// catalogue lane.
pub(in crate::engine) struct EngineLocalBackupDriver {
    state_root: PathBuf,
    distribution_executable: PathBuf,
}

/// The resolved filesystem coordinates of one managed instance.
struct ManagedInstance {
    catalogue_root: rrd_store::filesystem::AnchoredPath,
    restore_parent: rrd_store::filesystem::AnchoredPath,
}

/// Resolves `instances/<instance>` below the state root as an installed
/// project: reads its locator, requires the supplied executable to be the
/// installed distribution, and returns the instance's backup catalogue and
/// restore staging roots. Nothing is opened and nothing is created.
fn resolve_managed_instance(
    state_root: &Path,
    instance_id: &CanonicalId,
    distribution_executable: &Path,
) -> Result<ManagedInstance> {
    let instances = state_root.join("instances");
    require_direct_directory(&instances, "managed instances root")?;
    let project = instances.join(instance_id.as_str());
    require_direct_directory(&project, "managed instance project")?;
    let (project, locator, _) = read_locator(&project)?;
    require_distribution_binding(&locator, distribution_executable)?;
    let installed = &locator.identity.instance_id;
    Ok(ManagedInstance {
        catalogue_root: instance_operation_root(&project, BACKUPS_DIRECTORY_NAME, installed)?,
        restore_parent: instance_operation_root(&project, RESTORES_DIRECTORY_NAME, installed)?,
    })
}

impl EngineLocalBackupDriver {
    pub(in crate::engine) fn new(
        state_root: impl AsRef<Path>,
        distribution_executable: impl AsRef<Path>,
    ) -> std::result::Result<Self, String> {
        let supplied = state_root.as_ref();
        if !supplied.is_absolute() {
            return Err("local backup state root must be absolute".into());
        }
        let state_root = fs::canonicalize(supplied)
            .map_err(|error| format!("cannot resolve local backup state root: {error}"))?;
        if !state_root.is_dir() {
            return Err("local backup state root must be a directory".into());
        }
        ensure_direct_directory(&state_root, "instances")?;
        ensure_direct_directory(&state_root, "processes")?;
        let executable = distribution_executable.as_ref();
        if !executable.is_absolute() {
            return Err("distribution executable path must be absolute".into());
        }
        let distribution_executable = fs::canonicalize(executable)
            .map_err(|error| format!("cannot resolve distribution executable: {error}"))?;
        if !distribution_executable.is_file() {
            return Err("distribution executable must be a regular file".into());
        }
        Ok(Self {
            state_root,
            distribution_executable,
        })
    }

    fn project_path(&self, request: &rrd_engram::BackupDriverRequest) -> PathBuf {
        self.state_root
            .join("instances")
            .join(request.instance_id.as_str())
    }

    fn create_inner(
        &self,
        request: &rrd_engram::BackupDriverRequest,
    ) -> std::result::Result<rrd_engram::BackupResult, rrd_engram::DriverError> {
        let process_record = self
            .state_root
            .join("processes")
            .join(format!("{}.json", request.instance_id));
        match fs::symlink_metadata(&process_record) {
            Ok(_) => {
                return Err(permanent(
                    request,
                    "backup denied because the instance has a retained process record",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(retryable(
                    request,
                    format!("cannot inspect the instance process record: {error}"),
                ));
            }
        }

        let project = self.project_path(request);
        require_direct_directory(&project, "managed instance project")
            .map_err(|error| permanent(request, error.to_string()))?;
        // The instance is opened exactly as `serve` opens it: through its
        // locator with the engram's distribution binding and its own token
        // key. A project that is not an installed engram, or one installed by
        // a different distribution, fails here without creating anything.
        let engine = RrdEngine::open_installed(&project, &self.distribution_executable).map_err(
            |error| permanent(request, format!("installed instance open failed: {error}")),
        )?;
        let catalogue_root = engine.backup_root().map_err(|error| {
            permanent(
                request,
                format!("backup catalogue root unresolved: {error}"),
            )
        })?;
        RrdEngine::verify_backup_catalogue(&catalogue_root).map_err(|error| {
            permanent(
                request,
                format!("backup catalogue verification failed: {error}"),
            )
        })?;
        let entry = rrd_store::create_application_backup(
            &engine.storage,
            &engine.objects,
            &catalogue_root,
            &request.label,
            request.created_at,
            &super::backup::installed_control_families(&engine.instance),
        )
        .map_err(|error| {
            retryable(
                request,
                format!("application-complete backup creation failed: {error}"),
            )
        })?;
        let verified = RrdEngine::verify_backup_catalogue(&catalogue_root).map_err(|error| {
            permanent(
                request,
                format!("created backup verification failed: {error}"),
            )
        })?;
        let retained = verified
            .backups
            .iter()
            .find(|candidate| candidate.backup_id == entry.backup_id)
            .ok_or_else(|| permanent(request, "created backup is absent from its catalogue"))?;
        if retained != &entry {
            return Err(permanent(
                request,
                "created backup entry diverges from its verified catalogue",
            ));
        }
        let evidence_sha256 = digest::sha256_hex(
            &serde_json::to_vec(&(
                "rrd-local-engram-backup-v1",
                request,
                &entry.backup_id,
                &entry.archive.archive_sha256,
                &verified.catalogue_sha256,
            ))
            .expect("validated local backup evidence serializes"),
        );
        Ok(rrd_engram::BackupResult {
            backup_id: entry.backup_id.clone(),
            archive_sha256: entry.archive.archive_sha256.clone(),
            catalogue_sha256: verified.catalogue_sha256,
            evidence_sha256,
        })
    }
}

impl engramBackupDriver for EngineLocalBackupDriver {
    fn create(
        &mut self,
        request: &rrd_engram::BackupDriverRequest,
    ) -> std::result::Result<rrd_engram::BackupResult, rrd_engram::DriverError> {
        self.create_inner(request)
    }
}

fn ensure_direct_directory(root: &Path, child: &str) -> std::result::Result<(), String> {
    let path = root.join(child);
    create_direct_directory(&path)
        .map_err(|error| format!("cannot create local {child} root: {error}"))?;
    let canonical = fs::canonicalize(&path)
        .map_err(|error| format!("cannot resolve local {child} root: {error}"))?;
    if canonical != path || !canonical.is_dir() {
        return Err(format!("local {child} root must be a direct directory"));
    }
    Ok(())
}

fn create_direct_directory(path: &Path) -> std::io::Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(std::io::Error::other("path is not a direct directory"));
            }
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn authorize_local_recovery(
    policy_path: &Path,
    key_path: &Path,
    engram_id: CanonicalId,
    permission: rrd_engram::LocalengramPermission,
    at: u64,
) -> Result<rrd_engram::LocalOperatorAuthorization> {
    rrd_engram::LocalOperatorPolicy::load_json(policy_path)
        .map_err(ServiceError::Contract)?
        .authorize_key_file(key_path, engram_id, permission, at)
        .map_err(ServiceError::Contract)
}

fn canonical_local_state_root(supplied: &Path) -> Result<PathBuf> {
    if !supplied.is_absolute() {
        return Err(ServiceError::Contract(
            "local recovery state root must be absolute".into(),
        ));
    }
    let root = fs::canonicalize(supplied).map_err(|error| {
        ServiceError::Contract(format!("cannot resolve local recovery state root: {error}"))
    })?;
    if root != supplied || !root.is_dir() {
        return Err(ServiceError::Contract(
            "local recovery state root must be a direct canonical directory".into(),
        ));
    }
    Ok(root)
}

fn require_direct_directory(
    path: &(impl rrd_store::filesystem::AnchorInput + ?Sized),
    name: &str,
) -> Result<()> {
    let path = path
        .anchor()
        .map_err(|e| ServiceError::Contract(e.to_string()))?;
    let metadata = path
        .metadata()
        .map_err(|error| ServiceError::Contract(format!("cannot inspect {name}: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ServiceError::Contract(format!(
            "{name} must be a direct canonical directory"
        )));
    }
    Ok(())
}

fn validate_local_recovery_request(at: u64, idempotency_key: &str) -> Result<()> {
    if at == 0
        || idempotency_key.is_empty()
        || idempotency_key.len() > 128
        || !idempotency_key.is_ascii()
        || idempotency_key.as_bytes().contains(&0)
    {
        return Err(ServiceError::Contract(
            "local recovery request time or idempotency key is invalid".into(),
        ));
    }
    Ok(())
}

fn validate_sha256_contract(value: &str, name: &str) -> Result<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ServiceError::Contract(format!("{name} is not SHA-256")))
    }
}

fn local_recovery_operation_key(
    kind: &str,
    engram_id: &CanonicalId,
    instance_id: &CanonicalId,
    idempotency_key: &str,
) -> String {
    let identity = digest::sha256_hex(
        format!("engram-recovery\0{kind}\0{engram_id}\0{instance_id}\0{idempotency_key}")
            .as_bytes(),
    );
    format!("server/state/engram-recovery/{engram_id}/{kind}/{identity}")
}

fn local_recovery_idempotency_key(kind: &str, idempotency_key: &str) -> String {
    let identity = digest::sha256_hex(format!("{kind}\0{idempotency_key}").as_bytes());
    format!("{kind}-{}", &identity[..32])
}

fn derived_recovery_operation_id(
    kind: &str,
    engram_id: &CanonicalId,
    instance_id: &CanonicalId,
    idempotency_key: &str,
) -> Result<CanonicalId> {
    let digest = digest::sha256_hex(
        format!("{kind}\0{engram_id}\0{instance_id}\0{idempotency_key}").as_bytes(),
    );
    CanonicalId::new(format!("{kind}-{}", &digest[..32]))
        .map_err(|error| ServiceError::Contract(error.to_string()))
}

#[allow(clippy::too_many_arguments)]
fn commit_local_recovery_control(
    engine: &impl StorageEngine,
    key: String,
    expected: Option<Vec<u8>>,
    state: &impl Serialize,
    at: u64,
    actor: &str,
    action: &str,
    request_id: &str,
    operation_id: &str,
) -> Result<()> {
    engine.control().commit(&ControlTransition {
        key,
        expected,
        replacement: Some(serde_json::to_vec(state).map_err(contract_json)?),
        at,
        actor: actor.into(),
        action: action.into(),
        request_id: request_id.into(),
        operation_id: operation_id.into(),
    })?;
    Ok(())
}

fn permanent(
    request: &rrd_engram::BackupDriverRequest,
    message: impl Into<String>,
) -> rrd_engram::DriverError {
    let message = message.into();
    rrd_engram::DriverError::permanent(
        message.clone(),
        digest::sha256_hex(format!("backup-permanent:{}:{message}", request.job_id).as_bytes()),
    )
}

fn retryable(
    request: &rrd_engram::BackupDriverRequest,
    message: impl Into<String>,
) -> rrd_engram::DriverError {
    let message = message.into();
    rrd_engram::DriverError::retryable(
        message.clone(),
        digest::sha256_hex(format!("backup-retryable:{}:{message}", request.job_id).as_bytes()),
    )
}

#[cfg(debug_assertions)]
fn maybe_test_hold(marker: Option<&Path>, contents: &str) -> std::io::Result<()> {
    let Some(path) = marker else {
        return Ok(());
    };
    let temporary = path.with_extension("new");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    loop {
        std::thread::park_timeout(Duration::from_secs(60));
    }
}

#[cfg(not(debug_assertions))]
fn maybe_test_hold(marker: Option<&Path>, _contents: &str) -> std::io::Result<()> {
    if marker.is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "test hold failpoints are unavailable in release builds",
        ));
    }
    Ok(())
}
