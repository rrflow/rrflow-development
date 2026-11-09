use std::process::Command;

#[test]
fn existing_summary_preserves_paired_regression_and_token_accounting() {
    let temp = tempfile::tempdir().unwrap();
    for (arm, success) in [("baseline", true), ("runtime", false)] {
        let raw_output = if success { "answer" } else { "no match" };
        let trial = serde_json::json!({"provider":"codex","repository":"fixture","scenario":"stale","arm":arm,"success":success,"expected":"answer","attempts":1,"tool_calls":2,"input_tokens":10,"output_tokens":5,"latency_ms":100,"exit_code":0,"raw_output":raw_output,"stderr":""});
        std::fs::write(
            temp.path().join(format!("{arm}.json")),
            serde_json::to_vec(&trial).unwrap(),
        )
        .unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow-eval"))
        .arg("summarize")
        .arg(temp.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).starts_with(
        "trials=2 success=1/2 regressions=1 total_latency_ms=200 known_token_pairs=2/2"
    ));
}

#[test]
fn screening_summary_is_constructed_from_complete_pairs_and_detects_tampering() {
    let temp = tempfile::tempdir().unwrap();
    let trials = temp.path().join("trials");
    std::fs::create_dir(&trials).unwrap();
    for provider in ["a", "b"] {
        for repository in ["x", "y"] {
            for arm in ["baseline", "runtime"] {
                let trial = serde_json::json!({
                    "provider":provider, "repository":repository, "scenario":"s", "arm":arm,
                    "success":true, "expected":"answer", "attempts":1, "retries":0,
                    "tool_calls":null, "input_tokens":null, "output_tokens":null,
                    "latency_ms":100, "exit_code":0, "raw_output":"answer", "stderr":""
                });
                std::fs::write(
                    trials.join(format!("{provider}-{repository}-{arm}.json")),
                    serde_json::to_vec(&trial).unwrap(),
                )
                .unwrap();
            }
        }
    }
    let report = temp.path().join("report.json");
    let binary = env!("CARGO_BIN_EXE_rrflow-eval");
    let output = Command::new(binary)
        .arg("summarize")
        .arg(&trials)
        .arg(&report)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(Command::new(binary)
        .arg("verify")
        .arg(&report)
        .status()
        .unwrap()
        .success());
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    for change in [
        "count",
        "duplicates",
        "pair",
        "result",
        "regressions",
        "version",
    ] {
        let mut tampered = value.clone();
        match change {
            "count" => tampered["successes"] = serde_json::json!(7),
            "duplicates" => tampered["providers"][1] = serde_json::json!("a"),
            "pair" => tampered["trials_detail"][1]["repository"] = serde_json::json!("y"),
            "result" => tampered["trials_detail"][0]["success"] = serde_json::json!(false),
            "regressions" => tampered["paired_regressions"] = serde_json::json!(1),
            "version" => tampered["format_version"] = serde_json::json!(2),
            _ => unreachable!(),
        }
        std::fs::write(&report, serde_json::to_vec(&tampered).unwrap()).unwrap();
        assert!(
            !Command::new(binary)
                .arg("verify")
                .arg(&report)
                .output()
                .unwrap()
                .status
                .success(),
            "accepted {change}"
        );
    }
}

#[test]
fn provider_run_argument_rejection_does_not_invoke_a_provider() {
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow-eval"))
        .arg("run")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("run expects seven arguments"));
    for (arm, expected) in [("arbitrary", "x"), ("runtime", ""), ("runtime", "  ")] {
        let output = Command::new(env!("CARGO_BIN_EXE_rrflow-eval"))
            .args([
                "run",
                "codex",
                ".",
                "scenario",
                arm,
                expected,
                "absent-prompt",
                "absent-output",
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("screening requires"));
    }
}

#[test]
fn summarize_refuses_to_pair_different_repositories() {
    let temp = tempfile::tempdir().unwrap();
    for (arm, repository) in [("baseline", "first"), ("runtime", "second")] {
        let trial = serde_json::json!({"provider":"codex","repository":repository,"scenario":"s","arm":arm,"success":true,"expected":"answer","attempts":1,"tool_calls":null,"input_tokens":null,"output_tokens":null,"latency_ms":1,"exit_code":0,"raw_output":"answer","stderr":""});
        std::fs::write(
            temp.path().join(format!("{arm}.json")),
            serde_json::to_vec(&trial).unwrap(),
        )
        .unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow-eval"))
        .arg("summarize")
        .arg(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("incomplete"));
}

#[test]
fn fabricated_success_and_duplicate_provider_names_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let trial = serde_json::json!({
        "provider":"only", "scenario":"same", "arm":"runtime", "success":false,
        "input_tokens":null, "output_tokens":null, "tool_calls":0, "latency_ms":0
    });
    let report = serde_json::json!({
        "trials":8, "successes":8, "paired_regressions":0,
        "providers":["only","only"], "repositories":["only","only"],
        "trials_detail":vec![trial; 8]
    });
    let path = temp.path().join("contradictory.json");
    std::fs::write(&path, serde_json::to_vec(&report).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow-eval"))
        .args(["verify", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "contradictory evidence must not verify"
    );
}

#[test]
fn historical_summary_with_no_repository_per_trial_is_not_qualified() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../eval/results/2026-08-18-summary.json");
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow-eval"))
        .arg("verify")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "missing pair identities cannot be verified"
    );
}
