//! rrflowKV against the Fjall baseline, on one semantic-storage workload.
//!
//! The workload is the one `engine_benchmark` runs on rrflowKV: append one
//! claim per ordinal in authoritative batches, reopen, verify the whole corpus
//! in bounded pages, read deterministic bounded sequence ranges, maintain, and
//! verify again. Here it runs through the `StorageEngine` port for each
//! backend, so both engines see identical claims, batches and reads.
//!
//! Every trial of every backend runs in its own child process, and the
//! reopened reads run in a further child, so allocator and process high-water
//! marks do not leak between trials or engines. One document is emitted: each
//! backend's raw trials and median aggregate, and the ratios between them.
//!
//! The reads are measured twice: after a clean reopen, and again after each
//! engine's own maintenance. The second pass is the one served from flushed
//! and compacted data, so its ratios are reported beside the first.
//!
//! This is a diagnostic from one machine. It states no verdict: whether either
//! engine is acceptable is a release-gate decision, not a program output.
//!
//! ```text
//! cargo run --release -p rrd-store --features fjall-baseline \
//!   --example storage_comparison -- --trials 5 --operations 16384 --output out.json
//! ```

use rrd_store::{
    measure_storage_footprint, FjallBaselineStore, RrflowKvStore, StorageEngine, StorageFootprint,
};
use rrflow_kernel::{Claim, Predicate, Producer, Subject};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FORMAT_VERSION: u16 = 2;
const BACKENDS: [Backend; 2] = [Backend::RrflowKv, Backend::Fjall];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Backend {
    RrflowKv,
    Fjall,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Self::RrflowKv => "rrflow_kv",
            Self::Fjall => "fjall",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        BACKENDS
            .into_iter()
            .find(|backend| backend.name() == value)
            .ok_or_else(|| format!("unknown backend {value}"))
    }

    fn open(self, path: &Path) -> Result<Box<dyn StorageEngine>, String> {
        match self {
            Self::RrflowKv => RrflowKvStore::open(path)
                .map(|store| Box::new(store) as Box<dyn StorageEngine>)
                .map_err(|error| error.to_string()),
            Self::Fjall => FjallBaselineStore::open(path)
                .map(|store| Box::new(store) as Box<dyn StorageEngine>)
                .map_err(|error| error.to_string()),
        }
    }

    /// Flush, compact and collect, each engine by its own means, then close.
    fn maintain(self, path: &Path) -> Result<(), String> {
        match self {
            Self::RrflowKv => {
                let store = RrflowKvStore::open(path).map_err(|error| error.to_string())?;
                store.compact(1, 1).map_err(|error| error.to_string())?;
                store
                    .garbage_collect(1, 1)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            }
            Self::Fjall => FjallBaselineStore::open(path)
                .and_then(|store| store.maintain())
                .map_err(|error| error.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    trials: usize,
    operations: usize,
    batch_size: usize,
    reads: usize,
    read_width: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trials: 3,
            operations: 4_096,
            batch_size: 64,
            reads: 512,
            read_width: 32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Latency {
    samples: usize,
    p50_ns: u64,
    p95_ns: u64,
    p99_ns: u64,
    maximum_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Footprint {
    apparent_bytes: u64,
    allocated_bytes: Option<u64>,
    files: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Probe {
    correctness_verified: bool,
    semantic_sequence: u64,
    recovery_ns: u64,
    read_operations_per_second: f64,
    read_latency: Latency,
    peak_rss_kib: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trial {
    backend: Backend,
    correctness_verified: bool,
    write_operations_per_second: f64,
    write_batch_latency: Latency,
    read_operations_per_second: f64,
    read_latency: Latency,
    maintained_read_operations_per_second: f64,
    maintained_read_latency: Latency,
    recovery_ns: u64,
    maintained_recovery_ns: u64,
    maintenance_ns: u64,
    write_peak_rss_kib: Option<u64>,
    peak_rss_kib: Option<u64>,
    active: Footprint,
    reopened: Footprint,
    maintained: Footprint,
    semantic_sequence: u64,
}

#[derive(Debug, Serialize)]
struct Evidence {
    format_version: u16,
    kind: &'static str,
    measured_at_unix_ms: u128,
    architecture: &'static str,
    operating_system: &'static str,
    logical_cpus: usize,
    config: Config,
    aggregation: &'static str,
    caveat: &'static str,
    rrflow_kv_trials: Vec<Trial>,
    fjall_trials: Vec<Trial>,
    rrflow_kv: Trial,
    fjall: Trial,
    /// rrflowKV divided by Fjall. For throughput above 1 favours rrflowKV; for
    /// latency, recovery, memory and size below 1 favours rrflowKV. A missing
    /// measurement is null, never zero.
    ratios: Ratios,
}

#[derive(Debug, Serialize)]
struct Ratios {
    write_throughput: Option<f64>,
    read_throughput: Option<f64>,
    write_p95: Option<f64>,
    read_p95: Option<f64>,
    /// The same reads after each engine's own flush and compaction.
    maintained_read_throughput: Option<f64>,
    maintained_read_p95: Option<f64>,
    recovery: Option<f64>,
    peak_rss: Option<f64>,
    reopened_allocated: Option<f64>,
    maintained_allocated: Option<f64>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("storage comparison failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    match arguments.first().map(String::as_str) {
        Some("--child") => return run_child(&arguments[1..]),
        Some("--probe") => return run_probe(&arguments[1..]),
        _ => {}
    }
    let (config, output) = parse(&arguments, false)?.into_parent();
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let mut trials = [Vec::new(), Vec::new()];
    // Alternate the engines within each round so slow drift on the machine
    // (thermal state, background load) reaches both rather than only the last.
    for round in 0..config.trials {
        for (slot, backend) in BACKENDS.into_iter().enumerate() {
            let root = directory.path().join(format!("{}-{round}", backend.name()));
            trials[slot].push(launch(&["--child"], backend, &root, &config)?);
        }
    }
    let [kv_trials, fjall_trials] = trials;
    let rrflow_kv = aggregate(&kv_trials);
    let fjall = aggregate(&fjall_trials);
    for trial in [&rrflow_kv, &fjall] {
        if !trial.correctness_verified {
            return Err(format!(
                "{} failed correctness in at least one isolated trial",
                trial.backend.name()
            ));
        }
    }
    let evidence = Evidence {
        format_version: FORMAT_VERSION,
        kind: "rrflow-kv-vs-fjall-baseline",
        measured_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis(),
        architecture: std::env::consts::ARCH,
        operating_system: std::env::consts::OS,
        logical_cpus: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
        aggregation: "each top-level metric is the median of that metric across the isolated trials",
        caveat: "one machine, one run; both engines commit every batch at authoritative durability, rrflowKV with sync_data and Fjall with PersistMode::SyncAll; peak RSS exists only on Linux",
        ratios: Ratios {
            write_throughput: ratio(rrflow_kv.write_operations_per_second, fjall.write_operations_per_second),
            read_throughput: ratio(rrflow_kv.read_operations_per_second, fjall.read_operations_per_second),
            write_p95: ratio(rrflow_kv.write_batch_latency.p95_ns as f64, fjall.write_batch_latency.p95_ns as f64),
            read_p95: ratio(rrflow_kv.read_latency.p95_ns as f64, fjall.read_latency.p95_ns as f64),
            maintained_read_throughput: ratio(
                rrflow_kv.maintained_read_operations_per_second,
                fjall.maintained_read_operations_per_second,
            ),
            maintained_read_p95: ratio(
                rrflow_kv.maintained_read_latency.p95_ns as f64,
                fjall.maintained_read_latency.p95_ns as f64,
            ),
            recovery: ratio(rrflow_kv.recovery_ns as f64, fjall.recovery_ns as f64),
            peak_rss: ratio_option(rrflow_kv.peak_rss_kib, fjall.peak_rss_kib),
            reopened_allocated: ratio_option(rrflow_kv.reopened.allocated_bytes, fjall.reopened.allocated_bytes),
            maintained_allocated: ratio_option(rrflow_kv.maintained.allocated_bytes, fjall.maintained.allocated_bytes),
        },
        config,
        rrflow_kv_trials: kv_trials,
        fjall_trials,
        rrflow_kv,
        fjall,
    };
    let bytes = serde_json::to_vec_pretty(&evidence).map_err(|error| error.to_string())?;
    if let Some(output) = output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&output, &bytes).map_err(|error| error.to_string())?;
        eprintln!("wrote comparison evidence to {}", output.display());
    }
    println!(
        "{}",
        String::from_utf8(bytes).map_err(|error| error.to_string())?
    );
    Ok(())
}

struct Parsed {
    config: Config,
    output: Option<PathBuf>,
    backend: Option<Backend>,
    path: Option<PathBuf>,
}

impl Parsed {
    fn into_parent(self) -> (Config, Option<PathBuf>) {
        (self.config, self.output)
    }
}

fn parse(arguments: &[String], isolated: bool) -> Result<Parsed, String> {
    let mut parsed = Parsed {
        config: Config::default(),
        output: None,
        backend: None,
        path: None,
    };
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("missing value after {}", arguments[index]))?;
        match arguments[index].as_str() {
            "--trials" => parsed.config.trials = positive(value, "trials")?,
            "--operations" => parsed.config.operations = positive(value, "operations")?,
            "--batch-size" => parsed.config.batch_size = positive(value, "batch size")?,
            "--reads" => parsed.config.reads = positive(value, "reads")?,
            "--read-width" => parsed.config.read_width = positive(value, "read width")?,
            "--output" if !isolated => parsed.output = Some(PathBuf::from(value)),
            "--backend" if isolated => parsed.backend = Some(Backend::parse(value)?),
            "--path" if isolated => parsed.path = Some(PathBuf::from(value)),
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 2;
    }
    let config = &parsed.config;
    if config.batch_size > config.operations || config.read_width > config.operations {
        return Err("batch size and read width cannot exceed operations".into());
    }
    Ok(parsed)
}

fn positive(value: &str, label: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(0) => Err(format!("{label} must be greater than zero")),
        Ok(parsed) => Ok(parsed),
        Err(error) => Err(format!("invalid {label}: {error}")),
    }
}

/// Runs this program again as a child (`--child` or `--probe`) and decodes its JSON.
fn launch<T: for<'de> Deserialize<'de>>(
    mode: &[&str],
    backend: Backend,
    path: &Path,
    config: &Config,
) -> Result<T, String> {
    let output = Command::new(std::env::current_exe().map_err(|error| error.to_string())?)
        .args(mode)
        .args(["--backend", backend.name(), "--path"])
        .arg(path)
        .args([
            "--operations",
            &config.operations.to_string(),
            "--batch-size",
            &config.batch_size.to_string(),
            "--reads",
            &config.reads.to_string(),
            "--read-width",
            &config.read_width.to_string(),
        ])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "{} child failed: {}",
            backend.name(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
}

fn isolated(arguments: &[String]) -> Result<(Backend, PathBuf, Config), String> {
    let parsed = parse(arguments, true)?;
    Ok((
        parsed.backend.ok_or("a child needs --backend")?,
        parsed.path.ok_or("a child needs --path")?,
        parsed.config,
    ))
}

fn run_child(arguments: &[String]) -> Result<(), String> {
    let (backend, path, config) = isolated(arguments)?;
    let trial = run_trial(backend, &path, &config)?;
    println!(
        "{}",
        serde_json::to_string(&trial).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn run_probe(arguments: &[String]) -> Result<(), String> {
    let (backend, path, config) = isolated(arguments)?;
    let started = Instant::now();
    let engine = backend.open(&path)?;
    let sequence = engine
        .claims()
        .sequence()
        .map_err(|error| error.to_string())?;
    let recovery = started.elapsed();
    let correctness_verified = verify(engine.as_ref(), &config, sequence)?;
    let (samples, elapsed) = read_workload(engine.as_ref(), &config)?;
    let probe = Probe {
        correctness_verified,
        semantic_sequence: sequence,
        recovery_ns: nanos(recovery),
        read_operations_per_second: rate(config.reads, elapsed),
        read_latency: summarize(samples),
        peak_rss_kib: peak_rss_kib(),
    };
    println!(
        "{}",
        serde_json::to_string(&probe).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn run_trial(backend: Backend, path: &Path, config: &Config) -> Result<Trial, String> {
    let engine = backend.open(path)?;
    let (write_samples, write_elapsed) = write_workload(engine.as_ref(), config)?;
    let write_peak_rss_kib = peak_rss_kib();
    let active = footprint(path)?;
    drop(engine);

    let probe: Probe = launch(&["--probe"], backend, path, config)?;
    let reopened = footprint(path)?;

    let maintenance_started = Instant::now();
    backend.maintain(path)?;
    let maintenance_ns = nanos(maintenance_started.elapsed());

    let maintained_probe: Probe = launch(&["--probe"], backend, path, config)?;
    let maintained = footprint(path)?;
    Ok(Trial {
        backend,
        correctness_verified: probe.correctness_verified && maintained_probe.correctness_verified,
        write_operations_per_second: rate(config.operations, write_elapsed),
        write_batch_latency: summarize(write_samples),
        read_operations_per_second: probe.read_operations_per_second,
        read_latency: probe.read_latency,
        maintained_read_operations_per_second: maintained_probe.read_operations_per_second,
        maintained_read_latency: maintained_probe.read_latency,
        recovery_ns: probe.recovery_ns,
        maintained_recovery_ns: maintained_probe.recovery_ns,
        maintenance_ns,
        write_peak_rss_kib,
        peak_rss_kib: probe.peak_rss_kib,
        active,
        reopened,
        maintained,
        semantic_sequence: probe.semantic_sequence,
    })
}

fn write_workload(
    engine: &dyn StorageEngine,
    config: &Config,
) -> Result<(Vec<Duration>, Duration), String> {
    let corpus = (0..config.operations)
        .map(|index| {
            Claim::new(
                Subject::new(format!("benchmark-{index:012}")).expect("valid subject"),
                Predicate::new("value").expect("valid predicate"),
                format!("payload-{index:012}"),
                index as u64 + 1,
                index as u64 + 1,
                Producer {
                    actor: "benchmark".into(),
                    on_behalf_of: None,
                    session: Some("storage-comparison-v1".into()),
                },
            )
        })
        .collect::<Vec<_>>();
    let started = Instant::now();
    let mut samples = Vec::with_capacity(config.operations.div_ceil(config.batch_size));
    for claims in corpus.chunks(config.batch_size) {
        let batch_started = Instant::now();
        engine
            .claims()
            .append_batch(claims)
            .map_err(|error| error.to_string())?;
        samples.push(batch_started.elapsed());
    }
    Ok((samples, started.elapsed()))
}

fn read_workload(
    engine: &dyn StorageEngine,
    config: &Config,
) -> Result<(Vec<Duration>, Duration), String> {
    let started = Instant::now();
    let mut samples = Vec::with_capacity(config.reads);
    let span = config.operations - config.read_width + 1;
    for iteration in 0..config.reads {
        let start = (iteration.wrapping_mul(7_919) % span) as u64;
        let read_started = Instant::now();
        let claims = engine
            .claims()
            .claims_in_range(start, start + config.read_width as u64)
            .map_err(|error| error.to_string())?;
        if claims.len() != config.read_width {
            return Err(format!(
                "bounded replay returned {} claims, expected {}",
                claims.len(),
                config.read_width
            ));
        }
        black_box(claims);
        samples.push(read_started.elapsed());
    }
    Ok((samples, started.elapsed()))
}

/// The whole corpus, in bounded pages, each claim checked for its exact payload.
fn verify(engine: &dyn StorageEngine, config: &Config, sequence: u64) -> Result<bool, String> {
    if sequence != config.operations as u64 {
        return Ok(false);
    }
    let width = config.read_width as u64;
    let mut from = 0u64;
    while from < sequence {
        let to = from.saturating_add(width).min(sequence);
        let claims = engine
            .claims()
            .claims_in_range(from, to)
            .map_err(|error| error.to_string())?;
        if claims.len() as u64 != to - from {
            return Ok(false);
        }
        for (offset, claim) in claims.iter().enumerate() {
            if claim.object != format!("payload-{:012}", from + offset as u64) {
                return Ok(false);
            }
        }
        from = to;
    }
    Ok(true)
}

fn summarize(mut samples: Vec<Duration>) -> Latency {
    samples.sort_unstable();
    let at = |percentile: f64| {
        let index = ((samples.len() as f64 * percentile).ceil() as usize).clamp(1, samples.len());
        nanos(samples[index - 1])
    };
    Latency {
        samples: samples.len(),
        p50_ns: at(0.50),
        p95_ns: at(0.95),
        p99_ns: at(0.99),
        maximum_ns: nanos(*samples.last().expect("at least one sample")),
    }
}

fn aggregate(trials: &[Trial]) -> Trial {
    let first = &trials[0];
    let f64s = |pick: fn(&Trial) -> f64| median_f64(trials.iter().map(pick).collect());
    let u64s = |pick: fn(&Trial) -> u64| median_u64(trials.iter().map(pick).collect());
    let option = |pick: fn(&Trial) -> Option<u64>| {
        let values = trials.iter().filter_map(pick).collect::<Vec<_>>();
        (values.len() == trials.len()).then(|| median_u64(values))
    };
    let latency = |pick: fn(&Trial) -> &Latency| Latency {
        samples: pick(first).samples,
        p50_ns: median_u64(trials.iter().map(|t| pick(t).p50_ns).collect()),
        p95_ns: median_u64(trials.iter().map(|t| pick(t).p95_ns).collect()),
        p99_ns: median_u64(trials.iter().map(|t| pick(t).p99_ns).collect()),
        maximum_ns: median_u64(trials.iter().map(|t| pick(t).maximum_ns).collect()),
    };
    let footprint = |pick: fn(&Trial) -> &Footprint| Footprint {
        apparent_bytes: median_u64(trials.iter().map(|t| pick(t).apparent_bytes).collect()),
        allocated_bytes: {
            let values = trials
                .iter()
                .filter_map(|t| pick(t).allocated_bytes)
                .collect::<Vec<_>>();
            (values.len() == trials.len()).then(|| median_u64(values))
        },
        files: median_u64(trials.iter().map(|t| pick(t).files).collect()),
    };
    Trial {
        backend: first.backend,
        correctness_verified: trials.iter().all(|trial| trial.correctness_verified),
        write_operations_per_second: f64s(|t| t.write_operations_per_second),
        write_batch_latency: latency(|t| &t.write_batch_latency),
        read_operations_per_second: f64s(|t| t.read_operations_per_second),
        read_latency: latency(|t| &t.read_latency),
        maintained_read_operations_per_second: f64s(|t| t.maintained_read_operations_per_second),
        maintained_read_latency: latency(|t| &t.maintained_read_latency),
        recovery_ns: u64s(|t| t.recovery_ns),
        maintained_recovery_ns: u64s(|t| t.maintained_recovery_ns),
        maintenance_ns: u64s(|t| t.maintenance_ns),
        write_peak_rss_kib: option(|t| t.write_peak_rss_kib),
        peak_rss_kib: option(|t| t.peak_rss_kib),
        active: footprint(|t| &t.active),
        reopened: footprint(|t| &t.reopened),
        maintained: footprint(|t| &t.maintained),
        semantic_sequence: first.semantic_sequence,
    }
}

fn median_u64(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values[values.len() / 2]
}

fn median_f64(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn ratio(numerator: f64, denominator: f64) -> Option<f64> {
    (denominator.is_finite() && denominator > 0.0 && numerator.is_finite())
        .then(|| numerator / denominator)
}

fn ratio_option(numerator: Option<u64>, denominator: Option<u64>) -> Option<f64> {
    ratio(numerator? as f64, denominator? as f64)
}

fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn rate(operations: usize, elapsed: Duration) -> f64 {
    operations as f64 / elapsed.as_secs_f64()
}

fn peak_rss_kib() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("VmHWM:")?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

fn footprint(path: &Path) -> Result<Footprint, String> {
    let measured: StorageFootprint =
        measure_storage_footprint(path).map_err(|error| error.to_string())?;
    Ok(Footprint {
        apparent_bytes: measured.apparent_bytes,
        allocated_bytes: measured.allocated_bytes,
        files: measured.files,
    })
}
