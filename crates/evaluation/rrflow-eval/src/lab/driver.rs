//! Runs one system at a time so systems never compete for a machine, then combines
//! the runs into one report. A failing system stays visible and cannot vanish.
use super::adapter::{CollectionSpec, PointBatch, SearchRequest, SystemAdapter};
use super::corpus::{exact_answers, generate, GroundTruth, VectorSet};
use super::manifest::{
    Availability, SearchSetting, SystemBinding, SystemKind, VectorComparisonManifest,
};
use super::manifest::{CorpusSource, CorpusSpec, QualitySpec, WorkloadSpec};
use super::report::TestIdentity;
use super::report::{
    ArtifactReference, DimensionDefinition, LabEvidenceClass, LabReport, MetricDefinition,
    MetricDirection, Observation, ReportSystem, SystemStatus, Verdict, VerdictOutcome,
    ViewDefinition, REPORT_SCHEMA_VERSION,
};
use super::{canonical_json, require, sha256_hex, Result};
use crate::storage::{Digest, Id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const RUN_SCHEMA_VERSION: u16 = 1;
/// Stop hammering a system that has answered nothing but errors.
const ERRORS_BEFORE_GIVING_UP: usize = 20;
const FLOOR_REQUESTS_PER_CLIENT: usize = 32;
const Z95: f64 = 1.96;

/// Transient operator observations. These are not a durable audit or a run result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case", deny_unknown_fields)]
pub enum LabProgress {
    Prepared,
    IngestBatchStarted {
        completed: u64,
        total: u64,
    },
    IngestBatchCompleted {
        completed: u64,
        total: u64,
        elapsed_ms: u64,
    },
    IndexBuildStarted,
    IndexReady {
        elapsed_ms: u64,
    },
    CaseStarted {
        setting: Id,
        filtered: bool,
        concurrency: u32,
    },
    CaseCompleted {
        setting: Id,
        filtered: bool,
        concurrency: u32,
        attempted: u64,
        succeeded: u64,
        errors: u64,
        truncated: bool,
    },
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IngestResult {
    pub points: u64,
    pub seconds: f64,
    pub points_per_second: f64,
    pub ready_seconds: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaseResult {
    pub setting: Id,
    pub filtered: bool,
    pub concurrency: u32,
    pub attempted: u64,
    pub succeeded: u64,
    pub errors: u64,
    /// The case reached its time limit before finishing every query.
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_error: Option<String>,
    pub wall_seconds: f64,
    pub queries_per_second: f64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub latency_max_ms: f64,
    pub recall_mean: f64,
    pub recall_low: f64,
    pub recall_high: f64,
    pub recall_min: f64,
}

/// What a system's cheapest request costs, the one that reads its version. A search cannot
/// be faster than this, so a latency close to it measured the transport and not the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RequestFloor {
    pub clients: u32,
    pub p50_ms: f64,
    pub p95_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SystemRun {
    pub schema_version: u16,
    pub system: Id,
    pub identities: BTreeMap<String, Digest>,
    /// The binding this run was made under, so a changed binding cannot reuse it.
    pub binding_sha256: Digest,
    pub environment: BTreeMap<String, String>,
    pub status: SystemStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingest: Option<IngestResult>,
    pub cases: Vec<CaseResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_bytes: Option<u64>,
    /// What served the searches, in the system's own words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_floor: Option<RequestFloor>,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: u64,
}

/// Every latency and recall sample behind a case, retained by digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCase {
    pub setting: Id,
    pub filtered: bool,
    pub concurrency: u32,
    pub latency_ns: Vec<u64>,
    pub recall: Vec<f64>,
    /// Every refused or failed request, including its measured duration.
    pub failures: Vec<RawFailure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFailure {
    pub latency_ns: u64,
    /// Fixed category; do not duplicate untrusted server text in each sample.
    pub category: String,
}

pub struct Inputs {
    pub points: VectorSet,
    pub queries: VectorSet,
    pub truth: GroundTruth,
    pub filtered_truth: Option<GroundTruth>,
    pub identities: BTreeMap<String, Digest>,
}

fn digest(text: String) -> Result<Digest> {
    Digest::try_from(text)
}

/// What every run in a report must share. Systems stay out of it, so adding one to a
/// test leaves the runs already made usable. Each run keeps its own binding's digest.
pub fn test_sha256(manifest: &VectorComparisonManifest) -> Result<String> {
    #[derive(Serialize)]
    struct Shared<'a> {
        schema_version: u16,
        test: &'a TestIdentity,
        corpus: &'a CorpusSpec,
        workload: &'a WorkloadSpec,
        quality: &'a QualitySpec,
    }
    Ok(sha256_hex(&canonical_json(&Shared {
        schema_version: manifest.schema_version,
        test: &manifest.test,
        corpus: &manifest.corpus,
        workload: &manifest.workload,
        quality: &manifest.quality,
    })?))
}

pub fn binding_sha256(binding: &SystemBinding) -> Digest {
    // A binding is plain data, so encoding it cannot fail.
    let encoded = canonical_json(binding).expect("a system binding encodes as JSON");
    Digest::try_from(sha256_hex(&encoded)).expect("a SHA-256 is a valid digest")
}

pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

/// Computed before any system is contacted, so an answer cannot influence its grade.
pub fn prepare_inputs(manifest: &VectorComparisonManifest) -> Result<Inputs> {
    let threads = std::thread::available_parallelism().map_or(1, |count| count.get());
    let (points, queries) = generate(&manifest.corpus, manifest.workload.queries)?;
    let k = manifest.workload.k as usize;
    let metric = manifest.corpus.metric;
    let truth = exact_answers(&points, &queries, metric, k, false, threads)?;
    let filtered_truth = if manifest.workload.filtered {
        Some(exact_answers(&points, &queries, metric, k, true, threads)?)
    } else {
        None
    };
    let mut identities = BTreeMap::from([
        ("test".to_string(), digest(test_sha256(manifest)?)?),
        ("points".to_string(), digest(points.digest())?),
        ("queries".to_string(), digest(queries.digest())?),
        ("truth".to_string(), digest(truth.digest())?),
    ]);
    if let Some(filtered) = &filtered_truth {
        identities.insert("filtered_truth".to_string(), digest(filtered.digest())?);
    }
    Ok(Inputs {
        points,
        queries,
        truth,
        filtered_truth,
        identities,
    })
}

fn percentile(sorted: &[u64], percent: u64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (sorted.len() as u64 * percent).div_ceil(100).max(1) as usize;
    sorted[rank - 1] as f64 / 1_000_000.0
}

/// `None` if the system stops answering, which the cases have already recorded.
fn measure_request_floor(adapter: &dyn SystemAdapter, clients: u32) -> Option<RequestFloor> {
    let samples = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..clients {
            scope.spawn(|| {
                let mut local = Vec::with_capacity(FLOOR_REQUESTS_PER_CLIENT);
                for _ in 0..FLOOR_REQUESTS_PER_CLIENT {
                    let started = Instant::now();
                    if adapter.observed_version().is_err() {
                        return;
                    }
                    local.push(started.elapsed().as_nanos() as u64);
                }
                if let Ok(mut all) = samples.lock() {
                    all.extend(local);
                }
            });
        }
    });
    let mut sorted = samples.into_inner().ok()?;
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_unstable();
    Some(RequestFloor {
        clients,
        p50_ms: percentile(&sorted, 50),
        p95_ms: percentile(&sorted, 95),
    })
}

struct CaseRequest<'a> {
    setting: &'a SearchSetting,
    truth: &'a GroundTruth,
    filtered: bool,
    concurrency: u32,
}

struct Sample {
    latency_ns: u64,
    outcome: std::result::Result<f64, String>,
}

fn measure_case(
    adapter: &dyn SystemAdapter,
    manifest: &VectorComparisonManifest,
    inputs: &Inputs,
    request: &CaseRequest<'_>,
) -> (CaseResult, RawCase) {
    let queries = inputs.queries.len();
    let k = manifest.workload.k;
    let search = |index: usize| {
        let query = index % queries;
        let category = request.filtered.then(|| inputs.queries.categories[query]);
        let started = Instant::now();
        let answer = adapter.search(&SearchRequest {
            vector: inputs.queries.vector(query),
            k,
            category,
            setting: request.setting,
        });
        Sample {
            latency_ns: started.elapsed().as_nanos() as u64,
            outcome: answer.map(|ids| request.truth.recall(query, &ids)),
        }
    };
    let limit = Duration::from_secs(manifest.workload.max_case_seconds);
    let warmup_deadline = Instant::now() + limit;
    // A system refusing every query has nothing to warm, and a refusal can be slow.
    let mut refused = 0;
    for index in 0..manifest.workload.warmup_queries as usize {
        if Instant::now() >= warmup_deadline || refused >= ERRORS_BEFORE_GIVING_UP {
            break;
        }
        refused = if search(index).outcome.is_err() {
            refused + 1
        } else {
            0
        };
    }
    let next = AtomicUsize::new(0);
    let samples = Mutex::new(Vec::with_capacity(queries));
    let started = Instant::now();
    let deadline = started + limit;
    std::thread::scope(|scope| {
        for _ in 0..request.concurrency {
            scope.spawn(|| {
                let mut local = Vec::new();
                loop {
                    if Instant::now() >= deadline {
                        break;
                    }
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    if index >= queries {
                        break;
                    }
                    let sample = search(index);
                    let failed = sample.outcome.is_err();
                    local.push(sample);
                    if failed
                        && local.len() >= ERRORS_BEFORE_GIVING_UP
                        && local.iter().all(|entry: &Sample| entry.outcome.is_err())
                    {
                        break;
                    }
                }
                if let Ok(mut all) = samples.lock() {
                    all.extend(local);
                }
            });
        }
    });
    let wall_seconds = started.elapsed().as_secs_f64();
    let samples = samples.into_inner().unwrap_or_default();
    let attempted = samples.len() as u64;
    let mut latency_ns = Vec::with_capacity(samples.len());
    let mut recall = Vec::with_capacity(samples.len());
    let mut first_error = None;
    let mut failures = Vec::new();
    for sample in samples {
        match sample.outcome {
            Ok(value) => {
                latency_ns.push(sample.latency_ns);
                recall.push(value);
            }
            Err(error) => {
                // Bound diagnostic text without splitting UTF-8 characters.
                let error: String = error.chars().take(512).collect();
                first_error.get_or_insert_with(|| error.clone());
                failures.push(RawFailure {
                    latency_ns: sample.latency_ns,
                    category: "request_failed".to_string(),
                });
            }
        }
    }
    let succeeded = latency_ns.len() as u64;
    let mut sorted = latency_ns.clone();
    sorted.sort_unstable();
    let count = recall.len().max(1) as f64;
    // A fold, because an empty f64 sum is negative zero.
    let mean = recall.iter().fold(0.0, |total, value| total + value) / count;
    let spread = if recall.len() > 1 {
        let variance = recall
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (recall.len() - 1) as f64;
        Z95 * (variance / recall.len() as f64).sqrt()
    } else {
        0.0
    };
    let case = CaseResult {
        setting: request.setting.id.clone(),
        filtered: request.filtered,
        concurrency: request.concurrency,
        attempted,
        succeeded,
        errors: attempted - succeeded,
        truncated: attempted < queries as u64,
        first_error,
        wall_seconds,
        queries_per_second: if wall_seconds > 0.0 {
            succeeded as f64 / wall_seconds
        } else {
            0.0
        },
        latency_p50_ms: percentile(&sorted, 50),
        latency_p95_ms: percentile(&sorted, 95),
        latency_p99_ms: percentile(&sorted, 99),
        latency_max_ms: sorted
            .last()
            .map_or(0.0, |value| *value as f64 / 1_000_000.0),
        recall_mean: mean,
        recall_low: (mean - spread).max(0.0),
        recall_high: (mean + spread).min(1.0),
        recall_min: if recall.is_empty() {
            0.0
        } else {
            recall.iter().copied().fold(1.0, f64::min)
        },
    };
    let raw = RawCase {
        setting: request.setting.id.clone(),
        filtered: request.filtered,
        concurrency: request.concurrency,
        latency_ns,
        recall,
        failures,
    };
    (case, raw)
}

fn probe(argv: &Option<Vec<String>>) -> Option<u64> {
    let argv = argv.as_ref()?;
    let (program, arguments) = argv.split_first()?;
    let output = std::process::Command::new(program)
        .args(arguments)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

fn collection_name(manifest: &VectorComparisonManifest) -> String {
    format!("lab_{}", manifest.test.id.as_str().replace(['.', '-'], "_"))
}

fn run_phases(
    manifest: &VectorComparisonManifest,
    binding: &SystemBinding,
    adapter: &dyn SystemAdapter,
    inputs: &Inputs,
    run: &mut SystemRun,
    raw: &mut Vec<RawCase>,
    progress: &mut dyn FnMut(LabProgress),
) -> Result<()> {
    run.observed_version = Some(adapter.observed_version()?);
    adapter.prepare(&CollectionSpec {
        name: &collection_name(manifest),
        dimensions: manifest.corpus.dimensions,
        metric: manifest.corpus.metric,
        categories: manifest.corpus.categories,
        index: &binding.index,
    })?;
    progress(LabProgress::Prepared);
    let points = &inputs.points;
    let ids: Vec<u64> = (0..points.len()).map(VectorSet::point_id).collect();
    let batch_size = manifest.workload.ingest_batch as usize;
    let started = Instant::now();
    let mut start = 0;
    while start < points.len() {
        let end = (start + batch_size).min(points.len());
        progress(LabProgress::IngestBatchStarted {
            completed: start as u64,
            total: points.len() as u64,
        });
        adapter.ingest(&PointBatch {
            ids: &ids[start..end],
            vectors: &points.values[start * points.dimensions..end * points.dimensions],
            categories: &points.categories[start..end],
            dimensions: points.dimensions,
        })?;
        start = end;
        progress(LabProgress::IngestBatchCompleted {
            completed: start as u64,
            total: points.len() as u64,
            elapsed_ms: elapsed_ms(started),
        });
    }
    let seconds = started.elapsed().as_secs_f64();
    let ready_started = Instant::now();
    progress(LabProgress::IndexBuildStarted);
    let ready = adapter.finish_ingest(
        ready_started + Duration::from_millis(manifest.workload.readiness_timeout_ms),
    );
    // A system that ingested and then could not become ready still has an ingest rate.
    run.ingest = Some(IngestResult {
        points: points.len() as u64,
        seconds,
        points_per_second: points.len() as f64 / seconds.max(f64::EPSILON),
        ready_seconds: ready_started.elapsed().as_secs_f64(),
    });
    ready?;
    progress(LabProgress::IndexReady {
        elapsed_ms: elapsed_ms(ready_started),
    });
    for setting in &binding.search_ladder {
        for filtered in [false, true] {
            let truth = if filtered {
                match &inputs.filtered_truth {
                    Some(truth) => truth,
                    None => continue,
                }
            } else {
                &inputs.truth
            };
            for concurrency in &manifest.workload.concurrency {
                progress(LabProgress::CaseStarted {
                    setting: setting.id.clone(),
                    filtered,
                    concurrency: *concurrency,
                });
                let (case, samples) = measure_case(
                    adapter,
                    manifest,
                    inputs,
                    &CaseRequest {
                        setting,
                        truth,
                        filtered,
                        concurrency: *concurrency,
                    },
                );
                progress(LabProgress::CaseCompleted {
                    setting: setting.id.clone(),
                    filtered,
                    concurrency: *concurrency,
                    attempted: case.attempted,
                    succeeded: case.succeeded,
                    errors: case.errors,
                    truncated: case.truncated,
                });
                run.cases.push(case);
                raw.push(samples);
            }
        }
    }
    run.request_floor = measure_request_floor(
        adapter,
        manifest
            .workload
            .concurrency
            .iter()
            .copied()
            .max()
            .unwrap_or(1),
    );
    run.index_evidence = match adapter.index_evidence() {
        Ok(evidence) => evidence,
        Err(error) => Some(format!("could not read index evidence: {error}")),
    };
    run.memory_bytes = probe(&binding.memory_probe);
    run.disk_bytes = probe(&binding.disk_probe);
    Ok(())
}

pub fn empty_run(
    binding: &SystemBinding,
    inputs: &Inputs,
    environment: BTreeMap<String, String>,
) -> SystemRun {
    let now = now_unix_ms();
    SystemRun {
        schema_version: RUN_SCHEMA_VERSION,
        system: binding.id.clone(),
        identities: inputs.identities.clone(),
        binding_sha256: binding_sha256(binding),
        environment,
        status: SystemStatus::Failed,
        reason: None,
        observed_version: None,
        ingest: None,
        cases: Vec::new(),
        memory_bytes: None,
        disk_bytes: None,
        index_evidence: None,
        request_floor: None,
        started_at_unix_ms: now,
        finished_at_unix_ms: now,
    }
}

/// Never returns an error for a system's own failure: that failure is the result.
pub fn run_system(
    manifest: &VectorComparisonManifest,
    binding: &SystemBinding,
    adapter: &dyn SystemAdapter,
    inputs: &Inputs,
    environment: BTreeMap<String, String>,
) -> (SystemRun, Vec<RawCase>) {
    run_system_with_progress(manifest, binding, adapter, inputs, environment, &mut |_| {})
}

/// Progress callbacks run outside measured query cases and must not mutate the system.
pub fn run_system_with_progress(
    manifest: &VectorComparisonManifest,
    binding: &SystemBinding,
    adapter: &dyn SystemAdapter,
    inputs: &Inputs,
    environment: BTreeMap<String, String>,
    progress: &mut dyn FnMut(LabProgress),
) -> (SystemRun, Vec<RawCase>) {
    let mut run = empty_run(binding, inputs, environment);
    let mut raw = Vec::new();
    match run_phases(
        manifest, binding, adapter, inputs, &mut run, &mut raw, progress,
    ) {
        Ok(()) if run.cases.iter().all(|case| case.succeeded == 0) => {
            run.status = SystemStatus::Failed;
            run.reason = Some(
                run.cases
                    .iter()
                    .find_map(|case| case.first_error.clone())
                    .unwrap_or_else(|| "no search succeeded".into()),
            );
        }
        Ok(()) => {
            if let Some(reason) = partial_reason(&run.cases) {
                run.status = SystemStatus::Partial;
                run.reason = Some(reason);
            } else {
                run.status = SystemStatus::Completed;
            }
        }
        Err(error) => {
            run.status = SystemStatus::Failed;
            run.reason = Some(error);
        }
    }
    if matches!(run.status, SystemStatus::Completed | SystemStatus::Partial) {
        if let Err(error) = adapter.teardown() {
            run.status = SystemStatus::Partial;
            let prior = run
                .reason
                .take()
                .map(|reason| format!("{reason}; "))
                .unwrap_or_default();
            run.reason = Some(format!("{prior}teardown failed: {error}"));
        }
    }
    run.finished_at_unix_ms = now_unix_ms();
    (run, raw)
}

/// Summarize incomplete cases both on new runs and when combining retained
/// runs whose earlier implementation marked a partial run completed.
fn partial_reason(cases: &[CaseResult]) -> Option<String> {
    let incomplete = cases
        .iter()
        .filter(|case| case.errors > 0 || case.truncated)
        .count();
    if incomplete == 0 {
        return None;
    }
    let errors: u64 = cases.iter().map(|case| case.errors).sum();
    let truncated = cases.iter().filter(|case| case.truncated).count();
    let first = cases.iter().find_map(|case| case.first_error.as_deref());
    let detail = first
        .map(|error| {
            format!(
                "; first error: {}",
                error.chars().take(512).collect::<String>()
            )
        })
        .unwrap_or_default();
    Some(format!("{incomplete} incomplete case(s), {errors} failed request(s), {truncated} truncated case(s){detail}"))
}

fn metric(id: &str, label: &str, unit: &str, better: MetricDirection) -> Result<MetricDefinition> {
    Ok(MetricDefinition {
        id: Id::try_from(id.to_string())?,
        label: label.into(),
        unit: unit.into(),
        better,
    })
}

fn report_metrics() -> Result<Vec<MetricDefinition>> {
    use MetricDirection::{Higher, Lower};
    Ok(vec![
        metric("recall_at_k", "Recall at k", "fraction", Higher)?,
        metric("queries_per_second", "Queries per second", "1/s", Higher)?,
        metric(
            "qps_at_recall_target",
            "Queries per second at target recall",
            "1/s",
            Higher,
        )?,
        metric("latency_p50_ms", "Latency p50", "ms", Lower)?,
        metric("latency_p95_ms", "Latency p95", "ms", Lower)?,
        metric("latency_p99_ms", "Latency p99", "ms", Lower)?,
        metric("case_attempted", "Searches attempted", "count", Higher)?,
        metric("case_succeeded", "Searches succeeded", "count", Higher)?,
        metric("search_errors", "Search errors", "count", Lower)?,
        metric("case_truncated", "Case truncated", "boolean", Lower)?,
        metric(
            "ingest_points_per_second",
            "Ingest rate",
            "points/s",
            Higher,
        )?,
        metric(
            "ready_seconds",
            "Time until search-ready after ingest",
            "s",
            Lower,
        )?,
        metric("memory_bytes", "Server memory", "bytes", Lower)?,
        metric("disk_bytes", "Server disk", "bytes", Lower)?,
    ])
}

fn report_dimensions() -> Result<Vec<DimensionDefinition>> {
    [
        ("filter", "Filter"),
        ("setting", "Search setting"),
        ("concurrency", "Concurrent clients"),
    ]
    .into_iter()
    .map(|(id, label)| {
        Ok(DimensionDefinition {
            id: Id::try_from(id.to_string())?,
            label: label.into(),
        })
    })
    .collect()
}

fn fixed(entries: &[(&str, String)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), value.clone()))
        .collect()
}

fn view_id(text: &str) -> Result<Id> {
    Id::try_from(text.to_string())
}

fn report_views(manifest: &VectorComparisonManifest) -> Result<Vec<ViewDefinition>> {
    let concurrency = manifest.workload.concurrency[0].to_string();
    let mut views = Vec::new();
    let mut filters = vec!["none"];
    if manifest.workload.filtered {
        filters.push("category");
    }
    for filter in filters {
        let selection = fixed(&[
            ("filter", filter.into()),
            ("concurrency", concurrency.clone()),
        ]);
        views.push(ViewDefinition::Curve {
            id: view_id(&format!("recall-speed-{filter}"))?,
            title: format!("Recall against speed, filter: {filter}, {concurrency} client(s)"),
            x: view_id("recall_at_k")?,
            y: view_id("queries_per_second")?,
            fixed: selection.clone(),
        });
        views.push(ViewDefinition::Bars {
            id: view_id(&format!("matched-quality-{filter}"))?,
            title: format!(
                "Speed at recall {} or better, filter: {filter}",
                manifest.quality.recall_target
            ),
            metric: view_id("qps_at_recall_target")?,
            fixed: selection,
        });
    }
    for (id, title, metric) in [
        ("ingest", "Ingest rate", "ingest_points_per_second"),
        ("ready", "Time until search-ready", "ready_seconds"),
        ("memory", "Server memory", "memory_bytes"),
        ("disk", "Server disk", "disk_bytes"),
    ] {
        views.push(ViewDefinition::Bars {
            id: view_id(id)?,
            title: title.into(),
            metric: view_id(metric)?,
            fixed: BTreeMap::new(),
        });
    }
    views.push(ViewDefinition::Table {
        id: view_id("all-cases")?,
        title: "Every measured case".into(),
        metrics: [
            "recall_at_k",
            "queries_per_second",
            "latency_p50_ms",
            "latency_p95_ms",
            "latency_p99_ms",
            "case_attempted",
            "case_succeeded",
            "search_errors",
            "case_truncated",
        ]
        .into_iter()
        .map(view_id)
        .collect::<Result<_>>()?,
        rows: ["setting", "filter", "concurrency"]
            .into_iter()
            .map(view_id)
            .collect::<Result<_>>()?,
        fixed: BTreeMap::new(),
    });
    Ok(views)
}

fn system_kind(kind: SystemKind) -> &'static str {
    match kind {
        SystemKind::Rrflow => "rrflow",
        SystemKind::Qdrant => "qdrant",
        SystemKind::Surrealdb => "surrealdb",
        SystemKind::Milvus => "milvus",
        SystemKind::Custom => "custom",
    }
}

fn observe(
    list: &mut Vec<Observation>,
    system: &Id,
    id: &str,
    case: &BTreeMap<String, String>,
    value: f64,
    samples: u64,
) -> Result<()> {
    list.push(Observation {
        system: system.clone(),
        metric: Id::try_from(id.to_string())?,
        case: case.clone(),
        value,
        samples,
        low: None,
        high: None,
    });
    Ok(())
}

fn case_observations(run: &SystemRun, list: &mut Vec<Observation>) -> Result<()> {
    let system = &run.system;
    if let Some(ingest) = &run.ingest {
        let none = BTreeMap::new();
        observe(
            list,
            system,
            "ingest_points_per_second",
            &none,
            ingest.points_per_second,
            ingest.points,
        )?;
        observe(
            list,
            system,
            "ready_seconds",
            &none,
            ingest.ready_seconds,
            1,
        )?;
    }
    for (id, value) in [
        ("memory_bytes", run.memory_bytes),
        ("disk_bytes", run.disk_bytes),
    ] {
        if let Some(value) = value {
            observe(list, system, id, &BTreeMap::new(), value as f64, 1)?;
        }
    }
    for case in &run.cases {
        let labels = fixed(&[
            (
                "filter",
                if case.filtered { "category" } else { "none" }.to_string(),
            ),
            ("setting", case.setting.as_str().to_string()),
            ("concurrency", case.concurrency.to_string()),
        ]);
        observe(
            list,
            system,
            "case_attempted",
            &labels,
            case.attempted as f64,
            case.attempted,
        )?;
        observe(
            list,
            system,
            "case_succeeded",
            &labels,
            case.succeeded as f64,
            case.attempted,
        )?;
        observe(
            list,
            system,
            "search_errors",
            &labels,
            case.errors as f64,
            case.attempted,
        )?;
        observe(
            list,
            system,
            "case_truncated",
            &labels,
            if case.truncated { 1.0 } else { 0.0 },
            case.attempted,
        )?;
        // Incomplete cases keep counts and raw samples but must not enter
        // the normal recall/speed views as if they completed the workload.
        if case.succeeded == 0 || case.errors > 0 || case.truncated {
            continue;
        }
        list.push(Observation {
            system: system.clone(),
            metric: Id::try_from("recall_at_k".to_string())?,
            case: labels.clone(),
            value: case.recall_mean,
            samples: case.succeeded,
            low: Some(case.recall_low),
            high: Some(case.recall_high),
        });
        for (id, value) in [
            ("queries_per_second", case.queries_per_second),
            ("latency_p50_ms", case.latency_p50_ms),
            ("latency_p95_ms", case.latency_p95_ms),
            ("latency_p99_ms", case.latency_p99_ms),
        ] {
            observe(list, system, id, &labels, value, case.succeeded)?;
        }
    }
    Ok(())
}

/// The fastest setting that met the declared recall, for each filter and load.
/// A system that never met it has no observation, which a reader can see.
fn matched_quality(run: &SystemRun, target: f64, list: &mut Vec<Observation>) -> Result<()> {
    let mut best: BTreeMap<(bool, u32), &CaseResult> = BTreeMap::new();
    for case in &run.cases {
        if case.truncated || case.errors > 0 || case.succeeded == 0 || case.recall_mean < target {
            continue;
        }
        let key = (case.filtered, case.concurrency);
        if best
            .get(&key)
            .is_none_or(|current| case.queries_per_second > current.queries_per_second)
        {
            best.insert(key, case);
        }
    }
    for ((filtered, concurrency), case) in best {
        let labels = fixed(&[
            (
                "filter",
                if filtered { "category" } else { "none" }.to_string(),
            ),
            ("concurrency", concurrency.to_string()),
        ]);
        observe(
            list,
            &run.system,
            "qps_at_recall_target",
            &labels,
            case.queries_per_second,
            case.succeeded,
        )?;
    }
    Ok(())
}

fn recall_verdict(run: &SystemRun, filtered: bool, target: f64) -> Result<Verdict> {
    let scope = if filtered { "filtered" } else { "unfiltered" };
    let best = run
        .cases
        .iter()
        .filter(|case| {
            case.filtered == filtered && case.succeeded > 0 && case.errors == 0 && !case.truncated
        })
        .map(|case| case.recall_mean)
        .fold(None, |best: Option<f64>, value| {
            Some(best.map_or(value, |current| current.max(value)))
        });
    let outcome = match best {
        Some(value) if value >= target => VerdictOutcome::Met,
        Some(_) => VerdictOutcome::NotMet,
        None => VerdictOutcome::NotRun,
    };
    Ok(Verdict {
        id: Id::try_from(format!("{scope}-recall-{}", run.system.as_str()))?,
        question: format!("Did it reach {scope} recall of {target} at any tested setting?"),
        system: Some(run.system.clone()),
        outcome,
        basis: best.map_or("no complete error-free case".to_string(), |value| {
            format!("best error-free mean recall {value:.4}")
        }),
    })
}

fn report_limits(manifest: &VectorComparisonManifest) -> Vec<String> {
    let mut limits = vec![
        "Diagnostic evidence for optimization: not J-04 acceptance, and no parity or superiority claim.".to_string(),
        "Cases with errors or truncation cannot qualify for speed at the recall target. Server resource equivalence is not established by matching client metadata.".to_string(),
        "Closed-loop clients: each waits for its reply, so this understates latency under overload and is not an open-loop schedule.".to_string(),
        "Ingest is one sequential writer, which does not show any system's parallel write capacity.".to_string(),
        match &manifest.corpus.source {
            CorpusSource::GaussianMixture { .. } => {
                "The corpus is a synthetic Gaussian mixture, not embeddings of real content."
                    .to_string()
            }
            CorpusSource::Imported { origin, license, .. } => format!(
                "The corpus is {origin}, under {license}. Its filter categories are assigned at random, so a filter here follows no structure in the data."
            ),
        },
        "A configured index can go unused at small sizes: check each system's index evidence before reading its result as an index result.".to_string(),
        "Latency is measured in the harness and includes its own request encoding and reply parsing.".to_string(),
        "Memory compares an engine's resident set with a container's cgroup usage.".to_string(),
    ];
    for binding in &manifest.systems {
        limits.push(format!(
            "{}: {:?} index, storage {}, artifact {}, {:?} endpoint.",
            binding.id.as_str(),
            binding.index,
            binding.storage_profile,
            binding.artifact,
            binding.location
        ));
        for limit in &binding.limits {
            limits.push(format!("{}: {limit}", binding.id.as_str()));
        }
    }
    limits
}

pub fn combine(
    manifest: &VectorComparisonManifest,
    runs: &[SystemRun],
    artifacts: Vec<ArtifactReference>,
) -> Result<LabReport> {
    manifest.validate()?;
    let expected = digest(test_sha256(manifest)?)?;
    let mut identities: Option<&BTreeMap<String, Digest>> = None;
    for run in runs {
        require(
            run.schema_version == RUN_SCHEMA_VERSION,
            "unsupported run schema version",
        )?;
        require(
            run.identities.get("test") == Some(&expected),
            &format!("{} ran a different test", run.system.as_str()),
        )?;
        require(
            manifest
                .binding(run.system.as_str())
                .is_ok_and(|binding| binding_sha256(binding) == run.binding_sha256),
            &format!("{} ran under a different binding", run.system.as_str()),
        )?;
        if let Some(first) = identities {
            require(
                first == &run.identities,
                &format!("{} ran different inputs", run.system.as_str()),
            )?;
        }
        identities = Some(&run.identities);
    }
    let target = manifest.quality.recall_target;
    let mut systems = Vec::new();
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for binding in &manifest.systems {
        let run = runs.iter().find(|run| run.system == binding.id);
        let declared = match &binding.availability {
            Availability::Available => None,
            Availability::Unavailable { reason } => {
                Some((SystemStatus::Unavailable, reason.clone()))
            }
            Availability::Unimplemented { reason } => {
                Some((SystemStatus::Unimplemented, reason.clone()))
            }
            Availability::NotApplicable { reason } => {
                Some((SystemStatus::NotApplicable, reason.clone()))
            }
        };
        require(
            declared.is_none() || run.is_none(),
            &format!(
                "{} is declared as not running but has a run",
                binding.id.as_str()
            ),
        )?;
        let (status, reason) = match (&declared, run) {
            (Some((status, reason)), _) => (*status, Some(reason.clone())),
            (None, Some(run))
                if run.status == SystemStatus::Completed
                    && partial_reason(&run.cases).is_some() =>
            {
                (SystemStatus::Partial, partial_reason(&run.cases))
            }
            (None, Some(run)) => (run.status, run.reason.clone()),
            (None, None) => (
                SystemStatus::NotRun,
                Some("no run file was supplied".to_string()),
            ),
        };
        let mut configuration = BTreeMap::from([
            ("artifact".to_string(), binding.artifact.clone()),
            (
                "storage_profile".to_string(),
                binding.storage_profile.clone(),
            ),
            (
                "index".to_string(),
                serde_json::to_string(&binding.index).map_err(|error| error.to_string())?,
            ),
        ]);
        if let Some(evidence) = run.and_then(|run| run.index_evidence.clone()) {
            configuration.insert("index_evidence".to_string(), evidence);
        }
        if let Some(floor) = run.and_then(|run| run.request_floor.as_ref()) {
            configuration.insert(
                "request_floor".to_string(),
                format!(
                    "p50 {:.2} ms, p95 {:.2} ms at {} clients",
                    floor.p50_ms, floor.p95_ms, floor.clients
                ),
            );
        }
        systems.push(ReportSystem {
            id: binding.id.clone(),
            label: binding.label.clone(),
            kind: system_kind(binding.kind).to_string(),
            status,
            reason,
            observed_version: run.and_then(|run| run.observed_version.clone()),
            configuration,
        });
        if let Some(run) = run {
            case_observations(run, &mut observations)?;
            matched_quality(run, target, &mut observations)?;
            verdicts.push(recall_verdict(run, false, target)?);
            if manifest.workload.filtered {
                verdicts.push(recall_verdict(run, true, target)?);
            }
        }
    }
    let completed: Vec<_> = runs
        .iter()
        .filter(|run| run.status == SystemStatus::Completed)
        .collect();
    let same_environment = completed
        .windows(2)
        .all(|pair| pair[0].environment == pair[1].environment);
    verdicts.push(Verdict {
        id: Id::try_from("same-environment".to_string())?,
        question: "Did the benchmark client metadata match across runs?".into(),
        system: None,
        outcome: match (completed.len() < 2, same_environment) {
            (true, _) => VerdictOutcome::NotRun,
            (false, true) => VerdictOutcome::Met,
            (false, false) => VerdictOutcome::NotMet,
        },
        basis: format!(
            "{} completed run(s); client metadata only, not server resource equivalence",
            completed.len()
        ),
    });
    verdicts.push(Verdict {
        id: Id::try_from("server-resource-equivalence".to_string())?,
        question: "Were the server resource limits and deployment conditions equivalent?".into(),
        system: None,
        outcome: VerdictOutcome::NotRun,
        basis: "Server CPU, memory, storage and deployment equivalence has not been independently established by this report.".into(),
    });
    let environment = completed
        .first()
        .map(|run| run.environment.clone())
        .unwrap_or_default();
    let report = LabReport {
        schema_version: REPORT_SCHEMA_VERSION,
        test: manifest.test.clone(),
        evidence_class: LabEvidenceClass::Diagnostic,
        generated_by: format!("rrflow-lab {}", env!("CARGO_PKG_VERSION")),
        generated_at_unix_ms: now_unix_ms(),
        environment,
        identities: identities.cloned().unwrap_or_default(),
        systems,
        dimensions: report_dimensions()?,
        metrics: report_metrics()?,
        observations,
        verdicts,
        views: report_views(manifest)?,
        limits: report_limits(manifest),
        artifacts,
    };
    report.validate()?;
    Ok(report)
}
