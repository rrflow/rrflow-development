use super::*;
use crate::engine::clock::FixedClock;
use crate::{AttunementCommand, AttunementControlResult, InstallationPreview};
use rrd_contract::{
    AttunementJob, AttunementJobState as State, AttunementPhase as Phase, AttunementStatus,
    CancelAttunement, InstallationTargetKind, ResumeAttunement, SessionLease,
};
use rrd_core::{RuntimeMutation, RuntimeValue, ScopeId};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};

// Fixture times start from BASE, but a committed transition records its supplied
// time plus real elapsed work. Every time after one follows the completion it
// returned, never a fixed later offset that assumes how long the call took.
const BASE: u64 = 1_800_000_000_000;
mod inventory_delta;
mod startup_binding;

struct Installed {
    root: tempfile::TempDir,
    executable: PathBuf,
    preview: InstallationPreview,
    job: CanonicalId,
}
impl Installed {
    fn new(mode: InstallationTargetKind) -> (Self, RrdEngine, SessionLease) {
        Self::with_proposal(mode, None)
    }
    fn with_proposal(
        mode: InstallationTargetKind,
        proposal: Option<&crate::AttunementProposal>,
    ) -> (Self, RrdEngine, SessionLease) {
        Self::with_source(mode, proposal, |_| {})
    }
    fn with_source(
        mode: InstallationTargetKind,
        proposal: Option<&crate::AttunementProposal>,
        setup: impl FnOnce(&Path),
    ) -> (Self, RrdEngine, SessionLease) {
        Self::with_configuration(mode, proposal, setup, None)
    }
    fn with_configuration(
        mode: InstallationTargetKind,
        proposal: Option<&crate::AttunementProposal>,
        setup: impl FnOnce(&Path),
        configuration: Option<&Path>,
    ) -> (Self, RrdEngine, SessionLease) {
        let root = tempfile::tempdir().unwrap();
        setup(root.path());
        if mode == InstallationTargetKind::ExistingProject {
            std::fs::write(root.path().join("README.md"), "real project source\n").unwrap();
        }
        let executable = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let preview = RrdEngine::plan_installation_with_proposal(
            root.path(),
            mode,
            &RrdEngine::generate_installation_identity().unwrap(),
            "default",
            configuration,
            proposal,
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
fn publicly_selected_plan_replans_after_progress_and_reopens() {
    selected_plan_replans_after_progress_and_reopens(false);
}

#[test]
fn selected_parse_policy_survives_installation_and_engine_reopen() {
    let policy = rrd_contract::SourceParsePolicy {
        format_version: 1,
        bindings: vec![rrd_contract::SourceParseBinding {
            extension: "rs".into(),
            language: CanonicalId::new("rust").unwrap(),
        }],
        max_files: 64,
        max_file_bytes: 1_048_576,
        max_input_bytes: 4_194_304,
        max_nodes: 10_000,
        max_depth: 64,
        max_output_bytes: 1_048_576,
        max_commit_bytes: 262_144,
        max_elapsed_ms: 10_000,
        max_memory_bytes: 8_388_608,
    };
    let proposal = crate::AttunementProposal {
        format_version: crate::ATTUNEMENT_PROPOSAL_FORMAT_VERSION,
        steps: [
            Phase::Connect,
            Phase::Inventory,
            Phase::Parse,
            Phase::Verify,
        ]
        .into_iter()
        .map(|phase| crate::AttunementProposalStep {
            phase,
            description: format!("{phase:?}"),
            estimated_items: 0,
            estimated_input_bytes: 0,
            parse_policy: (phase == Phase::Parse).then(|| policy.clone()),
        })
        .collect(),
        estimated_min_duration_ms: 0,
        estimated_max_duration_ms: 30_000,
    };
    let (installed, engine, _) =
        Installed::with_proposal(InstallationTargetKind::ExistingProject, Some(&proposal));
    let planned = installed
        .preview
        .attunement
        .phases
        .iter()
        .find(|step| step.phase == Phase::Parse)
        .unwrap();
    assert_eq!(planned.parse_policy.as_ref(), Some(&policy));
    drop(engine);
    let engine = installed.reopen(BASE + 4_000);
    let session = installed.session(&engine, "parse-policy-reopen", BASE + 4_100);
    let read = engine
        .read_attunement_detail(
            &session.session_id,
            &session.token,
            &rrd_contract::ReadAttunement {
                contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                job_id: installed.job.clone(),
            },
            BASE + 4_200,
            "request-parse-policy-read",
            "operation-parse-policy-read",
        )
        .unwrap();
    assert_eq!(read.plan, installed.preview.attunement);
    assert_eq!(
        read.plan
            .phases
            .iter()
            .find(|step| step.phase == Phase::Parse)
            .unwrap()
            .parse_policy
            .as_ref(),
        Some(&policy)
    );

    let mut too_large = proposal.clone();
    let oversized = too_large
        .steps
        .iter_mut()
        .find(|step| step.phase == Phase::Parse)
        .unwrap()
        .parse_policy
        .as_mut()
        .unwrap();
    let (profile, _) = crate::engine::installation::DefaultInstallProfile::load("default").unwrap();
    oversized.max_file_bytes = profile.inventory_policy.limits.file_bytes + 1;
    oversized.max_input_bytes = oversized.max_file_bytes;
    let other = tempfile::tempdir().unwrap();
    std::fs::write(other.path().join("README.md"), b"existing source").unwrap();
    let result = RrdEngine::plan_installation_with_proposal(
        other.path(),
        InstallationTargetKind::ExistingProject,
        &RrdEngine::generate_installation_identity().unwrap(),
        "default",
        None,
        Some(&too_large),
        &installed.executable,
    );
    assert!(matches!(result, Err(ServiceError::ConfigurationLimit(_))));
    assert!(!other.path().join(".rrflow").exists());
    assert_eq!(
        std::fs::read(other.path().join("README.md")).unwrap(),
        b"existing source"
    );

    let mut streamed = proposal.clone();
    let transaction_limit = installed
        .preview
        .installation
        .configuration
        .storage
        .transaction_admission_bytes;
    let bounds = streamed
        .steps
        .iter_mut()
        .find(|step| step.phase == Phase::Parse)
        .unwrap()
        .parse_policy
        .as_mut()
        .unwrap();
    bounds.max_output_bytes = transaction_limit + 1;
    bounds.max_commit_bytes = transaction_limit;
    let identity = RrdEngine::generate_installation_identity().unwrap();
    assert!(RrdEngine::plan_installation_with_proposal(
        other.path(),
        InstallationTargetKind::ExistingProject,
        &identity,
        "default",
        None,
        Some(&streamed),
        &installed.executable,
    )
    .is_ok());
    streamed
        .steps
        .iter_mut()
        .find(|step| step.phase == Phase::Parse)
        .unwrap()
        .parse_policy
        .as_mut()
        .unwrap()
        .max_commit_bytes = transaction_limit + 1;
    assert!(matches!(
        RrdEngine::plan_installation_with_proposal(
            other.path(),
            InstallationTargetKind::ExistingProject,
            &identity,
            "default",
            None,
            Some(&streamed),
            &installed.executable,
        ),
        Err(ServiceError::ConfigurationLimit(_))
    ));
    assert!(!other.path().join(".rrflow").exists());
}

#[test]
fn parse_source_is_bound_to_the_committed_inventory_after_reopen() {
    let proposal = crate::AttunementProposal {
        format_version: crate::ATTUNEMENT_PROPOSAL_FORMAT_VERSION,
        steps: [
            Phase::Connect,
            Phase::Inventory,
            Phase::Parse,
            Phase::Verify,
        ]
        .into_iter()
        .map(|phase| crate::AttunementProposalStep {
            phase,
            description: format!("{phase:?}"),
            estimated_items: 0,
            estimated_input_bytes: 0,
            parse_policy: None,
        })
        .collect(),
        estimated_min_duration_ms: 0,
        estimated_max_duration_ms: 30_000,
    };
    let (installed, engine, session) = Installed::with_source(
        InstallationTargetKind::ExistingProject,
        Some(&proposal),
        |root| std::fs::write(root.join("source.rs"), b"fn answer() { }\n").unwrap(),
    );
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "source-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "source-connect",
        BASE + 2_000,
    )
    .unwrap();
    let inventory = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "source-inventory",
        BASE + 3_000,
    )
    .unwrap();
    let job = inventory.status.job;
    assert_eq!(job.current_phase, Some(Phase::Parse));
    let mut remaining_keys = 10_000;
    assert!(crate::engine::attunement::load_parse_inventory(
        &engine,
        &connected.status.job,
        &installed.preview.attunement,
        &mut remaining_keys
    )
    .is_err());
    let mut substituted = job.clone();
    let checkpoint = substituted.checkpoints.last_mut().unwrap();
    checkpoint.output_sha256 = "0".repeat(64);
    checkpoint.checkpoint_sha256 = rrd_contract::attunement_checkpoint_sha256(checkpoint).unwrap();
    let mut denied_keys = 1;
    assert!(matches!(
        crate::engine::attunement::load_parse_inventory(
            &engine,
            &substituted,
            &installed.preview.attunement,
            &mut denied_keys
        ),
        Err(ServiceError::ProjectBindingMismatch)
    ));
    assert_eq!(
        denied_keys, 1,
        "reject a changed checkpoint before inventory reads"
    );
    let mut other_plan = installed.preview.attunement.clone();
    other_plan.source_sha256 = "1".repeat(64);
    other_plan.plan_sha256 = rrd_contract::attunement_plan_sha256(&other_plan).unwrap();
    let mut other_job = job.clone();
    other_job.plan_sha256 = other_plan.plan_sha256.clone();
    for (index, checkpoint) in other_job.checkpoints.iter_mut().enumerate() {
        checkpoint.plan_sha256 = other_plan.plan_sha256.clone();
        if index == 0 {
            checkpoint.input_sha256 = other_plan.source_sha256.clone();
        }
        checkpoint.checkpoint_sha256 =
            rrd_contract::attunement_checkpoint_sha256(checkpoint).unwrap();
    }
    other_job.validate(&other_plan).unwrap();
    assert!(matches!(
        crate::engine::attunement::load_parse_inventory(
            &engine,
            &other_job,
            &other_plan,
            &mut remaining_keys
        ),
        Err(ServiceError::ProjectBindingMismatch)
    ));
    drop(engine);
    let engine = installed.reopen(BASE + 4_000);
    let snapshot = crate::engine::attunement::load_parse_inventory(
        &engine,
        &job,
        &installed.preview.attunement,
        &mut remaining_keys,
    )
    .unwrap();
    let source = snapshot
        .entries()
        .iter()
        .find(|entry| entry.path == "source.rs")
        .unwrap();
    let reopened_session = installed.session(&engine, "source-reopened", BASE + 4_100);
    let traversal = engine
        .execute_query(
            &reopened_session.session_id,
            &reopened_session.token,
            &rrd_contract::ExecuteQuery {
                scope: format!("instance:{}", engine.instance_id()),
                query: format!(
                    "FROM traverse:source-tree-contains START source-tree-snapshot:\"{}\" DIRECTION OUTGOING DEPTH 2 AT VALID {} KNOWN HEAD PROJECT node_id, depth",
                    job.checkpoints.last().unwrap().output_sha256,
                    BASE + 4_200
                ),
                parameters: Default::default(),
                budget: rrd_contract::QueryBudget::default(),
            },
            BASE + 4_200,
            "request-source-query",
            "operation-source-query",
        )
        .unwrap();
    assert!(
        traversal.rows.iter().any(|row| {
            row.values.get("node_id")
                == Some(&rrd_contract::QueryValue::String(source.identity.clone()))
        }),
        "rrflowQL must reach the actual inventoried source identity"
    );
    let root = engine.installed_project_root.as_ref().unwrap();
    let mut remaining_bytes = 1024;
    let mut output = Vec::new();
    crate::engine::installation::read_selected_source(
        root,
        &snapshot,
        source,
        1024,
        &mut remaining_bytes,
        &mut output,
        &|| Ok(()),
    )
    .unwrap();
    assert_eq!(output, b"fn answer() { }\n");
    std::fs::write(
        installed.root.path().join("source.rs"),
        b"fn changed() {}\n",
    )
    .unwrap();
    assert!(crate::engine::installation::read_selected_source(
        root,
        &snapshot,
        source,
        1024,
        &mut remaining_bytes,
        &mut output,
        &|| Ok(())
    )
    .is_err());
}

#[test]
fn inventory_checkpoint_and_source_records_commit_reopen_and_replan_together() {
    let proposal = crate::AttunementProposal {
        format_version: crate::ATTUNEMENT_PROPOSAL_FORMAT_VERSION,
        steps: [Phase::Connect, Phase::Inventory, Phase::Verify]
            .into_iter()
            .map(|phase| crate::AttunementProposalStep {
                phase,
                description: format!("{phase:?}"),
                estimated_items: 0,
                estimated_input_bytes: 0,
                parse_policy: None,
            })
            .collect(),
        estimated_min_duration_ms: 0,
        estimated_max_duration_ms: 30_000,
    };
    let (installed, engine, session) = Installed::with_source(
        InstallationTargetKind::ExistingProject,
        Some(&proposal),
        |root| {
            std::fs::write(root.join("before.txt"), "rename content\n").unwrap();
            std::fs::write(root.join("removed.txt"), "remove content\n").unwrap();
        },
    );
    let running = call(
        &engine,
        &session,
        &installed.start(30_000),
        "inventory-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&running.status.job),
        "inventory-connect",
        running.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let inventoried = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "inventory-step",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let inventoried_at = inventoried.status.job.updated_at_unix_ms;
    assert_eq!(inventoried.status.job.state, State::Running);
    assert_eq!(inventoried.status.job.checkpoints.len(), 2);
    let checkpoint = inventoried.status.job.checkpoints.last().unwrap();
    assert_eq!(checkpoint.phase, Phase::Inventory);
    let scope = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
    let read = engine.storage.runtime().read_stamp(&scope).unwrap();
    assert_eq!(checkpoint.coordinates.runtime_cursor, read.commit_cursor);
    assert_eq!(
        checkpoint.coordinates.runtime_manifest_sha256,
        read.manifest_id
    );
    let replay = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "inventory-step",
        inventoried_at + 1,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.status, inventoried.status);
    assert_eq!(engine.storage.runtime().read_stamp(&scope).unwrap(), read);
    let changes = engine
        .storage
        .runtime()
        .read_changes(&read, 0, 512)
        .unwrap();
    assert!(changes.changes.iter().any(|change| matches!(&change.mutation,
        RuntimeMutation::Record { record } if record.reference.kind.as_str() == "source-tree-snapshot")));
    assert!(changes.changes.iter().any(|change| matches!(&change.mutation,
        RuntimeMutation::Relation { relation } if relation.reference.kind.as_str() == "source-tree-contains")));
    drop(engine);
    let engine = installed.reopen(inventoried_at + 2);
    let session = installed.session(&engine, "inventory-reopen", inventoried_at + 3);
    assert_eq!(
        installed
            .read(&engine, &session, inventoried_at + 4)
            .job
            .checkpoints,
        inventoried.status.job.checkpoints
    );
    std::fs::rename(
        installed.root.path().join("before.txt"),
        installed.root.path().join("after.txt"),
    )
    .unwrap();
    std::fs::remove_file(installed.root.path().join("removed.txt")).unwrap();
    std::fs::write(installed.root.path().join("added.txt"), "added content\n").unwrap();
    std::fs::write(
        installed.root.path().join("README.md"),
        "edited after Inventory\n",
    )
    .unwrap();
    assert!(matches!(
        call(
            &engine,
            &session,
            &step(&inventoried.status.job),
            "inventory-stale",
            inventoried_at + 5
        ),
        Err(ServiceError::AttunementSourceChanged)
    ));
    let replan = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            inventoried.status.job.revision,
            inventoried_at + 6,
            "request-inventory-replan",
            "operation-inventory-replan",
        )
        .unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &replan,
            &replan.proposal_sha256,
            &context("inventory-accept"),
            inventoried_at + 7,
        )
        .unwrap();
    let successor = &accepted.successor.job;
    let started = call(
        &engine,
        &session,
        &AttunementCommand::Start {
            job_id: successor.id.clone(),
            plan_sha256: successor.plan_sha256.clone(),
            expected_revision: successor.revision,
            lease_ms: 30_000,
        },
        "inventory-successor-start",
        successor.updated_at_unix_ms + 1,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "inventory-successor-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let next = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "inventory-successor-step",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(next.status.job.checkpoints.len(), 2);
    assert_ne!(
        next.status.job.checkpoints[1].output_sha256,
        checkpoint.output_sha256
    );
    let new_read = engine.storage.runtime().read_stamp(&scope).unwrap();
    let delta = engine
        .storage
        .runtime()
        .read_changes(&new_read, read.commit_cursor, 512)
        .unwrap();
    assert!(delta.changes.iter().any(|change| matches!(&change.mutation,
        RuntimeMutation::Record { record } if record.reference.kind.as_str() == "source-tree-change-set"
            && matches!(record.properties.get("modified"), Some(RuntimeValue::List(values)) if !values.is_empty()))));
    let change_set = delta
        .changes
        .iter()
        .find_map(|change| match &change.mutation {
            RuntimeMutation::Record { record }
                if record.reference.kind.as_str() == "source-tree-change-set" =>
            {
                Some(record)
            }
            _ => None,
        })
        .unwrap();
    for field in ["added", "removed"] {
        assert!(
            matches!(change_set.properties.get(field), Some(RuntimeValue::List(values)) if values.len() == 2)
        );
    }
    assert!(
        matches!(change_set.properties.get("rename_candidates"), Some(RuntimeValue::List(values)) if values.len() == 1)
    );
    let retired: Vec<_> = delta
        .changes
        .iter()
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Retire { retirement }
                if retirement.reference.kind.as_str() == "source-tree-entry" =>
            {
                Some(retirement.reference.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(retired.len(), 2);
    for path in ["before.txt", "removed.txt"] {
        let original = changes
            .changes
            .iter()
            .find_map(|change| match &change.mutation {
                RuntimeMutation::Record { record }
                    if record.reference.kind.as_str() == "source-tree-entry"
                        && record.properties.get("path")
                            == Some(&RuntimeValue::String(path.into())) =>
                {
                    Some(&record.reference)
                }
                _ => None,
            })
            .unwrap();
        assert!(retired.contains(original));
    }
}

#[test]
fn inventory_source_rejection_preserves_runtime_and_allows_retry() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "retry-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "retry-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let connected_at = connected.status.job.updated_at_unix_ms;
    let scope = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
    let before = engine.storage.runtime().read_stamp(&scope).unwrap();
    let head = format!("server/state/{}/inventory/head", engine.instance_id());
    let original = std::fs::read(installed.root.path().join("README.md")).unwrap();
    std::fs::write(installed.root.path().join("README.md"), "changed input\n").unwrap();
    assert!(matches!(
        call(
            &engine,
            &session,
            &step(&connected.status.job),
            "retry-inventory",
            connected_at + 1
        ),
        Err(ServiceError::AttunementSourceChanged)
    ));
    assert_eq!(engine.storage.runtime().read_stamp(&scope).unwrap(), before);
    assert!(engine.storage.control().get(&head).unwrap().is_none());
    assert_eq!(
        installed.read(&engine, &session, connected_at + 2).job,
        connected.status.job
    );
    std::fs::write(installed.root.path().join("README.md"), original).unwrap();
    let accepted = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "retry-inventory",
        connected_at + 3,
    )
    .unwrap();
    assert!(
        !accepted.idempotent_replay,
        "a failed attempt must not consume its receipt key"
    );
    assert_eq!(accepted.status.job.checkpoints.len(), 2);
    assert!(engine.storage.control().get(&head).unwrap().is_some());
}

#[test]
fn inventory_larger_than_one_physical_batch_publishes_and_reopens() {
    let (installed, engine, session) =
        Installed::with_source(InstallationTargetKind::ExistingProject, None, |root| {
            let source = root.join("src");
            std::fs::create_dir(&source).unwrap();
            for index in 0..2048 {
                std::fs::write(
                    source.join(format!("entry-{index:04}.txt")),
                    format!("source entry {index}\n"),
                )
                .unwrap();
            }
        });
    let started = call(
        &engine,
        &session,
        &installed.start(60_000),
        "large-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "large-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let inventoried = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "large-inventory",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(inventoried.status.job.checkpoints.len(), 2);
    assert_eq!(
        inventoried.status.job.checkpoints[1].phase,
        Phase::Inventory
    );
    let groups = engine
        .storage
        .control()
        .journal_since(0, 1024)
        .unwrap()
        .into_iter()
        .filter(|entry| entry.action == "attunement.inventory_group_published")
        .count();
    assert!(
        groups > 1,
        "the whole input must cross the native atomic batch boundary"
    );
    let reopened_at = inventoried.status.job.updated_at_unix_ms + 1_000;
    drop(engine);
    let engine = installed.reopen(reopened_at);
    let session = installed.session(&engine, "large-reopen", reopened_at + 100);
    assert_eq!(
        installed.read(&engine, &session, reopened_at + 200).job,
        inventoried.status.job
    );
}

#[test]
fn inventory_output_bound_preserves_runtime_checkpoint_and_head_after_reopen() {
    let defaults = rrd_contract::EstateConfiguration::default();
    let input = rrd_contract::EstateConfigurationInput {
        format_version: defaults.format_version,
        clock: defaults.clock,
        storage: rrd_contract::StorageLimits {
            transaction_admission_bytes: 16_384,
            ..defaults.storage
        },
        reasoning: defaults.reasoning,
        recall: defaults.recall,
        query: defaults.query,
    };
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), toml::to_string(&input).unwrap()).unwrap();
    let (installed, engine, session) = Installed::with_configuration(
        InstallationTargetKind::ExistingProject,
        None,
        |root| {
            for index in 0..64 {
                std::fs::write(root.join(format!("source-{index}.txt")), "bounded source\n")
                    .unwrap();
            }
        },
        Some(file.path()),
    );
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "bound-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "bound-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let connected_at = connected.status.job.updated_at_unix_ms;
    let scope = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
    let before = engine.storage.runtime().read_stamp(&scope).unwrap();
    let head = format!("server/state/{}/inventory/head", engine.instance_id());
    assert!(matches!(
        call(
            &engine,
            &session,
            &step(&connected.status.job),
            "bound-inventory",
            connected_at + 1
        ),
        Err(ServiceError::ConfigurationLimit(_))
    ));
    assert_eq!(engine.storage.runtime().read_stamp(&scope).unwrap(), before);
    assert!(engine.storage.control().get(&head).unwrap().is_none());
    assert_eq!(
        installed.read(&engine, &session, connected_at + 2).job,
        connected.status.job
    );
    assert_eq!(
        engine
            .transaction_admission_snapshot()
            .unwrap()
            .logical_bytes,
        0
    );
    drop(engine);
    let engine = installed.reopen(connected_at + 3);
    let session = installed.session(&engine, "bound-reopen", connected_at + 4);
    assert_eq!(engine.storage.runtime().read_stamp(&scope).unwrap(), before);
    assert!(engine.storage.control().get(&head).unwrap().is_none());
    assert_eq!(
        installed.read(&engine, &session, connected_at + 5).job,
        connected.status.job
    );
}

#[test]
fn inventory_interruption_preserves_incomplete_groups_and_resumes_after_reopen() {
    partial_inventory_recovery(false);
}

#[test]
fn inventory_edit_during_publication_rejects_stale_completion_and_allows_replan() {
    partial_inventory_recovery(true);
}

fn partial_inventory_recovery(edit_source: bool) {
    let (installed, engine, session) =
        Installed::with_source(InstallationTargetKind::ExistingProject, None, |root| {
            let source = root.join("src");
            std::fs::create_dir(&source).unwrap();
            for index in 0..2048 {
                std::fs::write(
                    source.join(format!("entry-{index:04}.txt")),
                    format!("source entry {index}\n"),
                )
                .unwrap();
            }
        });
    let started = call(
        &engine,
        &session,
        &installed.start(60_000),
        "interrupt-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "interrupt-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let at = connected.status.job.updated_at_unix_ms + 1;
    let progress_key = format!(
        "server/state/{}/inventory/progress/{}/1",
        engine.instance_id(),
        installed.job
    );
    let head_key = format!("server/state/{}/inventory/head", engine.instance_id());
    let scope_id = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
    std::thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::channel();
        let engine = &engine;
        let session = &session;
        let command = step(&connected.status.job);
        let worker = scope.spawn(move || {
            let authorized = engine
                .begin_invocation(
                    Invocation {
                        context: context("interrupt-inventory"),
                        resource: ResourcePath {
                            segments: vec![rrd_contract::ResourceId::new(
                                rrd_contract::ResourceKind::Instance,
                                engine.instance_id().as_str(),
                            )
                            .unwrap()],
                        },
                        observed_at_unix_ms: at,
                        attempt: 1,
                        request_sha256: digest::sha256_hex(&serde_json::to_vec(&command).unwrap()),
                    },
                    RrdOperation::EstateAdmin,
                    InvocationCredential::Session {
                        session_id: &session.session_id,
                        token: &session.token,
                    },
                )
                .unwrap();
            sender.send(authorized).unwrap();
            call(engine, session, &command, "interrupt-inventory", at)
        });
        let authorized = receiver
            .recv_timeout(std::time::Duration::from_secs(30))
            .unwrap();
        let deadline = std::time::Instant::now();
        while engine
            .storage
            .control()
            .get(&progress_key)
            .unwrap()
            .is_none()
        {
            assert!(
                !worker.is_finished(),
                "Inventory finished before recording progress"
            );
            assert!(deadline.elapsed() < std::time::Duration::from_secs(60));
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if edit_source {
            std::fs::write(
                installed.root.path().join("README.md"),
                "edited during publication\n",
            )
            .unwrap();
            assert!(matches!(
                worker.join().unwrap(),
                Err(ServiceError::AttunementSourceChanged)
            ));
        } else {
            authorized.cancel_pending();
            assert!(matches!(
                worker.join().unwrap(),
                Err(ServiceError::AdmissionCancelled)
            ));
        }
    });
    assert!(engine.storage.control().get(&head_key).unwrap().is_none());
    let partial = engine.storage.runtime().read_stamp(&scope_id).unwrap();
    assert!(
        partial.commit_cursor
            > connected.status.job.checkpoints[0]
                .coordinates
                .runtime_cursor
    );
    assert_eq!(
        installed.read(&engine, &session, at + 30_000).job,
        connected.status.job
    );
    let partial_rows = engine
        .storage
        .runtime()
        .read_changes(&partial, 0, 10_000)
        .unwrap();
    assert!(!partial_rows.changes.iter().any(|change| matches!(&change.mutation,
        RuntimeMutation::Record { record } if record.reference.kind.as_str() == "source-tree-snapshot")));
    let reopened_at = connected
        .status
        .job
        .lease
        .as_ref()
        .unwrap()
        .expires_at_unix_ms
        + 1_000;
    drop(engine);
    let engine = installed.reopen(reopened_at);
    let session = installed.session(&engine, "interrupt-reopen", reopened_at + 100);
    let recovered = call(
        &engine,
        &session,
        &AttunementCommand::RecoverExpired {
            job_id: installed.job.clone(),
            expected_revision: connected.status.job.revision,
        },
        "interrupt-recover",
        reopened_at + 200,
    )
    .unwrap();
    let recovered_at = recovered.status.job.updated_at_unix_ms;
    let resumed = call(
        &engine,
        &session,
        &resume(&recovered.status.job, recovered_at + 1),
        "interrupt-resume",
        recovered_at + 1,
    );
    if edit_source {
        assert!(matches!(
            resumed,
            Err(ServiceError::AttunementSourceChanged)
        ));
        assert!(engine.storage.control().get(&head_key).unwrap().is_none());
        let replan = engine
            .preview_attunement_replan(
                &session.session_id,
                &session.token,
                &installed.job,
                recovered.status.job.revision,
                recovered_at + 2,
                "request-partial-replan",
                "operation-partial-replan",
            )
            .unwrap();
        let accepted = engine
            .accept_attunement_replan(
                &session.session_id,
                &session.token,
                &replan,
                &replan.proposal_sha256,
                &context("partial-accept"),
                recovered_at + 3,
            )
            .unwrap();
        assert_ne!(
            accepted.successor.job.plan_sha256,
            connected.status.job.plan_sha256
        );
        assert!(accepted.successor.job.checkpoints.is_empty());
        assert!(engine.storage.control().get(&head_key).unwrap().is_none());
        return;
    }
    let resumed = resumed.unwrap();
    let complete = call(
        &engine,
        &session,
        &step(&resumed.status.job),
        "interrupt-continue",
        resumed.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(complete.status.job.checkpoints.len(), 2);
    assert!(engine
        .storage
        .control()
        .get(&progress_key)
        .unwrap()
        .is_none());
    assert!(engine.storage.control().get(&head_key).unwrap().is_some());
    let read = engine.storage.runtime().read_stamp(&scope_id).unwrap();
    let changes = engine
        .storage
        .runtime()
        .read_changes(&read, 0, 10_000)
        .unwrap();
    let identities: Vec<_> = changes
        .changes
        .iter()
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Record { record }
                if record.reference.kind.as_str() == "source-tree-entry" =>
            {
                Some(record.reference.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        identities.len(),
        2052, // Root, src, README, the excluded .rrflow marker, and 2,048 files.
        "resume must not republish already committed groups"
    );
    assert_eq!(
        identities
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        identities.len()
    );
}

#[test]
fn git_selected_plan_ignores_irrelevant_edits_replans_tracked_edits_and_reopens() {
    selected_plan_replans_after_progress_and_reopens(true);
}

fn selected_plan_replans_after_progress_and_reopens(git: bool) {
    let configured = |phase, name: &str| crate::AttunementProposalStep {
        phase,
        description: name.into(),
        estimated_items: 1,
        estimated_input_bytes: 0,
        parse_policy: None,
    };
    let proposal = crate::AttunementProposal {
        format_version: crate::ATTUNEMENT_PROPOSAL_FORMAT_VERSION,
        steps: vec![
            configured(Phase::Connect, "first"),
            configured(Phase::Connect, "second"),
            configured(Phase::Verify, "verify"),
        ],
        estimated_min_duration_ms: 0,
        estimated_max_duration_ms: 1000,
    };
    let (installed, engine, session) = Installed::with_source(
        InstallationTargetKind::ExistingProject,
        Some(&proposal),
        |root| {
            if git {
                std::fs::create_dir(root.join(".git")).unwrap();
                std::fs::create_dir(root.join("src")).unwrap();
                std::fs::write(
                    root.join(".git/index"),
                    include_bytes!(
                        "../../../../../compute/rrd-attunement/tests/fixtures/git-index/sha1-v2.index"
                    ),
                )
                .unwrap();
                std::fs::write(root.join(".gitignore"), b"*.log\nsrc/\n").unwrap();
                std::fs::write(root.join("src/main.rs"), b"tracked source\n").unwrap();
            }
        },
    );
    assert_eq!(installed.preview.attunement.phases.len(), 3);
    if git {
        std::fs::write(
            installed.root.path().join("ignored.log"),
            b"irrelevant edit\n",
        )
        .unwrap();
        std::fs::write(
            installed.root.path().join("src/untracked"),
            b"ignored sibling\n",
        )
        .unwrap();
    }
    let running = call(
        &engine,
        &session,
        &installed.start(30_000),
        "public-start",
        BASE + 1_000,
    )
    .unwrap();
    let first = call(
        &engine,
        &session,
        &step(&running.status.job),
        "public-step-1",
        running.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let second = call(
        &engine,
        &session,
        &step(&first.status.job),
        "public-step-2",
        first.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let second_at = second.status.job.updated_at_unix_ms;
    assert_eq!(second.status.job.checkpoints.len(), 2);
    std::fs::write(
        installed
            .root
            .path()
            .join(if git { "src/main.rs" } else { "README.md" }),
        "source changed after progress\n",
    )
    .unwrap();
    assert!(matches!(
        call(
            &engine,
            &session,
            &step(&second.status.job),
            "public-stale-step",
            second_at + 1
        ),
        Err(ServiceError::AttunementSourceChanged)
    ));
    let proposed = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            second.status.job.revision,
            second_at + 2,
            "request-public-preview",
            "operation-public-preview",
        )
        .unwrap();
    assert_ne!(
        proposed.plan.source_sha256,
        installed.preview.attunement.source_sha256
    );
    assert_eq!(proposed.plan.phases, installed.preview.attunement.phases);
    let sequence = engine.storage.control().sequence().unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &proposed,
            &proposed.proposal_sha256,
            &context("public-accept"),
            second_at + 3,
        )
        .unwrap();
    let accepted_at = accepted.predecessor.job.updated_at_unix_ms;
    assert_eq!(
        accepted.predecessor.job.checkpoints,
        second.status.job.checkpoints
    );
    assert!(accepted.successor.job.checkpoints.is_empty());
    let journal = engine
        .storage
        .control()
        .journal_since(sequence, 32)
        .unwrap();
    assert!(journal
        .iter()
        .any(|entry| entry.action == "attunement.replan_receipted"));
    drop(engine);
    let engine = installed.reopen(accepted_at + 1);
    let session = installed.session(&engine, "public-reopen", accepted_at + 2);
    let replay = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &proposed,
            &proposed.proposal_sha256,
            &context("public-accept"),
            accepted_at + 3,
        )
        .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.successor, accepted.successor);
    assert_eq!(replay.predecessor, accepted.predecessor);
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
            // This injected test job is not the installed initial job, so it
            // needs the same generation record as an admitted successor.
            transition(
                format!(
                    "{}/replan-generation",
                    crate::engine::installation::attunement_job_key(engine.instance_id(), &job_id)
                ),
                serde_json::to_vec(&1_u64).unwrap(),
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
        running.status.job.updated_at_unix_ms + 1,
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
        first.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let paused_at = paused.status.job.updated_at_unix_ms;
    drop(engine);
    let engine = installed.reopen(paused_at + 1);
    let renewed = installed.session(&engine, "variable-reopened", paused_at + 2);
    let read = engine
        .read_attunement(
            &renewed.session_id,
            &renewed.token,
            &job_id,
            paused_at + 3,
            "variable-read",
            "variable-read",
        )
        .unwrap();
    assert_eq!(read.job, paused.status.job);
    let running = call(
        &engine,
        &renewed,
        &resume(&read.job, paused_at + 4),
        "variable-resume",
        paused_at + 4,
    )
    .unwrap();
    let second = call(
        &engine,
        &renewed,
        &step(&running.status.job),
        "variable-second",
        running.status.job.updated_at_unix_ms + 1,
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
        second.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(unavailable.status.job.state, State::Failed);
    assert_eq!(unavailable.status.job.checkpoints.len(), 2);
    assert_eq!(
        unavailable
            .status
            .job
            .failure
            .as_ref()
            .unwrap()
            .code
            .as_str(),
        "phase-unavailable"
    );

    // A normal source edit after multiple completed, read-only Connect steps
    // must allow a successor without erasing the old checkpoint evidence.
    std::fs::write(
        installed.root.path().join("README.md"),
        "edited after progress\n",
    )
    .unwrap();
    let preview = engine
        .preview_attunement_replan(
            &renewed.session_id,
            &renewed.token,
            &job_id,
            unavailable.status.job.revision,
            unavailable.status.job.updated_at_unix_ms + 1,
            "request-variable-replan",
            "operation-variable-replan",
        )
        .unwrap();
    assert_ne!(preview.plan.source_sha256, plan.source_sha256);
    assert_eq!(preview.plan.phases, plan.phases);
    let result = engine
        .accept_attunement_replan(
            &renewed.session_id,
            &renewed.token,
            &preview,
            &preview.proposal_sha256,
            &context("variable-replan"),
            unavailable.status.job.updated_at_unix_ms + 2,
        )
        .unwrap();
    let accepted_at = result.predecessor.job.updated_at_unix_ms;
    assert_eq!(
        result.predecessor.job.checkpoints,
        unavailable.status.job.checkpoints
    );
    assert_eq!(result.successor.job.state, State::Pending);
    assert!(result.successor.job.checkpoints.is_empty());
    drop(engine);
    let engine = installed.reopen(accepted_at + 1);
    let renewed = installed.session(&engine, "variable-replan-reopen", accepted_at + 2);
    let replay = engine
        .accept_attunement_replan(
            &renewed.session_id,
            &renewed.token,
            &preview,
            &preview.proposal_sha256,
            &context("variable-replan"),
            accepted_at + 3,
        )
        .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.successor, result.successor);
    assert_eq!(replay.predecessor, result.predecessor);
}

#[test]
fn stale_attunement_can_be_replanned_without_reinstalling_or_losing_receipts() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    std::fs::write(installed.root.path().join("README.md"), "first edit\n").unwrap();
    assert!(matches!(
        call(
            &engine,
            &session,
            &installed.start(30_000),
            "old-start",
            BASE + 1_000
        ),
        Err(ServiceError::AttunementSourceChanged)
    ));
    assert_eq!(
        installed.read(&engine, &session, BASE + 1_100).job.revision,
        1
    );
    assert!(engine
        .read_attunement_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("old-start"),
            BASE + 1_110,
            "request-no-old-start-receipt",
            "operation-no-old-start-receipt"
        )
        .unwrap()
        .is_none());
    let proposal = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            1,
            BASE + 1_200,
            "request-preview",
            "operation-preview",
        )
        .unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &proposal,
            &proposal.proposal_sha256,
            &context("accept"),
            BASE + 1_300,
        )
        .unwrap();
    assert_ne!(accepted.successor.job.id, installed.job);
    assert_ne!(
        accepted.plan.plan_sha256,
        installed.preview.attunement.plan_sha256
    );
    assert_eq!(accepted.predecessor.job.state, State::CancelRequested);
    assert_eq!(accepted.successor.job.state, State::Pending);
    let start = AttunementCommand::Start {
        job_id: accepted.successor.job.id.clone(),
        expected_revision: 1,
        plan_sha256: accepted.plan.plan_sha256.clone(),
        lease_ms: 30_000,
    };
    let running = call(
        &engine,
        &session,
        &start,
        "new-start",
        accepted.successor.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let running_at = running.status.job.updated_at_unix_ms;
    std::fs::write(installed.root.path().join("README.md"), "second edit\n").unwrap();
    let work = step(&running.status.job);
    assert!(matches!(
        call(&engine, &session, &work, "stale-connect", running_at + 1),
        Err(ServiceError::AttunementSourceChanged)
    ));
    let current = engine
        .read_attunement(
            &session.session_id,
            &session.token,
            &accepted.successor.job.id,
            running_at + 2,
            "request-new-read",
            "operation-new-read",
        )
        .unwrap();
    assert!(current.job.checkpoints.is_empty());
    assert!(engine
        .read_attunement_receipt(
            &session.session_id,
            &session.token,
            &accepted.successor.job.id,
            &id("stale-connect"),
            running_at + 3,
            "request-no-stale-receipt",
            "operation-no-stale-receipt"
        )
        .unwrap()
        .is_none());
    drop(engine);
    let engine = installed.reopen(running_at + 4);
    let renewed = installed.session(&engine, "replan-reopened", running_at + 5);
    let replay = engine
        .accept_attunement_replan(
            &renewed.session_id,
            &renewed.token,
            &proposal,
            &proposal.proposal_sha256,
            &context("accept"),
            running_at + 6,
        )
        .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.successor, accepted.successor);
    assert_eq!(replay.predecessor, accepted.predecessor);
}

#[test]
fn replan_rechecks_inputs_and_fences_the_old_worker() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let running = call(
        &engine,
        &session,
        &installed.start(30_000),
        "old-running",
        BASE + 1_000,
    )
    .unwrap();
    let running_at = running.status.job.updated_at_unix_ms;
    let old_step = step(&running.status.job);
    std::fs::write(installed.root.path().join("README.md"), "new source\n").unwrap();
    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            running.status.job.revision,
            running_at + 1,
            "request-preview-1",
            "operation-preview-1",
        )
        .unwrap();
    std::fs::write(installed.root.path().join("README.md"), "changed again\n").unwrap();
    assert!(matches!(
        engine.accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context("stale-accept"),
            running_at + 2
        ),
        Err(ServiceError::AttunementSourceChanged)
    ));
    assert_eq!(
        installed.read(&engine, &session, running_at + 3).job,
        running.status.job
    );
    assert!(matches!(
        engine.preview_attunement_replan(
            &session.session_id,
            &id("wrong-token"),
            &installed.job,
            running.status.job.revision,
            running_at + 4,
            "request-denied",
            "operation-denied"
        ),
        Err(ServiceError::Unauthenticated)
    ));
    let replacement = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            running.status.job.revision,
            running_at + 5,
            "request-preview-2",
            "operation-preview-2",
        )
        .unwrap();
    let mut malformed = replacement.clone();
    malformed.generation += 1;
    assert!(matches!(
        engine.accept_attunement_replan(
            &session.session_id,
            &session.token,
            &malformed,
            &malformed.proposal_sha256,
            &context("invalid-proposal"),
            running_at + 6
        ),
        Err(ServiceError::OperationDigestMismatch)
    ));
    let before_sequence = engine.storage.control().sequence().unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &replacement,
            &replacement.proposal_sha256,
            &context("replacement"),
            running_at + 7,
        )
        .unwrap();
    let accepted_at = accepted.predecessor.job.updated_at_unix_ms;
    let journal = engine
        .storage
        .control()
        .journal_since(before_sequence, 32)
        .unwrap();
    for action in [
        "attunement.replan_fenced",
        "attunement.replan_plan_created",
        "attunement.replan_job_created",
        "attunement.replan_generation",
        "attunement.replan_linked",
        "attunement.replan_receipted",
    ] {
        assert_eq!(
            journal
                .iter()
                .filter(|entry| entry.action == action)
                .count(),
            1
        );
    }
    assert!(matches!(
        call(&engine, &session, &old_step, "old-worker", accepted_at + 1),
        Err(ServiceError::StorageConflict(_))
    ));
    let mut changed = replacement.clone();
    changed.generation += 1;
    assert!(matches!(
        engine.accept_attunement_replan(
            &session.session_id,
            &session.token,
            &changed,
            &changed.proposal_sha256,
            &context("replacement"),
            accepted_at + 2
        ),
        Err(ServiceError::IdempotencyConflict)
    ));
    let cancellation = call(
        &engine,
        &session,
        &step(&accepted.predecessor.job),
        "cancel-old",
        accepted_at + 3,
    )
    .unwrap();
    assert_eq!(cancellation.status.job.state, State::Cancelled);
    assert!(cancellation.status.job.checkpoints.is_empty());
}

#[test]
fn competing_replan_acceptances_publish_one_successor() {
    let (installed, engine, first) = Installed::new(InstallationTargetKind::ExistingProject);
    let second = installed.session(&engine, "replan-competitor", BASE + 200);
    std::fs::write(installed.root.path().join("README.md"), "edited\n").unwrap();
    let preview = engine
        .preview_attunement_replan(
            &first.session_id,
            &first.token,
            &installed.job,
            1,
            BASE + 500,
            "request-compete-preview",
            "operation-compete-preview",
        )
        .unwrap();
    let engine = Arc::new(engine);
    let barrier = Arc::new(Barrier::new(2));
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for (session, name) in [(&first, "accept-first"), (&second, "accept-second")] {
            let engine = Arc::clone(&engine);
            let barrier = Arc::clone(&barrier);
            let preview = preview.clone();
            handles.push(scope.spawn(move || {
                barrier.wait();
                engine.accept_attunement_replan(
                    &session.session_id,
                    &session.token,
                    &preview,
                    &preview.proposal_sha256,
                    &context(name),
                    BASE + 1_000,
                )
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let winner = results.into_iter().find_map(Result::ok).unwrap();
    assert_eq!(winner.predecessor.job.state, State::CancelRequested);
    let successor = engine
        .read_attunement(
            &first.session_id,
            &first.token,
            &winner.successor.job.id,
            winner.successor.job.updated_at_unix_ms + 1,
            "request-winning-job",
            "operation-winning-job",
        )
        .unwrap();
    assert_eq!(successor.job, winner.successor.job);
}

#[test]
fn replan_refuses_an_unreconciled_failure() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "unknown-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "unknown-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let inventoried = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "unknown-failure",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(inventoried.status.job.current_phase, Some(Phase::Parse));
    let failed = call(
        &engine,
        &session,
        &step(&inventoried.status.job),
        "unknown-parse",
        inventoried.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let failure_at = failed.status.job.updated_at_unix_ms;
    assert!(engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            failed.status.job.revision,
            failure_at + 1,
            "request-known-failure",
            "operation-known-failure"
        )
        .is_ok());
    let key = crate::engine::installation::attunement_job_key(
        &installed.preview.attunement.target.instance_id,
        &installed.job,
    );
    let expected = engine.storage.control().get(&key).unwrap();
    let mut unknown = failed.status.job;
    let failure = unknown.failure.as_mut().unwrap();
    failure.code = CanonicalId::new("external-outcome-unknown").unwrap();
    failure.retryable = false;
    unknown.validate(&installed.preview.attunement).unwrap();
    engine
        .storage
        .control()
        .commit(&ControlTransition {
            key,
            expected,
            replacement: Some(serde_json::to_vec(&unknown).unwrap()),
            at: failure_at + 2,
            actor: "test-fixture".into(),
            action: "attunement.fixture_unknown".into(),
            request_id: "request-fixture-unknown".into(),
            operation_id: "operation-fixture-unknown".into(),
        })
        .unwrap();
    assert!(matches!(
        engine.preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            unknown.revision,
            failure_at + 3,
            "request-unknown",
            "operation-unknown"
        ),
        Err(ServiceError::StorageConflict(_))
    ));
    assert_eq!(
        installed.read(&engine, &session, failure_at + 4).job,
        unknown
    );
}

#[test]
fn replan_keeps_historical_control_receipt_after_restart() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "historical-start",
        BASE + 1_000,
    )
    .unwrap();
    let old_step = step(&started.status.job);
    let connected = call(
        &engine,
        &session,
        &old_step,
        "historical-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    std::fs::write(
        installed.root.path().join("README.md"),
        "changed after connect\n",
    )
    .unwrap();
    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            connected.status.job.revision,
            connected.status.job.updated_at_unix_ms + 1,
            "request-historical-preview",
            "operation-historical-preview",
        )
        .unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context("historical-replan"),
            connected.status.job.updated_at_unix_ms + 2,
        )
        .unwrap();
    let accepted_at = accepted.predecessor.job.updated_at_unix_ms;
    assert_eq!(accepted.predecessor.job.state, State::CancelRequested);
    drop(engine);
    let engine = installed.reopen(accepted_at + 1);
    let renewed = installed.session(&engine, "historical-reopened", accepted_at + 2);
    let replay = call(
        &engine,
        &renewed,
        &old_step,
        "historical-connect",
        accepted_at + 3,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.status, connected.status);
    let receipt = engine
        .read_attunement_receipt(
            &renewed.session_id,
            &renewed.token,
            &installed.job,
            &id("historical-connect"),
            accepted_at + 4,
            "request-historical-receipt",
            "operation-historical-receipt",
        )
        .unwrap()
        .unwrap();
    assert_eq!(receipt.status, connected.status);
    assert_eq!(receipt.plan, installed.preview.attunement);
}

#[test]
fn repeated_ordinary_edits_are_not_stopped_by_a_fixed_generation_limit() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let mut current = installed.job.clone();
    let mut at = BASE + 1_000;
    for generation in 1_u64..=33 {
        std::fs::write(
            installed.root.path().join("README.md"),
            format!("edit {generation}\n"),
        )
        .unwrap();
        let preview = engine
            .preview_attunement_replan(
                &session.session_id,
                &session.token,
                &current,
                1,
                at,
                "request-generation-preview",
                "operation-generation-preview",
            )
            .unwrap();
        assert_eq!(preview.generation, generation);
        let key = format!("generation-{generation}");
        let result = engine
            .accept_attunement_replan(
                &session.session_id,
                &session.token,
                &preview,
                &preview.proposal_sha256,
                &context(&key),
                at + 1,
            )
            .unwrap();
        at = result.successor.job.updated_at_unix_ms + 1_000;
        current = result.successor.job.id;
    }
}

#[test]
#[ignore = "native replan lost-response subprocess fixture"]
fn attunement_replan_crash_child() {
    let root = PathBuf::from(std::env::var_os("RRFLOW_REPLAN_TEST_ROOT").unwrap());
    let ready = PathBuf::from(std::env::var_os("RRFLOW_REPLAN_TEST_READY").unwrap());
    let preview_path = PathBuf::from(std::env::var_os("RRFLOW_REPLAN_TEST_PREVIEW").unwrap());
    let preview: crate::AttunementReplanPreview =
        serde_json::from_slice(&std::fs::read(preview_path).unwrap()).unwrap();
    let executable = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let engine =
        RrdEngine::open_installed_with_clock(&root, &executable, &FixedClock::at(BASE + 2_000))
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
            &id("replan-crash-session"),
            BASE + 2_100,
            "request-replan-crash-session",
            "operation-replan-crash-session",
        )
        .unwrap();
    engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context("replan-lost-response"),
            BASE + 2_200,
        )
        .unwrap();
    std::fs::write(ready, "committed").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(30));
    panic!("parent failed to terminate the replan fixture");
}

#[test]
fn replan_lost_response_survives_process_kill_and_replays() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    std::fs::write(
        installed.root.path().join("README.md"),
        "edited before replan\n",
    )
    .unwrap();
    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            1,
            BASE + 1_000,
            "request-crash-preview",
            "operation-crash-preview",
        )
        .unwrap();
    drop(engine);
    let artifacts = tempfile::tempdir().unwrap();
    let preview_path = artifacts.path().join("preview.json");
    let ready = artifacts.path().join("committed");
    let output_path = artifacts.path().join("child.log");
    std::fs::write(&preview_path, serde_json::to_vec(&preview).unwrap()).unwrap();
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
                "engine::tests::attunement::attunement_replan_crash_child",
                "--ignored",
                "--nocapture",
            ])
            .env("RRFLOW_REPLAN_TEST_ROOT", installed.root.path())
            .env("RRFLOW_REPLAN_TEST_READY", &ready)
            .env("RRFLOW_REPLAN_TEST_PREVIEW", &preview_path)
            .stdout(output.try_clone().unwrap())
            .stderr(output)
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !ready.exists() {
        assert!(
            child.0.try_wait().unwrap().is_none() && std::time::Instant::now() < deadline,
            "child failed before replan commit: {}",
            std::fs::read_to_string(&output_path).unwrap()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let engine = installed.reopen(BASE + 4_000);
    let renewed = installed.session(&engine, "replan-after-kill", BASE + 4_100);
    let replay = engine
        .accept_attunement_replan(
            &renewed.session_id,
            &renewed.token,
            &preview,
            &preview.proposal_sha256,
            &context("replan-lost-response"),
            BASE + 4_200,
        )
        .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.predecessor.job.state, State::CancelRequested);
    // The killed child committed this job, so its completion first returns here.
    let successor = engine
        .read_attunement(
            &renewed.session_id,
            &renewed.token,
            &replay.successor.job.id,
            (BASE + 4_300).max(replay.successor.job.updated_at_unix_ms + 1),
            "request-after-kill",
            "operation-after-kill",
        )
        .unwrap();
    assert_eq!(successor.job, replay.successor.job);
}

#[test]
fn replan_receipt_read_is_principal_scoped_and_does_not_replay() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    std::fs::write(installed.root.path().join("README.md"), "receipt source\n").unwrap();
    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            1,
            BASE + 1_000,
            "request-replan-read-preview",
            "operation-replan-read-preview",
        )
        .unwrap();
    assert!(engine
        .read_attunement_replan_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("read-replan"),
            BASE + 1_100,
            "request-missing-replan",
            "operation-missing-replan"
        )
        .unwrap()
        .is_none());
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context("read-replan"),
            BASE + 1_200,
        )
        .unwrap();
    // Effects include real elapsed work. Continue from their committed time,
    // rather than assuming the previous call finished within a fixed window.
    let receipt_at = accepted.predecessor.job.updated_at_unix_ms + 1;
    let before = engine.storage.control().journal_since(0, 1024).unwrap();
    let receipt = engine
        .read_attunement_replan_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("read-replan"),
            receipt_at,
            "request-read-replan",
            "operation-read-replan",
        )
        .unwrap()
        .unwrap();
    assert_eq!(receipt.preview, preview);
    assert_eq!(receipt.predecessor_plan, installed.preview.attunement);
    assert_eq!(receipt.result, accepted);
    assert!(engine
        .read_attunement_replan_receipt(
            &session.session_id,
            &id("bad-token"),
            &installed.job,
            &id("read-replan"),
            receipt_at + 1,
            "request-bad-replan",
            "operation-bad-replan"
        )
        .is_err());
    let after = engine.storage.control().journal_since(0, 1024).unwrap();
    let count = |rows: &[rrd_store::ControlJournalEntry]| {
        rows.iter()
            .filter(|row| row.action.starts_with("attunement.replan_"))
            .count()
    };
    assert_eq!(count(&before), count(&after));
    assert_eq!(
        installed.read(&engine, &session, receipt_at + 2).job,
        accepted.predecessor.job
    );
    let successor_start = AttunementCommand::Start {
        job_id: accepted.successor.job.id.clone(),
        plan_sha256: accepted.plan.plan_sha256.clone(),
        expected_revision: 1,
        lease_ms: 30_000,
    };
    let successor = call(
        &engine,
        &session,
        &successor_start,
        "receipt-successor-start",
        receipt_at + 3,
    )
    .unwrap();
    std::fs::write(installed.root.path().join("README.md"), "later edit\n").unwrap();
    let later_at = successor.status.job.updated_at_unix_ms + 1;
    let later = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &successor.status.job.id,
            successor.status.job.revision,
            later_at,
            "request-later-preview",
            "operation-later-preview",
        )
        .unwrap();
    let later_accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &later,
            &later.proposal_sha256,
            &context("later-replan"),
            later_at + 1,
        )
        .unwrap();
    let historical_at = later_accepted.predecessor.job.updated_at_unix_ms + 1;
    let historical = engine
        .read_attunement_replan_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("read-replan"),
            historical_at,
            "request-historical-replan-read",
            "operation-historical-replan-read",
        )
        .unwrap()
        .unwrap();
    assert_eq!(historical, receipt);
    drop(engine);
    let engine = installed.reopen(historical_at + 1);
    let renewed = installed.session(&engine, "replan-read-reopen", historical_at + 2);
    let reopened = engine
        .read_attunement_replan_receipt(
            &renewed.session_id,
            &renewed.token,
            &installed.job,
            &id("read-replan"),
            historical_at + 3,
            "request-replan-reopen",
            "operation-replan-reopen",
        )
        .unwrap()
        .unwrap();
    assert_eq!(reopened, receipt);
}

#[test]
fn replan_receipt_and_replay_reject_a_corrupt_successor_link() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            1,
            BASE + 1_000,
            "request-corrupt-preview",
            "operation-corrupt-preview",
        )
        .unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context("corrupt-replan"),
            BASE + 1_200,
        )
        .unwrap();
    let accepted_at = accepted.predecessor.job.updated_at_unix_ms;
    let link = format!(
        "{}/successor",
        crate::engine::installation::attunement_job_key(engine.instance_id(), &installed.job)
    );
    let expected = engine.storage.control().get(&link).unwrap();
    let mut changed = preview.clone();
    changed.generation += 1;
    engine
        .storage
        .control()
        .commit(&ControlTransition {
            key: link,
            expected,
            replacement: Some(serde_json::to_vec(&changed).unwrap()),
            at: accepted_at + 1,
            actor: "test-fixture".into(),
            action: "attunement.fixture_corrupt_link".into(),
            request_id: "request-corrupt-link".into(),
            operation_id: "operation-corrupt-link".into(),
        })
        .unwrap();
    assert!(engine
        .read_attunement_replan_receipt(
            &session.session_id,
            &session.token,
            &installed.job,
            &id("corrupt-replan"),
            accepted_at + 2,
            "request-corrupt-read",
            "operation-corrupt-read"
        )
        .is_err());
    assert!(engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context("corrupt-replan"),
            accepted_at + 3
        )
        .is_err());
}

#[test]
fn replan_receipt_is_not_visible_to_another_authorized_principal() {
    let (installed, engine, owner) = Installed::new(InstallationTargetKind::ExistingProject);
    let preview = engine
        .preview_attunement_replan(
            &owner.session_id,
            &owner.token,
            &installed.job,
            1,
            BASE + 1_000,
            "request-private-preview",
            "operation-private-preview",
        )
        .unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &owner.session_id,
            &owner.token,
            &preview,
            &preview.proposal_sha256,
            &context("private-replan"),
            BASE + 1_200,
        )
        .unwrap();
    let accepted_at = accepted.predecessor.job.updated_at_unix_ms;
    let repository = SecurityRepository::new(&engine.storage, engine.instance_id().clone());
    let mut authority = repository.load().unwrap().unwrap();
    let revision = authority.revision;
    authority.revision += 1;
    let principal_id = CanonicalId::new("other-replan-reader").unwrap();
    let resource = ResourcePath {
        segments: vec![
            ResourceId::new(ResourceKind::Instance, engine.instance_id().as_str()).unwrap(),
        ],
    };
    authority.principals.insert(
        principal_id.clone(),
        Principal {
            id: principal_id.clone(),
            kind: PrincipalKind::Service,
            credential_sha256: digest::sha256_hex(b"other-replan-reader"),
            credential_revision: 1,
            not_before_unix_ms: 1,
            expires_at_unix_ms: u64::MAX,
            disabled: false,
            role_ids: Default::default(),
            grants: [SecurityAction::SessionCreate, SecurityAction::EstateRead]
                .into_iter()
                .map(|action| ResourceGrant {
                    action,
                    resource_prefix: resource.clone(),
                    data_policy: None,
                })
                .collect(),
        },
    );
    repository
        .replace(
            revision,
            authority,
            accepted_at + 1,
            "test",
            "reader-update",
            "reader-update",
        )
        .unwrap();
    let reader = engine
        .create_authenticated_session(
            &principal_id,
            b"other-replan-reader",
            &CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 120_000,
                    absolute_timeout_ms: 120_000,
                    max_open_transactions: 2,
                },
            },
            &id("other-replan-session"),
            accepted_at + 2,
            "request-other-replan-session",
            "operation-other-replan-session",
        )
        .unwrap();
    assert!(engine
        .read_attunement_replan_receipt(
            &reader.session_id,
            &reader.token,
            &installed.job,
            &id("private-replan"),
            accepted_at + 3,
            "request-foreign-replan",
            "operation-foreign-replan"
        )
        .unwrap()
        .is_none());
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
    let connected = call(
        &engine,
        &session,
        &command,
        "connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let inventoried = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "inventory",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(inventoried.status.job.current_phase, Some(Phase::Parse));
    let failed = call(
        &engine,
        &session,
        &step(&inventoried.status.job),
        "receipt-parse",
        inventoried.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(failed.status.job.state, State::Failed);
    let failure_at = failed.status.job.updated_at_unix_ms;
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
            failure_at + 1,
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
            failure_at + 2,
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
            failure_at + 3,
            "receipt-foreign",
            "receipt-foreign",
        )
        .unwrap()
        .is_none());
    let after = engine.storage.control().journal_since(0, 1024).unwrap();
    assert_eq!(count_receipts(&before), count_receipts(&after));
    assert_eq!(
        installed.read(&engine, &session, failure_at + 4).job,
        failed.status.job
    );
    drop(engine);
    let engine = installed.reopen(failure_at + 5);
    let renewed = installed.session(&engine, "receipt-renewed", failure_at + 6);
    let reopened = engine
        .read_attunement_receipt(
            &renewed.session_id,
            &renewed.token,
            &installed.job,
            &id("connect"),
            failure_at + 7,
            "receipt-reopened",
            "receipt-reopened",
        )
        .unwrap()
        .unwrap();
    assert_eq!(reopened, receipt);
    assert_eq!(
        installed.read(&engine, &renewed, failure_at + 8).job,
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
    let started = call(
        &engine,
        &owner,
        &installed.start(30_000),
        "owner-receipt",
        BASE + 1_000,
    )
    .unwrap();
    let started_at = started.status.job.updated_at_unix_ms;
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
                started_at + 1,
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
            started_at + 2,
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
            started_at + 3,
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
            started_at + 4,
            "owner-read",
            "owner-read"
        )
        .unwrap()
        .is_some());
}

#[test]
fn attunement_receipt_readback_rejects_corrupt_identity() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::FreshProject);
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "receipt-corrupt",
        BASE + 1_000,
    )
    .unwrap();
    let started_at = started.status.job.updated_at_unix_ms;
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
            at: started_at + 1,
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
            started_at + 2,
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
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let inventoried = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "detail-inventory",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(inventoried.status.job.current_phase, Some(Phase::Parse));
    let failed = call(
        &engine,
        &session,
        &step(&inventoried.status.job),
        "detail-parse",
        inventoried.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let failure_at = failed.status.job.updated_at_unix_ms;
    std::fs::write(installed.root.path().join("README.md"), "changed source\n").unwrap();
    let read = engine
        .read_attunement_detail(
            &session.session_id,
            &session.token,
            &request,
            failure_at + 1,
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
            failure_at + 2,
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
        let connected = call(
            &engine,
            &session,
            &work,
            "connect",
            start.status.job.updated_at_unix_ms + 1,
        )
        .unwrap();
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
        let inventoried = call(
            &engine,
            &session,
            &step(&connected.status.job),
            "inventory",
            connected.status.job.updated_at_unix_ms + 1,
        )
        .unwrap();
        assert_eq!(inventoried.status.job.current_phase, Some(Phase::Parse));
        let failed = call(
            &engine,
            &session,
            &step(&inventoried.status.job),
            "parse",
            inventoried.status.job.updated_at_unix_ms + 1,
        )
        .unwrap();
        assert_eq!(failed.status.job.state, State::Failed);
        assert_eq!(
            failed.status.job.failure.as_ref().unwrap().code.as_str(),
            "phase-unavailable"
        );
        assert_eq!(
            failed.status.job.checkpoints,
            inventoried.status.job.checkpoints
        );
        assert!(failed.status.job.lease.is_none());
        let failure_at = failed.status.job.updated_at_unix_ms;
        drop(engine);
        let engine = installed.reopen(failure_at + 1);
        let renewed = installed.session(&engine, "second-session", failure_at + 2);
        let replay = call(&engine, &renewed, &work, "connect", failure_at + 3).unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(
            replay.status, connected.status,
            "replay returns the original receipt even after later failure"
        );
        assert_eq!(
            installed.read(&engine, &renewed, failure_at + 4).job,
            failed.status.job
        );
        let resumed = call(
            &engine,
            &renewed,
            &resume(&failed.status.job, failure_at + 5),
            "resume",
            failure_at + 5,
        )
        .unwrap();
        assert_eq!(resumed.status.job.current_phase, Some(Phase::Parse));
        assert!(
            resumed.status.job.lease.as_ref().unwrap().generation
                > start.status.job.lease.as_ref().unwrap().generation
        );
        let failed_again = call(
            &engine,
            &renewed,
            &step(&resumed.status.job),
            "parse-again",
            resumed.status.job.updated_at_unix_ms + 1,
        )
        .unwrap();
        assert_eq!(
            failed_again.status.job.checkpoints.len(),
            2,
            "unsupported Parse must not fabricate a checkpoint"
        );
        drop(engine);
        let report = RrdEngine::inspect_installed_with_clock(
            installed.root.path(),
            &installed.executable,
            &FixedClock::at(failed_again.status.job.updated_at_unix_ms + 1),
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
    let claimed_at = results
        .iter()
        .find_map(|result| result.as_ref().ok())
        .unwrap()
        .status
        .job
        .updated_at_unix_ms;
    let running = installed.read(&engine, winner, claimed_at + 1).job;
    assert!(matches!(
        call(
            &engine,
            loser,
            &step(&running),
            "foreign-step",
            claimed_at + 2
        ),
        Err(ServiceError::StorageConflict(_))
    ));
    assert_eq!(installed.read(&engine, winner, claimed_at + 3).job, running);
    assert!(call(
        &engine,
        winner,
        &step(&running),
        "winner-step",
        claimed_at + 4
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
    let started_at = started.status.job.updated_at_unix_ms;
    let expires_at = started
        .status
        .job
        .lease
        .as_ref()
        .unwrap()
        .expires_at_unix_ms;
    assert!(call(
        &engine,
        &second,
        &recovery,
        "early-recovery",
        started_at + 1
    )
    .is_err());
    assert!(call(&engine, &first, &old_work, "expired-step", expires_at + 1).is_err());
    let paused = call(&engine, &second, &recovery, "recovery", expires_at + 2).unwrap();
    assert_eq!(paused.status.job.state, State::Paused);
    assert!(paused.status.job.checkpoints.is_empty());
    let paused_at = paused.status.job.updated_at_unix_ms;
    drop(engine);
    let engine = installed.reopen(paused_at + 1);
    let resumed = call(
        &engine,
        &second,
        &resume(&paused.status.job, paused_at + 2),
        "resume",
        paused_at + 2,
    )
    .unwrap();
    let resumed_at = resumed.status.job.updated_at_unix_ms;
    assert!(call(&engine, &first, &old_work, "zombie", resumed_at + 1).is_err());
    let mut forged = step(&resumed.status.job);
    if let AttunementCommand::Step {
        lease_generation, ..
    } = &mut forged
    {
        *lease_generation = 2;
    }
    assert!(call(&engine, &second, &forged, "old-generation", resumed_at + 2).is_err());
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
        resumed_at + 3,
    )
    .unwrap();
    assert_eq!(paused.status.job.state, State::Paused);
    assert!(call(
        &engine,
        &second,
        &step(&resumed.status.job),
        "paused-worker",
        paused.status.job.updated_at_unix_ms + 1
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
    let running_at = running.status.job.updated_at_unix_ms;
    let old_work = step(&running.status.job);
    let command = cancel(&running.status.job, running_at + 1);
    let requested = call(&engine, &session, &command, "cancel", running_at + 1).unwrap();
    let requested_at = requested.status.job.updated_at_unix_ms;
    assert_eq!(requested.status.job.state, State::CancelRequested);
    assert!(call(
        &engine,
        &session,
        &old_work,
        "stale-after-cancel",
        requested_at + 1
    )
    .is_err());
    drop(engine);
    let engine = installed.reopen(requested_at + 2);
    let complete = call(
        &engine,
        &session,
        &step(&requested.status.job),
        "ack-cancel",
        requested_at + 3,
    )
    .unwrap();
    let complete_at = complete.status.job.updated_at_unix_ms;
    assert_eq!(complete.status.job.state, State::Cancelled);
    assert!(complete.status.job.lease.is_none());
    assert!(complete.status.job.checkpoints.is_empty());
    let replay = call(&engine, &session, &command, "cancel", complete_at + 1).unwrap();
    assert_eq!(replay.status, requested.status);
    assert!(replay.idempotent_replay);
    assert_eq!(
        installed.read(&engine, &session, complete_at + 2).job,
        complete.status.job
    );
    assert!(call(
        &engine,
        &session,
        &installed.start(30_000),
        "restart-cancelled",
        complete_at + 3
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
    let accepted = call(
        &engine,
        &session,
        &installed.start(30_000),
        "accepted-key",
        BASE + 4_000,
    )
    .unwrap();
    let accepted_at = accepted.status.job.updated_at_unix_ms;
    assert!(matches!(
        call(
            &engine,
            &session,
            &installed.start(29_000),
            "accepted-key",
            accepted_at + 1
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
                accepted_at + 2
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
    let running_at = running.status.job.updated_at_unix_ms;
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
            at: running_at + 1,
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
            running_at + 2
        ),
        Err(ServiceError::ConfigurationLimit(_))
    ));
    let requested = call(
        &engine,
        &session,
        &cancel(&exhausted, running_at + 3),
        "cancel-limit",
        running_at + 3,
    )
    .unwrap();
    let cancelled = call(
        &engine,
        &session,
        &step(&requested.status.job),
        "ack-limit",
        requested.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(cancelled.status.job.state, State::Cancelled);
    assert_eq!(cancelled.status.job.revision, 1026);
    assert!(call(
        &engine,
        &session,
        &cancel(
            &cancelled.status.job,
            cancelled.status.job.updated_at_unix_ms + 1
        ),
        "cancel-terminal",
        cancelled.status.job.updated_at_unix_ms + 1
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
    // The killed child claimed this lease, so its expiry first returns in this read.
    let recover_at = (BASE + 43_000).max(running.lease.as_ref().unwrap().expires_at_unix_ms + 1);
    let paused = call(
        &engine,
        &session,
        &AttunementCommand::RecoverExpired {
            job_id: installed.job.clone(),
            expected_revision: 2,
        },
        "recover-kill",
        recover_at,
    )
    .unwrap();
    let running = call(
        &engine,
        &session,
        &resume(&paused.status.job, paused.status.job.updated_at_unix_ms + 1),
        "resume-kill",
        paused.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&running.status.job),
        "connect-after-kill",
        running.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(connected.status.job.checkpoints.len(), 1);
    assert_eq!(connected.status.job.current_phase, Some(Phase::Inventory));
    let replay = call(
        &engine,
        &session,
        &step(&running.status.job),
        "connect-after-kill",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.status, connected.status);
}
