use ring::digest::{Context, SHA256};
use rrd_lsm::{
    run_physical_policy_trial, CandidateDecision, IntegratedCachePolicyObservation,
    PageCachePolicy, PhysicalPolicyConfig, PhysicalPolicyTrial, PHYSICAL_POLICY_EVIDENCE_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const EVIDENCE_FORMAT_VERSION: u16 = 4;
const MAX_TRIALS: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Arguments {
    child: bool,
    allow_dirty: bool,
    trials: usize,
    output: Option<PathBuf>,
    config: PhysicalPolicyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SourceProvenance {
    repository_root: String,
    revision: String,
    tree: String,
    branch: String,
    clean_worktree: bool,
    worktree_status: String,
    cargo_lock_sha256: String,
    executable_sha256: String,
    rustc_verbose: String,
    cargo_version: String,
    target_triple: String,
    build_profile: String,
    rustflags: String,
    command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HostProvenance {
    operating_system: String,
    architecture: String,
    kernel: String,
    cpu_model: String,
    logical_cpu_count: usize,
    total_memory_bytes: Option<u64>,
    filesystem: String,
    block_devices: String,
    cpu_frequency_policy: String,
    load_average: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Distribution {
    minimum: u64,
    p50: u64,
    p95: u64,
    p99: u64,
    p99_9: u64,
    maximum: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CodecAggregate {
    codec: String,
    adaptive_stored_bytes: u64,
    savings_basis_points: u64,
    selected_pages: u64,
    encode_nanoseconds: Distribution,
    decode_nanoseconds: Distribution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AggregateEvidence {
    trial_elapsed_nanoseconds: Distribution,
    operations_per_second: Distribution,
    point_miss_elapsed_nanoseconds: Distribution,
    point_misses_per_second: Distribution,
    peak_rss_bytes: Option<Distribution>,
    codecs: Vec<CodecAggregate>,
    persisted_filter_false_positive_parts_per_million: u64,
    persisted_filter_bytes: u64,
    lru_post_scan_hot_hits: u64,
    segmented_lru_post_scan_hot_hits: u64,
    rrflowkv_open_persisted_filter_count: u64,
    rrflowkv_open_persisted_filter_bytes: u64,
    rrflowkv_open_semantic_page_operations: u64,
    rrflowkv_open_semantic_page_bytes: u64,
    rrflowkv_open_raw_page_count: u64,
    rrflowkv_open_compressed_page_count: u64,
    rrflowkv_open_stored_page_bytes: u64,
    rrflowkv_open_logical_page_bytes: u64,
    rrflowkv_query_decompressed_bytes: u64,
    rrflowkv_reopen_filter_negatives: u64,
    rrflowkv_reopen_page_loads: u64,
    rrflowkv_exact_lru_post_scan_hot_loads: u64,
    rrflowkv_scan_resistant_post_scan_hot_loads: u64,
    rrflowkv_scan_resistant_same_scope_hits: u64,
    rrflowkv_scan_resistant_promotions: u64,
    rrflowkv_scan_resistant_protected_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PhysicalPolicyEvidence {
    evidence_format_version: u16,
    physical_policy_evidence_version: u16,
    evidence_kind: String,
    fixed_machine_integration_verification: bool,
    release_evidence_eligible: bool,
    release_ineligibility_reasons: Vec<String>,
    source: SourceProvenance,
    host: HostProvenance,
    configuration: PhysicalPolicyConfig,
    trial_count: usize,
    warmup_trials: usize,
    warmup_exit_code: i32,
    raw_trial_exit_codes: Vec<i32>,
    warmup_policy: String,
    outlier_policy: String,
    device_cache_policy: String,
    raw_trials: Vec<PhysicalPolicyTrial>,
    aggregates: AggregateEvidence,
    candidate_decisions: Vec<CandidateDecision>,
    limitations: Vec<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rrflowKV physical-policy evidence failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = parse_arguments(env::args().skip(1))?;
    arguments
        .config
        .validate()
        .map_err(|error| error.to_string())?;
    if arguments.child {
        return run_child(arguments.config);
    }
    if cfg!(debug_assertions) {
        return Err("evidence parent must be compiled with the release profile".into());
    }

    let executable =
        env::current_exe().map_err(|error| format!("cannot resolve executable: {error}"))?;
    let repository_root = repository_root()?;
    let source = source_provenance(&repository_root, &executable)?;
    if !source.clean_worktree && !arguments.allow_dirty {
        return Err(format!(
            "clean evidence mode rejected repository changes:\n{}",
            source.worktree_status
        ));
    }
    let host = host_provenance(&repository_root)?;

    let (warmup, warmup_exit_code) = spawn_child(&executable, arguments.config)?;
    require_integrated_cache_result(&warmup)?;
    let mut raw_trials = Vec::with_capacity(arguments.trials);
    let mut raw_trial_exit_codes = Vec::with_capacity(arguments.trials);
    for _ in 0..arguments.trials {
        let (trial, exit_code) = spawn_child(&executable, arguments.config)?;
        require_trial_identity(&warmup, &trial)?;
        require_integrated_cache_result(&trial)?;
        raw_trials.push(trial);
        raw_trial_exit_codes.push(exit_code);
    }
    let aggregates = aggregate_trials(&raw_trials)?;
    let candidate_decisions = raw_trials
        .first()
        .ok_or_else(|| "no raw physical-policy trials completed".to_owned())?
        .decisions
        .clone();
    for trial in &raw_trials[1..] {
        if trial.decisions != candidate_decisions {
            return Err("candidate verdicts drifted across identical child trials".into());
        }
    }

    let mut release_ineligibility_reasons = Vec::new();
    if !source.clean_worktree {
        release_ineligibility_reasons.push("source worktree was dirty".into());
    }
    release_ineligibility_reasons.extend([
        "this verifies the bounded C-06j rrflowKV scan-resistant-cache integration, not an installed end-to-end release workload".into(),
        "device cache and competing host load were observed but not controlled".into(),
        "one machine and one corpus do not establish cross-platform or all-workload behavior".into(),
    ]);
    let evidence = PhysicalPolicyEvidence {
        evidence_format_version: EVIDENCE_FORMAT_VERSION,
        physical_policy_evidence_version: PHYSICAL_POLICY_EVIDENCE_VERSION,
        evidence_kind: "rrflowkv-c06j-scan-resistant-cache-integration".into(),
        fixed_machine_integration_verification: source.clean_worktree,
        release_evidence_eligible: false,
        release_ineligibility_reasons,
        source,
        host,
        configuration: arguments.config,
        trial_count: raw_trials.len(),
        warmup_trials: 1,
        warmup_exit_code,
        raw_trial_exit_codes,
        warmup_policy: "one complete isolated child trial with identical corpus before retained trials".into(),
        outlier_policy: "retain every raw child; aggregate nearest-rank p50/p95/p99/p99.9; discard none".into(),
        device_cache_policy: "uncontrolled operating-system/device cache; same executable and corpus; child processes isolate allocator high-water state".into(),
        raw_trials,
        aggregates,
        candidate_decisions,
        limitations: vec![
            "segment v6 integrates deterministic per-page adaptive LZ4 and the default exact-byte scan-resistant immutable-page cache; exact LRU remains an explicit comparison policy".into(),
            "Zstandard and the trace-only cache simulators remain candidate mechanics over none-policy v6 page bodies, not integrated production policy".into(),
            "value separation is model-only and is categorically ineligible for adoption from this evidence".into(),
            "C-07, F-01, installed-binary, graph/BM25/vector/TurboQuant, reasoning/recall, and release gates remain open".into(),
        ],
    };
    emit_evidence(&evidence, arguments.output.as_deref())
}

fn run_child(config: PhysicalPolicyConfig) -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| format!("cannot create trial root: {error}"))?;
    let trial =
        run_physical_policy_trial(root.path(), config).map_err(|error| error.to_string())?;
    serde_json::to_writer(std::io::stdout().lock(), &trial)
        .map_err(|error| format!("cannot encode child trial: {error}"))?;
    Ok(())
}

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut child = false;
    let mut allow_dirty = false;
    let mut trials = 3usize;
    let mut output = None;
    let mut config = PhysicalPolicyConfig {
        seed: 0xca7c_4f10_0000_0001,
        records_per_family: 1024,
        versions_per_key: 2,
        value_bytes: 512,
        point_misses: 16_384,
        cache_bytes: 1024 * 1024,
    };
    let mut seen = BTreeSet::new();
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        if !seen.insert(argument.clone()) {
            return Err(format!("duplicate argument {argument}"));
        }
        match argument.as_str() {
            "--child" => child = true,
            "--allow-dirty" => allow_dirty = true,
            "--trials" => trials = parse_usize(&mut arguments, "--trials")?,
            "--seed" => config.seed = parse_u64(&mut arguments, "--seed")?,
            "--records-per-family" => {
                config.records_per_family = parse_usize(&mut arguments, "--records-per-family")?
            }
            "--versions-per-key" => {
                config.versions_per_key = parse_usize(&mut arguments, "--versions-per-key")?
            }
            "--value-bytes" => config.value_bytes = parse_usize(&mut arguments, "--value-bytes")?,
            "--misses" => config.point_misses = parse_usize(&mut arguments, "--misses")?,
            "--cache-bytes" => config.cache_bytes = parse_usize(&mut arguments, "--cache-bytes")?,
            "--output" => {
                output = Some(PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "--output requires a path".to_owned())?,
                ));
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if trials == 0 || trials > MAX_TRIALS {
        return Err(format!("--trials must be in 1..={MAX_TRIALS}"));
    }
    if child && (allow_dirty || output.is_some()) {
        return Err("child mode cannot accept --allow-dirty or --output".into());
    }
    Ok(Arguments {
        child,
        allow_dirty,
        trials,
        output,
        config,
    })
}

fn parse_usize(arguments: &mut impl Iterator<Item = String>, name: &str) -> Result<usize, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{name} requires a value"))?
        .parse()
        .map_err(|_| format!("{name} requires a base-10 usize"))
}

fn parse_u64(arguments: &mut impl Iterator<Item = String>, name: &str) -> Result<u64, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{name} requires a value"))?
        .parse()
        .map_err(|_| format!("{name} requires a base-10 u64"))
}

fn spawn_child(
    executable: &Path,
    config: PhysicalPolicyConfig,
) -> Result<(PhysicalPolicyTrial, i32), String> {
    let output = Command::new(executable)
        .args([
            "--child".to_owned(),
            "--seed".to_owned(),
            config.seed.to_string(),
            "--records-per-family".to_owned(),
            config.records_per_family.to_string(),
            "--versions-per-key".to_owned(),
            config.versions_per_key.to_string(),
            "--value-bytes".to_owned(),
            config.value_bytes.to_string(),
            "--misses".to_owned(),
            config.point_misses.to_string(),
            "--cache-bytes".to_owned(),
            config.cache_bytes.to_string(),
        ])
        .output()
        .map_err(|error| format!("cannot start isolated child: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "isolated child failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !output.stderr.is_empty() {
        return Err(format!(
            "isolated child emitted unexpected stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let exit_code = output
        .status
        .code()
        .ok_or_else(|| "successful isolated child omitted an exit code".to_owned())?;
    let trial = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("cannot decode isolated child JSON: {error}"))?;
    Ok((trial, exit_code))
}

fn require_trial_identity(
    expected: &PhysicalPolicyTrial,
    actual: &PhysicalPolicyTrial,
) -> Result<(), String> {
    if expected.evidence_version != actual.evidence_version
        || expected.evidence_scope != actual.evidence_scope
        || expected.config != actual.config
        || expected.corpus_digest != actual.corpus_digest
        || expected.operation_count != actual.operation_count
        || expected.key_count != actual.key_count
        || expected.row_group_count != actual.row_group_count
        || expected.page_count != actual.page_count
        || expected.segment_format_version != actual.segment_format_version
        || expected.segment_physical_bytes != actual.segment_physical_bytes
        || expected.logical_page_bytes != actual.logical_page_bytes
        || expected.codecs.len() != actual.codecs.len()
        || expected
            .codecs
            .iter()
            .zip(&actual.codecs)
            .any(|(expected, actual)| !same_codec_identity(expected, actual))
        || expected.persisted_row_group_filter != actual.persisted_row_group_filter
        || expected.cache_candidates != actual.cache_candidates
        || expected.integrated_cache_policies != actual.integrated_cache_policies
        || expected.value_placement_candidate != actual.value_placement_candidate
        || expected.families != actual.families
        || !same_reopened_identity(&expected.reopened_point_miss, &actual.reopened_point_miss)
        || expected.decisions != actual.decisions
        || expected.limitations != actual.limitations
    {
        return Err("deterministic trial identity drifted from warm-up".into());
    }
    Ok(())
}

fn integrated_cache_pair(
    trial: &PhysicalPolicyTrial,
) -> Result<
    (
        &IntegratedCachePolicyObservation,
        &IntegratedCachePolicyObservation,
    ),
    String,
> {
    let exact = trial
        .integrated_cache_policies
        .iter()
        .find(|observation| observation.policy == PageCachePolicy::ExactLru.kind())
        .ok_or_else(|| "integrated exact-LRU observation is absent".to_owned())?;
    let scan_resistant = trial
        .integrated_cache_policies
        .iter()
        .find(|observation| observation.policy == PageCachePolicy::default().kind())
        .ok_or_else(|| "integrated scan-resistant observation is absent".to_owned())?;
    Ok((exact, scan_resistant))
}

fn require_integrated_cache_result(trial: &PhysicalPolicyTrial) -> Result<(), String> {
    let (exact, scan_resistant) = integrated_cache_pair(trial)?;
    let configured_capacity = u64::try_from(trial.config.cache_bytes)
        .map_err(|_| "configured cache capacity exceeds u64".to_owned())?;
    let same_identity = exact.snapshot_sequence == scan_resistant.snapshot_sequence
        && exact.manifest_digest == scan_resistant.manifest_digest
        && exact.semantic_digest == scan_resistant.semantic_digest
        && exact.projected_row_digest == scan_resistant.projected_row_digest;
    let exact_regions_are_empty = exact.probationary_resident_bytes == 0
        && exact.protected_resident_bytes == 0
        && exact.probationary_entries == 0
        && exact.protected_entries == 0;
    let scan_regions_are_exact = scan_resistant
        .probationary_resident_bytes
        .checked_add(scan_resistant.protected_resident_bytes)
        == Some(scan_resistant.final_resident_bytes)
        && scan_resistant
            .probationary_entries
            .checked_add(scan_resistant.protected_entries)
            == Some(scan_resistant.final_entries);
    if !exact.semantic_identity_exact
        || !scan_resistant.semantic_identity_exact
        || !same_identity
        || !exact.exact_capacity_respected
        || !scan_resistant.exact_capacity_respected
        || !exact_regions_are_empty
        || !scan_regions_are_exact
        || exact.capacity_bytes != configured_capacity
        || scan_resistant.capacity_bytes != configured_capacity
        || exact.post_scan_hot_loads == 0
        || scan_resistant.post_scan_hot_loads >= exact.post_scan_hot_loads
        || scan_resistant.same_scope_hits == 0
        || scan_resistant.promotions == 0
        || scan_resistant.protected_entries == 0
    {
        return Err(format!(
            "integrated cache evidence failed: identity_exact={same_identity}, exact_post_scan_loads={}, scan_resistant_post_scan_loads={}, same_scope_hits={}, promotions={}, protected_entries={}",
            exact.post_scan_hot_loads,
            scan_resistant.post_scan_hot_loads,
            scan_resistant.same_scope_hits,
            scan_resistant.promotions,
            scan_resistant.protected_entries
        ));
    }
    Ok(())
}

fn same_codec_identity(
    expected: &rrd_lsm::CodecObservation,
    actual: &rrd_lsm::CodecObservation,
) -> bool {
    expected.codec == actual.codec
        && expected.pages == actual.pages
        && expected.logical_bytes == actual.logical_bytes
        && expected.codec_bytes == actual.codec_bytes
        && expected.adaptive_stored_bytes == actual.adaptive_stored_bytes
        && expected.adaptive_selected_pages == actual.adaptive_selected_pages
        && expected.savings_basis_points == actual.savings_basis_points
        && expected.round_trip_exact == actual.round_trip_exact
}

fn same_reopened_identity(
    expected: &rrd_lsm::ReopenedPointMissObservation,
    actual: &rrd_lsm::ReopenedPointMissObservation,
) -> bool {
    expected.integrated_rrflowkv == actual.integrated_rrflowkv
        && expected.open_none_policy_segment_count == actual.open_none_policy_segment_count
        && expected.open_adaptive_lz4_policy_segment_count
            == actual.open_adaptive_lz4_policy_segment_count
        && expected.open_raw_page_count == actual.open_raw_page_count
        && expected.open_compressed_page_count == actual.open_compressed_page_count
        && expected.open_stored_page_bytes == actual.open_stored_page_bytes
        && expected.open_logical_page_bytes == actual.open_logical_page_bytes
        && expected.open_persisted_filter_count == actual.open_persisted_filter_count
        && expected.open_persisted_filter_bytes == actual.open_persisted_filter_bytes
        && expected.open_semantic_page_operations == actual.open_semantic_page_operations
        && expected.open_semantic_page_bytes == actual.open_semantic_page_bytes
        && expected.misses_verified == actual.misses_verified
        && expected.hit_samples_verified == actual.hit_samples_verified
        && expected.segment_physical_bytes == actual.segment_physical_bytes
        && expected.filter_checks == actual.filter_checks
        && expected.filter_negatives == actual.filter_negatives
        && expected.page_cache_hits == actual.page_cache_hits
        && expected.page_cache_misses == actual.page_cache_misses
        && expected.page_loads == actual.page_loads
        && expected.bytes_read == actual.bytes_read
        && expected.bytes_decoded == actual.bytes_decoded
        && expected.bytes_decompressed == actual.bytes_decompressed
        && expected.final_cache_resident_bytes == actual.final_cache_resident_bytes
}

fn aggregate_trials(trials: &[PhysicalPolicyTrial]) -> Result<AggregateEvidence, String> {
    let first = trials
        .first()
        .ok_or_else(|| "cannot aggregate zero trials".to_owned())?;
    let codecs = first
        .codecs
        .iter()
        .enumerate()
        .map(|(index, codec)| {
            let observations = trials
                .iter()
                .map(|trial| {
                    trial
                        .codecs
                        .get(index)
                        .filter(|actual| actual.codec == codec.codec)
                        .ok_or_else(|| format!("codec {} is absent or reordered", codec.codec))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if observations.iter().any(|actual| {
                actual.adaptive_stored_bytes != codec.adaptive_stored_bytes
                    || actual.savings_basis_points != codec.savings_basis_points
                    || actual.adaptive_selected_pages != codec.adaptive_selected_pages
                    || !actual.round_trip_exact
            }) {
                return Err(format!("codec {} deterministic bytes drifted", codec.codec));
            }
            Ok(CodecAggregate {
                codec: codec.codec.clone(),
                adaptive_stored_bytes: codec.adaptive_stored_bytes,
                savings_basis_points: codec.savings_basis_points,
                selected_pages: codec.adaptive_selected_pages,
                encode_nanoseconds: distribution(
                    observations
                        .iter()
                        .map(|actual| actual.encode_nanoseconds)
                        .collect(),
                )?,
                decode_nanoseconds: distribution(
                    observations
                        .iter()
                        .map(|actual| actual.decode_nanoseconds)
                        .collect(),
                )?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let peak = trials
        .iter()
        .map(|trial| trial.peak_rss_bytes)
        .collect::<Option<Vec<_>>>()
        .map(distribution)
        .transpose()?;
    let lru = first
        .cache_candidates
        .iter()
        .find(|cache| cache.policy == "exact-byte-lru")
        .ok_or_else(|| "LRU observation is absent".to_owned())?;
    let segmented = first
        .cache_candidates
        .iter()
        .find(|cache| cache.policy == "exact-byte-segmented-lru-candidate")
        .ok_or_else(|| "segmented-LRU observation is absent".to_owned())?;
    let (exact_integrated, scan_resistant_integrated) = integrated_cache_pair(first)?;
    Ok(AggregateEvidence {
        trial_elapsed_nanoseconds: distribution(
            trials
                .iter()
                .map(|trial| trial.elapsed_nanoseconds)
                .collect(),
        )?,
        operations_per_second: distribution(
            trials
                .iter()
                .map(|trial| {
                    throughput_per_second(trial.operation_count, trial.elapsed_nanoseconds)
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        point_miss_elapsed_nanoseconds: distribution(
            trials
                .iter()
                .map(|trial| trial.reopened_point_miss.elapsed_nanoseconds)
                .collect(),
        )?,
        point_misses_per_second: distribution(
            trials
                .iter()
                .map(|trial| {
                    throughput_per_second(
                        trial.reopened_point_miss.misses_verified,
                        trial.reopened_point_miss.elapsed_nanoseconds,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        peak_rss_bytes: peak,
        codecs,
        persisted_filter_false_positive_parts_per_million: first
            .persisted_row_group_filter
            .false_positive_parts_per_million,
        persisted_filter_bytes: first.persisted_row_group_filter.serialized_bytes,
        lru_post_scan_hot_hits: lru.post_scan_hot_hits,
        segmented_lru_post_scan_hot_hits: segmented.post_scan_hot_hits,
        rrflowkv_open_persisted_filter_count: first.reopened_point_miss.open_persisted_filter_count,
        rrflowkv_open_persisted_filter_bytes: first.reopened_point_miss.open_persisted_filter_bytes,
        rrflowkv_open_semantic_page_operations: first
            .reopened_point_miss
            .open_semantic_page_operations,
        rrflowkv_open_semantic_page_bytes: first.reopened_point_miss.open_semantic_page_bytes,
        rrflowkv_open_raw_page_count: first.reopened_point_miss.open_raw_page_count,
        rrflowkv_open_compressed_page_count: first.reopened_point_miss.open_compressed_page_count,
        rrflowkv_open_stored_page_bytes: first.reopened_point_miss.open_stored_page_bytes,
        rrflowkv_open_logical_page_bytes: first.reopened_point_miss.open_logical_page_bytes,
        rrflowkv_query_decompressed_bytes: first.reopened_point_miss.bytes_decompressed,
        rrflowkv_reopen_filter_negatives: first.reopened_point_miss.filter_negatives,
        rrflowkv_reopen_page_loads: first.reopened_point_miss.page_loads,
        rrflowkv_exact_lru_post_scan_hot_loads: exact_integrated.post_scan_hot_loads,
        rrflowkv_scan_resistant_post_scan_hot_loads: scan_resistant_integrated.post_scan_hot_loads,
        rrflowkv_scan_resistant_same_scope_hits: scan_resistant_integrated.same_scope_hits,
        rrflowkv_scan_resistant_promotions: scan_resistant_integrated.promotions,
        rrflowkv_scan_resistant_protected_entries: scan_resistant_integrated.protected_entries,
    })
}

fn distribution(mut values: Vec<u64>) -> Result<Distribution, String> {
    if values.is_empty() {
        return Err("cannot summarize an empty distribution".into());
    }
    values.sort_unstable();
    Ok(Distribution {
        minimum: values[0],
        p50: nearest_rank(&values, 500),
        p95: nearest_rank(&values, 950),
        p99: nearest_rank(&values, 990),
        p99_9: nearest_rank(&values, 999),
        maximum: *values.last().unwrap(),
    })
}

fn nearest_rank(values: &[u64], permille: usize) -> u64 {
    let rank = values.len().saturating_mul(permille).saturating_add(999) / 1000;
    values[rank.saturating_sub(1).min(values.len() - 1)]
}

fn throughput_per_second(units: u64, elapsed_nanoseconds: u64) -> Result<u64, String> {
    if elapsed_nanoseconds == 0 {
        return Err("cannot calculate throughput from zero elapsed nanoseconds".into());
    }
    units
        .checked_mul(1_000_000_000)
        .ok_or_else(|| "throughput numerator overflow".to_owned())
        .map(|numerator| numerator / elapsed_nanoseconds)
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot resolve repository root from CARGO_MANIFEST_DIR".into())
}

fn source_provenance(root: &Path, executable: &Path) -> Result<SourceProvenance, String> {
    let status = command_at(
        root,
        "git",
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    let rustc_verbose = command_at(root, "rustc", &["--version", "--verbose"])?;
    let target_triple = rustc_verbose
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "rustc --version --verbose omitted host triple".to_owned())?
        .into();
    let lock = root.join("Cargo.lock");
    if !lock.is_file() {
        return Err("Cargo.lock is absent".into());
    }
    Ok(SourceProvenance {
        repository_root: ".".into(),
        revision: command_at(root, "git", &["rev-parse", "HEAD^{commit}"])?
            .trim()
            .into(),
        tree: command_at(root, "git", &["rev-parse", "HEAD^{tree}"])?
            .trim()
            .into(),
        branch: command_at(root, "git", &["branch", "--show-current"])?
            .trim()
            .into(),
        clean_worktree: status.is_empty(),
        worktree_status: status,
        cargo_lock_sha256: sha256_file(&lock)?,
        executable_sha256: sha256_file(executable)?,
        rustc_verbose,
        cargo_version: command_at(root, "cargo", &["--version"])?.trim().into(),
        target_triple,
        build_profile: "release".into(),
        rustflags: option_env!("RUSTFLAGS").unwrap_or("").into(),
        command: env::args().collect(),
    })
}

fn host_provenance(root: &Path) -> Result<HostProvenance, String> {
    let cpu_model = read_cpu_model().ok_or_else(|| "cannot determine CPU model".to_owned())?;
    let logical_cpu_count = std::thread::available_parallelism()
        .map_err(|error| format!("cannot determine logical CPU count: {error}"))?
        .get();
    let kernel = command_at(root, "uname", &["-srvmo"])?;
    let filesystem = command_at(
        root,
        "findmnt",
        &["-T", ".", "-n", "-o", "SOURCE,FSTYPE,OPTIONS"],
    )?;
    let block_devices = optional_command_at(
        root,
        "lsblk",
        &["-d", "-n", "-o", "NAME,MODEL,SIZE,ROTA,TYPE"],
    );
    Ok(HostProvenance {
        operating_system: env::consts::OS.into(),
        architecture: env::consts::ARCH.into(),
        kernel: kernel.trim().into(),
        cpu_model,
        logical_cpu_count,
        total_memory_bytes: total_memory_bytes(),
        filesystem: filesystem.trim().into(),
        block_devices: block_devices.trim().into(),
        cpu_frequency_policy: cpu_frequency_policy(),
        load_average: std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_else(|_| "unavailable".into())
            .trim()
            .into(),
    })
}

fn command_at(root: &Path, program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| format!("cannot run {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {:?} failed with {}: {}",
            arguments,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("{program} output is not UTF-8: {error}"))
}

fn optional_command_at(root: &Path, program: &str, arguments: &[&str]) -> String {
    command_at(root, program, arguments).unwrap_or_else(|error| format!("unavailable: {error}"))
}

fn read_cpu_model() -> Option<String> {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    cpuinfo.lines().find_map(|line| {
        line.split_once(':').and_then(|(name, value)| {
            matches!(name.trim(), "model name" | "Hardware" | "Processor")
                .then(|| value.trim().to_owned())
        })
    })
}

fn total_memory_bytes() -> Option<u64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kib = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    kib.checked_mul(1024)
}

fn cpu_frequency_policy() -> String {
    let governor = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor");
    std::fs::read_to_string(governor)
        .unwrap_or_else(|_| "unavailable".into())
        .trim()
        .into()
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        File::open(path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut context = Context::new(&SHA256);
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        context.update(&buffer[..read]);
    }
    Ok(context
        .finish()
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn emit_evidence(evidence: &PhysicalPolicyEvidence, output: Option<&Path>) -> Result<(), String> {
    match output {
        Some(path) => {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
            let mut temporary = tempfile::NamedTempFile::new_in(parent)
                .map_err(|error| format!("cannot create evidence temporary file: {error}"))?;
            serde_json::to_writer_pretty(temporary.as_file_mut(), evidence)
                .map_err(|error| format!("cannot encode evidence: {error}"))?;
            temporary
                .as_file_mut()
                .write_all(b"\n")
                .map_err(|error| format!("cannot terminate evidence JSON: {error}"))?;
            temporary
                .as_file()
                .sync_all()
                .map_err(|error| format!("cannot sync evidence JSON: {error}"))?;
            temporary
                .persist(path)
                .map_err(|error| format!("cannot publish {}: {}", path.display(), error.error))?;
            Ok(())
        }
        None => {
            serde_json::to_writer_pretty(std::io::stdout().lock(), evidence)
                .map_err(|error| format!("cannot encode evidence: {error}"))?;
            println!();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_reject_unknown_duplicate_and_zero_values() {
        assert!(parse_arguments(["--unknown".into()].into_iter()).is_err());
        assert!(parse_arguments(
            ["--seed".into(), "1".into(), "--seed".into(), "2".into()].into_iter()
        )
        .is_err());
        assert!(parse_arguments(["--trials".into(), "0".into()].into_iter()).is_err());
    }

    #[test]
    fn distribution_uses_nearest_rank_and_retains_extremes() {
        let observed = distribution(vec![9, 1, 5, 3, 7]).unwrap();
        assert_eq!(observed.minimum, 1);
        assert_eq!(observed.p50, 5);
        assert_eq!(observed.p95, 9);
        assert_eq!(observed.maximum, 9);
        assert_eq!(throughput_per_second(10, 2_000_000_000).unwrap(), 5);
        assert!(throughput_per_second(1, 0).is_err());
    }

    #[test]
    fn child_identity_rejects_deterministic_codec_and_reopen_drift() {
        let root = tempfile::tempdir().unwrap();
        let trial = run_physical_policy_trial(
            root.path(),
            PhysicalPolicyConfig {
                seed: 7,
                records_per_family: 32,
                versions_per_key: 2,
                value_bytes: 128,
                point_misses: 512,
                cache_bytes: 32 * 1024,
            },
        )
        .unwrap();

        let mut codec_drift = trial.clone();
        codec_drift.codecs[0].codec_bytes += 1;
        assert!(require_trial_identity(&trial, &codec_drift).is_err());

        let mut reopen_drift = trial.clone();
        reopen_drift.reopened_point_miss.page_loads += 1;
        assert!(require_trial_identity(&trial, &reopen_drift).is_err());

        require_integrated_cache_result(&trial).unwrap();
        let mut invalid_cache = trial.clone();
        invalid_cache
            .integrated_cache_policies
            .iter_mut()
            .find(|cache| cache.policy == PageCachePolicy::default().kind())
            .unwrap()
            .same_scope_hits = 0;
        assert!(require_integrated_cache_result(&invalid_cache).is_err());
        assert!(require_trial_identity(&trial, &invalid_cache).is_err());
    }
}
