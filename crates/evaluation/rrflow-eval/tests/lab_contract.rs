//! The lab must grade independently, keep failures visible, and refuse results that
//! cannot honestly sit side by side.
use rrflow_eval::lab::adapter::{CollectionSpec, PointBatch, SearchRequest, SystemAdapter};
use rrflow_eval::lab::corpus::{exact_answers, generate, GroundTruth, VectorSet};
use rrflow_eval::lab::custom::CustomProtocol;
use rrflow_eval::lab::driver::{combine, prepare_inputs, run_system, Inputs, SystemRun};
use rrflow_eval::lab::manifest::{
    Availability, CorpusSource, CorpusSpec, CredentialReferences, IndexSpec, LabMetric,
    QualitySpec, SearchSetting, SystemBinding, SystemKind, SystemLocation,
    VectorComparisonManifest, WorkloadSpec,
};
use rrflow_eval::lab::report::{LabReport, SystemStatus, TestIdentity, VerdictOutcome};
use rrflow_eval::lab::{canonical_json, Result};
use rrflow_eval::storage::{schema, Id};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

fn id(text: &str) -> Id {
    Id::try_from(text.to_string()).unwrap()
}

fn binding(name: &str) -> SystemBinding {
    SystemBinding {
        id: id(name),
        label: name.to_string(),
        kind: SystemKind::Custom,
        location: SystemLocation::Loopback,
        endpoint: "http://127.0.0.1:9".into(),
        artifact: "test-artifact".into(),
        storage_profile: "memory".into(),
        availability: Availability::Available,
        index: IndexSpec::Exact,
        search_ladder: vec![SearchSetting {
            id: id("default"),
            ef: None,
            exact_rerank: None,
        }],
        credentials: CredentialReferences::default(),
        memory_probe: None,
        disk_probe: None,
    }
}

fn manifest(systems: Vec<SystemBinding>) -> VectorComparisonManifest {
    VectorComparisonManifest {
        schema_version: 1,
        test: TestIdentity {
            id: id("unit"),
            title: "Unit comparison".into(),
            family: id("vector-search"),
            revision: 1,
        },
        corpus: CorpusSpec {
            source: CorpusSource::GaussianMixture {
                seed: 11,
                clusters: 4,
                spread_milli: 300,
            },
            points: 200,
            dimensions: 8,
            metric: LabMetric::Cosine,
            categories: 4,
        },
        workload: WorkloadSpec {
            queries: 16,
            k: 5,
            warmup_queries: 2,
            ingest_batch: 64,
            concurrency: vec![1, 2],
            filtered: true,
            request_timeout_ms: 1_000,
            readiness_timeout_ms: 1_000,
            max_case_seconds: 30,
        },
        quality: QualitySpec { recall_target: 0.9 },
        systems,
    }
}

/// Answers exactly, as a correct system would.
struct PerfectSystem {
    points: Mutex<Vec<(u64, Vec<f32>, u32)>>,
}

impl PerfectSystem {
    fn new() -> Self {
        Self {
            points: Mutex::new(Vec::new()),
        }
    }
}

impl SystemAdapter for PerfectSystem {
    fn observed_version(&self) -> Result<String> {
        Ok("perfect 1".into())
    }
    fn prepare(&self, _: &CollectionSpec<'_>) -> Result<()> {
        Ok(())
    }
    fn ingest(&self, batch: &PointBatch<'_>) -> Result<()> {
        let mut points = self.points.lock().unwrap();
        for index in 0..batch.ids.len() {
            points.push((
                batch.ids[index],
                batch.vector(index).to_vec(),
                batch.categories[index],
            ));
        }
        Ok(())
    }
    fn finish_ingest(&self, _: Instant) -> Result<()> {
        Ok(())
    }
    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        let points = self.points.lock().unwrap();
        let mut scored: Vec<(f32, u64)> = points
            .iter()
            .filter(|point| request.category.is_none_or(|wanted| point.2 == wanted))
            .map(|point| {
                let score = point.1.iter().zip(request.vector).map(|(a, b)| a * b).sum();
                (score, point.0)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
        Ok(scored
            .into_iter()
            .take(request.k as usize)
            .map(|entry| entry.1)
            .collect())
    }
    fn teardown(&self) -> Result<()> {
        Ok(())
    }
}

/// Answers confidently and wrongly. Its own opinion must not matter.
struct WrongSystem;

impl SystemAdapter for WrongSystem {
    fn observed_version(&self) -> Result<String> {
        Ok("wrong 1".into())
    }
    fn prepare(&self, _: &CollectionSpec<'_>) -> Result<()> {
        Ok(())
    }
    fn ingest(&self, _: &PointBatch<'_>) -> Result<()> {
        Ok(())
    }
    fn finish_ingest(&self, _: Instant) -> Result<()> {
        Ok(())
    }
    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        Ok((0..u64::from(request.k))
            .map(|index| 1_000_000 + index)
            .collect())
    }
    fn teardown(&self) -> Result<()> {
        Ok(())
    }
}

/// Fails at one chosen phase, or on some searches.
struct FailingSystem {
    fail_prepare: bool,
    fail_every_nth_search: usize,
    calls: AtomicUsize,
    inner: PerfectSystem,
}

impl FailingSystem {
    fn new(fail_prepare: bool, fail_every_nth_search: usize) -> Self {
        Self {
            fail_prepare,
            fail_every_nth_search,
            calls: AtomicUsize::new(0),
            inner: PerfectSystem::new(),
        }
    }
}

impl SystemAdapter for FailingSystem {
    fn observed_version(&self) -> Result<String> {
        Ok("failing 1".into())
    }
    fn prepare(&self, _: &CollectionSpec<'_>) -> Result<()> {
        if self.fail_prepare {
            Err("collection refused".into())
        } else {
            Ok(())
        }
    }
    fn ingest(&self, batch: &PointBatch<'_>) -> Result<()> {
        self.inner.ingest(batch)
    }
    fn finish_ingest(&self, _: Instant) -> Result<()> {
        Ok(())
    }
    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        let call = self.calls.fetch_add(1, Ordering::Relaxed);
        if self.fail_every_nth_search != 0 && call.is_multiple_of(self.fail_every_nth_search) {
            return Err("search refused".into());
        }
        self.inner.search(request)
    }
    fn teardown(&self) -> Result<()> {
        Ok(())
    }
}

fn run_with(
    manifest: &VectorComparisonManifest,
    system: &str,
    adapter: &dyn SystemAdapter,
    inputs: &Inputs,
) -> SystemRun {
    let binding = manifest.binding(system).unwrap();
    run_system(manifest, binding, adapter, inputs, BTreeMap::new()).0
}

fn verdict(report: &LabReport, verdict_id: &str) -> VerdictOutcome {
    report
        .verdicts
        .iter()
        .find(|entry| entry.id.as_str() == verdict_id)
        .unwrap_or_else(|| panic!("no verdict {verdict_id}"))
        .outcome
}

#[test]
fn corpus_is_pinned_so_every_platform_tests_the_same_data() {
    let manifest = manifest(vec![binding("a")]);
    let (points, queries) = generate(&manifest.corpus, manifest.workload.queries);
    assert_eq!(points.len(), 200);
    assert_eq!(queries.len(), 16);
    assert_ne!(points.vector(0), queries.vector(0));
    assert_eq!(
        points.digest(),
        "9b3c735797e4a02819baf858e9e801556a2d92690ed56583c213ace6a264587e"
    );
    assert_eq!(
        queries.digest(),
        "64000b77bf9b1c4c8bda0c88c0b527ae0c46e4f18f3f2c4c8dd2a54b041d44a6"
    );
    for index in 0..points.len() {
        let norm: f32 = points.vector(index).iter().map(|value| value * value).sum();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "cosine corpus must be normalized"
        );
    }
}

#[test]
fn exact_oracle_finds_the_nearest_points_on_a_hand_checked_set() {
    let points = VectorSet {
        dimensions: 2,
        values: vec![1.0, 0.0, 0.0, 1.0, 0.8, 0.6, -1.0, 0.0],
        categories: vec![0, 1, 1, 0],
    };
    let queries = VectorSet {
        dimensions: 2,
        values: vec![1.0, 0.0],
        categories: vec![1],
    };
    let plain = exact_answers(&points, &queries, LabMetric::Cosine, 2, false, 2).unwrap();
    assert_eq!(plain.ids, vec![vec![1, 3]]);
    let filtered = exact_answers(&points, &queries, LabMetric::Cosine, 2, true, 2).unwrap();
    assert_eq!(filtered.ids, vec![vec![3, 2]]);
    let euclidean = exact_answers(&points, &queries, LabMetric::Euclidean, 1, false, 1).unwrap();
    assert_eq!(euclidean.ids, vec![vec![1]]);
}

#[test]
fn recall_counts_found_answers_and_ignores_repeats() {
    let truth = GroundTruth {
        ids: vec![vec![1, 2, 3, 4], Vec::new()],
    };
    assert_eq!(truth.recall(0, &[1, 2, 3, 4]), 1.0);
    assert_eq!(truth.recall(0, &[1, 1, 1, 1]), 0.25);
    assert_eq!(truth.recall(0, &[9]), 0.0);
    assert_eq!(truth.recall(1, &[9]), 1.0);
}

#[test]
fn a_correct_system_scores_full_recall_and_meets_the_target() {
    let manifest = manifest(vec![binding("good")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let run = run_with(&manifest, "good", &PerfectSystem::new(), &inputs);
    assert_eq!(run.status, SystemStatus::Completed);
    // One setting, two filter modes, two concurrency levels.
    assert_eq!(run.cases.len(), 4);
    for case in &run.cases {
        assert_eq!(case.recall_mean, 1.0, "{case:?}");
        assert_eq!(case.errors, 0);
        assert_eq!(case.attempted, 16);
        assert!(!case.truncated);
    }
    let report = combine(&manifest, &[run], Vec::new()).unwrap();
    assert_eq!(
        verdict(&report, "unfiltered-recall-good"),
        VerdictOutcome::Met
    );
    assert_eq!(
        verdict(&report, "filtered-recall-good"),
        VerdictOutcome::Met
    );
    assert!(report
        .observations
        .iter()
        .any(|entry| entry.metric.as_str() == "qps_at_recall_target"));
}

#[test]
fn a_wrong_system_cannot_grade_itself_and_gets_no_matched_speed() {
    let manifest = manifest(vec![binding("wrong")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let run = run_with(&manifest, "wrong", &WrongSystem, &inputs);
    assert_eq!(
        run.status,
        SystemStatus::Completed,
        "answering is not failing"
    );
    assert!(run.cases.iter().all(|case| case.recall_mean == 0.0));
    let report = combine(&manifest, &[run], Vec::new()).unwrap();
    assert_eq!(
        verdict(&report, "unfiltered-recall-wrong"),
        VerdictOutcome::NotMet
    );
    assert!(!report
        .observations
        .iter()
        .any(|entry| entry.metric.as_str() == "qps_at_recall_target"));
}

#[test]
fn a_failing_system_stays_visible_with_its_reason() {
    let manifest = manifest(vec![binding("broken"), binding("good")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let broken = run_with(&manifest, "broken", &FailingSystem::new(true, 0), &inputs);
    assert_eq!(broken.status, SystemStatus::Failed);
    assert_eq!(broken.reason.as_deref(), Some("collection refused"));
    let good = run_with(&manifest, "good", &PerfectSystem::new(), &inputs);
    let report = combine(&manifest, &[broken, good], Vec::new()).unwrap();
    let system = report
        .systems
        .iter()
        .find(|s| s.id.as_str() == "broken")
        .unwrap();
    assert_eq!(system.status, SystemStatus::Failed);
    assert_eq!(system.reason.as_deref(), Some("collection refused"));
}

#[test]
fn search_errors_are_counted_and_block_matched_quality() {
    let manifest = manifest(vec![binding("flaky")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let run = run_with(&manifest, "flaky", &FailingSystem::new(false, 3), &inputs);
    assert!(run.cases.iter().any(|case| case.errors > 0));
    assert_eq!(
        run.cases
            .iter()
            .find(|case| case.errors > 0)
            .unwrap()
            .first_error
            .as_deref(),
        Some("search refused")
    );
    let report = combine(&manifest, &[run], Vec::new()).unwrap();
    assert!(!report
        .observations
        .iter()
        .any(|entry| entry.metric.as_str() == "qps_at_recall_target"));
}

#[test]
fn every_search_failing_is_a_failed_run_not_a_silent_zero() {
    let manifest = manifest(vec![binding("down")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let run = run_with(&manifest, "down", &FailingSystem::new(false, 1), &inputs);
    assert_eq!(run.status, SystemStatus::Failed);
    assert_eq!(run.reason.as_deref(), Some("search refused"));
    let case = &run.cases[0];
    assert_eq!(case.succeeded, 0);
    assert!(case.recall_mean.is_sign_positive() && case.recall_mean == 0.0);
    assert_eq!(case.recall_min, 0.0);
}

#[test]
fn a_system_with_no_run_is_reported_as_not_run() {
    let manifest = manifest(vec![binding("ran"), binding("absent")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let run = run_with(&manifest, "ran", &PerfectSystem::new(), &inputs);
    let report = combine(&manifest, &[run], Vec::new()).unwrap();
    let absent = report
        .systems
        .iter()
        .find(|s| s.id.as_str() == "absent")
        .unwrap();
    assert_eq!(absent.status, SystemStatus::NotRun);
}

#[test]
fn a_declared_gap_is_shown_as_unimplemented_and_cannot_have_results() {
    let mut gap = binding("gap");
    gap.availability = Availability::Unimplemented {
        reason: "no public operation builds this index".into(),
    };
    let manifest = manifest(vec![binding("ran"), gap]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let run = run_with(&manifest, "ran", &PerfectSystem::new(), &inputs);
    let report = combine(&manifest, std::slice::from_ref(&run), Vec::new()).unwrap();
    let shown = report
        .systems
        .iter()
        .find(|s| s.id.as_str() == "gap")
        .unwrap();
    assert_eq!(shown.status, SystemStatus::Unimplemented);
    assert_eq!(
        shown.reason.as_deref(),
        Some("no public operation builds this index")
    );
    let mut forged = run.clone();
    forged.system = id("gap");
    assert!(combine(&manifest, &[run, forged], Vec::new()).is_err());
}

#[test]
fn runs_with_different_inputs_or_manifests_cannot_be_combined() {
    let manifest_a = manifest(vec![binding("a"), binding("b")]);
    let inputs = prepare_inputs(&manifest_a).unwrap();
    let a = run_with(&manifest_a, "a", &PerfectSystem::new(), &inputs);
    let mut b = run_with(&manifest_a, "b", &PerfectSystem::new(), &inputs);
    b.identities.insert(
        "truth".into(),
        rrflow_eval::storage::Digest::try_from("0".repeat(64)).unwrap(),
    );
    let refused = combine(&manifest_a, &[a.clone(), b], Vec::new()).unwrap_err();
    assert!(refused.contains("different inputs"), "{refused}");
    let mut changed = manifest_a.clone();
    changed.workload.k = 4;
    let refused = combine(&changed, &[a], Vec::new()).unwrap_err();
    assert!(refused.contains("different manifest"), "{refused}");
}

#[test]
fn different_environments_are_reported_not_hidden() {
    let manifest = manifest(vec![binding("a"), binding("b")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let a = run_with(&manifest, "a", &PerfectSystem::new(), &inputs);
    let mut b = run_with(&manifest, "b", &PerfectSystem::new(), &inputs);
    let same = combine(&manifest, &[a.clone(), b.clone()], Vec::new()).unwrap();
    assert_eq!(verdict(&same, "same-environment"), VerdictOutcome::Met);
    b.environment
        .insert("cpu_model".into(), "another machine".into());
    let different = combine(&manifest, &[a, b], Vec::new()).unwrap();
    assert_eq!(
        verdict(&different, "same-environment"),
        VerdictOutcome::NotMet
    );
}

fn refused(change: impl FnOnce(&mut VectorComparisonManifest)) -> String {
    let mut manifest = manifest(vec![binding("a")]);
    change(&mut manifest);
    manifest.validate().unwrap_err()
}

#[test]
fn manifests_refuse_what_would_make_a_result_misleading_or_unsafe() {
    assert!(
        refused(|m| m.systems[0].endpoint = "http://user:secret@127.0.0.1:9".into())
            .contains("cannot carry credentials")
    );
    assert!(
        refused(|m| m.systems[0].endpoint = "http://10.0.0.5:9".into())
            .contains("declared loopback")
    );
    assert!(refused(|m| m.systems[0].location = SystemLocation::Lan).contains("declared lan"));
    assert!(
        refused(|m| m.systems[0].credentials.secret_env = Some("not a name".into()))
            .contains("environment variable")
    );
    assert!(refused(|m| {
        m.systems[0].kind = SystemKind::Rrflow;
        m.systems[0].index = IndexSpec::Hnsw {
            m: 16,
            ef_construction: 100,
        };
        m.systems[0].search_ladder[0].ef = Some(32);
    })
    .contains("no public operation"));
    assert!(refused(|m| {
        m.systems[0].index = IndexSpec::Hnsw {
            m: 16,
            ef_construction: 100,
        };
    })
    .contains("needs ef"));
    assert!(refused(|m| m.systems[0].search_ladder[0].ef = Some(1)).contains("at least k"));
    assert!(refused(|m| {
        m.systems[0].search_ladder[0].ef = Some(40);
        m.systems[0].search_ladder[0].exact_rerank = Some(60);
    })
    .contains("rerank"));
    assert!(refused(|m| {
        m.systems[0].search_ladder[0].ef = Some(40);
        m.systems[0].search_ladder[0].exact_rerank = Some(1);
    })
    .contains("rerank"));
    assert!(refused(|m| m.systems.push(binding("a"))).contains("duplicate"));
    assert!(refused(|m| m.quality.recall_target = 1.5).contains("recall target"));
    assert!(refused(|m| m.workload.k = 0).contains("k is outside"));
    assert!(refused(
        |m| m.systems[0].availability = Availability::Unavailable { reason: " ".into() }
    )
    .contains("say why"));
}

#[test]
fn manifests_reject_unknown_fields_so_a_secret_cannot_be_smuggled_in() {
    let mut value = serde_json::to_value(manifest(vec![binding("a")])).unwrap();
    value["systems"][0]["password"] = json!("hunter2");
    let error = serde_json::from_value::<VectorComparisonManifest>(value)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown field"), "{error}");
    assert!(!error.contains("hunter2"), "errors must not echo the value");
}

fn sample_report() -> Value {
    let manifest = manifest(vec![binding("good")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let run = run_with(&manifest, "good", &PerfectSystem::new(), &inputs);
    serde_json::to_value(combine(&manifest, &[run], Vec::new()).unwrap()).unwrap()
}

fn report_error(change: impl FnOnce(&mut Value)) -> String {
    let mut value = sample_report();
    change(&mut value);
    let report: LabReport = serde_json::from_value(value).unwrap();
    report.validate().unwrap_err()
}

#[test]
fn reports_are_self_describing_and_refuse_dangling_references() {
    let report: LabReport = serde_json::from_value(sample_report()).unwrap();
    report.validate().unwrap();
    assert!(report
        .limits
        .iter()
        .any(|limit| limit.contains("not J-04 acceptance")));
    assert!(
        report_error(|v| v["observations"][0]["metric"] = json!("invented"))
            .contains("undeclared metric")
    );
    assert!(
        report_error(|v| v["observations"][0]["system"] = json!("invented"))
            .contains("undeclared system")
    );
    assert!(report_error(|v| v["views"][0]["x"] = json!("invented")).contains("undeclared metric"));
    assert!(
        report_error(|v| v["systems"][0]["status"] = json!("failed")).contains("needs a reason")
    );
    assert!(report_error(|v| {
        v["systems"][0]["status"] = json!("not_run");
        v["systems"][0]["reason"] = json!("skipped");
    })
    .contains("did not run"));
    assert!(report_error(|v| v["observations"] = json!([])).contains("at least one observation"));
}

#[test]
fn reports_have_no_winner_field_to_claim_superiority_with() {
    let mut value = sample_report();
    value["winner"] = json!("good");
    assert!(serde_json::from_value::<LabReport>(value).is_err());
}

#[test]
fn generated_schemas_match_the_checked_in_projections() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../eval/schemas");
    for (name, generated) in [
        ("lab-manifest-v1.json", schema::<VectorComparisonManifest>()),
        ("lab-run-v1.json", schema::<SystemRun>()),
        ("lab-report-v1.json", schema::<LabReport>()),
        ("lab-custom-protocol-v1.json", schema::<CustomProtocol>()),
    ] {
        let expected = canonical_json(&generated).unwrap();
        let actual = std::fs::read(directory.join(name))
            .unwrap_or_else(|error| panic!("{name}: {error}; run rrflow-lab schemas"))
            .into_iter()
            .filter(|byte| *byte != b'\r')
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{name} is stale; run rrflow-lab schemas");
    }
}

#[test]
fn the_checked_in_manifests_are_valid_and_keep_every_comparator() {
    for name in ["vector-small.json", "vector-medium.json"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../eval/lab")
            .join(name);
        let manifest: VectorComparisonManifest =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        manifest.validate().unwrap();
        for kind in [
            SystemKind::Rrflow,
            SystemKind::Qdrant,
            SystemKind::Surrealdb,
            SystemKind::Milvus,
            SystemKind::Custom,
        ] {
            assert!(
                manifest.systems.iter().any(|system| system.kind == kind),
                "{kind:?} must stay in {name}"
            );
        }
    }
}

#[test]
fn index_evidence_travels_into_the_report() {
    let manifest = manifest(vec![binding("good")]);
    let inputs = prepare_inputs(&manifest).unwrap();
    let mut run = run_with(&manifest, "good", &PerfectSystem::new(), &inputs);
    assert_eq!(run.index_evidence, None);
    run.index_evidence = Some("0 of 200 points indexed".into());
    let report = combine(&manifest, &[run], Vec::new()).unwrap();
    assert_eq!(
        report.systems[0]
            .configuration
            .get("index_evidence")
            .map(String::as_str),
        Some("0 of 200 points indexed")
    );
}
