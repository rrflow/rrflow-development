//! Validation for comparative evidence that still informs rrflowKV design.
//!
//! Reports for removed pre-release stores are not executable release evidence.
//! The SurrealDB diagnostic remains a bounded external comparison; it does not
//! claim general superiority.

#[test]
fn historical_storage_artifacts_remain_parseable_and_retain_failed_runs() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../eval/results");
    let mut files = std::fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("rrd-lsm") && name.ends_with(".json"))
        })
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files.len(), 35, "historical artifact inventory drifted");

    let mut passing_verdicts = 0usize;
    let mut failing_verdicts = 0usize;
    for file in files {
        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        assert!(
            (1..=4).contains(&evidence["format_version"].as_u64().unwrap()),
            "{} has an unknown historical schema",
            file.display()
        );
        for profile in ["fjall", "native"] {
            assert_eq!(
                evidence[profile]["correctness_verified"].as_bool(),
                Some(true),
                "{} has failed {profile} correctness",
                file.display()
            );
        }
        if let Some(passes) = evidence["promotion"]["passes"].as_bool() {
            if passes {
                passing_verdicts += 1;
            } else {
                failing_verdicts += 1;
                assert!(
                    evidence["promotion"]["failures"]
                        .as_array()
                        .is_some_and(|failures| !failures.is_empty()),
                    "{} drops the reason for a retained failure",
                    file.display()
                );
            }
        }
    }
    assert!(
        passing_verdicts > 0,
        "no retained passing diagnostic exists"
    );
    assert!(failing_verdicts > 0, "retained failures must not be erased");
}

#[test]
fn surrealdb_claim_diagnostic_records_bounded_wins_and_the_disk_loss() {
    let file = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../eval/results/2026-08-23-rrflow-surrealdb-3.0.5-claim-diagnostic-v1.json"
    );
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    assert_eq!(evidence["schema"], "rrflow.surrealdb-claim-differential.v1");
    assert_eq!(evidence["config"]["trials"], 3);
    assert_eq!(evidence["config"]["operations"], 2_048);
    assert_eq!(
        evidence["surrealdb"]["version"],
        "3.0.5 for linux on x86_64"
    );
    assert_eq!(
        evidence["surrealdb"]["binary_sha256"],
        "a9a5e9e36e4f6fe922e1991a4fb0ea1ee4fe90819c5e3a8dce238a56666e8cec"
    );
    assert_eq!(evidence["native"]["correctness_verified"], true);
    assert_eq!(evidence["surreal"]["correctness_verified"], true);
    for ratio in [
        "rrflow_to_surreal_write_throughput",
        "rrflow_to_surreal_read_throughput",
        "rrflow_to_surreal_server_write_throughput",
        "rrflow_to_surreal_server_read_throughput",
    ] {
        assert!(evidence["ratios"][ratio].as_f64().unwrap() > 1.0, "{ratio}");
    }
    for ratio in [
        "rrflow_to_surreal_write_p95",
        "rrflow_to_surreal_read_p95",
        "rrflow_to_surreal_server_write_p95",
        "rrflow_to_surreal_server_read_p95",
        "rrflow_to_surreal_recovery",
        "rrflow_to_surreal_peak_rss",
    ] {
        assert!(evidence["ratios"][ratio].as_f64().unwrap() < 1.0, "{ratio}");
    }
    assert!(
        evidence["ratios"]["rrflow_to_surreal_reopened_allocated"]
            .as_f64()
            .unwrap()
            > 1.0
    );
    assert_eq!(
        evidence["bounded_verdict"]["all_measured_cells_favor_rrflow"],
        false
    );
    assert_eq!(
        evidence["bounded_verdict"]["general_database_superiority"],
        false
    );
}
