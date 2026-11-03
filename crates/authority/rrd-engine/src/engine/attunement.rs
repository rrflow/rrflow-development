//! Explicit engine-owned attunement control. Bounded precondition observations
//! do not activate background execution or external capabilities. Connect verifies
//! the installation; Inventory commits bounded source observations with its
//! checkpoint. Other phases fail explicitly until their implementation is present.

use super::installation::{
    attunement_job_key, attunement_plan_key, canonical_prefixed, estate_configuration_key,
    project_inventory_sha256, DefaultInstallProfile, InstalledEstateRecord, INSTALLED_RECORD_KEY,
};
use super::*;
use rrd_contract::{
    attunement_checkpoint_sha256, attunement_plan_sha256, attunement_replan_accept_sha256,
    attunement_replan_preview_sha256, attunement_replan_seed, AttunementFailure, AttunementJob,
    AttunementJobState, AttunementLease, AttunementPhase, AttunementPhaseCheckpoint,
    AttunementPlan, AttunementReadResult, AttunementRuntimeCoordinates,
    AttunementSourceVerification, AttunementStatus, CancelAttunement, ReadAttunement,
    ResumeAttunement, ATTUNEMENT_CONTRACT_VERSION,
};
pub use rrd_contract::{
    AttunementReplanPreview, AttunementReplanReceiptView, AttunementReplanResult,
};
use std::time::Instant;

mod discovery;
mod inventory;

const MAX_CONTROL_BYTES: usize = 64 * 1024;
const MAX_JOB_TRANSITIONS: u64 = 1024;
const MAX_LEASE_MS: u64 = 60_000;

/// Engine admission. Callers supply exact revision/plan identity, never a
/// checkpoint or phase output; transports expose only catalogued commands.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AttunementCommand {
    Start {
        job_id: CanonicalId,
        plan_sha256: String,
        expected_revision: u64,
        lease_ms: u64,
    },
    Resume {
        request: ResumeAttunement,
        lease_ms: u64,
    },
    Step {
        job_id: CanonicalId,
        expected_revision: u64,
        lease_generation: u64,
    },
    Pause {
        job_id: CanonicalId,
        expected_revision: u64,
        lease_generation: u64,
    },
    RecoverExpired {
        job_id: CanonicalId,
        expected_revision: u64,
    },
    Cancel {
        request: CancelAttunement,
    },
}

impl AttunementCommand {
    fn binding(&self) -> (&CanonicalId, u64) {
        match self {
            Self::Start {
                job_id,
                expected_revision,
                ..
            }
            | Self::Step {
                job_id,
                expected_revision,
                ..
            }
            | Self::Pause {
                job_id,
                expected_revision,
                ..
            }
            | Self::RecoverExpired {
                job_id,
                expected_revision,
            } => (job_id, *expected_revision),
            Self::Resume { request, .. } => (&request.job_id, request.expected_job_revision),
            Self::Cancel { request } => (&request.job_id, request.expected_job_revision),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttunementControlResult {
    /// On replay, the exact accepted snapshot, not the potentially newer job.
    pub status: AttunementStatus,
    pub idempotent_replay: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplanReceipt {
    format_version: u16,
    principal_id: CanonicalId,
    operation_sha256: String,
    result: AttunementReplanResult,
}

/// An existing committed control receipt, read without replaying its command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttunementReceiptView {
    pub plan: AttunementPlan,
    pub principal_id: CanonicalId,
    pub operation_sha256: String,
    pub status: AttunementStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    format_version: u16,
    principal_id: CanonicalId,
    operation_sha256: String,
    status: AttunementStatus,
}

impl RrdEngine {
    fn validate_replan_receipt(
        &self,
        principal: &CanonicalId,
        old: &AttunementJob,
        old_plan: &AttunementPlan,
        receipt: ReplanReceipt,
    ) -> Result<AttunementReplanReceiptView> {
        if receipt.format_version != 1 || &receipt.principal_id != principal {
            return Err(ServiceError::Storage(
                "invalid attunement replan receipt identity".into(),
            ));
        }
        let link_key = format!("{}/successor", attunement_job_key(&self.instance, &old.id));
        let preview: AttunementReplanPreview =
            decode(
                &self.storage.control().get(&link_key)?.ok_or_else(|| {
                    ServiceError::Storage("attunement replan link missing".into())
                })?,
            )?;
        let result = receipt.result;
        result.predecessor.validate(old_plan).map_err(contract)?;
        result.successor.validate(&result.plan).map_err(contract)?;
        let operation_digest = attunement_replan_accept_sha256(&preview, &preview.proposal_sha256)
            .map_err(contract)?;
        let (current_generation, expected_generation) =
            (self.attunement_replan_generation(old)?, preview.generation);
        if preview
            .validate_for(&old.id, preview.predecessor_revision, old_plan)
            .is_err()
            || result.validate_for(&preview, old_plan).is_err()
            || receipt.operation_sha256 != operation_digest
            || current_generation.checked_add(1) != Some(expected_generation)
            || old.revision < result.predecessor.job.revision
            || result.idempotent_replay
        {
            return Err(ServiceError::Storage(
                "invalid attunement replan receipt lineage".into(),
            ));
        }
        let (_, successor, stored_plan) = self.load_attunement(&result.successor.job.id)?;
        if stored_plan != preview.plan
            || self.attunement_replan_generation(&successor)? != expected_generation
        {
            return Err(ServiceError::Storage(
                "invalid attunement successor binding".into(),
            ));
        }
        Ok(AttunementReplanReceiptView {
            principal_id: receipt.principal_id,
            operation_sha256: receipt.operation_sha256,
            preview,
            predecessor_plan: old_plan.clone(),
            result,
        })
    }

    /// Read the original principal's receipt without replaying acceptance.
    #[allow(clippy::too_many_arguments)]
    pub fn read_attunement_replan_receipt(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        job_id: &CanonicalId,
        idempotency_key: &CorrelationId,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<Option<AttunementReplanReceiptView>> {
        let (_, authorization) = self.authorize(
            session,
            token,
            SecurityAction::EstateRead,
            now,
            request_id,
            operation_id,
        )?;
        let principal = authorization
            .principal_id
            .ok_or(ServiceError::Unauthenticated)?;
        let key =
            attunement_replan_receipt_key(&self.instance, job_id, &principal, idempotency_key)?;
        let Some(bytes) = self.storage.control().get(&key)? else {
            return Ok(None);
        };
        let receipt: ReplanReceipt = decode(&bytes)?;
        let (_, old, old_plan) = self.load_attunement(job_id)?;
        self.validate_replan_receipt(&principal, &old, &old_plan, receipt)
            .map(Some)
    }

    fn attunement_installed_record(&self) -> Result<InstalledEstateRecord> {
        let record: InstalledEstateRecord = decode(
            &self
                .storage
                .control()
                .get(INSTALLED_RECORD_KEY)?
                .ok_or_else(|| ServiceError::Storage("installed estate record missing".into()))?,
        )?;
        record.validate()?;
        if self.installed_estate_identity() != Some(&record.installation.target) {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        Ok(record)
    }

    fn attunement_source_observation(&self) -> Result<String> {
        let record = self.attunement_installed_record()?;
        let (profile, digest) =
            DefaultInstallProfile::load(record.installation.profile_id.as_str())?;
        if digest != record.installation.profile_sha256 {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let root = self
            .installed_project_root
            .as_ref()
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        project_inventory_sha256(root, &profile)
    }

    fn attunement_replan_generation(&self, job: &AttunementJob) -> Result<u64> {
        let key = format!(
            "{}/replan-generation",
            attunement_job_key(&self.instance, &job.id)
        );
        match self.storage.control().get(&key)? {
            Some(value) => {
                let generation: u64 = decode(&value)?;
                if generation == 0 {
                    return Err(ServiceError::Storage(
                        "invalid attunement replan generation".into(),
                    ));
                }
                Ok(generation)
            }
            None => {
                let record = self.attunement_installed_record()?;
                let initial = canonical_prefixed("job", &record.attunement_plan_sha256)?;
                if job.id != initial || job.plan_sha256 != record.attunement_plan_sha256 {
                    return Err(ServiceError::Storage(
                        "attunement replan generation missing".into(),
                    ));
                }
                Ok(0)
            }
        }
    }

    fn make_attunement_replan(
        &self,
        job: &AttunementJob,
        old: &AttunementPlan,
    ) -> Result<AttunementReplanPreview> {
        // Connect and Inventory record read-only source observations. A successor
        // retains their history and binds fresh source. Other completed effects
        // require explicit reconciliation before a successor can start.
        if job.checkpoints.iter().any(|checkpoint| {
            !matches!(
                checkpoint.phase,
                AttunementPhase::Connect | AttunementPhase::Inventory
            )
        }) || !matches!(
            job.state,
            AttunementJobState::Pending
                | AttunementJobState::Running
                | AttunementJobState::Paused
                | AttunementJobState::Failed
        ) {
            return Err(ServiceError::StorageConflict(
                "attunement replan requires an unresolved, locally fenced job".into(),
            ));
        }
        if job.state == AttunementJobState::Failed
            && !job.failure.as_ref().is_some_and(|failure| {
                failure.phase != AttunementPhase::Connect
                    && failure.code.as_str() == "phase-unavailable"
                    && failure.retryable
            })
        {
            return Err(ServiceError::StorageConflict(
                "attunement failure requires explicit reconciliation before replan".into(),
            ));
        }
        let generation = self
            .attunement_replan_generation(job)?
            .checked_add(1)
            .ok_or_else(|| {
                ServiceError::ConfigurationLimit("attunement replan generation overflow".into())
            })?;
        let source = self.attunement_source_observation()?;
        let seed =
            attunement_replan_seed(&job.id, job.revision, &old.plan_sha256, generation, &source)
                .map_err(contract)?;
        let mut plan = old.clone();
        plan.id = canonical_prefixed("attune", &seed)?;
        plan.source_sha256 = source;
        plan.plan_sha256 = attunement_plan_sha256(&plan).map_err(contract)?;
        plan.validate().map_err(contract)?;
        let proposal_sha256 =
            attunement_replan_preview_sha256(&job.id, job.revision, generation, &plan)
                .map_err(contract)?;
        Ok(AttunementReplanPreview {
            predecessor_job_id: job.id.clone(),
            predecessor_revision: job.revision,
            generation,
            plan,
            proposal_sha256,
        })
    }

    /// A proposal does not accept a plan or authorize any source mutation.
    #[allow(clippy::too_many_arguments)]
    pub fn preview_attunement_replan(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        job_id: &CanonicalId,
        expected_revision: u64,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<AttunementReplanPreview> {
        let started = Instant::now();
        self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            now,
            request_id,
            operation_id,
        )?;
        let (_, job, plan) = self.load_attunement(job_id)?;
        if job.revision != expected_revision {
            return Err(ServiceError::StorageConflict(
                "attunement job revision changed".into(),
            ));
        }
        let preview = self.make_attunement_replan(&job, &plan)?;
        self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            elapsed_time(now, started)?,
            request_id,
            operation_id,
        )?;
        Ok(preview)
    }

    /// Atomically fence the prior job and publish the accepted successor.
    #[allow(clippy::too_many_arguments)]
    pub fn accept_attunement_replan(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        preview: &AttunementReplanPreview,
        expected_sha256: &str,
        context: &RequestContext,
        now: u64,
    ) -> Result<AttunementReplanResult> {
        let started = Instant::now();
        context.validate(true).map_err(contract)?;
        check_deadline(context, now)?;
        let (_, auth) = self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            now,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let principal = auth.principal_id.ok_or(ServiceError::Unauthenticated)?;
        let _ = encode(preview)?;
        if expected_sha256 != preview.proposal_sha256 {
            return Err(ServiceError::OperationDigestMismatch);
        }
        let operation_sha256 =
            attunement_replan_accept_sha256(preview, expected_sha256).map_err(contract)?;
        let receipt_key = attunement_replan_receipt_key(
            &self.instance,
            &preview.predecessor_job_id,
            &principal,
            context.idempotency_key.as_ref().ok_or_else(|| {
                ServiceError::Contract("attunement replan requires idempotency".into())
            })?,
        )?;
        let _gate = self.admit_transaction(
            preview,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let at = elapsed_time(now, started)?;
        check_deadline(context, at)?;
        self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            at,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let (old_bytes, old, old_plan) = self.load_attunement(&preview.predecessor_job_id)?;
        if let Some(bytes) = self.storage.control().get(&receipt_key)? {
            let receipt: ReplanReceipt = decode(&bytes)?;
            if receipt.principal_id != principal || receipt.operation_sha256 != operation_sha256 {
                return Err(ServiceError::IdempotencyConflict);
            }
            let view = self.validate_replan_receipt(&principal, &old, &old_plan, receipt)?;
            if view.preview != *preview {
                return Err(ServiceError::IdempotencyConflict);
            }
            return Ok(AttunementReplanResult {
                idempotent_replay: true,
                ..view.result
            });
        }
        if old.revision != preview.predecessor_revision {
            return Err(ServiceError::StorageConflict(
                "attunement job revision changed".into(),
            ));
        }
        let refreshed = self.make_attunement_replan(&old, &old_plan)?;
        if refreshed.plan.source_sha256 != preview.plan.source_sha256 {
            return Err(ServiceError::AttunementSourceChanged);
        }
        if refreshed != *preview {
            return Err(ServiceError::OperationDigestMismatch);
        }
        let at = elapsed_time(now, started)?;
        check_deadline(context, at)?;
        self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            at,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let finished = elapsed_time(now, started)?;
        check_deadline(context, finished)?;
        if finished <= old.updated_at_unix_ms {
            return Err(ServiceError::Contract(
                "attunement transition time must advance".into(),
            ));
        }
        let mut predecessor = old.clone();
        predecessor.revision = old
            .revision
            .checked_add(1)
            .ok_or_else(|| ServiceError::Contract("attunement revision overflow".into()))?;
        predecessor.updated_at_unix_ms = finished;
        predecessor.state = AttunementJobState::CancelRequested;
        predecessor.current_phase = predecessor.next_phase(&old_plan);
        predecessor.failure = None;
        old.validate_transition(&predecessor, &old_plan)
            .map_err(contract)?;
        let successor = AttunementJob {
            contract_version: ATTUNEMENT_CONTRACT_VERSION,
            id: canonical_prefixed("job", &preview.plan.plan_sha256)?,
            plan_id: preview.plan.id.clone(),
            plan_sha256: preview.plan.plan_sha256.clone(),
            revision: 1,
            state: AttunementJobState::Pending,
            current_phase: None,
            checkpoints: Vec::new(),
            lease: None,
            failure: None,
            created_at_unix_ms: finished,
            updated_at_unix_ms: finished,
        };
        let result = AttunementReplanResult {
            predecessor: status(predecessor, &old_plan, finished)?,
            successor: status(successor, &preview.plan, finished)?,
            plan: preview.plan.clone(),
            idempotent_replay: false,
        };
        let receipt = ReplanReceipt {
            format_version: 1,
            principal_id: principal,
            operation_sha256,
            result: result.clone(),
        };
        let transition = |key, expected, replacement, action: &str| ControlTransition {
            key,
            expected,
            replacement: Some(replacement),
            at: finished,
            actor: format!("session:{}", session.as_str()),
            action: action.into(),
            request_id: context.request_id.as_str().into(),
            operation_id: context.operation_id.as_str().into(),
        };
        let old_key = attunement_job_key(&self.instance, &old.id);
        let new_key = attunement_job_key(&self.instance, &result.successor.job.id);
        self.storage.control().commit_batch(&[
            transition(
                old_key.clone(),
                Some(old_bytes),
                encode(&result.predecessor.job)?,
                "attunement.replan_fenced",
            ),
            transition(
                attunement_plan_key(&self.instance, &preview.plan.id),
                None,
                encode(&preview.plan)?,
                "attunement.replan_plan_created",
            ),
            transition(
                new_key.clone(),
                None,
                encode(&result.successor.job)?,
                "attunement.replan_job_created",
            ),
            transition(
                format!("{new_key}/replan-generation"),
                None,
                encode(&preview.generation)?,
                "attunement.replan_generation",
            ),
            transition(
                format!("{old_key}/successor"),
                None,
                encode(preview)?,
                "attunement.replan_linked",
            ),
            transition(
                receipt_key,
                None,
                encode(&receipt)?,
                "attunement.replan_receipted",
            ),
        ])?;
        Ok(result)
    }

    /// Read the installed job through current estate authorization.
    #[allow(clippy::too_many_arguments)]
    pub fn read_attunement(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        job: &CanonicalId,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<AttunementStatus> {
        self.read_attunement_detail(
            session,
            token,
            &ReadAttunement {
                contract_version: ATTUNEMENT_CONTRACT_VERSION,
                job_id: job.clone(),
            },
            now,
            request_id,
            operation_id,
        )
        .map(|result| result.status)
    }

    /// Return the stored plan with its job. Source freshness is not checked.
    #[allow(clippy::too_many_arguments)]
    pub fn read_attunement_detail(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &ReadAttunement,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<AttunementReadResult> {
        request.validate().map_err(contract)?;
        self.authorize(
            session,
            token,
            SecurityAction::EstateRead,
            now,
            request_id,
            operation_id,
        )?;
        let (_, job, plan) = self.load_attunement(&request.job_id)?;
        let result = AttunementReadResult {
            status: status(job, &plan, now)?,
            plan,
            source_verification: AttunementSourceVerification::NotChecked,
        };
        result.validate().map_err(contract)?;
        Ok(result)
    }

    /// Read the caller's previously committed receipt by its original key.
    /// This does not execute a command or verify the project's current sources.
    #[allow(clippy::too_many_arguments)]
    pub fn read_attunement_receipt(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        job_id: &CanonicalId,
        idempotency_key: &CorrelationId,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<Option<AttunementReceiptView>> {
        let (_, authorization) = self.authorize(
            session,
            token,
            SecurityAction::EstateRead,
            now,
            request_id,
            operation_id,
        )?;
        let principal = authorization
            .principal_id
            .ok_or(ServiceError::Unauthenticated)?;
        let identity = self
            .installed_estate_identity()
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        let key = attunement_receipt_key(&self.instance, job_id, &principal, idempotency_key)?;
        let Some(bytes) = self.storage.control().get(&key)? else {
            return Ok(None);
        };
        let receipt: Receipt = decode(&bytes)?;
        if receipt.format_version != 1
            || receipt.principal_id != principal
            || receipt.status.job.id != *job_id
            || receipt.operation_sha256.len() != 64
            || !receipt
                .operation_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(ServiceError::Storage(
                "invalid attunement receipt identity".into(),
            ));
        }
        // Resolve the receipt's plan, not the current job's plan: a later
        // successor must not make a valid historical receipt disappear.
        let plan: AttunementPlan = decode(
            &self
                .storage
                .control()
                .get(&attunement_plan_key(
                    &self.instance,
                    &receipt.status.job.plan_id,
                ))?
                .ok_or_else(|| ServiceError::Storage("attunement receipt plan missing".into()))?,
        )?;
        if plan.target != *identity {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        receipt.status.validate(&plan).map_err(contract)?;
        Ok(Some(AttunementReceiptView {
            plan,
            principal_id: receipt.principal_id,
            operation_sha256: receipt.operation_sha256,
            status: receipt.status,
        }))
    }

    /// One finite, explicitly authorized transition. The job and its immutable
    /// accepted response commit together through the existing control WAL/CAS.
    /// A current principal may replay across renewed sessions; workers remain
    /// fenced by the original session, revision and lease generation.
    pub fn control_attunement(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        command: &AttunementCommand,
        context: &RequestContext,
        now: u64,
    ) -> Result<AttunementControlResult> {
        let started = Instant::now();
        context.validate(true).map_err(contract)?;
        check_deadline(context, now)?;
        let (_, auth) = self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            now,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let principal = auth.principal_id.ok_or(ServiceError::Unauthenticated)?;
        // Bound encoding before digesting or acquiring the transaction gate.
        let operation_sha256 = digest::sha256_hex(&encode(command)?);
        let (job_id, expected_revision) = command.binding();
        let key = attunement_job_key(&self.instance, job_id);
        let receipt_key = attunement_receipt_key(
            &self.instance,
            job_id,
            &principal,
            context.idempotency_key.as_ref().ok_or_else(|| {
                ServiceError::Contract("attunement mutation requires idempotency".into())
            })?,
        )?;
        let _gate = self.admit_transaction(
            command,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let admitted_at = elapsed_time(now, started)?;
        check_deadline(context, admitted_at)?;
        self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            admitted_at,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let (before_bytes, before, plan) = self.load_attunement(job_id)?;
        if let Some(bytes) = self.storage.control().get(&receipt_key)? {
            let receipt: Receipt = decode(&bytes)?;
            if receipt.format_version != 1
                || receipt.principal_id != principal
                || receipt.operation_sha256 != operation_sha256
            {
                return Err(ServiceError::IdempotencyConflict);
            }
            receipt.status.validate(&plan).map_err(contract)?;
            if receipt.status.job.id != *job_id {
                return Err(ServiceError::ProjectBindingMismatch);
            }
            return Ok(AttunementControlResult {
                status: receipt.status,
                idempotent_replay: true,
            });
        }
        if before.revision != expected_revision {
            return Err(ServiceError::StorageConflict(
                "attunement job revision changed".into(),
            ));
        }
        // Reserve cancellation even when the normal work/receipt budget is
        // exhausted. At most the request and its acknowledgement add two
        // transitions; terminal jobs cannot grow this exception indefinitely.
        let cancelling = matches!(command, AttunementCommand::Cancel { .. })
            || (matches!(command, AttunementCommand::Step { .. })
                && before.state == AttunementJobState::CancelRequested);
        if before.revision >= MAX_JOB_TRANSITIONS && !cancelling {
            return Err(ServiceError::ConfigurationLimit(
                "attunement transition/receipt limit reached".into(),
            ));
        }
        if let AttunementCommand::Step {
            lease_generation, ..
        } = command
        {
            if !cancelling && before.current_phase == Some(AttunementPhase::Inventory) {
                require_worker(&before, &worker(session)?, *lease_generation, admitted_at)?;
                return inventory::execute(
                    self,
                    inventory::Work {
                        session,
                        token,
                        context,
                        now,
                        started,
                        principal,
                        operation_sha256,
                        before_bytes,
                        before,
                        plan,
                        receipt_key,
                        lease_generation: *lease_generation,
                    },
                    _gate,
                );
            }
        }
        // Other controls still observe the installed precondition. This cannot
        // authorize a project-owned file mutation.
        let requires_source = matches!(
            command,
            AttunementCommand::Start { .. } | AttunementCommand::Resume { .. }
        ) || (matches!(command, AttunementCommand::Step { .. })
            && !cancelling);
        if requires_source && self.attunement_source_observation()? != plan.source_sha256 {
            return Err(ServiceError::AttunementSourceChanged);
        }
        let at = elapsed_time(now, started)?;
        check_deadline(context, at)?;
        if at <= before.updated_at_unix_ms {
            return Err(ServiceError::Contract(
                "attunement transition time must advance".into(),
            ));
        }
        let mut next = before.clone();
        next.revision = before
            .revision
            .checked_add(1)
            .ok_or_else(|| ServiceError::Contract("attunement revision overflow".into()))?;
        next.updated_at_unix_ms = at;
        let holder = worker(session)?;
        let action = match command {
            AttunementCommand::Start {
                plan_sha256,
                lease_ms,
                ..
            } => {
                if before.state != AttunementJobState::Pending || *plan_sha256 != plan.plan_sha256 {
                    return Err(ServiceError::Contract(
                        "start must bind the pending installed plan".into(),
                    ));
                }
                claim(&mut next, &plan, &holder, *lease_ms, at)?;
                "attunement.started"
            }
            AttunementCommand::Resume { request, lease_ms } => {
                request.validate_for(&before, &plan).map_err(contract)?;
                if request.requested_at_unix_ms > at {
                    return Err(ServiceError::Contract(
                        "resume time is in the future".into(),
                    ));
                }
                claim(&mut next, &plan, &holder, *lease_ms, at)?;
                "attunement.resumed"
            }
            AttunementCommand::Pause {
                lease_generation, ..
            } => {
                require_worker(&before, &holder, *lease_generation, at)?;
                next.state = AttunementJobState::Paused;
                next.lease = None;
                "attunement.paused"
            }
            AttunementCommand::RecoverExpired { .. } => {
                if before.state != AttunementJobState::Running
                    || before
                        .lease
                        .as_ref()
                        .is_none_or(|lease| at < lease.expires_at_unix_ms)
                {
                    return Err(ServiceError::StorageConflict(
                        "attunement worker is not expired".into(),
                    ));
                }
                next.state = AttunementJobState::Paused;
                next.lease = None;
                "attunement.lease_recovered"
            }
            AttunementCommand::Cancel { request } => {
                request.validate_for(&before, &plan).map_err(contract)?;
                if request.requested_at_unix_ms > at {
                    return Err(ServiceError::Contract(
                        "cancel time is in the future".into(),
                    ));
                }
                next.state = AttunementJobState::CancelRequested;
                next.current_phase = next.next_phase(&plan);
                next.failure = None;
                "attunement.cancel_requested"
            }
            AttunementCommand::Step {
                lease_generation, ..
            } => {
                if before.state == AttunementJobState::CancelRequested {
                    // This worker owns no asynchronous/external resources. CAS
                    // invalidates any stale step before publication.
                    next.state = AttunementJobState::Cancelled;
                    next.current_phase = None;
                    next.lease = None;
                    "attunement.cancelled"
                } else {
                    require_worker(&before, &holder, *lease_generation, at)?;
                    if before.current_phase == Some(AttunementPhase::Connect) {
                        self.attunement_connect(&plan, &mut next, at)?;
                        "attunement.connect_checkpointed"
                    } else {
                        let phase = before
                            .current_phase
                            .ok_or_else(|| ServiceError::Contract("missing worker phase".into()))?;
                        next.state = AttunementJobState::Failed;
                        next.lease = None;
                        next.failure = Some(AttunementFailure {
                            phase, code: CanonicalId::new("phase-unavailable").map_err(contract)?,
                            message: "This attunement phase has no installed executor; no source or index work was performed.".into(),
                            retryable: true,
                            evidence_sha256: operation_digest(&("phase-unavailable-v1", &plan.plan_sha256, phase))?,
                            observed_at_unix_ms: at,
                        });
                        "attunement.phase_unavailable"
                    }
                }
            }
        };
        // Recheck authorization after waiting/computation, then account for
        // the time spent doing that check before evaluating the lease/deadline.
        let authorization_at = elapsed_time(now, started)?;
        self.authorize(
            session,
            token,
            SecurityAction::EstateAdmin,
            authorization_at,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let finished = elapsed_time(now, started)?;
        check_deadline(context, finished)?;
        // Never publish a worker result after its lease expired while waiting
        // or computing. Claim leases also start before their publication.
        if let AttunementCommand::Step {
            lease_generation, ..
        }
        | AttunementCommand::Pause {
            lease_generation, ..
        } = command
        {
            if before.state == AttunementJobState::Running {
                require_worker(&before, &holder, *lease_generation, finished)?;
            }
        }
        if next.state == AttunementJobState::Running
            && next
                .lease
                .as_ref()
                .is_none_or(|lease| finished >= lease.expires_at_unix_ms)
        {
            return Err(ServiceError::DeadlineExceeded);
        }
        next.updated_at_unix_ms = finished;
        before.validate_transition(&next, &plan).map_err(contract)?;
        let accepted = status(next, &plan, finished)?;
        let receipt = Receipt {
            format_version: 1,
            principal_id: principal,
            operation_sha256,
            status: accepted.clone(),
        };
        let transition = |key, expected, replacement, action: &str| ControlTransition {
            key,
            expected,
            replacement: Some(replacement),
            at: finished,
            actor: format!("session:{}", session.as_str()),
            action: action.into(),
            request_id: context.request_id.as_str().into(),
            operation_id: context.operation_id.as_str().into(),
        };
        self.storage.control().commit_batch(&[
            transition(key, Some(before_bytes), encode(&accepted.job)?, action),
            transition(receipt_key, None, encode(&receipt)?, "attunement.receipted"),
        ])?;
        Ok(AttunementControlResult {
            status: accepted,
            idempotent_replay: false,
        })
    }

    fn load_attunement(
        &self,
        id: &CanonicalId,
    ) -> Result<(Vec<u8>, AttunementJob, AttunementPlan)> {
        let identity = self
            .installed_estate_identity()
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        let bytes = self
            .storage
            .control()
            .get(&attunement_job_key(&self.instance, id))?
            .ok_or_else(|| ServiceError::Contract("installed attunement job not found".into()))?;
        let job: AttunementJob = decode(&bytes)?;
        if job.id != *id {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let plan: AttunementPlan = decode(
            &self
                .storage
                .control()
                .get(&attunement_plan_key(&self.instance, &job.plan_id))?
                .ok_or_else(|| ServiceError::Storage("installed attunement plan missing".into()))?,
        )?;
        if plan.target != *identity {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        job.validate(&plan).map_err(contract)?;
        Ok((bytes, job, plan))
    }

    fn attunement_connect(
        &self,
        plan: &AttunementPlan,
        next: &mut AttunementJob,
        at: u64,
    ) -> Result<()> {
        let configuration: EstateConfiguration = decode(
            &self
                .storage
                .control()
                .get(&estate_configuration_key(&self.instance))?
                .ok_or_else(|| ServiceError::Storage("installed configuration missing".into()))?,
        )?;
        configuration.validate().map_err(contract)?;
        if configuration != self.estate_configuration {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let read = self
            .storage
            .runtime()
            .read_stamp(&ScopeId::new(format!("instance:{}", self.instance)).map_err(contract)?)?;
        let output_sha256 =
            operation_digest(&("attunement-connect-v1", &plan.target, &configuration, &read))?;
        let planned = plan
            .phases
            .get(next.checkpoints.len())
            .ok_or_else(|| ServiceError::Contract("attunement plan has no next step".into()))?;
        if planned.phase != AttunementPhase::Connect {
            return Err(ServiceError::Contract(
                "attunement step is not connect".into(),
            ));
        }
        let input_sha256 = next.checkpoints.last().map_or_else(
            || plan.source_sha256.clone(),
            |checkpoint| checkpoint.output_sha256.clone(),
        );
        let mut checkpoint = AttunementPhaseCheckpoint {
            contract_version: ATTUNEMENT_CONTRACT_VERSION,
            job_id: next.id.clone(),
            plan_sha256: plan.plan_sha256.clone(),
            phase: AttunementPhase::Connect,
            sequence: planned.sequence,
            configuration_sha256: planned.configuration_sha256.clone(),
            input_sha256,
            output_sha256,
            coordinates: AttunementRuntimeCoordinates {
                runtime_manifest_sha256: read.manifest_id,
                runtime_cursor: read.commit_cursor,
                schema_revision: read.schema_revision,
                catalogue_revision: read.catalog_revision,
            },
            committed_at_unix_ms: at,
            checkpoint_sha256: "0".repeat(64),
        };
        checkpoint.checkpoint_sha256 =
            attunement_checkpoint_sha256(&checkpoint).map_err(contract)?;
        next.checkpoints.push(checkpoint);
        next.current_phase = next.next_phase(plan);
        Ok(())
    }
}

fn attunement_replan_receipt_key(
    instance: &CanonicalId,
    job_id: &CanonicalId,
    principal: &CanonicalId,
    key: &CorrelationId,
) -> Result<String> {
    Ok(format!(
        "{}/replan-receipt/{}",
        attunement_job_key(instance, job_id),
        operation_digest(&(principal, key))?
    ))
}

fn attunement_receipt_key(
    instance: &CanonicalId,
    job_id: &CanonicalId,
    principal: &CanonicalId,
    idempotency_key: &CorrelationId,
) -> Result<String> {
    Ok(format!(
        "{}/receipt/{}",
        attunement_job_key(instance, job_id),
        operation_digest(&(principal, idempotency_key))?
    ))
}

fn claim(
    job: &mut AttunementJob,
    plan: &AttunementPlan,
    holder: &CanonicalId,
    lease_ms: u64,
    at: u64,
) -> Result<()> {
    if lease_ms == 0 || lease_ms > MAX_LEASE_MS {
        return Err(ServiceError::ConfigurationLimit(
            "attunement lease must be 1..=60000 milliseconds".into(),
        ));
    }
    job.state = AttunementJobState::Running;
    job.current_phase = job.next_phase(plan);
    job.failure = None;
    job.lease = Some(AttunementLease {
        id: CanonicalId::new(format!(
            "lease-{}",
            operation_digest(&(&job.id, job.revision))?
        ))
        .map_err(contract)?,
        holder: holder.clone(),
        generation: job.revision,
        acquired_at_unix_ms: at,
        expires_at_unix_ms: at
            .checked_add(lease_ms)
            .ok_or_else(|| ServiceError::Contract("lease time overflow".into()))?,
    });
    Ok(())
}

fn require_worker(
    job: &AttunementJob,
    holder: &CanonicalId,
    generation: u64,
    at: u64,
) -> Result<()> {
    if job.state != AttunementJobState::Running
        || job.lease.as_ref().is_none_or(|lease| {
            &lease.holder != holder
                || lease.generation != generation
                || at < lease.acquired_at_unix_ms
                || at >= lease.expires_at_unix_ms
        })
    {
        return Err(ServiceError::StorageConflict(
            "attunement worker lease is stale, expired or foreign".into(),
        ));
    }
    Ok(())
}
fn worker(session: &CorrelationId) -> Result<CanonicalId> {
    CanonicalId::new(format!(
        "worker-{}",
        digest::sha256_hex(session.as_str().as_bytes())
    ))
    .map_err(contract)
}
fn status(job: AttunementJob, plan: &AttunementPlan, at: u64) -> Result<AttunementStatus> {
    let status = AttunementStatus {
        contract_version: ATTUNEMENT_CONTRACT_VERSION,
        observed_at_unix_ms: at,
        job,
    };
    status.validate(plan).map_err(contract)?;
    Ok(status)
}
// Bound retained output during serialization, not after allocating arbitrary
// caller-supplied strings. One encoded control envelope is at most 64 KiB.
fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > MAX_CONTROL_BYTES.saturating_sub(self.0.len()) {
                return Err(std::io::Error::other("attunement control record too large"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Bounded(Vec::with_capacity(MAX_CONTROL_BYTES));
    serde_json::to_writer(&mut writer, value).map_err(|error| {
        if error.is_io() {
            ServiceError::ConfigurationLimit("attunement control record too large".into())
        } else {
            contract_json(error)
        }
    })?;
    Ok(writer.0)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_CONTROL_BYTES {
        return Err(ServiceError::ConfigurationLimit(
            "attunement control record too large".into(),
        ));
    }
    serde_json::from_slice(bytes)
        .map_err(|error| ServiceError::Storage(format!("invalid attunement control: {error}")))
}
fn contract(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::Contract(error.to_string())
}
fn elapsed_time(now: u64, started: Instant) -> Result<u64> {
    let elapsed =
        u64::try_from(started.elapsed().as_millis()).map_err(|_| ServiceError::DeadlineExceeded)?;
    now.checked_add(elapsed)
        .ok_or(ServiceError::DeadlineExceeded)
}
fn check_deadline(context: &RequestContext, now: u64) -> Result<()> {
    if context
        .deadline_unix_ms
        .is_some_and(|deadline| now >= deadline)
    {
        return Err(ServiceError::DeadlineExceeded);
    }
    Ok(())
}
