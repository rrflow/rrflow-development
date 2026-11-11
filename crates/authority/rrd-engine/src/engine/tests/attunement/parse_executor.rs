//! Parse publishes readable facts, resumes across Steps and re-parses only what
//! changed. The in-process parser stands in for the supervised worker here.

use super::*;
use rrd_contract::{
    ChangeMutationSnapshot, ExecuteQuery, QueryBudget, QueryValue, ReadChangefeed,
    SourceParseBinding, SourceParsePolicy, TransactionMutation,
};

fn policy(max_commit_bytes: u64) -> SourceParsePolicy {
    SourceParsePolicy {
        format_version: 1,
        bindings: vec![SourceParseBinding {
            extension: "rs".into(),
            language: CanonicalId::new("rust").unwrap(),
        }],
        max_files: 64,
        max_file_bytes: 1_048_576,
        max_input_bytes: 4_194_304,
        max_nodes: 100_000,
        max_depth: 256,
        max_output_bytes: 4_194_304,
        max_commit_bytes,
        max_elapsed_ms: 10_000,
        max_memory_bytes: 8_388_608,
    }
}

fn proposal(max_commit_bytes: u64) -> crate::AttunementProposal {
    crate::AttunementProposal {
        format_version: crate::ATTUNEMENT_PROPOSAL_FORMAT_VERSION,
        steps: [Phase::Connect, Phase::Inventory, Phase::Parse]
            .into_iter()
            .map(|phase| crate::AttunementProposalStep {
                phase,
                description: format!("{phase:?}"),
                estimated_items: 0,
                estimated_input_bytes: 0,
                parse_policy: (phase == Phase::Parse).then(|| policy(max_commit_bytes)),
            })
            .collect(),
        estimated_min_duration_ms: 0,
        estimated_max_duration_ms: 30_000,
    }
}

fn rows(
    engine: &RrdEngine,
    session: &SessionLease,
    query: &str,
    at: u64,
) -> Vec<rrd_contract::QueryRowSnapshot> {
    engine
        .execute_query(
            &session.session_id,
            &session.token,
            &ExecuteQuery {
                scope: format!("instance:{}", engine.instance_id()),
                query: query.to_owned(),
                parameters: Default::default(),
                budget: QueryBudget::default(),
            },
            at,
            "request-parse-query",
            "operation-parse-query",
        )
        .unwrap()
        .rows
}

fn text(row: &rrd_contract::QueryRowSnapshot, column: &str) -> String {
    match row.values.get(column) {
        Some(QueryValue::String(value)) => value.clone(),
        other => panic!("{column} was {other:?}"),
    }
}

/// Runs a job to the end of its plan, returning every job snapshot.
fn run_job(
    engine: &RrdEngine,
    session: &SessionLease,
    mut job: AttunementJob,
    label: &str,
) -> Vec<AttunementJob> {
    let mut history = Vec::new();
    for attempt in 0..64 {
        if job.state != State::Running {
            return history;
        }
        job = call(
            engine,
            session,
            &step(&job),
            &format!("{label}-{attempt}"),
            job.updated_at_unix_ms + 1,
        )
        .unwrap()
        .status
        .job;
        history.push(job.clone());
    }
    panic!("the job did not finish");
}

fn started(
    max_commit_bytes: u64,
    setup: impl FnOnce(&Path),
) -> (Installed, RrdEngine, SessionLease, AttunementJob) {
    let (installed, engine, session) = Installed::with_source(
        InstallationTargetKind::ExistingProject,
        Some(&proposal(max_commit_bytes)),
        setup,
    );
    let job = call(
        &engine,
        &session,
        &installed.start(30_000),
        "parse-start",
        BASE + 1_000,
    )
    .unwrap()
    .status
    .job;
    (installed, engine, session, job)
}

const SOURCE: &str = "pub struct Engine;\n\nfn helper() {}\n\nfn start() {\n    helper();\n}\n";

#[test]
fn parsed_facts_are_readable_through_the_query_language() {
    let (_installed, engine, session, job) = started(4_194_304, |root| {
        std::fs::write(root.join("engine.rs"), SOURCE).unwrap();
        std::fs::write(root.join("notes.txt"), "not parsed\n").unwrap();
    });
    let history = run_job(&engine, &session, job, "facts");
    let done = history.last().unwrap();
    assert_eq!(done.state, State::Succeeded);
    assert_eq!(done.checkpoints.last().unwrap().phase, Phase::Parse);
    let at = done.updated_at_unix_ms + 10;

    let symbols = rows(
        &engine,
        &session,
        &format!("FROM record:source-symbol AT VALID {at} KNOWN HEAD WHERE role = \"definition\" PROJECT name, symbol_kind, path"),
        at,
    );
    let mut names: Vec<_> = symbols.iter().map(|row| text(row, "name")).collect();
    names.sort();
    assert_eq!(names, ["Engine", "helper", "start"]);
    assert!(symbols.iter().all(|row| text(row, "path") == "engine.rs"));

    let reference = rows(
        &engine,
        &session,
        &format!("FROM record:source-symbol AT VALID {at} KNOWN HEAD WHERE role = \"reference\" PROJECT name"),
        at,
    );
    assert_eq!(text(&reference[0], "name"), "helper");

    let chunks = rows(
        &engine,
        &session,
        &format!("FROM record:source-chunk AT VALID {at} KNOWN HEAD PROJECT text"),
        at,
    );
    assert_eq!(chunks.len(), 1);
    assert_eq!(text(&chunks[0], "text").trim(), SOURCE.trim());

    let spans = rows(
        &engine,
        &session,
        &format!("FROM record:source-span AT VALID {at} KNOWN HEAD PROJECT span_kind"),
        at,
    );
    assert_eq!(spans.len(), 3);

    let parsed = rows(
        &engine,
        &session,
        &format!("FROM record:source-parse AT VALID {at} KNOWN HEAD PROJECT path, status"),
        at,
    );
    assert_eq!(parsed.len(), 1, "only the bound language is parsed");
    assert_eq!(text(&parsed[0], "status"), "parsed");
}

#[test]
fn a_large_project_parses_across_steps_that_each_commit() {
    let (_installed, engine, session, job) = started(16_384, |root| {
        for index in 0..12 {
            let body = format!("fn function_{index}() {{}}\n").repeat(40);
            std::fs::write(root.join(format!("file_{index:02}.rs")), body).unwrap();
        }
    });
    let history = run_job(&engine, &session, job, "steps");
    let parse_steps: Vec<_> = history
        .iter()
        .filter(|job| job.checkpoints.len() == 2)
        .collect();
    assert!(
        parse_steps.len() >= 2,
        "the bound must force several Steps, saw {}",
        parse_steps.len()
    );
    let mut generation = 0;
    for step in parse_steps.iter().filter(|job| job.state == State::Running) {
        let lease = step.lease.as_ref().unwrap();
        assert!(
            lease.generation > generation,
            "each Step renews the worker's lease"
        );
        assert_eq!(
            lease.expires_at_unix_ms - lease.acquired_at_unix_ms,
            30_000,
            "a renewal restarts the claimed duration and never extends it"
        );
        generation = lease.generation;
    }
    let done = history.last().unwrap();
    assert_eq!(done.state, State::Succeeded);
    let at = done.updated_at_unix_ms + 10;
    let parsed = rows(
        &engine,
        &session,
        &format!("FROM record:source-parse AT VALID {at} KNOWN HEAD PROJECT path"),
        at,
    );
    assert_eq!(parsed.len(), 12);
}

#[test]
fn a_step_replay_returns_its_receipt_and_commits_nothing() {
    let (_installed, engine, session, job) = started(4_194_304, |root| {
        std::fs::write(root.join("engine.rs"), SOURCE).unwrap();
    });
    let mut current = job;
    for name in ["replay-connect", "replay-inventory"] {
        current = call(
            &engine,
            &session,
            &step(&current),
            name,
            current.updated_at_unix_ms + 1,
        )
        .unwrap()
        .status
        .job;
    }
    let command = step(&current);
    let first = call(
        &engine,
        &session,
        &command,
        "replay-parse",
        current.updated_at_unix_ms + 1,
    )
    .unwrap();
    let scope = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
    let stamp = engine.storage.runtime().read_stamp(&scope).unwrap();
    let replay = call(
        &engine,
        &session,
        &command,
        "replay-parse",
        first.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.status.job, first.status.job);
    assert_eq!(engine.storage.runtime().read_stamp(&scope).unwrap(), stamp);
}

#[test]
fn an_edit_reparses_only_the_edited_file_and_a_deletion_retires_its_facts() {
    let (installed, engine, session, job) = started(4_194_304, |root| {
        std::fs::write(root.join("kept.rs"), "fn kept() {}\n").unwrap();
        std::fs::write(root.join("edited.rs"), "fn before_edit() {}\n").unwrap();
        std::fs::write(root.join("removed.rs"), "fn removed() {}\n").unwrap();
    });
    let done = run_job(&engine, &session, job, "first").pop().unwrap();
    let scope = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
    let before = engine.storage.runtime().read_stamp(&scope).unwrap();

    std::fs::write(
        installed.root.path().join("edited.rs"),
        "fn after_edit() {}\n",
    )
    .unwrap();
    std::fs::remove_file(installed.root.path().join("removed.rs")).unwrap();
    let done = replan_and_run(&engine, &session, &done, "second");
    assert_eq!(done.state, State::Succeeded, "{:?}", done.failure);
    let at = done.updated_at_unix_ms + 10;

    let names: Vec<_> = rows(
        &engine,
        &session,
        &format!("FROM record:source-symbol AT VALID {at} KNOWN HEAD WHERE role = \"definition\" PROJECT name"),
        at,
    )
    .iter()
    .map(|row| text(row, "name"))
    .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(
        sorted,
        ["after_edit", "kept"],
        "stale facts must be retired"
    );

    // The unchanged file's state was not rewritten by the second job.
    let changes = engine
        .read_changefeed(
            &session.session_id,
            &session.token,
            &ReadChangefeed {
                scope: scope.as_str().to_owned(),
                after_cursor: before.commit_cursor,
                limit: rrd_contract::MAX_CHANGEFEED_PAGE,
            },
            at,
            "request-parse-changes",
            "operation-parse-changes",
        )
        .unwrap();
    assert!(!changes.has_more);
    let rewritten: Vec<_> = changes
        .changes
        .iter()
        .filter_map(|change| match &change.mutation {
            ChangeMutationSnapshot::Data {
                mutation:
                    TransactionMutation::PutRecord {
                        reference,
                        properties,
                        ..
                    },
            } if reference.kind.as_str() == "source-parse" => properties.get("path").cloned(),
            _ => None,
        })
        .collect();
    assert_eq!(rewritten, [QueryValue::String("edited.rs".into())]);
}

fn replan_and_run(
    engine: &RrdEngine,
    session: &SessionLease,
    done: &AttunementJob,
    label: &str,
) -> AttunementJob {
    let at = done.updated_at_unix_ms + 10;
    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            &done.id,
            done.revision,
            at,
            &format!("{label}-preview"),
            &format!("{label}-preview"),
        )
        .unwrap();
    let accepted = engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context(&format!("{label}-accept")),
            at + 1,
        )
        .unwrap();
    let successor = accepted.successor.job;
    let successor = call(
        engine,
        session,
        &AttunementCommand::Start {
            job_id: successor.id.clone(),
            plan_sha256: successor.plan_sha256.clone(),
            expected_revision: successor.revision,
            lease_ms: 30_000,
        },
        &format!("{label}-start"),
        successor.updated_at_unix_ms + 1,
    )
    .unwrap()
    .status
    .job;
    run_job(engine, session, successor, label).pop().unwrap()
}

#[test]
fn a_deletion_larger_than_one_commit_retires_across_steps() {
    let (installed, engine, session, job) = started(32_768, |root| {
        std::fs::write(
            root.join("kept.rs"),
            "fn kept() {}
",
        )
        .unwrap();
        for index in 0..12 {
            let body = format!(
                "fn function_{index}() {{}}
"
            )
            .repeat(5);
            std::fs::write(root.join(format!("file_{index:02}.rs")), body).unwrap();
        }
    });
    let done = run_job(&engine, &session, job, "delete-first")
        .pop()
        .unwrap();
    assert_eq!(done.state, State::Succeeded, "{:?}", done.failure);
    for index in 0..12 {
        std::fs::remove_file(installed.root.path().join(format!("file_{index:02}.rs"))).unwrap();
    }
    let done = replan_and_run(&engine, &session, &done, "delete-second");
    assert_eq!(done.state, State::Succeeded, "{:?}", done.failure);
    let at = done.updated_at_unix_ms + 10;
    let names: Vec<_> = rows(
        &engine,
        &session,
        &format!("FROM record:source-symbol AT VALID {at} KNOWN HEAD PROJECT name"),
        at,
    )
    .iter()
    .map(|row| text(row, "name"))
    .collect();
    assert_eq!(
        names,
        ["kept"],
        "Parse finished before every stale fact was retired"
    );
}

#[test]
fn a_file_whose_facts_exceed_one_commit_is_rejected_not_retried() {
    let (_installed, engine, session, job) = started(16_384, |root| {
        std::fs::write(
            root.join("small.rs"),
            "fn small() {}
",
        )
        .unwrap();
        let body: String = (0..40)
            .map(|index| {
                format!(
                    "fn function_{index}() {{}}
"
                )
            })
            .collect();
        std::fs::write(root.join("big.rs"), body).unwrap();
    });
    let done = run_job(&engine, &session, job, "too-big").pop().unwrap();
    assert_eq!(done.state, State::Succeeded, "{:?}", done.failure);
    let at = done.updated_at_unix_ms + 10;
    let mut states: Vec<_> = rows(
        &engine,
        &session,
        &format!("FROM record:source-parse AT VALID {at} KNOWN HEAD PROJECT path, status, reason"),
        at,
    )
    .iter()
    .map(|row| (text(row, "path"), text(row, "status"), text(row, "reason")))
    .collect();
    states.sort();
    assert_eq!(
        states,
        [
            (
                "big.rs".to_owned(),
                "rejected".to_owned(),
                "facts_too_large".to_owned()
            ),
            ("small.rs".to_owned(), "parsed".to_owned(), String::new()),
        ]
    );
}
