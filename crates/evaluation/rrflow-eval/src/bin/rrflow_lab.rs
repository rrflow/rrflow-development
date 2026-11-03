//! Run and combine comparison-lab tests. See eval/lab/README.md.
use rrflow_eval::lab::adapter::connect;
use rrflow_eval::lab::custom::CustomProtocol;
use rrflow_eval::lab::driver::{
    combine, empty_run, now_unix_ms, prepare_inputs, run_system, SystemRun,
};
use rrflow_eval::lab::import::{import_vectordbbench, MAX_IMPORT_BYTES};
use rrflow_eval::lab::manifest::{Availability, VectorComparisonManifest};
use rrflow_eval::lab::provider::ClaudeCode;
use rrflow_eval::lab::report::{ArtifactReference, LabReport, SystemStatus};
use rrflow_eval::lab::sources::SourceList;
use rrflow_eval::lab::tasks::{
    combine_task_runs, run_order, run_task, RunInputs, RunKey, TaskComparisonManifest, TaskRun,
};
use rrflow_eval::lab::{canonical_json, sha256_hex, Result};
use rrflow_eval::storage::{schema, Digest, Id};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const USAGE: &str = "usage:
  rrflow-lab validate --manifest FILE
  rrflow-lab inputs --manifest FILE
  rrflow-lab run --manifest FILE --system ID --out DIR [--note KEY=VALUE]...
  rrflow-lab combine --manifest FILE --in DIR --out FILE
  rrflow-lab import --from vectordbbench --result FILE --harness-version VERSION --recall-target NUMBER --out FILE
  rrflow-lab sources --file FILE
  rrflow-lab tasks-validate --manifest FILE
  rrflow-lab tasks-order --manifest FILE
  rrflow-lab tasks-run --manifest FILE --task ID --arm ID --model ID --repetition N --checkout DIR --empty DIR --out DIR
  rrflow-lab tasks-combine --manifest FILE --in DIR --out FILE
  rrflow-lab schemas --out DIR";

struct Arguments {
    values: BTreeMap<String, String>,
    notes: BTreeMap<String, String>,
}

fn parse(arguments: &[String]) -> Result<Arguments> {
    let mut values = BTreeMap::new();
    let mut notes = BTreeMap::new();
    let mut iter = arguments.iter();
    while let Some(flag) = iter.next() {
        let name = flag
            .strip_prefix("--")
            .ok_or_else(|| format!("unexpected argument {flag}"))?;
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
    Ok(Arguments { values, notes })
}

impl Arguments {
    fn required(&self, name: &str) -> Result<&str> {
        self.values
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| format!("--{name} is required"))
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
    let (run, raw) = match connect(binding, &manifest.workload) {
        Ok(adapter) => run_system(&manifest, binding, adapter.as_ref(), &inputs, environment),
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
    if run.status == SystemStatus::Completed {
        eprintln!("{id}: completed, {} case(s)", run.cases.len());
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!("{id}: failed: {}", run.reason.unwrap_or_default());
        Ok(ExitCode::from(3))
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

fn check_sources(arguments: &Arguments) -> Result<ExitCode> {
    let path = arguments.required("file")?;
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let list: SourceList =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    list.validate()?;
    println!("{} sources are valid", list.sources.len());
    Ok(ExitCode::SUCCESS)
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
    let spent_usd = saved_task_runs(&directory)?
        .iter()
        .filter_map(|(_, run)| run.cost_usd)
        .sum();
    let (run, raw) = run_task(
        &manifest,
        &key,
        &RunInputs {
            provider: &ClaudeCode,
            checkout: Path::new(arguments.required("checkout")?),
            empty_directory: Path::new(arguments.required("empty")?),
            spent_usd,
        },
    )?;
    publish(&directory.join(format!("{name}.reply.json")), &raw)?;
    publish(&run_path, &canonical_json(&run)?)?;
    eprintln!("{name}: {:?}, solved {}", run.outcome, run.score.solved);
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

fn dispatch(arguments: &[String]) -> Result<ExitCode> {
    let Some((command, rest)) = arguments.split_first() else {
        return Err(USAGE.into());
    };
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
        "tasks-run" => run_one_task(&parsed),
        "tasks-combine" => combine_tasks(&parsed),
        "schemas" => write_schemas(&parsed),
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
