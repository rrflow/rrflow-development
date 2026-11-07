//! A study has to be fit to answer its question before it can spend anything, and its runs have
//! to prove their data arrived. No test here can start a model.
use rrflow_eval::lab::manifest::Availability;
use rrflow_eval::lab::provider::{parse_claude_output, AgentProvider, AgentReply, AgentRequest};
use rrflow_eval::lab::report::{SystemStatus, TestIdentity, VerdictOutcome};
use rrflow_eval::lab::sources::SourceList;
use rrflow_eval::lab::study::{
    audit_runs, check_study, discordant_pairs_needed, plan_study, resolve_study, AnalysisChoices,
    CatalogReference, RequirementStatus, StudyCheck, StudyConfig, StudyRequirements, TaskSelection,
};
use rrflow_eval::lab::tasks::{
    charged_usd, combine_task_runs, run_order, run_task, AgentTask, Preregistration, RunInputs,
    RunKey, SpendLimits, StudyStage, TaskArm, TaskComparisonManifest, TaskModel, TaskProviderKind,
    TaskRun, TaskRunOutcome,
};
use rrflow_eval::lab::Result;
use rrflow_eval::storage::Id;
use std::collections::BTreeMap;
use std::path::Path;

const MCP_TOOL: &str = "mcp__rrflow__rrflow_context";
const GOOD: &str = r#"{"files":["src/search.rs","tests/search.rs"]}"#;
const WRONG: &str = r#"{"files":["src/other.rs"]}"#;

fn id(text: &str) -> Id {
    Id::try_from(text.to_string()).unwrap()
}

fn lab_file(name: &str) -> String {
    let path = format!("{}/../../../eval/lab/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn sources() -> SourceList {
    serde_json::from_str(&lab_file("sources.json")).unwrap()
}

fn requirements() -> StudyRequirements {
    let requirements: StudyRequirements =
        serde_json::from_str(&lab_file("study-requirements.json")).unwrap();
    requirements.validate(&sources()).unwrap();
    requirements
}

fn arm(name: &str, repository_access: bool, tools: &[&str]) -> TaskArm {
    TaskArm {
        id: id(name),
        label: name.into(),
        availability: Availability::Available,
        repository_access,
        tools: tools.iter().map(|tool| (*tool).to_string()).collect(),
        preface: None,
        limits: Vec::new(),
    }
}

fn catalog() -> TaskComparisonManifest {
    let mut gap = arm("gap", true, &[]);
    gap.availability = Availability::Unimplemented {
        reason: "Not built.".into(),
    };
    TaskComparisonManifest {
        schema_version: 1,
        test: TestIdentity {
            id: id("unit-catalog"),
            title: "Unit catalog".into(),
            family: id("agent-tasks"),
            revision: 1,
        },
        preregistration: Preregistration {
            question: "Does the arm help?".into(),
            baseline_arm: id("alone"),
            floor_arm: id("floor"),
            repetitions: 3,
            order_seed: 7,
            alpha: 0.05,
            min_file_recall: 0.5,
            min_file_precision: 0.5,
            max_floor_file_match_rate: 0.1,
            stage: StudyStage::Confirmatory,
            required_captures: Vec::new(),
        },
        models: Vec::new(),
        spend: None,
        arms: vec![
            arm("floor", false, &[]),
            arm("alone", true, &["Read", "Grep"]),
            arm("helper", true, &["Read", "Grep", MCP_TOOL]),
            arm("another", true, &["Read"]),
            gap,
        ],
        tasks: (1..=30)
            .map(|number| AgentTask {
                id: id(&format!("t{number}")),
                title: format!("task {number}"),
                repository: "https://example.org/repository".into(),
                revision: "a".repeat(40),
                prompt: "Make searches refuse earlier.".into(),
                expected_files: vec!["src/search.rs".into(), "tests/search.rs".into()],
                expected_records: Vec::new(),
                source: format!("pull request {number}"),
            })
            .collect(),
    }
}

/// The pilot recommended to the owner: 5 tasks in 3 arms, once each, at $2 and $30.
fn pilot() -> StudyConfig {
    StudyConfig {
        schema_version: 1,
        id: id("unit-pilot"),
        title: "Unit pilot".into(),
        stage: StudyStage::Pilot,
        catalog: CatalogReference {
            path: "catalog.json".into(),
            sha256: catalog().digest().unwrap(),
        },
        tasks: TaskSelection::Seeded { count: 5, seed: 11 },
        arms: vec![id("floor"), id("alone"), id("helper")],
        baseline_arm: id("alone"),
        floor_arm: id("floor"),
        models: vec![TaskModel {
            id: id("sonnet"),
            provider: TaskProviderKind::ClaudeCode,
            model: "claude-sonnet-5-5".into(),
            effort: None,
        }],
        repetitions: 1,
        spend: Some(SpendLimits {
            max_seconds_per_run: 900,
            max_usd_per_run: 2.0,
            max_usd_total: 30.0,
        }),
        credential_env: Some("ANTHROPIC_API_KEY".into()),
        analysis: AnalysisChoices::default(),
        capture_proof_sha256: None,
    }
}

fn status(check: &StudyCheck, requirement: &str) -> RequirementStatus {
    check
        .outcomes
        .iter()
        .find(|outcome| outcome.id.as_str() == requirement)
        .unwrap_or_else(|| panic!("no outcome for {requirement}"))
        .status
}

fn detail(check: &StudyCheck, requirement: &str) -> String {
    check
        .outcomes
        .iter()
        .find(|outcome| outcome.id.as_str() == requirement)
        .unwrap()
        .detail
        .clone()
}

type Damage = fn(&mut AgentReply);

/// Replies with everything a real provider reports.
struct Scripted(AgentReply);

impl Scripted {
    fn replying(text: &str) -> Self {
        Self(AgentReply {
            text: text.into(),
            raw: text.as_bytes().to_vec(),
            exit_code: Some(0),
            input_tokens: Some(100),
            output_tokens: Some(20),
            cost_usd: Some(0.5),
            turns: Some(3),
            cache_read_tokens: Some(40),
            cache_write_tokens: Some(0),
            model_reported: Some("claude-sonnet-5-5-20261001".into()),
            provider_version: Some("2.9.0".into()),
            mcp_servers: Some(BTreeMap::from([("rrflow".into(), "connected".into())])),
            tool_calls: Some(BTreeMap::from([(MCP_TOOL.into(), 1), ("Read".into(), 4)])),
            mcp_results: vec!["item record:lab_chunk:c1, and record:lab_chunk:c3".into()],
            ..AgentReply::default()
        })
    }
}

impl AgentProvider for Scripted {
    fn run(&self, _: &AgentRequest<'_>) -> Result<AgentReply> {
        Ok(self.0.clone())
    }
}

fn chunks() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("c1".to_string(), "src/search.rs".to_string()),
        ("c2".to_string(), "tests/search.rs".to_string()),
        ("c3".to_string(), "src/other.rs".to_string()),
    ])
}

fn run_key(
    manifest: &TaskComparisonManifest,
    provider: &Scripted,
    key: &RunKey,
) -> Result<(TaskRun, Vec<u8>)> {
    let chunks = chunks();
    let uses_mcp = manifest
        .arm(key.arm.as_str())?
        .tools
        .iter()
        .any(|tool| tool.starts_with("mcp__"));
    run_task(
        manifest,
        key,
        &RunInputs {
            provider,
            checkout: Path::new("checkout"),
            empty_directory: Path::new("empty"),
            spent_usd: 0.0,
            mcp_config: uses_mcp.then(|| Path::new("mcp.json")),
            chunk_files: uses_mcp.then_some(&chunks),
        },
    )
}

fn planned(config: &StudyConfig) -> TaskComparisonManifest {
    plan_study(config, &catalog(), &requirements(), None).unwrap()
}

fn key(manifest: &TaskComparisonManifest, arm: &str, task: usize) -> RunKey {
    RunKey {
        task: manifest.tasks[task].id.clone(),
        arm: id(arm),
        model: manifest.models[0].id.clone(),
        repetition: 1,
    }
}

/// Every run of a plan, each arm replying as `reply_for` says.
fn run_all(
    manifest: &TaskComparisonManifest,
    reply_for: impl Fn(&str) -> &'static str,
) -> Vec<TaskRun> {
    run_order(manifest)
        .iter()
        .map(|key| {
            let provider = Scripted::replying(reply_for(key.arm.as_str()));
            run_key(manifest, &provider, key).unwrap().0
        })
        .collect()
}

#[test]
fn every_checked_in_requirement_cites_something_that_exists() {
    let requirements = requirements();
    assert!(requirements.requirements.len() >= 10);
    for requirement in &requirements.requirements {
        assert!(
            !requirement.sources.is_empty() || !requirement.records.is_empty(),
            "{}",
            requirement.id.as_str()
        );
    }
}

#[test]
fn a_requirement_cannot_cite_nothing_an_unknown_source_or_an_unknown_check() {
    let mut unknown = requirements();
    unknown.requirements[0].sources = vec![id("a-study-nobody-read")];
    let refused = unknown.validate(&sources()).unwrap_err();
    assert!(refused.contains("a-study-nobody-read"), "{refused}");

    let mut uncited = requirements();
    uncited.requirements[0].sources.clear();
    uncited.requirements[0].records.clear();
    let refused = uncited.validate(&sources()).unwrap_err();
    assert!(refused.contains("must cite"), "{refused}");

    let text = lab_file("study-requirements.json").replace("\"frozen_design\"", "\"trust_me\"");
    assert!(serde_json::from_str::<StudyRequirements>(&text).is_err());
}

#[test]
fn the_recommended_pilot_can_run_and_says_it_cannot_answer_the_question() {
    let check = check_study(&pilot(), &catalog(), &requirements(), None).unwrap();
    assert!(check.ready);
    assert_eq!(check.runs, 15);
    assert_eq!(check.worst_case_usd, Some(30.0));
    assert_eq!(check.discordant_pairs_needed, 6);
    assert!(!check.can_reach_significance);
    assert_eq!(
        status(&check, "can-reach-significance"),
        RequirementStatus::AdvisedNotMet
    );
    assert!(detail(&check, "can-reach-significance").contains("cannot show a difference"));
    assert_eq!(
        status(&check, "capture-proven"),
        RequirementStatus::AdvisedNotMet
    );
}

#[test]
fn a_confirmatory_study_needs_enough_tasks_and_a_proven_capture() {
    let mut config = pilot();
    config.stage = StudyStage::Confirmatory;
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert!(!check.ready);
    assert_eq!(
        status(&check, "can-reach-significance"),
        RequirementStatus::NotMet
    );
    assert_eq!(status(&check, "capture-proven"), RequirementStatus::NotMet);
    let refused = plan_study(&config, &catalog(), &requirements(), None).unwrap_err();
    assert!(refused.contains("can-reach-significance"), "{refused}");
    assert!(refused.contains("capture-proven"), "{refused}");
}

#[test]
fn the_least_pairs_that_can_reach_alpha_match_the_exact_test() {
    assert_eq!(discordant_pairs_needed(0.05), 6, "2 x 0.5^6 = 0.03125");
    assert_eq!(discordant_pairs_needed(0.01), 8, "2 x 0.5^8 = 0.0078125");
}

#[test]
fn only_an_exact_model_name_counts_as_chosen() {
    for floating in [
        "sonnet",
        "opus",
        "latest",
        "claude-sonnet-latest",
        "claude-sonnet",
    ] {
        let mut config = pilot();
        config.models[0].model = floating.into();
        let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
        assert_eq!(
            status(&check, "pinned-model"),
            RequirementStatus::NotMet,
            "{floating}"
        );
        assert!(!check.ready, "{floating}");
    }
    let mut none = pilot();
    none.models.clear();
    let check = check_study(&none, &catalog(), &requirements(), None).unwrap();
    assert_eq!(status(&check, "pinned-model"), RequirementStatus::NotMet);
}

#[test]
fn spend_limits_have_to_cover_every_run_at_its_limit() {
    let mut config = pilot();
    config.spend.as_mut().unwrap().max_usd_total = 29.0;
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert_eq!(
        status(&check, "spend-covers-plan"),
        RequirementStatus::NotMet
    );
    assert!(detail(&check, "spend-covers-plan").contains("above the $29 total"));

    config.spend = None;
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert_eq!(
        status(&check, "spend-covers-plan"),
        RequirementStatus::NotMet
    );
    assert_eq!(check.worst_case_usd, None);
}

#[test]
fn a_study_names_its_key_and_never_holds_it() {
    let mut config = pilot();
    config.credential_env = Some("sk-ant-not-a-real-key".into());
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert_eq!(
        status(&check, "credential-by-name"),
        RequirementStatus::NotMet
    );
    assert!(
        !detail(&check, "credential-by-name").contains("sk-ant"),
        "a check must not repeat what looks like a key"
    );

    config.credential_env = None;
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert_eq!(
        status(&check, "credential-by-name"),
        RequirementStatus::NotMet
    );
}

#[test]
fn a_changed_catalog_or_an_arm_that_cannot_run_stops_the_check() {
    let mut changed = pilot();
    changed.catalog.sha256 = rrflow_eval::storage::Digest::try_from("b".repeat(64)).unwrap();
    let refused = check_study(&changed, &catalog(), &requirements(), None).unwrap_err();
    assert!(refused.contains("catalog changed"), "{refused}");

    let mut unknown = pilot();
    unknown.arms.push(id("nobody"));
    let refused = check_study(&unknown, &catalog(), &requirements(), None).unwrap_err();
    assert!(refused.contains("nobody"), "{refused}");

    let mut gap = pilot();
    gap.arms.push(id("gap"));
    let refused = check_study(&gap, &catalog(), &requirements(), None).unwrap_err();
    assert!(refused.contains("Not built"), "{refused}");
}

#[test]
fn a_baseline_holding_the_capability_or_a_floor_with_access_is_not_met() {
    let mut config = pilot();
    config.baseline_arm = id("helper");
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert_eq!(status(&check, "baseline-arm"), RequirementStatus::NotMet);
    assert!(!check.ready);

    let mut config = pilot();
    config.floor_arm = id("alone");
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert_eq!(status(&check, "leak-floor"), RequirementStatus::NotMet);

    let mut config = pilot();
    config.arms.retain(|arm| arm.as_str() != "helper");
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert_eq!(
        status(&check, "paired-comparison"),
        RequirementStatus::NotMet
    );
}

#[test]
fn a_plan_freezes_the_choices_and_keeps_what_was_left_out_visible() {
    let config = pilot();
    let plan = planned(&config);
    assert_eq!(plan.tasks.len(), 5);
    assert_eq!(plan.preregistration.stage, StudyStage::Pilot);
    assert_eq!(plan.preregistration.repetitions, 1);
    assert_eq!(plan.preregistration.required_captures.len(), 9);
    assert_eq!(
        plan.digest().unwrap(),
        planned(&config).digest().unwrap(),
        "repeatable"
    );
    let left_out = plan.arm("another").unwrap();
    assert!(matches!(
        left_out.availability,
        Availability::NotApplicable { .. }
    ));
    assert!(matches!(
        plan.arm("gap").unwrap().availability,
        Availability::Unimplemented { .. }
    ));

    let mut listed = pilot();
    listed.tasks = TaskSelection::Listed {
        ids: vec![id("t2"), id("t1"), id("t2")],
    };
    let manifest = resolve_study(&listed, &catalog(), &requirements()).unwrap();
    let names: Vec<&str> = manifest.tasks.iter().map(|task| task.id.as_str()).collect();
    assert_eq!(names, ["t1", "t2"]);

    let mut too_many = pilot();
    too_many.tasks = TaskSelection::Seeded { count: 31, seed: 1 };
    assert!(check_study(&too_many, &catalog(), &requirements(), None).is_err());
}

#[test]
fn a_run_with_no_captured_cost_is_charged_at_its_limit() {
    let plan = planned(&pilot());
    let mut provider = Scripted::replying(GOOD);
    provider.0.cost_usd = None;
    let (run, _) = run_key(&plan, &provider, &key(&plan, "alone", 0)).unwrap();
    assert_eq!(run.cost_usd, None);
    assert_eq!(
        charged_usd(&run, plan.spend.as_ref().unwrap()),
        2.0,
        "a missing cost must not read as free, or the total stops limiting"
    );
    let paid = run_key(&plan, &Scripted::replying(GOOD), &key(&plan, "alone", 0))
        .unwrap()
        .0;
    assert_eq!(charged_usd(&paid, plan.spend.as_ref().unwrap()), 0.5);
}

#[test]
fn a_run_whose_mcp_server_never_connected_says_nothing_about_the_arm() {
    let plan = planned(&pilot());
    let helper = key(&plan, "helper", 0);
    for servers in [
        Some(BTreeMap::from([(
            "rrflow".to_string(),
            "failed".to_string(),
        )])),
        Some(BTreeMap::new()),
        None,
    ] {
        let mut provider = Scripted::replying(WRONG);
        provider.0.mcp_servers = servers;
        let (run, _) = run_key(&plan, &provider, &helper).unwrap();
        assert_eq!(run.outcome, TaskRunOutcome::ToolingFailed);
        assert!(run.reason.as_deref().unwrap().contains("rrflow"));
        assert!(!run.score.matches_file_threshold);
    }

    // The arm's only runs never came up, so the verdict has no pairs to judge.
    let mut runs = Vec::new();
    for task in 0..plan.tasks.len() {
        let mut provider = Scripted::replying(WRONG);
        provider.0.mcp_servers = Some(BTreeMap::from([("rrflow".into(), "failed".into())]));
        runs.push(
            run_key(&plan, &provider, &key(&plan, "helper", task))
                .unwrap()
                .0,
        );
        runs.push(
            run_key(&plan, &Scripted::replying(GOOD), &key(&plan, "alone", task))
                .unwrap()
                .0,
        );
    }
    let report = combine_task_runs(&plan, &runs, Vec::new(), 1).unwrap();
    let system = report
        .systems
        .iter()
        .find(|system| system.id.as_str() == "helper.sonnet")
        .unwrap();
    assert_eq!(system.status, SystemStatus::Failed);
    let verdict = report
        .verdicts
        .iter()
        .find(|verdict| verdict.id.as_str() == "helps-helper.sonnet")
        .unwrap();
    assert_eq!(verdict.outcome, VerdictOutcome::NotRun);
    assert!(
        verdict.basis.contains("no paired runs"),
        "{}",
        verdict.basis
    );

    let audit = audit_runs(&plan, &runs).unwrap();
    assert!(!audit.complete);
    let entry = audit
        .arms
        .iter()
        .find(|arm| arm.arm.as_str() == "helper")
        .unwrap();
    assert_eq!(entry.runs_without_tools, 5);
    assert!(!entry.proven);
}

#[test]
fn a_mcp_arm_with_a_connected_server_runs_normally_even_if_the_agent_never_calls_it() {
    let plan = planned(&pilot());
    let mut provider = Scripted::replying(GOOD);
    provider.0.tool_calls = Some(BTreeMap::from([("Read".to_string(), 2)]));
    provider.0.mcp_results.clear();
    let (run, _) = run_key(&plan, &provider, &key(&plan, "helper", 0)).unwrap();
    assert_eq!(run.outcome, TaskRunOutcome::Planned);
    let retrieval = run.retrieval.unwrap();
    assert_eq!(retrieval.calls, 0);
    assert_eq!(retrieval.needed_files_used, 0);
}

#[test]
fn an_event_stream_is_read_for_everything_it_reports() {
    let stream = [
        r#"{"type":"system","subtype":"init","model":"claude-sonnet-5-5","claude_code_version":"2.9.0","tools":["Read",""#.to_string()
            + MCP_TOOL
            + r#""],"mcp_servers":[{"name":"rrflow","status":"connected"}]}"#,
        r#"{"type":"assistant","message":{"model":"claude-sonnet-5-5-20261001","content":[{"type":"text","text":"looking"},{"type":"tool_use","id":"t1","name":""#.to_string()
            + MCP_TOOL
            + r#"","input":{"query":"search"}},{"type":"tool_use","id":"t2","name":"Read","input":{"file_path":"src/search.rs"}}]}}"#,
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":[{"type":"text","text":"found record:lab_chunk:c1"}]},{"type":"tool_result","tool_use_id":"t2","content":"file text with c2 in it"}]}}"#.to_string(),
        r#"not an event"#.to_string(),
        r#"{"type":"result","subtype":"success","result":"the reply","total_cost_usd":0.25,"num_turns":4,"usage":{"input_tokens":100,"output_tokens":7,"cache_read_input_tokens":30,"cache_creation_input_tokens":5}}"#.to_string(),
    ]
    .join("\n");
    let reply = parse_claude_output(stream.clone().into_bytes(), Some(0), false);
    assert_eq!(reply.text, "the reply");
    assert_eq!(
        reply.model_reported.as_deref(),
        Some("claude-sonnet-5-5-20261001")
    );
    assert_eq!(reply.provider_version.as_deref(), Some("2.9.0"));
    assert_eq!(reply.cache_read_tokens, Some(30));
    assert_eq!(reply.cache_write_tokens, Some(5));
    assert_eq!(reply.input_tokens, Some(135), "includes the cache");
    assert_eq!(reply.output_tokens, Some(7));
    assert_eq!(reply.cost_usd, Some(0.25));
    assert_eq!(reply.turns, Some(4));
    assert_eq!(reply.mcp_servers.as_ref().unwrap()["rrflow"], "connected");
    let calls = reply.tool_calls.as_ref().unwrap();
    assert_eq!(calls[MCP_TOOL], 1);
    assert_eq!(calls["Read"], 1);
    assert_eq!(
        reply.mcp_results,
        ["found record:lab_chunk:c1"],
        "only MCP tools' replies, so a file that mentions an identity is not a retrieval"
    );
    assert_eq!(reply.raw, stream.into_bytes(), "the whole stream is kept");

    let unreadable = parse_claude_output(b"not json".to_vec(), Some(0), false);
    assert_eq!(
        unreadable.tool_calls, None,
        "unknown, which is not the same as none"
    );
    assert_eq!(unreadable.cost_usd, None);
}

#[test]
fn what_retrieval_returned_is_kept_apart_from_what_the_agent_used() {
    let plan = planned(&pilot());
    // The tool returned one needed file and one other, and the plan names neither needed file.
    let (run, _) = run_key(&plan, &Scripted::replying(WRONG), &key(&plan, "helper", 0)).unwrap();
    let retrieval = run.retrieval.clone().unwrap();
    assert_eq!(retrieval.calls, 1);
    assert_eq!(retrieval.files_returned, 2);
    assert_eq!(retrieval.needed_files_returned, 1);
    assert_eq!(retrieval.needed_files_used, 0, "returned and not acted on");

    let (run, _) = run_key(&plan, &Scripted::replying(GOOD), &key(&plan, "helper", 0)).unwrap();
    let retrieval = run.retrieval.unwrap();
    assert_eq!(retrieval.needed_files_returned, 1);
    assert_eq!(retrieval.needed_files_used, 1);

    let (run, _) = run_key(&plan, &Scripted::replying(GOOD), &key(&plan, "alone", 0)).unwrap();
    assert_eq!(
        run.retrieval, None,
        "an arm with no such tool has none to report"
    );
}

#[test]
fn a_finished_run_missing_any_required_field_fails_the_audit_and_does_not_count() {
    let plan = planned(&pilot());
    let breaks: [(&str, Damage); 7] = [
        ("the model that answered", |reply| {
            reply.model_reported = None
        }),
        ("the provider's version", |reply| {
            reply.provider_version = None
        }),
        ("tokens", |reply| reply.output_tokens = None),
        ("cache tokens", |reply| reply.cache_read_tokens = None),
        ("cost", |reply| reply.cost_usd = None),
        ("turns", |reply| reply.turns = None),
        ("tool calls", |reply| reply.tool_calls = None),
    ];
    for (label, damage) in breaks {
        let mut provider = Scripted::replying(GOOD);
        damage(&mut provider.0);
        let mut runs = vec![
            run_key(&plan, &provider, &key(&plan, "alone", 0))
                .unwrap()
                .0,
        ];
        for arm in ["floor", "helper"] {
            let good = Scripted::replying(GOOD);
            runs.push(run_key(&plan, &good, &key(&plan, arm, 0)).unwrap().0);
        }
        let audit = audit_runs(&plan, &runs).unwrap();
        assert!(!audit.complete, "{label}");
        assert!(
            audit.problems.iter().any(|problem| problem.contains(label)),
            "{label}: {:?}",
            audit.problems
        );
        let report = combine_task_runs(&plan, &runs, Vec::new(), 1).unwrap();
        let system = report
            .systems
            .iter()
            .find(|system| system.id.as_str() == "alone.sonnet")
            .unwrap();
        assert_eq!(system.status, SystemStatus::Failed, "{label}");
        assert!(
            report
                .observations
                .iter()
                .any(|entry| entry.metric.as_str() == "runs_not_counted"),
            "{label}"
        );
    }
}

#[test]
fn missing_capture_in_one_scheduled_run_keeps_the_system_partial() {
    let plan = planned(&pilot());
    let mut runs = run_all(&plan, |_| GOOD);
    let damaged = runs
        .iter_mut()
        .find(|run| run.arm.as_str() == "helper")
        .unwrap();
    damaged.cost_usd = None;
    let report = combine_task_runs(&plan, &runs, Vec::new(), 1).unwrap();
    let system = report
        .systems
        .iter()
        .find(|system| system.id.as_str() == "helper.sonnet")
        .unwrap();
    assert_eq!(system.status, SystemStatus::Partial);
    let verdict = report
        .verdicts
        .iter()
        .find(|verdict| verdict.id.as_str() == "helps-helper.sonnet")
        .unwrap();
    assert_eq!(
        verdict.outcome,
        VerdictOutcome::NotRun,
        "pilots cannot claim a win"
    );
    assert!(verdict.basis.contains("4 of 5 scheduled pairs"));
}

#[test]
fn a_different_model_answering_fails_the_audit() {
    let plan = planned(&pilot());
    let mut provider = Scripted::replying(GOOD);
    provider.0.model_reported = Some("claude-haiku-5-5".into());
    let mut runs = vec![
        run_key(&plan, &provider, &key(&plan, "alone", 0))
            .unwrap()
            .0,
    ];
    for arm in ["floor", "helper"] {
        let good = Scripted::replying(GOOD);
        runs.push(run_key(&plan, &good, &key(&plan, arm, 0)).unwrap().0);
    }
    let audit = audit_runs(&plan, &runs).unwrap();
    assert!(!audit.complete);
    assert!(
        audit
            .problems
            .iter()
            .any(|problem| problem.contains("claude-haiku-5-5")),
        "{:?}",
        audit.problems
    );
}

#[test]
fn a_failed_run_is_a_result_and_owes_no_data() {
    let plan = planned(&pilot());
    let mut provider = Scripted::replying(GOOD);
    provider.0.exit_code = Some(1);
    provider.0.cost_usd = None;
    let (run, _) = run_key(&plan, &provider, &key(&plan, "alone", 0)).unwrap();
    assert_eq!(run.outcome, TaskRunOutcome::ProviderFailed);
    let audit = audit_runs(&plan, &[run]).unwrap();
    let entry = audit
        .arms
        .iter()
        .find(|arm| arm.arm.as_str() == "alone")
        .unwrap();
    assert_eq!(entry.runs_failed, 1);
    assert_eq!(entry.runs_counted, 0);
    assert!(audit.problems.is_empty(), "{:?}", audit.problems);
    assert!(!audit.complete, "and it proves nothing about capture");
}

#[test]
fn a_pilot_cannot_say_an_arm_helped_however_large_the_difference() {
    let mut config = pilot();
    config.tasks = TaskSelection::Seeded { count: 8, seed: 11 };
    config.spend.as_mut().unwrap().max_usd_total = 48.0;
    let plan = planned(&config);
    let runs = run_all(&plan, |arm| if arm == "helper" { GOOD } else { WRONG });
    let report = combine_task_runs(&plan, &runs, Vec::new(), 1).unwrap();
    let verdict = report
        .verdicts
        .iter()
        .find(|verdict| verdict.id.as_str() == "helps-helper.sonnet")
        .unwrap();
    assert_eq!(verdict.outcome, VerdictOutcome::NotRun);
    assert!(
        verdict.basis.starts_with("A pilot tests the test"),
        "{}",
        verdict.basis
    );
    assert!(report
        .limits
        .iter()
        .any(|limit| limit.contains("This is a pilot")));
}

#[test]
fn a_capture_proof_has_to_cover_the_confirmatory_study_it_is_named_by() {
    let plan = planned(&pilot());
    let proof = audit_runs(&plan, &run_all(&plan, |_| GOOD)).unwrap();
    assert!(proof.complete, "{:?}", proof.problems);

    let mut config = pilot();
    config.stage = StudyStage::Confirmatory;
    config.tasks = TaskSelection::All;
    config.spend.as_mut().unwrap().max_usd_total = 180.0;
    config.capture_proof_sha256 = Some(proof.digest().unwrap());
    let check = check_study(&config, &catalog(), &requirements(), Some(&proof)).unwrap();
    assert!(check.ready, "{:?}", check.outcomes);
    assert_eq!(status(&check, "capture-proven"), RequirementStatus::Met);
    assert!(check.can_reach_significance);
    assert_eq!(check.runs, 90);

    let mut other_model = config.clone();
    other_model.models[0].model = "claude-opus-5-5".into();
    let check = check_study(&other_model, &catalog(), &requirements(), Some(&proof)).unwrap();
    assert_eq!(status(&check, "capture-proven"), RequirementStatus::NotMet);
    assert!(detail(&check, "capture-proven").contains("claude-opus-5-5"));

    let mut other_proof = proof.clone();
    other_proof.runs_counted += 1;
    assert!(
        check_study(&config, &catalog(), &requirements(), Some(&other_proof)).is_err(),
        "the audit has to be the one this study names"
    );

    let mut incomplete = proof.clone();
    incomplete.arms[0].proven = false;
    config.capture_proof_sha256 = Some(incomplete.digest().unwrap());
    let check = check_study(&config, &catalog(), &requirements(), Some(&incomplete)).unwrap();
    assert_eq!(status(&check, "capture-proven"), RequirementStatus::NotMet);
}

#[test]
fn repeated_tasks_cannot_claim_confirmatory_significance_without_clustered_analysis() {
    let mut config = pilot();
    config.stage = StudyStage::Confirmatory;
    config.tasks = TaskSelection::All;
    config.repetitions = 2;
    config.spend.as_mut().unwrap().max_usd_total = 360.0;
    let check = check_study(&config, &catalog(), &requirements(), None).unwrap();
    assert!(!check.can_reach_significance);
    assert_eq!(
        status(&check, "can-reach-significance"),
        RequirementStatus::NotMet
    );
    assert!(!check.ready);
}

#[test]
fn a_study_goes_from_its_choices_to_a_report_with_no_provider_process() {
    let plan = planned(&pilot());
    let runs = run_all(&plan, |arm| if arm == "floor" { WRONG } else { GOOD });
    assert_eq!(runs.len(), 15);
    let audit = audit_runs(&plan, &runs).unwrap();
    assert!(audit.complete, "{:?}", audit.problems);
    assert_eq!(audit.runs_present, 15);
    assert_eq!(audit.runs_expected, 15);
    assert_eq!(audit.runs_counted, 15);
    for arm in &audit.arms {
        assert!(arm.proven);
        assert_eq!(arm.fields.len(), 9);
        for field in &arm.fields {
            assert_eq!(field.missing, 0, "{:?}", field.field);
        }
    }
    let report = combine_task_runs(&plan, &runs, Vec::new(), 1).unwrap();
    report.validate().unwrap();
    let floor = report
        .verdicts
        .iter()
        .find(|verdict| verdict.id.as_str() == "floor-match-sonnet")
        .unwrap();
    assert_eq!(floor.outcome, VerdictOutcome::Met);
    assert!(
        report
            .observations
            .iter()
            .any(|entry| entry.metric.as_str() == "returned_used"),
        "what was returned and what was used both reach the report"
    );
}

#[test]
fn the_checked_in_pilot_chooses_no_model_so_nothing_can_spend() {
    let config: StudyConfig =
        serde_json::from_str(&lab_file("studies/rrflow-planning-pilot.json")).unwrap();
    let catalog: TaskComparisonManifest =
        serde_json::from_str(&lab_file("tasks/rrflow-package-planning.json")).unwrap();
    assert_eq!(
        config.catalog.sha256,
        catalog.digest().unwrap(),
        "this study was configured against an older catalog"
    );
    assert_eq!(config.stage, StudyStage::Pilot);
    let check = check_study(&config, &catalog, &requirements(), None).unwrap();
    assert!(!check.ready);
    assert_eq!(check.tasks, 5);
    assert_eq!(check.arms, 3);
    for unmet in ["pinned-model", "spend-covers-plan", "credential-by-name"] {
        assert_eq!(status(&check, unmet), RequirementStatus::NotMet, "{unmet}");
    }
    assert!(plan_study(&config, &catalog, &requirements(), None).is_err());
}
