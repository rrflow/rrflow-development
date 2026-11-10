//! Real-process proof for the installed, UI-discoverable walking product.
//!
//! This is one bounded child lifecycle, not a generic process-kill matrix: the
//! exact primary executable publishes its address, serves generated discovery,
//! accepts its generated credential, survives restart, and passes the read-only
//! verifier. The initial process is terminated after acknowledged writes; subsequent
//! Unix shutdowns use graceful SIGINT. Only test-owned children are affected.

use rrd_client::{ClientConfig, RequestOptions, RrdClient};
use rrd_contract::{
    transaction_operation_sha256, AssessmentInputKind, AssessmentOption,
    AssessmentPracticeIdentity, AssessmentScoreDirection, BackupCoverageSnapshot, BeginTransaction,
    CanonicalId, CloseSession, CommitTransaction, CreateInstanceBackup, CreateSession,
    EndpointAssessmentPolicy, EndpointGenerationSettings, EndpointResponseFormat,
    EndpointTokenParameter, ErrorCode, ExecuteQuery, QueryBudget, QueryResult, QueryValue,
    RestoreInstanceBackup, SessionLimits, TaskAssessmentDecision, TaskAssessmentDescriptor,
    TaskAssessmentRequest, TaskAssessmentValue, TransactionMutation, TASK_ASSESSMENT_VERSION,
};
use rrd_engine::RrdEngine;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LOOPBACK_CONFIGURATION_BYTES: &[u8] = include_bytes!(
    "../../../transport/rrd-contract/tests/fixtures/endpoint-selection-observed/available/configuration.json"
);
const LOOPBACK_ALLOWANCE_BYTES: &[u8] = include_bytes!(
    "../../../transport/rrd-contract/tests/fixtures/endpoint-selection-observed/available/allowance.json"
);

#[path = "support/adaptive_installed.rs"]
mod adaptive_installed;
#[path = "support/process.rs"]
mod process;
#[path = "support/storage_history.rs"]
mod storage_history;
// Authenticated readiness includes the generated discovery catalogue (>4 MiB).
const OUTPUT_LIMIT: usize = 16 * 1024 * 1024;
#[cfg(unix)]
use std::time::Instant;

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
            .prefix("rrflow standalone ")
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
        // Native OS discovery is the only inherited environment. No Cargo,
        // provider, checkout, credential, proxy or developer PATH survives.
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
    // The fixture outlives this test's child handles and is removed when its
    // test thread exits. No global working-directory/environment mutation.
    static DISTRIBUTION: StandaloneDistribution = StandaloneDistribution::stage();
}

fn binary() -> PathBuf {
    DISTRIBUTION.with(|distribution| distribution.executable.clone())
}

async fn inventory_through_client(
    client: &RrdClient,
    session: &rrd_client::Session,
    pending: &rrd_contract::AttunementJob,
    plan: &rrd_contract::AttunementPlan,
    prefix: &str,
) -> rrd_contract::AttunementJob {
    let options = |suffix: &str| {
        let key = format!("{prefix}-{suffix}");
        RequestOptions::mutation(&key, &key, &key).unwrap()
    };
    let mut job = client
        .start_attunement(
            session,
            rrd_contract::StartAttunement {
                contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                job_id: pending.id.clone(),
                plan_sha256: pending.plan_sha256.clone(),
                expected_revision: pending.revision,
                lease_ms: 60_000,
            },
            plan,
            options("start"),
        )
        .await
        .unwrap()
        .status
        .job;
    for phase in [
        rrd_contract::AttunementPhase::Connect,
        rrd_contract::AttunementPhase::Inventory,
    ] {
        assert_eq!(job.current_phase, Some(phase));
        job = client
            .step_attunement(
                session,
                rrd_contract::StepAttunement {
                    contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                    job_id: job.id.clone(),
                    expected_revision: job.revision,
                    lease_generation: job.lease.as_ref().unwrap().generation,
                },
                plan,
                options(&format!("{phase:?}")),
            )
            .await
            .unwrap()
            .status
            .job;
        assert_eq!(job.checkpoints.last().unwrap().phase, phase);
    }
    job
}

fn installed_binary_replans_and_reads_receipt_after_forced_restart(expect_parse: bool) {
    use serde_json::{json, Value};
    use std::io::{Read, Write};
    use std::net::TcpStream;

    fn post(address: &str, path: &str, body: &Value, headers: &[(&str, String)]) -> (u16, Value) {
        let bytes = serde_json::to_vec(body).unwrap();
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .unwrap();
        write!(stream, "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n", bytes.len()).unwrap();
        for (name, value) in headers {
            write!(stream, "{name}: {value}\r\n").unwrap();
        }
        stream.write_all(b"\r\n").unwrap();
        stream.write_all(&bytes).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        let split = response
            .windows(4)
            .position(|part| part == b"\r\n\r\n")
            .unwrap();
        let status = String::from_utf8_lossy(&response[..split])
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
        (
            status,
            serde_json::from_slice(&response[split + 4..]).unwrap(),
        )
    }
    fn payload(response: &Value) -> &Value {
        &response["outcome"]["payload"]
    }

    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("README.md"), "original source\n").unwrap();
    std::fs::write(
        project.join("sample.rs"),
        "pub fn project_answer() -> u32 { 42 }\n",
    )
    .unwrap();
    let proposal_file = root.path().join("source-attunement.json");
    let step = |phase| {
        json!({
            "phase": phase, "description": phase,
            "estimated_items": 0, "estimated_input_bytes": 0
        })
    };
    let mut parse = step("parse");
    parse["parse_policy"] = json!({
        "format_version": 1,
        "bindings": [{"extension": "rs", "language": "rust"}],
        "max_files": 64, "max_file_bytes": 1048576,
        "max_input_bytes": 4194304, "max_nodes": 10000,
        "max_depth": 64, "max_output_bytes": 1048576,
        "max_commit_bytes": 262144, "max_elapsed_ms": 10000,
        "max_memory_bytes": 268435456
    });
    std::fs::write(
        &proposal_file,
        serde_json::to_vec(&json!({
            "format_version": 1,
            "steps": [step("connect"), step("inventory"), parse, step("verify")],
            "estimated_min_duration_ms": 0, "estimated_max_duration_ms": 30000
        }))
        .unwrap(),
    )
    .unwrap();
    let identity = invoke(&["install", "identity"]);
    assert_success(&identity, "identity");
    let identity: Value = serde_json::from_slice(&identity.stdout).unwrap();
    let project_arg = project.to_str().unwrap();
    let plan = invoke(&[
        "install",
        "plan",
        "--identity",
        identity["installation_identity"].as_str().unwrap(),
        "--mode",
        "existing",
        "--project",
        project_arg,
        "--attunement-proposal",
        proposal_file.to_str().unwrap(),
    ]);
    assert_success(&plan, "install plan");
    let preview: Value = serde_json::from_slice(&plan.stdout).unwrap();
    let plan_file = root.path().join("plan.json");
    std::fs::write(&plan_file, &plan.stdout).unwrap();
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
    let instance = preview["installation"]["target"]["instance_id"]
        .as_str()
        .unwrap();
    let plan_sha = preview["attunement"]["plan_sha256"].as_str().unwrap();
    let job = format!("job-{}", &plan_sha[..24]);
    let credential = RrdEngine::read_installed_api_key(&project).unwrap();
    let make = |payload: Value, operation: &str, mutation: bool| {
        json!({
            "protocol":"rrd", "protocol_version":1,
            "context":{"request_id":format!("request-{operation}"),"operation_id":format!("operation-{operation}"),"idempotency_key":if mutation { Some(operation) } else { None },"deadline_unix_ms":null},
            "resource":{"segments":[{"kind":"instance","id":instance}]}, "payload":payload
        })
    };
    std::fs::write(project.join("README.md"), "edited source\n").unwrap();
    let mut captured = Vec::new();
    let mut acceptance = Value::Null;
    let mut successor = Value::Null;
    for pass in 0..2 {
        let (mut child, address) = RunningEngine::start(&project, &mut captured);
        let create = make(
            json!({"limits":{"idle_timeout_ms":60000,"absolute_timeout_ms":300000,"max_open_transactions":1}}),
            &format!("binary-session-{pass}"),
            true,
        );
        let (status, session) = post(
            &address,
            "/v1/sessions",
            &create,
            &[
                (
                    "X-RRD-Principal",
                    credential.principal_id.as_str().to_owned(),
                ),
                (
                    "Authorization",
                    format!("ApiKey {}", credential.credential()),
                ),
            ],
        );
        assert_eq!(status, 200, "session creation failed");
        let headers = [
            (
                "X-RRD-Session",
                payload(&session)["session_id"].as_str().unwrap().to_owned(),
            ),
            (
                "Authorization",
                format!("Bearer {}", payload(&session)["token"].as_str().unwrap()),
            ),
        ];
        if pass == 0 {
            let start = make(
                json!({"contract_version":rrd_contract::ATTUNEMENT_CONTRACT_VERSION,"job_id":job,"plan_sha256":plan_sha,"expected_revision":1,"lease_ms":60000}),
                "binary-stale-start",
                true,
            );
            let (status, stale) = post(&address, "/v1/attunement/start", &start, &headers);
            assert_eq!(status, 412, "{stale}");
            let proposal = make(
                json!({"contract_version":rrd_contract::ATTUNEMENT_CONTRACT_VERSION,"job_id":job,"expected_revision":1}),
                "binary-preview",
                false,
            );
            let (status, proposed) = post(
                &address,
                "/v1/attunement/replan/preview",
                &proposal,
                &headers,
            );
            assert_eq!(status, 200, "{proposed}");
            acceptance = make(
                json!({"contract_version":rrd_contract::ATTUNEMENT_CONTRACT_VERSION,"preview":payload(&proposed),"proposal_sha256":payload(&proposed)["proposal_sha256"]}),
                "binary-accept",
                true,
            );
            let (status, accepted) = post(
                &address,
                "/v1/attunement/replan/accept",
                &acceptance,
                &headers,
            );
            assert_eq!(status, 200, "{accepted}");
            assert_eq!(payload(&accepted)["idempotent_replay"], false);
            successor = payload(&accepted)["successor"]["job"].clone();
            child.terminate(&mut captured);
        } else {
            let (status, replay) = post(
                &address,
                "/v1/attunement/replan/accept",
                &acceptance,
                &headers,
            );
            assert_eq!(status, 200, "{replay}");
            assert_eq!(payload(&replay)["idempotent_replay"], true);
            assert_eq!(payload(&replay)["successor"]["job"], successor);
            let read = make(
                json!({"contract_version":rrd_contract::ATTUNEMENT_CONTRACT_VERSION,"job_id":job,"idempotency_key":"binary-accept"}),
                "binary-receipt",
                false,
            );
            let (status, receipt) = post(
                &address,
                "/v1/attunement/replan/receipt/read",
                &read,
                &headers,
            );
            assert_eq!(status, 200, "{receipt}");
            assert_eq!(
                payload(&receipt)["receipt"]["result"]["successor"]["job"],
                successor
            );
            child.stop(&mut captured);
        }
    }

    // Exercise the same durable successor through the generated Rust client,
    // with a second source edit and a forced primary-process restart.
    let old_plan: rrd_contract::AttunementPlan =
        serde_json::from_value(acceptance["payload"]["preview"]["plan"].clone()).unwrap();
    let old_job: rrd_contract::AttunementJob = serde_json::from_value(successor).unwrap();
    std::fs::write(project.join("README.md"), "typed client edit\n").unwrap();
    let mut typed_acceptance = None;
    let mut typed_successor = None;
    let mut typed_inventory = None;
    for pass in 0..2 {
        let (mut child, address) = RunningEngine::start(&project, &mut captured);
        let client = RrdClient::connect_local(
            address.parse().unwrap(),
            old_plan.target.instance_id.clone(),
            ClientConfig::default(),
        )
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
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
                        &format!("typed-session-{pass}"),
                        &format!("typed-session-{pass}"),
                        &format!("typed-session-{pass}"),
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            if pass == 0 {
                let preview = client
                    .preview_attunement_replan(
                        &session,
                        rrd_contract::PreviewAttunementReplan {
                            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                            job_id: old_job.id.clone(),
                            expected_revision: old_job.revision,
                        },
                        &old_plan,
                        RequestOptions::read("typed-preview-request", "typed-preview-operation")
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                let request = rrd_contract::AcceptAttunementReplan {
                    contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                    proposal_sha256: preview.proposal_sha256.clone(),
                    preview,
                };
                let result = client
                    .accept_attunement_replan(
                        &session,
                        request.clone(),
                        &old_plan,
                        RequestOptions::mutation(
                            "typed-accept-request",
                            "typed-accept-operation",
                            "typed-binary-accept",
                        )
                        .unwrap(),
                    )
                    .await
                    .unwrap();
                assert!(!result.idempotent_replay);
                typed_inventory = Some(
                    inventory_through_client(
                        &client,
                        &session,
                        &result.successor.job,
                        &request.preview.plan,
                        "binary-inventory",
                    )
                    .await,
                );
                typed_successor = Some(result.successor.job);
                typed_acceptance = Some(request);
            } else {
                let result = client
                    .accept_attunement_replan(
                        &session,
                        typed_acceptance.clone().unwrap(),
                        &old_plan,
                        RequestOptions::mutation(
                            "typed-accept-request",
                            "typed-accept-operation",
                            "typed-binary-accept",
                        )
                        .unwrap(),
                    )
                    .await
                    .unwrap();
                assert!(result.idempotent_replay);
                assert_eq!(Some(result.successor.job), typed_successor);
                let receipt = client
                    .read_attunement_replan_receipt(
                        &session,
                        rrd_contract::ReadAttunementReceipt {
                            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                            job_id: old_job.id.clone(),
                            idempotency_key: rrd_contract::CorrelationId::new(
                                "typed-binary-accept",
                            )
                            .unwrap(),
                        },
                        RequestOptions::read("typed-receipt-request", "typed-receipt-operation")
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(
                    Some(receipt.receipt.unwrap().result.successor.job),
                    typed_successor
                );
                let inventoried = typed_inventory.as_ref().unwrap();
                let current_plan = &typed_acceptance.as_ref().unwrap().preview.plan;
                let read = client
                    .read_attunement(
                        &session,
                        rrd_contract::ReadAttunement {
                            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                            job_id: inventoried.id.clone(),
                        },
                        RequestOptions::read("binary-inventory-read", "binary-inventory-read")
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(
                    read.status.job, *inventoried,
                    "acknowledged Inventory survives process death"
                );
                // Derive the fixture identities independently of the returned
                // rows. A job checkpoint alone does not prove that its source
                // records are reachable through the public query boundary.
                #[derive(serde::Serialize)]
                struct SourceScope<'a> {
                    estate: &'a str,
                    project: &'a str,
                    root: &'a str,
                }
                let scope = SourceScope {
                    estate: current_plan.target.estate_id.as_str(),
                    project: current_plan.target.project_id.as_str(),
                    root: "project-source",
                };
                let checkpoint = &inventoried.checkpoints[1];
                let result = client
                    .execute_query(
                        &session,
                        ExecuteQuery {
                            scope: format!("instance:{}", client.instance_id()),
                            query: format!(
                                "FROM traverse:source-tree-contains START source-tree-snapshot:\"{}\" DIRECTION OUTGOING DEPTH 2 AT VALID {} KNOWN {} PROJECT node_id, depth",
                                checkpoint.output_sha256,
                                checkpoint.committed_at_unix_ms,
                                checkpoint.coordinates.runtime_cursor,
                            ),
                            parameters: BTreeMap::new(),
                            budget: QueryBudget::default(),
                        },
                        RequestOptions::read("binary-source-readback", "binary-source-readback")
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(result.known_at_cursor, checkpoint.coordinates.runtime_cursor);
                for path in ["README.md", "sample.rs"] {
                    let bytes = serde_json::to_vec(&("rrflow-source-entry-v1", &scope, path))
                        .unwrap();
                    let identity = rrd_engine::digest::sha256_hex(&bytes);
                    assert!(
                        result.rows.iter().any(|row| {
                            row.values.get("node_id") == Some(&QueryValue::String(identity.clone()))
                        }),
                        "committed Inventory does not reach fixture source {path}"
                    );
                }
                eprintln!("installed inventory query readback verified after restart");
                std::fs::write(project.join("README.md"), "edit after durable Inventory\n")
                    .unwrap();
                let preview = client
                    .preview_attunement_replan(
                        &session,
                        rrd_contract::PreviewAttunementReplan {
                            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                            job_id: inventoried.id.clone(),
                            expected_revision: inventoried.revision,
                        },
                        current_plan,
                        RequestOptions::read("binary-inventory-replan", "binary-inventory-replan")
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                let successor_plan = preview.plan.clone();
                let result = client
                    .accept_attunement_replan(
                        &session,
                        rrd_contract::AcceptAttunementReplan {
                            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                            proposal_sha256: preview.proposal_sha256.clone(),
                            preview,
                        },
                        current_plan,
                        RequestOptions::mutation(
                            "binary-inventory-accept",
                            "binary-inventory-accept",
                            "binary-inventory-accept",
                        )
                        .unwrap(),
                    )
                    .await
                    .unwrap();
                let next = inventory_through_client(
                    &client,
                    &session,
                    &result.successor.job,
                    &successor_plan,
                    "binary-second-inventory",
                )
                .await;
                assert_ne!(
                    next.checkpoints[1].output_sha256,
                    inventoried.checkpoints[1].output_sha256
                );
                assert!(
                    next.checkpoints[1].coordinates.runtime_cursor
                        > inventoried.checkpoints[1].coordinates.runtime_cursor
                );
                assert_eq!(
                    next.current_phase,
                    Some(rrd_contract::AttunementPhase::Parse)
                );
                let parsed = client
                    .step_attunement(
                        &session,
                        rrd_contract::StepAttunement {
                            contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                            job_id: next.id.clone(),
                            expected_revision: next.revision,
                            lease_generation: next.lease.as_ref().unwrap().generation,
                        },
                        &successor_plan,
                        RequestOptions::mutation("binary-parse", "binary-parse", "binary-parse")
                            .unwrap(),
                    )
                    .await
                    .unwrap()
                    .status
                    .job;
                if expect_parse {
                    // The D-04 red acceptance oracle, recorded in POAM-006: an
                    // explicitly selected Parse step after an accepted
                    // Inventory replan must checkpoint. The offline harness
                    // requires every listed case to run and pass, so this
                    // branch is reached only when the Parse executor lands.
                    assert_eq!(
                        parsed.checkpoints.last().map(|checkpoint| checkpoint.phase),
                        Some(rrd_contract::AttunementPhase::Parse),
                        "installed source parsing did not checkpoint: {:?}",
                        parsed.failure
                    );
                } else {
                    // The installed Parse executor is not wired (POAM-006): the
                    // step must fail closed at Parse, keep the Inventory
                    // checkpoint and stay retryable. This records what the
                    // engine does today.
                    assert_eq!(
                        parsed.checkpoints.last().map(|checkpoint| checkpoint.phase),
                        Some(rrd_contract::AttunementPhase::Inventory)
                    );
                    let failure = parsed.failure.as_ref().expect("Parse fails closed today");
                    assert_eq!(failure.phase, rrd_contract::AttunementPhase::Parse);
                    assert_eq!(failure.code.as_str(), "phase-unavailable");
                    assert!(failure.retryable);
                }
            }
        });
        if pass == 0 {
            child.terminate(&mut captured);
        } else {
            child.stop(&mut captured);
        }
    }
}

fn product_command() -> Command {
    DISTRIBUTION.with(StandaloneDistribution::command)
}

async fn verify_parquet_export(
    client: &RrdClient,
    session: &rrd_client::Session,
    expected: &QueryResult,
) {
    let request = rrd_contract::ExportQuery {
        query: ExecuteQuery {
            scope: format!("instance:{}", client.instance_id()),
            query: expected.canonical_query.clone(),
            parameters: BTreeMap::new(),
            budget: QueryBudget {
                max_storage_keys: 75_000,
                max_rows: 768,
                max_output_bytes: 393_216,
                max_batch_rows: 192,
                max_memory_bytes: 33_554_432,
                max_spill_bytes: 67_108_864,
                max_elapsed_ms: 20_000,
            },
        },
        max_output_bytes: 393_216,
    };
    let export = client
        .export_query(
            session,
            request.clone(),
            RequestOptions::read("parquet-read", "parquet-export").unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export.source.canonical_query, expected.canonical_query);
    assert_eq!(
        export.source.read_manifest_sha256,
        expected.read_manifest_sha256
    );
    assert_eq!(export.source.known_at_cursor, expected.known_at_cursor);
    assert_eq!(export.source.schema_revision, expected.schema_revision);
    assert_eq!(
        export.source.plan.authorization_sha256,
        expected.plan.authorization_sha256
    );
    let reader = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(
        bytes::Bytes::from(export.decode_artifact().unwrap()),
    )
    .unwrap();
    let metadata = reader
        .metadata()
        .file_metadata()
        .key_value_metadata()
        .unwrap();
    let source = metadata
        .iter()
        .find(|entry| entry.key == "rrflow.export.source")
        .unwrap();
    assert_eq!(
        serde_json::from_str::<rrd_contract::QueryExportSource>(source.value.as_ref().unwrap())
            .unwrap(),
        export.source
    );
    let rows: Vec<_> = reader
        .build()
        .unwrap()
        .flat_map(|batch| rrd_query::record_batch_to_rows(&batch.unwrap()).unwrap())
        .collect();
    assert_eq!(rows.len(), expected.rows.len());
    for (row, expected) in rows.iter().zip(&expected.rows) {
        assert_eq!(row.identity, expected.identity);
        assert_eq!(
            row.values.get("object").unwrap(),
            &rrd_core::RuntimeValue::String(match &expected.values["object"] {
                QueryValue::String(value) => value.clone(),
                other => panic!("unexpected fixture object: {other:?}"),
            })
        );
    }
    let mut tiny = request.clone();
    tiny.max_output_bytes = 12;
    let denied = client
        .export_query(
            session,
            tiny,
            RequestOptions::read("parquet-small-read", "parquet-small-export").unwrap(),
        )
        .await
        .unwrap_err();
    assert!(matches!(denied, rrd_client::Error::Api { error, .. }
        if error.code == ErrorCode::ResourceExhausted));
    let mut foreign = request;
    foreign.query.scope = "instance:another-project".into();
    assert!(client
        .export_query(
            session,
            foreign,
            RequestOptions::read("parquet-foreign-read", "parquet-foreign-export").unwrap()
        )
        .await
        .is_err());
    let mut tampered = export;
    tampered.artifact_sha256 = "0".repeat(64);
    assert!(tampered.decode_artifact().is_err());
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
    let output = process
        .finish()
        .unwrap_or_else(|error| panic!("bounded rrflow command {:?}: {error}", args.get(1)));
    if args.contains(&"ready") {
        eprintln!(
            "native readiness output stdout_bytes={} stderr_bytes={}",
            output.stdout.len(),
            output.stderr.len()
        );
    }
    output
}

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn capture_output(output: &Output, captured: &mut Vec<u8>) {
    assert!(
        captured.len() + output.stdout.len() + output.stderr.len() <= 128 * 1024 * 1024,
        "lifecycle output exceeded its aggregate limit"
    );
    captured.extend_from_slice(&output.stdout);
    captured.extend_from_slice(&output.stderr);
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = std::fs::read_dir(current)
            .unwrap()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, output);
            } else {
                output.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
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

// Test-owned handles are always reaped, including assertion/startup failures.
// No PID search or process-name kill may affect another installed engine.
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
        let locator = RrdEngine::read_project_locator(project).unwrap();
        assert_eq!(
            announcement["instance_id"],
            locator.identity.instance_id.as_str()
        );
        static BUILD: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
        let build = BUILD.get_or_init(|| {
            let output = invoke(&["--json", "version"]);
            assert_success(&output, "build identity");
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["build"].clone()
        });
        assert_eq!(
            &announcement["build"], build,
            "serve/version identity differs"
        );
        let address = announcement["url"]
            .as_str()
            .unwrap()
            .strip_prefix("http://")
            .unwrap()
            .to_owned();
        (process, address)
    }

    fn terminate(&mut self, captured: &mut Vec<u8>) {
        self.child.terminate();
        let output = self.child.finish().expect("reap bounded rrflow process");
        capture_output(&output, captured);
        self.finished = true;
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
        #[cfg(unix)]
        assert!(
            output.status.success(),
            "rrflow serve failed during shutdown"
        );
        capture_output(&output, captured);
        self.finished = true;
    }
}

fn assert_restarted_status(
    project: &Path,
    valid_at: u64,
    expected: &str,
    captured: &mut Vec<u8>,
) -> QueryResult {
    let (mut process, address) = RunningEngine::start(project, captured);
    let locator = RrdEngine::read_project_locator(project).unwrap();
    let ready = invoke(&[
        "--json",
        "ready",
        "--project",
        project.to_str().unwrap(),
        "--address",
        &address,
    ]);
    capture_output(&ready, captured);
    assert_success(&ready, "restarted authenticated discovery");
    let ready: serde_json::Value = serde_json::from_slice(&ready.stdout).unwrap();
    assert_eq!(ready["status"], "ready");
    assert_eq!(
        ready["capabilities"]["instance"]["id"],
        locator.identity.instance_id.as_str()
    );
    assert_eq!(ready["authentication"]["session_closed"], true);
    let credential = RrdEngine::read_installed_api_key(project).unwrap();
    let client = RrdClient::connect_local(
        address.parse().unwrap(),
        locator.identity.instance_id,
        ClientConfig::default(),
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let result = runtime.block_on(async {
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
                    "restart-session-request",
                    "restart-session-operation",
                    "restart-session-idempotency",
                ).unwrap(),
            )
            .await
            .unwrap();
        let query = format!(
            "FROM claim:status AT VALID {valid_at} KNOWN HEAD WHERE subject = \"installed-walking-product\" PROJECT subject, object EXPLAIN CONTRACT"
        );
        let result = client
            .execute_query(
                &session,
                ExecuteQuery {
                    scope: format!("instance:{}", client.instance_id()),
                    query,
                    parameters: BTreeMap::new(),
                    budget: QueryBudget {
                        max_storage_keys: 75_000,
                        max_rows: 768,
                        max_output_bytes: 393_216,
                        max_batch_rows: 192,
                        max_memory_bytes: 33_554_432,
                        max_spill_bytes: 67_108_864,
                        max_elapsed_ms: 20_000,
                    },
                },
                RequestOptions::read("restart-query-request", "restart-query-operation").unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].identity, "claim:f2f88858872febdd0b233f7aa5c42f01b042d67faa6afd244d35e69feded14e7");
        assert_eq!(result.rows[0].values.get("object"), Some(&QueryValue::String(expected.into())));
        let context = client.assemble_context(&session, rrd_contract::AssembleContext {
            scope: format!("instance:{}", client.instance_id()), query: expected.into(), valid_at,
            seeds: Vec::new(), max_graph_depth: 0, max_items: 8, max_output_bytes: 32_768,
            max_storage_keys: 75_000,
        }, RequestOptions::read("restart-context-request", "restart-context-operation").unwrap()).await.unwrap();
        assert_eq!(context.result_identity_version, result.result_identity_version);
        assert!(context.items.iter().any(|item| item.identity == result.rows[0].identity));
        verify_parquet_export(&client, &session, &result).await;
        let cursor_request = |action| rrd_contract::InspectStorageCursor {
            scope: format!("instance:{}", client.instance_id()),
            cursor_id: rrd_contract::CorrelationId::new("restart-range-cursor").unwrap(),
            action,
        };
        let open_range = rrd_contract::StorageCursorAction::OpenRange {
            batch_rows: 1,
            batch_bytes: 4096,
            lifetime_ms: 30_000,
            start: vec![0],
            end: vec![255],
        };
        let page = client
            .inspect_storage_cursor(
                &session,
                cursor_request(open_range.clone()),
                RequestOptions::read("restart-range-open", "restart-range-open").unwrap(),
            )
            .await
            .unwrap();
        assert!(page.source_open && page.rows > 0);
        assert_eq!(
            client
                .inspect_storage_cursor(
                    &session,
                    cursor_request(open_range),
                    RequestOptions::read("restart-range-retry", "restart-range-retry").unwrap(),
                )
                .await
                .unwrap(),
            page
        );
        assert!(client
            .inspect_storage_cursor(
                &session,
                cursor_request(rrd_contract::StorageCursorAction::Release {}),
                RequestOptions::read("restart-range-release", "restart-range-release").unwrap(),
            )
            .await
            .unwrap()
            .released);
        client
            .close_session(
                &session,
                CloseSession {},
                RequestOptions::mutation(
                    "restart-close-request",
                    "restart-close-operation",
                    "restart-close-idempotency",
                ).unwrap(),
            )
            .await
            .unwrap();
        result
    });
    process.stop(captured);
    assert!(!String::from_utf8_lossy(captured).contains(credential.credential()));
    result
}

#[test]
fn repository_discovery_is_read_only_without_an_installed_engine() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("source.txt"), b"project-owned").unwrap();
    let before = inventory(project.path());
    let output = invoke(&[
        "--json",
        "install",
        "discover",
        "--project",
        project.path().to_str().unwrap(),
    ]);
    assert_success(&output, "repository discovery");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["git_index_verified"], false);
    assert_eq!(report["selection_available"], false);
    assert_eq!(
        report["coverage"],
        "present_tracked_gitlink_directories_only"
    );
    assert_eq!(report["candidates"], serde_json::json!([]));
    assert_eq!(inventory(project.path()), before);
    assert!(!project.path().join(".rrflow").exists());
}

#[test]
fn primary_help_exposes_only_the_installed_product_lifecycle() {
    let output = invoke(&["--help"]);
    assert_success(&output, "primary help");
    let help = String::from_utf8(output.stdout).unwrap();
    for command in [
        "version", "install", "serve", "ready", "verify", "restore", "repair",
    ] {
        assert!(
            help.contains(command),
            "missing {command} from help:\n{help}"
        );
    }
    for removed in [
        "assert",
        "as-of",
        "history",
        "status",
        "invocations",
        "query",
        "context",
        "identity",
        "storage",
        "dev",
    ] {
        assert!(
            !help.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(removed)
                    .is_some_and(|tail| tail.starts_with(char::is_whitespace))
            }),
            "removed command {removed} remains in help:\n{help}"
        );
    }
}

#[test]
fn raw_database_and_development_commands_fail_during_argument_parsing() {
    for removed in [
        "assert",
        "as-of",
        "history",
        "status",
        "invocations",
        "query",
        "context",
        "identity",
        "storage",
        "dev",
    ] {
        let output = invoke(&[removed]);
        assert!(!output.status.success(), "{removed} unexpectedly succeeded");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains(&format!("unrecognized subcommand '{removed}'")),
            "unexpected {removed} rejection: {error}"
        );
    }

    for removed_option in ["--db", "--reader"] {
        let output = invoke(&[removed_option, "unbound", "version"]);
        assert!(
            !output.status.success(),
            "{removed_option} unexpectedly succeeded"
        );
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("unexpected argument"),
            "unexpected {removed_option} rejection: {error}"
        );
    }
}

#[test]
fn installed_plan_accepts_selected_attunement_and_rejects_tampering() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("README.md"), "selected project\n").unwrap();
    let before = inventory(&project);
    let schema = invoke(&["install", "attunement-schema"]);
    assert_success(&schema, "attunement schema");
    let schema: serde_json::Value = serde_json::from_slice(&schema.stdout).unwrap();
    assert_eq!(
        schema["properties"]["format_version"]["const"],
        rrd_engine::ATTUNEMENT_PROPOSAL_FORMAT_VERSION
    );
    assert_eq!(schema["properties"]["steps"]["minItems"], 1);
    assert_eq!(
        schema["properties"]["steps"]["maxItems"],
        rrd_contract::MAX_ATTUNEMENT_PHASES
    );
    assert_eq!(
        schema["$defs"]["AttunementProposalStep"]["properties"]["description"]["maxLength"],
        1024
    );
    assert_eq!(
        inventory(&project),
        before,
        "schema discovery wrote project state"
    );
    let identity = invoke(&["install", "identity"]);
    assert_success(&identity, "identity");
    let identity: serde_json::Value = serde_json::from_slice(&identity.stdout).unwrap();
    let proposal_path = root.path().join("attunement.json");
    let step = |phase: &str, config: &str| {
        serde_json::json!({
            "phase": phase, "description": config,
            "estimated_items": 1, "estimated_input_bytes": 0
        })
    };
    let proposal = serde_json::json!({
        "format_version": 1, "steps": [step("connect", "first"), step("connect", "second"), step("verify", "verify")],
        "estimated_min_duration_ms": 0, "estimated_max_duration_ms": 1000
    });
    std::fs::write(&proposal_path, serde_json::to_vec(&proposal).unwrap()).unwrap();
    let plan = invoke(&[
        "install",
        "plan",
        "--identity",
        identity["installation_identity"].as_str().unwrap(),
        "--mode",
        "existing",
        "--project",
        project.to_str().unwrap(),
        "--attunement-proposal",
        proposal_path.to_str().unwrap(),
    ]);
    assert_success(&plan, "selected install plan");
    assert_eq!(inventory(&project), before, "preview wrote project state");
    let preview: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(preview["attunement_proposal"], proposal);
    let phases = preview["attunement"]["phases"].as_array().unwrap();
    assert_eq!(phases.len(), 3);
    assert_eq!(phases[0]["phase"], "connect");
    assert_eq!(phases[1]["phase"], "connect");
    assert_eq!(phases[1]["sequence"], 2);
    assert_eq!(phases[2]["phase"], "verify");
    let digest = preview["installation"]["plan_sha256"].as_str().unwrap();
    let mut second = proposal.clone();
    second["steps"] = serde_json::json!([step("connect", "other"), step("verify", "verify")]);
    std::fs::write(&proposal_path, serde_json::to_vec(&second).unwrap()).unwrap();
    let another = invoke(&[
        "install",
        "plan",
        "--identity",
        identity["installation_identity"].as_str().unwrap(),
        "--mode",
        "existing",
        "--project",
        project.to_str().unwrap(),
        "--attunement-proposal",
        proposal_path.to_str().unwrap(),
    ]);
    assert_success(&another, "second selected plan");
    let another: serde_json::Value = serde_json::from_slice(&another.stdout).unwrap();
    assert_ne!(another["installation"]["plan_sha256"], digest);
    assert_eq!(another["attunement"]["phases"].as_array().unwrap().len(), 2);
    let mut reordered = proposal.clone();
    reordered["steps"] = serde_json::json!([
        step("inventory", "inventory"),
        step("connect", "connect"),
        step("verify", "verify")
    ]);
    std::fs::write(&proposal_path, serde_json::to_vec(&reordered).unwrap()).unwrap();
    let reordered_preview = invoke(&[
        "install",
        "plan",
        "--identity",
        identity["installation_identity"].as_str().unwrap(),
        "--mode",
        "existing",
        "--project",
        project.to_str().unwrap(),
        "--attunement-proposal",
        proposal_path.to_str().unwrap(),
    ]);
    assert_success(&reordered_preview, "reordered selected plan");
    let reordered_preview: serde_json::Value =
        serde_json::from_slice(&reordered_preview.stdout).unwrap();
    assert_eq!(
        reordered_preview["attunement"]["phases"][0]["phase"],
        "inventory"
    );
    assert_eq!(
        reordered_preview["attunement"]["phases"][1]["phase"],
        "connect"
    );
    assert_ne!(reordered_preview["installation"]["plan_sha256"], digest);
    assert_eq!(
        inventory(&project),
        before,
        "alternative previews wrote project state"
    );
    for invalid in [
        serde_json::json!({"format_version": 1, "steps": [step("unknown-effect", "x"), step("verify", "v")], "estimated_min_duration_ms": 0, "estimated_max_duration_ms": 1}),
        serde_json::json!({"format_version": 1, "steps": [step("verify", "v"), step("connect", "x")], "estimated_min_duration_ms": 0, "estimated_max_duration_ms": 1}),
        serde_json::json!({"format_version": 99, "steps": [step("verify", "v")], "estimated_min_duration_ms": 0, "estimated_max_duration_ms": 1}),
        serde_json::json!({"format_version": 1, "steps": [step("connect", &"x".repeat(1025)), step("verify", "v")], "estimated_min_duration_ms": 0, "estimated_max_duration_ms": 1}),
        serde_json::json!({"format_version": 1, "steps": vec![step("verify", "v"); rrd_contract::MAX_ATTUNEMENT_PHASES + 1], "estimated_min_duration_ms": 0, "estimated_max_duration_ms": 1}),
    ] {
        std::fs::write(&proposal_path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        let rejected = invoke(&[
            "install",
            "plan",
            "--identity",
            identity["installation_identity"].as_str().unwrap(),
            "--mode",
            "existing",
            "--project",
            project.to_str().unwrap(),
            "--attunement-proposal",
            proposal_path.to_str().unwrap(),
        ]);
        assert!(!rejected.status.success(), "invalid proposal was accepted");
        assert_eq!(
            inventory(&project),
            before,
            "invalid preview wrote project state"
        );
    }
    let path = root.path().join("preview.json");
    let mut tampered = preview.clone();
    tampered["attunement_proposal"]["steps"][0]["description"] = "changed".into();
    std::fs::write(&path, serde_json::to_vec(&tampered).unwrap()).unwrap();
    let rejected = invoke(&[
        "install",
        "apply",
        "--mode",
        "existing",
        "--project",
        project.to_str().unwrap(),
        "--plan",
        path.to_str().unwrap(),
        "--expect",
        digest,
    ]);
    assert!(!rejected.status.success(), "changed proposal was accepted");
    assert_eq!(
        inventory(&project),
        before,
        "rejected proposal wrote project state"
    );
    for version in [None, Some(99)] {
        let mut invalid = preview.clone();
        if let Some(version) = version {
            invalid["preview_format_version"] = version.into();
        } else {
            invalid
                .as_object_mut()
                .unwrap()
                .remove("preview_format_version");
        }
        std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        let rejected = invoke(&[
            "install",
            "apply",
            "--mode",
            "existing",
            "--project",
            project.to_str().unwrap(),
            "--plan",
            path.to_str().unwrap(),
            "--expect",
            digest,
        ]);
        assert!(
            !rejected.status.success(),
            "unknown preview version was accepted"
        );
        assert_eq!(
            inventory(&project),
            before,
            "invalid preview wrote project state"
        );
    }
    // Apply consumes the sealed preview rather than rereading the changed input file.
    std::fs::write(&proposal_path, b"no longer a proposal").unwrap();
    std::fs::write(&path, &plan.stdout).unwrap();
    let applied = invoke(&[
        "install",
        "apply",
        "--mode",
        "existing",
        "--project",
        project.to_str().unwrap(),
        "--plan",
        path.to_str().unwrap(),
        "--expect",
        digest,
    ]);
    assert_success(&applied, "apply selected plan");
    let verified = invoke(&["verify", "--project", project.to_str().unwrap()]);
    assert_success(&verified, "verify installed selected plan");
}

#[test]
fn install_requires_an_explicit_plan_digest_and_verify_is_read_only_by_shape() {
    let apply = invoke(&["install", "apply", "--project", "."]);
    assert!(!apply.status.success());
    let error = String::from_utf8(apply.stderr).unwrap();
    assert!(error.contains("--plan <PLAN>"));
    assert!(error.contains("--expect <EXPECT>"));

    let verify = invoke(&["verify", "--help"]);
    assert_success(&verify, "verify help");
    let help = String::from_utf8(verify.stdout).unwrap();
    assert!(help.contains("--level <LEVEL>"));
    assert!(help.contains("quick"));
    assert!(!help.contains("repair"));
    assert!(!help.contains("restore"));
}

#[test]
fn installed_engine_plans_applies_serves_discovers_authenticates_and_reopens() {
    installed_lifecycle("existing");
    installed_binary_replans_and_reads_receipt_after_forced_restart(false);
}

#[test]
fn fresh_installed_engine_survives_restart_and_restore() {
    installed_lifecycle("fresh");
}

#[test]
fn installed_source_read_choice_is_explicit_and_reopens() {
    let root = tempfile::tempdir().unwrap();
    for (name, opt_in) in [("default", false), ("source", true)] {
        let project = root.path().join(name);
        std::fs::create_dir(&project).unwrap();
        std::fs::write(project.join("source.txt"), "bounded source fixture\n").unwrap();
        std::fs::create_dir(project.join(".git")).unwrap();
        std::fs::write(
            project.join(".git/config"),
            b"[core]\nrepositoryFormatVersion=0\nignoreCase=false\n",
        )
        .unwrap();
        let project_arg = project.to_str().unwrap();
        let identity_output = invoke(&["install", "identity"]);
        assert_success(&identity_output, "allocate installation identity");
        let identity: serde_json::Value = serde_json::from_slice(&identity_output.stdout).unwrap();
        let identity = identity["installation_identity"].as_str().unwrap();
        let mut arguments = vec![
            "install",
            "plan",
            "--identity",
            identity,
            "--mode",
            "existing",
            "--project",
            project_arg,
        ];
        if opt_in {
            arguments.push("--allow-project-source-read");
        }
        let preview_output = invoke(&arguments);
        assert_success(&preview_output, "plan installation");
        let preview: serde_json::Value = serde_json::from_slice(&preview_output.stdout).unwrap();
        let grant_names: Vec<&str> = preview["installation"]["initial_grants"]
            .as_array()
            .unwrap()
            .iter()
            .map(|grant| grant.as_str().unwrap())
            .collect();
        assert_eq!(
            grant_names.contains(&"project_source_read"),
            opt_in,
            "the explicit choice must be the only source-read grant switch"
        );
        if opt_in {
            assert_eq!(preview["source_read_resource_scope"], "instance_prefix");
        } else {
            assert!(preview.get("source_read_resource_scope").is_none());
        }
        let plan_path = root.path().join(format!("{name}-plan.json"));
        std::fs::write(&plan_path, &preview_output.stdout).unwrap();
        let plan_arg = plan_path.to_str().unwrap();
        let expected = preview["installation"]["plan_sha256"].as_str().unwrap();
        let apply = invoke(&[
            "--json",
            "install",
            "apply",
            "--mode",
            "existing",
            "--project",
            project_arg,
            "--plan",
            plan_arg,
            "--expect",
            expected,
        ]);
        assert_success(&apply, "apply exact installation plan");
        let applied: serde_json::Value = serde_json::from_slice(&apply.stdout).unwrap();
        let reopened = invoke(&[
            "--json",
            "verify",
            "--project",
            project_arg,
            "--level",
            "quick",
        ]);
        assert_success(&reopened, "reopen and verify installed grant binding");
        let report: serde_json::Value = serde_json::from_slice(&reopened.stdout).unwrap();
        assert_eq!(report["status"], "passed");
        exercise_installed_source_worker(
            &project,
            opt_in,
            applied["attunement_job_id"].as_str().unwrap(),
        );
    }
}

fn unix_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .unwrap()
}

fn loopback_worker_profile(
    listener: &TcpListener,
    allow_context_export: bool,
    allow_source_export: bool,
) -> rrd_contract::EndpointProfileInput {
    let mut configuration: rrd_contract::EndpointConfiguration =
        serde_json::from_slice(LOOPBACK_CONFIGURATION_BYTES).unwrap();
    let allowance: rrd_contract::EndpointSelectionAllowance =
        serde_json::from_slice(LOOPBACK_ALLOWANCE_BYTES).unwrap();
    let profile = configuration.profile.take().unwrap();
    let mut bindings = profile.bindings;
    let controller_service = bindings[0].service_id.clone();
    let controller_binding_id = bindings
        .iter()
        .find(|binding| binding.service_id == controller_service)
        .unwrap()
        .id
        .clone();
    let maximum_concurrent_proposals = bindings
        .iter()
        .find(|binding| binding.id == controller_binding_id)
        .unwrap()
        .limits
        .max_concurrency;
    let service_id = rrd_contract::CanonicalId::new("controlled-loopback-worker").unwrap();
    let binding_id = rrd_contract::CanonicalId::new("controlled-loopback-binding").unwrap();
    let manifest_sha256 = rrd_core::digest::sha256_hex(b"controlled loopback endpoint fixture");
    for binding in &mut bindings {
        binding.id = binding_id.clone();
        binding.endpoint = rrd_contract::EndpointLocator::Url {
            url: format!("http://{}/v1", listener.local_addr().unwrap()),
        };
        binding.credential = None;
        binding.manifest_sha256 = Some(manifest_sha256.clone());
        binding.service_id = service_id.clone();
        binding.limits.timeout_ms = 5_000;
    }
    let mut candidates = allowance.candidates;
    for candidate in &mut candidates {
        candidate.binding_id = binding_id.clone();
        candidate.manifest_sha256 = Some(manifest_sha256.clone());
        candidate.service_id = service_id.clone();
    }
    let selection_policy = rrd_contract::EndpointSelectionPolicy {
        contract_version: allowance.contract_version,
        purpose: rrd_contract::EndpointSelectionPurpose::ExperimentalProposal,
        allow_unverified_identity: true,
        allow_network: true,
        maximum_concurrent_proposals,
        controller_binding_id: binding_id,
        workload_class: allowance.workload_class,
        limits: allowance.limits,
        candidates,
        system_prompt: "Select the single explicitly allowed controlled loopback test binding."
            .into(),
        maximum_output_tokens: 64,
        temperature_milli: 0,
        lfg: None,
    };
    let worker_binding = selection_policy.candidates[0].binding_id.clone();
    rrd_contract::EndpointProfileInput {
        contract_version: profile.contract_version,
        name: rrd_contract::CanonicalId::new("controlled-installed-source-worker").unwrap(),
        bindings,
        selection_policy: Some(selection_policy),
        observation_policy: None,
        assessment_policy: None,
        worker_policy: Some(rrd_contract::EndpointWorkerPolicy {
            contract_version: rrd_contract::ENDPOINT_WORKER_VERSION,
            allow_network: true,
            allow_context_export,
            allow_source_export,
            allow_unverified_identity: true,
            maximum_concurrent_calls: 1,
            bindings: vec![rrd_contract::EndpointWorkerBinding {
                binding_id: worker_binding,
                protocol_evidence_sha256: rrd_core::digest::sha256_hex(
                    b"controlled loopback protocol fixture; no model or quality claim",
                ),
                adapter: rrd_contract::EndpointWorkerAdapter::default(),
                system_prompt: "Return a bounded draft. Do not mutate project files.".into(),
                generation: rrd_contract::EndpointGenerationSettings {
                    maximum_output_tokens: 128,
                    token_parameter: rrd_contract::EndpointTokenParameter::MaxTokens,
                    temperature_milli: None,
                    response_format: rrd_contract::EndpointResponseFormat::Text,
                },
                maximum_response_bytes: 16_384,
                effort_values: Default::default(),
                allow_default_effort: true,
            }],
        }),
    }
}

fn accept_loopback_chat(listener: &TcpListener) -> (TcpStream, serde_json::Value) {
    listener.set_nonblocking(true).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let (mut socket, _) = loop {
        match listener.accept() {
            Ok(socket) => break socket,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("bounded loopback provider accept failed: {error}"),
        }
    };
    socket.set_nonblocking(false).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        header.push(byte[0]);
        assert!(header.len() < 8_192);
    }
    let header = String::from_utf8(header).unwrap();
    assert!(header.starts_with("POST /v1/chat/completions HTTP/1.1"));
    let length: usize = header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().unwrap())
        })
        .unwrap();
    assert!(length < 524_288);
    let mut body = vec![0; length];
    socket.read_exact(&mut body).unwrap();
    (socket, serde_json::from_slice(&body).unwrap())
}

fn reply_loopback_chat(socket: &mut TcpStream, model: &str, content: &str) {
    let body = serde_json::to_vec(&serde_json::json!({
        "model": model,
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": content}}]
    }))
    .unwrap();
    write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
    socket.write_all(&body).unwrap();
}

fn exercise_installed_source_worker(project: &Path, source_read_enabled: bool, job_id: &str) {
    let mut lifecycle_output = Vec::new();
    let (mut service, address) = RunningEngine::start(project, &mut lifecycle_output);
    let locator = RrdEngine::read_project_locator(project).unwrap();
    let credential = RrdEngine::read_installed_api_key(project).unwrap();
    let client = RrdClient::connect_local(
        address.parse().unwrap(),
        locator.identity.instance_id.clone(),
        ClientConfig::default(),
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let session = runtime
        .block_on(
            client.create_session(
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
                    "source-choice-session-request",
                    "source-choice-session-operation",
                    "source-choice-session-idempotency",
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let read = runtime
        .block_on(
            client.read_attunement(
                &session,
                rrd_contract::ReadAttunement {
                    contract_version: rrd_contract::ATTUNEMENT_CONTRACT_VERSION,
                    job_id: rrd_contract::CanonicalId::new(job_id).unwrap(),
                },
                RequestOptions::read(
                    "source-choice-attunement-read",
                    "source-choice-attunement-read",
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let job = runtime.block_on(inventory_through_client(
        &client,
        &session,
        &read.status.job,
        &read.plan,
        "source-choice-inventory",
    ));
    let now = unix_ms().max(job.updated_at_unix_ms + 2);
    let scope = format!("instance:{}", client.instance_id());
    let source_query = runtime
        .block_on(client.execute_query(
            &session,
            ExecuteQuery {
                scope: scope.clone(),
                query: format!("FROM record:source-tree-entry AT VALID {now} KNOWN HEAD WHERE path = \"source.txt\" PROJECT identity, path, source_file"),
                parameters: BTreeMap::new(),
                budget: QueryBudget::default(),
            },
            RequestOptions::read("source-choice-inventory-query", "source-choice-inventory-query")
                .unwrap(),
        ))
        .unwrap();
    assert_eq!(source_query.rows.len(), 1);
    let values = &source_query.rows[0].values;
    let source_identity = match values.get("identity").unwrap() {
        QueryValue::String(value) | QueryValue::Digest(value) => value.clone(),
        value => panic!("unexpected source identity value {value:?}"),
    };
    let source_path = match values.get("path").unwrap() {
        QueryValue::String(value) => value.clone(),
        value => panic!("unexpected source path value {value:?}"),
    };
    let content_sha256 = match values.get("source_file").unwrap() {
        QueryValue::Map(values) => match values.get("content_sha256").unwrap() {
            QueryValue::String(value) | QueryValue::Digest(value) => value.clone(),
            value => panic!("unexpected source content digest {value:?}"),
        },
        value => panic!("unexpected source_file value {value:?}"),
    };
    let source_content = std::fs::read_to_string(project.join(&source_path)).unwrap();
    let task = runtime
        .block_on(
            client.open_task(
                &session,
                rrd_contract::OpenTask {
                    scope: scope.clone(),
                    title: "Inspect selected source".into(),
                    goal: "Return one bounded source-grounded draft.".into(),
                },
                RequestOptions::mutation(
                    "source-choice-open-task",
                    "source-choice-open-task",
                    "source-choice-open-task",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .task;
    let task_read = runtime
        .block_on(client.read_task(
            &session,
            rrd_contract::ReadTask {
                scope: scope.clone(),
                task_id: task.task_id.clone(),
                max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
            },
            RequestOptions::read("source-choice-read-task", "source-choice-read-task").unwrap(),
        ))
        .unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let worker = InstalledWorker {
        runtime: &runtime,
        client: &client,
        session: &session,
        listener: &listener,
    };
    let selection_configuration = install_loopback_profile(
        &runtime, &client, &session, &listener, true, true, "initial",
    );
    let selection_request = rrd_contract::EndpointSelectionRequest {
        scope: scope.clone(),
        task_id: task.task_id.clone(),
        attempt_id: rrd_contract::CorrelationId::new("source-choice-select-initial").unwrap(),
        max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
        expected_task_sha256: rrd_contract::EndpointSelectionTask::from_task(&task_read.task)
            .unwrap()
            .task_sha256,
        expected_configuration_sha256: selection_configuration.sha256().unwrap(),
        flight: None,
        routing: None,
    };

    if !source_read_enabled {
        let proposal = propose_over_loopback(
            &runtime,
            &client,
            &session,
            &listener,
            selection_request,
            "default-denial",
        );
        worker.assert_denied(
            WorkerAttempt {
                proposal: &proposal,
                scope: &scope,
                task_id: &task.task_id,
                source: Some((&source_identity, &content_sha256)),
                context: None,
                suffix: "default-denial",
            },
            rrd_contract::ErrorCode::PermissionDenied,
        );
        runtime
            .block_on(
                client.close_session(
                    &session,
                    rrd_contract::CloseSession {},
                    RequestOptions::mutation(
                        "source-choice-default-close-session",
                        "source-choice-default-close-session",
                        "source-choice-default-close-session",
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        service.stop(&mut lifecycle_output);
        return;
    }

    let source_denial_profile = install_loopback_profile(
        &runtime,
        &client,
        &session,
        &listener,
        true,
        false,
        "source-export-denial",
    );
    let source_denial_request = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &source_denial_profile,
        "source-choice-select-source-denial",
    );
    let proposal = propose_over_loopback(
        &runtime,
        &client,
        &session,
        &listener,
        source_denial_request,
        "source-export-denial",
    );
    worker.assert_denied(
        WorkerAttempt {
            proposal: &proposal,
            scope: &scope,
            task_id: &task.task_id,
            source: Some((&source_identity, &content_sha256)),
            context: None,
            suffix: "source-export-denial",
        },
        rrd_contract::ErrorCode::PermissionDenied,
    );

    let context_denial_profile = install_loopback_profile(
        &runtime,
        &client,
        &session,
        &listener,
        false,
        true,
        "context-export-denial",
    );
    let context_denial_request = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &context_denial_profile,
        "source-choice-select-context-denial",
    );
    let proposal = propose_over_loopback(
        &runtime,
        &client,
        &session,
        &listener,
        context_denial_request,
        "context-export-denial",
    );
    let context = rrd_contract::AssembleContext {
        scope: scope.clone(),
        query: "selected source".into(),
        valid_at: now,
        seeds: Vec::new(),
        max_graph_depth: 1,
        max_items: 8,
        max_output_bytes: 8192,
        max_storage_keys: 10_000,
    };
    worker.assert_denied(
        WorkerAttempt {
            proposal: &proposal,
            scope: &scope,
            task_id: &task.task_id,
            source: Some((&source_identity, &content_sha256)),
            context: Some(context.clone()),
            suffix: "context-export-denial",
        },
        rrd_contract::ErrorCode::PermissionDenied,
    );

    let ready_profile =
        install_loopback_profile(&runtime, &client, &session, &listener, true, true, "ready");
    let ignored_path_selection = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &ready_profile,
        "source-choice-select-ignored-path",
    );
    let proposal = propose_over_loopback(
        &runtime,
        &client,
        &session,
        &listener,
        ignored_path_selection,
        "ignored-source-path",
    );
    std::fs::write(project.join(".gitignore"), b"source.txt\n").unwrap();
    assert_eq!(
        std::fs::read(project.join(&source_path)).unwrap(),
        source_content.as_bytes()
    );
    worker.assert_denied(
        WorkerAttempt {
            proposal: &proposal,
            scope: &scope,
            task_id: &task.task_id,
            source: Some((&source_identity, &content_sha256)),
            context: None,
            suffix: "ignored-source-path",
        },
        rrd_contract::ErrorCode::FailedPrecondition,
    );
    std::fs::remove_file(project.join(".gitignore")).unwrap();
    let ready_selection = endpoint_selection_request(
        &scope,
        &task.task_id,
        &task_read.task,
        &ready_profile,
        "source-choice-select-ready",
    );
    let (proposal, wire_source) = worker.propose_and_execute(
        ready_selection,
        &scope,
        &task.task_id,
        Some((&source_identity, &content_sha256)),
        Some(context.clone()),
    );
    let output = wire_source.expect("successful loopback worker received source packet");
    assert_eq!(output, source_content);
    assert_eq!(proposal.content, "Reviewed one selected file.");
    assert!(proposal.source.is_some());

    let assessment_task = runtime
        .block_on(
            client.open_task(
                &session,
                rrd_contract::OpenTask {
                    scope: scope.clone(),
                    title: "Assess selected source separately".into(),
                    goal: "Evaluate explicit typed questions over one accepted source file.".into(),
                },
                RequestOptions::mutation(
                    "source-choice-assessment-open-task",
                    "source-choice-assessment-open-task",
                    "source-choice-assessment-open-task",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .task;
    let assessment_task_read = runtime
        .block_on(
            client.read_task(
                &session,
                rrd_contract::ReadTask {
                    scope: scope.clone(),
                    task_id: assessment_task.task_id.clone(),
                    max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
                },
                RequestOptions::read(
                    "source-choice-assessment-read-task",
                    "source-choice-assessment-read-task",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .task;
    assert_ne!(assessment_task_read.task_id, task.task_id);

    let budget_request = rrd_contract::ConfigureTaskBudget {
        scope: scope.clone(),
        task_id: assessment_task_read.task_id.clone(),
        limits: rrd_contract::ReasoningLimits {
            max_run_elapsed_ms: 120_000,
            max_steps: 8,
            max_step_elapsed_ms: 30_000,
        },
        repetition_policy: None,
    };
    let budget_before = runtime
        .block_on(
            client.configure_task_budget(
                &session,
                budget_request.clone(),
                RequestOptions::mutation(
                    "source-choice-assessment-budget-configure",
                    "source-choice-assessment-budget-configure",
                    "source-choice-assessment-budget-configure",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .budget
        .expect("public assessment task budget is configured");
    assert_eq!(budget_before.reserved_steps, 0);

    let assessment_configuration = install_loopback_assessment_profile(
        &runtime,
        &client,
        &session,
        &listener,
        true,
        true,
        "assessment-enabled",
    );
    let assessment_policy = assessment_configuration
        .profile
        .as_ref()
        .unwrap()
        .assessment_policy
        .as_ref()
        .expect("assessment requires explicit endpoint policy");
    assert!(assessment_policy.allow_network);
    assert!(assessment_policy.allow_source_export);
    assert!(assessment_policy.allow_context_export);
    let task_sha256 = rrd_contract::EndpointSelectionTask::from_task(&assessment_task_read)
        .unwrap()
        .task_sha256;
    let source_request = rrd_contract::TaskWorkerSourceRequest {
        contract_version: rrd_contract::TASK_WORKER_SOURCE_VERSION,
        entry_identity: source_identity.clone(),
        expected_content_sha256: content_sha256.clone(),
        maximum_input_bytes: 16_384,
    };
    let practice_identity = AssessmentPracticeIdentity {
        question_set_id: CanonicalId::new("installed-assessment-set").unwrap(),
        question_id: CanonicalId::new("installed-assessment-question").unwrap(),
        template_id: CanonicalId::new("installed-assessment-template").unwrap(),
        template_revision: 1,
        template_sha256: rrd_core::digest::sha256_hex(b"controlled installed practice declaration"),
        requirement_id: CanonicalId::new("installed-assessment-requirement").unwrap(),
    };
    let practice_text = "Review only the selected source bytes against this declared practice.";
    let descriptors = [
        TaskAssessmentDescriptor::Binary {
            proposition: "The selected source contains the stated fixture phrase.".into(),
        },
        TaskAssessmentDescriptor::Choice {
            question: "Which declared relation best matches the selected source?".into(),
            options: vec![
                AssessmentOption {
                    id: CanonicalId::new("supported").unwrap(),
                    description: "The selected bytes contain direct support.".into(),
                },
                AssessmentOption {
                    id: CanonicalId::new("uncertain").unwrap(),
                    description: "The selected bytes do not settle the question.".into(),
                },
            ],
        },
        TaskAssessmentDescriptor::Score {
            rubric: "Score how clearly the selected bytes state the fixture phrase.".into(),
            minimum: 0.0,
            maximum: 5.0,
            direction: AssessmentScoreDirection::HigherIsBetter,
        },
        TaskAssessmentDescriptor::Binary {
            proposition: "The selected source supports the declared practice.".into(),
        },
    ];
    let decisions = [
        (
            serde_json::json!({
                "outcome": "answered",
                "value": { "kind": "binary", "value": true }
            }),
            Some("source"),
        ),
        (
            serde_json::json!({
                "outcome": "answered",
                "value": { "kind": "choice", "option_id": "supported" }
            }),
            Some("source"),
        ),
        (
            serde_json::json!({
                "outcome": "answered",
                "value": { "kind": "score", "value": 3.0 }
            }),
            Some("practice"),
        ),
        (
            serde_json::json!({
                "outcome": "abstained",
                "reason": "The controlled packet does not contain the requested evidence."
            }),
            None,
        ),
    ];
    let mut assessment_outputs = Vec::new();
    let mut observed_budget = budget_before.clone();
    for (index, (descriptor, (decision, reference_input))) in
        descriptors.into_iter().zip(decisions).enumerate()
    {
        let suffix = ["binary", "choice", "score", "abstention"][index];
        let request = TaskAssessmentRequest {
            contract_version: TASK_ASSESSMENT_VERSION,
            scope: scope.clone(),
            task_id: assessment_task_read.task_id.clone(),
            attempt_id: rrd_contract::CorrelationId::new(format!(
                "source-choice-assessment-{suffix}"
            ))
            .unwrap(),
            expected_task_sha256: task_sha256.clone(),
            expected_configuration_sha256: assessment_configuration.sha256().unwrap(),
            max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
            source: source_request.clone(),
            descriptor,
            practice_identity: practice_identity.clone(),
            practice_text: practice_text.into(),
        };
        let (assessment, packet) = execute_installed_assessment(
            &runtime,
            &client,
            &session,
            &listener,
            &request,
            decision,
            reference_input,
        );
        assert_eq!(packet["scope"], request.scope);
        assert_eq!(packet["task_id"], request.task_id.as_str());
        assert_eq!(packet["task_sha256"], task_sha256);
        assert_eq!(
            packet["configuration_sha256"],
            request.expected_configuration_sha256
        );
        assert_eq!(
            packet["descriptor"],
            serde_json::to_value(&request.descriptor).unwrap()
        );
        assert_eq!(
            packet["practice_identity"],
            serde_json::to_value(&practice_identity).unwrap()
        );
        assert_eq!(packet["practice_text"], practice_text);
        assert_eq!(
            packet["practice_sha256"],
            rrd_core::digest::sha256_hex(practice_text.as_bytes())
        );
        assert_eq!(packet["source_content"], source_content);
        assert_eq!(packet["source"]["entry_identity"], source_identity);
        assert_eq!(packet["source"]["content_sha256"], content_sha256);
        assert_eq!(assessment.source.entry_identity, source_identity);
        assert_eq!(assessment.source.content_sha256, content_sha256);
        assert_eq!(
            assessment.practice_sha256,
            rrd_core::digest::sha256_hex(practice_text.as_bytes())
        );
        match (&request.descriptor, &assessment.decision, reference_input) {
            (
                TaskAssessmentDescriptor::Binary { .. },
                TaskAssessmentDecision::Answered {
                    value: TaskAssessmentValue::Binary { value: true },
                },
                Some("source"),
            ) if index == 0 => {}
            (
                TaskAssessmentDescriptor::Choice { .. },
                TaskAssessmentDecision::Answered {
                    value: TaskAssessmentValue::Choice { option_id },
                },
                Some("source"),
            ) if index == 1 && option_id.as_str() == "supported" => {}
            (
                TaskAssessmentDescriptor::Score { .. },
                TaskAssessmentDecision::Answered {
                    value: TaskAssessmentValue::Score { value: 3.0 },
                },
                Some("practice"),
            ) if index == 2 => {}
            (
                TaskAssessmentDescriptor::Binary { .. },
                TaskAssessmentDecision::Abstained { reason },
                None,
            ) if index == 3 && !reason.is_empty() => {}
            other => panic!("unexpected controlled assessment result: {other:?}"),
        }
        match (reference_input, assessment.references.as_slice()) {
            (Some("source"), [reference]) => {
                assert_eq!(reference.input, AssessmentInputKind::Source);
                assert_eq!(reference.input_sha256, content_sha256);
                assert_eq!((reference.start_byte, reference.end_byte), (0, 7));
                assert_eq!(&source_content[0..7], "bounded");
            }
            (Some("practice"), [reference]) => {
                assert_eq!(reference.input, AssessmentInputKind::Practice);
                assert_eq!(
                    reference.input_sha256,
                    rrd_core::digest::sha256_hex(practice_text.as_bytes())
                );
                assert_eq!((reference.start_byte, reference.end_byte), (0, 6));
                assert_eq!(&practice_text[0..6], "Review");
            }
            (None, []) => {}
            other => panic!("unexpected assessment reference binding: {other:?}"),
        }
        let next_budget = runtime
            .block_on(
                client.read_task_budget(
                    &session,
                    rrd_contract::ReadTaskBudget {
                        scope: scope.clone(),
                        task_id: assessment_task_read.task_id.clone(),
                    },
                    RequestOptions::read(
                        &format!("source-choice-assessment-budget-read-{suffix}"),
                        &format!("source-choice-assessment-budget-read-{suffix}"),
                    )
                    .unwrap(),
                ),
            )
            .unwrap()
            .budget
            .expect("public task budget remains readable");
        assert_eq!(
            next_budget.reserved_steps,
            observed_budget.reserved_steps + 1,
            "one explicit attempt reserves exactly one measured task-budget step"
        );
        assert!(next_budget.active_attempt.is_none());
        assert!(!next_budget.execution_may_continue);
        observed_budget = next_budget;
        assessment_outputs.push((request, assessment));
    }
    assert_eq!(observed_budget.reserved_steps, 4);

    let raw_session = open_raw_public_session(
        &address,
        &locator.identity.instance_id,
        &credential.principal_id,
        credential.credential(),
    );
    let mut malformed_request = assessment_outputs[0].0.clone();
    malformed_request.attempt_id =
        rrd_contract::CorrelationId::new("source-choice-assessment-unsupported-context").unwrap();
    let mut payload = serde_json::to_value(malformed_request).unwrap();
    payload["context"] = serde_json::to_value(&context).unwrap();
    let malformed_envelope = raw_envelope(
        &locator.identity.instance_id,
        payload,
        "source-choice-assessment-unsupported-context-request",
        "source-choice-assessment-unsupported-context-operation",
        Some("source-choice-assessment-unsupported-context-idempotency"),
    );
    let assessment_catalogue = rrd_contract::endpoint_catalogue();
    let assessment_execute_path = assessment_catalogue
        .endpoints
        .iter()
        .find(|endpoint| endpoint.operation.as_str() == "task-assessment-execute")
        .expect("public assessment execute endpoint is catalogued")
        .path
        .as_str();
    let (status, response) = raw_public_http(
        &address,
        "POST",
        assessment_execute_path,
        &malformed_envelope,
        &[
            ("X-RRD-Session", raw_session.session_id.as_str().to_owned()),
            (
                "Authorization",
                format!("Bearer {}", raw_session.token.as_str()),
            ),
        ],
    );
    assert_eq!(status, 400);
    assert_eq!(response["outcome"]["status"], "error");
    assert_eq!(response["outcome"]["error"]["code"], "invalid_argument");
    assert!(response["outcome"]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown field `context`"));
    assert_listener_empty(&listener);
    let after_unsupported_context = runtime
        .block_on(
            client.read_task_budget(
                &session,
                rrd_contract::ReadTaskBudget {
                    scope: scope.clone(),
                    task_id: assessment_task_read.task_id.clone(),
                },
                RequestOptions::read(
                    "source-choice-assessment-budget-after-unsupported-context",
                    "source-choice-assessment-budget-after-unsupported-context",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .budget
        .unwrap();
    assert_eq!(after_unsupported_context, observed_budget);

    for (suffix, allow_context_export, allow_source_export) in [
        ("source-export-denied", true, false),
        ("context-export-denied", false, true),
    ] {
        install_loopback_assessment_profile(
            &runtime,
            &client,
            &session,
            &listener,
            allow_context_export,
            allow_source_export,
            suffix,
        );
        let read = runtime.block_on(
            client.call_operation::<rrd_client::operations::TaskAssessmentRead>(
                &session,
                rrd_contract::ReadTaskAssessment {
                    contract_version: TASK_ASSESSMENT_VERSION,
                    scope: scope.clone(),
                    task_id: assessment_task_read.task_id.clone(),
                    attempt_id: assessment_outputs[0].0.attempt_id.clone(),
                },
                RequestOptions::read(
                    &format!("source-choice-assessment-read-{suffix}"),
                    &format!("source-choice-assessment-read-{suffix}"),
                )
                .unwrap(),
            ),
        );
        assert!(matches!(
            read,
            Err(rrd_client::Error::Api { error, .. })
                if error.code == ErrorCode::PermissionDenied
        ));
        assert_listener_empty(&listener);
    }
    install_loopback_assessment_profile(
        &runtime,
        &client,
        &session,
        &listener,
        true,
        true,
        "assessment-export-restored",
    );
    for (request, expected) in &assessment_outputs {
        let retained = runtime
            .block_on(
                client.call_operation::<rrd_client::operations::TaskAssessmentRead>(
                    &session,
                    rrd_contract::ReadTaskAssessment {
                        contract_version: TASK_ASSESSMENT_VERSION,
                        scope: scope.clone(),
                        task_id: assessment_task_read.task_id.clone(),
                        attempt_id: request.attempt_id.clone(),
                    },
                    RequestOptions::read(
                        &format!(
                            "source-choice-assessment-read-{}-restored",
                            request.attempt_id.as_str()
                        ),
                        &format!(
                            "source-choice-assessment-read-{}-restored",
                            request.attempt_id.as_str()
                        ),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(retained.assessment, Some(expected.clone()));
    }

    raw_close_public_session(&address, &locator.identity.instance_id, &raw_session);
    let attempt_id = proposal.attempt_id.clone();

    let close = runtime.block_on(
        client.close_session(
            &session,
            rrd_contract::CloseSession {},
            RequestOptions::mutation(
                "source-choice-close-session",
                "source-choice-close-session",
                "source-choice-close-session",
            )
            .unwrap(),
        ),
    );
    close.unwrap();
    service.stop(&mut lifecycle_output);
    let (mut reopened_service, reopened_address) =
        RunningEngine::start(project, &mut lifecycle_output);
    let reopened_locator = RrdEngine::read_project_locator(project).unwrap();
    let reopened_client = RrdClient::connect_local(
        reopened_address.parse().unwrap(),
        reopened_locator.identity.instance_id,
        ClientConfig::default(),
    )
    .unwrap();
    let fresh_session = runtime
        .block_on(
            reopened_client.create_session(
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
                    "source-choice-reauth-session-request",
                    "source-choice-reauth-session-operation",
                    "source-choice-reauth-session-idempotency",
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let readback = runtime
        .block_on(reopened_client.read_task_attempt(
            &fresh_session,
            rrd_contract::ReadTaskAttempt {
                scope: scope.clone(),
                task_id: task.task_id,
                attempt_id,
            },
            RequestOptions::read("source-choice-readback", "source-choice-readback").unwrap(),
        ))
        .unwrap();
    let retained = readback.attempt.unwrap().worker_output.unwrap();
    assert_eq!(retained.content, "Reviewed one selected file.");
    assert_eq!(retained.source.unwrap().content_sha256, content_sha256);
    assert_eq!(
        std::fs::read_to_string(project.join(source_path)).unwrap(),
        source_content
    );
    for (request, expected) in &assessment_outputs {
        let assessment_read = runtime
            .block_on(
                reopened_client.call_operation::<rrd_client::operations::TaskAssessmentRead>(
                    &fresh_session,
                    rrd_contract::ReadTaskAssessment {
                        contract_version: TASK_ASSESSMENT_VERSION,
                        scope: scope.clone(),
                        task_id: assessment_task_read.task_id.clone(),
                        attempt_id: request.attempt_id.clone(),
                    },
                    RequestOptions::read(
                        &format!(
                            "source-choice-assessment-reopen-read-{}",
                            request.attempt_id.as_str()
                        ),
                        &format!(
                            "source-choice-assessment-reopen-read-{}",
                            request.attempt_id.as_str()
                        ),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(assessment_read.assessment, Some(expected.clone()));
    }
    let reopened_budget = runtime
        .block_on(
            reopened_client.read_task_budget(
                &fresh_session,
                rrd_contract::ReadTaskBudget {
                    scope: scope.clone(),
                    task_id: assessment_task_read.task_id.clone(),
                },
                RequestOptions::read(
                    "source-choice-assessment-budget-reopen-read",
                    "source-choice-assessment-budget-reopen-read",
                )
                .unwrap(),
            ),
        )
        .unwrap()
        .budget
        .expect("assessment budget survives authenticated reopen");
    assert_eq!(reopened_budget.reserved_steps, 4);
    assert_eq!(reopened_budget.limits, budget_request.limits);
    assert_eq!(
        reopened_budget.deadline_unix_ms,
        observed_budget.deadline_unix_ms
    );
    assert_eq!(
        reopened_budget.charged_elapsed_ms,
        observed_budget.charged_elapsed_ms
    );
    assert!(reopened_budget.active_attempt.is_none());
    assert!(!reopened_budget.execution_may_continue);
    reopened_service.stop(&mut lifecycle_output);
    let verify = invoke(&[
        "--json",
        "verify",
        "--project",
        project.to_str().unwrap(),
        "--level",
        "quick",
    ]);
    assert_success(&verify, "verify source-read grant binding after reopen");
}

fn execute_installed_assessment(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    request: &TaskAssessmentRequest,
    decision: serde_json::Value,
    reference_input: Option<&str>,
) -> (rrd_contract::TaskAssessmentOutput, serde_json::Value) {
    std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, wire) = accept_loopback_chat(listener);
            assert_eq!(wire["messages"][0]["role"], "system");
            assert_eq!(
                wire["messages"][0]["content"],
                "Assess only the supplied packet; this controlled fixture is not a model result."
            );
            let packet: serde_json::Value =
                serde_json::from_str(wire["messages"][1]["content"].as_str().unwrap()).unwrap();
            assert!(packet.get("context").is_none());
            let response = assessment_reply_for_packet(&packet, decision, reference_input);
            reply_loopback_chat(&mut socket, wire["model"].as_str().unwrap(), &response);
            packet
        });
        let output = runtime
            .block_on(
                client.call_operation::<rrd_client::operations::TaskAssessmentExecute>(
                    session,
                    request.clone(),
                    RequestOptions::mutation(
                        &format!("assessment-execute-{}", request.attempt_id.as_str()),
                        &format!("assessment-execute-{}", request.attempt_id.as_str()),
                        &format!("assessment-execute-{}", request.attempt_id.as_str()),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        let packet = server.join().unwrap();
        assert_listener_empty(listener);
        (output, packet)
    })
}

fn assessment_reply_for_packet(
    packet: &serde_json::Value,
    decision: serde_json::Value,
    reference_input: Option<&str>,
) -> String {
    let references = reference_input
        .map(|input| {
            let (sha256, end_byte) = match input {
                "source" => (packet["source"]["content_sha256"].as_str().unwrap(), 7),
                "practice" => (packet["practice_sha256"].as_str().unwrap(), 6),
                other => panic!("unsupported controlled assessment reference input {other}"),
            };
            serde_json::json!([{
                "input": input,
                "input_sha256": sha256,
                "start_byte": 0,
                "end_byte": end_byte
            }])
        })
        .unwrap_or_else(|| serde_json::json!([]));
    serde_json::json!({
        "contract_version": TASK_ASSESSMENT_VERSION,
        "decision": decision,
        "references": references
    })
    .to_string()
}

fn assert_listener_empty(listener: &TcpListener) {
    listener.set_nonblocking(true).unwrap();
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

fn raw_envelope<T: serde::Serialize>(
    instance_id: &CanonicalId,
    payload: T,
    request_id: &str,
    operation_id: &str,
    idempotency_key: Option<&str>,
) -> rrd_contract::RequestEnvelope<T> {
    rrd_contract::RequestEnvelope {
        protocol: rrd_contract::PROTOCOL.into(),
        protocol_version: rrd_contract::PROTOCOL_VERSION,
        context: rrd_contract::RequestContext {
            request_id: rrd_contract::CorrelationId::new(request_id).unwrap(),
            operation_id: rrd_contract::CorrelationId::new(operation_id).unwrap(),
            idempotency_key: idempotency_key
                .map(|key| rrd_contract::CorrelationId::new(key).unwrap()),
            deadline_unix_ms: None,
        },
        resource: rrd_contract::ResourcePath {
            segments: vec![rrd_contract::ResourceId::new(
                rrd_contract::ResourceKind::Instance,
                instance_id.as_str(),
            )
            .unwrap()],
        },
        payload,
    }
}

fn raw_public_http(
    address: &str,
    method: &str,
    path: &str,
    envelope: &impl serde::Serialize,
    headers: &[(&str, String)],
) -> (u16, serde_json::Value) {
    let body = serde_json::to_vec(envelope).unwrap();
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n", body.len()).unwrap();
    for (name, value) in headers {
        write!(stream, "{name}: {value}\r\n").unwrap();
    }
    stream.write_all(b"\r\n").unwrap();
    stream.write_all(&body).unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    let split = response
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .unwrap();
    let status = String::from_utf8_lossy(&response[..split])
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    (
        status,
        serde_json::from_slice(&response[split + 4..]).unwrap(),
    )
}

fn open_raw_public_session(
    address: &str,
    instance_id: &CanonicalId,
    principal_id: &CanonicalId,
    api_key: &str,
) -> rrd_contract::SessionLease {
    let envelope = raw_envelope(
        instance_id,
        CreateSession {
            limits: SessionLimits {
                idle_timeout_ms: 60_000,
                absolute_timeout_ms: 300_000,
                max_open_transactions: 1,
            },
        },
        "assessment-raw-session-request",
        "assessment-raw-session-operation",
        Some("assessment-raw-session-idempotency"),
    );
    let (status, response) = raw_public_http(
        address,
        "POST",
        "/v1/sessions",
        &envelope,
        &[
            ("X-RRD-Principal", principal_id.as_str().to_owned()),
            ("Authorization", format!("ApiKey {api_key}")),
        ],
    );
    assert_eq!(status, 200);
    assert_eq!(response["outcome"]["status"], "ok");
    serde_json::from_value(response["outcome"]["payload"].clone()).unwrap()
}

fn raw_close_public_session(
    address: &str,
    instance_id: &CanonicalId,
    lease: &rrd_contract::SessionLease,
) {
    let envelope = raw_envelope(
        instance_id,
        CloseSession {},
        "assessment-raw-session-close-request",
        "assessment-raw-session-close-operation",
        Some("assessment-raw-session-close-idempotency"),
    );
    let (status, response) = raw_public_http(
        address,
        "DELETE",
        &format!("/v1/sessions/{}", lease.session_id.as_str()),
        &envelope,
        &[
            ("X-RRD-Session", lease.session_id.as_str().to_owned()),
            ("Authorization", format!("Bearer {}", lease.token.as_str())),
        ],
    );
    assert_eq!(status, 200);
    assert_eq!(response["outcome"]["status"], "ok");
}

fn install_loopback_profile(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    allow_context_export: bool,
    allow_source_export: bool,
    suffix: &str,
) -> rrd_contract::EndpointConfiguration {
    let profile = loopback_worker_profile(listener, allow_context_export, allow_source_export);
    apply_loopback_profile(runtime, client, session, suffix, profile)
}

fn install_loopback_assessment_profile(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    allow_context_export: bool,
    allow_source_export: bool,
    suffix: &str,
) -> rrd_contract::EndpointConfiguration {
    let mut profile = loopback_worker_profile(listener, allow_context_export, allow_source_export);
    profile.assessment_policy = Some(EndpointAssessmentPolicy {
        contract_version: TASK_ASSESSMENT_VERSION,
        allow_network: true,
        allow_source_export,
        allow_context_export,
        system_prompt:
            "Assess only the supplied packet; this controlled fixture is not a model result.".into(),
        generation: EndpointGenerationSettings {
            maximum_output_tokens: 128,
            token_parameter: EndpointTokenParameter::MaxTokens,
            temperature_milli: None,
            response_format: EndpointResponseFormat::JsonObject,
        },
        maximum_input_bytes: 16_384,
        maximum_response_bytes: 16_384,
    });
    apply_loopback_profile(runtime, client, session, suffix, profile)
}

fn apply_loopback_profile(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    suffix: &str,
    profile: rrd_contract::EndpointProfileInput,
) -> rrd_contract::EndpointConfiguration {
    let current = runtime
        .block_on(
            client.read_endpoint_configuration(
                session,
                rrd_contract::ReadEndpointConfiguration {},
                RequestOptions::read(
                    &format!("source-choice-profile-read-{suffix}"),
                    &format!("source-choice-profile-read-{suffix}"),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let plan = runtime
        .block_on(
            client.preview_endpoint_configuration(
                session,
                rrd_contract::PreviewEndpointConfiguration {
                    expected_revision: current.revision,
                    profile: Some(profile),
                },
                RequestOptions::mutation(
                    &format!("source-choice-profile-preview-{suffix}"),
                    &format!("source-choice-profile-preview-{suffix}"),
                    &format!("source-choice-profile-preview-{suffix}"),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    runtime
        .block_on(
            client.apply_endpoint_configuration(
                session,
                rrd_contract::ApplyEndpointConfiguration {
                    approved_plan_sha256: plan.plan_sha256.clone(),
                    plan,
                },
                RequestOptions::mutation(
                    &format!("source-choice-profile-apply-{suffix}"),
                    &format!("source-choice-profile-apply-{suffix}"),
                    &format!("source-choice-profile-apply-{suffix}"),
                )
                .unwrap(),
            ),
        )
        .unwrap()
}

fn endpoint_selection_request(
    scope: &str,
    task_id: &rrd_contract::CanonicalId,
    task: &rrd_contract::TaskRecord,
    configuration: &rrd_contract::EndpointConfiguration,
    attempt_id: &str,
) -> rrd_contract::EndpointSelectionRequest {
    rrd_contract::EndpointSelectionRequest {
        scope: scope.to_owned(),
        task_id: task_id.clone(),
        attempt_id: rrd_contract::CorrelationId::new(attempt_id).unwrap(),
        max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
        expected_task_sha256: rrd_contract::EndpointSelectionTask::from_task(task)
            .unwrap()
            .task_sha256,
        expected_configuration_sha256: configuration.sha256().unwrap(),
        flight: None,
        routing: None,
    }
}

fn propose_over_loopback(
    runtime: &tokio::runtime::Runtime,
    client: &RrdClient,
    session: &rrd_client::Session,
    listener: &TcpListener,
    request: rrd_contract::EndpointSelectionRequest,
    suffix: &str,
) -> rrd_contract::EndpointSelectionProposal {
    std::thread::scope(|scope| {
        let server = scope.spawn(|| {
            let (mut socket, wire) = accept_loopback_chat(listener);
            let binding_id = rrd_contract::CanonicalId::new("controlled-loopback-binding").unwrap();
            let choice = rrd_contract::EndpointSelectionChoice::Select {
                binding_id,
                effort: None,
            };
            let model = wire["model"].as_str().unwrap();
            reply_loopback_chat(&mut socket, model, &serde_json::to_string(&choice).unwrap());
        });
        let result = runtime
            .block_on(
                client.call_operation::<rrd_client::operations::TaskEndpointPropose>(
                    session,
                    request,
                    RequestOptions::mutation(
                        &format!("source-choice-proposal-{suffix}"),
                        &format!("source-choice-proposal-{suffix}"),
                        &format!("source-choice-proposal-{suffix}"),
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        server.join().unwrap();
        result
    })
}

fn worker_request(
    proposal: &rrd_contract::EndpointSelectionProposal,
    scope: &str,
    task_id: &rrd_contract::CanonicalId,
    source: Option<(&str, &str)>,
    context: Option<rrd_contract::AssembleContext>,
    suffix: &str,
) -> rrd_contract::TaskWorkerRequest {
    rrd_contract::TaskWorkerRequest {
        scope: scope.to_owned(),
        task_id: task_id.clone(),
        attempt_id: rrd_contract::CorrelationId::new(format!("source-choice-worker-{suffix}"))
            .unwrap(),
        selection_attempt_id: proposal.attempt_id.clone(),
        expected_selection_sha256: rrd_core::digest::sha256_hex(
            &serde_json::to_vec(proposal).unwrap(),
        ),
        expected_task_sha256: proposal.task_sha256.clone(),
        expected_configuration_sha256: proposal.configuration_sha256.clone(),
        max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
        context,
        source: source.map(|(entry_identity, expected_content_sha256)| {
            rrd_contract::TaskWorkerSourceRequest {
                contract_version: rrd_contract::TASK_WORKER_SOURCE_VERSION,
                entry_identity: entry_identity.to_owned(),
                expected_content_sha256: expected_content_sha256.to_owned(),
                maximum_input_bytes: 64 * 1024,
            }
        }),
    }
}

struct InstalledWorker<'a> {
    runtime: &'a tokio::runtime::Runtime,
    client: &'a RrdClient,
    session: &'a rrd_client::Session,
    listener: &'a TcpListener,
}

struct WorkerAttempt<'a> {
    proposal: &'a rrd_contract::EndpointSelectionProposal,
    scope: &'a str,
    task_id: &'a rrd_contract::CanonicalId,
    source: Option<(&'a str, &'a str)>,
    context: Option<rrd_contract::AssembleContext>,
    suffix: &'a str,
}

impl InstalledWorker<'_> {
    fn assert_denied(&self, attempt: WorkerAttempt<'_>, expected_error: rrd_contract::ErrorCode) {
        let request = worker_request(
            attempt.proposal,
            attempt.scope,
            attempt.task_id,
            attempt.source,
            attempt.context,
            attempt.suffix,
        );
        let result = self.runtime.block_on(
            self.client.execute_task_worker(
                self.session,
                request,
                RequestOptions::mutation(
                    &format!("source-choice-worker-deny-{}", attempt.suffix),
                    &format!("source-choice-worker-deny-{}", attempt.suffix),
                    &format!("source-choice-worker-deny-{}", attempt.suffix),
                )
                .unwrap(),
            ),
        );
        assert!(matches!(
            result,
            Err(rrd_client::Error::Api { error, .. }) if error.code == expected_error
        ));
        self.listener.set_nonblocking(true).unwrap();
        assert!(matches!(
            self.listener.accept(),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
        ));
    }

    fn propose_and_execute(
        &self,
        selection_request: rrd_contract::EndpointSelectionRequest,
        scope: &str,
        task_id: &rrd_contract::CanonicalId,
        source: Option<(&str, &str)>,
        context: Option<rrd_contract::AssembleContext>,
    ) -> (rrd_contract::TaskWorkerOutput, Option<String>) {
        std::thread::scope(|scope_thread| {
            let server = scope_thread.spawn(|| {
                let (mut selection_socket, selection_wire) = accept_loopback_chat(self.listener);
                let choice = rrd_contract::EndpointSelectionChoice::Select {
                    binding_id: rrd_contract::CanonicalId::new("controlled-loopback-binding")
                        .unwrap(),
                    effort: None,
                };
                let model = selection_wire["model"].as_str().unwrap();
                reply_loopback_chat(
                    &mut selection_socket,
                    model,
                    &serde_json::to_string(&choice).unwrap(),
                );
                let (mut worker_socket, worker_wire) = accept_loopback_chat(self.listener);
                let packet: serde_json::Value =
                    serde_json::from_str(worker_wire["messages"][1]["content"].as_str().unwrap())
                        .unwrap();
                let source_text = packet["source"]["content"].as_str().map(str::to_owned);
                let model = worker_wire["model"].as_str().unwrap();
                reply_loopback_chat(&mut worker_socket, model, "Reviewed one selected file.");
                source_text
            });
            let proposal = self
                .runtime
                .block_on(
                    self.client
                        .call_operation::<rrd_client::operations::TaskEndpointPropose>(
                            self.session,
                            selection_request,
                            RequestOptions::mutation(
                                "source-choice-ready-proposal",
                                "source-choice-ready-proposal",
                                "source-choice-ready-proposal",
                            )
                            .unwrap(),
                        ),
                )
                .unwrap();
            let request = worker_request(&proposal, scope, task_id, source, context, "ready");
            let output = self
                .runtime
                .block_on(
                    self.client.execute_task_worker(
                        self.session,
                        request,
                        RequestOptions::mutation(
                            "source-choice-ready-worker",
                            "source-choice-ready-worker",
                            "source-choice-ready-worker",
                        )
                        .unwrap(),
                    ),
                )
                .unwrap();
            let source_text = server.join().unwrap();
            (output, source_text)
        })
    }
}

fn installed_lifecycle(mode: &str) {
    eprintln!(
        "native {mode} lifecycle executable bytes={}",
        std::fs::metadata(binary()).unwrap().len()
    );
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    if mode == "existing" {
        std::fs::write(project.join("README.md"), "installed walking product\n").unwrap();
    }
    let plan_file = root.path().join("install-plan.json");
    let configuration_file = root.path().join("estate-configuration.toml");
    std::fs::write(
        &configuration_file,
        r#"format_version = 3

[storage]
mutable_accounted_bytes = 134217728
mutable_versions = 1048576
projected_output_bytes = 134217728
source_page_bytes = 268435456
native_transactions = 64
native_transaction_bytes = 67108864
checkpoint_entries = 1024
active_read_views = 1024
pinned_read_bytes = 4294967296
page_cache_bytes = 67108864
wal_flush_target_bytes = 67108864
memtable_flush_target_versions = 524288
compaction_input_bytes = 1073741824
compaction_input_logical_bytes = 268435456
compaction_input_versions = 4194304
compaction_output_segments = 256
compaction_elapsed_ms = 30000
compaction_l0_segments = 64
retained_mutable_bytes = 536870912
retained_mutable_versions = 4194304
retained_mutable_generations = 64
reclamation_inventory_entries = 262144
reclamation_metadata_bytes = 67108864
reclamation_delete_files = 1024
reclamation_delete_bytes = 1073741824
reclamation_elapsed_ms = 30000
transaction_admission_requests = 64
transaction_admission_bytes = 67108864
transaction_admission_wait_ms = 30000
recovery_metadata_bytes = 67108864
recovery_files = 262144
recovery_read_bytes = 1099511627776
recovery_wal_bytes = 268435456
recovery_wal_frames = 1048576
recovery_elapsed_ms = 120000
recovery_reserve_bytes = 16777216

[clock]
maximum_rollback_ms = 0

[reasoning]
max_run_elapsed_ms = 720000
max_steps = 192
max_step_elapsed_ms = 45000

[recall]
max_graph_depth = 3
max_items = 96
max_output_bytes = 393216
max_storage_keys = 75000

[query]
max_storage_keys = 75000
max_rows = 768
max_output_bytes = 393216
max_batch_rows = 192
max_memory_bytes = 33554432
max_spill_bytes = 67108864
max_elapsed_ms = 20000
"#,
    )
    .unwrap();
    let project_arg = project.to_str().unwrap();
    let plan_arg = plan_file.to_str().unwrap();
    let configuration_arg = configuration_file.to_str().unwrap();
    let mut lifecycle_output = Vec::new();

    let version = invoke(&["--json", "version"]);
    capture_output(&version, &mut lifecycle_output);
    assert_success(&version, "version");
    let version: serde_json::Value = serde_json::from_slice(&version.stdout).unwrap();
    assert_eq!(version["product_version"], "1.0.0");
    assert_eq!(version["installation_plan_contract_version"], 4);
    assert_eq!(version["release_stage"], "pre-release");

    let before = inventory(&project);
    let identity_output = invoke(&["install", "identity"]);
    assert_success(&identity_output, "allocate explicit installation identity");
    let identity: serde_json::Value = serde_json::from_slice(&identity_output.stdout).unwrap();
    let identity = identity["installation_identity"].as_str().unwrap();
    assert_eq!(identity.len(), 64);
    let next_identity = invoke(&["install", "identity"]);
    assert_success(&next_identity, "allocate another independent identity");
    assert_ne!(identity_output.stdout, next_identity.stdout);
    assert_eq!(
        inventory(&project),
        before,
        "identity allocation wrote project state"
    );
    let plan_arguments = [
        "install",
        "plan",
        "--identity",
        identity,
        "--mode",
        mode,
        "--project",
        project_arg,
        "--configuration",
        configuration_arg,
    ];
    let first = invoke(&plan_arguments);
    let second = invoke(&plan_arguments);
    capture_output(&first, &mut lifecycle_output);
    capture_output(&second, &mut lifecycle_output);
    assert_success(&first, "first install plan");
    assert_success(&second, "second install plan");
    assert_eq!(first.stdout, second.stdout, "preview must be byte stable");
    assert_eq!(inventory(&project), before, "preview mutated the project");
    let preview: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(preview["installation"]["contract_version"], 4);
    assert_eq!(preview["installation"]["installation_identity"], identity);
    assert_eq!(
        preview["installation"]["configuration"]["reasoning"]["max_run_elapsed_ms"],
        720_000
    );
    assert_eq!(
        preview["installation"]["configuration"]["recall"]["max_items"],
        96
    );
    assert_eq!(
        preview["installation"]["deployment"]["deployment_form"],
        "single_node_server"
    );
    let managed_paths = preview["installation"]["managed_paths"].as_array().unwrap();
    assert_eq!(managed_paths.len(), 13);
    let bootstrap: rrd_contract::InstallationBootstrap =
        serde_json::from_value(preview["installation"]["bootstrap"].clone()).unwrap();
    bootstrap.validate().unwrap();
    assert_eq!(bootstrap.specialization.id.as_str(), "generic");
    assert!(bootstrap.specialization.endpoint_profile.is_none());
    assert!(bootstrap.process_invocations.is_empty());
    assert!(bootstrap.network_endpoints.is_empty());

    assert!(managed_paths
        .iter()
        .any(|path| path["relative_path"] == ".rrflow/config.toml"));
    assert!(managed_paths
        .iter()
        .all(|path| !path["relative_path"].as_str().unwrap().starts_with('/')));
    let plan_sha256 = preview["installation"]["plan_sha256"]
        .as_str()
        .unwrap()
        .to_owned();
    std::fs::write(&plan_file, &first.stdout).unwrap();

    let foreign_project = root.path().join("foreign-project");
    std::fs::create_dir(&foreign_project).unwrap();
    for (relative, content) in &before {
        std::fs::write(foreign_project.join(relative), content).unwrap();
    }
    let foreign = invoke(&[
        "install",
        "apply",
        "--mode",
        mode,
        "--project",
        foreign_project.to_str().unwrap(),
        "--plan",
        plan_arg,
        "--expect",
        &plan_sha256,
    ]);
    assert!(
        !foreign.status.success(),
        "a plan cannot initialize an identical foreign project"
    );
    assert_eq!(
        inventory(&foreign_project),
        before,
        "foreign apply wrote managed state"
    );

    let rejected = invoke(&[
        "install",
        "apply",
        "--mode",
        mode,
        "--project",
        project_arg,
        "--plan",
        plan_arg,
        "--expect",
        &"0".repeat(64),
    ]);
    capture_output(&rejected, &mut lifecycle_output);
    assert!(!rejected.status.success());
    assert!(!project.join(".rrflow").exists());

    let applied = invoke(&[
        "--json",
        "install",
        "apply",
        "--mode",
        mode,
        "--project",
        project_arg,
        "--plan",
        plan_arg,
        "--expect",
        &plan_sha256,
    ]);
    capture_output(&applied, &mut lifecycle_output);
    assert_success(&applied, "install apply");
    assert_eq!(
        std::fs::read(project.join(".rrflow/bootstrap.md")).unwrap(),
        bootstrap.rendered_instructions().as_bytes()
    );
    for (relative, bytes) in &before {
        assert_eq!(
            std::fs::read(project.join(relative)).unwrap(),
            *bytes,
            "installation changed project-owned content"
        );
    }

    let applied: serde_json::Value = serde_json::from_slice(&applied.stdout).unwrap();
    assert_eq!(applied["plan_sha256"], plan_sha256);
    assert_eq!(applied["runtime_cursor"], 2);
    assert_eq!(applied["control_journal_sequence"], 8);
    assert_eq!(applied["idempotent_replay"], false);
    let installed_at = applied["applied_at_unix_ms"].as_u64().unwrap();

    let installed_before_verify = inventory(&project);
    let credential_path = project.join(
        managed_paths
            .iter()
            .find(|path| path["kind"] == "operator_credential")
            .unwrap()["relative_path"]
            .as_str()
            .unwrap(),
    );
    let alias = root.path().join("foreign-credential-alias");
    std::fs::hard_link(&credential_path, &alias).unwrap();
    let denied_verify = invoke(&[
        "--json",
        "verify",
        "--project",
        project_arg,
        "--level",
        "quick",
    ]);
    capture_output(&denied_verify, &mut lifecycle_output);
    assert!(
        !denied_verify.status.success(),
        "aliased credentials must fail verification"
    );
    assert_eq!(
        inventory(&project),
        installed_before_verify,
        "denied verify changed state"
    );
    assert!(
        std::fs::read(&alias).unwrap() == std::fs::read(&credential_path).unwrap(),
        "denial changed credential bytes"
    );
    std::fs::remove_file(&alias).unwrap();
    let verified = invoke(&[
        "--json",
        "verify",
        "--project",
        project_arg,
        "--level",
        "quick",
    ]);
    capture_output(&verified, &mut lifecycle_output);
    assert_success(&verified, "offline verify");
    let verified: serde_json::Value = serde_json::from_slice(&verified.stdout).unwrap();
    assert_eq!(verified["status"], "passed");
    assert_eq!(verified["runtime_cursor"], 2);
    assert_eq!(verified["clock"]["status"], "current");
    assert_eq!(verified["clock"]["anchor"]["source"], "host_system_time");
    assert_eq!(verified["clock"]["anchor"]["trust"], "unverified");
    assert_eq!(
        verified["clock"]["anchor"]["observed_at_unix_ms"],
        installed_at
    );
    assert_eq!(inventory(&project), installed_before_verify);

    let replay = invoke(&[
        "--json",
        "install",
        "apply",
        "--mode",
        mode,
        "--project",
        project_arg,
        "--plan",
        plan_arg,
        "--expect",
        &plan_sha256,
    ]);
    capture_output(&replay, &mut lifecycle_output);
    assert_success(&replay, "idempotent install replay");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&replay.stdout).unwrap()["idempotent_replay"],
        true
    );

    let (mut child, address) = RunningEngine::start(&project, &mut lifecycle_output);
    let address = address.as_str();

    let ready = invoke(&[
        "--json",
        "ready",
        "--project",
        project_arg,
        "--address",
        address,
    ]);
    capture_output(&ready, &mut lifecycle_output);
    assert_success(&ready, "live UI discovery");
    let ready: serde_json::Value = serde_json::from_slice(&ready.stdout).unwrap();
    assert_eq!(ready["status"], "ready");
    assert_eq!(ready["readiness"]["runtime_cursor"], 2);
    assert_eq!(ready["authentication"]["principal_id"], "local-operator");
    assert_eq!(ready["authentication"]["session_closed"], true);
    assert_eq!(ready["openapi"]["openapi"], "3.1.0");
    assert_eq!(
        ready["capabilities"]["configuration"]["reasoning"]["max_run_elapsed_ms"],
        720_000
    );
    assert_eq!(
        ready["capabilities"]["configuration"]["recall"]["max_items"],
        96
    );
    assert_eq!(
        ready["capabilities"]["deployment"]["endpoint_presentation"],
        "loopback_http_websocket"
    );
    assert!(
        ready["endpoint_catalogue"]["endpoints"]
            .as_array()
            .unwrap()
            .len()
            > 20
    );

    let locator = RrdEngine::read_project_locator(&project).unwrap();
    let credential = RrdEngine::read_installed_api_key(&project).unwrap();
    let client = RrdClient::connect_local(
        address.parse().unwrap(),
        locator.identity.instance_id,
        // Backup/restore include authenticated physical verification. Their
        // explicit budget is separate from the default short-request timeout;
        // overlapping attempts are covered by deterministic engine tests.
        ClientConfig {
            request_timeout: Duration::from_secs(60),
            ..ClientConfig::default()
        },
    )
    .unwrap();
    let (backup_sha256, first_query, committed_claim_sequence) = tokio::runtime::Builder::new_current_thread()
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
                        "installed-test-session-request",
                        "installed-test-session-operation",
                        "installed-test-session-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let transaction = client
                .begin_transaction(
                    &session,
                    BeginTransaction {
                        scope: CanonicalId::new("claims").unwrap(),
                        timeout_ms: 10_000,
                    },
                    RequestOptions::mutation(
                        "installed-test-begin-request",
                        "installed-test-begin-operation",
                        "installed-test-begin-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let mutations = vec![TransactionMutation::AssertClaim {
                subject: CanonicalId::new("installed-walking-product").unwrap(),
                predicate: CanonicalId::new("status").unwrap(),
                object: "ui-queryable".into(),
                valid_from: installed_at,
                tx_time: installed_at,
                producer: CanonicalId::new("local-operator").unwrap(),
                confidence: Some(1.0),
            }];
            let receipt = client
                .commit_transaction(
                    &session,
                    &transaction.transaction_id,
                    CommitTransaction {
                        operation_sha256: transaction_operation_sha256(&mutations),
                        mutations,
                    },
                    RequestOptions::new(
                        "installed-test-commit-request",
                        "installed-test-commit-operation",
                        Some("installed-test-commit-idempotency"),
                        Some(u64::MAX),
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(receipt.mutation_count, 1);

            let query_text = format!(
                "FROM claim:status AT VALID {installed_at} KNOWN HEAD WHERE subject = \"installed-walking-product\" PROJECT subject, object EXPLAIN CONTRACT"
            );
            let denied = client
                .execute_query(
                    &session,
                    ExecuteQuery {
                        scope: format!("instance:{}", client.instance_id()),
                        query: query_text.clone(),
                        parameters: BTreeMap::new(),
                        budget: QueryBudget::default(),
                    },
                    RequestOptions::read(
                        "installed-test-over-ceiling-request",
                        "installed-test-over-ceiling-operation",
                    )
                    .unwrap(),
                )
                .await
                .unwrap_err();
            match denied {
                rrd_client::Error::Api { status, error } => {
                    assert_eq!(status.as_u16(), 429);
                    assert_eq!(error.code, ErrorCode::ResourceExhausted);
                }
                other => panic!("unexpected configured-ceiling error: {other}"),
            }

            let query = client
                .execute_query(
                    &session,
                    ExecuteQuery {
                        scope: format!("instance:{}", client.instance_id()),
                        query: query_text,
                        parameters: BTreeMap::new(),
                        budget: QueryBudget {
                            max_storage_keys: 75_000,
                            max_rows: 768,
                            max_output_bytes: 393_216,
                            max_batch_rows: 192,
                            max_memory_bytes: 33_554_432,
                            max_spill_bytes: 67_108_864,
                            max_elapsed_ms: 20_000,
                        },
                    },
                    RequestOptions::read(
                        "installed-test-query-request",
                        "installed-test-query-operation",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(query.rows.len(), 1);
            assert_eq!(
                query.rows[0].identity,
                "claim:f2f88858872febdd0b233f7aa5c42f01b042d67faa6afd244d35e69feded14e7"
            );
            assert_eq!(
                query.rows[0].values.get("object"),
                Some(&QueryValue::String("ui-queryable".into()))
            );
            let context = client.assemble_context(&session, rrd_contract::AssembleContext {
                scope: format!("instance:{}", client.instance_id()), query: "ui-queryable".into(),
                valid_at: installed_at, seeds: Vec::new(), max_graph_depth: 0, max_items: 8,
                max_output_bytes: 32_768, max_storage_keys: 75_000,
            }, RequestOptions::read("installed-test-context-request", "installed-test-context-operation").unwrap()).await.unwrap();
            assert_eq!(context.result_identity_version, query.result_identity_version);
            assert!(context.items.iter().any(|item| item.identity == query.rows[0].identity));
            verify_parquet_export(&client, &session, &query).await;

            // A backup at runtime cursor 3, a later commit the backup does not
            // carry, and a staged restore of the backup for offline publication.
            let backup = client
                .create_backup(
                    &session,
                    CreateInstanceBackup {
                        label: "cli-publication".into(),
                        created_at_unix_ms: installed_at + 1_000,
                    },
                    RequestOptions::mutation(
                        "installed-test-backup-request",
                        "installed-test-backup-operation",
                        "installed-test-backup-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert!(backup.backup.application_complete);
            assert_eq!(
                backup.backup.installed_control,
                BackupCoverageSnapshot::Included,
                "the SDK-visible backup names the installed control state it carries"
            );
            let later = client
                .begin_transaction(
                    &session,
                    BeginTransaction {
                        scope: CanonicalId::new("claims").unwrap(),
                        timeout_ms: 10_000,
                    },
                    RequestOptions::mutation(
                        "installed-test-later-begin-request",
                        "installed-test-later-begin-operation",
                        "installed-test-later-begin-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let later_mutations = vec![TransactionMutation::AssertClaim {
                subject: CanonicalId::new("installed-walking-product").unwrap(),
                predicate: CanonicalId::new("status").unwrap(),
                object: "after-backup".into(),
                valid_from: installed_at + 2_000,
                tx_time: installed_at + 2_000,
                producer: CanonicalId::new("local-operator").unwrap(),
                confidence: Some(1.0),
            }];
            client
                .commit_transaction(
                    &session,
                    &later.transaction_id,
                    CommitTransaction {
                        operation_sha256: transaction_operation_sha256(&later_mutations),
                        mutations: later_mutations,
                    },
                    RequestOptions::new(
                        "installed-test-later-commit-request",
                        "installed-test-later-commit-operation",
                        Some("installed-test-later-commit-idempotency"),
                        Some(u64::MAX),
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            let staged = client
                .restore_backup(
                    &session,
                    RestoreInstanceBackup {
                        backup_sha256: backup.backup.backup_sha256.clone(),
                        restore_id: CanonicalId::new("restore-cli").unwrap(),
                        restored_at_unix_ms: installed_at + 3_000,
                    },
                    RequestOptions::mutation(
                        "installed-test-restore-request",
                        "installed-test-restore-operation",
                        "installed-test-restore-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(staged.inventory.runtime_cursor, 3);
            client
                .close_session(
                    &session,
                    CloseSession {},
                    RequestOptions::mutation(
                        "installed-test-close-request",
                        "installed-test-close-operation",
                        "installed-test-close-idempotency",
                    )
                    .unwrap(),
                )
                .await
                .unwrap();
            (backup.backup.backup_sha256, query, receipt.last_claim_sequence)
        });

    let live_verify = invoke(&["verify", "--project", project_arg, "--level", "quick"]);
    capture_output(&live_verify, &mut lifecycle_output);
    assert!(
        !live_verify.status.success(),
        "offline verifier must not race the live writer"
    );

    child.terminate(&mut lifecycle_output);
    assert_restarted_status(
        &project,
        installed_at + 2_000,
        "after-backup",
        &mut lifecycle_output,
    );
    let reopened = invoke(&[
        "--json",
        "verify",
        "--project",
        project_arg,
        "--level",
        "quick",
    ]);
    capture_output(&reopened, &mut lifecycle_output);
    assert_success(&reopened, "post-shutdown reopen and verify");
    let reopened: serde_json::Value = serde_json::from_slice(&reopened.stdout).unwrap();
    assert_eq!(reopened["status"], "passed");
    assert_eq!(reopened["runtime_cursor"], 4);
    assert!(reopened["control_journal_sequence"].as_u64().unwrap() > 8);

    // Offline publication of the staged restore through the primary command
    // tree: a deterministic, no-write plan, an exact-digest apply, and a
    // verified estate at the backup's state with the old root quarantined.
    let publication_plan_file = root.path().join("restore-plan.json");
    let publication_plan_arg = publication_plan_file.to_str().unwrap();
    let plan_arguments = [
        "restore",
        "plan",
        "--project",
        project_arg,
        "--restore",
        "restore-cli",
        "--backup",
        &backup_sha256,
    ];
    let before_publication = inventory(&project);
    let first_plan = invoke(&plan_arguments);
    let second_plan = invoke(&plan_arguments);
    capture_output(&first_plan, &mut lifecycle_output);
    assert_success(&first_plan, "restore plan");
    assert_success(&second_plan, "second restore plan");
    assert_eq!(
        first_plan.stdout, second_plan.stdout,
        "plan must be byte stable"
    );
    assert_eq!(
        inventory(&project),
        before_publication,
        "planning mutated the project"
    );
    let publication: serde_json::Value = serde_json::from_slice(&first_plan.stdout).unwrap();
    assert_eq!(publication["active_root"], locator.storage_root);
    assert_eq!(publication["candidate"]["runtime_cursor"], 3);
    assert_eq!(publication["active"]["runtime_cursor"], 4);
    let publication_sha256 = publication["plan_sha256"].as_str().unwrap().to_owned();
    std::fs::write(&publication_plan_file, &first_plan.stdout).unwrap();
    let rejected = invoke(&[
        "restore",
        "apply",
        "--project",
        project_arg,
        "--plan",
        publication_plan_arg,
        "--expect",
        &"0".repeat(64),
    ]);
    capture_output(&rejected, &mut lifecycle_output);
    assert!(!rejected.status.success());
    assert_eq!(inventory(&project), before_publication);
    let published = invoke(&[
        "--json",
        "restore",
        "apply",
        "--project",
        project_arg,
        "--plan",
        publication_plan_arg,
        "--expect",
        &publication_sha256,
    ]);
    capture_output(&published, &mut lifecycle_output);
    assert_success(&published, "restore apply");
    let published: serde_json::Value = serde_json::from_slice(&published.stdout).unwrap();
    assert_eq!(published["plan_sha256"], publication_sha256);
    assert_eq!(published["runtime_cursor"], 3);
    assert_eq!(published["idempotent_replay"], false);
    assert!(project
        .join(published["quarantine_root"].as_str().unwrap())
        .is_dir());
    let after_publication = invoke(&[
        "--json",
        "verify",
        "--project",
        project_arg,
        "--level",
        "quick",
    ]);
    capture_output(&after_publication, &mut lifecycle_output);
    assert_success(&after_publication, "post-publication verify");
    let after_publication: serde_json::Value =
        serde_json::from_slice(&after_publication.stdout).unwrap();
    assert_eq!(after_publication["status"], "passed");
    assert_eq!(after_publication["runtime_cursor"], 3);

    let restored_query = assert_restarted_status(
        &project,
        installed_at,
        "ui-queryable",
        &mut lifecycle_output,
    );
    assert!(restored_query.known_at_cursor >= committed_claim_sequence);
    assert_eq!(restored_query.canonical_query, first_query.canonical_query);
    assert_eq!(restored_query.scope, first_query.scope);
    assert_eq!(restored_query.schema_revision, first_query.schema_revision);
    assert_eq!(
        restored_query.read_manifest_sha256,
        first_query.read_manifest_sha256
    );
    assert_eq!(restored_query.rows, first_query.rows);
    for (relative, bytes) in before {
        assert_eq!(std::fs::read(project.join(relative)).unwrap(), bytes);
    }

    // Neither successful nor rejected lifecycle stdout/stderr may contain the
    // generated credential.
    assert!(!String::from_utf8_lossy(&lifecycle_output).contains(credential.credential()));
    println!(
        "native_lifecycle_receipt={}",
        serde_json::json!({
            "mode": mode,
            "build": {
                "executable_sha256": version["build"]["executable_sha256"],
                "metadata_sha256": version["build"]["metadata_sha256"],
                "public_contract_sha256": version["build"]["public_contract_sha256"],
                "signal_catalogue_sha256": version["build"]["signal_catalogue_sha256"],
            },
            "installation_plan_sha256": plan_sha256,
            "restore_plan_sha256": publication_sha256,
            "backup_sha256": backup_sha256,
            "installed_runtime_cursor": applied["runtime_cursor"],
            "reopened_runtime_cursor": reopened["runtime_cursor"],
            "restored_runtime_cursor": after_publication["runtime_cursor"],
            "committed_claim_sequence": committed_claim_sequence,
            "restored_read_manifest_sha256": restored_query.read_manifest_sha256,
            "captured_output_bytes": lifecycle_output.len(),
            "stream_limit_bytes": OUTPUT_LIMIT,
        })
    );
    storage_history::run(
        &project,
        mode,
        installed_at,
        &version["build"],
        &mut lifecycle_output,
    );
}

/// H-07h: an instance started from the project folder records its endpoint,
/// another process proves it through `ready` without an address, and a second
/// serve of the same estate fails before binding.
#[test]
fn a_served_engram_is_found_proved_and_not_served_twice() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("README.md"), "served endpoint record\n").unwrap();
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
    let locator = RrdEngine::read_project_locator(&project).unwrap();
    let record_path = project.join(".rrflow").join("serving.json");
    let read_record = || -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap()
    };
    let ready_from_record = || invoke(&["--json", "ready", "--project", project_arg]);
    let mut captured = Vec::new();

    let unrecorded = ready_from_record();
    assert!(!unrecorded.status.success(), "nothing has been served yet");
    assert!(String::from_utf8_lossy(&unrecorded.stderr).contains("no served endpoint is recorded"));

    let (mut first, address) = RunningEngine::start(&project, &mut captured);
    let record = read_record();
    assert_eq!(record["url"], format!("http://{address}"));
    assert_eq!(record["instance_id"], locator.identity.instance_id.as_str());
    assert_eq!(record["process_id"], first.child.id());
    let ready = ready_from_record();
    capture_output(&ready, &mut captured);
    assert_success(&ready, "ready from the served endpoint record");
    let ready: serde_json::Value = serde_json::from_slice(&ready.stdout).unwrap();
    assert_eq!(ready["status"], "ready");
    assert_eq!(ready["url"], format!("http://{address}"));
    assert_eq!(ready["authentication"]["session_closed"], true);

    let second = invoke(&["serve", "--project", project_arg, "--bind", "127.0.0.1:0"]);
    capture_output(&second, &mut captured);
    assert!(!second.status.success(), "a second serve of an open estate");
    let refusal = String::from_utf8_lossy(&second.stderr);
    assert!(
        refusal.contains("already open in another process"),
        "{refusal}"
    );
    assert!(refusal.contains(&format!("http://{address}")), "{refusal}");
    assert_eq!(read_record()["url"], format!("http://{address}"));
    assert_success(&ready_from_record(), "the first server still proves ready");

    // A killed server cannot withdraw its record; the record alone is not readiness.
    first.terminate(&mut captured);
    assert!(record_path.exists());
    let stale = ready_from_record();
    assert!(!stale.status.success(), "a stale record proved ready");
    assert!(String::from_utf8_lossy(&stale.stderr).contains("is not ready"));

    let (mut restarted, replacement) = RunningEngine::start(&project, &mut captured);
    assert_eq!(read_record()["url"], format!("http://{replacement}"));
    assert_eq!(read_record()["process_id"], restarted.child.id());
    assert_success(
        &ready_from_record(),
        "ready after the stale record is replaced",
    );
    restarted.stop(&mut captured);
    #[cfg(unix)]
    assert!(
        !record_path.exists(),
        "a clean shutdown withdraws the record"
    );

    let locator_text =
        std::fs::read_to_string(project.join(".rrflow").join("config.toml")).unwrap();
    assert!(
        !locator_text.contains(&address),
        "the locator holds no endpoint"
    );
    let credential = RrdEngine::read_installed_api_key(&project).unwrap();
    assert!(!String::from_utf8_lossy(&captured).contains(credential.credential()));
    assert!(!std::fs::read_to_string(&record_path)
        .unwrap_or_default()
        .contains(credential.credential()));
}
