//! The installed, served product engine builds, activates and searches with a
//! TurboQuant generation through the Rust client (H-04q2).
//!
//! This is its own test target because the offline qualification requires the
//! installed lifecycle harness to hold exactly its product lifecycle cases.

use rrd_client::{ClientConfig, RequestOptions, RrdClient};
use rrd_contract::{
    transaction_operation_sha256, BeginTransaction, CanonicalId, CloseSession, CommitTransaction,
    CreateSession, SessionLimits, TransactionMutation,
};
use rrd_engine::RrdEngine;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

#[path = "support/process.rs"]
mod process;

const OUTPUT_LIMIT: usize = 16 * 1024 * 1024;
#[cfg(unix)]
use std::time::Instant;

/// The staged primary executable with a sealed environment, as the installed
/// lifecycle harness stages it.
struct StandaloneDistribution {
    root: tempfile::TempDir,
    executable: PathBuf,
}

impl StandaloneDistribution {
    fn stage() -> Self {
        let source = std::env::var_os("RRFLOW_TEST_BINARY")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_rrflow")));
        let root = tempfile::Builder::new()
            .prefix("rrflow quantization ")
            .tempdir()
            .unwrap();
        let executable = root.path().join(if cfg!(windows) {
            "rrflow.exe"
        } else {
            "rrflow"
        });
        process::stage_executable(&source, &executable)
            .expect("stage the actual primary executable");
        for directory in ["working", "home", "temporary", "empty-path"] {
            std::fs::create_dir(root.path().join(directory)).unwrap();
        }
        Self { root, executable }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .current_dir(self.root.path().join("working"));
        for name in ["SystemRoot", "WINDIR"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        for name in ["HOME", "USERPROFILE"] {
            command.env(name, self.root.path().join("home"));
        }
        for name in ["TMP", "TEMP", "TMPDIR"] {
            command.env(name, self.root.path().join("temporary"));
        }
        command.env("PATH", self.root.path().join("empty-path"));
        command.env("LANG", "C");
        command
    }
}

thread_local! {
    static DISTRIBUTION: StandaloneDistribution = StandaloneDistribution::stage();
}

fn product_command() -> Command {
    DISTRIBUTION.with(StandaloneDistribution::command)
}

fn invoke(args: &[&str]) -> Output {
    let mut command = product_command();
    let mut process = process::Process::start(
        command.args(args),
        Duration::from_secs(120),
        OUTPUT_LIMIT,
        None,
    )
    .expect("run rrflow");
    process
        .finish()
        .unwrap_or_else(|error| panic!("bounded rrflow command {:?}: {error}", args.get(1)))
}

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn capture_output(output: &Output, captured: &mut Vec<u8>) {
    captured.extend_from_slice(&output.stdout);
    captured.extend_from_slice(&output.stderr);
}

#[cfg(unix)]
fn stop_gracefully(child: &process::Process) {
    let mut command = Command::new("kill");
    command.args(["-INT", &child.id().to_string()]);
    let mut signal = process::Process::start(&mut command, Duration::from_secs(5), 4096, None)
        .expect("send SIGINT to the exact rrflow child");
    assert!(signal
        .finish()
        .expect("bounded signal helper")
        .status
        .success());
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.running() {
        if Instant::now() >= deadline {
            child.terminate();
            panic!("rrflow serve did not complete graceful SIGINT shutdown");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A test-owned served engine, always reaped.
struct RunningEngine {
    child: process::Process,
    finished: bool,
}

impl RunningEngine {
    fn start(project: &Path, captured: &mut Vec<u8>) -> (Self, String) {
        let mut command = product_command();
        command
            .args(["--json", "serve", "--project"])
            .arg(project)
            .args(["--bind", "127.0.0.1:0"]);
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        let mut process = Self {
            child: process::Process::start(
                &mut command,
                Duration::from_secs(480),
                OUTPUT_LIMIT,
                Some(send),
            )
            .expect("start bounded installed rrflow service"),
            finished: false,
        };
        assert!(process.child.id() > 0);
        let announcement = receive
            .recv_timeout(Duration::from_secs(60))
            .expect("installed engine must announce readiness within 60 seconds")
            .expect("read installed engine announcement");
        if announcement.is_empty() {
            process.stop(captured);
            panic!("serve exited before its listening announcement");
        }
        let announcement: serde_json::Value = serde_json::from_slice(&announcement).unwrap();
        assert_eq!(announcement["status"], "listening");
        let address = announcement["url"]
            .as_str()
            .unwrap()
            .strip_prefix("http://")
            .unwrap()
            .to_owned();
        (process, address)
    }

    fn stop(&mut self, captured: &mut Vec<u8>) {
        if self.finished {
            return;
        }
        if self.child.running() {
            #[cfg(unix)]
            stop_gracefully(&self.child);
            #[cfg(not(unix))]
            self.child.terminate();
        }
        let output = self.child.finish().expect("capture bounded rrflow service");
        capture_output(&output, captured);
        self.finished = true;
    }
}

impl Drop for RunningEngine {
    fn drop(&mut self) {
        if !self.finished {
            self.child.terminate();
            let _ = self.child.finish();
        }
    }
}

/// H-04q2: the installed, served engine builds a TurboQuant generation,
/// activates it, and serves an approximate search with it through the Rust
/// client.
#[test]
fn installed_engine_builds_activates_and_searches_turboquant() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("README.md"), "installed TurboQuant\n").unwrap();
    let project_arg = project.to_str().unwrap();
    let identity = invoke(&["install", "identity"]);
    assert_success(&identity, "allocate installation identity");
    let identity: serde_json::Value = serde_json::from_slice(&identity.stdout).unwrap();
    let plan = invoke(&[
        "install",
        "plan",
        "--identity",
        identity["installation_identity"].as_str().unwrap(),
        "--mode",
        "existing",
        "--project",
        project_arg,
    ]);
    assert_success(&plan, "install plan");
    let plan_file = root.path().join("install-plan.json");
    std::fs::write(&plan_file, &plan.stdout).unwrap();
    let preview: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    let applied = invoke(&[
        "install",
        "apply",
        "--mode",
        "existing",
        "--project",
        project_arg,
        "--plan",
        plan_file.to_str().unwrap(),
        "--expect",
        preview["installation"]["plan_sha256"].as_str().unwrap(),
    ]);
    assert_success(&applied, "install apply");
    let mut captured = Vec::new();
    let (mut engine, address) = RunningEngine::start(&project, &mut captured);
    let locator = RrdEngine::read_project_locator(&project).unwrap();
    let credential = RrdEngine::read_installed_api_key(&project).unwrap();
    let client = RrdClient::connect_local(
        address.parse().unwrap(),
        locator.identity.instance_id,
        ClientConfig::default(),
    )
    .unwrap();
    let scope = format!("instance:{}", client.instance_id());
    fn from_json<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
        serde_json::from_value(value).unwrap()
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let session = client
                .create_session(
                    credential.principal_id.clone(),
                    credential.credential(),
                    CreateSession {
                        limits: SessionLimits {
                            idle_timeout_ms: 60_000,
                            absolute_timeout_ms: 300_000,
                            max_open_transactions: 1,
                        },
                    },
                    RequestOptions::mutation(
                        "turboquant-session-request",
                        "turboquant-session-operation",
                        "turboquant-session-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            // Extend the installed schema with a document table and a vector table.
            let installed = client
                .read_data_schema(
                    &session,
                    from_json(serde_json::json!({"scope": scope, "max_storage_keys": 10_000})),
                    RequestOptions::read(
                        "turboquant-schema-request",
                        "turboquant-schema-operation",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let mut registry = serde_json::to_value(installed.schema.unwrap()).unwrap();
            registry["revision"] = serde_json::json!(registry["revision"].as_u64().unwrap() + 1);
            registry["migration"] = serde_json::json!("add installed TurboQuant documents");
            registry["tables"]["document"] =
                serde_json::json!({"model": "relational", "mode": "strict"});
            registry["tables"]["embedding"] =
                serde_json::json!({"model": "vector", "mode": "schemaless"});
            registry["records"]["document"] = serde_json::json!({
                "properties": {},
                "allow_additional_properties": true,
                "unique_properties": []
            });
            client
                .ensure_vector_collection(
                    &session,
                    from_json(serde_json::json!({
                        "scope": scope,
                        "collection_id": "documents",
                        "vectors": [{
                            "name": "title",
                            "field": "title-embedding",
                            "kind": "dense",
                            "dimensions": 2,
                            "metric": "cosine",
                            "memory_tier": "cached"
                        }]
                    })),
                    RequestOptions::mutation(
                        "turboquant-collection-request",
                        "turboquant-collection-operation",
                        "turboquant-collection-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let transaction = client
                .begin_transaction(
                    &session,
                    BeginTransaction {
                        scope: CanonicalId::new("data").unwrap(),
                        timeout_ms: 10_000,
                    },
                    RequestOptions::mutation(
                        "turboquant-begin-request",
                        "turboquant-begin-operation",
                        "turboquant-begin-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let mut mutations =
                vec![serde_json::json!({"mutation": "put_schema", "registry": registry})];
            for (id, values) in [
                ("alpha", [0.6, 0.8]),
                ("beta", [1.0, 0.0]),
                ("gamma", [0.0, 1.0]),
            ] {
                mutations.push(serde_json::json!({
                    "mutation": "put_record",
                    "reference": {"kind": "document", "id": id},
                    "valid_from": 100,
                    "properties": {}
                }));
                mutations.push(serde_json::json!({
                    "mutation": "put_vector",
                    "reference": {"kind": "embedding", "id": format!("{id}-title")},
                    "subject": {"kind": "document", "id": id},
                    "collection_id": "documents",
                    "vector_name": "title",
                    "field": "title-embedding",
                    "valid_from": 100,
                    "value": {"kind": "dense", "values": values},
                    "properties": {}
                }));
            }
            let mutations: Vec<TransactionMutation> =
                from_json(serde_json::Value::Array(mutations));
            client
                .commit_transaction(
                    &session,
                    &transaction.transaction_id,
                    CommitTransaction {
                        operation_sha256: transaction_operation_sha256(&mutations),
                        mutations,
                    },
                    RequestOptions::new(
                        "turboquant-commit-request",
                        "turboquant-commit-operation",
                        Some("turboquant-commit-idempotency"),
                        Some(u64::MAX),
                    )
                    .unwrap(),
                )
                .await
                .unwrap();

            let built = client
                .build_vector_quantization(
                    &session,
                    from_json(serde_json::json!({
                        "scope": scope,
                        "collection_id": "documents",
                        "vector_name": "title",
                        "method": {"method": "turbo_quant", "bits": "bits4", "seed": 7},
                        "max_storage_keys": 10_000
                    })),
                    RequestOptions::mutation(
                        "turboquant-build-request",
                        "turboquant-build-operation",
                        "turboquant-build-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert!(!built.idempotent_replay);
            client
                .activate_vector_quantization(
                    &session,
                    from_json(serde_json::json!({
                        "scope": scope,
                        "artifact_id": built.artifact.artifact_id,
                        "generation": built.artifact.generation
                    })),
                    RequestOptions::mutation(
                        "turboquant-activate-request",
                        "turboquant-activate-operation",
                        "turboquant-activate-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let searched = client
                .search_vectors(
                    &session,
                    from_json(serde_json::json!({
                        "scope": scope,
                        "valid_at": 200,
                        "collection_id": "documents",
                        "vector_name": "title",
                        "query": {"kind": "dense", "values": [0.6, 0.8]},
                        "top_k": 1,
                        "mode": {"mode": "require_approximate", "exact_rerank": 3, "ef_search": 3},
                        "max_storage_keys": 10_000
                    })),
                    RequestOptions::read(
                        "turboquant-search-request",
                        "turboquant-search-operation",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(searched.access_path.as_str(), "turboquant");
            assert!(!searched.exact);
            assert_eq!(searched.hits[0].reference.id.as_str(), "alpha-title");
            client
                .close_session(
                    &session,
                    CloseSession {},
                    RequestOptions::mutation(
                        "turboquant-close-request",
                        "turboquant-close-operation",
                        "turboquant-close-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
        });
    engine.stop(&mut captured);
    assert!(!String::from_utf8_lossy(&captured).contains(credential.credential()));
}
