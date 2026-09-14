//! Reproducible rrflowKV semantic-storage benchmark.
//!
//! Each trial runs in an isolated child process against a fresh directory. The
//! parent emits one versioned JSON evidence document containing absolute
//! correctness, throughput, latency, recovery, memory, maintenance, and
//! footprint measurements. A single machine run is not a universal claim.

use rrd_core::{Claim, Predicate, Producer, Subject};
use rrd_lsm::{
    WritePathDiagnostics, WritePathPhaseTiming, WritePathSample,
    WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION,
};
use rrd_store::{
    measure_storage_footprint, FootprintBytes, RrflowKvStore, StorageEngine, StorageFootprint,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FORMAT_VERSION: u16 = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    trials: usize,
    operations: usize,
    batch_size: usize,
    reads: usize,
    read_width: usize,
    write_path_diagnostics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trials: 3,
            operations: 4_096,
            batch_size: 64,
            reads: 512,
            read_width: 32,
            write_path_diagnostics: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Latency {
    samples: usize,
    minimum_ns: u64,
    p50_ns: u64,
    p95_ns: u64,
    p99_ns: u64,
    p99_9_ns: u64,
    maximum_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trial {
    storage_profile: String,
    correctness_verified: bool,
    write_operations_per_second: f64,
    write_batch_latency: Latency,
    read_operations_per_second: f64,
    read_latency: Latency,
    maintained_read_operations_per_second: f64,
    maintained_read_latency: Latency,
    recovery_ns: u64,
    maintained_recovery_ns: u64,
    uncompacted_recovery_ns: Option<u64>,
    maintenance_ns: Option<u64>,
    write_peak_rss_kib: Option<u64>,
    peak_rss_kib: Option<u64>,
    maintenance_peak_rss_kib: Option<u64>,
    footprint: LifecycleFootprint,
    semantic_sequence: u64,
    maintenance: MaintenanceEvidence,
    write_path_diagnostics: Option<WritePathDiagnosticEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LifecycleFootprint {
    active: StorageFootprint,
    reopened: StorageFootprint,
    maintained: StorageFootprint,
    maintained_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaintenanceEvidence {
    wal_payload_bytes: u64,
    wal_payload_max_bytes: u64,
    memtable_versions: u64,
    memtable_max_versions: u64,
    memtable_bytes: u64,
    memtable_keys: u64,
    memtable_key_payload_bytes: u64,
    memtable_value_payload_bytes: u64,
    memtable_tombstones: u64,
    memtable_spilled_chains: u64,
    memtable_spilled_version_capacity: u64,
    memtable_version_record_bytes: u64,
    memtable_owned_bytes_lower_bound: u64,
    automatic_flushes: u64,
    write_stalls: u64,
    failed_flushes: u64,
    oversized_batches: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BatchWritePathSample {
    semantic_batch_ordinal: u64,
    semantic_operation_count: usize,
    semantic_append_wall_ns: u64,
    semantic_setup_and_harness_ns: u64,
    native: WritePathSample,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WritePathDiagnosticEvidence {
    contract_version: u16,
    raw_samples_location: String,
    raw_sample_count: usize,
    dropped_samples: u64,
    thread_resource_scope: String,
    attribution_contract: String,
    phase_latency: WritePathLatencyEvidence,
    thread_resources: ThreadResourceEvidence,
    raw_samples: Vec<BatchWritePathSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WritePathLatencyEvidence {
    semantic_append_wall: Latency,
    semantic_setup_and_harness: Latency,
    begin_mutex_wait: Latency,
    commit_mutex_wait: Latency,
    transaction_validation: Latency,
    batch_prepare: Latency,
    batch_encode: Latency,
    maintenance: Latency,
    wal_record_prepare: Latency,
    wal_initial_reservation: Latency,
    wal_write: Latency,
    wal_sync: Latency,
    memtable_apply: Latency,
    bookkeeping: Latency,
    physical_measured: Latency,
    physical_unattributed: Latency,
    physical_total_wall: Latency,
    physical_total_thread_cpu: Option<Latency>,
    physical_total_off_cpu: Option<Latency>,
    wal_sync_thread_cpu: Option<Latency>,
    wal_sync_off_cpu: Option<Latency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterEvidence {
    total: u64,
    maximum_per_batch: u64,
    nonzero_samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThreadResourceEvidence {
    available_samples: usize,
    minor_page_faults: CounterEvidence,
    major_page_faults: CounterEvidence,
    block_input_operations: CounterEvidence,
    block_output_operations: CounterEvidence,
    voluntary_context_switches: CounterEvidence,
    involuntary_context_switches: CounterEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProbeResult {
    correctness_verified: bool,
    semantic_sequence: u64,
    recovery_ns: u64,
    read_operations_per_second: f64,
    read_latency: Latency,
    peak_rss_kib: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Evidence {
    format_version: u16,
    measured_at_unix_ms: u128,
    architecture: String,
    operating_system: String,
    logical_cpus: usize,
    aggregation: String,
    footprint_contract: String,
    verification_contract: String,
    config: Config,
    trials: Vec<Trial>,
    rrflow_kv: Trial,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("engine benchmark failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .first()
        .is_some_and(|argument| argument == "--child")
    {
        return run_child(&arguments);
    }
    if arguments
        .first()
        .is_some_and(|argument| argument == "--probe")
    {
        return run_probe(&arguments);
    }
    let (config, output) = parse_parent(&arguments)?;
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let mut trials = Vec::with_capacity(config.trials);
    for trial in 0..config.trials {
        let trial_root = directory.path().join(format!("trial-{trial}"));
        fs::create_dir_all(&trial_root).map_err(|error| error.to_string())?;
        trials.push(launch_child(&trial_root.join("rrflow-kv"), &config)?);
    }
    let rrflow_kv = aggregate("rrflow_kv", &trials);
    if !rrflow_kv.correctness_verified {
        return Err("one or more isolated rrflowKV trials failed correctness".into());
    }
    let evidence = Evidence {
        format_version: FORMAT_VERSION,
        measured_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis(),
        architecture: std::env::consts::ARCH.into(),
        operating_system: std::env::consts::OS.into(),
        logical_cpus: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
        aggregation: "median of per-trial top-level metrics; top-level latency percentiles are medians of each isolated trial's percentile; aggregate write-path phase summaries pool the raw samples retained in trials"
            .into(),
        footprint_contract: "active follows logical writes while rrflowKV is open; reopened follows a clean close/open and full verification before maintenance; maintained follows compaction, unreachable-file collection, and another clean reopen"
            .into(),
        verification_contract: "the complete corpus is verified in read-width pages before measured reads; every page has its exact cardinality and every claim object is checked against its append ordinal; peak RSS is process VmHWM across recovery, paged verification, and measured bounded reads"
            .into(),
        config,
        trials,
        rrflow_kv,
    };
    let bytes = serde_json::to_vec_pretty(&evidence).map_err(|error| error.to_string())?;
    if let Some(output) = output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&output, &bytes).map_err(|error| error.to_string())?;
        eprintln!("wrote benchmark evidence to {}", output.display());
    }
    println!(
        "{}",
        String::from_utf8(bytes).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn parse_parent(arguments: &[String]) -> Result<(Config, Option<PathBuf>), String> {
    let mut config = Config::default();
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--write-path-diagnostics" {
            if config.write_path_diagnostics {
                return Err("--write-path-diagnostics was supplied more than once".into());
            }
            config.write_path_diagnostics = true;
            index += 1;
            continue;
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("missing value after {}", arguments[index]))?;
        match arguments[index].as_str() {
            "--trials" => config.trials = parse_positive(value, "trials")?,
            "--operations" => config.operations = parse_positive(value, "operations")?,
            "--batch-size" => config.batch_size = parse_positive(value, "batch size")?,
            "--reads" => config.reads = parse_positive(value, "reads")?,
            "--read-width" => config.read_width = parse_positive(value, "read width")?,
            "--output" => output = Some(PathBuf::from(value)),
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 2;
    }
    if config.batch_size > config.operations || config.read_width > config.operations {
        return Err("batch size and read width cannot exceed operations".into());
    }
    Ok((config, output))
}

fn parse_positive(value: &str, label: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|error| format!("invalid {label}: {error}"))?;
    if parsed == 0 {
        return Err(format!("{label} must be greater than zero"));
    }
    Ok(parsed)
}

fn launch_child(path: &Path, config: &Config) -> Result<Trial, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(executable);
    command
        .arg("--child")
        .arg("--path")
        .arg(path)
        .arg("--operations")
        .arg(config.operations.to_string())
        .arg("--batch-size")
        .arg(config.batch_size.to_string())
        .arg("--reads")
        .arg(config.reads.to_string())
        .arg("--read-width")
        .arg(config.read_width.to_string())
        .arg("--trials")
        .arg("1");
    if config.write_path_diagnostics {
        command.arg("--write-path-diagnostics");
    }
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "rrflowKV child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
}

fn run_child(arguments: &[String]) -> Result<(), String> {
    if arguments.get(1).map(String::as_str) != Some("--path") {
        return Err("child requires --path".into());
    }
    let path = PathBuf::from(
        arguments
            .get(2)
            .ok_or_else(|| "child requires a path".to_owned())?,
    );
    let (config, output) = parse_parent(&arguments[3..])?;
    if output.is_some() {
        return Err("child cannot write an output file".into());
    }
    let result = run_rrflow_kv(&path, &config)?;
    println!(
        "{}",
        serde_json::to_string(&result).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn launch_probe(path: &Path, config: &Config) -> Result<ProbeResult, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let output = Command::new(executable)
        .arg("--probe")
        .arg("--path")
        .arg(path)
        .arg("--operations")
        .arg(config.operations.to_string())
        .arg("--batch-size")
        .arg(config.batch_size.to_string())
        .arg("--reads")
        .arg(config.reads.to_string())
        .arg("--read-width")
        .arg(config.read_width.to_string())
        .arg("--trials")
        .arg("1")
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "rrflowKV probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
}

fn run_probe(arguments: &[String]) -> Result<(), String> {
    if arguments.get(1).map(String::as_str) != Some("--path") {
        return Err("probe requires --path".into());
    }
    let path = PathBuf::from(
        arguments
            .get(2)
            .ok_or_else(|| "probe requires a path".to_owned())?,
    );
    let (config, output) = parse_parent(&arguments[3..])?;
    if output.is_some() {
        return Err("probe cannot write an output file".into());
    }
    let started = Instant::now();
    let engine = RrflowKvStore::open(&path).map_err(|error| error.to_string())?;
    let sequence = engine
        .claims()
        .sequence()
        .map_err(|error| error.to_string())?;
    let recovery = started.elapsed();
    let correctness_verified = verify(&engine, &config, sequence)?;
    let (read_samples, read_elapsed) = read_workload(&engine, &config)?;
    let result = ProbeResult {
        correctness_verified,
        semantic_sequence: sequence,
        recovery_ns: nanos(recovery),
        read_operations_per_second: rate(config.reads, read_elapsed),
        read_latency: summarize(read_samples),
        peak_rss_kib: peak_rss_kib(),
    };
    println!(
        "{}",
        serde_json::to_string(&result).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn run_rrflow_kv(path: &Path, config: &Config) -> Result<Trial, String> {
    let store = RrflowKvStore::open(path).map_err(|error| error.to_string())?;
    if config.write_path_diagnostics {
        let capacity = config.operations.div_ceil(config.batch_size);
        store
            .enable_native_write_path_diagnostics(capacity)
            .map_err(|error| error.to_string())?;
    }
    let (write_samples, semantic_batch_sizes, write_elapsed) = write_workload(&store, config)?;
    let write_path_diagnostics = if config.write_path_diagnostics {
        let native = store
            .take_native_write_path_diagnostics()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "enabled native write diagnostics returned no collector".to_owned())?;
        Some(pair_write_path_diagnostics(
            native,
            &write_samples,
            &semantic_batch_sizes,
            "this_trial",
            true,
        )?)
    } else {
        None
    };
    let physical = store
        .physical_store_evidence()
        .map_err(|error| error.to_string())?;
    let maintenance = MaintenanceEvidence {
        wal_payload_bytes: required(physical.wal_payload_bytes, "WAL payload bytes")?,
        wal_payload_max_bytes: required(physical.wal_payload_max_bytes, "WAL payload byte limit")?,
        memtable_versions: required(physical.memtable_versions, "memtable versions")?,
        memtable_max_versions: required(physical.memtable_max_versions, "memtable version limit")?,
        memtable_bytes: required(physical.memtable_bytes, "memtable bytes")?,
        memtable_keys: required(physical.memtable_keys, "memtable keys")?,
        memtable_key_payload_bytes: required(
            physical.memtable_key_payload_bytes,
            "memtable key payload bytes",
        )?,
        memtable_value_payload_bytes: required(
            physical.memtable_value_payload_bytes,
            "memtable value payload bytes",
        )?,
        memtable_tombstones: required(physical.memtable_tombstones, "memtable tombstones")?,
        memtable_spilled_chains: required(
            physical.memtable_spilled_chains,
            "memtable spilled chains",
        )?,
        memtable_spilled_version_capacity: required(
            physical.memtable_spilled_version_capacity,
            "memtable spilled version capacity",
        )?,
        memtable_version_record_bytes: required(
            physical.memtable_version_record_bytes,
            "memtable version record bytes",
        )?,
        memtable_owned_bytes_lower_bound: required(
            physical.memtable_owned_bytes_lower_bound,
            "memtable owned bytes lower bound",
        )?,
        automatic_flushes: required(physical.automatic_flushes, "automatic flushes")?,
        write_stalls: required(
            physical.maintenance_write_stalls,
            "maintenance write stalls",
        )?,
        failed_flushes: required(
            physical.failed_maintenance_flushes,
            "failed maintenance flushes",
        )?,
        oversized_batches: required(physical.oversized_batches, "oversized batches")?,
    };
    let write_peak_rss_kib = peak_rss_kib();
    let active = storage_footprint(path)?;
    drop(store);
    let probe = launch_probe(path, config)?;
    let reopened_footprint = storage_footprint(path)?;
    let reopened = RrflowKvStore::open(path).map_err(|error| error.to_string())?;
    let sequence = reopened
        .claims()
        .sequence()
        .map_err(|error| error.to_string())?;
    let verified = verify(&reopened, config, sequence)?;
    let maintenance_started = Instant::now();
    reopened.compact(1, 1).map_err(|error| error.to_string())?;
    reopened
        .garbage_collect(1, 1)
        .map_err(|error| error.to_string())?;
    let maintenance_elapsed = maintenance_started.elapsed();
    let maintenance_peak_rss_kib = peak_rss_kib();
    drop(reopened);
    let maintained_store = RrflowKvStore::open(path).map_err(|error| error.to_string())?;
    let maintained_sequence = maintained_store
        .claims()
        .sequence()
        .map_err(|error| error.to_string())?;
    let maintained_verified = verify(&maintained_store, config, maintained_sequence)?;
    drop(maintained_store);
    let maintained_probe = launch_probe(path, config)?;
    let maintained = storage_footprint(path)?;
    Ok(Trial {
        storage_profile: "rrflow_kv".into(),
        correctness_verified: verified && probe.correctness_verified && maintained_verified,
        write_operations_per_second: rate(config.operations, write_elapsed),
        write_batch_latency: summarize(write_samples),
        read_operations_per_second: probe.read_operations_per_second,
        read_latency: probe.read_latency,
        maintained_read_operations_per_second: maintained_probe.read_operations_per_second,
        maintained_read_latency: maintained_probe.read_latency,
        recovery_ns: probe.recovery_ns,
        maintained_recovery_ns: maintained_probe.recovery_ns,
        uncompacted_recovery_ns: Some(probe.recovery_ns),
        maintenance_ns: Some(nanos(maintenance_elapsed)),
        write_peak_rss_kib,
        peak_rss_kib: probe.peak_rss_kib,
        maintenance_peak_rss_kib,
        footprint: LifecycleFootprint {
            active,
            reopened: reopened_footprint,
            maintained,
            maintained_actions: vec![
                "flush_active_memtable".into(),
                "compact_unpinned_history".into(),
                "collect_unreachable_files".into(),
                "clean_reopen".into(),
            ],
        },
        semantic_sequence: probe.semantic_sequence,
        maintenance,
        write_path_diagnostics,
    })
}

fn required(value: Option<u64>, name: &str) -> Result<u64, String> {
    value.ok_or_else(|| format!("rrflowKV physical evidence omitted {name}"))
}

fn pair_write_path_diagnostics(
    native: WritePathDiagnostics,
    outer_samples: &[Duration],
    semantic_batch_sizes: &[usize],
    raw_samples_location: &str,
    retain_raw_samples: bool,
) -> Result<WritePathDiagnosticEvidence, String> {
    if native.contract_version != WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION {
        return Err(format!(
            "native write diagnostic contract {} does not match {}",
            native.contract_version, WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION
        ));
    }
    if outer_samples.len() != semantic_batch_sizes.len() {
        return Err("semantic write latency and batch-size counts differ".into());
    }
    let expected = outer_samples.len();
    let observed = usize::try_from(native.observed_samples)
        .map_err(|_| "native write diagnostic sample count exceeds usize".to_owned())?;
    if native.capacity != expected {
        return Err(format!(
            "native write diagnostic capacity {} does not match {expected} semantic batches",
            native.capacity
        ));
    }
    if observed != expected || native.samples.len() != expected || native.dropped_samples != 0 {
        return Err(format!(
            "native write diagnostics expected {expected} raw samples, observed {}, retained {}, dropped {}",
            native.observed_samples,
            native.samples.len(),
            native.dropped_samples
        ));
    }

    let mut raw_samples = Vec::with_capacity(expected);
    let mut previous_last_sequence: Option<u64> = None;
    for (index, ((native, outer), semantic_operation_count)) in native
        .samples
        .into_iter()
        .zip(outer_samples)
        .zip(semantic_batch_sizes)
        .enumerate()
    {
        let ordinal = u64::try_from(index)
            .map_err(|_| "semantic write batch ordinal exceeds u64".to_owned())?;
        if native.ordinal != ordinal {
            return Err(format!(
                "native write sample ordinal {} does not match semantic batch {ordinal}",
                native.ordinal
            ));
        }
        let expected_first_sequence = match previous_last_sequence {
            Some(previous) => previous
                .checked_add(1)
                .ok_or_else(|| format!("native batch {ordinal} follows a terminal sequence"))?,
            None => 1,
        };
        if native.first_sequence != expected_first_sequence {
            return Err(format!(
                "native batch {ordinal} starts at sequence {}, expected {expected_first_sequence}",
                native.first_sequence
            ));
        }
        let expected_mutations = semantic_operation_count
            .checked_mul(2)
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| "semantic-to-physical mutation count overflowed".to_owned())?;
        if native.mutation_count != expected_mutations {
            return Err(format!(
                "semantic batch {ordinal} expected {expected_mutations} physical mutations, observed {}",
                native.mutation_count
            ));
        }
        if native.durability != rrd_lsm::Durability::Authoritative {
            return Err(format!(
                "semantic batch {ordinal} was not measured at authoritative durability"
            ));
        }
        let sequence_count = native
            .last_sequence
            .checked_sub(native.first_sequence)
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| format!("native batch {ordinal} has an invalid sequence range"))?;
        if sequence_count != native.mutation_count as u64 {
            return Err(format!(
                "native batch {ordinal} sequence count {sequence_count} differs from mutation count {}",
                native.mutation_count
            ));
        }
        if native.wal_frame_bytes <= native.encoded_payload_bytes {
            return Err(format!(
                "native batch {ordinal} WAL frame {} does not enclose encoded payload {}",
                native.wal_frame_bytes, native.encoded_payload_bytes
            ));
        }
        let measured = native.phases.measured_wall_ns();
        if measured > native.phases.physical_total.wall_ns {
            return Err(format!(
                "native batch {ordinal} measured phases {measured} exceed physical total {}",
                native.phases.physical_total.wall_ns
            ));
        }
        let semantic_append_wall_ns = nanos(*outer);
        let native_accounted = native
            .lock_wait
            .total_ns()
            .saturating_add(native.phases.physical_total.wall_ns);
        if native_accounted > semantic_append_wall_ns {
            return Err(format!(
                "native batch {ordinal} accounted {native_accounted} ns beyond semantic append {semantic_append_wall_ns} ns"
            ));
        }
        raw_samples.push(BatchWritePathSample {
            semantic_batch_ordinal: ordinal,
            semantic_operation_count: *semantic_operation_count,
            semantic_append_wall_ns,
            semantic_setup_and_harness_ns: semantic_append_wall_ns - native_accounted,
            native,
        });
        previous_last_sequence = raw_samples.last().map(|sample| sample.native.last_sequence);
    }

    Ok(build_write_path_evidence(
        &raw_samples,
        native.dropped_samples,
        native.thread_resource_scope,
        raw_samples_location,
        retain_raw_samples,
    ))
}

fn build_write_path_evidence(
    samples: &[BatchWritePathSample],
    dropped_samples: u64,
    thread_resource_scope: String,
    raw_samples_location: &str,
    retain_raw_samples: bool,
) -> WritePathDiagnosticEvidence {
    WritePathDiagnosticEvidence {
        contract_version: WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION,
        raw_samples_location: raw_samples_location.into(),
        raw_sample_count: samples.len(),
        dropped_samples,
        thread_resource_scope,
        attribution_contract: "semantic append encloses begin mutex wait, semantic transaction setup/claim validation/key and JSON preparation, commit mutex wait, and native physical total; physical total encloses the ten mutually exclusive measured phases plus clock/resource-probe transitions; wall minus thread CPU is off-CPU time but is not by itself a causal verdict"
            .into(),
        phase_latency: summarize_write_path_phases(samples),
        thread_resources: summarize_thread_resources(samples),
        raw_samples: if retain_raw_samples {
            samples.to_vec()
        } else {
            Vec::new()
        },
    }
}

fn summarize_write_path_phases(samples: &[BatchWritePathSample]) -> WritePathLatencyEvidence {
    WritePathLatencyEvidence {
        semantic_append_wall: latency_by(samples, |sample| sample.semantic_append_wall_ns),
        semantic_setup_and_harness: latency_by(samples, |sample| {
            sample.semantic_setup_and_harness_ns
        }),
        begin_mutex_wait: latency_by(samples, |sample| {
            sample.native.lock_wait.begin_mutex_wait_ns
        }),
        commit_mutex_wait: latency_by(samples, |sample| {
            sample.native.lock_wait.commit_mutex_wait_ns
        }),
        transaction_validation: phase_wall_latency(samples, |sample| {
            sample.phases.transaction_validation
        }),
        batch_prepare: phase_wall_latency(samples, |sample| sample.phases.batch_prepare),
        batch_encode: phase_wall_latency(samples, |sample| sample.phases.batch_encode),
        maintenance: phase_wall_latency(samples, |sample| sample.phases.maintenance),
        wal_record_prepare: phase_wall_latency(samples, |sample| sample.phases.wal_record_prepare),
        wal_initial_reservation: phase_wall_latency(samples, |sample| {
            sample.phases.wal_initial_reservation
        }),
        wal_write: phase_wall_latency(samples, |sample| sample.phases.wal_write),
        wal_sync: phase_wall_latency(samples, |sample| sample.phases.wal_sync),
        memtable_apply: phase_wall_latency(samples, |sample| sample.phases.memtable_apply),
        bookkeeping: phase_wall_latency(samples, |sample| sample.phases.bookkeeping),
        physical_measured: latency_by(samples, |sample| sample.native.phases.measured_wall_ns()),
        physical_unattributed: latency_by(samples, |sample| {
            sample
                .native
                .phases
                .physical_total
                .wall_ns
                .saturating_sub(sample.native.phases.measured_wall_ns())
        }),
        physical_total_wall: phase_wall_latency(samples, |sample| sample.phases.physical_total),
        physical_total_thread_cpu: optional_latency_by(samples, |sample| {
            sample.native.phases.physical_total.thread_cpu_ns
        }),
        physical_total_off_cpu: optional_latency_by(samples, |sample| {
            sample
                .native
                .phases
                .physical_total
                .thread_cpu_ns
                .map(|cpu| {
                    sample
                        .native
                        .phases
                        .physical_total
                        .wall_ns
                        .saturating_sub(cpu)
                })
        }),
        wal_sync_thread_cpu: optional_latency_by(samples, |sample| {
            sample.native.phases.wal_sync.thread_cpu_ns
        }),
        wal_sync_off_cpu: optional_latency_by(samples, |sample| {
            sample
                .native
                .phases
                .wal_sync
                .thread_cpu_ns
                .map(|cpu| sample.native.phases.wal_sync.wall_ns.saturating_sub(cpu))
        }),
    }
}

fn phase_wall_latency<F>(samples: &[BatchWritePathSample], phase: F) -> Latency
where
    F: Fn(&WritePathSample) -> WritePathPhaseTiming,
{
    latency_by(samples, |sample| phase(&sample.native).wall_ns)
}

fn latency_by<F>(samples: &[BatchWritePathSample], value: F) -> Latency
where
    F: Fn(&BatchWritePathSample) -> u64,
{
    summarize_ns(samples.iter().map(value).collect())
}

fn optional_latency_by<F>(samples: &[BatchWritePathSample], value: F) -> Option<Latency>
where
    F: Fn(&BatchWritePathSample) -> Option<u64>,
{
    samples
        .iter()
        .map(value)
        .collect::<Option<Vec<_>>>()
        .map(summarize_ns)
}

fn summarize_thread_resources(samples: &[BatchWritePathSample]) -> ThreadResourceEvidence {
    let resources = samples
        .iter()
        .filter_map(|sample| sample.native.thread_resources.as_ref())
        .collect::<Vec<_>>();
    ThreadResourceEvidence {
        available_samples: resources.len(),
        minor_page_faults: counter_evidence(resources.iter().map(|value| value.minor_page_faults)),
        major_page_faults: counter_evidence(resources.iter().map(|value| value.major_page_faults)),
        block_input_operations: counter_evidence(
            resources.iter().map(|value| value.block_input_operations),
        ),
        block_output_operations: counter_evidence(
            resources.iter().map(|value| value.block_output_operations),
        ),
        voluntary_context_switches: counter_evidence(
            resources
                .iter()
                .map(|value| value.voluntary_context_switches),
        ),
        involuntary_context_switches: counter_evidence(
            resources
                .iter()
                .map(|value| value.involuntary_context_switches),
        ),
    }
}

fn counter_evidence(values: impl Iterator<Item = u64>) -> CounterEvidence {
    let values = values.collect::<Vec<_>>();
    CounterEvidence {
        total: values.iter().copied().fold(0u64, u64::saturating_add),
        maximum_per_batch: values.iter().copied().max().unwrap_or(0),
        nonzero_samples: values.iter().filter(|value| **value != 0).count(),
    }
}

fn write_workload(
    engine: &dyn StorageEngine,
    config: &Config,
) -> Result<(Vec<Duration>, Vec<usize>, Duration), String> {
    let corpus = (0..config.operations)
        .map(|index| {
            Claim::new(
                Subject::new(format!("benchmark:{index:012}")).expect("valid subject"),
                Predicate::new("value").expect("valid predicate"),
                format!("payload-{index:012}"),
                index as u64 + 1,
                index as u64 + 1,
                Producer {
                    actor: "benchmark".into(),
                    on_behalf_of: None,
                    session: Some("rrflow-kv-benchmark-v1".into()),
                },
            )
        })
        .collect::<Vec<_>>();
    let write_started = Instant::now();
    let mut write_samples = Vec::with_capacity(config.operations.div_ceil(config.batch_size));
    let mut semantic_batch_sizes = Vec::with_capacity(write_samples.capacity());
    for claims in corpus.chunks(config.batch_size) {
        let started = Instant::now();
        engine
            .claims()
            .append_batch(claims)
            .map_err(|error| error.to_string())?;
        write_samples.push(started.elapsed());
        semantic_batch_sizes.push(claims.len());
    }
    let write_elapsed = write_started.elapsed();

    Ok((write_samples, semantic_batch_sizes, write_elapsed))
}

fn read_workload(
    engine: &dyn StorageEngine,
    config: &Config,
) -> Result<(Vec<Duration>, Duration), String> {
    let read_started = Instant::now();
    let mut read_samples = Vec::with_capacity(config.reads);
    let span = config.operations - config.read_width + 1;
    for iteration in 0..config.reads {
        let start = (iteration.wrapping_mul(7_919) % span) as u64;
        let started = Instant::now();
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
        read_samples.push(started.elapsed());
    }
    Ok((read_samples, read_started.elapsed()))
}

fn verify(engine: &dyn StorageEngine, config: &Config, sequence: u64) -> Result<bool, String> {
    if sequence != config.operations as u64 {
        return Ok(false);
    }
    let page_width = u64::try_from(config.read_width.max(1))
        .map_err(|_| "verification page width exceeds u64".to_owned())?;
    let mut from = 0u64;
    while from < sequence {
        let to = from.saturating_add(page_width).min(sequence);
        let claims = engine
            .claims()
            .claims_in_range(from, to)
            .map_err(|error| error.to_string())?;
        let expected = usize::try_from(to - from)
            .map_err(|_| "verification page cardinality exceeds usize".to_owned())?;
        if claims.len() != expected {
            return Ok(false);
        }
        for (offset, claim) in claims.iter().enumerate() {
            let ordinal = from + offset as u64;
            if claim.object != format!("payload-{ordinal:012}") {
                return Ok(false);
            }
        }
        from = to;
    }
    Ok(true)
}

fn summarize(mut samples: Vec<Duration>) -> Latency {
    samples.sort_unstable();
    Latency {
        samples: samples.len(),
        minimum_ns: sample(&samples, 0.0),
        p50_ns: sample(&samples, 0.50),
        p95_ns: sample(&samples, 0.95),
        p99_ns: sample(&samples, 0.99),
        p99_9_ns: sample(&samples, 0.999),
        maximum_ns: sample(&samples, 1.0),
    }
}

fn summarize_ns(mut samples: Vec<u64>) -> Latency {
    samples.sort_unstable();
    Latency {
        samples: samples.len(),
        minimum_ns: sample_u64(&samples, 0.0),
        p50_ns: sample_u64(&samples, 0.50),
        p95_ns: sample_u64(&samples, 0.95),
        p99_ns: sample_u64(&samples, 0.99),
        p99_9_ns: sample_u64(&samples, 0.999),
        maximum_ns: sample_u64(&samples, 1.0),
    }
}

fn aggregate(storage_profile: &str, trials: &[Trial]) -> Trial {
    Trial {
        storage_profile: storage_profile.into(),
        correctness_verified: trials.iter().all(|trial| trial.correctness_verified),
        write_operations_per_second: median_f64(
            trials
                .iter()
                .map(|trial| trial.write_operations_per_second)
                .collect(),
        ),
        write_batch_latency: aggregate_latency(
            trials
                .iter()
                .map(|trial| &trial.write_batch_latency)
                .collect(),
        ),
        read_operations_per_second: median_f64(
            trials
                .iter()
                .map(|trial| trial.read_operations_per_second)
                .collect(),
        ),
        read_latency: aggregate_latency(trials.iter().map(|trial| &trial.read_latency).collect()),
        maintained_read_operations_per_second: median_f64(
            trials
                .iter()
                .map(|trial| trial.maintained_read_operations_per_second)
                .collect(),
        ),
        maintained_read_latency: aggregate_latency(
            trials
                .iter()
                .map(|trial| &trial.maintained_read_latency)
                .collect(),
        ),
        recovery_ns: median_u64(trials.iter().map(|trial| trial.recovery_ns).collect()),
        maintained_recovery_ns: median_u64(
            trials
                .iter()
                .map(|trial| trial.maintained_recovery_ns)
                .collect(),
        ),
        uncompacted_recovery_ns: median_option(
            trials
                .iter()
                .filter_map(|trial| trial.uncompacted_recovery_ns)
                .collect(),
        ),
        maintenance_ns: median_option(
            trials
                .iter()
                .filter_map(|trial| trial.maintenance_ns)
                .collect(),
        ),
        write_peak_rss_kib: median_option(
            trials
                .iter()
                .filter_map(|trial| trial.write_peak_rss_kib)
                .collect(),
        ),
        peak_rss_kib: median_option(
            trials
                .iter()
                .filter_map(|trial| trial.peak_rss_kib)
                .collect(),
        ),
        maintenance_peak_rss_kib: median_option(
            trials
                .iter()
                .filter_map(|trial| trial.maintenance_peak_rss_kib)
                .collect(),
        ),
        footprint: aggregate_lifecycle_footprint(trials),
        semantic_sequence: median_u64(trials.iter().map(|trial| trial.semantic_sequence).collect()),
        maintenance: aggregate_maintenance(trials.iter().map(|trial| &trial.maintenance).collect()),
        write_path_diagnostics: aggregate_write_path_diagnostics(trials),
    }
}

fn aggregate_write_path_diagnostics(trials: &[Trial]) -> Option<WritePathDiagnosticEvidence> {
    let diagnostics = trials
        .iter()
        .filter_map(|trial| trial.write_path_diagnostics.as_ref())
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        return None;
    }
    assert_eq!(
        diagnostics.len(),
        trials.len(),
        "write diagnostic mode must agree across isolated trials"
    );
    let samples = diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.raw_samples.iter().cloned())
        .collect::<Vec<_>>();
    let dropped_samples = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.dropped_samples)
        .fold(0u64, u64::saturating_add);
    let thread_resource_scope = diagnostics[0].thread_resource_scope.clone();
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.thread_resource_scope == thread_resource_scope));
    Some(build_write_path_evidence(
        &samples,
        dropped_samples,
        thread_resource_scope,
        "trials",
        false,
    ))
}

fn aggregate_lifecycle_footprint(trials: &[Trial]) -> LifecycleFootprint {
    LifecycleFootprint {
        active: aggregate_storage_footprint(
            trials.iter().map(|trial| &trial.footprint.active).collect(),
        ),
        reopened: aggregate_storage_footprint(
            trials
                .iter()
                .map(|trial| &trial.footprint.reopened)
                .collect(),
        ),
        maintained: aggregate_storage_footprint(
            trials
                .iter()
                .map(|trial| &trial.footprint.maintained)
                .collect(),
        ),
        maintained_actions: trials[0].footprint.maintained_actions.clone(),
    }
}

fn aggregate_storage_footprint(values: Vec<&StorageFootprint>) -> StorageFootprint {
    let classes = values
        .iter()
        .flat_map(|value| value.by_class.keys().cloned())
        .collect::<BTreeSet<_>>();
    let by_class = classes
        .into_iter()
        .map(|class| {
            let measurements = values
                .iter()
                .map(|value| value.by_class.get(&class).cloned().unwrap_or_default())
                .collect::<Vec<_>>();
            (class, aggregate_footprint_bytes(&measurements))
        })
        .collect();
    let allocated = values
        .iter()
        .filter_map(|value| value.allocated_bytes)
        .collect::<Vec<_>>();
    StorageFootprint {
        apparent_bytes: median_u64(values.iter().map(|value| value.apparent_bytes).collect()),
        allocated_bytes: (allocated.len() == values.len()).then(|| median_u64(allocated)),
        allocated_bytes_source: values[0].allocated_bytes_source.clone(),
        files: median_u64(values.iter().map(|value| value.files).collect()),
        by_class,
    }
}

fn aggregate_footprint_bytes(values: &[FootprintBytes]) -> FootprintBytes {
    let allocated = values
        .iter()
        .filter_map(|value| value.allocated_bytes)
        .collect::<Vec<_>>();
    FootprintBytes {
        apparent_bytes: median_u64(values.iter().map(|value| value.apparent_bytes).collect()),
        allocated_bytes: (allocated.len() == values.len()).then(|| median_u64(allocated)),
        files: median_u64(values.iter().map(|value| value.files).collect()),
    }
}

fn aggregate_maintenance(values: Vec<&MaintenanceEvidence>) -> MaintenanceEvidence {
    MaintenanceEvidence {
        wal_payload_bytes: median_u64(values.iter().map(|value| value.wal_payload_bytes).collect()),
        wal_payload_max_bytes: median_u64(
            values
                .iter()
                .map(|value| value.wal_payload_max_bytes)
                .collect(),
        ),
        memtable_versions: median_u64(values.iter().map(|value| value.memtable_versions).collect()),
        memtable_max_versions: median_u64(
            values
                .iter()
                .map(|value| value.memtable_max_versions)
                .collect(),
        ),
        memtable_bytes: median_u64(values.iter().map(|value| value.memtable_bytes).collect()),
        memtable_keys: median_u64(values.iter().map(|value| value.memtable_keys).collect()),
        memtable_key_payload_bytes: median_u64(
            values
                .iter()
                .map(|value| value.memtable_key_payload_bytes)
                .collect(),
        ),
        memtable_value_payload_bytes: median_u64(
            values
                .iter()
                .map(|value| value.memtable_value_payload_bytes)
                .collect(),
        ),
        memtable_tombstones: median_u64(
            values
                .iter()
                .map(|value| value.memtable_tombstones)
                .collect(),
        ),
        memtable_spilled_chains: median_u64(
            values
                .iter()
                .map(|value| value.memtable_spilled_chains)
                .collect(),
        ),
        memtable_spilled_version_capacity: median_u64(
            values
                .iter()
                .map(|value| value.memtable_spilled_version_capacity)
                .collect(),
        ),
        memtable_version_record_bytes: median_u64(
            values
                .iter()
                .map(|value| value.memtable_version_record_bytes)
                .collect(),
        ),
        memtable_owned_bytes_lower_bound: median_u64(
            values
                .iter()
                .map(|value| value.memtable_owned_bytes_lower_bound)
                .collect(),
        ),
        automatic_flushes: median_u64(values.iter().map(|value| value.automatic_flushes).collect()),
        write_stalls: median_u64(values.iter().map(|value| value.write_stalls).collect()),
        failed_flushes: median_u64(values.iter().map(|value| value.failed_flushes).collect()),
        oversized_batches: median_u64(values.iter().map(|value| value.oversized_batches).collect()),
    }
}

fn aggregate_latency(trials: Vec<&Latency>) -> Latency {
    Latency {
        samples: trials.iter().map(|latency| latency.samples).sum(),
        minimum_ns: median_u64(trials.iter().map(|latency| latency.minimum_ns).collect()),
        p50_ns: median_u64(trials.iter().map(|latency| latency.p50_ns).collect()),
        p95_ns: median_u64(trials.iter().map(|latency| latency.p95_ns).collect()),
        p99_ns: median_u64(trials.iter().map(|latency| latency.p99_ns).collect()),
        p99_9_ns: median_u64(trials.iter().map(|latency| latency.p99_9_ns).collect()),
        maximum_ns: median_u64(trials.iter().map(|latency| latency.maximum_ns).collect()),
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

fn median_option(values: Vec<u64>) -> Option<u64> {
    (!values.is_empty()).then(|| median_u64(values))
}

fn sample(samples: &[Duration], percentile: f64) -> u64 {
    let index = ((samples.len() - 1) as f64 * percentile).ceil() as usize;
    nanos(samples[index])
}

fn sample_u64(samples: &[u64], percentile: f64) -> u64 {
    let index = ((samples.len() - 1) as f64 * percentile).ceil() as usize;
    samples[index]
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

fn storage_footprint(path: &Path) -> Result<StorageFootprint, String> {
    measure_storage_footprint(path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_sample(index: u64) -> BatchWritePathSample {
        let sync_wall_ns = index + 1;
        let mut phases = rrd_lsm::WritePathPhases::default();
        phases.wal_sync = WritePathPhaseTiming {
            wall_ns: sync_wall_ns,
            thread_cpu_ns: Some(1),
        };
        phases.physical_total = WritePathPhaseTiming {
            wall_ns: sync_wall_ns + 100,
            thread_cpu_ns: Some(11),
        };
        BatchWritePathSample {
            semantic_batch_ordinal: index,
            semantic_operation_count: 1,
            semantic_append_wall_ns: sync_wall_ns + 110,
            semantic_setup_and_harness_ns: 7,
            native: WritePathSample {
                ordinal: index,
                durability: rrd_lsm::Durability::Authoritative,
                mutation_count: 3,
                encoded_payload_bytes: 64,
                wal_frame_bytes: 96,
                first_sequence: index * 3 + 1,
                last_sequence: index * 3 + 3,
                memtable_bytes_added: 128,
                lock_wait: rrd_lsm::WriteLockWait {
                    begin_mutex_wait_ns: 1,
                    commit_mutex_wait_ns: 2,
                },
                maintenance: rrd_lsm::WriteMaintenanceDelta::default(),
                phases,
                thread_resources: Some(rrd_lsm::WriteThreadResourceDelta {
                    minor_page_faults: 1,
                    ..rrd_lsm::WriteThreadResourceDelta::default()
                }),
            },
        }
    }

    #[test]
    fn latency_summary_retains_p99_9_tail() {
        let samples = (1..=1_001).map(Duration::from_nanos).collect();
        let summary = summarize(samples);
        assert_eq!(summary.p99_9_ns, 1_000);
        assert_eq!(summary.maximum_ns, 1_001);
    }

    #[test]
    fn diagnostics_flag_is_bare_and_explicit() {
        let arguments = vec![
            "--operations".into(),
            "128".into(),
            "--batch-size".into(),
            "16".into(),
            "--write-path-diagnostics".into(),
        ];
        let (config, output) = parse_parent(&arguments).unwrap();
        assert!(config.write_path_diagnostics);
        assert!(output.is_none());
    }

    #[test]
    fn write_path_summary_is_derived_from_retained_raw_samples() {
        let samples = (0..1_001).map(raw_sample).collect::<Vec<_>>();
        let evidence = build_write_path_evidence(
            &samples,
            0,
            "linux_rusage_thread".into(),
            "this_trial",
            true,
        );

        assert_eq!(evidence.raw_sample_count, 1_001);
        assert_eq!(evidence.raw_samples, samples);
        assert_eq!(evidence.phase_latency.wal_sync.p99_9_ns, 1_000);
        assert_eq!(evidence.phase_latency.wal_sync.maximum_ns, 1_001);
        assert_eq!(evidence.phase_latency.physical_unattributed.p50_ns, 100);
        assert_eq!(
            evidence
                .phase_latency
                .physical_total_off_cpu
                .unwrap()
                .p50_ns,
            590
        );
        assert_eq!(evidence.thread_resources.available_samples, 1_001);
        assert_eq!(evidence.thread_resources.minor_page_faults.total, 1_001);
    }

    #[test]
    fn write_path_pairing_requires_exact_capacity_and_sequence_continuity() {
        let expected = raw_sample(0);
        let diagnostics = WritePathDiagnostics {
            contract_version: WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION,
            capacity: 1,
            observed_samples: 1,
            dropped_samples: 0,
            thread_resource_scope: "linux_rusage_thread".into(),
            samples: vec![expected.native.clone()],
        };
        let evidence = pair_write_path_diagnostics(
            diagnostics.clone(),
            &[Duration::from_nanos(expected.semantic_append_wall_ns)],
            &[1],
            "this_trial",
            true,
        )
        .unwrap();
        assert_eq!(evidence.raw_samples, vec![expected]);

        let mut wrong_capacity = diagnostics.clone();
        wrong_capacity.capacity = 2;
        assert!(pair_write_path_diagnostics(
            wrong_capacity,
            &[Duration::from_nanos(111)],
            &[1],
            "this_trial",
            true,
        )
        .is_err());

        let mut wrong_sequence = diagnostics;
        wrong_sequence.samples[0].first_sequence = 2;
        assert!(pair_write_path_diagnostics(
            wrong_sequence,
            &[Duration::from_nanos(111)],
            &[1],
            "this_trial",
            true,
        )
        .is_err());
    }
}
