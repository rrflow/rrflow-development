//! Does a capability help an agent do real work, at what cost? Each task is a real change
//! with a known answer. The design is frozen before any run, runs are paired across arms,
//! and the plan the agent wrote is graded mechanically, because a tool that returned the right
//! thing proves nothing if the agent did not use it. Results use the lab's ordinary report.
use super::manifest::Availability;
use super::provider::{AgentProvider, AgentReply, AgentRequest};
use super::report::{
    ArtifactReference, DimensionDefinition, LabEvidenceClass, LabReport, MetricDefinition,
    MetricDirection, Observation, ReportSystem, SystemStatus, TestIdentity, Verdict,
    VerdictOutcome, ViewDefinition, REPORT_SCHEMA_VERSION,
};
use super::{canonical_json, require, sha256_hex, Result};
use crate::storage::{Digest, Id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const TASKS_SCHEMA_VERSION: u16 = 1;

/// A pilot tests the test, so its report cannot answer the question. A confirmatory study can.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StudyStage {
    Pilot,
    #[default]
    Confirmatory,
}

/// Data a study requires of every finished run.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CaptureField {
    Model,
    ProviderVersion,
    Tokens,
    CacheTokens,
    Cost,
    Turns,
    ToolCalls,
    Retrieval,
    Transcript,
}

impl CaptureField {
    pub fn label(self) -> &'static str {
        match self {
            Self::Model => "the model that answered",
            Self::ProviderVersion => "the provider's version",
            Self::Tokens => "tokens",
            Self::CacheTokens => "cache tokens",
            Self::Cost => "cost",
            Self::Turns => "turns",
            Self::ToolCalls => "tool calls",
            Self::Retrieval => "what retrieval returned",
            Self::Transcript => "the transcript",
        }
    }
}
pub const MAX_TASKS: usize = 1_024;
pub const MAX_REPETITIONS: u32 = 32;
const Z95: f64 = 1.96;

/// Frozen before the first run. Changing any of it starts a different study.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Preregistration {
    pub question: String,
    /// The arm every other arm is compared with, pair by pair.
    pub baseline_arm: Id,
    /// The arm with no access to the repository. If it scores well, the tasks leak.
    pub floor_arm: Id,
    pub repetitions: u32,
    pub order_seed: u64,
    pub alpha: f64,
    /// A plan counts as solved only at or above both of these.
    pub min_file_recall: f64,
    pub min_file_precision: f64,
    pub max_floor_solve_rate: f64,
    #[serde(default)]
    pub stage: StudyStage,
    /// A finished run lacking one of these stays visible and does not count.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_captures: Vec<CaptureField>,
}

/// A run cannot start without these, and the person who pays decides them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpendLimits {
    pub max_seconds_per_run: u64,
    pub max_usd_per_run: f64,
    pub max_usd_total: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskProviderKind {
    ClaudeCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskModel {
    pub id: Id,
    pub provider: TaskProviderKind,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskArm {
    pub id: Id,
    pub label: String,
    pub availability: Availability,
    /// False runs the agent in an empty directory.
    pub repository_access: bool,
    /// Tool names the agent may use. Empty means none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    /// Text placed ahead of the task, the same for every task in the arm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preface: Option<String>,
    /// What a reader must know about this arm's results, repeated in the report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentTask {
    pub id: Id,
    pub title: String,
    pub repository: String,
    /// The revision the change started from, which holds nothing of its answer.
    pub revision: String,
    pub prompt: String,
    /// The answer key. It stays in the harness and never reaches the agent.
    pub expected_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_records: Vec<String>,
    /// Where the task and its answer came from.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskComparisonManifest {
    pub schema_version: u16,
    pub test: TestIdentity,
    pub preregistration: Preregistration,
    /// Empty until the owner chooses, and nothing can run while it is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<TaskModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spend: Option<SpendLimits>,
    pub arms: Vec<TaskArm>,
    pub tasks: Vec<AgentTask>,
}

/// The file name a path ends in, so a prompt cannot give the answer away that way either.
fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

impl TaskComparisonManifest {
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == TASKS_SCHEMA_VERSION,
            "unsupported task manifest schema version",
        )?;
        let plan = &self.preregistration;
        require(
            !plan.question.trim().is_empty(),
            "the preregistration must state its question",
        )?;
        require(
            (1..=MAX_REPETITIONS).contains(&plan.repetitions),
            "repetitions are outside bounds",
        )?;
        for (value, label) in [
            (plan.alpha, "alpha"),
            (plan.min_file_recall, "file recall threshold"),
            (plan.min_file_precision, "file precision threshold"),
            (plan.max_floor_solve_rate, "floor threshold"),
        ] {
            require(
                value > 0.0 && value < 1.0,
                &format!("{label} must be between 0 and 1"),
            )?;
        }
        let mut arms = BTreeSet::new();
        for arm in &self.arms {
            let id = arm.id.as_str();
            require(arms.insert(&arm.id), &format!("{id} is listed twice"))?;
            require(!arm.label.trim().is_empty(), &format!("{id} needs a label"))?;
            if let Availability::Unavailable { reason }
            | Availability::Unimplemented { reason }
            | Availability::NotApplicable { reason } = &arm.availability
            {
                require(
                    !reason.trim().is_empty(),
                    &format!("{id} must say why it does not run"),
                )?;
            }
            require(
                arm.repository_access || arm.tools.is_empty(),
                &format!("{id} has no repository, so it cannot have tools"),
            )?;
        }
        for (role, id) in [("baseline", &plan.baseline_arm), ("floor", &plan.floor_arm)] {
            let arm = self
                .arms
                .iter()
                .find(|arm| &arm.id == id)
                .ok_or_else(|| format!("the {role} arm is not listed"))?;
            require(
                arm.availability == Availability::Available,
                &format!("the {role} arm must be able to run"),
            )?;
        }
        let floor = self.arm(plan.floor_arm.as_str())?;
        require(
            !floor.repository_access,
            "the floor arm must not see the repository",
        )?;
        require(
            self.arm(plan.baseline_arm.as_str())?.repository_access,
            "the baseline arm must see the repository",
        )?;
        let mut models = BTreeSet::new();
        for model in &self.models {
            require(models.insert(&model.id), "a model is listed twice")?;
            require(!model.model.trim().is_empty(), "a model must be named")?;
        }
        if let Some(spend) = &self.spend {
            require(
                spend.max_seconds_per_run > 0
                    && spend.max_usd_per_run > 0.0
                    && spend.max_usd_total >= spend.max_usd_per_run,
                "spend limits must be positive, and the total must cover one run",
            )?;
        }
        require(
            !self.tasks.is_empty() && self.tasks.len() <= MAX_TASKS,
            "task count is outside bounds",
        )?;
        let mut tasks = BTreeSet::new();
        for task in &self.tasks {
            let id = task.id.as_str();
            require(tasks.insert(&task.id), &format!("{id} is listed twice"))?;
            require(
                !task.prompt.trim().is_empty() && !task.expected_files.is_empty(),
                &format!("{id} needs a prompt and an answer"),
            )?;
            require(
                task.revision.len() == 40
                    && task.revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
                &format!("{id} must pin a full revision"),
            )?;
            for path in &task.expected_files {
                require(
                    !task.prompt.contains(path.as_str()) && !task.prompt.contains(file_name(path)),
                    &format!("{id}'s prompt gives away {path}"),
                )?;
            }
        }
        Ok(())
    }

    pub fn arm(&self, id: &str) -> Result<&TaskArm> {
        self.arms
            .iter()
            .find(|arm| arm.id.as_str() == id)
            .ok_or_else(|| format!("no arm named {id}"))
    }

    /// Refuses unless the people paying have chosen models and a limit.
    pub fn require_runnable(&self) -> Result<&SpendLimits> {
        require(
            !self.models.is_empty(),
            "no model is chosen, so nothing can run",
        )?;
        require(
            !self.preregistration.required_captures.is_empty(),
            "the study requires no data of its runs, so nothing can run",
        )?;
        self.spend
            .as_ref()
            .ok_or_else(|| "no spend limit is set, so nothing can run".to_string())
    }

    pub fn digest(&self) -> Result<Digest> {
        Digest::try_from(sha256_hex(&canonical_json(self)?))
    }
}

/// One run in the frozen plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunKey {
    pub task: Id,
    pub arm: Id,
    pub model: Id,
    pub repetition: u32,
}

/// Every run in a seeded shuffle, so arms spread evenly over time of day and any service change.
pub fn run_order(manifest: &TaskComparisonManifest) -> Vec<RunKey> {
    let mut keys = Vec::new();
    for task in &manifest.tasks {
        for arm in manifest
            .arms
            .iter()
            .filter(|arm| arm.availability == Availability::Available)
        {
            for model in &manifest.models {
                for repetition in 1..=manifest.preregistration.repetitions {
                    keys.push(RunKey {
                        task: task.id.clone(),
                        arm: arm.id.clone(),
                        model: model.id.clone(),
                        repetition,
                    });
                }
            }
        }
    }
    shuffle(&mut keys, manifest.preregistration.order_seed);
    keys
}

/// A shuffle that a seed fully decides.
pub fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed;
    for index in (1..items.len()).rev() {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        items.swap(index, (z % (index as u64 + 1)) as usize);
    }
}

/// What the agent was asked to reply with. Unknown fields are ignored, so a longer reply
/// is still graded on what it names.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct AgentPlan {
    pub files: Vec<String>,
    pub records: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub open_questions: Vec<String>,
}

const REPLY_FORMAT: &str = "Plan this change. Do not change any file. Reply with one JSON object and nothing else:\n{\"files\": [\"each repository path the change would edit or add\"], \"records\": [\"identifiers of the records that own this work, such as OBJ-11, POAM-046 or E-04\"], \"stop_conditions\": [\"what should stop the work\"], \"open_questions\": [\"what you would need answered first\"]}";

pub fn task_prompt(task: &AgentTask, arm: &TaskArm) -> String {
    let mut prompt = String::new();
    if let Some(preface) = &arm.preface {
        prompt.push_str(preface);
        prompt.push_str("\n\n");
    }
    prompt.push_str(&task.prompt);
    prompt.push_str("\n\n");
    prompt.push_str(REPLY_FORMAT);
    prompt
}

/// The last JSON object in a reply with a `files` list, whether or not it is fenced.
pub fn parse_plan(reply: &str) -> Option<AgentPlan> {
    let trimmed = reply.trim();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if value.get("files").is_some() {
            return serde_json::from_value(value).ok();
        }
    }
    let mut found = None;
    for (start, _) in trimmed.match_indices('{') {
        let mut stream =
            serde_json::Deserializer::from_str(&trimmed[start..]).into_iter::<serde_json::Value>();
        if let Some(Ok(value)) = stream.next() {
            if value.get("files").is_some() {
                if let Ok(plan) = serde_json::from_value::<AgentPlan>(value) {
                    found = Some(plan);
                }
            }
        }
    }
    found
}

/// Agents write paths several ways, and none of them should change a score.
pub fn normalize_path(path: &str) -> String {
    let forward = path.trim().trim_matches('`').replace('\\', "/");
    let relative = forward.trim_start_matches("./").trim_start_matches('/');
    let bare = relative.split(['#', ' ']).next().unwrap_or(relative);
    // A line reference such as `reader.rs:42` names the same file.
    match bare.rsplit_once(':') {
        Some((file, line)) if !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()) => {
            file.to_string()
        }
        _ => bare.to_string(),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanScore {
    pub files_named: u64,
    pub files_matched: u64,
    pub files_expected: u64,
    pub file_recall: f64,
    pub file_precision: f64,
    pub records_cited: u64,
    pub records_expected: u64,
    pub solved: bool,
}

pub fn score_plan(task: &AgentTask, plan: &AgentPlan, rules: &Preregistration) -> PlanScore {
    let expected: BTreeSet<String> = task
        .expected_files
        .iter()
        .map(|path| normalize_path(path))
        .collect();
    let named: BTreeSet<String> = plan.files.iter().map(|path| normalize_path(path)).collect();
    let matched = named.intersection(&expected).count() as u64;
    let file_recall = matched as f64 / expected.len().max(1) as f64;
    let file_precision = if named.is_empty() {
        0.0
    } else {
        matched as f64 / named.len() as f64
    };
    let records_cited = task
        .expected_records
        .iter()
        .filter(|record| {
            plan.records
                .iter()
                .any(|cited| cited.eq_ignore_ascii_case(record))
        })
        .count() as u64;
    PlanScore {
        files_named: named.len() as u64,
        files_matched: matched,
        files_expected: expected.len() as u64,
        file_recall,
        file_precision,
        records_cited,
        records_expected: task.expected_records.len() as u64,
        solved: file_recall >= rules.min_file_recall && file_precision >= rules.min_file_precision,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskRunOutcome {
    /// It replied with a plan, which was graded.
    Planned,
    /// It replied without a readable plan, which counts as not solved.
    NoPlan,
    /// It hit its time limit, which counts as not solved.
    TimedOut,
    /// The provider failed, which counts as not solved and stays visible.
    ProviderFailed,
    /// The arm's own tools did not come up, so this measured nothing about the arm.
    ToolingFailed,
}

/// What an MCP tool gave the agent and what the agent did with it, separately, because a
/// correct result that goes unused is a different failure from a search that missed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalCapture {
    pub calls: u64,
    pub bytes_returned: u64,
    pub files_returned: u64,
    /// Files the change needed that the tool returned.
    pub needed_files_returned: u64,
    /// Of those, the ones the plan named.
    pub needed_files_used: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskRun {
    pub schema_version: u16,
    pub manifest_sha256: Digest,
    pub task: Id,
    pub arm: Id,
    pub model: Id,
    pub repetition: u32,
    pub outcome: TaskRunOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub score: PlanScore,
    pub seconds: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turns: Option<u64>,
    /// The provider's whole output, kept beside the run under this digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_sha256: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_reported: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<BTreeMap<String, u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<RetrievalCapture>,
}

impl TaskRun {
    /// What a finished run lacks. A run that failed is a result and owes nothing further.
    pub fn missing_captures(&self, arm: &TaskArm, required: &[CaptureField]) -> Vec<CaptureField> {
        if !matches!(
            self.outcome,
            TaskRunOutcome::Planned | TaskRunOutcome::NoPlan
        ) {
            return Vec::new();
        }
        required
            .iter()
            .copied()
            .filter(|field| !self.holds(*field, arm))
            .collect()
    }

    fn holds(&self, field: CaptureField, arm: &TaskArm) -> bool {
        match field {
            CaptureField::Model => self.model_reported.is_some(),
            CaptureField::ProviderVersion => self.provider_version.is_some(),
            CaptureField::Tokens => self.input_tokens.is_some() && self.output_tokens.is_some(),
            CaptureField::CacheTokens => {
                self.cache_read_tokens.is_some() && self.cache_write_tokens.is_some()
            }
            CaptureField::Cost => self.cost_usd.is_some(),
            CaptureField::Turns => self.turns.is_some(),
            CaptureField::ToolCalls => self.tool_calls.is_some(),
            CaptureField::Retrieval => {
                !arm.tools.iter().any(|tool| tool.starts_with("mcp__")) || self.retrieval.is_some()
            }
            CaptureField::Transcript => self.reply_sha256.is_some(),
        }
    }

    /// Whether this run can count for or against its arm.
    pub fn counts(&self, arm: &TaskArm, required: &[CaptureField]) -> bool {
        self.outcome != TaskRunOutcome::ToolingFailed
            && self.missing_captures(arm, required).is_empty()
    }
}

/// What a run took from the total. A provider that reported no cost is charged at the run's limit.
pub fn charged_usd(run: &TaskRun, _spend: &SpendLimits) -> f64 {
    run.cost_usd.unwrap_or(0.0)
}

/// Which files an MCP tool returned, read from the chunk identities in its replies. Any shape
/// of reply works, and a reply without identities shows as nothing returned.
fn capture_retrieval(
    task: &AgentTask,
    reply: &AgentReply,
    chunk_files: &BTreeMap<String, String>,
    plan: Option<&AgentPlan>,
) -> RetrievalCapture {
    let mut returned = BTreeSet::new();
    for text in &reply.mcp_results {
        for token in text.split(|character: char| !character.is_ascii_alphanumeric()) {
            if let Some(path) = chunk_files.get(token) {
                returned.insert(normalize_path(path));
            }
        }
    }
    let named: BTreeSet<String> = plan
        .map(|plan| plan.files.iter().map(|path| normalize_path(path)).collect())
        .unwrap_or_default();
    let needed: BTreeSet<String> = task
        .expected_files
        .iter()
        .map(|path| normalize_path(path))
        .collect();
    let found: Vec<&String> = needed.intersection(&returned).collect();
    RetrievalCapture {
        calls: reply
            .tool_calls
            .iter()
            .flatten()
            .filter(|(name, _)| name.starts_with("mcp__"))
            .map(|(_, calls)| *calls)
            .sum(),
        bytes_returned: reply.mcp_results.iter().map(|text| text.len() as u64).sum(),
        files_returned: returned.len() as u64,
        needed_files_returned: found.len() as u64,
        needed_files_used: found.iter().filter(|path| named.contains(**path)).count() as u64,
    }
}

pub struct RunInputs<'a> {
    pub provider: &'a dyn AgentProvider,
    /// The task's repository at its revision.
    pub checkout: &'a Path,
    /// An empty directory, for an arm that must not see the repository.
    pub empty_directory: &'a Path,
    /// Spent so far in this study, which a run must not push past the total.
    pub spent_usd: f64,
    /// Where an arm with MCP tools finds its server.
    pub mcp_config: Option<&'a Path>,
    /// Each imported chunk's file, so what an MCP tool returned can be read.
    pub chunk_files: Option<&'a BTreeMap<String, String>>,
}

/// Runs one key from the plan. A provider's failure is a result and comes back as a run.
pub fn run_task(
    manifest: &TaskComparisonManifest,
    key: &RunKey,
    inputs: &RunInputs<'_>,
) -> Result<(TaskRun, Vec<u8>)> {
    manifest.validate()?;
    let spend = manifest.require_runnable()?;
    require(
        inputs.spent_usd + spend.max_usd_per_run <= spend.max_usd_total,
        "another run could pass the total spend limit",
    )?;
    let arm = manifest.arm(key.arm.as_str())?;
    require(
        arm.availability == Availability::Available,
        &format!("{} is declared as not running", arm.id.as_str()),
    )?;
    let uses_mcp = arm.tools.iter().any(|tool| tool.starts_with("mcp__"));
    require(
        uses_mcp == inputs.mcp_config.is_some(),
        &format!(
            "{} must have an MCP configuration exactly when it uses an MCP tool",
            arm.id.as_str()
        ),
    )?;
    require(
        uses_mcp == inputs.chunk_files.is_some(),
        &format!(
            "{} must have a chunk index exactly when it uses an MCP tool",
            arm.id.as_str()
        ),
    )?;
    let task = manifest
        .tasks
        .iter()
        .find(|task| task.id == key.task)
        .ok_or_else(|| format!("no task named {}", key.task.as_str()))?;
    let model = manifest
        .models
        .iter()
        .find(|model| model.id == key.model)
        .ok_or_else(|| format!("no model named {}", key.model.as_str()))?;
    let prompt = task_prompt(task, arm);
    let started = std::time::Instant::now();
    let reply = inputs.provider.run(&AgentRequest {
        prompt: &prompt,
        directory: if arm.repository_access {
            inputs.checkout
        } else {
            inputs.empty_directory
        },
        model: &model.model,
        effort: model.effort.as_deref(),
        tools: &arm.tools,
        mcp_config: inputs.mcp_config,
        max_seconds: spend.max_seconds_per_run,
        max_usd: spend.max_usd_per_run,
    });
    let seconds = started.elapsed().as_secs_f64();
    let manifest_sha256 = manifest.digest()?;
    let blank = |outcome, reason: Option<String>, reply: &AgentReply| TaskRun {
        schema_version: TASKS_SCHEMA_VERSION,
        manifest_sha256: manifest_sha256.clone(),
        task: key.task.clone(),
        arm: key.arm.clone(),
        model: key.model.clone(),
        repetition: key.repetition,
        outcome,
        reason,
        score: PlanScore {
            files_expected: task.expected_files.len() as u64,
            records_expected: task.expected_records.len() as u64,
            ..PlanScore::default()
        },
        seconds,
        input_tokens: reply.input_tokens,
        output_tokens: reply.output_tokens,
        cost_usd: reply.cost_usd,
        turns: reply.turns,
        reply_sha256: None,
        model_reported: reply.model_reported.clone(),
        provider_version: reply.provider_version.clone(),
        cache_read_tokens: reply.cache_read_tokens,
        cache_write_tokens: reply.cache_write_tokens,
        tool_calls: reply.tool_calls.clone(),
        retrieval: None,
    };
    let reply = match reply {
        Ok(reply) => reply,
        Err(error) => {
            return Ok((
                blank(
                    TaskRunOutcome::ProviderFailed,
                    Some(error),
                    &AgentReply::default(),
                ),
                Vec::new(),
            ))
        }
    };
    let digest = Digest::try_from(sha256_hex(&reply.raw))?;
    let mut run = if reply.timed_out {
        blank(TaskRunOutcome::TimedOut, None, &reply)
    } else if reply.exit_code != Some(0) {
        blank(
            TaskRunOutcome::ProviderFailed,
            Some(format!("the provider exited with {:?}", reply.exit_code)),
            &reply,
        )
    } else if let Some(plan) = parse_plan(&reply.text) {
        let mut run = blank(TaskRunOutcome::Planned, None, &reply);
        run.score = score_plan(task, &plan, &manifest.preregistration);
        run.retrieval = inputs
            .chunk_files
            .map(|chunks| capture_retrieval(task, &reply, chunks, Some(&plan)));
        run
    } else {
        let mut run = blank(TaskRunOutcome::NoPlan, None, &reply);
        run.retrieval = inputs
            .chunk_files
            .map(|chunks| capture_retrieval(task, &reply, chunks, None));
        run
    };
    run.reply_sha256 = Some(digest);
    Ok((run, reply.raw))
}

/// Wilson's interval, which stays sensible at few runs and at rates near 0 or 1.
pub fn wilson_interval(successes: u64, total: u64) -> (f64, f64) {
    if total == 0 {
        return (0.0, 0.0);
    }
    let n = total as f64;
    let p = successes as f64 / n;
    let z2 = Z95 * Z95;
    let center = p + z2 / (2.0 * n);
    let spread = Z95 * ((p * (1.0 - p) + z2 / (4.0 * n)) / n).sqrt();
    let denominator = 1.0 + z2 / n;
    (
        ((center - spread) / denominator).max(0.0),
        ((center + spread) / denominator).min(1.0),
    )
}

/// Two-sided exact McNemar test on the pairs where the two arms disagreed.
pub fn exact_mcnemar(only_first: u64, only_second: u64) -> f64 {
    let n = only_first + only_second;
    if n == 0 {
        return 1.0;
    }
    let smaller = only_first.min(only_second);
    let mut log_term = -(n as f64) * std::f64::consts::LN_2;
    let mut tail = log_term.exp();
    for i in 1..=smaller {
        log_term += ((n - i + 1) as f64).ln() - (i as f64).ln();
        tail += log_term.exp();
    }
    (2.0 * tail).min(1.0)
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    Some(if values.len() % 2 == 1 {
        values[middle]
    } else {
        (values[middle - 1] + values[middle]) / 2.0
    })
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

fn id(text: &str) -> Result<Id> {
    Id::try_from(text.to_string())
}

fn system_id(arm: &Id, model: &Id) -> Result<Id> {
    id(&format!("{}.{}", arm.as_str(), model.as_str()))
}

fn metric(key: &str, label: &str, unit: &str, better: MetricDirection) -> Result<MetricDefinition> {
    Ok(MetricDefinition {
        id: id(key)?,
        label: label.into(),
        unit: unit.into(),
        better,
    })
}

fn observation(
    system: &Id,
    metric_id: &str,
    case: BTreeMap<String, String>,
    value: f64,
    samples: u64,
    interval: Option<(f64, f64)>,
) -> Result<Observation> {
    Ok(Observation {
        system: system.clone(),
        metric: id(metric_id)?,
        case,
        value,
        samples,
        low: interval.map(|bounds| bounds.0),
        high: interval.map(|bounds| bounds.1),
    })
}

fn system_observations(system: &Id, runs: &[&TaskRun], list: &mut Vec<Observation>) -> Result<()> {
    let total = runs.len() as u64;
    let solved = runs.iter().filter(|run| run.score.solved).count() as u64;
    let none = BTreeMap::new;
    list.push(observation(
        system,
        "solve_rate",
        none(),
        solved as f64 / total as f64,
        total,
        Some(wilson_interval(solved, total)),
    )?);
    let recalls: Vec<f64> = runs.iter().map(|run| run.score.file_recall).collect();
    let precisions: Vec<f64> = runs.iter().map(|run| run.score.file_precision).collect();
    list.push(observation(
        system,
        "file_recall",
        none(),
        mean(&recalls),
        total,
        None,
    )?);
    list.push(observation(
        system,
        "file_precision",
        none(),
        mean(&precisions),
        total,
        None,
    )?);
    let cited: u64 = runs.iter().map(|run| run.score.records_cited).sum();
    let expected: u64 = runs.iter().map(|run| run.score.records_expected).sum();
    if expected > 0 {
        list.push(observation(
            system,
            "records_cited",
            none(),
            cited as f64 / expected as f64,
            total,
            None,
        )?);
    }
    list.push(observation(
        system,
        "runs_without_a_plan",
        none(),
        runs.iter()
            .filter(|run| run.outcome != TaskRunOutcome::Planned)
            .count() as f64,
        total,
        None,
    )?);
    let retrievals: Vec<&RetrievalCapture> = runs
        .iter()
        .filter_map(|run| run.retrieval.as_ref())
        .collect();
    if !retrievals.is_empty() {
        let returned: u64 = retrievals
            .iter()
            .map(|retrieval| retrieval.needed_files_returned)
            .sum();
        let used: u64 = retrievals
            .iter()
            .map(|retrieval| retrieval.needed_files_used)
            .sum();
        let needed: u64 = runs
            .iter()
            .filter(|run| run.retrieval.is_some())
            .map(|run| run.score.files_expected)
            .sum();
        list.push(observation(
            system,
            "needed_returned",
            none(),
            returned as f64 / needed.max(1) as f64,
            needed,
            None,
        )?);
        if returned > 0 {
            list.push(observation(
                system,
                "returned_used",
                none(),
                used as f64 / returned as f64,
                returned,
                None,
            )?);
        }
    }
    let medians: [(&str, Vec<f64>); 6] = [
        ("seconds", runs.iter().map(|run| run.seconds).collect()),
        (
            "input_tokens",
            runs.iter()
                .filter_map(|run| run.input_tokens.map(|value| value as f64))
                .collect(),
        ),
        (
            "output_tokens",
            runs.iter()
                .filter_map(|run| run.output_tokens.map(|value| value as f64))
                .collect(),
        ),
        (
            "cost_usd",
            runs.iter().filter_map(|run| run.cost_usd).collect(),
        ),
        (
            "cache_read_tokens",
            runs.iter()
                .filter_map(|run| run.cache_read_tokens.map(|value| value as f64))
                .collect(),
        ),
        (
            "tool_calls",
            runs.iter()
                .filter_map(|run| {
                    run.tool_calls
                        .as_ref()
                        .map(|calls| calls.values().sum::<u64>() as f64)
                })
                .collect(),
        ),
    ];
    for (metric_id, values) in medians {
        let samples = values.len() as u64;
        if let Some(value) = median(values) {
            list.push(observation(
                system,
                metric_id,
                none(),
                value,
                samples,
                None,
            )?);
        }
    }
    let mut by_task: BTreeMap<&Id, (u64, u64)> = BTreeMap::new();
    for run in runs {
        let entry = by_task.entry(&run.task).or_default();
        entry.1 += 1;
        if run.score.solved {
            entry.0 += 1;
        }
    }
    for (task, (solved, total)) in by_task {
        list.push(observation(
            system,
            "solve_rate",
            BTreeMap::from([("task".to_string(), task.as_str().to_string())]),
            solved as f64 / total as f64,
            total,
            Some(wilson_interval(solved, total)),
        )?);
    }
    Ok(())
}

/// Compares an arm with the baseline only on runs that exist for both.
fn paired_verdict(
    manifest: &TaskComparisonManifest,
    runs: &[TaskRun],
    arm: &TaskArm,
    model: &TaskModel,
) -> Result<Verdict> {
    let rules = &manifest.preregistration;
    let lookup = |arm_id: &Id| -> BTreeMap<(&Id, u32), bool> {
        runs.iter()
            .filter(|run| &run.arm == arm_id && run.model == model.id)
            .map(|run| ((&run.task, run.repetition), run.score.solved))
            .collect()
    };
    let this_arm = lookup(&arm.id);
    let baseline = lookup(&rules.baseline_arm);
    let (mut pairs, mut only_arm, mut only_baseline) = (0_u64, 0_u64, 0_u64);
    for (key, solved) in &this_arm {
        if let Some(other) = baseline.get(key) {
            pairs += 1;
            match (solved, other) {
                (true, false) => only_arm += 1,
                (false, true) => only_baseline += 1,
                _ => {}
            }
        }
    }
    let p = exact_mcnemar(only_arm, only_baseline);
    let pilot = rules.stage == StudyStage::Pilot;
    let outcome = if pairs == 0 || pilot {
        VerdictOutcome::NotRun
    } else if only_arm > only_baseline && p < rules.alpha {
        VerdictOutcome::Met
    } else {
        VerdictOutcome::NotMet
    };
    let mut verdict = Verdict {
        id: id(&format!(
            "helps-{}",
            system_id(&arm.id, &model.id)?.as_str()
        ))?,
        question: format!(
            "Did {} solve more tasks than {} on the same tasks, at alpha {}?",
            arm.id.as_str(),
            rules.baseline_arm.as_str(),
            rules.alpha
        ),
        system: Some(system_id(&arm.id, &model.id)?),
        outcome,
        basis: if pairs == 0 {
            "no paired runs".to_string()
        } else {
            format!(
                "{pairs} pairs: only this arm solved {only_arm}, only the baseline solved {only_baseline}, exact McNemar p = {p:.4}. A result that is not significant does not show the arm has no effect."
            )
        },
    };
    if pilot {
        verdict.basis = format!(
            "A pilot tests the test and cannot answer this. {}",
            verdict.basis
        );
    }
    Ok(verdict)
}

fn floor_verdict(
    manifest: &TaskComparisonManifest,
    runs: &[TaskRun],
    model: &TaskModel,
) -> Result<Verdict> {
    let rules = &manifest.preregistration;
    let floor: Vec<&TaskRun> = runs
        .iter()
        .filter(|run| run.arm == rules.floor_arm && run.model == model.id)
        .collect();
    let solved = floor.iter().filter(|run| run.score.solved).count();
    let rate = solved as f64 / floor.len().max(1) as f64;
    Ok(Verdict {
        id: id(&format!("no-leak-{}", model.id.as_str()))?,
        question: format!(
            "Did the arm with no repository solve at most {} of the tasks, so the tasks do not leak?",
            rules.max_floor_solve_rate
        ),
        system: Some(system_id(&rules.floor_arm, &model.id)?),
        outcome: if floor.is_empty() {
            VerdictOutcome::NotRun
        } else if rate <= rules.max_floor_solve_rate {
            VerdictOutcome::Met
        } else {
            VerdictOutcome::NotMet
        },
        basis: if floor.is_empty() {
            "no floor runs".to_string()
        } else {
            format!("{solved} of {} solved", floor.len())
        },
    })
}

/// One report over every run, refusing any run made under a different frozen design.
pub fn combine_task_runs(
    manifest: &TaskComparisonManifest,
    runs: &[TaskRun],
    artifacts: Vec<ArtifactReference>,
    generated_at_unix_ms: u64,
) -> Result<LabReport> {
    manifest.validate()?;
    let digest = manifest.digest()?;
    let mut seen = BTreeSet::new();
    for run in runs {
        require(
            run.manifest_sha256 == digest,
            &format!("{} ran under a different frozen design", run.task.as_str()),
        )?;
        require(
            seen.insert((&run.task, &run.arm, &run.model, run.repetition)),
            "a run appears twice",
        )?;
        let arm = manifest.arm(run.arm.as_str())?;
        require(
            arm.availability == Availability::Available,
            &format!(
                "{} is declared as not running but has a run",
                arm.id.as_str()
            ),
        )?;
    }
    let required = &manifest.preregistration.required_captures;
    let counted: Vec<TaskRun> = runs
        .iter()
        .filter(|run| {
            manifest
                .arm(run.arm.as_str())
                .is_ok_and(|arm| run.counts(arm, required))
        })
        .cloned()
        .collect();
    let mut systems = Vec::new();
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for arm in &manifest.arms {
        let declared = match &arm.availability {
            Availability::Available => None,
            Availability::Unavailable { reason } => Some((SystemStatus::Unavailable, reason)),
            Availability::Unimplemented { reason } => Some((SystemStatus::Unimplemented, reason)),
            Availability::NotApplicable { reason } => Some((SystemStatus::NotApplicable, reason)),
        };
        if let Some((status, reason)) = declared {
            systems.push(ReportSystem {
                id: arm.id.clone(),
                label: arm.label.clone(),
                kind: "agent-arm".into(),
                status,
                reason: Some(reason.clone()),
                observed_version: None,
                configuration: BTreeMap::new(),
            });
            continue;
        }
        if manifest.models.is_empty() {
            systems.push(ReportSystem {
                id: arm.id.clone(),
                label: arm.label.clone(),
                kind: "agent-arm".into(),
                status: SystemStatus::NotRun,
                reason: Some("no model is chosen yet".into()),
                observed_version: None,
                configuration: BTreeMap::new(),
            });
            continue;
        }
        for model in &manifest.models {
            let system = system_id(&arm.id, &model.id)?;
            let own: Vec<&TaskRun> = counted
                .iter()
                .filter(|run| run.arm == arm.id && run.model == model.id)
                .collect();
            let made = runs
                .iter()
                .filter(|run| run.arm == arm.id && run.model == model.id)
                .count();
            let status = if !own.is_empty() {
                SystemStatus::Completed
            } else if made > 0 {
                SystemStatus::Failed
            } else {
                SystemStatus::NotRun
            };
            systems.push(ReportSystem {
                id: system.clone(),
                label: format!("{}, {}", arm.label, model.model),
                kind: "agent-arm".into(),
                status,
                reason: own.is_empty().then(|| {
                    if made > 0 {
                        format!("{made} runs were made, and none can count: see the capture audit")
                    } else {
                        "no run file was supplied".to_string()
                    }
                }),
                observed_version: Some(model.model.clone()),
                configuration: BTreeMap::new(),
            });
            if !own.is_empty() {
                system_observations(&system, &own, &mut observations)?;
            }
            if made > own.len() {
                observations.push(observation(
                    &system,
                    "runs_not_counted",
                    BTreeMap::new(),
                    (made - own.len()) as f64,
                    made as u64,
                    None,
                )?);
            }
            if arm.id != manifest.preregistration.baseline_arm
                && arm.id != manifest.preregistration.floor_arm
            {
                verdicts.push(paired_verdict(manifest, &counted, arm, model)?);
            }
        }
    }
    for model in &manifest.models {
        verdicts.push(floor_verdict(manifest, &counted, model)?);
    }
    use MetricDirection::{Higher, Lower};
    let mut limits = vec![
        "Diagnostic evidence for optimization: not J-04 acceptance, and no parity or superiority claim.".to_string(),
        "A task's answer is the files one real change touched. Another valid plan can score lower, so compare arms pair by pair and do not read a solve rate as an absolute.".to_string(),
        "The thresholds for solved were frozen before any run, and they are a judgement.".to_string(),
        "Grading reads the plan. It does not check that the plan would work if carried out.".to_string(),
    ];
    if manifest.preregistration.stage == StudyStage::Pilot {
        limits.insert(
            0,
            "This is a pilot. It tests the test, and it cannot say whether an arm helps."
                .to_string(),
        );
    }
    for arm in &manifest.arms {
        for limit in &arm.limits {
            limits.push(format!("{}: {limit}", arm.id.as_str()));
        }
    }
    let report = LabReport {
        schema_version: REPORT_SCHEMA_VERSION,
        test: manifest.test.clone(),
        evidence_class: LabEvidenceClass::Diagnostic,
        generated_by: format!("rrflow-lab {}", env!("CARGO_PKG_VERSION")),
        generated_at_unix_ms,
        environment: BTreeMap::new(),
        identities: BTreeMap::from([("manifest".to_string(), digest)]),
        systems,
        dimensions: vec![DimensionDefinition {
            id: id("task")?,
            label: "Task".into(),
        }],
        metrics: vec![
            metric("solve_rate", "Tasks solved", "fraction", Higher)?,
            metric(
                "file_recall",
                "Needed files the plan named",
                "fraction",
                Higher,
            )?,
            metric(
                "file_precision",
                "Named files that were needed",
                "fraction",
                Higher,
            )?,
            metric("records_cited", "Owning records cited", "fraction", Higher)?,
            metric(
                "runs_without_a_plan",
                "Runs that gave no plan",
                "count",
                Lower,
            )?,
            metric(
                "runs_not_counted",
                "Runs that cannot count, for missing data or tools that never came up",
                "count",
                Lower,
            )?,
            metric(
                "needed_returned",
                "Needed files the tool returned",
                "fraction",
                Higher,
            )?,
            metric(
                "returned_used",
                "Returned needed files the plan named",
                "fraction",
                Higher,
            )?,
            metric(
                "cache_read_tokens",
                "Median cache-read tokens per run",
                "tokens",
                Lower,
            )?,
            metric("tool_calls", "Median tool calls per run", "calls", Lower)?,
            metric("seconds", "Median time per run", "s", Lower)?,
            metric(
                "input_tokens",
                "Median input tokens per run",
                "tokens",
                Lower,
            )?,
            metric(
                "output_tokens",
                "Median output tokens per run",
                "tokens",
                Lower,
            )?,
            metric("cost_usd", "Median cost per run", "USD", Lower)?,
        ],
        observations,
        verdicts,
        views: vec![
            ViewDefinition::Bars {
                id: id("solved")?,
                title: "Tasks solved".into(),
                metric: id("solve_rate")?,
                fixed: BTreeMap::new(),
            },
            ViewDefinition::Bars {
                id: id("cost")?,
                title: "Median cost per run".into(),
                metric: id("cost_usd")?,
                fixed: BTreeMap::new(),
            },
            ViewDefinition::Bars {
                id: id("tokens")?,
                title: "Median input tokens per run".into(),
                metric: id("input_tokens")?,
                fixed: BTreeMap::new(),
            },
            ViewDefinition::Table {
                id: id("by-task")?,
                title: "Tasks solved, by task".into(),
                metrics: vec![id("solve_rate")?],
                rows: vec![id("task")?],
                fixed: BTreeMap::new(),
            },
        ],
        limits,
        artifacts,
    };
    report.validate()?;
    Ok(report)
}
