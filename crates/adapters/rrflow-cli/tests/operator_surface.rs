//! Operator surface behavior through the public engine boundary.
//!
//! The tests drive the compiled binary, so they exercise the path an operator
//! uses rather than a test-only entry point. That is what makes the recording
//! guarantee meaningful: a command that forgot to record itself would pass a
//! library-level test and fail here.

use rrd_store::{RrflowKvStore, StorageEngine};
use std::path::{Path, PathBuf};
use std::process::Command;

fn rrflow(db: &Path, args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_rrflow"))
        .arg("--db")
        .arg(db)
        .args(args)
        .output()
        .expect("run rrflow");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// A directory beside the compiled binary, avoiding tmpfs.
fn scratch(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_rrflow"));
    path.pop();
    path.push("cli-scratch");
    path.push(name);
    let _ = std::fs::remove_dir_all(&path);
    project_db(&path, name)
}

fn project_db(project: &Path, instance: &str) -> PathBuf {
    std::fs::create_dir_all(project).expect("create project directory");
    rrd_engine::InstanceManifest::ensure_dedicated_as(project, instance)
        .expect("initialize project instance");
    project.join(rrd_engine::STORE_DIR)
}

#[test]
fn a_claim_can_be_asserted_and_resolved() {
    let db = scratch("assert-resolve");
    let (ok, out, err) = rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "wp3",
            "--predicate",
            "status",
            "--object",
            "in_progress",
            "--valid-from",
            "1000",
        ],
    );
    assert!(ok, "assert failed: {err}");
    assert!(out.contains("sequence 1"), "unexpected output: {out}");

    let (ok, out, err) = rrflow(
        &db,
        &[
            "as-of",
            "--subject",
            "wp3",
            "--predicate",
            "status",
            "--at",
            "2000",
        ],
    );
    assert!(ok, "as-of failed: {err}");
    assert!(out.contains("in_progress"), "unexpected output: {out}");

    let (ok, out, err) = rrflow(&db, &["status", "--json"]);
    assert!(ok, "status failed: {err}");
    let status: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(status["storage_backend"], "rrflow_kv");
}

#[test]
fn storage_rejects_nonexistent_selector_migration_and_upgrade_actions() {
    let db = scratch("storage-command-boundary");
    for action in [
        "migrate",
        "status",
        "rollback",
        "format-upgrade",
        "format-rollback",
        "format-status",
    ] {
        let (ok, _, error) = rrflow(&db, &["storage", action]);
        assert!(!ok, "storage {action} unexpectedly parsed");
        assert!(
            error.contains(&format!("unrecognized subcommand '{action}'")),
            "unexpected rejection for storage {action}: {error}"
        );
    }
}

#[test]
fn logical_archive_cli_exports_inspects_and_restores_a_new_root() {
    let root = tempfile::tempdir().unwrap();
    let source = project_db(&root.path().join("archive-source"), "archive-source");
    let target = project_db(&root.path().join("archive-restored"), "archive-restored");
    let archive = root.path().join("source.rrd-archive");
    let archive_arg = archive.to_str().unwrap();
    let (ok, _, err) = rrflow(
        &source,
        &[
            "assert",
            "--subject",
            "archive:claim",
            "--predicate",
            "status",
            "--object",
            "preserved",
        ],
    );
    assert!(ok, "source assertion failed: {err}");

    let (ok, out, err) = rrflow(
        &source,
        &[
            "storage",
            "archive-export",
            "--archive",
            archive_arg,
            "--json",
        ],
    );
    assert!(ok, "archive export failed: {err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["claim_sequence"],
        1
    );
    let (ok, _, err) = rrflow(
        &source,
        &["storage", "archive-inspect", "--archive", archive_arg],
    );
    assert!(ok, "archive inspect failed: {err}");
    let (ok, out, err) = rrflow(
        &target,
        &[
            "storage",
            "archive-restore",
            "--archive",
            archive_arg,
            "--json",
        ],
    );
    assert!(ok, "archive restore failed: {err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["reopened"],
        true
    );
    let (ok, out, err) = rrflow(
        &target,
        &[
            "as-of",
            "--subject",
            "archive:claim",
            "--predicate",
            "status",
        ],
    );
    assert!(ok, "restored query failed: {err}");
    assert!(out.contains("preserved"));
}

#[test]
fn backup_catalogue_cli_creates_lists_and_restores() {
    let root = tempfile::tempdir().unwrap();
    let source = project_db(&root.path().join("backup-source"), "backup-source");
    let target = root.path().join("backup-restored");
    let catalogue = root.path().join("backups");
    let catalogue_arg = catalogue.to_str().unwrap();
    assert!(
        rrflow(
            &source,
            &[
                "assert",
                "--subject",
                "backup:claim",
                "--predicate",
                "status",
                "--object",
                "retained",
            ]
        )
        .0
    );
    let (ok, out, err) = rrflow(
        &source,
        &[
            "storage",
            "backup-create",
            "--catalogue",
            catalogue_arg,
            "--label",
            "alpha",
            "--json",
        ],
    );
    assert!(ok, "backup create failed: {err}");
    let entry: serde_json::Value = serde_json::from_str(&out).unwrap();
    let backup_id = entry["backup_id"].as_str().unwrap();

    let (ok, out, err) = rrflow(
        &source,
        &[
            "storage",
            "backup-list",
            "--catalogue",
            catalogue_arg,
            "--json",
        ],
    );
    assert!(ok, "backup list failed: {err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["backups"][0]["backup_id"],
        backup_id
    );
    let (ok, _, err) = rrflow(
        &target,
        &[
            "storage",
            "backup-restore",
            "--catalogue",
            catalogue_arg,
            "--backup-id",
            backup_id,
        ],
    );
    assert!(ok, "backup restore failed: {err}");
    assert_eq!(RrflowKvStore::open(&target).unwrap().sequence().unwrap(), 1);
}

#[test]
fn every_invocation_is_recorded_including_failures() {
    let db = scratch("recording");

    rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "wp3",
            "--predicate",
            "status",
            "--object",
            "v1",
        ],
    );
    rrflow(&db, &["as-of", "--subject", "wp3", "--predicate", "status"]);
    rrflow(&db, &["status"]);
    // Invalid: the separator byte is rejected by the identifier type.
    let (ok, _, _) = rrflow(&db, &["as-of", "--subject", "", "--predicate", "status"]);
    assert!(!ok, "an empty subject must fail");

    let (ok, out, err) = rrflow(&db, &["invocations"]);
    assert!(ok, "invocations failed: {err}");

    for expected in ["assert", "as-of", "status"] {
        assert!(
            out.contains(expected),
            "{expected} missing from the log:\n{out}"
        );
    }
    assert!(
        out.contains("error"),
        "the failed invocation was not recorded:\n{out}"
    );
    // Five commands ran before this one, so this run is the sixth.
    assert!(out.contains("manual"), "trigger not recorded:\n{out}");
}

#[test]
fn the_recorded_log_is_queryable_as_json_with_arguments_and_outcome() {
    let db = scratch("queryable");
    rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "wp3",
            "--predicate",
            "status",
            "--object",
            "v1",
            "--valid-from",
            "500",
        ],
    );

    let (ok, out, err) = rrflow(&db, &["invocations", "--json"]);
    assert!(ok, "invocations --json failed: {err}");
    let records: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    let first = &records[0];

    assert_eq!(first["command"], "assert");
    assert_eq!(first["trigger"], "manual");
    assert_eq!(first["outcome"], "ok");
    assert!(first["ordinal"].as_u64().unwrap() >= 1);
    assert!(first["at"].as_u64().unwrap() > 0);

    // Arguments are recorded so a run can be reproduced from its record.
    let arguments = first["arguments"].as_array().unwrap();
    let joined: Vec<String> = arguments
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect();
    assert!(
        joined.contains(&"subject=wp3".to_string()),
        "arguments not recorded: {joined:?}"
    );
    assert!(
        joined.contains(&"valid_from=500".to_string()),
        "arguments not recorded: {joined:?}"
    );
}

#[test]
fn invocation_ordinals_are_monotonic_across_processes() {
    let db = scratch("ordinals");
    for _ in 0..5 {
        rrflow(&db, &["status"]);
    }
    let (_, out, _) = rrflow(&db, &["invocations", "--json"]);
    let records: serde_json::Value = serde_json::from_str(&out).unwrap();
    let ordinals: Vec<u64> = records
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["ordinal"].as_u64().unwrap())
        .collect();
    assert_eq!(
        ordinals,
        vec![1, 2, 3, 4, 5],
        "ordinals restarted or collided across processes"
    );
}

#[test]
fn history_shows_supersession_newest_first() {
    let db = scratch("history");
    for (object, from) in [("blocked", "100"), ("in_progress", "200"), ("done", "300")] {
        rrflow(
            &db,
            &[
                "assert",
                "--subject",
                "wp3",
                "--predicate",
                "status",
                "--object",
                object,
                "--valid-from",
                from,
            ],
        );
    }
    let (ok, out, err) = rrflow(
        &db,
        &["history", "--subject", "wp3", "--predicate", "status"],
    );
    assert!(ok, "history failed: {err}");
    let done = out.find("done").expect("done missing");
    let blocked = out.find("blocked").expect("blocked missing");
    assert!(done < blocked, "history is not newest-first:\n{out}");
}

#[test]
fn a_read_through_the_surface_retains_the_pair_under_gc() {
    let db = scratch("gc-retention");
    rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "read",
            "--predicate",
            "status",
            "--object",
            "v1",
        ],
    );
    rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "unread",
            "--predicate",
            "status",
            "--object",
            "v1",
        ],
    );
    // Reading through the surface records an access, which must retain the pair.
    rrflow(
        &db,
        &["as-of", "--subject", "read", "--predicate", "status"],
    );

    let (ok, out, err) = rrflow(&db, &["gc", "--json"]);
    assert!(ok, "gc failed: {err}");
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();

    let candidates: Vec<String> = report["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["subject"].as_str().unwrap().to_string())
        .collect();
    let retained: Vec<String> = report["retained"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["subject"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(retained, vec!["read"]);
    assert_eq!(candidates, vec!["unread"]);
}

#[test]
fn status_reports_counters_that_track_the_commands_run() {
    let db = scratch("status");
    rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "a",
            "--predicate",
            "p",
            "--object",
            "1",
        ],
    );
    rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "b",
            "--predicate",
            "p",
            "--object",
            "2",
        ],
    );

    let (ok, out, err) = rrflow(&db, &["status", "--json"]);
    assert!(ok, "status failed: {err}");
    let status: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(status["claim_sequence"], 2);
    // Two asserts recorded before this status command.
    assert_eq!(status["invocations"], 2);
}

#[test]
fn an_absent_claim_is_reported_rather_than_treated_as_an_error() {
    let db = scratch("absent");
    let (ok, out, err) = rrflow(
        &db,
        &["as-of", "--subject", "nothing", "--predicate", "here"],
    );
    assert!(ok, "a missing claim must not be an error: {err}");
    assert!(
        out.contains("no claim in force"),
        "unexpected output: {out}"
    );
}

#[test]
fn context_reads_cli_claims_through_the_bound_engine_without_scope_wiring() {
    let db = scratch("context-bound-engine");
    let project = db.parent().unwrap().parent().unwrap();
    let (ok, _, err) = rrflow(
        &db,
        &[
            "assert",
            "--subject",
            "project-alpha",
            "--predicate",
            "status",
            "--object",
            "durable authentication recovery is operational",
        ],
    );
    assert!(ok, "claim assertion failed: {err}");

    let project = project.to_str().unwrap();
    let (ok, out, err) = rrflow(
        &db,
        &[
            "context",
            "--root",
            project,
            "--query",
            "authentication recovery",
            "--json",
        ],
    );
    assert!(ok, "context assembly failed: {err}");
    let packet: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(packet["scope"], "instance:context-bound-engine");
    assert!(packet["packet_sha256"].as_str().unwrap().len() == 64);
    assert!(packet["items"].as_array().unwrap().iter().any(|item| {
        item["values"]["object"]["value"] == "durable authentication recovery is operational"
            && item["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|evidence| evidence["kind"] == "text")
    }));
}

#[test]
fn identity_bind_resolve_and_readme_warp_share_the_persistent_engine() {
    let db = scratch("identity-warp");
    let project = db.parent().unwrap().parent().unwrap().to_str().unwrap();
    let provider_subject = "provider-subject-must-not-be-persisted";
    let (ok, out, err) = rrflow(
        &db,
        &[
            "identity",
            "bind",
            "--root",
            project,
            "--seat",
            "clyffy",
            "--provider",
            "provider-alpha",
            "--provider-identity",
            "alpha-account",
            "--provider-subject",
            provider_subject,
            "--representation",
            "alpha-represents-clyffy",
            "--json",
        ],
    );
    assert!(ok, "identity bind failed: {err}");
    assert!(!out.contains(provider_subject));
    let bound: serde_json::Value = serde_json::from_str(&out).unwrap();
    let uri = bound["seat_uri"].as_str().unwrap();
    assert_eq!(uri, "rrflow://identity-warp/data/rrflow-seat/clyffy");
    assert!(bound["receipt"]["last_runtime_cursor"].as_u64().unwrap() > 0);

    let (ok, out, err) = rrflow(
        &db,
        &[
            "identity", "resolve", "--root", project, "--seat", "clyffy", "--json",
        ],
    );
    assert!(ok, "identity resolve failed after reopen: {err}");
    let identity: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(identity["uri"], uri);
    assert_eq!(identity["seat_id"], "clyffy");
    assert_eq!(identity["representations"][0]["provider"], "provider-alpha");
    assert_eq!(
        identity["representations"][0]["subject_sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );

    let (ok, out, err) = rrflow(
        &db,
        &[
            "context",
            "--root",
            project,
            "--warp",
            uri,
            "--max-graph-depth",
            "1",
            "--json",
        ],
    );
    assert!(ok, "warp resolution failed after reopen: {err}");
    let resolved: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resolved["uri"], uri);
    let items = resolved["context"]["items"].as_array().unwrap();
    assert!(items
        .iter()
        .any(|item| item["identity"] == "record:rrflow-seat:clyffy"));
    assert!(items.iter().any(|item| {
        item["identity"] == "record:rrflow-provider-identity:alpha-account"
            && item["evidence"].as_array().unwrap().iter().any(|evidence| {
                evidence["kind"] == "graph"
                    && evidence["source"] == "relation:rrflow-represents:alpha-represents-clyffy"
            })
    }));

    let (ok, out, err) = rrflow(&db, &["invocations", "--json"]);
    assert!(ok, "invocation read failed: {err}");
    assert!(!out.contains(provider_subject));
    assert!(out.contains("provider_subject_sha256="));
}

#[test]
fn unsupported_reasoning_ledger_commands_are_not_an_operator_surface() {
    let db = scratch("unsupported-reasoning-ledger");
    let (ok, _, error) = rrflow(&db, &["reasoning", "show"]);
    assert!(!ok, "an unsupported router command must not parse");
    assert!(error.contains("unrecognized subcommand 'reasoning'"));
}
