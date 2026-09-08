//! Validation for retained storage characterization artifacts.
//!
//! Parsing these diagnostic inputs is not current-engine or release evidence.

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
