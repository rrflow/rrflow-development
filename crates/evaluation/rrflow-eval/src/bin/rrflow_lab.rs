//! Run and combine comparison-lab tests. See eval/lab/README.md.
use rrflow_eval::lab::definitions::content::{
    lab_files, pinned_keys, references_of, RrflowContent, RRFLOW_PRODUCT,
};
use rrflow_eval::lab::definitions::home::{DefinitionHome, PublishKind, PublishOptions};
use rrflow_eval::lab::definitions::pack::{
    build_pack, generate_key, read_pack, InstallOutcome, PackContent as _, PackRequest, PackStore,
    StoreStatus, TrustedKey, TrustedKeys, VerifiedPack, MAX_PACK_BYTES,
};
use rrflow_eval::lab::provider::ClaudeCode;
use rrflow_eval::lab::settings::{
    approve_all, build_plan, launch_digest, needs_approval, planned_files, runs_somewhere,
    waiting_for_approval, Definitions, LabPlan, LabSettings, PlannedTest, SystemState,
};
use rrflow_eval::lab::study::{
    audit_runs, check_study, plan_study, CaptureAudit, RequirementStatus, StudyCheck, StudyConfig,
    StudyRequirements,
};
use rrflow_eval::lab::tasks::{
    charged_usd, combine_task_runs, run_order, run_task, RunInputs, RunKey, TaskComparisonManifest,
    TaskRun,
};
use rrflow_eval::storage::{schema, Digest, Id};
use rrflow_evaluation_core::report::{ArtifactReference, LabReport, SystemStatus};
use rrflow_evaluation_core::Availability;
use rrflow_evaluation_core::{canonical_json, sha256_hex, Result};
use rrflow_lab_benchmarks::catalog::{
    launch_arguments, launch_info, resolve_test, vectordbbench_arguments, Pod, SystemDefinition,
    TestDefinition,
};
use rrflow_lab_benchmarks::driver::{
    combine, empty_run, now_unix_ms, prepare_inputs, run_system_with_progress, SystemRun,
};
use rrflow_lab_benchmarks::import::{import_vectordbbench, MAX_IMPORT_BYTES};
use rrflow_lab_benchmarks::manifest::VectorComparisonManifest;
use rrflow_lab_benchmarks::references::{
    compare, ComparedSeek, Comparison, HardwareClass, ListedAnswer, MeasuredSpeed, ReferenceFile,
    TestConditions,
};
use rrflow_lab_datasets::knowledge::{read_repository_text, CHUNK_TABLE};
use rrflow_lab_systems::adapters::custom::CustomProtocol;
use rrflow_lab_systems::adapters::rrflow::RrflowAdapter;
use rrflow_lab_systems::registry::connect;
use rrflow_lab_systems::sources::SourceList;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const HOME_VARIABLE: &str = "RRFLOW_LAB_HOME";
const SETTINGS_FILE: &str = "settings.json";
const DEFAULT_PACK_DAYS: u64 = 90;
const DAY_SECONDS: u64 = 86_400;
/// Flags that take no value.
const SWITCHES: [&str; 3] = ["json", "all", "details"];
const USAGE: &str = "usage:
  rrflow-lab validate --manifest FILE
  rrflow-lab resolve-all --tests DIR --systems DIR --out DIR
  rrflow-lab launch-info --system FILE
  rrflow-lab launch-args --system FILE --data DIR --lab DIR [--cpuset CORES] [--pod CONTAINER --volume NAME]
  rrflow-lab vdbbench-args --system FILE --data DIR --ef N
  rrflow-lab inputs --manifest FILE
  rrflow-lab run --manifest FILE --system ID --out DIR [--note KEY=VALUE]...
  rrflow-lab combine --manifest FILE --in DIR --out FILE
  rrflow-lab import --from vectordbbench --result FILE --harness-version VERSION --recall-target NUMBER --out FILE
  rrflow-lab sources --file FILE
  rrflow-lab schemas --out DIR
  rrflow-lab plan [--test ID] [--out NEW_DIR] [--json]
  rrflow-lab compare [--report FILE] [--details] [--json]
  rrflow-lab settings [--json]
  rrflow-lab settings on|off ID
  rrflow-lab settings tune SYSTEM NAME=VALUE...
  rrflow-lab settings approve SYSTEM | --all
  rrflow-lab settings reset [ID]
  rrflow-lab definitions publish [--lab DIR] [--days N]
  rrflow-lab definitions update [--from FOLDER]
  rrflow-lab definitions status [--json] [--trust FILE] [--store DIR]
  rrflow-lab definitions trust --file FILE
  rrflow-lab definitions keygen --private NEW_FILE --trust NEW_FILE
  rrflow-lab definitions build --lab DIR --key FILE --sequence N --days N --out FILE
  rrflow-lab definitions patch --lab DIR --key FILE --sequence N --days N --out FILE --base PACK [--trust FILE]
  rrflow-lab definitions verify --pack FILE [--trust FILE]
  rrflow-lab definitions install --pack FILE [--trust FILE] [--store DIR]
  rrflow-lab definitions revert --to N --reason TEXT [--trust FILE] [--store DIR]
  rrflow-lab tasks-validate --manifest FILE
  rrflow-lab tasks-order --manifest FILE
  rrflow-lab tasks-digest --manifest FILE
  rrflow-lab tasks-run --manifest FILE --task ID --arm ID --model ID --repetition N --checkout DIR --empty DIR --out DIR [--mcp-config FILE --chunks FILE]
  rrflow-lab tasks-audit --manifest FILE --in DIR [--out FILE]
  rrflow-lab tasks-combine --manifest FILE --in DIR --out FILE
  rrflow-lab tasks-import --checkout DIR --endpoint URL [--chunks-out FILE]
  rrflow-lab study-check --config FILE --catalog FILE --requirements FILE --sources FILE [--capture-proof FILE] [--out FILE]
  rrflow-lab study-plan --config FILE --catalog FILE --requirements FILE --sources FILE [--capture-proof FILE] --out FILE
  rrflow-lab report-validate --file FILE

Definitions, settings and any signing key live in this user's data folder. Set RRFLOW_LAB_HOME to use another.";

struct Arguments {
    values: BTreeMap<String, String>,
    notes: BTreeMap<String, String>,
    switches: BTreeSet<String>,
}

fn parse(arguments: &[String]) -> Result<Arguments> {
    let mut values = BTreeMap::new();
    let mut notes = BTreeMap::new();
    let mut switches = BTreeSet::new();
    let mut iter = arguments.iter();
    while let Some(flag) = iter.next() {
        let name = flag
            .strip_prefix("--")
            .ok_or_else(|| format!("unexpected argument {flag}"))?;
        if SWITCHES.contains(&name) {
            switches.insert(name.to_owned());
            continue;
        }
        let value = iter
            .next()
            .ok_or_else(|| format!("--{name} needs a value"))?;
        if name == "note" {
            let (key, note) = value.split_once('=').ok_or("--note needs KEY=VALUE")?;
            notes.insert(format!("note.{key}"), note.to_string());
        } else if values.insert(name.to_string(), value.clone()).is_some() {
            return Err(format!("--{name} was given twice"));
        }
    }
    Ok(Arguments {
        values,
        notes,
        switches,
    })
}

impl Arguments {
    fn required(&self, name: &str) -> Result<&str> {
        self.values
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| format!("--{name} is required"))
    }

    fn json(&self) -> bool {
        self.switches.contains("json")
    }
}

fn load_manifest(path: &str) -> Result<VectorComparisonManifest> {
    let metadata = std::fs::metadata(path).map_err(|error| format!("{path}: {error}"))?;
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(format!("{path} exceeds the manifest size bound"));
    }
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let manifest: VectorComparisonManifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    manifest.validate()?;
    Ok(manifest)
}

fn cpu_model() -> Option<String> {
    let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let line = text
        .lines()
        .find(|line| line.starts_with("model name") || line.starts_with("Processor"))?;
    Some(line.split_once(':')?.1.trim().to_string())
}

fn memory_total() -> Option<String> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = text.lines().find(|line| line.starts_with("MemTotal:"))?;
    let kilobytes: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some((kilobytes * 1024).to_string())
}

fn environment(notes: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut environment = notes.clone();
    environment.insert("os".into(), std::env::consts::OS.into());
    environment.insert("architecture".into(), std::env::consts::ARCH.into());
    environment.insert(
        "logical_cpus".into(),
        std::thread::available_parallelism()
            .map_or(0, |count| count.get())
            .to_string(),
    );
    if let Some(model) = cpu_model() {
        environment.insert("cpu_model".into(), model);
    }
    if let Some(memory) = memory_total() {
        environment.insert("memory_total_bytes".into(), memory);
    }
    environment
}

/// Evidence is never silently replaced: a repeated run needs a new directory.
fn publish(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    let temporary = path.with_extension("partial");
    std::fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    std::fs::rename(&temporary, path).map_err(|error| error.to_string())
}

fn run(arguments: &Arguments) -> Result<ExitCode> {
    let manifest = load_manifest(arguments.required("manifest")?)?;
    let id = arguments.required("system")?;
    let binding = manifest.binding(id)?;
    if !matches!(binding.availability, Availability::Available) {
        return Err(format!(
            "{id} is declared as not running: {:?}",
            binding.availability
        ));
    }
    let directory = PathBuf::from(arguments.required("out")?);
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    eprintln!("preparing corpus and exact answers");
    let inputs = prepare_inputs(&manifest)?;
    let environment = environment(&arguments.notes);
    eprintln!("running {id}");
    let (run, raw) = match connect(binding, &manifest.workload.connection_limits()) {
        Ok(adapter) => run_system_with_progress(
            &manifest,
            binding,
            adapter.as_ref(),
            &inputs,
            environment,
            &mut |progress| {
                eprintln!(
                    "{}",
                    serde_json::json!({"system": id, "progress": progress})
                );
            },
        ),
        Err(error) => {
            let mut run = empty_run(binding, &inputs, environment);
            run.reason = Some(error);
            (run, Vec::new())
        }
    };
    publish(
        &directory.join(format!("{id}.samples.json")),
        &serde_json::to_vec(&raw).map_err(|error| error.to_string())?,
    )?;
    publish(
        &directory.join(format!("{id}.run.json")),
        &canonical_json(&run)?,
    )?;
    match run.status {
        SystemStatus::Completed => {
            eprintln!("{id}: completed, {} case(s)", run.cases.len());
            Ok(ExitCode::SUCCESS)
        }
        status => {
            eprintln!("{id}: {status:?}: {}", run.reason.unwrap_or_default());
            Ok(ExitCode::from(3))
        }
    }
}

fn artifact(path: &Path, description: &str) -> Result<ArtifactReference> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(ArtifactReference {
        path: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        sha256: Digest::try_from(sha256_hex(&bytes))?,
        bytes: bytes.len() as u64,
        description: description.into(),
    })
}

fn combine_runs(arguments: &Arguments) -> Result<ExitCode> {
    let manifest = load_manifest(arguments.required("manifest")?)?;
    let directory = PathBuf::from(arguments.required("in")?);
    let mut runs = Vec::new();
    let mut artifacts = Vec::new();
    for binding in &manifest.systems {
        let id = binding.id.as_str();
        let path = directory.join(format!("{id}.run.json"));
        if !path.exists() {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
        let run: SystemRun = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if run.system != binding.id {
            return Err(format!("{} holds a run for another system", path.display()));
        }
        runs.push(run);
        artifacts.push(artifact(&path, "one system's measured run")?);
        artifacts.push(artifact(
            &directory.join(format!("{id}.samples.json")),
            "every latency and recall sample behind that run",
        )?);
    }
    let report = combine(&manifest, &runs, artifacts)?;
    publish(
        Path::new(arguments.required("out")?),
        &canonical_json(&report)?,
    )?;
    Ok(ExitCode::SUCCESS)
}

fn import_results(arguments: &Arguments) -> Result<ExitCode> {
    let format = arguments.required("from")?;
    if format != "vectordbbench" {
        return Err(format!(
            "no importer for {format}; vectordbbench is the one available"
        ));
    }
    let path = arguments.required("result")?;
    let metadata = std::fs::metadata(path).map_err(|error| format!("{path}: {error}"))?;
    if metadata.len() > MAX_IMPORT_BYTES {
        return Err(format!("{path} exceeds the import size bound"));
    }
    let target = arguments
        .required("recall-target")?
        .parse::<f64>()
        .map_err(|error| format!("recall target: {error}"))?;
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let report =
        import_vectordbbench(&bytes, path, arguments.required("harness-version")?, target)?;
    publish(
        Path::new(arguments.required("out")?),
        &canonical_json(&report)?,
    )?;
    Ok(ExitCode::SUCCESS)
}

fn load_system(path: &str) -> Result<SystemDefinition> {
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let definition: SystemDefinition =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    definition.validate()?;
    Ok(definition)
}

/// Words a shell reads back with `mapfile -d ''`, so no value is ever parsed by a shell.
fn print_words(words: &[String]) -> Result<ExitCode> {
    let mut out = std::io::stdout().lock();
    for word in words {
        out.write_all(word.as_bytes())
            .and_then(|()| out.write_all(&[0]))
            .map_err(|error| error.to_string())?;
    }
    Ok(ExitCode::SUCCESS)
}

fn resolve_all(arguments: &Arguments) -> Result<ExitCode> {
    let mut systems = BTreeMap::new();
    let directory = arguments.required("systems")?;
    for entry in std::fs::read_dir(directory).map_err(|error| format!("{directory}: {error}"))? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let definition = load_system(&path.to_string_lossy())?;
        systems.insert(definition.binding.id.clone(), definition);
    }
    let out = PathBuf::from(arguments.required("out")?);
    let directory = arguments.required("tests")?;
    for entry in std::fs::read_dir(directory).map_err(|error| format!("{directory}: {error}"))? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
        let test: TestDefinition = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let manifest = resolve_test(&test, &systems)?;
        let mut text =
            serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
        text.push('\n');
        let target = out.join(path.file_name().ok_or("a test has no file name")?);
        std::fs::write(&target, text).map_err(|error| error.to_string())?;
        println!("{}", target.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn check_sources(arguments: &Arguments) -> Result<ExitCode> {
    let path = arguments.required("file")?;
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let list: SourceList =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    list.validate()?;
    println!("{} sources are valid", list.sources.len());
    Ok(ExitCode::SUCCESS)
}

fn read_json<T: DeserializeOwned>(path: &str) -> Result<T> {
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))
}

struct StudyFiles {
    config: StudyConfig,
    catalog: TaskComparisonManifest,
    requirements: StudyRequirements,
    proof: Option<CaptureAudit>,
}

fn load_study(arguments: &Arguments) -> Result<StudyFiles> {
    let sources: SourceList = read_json(arguments.required("sources")?)?;
    sources.validate()?;
    let requirements: StudyRequirements = read_json(arguments.required("requirements")?)?;
    requirements.validate(&sources)?;
    Ok(StudyFiles {
        config: read_json(arguments.required("config")?)?,
        catalog: load_tasks(arguments.required("catalog")?)?,
        requirements,
        proof: arguments
            .values
            .get("capture-proof")
            .map(|path| read_json(path))
            .transpose()?,
    })
}

fn print_check(check: &StudyCheck) {
    println!(
        "{} ({:?}): {} runs across {} tasks, {} arms and {} models",
        check.study.as_str(),
        check.stage,
        check.runs,
        check.tasks,
        check.arms,
        check.models
    );
    if let Some(worst) = check.worst_case_usd {
        println!("at most ${worst}");
    }
    println!(
        "exact McNemar needs at least {} one-sided pairs, so this design {} reach significance",
        check.discordant_pairs_needed,
        if check.can_reach_significance {
            "can"
        } else {
            "cannot"
        }
    );
    for outcome in &check.outcomes {
        let label = match outcome.status {
            RequirementStatus::Met => "met    ",
            RequirementStatus::NotMet => "NOT MET",
            RequirementStatus::AdvisedNotMet => "advised",
        };
        println!("{label} {}: {}", outcome.id.as_str(), outcome.detail);
    }
    println!("{}", if check.ready { "ready" } else { "not ready" });
}

fn study_check(arguments: &Arguments) -> Result<ExitCode> {
    let files = load_study(arguments)?;
    let check = check_study(
        &files.config,
        &files.catalog,
        &files.requirements,
        files.proof.as_ref(),
    )?;
    print_check(&check);
    if let Some(path) = arguments.values.get("out") {
        std::fs::write(path, canonical_json(&check)?).map_err(|error| error.to_string())?;
    }
    Ok(if check.ready {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn study_plan(arguments: &Arguments) -> Result<ExitCode> {
    let files = load_study(arguments)?;
    let manifest = plan_study(
        &files.config,
        &files.catalog,
        &files.requirements,
        files.proof.as_ref(),
    )?;
    let path = PathBuf::from(arguments.required("out")?);
    publish(&path, &canonical_json(&manifest)?)?;
    println!("{} {}", manifest.digest()?.as_str(), path.display());
    Ok(ExitCode::SUCCESS)
}

fn audit_tasks(arguments: &Arguments) -> Result<ExitCode> {
    let manifest = load_tasks(arguments.required("manifest")?)?;
    let runs: Vec<TaskRun> = saved_task_runs(Path::new(arguments.required("in")?))?
        .into_iter()
        .map(|(_, run)| run)
        .collect();
    let audit = audit_runs(&manifest, &runs)?;
    for arm in &audit.arms {
        println!(
            "{} {}: {} of {} runs present, {} counted, {} failed, {} without tools, {}",
            arm.arm.as_str(),
            arm.model_name,
            arm.runs_present,
            arm.runs_expected,
            arm.runs_counted,
            arm.runs_failed,
            arm.runs_without_tools,
            if arm.proven { "proven" } else { "not proven" }
        );
        for field in &arm.fields {
            println!(
                "  {}: {} captured, {} missing",
                field.field.label(),
                field.captured,
                field.missing
            );
        }
    }
    for problem in &audit.problems {
        println!("problem: {problem}");
    }
    if let Some(path) = arguments.values.get("out") {
        std::fs::write(path, canonical_json(&audit)?).map_err(|error| error.to_string())?;
        println!("{} {path}", audit.digest()?.as_str());
    }
    println!(
        "{}",
        if audit.complete {
            "complete"
        } else {
            "incomplete"
        }
    );
    Ok(if audit.complete {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// Each imported chunk's file, keyed by the identity that appears in a tool's reply.
fn load_chunk_files(path: &str) -> Result<BTreeMap<String, String>> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    let mut files = BTreeMap::new();
    for line in text.lines() {
        let chunk: serde_json::Value =
            serde_json::from_str(line).map_err(|error| format!("{path}: {error}"))?;
        let (Some(identity), Some(file)) = (chunk["id"].as_str(), chunk["path"].as_str()) else {
            return Err(format!("{path}: a line has no id or path"));
        };
        let short = identity.rsplit(':').next().unwrap_or(identity);
        files.insert(short.to_string(), file.to_string());
    }
    Ok(files)
}

fn load_tasks(path: &str) -> Result<TaskComparisonManifest> {
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let manifest: TaskComparisonManifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    manifest.validate()?;
    Ok(manifest)
}

fn task_id(text: &str) -> Result<Id> {
    Id::try_from(text.to_string())
}

fn saved_task_runs(directory: &Path) -> Result<Vec<(PathBuf, TaskRun)>> {
    let mut runs = Vec::new();
    if !directory.exists() {
        return Ok(runs);
    }
    for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.to_string_lossy().ends_with(".run.json") {
            let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
            let run: TaskRun = serde_json::from_slice(&bytes)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            runs.push((path, run));
        }
    }
    runs.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(runs)
}

fn run_one_task(arguments: &Arguments) -> Result<ExitCode> {
    let manifest = load_tasks(arguments.required("manifest")?)?;
    let key = RunKey {
        task: task_id(arguments.required("task")?)?,
        arm: task_id(arguments.required("arm")?)?,
        model: task_id(arguments.required("model")?)?,
        repetition: arguments
            .required("repetition")?
            .parse()
            .map_err(|error| format!("repetition: {error}"))?,
    };
    let directory = PathBuf::from(arguments.required("out")?);
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let name = format!(
        "{}.{}.{}.{}",
        key.task.as_str(),
        key.arm.as_str(),
        key.model.as_str(),
        key.repetition
    );
    let run_path = directory.join(format!("{name}.run.json"));
    if run_path.exists() {
        eprintln!("{name}: already measured");
        return Ok(ExitCode::SUCCESS);
    }
    let spend = manifest.require_runnable()?;
    let spent_usd = saved_task_runs(&directory)?
        .iter()
        .map(|(_, run)| charged_usd(run, spend))
        .sum();
    let chunk_files = arguments
        .values
        .get("chunks")
        .map(|path| load_chunk_files(path))
        .transpose()?;
    let (run, raw) = run_task(
        &manifest,
        &key,
        &RunInputs {
            provider: &ClaudeCode,
            checkout: Path::new(arguments.required("checkout")?),
            empty_directory: Path::new(arguments.required("empty")?),
            spent_usd,
            mcp_config: arguments.values.get("mcp-config").map(Path::new),
            chunk_files: chunk_files.as_ref(),
        },
    )?;
    publish(&directory.join(format!("{name}.reply.json")), &raw)?;
    publish(&run_path, &canonical_json(&run)?)?;
    eprintln!(
        "{name}: {:?}, matched historical file threshold {}",
        run.outcome, run.score.matches_file_threshold
    );
    Ok(ExitCode::SUCCESS)
}

/// Credentials come from the environment, as for a lab run, and never reach a file or log.
fn import_repository(arguments: &Arguments) -> Result<ExitCode> {
    let credential = |name: &str| {
        std::env::var(name).map_err(|_| format!("{name} must name the engine's credential"))
    };
    let checkout = Path::new(arguments.required("checkout")?);
    let started = std::time::Instant::now();
    let text = read_repository_text(checkout)?;
    if let Some(path) = arguments.values.get("chunks-out") {
        let mut lines = Vec::new();
        for chunk in &text.chunks {
            lines.extend(
                serde_json::to_vec(&serde_json::json!({
                    "id": format!("record:{CHUNK_TABLE}:{}", chunk.reference_id()),
                    "path": chunk.path,
                    "text": chunk.text
                }))
                .map_err(|error| error.to_string())?,
            );
            lines.push(b'\n');
        }
        publish(Path::new(path), &lines)?;
    }
    let adapter = RrflowAdapter::connect_to(
        arguments.required("endpoint")?,
        &credential("RRFLOW_LAB_INSTANCE")?,
        &credential("RRFLOW_LAB_PRINCIPAL")?,
        credential("RRFLOW_LAB_SECRET")?,
        std::time::Duration::from_secs(300),
    )?;
    let records = text
        .chunks
        .iter()
        .map(|chunk| chunk.mutation())
        .collect::<Vec<_>>();
    adapter.import_records(
        CHUNK_TABLE,
        "add the lab's imported repository text",
        &records,
    )?;
    println!(
        "{}",
        serde_json::json!({
            "files": text.files,
            "files_left_out": text.files_left_out,
            "chunks": text.chunks.len(),
            "seconds": started.elapsed().as_secs_f64()
        })
    );
    Ok(ExitCode::SUCCESS)
}

fn validate_report(arguments: &Arguments) -> Result<ExitCode> {
    let path = arguments.required("file")?;
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let report: LabReport =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    report.validate()?;
    println!("report is valid");
    Ok(ExitCode::SUCCESS)
}

fn combine_tasks(arguments: &Arguments) -> Result<ExitCode> {
    let manifest = load_tasks(arguments.required("manifest")?)?;
    let directory = PathBuf::from(arguments.required("in")?);
    let mut runs = Vec::new();
    let mut artifacts = Vec::new();
    for (path, run) in saved_task_runs(&directory)? {
        artifacts.push(artifact(&path, "one graded task run")?);
        runs.push(run);
    }
    let report = combine_task_runs(&manifest, &runs, artifacts, now_unix_ms())?;
    publish(
        Path::new(arguments.required("out")?),
        &canonical_json(&report)?,
    )?;
    Ok(ExitCode::SUCCESS)
}

fn write_schemas(arguments: &Arguments) -> Result<ExitCode> {
    let directory = PathBuf::from(arguments.required("out")?);
    for (name, value) in [
        ("lab-manifest-v1.json", schema::<VectorComparisonManifest>()),
        ("lab-run-v1.json", schema::<SystemRun>()),
        ("lab-report-v1.json", schema::<LabReport>()),
        ("lab-custom-protocol-v1.json", schema::<CustomProtocol>()),
        ("lab-sources-v1.json", schema::<SourceList>()),
        ("lab-system-v1.json", schema::<SystemDefinition>()),
        ("lab-test-v1.json", schema::<TestDefinition>()),
        ("lab-settings-v1.json", schema::<LabSettings>()),
        ("lab-plan-v1.json", schema::<LabPlan>()),
        ("lab-reference-v1.json", schema::<ReferenceFile>()),
        ("lab-comparison-v1.json", schema::<Comparison>()),
        ("lab-study-config-v1.json", schema::<StudyConfig>()),
        (
            "lab-study-requirements-v1.json",
            schema::<StudyRequirements>(),
        ),
        ("lab-study-check-v1.json", schema::<StudyCheck>()),
        ("lab-capture-audit-v1.json", schema::<CaptureAudit>()),
        (
            "lab-task-manifest-v1.json",
            schema::<TaskComparisonManifest>(),
        ),
        ("lab-task-run-v1.json", schema::<TaskRun>()),
    ] {
        std::fs::write(directory.join(name), canonical_json(&value)?)
            .map_err(|error| error.to_string())?;
    }
    Ok(ExitCode::SUCCESS)
}

fn now_unix_s() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(|error| error.to_string())
}

fn number(arguments: &Arguments, name: &str) -> Result<u64> {
    let text = arguments.required(name)?;
    text.parse()
        .map_err(|_| format!("--{name} must be a whole number, not {text}"))
}

fn lab_home() -> Result<DefinitionHome> {
    DefinitionHome::resolve(
        std::env::var_os(HOME_VARIABLE).map(PathBuf::from),
        RRFLOW_PRODUCT,
        &|name| std::env::var(name).ok(),
    )
    .map_err(|error| format!("{error}: set {HOME_VARIABLE}"))
}

/// A named file if given. Otherwise the keys pinned in this build and this machine's own.
fn trusted_keys(arguments: &Arguments) -> Result<TrustedKeys> {
    match arguments.values.get("trust") {
        Some(path) => {
            let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
            serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))
        }
        None => lab_home()?.trusted_keys(&pinned_keys()?),
    }
}

fn open_store(arguments: &Arguments) -> Result<PackStore> {
    match arguments.values.get("store") {
        Some(path) => PackStore::open(PathBuf::from(path), RRFLOW_PRODUCT),
        None => lab_home()?.store(),
    }
}

fn settings_path(home: &DefinitionHome) -> PathBuf {
    home.root().join(SETTINGS_FILE)
}

fn load_settings(home: &DefinitionHome) -> Result<LabSettings> {
    LabSettings::load(&settings_path(home))
}

/// What a person sees in place of "not found", wherever nothing is installed yet.
const NOTHING_INSTALLED: &str = "No test definitions are installed yet.
In a checkout of this repository, run: rrflow-lab definitions publish
Otherwise, run: rrflow-lab definitions update --from FOLDER";

fn installed_definitions(store: &PackStore, keys: &TrustedKeys) -> Result<Definitions> {
    Definitions::read(&store.current_files(keys)?)
}

fn days_word(days: u64) -> String {
    if days == 1 {
        "1 day".to_owned()
    } else {
        format!("{days} days")
    }
}

fn more_days(days: u64) -> String {
    if days == 1 {
        "1 more day".to_owned()
    } else {
        format!("{days} more days")
    }
}

fn grouped(number: u64) -> String {
    let digits = number.to_string();
    let mut text = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            text.push(',');
        }
        text.push(digit);
    }
    text
}

fn listed(ids: &[Id]) -> String {
    ids.iter().map(Id::as_str).collect::<Vec<_>>().join(", ")
}

/// Why a pack's life is as it is, in one line.
fn pack_line(status: &StoreStatus, now: u64) -> String {
    let sequence = status.current_sequence.unwrap_or(0);
    let published = status.published_at_unix_s.unwrap_or(now);
    let expires = status.expires_at_unix_s.unwrap_or(now);
    let age = now.saturating_sub(published) / DAY_SECONDS;
    let published = match age {
        0 => "today".to_owned(),
        _ => format!("{} ago", days_word(age)),
    };
    if status.expired {
        format!(
            "Test definitions: pack {sequence}, published {published}. It has expired. It still \
             works and results made with it say so, but get a newer one."
        )
    } else {
        format!(
            "Test definitions: pack {sequence}, published {published}, good for {}.",
            more_days(expires.saturating_sub(now) / DAY_SECONDS)
        )
    }
}

fn approval_hint(waiting: &[Id]) -> Option<String> {
    (!waiting.is_empty()).then(|| {
        format!(
            "{} waiting for your approval: {}.\nSee what each would start: rrflow-lab plan\n\
             Then approve them: rrflow-lab settings approve --all",
            if waiting.len() == 1 {
                "1 system is".to_owned()
            } else {
                format!("{} systems are", waiting.len())
            },
            listed(waiting)
        )
    })
}

fn print_changes(outcome: &InstallOutcome) {
    for (word, paths) in [
        ("Added", &outcome.added),
        ("Changed", &outcome.changed),
        ("Removed", &outcome.removed),
    ] {
        match paths.len() {
            0 => {}
            1..=12 => println!("{word}: {}", paths.join(", ")),
            count => println!("{word}: {count} files"),
        }
    }
    for line in &outcome.notable {
        println!("Look at this: {line}");
    }
}

/// The lab folder of the checkout this is run in.
fn find_lab(arguments: &Arguments) -> Result<PathBuf> {
    if let Some(lab) = arguments.values.get("lab") {
        return Ok(PathBuf::from(lab));
    }
    let here = std::env::current_dir().map_err(|error| error.to_string())?;
    here.ancestors()
        .map(|folder| folder.join("eval").join("lab"))
        .find(|lab| lab.join("sources.json").is_file())
        .ok_or_else(|| {
            "no eval/lab was found from this folder. Run this inside a checkout, or name the \
             folder with --lab DIR"
                .to_owned()
        })
}

fn publish_definitions(arguments: &Arguments) -> Result<ExitCode> {
    let files = lab_files(&find_lab(arguments)?)?;
    let home = lab_home()?;
    let pinned = pinned_keys()?;
    let days = match arguments.values.get("days") {
        Some(_) => number(arguments, "days")?,
        None => DEFAULT_PACK_DAYS,
    };
    let outcome = home.publish(
        &files,
        &RrflowContent,
        &pinned,
        &PublishOptions {
            days,
            now_unix_s: now_unix_s()?,
        },
    )?;
    if outcome.created_key {
        println!(
            "Created your signing key: {}\nBack this file up and keep it private. Anyone holding \
             it can publish tests your machines will trust.\n",
            home.signing_key_path().display()
        );
    }
    if outcome.kind == PublishKind::Unchanged {
        println!(
            "Nothing changed since pack {}, so nothing was published. It is good for {}.",
            outcome.sequence,
            more_days(outcome.days_left)
        );
        return Ok(ExitCode::SUCCESS);
    }
    println!(
        "Published test definitions pack {}, good for {}.",
        outcome.sequence,
        days_word(outcome.days_left)
    );
    if let Some(install) = &outcome.install {
        print_changes(install);
    }
    match (outcome.patch_bytes, outcome.full_bytes) {
        (Some(patch), Some(full)) => println!(
            "Size: {} bytes as a patch, {} bytes in full.",
            grouped(patch),
            grouped(full)
        ),
        (None, Some(full)) => println!("Size: {} bytes.", grouped(full)),
        _ => {}
    }
    // Whoever publishes a launch has read it, on this machine only.
    let mut settings = load_settings(&home)?;
    approve_all(&mut settings, &Definitions::read(&files)?)?;
    settings.save(&settings_path(&home))?;
    let outbox = home.outbox();
    println!("Installed here, and what it starts is approved on this machine.");
    println!("To send it, copy this folder: {}", outbox.display());
    if !outcome.key_pinned {
        println!(
            "Other machines do not trust this key yet. On each one, run:\n  rrflow-lab definitions \
             trust --file {}",
            outbox
                .join(format!("{RRFLOW_PRODUCT}-trusted-keys.json"))
                .display()
        );
    }
    println!("Then, on each machine: rrflow-lab definitions update --from FOLDER");
    Ok(ExitCode::SUCCESS)
}

fn update_definitions(arguments: &Arguments) -> Result<ExitCode> {
    let home = lab_home()?;
    let pinned = pinned_keys()?;
    let mut settings = load_settings(&home)?;
    let named = arguments.values.get("from").map(PathBuf::from);
    let folder = named
        .clone()
        .or_else(|| settings.update_from.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| home.outbox());
    if named.is_none() && settings.update_from.is_none() && !folder.exists() {
        println!("No update folder is set yet. Name one with: rrflow-lab definitions update --from FOLDER");
        return Ok(ExitCode::SUCCESS);
    }
    let now = now_unix_s()?;
    let outcome = match home.update(&folder, &RrflowContent, &pinned, now) {
        Ok(outcome) => outcome,
        // A folder given just now is a mistake. A remembered one may simply be unreachable, and
        // the pack already here still works.
        Err(error) if named.is_some() => return Err(error),
        Err(error) => {
            println!("Could not check for updates: {error}");
            return Ok(ExitCode::SUCCESS);
        }
    };
    if let Some(from) = named {
        let text = from.to_string_lossy().into_owned();
        if settings.update_from.as_deref() != Some(text.as_str()) {
            settings.update_from = Some(text);
            settings.save(&settings_path(&home))?;
        }
    }
    let ignored_untrusted = outcome
        .ignored
        .iter()
        .any(|pack| pack.reason.contains("not trusted"));
    for pack in &outcome.ignored {
        println!("Did not use {}: {}", pack.file, pack.reason);
    }
    if ignored_untrusted {
        println!(
            "If you trust this publisher, run: rrflow-lab definitions trust --file {}",
            folder
                .join(format!("{RRFLOW_PRODUCT}-trusted-keys.json"))
                .display()
        );
    }
    match (&outcome.installed, outcome.current_sequence) {
        (Some(install), _) => {
            println!("Updated to test definitions pack {}.", install.sequence);
            print_changes(install);
            let keys = home.trusted_keys(&pinned)?;
            let definitions = installed_definitions(&home.store()?, &keys)?;
            if let Some(hint) = approval_hint(&waiting_for_approval(&definitions, &settings)?) {
                println!("{hint}");
            }
        }
        (None, Some(sequence)) => println!("Already up to date with pack {sequence}."),
        (None, None) => println!(
            "No test definitions could be installed from {}.",
            folder.display()
        ),
    }
    Ok(ExitCode::SUCCESS)
}

fn show_status(arguments: &Arguments) -> Result<ExitCode> {
    let store = open_store(arguments)?;
    let keys = trusted_keys(arguments)?;
    let now = now_unix_s()?;
    let status = store.status(&keys, now)?;
    let settings = if arguments.values.contains_key("store") {
        LabSettings::default()
    } else {
        load_settings(&lab_home()?)?
    };
    let definitions = status
        .current_sequence
        .map(|_| installed_definitions(&store, &keys))
        .transpose()?;
    let waiting = match &definitions {
        Some(definitions) => waiting_for_approval(definitions, &settings)?,
        None => Vec::new(),
    };
    if arguments.json() {
        let mut facts = serde_json::to_value(&status).map_err(|error| error.to_string())?;
        facts["systems"] = definitions.as_ref().map(|d| d.systems.len()).into();
        facts["tests"] = definitions.as_ref().map(|d| d.tests.len()).into();
        facts["waiting_for_approval"] =
            serde_json::to_value(&waiting).map_err(|error| error.to_string())?;
        facts["turned_off"] =
            serde_json::to_value(&settings.off).map_err(|error| error.to_string())?;
        facts["tuned"] = serde_json::to_value(settings.tuning.keys().collect::<Vec<_>>())
            .map_err(|error| error.to_string())?;
        facts["update_from"] = settings.update_from.clone().into();
        return print_json(&facts);
    }
    let Some(definitions) = definitions else {
        println!("{NOTHING_INSTALLED}");
        return Ok(ExitCode::SUCCESS);
    };
    println!("{}", pack_line(&status, now));
    println!(
        "{} systems and {} tests, checked against their signature.",
        definitions.systems.len(),
        definitions.tests.len()
    );
    if let Some(reason) = &status.reverted_because {
        println!(
            "Stepped back from pack {}: {reason}",
            status.highest_sequence
        );
    }
    if !settings.off.is_empty() {
        let off: Vec<Id> = settings.off.iter().cloned().collect();
        println!("Turned off: {}.", listed(&off));
    }
    for (id, tuning) in &settings.tuning {
        println!("Tuned: {} ({}).", id.as_str(), tuning.describe().join(", "));
    }
    if let Some(hint) = approval_hint(&waiting) {
        println!("{hint}");
    }
    Ok(ExitCode::SUCCESS)
}

fn trust_keys(arguments: &Arguments) -> Result<ExitCode> {
    let path = arguments.required("file")?;
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let keys: TrustedKeys =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    let added = lab_home()?.add_trusted_keys(&keys)?;
    if added.is_empty() {
        println!("Those keys were already trusted here.");
    }
    for key in added {
        println!("Now trusting publisher key {key} on this machine.");
    }
    Ok(ExitCode::SUCCESS)
}

fn print_planned_test(test: &PlannedTest) {
    println!("{} ({})", test.title, test.id.as_str());
    if !test.on {
        println!(
            "  Turned off. Turn it on with: rrflow-lab settings on {}",
            test.id.as_str()
        );
        return;
    }
    println!(
        "  {} points, {} dimensions, {} queries, top {}, recall target {}",
        grouped(test.points),
        grouped(u64::from(test.dimensions)),
        grouped(u64::from(test.queries)),
        test.k,
        test.recall_target
    );
    if let Some(problem) = &test.problem {
        println!("  Cannot run as configured: {problem}");
        return;
    }
    let named = |state: SystemState| -> Vec<&str> {
        test.systems
            .iter()
            .filter(|system| system.state == state)
            .map(|system| system.id.as_str())
            .collect()
    };
    let will_run = named(SystemState::WillRun);
    if !will_run.is_empty() {
        println!("  Will run: {}", will_run.join(", "));
    }
    let unset = |name: &String| std::env::var_os(name).is_none();
    for name in test.needs.iter().filter(|name| unset(name)) {
        println!("  Before it runs, set {name} to the folder holding this test's data");
    }
    let mut missing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for system in &test.systems {
        if system.state == SystemState::WillRun {
            for name in system.needs.iter().filter(|name| unset(name)) {
                missing.entry(name).or_default().push(system.id.as_str());
            }
        }
    }
    for (name, systems) in missing {
        println!("  Before it runs, set {name} (for {})", systems.join(", "));
    }
    for system in test
        .systems
        .iter()
        .filter(|system| !system.tuned.is_empty())
    {
        println!(
            "  Tuned: {} ({})",
            system.id.as_str(),
            system.tuned.join(", ")
        );
    }
    let off = named(SystemState::TurnedOff);
    if !off.is_empty() {
        println!("  Turned off: {}", off.join(", "));
    }
    let held: Vec<_> = test
        .systems
        .iter()
        .filter(|system| system.state == SystemState::NeedsApproval)
        .collect();
    if !held.is_empty() {
        println!("  Waiting for approval:");
        for system in held {
            println!(
                "    {} would start {}",
                system.id.as_str(),
                system.starts.as_deref().unwrap_or("something")
            );
        }
        println!("    Approve one with: rrflow-lab settings approve NAME");
    }
    let gaps: Vec<_> = test
        .systems
        .iter()
        .filter(|system| system.state == SystemState::NotAvailable)
        .collect();
    if !gaps.is_empty() {
        println!("  Not available:");
        for system in gaps {
            println!(
                "    {}: {}",
                system.id.as_str(),
                system.reason.as_deref().unwrap_or("no reason given")
            );
        }
    }
}

/// Which pack a plan came from, and so which pack a run's report names.
fn pack_identity(status: &StoreStatus) -> serde_json::Value {
    serde_json::json!({
        "sequence": status.current_sequence,
        "manifest_sha256": status.manifest_sha256,
        "stale": status.expired,
    })
}

fn write_planned(
    directory: &Path,
    files: &BTreeMap<String, String>,
    plan: &LabPlan,
    status: &StoreStatus,
) -> Result<()> {
    if directory.exists() {
        return Err(format!("{} already exists", directory.display()));
    }
    let write = |relative: &str, text: &str| -> Result<()> {
        let path = directory.join(relative);
        let parent = path.parent().ok_or("a planned file has no parent")?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        std::fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))
    };
    for (relative, text) in files {
        write(relative, text)?;
    }
    let plan = serde_json::to_string_pretty(plan).map_err(|error| error.to_string())?;
    write("plan.json", &plan)?;
    write(
        "definitions.env",
        &format!(
            "sequence={}\nmanifest_sha256={}\nstale={}\n",
            status.current_sequence.unwrap_or(0),
            status.manifest_sha256.as_deref().unwrap_or_default(),
            status.expired
        ),
    )
}

fn show_plan(arguments: &Arguments) -> Result<ExitCode> {
    let home = lab_home()?;
    let keys = home.trusted_keys(&pinned_keys()?)?;
    let store = home.store()?;
    let now = now_unix_s()?;
    let status = store.status(&keys, now)?;
    if status.current_sequence.is_none() {
        return Err(NOTHING_INSTALLED.into());
    }
    let definitions = installed_definitions(&store, &keys)?;
    let settings = load_settings(&home)?;
    let mut plan = build_plan(&definitions, &settings)?;
    if let Some(wanted) = arguments.values.get("test") {
        if !plan.tests.iter().any(|test| test.id.as_str() == wanted) {
            let known: Vec<&str> = plan.tests.iter().map(|test| test.id.as_str()).collect();
            return Err(format!(
                "no test is named {wanted}. These are installed: {}",
                known.join(", ")
            ));
        }
        if let Some(out) = arguments.values.get("out") {
            let files = planned_files(&definitions, &settings, &plan, wanted)?;
            plan.tests.retain(|test| test.id.as_str() == wanted);
            write_planned(Path::new(out), &files, &plan, &status)?;
        } else {
            plan.tests.retain(|test| test.id.as_str() == wanted);
        }
    } else if arguments.values.contains_key("out") {
        return Err("--out needs one test: name it with --test ID".into());
    }
    if arguments.json() {
        return print_json(&serde_json::json!({
            "definitions": pack_identity(&status),
            "tests": plan.tests,
        }));
    }
    println!("{}", pack_line(&status, now));
    for test in &plan.tests {
        println!();
        print_planned_test(test);
    }
    println!();
    println!("Nothing has started. To run a test on a Linux host with Docker:");
    println!("  eval/lab/run_all.sh TEST_ID RUN_FOLDER");
    Ok(ExitCode::SUCCESS)
}

fn show_settings(json: bool) -> Result<ExitCode> {
    let home = lab_home()?;
    let settings = load_settings(&home)?;
    if json {
        return print_json(&settings);
    }
    let keys = home.trusted_keys(&pinned_keys()?)?;
    let store = home.store()?;
    if store
        .status(&keys, now_unix_s()?)?
        .current_sequence
        .is_none()
    {
        return Err(NOTHING_INSTALLED.into());
    }
    let definitions = installed_definitions(&store, &keys)?;
    let waiting = waiting_for_approval(&definitions, &settings)?;
    println!("Systems");
    for (id, system) in &definitions.systems {
        let state = if !runs_somewhere(&definitions, id, system) {
            "n/a"
        } else if settings.off.contains(id) {
            "off"
        } else if waiting.contains(id) {
            "wait"
        } else {
            "on"
        };
        let tuned = settings
            .tuning
            .get(id)
            .map(|tuning| format!("  tuned: {}", tuning.describe().join(", ")))
            .unwrap_or_default();
        println!(
            "  {state:<5}{:<28}{}{tuned}",
            id.as_str(),
            system.binding.label
        );
    }
    println!("Tests");
    for (id, test) in &definitions.tests {
        let state = if settings.off.contains(id) {
            "off"
        } else {
            "on"
        };
        println!("  {state:<5}{:<28}{}", id.as_str(), test.test.title);
    }
    println!("on = runs, off = you turned it off, wait = needs your approval, n/a = not available");
    Ok(ExitCode::SUCCESS)
}

/// One system's name from the installed pack, or a message that lists what there is.
fn known_system<'a>(
    definitions: &'a Definitions,
    name: &str,
) -> Result<(Id, &'a rrflow_lab_benchmarks::catalog::SystemDefinition)> {
    definitions
        .systems
        .iter()
        .find(|(id, _)| id.as_str() == name)
        .map(|(id, system)| (id.clone(), system))
        .ok_or_else(|| {
            let known: Vec<&str> = definitions.systems.keys().map(Id::as_str).collect();
            format!(
                "no system is named {name}. These are installed: {}",
                known.join(", ")
            )
        })
}

fn change_settings(verb: &str, rest: &[String]) -> Result<ExitCode> {
    let home = lab_home()?;
    let keys = home.trusted_keys(&pinned_keys()?)?;
    let store = home.store()?;
    if store
        .status(&keys, now_unix_s()?)?
        .current_sequence
        .is_none()
    {
        return Err(NOTHING_INSTALLED.into());
    }
    let definitions = installed_definitions(&store, &keys)?;
    let mut settings = load_settings(&home)?;
    match (verb, rest) {
        ("on" | "off", [name]) => {
            let is_system = definitions.systems.keys().any(|id| id.as_str() == name);
            let is_test = definitions.tests.keys().any(|id| id.as_str() == name);
            if is_system == is_test {
                return Err(if is_system {
                    format!("{name} names both a system and a test")
                } else {
                    format!("no system or test is named {name}")
                });
            }
            let id = Id::try_from(name.clone())?;
            if verb == "off" {
                settings.off.insert(id);
                println!("{name} is off. It will still appear in reports, marked as turned off.");
            } else {
                settings.off.remove(&id);
                println!("{name} is on.");
            }
        }
        ("tune", [name, pairs @ ..]) if !pairs.is_empty() => {
            let (id, system) = known_system(&definitions, name)?;
            let mut tuning = settings.tuning.get(&id).cloned().unwrap_or_default();
            for pair in pairs {
                let (setting, value) = pair
                    .split_once('=')
                    .ok_or_else(|| format!("{pair} needs the form NAME=VALUE"))?;
                tuning.set(setting, value)?;
            }
            settings.tuning.insert(id.clone(), tuning.clone());
            // Checked now, on every test it reaches, so a bad value is refused while it is typed.
            let mut check = settings.clone();
            check.off.remove(&id);
            check
                .approved_launches
                .insert(id.clone(), launch_digest(system)?);
            for test in build_plan(&definitions, &check)?.tests {
                if let Some(problem) = test.problem {
                    return Err(format!(
                        "That was not saved, because {} could not run with it: {problem}",
                        test.id.as_str()
                    ));
                }
            }
            println!(
                "{name} is tuned: {}. Its reports will say so.",
                tuning.describe().join(", ")
            );
        }
        ("approve", [all]) if all == "--all" => {
            let changed = approve_all(&mut settings, &definitions)?;
            for id in &changed {
                let summary = definitions.systems[id].launch.summary().unwrap_or_default();
                println!("Approved {}: {summary}", id.as_str());
            }
            if changed.is_empty() {
                println!("Everything was already approved.");
            }
        }
        ("approve", [name]) => {
            let (id, system) = known_system(&definitions, name)?;
            if needs_approval(system) {
                settings
                    .approved_launches
                    .insert(id, launch_digest(system)?);
                println!(
                    "Approved {name}: {}",
                    system.launch.summary().unwrap_or_default()
                );
            } else {
                println!("{name} starts nothing new, so it needs no approval.");
            }
        }
        ("reset", []) => {
            settings.off.clear();
            settings.tuning.clear();
            println!("Everything is on and at its published settings. Approvals are kept.");
        }
        ("reset", [name]) => {
            let id = Id::try_from(name.clone())?;
            settings.off.remove(&id);
            settings.tuning.remove(&id);
            println!("{name} is on and at its published settings.");
        }
        _ => return Err(USAGE.into()),
    }
    settings.save(&settings_path(&home))?;
    Ok(ExitCode::SUCCESS)
}

fn short_number(value: f64) -> String {
    if value >= 100.0 {
        grouped(value.round() as u64)
    } else if value >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}

fn print_speeds(speeds: &[&MeasuredSpeed]) {
    for speed in speeds {
        println!(
            "  {:<28}{:>2} client{}  {:>9} per second  {}",
            speed.system.as_str(),
            speed.concurrency,
            if speed.concurrency == 1 { " " } else { "s" },
            short_number(speed.queries_per_second),
            speed
                .latency_p99_ms
                .map_or("p99 not recorded".to_owned(), |latency| format!(
                    "p99 {} ms",
                    short_number(latency)
                ))
        );
    }
}

/// Enough digits to tell results apart, and not the sixteen a source may hold.
fn recall_words(recall: f64) -> String {
    let text = format!("{recall:.4}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn print_seek_row(row: &ComparedSeek, details: bool, ranked_beside_a_test: bool) {
    let result = &row.result;
    println!(
        "  {:>9}/s  recall {:<6}  {:<16}  {}",
        short_number(result.queries_per_second),
        recall_words(result.recall),
        result
            .latency_p99_ms
            .map_or("p99 not stated".to_owned(), |latency| format!(
                "p99 {} ms",
                short_number(latency)
            )),
        result.system,
    );
    println!(
        "              {}, top {}, {}{}. {} ({}).",
        result.dataset,
        result
            .k
            .map_or("k not stated".to_owned(), |k| k.to_string()),
        result.hardware_class.words(),
        if result.filtered { ", filtered" } else { "" },
        row.source_title,
        result.origin.provenance.words(),
    );
    if row.stale {
        println!(
            "              Last read at its source on {}, so check it again.",
            row.checked_on
        );
    }
    if details {
        println!(
            "              Measured by {}, {}. Read on {}.",
            result.origin.measured_by,
            result
                .origin
                .published
                .as_deref()
                .unwrap_or("date not stated"),
            row.checked_on
        );
        println!("              {}", result.origin.url);
        println!("              Hardware: {}.", result.hardware);
        for limit in &result.limits {
            println!("              Note: {limit}");
        }
        if ranked_beside_a_test && !row.same_conditions {
            println!("              Differs: {}.", row.differs.join("; "));
        }
    }
}

fn print_answer_row(row: &ListedAnswer, details: bool) {
    let result = &row.result;
    println!(
        "  {:>7}  {}  ({})",
        result.score, result.system, result.score_name
    );
    println!(
        "           Reader {}, judge {} ({}).{}",
        result.reader_model.as_deref().unwrap_or("not stated"),
        result.judge_model.as_deref().unwrap_or("not stated"),
        result.origin.provenance.words(),
        if result.disputed { " Disputed." } else { "" },
    );
    if row.stale {
        println!(
            "           Last read at its source on {}, so check it again.",
            row.checked_on
        );
    }
    if details {
        println!(
            "           Measured by {}, {}. Read on {}.",
            result.origin.measured_by,
            result
                .origin
                .published
                .as_deref()
                .unwrap_or("date not stated"),
            row.checked_on
        );
        println!("           {}", result.origin.url);
        for limit in &result.limits {
            println!("           Note: {limit}");
        }
    }
}

fn print_comparison(comparison: &Comparison, report: Option<(&str, &LabReport)>, details: bool) {
    println!("Seek time");
    if let Some(conditions) = &comparison.conditions {
        println!(
            "{} ({}), top {}, recall {} or better.",
            conditions.title,
            conditions
                .dataset
                .as_deref()
                .unwrap_or("generated data, with no published name"),
            conditions.k,
            conditions.recall_target
        );
    }
    if let Some((path, report)) = report {
        let pack = report
            .environment
            .get("note.definitions_sequence")
            .map_or("no pack recorded".to_owned(), |sequence| {
                format!("pack {sequence}")
            });
        println!();
        println!("Measured in this run ({path}, {pack})");
        let plain: Vec<_> = comparison
            .measured
            .iter()
            .filter(|speed| !speed.filtered)
            .collect();
        print_speeds(&plain);
        let filtered: Vec<_> = comparison
            .measured
            .iter()
            .filter(|speed| speed.filtered)
            .collect();
        if !filtered.is_empty() {
            println!("  With a filter:");
            print_speeds(&filtered);
        }
        if !comparison.without_speed.is_empty() {
            println!("  No speed to show:");
            for entry in &comparison.without_speed {
                println!("    {}: {}", entry.system.as_str(), entry.reason);
            }
        }
    }
    let same: Vec<_> = comparison
        .published_seek
        .iter()
        .filter(|row| row.same_conditions)
        .collect();
    let context: Vec<_> = comparison
        .published_seek
        .iter()
        .filter(|row| !row.same_conditions)
        .collect();
    if comparison.conditions.is_some() {
        println!();
        if same.is_empty() {
            println!("Published under the same conditions: none. Nothing below can be ranked against this run.");
        } else {
            println!("Published under the same conditions");
            for row in &same {
                print_seek_row(row, details, comparison.conditions.is_some());
            }
        }
    }
    if !context.is_empty() {
        println!();
        println!("Published under other conditions, for context only");
        for row in &context {
            print_seek_row(row, details, comparison.conditions.is_some());
        }
    }
    println!();
    println!("Answer quality");
    println!(
        "{} No ranking is accepted across these benchmarks, because each has its own data, reader and judge, so read a score only beside others from the same benchmark with the same reader and judge.",
        comparison.answers_note
    );
    let mut benchmarks: BTreeMap<&str, Vec<&ListedAnswer>> = BTreeMap::new();
    for row in &comparison.published_answers {
        benchmarks
            .entry(row.result.benchmark.as_str())
            .or_default()
            .push(row);
    }
    for (benchmark, rows) in benchmarks {
        println!();
        println!("{benchmark}");
        for row in rows {
            print_answer_row(row, details);
        }
    }
    if !details {
        println!();
        println!("Add --details for each source's address, notes and what differs.");
    }
}

fn compare_command(arguments: &Arguments) -> Result<ExitCode> {
    let home = lab_home()?;
    let keys = home.trusted_keys(&pinned_keys()?)?;
    let store = home.store()?;
    let now = now_unix_s()?;
    if store.status(&keys, now)?.current_sequence.is_none() {
        return Err(NOTHING_INSTALLED.into());
    }
    let files = store.current_files(&keys)?;
    let references = references_of(&files)?;
    let definitions = Definitions::read(&files)?;
    let report = arguments
        .values
        .get("report")
        .map(|path| -> Result<(String, LabReport)> {
            let metadata = std::fs::metadata(path).map_err(|error| format!("{path}: {error}"))?;
            if metadata.len() > MAX_MANIFEST_BYTES {
                return Err(format!("{path} is larger than a report may be"));
            }
            let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
            let report: LabReport =
                serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
            report
                .validate()
                .map_err(|error| format!("{path}: {error}"))?;
            Ok((path.clone(), report))
        })
        .transpose()?;
    let conditions = report.as_ref().and_then(|(_, report)| {
        definitions
            .tests
            .get(&report.test.id)
            .map(|test| TestConditions {
                test: test.test.id.clone(),
                title: test.test.title.clone(),
                dataset: test.reference_dataset.clone(),
                k: test.workload.k,
                recall_target: test.quality.recall_target,
                // The lab starts each system as a server on a many-core host.
                hardware_class: HardwareClass::MultiCoreServer,
            })
    });
    let comparison = compare(
        &references,
        conditions.as_ref(),
        report.as_ref().map(|(_, report)| report),
        now,
    )?;
    if arguments.json() {
        return print_json(&comparison);
    }
    print_comparison(
        &comparison,
        report
            .as_ref()
            .map(|(path, report)| (path.as_str(), report)),
        arguments.switches.contains("details"),
    );
    Ok(ExitCode::SUCCESS)
}

fn settings_command(arguments: &[String]) -> Result<ExitCode> {
    match arguments.split_first() {
        None => show_settings(false),
        Some((flag, [])) if flag == "--json" => show_settings(true),
        Some((verb, rest)) => change_settings(verb, rest),
    }
}

fn pack_bytes(path: &str) -> Result<Vec<u8>> {
    let metadata = std::fs::metadata(path).map_err(|error| format!("{path}: {error}"))?;
    if metadata.len() > MAX_PACK_BYTES {
        return Err(format!("{path} exceeds the pack size bound"));
    }
    std::fs::read(path).map_err(|error| format!("{path}: {error}"))
}

fn print_json(value: &impl serde::Serialize) -> Result<ExitCode> {
    let text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    println!("{text}");
    Ok(ExitCode::SUCCESS)
}

/// Writes a new file and never replaces one, so a key cannot be overwritten by accident.
fn create_new(path: &str, text: &str, private: bool) -> Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if private {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    let _ = private;
    let mut file = options
        .open(path)
        .map_err(|error| format!("{path}: {error}"))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("{path}: {error}"))
}

fn keygen(arguments: &Arguments) -> Result<ExitCode> {
    let key = generate_key()?;
    let trust = TrustedKeys {
        keys: vec![TrustedKey {
            id: key.key_id.clone(),
            public_key: key.public_hex,
        }],
    };
    let text = serde_json::to_string_pretty(&trust).map_err(|error| error.to_string())?;
    create_new(arguments.required("trust")?, &text, false)?;
    create_new(arguments.required("private")?, &key.private_hex, true)?;
    println!(
        "key {} created. Keep its private file out of every repository.",
        key.key_id
    );
    Ok(ExitCode::SUCCESS)
}

fn build_definitions(arguments: &Arguments, base: Option<&VerifiedPack>) -> Result<ExitCode> {
    let files = lab_files(Path::new(arguments.required("lab")?))?;
    RrflowContent.check(&files)?;
    let key_path = arguments.required("key")?;
    let signing_key =
        std::fs::read_to_string(key_path).map_err(|error| format!("{key_path}: {error}"))?;
    let now = now_unix_s()?;
    let days = number(arguments, "days")?;
    let expires = days
        .checked_mul(86_400)
        .and_then(|seconds| seconds.checked_add(now))
        .ok_or("--days is too large")?;
    let signed = build_pack(&PackRequest {
        product: RRFLOW_PRODUCT,
        content_schema: RrflowContent.schema(),
        sequence: number(arguments, "sequence")?,
        published_at_unix_s: now,
        expires_at_unix_s: expires,
        files: &files,
        signing_key: signing_key.trim(),
        base: base.map(|pack| &pack.manifest),
    })?;
    let text = serde_json::to_string_pretty(&signed).map_err(|error| error.to_string())?;
    let out = arguments.required("out")?;
    std::fs::write(out, &text).map_err(|error| format!("{out}: {error}"))?;
    println!(
        "pack {out}: {} of {} files carried, {} bytes",
        signed.files.len(),
        files.len(),
        text.len()
    );
    for path in signed
        .files
        .keys()
        .take(if base.is_some() { 64 } else { 0 })
    {
        println!("  {path}");
    }
    Ok(ExitCode::SUCCESS)
}

fn definitions(arguments: &[String]) -> Result<ExitCode> {
    let Some((verb, rest)) = arguments.split_first() else {
        return Err(USAGE.into());
    };
    let parsed = parse(rest)?;
    match verb.as_str() {
        "publish" => publish_definitions(&parsed),
        "update" => update_definitions(&parsed),
        "trust" => trust_keys(&parsed),
        "keygen" => keygen(&parsed),
        "build" => build_definitions(&parsed, None),
        "patch" => {
            let base = read_pack(
                &pack_bytes(parsed.required("base")?)?,
                &trusted_keys(&parsed)?,
            )?;
            build_definitions(&parsed, Some(&base))
        }
        "verify" => {
            let keys = trusted_keys(&parsed)?;
            let pack = read_pack(&pack_bytes(parsed.required("pack")?)?, &keys)?;
            print_json(&serde_json::json!({
                "product": pack.manifest.product,
                "sequence": pack.manifest.sequence,
                "published_at_unix_s": pack.manifest.published_at_unix_s,
                "expires_at_unix_s": pack.manifest.expires_at_unix_s,
                "manifest_sha256": pack.manifest_sha256,
                "files_listed": pack.manifest.files.len(),
                "files_carried": pack.carried.len(),
            }))
        }
        "install" => {
            let bytes = pack_bytes(parsed.required("pack")?)?;
            let outcome = open_store(&parsed)?.install(
                &bytes,
                &trusted_keys(&parsed)?,
                &RrflowContent,
                now_unix_s()?,
            )?;
            print_json(&outcome)
        }
        "status" => show_status(&parsed),
        "revert" => {
            let sequence = number(&parsed, "to")?;
            open_store(&parsed)?.revert(
                sequence,
                parsed.required("reason")?,
                &trusted_keys(&parsed)?,
                now_unix_s()?,
            )?;
            println!("reverted to pack {sequence}");
            Ok(ExitCode::SUCCESS)
        }
        _ => Err(USAGE.into()),
    }
}

fn dispatch(arguments: &[String]) -> Result<ExitCode> {
    let Some((command, rest)) = arguments.split_first() else {
        return Err(USAGE.into());
    };
    if command == "definitions" {
        return definitions(rest);
    }
    if command == "settings" {
        return settings_command(rest);
    }
    let parsed = parse(rest)?;
    match command.as_str() {
        "validate" => {
            load_manifest(parsed.required("manifest")?)?;
            println!("manifest is valid");
            Ok(ExitCode::SUCCESS)
        }
        "inputs" => {
            let manifest = load_manifest(parsed.required("manifest")?)?;
            let inputs = prepare_inputs(&manifest)?;
            println!(
                "{}",
                String::from_utf8_lossy(&canonical_json(&inputs.identities)?).trim_end()
            );
            Ok(ExitCode::SUCCESS)
        }
        "run" => run(&parsed),
        "combine" => combine_runs(&parsed),
        "import" => import_results(&parsed),
        "sources" => check_sources(&parsed),
        "resolve-all" => resolve_all(&parsed),
        "launch-info" => {
            for (key, value) in launch_info(&load_system(parsed.required("system")?)?) {
                println!("{key}={value}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "launch-args" => {
            let pod = match (parsed.values.get("pod"), parsed.values.get("volume")) {
                (Some(container), Some(volume)) => Some(Pod { container, volume }),
                (None, None) => None,
                _ => return Err("--pod and --volume go together".into()),
            };
            print_words(&launch_arguments(
                &load_system(parsed.required("system")?)?,
                parsed.required("data")?,
                parsed.required("lab")?,
                parsed.values.get("cpuset").map(String::as_str),
                pod,
                &|name| std::env::var(name).ok(),
            )?)
        }
        "vdbbench-args" => print_words(&vectordbbench_arguments(
            &load_system(parsed.required("system")?)?,
            parsed.required("data")?,
            parsed.required("ef")?,
        )?),
        "tasks-validate" => {
            let manifest = load_tasks(parsed.required("manifest")?)?;
            println!("{} tasks are valid", manifest.tasks.len());
            Ok(ExitCode::SUCCESS)
        }
        "tasks-order" => {
            let manifest = load_tasks(parsed.required("manifest")?)?;
            for key in run_order(&manifest) {
                println!(
                    "{}",
                    serde_json::to_string(&key).map_err(|error| error.to_string())?
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        "tasks-digest" => {
            let manifest = load_tasks(parsed.required("manifest")?)?;
            println!("{}", manifest.digest()?.as_str());
            Ok(ExitCode::SUCCESS)
        }
        "tasks-run" => run_one_task(&parsed),
        "tasks-audit" => audit_tasks(&parsed),
        "study-check" => study_check(&parsed),
        "study-plan" => study_plan(&parsed),
        "tasks-combine" => combine_tasks(&parsed),
        "tasks-import" => import_repository(&parsed),
        "report-validate" => validate_report(&parsed),
        "schemas" => write_schemas(&parsed),
        "plan" => show_plan(&parsed),
        "compare" => compare_command(&parsed),
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    dispatch(&arguments).unwrap_or_else(|error| {
        eprintln!("rrflow-lab: {error}");
        ExitCode::FAILURE
    })
}
