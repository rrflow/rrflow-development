//! Durable, read-only Inventory through the existing selected plan and store.
use super::*;
use crate::engine::transaction_admission::TransactionPermit;
use rrd_attunement::inventory::{Proposal, SourceScope};
mod records;

pub(super) struct Work<'a> {
    pub session: &'a CorrelationId,
    pub token: &'a CorrelationId,
    pub context: &'a RequestContext,
    pub now: u64,
    pub started: Instant,
    pub principal: CanonicalId,
    pub operation_sha256: String,
    pub before_bytes: Vec<u8>,
    pub before: AttunementJob,
    pub plan: AttunementPlan,
    pub receipt_key: String,
    pub lease_generation: u64,
}

pub(super) fn execute(
    engine: &RrdEngine,
    work: Work<'_>,
    mut permit: TransactionPermit<'_>,
) -> Result<AttunementControlResult> {
    let Work {
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
        lease_generation,
    } = work;
    let holder = worker(session)?;
    let invocation =
        engine.invocation_admission(context.request_id.as_str(), context.operation_id.as_str())?;
    let check = || {
        if let Some(invocation) = &invocation {
            invocation.check()?;
        }
        let at = elapsed_time(now, started)?;
        check_deadline(context, at)?;
        require_worker(&before, &holder, lease_generation, at)
    };
    check()?;
    let installed = engine.attunement_installed_record()?;
    let (profile, profile_sha256) =
        DefaultInstallProfile::load(installed.installation.profile_id.as_str())?;
    if profile_sha256 != installed.installation.profile_sha256 {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let root = engine
        .installed_project_root
        .as_ref()
        .ok_or(ServiceError::ProjectBindingMismatch)?;
    let policy = &profile.inventory_policy;
    let output_limit = policy.limits.output_bytes.min(
        engine
            .estate_configuration
            .storage
            .transaction_admission_bytes,
    );
    let (prior_bytes, prior) =
        records::load_prior(engine, &plan, output_limit, policy.limits.entries)?;
    let source_scope = SourceScope {
        estate: plan.target.estate_id.as_str().into(),
        project: plan.target.project_id.as_str().into(),
        root: policy.logical_root.as_str().into(),
    };
    let (precondition, proposal) = super::super::installation::prepare_project_inventory(
        root,
        &profile,
        source_scope,
        prior.as_ref(),
        &check,
    )?;
    if precondition != plan.source_sha256 {
        return Err(ServiceError::AttunementSourceChanged);
    }
    check()?;
    let scope = ScopeId::new(format!("instance:{}", engine.instance)).map_err(contract)?;
    let before_read = engine.storage.runtime().read_stamp(&scope)?;
    let prepared_at = elapsed_time(now, started)?;
    let mutations = records::mutations(
        engine,
        &scope,
        &before_read,
        &proposal,
        &precondition,
        prepared_at,
    )?;
    let commit = RuntimeCommit {
        scope,
        at: prepared_at,
        actor: format!("session:{}", session.as_str()),
        expected_cursor: before_read.commit_cursor,
        mutations,
    };
    bounded(&commit, output_limit)?;
    permit.charge(
        &commit,
        engine
            .estate_configuration
            .storage
            .transaction_admission_bytes,
    )?;
    engine.authorize(
        session,
        token,
        SecurityAction::EstateAdmin,
        elapsed_time(now, started)?,
        context.request_id.as_str(),
        context.operation_id.as_str(),
    )?;
    check()?;
    let key = attunement_job_key(&engine.instance, &before.id);
    let (_, _, accepted) =
        engine
            .storage
            .runtime()
            .commit_with_control_builder(&commit, |read, outcome| {
                check()?;
                if outcome.last_cursor != read.commit_cursor {
                    return Err(ServiceError::Storage(
                        "Inventory checkpoint cursor mismatch".into(),
                    ));
                }
                let finished = elapsed_time(now, started)?;
                if finished <= before.updated_at_unix_ms {
                    return Err(contract("attunement transition time must advance"));
                }
                let mut next = before.clone();
                next.revision = next
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| contract("attunement revision overflow"))?;
                next.updated_at_unix_ms = finished;
                let selected = plan
                    .phases
                    .get(next.checkpoints.len())
                    .ok_or_else(|| contract("missing selected Inventory step"))?;
                if selected.phase != AttunementPhase::Inventory {
                    return Err(contract("selected phase is not Inventory"));
                }
                let input_sha256 = next
                    .checkpoints
                    .last()
                    .map_or_else(|| plan.source_sha256.clone(), |c| c.output_sha256.clone());
                let mut checkpoint = AttunementPhaseCheckpoint {
                    contract_version: ATTUNEMENT_CONTRACT_VERSION,
                    job_id: next.id.clone(),
                    plan_sha256: plan.plan_sha256.clone(),
                    phase: AttunementPhase::Inventory,
                    sequence: selected.sequence,
                    configuration_sha256: selected.configuration_sha256.clone(),
                    input_sha256,
                    output_sha256: proposal.proposal_sha256().into(),
                    coordinates: AttunementRuntimeCoordinates {
                        runtime_manifest_sha256: read.manifest_id.clone(),
                        runtime_cursor: read.commit_cursor,
                        schema_revision: read.schema_revision,
                        catalogue_revision: read.catalog_revision,
                    },
                    committed_at_unix_ms: finished,
                    checkpoint_sha256: "0".repeat(64),
                };
                checkpoint.checkpoint_sha256 =
                    attunement_checkpoint_sha256(&checkpoint).map_err(contract)?;
                next.checkpoints.push(checkpoint);
                next.current_phase = next.next_phase(&plan);
                before.validate_transition(&next, &plan).map_err(contract)?;
                let accepted = status(next, &plan, finished)?;
                let receipt = Receipt {
                    format_version: 1,
                    principal_id: principal.clone(),
                    operation_sha256: operation_sha256.clone(),
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
                let head = records::Head {
                    format_version: 1,
                    project_id: plan.target.project_id.clone(),
                    proposal_sha256: proposal.proposal_sha256().into(),
                    read: read.clone(),
                };
                let transitions = vec![
                    transition(
                        key,
                        Some(before_bytes),
                        encode(&accepted.job)?,
                        "attunement.inventory_checkpointed",
                    ),
                    transition(receipt_key, None, encode(&receipt)?, "attunement.receipted"),
                    transition(
                        records::head_key(&engine.instance),
                        prior_bytes,
                        encode(&head)?,
                        "attunement.inventory_published",
                    ),
                ];
                bounded(&(&commit, &transitions), output_limit)?;
                permit.charge(
                    &(&commit, &transitions),
                    engine
                        .estate_configuration
                        .storage
                        .transaction_admission_bytes,
                )?;
                check()?;
                Ok::<_, ServiceError>((transitions, accepted))
            })?;
    Ok(AttunementControlResult {
        status: accepted,
        idempotent_replay: false,
    })
}

fn bounded(value: &impl Serialize, limit: u64) -> Result<()> {
    struct Counter {
        bytes: u64,
        limit: u64,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(bytes.len() as u64)
                .filter(|b| *b <= self.limit)
                .ok_or_else(|| std::io::Error::other("Inventory output byte bound"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(&mut Counter { bytes: 0, limit }, value).map_err(|_| {
        ServiceError::ConfigurationLimit("Inventory output byte bound exceeded".into())
    })
}
