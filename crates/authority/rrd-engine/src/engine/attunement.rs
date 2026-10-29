//! Explicit engine-owned attunement control. Existing transport exposes read
//! and cancellation; this module does not activate background execution,
//! source scanning or external capabilities.
//! The first worker phase verifies the installed connection; later phases fail
//! explicitly until their real implementation is present.

use super::installation::{attunement_job_key, attunement_plan_key, estate_configuration_key};
use super::*;
use rrd_contract::{
    attunement_checkpoint_sha256, AttunementFailure, AttunementJob, AttunementJobState,
    AttunementLease, AttunementPhase, AttunementPhaseCheckpoint, AttunementPlan,
    AttunementReadResult, AttunementRuntimeCoordinates, AttunementSourceVerification,
    AttunementStatus, CancelAttunement, ReadAttunement, ResumeAttunement,
    INSTALL_ATTUNEMENT_CONTRACT_VERSION,
};
use std::time::Instant;

const MAX_CONTROL_BYTES: usize = 64 * 1024;
const MAX_JOB_TRANSITIONS: u64 = 1024;
const MAX_LEASE_MS: u64 = 60_000;

/// Embedded admission only. This does not change the HTTP operation catalogue.
/// Callers supply exact revision/plan identity, never a checkpoint or phase output.
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

/// An existing committed control receipt, read without replaying its command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttunementReceiptView {
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
                contract_version: INSTALL_ATTUNEMENT_CONTRACT_VERSION,
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
                claim(&mut next, &holder, *lease_ms, at)?;
                "attunement.started"
            }
            AttunementCommand::Resume { request, lease_ms } => {
                request.validate_for(&before, &plan).map_err(contract)?;
                if request.requested_at_unix_ms > at {
                    return Err(ServiceError::Contract(
                        "resume time is in the future".into(),
                    ));
                }
                claim(&mut next, &holder, *lease_ms, at)?;
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
                next.current_phase = next.next_phase();
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
        let mut checkpoint = AttunementPhaseCheckpoint {
            contract_version: INSTALL_ATTUNEMENT_CONTRACT_VERSION,
            job_id: next.id.clone(),
            plan_sha256: plan.plan_sha256.clone(),
            phase: AttunementPhase::Connect,
            sequence: 1,
            configuration_sha256: plan.phases[0].configuration_sha256.clone(),
            input_sha256: plan.source_sha256.clone(),
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
        next.current_phase = next.next_phase();
        Ok(())
    }
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

fn claim(job: &mut AttunementJob, holder: &CanonicalId, lease_ms: u64, at: u64) -> Result<()> {
    if lease_ms == 0 || lease_ms > MAX_LEASE_MS {
        return Err(ServiceError::ConfigurationLimit(
            "attunement lease must be 1..=60000 milliseconds".into(),
        ));
    }
    job.state = AttunementJobState::Running;
    job.current_phase = job.next_phase();
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
        contract_version: INSTALL_ATTUNEMENT_CONTRACT_VERSION,
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
