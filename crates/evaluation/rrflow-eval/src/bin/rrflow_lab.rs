//! Run and combine comparison-lab tests. See eval/lab/README.md.
use rrflow_eval::lab::adapter::connect;
use rrflow_eval::lab::catalog::{
    launch_arguments, launch_info, resolve_test, vectordbbench_arguments, SystemDefinition,
    TestDefinition,
};
use rrflow_eval::lab::custom::CustomProtocol;
use rrflow_eval::lab::driver::{combine, empty_run, prepare_inputs, run_system, SystemRun};
use rrflow_eval::lab::import::{import_vectordbbench, MAX_IMPORT_BYTES};
use rrflow_eval::lab::manifest::{Availability, VectorComparisonManifest};
use rrflow_eval::lab::report::{ArtifactReference, LabReport, SystemStatus};
use rrflow_eval::lab::sources::SourceList;
use rrflow_eval::lab::{canonical_json, sha256_hex, Result};
use rrflow_eval::storage::{schema, Digest};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const USAGE: &str = "usage:
  rrflow-lab validate --manifest FILE
  rrflow-lab resolve-all --tests DIR --systems DIR --out DIR
  rrflow-lab launch-info --system FILE
  rrflow-lab launch-args --system FILE --data DIR --lab DIR [--cpuset CORES]
  rrflow-lab vdbbench-args --system FILE --data DIR --ef N
  rrflow-lab inputs --manifest FILE
  rrflow-lab run --manifest FILE --system ID --out DIR [--note KEY=VALUE]...
  rrflow-lab combine --manifest FILE --in DIR --out FILE
  rrflow-lab import --from vectordbbench --result FILE --harness-version VERSION --recall-target NUMBER --out FILE
  rrflow-lab sources --file FILE
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
        "resolve-all" => resolve_all(&parsed),
        "launch-info" => {
            for (key, value) in launch_info(&load_system(parsed.required("system")?)?) {
                println!("{key}={value}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "launch-args" => print_words(&launch_arguments(
            &load_system(parsed.required("system")?)?,
            parsed.required("data")?,
            parsed.required("lab")?,
            parsed.values.get("cpuset").map(String::as_str),
            &|name| std::env::var(name).ok(),
        )?),
        "vdbbench-args" => print_words(&vectordbbench_arguments(
            &load_system(parsed.required("system")?)?,
            parsed.required("data")?,
            parsed.required("ef")?,
        )?),
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
