//! The default plan is Connect and Inventory; after the Inventory checkpoint
//! the engine derives the successor plan from the committed inventory
//! evidence and the executors it has. P1 of the D-02 evidence-driven
//! attunement plan.

use super::*;
use rrd_contract::{derive_attunement_plan, CanonicalId, ListAttunementJobs};

fn language(name: &str) -> CanonicalId {
    CanonicalId::new(name).unwrap()
}

#[test]
fn default_plan_completes_after_inventory_and_derives_the_successor_from_evidence() {
    let (installed, engine, session) =
        Installed::with_source(InstallationTargetKind::ExistingProject, None, |root| {
            std::fs::create_dir_all(root.join("src")).unwrap();
            std::fs::write(root.join("src/lib.rs"), "pub fn answer() -> u32 { 42 }\n").unwrap();
            std::fs::write(root.join("src/tool.py"), "print(42)\n").unwrap();
            std::fs::write(root.join("notes.txt"), "no grammar\n").unwrap();
        });
    let planned: Vec<Phase> = installed
        .preview
        .attunement
        .phases
        .iter()
        .map(|step| step.phase)
        .collect();
    assert_eq!(
        planned,
        vec![Phase::Connect, Phase::Inventory],
        "the bundled profile plans only the two observing effects"
    );

    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "d-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "d-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let inventoried = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "d-inventory",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(inventoried.status.job.state, State::Succeeded);
    assert_eq!(inventoried.status.job.checkpoints.len(), 2);
    assert!(
        inventoried.status.job.failure.is_none(),
        "no phase is ever unavailable by default"
    );
    let at = inventoried.status.job.updated_at_unix_ms;

    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &installed.job,
            inventoried.status.job.revision,
            at + 1,
            "request-d-preview",
            "operation-d-preview",
        )
        .unwrap();
    let evidence = preview
        .evidence
        .as_ref()
        .expect("a committed Inventory yields evidence");
    assert_eq!(
        evidence.inventory_sha256,
        inventoried.status.job.checkpoints[1].output_sha256
    );
    assert_eq!(
        evidence.files, 4,
        "lib.rs, tool.py, notes.txt and README.md"
    );
    assert_eq!(evidence.languages.get(&language("rust")), Some(&1));
    assert_eq!(evidence.languages.get(&language("python")), Some(&1));
    assert_eq!(evidence.languages.get(&language("markdown")), Some(&1));
    assert!(!evidence.languages.contains_key(&language("text")));
    assert_eq!(
        evidence
            .bound_grammars
            .iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>(),
        vec!["python", "rust"]
    );
    assert!(!evidence.embedding_binding);
    assert_eq!(
        evidence.available.iter().copied().collect::<Vec<_>>(),
        vec![Phase::Connect, Phase::Inventory, Phase::Parse],
        "only installed executors are available"
    );
    let derived: Vec<Phase> = preview.plan.phases.iter().map(|step| step.phase).collect();
    assert_eq!(
        derived,
        vec![Phase::Connect, Phase::Inventory, Phase::Parse],
        "the Rust and Python sources have bound grammars, so Parse is selected"
    );
    let policy = preview.plan.phases[2]
        .parse_policy
        .as_ref()
        .expect("a derived Parse step carries its policy");
    assert_eq!(
        policy
            .bindings
            .iter()
            .map(|binding| binding.extension.as_str())
            .collect::<Vec<_>>(),
        vec!["py", "pyi", "rs"]
    );
    // Any client can recompute the plan from the evidence the preview carries.
    let recomputed = derive_attunement_plan(
        &installed.preview.attunement,
        evidence,
        preview.plan.id.clone(),
        preview.plan.source_sha256.clone(),
    )
    .unwrap();
    assert_eq!(recomputed, preview.plan);
    assert_ne!(
        preview.plan.phases[0].configuration_sha256,
        installed.preview.attunement.phases[0].configuration_sha256,
        "derived steps bind the evidence digest"
    );

    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context("d-accept"),
            at + 2,
        )
        .unwrap();
    assert_eq!(accepted.predecessor.job.state, State::Succeeded);
    assert_eq!(accepted.successor.job.state, State::Pending);

    // Discovery lists the succeeded predecessor and its derived successor.
    let page = engine
        .list_attunement_jobs(
            &session.session_id,
            &session.token,
            &ListAttunementJobs {
                contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                after_job_id: None,
                limit: 8,
            },
            at + 10_000,
            "request-d-list",
            "operation-d-list",
        )
        .unwrap();
    let ids: Vec<&str> = page.jobs.iter().map(|job| job.job_id.as_str()).collect();
    assert_eq!(
        ids,
        vec![installed.job.as_str(), accepted.successor.job.id.as_str()]
    );

    // The successor runs Connect, Inventory and Parse to completion and the
    // receipts survive a reopen.
    let successor = accepted.successor.job.clone();
    let s_started = call(
        &engine,
        &session,
        &AttunementCommand::Start {
            job_id: successor.id.clone(),
            plan_sha256: successor.plan_sha256.clone(),
            expected_revision: successor.revision,
            lease_ms: 30_000,
        },
        "d-successor-start",
        at + 10_001,
    )
    .unwrap();
    let s_connected = call(
        &engine,
        &session,
        &step(&s_started.status.job),
        "d-successor-connect",
        s_started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let s_inventoried = call(
        &engine,
        &session,
        &step(&s_connected.status.job),
        "d-successor-inventory",
        s_connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(s_inventoried.status.job.current_phase, Some(Phase::Parse));
    let s_done = call(
        &engine,
        &session,
        &step(&s_inventoried.status.job),
        "d-successor-parse",
        s_inventoried.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert_eq!(s_done.status.job.state, State::Succeeded);
    assert_eq!(
        s_done.status.job.checkpoints.last().map(|c| c.phase),
        Some(Phase::Parse)
    );
    drop(engine);
    let late = s_done.status.job.updated_at_unix_ms + 1_000;
    let engine = installed.reopen(late);
    let renewed = installed.session(&engine, "d-reopened", late + 1);
    let again = engine
        .read_attunement(
            &renewed.session_id,
            &renewed.token,
            &successor.id,
            late + 2,
            "request-d-read",
            "operation-d-read",
        )
        .unwrap();
    assert_eq!(again.job, s_done.status.job);
}
