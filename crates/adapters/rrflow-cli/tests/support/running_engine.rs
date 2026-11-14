//! A served installed engine under test, and the restart status it must report.

use super::*;

// Test-owned handles are always reaped, including assertion/startup failures.
// No PID search or process-name kill may affect another installed engine.
pub(super) struct RunningEngine {
    pub(super) child: process::Process,
    pub(super) finished: bool,
}

impl RunningEngine {
    pub(super) fn start(project: &Path, captured: &mut Vec<u8>) -> (Self, String) {
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

    pub(super) fn terminate(&mut self, captured: &mut Vec<u8>) {
        self.child.terminate();
        let output = self.child.finish().expect("reap bounded rrflow process");
        capture_output(&output, captured);
        self.finished = true;
    }

    pub(super) fn stop(&mut self, captured: &mut Vec<u8>) {
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

pub(super) fn assert_restarted_status(
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
