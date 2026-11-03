//! A task comparison must grade independently, refuse to spend without a decision, and never
//! claim an arm helped without paired evidence. No test here can start a model.
use rrflow_eval::lab::knowledge::{chunk_text, importable, MAX_CHUNK_BYTES, MAX_FILE_BYTES};
use rrflow_eval::lab::manifest::Availability;
use rrflow_eval::lab::provider::{
    claude_arguments, parse_claude_output, AgentProvider, AgentReply, AgentRequest,
};
use rrflow_eval::lab::report::{SystemStatus, TestIdentity, VerdictOutcome};
use rrflow_eval::lab::tasks::{
    combine_task_runs, exact_mcnemar, normalize_path, parse_plan, run_order, run_task, score_plan,
    task_prompt, wilson_interval, AgentPlan, AgentTask, Preregistration, RunInputs, RunKey,
    SpendLimits, TaskArm, TaskComparisonManifest, TaskModel, TaskProviderKind, TaskRun,
    TaskRunOutcome,
};
use rrflow_eval::lab::Result;
use rrflow_eval::storage::Id;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

fn id(text: &str) -> Id {
    Id::try_from(text.to_string()).unwrap()
}

fn arm(name: &str, repository_access: bool) -> TaskArm {
    TaskArm {
        id: id(name),
        label: name.into(),
        availability: Availability::Available,
        repository_access,
        tools: if repository_access {
            vec!["Read".into(), "Grep".into()]
        } else {
            Vec::new()
        },
        preface: None,
        limits: Vec::new(),
    }
}

fn task(name: &str) -> AgentTask {
    AgentTask {
        id: id(name),
        title: name.into(),
        repository: "https://example.org/repository".into(),
        revision: "a".repeat(40),
        prompt: "Make searches refuse earlier.".into(),
        expected_files: vec!["src/search.rs".into(), "tests/search.rs".into()],
        expected_records: vec!["POAM-055".into()],
        source: "pull request 1".into(),
    }
}

fn manifest() -> TaskComparisonManifest {
    TaskComparisonManifest {
        schema_version: 1,
        test: TestIdentity {
            id: id("unit-tasks"),
            title: "Unit tasks".into(),
            family: id("agent-tasks"),
            revision: 1,
        },
        preregistration: Preregistration {
            question: "Does the arm help?".into(),
            baseline_arm: id("alone"),
            floor_arm: id("floor"),
            repetitions: 2,
            order_seed: 7,
            alpha: 0.05,
            min_file_recall: 0.5,
            min_file_precision: 0.5,
            max_floor_solve_rate: 0.1,
        },
        models: vec![TaskModel {
            id: id("model-a"),
            provider: TaskProviderKind::ClaudeCode,
            model: "model-a".into(),
            effort: None,
        }],
        spend: Some(SpendLimits {
            max_seconds_per_run: 60,
            max_usd_per_run: 1.0,
            max_usd_total: 10.0,
        }),
        arms: vec![arm("floor", false), arm("alone", true), arm("helper", true)],
        tasks: vec![task("t1"), task("t2")],
    }
}

/// Replies from a script and records where each run was sent.
struct Scripted {
    reply: AgentReply,
    directories: RefCell<Vec<PathBuf>>,
    prompts: RefCell<Vec<String>>,
}

impl Scripted {
    fn replying(text: &str) -> Self {
        Self {
            reply: AgentReply {
                text: text.into(),
                raw: text.as_bytes().to_vec(),
                exit_code: Some(0),
                input_tokens: Some(100),
                output_tokens: Some(20),
                cost_usd: Some(0.5),
                turns: Some(3),
                ..AgentReply::default()
            },
            directories: RefCell::new(Vec::new()),
            prompts: RefCell::new(Vec::new()),
        }
    }
}

impl AgentProvider for Scripted {
    fn run(&self, request: &AgentRequest<'_>) -> Result<AgentReply> {
        self.directories
            .borrow_mut()
            .push(request.directory.to_path_buf());
        self.prompts.borrow_mut().push(request.prompt.to_string());
        Ok(self.reply.clone())
    }
}

fn key(task: &str, arm: &str, repetition: u32) -> RunKey {
    RunKey {
        task: id(task),
        arm: id(arm),
        model: id("model-a"),
        repetition,
    }
}

fn run_with(manifest: &TaskComparisonManifest, provider: &Scripted, key: &RunKey) -> TaskRun {
    run_task(
        manifest,
        key,
        &RunInputs {
            provider,
            checkout: Path::new("checkout"),
            empty_directory: Path::new("empty"),
            spent_usd: 0.0,
            mcp_config: None,
        },
    )
    .unwrap()
    .0
}

const GOOD: &str = r#"{"files":["src/search.rs","tests/search.rs"],"records":["poam-055"]}"#;
const WRONG: &str = r#"{"files":["src/other.rs"]}"#;

#[test]
fn a_plan_is_scored_on_what_it_names() {
    let manifest = manifest();
    let rules = &manifest.preregistration;
    let task = task("t1");
    let plan = |files: &[&str]| AgentPlan {
        files: files.iter().map(|path| path.to_string()).collect(),
        ..AgentPlan::default()
    };
    let both = score_plan(&task, &plan(&["src/search.rs", "tests/search.rs"]), rules);
    assert_eq!((both.file_recall, both.file_precision), (1.0, 1.0));
    assert!(both.solved);

    let half = score_plan(&task, &plan(&["src/search.rs"]), rules);
    assert_eq!((half.file_recall, half.file_precision), (0.5, 1.0));
    assert!(half.solved, "the thresholds are inclusive");

    let padded = score_plan(
        &task,
        &plan(&["src/search.rs", "a.rs", "b.rs", "c.rs", "d.rs"]),
        rules,
    );
    assert_eq!(padded.file_precision, 0.2);
    assert!(!padded.solved, "naming everything cannot solve a task");

    let directory = score_plan(&task, &plan(&["src/", "tests/"]), rules);
    assert_eq!(directory.files_matched, 0);
    assert!(!directory.solved, "a directory is not the file");

    let empty = score_plan(&task, &plan(&[]), rules);
    assert_eq!(empty.file_precision, 0.0);
    assert!(!empty.solved);
}

#[test]
fn paths_written_differently_are_the_same_path() {
    for written in [
        "src/search.rs",
        "./src/search.rs",
        "/src/search.rs",
        "src\\search.rs",
        "`src/search.rs`",
        "src/search.rs:42",
        "src/search.rs#L42",
        "src/search.rs (the refusal)",
    ] {
        assert_eq!(normalize_path(written), "src/search.rs", "{written}");
    }
}

#[test]
fn a_plan_is_found_in_a_reply_or_the_reply_has_none() {
    assert!(parse_plan(GOOD).is_some());
    let fenced = format!("Here is the plan.\n```json\n{GOOD}\n```\nDone.");
    assert_eq!(parse_plan(&fenced).unwrap().files.len(), 2);
    let two = format!("{WRONG}\nOn reflection:\n{GOOD}");
    assert_eq!(
        parse_plan(&two).unwrap().files.len(),
        2,
        "the last plan counts"
    );
    assert!(parse_plan("I cannot help with that.").is_none());
    assert!(parse_plan(r#"{"other":1}"#).is_none());
    assert!(parse_plan("").is_none());
}

#[test]
fn statistics_match_independently_computed_values() {
    // Exact binomial, two-sided: 2 * P(X <= 1) for n = 8, p = 0.5 is 18 / 256.
    assert!((exact_mcnemar(7, 1) - 0.070_312_5).abs() < 1e-9);
    assert!((exact_mcnemar(1, 7) - 0.070_312_5).abs() < 1e-9);
    // 2 * P(X <= 3) for n = 13 is 756 / 8192.
    assert!((exact_mcnemar(10, 3) - 0.092_285_156_25).abs() < 1e-9);
    assert!((exact_mcnemar(8, 0) - 0.007_812_5).abs() < 1e-9);
    assert_eq!(exact_mcnemar(0, 0), 1.0);
    assert_eq!(exact_mcnemar(5, 5), 1.0);
    // Stays finite where a direct power of a half would underflow.
    assert!(exact_mcnemar(2_000, 2_000) <= 1.0);
    assert!(exact_mcnemar(2_000, 0) >= 0.0);

    // Wilson, 95%, 8 of 10: 0.4902 to 0.9433 by the published formula.
    let (low, high) = wilson_interval(8, 10);
    assert!((low - 0.4902).abs() < 0.001, "{low}");
    assert!((high - 0.9433).abs() < 0.001, "{high}");
    assert_eq!(wilson_interval(0, 0), (0.0, 0.0));
    assert!(wilson_interval(0, 5).0 == 0.0 && wilson_interval(5, 5).1 == 1.0);
}

#[test]
fn nothing_runs_until_models_and_a_spend_limit_are_chosen() {
    let provider = Scripted::replying(GOOD);
    let refused = |manifest: TaskComparisonManifest| {
        run_task(
            &manifest,
            &key("t1", "alone", 1),
            &RunInputs {
                provider: &provider,
                checkout: Path::new("checkout"),
                empty_directory: Path::new("empty"),
                spent_usd: 0.0,
                mcp_config: None,
            },
        )
        .unwrap_err()
    };
    let mut no_models = manifest();
    no_models.models.clear();
    assert!(refused(no_models).contains("no model is chosen"));
    let mut no_spend = manifest();
    no_spend.spend = None;
    assert!(refused(no_spend).contains("no spend limit"));
    assert!(provider.prompts.borrow().is_empty(), "nothing was called");
}

#[test]
fn a_run_that_could_pass_the_total_spend_is_refused() {
    let provider = Scripted::replying(GOOD);
    let manifest = manifest();
    let refused = run_task(
        &manifest,
        &key("t1", "alone", 1),
        &RunInputs {
            provider: &provider,
            checkout: Path::new("checkout"),
            empty_directory: Path::new("empty"),
            spent_usd: 9.5,
            mcp_config: None,
        },
    )
    .unwrap_err();
    assert!(refused.contains("total spend limit"), "{refused}");
    assert!(provider.prompts.borrow().is_empty());
}

#[test]
fn the_floor_arm_sees_no_repository_and_no_prompt_holds_the_answer() {
    let manifest = manifest();
    let provider = Scripted::replying(GOOD);
    run_with(&manifest, &provider, &key("t1", "floor", 1));
    run_with(&manifest, &provider, &key("t1", "alone", 1));
    assert_eq!(
        *provider.directories.borrow(),
        vec![PathBuf::from("empty"), PathBuf::from("checkout")]
    );
    for prompt in provider.prompts.borrow().iter() {
        assert!(!prompt.contains("src/search.rs"), "{prompt}");
        assert!(!prompt.contains("POAM-055"), "{prompt}");
    }
}

#[test]
fn a_manifest_refuses_a_prompt_that_gives_its_answer_away() {
    for leak in ["see src/search.rs first", "start in search.rs"] {
        let mut leaking = manifest();
        leaking.tasks[0].prompt = leak.into();
        let refused = leaking.validate().unwrap_err();
        assert!(refused.contains("gives away"), "{refused}");
    }
}

#[test]
fn a_manifest_keeps_its_comparison_honest() {
    let check = |change: fn(&mut TaskComparisonManifest), expected: &str| {
        let mut changed = manifest();
        change(&mut changed);
        let refused = changed.validate().unwrap_err();
        assert!(refused.contains(expected), "{refused}");
    };
    check(
        |m| m.arms[0].repository_access = true,
        "must not see the repository",
    );
    check(
        |m| m.arms[1].repository_access = false,
        "no repository, so it cannot have tools",
    );
    check(
        |m| {
            m.arms[1].availability = Availability::Unimplemented {
                reason: "not yet".into(),
            }
        },
        "baseline arm must be able to run",
    );
    check(
        |m| m.arms[2].availability = Availability::Unimplemented { reason: " ".into() },
        "must say why",
    );
    check(|m| m.tasks[0].revision = "main".into(), "full revision");
    check(|m| m.preregistration.repetitions = 0, "repetitions");
    check(|m| m.preregistration.alpha = 1.0, "alpha");
    check(
        |m| {
            m.spend = Some(SpendLimits {
                max_seconds_per_run: 1,
                max_usd_per_run: 5.0,
                max_usd_total: 1.0,
            })
        },
        "total must cover one run",
    );
}

#[test]
fn the_run_order_is_complete_shuffled_and_repeatable() {
    let manifest = manifest();
    let order = run_order(&manifest);
    assert_eq!(order.len(), 2 * 3 * 2, "tasks, arms and repetitions");
    assert_eq!(order, run_order(&manifest));
    let mut other = manifest.clone();
    other.preregistration.order_seed = 8;
    assert_ne!(order, run_order(&other));
    let mut with_gap = manifest.clone();
    with_gap.arms[2].availability = Availability::Unimplemented {
        reason: "not yet".into(),
    };
    assert!(run_order(&with_gap)
        .iter()
        .all(|key| key.arm.as_str() != "helper"));
}

fn all_runs(manifest: &TaskComparisonManifest, replies: &[(&str, &str, &str)]) -> Vec<TaskRun> {
    let mut runs = Vec::new();
    for (task_id, arm_id, text) in replies {
        for repetition in 1..=2 {
            let provider = Scripted::replying(text);
            runs.push(run_with(
                manifest,
                &provider,
                &key(task_id, arm_id, repetition),
            ));
        }
    }
    runs
}

#[test]
fn a_report_says_whether_an_arm_helped_only_from_paired_runs() {
    let manifest = manifest();
    let runs = all_runs(
        &manifest,
        &[
            ("t1", "floor", WRONG),
            ("t2", "floor", WRONG),
            ("t1", "alone", WRONG),
            ("t2", "alone", GOOD),
            ("t1", "helper", GOOD),
            ("t2", "helper", GOOD),
        ],
    );
    let report = combine_task_runs(&manifest, &runs, Vec::new(), 1).unwrap();
    let verdict = |name: &str| {
        report
            .verdicts
            .iter()
            .find(|verdict| verdict.id.as_str() == name)
            .unwrap()
    };
    let helps = verdict("helps-helper.model-a");
    assert_eq!(
        helps.outcome,
        VerdictOutcome::NotMet,
        "2 discordant pairs cannot be significant"
    );
    assert!(helps.basis.contains("4 pairs"), "{}", helps.basis);
    assert!(helps.basis.contains("does not show the arm has no effect"));
    assert_eq!(verdict("no-leak-model-a").outcome, VerdictOutcome::Met);

    let solved = report
        .observations
        .iter()
        .find(|entry| {
            entry.metric.as_str() == "solve_rate"
                && entry.system.as_str() == "helper.model-a"
                && entry.case.is_empty()
        })
        .unwrap();
    assert_eq!(solved.value, 1.0);
    assert_eq!(solved.samples, 4);
    assert!(solved.low.unwrap() < 1.0, "an interval shows how few runs");
    assert!(report
        .observations
        .iter()
        .any(|entry| entry.metric.as_str() == "cost_usd"));
}

#[test]
fn a_large_paired_difference_can_be_significant() {
    let mut big = manifest();
    big.tasks = (0..12).map(|index| task(&format!("t{index}"))).collect();
    big.preregistration.repetitions = 1;
    let mut runs = Vec::new();
    for task in &big.tasks {
        let name = task.id.as_str().to_string();
        for (arm_id, text) in [("alone", WRONG), ("helper", GOOD)] {
            let provider = Scripted::replying(text);
            runs.push(run_with(&big, &provider, &key(&name, arm_id, 1)));
        }
    }
    let report = combine_task_runs(&big, &runs, Vec::new(), 1).unwrap();
    let helps = report
        .verdicts
        .iter()
        .find(|verdict| verdict.id.as_str() == "helps-helper.model-a")
        .unwrap();
    assert_eq!(helps.outcome, VerdictOutcome::Met, "{}", helps.basis);
    let floor = report
        .verdicts
        .iter()
        .find(|verdict| verdict.id.as_str() == "no-leak-model-a")
        .unwrap();
    assert_eq!(floor.outcome, VerdictOutcome::NotRun);
}

#[test]
fn leaking_tasks_show_up_in_the_floor_verdict() {
    let manifest = manifest();
    let runs = all_runs(&manifest, &[("t1", "floor", GOOD), ("t2", "floor", GOOD)]);
    let report = combine_task_runs(&manifest, &runs, Vec::new(), 1).unwrap();
    let floor = report
        .verdicts
        .iter()
        .find(|verdict| verdict.id.as_str() == "no-leak-model-a")
        .unwrap();
    assert_eq!(floor.outcome, VerdictOutcome::NotMet);
}

#[test]
fn a_changed_design_or_a_declared_gap_cannot_use_a_run() {
    let original = manifest();
    let runs = all_runs(&original, &[("t1", "alone", GOOD)]);

    let mut changed = original.clone();
    changed.preregistration.min_file_recall = 0.9;
    let refused = combine_task_runs(&changed, &runs, Vec::new(), 1).unwrap_err();
    assert!(refused.contains("different frozen design"), "{refused}");

    let mut twice = runs.clone();
    twice.push(runs[0].clone());
    let refused = combine_task_runs(&original, &twice, Vec::new(), 1).unwrap_err();
    assert!(refused.contains("appears twice"), "{refused}");

    let mut gap = original.clone();
    gap.arms[2].availability = Availability::Unimplemented {
        reason: "not yet".into(),
    };
    let provider = Scripted::replying(GOOD);
    let refused = run_task(
        &gap,
        &key("t1", "helper", 1),
        &RunInputs {
            provider: &provider,
            checkout: Path::new("checkout"),
            empty_directory: Path::new("empty"),
            spent_usd: 0.0,
            mcp_config: None,
        },
    )
    .unwrap_err();
    assert!(refused.contains("declared as not running"), "{refused}");
    let report = combine_task_runs(&gap, &[], Vec::new(), 1).unwrap();
    let shown = report
        .systems
        .iter()
        .find(|system| system.id.as_str() == "helper")
        .unwrap();
    assert_eq!(shown.status, SystemStatus::Unimplemented);
}

#[test]
fn a_reply_without_a_plan_or_a_failed_run_stays_visible_and_unsolved() {
    let manifest = manifest();
    let refusal = run_with(
        &manifest,
        &Scripted::replying("I cannot help with that."),
        &key("t1", "alone", 1),
    );
    assert_eq!(refusal.outcome, TaskRunOutcome::NoPlan);
    assert!(!refusal.score.solved);
    assert_eq!(refusal.score.files_expected, 2);
    assert!(refusal.reply_sha256.is_some());

    let mut failing = Scripted::replying(GOOD);
    failing.reply.exit_code = Some(1);
    let failed = run_with(&manifest, &failing, &key("t1", "alone", 1));
    assert_eq!(failed.outcome, TaskRunOutcome::ProviderFailed);
    assert!(
        !failed.score.solved,
        "a good plan from a failed run does not count"
    );

    let mut slow = Scripted::replying(GOOD);
    slow.reply.timed_out = true;
    let timed_out = run_with(&manifest, &slow, &key("t1", "alone", 1));
    assert_eq!(timed_out.outcome, TaskRunOutcome::TimedOut);
    assert!(!timed_out.score.solved);
}

#[test]
fn an_arm_with_an_mcp_tool_must_be_given_its_server_and_no_other_arm_may_be() {
    let mut manifest = manifest();
    manifest.arms[2]
        .tools
        .push("mcp__rrflow__rrflow_context".into());
    let provider = Scripted::replying(GOOD);
    let run = |arm: &str, config: Option<&Path>| {
        run_task(
            &manifest,
            &key("t1", arm, 1),
            &RunInputs {
                provider: &provider,
                checkout: Path::new("checkout"),
                empty_directory: Path::new("empty"),
                spent_usd: 0.0,
                mcp_config: config,
            },
        )
    };
    let config = Path::new("mcp.json");
    assert!(run("helper", None).is_err());
    assert!(run("alone", Some(config)).is_err());
    assert!(run("helper", Some(config)).is_ok());
    assert!(run("alone", None).is_ok());
}

#[test]
fn repository_text_is_chunked_on_lines_and_never_loses_any() {
    let text = "first line\n".repeat(400);
    let chunks = chunk_text("a/b.rs", &text);
    assert!(chunks.len() > 1);
    assert!(chunks
        .iter()
        .all(|chunk| chunk.text.len() <= MAX_CHUNK_BYTES));
    assert!(chunks.iter().all(|chunk| chunk.text.ends_with('\n')));
    let joined: String = chunks.iter().map(|chunk| chunk.text.as_str()).collect();
    assert_eq!(joined, text);
    let ordinals: Vec<u64> = chunks.iter().map(|chunk| chunk.ordinal).collect();
    assert_eq!(ordinals, (0..chunks.len() as u64).collect::<Vec<_>>());

    // One line longer than a chunk, with characters wider than one byte.
    let wide = "é".repeat(MAX_CHUNK_BYTES);
    let chunks = chunk_text("a.md", &wide);
    assert!(chunks
        .iter()
        .all(|chunk| chunk.text.len() <= MAX_CHUNK_BYTES));
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| chunk.text.as_str())
            .collect::<String>(),
        wide
    );
    assert!(chunk_text("a.md", "  \n\n").is_empty());
}

#[test]
fn only_a_trackable_text_file_is_imported_and_never_a_tests_answers() {
    assert!(importable("src/a.rs", b"fn a() {}"));
    assert!(!importable("src/a.bin", b"a\0b"), "binary");
    assert!(!importable("src/a.rs", &[0xFF, 0xFE]), "not UTF-8");
    assert!(!importable("src/a.rs", b""), "empty");
    assert!(!importable("Cargo.lock", b"text"), "a lock file");
    assert!(
        !importable("eval/lab/tasks/plan.json", b"text"),
        "a test's own answers"
    );
    assert!(
        !importable("big.md", &vec![b'a'; MAX_FILE_BYTES + 1]),
        "too large"
    );
}

#[test]
fn a_chunk_becomes_one_public_mutation_with_a_stable_identity() {
    let chunk = &chunk_text("src/a.rs", "fn a() {}\n")[0];
    let same = &chunk_text("src/a.rs", "fn changed() {}\n")[0];
    assert_eq!(chunk.reference_id(), same.reference_id(), "text can change");
    assert_ne!(
        chunk.reference_id(),
        chunk_text("src/b.rs", "fn a() {}\n")[0].reference_id()
    );
    let decoded: Vec<rrd_contract::TransactionMutation> =
        serde_json::from_value(serde_json::json!([chunk.mutation()])).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn the_prompt_is_the_task_and_the_reply_format() {
    let mut helper = arm("helper", true);
    helper.preface = Some("Use the tool.".into());
    let prompt = task_prompt(&task("t1"), &helper);
    assert!(prompt.starts_with("Use the tool.\n\nMake searches refuse earlier."));
    assert!(prompt.contains("Do not change any file"));
}

#[test]
fn claude_runs_isolated_with_only_what_the_arm_declares() {
    let tools = vec!["Read".to_string(), "Grep".to_string()];
    let request = AgentRequest {
        prompt: "the prompt",
        directory: Path::new("checkout"),
        model: "a-model",
        effort: Some("high"),
        tools: &tools,
        mcp_config: None,
        max_seconds: 60,
        max_usd: 2.5,
    };
    let arguments = claude_arguments(&request);
    let has = |flag: &str| arguments.iter().any(|argument| argument == flag);
    assert_eq!(
        arguments[0..2],
        ["-p", "the prompt"],
        "before any list flag"
    );
    for flag in ["--bare", "--strict-mcp-config", "--no-session-persistence"] {
        assert!(has(flag), "{flag}");
    }
    let value = |flag: &str| {
        let position = arguments
            .iter()
            .position(|argument| argument == flag)
            .unwrap();
        arguments[position + 1].as_str()
    };
    assert_eq!(value("--permission-mode"), "plan");
    assert_eq!(value("--max-budget-usd"), "2.5");
    assert_eq!(value("--tools"), "Read,Grep");
    assert_eq!(value("--model"), "a-model");

    let none: Vec<String> = Vec::new();
    let empty = claude_arguments(&AgentRequest {
        tools: &none,
        effort: None,
        ..request
    });
    let position = empty
        .iter()
        .position(|argument| argument == "--tools")
        .unwrap();
    assert_eq!(empty[position + 1], "", "no tools at all");
    assert!(!empty.iter().any(|argument| argument == "--allowedTools"));
}

#[test]
fn a_claude_result_is_read_for_its_reply_and_usage() {
    let raw = br#"{"type":"result","result":"the reply","total_cost_usd":0.25,"num_turns":4,"usage":{"input_tokens":10,"cache_read_input_tokens":90,"output_tokens":7}}"#;
    let reply = parse_claude_output(raw.to_vec(), Some(0), false);
    assert_eq!(reply.text, "the reply");
    assert_eq!(reply.input_tokens, Some(100));
    assert_eq!(reply.output_tokens, Some(7));
    assert_eq!(reply.cost_usd, Some(0.25));
    assert_eq!(reply.turns, Some(4));

    let broken = parse_claude_output(b"not json".to_vec(), Some(1), false);
    assert_eq!(broken.text, "");
    assert_eq!(broken.cost_usd, None);
}
