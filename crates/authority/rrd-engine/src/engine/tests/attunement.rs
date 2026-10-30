use super::*;
use crate::engine::clock::FixedClock;
use crate::{AttunementCommand, AttunementControlResult, InstallationPreview};
use rrd_contract::{
    AttunementJob, AttunementJobState as State, AttunementPhase as Phase, AttunementStatus,
    CancelAttunement, InstallationTargetKind, ResumeAttunement, SessionLease,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};

const BASE: u64 = 1_800_000_000_000;
struct Installed {
    root: tempfile::TempDir,
    executable: PathBuf,
    preview: InstallationPreview,
    job: CanonicalId,
}
impl Installed {
    fn new(mode: InstallationTargetKind) -> (Self, RrdEngine, SessionLease) {
        let root = tempfile::tempdir().unwrap();
        if mode == InstallationTargetKind::ExistingProject {
            std::fs::write(root.path().join("README.md"), "real project source\n").unwrap();
        }
        let executable = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let preview = RrdEngine::plan_installation(
            root.path(),
            mode,
            &RrdEngine::generate_installation_identity().unwrap(),
            "default",
            None,
            &executable,
        )
        .unwrap();
        RrdEngine::apply_installation_at(
            root.path(),
            mode,
            &preview,
            &preview.installation.plan_sha256,
            BASE,
            &executable,
        )
        .unwrap();
        let engine = RrdEngine::open_installed_with_clock(
            root.path(),
            &executable,
            &FixedClock::at(BASE + 1),
        )
        .unwrap();
        let job =
            CanonicalId::new(format!("job-{}", &preview.attunement.plan_sha256[..24])).unwrap();
        let installed = Self {
            root,
            executable,
            preview,
            job,
        };
        let session = installed.session(&engine, "first-session", BASE + 100);
        (installed, engine, session)
    }
    fn session(&self, engine: &RrdEngine, key: &str, at: u64) -> SessionLease {
        let credential = RrdEngine::read_installed_api_key(self.root.path()).unwrap();
        engine
            .create_authenticated_session(
                &credential.principal_id,
                credential.credential().as_bytes(),
                &CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 120_000,
                        absolute_timeout_ms: 120_000,
                        max_open_transactions: 2,
                    },
                },
                &id(key),
                at,
                &format!("request-{key}"),
                &format!("operation-{key}"),
            )
            .unwrap()
    }
    fn reopen(&self, at: u64) -> RrdEngine {
        RrdEngine::open_installed_with_clock(
            self.root.path(),
            &self.executable,
            &FixedClock::at(at),
        )
        .unwrap()
    }
    fn start(&self, lease_ms: u64) -> AttunementCommand {
        AttunementCommand::Start {
            job_id: self.job.clone(),
            expected_revision: 1,
            plan_sha256: self.preview.attunement.plan_sha256.clone(),
            lease_ms,
        }
    }
    fn read(&self, engine: &RrdEngine, session: &SessionLease, at: u64) -> AttunementStatus {
        engine
            .read_attunement(
                &session.session_id,
                &session.token,
                &self.job,
                at,
                "request-read-job",
                "operation-read-job",
            )
            .unwrap()
    }
}
fn context(key: &str) -> RequestContext {
    mutation_context(
        &id(key),
        &format!("request-{key}"),
        &format!("operation-{key}"),
    )
}
fn call(
    engine: &RrdEngine,
    session: &SessionLease,
    command: &AttunementCommand,
    key: &str,
    at: u64,
) -> crate::Result<AttunementControlResult> {
    engine.control_attunement(
        &session.session_id,
        &session.token,
        command,
        &context(key),
        at,
    )
}
fn step(job: &AttunementJob) -> AttunementCommand {
    AttunementCommand::Step {
        job_id: job.id.clone(),
        expected_revision: job.revision,
        lease_generation: job.lease.as_ref().map_or(0, |lease| lease.generation),
    }
}
fn resume(job: &AttunementJob, at: u64) -> AttunementCommand {
    AttunementCommand::Resume {
        request: ResumeAttunement {
            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
            job_id: job.id.clone(),
            plan_sha256: job.plan_sha256.clone(),
            expected_job_revision: job.revision,
            last_checkpoint_sha256: job
                .checkpoints
                .last()
                .map(|checkpoint| checkpoint.checkpoint_sha256.clone()),
            resume_from_phase: job.current_phase.unwrap(),
            requested_at_unix_ms: at,
        },
        lease_ms: 30_000,
    }
}
fn cancel(job: &AttunementJob, at: u64) -> AttunementCommand {
    AttunementCommand::Cancel {
        request: CancelAttunement {
            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
            job_id: job.id.clone(),
            plan_sha256: job.plan_sha256.clone(),
            expected_job_revision: job.revision,
            last_checkpoint_sha256: job
                .checkpoints
                .last()
                .map(|checkpoint| checkpoint.checkpoint_sha256.clone()),
            reason: "user stopped indexing".into(),
            requested_at_unix_ms: at,
        },
    }
}

#[test]
fn attunement_follows_a_bound_repeated_step_plan_after_reopen() {
    // Install a test-only plan through the actual control store. There is no
    // public operation for accepting custom attunement plans in this package.
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let mut plan = installed.preview.attunement.clone();
    plan.id = CanonicalId::new("test-variable-plan").unwrap();
    let mut second = plan.phases[0].clone();
    second.sequence = 2;
    second.configuration_sha256 = digest::sha256_hex(b"second-connect-configuration");
    let mut verify = plan.phases.last().unwrap().clone();
    verify.sequence = 3;
    plan.phases = vec![plan.phases[0].clone(), second, verify];
    plan.plan_sha256 = rrd_contract::attunement_plan_sha256(&plan).unwrap();
    plan.validate().unwrap();
    let job_id = CanonicalId::new("test-variable-job").unwrap();
    let mut job = installed.read(&engine, &session, BASE + 500).job;
    job.id = job_id.clone();
    job.plan_id = plan.id.clone();
    job.plan_sha256 = plan.plan_sha256.clone();
    job.validate(&plan).unwrap();
    let transition = |key, replacement| ControlTransition {
        key,
        expected: None,
        replacement: Some(replacement),
        at: BASE + 600,
        actor: "test:fixture".into(),
        action: "test.plan-fixture".into(),
        request_id: "test-plan".into(),
        operation_id: "test-plan".into(),
    };
    engine
        .storage
        .control()
        .commit_batch(&[
            transition(
                crate::engine::installation::attunement_plan_key(engine.instance_id(), &plan.id),
                serde_json::to_vec(&plan).unwrap(),
            ),
            transition(
                crate::engine::installation::attunement_job_key(engine.instance_id(), &job_id),
                serde_json::to_vec(&job).unwrap(),
            ),
        ])
        .unwrap();
    let start = AttunementCommand::Start {
        job_id: job_id.clone(),
        plan_sha256: plan.plan_sha256.clone(),
        expected_revision: 1,
        lease_ms: 30_000,
    };
    let running = call(&engine, &session, &start, "variable-start", BASE + 1_000).unwrap();
    let first = call(
        &engine,
        &session,
        &step(&running.status.job),
        "variable-first",
        BASE + 2_000,
    )
    .unwrap();
    assert_eq!(first.status.job.current_phase, Some(Phase::Connect));
    let paused = call(
        &engine,
        &session,
        &AttunementCommand::Pause {
            job_id: job_id.clone(),
            expected_revision: first.status.job.revision,
            lease_generation: first.status.job.lease.as_ref().unwrap().generation,
        },
        "variable-pause",
        BASE + 3_000,
    )
    .unwrap();
    drop(engine);
    let engine = installed.reopen(BASE + 4_000);
    let renewed = installed.session(&engine, "variable-reopened", BASE + 4_100);
    let read = engine
        .read_attunement(
            &renewed.session_id,
            &renewed.token,
            &job_id,
            BASE + 4_500,
            "variable-read",
            "variable-read",
        )
        .unwrap();
    assert_eq!(read.job, paused.status.job);
    let running = call(
        &engine,
        &renewed,
        &resume(&read.job, BASE + 5_000),
        "variable-resume",
        BASE + 5_000,
    )
    .unwrap();
    let second = call(
        &engine,
        &renewed,
        &step(&running.status.job),
        "variable-second",
        BASE + 6_000,
    )
    .unwrap();
    assert_eq!(second.status.job.current_phase, Some(Phase::Verify));
    assert_eq!(
        second.status.job.checkpoints[1].configuration_sha256,
        plan.phases[1].configuration_sha256
    );
    assert_eq!(
        second.status.job.checkpoints[1].input_sha256,
        second.status.job.checkpoints[0].output_sha256
    );
    let unavailable = call(
        &engine,
        &renewed,
        &step(&second.status.job),
        "variable-verify",
        BASE + 7_000,
    )
    .unwrap();
    assert_eq!(unavailable.status.job.state, State::Failed);
    assert_eq!(unavailable.status.job.checkpoints.len(), 2);
    assert_eq!(
        unavailable.status.job.failure.unwrap().code.as_str(),
        "phase-unavailable"
    );
}

#[test]
fn attunement_receipt_readback_preserves_original_result_after_reopen() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    assert!(engine
        .read_attunement_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("connect"),
            BASE + 500,
            "receipt-missing",
            "receipt-missing",
        )
        .unwrap()
        .is_none());
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "start",
        BASE + 1_000,
    )
    .unwrap();
    let command = step(&started.status.job);
    let connected = call(&engine, &session, &command, "connect", BASE + 2_000).unwrap();
    let failed = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "inventory",
        BASE + 3_000,
    )
    .unwrap();
    assert_eq!(failed.status.job.state, State::Failed);
    let before = engine.storage.control().journal_since(0, 1024).unwrap();
    let count_receipts = |entries: &[rrd_store::ControlJournalEntry]| {
        entries
            .iter()
            .filter(|entry| entry.action == "attunement.receipted")
            .count()
    };
    let receipt = engine
        .read_attunement_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("connect"),
            BASE + 3_100,
            "receipt-read",
            "receipt-read",
        )
        .unwrap()
        .unwrap();
    assert_eq!(receipt.status, connected.status);
    assert_eq!(
        receipt.operation_sha256,
        digest::sha256_hex(&serde_json::to_vec(&command).unwrap())
    );
    // Independently reconstruct the pre-existing on-disk key from the
    // canonical principal and key text, rather than calling the new helper.
    let key_input = format!("[\"{}\",\"connect\"]", receipt.principal_id.as_str());
    let expected_key = format!(
        "server/state/{}/attunement/job/{}/receipt/{}",
        engine.instance_id(),
        installed.job,
        digest::sha256_hex(key_input.as_bytes())
    );
    assert!(before
        .iter()
        .any(|entry| { entry.action == "attunement.receipted" && entry.key == expected_key }));
    assert!(engine
        .read_attunement_receipt(
            &session.session_id,
            &id("invalid-token"),
            &installed.job,
            &id("connect"),
            BASE + 3_200,
            "receipt-invalid",
            "receipt-invalid",
        )
        .is_err());
    assert!(engine
        .read_attunement_receipt(
            &session.session_id,
            &session.token,
            &CanonicalId::new("other-job").unwrap(),
            &id("connect"),
            BASE + 3_300,
            "receipt-foreign",
            "receipt-foreign",
        )
        .unwrap()
        .is_none());
    let after = engine.storage.control().journal_since(0, 1024).unwrap();
    assert_eq!(count_receipts(&before), count_receipts(&after));
    assert_eq!(
        installed.read(&engine, &session, BASE + 3_500).job,
        failed.status.job
    );
    drop(engine);
    let engine = installed.reopen(BASE + 4_000);
    let renewed = installed.session(&engine, "receipt-renewed", BASE + 4_100);
    let reopened = engine
        .read_attunement_receipt(
            &renewed.session_id,
            &renewed.token,
            &installed.job,
            &id("connect"),
            BASE + 5_000,
            "receipt-reopened",
            "receipt-reopened",
        )
        .unwrap()
        .unwrap();
    assert_eq!(reopened, receipt);
    assert_eq!(
        installed.read(&engine, &renewed, BASE + 5_500).job,
        failed.status.job
    );
}

#[test]
fn attunement_receipt_readback_is_principal_scoped_and_authorized() {
    let (installed, engine, _) = Installed::new(InstallationTargetKind::FreshProject);
    let repository = SecurityRepository::new(&engine.storage, engine.instance_id().clone());
    let mut authority = repository.load().unwrap().unwrap();
    let revision = authority.revision;
    authority.revision += 1;
    let resource = ResourcePath {
        segments: vec![
            ResourceId::new(ResourceKind::Instance, engine.instance_id().as_str()).unwrap(),
        ],
    };
    for (name, read) in [("other-reader", true), ("without-read", false)] {
        let principal_id = CanonicalId::new(name).unwrap();
        let mut grants = vec![ResourceGrant {
            action: SecurityAction::SessionCreate,
            resource_prefix: resource.clone(),
            data_policy: None,
        }];
        if read {
            grants.push(ResourceGrant {
                action: SecurityAction::EstateRead,
                resource_prefix: resource.clone(),
                data_policy: None,
            });
        }
        authority.principals.insert(
            principal_id.clone(),
            Principal {
                id: principal_id,
                kind: PrincipalKind::Service,
                credential_sha256: digest::sha256_hex(name.as_bytes()),
                credential_revision: 1,
                not_before_unix_ms: 1,
                expires_at_unix_ms: u64::MAX,
                disabled: false,
                role_ids: Default::default(),
                grants,
            },
        );
    }
    repository
        .replace(
            revision,
            authority,
            BASE + 300,
            "test",
            "security-update",
            "security-update",
        )
        .unwrap();
    let owner = installed.session(&engine, "receipt-owner", BASE + 500);
    call(
        &engine,
        &owner,
        &installed.start(30_000),
        "owner-receipt",
        BASE + 1_000,
    )
    .unwrap();
    let make_session = |name: &str| {
        engine
            .create_authenticated_session(
                &CanonicalId::new(name).unwrap(),
                name.as_bytes(),
                &CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 120_000,
                        absolute_timeout_ms: 120_000,
                        max_open_transactions: 2,
                    },
                },
                &id(name),
                BASE + 1_500,
                &format!("request-{name}"),
                &format!("operation-{name}"),
            )
            .unwrap()
    };
    let other = make_session("other-reader");
    let denied = make_session("without-read");
    assert!(engine
        .read_attunement_receipt(
            &other.session_id,
            &other.token,
            &installed.job,
            &id("owner-receipt"),
            BASE + 2_000,
            "other-read",
            "other-read"
        )
        .unwrap()
        .is_none());
    assert!(matches!(
        engine.read_attunement_receipt(
            &denied.session_id,
            &denied.token,
            &installed.job,
            &id("owner-receipt"),
            BASE + 2_100,
            "denied-read",
            "denied-read"
        ),
        Err(ServiceError::PermissionDenied)
    ));
    assert!(engine
        .read_attunement_receipt(
            &owner.session_id,
            &owner.token,
            &installed.job,
            &id("owner-receipt"),
            BASE + 2_200,
            "owner-read",
            "owner-read"
        )
        .unwrap()
        .is_some());
}

#[test]
fn attunement_receipt_readback_rejects_corrupt_identity() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::FreshProject);
    call(
        &engine,
        &session,
        &installed.start(30_000),
        "receipt-corrupt",
        BASE + 1_000,
    )
    .unwrap();
    let key = engine
        .storage
        .control()
        .journal_since(0, 1024)
        .unwrap()
        .into_iter()
        .find(|entry| entry.action == "attunement.receipted")
        .unwrap()
        .key;
    let original = engine.storage.control().get(&key).unwrap().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["operation_sha256"] = serde_json::json!("not-a-digest");
    engine
        .storage
        .control()
        .commit(&ControlTransition {
            key,
            expected: Some(original),
            replacement: Some(serde_json::to_vec(&value).unwrap()),
            at: BASE + 1_500,
            actor: "test".into(),
            action: "test.receipt-corruption".into(),
            request_id: "corrupt-request".into(),
            operation_id: "corrupt-operation".into(),
        })
        .unwrap();
    assert!(matches!(
        engine.read_attunement_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("receipt-corrupt"),
            BASE + 2_000,
            "corrupt-read",
            "corrupt-read"
        ),
        Err(ServiceError::Storage(_))
    ));
}

#[test]
fn attunement_detail_reports_failure_and_does_not_claim_source_verification() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let request = rrd_contract::ReadAttunement {
        contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
        job_id: installed.job.clone(),
    };
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "detail-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "detail-connect",
        BASE + 2_000,
    )
    .unwrap();
    let failed = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "detail-inventory",
        BASE + 3_000,
    )
    .unwrap();
    std::fs::write(installed.root.path().join("README.md"), "changed source\n").unwrap();
    let read = engine
        .read_attunement_detail(
            &session.session_id,
            &session.token,
            &request,
            BASE + 4_000,
            "request-detail",
            "operation-detail",
        )
        .unwrap();
    assert_eq!(read.plan, installed.preview.attunement);
    assert_eq!(read.status.job, failed.status.job);
    assert_eq!(read.status.job.state, State::Failed);
    assert_eq!(
        read.status.job.failure.as_ref().unwrap().code.as_str(),
        "phase-unavailable"
    );
    assert_eq!(
        read.source_verification,
        rrd_contract::AttunementSourceVerification::NotChecked
    );
    let invalid_token = id("invalid-token");
    assert!(engine
        .read_attunement_detail(
            &session.session_id,
            &invalid_token,
            &request,
            BASE + 4_001,
            "request-denied",
            "operation-denied"
        )
        .is_err());
}

#[test]
fn attunement_installed_connect_failure_and_exact_receipts_survive_reopen() {
    for mode in [
        InstallationTargetKind::FreshProject,
        InstallationTargetKind::ExistingProject,
    ] {
        let (installed, engine, session) = Installed::new(mode);
        let pending = installed.read(&engine, &session, BASE + 500);
        assert_eq!(pending.job.state, State::Pending);
        let start = call(
            &engine,
            &session,
            &installed.start(30_000),
            "start",
            BASE + 1_000,
        )
        .unwrap();
        assert_eq!(start.status.job.state, State::Running);
        let work = step(&start.status.job);
        let connected = call(&engine, &session, &work, "connect", BASE + 2_000).unwrap();
        assert_eq!(connected.status.job.current_phase, Some(Phase::Inventory));
        assert_eq!(connected.status.job.checkpoints.len(), 1);
        let checkpoint = &connected.status.job.checkpoints[0];
        assert_eq!(checkpoint.phase, Phase::Connect);
        let scope = rrd_core::ScopeId::new(format!(
            "instance:{}",
            installed.preview.attunement.target.instance_id
        ))
        .unwrap();
        let read = engine.storage.runtime().read_stamp(&scope).unwrap();
        assert_eq!(
            checkpoint.coordinates.runtime_manifest_sha256,
            read.manifest_id
        );
        assert_eq!(checkpoint.coordinates.runtime_cursor, read.commit_cursor);
        let failed = call(
            &engine,
            &session,
            &step(&connected.status.job),
            "inventory",
            BASE + 3_000,
        )
        .unwrap();
        assert_eq!(failed.status.job.state, State::Failed);
        assert_eq!(
            failed.status.job.failure.as_ref().unwrap().code.as_str(),
            "phase-unavailable"
        );
        assert_eq!(
            failed.status.job.checkpoints,
            connected.status.job.checkpoints
        );
        assert!(failed.status.job.lease.is_none());
        drop(engine);
        let engine = installed.reopen(BASE + 4_000);
        let renewed = installed.session(&engine, "second-session", BASE + 4_100);
        let replay = call(&engine, &renewed, &work, "connect", BASE + 5_000).unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(
            replay.status, connected.status,
            "replay returns the original receipt even after later failure"
        );
        assert_eq!(
            installed.read(&engine, &renewed, BASE + 5_500).job,
            failed.status.job
        );
        let resumed = call(
            &engine,
            &renewed,
            &resume(&failed.status.job, BASE + 6_000),
            "resume",
            BASE + 6_000,
        )
        .unwrap();
        assert_eq!(resumed.status.job.current_phase, Some(Phase::Inventory));
        assert!(
            resumed.status.job.lease.as_ref().unwrap().generation
                > start.status.job.lease.as_ref().unwrap().generation
        );
        let failed_again = call(
            &engine,
            &renewed,
            &step(&resumed.status.job),
            "inventory-again",
            BASE + 7_000,
        )
        .unwrap();
        assert_eq!(
            failed_again.status.job.checkpoints.len(),
            1,
            "no fake inventory checkpoint"
        );
        drop(engine);
        let report = RrdEngine::inspect_installed_with_clock(
            installed.root.path(),
            &installed.executable,
            &FixedClock::at(BASE + 8_000),
        )
        .unwrap();
        assert_eq!(report.status, crate::InstallationVerificationStatus::Passed);
    }
}

#[test]
fn attunement_competing_workers_only_one_claims_and_foreign_worker_cannot_publish() {
    let (installed, engine, first) = Installed::new(InstallationTargetKind::ExistingProject);
    let second = installed.session(&engine, "other-worker", BASE + 200);
    let engine = Arc::new(engine);
    let barrier = Arc::new(Barrier::new(2));
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for (session, key) in [(&first, "first-claim"), (&second, "second-claim")] {
            let engine = Arc::clone(&engine);
            let barrier = Arc::clone(&barrier);
            let command = installed.start(30_000);
            handles.push(scope.spawn(move || {
                barrier.wait();
                call(&engine, session, &command, key, BASE + 1_000)
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let winner = if results[0].is_ok() { &first } else { &second };
    let loser = if results[0].is_ok() { &second } else { &first };
    let running = installed.read(&engine, winner, BASE + 2_000).job;
    assert!(matches!(
        call(
            &engine,
            loser,
            &step(&running),
            "foreign-step",
            BASE + 3_000
        ),
        Err(ServiceError::StorageConflict(_))
    ));
    assert_eq!(installed.read(&engine, winner, BASE + 3_500).job, running);
    assert!(call(
        &engine,
        winner,
        &step(&running),
        "winner-step",
        BASE + 4_000
    )
    .is_ok());
}

#[test]
fn attunement_expiry_recovery_and_pause_fence_prior_workers() {
    let (installed, engine, first) = Installed::new(InstallationTargetKind::FreshProject);
    let second = installed.session(&engine, "recovery-worker", BASE + 200);
    let started = call(
        &engine,
        &first,
        &installed.start(2_000),
        "start",
        BASE + 1_000,
    )
    .unwrap();
    let old_work = step(&started.status.job);
    let recovery = AttunementCommand::RecoverExpired {
        job_id: installed.job.clone(),
        expected_revision: 2,
    };
    assert!(call(&engine, &second, &recovery, "early-recovery", BASE + 2_000).is_err());
    assert!(call(&engine, &first, &old_work, "expired-step", BASE + 4_000).is_err());
    let paused = call(&engine, &second, &recovery, "recovery", BASE + 5_000).unwrap();
    assert_eq!(paused.status.job.state, State::Paused);
    assert!(paused.status.job.checkpoints.is_empty());
    drop(engine);
    let engine = installed.reopen(BASE + 5_500);
    let resumed = call(
        &engine,
        &second,
        &resume(&paused.status.job, BASE + 6_000),
        "resume",
        BASE + 6_000,
    )
    .unwrap();
    assert!(call(&engine, &first, &old_work, "zombie", BASE + 7_000).is_err());
    let mut forged = step(&resumed.status.job);
    if let AttunementCommand::Step {
        lease_generation, ..
    } = &mut forged
    {
        *lease_generation = 2;
    }
    assert!(call(&engine, &second, &forged, "old-generation", BASE + 8_000).is_err());
    let lease = resumed.status.job.lease.as_ref().unwrap();
    let paused = call(
        &engine,
        &second,
        &AttunementCommand::Pause {
            job_id: installed.job.clone(),
            expected_revision: resumed.status.job.revision,
            lease_generation: lease.generation,
        },
        "pause",
        BASE + 9_000,
    )
    .unwrap();
    assert_eq!(paused.status.job.state, State::Paused);
    assert!(call(
        &engine,
        &second,
        &step(&resumed.status.job),
        "paused-worker",
        BASE + 10_000
    )
    .is_err());
}

#[test]
fn attunement_cancel_wins_over_stale_work_and_is_durable() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let running = call(
        &engine,
        &session,
        &installed.start(30_000),
        "start",
        BASE + 1_000,
    )
    .unwrap();
    let old_work = step(&running.status.job);
    let command = cancel(&running.status.job, BASE + 2_000);
    let requested = call(&engine, &session, &command, "cancel", BASE + 2_000).unwrap();
    assert_eq!(requested.status.job.state, State::CancelRequested);
    assert!(call(
        &engine,
        &session,
        &old_work,
        "stale-after-cancel",
        BASE + 3_000
    )
    .is_err());
    drop(engine);
    let engine = installed.reopen(BASE + 4_000);
    let complete = call(
        &engine,
        &session,
        &step(&requested.status.job),
        "ack-cancel",
        BASE + 5_000,
    )
    .unwrap();
    assert_eq!(complete.status.job.state, State::Cancelled);
    assert!(complete.status.job.lease.is_none());
    assert!(complete.status.job.checkpoints.is_empty());
    let replay = call(&engine, &session, &command, "cancel", BASE + 6_000).unwrap();
    assert_eq!(replay.status, requested.status);
    assert!(replay.idempotent_replay);
    assert_eq!(
        installed.read(&engine, &session, BASE + 7_000).job,
        complete.status.job
    );
    assert!(call(
        &engine,
        &session,
        &installed.start(30_000),
        "restart-cancelled",
        BASE + 8_000
    )
    .is_err());
}

#[test]
fn attunement_rejects_invalid_identity_auth_budget_and_changed_idempotency() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::FreshProject);
    let original = installed.read(&engine, &session, BASE + 500).job;
    let mut wrong_plan = installed.start(30_000);
    if let AttunementCommand::Start { plan_sha256, .. } = &mut wrong_plan {
        *plan_sha256 = "0".repeat(64);
    }
    assert!(call(&engine, &session, &wrong_plan, "wrong-plan", BASE + 1_000).is_err());
    for (lease_ms, key) in [(0, "zero-lease"), (60_001, "huge-lease")] {
        assert!(call(
            &engine,
            &session,
            &installed.start(lease_ms),
            key,
            BASE + 2_000
        )
        .is_err());
    }
    let mut deadline = context("expired-deadline");
    deadline.deadline_unix_ms = Some(BASE + 2_999);
    assert!(matches!(
        engine.control_attunement(
            &session.session_id,
            &session.token,
            &installed.start(30_000),
            &deadline,
            BASE + 3_000
        ),
        Err(ServiceError::DeadlineExceeded)
    ));
    assert!(engine
        .control_attunement(
            &session.session_id,
            &id("invalid-token"),
            &installed.start(30_000),
            &context("bad-token"),
            BASE + 3_000
        )
        .is_err());
    assert!(engine
        .read_attunement(
            &session.session_id,
            &session.token,
            &CanonicalId::new("foreign-job").unwrap(),
            BASE + 3_000,
            "request-foreign",
            "operation-foreign"
        )
        .is_err());
    assert_eq!(
        installed.read(&engine, &session, BASE + 3_500).job,
        original
    );
    call(
        &engine,
        &session,
        &installed.start(30_000),
        "accepted-key",
        BASE + 4_000,
    )
    .unwrap();
    assert!(matches!(
        call(
            &engine,
            &session,
            &installed.start(29_000),
            "accepted-key",
            BASE + 5_000
        ),
        Err(ServiceError::IdempotencyConflict)
    ));
    assert!(
        engine
            .control_attunement(
                &session.session_id,
                &id("invalid-token"),
                &installed.start(30_000),
                &context("accepted-key"),
                BASE + 6_000
            )
            .is_err(),
        "receipt cannot bypass auth"
    );
}

#[test]
fn attunement_control_encoding_is_bounded_and_cancel_remains_available_at_limit() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::FreshProject);
    let mut oversized = installed.start(30_000);
    if let AttunementCommand::Start { plan_sha256, .. } = &mut oversized {
        *plan_sha256 = "f".repeat(128 * 1024);
    }
    assert!(matches!(
        call(&engine, &session, &oversized, "oversized", BASE + 1_000),
        Err(ServiceError::ConfigurationLimit(_))
    ));
    assert_eq!(
        installed.read(&engine, &session, BASE + 1_500).job.state,
        State::Pending
    );
    let running = call(
        &engine,
        &session,
        &installed.start(30_000),
        "start",
        BASE + 2_000,
    )
    .unwrap();
    // Characterize the finite lifetime limit without manufacturing 1023 writes.
    // Only fixture setup writes this canonical control record directly.
    let key = crate::engine::installation::attunement_job_key(
        &installed.preview.attunement.target.instance_id,
        &installed.job,
    );
    let expected = engine.storage.control().get(&key).unwrap();
    let mut exhausted = running.status.job;
    exhausted.revision = 1024;
    exhausted.validate(&installed.preview.attunement).unwrap();
    engine
        .storage
        .control()
        .commit(&ControlTransition {
            key,
            expected,
            replacement: Some(serde_json::to_vec(&exhausted).unwrap()),
            at: BASE + 2_500,
            actor: "test-fixture".into(),
            action: "attunement.fixture_limit".into(),
            request_id: "fixture-limit".into(),
            operation_id: "fixture-limit".into(),
        })
        .unwrap();
    assert!(matches!(
        call(
            &engine,
            &session,
            &step(&exhausted),
            "limited-work",
            BASE + 3_000
        ),
        Err(ServiceError::ConfigurationLimit(_))
    ));
    let requested = call(
        &engine,
        &session,
        &cancel(&exhausted, BASE + 4_000),
        "cancel-limit",
        BASE + 4_000,
    )
    .unwrap();
    let cancelled = call(
        &engine,
        &session,
        &step(&requested.status.job),
        "ack-limit",
        BASE + 5_000,
    )
    .unwrap();
    assert_eq!(cancelled.status.job.state, State::Cancelled);
    assert_eq!(cancelled.status.job.revision, 1026);
    assert!(call(
        &engine,
        &session,
        &cancel(&cancelled.status.job, BASE + 6_000),
        "cancel-terminal",
        BASE + 6_000
    )
    .is_err());
}

#[test]
fn attunement_distinct_installed_projects_cannot_read_or_start_each_others_job() {
    let (first, first_engine, first_session) = Installed::new(InstallationTargetKind::FreshProject);
    let (second, second_engine, second_session) =
        Installed::new(InstallationTargetKind::FreshProject);
    assert_ne!(first.job, second.job);
    assert!(first_engine
        .read_attunement(
            &first_session.session_id,
            &first_session.token,
            &second.job,
            BASE + 1_000,
            "foreign-project",
            "foreign-project"
        )
        .is_err());
    assert!(call(
        &second_engine,
        &second_session,
        &first.start(30_000),
        "foreign-start",
        BASE + 2_000
    )
    .is_err());
    assert_eq!(
        first
            .read(&first_engine, &first_session, BASE + 3_000)
            .job
            .state,
        State::Pending
    );
    assert_eq!(
        second
            .read(&second_engine, &second_session, BASE + 3_000)
            .job
            .state,
        State::Pending
    );
}

// This ignored entry point is invoked by the native parent test below, not
// counted as independent qualification. It uses the real installed engine.
#[test]
#[ignore = "native attunement subprocess fixture"]
fn attunement_crash_child() {
    let root = PathBuf::from(
        std::env::var_os("RRFLOW_ATTUNEMENT_TEST_ROOT").expect("parent fixture root"),
    );
    let ready =
        PathBuf::from(std::env::var_os("RRFLOW_ATTUNEMENT_TEST_READY").expect("parent ready path"));
    let job = CanonicalId::new(std::env::var("RRFLOW_ATTUNEMENT_TEST_JOB").unwrap()).unwrap();
    let plan_sha256 = std::env::var("RRFLOW_ATTUNEMENT_TEST_PLAN").unwrap();
    let executable = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let engine =
        RrdEngine::open_installed_with_clock(&root, &executable, &FixedClock::at(BASE + 1_000))
            .unwrap();
    let credential = RrdEngine::read_installed_api_key(&root).unwrap();
    let session = engine
        .create_authenticated_session(
            &credential.principal_id,
            credential.credential().as_bytes(),
            &CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 120_000,
                    absolute_timeout_ms: 120_000,
                    max_open_transactions: 2,
                },
            },
            &id("crashed-worker-session"),
            BASE + 1_100,
            "request-crashed-session",
            "operation-crashed-session",
        )
        .unwrap();
    let claim = AttunementCommand::Start {
        job_id: job,
        plan_sha256,
        expected_revision: 1,
        lease_ms: 30_000,
    };
    call(&engine, &session, &claim, "crashed-start", BASE + 2_000).unwrap();
    // Signal only after the authoritative commit; the parent has not received
    // the accepted response. It terminates us with the engine still open.
    std::fs::write(ready, "claimed").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(30));
    panic!("parent failed to terminate the attunement fixture");
}

#[test]
fn attunement_native_worker_kill_recovers_claim_and_replays_lost_response() {
    let (installed, engine, _) = Installed::new(InstallationTargetKind::ExistingProject);
    drop(engine);
    let artifacts = tempfile::tempdir().unwrap();
    let ready = artifacts.path().join("claimed");
    let output_path = artifacts.path().join("child.log");
    let output = std::fs::File::create(&output_path).unwrap();
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = ChildGuard(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "engine::tests::attunement::attunement_crash_child",
                "--ignored",
                "--nocapture",
            ])
            .env("RRFLOW_ATTUNEMENT_TEST_ROOT", installed.root.path())
            .env("RRFLOW_ATTUNEMENT_TEST_READY", &ready)
            .env("RRFLOW_ATTUNEMENT_TEST_JOB", installed.job.as_str())
            .env(
                "RRFLOW_ATTUNEMENT_TEST_PLAN",
                &installed.preview.attunement.plan_sha256,
            )
            .stdout(output.try_clone().unwrap())
            .stderr(output)
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !ready.exists() {
        assert!(
            child.0.try_wait().unwrap().is_none() && std::time::Instant::now() < deadline,
            "child failed before claim: {}",
            std::fs::read_to_string(&output_path).unwrap()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let engine = installed.reopen(BASE + 40_000);
    let session = installed.session(&engine, "surviving-worker", BASE + 40_100);
    let running = installed.read(&engine, &session, BASE + 41_000).job;
    assert_eq!(running.state, State::Running);
    assert_eq!(running.revision, 2);
    assert!(running.checkpoints.is_empty());
    let replay = call(
        &engine,
        &session,
        &installed.start(30_000),
        "crashed-start",
        BASE + 42_000,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.status.job, running);
    let paused = call(
        &engine,
        &session,
        &AttunementCommand::RecoverExpired {
            job_id: installed.job.clone(),
            expected_revision: 2,
        },
        "recover-kill",
        BASE + 43_000,
    )
    .unwrap();
    let running = call(
        &engine,
        &session,
        &resume(&paused.status.job, BASE + 44_000),
        "resume-kill",
        BASE + 44_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&running.status.job),
        "connect-after-kill",
        BASE + 45_000,
    )
    .unwrap();
    assert_eq!(connected.status.job.checkpoints.len(), 1);
    assert_eq!(connected.status.job.current_phase, Some(Phase::Inventory));
    let replay = call(
        &engine,
        &session,
        &step(&running.status.job),
        "connect-after-kill",
        BASE + 46_000,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.status, connected.status);
}
