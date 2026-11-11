//! The Parse Step on an installed project: facts are committed with the job's
//! transition, read back through rrflowQL, survive a restart, and a batch limit
//! spreads the work over several Steps.

use super::*;
use std::collections::BTreeSet;

fn policy(max_files: u64) -> rrd_contract::SourceParsePolicy {
    rrd_contract::SourceParsePolicy {
        format_version: 1,
        bindings: vec![rrd_contract::SourceParseBinding {
            extension: "rs".into(),
            language: CanonicalId::new("rust").unwrap(),
        }],
        max_files,
        max_file_bytes: 1_048_576,
        max_input_bytes: 4_194_304,
        max_nodes: 10_000,
        max_depth: 64,
        max_output_bytes: 1_048_576,
        max_commit_bytes: 262_144,
        max_elapsed_ms: 10_000,
        max_memory_bytes: 268_435_456,
    }
}

/// An installed project with two Rust files, its job stepped to Parse.
fn at_parse(max_files: u64) -> (Installed, RrdEngine, SessionLease, AttunementJob) {
    let policy = policy(max_files);
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
    let (installed, engine, session) = Installed::with_source(
        InstallationTargetKind::ExistingProject,
        Some(&proposal),
        |root| {
            std::fs::write(root.join("alpha.rs"), b"fn alpha() {}\n").unwrap();
            std::fs::write(root.join("beta.rs"), b"fn beta() {\n    alpha();\n}\n").unwrap();
        },
    );
    let started = call(
        &engine,
        &session,
        &installed.start(60_000),
        "parse-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "parse-connect",
        BASE + 2_000,
    )
    .unwrap();
    let inventoried = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "parse-inventory",
        BASE + 3_000,
    )
    .unwrap();
    assert_eq!(inventoried.status.job.current_phase, Some(Phase::Parse));
    (installed, engine, session, inventoried.status.job)
}

/// Every `(name, role)` of the `source-symbol` records known now.
fn symbols(engine: &RrdEngine, session: &SessionLease, at: u64) -> BTreeSet<(String, String)> {
    let result = engine
        .execute_query(
            &session.session_id,
            &session.token,
            &rrd_contract::ExecuteQuery {
                scope: format!("instance:{}", engine.instance_id()),
                query: format!(
                    "FROM record:source-symbol AT VALID {at} KNOWN HEAD PROJECT name, role"
                ),
                parameters: Default::default(),
                budget: rrd_contract::QueryBudget::default(),
            },
            at,
            "request-symbol-query",
            "operation-symbol-query",
        )
        .unwrap();
    result
        .rows
        .iter()
        .map(|row| {
            let text = |key: &str| match row.values.get(key) {
                Some(rrd_contract::QueryValue::String(value)) => value.clone(),
                other => panic!("{key} is {other:?}"),
            };
            (text("name"), text("role"))
        })
        .collect()
}

fn expected() -> BTreeSet<(String, String)> {
    [
        ("alpha", "definition"),
        ("beta", "definition"),
        ("alpha", "reference"),
    ]
    .into_iter()
    .map(|(name, role)| (name.into(), role.into()))
    .collect()
}

#[test]
fn parse_commits_facts_with_its_checkpoint_and_they_survive_a_restart() {
    let (installed, engine, session, job) = at_parse(64);
    let parsed = call(&engine, &session, &step(&job), "parse-step", BASE + 4_000).unwrap();
    let job = parsed.status.job;
    let checkpoint = job.checkpoints.last().unwrap();
    assert_eq!(checkpoint.phase, Phase::Parse);
    assert_eq!(job.current_phase, Some(Phase::Verify));
    assert_eq!(job.state, State::Running);
    assert!(symbols(&engine, &session, BASE + 5_000).is_superset(&expected()));

    drop(engine);
    let engine = installed.reopen(BASE + 6_000);
    let session = installed.session(&engine, "parse-reopened", BASE + 6_100);
    assert!(symbols(&engine, &session, BASE + 6_200).is_superset(&expected()));
}

#[test]
fn a_file_limit_spreads_parse_over_steps_without_a_checkpoint_until_done() {
    let (_installed, engine, session, job) = at_parse(1);
    let first = call(&engine, &session, &step(&job), "parse-first", BASE + 4_000).unwrap();
    let job = first.status.job;
    assert_eq!(
        job.current_phase,
        Some(Phase::Parse),
        "one file is still waiting"
    );
    assert_eq!(job.checkpoints.last().unwrap().phase, Phase::Inventory);
    assert_eq!(symbols(&engine, &session, BASE + 4_500).len(), 1);

    let second = call(&engine, &session, &step(&job), "parse-second", BASE + 5_000).unwrap();
    let job = second.status.job;
    assert_eq!(job.checkpoints.last().unwrap().phase, Phase::Parse);
    assert_eq!(job.current_phase, Some(Phase::Verify));
    assert!(symbols(&engine, &session, BASE + 5_500).is_superset(&expected()));
}
