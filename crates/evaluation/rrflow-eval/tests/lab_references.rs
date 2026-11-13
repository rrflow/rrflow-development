//! Published results are claims someone else made. These tests hold the comparison to what makes
//! showing them honest: every row names its source, nothing is ranked unless every condition
//! matches, a gap cannot read as a result, and answer quality is not invented.
use rrflow_eval::lab::definitions::content::{lab_files, RrflowContent};
use rrflow_eval::lab::definitions::pack::{
    build_pack, generate_key, PackContent as _, PackRequest,
};
use rrflow_eval::storage::Id;
use rrflow_evaluation_core::report::{
    DimensionDefinition, LabEvidenceClass, LabReport, MetricDefinition, MetricDirection,
    Observation, ReportSystem, SystemStatus, TestIdentity,
};
use rrflow_lab_benchmarks::references::{
    compare, AnswerResult, Comparison, HardwareClass, Provenance, ReferenceFile, ReferenceOrigin,
    ReferenceSource, SeekResult, TestConditions, ANSWERS_NOT_MEASURED,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NOW: u64 = 1_793_750_400; // 2026-11-04
const DAY: u64 = 86_400;

fn id(text: &str) -> Id {
    Id::try_from(text.to_owned()).unwrap()
}

fn origin() -> ReferenceOrigin {
    ReferenceOrigin {
        url: "https://example.org/results".to_owned(),
        quote: "system-x 1000 qps at recall 0.95".to_owned(),
        measured_by: "Example maintainers".to_owned(),
        provenance: Provenance::Independent,
        published: Some("2026-10-01".to_owned()),
    }
}

fn seek() -> SeekResult {
    SeekResult {
        system: "system-x".to_owned(),
        queries_per_second: 1000.0,
        latency_p99_ms: Some(2.5),
        recall: 0.95,
        dataset: "named-data".to_owned(),
        dimensions: Some(100),
        k: Some(10),
        filtered: false,
        hardware: "a 16-core server".to_owned(),
        hardware_class: HardwareClass::MultiCoreServer,
        origin: origin(),
        limits: Vec::new(),
    }
}

fn answer() -> AnswerResult {
    AnswerResult {
        system: "memory-x".to_owned(),
        score: 90.5,
        score_name: "accuracy %".to_owned(),
        benchmark: "Some benchmark".to_owned(),
        reader_model: Some("reader-1".to_owned()),
        judge_model: None,
        disputed: true,
        origin: origin(),
        limits: vec!["Its answer key is disputed.".to_owned()],
    }
}

fn file(seek: Vec<SeekResult>, answers: Vec<AnswerResult>) -> ReferenceFile {
    ReferenceFile {
        schema_version: 1,
        source: ReferenceSource {
            id: id("example"),
            title: "Example benchmark".to_owned(),
            url: "https://example.org".to_owned(),
            checked_on: "2026-11-03".to_owned(),
        },
        seek,
        answers,
    }
}

fn conditions() -> TestConditions {
    TestConditions {
        test: id("a-test"),
        title: "A test".to_owned(),
        dataset: Some("named-data".to_owned()),
        k: 10,
        recall_target: 0.95,
        hardware_class: HardwareClass::MultiCoreServer,
    }
}

fn differs_for(result: SeekResult) -> Vec<String> {
    let comparison = compare(
        &[file(vec![result], Vec::new())],
        Some(&conditions()),
        None,
        NOW,
    )
    .unwrap();
    assert_eq!(comparison.published_seek.len(), 1);
    let row = &comparison.published_seek[0];
    assert_eq!(row.same_conditions, row.differs.is_empty());
    row.differs.clone()
}

#[test]
fn a_row_must_name_its_source_and_hold_sensible_numbers() {
    file(vec![seek()], vec![answer()]).validate().unwrap();
    let refused = |change: &dyn Fn(&mut ReferenceFile)| {
        let mut bad = file(vec![seek()], vec![answer()]);
        change(&mut bad);
        assert!(bad.validate().is_err(), "{bad:?}");
    };
    refused(&|f| f.seek[0].origin.url = "http://example.org".to_owned());
    refused(&|f| f.seek[0].origin.quote = " ".to_owned());
    refused(&|f| f.seek[0].origin.measured_by = String::new());
    refused(&|f| f.seek[0].origin.published = Some("2026-13-01".to_owned()));
    refused(&|f| f.seek[0].origin.published = Some("April 2025".to_owned()));
    refused(&|f| f.seek[0].queries_per_second = 0.0);
    refused(&|f| f.seek[0].queries_per_second = f64::NAN);
    refused(&|f| f.seek[0].recall = 1.5);
    refused(&|f| f.seek[0].recall = 95.0);
    refused(&|f| f.seek[0].latency_p99_ms = Some(-1.0));
    refused(&|f| f.seek[0].k = Some(0));
    refused(&|f| f.seek[0].dataset = String::new());
    refused(&|f| f.answers[0].origin.url = String::new());
    refused(&|f| f.answers[0].score = -1.0);
    refused(&|f| f.answers[0].score_name = String::new());
    refused(&|f| f.answers[0].benchmark = String::new());
    refused(&|f| f.source.checked_on = "yesterday".to_owned());
    refused(&|f| f.source.url = "ftp://example.org".to_owned());
    refused(&|f| f.schema_version = 99);
    refused(&|f| {
        f.seek.clear();
        f.answers.clear();
    });
}

#[test]
fn a_reference_file_refuses_a_field_it_does_not_know() {
    let mut value = serde_json::to_value(file(vec![seek()], Vec::new())).unwrap();
    value["seek"][0]["winner"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ReferenceFile>(value).is_err());
}

#[test]
fn only_a_row_matching_every_condition_is_set_beside_the_test() {
    assert_eq!(differs_for(seek()), Vec::<String>::new());

    let mut other = seek();
    other.dataset = "other-data".to_owned();
    let differs = differs_for(other);
    assert_eq!(differs.len(), 1);
    assert!(
        differs[0].contains("other-data") && differs[0].contains("named-data"),
        "{differs:?}"
    );

    let mut other = seek();
    other.k = Some(100);
    let differs = differs_for(other);
    assert_eq!(differs.len(), 1);
    assert!(differs[0].contains("100"), "{differs:?}");

    let mut other = seek();
    other.recall = 0.97;
    let differs = differs_for(other);
    assert_eq!(differs.len(), 1);
    assert!(differs[0].contains("0.97"), "{differs:?}");

    let mut near = seek();
    near.recall = 0.9506;
    assert!(
        differs_for(near).is_empty(),
        "within a point of recall is the same target"
    );

    let mut other = seek();
    other.hardware_class = HardwareClass::SingleCore;
    let differs = differs_for(other);
    assert_eq!(differs.len(), 1);
    assert!(differs[0].to_lowercase().contains("core"), "{differs:?}");

    let mut other = seek();
    other.filtered = true;
    let differs = differs_for(other);
    assert_eq!(differs.len(), 1);
    assert!(differs[0].contains("filter"), "{differs:?}");

    let mut all = seek();
    all.dataset = "other-data".to_owned();
    all.k = Some(100);
    all.recall = 0.5;
    all.hardware_class = HardwareClass::Gpu;
    assert_eq!(differs_for(all).len(), 4);
}

#[test]
fn what_a_source_does_not_state_counts_as_a_difference() {
    let mut other = seek();
    other.k = None;
    assert!(differs_for(other)[0].contains("not stated"));

    let mut other = seek();
    other.hardware_class = HardwareClass::NotStated;
    assert!(differs_for(other)[0].contains("not stated"));

    let unnamed = TestConditions {
        dataset: None,
        ..conditions()
    };
    let comparison = compare(&[file(vec![seek()], Vec::new())], Some(&unnamed), None, NOW).unwrap();
    assert!(
        !comparison.published_seek[0].same_conditions,
        "generated data matches nothing published"
    );
}

#[test]
fn same_conditions_come_first_and_nothing_is_called_a_winner() {
    let mut context = seek();
    context.dataset = "other-data".to_owned();
    context.queries_per_second = 9_999_999.0;
    let comparison = compare(
        &[file(vec![context, seek()], vec![answer()])],
        Some(&conditions()),
        None,
        NOW,
    )
    .unwrap();
    assert!(comparison.published_seek[0].same_conditions);
    assert!(!comparison.published_seek[1].same_conditions);

    let text = serde_json::to_string(&comparison).unwrap().to_lowercase();
    for word in ["winner", "\"rank", "best", "fastest", "faster than"] {
        assert!(!text.contains(word), "{word} must not appear: {text}");
    }
}

fn metric(name: &str) -> MetricDefinition {
    MetricDefinition {
        id: id(name),
        label: name.to_owned(),
        unit: "x".to_owned(),
        better: MetricDirection::Neither,
    }
}

fn observation(system: &str, name: &str, labels: &[(&str, &str)], value: f64) -> Observation {
    Observation {
        system: id(system),
        metric: id(name),
        case: labels
            .iter()
            .map(|(key, text)| ((*key).to_owned(), (*text).to_owned()))
            .collect(),
        value,
        samples: 100,
        low: None,
        high: None,
    }
}

fn report_system(name: &str, status: SystemStatus, reason: Option<&str>) -> ReportSystem {
    ReportSystem {
        id: id(name),
        label: format!("{name} label"),
        kind: "custom".to_owned(),
        status,
        reason: reason.map(str::to_owned),
        observed_version: None,
        configuration: BTreeMap::new(),
    }
}

fn report() -> LabReport {
    let plain = [("filter", "none"), ("concurrency", "1")];
    let eight = [("filter", "none"), ("concurrency", "8")];
    let tuned = [
        ("filter", "none"),
        ("concurrency", "1"),
        ("setting", "ef-32"),
    ];
    let other = [
        ("filter", "none"),
        ("concurrency", "1"),
        ("setting", "ef-16"),
    ];
    let report = LabReport {
        schema_version: 1,
        test: TestIdentity {
            id: id("a-test"),
            title: "A test".to_owned(),
            family: id("vector-search"),
            revision: 1,
        },
        evidence_class: LabEvidenceClass::Diagnostic,
        generated_by: "a test".to_owned(),
        generated_at_unix_ms: 1,
        environment: BTreeMap::from([("note.definitions_sequence".to_owned(), "4".to_owned())]),
        identities: BTreeMap::new(),
        systems: vec![
            report_system("fast", SystemStatus::Completed, None),
            report_system("missed", SystemStatus::Completed, None),
            report_system("broken", SystemStatus::Failed, Some("it did not start")),
            report_system(
                "off",
                SystemStatus::Unavailable,
                Some("turned off in this machine's lab settings"),
            ),
            report_system(
                "gap",
                SystemStatus::Unimplemented,
                Some("no public operation"),
            ),
        ],
        dimensions: ["filter", "concurrency", "setting"]
            .iter()
            .map(|name| DimensionDefinition {
                id: id(name),
                label: (*name).to_owned(),
            })
            .collect(),
        metrics: [
            "qps_at_recall_target",
            "queries_per_second",
            "latency_p99_ms",
            "recall_at_k",
        ]
        .iter()
        .map(|name| metric(name))
        .collect(),
        observations: vec![
            observation("fast", "qps_at_recall_target", &plain, 700.0),
            observation("fast", "qps_at_recall_target", &eight, 5000.0),
            observation("fast", "queries_per_second", &tuned, 700.0),
            observation("fast", "latency_p99_ms", &tuned, 1.9),
            observation("fast", "queries_per_second", &other, 900.0),
            observation("fast", "latency_p99_ms", &other, 1.1),
            // It answered quickly but never reached the target, so it has no speed here.
            observation("missed", "queries_per_second", &other, 99_999.0),
            observation("missed", "recall_at_k", &other, 0.40),
        ],
        verdicts: Vec::new(),
        views: Vec::new(),
        limits: Vec::new(),
        artifacts: Vec::new(),
    };
    report.validate().unwrap();
    report
}

fn compared_report() -> Comparison {
    compare(
        &[file(vec![seek()], vec![answer()])],
        Some(&conditions()),
        Some(&report()),
        NOW,
    )
    .unwrap()
}

#[test]
fn speed_comes_only_from_what_met_the_recall_target() {
    let comparison = compared_report();
    let speeds: Vec<_> = comparison
        .measured
        .iter()
        .map(|speed| {
            (
                speed.system.as_str(),
                speed.filtered,
                speed.concurrency,
                speed.queries_per_second,
            )
        })
        .collect();
    assert_eq!(
        speeds,
        vec![("fast", false, 1, 700.0), ("fast", false, 8, 5000.0)],
        "the 99,999 from a system that missed the target must not appear"
    );
}

#[test]
fn a_speed_carries_the_p99_of_the_setting_that_reached_it() {
    let comparison = compared_report();
    assert_eq!(
        comparison.measured[0].latency_p99_ms,
        Some(1.9),
        "not 1.1, from another setting"
    );
}

#[test]
fn a_system_with_no_speed_says_why_and_cannot_read_as_a_result() {
    let comparison = compared_report();
    let reason = |name: &str| {
        comparison
            .without_speed
            .iter()
            .find(|entry| entry.system.as_str() == name && !entry.filtered)
            .map(|entry| entry.reason.clone())
            .unwrap_or_else(|| panic!("{name} must be listed"))
    };
    assert!(reason("missed").contains("0.95"), "{}", reason("missed"));
    assert!(reason("broken").contains("did not start"));
    assert!(reason("off").contains("turned off"));
    assert!(reason("gap").contains("no public operation"));
    assert!(!comparison
        .without_speed
        .iter()
        .any(|entry| entry.system.as_str() == "fast" && !entry.filtered));
}

#[test]
fn answer_quality_is_not_invented() {
    let comparison = compared_report();
    assert!(!comparison.answers_measured);
    assert_eq!(comparison.answers_note, ANSWERS_NOT_MEASURED);
    assert_eq!(comparison.published_answers.len(), 1);
    assert_eq!(
        comparison.published_answers[0].result.limits,
        vec!["Its answer key is disputed.".to_owned()]
    );
}

#[test]
fn a_source_not_reread_lately_says_so() {
    let fresh = compare(&[file(vec![seek()], Vec::new())], None, None, NOW).unwrap();
    assert!(!fresh.published_seek[0].stale);
    let later = compare(
        &[file(vec![seek()], Vec::new())],
        None,
        None,
        NOW + 90 * DAY,
    )
    .unwrap();
    assert!(later.published_seek[0].stale);
}

#[test]
fn with_no_test_everything_is_listed_for_context() {
    let comparison = compare(&[file(vec![seek()], vec![answer()])], None, None, NOW).unwrap();
    assert!(comparison.conditions.is_none());
    assert!(comparison.measured.is_empty());
    assert!(!comparison.published_seek[0].same_conditions);
}

// The lab's real references.

fn real_lab() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../eval/lab")
}

fn real_references() -> Vec<ReferenceFile> {
    lab_files(&real_lab())
        .unwrap()
        .iter()
        .filter(|(path, _)| path.starts_with("references/"))
        .map(|(path, text)| {
            let file: ReferenceFile =
                serde_json::from_str(text).unwrap_or_else(|error| panic!("{path}: {error}"));
            file.validate()
                .unwrap_or_else(|error| panic!("{path}: {error}"));
            assert_eq!(
                path.as_str(),
                format!("references/{}.json", file.source.id.as_str()),
                "a source's file is named for it"
            );
            file
        })
        .collect()
}

#[test]
fn the_labs_references_are_valid_and_cover_both_measures() {
    let references = real_references();
    assert!(references.iter().any(|file| !file.seek.is_empty()));
    assert!(references.iter().any(|file| !file.answers.is_empty()));
    let kinds: Vec<Provenance> = references
        .iter()
        .flat_map(|file| {
            file.seek
                .iter()
                .map(|row| row.origin.provenance)
                .chain(file.answers.iter().map(|row| row.origin.provenance))
        })
        .collect();
    for kind in [
        Provenance::VendorOwn,
        Provenance::Independent,
        Provenance::PeerReviewed,
    ] {
        assert!(
            kinds.contains(&kind),
            "{kind:?} should appear so a reader can tell them apart"
        );
    }
}

#[test]
fn no_real_test_is_said_to_match_what_it_does_not() {
    // The lab's tests use generated data and COCO, and nothing published here uses either.
    let references = real_references();
    let coco = TestConditions {
        dataset: Some("coco-i2i-512-angular".to_owned()),
        ..conditions()
    };
    let comparison = compare(&references, Some(&coco), None, NOW).unwrap();
    assert!(comparison
        .published_seek
        .iter()
        .all(|row| !row.same_conditions));
    assert!(comparison
        .published_seek
        .iter()
        .all(|row| !row.differs.is_empty()));
}

#[test]
fn a_pack_carries_references_and_one_source_changing_is_a_small_patch() {
    let before = lab_files(&real_lab()).unwrap();
    assert!(before.keys().any(|path| path.starts_with("references/")));
    RrflowContent.check(&before).unwrap();

    let path = before
        .keys()
        .find(|path| path.starts_with("references/"))
        .unwrap()
        .clone();
    let mut after = before.clone();
    let mut value: serde_json::Value = serde_json::from_str(&after[&path]).unwrap();
    value["source"]["checked_on"] = serde_json::json!("2026-11-04");
    after.insert(path.clone(), serde_json::to_string_pretty(&value).unwrap());

    let key = generate_key().unwrap();
    let request = |files, base| PackRequest {
        product: "rrflow",
        content_schema: RrflowContent.schema(),
        sequence: 1,
        published_at_unix_s: NOW,
        expires_at_unix_s: NOW + DAY,
        files,
        signing_key: &key.private_hex,
        base,
    };
    let first = build_pack(&request(&before, None)).unwrap();
    let manifest = serde_json::from_str(&first.manifest).unwrap();
    let patch = build_pack(&request(&after, Some(&manifest))).unwrap();
    assert_eq!(patch.files.keys().collect::<Vec<_>>(), vec![&path]);
}

#[test]
fn a_pack_refuses_a_reference_that_is_not_valid() {
    let mut files = lab_files(&real_lab()).unwrap();
    let path = files
        .keys()
        .find(|path| path.starts_with("references/"))
        .unwrap()
        .clone();
    let mut value: serde_json::Value = serde_json::from_str(&files[&path]).unwrap();
    value["schema_version"] = serde_json::json!(99);
    files.insert(path.clone(), serde_json::to_string(&value).unwrap());
    let error = RrflowContent.check(&files).unwrap_err();
    assert!(error.contains(&path), "{error}");

    let mut files = lab_files(&real_lab()).unwrap();
    let text = files[&path].clone();
    files.remove(&path);
    files.insert("references/misnamed.json".to_owned(), text);
    let error = RrflowContent.check(&files).unwrap_err();
    assert!(error.contains("misnamed"), "{error}");
}

// The real binary.

fn run(home: &Path, folder: &Path, arguments: &[&str]) -> (bool, String, String) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rrflow-lab"))
        .args(arguments)
        .current_dir(folder)
        .env("RRFLOW_LAB_HOME", home)
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else {
                found.insert(path.clone(), std::fs::read(&path).unwrap());
            }
        }
    }
    found
}

fn published_home(root: &Path) -> PathBuf {
    let repository = root.join("repository");
    for (relative, text) in lab_files(&real_lab()).unwrap() {
        let target = repository.join("eval/lab").join(relative);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, text).unwrap();
    }
    let home = root.join("home");
    let (ok, _, error) = run(&home, &repository, &["definitions", "publish"]);
    assert!(ok, "{error}");
    home
}

#[test]
fn compare_with_no_report_lists_what_is_published_and_who_measured_it() {
    let root = tempfile::tempdir().unwrap();
    let home = published_home(root.path());
    let before = snapshot(&home);
    let (ok, out, error) = run(&home, root.path(), &["compare"]);
    assert!(ok, "{error}");
    assert!(out.contains("Seek time"), "{out}");
    assert!(out.contains("Answer quality"), "{out}");
    assert!(out.contains("has not measured answer quality"), "{out}");
    assert!(out.contains("the vendor's own"), "{out}");
    assert!(out.contains("independent"), "{out}");
    assert!(out.contains("Disputed"), "{out}");
    assert!(
        out.lines().count() < 110,
        "the default must stay readable: {}",
        out.lines().count()
    );
    assert!(
        !out.contains("0.9501901140684412"),
        "sixteen digits are noise: {out}"
    );
    assert!(!out.contains("no test was named"), "{out}");
    assert!(out.contains("--details"), "{out}");
    let (ok, full, error) = run(&home, root.path(), &["compare", "--details"]);
    assert!(ok, "{error}");
    assert!(full.contains("https://"), "{full}");
    assert!(full.contains("Note:"), "{full}");
    assert_eq!(snapshot(&home), before, "reading must change nothing");

    let (ok, out, error) = run(&home, root.path(), &["compare", "--json"]);
    assert!(ok, "{error}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["answers_measured"], false);
    assert!(value["published_seek"].as_array().unwrap().len() > 3);
}

#[test]
fn compare_reads_a_real_report_and_refuses_to_rank_what_differs() {
    let root = tempfile::tempdir().unwrap();
    let home = published_home(root.path());
    let mut made = report();
    made.test.id = id("vector-coco-i2i");
    let path = root.path().join("report.json");
    std::fs::write(&path, serde_json::to_vec(&made).unwrap()).unwrap();
    let (ok, out, error) = run(
        &home,
        root.path(),
        &["compare", "--report", path.to_str().unwrap()],
    );
    assert!(ok, "{error}");
    assert!(out.contains("fast"), "{out}");
    assert!(out.contains("700"), "{out}");
    assert!(out.contains("coco-i2i-512-angular"), "{out}");
    assert!(out.contains("did not start"), "{out}");
    assert!(out.contains("context only"), "{out}");
    assert!(out.contains("pack 4"), "the pack the run came from: {out}");
    let (ok, full, error) = run(
        &home,
        root.path(),
        &["compare", "--report", path.to_str().unwrap(), "--details"],
    );
    assert!(ok, "{error}");
    assert!(full.contains("Differs:"), "it says what differs: {full}");
    assert!(!out.to_lowercase().contains("winner"), "{out}");
    assert!(
        !out.contains("99,999"),
        "a system that missed the target has no speed: {out}"
    );
}

#[test]
fn compare_refuses_a_report_that_is_not_valid() {
    let root = tempfile::tempdir().unwrap();
    let home = published_home(root.path());
    let mut made = report();
    made.observations[0].system = id("nobody");
    let path = root.path().join("bad.json");
    std::fs::write(&path, serde_json::to_vec(&made).unwrap()).unwrap();
    let (ok, _, error) = run(
        &home,
        root.path(),
        &["compare", "--report", path.to_str().unwrap()],
    );
    assert!(!ok);
    assert!(error.contains("undeclared"), "{error}");
}
