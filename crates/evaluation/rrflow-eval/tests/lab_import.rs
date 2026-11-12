//! Another harness's results must stay labelled as imported, keep what the source said
//! about itself, and never carry its connection settings into a report.
use rrflow_eval::lab::import::import_vectordbbench;
use rrflow_lab_core::report::{LabEvidenceClass, LabReport, SystemStatus, VerdictOutcome};
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/vdbbench-qdrant-result.json");

fn imported(target: f64) -> LabReport {
    import_vectordbbench(FIXTURE.as_bytes(), "result.json", "2.0.0", target).unwrap()
}

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn values(report: &LabReport, metric: &str) -> Vec<f64> {
    report
        .observations
        .iter()
        .filter(|entry| entry.metric.as_str() == metric)
        .map(|entry| entry.value)
        .collect()
}

#[test]
fn imported_results_are_labelled_and_keep_the_sources_own_numbers() {
    let report = imported(0.95);
    assert_eq!(report.evidence_class, LabEvidenceClass::Imported);
    assert!(report.limits[0].contains("VectorDBBench 2.0.0"));
    assert_eq!(report.artifacts[0].path, "result.json");
    let system = &report.systems[0];
    assert_eq!(system.kind, "imported:vectordbbench");
    assert_eq!(system.status, SystemStatus::Completed);
    assert_eq!(system.observed_version.as_deref(), Some("1.14.1"));
    let first = &fixture()["results"][0]["metrics"];
    assert!(values(&report, "recall_at_k").contains(&first["recall"].as_f64().unwrap()));
    assert!(values(&report, "qps_at_recall_target").contains(&first["qps"].as_f64().unwrap()));
    // The source reports latency in seconds.
    let serial = first["serial_latency_p99"].as_f64().unwrap() * 1000.0;
    assert!(values(&report, "latency_p99_ms").contains(&serial));
}

#[test]
fn a_recall_below_target_earns_no_speed_at_target() {
    let mut source = fixture();
    source["results"][0]["metrics"]["recall"] = json!(0.5);
    source["results"].as_array_mut().unwrap().truncate(1);
    let low = import_vectordbbench(
        &serde_json::to_vec(&source).unwrap(),
        "result.json",
        "2.0.0",
        0.95,
    )
    .unwrap();
    assert!(values(&low, "qps_at_recall_target").is_empty());
    assert_eq!(low.verdicts[0].outcome, VerdictOutcome::NotMet);
}

#[test]
fn connection_settings_never_reach_a_report() {
    let mut source = fixture();
    let case = &mut source["results"][0]["task_config"];
    case["db_config"]["url"] = json!("https://private.example:6333");
    case["db_config"]["api_key"] = json!("do-not-copy");
    case["db_case_config"]["api_key"] = json!("do-not-copy");
    case["db_case_config"]["secret_token"] = json!("do-not-copy");
    case["db_config"]["note"] = json!("do-not-copy");
    let report = import_vectordbbench(
        &serde_json::to_vec(&source).unwrap(),
        "result.json",
        "2.0.0",
        0.95,
    )
    .unwrap();
    let text = serde_json::to_string(&report).unwrap();
    assert!(!text.contains("do-not-copy"), "{text}");
    assert!(!text.contains("private.example"), "{text}");
}

#[test]
fn a_failed_case_stays_visible_and_an_all_failed_system_is_failed() {
    let mut source = fixture();
    source["results"][1]["label"] = json!("x");
    let partly = import_vectordbbench(
        &serde_json::to_vec(&source).unwrap(),
        "result.json",
        "2.0.0",
        0.95,
    )
    .unwrap();
    assert_eq!(values(&partly, "failed_cases"), [1.0]);
    assert_eq!(partly.systems[0].status, SystemStatus::Partial);
    assert!(partly.systems[0].reason.as_deref().unwrap().contains("1"));
    assert!(!values(&partly, "recall_at_k").is_empty());

    for case in source["results"].as_array_mut().unwrap() {
        case["label"] = json!("x");
    }
    let failed = import_vectordbbench(
        &serde_json::to_vec(&source).unwrap(),
        "result.json",
        "2.0.0",
        0.95,
    )
    .unwrap();
    assert_eq!(failed.systems[0].status, SystemStatus::Failed);
    assert!(failed.observations.is_empty());
    assert_eq!(failed.verdicts[0].outcome, VerdictOutcome::NotRun);
}

#[test]
fn a_file_that_is_not_a_result_is_refused() {
    for (bytes, expected) in [
        (&b"not json"[..], "not a VectorDBBench result"),
        (br#"{"results":[]}"#, "not a VectorDBBench result"),
        (br#"{"run_id":"a","results":[]}"#, "holds no results"),
    ] {
        let refused = import_vectordbbench(bytes, "result.json", "2.0.0", 0.95).unwrap_err();
        assert!(refused.contains(expected), "{refused}");
    }
    assert!(import_vectordbbench(FIXTURE.as_bytes(), "result.json", " ", 0.95).is_err());
    assert!(import_vectordbbench(FIXTURE.as_bytes(), "result.json", "2.0.0", 0.0).is_err());
}
