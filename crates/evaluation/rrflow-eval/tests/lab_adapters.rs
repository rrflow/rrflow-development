//! Each adapter against a scripted server that replays responses captured from the
//! real systems: Qdrant 1.20.0, SurrealDB 3.3.5 and Milvus 3.0.5. These prove the
//! wire format stays right and failures are noticed. Measured results come from
//! real runs, recorded in the package journal.
use rrflow_eval::lab::adapter::{CollectionSpec, PointBatch, SearchRequest, SystemAdapter};
use rrflow_eval::lab::custom::CustomAdapter;
use rrflow_eval::lab::manifest::{
    Availability, CredentialReferences, IndexSpec, LabMetric, SearchSetting, SystemBinding,
    SystemKind, SystemLocation, WorkloadSpec,
};
use rrflow_eval::lab::milvus::MilvusAdapter;
use rrflow_eval::lab::qdrant::QdrantAdapter;
use rrflow_eval::lab::surrealdb::SurrealAdapter;
use rrflow_eval::storage::Id;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Recorded {
    method: String,
    path: String,
    headers: String,
    body: String,
}

impl Recorded {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap()
    }
}

/// Answers scripted responses in order and records what it was sent.
struct ScriptedServer {
    endpoint: String,
    requests: Arc<Mutex<Vec<Recorded>>>,
}

impl ScriptedServer {
    fn start(script: &[(u16, &str)]) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        let script: Vec<(u16, String)> = script
            .iter()
            .map(|(status, body)| (*status, body.to_string()))
            .collect();
        std::thread::spawn(move || {
            for (status, body) in script {
                let (mut stream, _) = listener.accept().unwrap();
                let mut bytes = Vec::new();
                let mut buffer = [0_u8; 65_536];
                let (head_end, length) = loop {
                    let read = stream.read(&mut buffer).unwrap();
                    bytes.extend_from_slice(&buffer[..read]);
                    if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                        let length = head
                            .lines()
                            .find_map(|line| line.strip_prefix("content-length:"))
                            .map_or(0, |value| value.trim().parse().unwrap());
                        break (end + 4, length);
                    }
                };
                while bytes.len() < head_end + length {
                    let read = stream.read(&mut buffer).unwrap();
                    bytes.extend_from_slice(&buffer[..read]);
                }
                let head = String::from_utf8_lossy(&bytes[..head_end]).to_string();
                let mut first = head.lines().next().unwrap().split_whitespace();
                recorded.lock().unwrap().push(Recorded {
                    method: first.next().unwrap().to_string(),
                    path: first.next().unwrap().to_string(),
                    headers: head.clone(),
                    body: String::from_utf8_lossy(&bytes[head_end..]).to_string(),
                });
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        Self { endpoint, requests }
    }

    fn request(&self, index: usize) -> Recorded {
        self.requests.lock().unwrap()[index].clone()
    }
}

fn workload() -> WorkloadSpec {
    WorkloadSpec {
        queries: 1,
        k: 3,
        warmup_queries: 0,
        ingest_batch: 2,
        concurrency: vec![1],
        filtered: true,
        request_timeout_ms: 5_000,
        readiness_timeout_ms: 5_000,
        max_case_seconds: 5,
    }
}

fn binding(endpoint: &str, kind: SystemKind, credentials: CredentialReferences) -> SystemBinding {
    SystemBinding {
        id: Id::try_from("subject".to_string()).unwrap(),
        label: "Subject".into(),
        kind,
        location: SystemLocation::Loopback,
        endpoint: endpoint.into(),
        artifact: "pinned".into(),
        storage_profile: "disk".into(),
        availability: Availability::Available,
        index: IndexSpec::Hnsw {
            m: 16,
            ef_construction: 100,
        },
        search_ladder: Vec::new(),
        engine_settings: Default::default(),
        limits: Vec::new(),
        credentials,
        memory_probe: None,
        disk_probe: None,
    }
}

const HNSW: IndexSpec = IndexSpec::Hnsw {
    m: 16,
    ef_construction: 100,
};

fn spec(index: &IndexSpec) -> CollectionSpec<'_> {
    CollectionSpec {
        name: "lab_unit",
        dimensions: 2,
        metric: LabMetric::Cosine,
        categories: 2,
        index,
    }
}

fn setting() -> SearchSetting {
    SearchSetting {
        id: Id::try_from("ef-32".to_string()).unwrap(),
        ef: Some(32),
        exact_rerank: None,
        nprobes: None,
    }
}

fn batch() -> PointBatch<'static> {
    PointBatch {
        ids: &[1, 2],
        vectors: &[1.0, 0.0, 0.0, 1.0],
        categories: &[0, 1],
        dimensions: 2,
    }
}

fn search_filtered(adapter: &dyn SystemAdapter) -> Vec<u64> {
    adapter
        .search(&SearchRequest {
            vector: &[1.0, 0.0],
            k: 3,
            category: Some(1),
            setting: &setting(),
        })
        .unwrap()
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

#[test]
fn qdrant_speaks_its_real_wire_format() {
    let ok = r#"{"result":true,"status":"ok","time":0.02}"#;
    let upsert = r#"{"result":{"operation_id":3,"status":"completed"},"status":"ok","time":0.1}"#;
    let info = r#"{"result":{"status":"green","optimizer_status":"ok","indexed_vectors_count":0,"points_count":40}}"#;
    let mut script: Vec<(u16, &str)> = vec![
        (
            200,
            r#"{"title":"qdrant - vector search engine","version":"1.20.0"}"#,
        ),
        (200, ok),
        (200, ok),
        (200, upsert),
        (200, upsert),
    ];
    script.extend(std::iter::repeat_n((200, info), 8));
    script.push((
        200,
        r#"{"result":{"points":[{"id":4,"version":3,"score":0.95},{"id":10,"version":3,"score":0.74}]},"status":"ok"}"#,
    ));
    script.push((200, ok));
    let server = ScriptedServer::start(&script);
    let adapter = QdrantAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Qdrant,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    assert_eq!(adapter.observed_version().unwrap(), "1.20.0");
    adapter.prepare(&spec(&HNSW)).unwrap();
    let create = server.request(2);
    assert_eq!(
        (create.method.as_str(), create.path.as_str()),
        ("PUT", "/collections/lab_unit")
    );
    assert_eq!(
        create.json()["vectors"],
        json!({"size": 2, "distance": "Cosine"})
    );
    assert_eq!(
        create.json()["hnsw_config"],
        json!({"m": 16, "ef_construct": 100})
    );
    assert_eq!(server.request(3).json()["field_name"], "category");
    adapter.ingest(&batch()).unwrap();
    let upserted = server.request(4);
    assert_eq!(upserted.path, "/collections/lab_unit/points?wait=true");
    assert_eq!(upserted.json()["points"][1]["payload"]["category"], 1);
    adapter.finish_ingest(deadline()).unwrap();
    assert_eq!(search_filtered(&adapter), vec![4, 10]);
    let query = server.request(13).json();
    assert_eq!(query["params"]["hnsw_ef"], 32);
    assert_eq!(query["filter"]["must"][0]["match"]["value"], 1);
    adapter.teardown().unwrap();
    assert_eq!(server.request(14).method, "DELETE");
}

#[test]
fn qdrant_http_errors_surface_with_their_text() {
    let server = ScriptedServer::start(&[
        (200, r#"{"result":true}"#),
        (400, r#"{"status":{"error":"bad vector size"}}"#),
    ]);
    let adapter = QdrantAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Qdrant,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    let error = adapter.prepare(&spec(&HNSW)).unwrap_err();
    assert!(
        error.contains("HTTP 400") && error.contains("bad vector size"),
        "{error}"
    );
}

fn surreal_credentials() -> CredentialReferences {
    std::env::set_var("LAB_TEST_SURREAL_USER", "root");
    std::env::set_var("LAB_TEST_SURREAL_SECRET", "root");
    CredentialReferences {
        principal_env: Some("LAB_TEST_SURREAL_USER".into()),
        secret_env: Some("LAB_TEST_SURREAL_SECRET".into()),
        instance_env: None,
    }
}

#[test]
fn surrealdb_speaks_its_real_wire_format() {
    let ok = r#"[{"result":null,"status":"OK","time":"1ms","type":null}]"#;
    let server = ScriptedServer::start(&[
        (200, "surrealdb-3.3.5"),
        (200, ok),
        (200, ok),
        (
            200,
            r#"[{"result":{"building":{"compacting":true,"initial":0,"pending":3,"status":"indexing","updated":0}},"status":"OK"}]"#,
        ),
        (
            200,
            r#"[{"result":{"building":{"compacting":true,"initial":0,"pending":0,"status":"ready","updated":0}},"status":"OK"}]"#,
        ),
        (
            200,
            r#"[{"result":[{"id":"lab_unit:4"},{"id":"lab_unit:10"}],"status":"OK","time":"1ms","type":null}]"#,
        ),
    ]);
    let adapter = SurrealAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Surrealdb,
            surreal_credentials(),
        ),
        &workload(),
    )
    .unwrap();
    assert_eq!(adapter.observed_version().unwrap(), "surrealdb-3.3.5");
    adapter.prepare(&spec(&HNSW)).unwrap();
    let define = server.request(1);
    assert!(
        define.headers.contains("Basic cm9vdDpyb290"),
        "{}",
        define.headers
    );
    assert!(define.headers.to_lowercase().contains("surreal-ns: lab"));
    assert!(
        define
            .body
            .contains("HNSW DIMENSION 2 DIST COSINE TYPE F32 EFC 100 M 16"),
        "{}",
        define.body
    );
    adapter.ingest(&batch()).unwrap();
    assert!(
        server
            .request(2)
            .body
            .contains("{id:2,v:[0.0,1.0],category:1}"),
        "{}",
        server.request(2).body
    );
    adapter.finish_ingest(deadline()).unwrap();
    assert_eq!(search_filtered(&adapter), vec![4, 10]);
    let search = server.request(5).body;
    assert!(
        search.contains("category = 1 AND v <|3,32|> [1.0,0.0]"),
        "{search}"
    );
}

#[test]
fn surrealdb_statement_errors_are_failures_despite_http_200() {
    let server = ScriptedServer::start(&[(
        200,
        r#"[{"result":"The index dimension does not match","status":"ERR","time":"1ms"}]"#,
    )]);
    let adapter = SurrealAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Surrealdb,
            surreal_credentials(),
        ),
        &workload(),
    )
    .unwrap();
    let error = adapter.prepare(&spec(&HNSW)).unwrap_err();
    assert!(error.contains("statement failed"), "{error}");
}

#[test]
fn surrealdb_requires_credentials_by_reference() {
    let missing = CredentialReferences {
        principal_env: Some("LAB_TEST_NEVER_SET_USER".into()),
        secret_env: Some("LAB_TEST_NEVER_SET_SECRET".into()),
        instance_env: None,
    };
    let result = SurrealAdapter::new(
        &binding("http://127.0.0.1:9", SystemKind::Surrealdb, missing),
        &workload(),
    );
    assert!(result.err().unwrap().contains("LAB_TEST_NEVER_SET_USER"));
}

#[test]
fn milvus_speaks_its_real_wire_format() {
    let ok = r#"{"code":0,"data":{}}"#;
    let server = ScriptedServer::start(&[
        (200, ok),
        (200, ok),
        (
            200,
            r#"{"code":0,"cost":0,"data":{"insertCount":2,"insertIds":[1,2]}}"#,
        ),
        (200, ok),
        (
            200,
            r#"{"code":0,"data":[{"indexState":"InProgress","indexedRows":0,"pendingRows":2,"totalRows":2}]}"#,
        ),
        (
            200,
            r#"{"code":0,"data":[{"failReason":"","indexName":"v","indexState":"Finished","indexType":"HNSW","indexedRows":2,"pendingRows":0,"totalRows":2}]}"#,
        ),
        (200, ok),
        (
            200,
            r#"{"code":0,"data":{"loadProgress":100,"loadState":"LoadStateLoaded"},"message":""}"#,
        ),
        (
            200,
            r#"{"code":0,"cost":0,"data":[{"distance":0.95,"id":4},{"distance":0.74,"id":"10"}],"topks":[2]}"#,
        ),
    ]);
    let adapter = MilvusAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Milvus,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    adapter.prepare(&spec(&HNSW)).unwrap();
    assert_eq!(server.request(0).path, "/v2/vectordb/collections/drop");
    let create = server.request(1).json();
    assert_eq!(
        create["schema"]["fields"][1]["elementTypeParams"]["dim"],
        "2"
    );
    assert_eq!(create["indexParams"][0]["metricType"], "COSINE");
    assert_eq!(create["indexParams"][0]["params"]["efConstruction"], 100);
    adapter.ingest(&batch()).unwrap();
    assert_eq!(server.request(2).json()["data"][1]["category"], 1);
    adapter.finish_ingest(deadline()).unwrap();
    assert_eq!(search_filtered(&adapter), vec![4, 10]);
    let search = server.request(8).json();
    assert_eq!(search["filter"], "category == 1");
    assert_eq!(search["searchParams"]["params"]["ef"], 32);
    assert_eq!(search["consistencyLevel"], "Strong");
}

#[test]
fn milvus_reachability_uses_its_post_list_route_and_claims_no_version() {
    let server = ScriptedServer::start(&[(200, r#"{"code":0,"data":[]}"#)]);
    let adapter = MilvusAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Milvus,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    assert_eq!(
        adapter.observed_version().unwrap(),
        "reachable; version from pinned image"
    );
    let request = server.request(0);
    assert_eq!(
        (request.method.as_str(), request.path.as_str()),
        ("POST", "/v2/vectordb/collections/list")
    );
}

#[test]
fn milvus_failures_are_noticed_despite_http_200() {
    let server = ScriptedServer::start(&[(200, r#"{"code":1100,"message":"invalid dimension"}"#)]);
    let adapter = MilvusAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Milvus,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    let error = adapter.prepare(&spec(&HNSW)).unwrap_err();
    assert!(error.contains("1100"), "{error}");
}

#[test]
fn milvus_failed_index_builds_stop_instead_of_waiting_out_the_deadline() {
    let ok = r#"{"code":0,"data":{}}"#;
    let server = ScriptedServer::start(&[
        (200, ok),
        (200, ok),
        (200, ok),
        (
            200,
            r#"{"code":0,"data":[{"failReason":"out of memory","indexState":"Failed"}]}"#,
        ),
    ]);
    let adapter = MilvusAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Milvus,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    adapter.prepare(&spec(&HNSW)).unwrap();
    let error = adapter.finish_ingest(deadline()).unwrap_err();
    assert!(error.contains("out of memory"), "{error}");
}

#[test]
fn custom_systems_follow_the_documented_protocol() {
    let server = ScriptedServer::start(&[
        (
            200,
            r#"{"protocol_version":1,"name":"my-store","version":"2.1"}"#,
        ),
        (200, "{}"),
        (200, "{}"),
        (200, r#"{"ready":false}"#),
        (200, r#"{"ready":true}"#),
        (200, r#"{"ids":[4,10]}"#),
        (200, "{}"),
    ]);
    let adapter = CustomAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Custom,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    assert_eq!(adapter.observed_version().unwrap(), "my-store 2.1");
    adapter.prepare(&spec(&HNSW)).unwrap();
    assert_eq!(server.request(1).path, "/prepare");
    assert_eq!(
        server.request(1).json()["index"],
        json!({"index": "hnsw", "m": 16, "ef_construction": 100})
    );
    adapter.ingest(&batch()).unwrap();
    assert_eq!(
        server.request(2).json()["vectors"],
        json!([[1.0, 0.0], [0.0, 1.0]])
    );
    adapter.finish_ingest(deadline()).unwrap();
    assert_eq!(search_filtered(&adapter), vec![4, 10]);
    assert_eq!(server.request(5).json()["category"], 1);
    adapter.teardown().unwrap();
    assert_eq!(server.request(6).path, "/teardown");
}

#[test]
fn custom_systems_on_another_protocol_version_are_refused() {
    let server = ScriptedServer::start(&[(
        200,
        r#"{"protocol_version":99,"name":"my-store","version":"2.1"}"#,
    )]);
    let adapter = CustomAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Custom,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    assert!(adapter
        .observed_version()
        .unwrap_err()
        .contains("protocol 99"));
}

#[test]
fn custom_responses_outside_the_protocol_are_violations() {
    let server = ScriptedServer::start(&[(200, r#"{"ids":[1],"cheating":true}"#)]);
    let adapter = CustomAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Custom,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    let error = adapter
        .search(&SearchRequest {
            vector: &[1.0],
            k: 1,
            category: None,
            setting: &setting(),
        })
        .unwrap_err();
    assert!(error.contains("protocol violation"), "{error}");
}

#[test]
fn surrealdb_splits_inserts_under_its_body_limit() {
    let ok = r#"[{"result":null,"status":"OK","time":"1ms","type":null}]"#;
    let server = ScriptedServer::start(&[(200, ok), (200, ok), (200, ok)]);
    let adapter = SurrealAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Surrealdb,
            surreal_credentials(),
        ),
        &workload(),
    )
    .unwrap();
    adapter.prepare(&spec(&HNSW)).unwrap();
    let vectors = vec![0.123_456_7_f32; 2 * 60_000];
    adapter
        .ingest(&PointBatch {
            ids: &[1, 2],
            vectors: &vectors,
            categories: &[0, 1],
            dimensions: 60_000,
        })
        .unwrap();
    let first = server.request(1).body;
    let second = server.request(2).body;
    assert!(first.len() < 1024 * 1024 && first.contains("{id:1,"));
    assert!(second.len() < 1024 * 1024 && second.contains("{id:2,"));
}

#[test]
fn index_evidence_quotes_what_each_system_reports() {
    let info = r#"{"result":{"status":"green","indexed_vectors_count":0,"points_count":40,"segments_count":8}}"#;
    let ok = r#"{"result":true}"#;
    let server = ScriptedServer::start(&[(200, ok), (200, ok), (200, ok), (200, info)]);
    let adapter = QdrantAdapter::new(
        &binding(
            &server.endpoint,
            SystemKind::Qdrant,
            CredentialReferences::default(),
        ),
        &workload(),
    );
    adapter.prepare(&spec(&HNSW)).unwrap();
    assert_eq!(
        adapter.index_evidence().unwrap().unwrap(),
        "0 of 40 points indexed across 8 segments"
    );
}

#[test]
fn a_custom_system_may_report_evidence_and_may_decline() {
    let binding = |endpoint: &str| {
        binding(
            endpoint,
            SystemKind::Custom,
            CredentialReferences::default(),
        )
    };
    let server = ScriptedServer::start(&[(200, r#"{"evidence":"IVF_RQ: 40 rows indexed"}"#)]);
    let adapter = CustomAdapter::new(&binding(&server.endpoint), &workload());
    assert_eq!(
        adapter.index_evidence().unwrap().as_deref(),
        Some("IVF_RQ: 40 rows indexed")
    );

    let server = ScriptedServer::start(&[(404, r#"{"error":"unknown route"}"#)]);
    let adapter = CustomAdapter::new(&binding(&server.endpoint), &workload());
    assert_eq!(adapter.index_evidence().unwrap(), None);

    let server = ScriptedServer::start(&[(200, r#"{"unexpected":true}"#)]);
    let adapter = CustomAdapter::new(&binding(&server.endpoint), &workload());
    let refused = adapter.index_evidence().unwrap_err();
    assert!(refused.contains("protocol violation"), "{refused}");
}
