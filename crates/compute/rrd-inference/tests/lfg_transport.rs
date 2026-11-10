#![cfg(feature = "endpoint-chat")]

use rrd_inference::lfg_transport::{LfgTransportConfig, PreparedLfgObservation};
use rrd_inference::{EmbeddingCancellation, EmbeddingExecutionControl, NetworkPolicy};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

const PROMPT: &str =
    "Synthetic fixture: inspect a software project and propose a bounded implementation plan.";

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/lfg-synthetic-observation.json")).unwrap()
}

fn config<'a>(url: &'a str, fixture: &'a Value) -> LfgTransportConfig<'a> {
    LfgTransportConfig {
        base_url: url,
        assessment_sha256: fixture["request"]["expectedAssessmentSha256"]
            .as_str()
            .unwrap(),
        rules_sha256: fixture["request"]["expectedRulesSha256"].as_str().unwrap(),
        timeout: Duration::from_secs(5),
        maximum_input_bytes: 65_536,
        maximum_request_bytes: 65_536,
        maximum_response_bytes: 65_536,
    }
}

#[test]
fn request_is_bounded_and_denied_network_makes_no_call() {
    let fixture = fixture();
    for url in [
        "http://localhost:4317",
        "http://127.0.0.1:4317/path",
        "https://127.0.0.1:4317",
        "http://127.0.0.1:4317?query=1",
    ] {
        assert!(
            PreparedLfgObservation::new(&config(url, &fixture), "rrflow-fixture", 1, PROMPT)
                .is_err()
        );
    }
    let mut config = config("http://127.0.0.1:4317", &fixture);
    config.maximum_request_bytes = 8;
    assert!(PreparedLfgObservation::new(&config, "rrflow-fixture", 1, PROMPT).is_err());
    config.maximum_request_bytes = 65_536;
    let request = PreparedLfgObservation::new(&config, "rrflow-fixture", 1, PROMPT).unwrap();
    let control = EmbeddingExecutionControl::new(
        Instant::now() + Duration::from_secs(5),
        EmbeddingCancellation::default(),
    );
    let error = request.execute(NetworkPolicy::Deny, &control).unwrap_err();
    assert!(!error.execution_may_continue);
    let cancellation = EmbeddingCancellation::default();
    cancellation.cancel();
    let control =
        EmbeddingExecutionControl::new(Instant::now() + Duration::from_secs(5), cancellation);
    let error = request.execute(NetworkPolicy::Allow, &control).unwrap_err();
    assert!(!error.execution_may_continue);
}

#[test]
fn sends_the_expected_input_and_checks_the_response_identity() {
    let fixture = fixture();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let response = serde_json::to_vec(&fixture["observation"]).unwrap();
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 1024];
        loop {
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0);
            bytes.extend_from_slice(&buffer[..read]);
            if let Some(split) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..split]);
                let length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap();
                if bytes.len() >= split + 4 + length {
                    let request: Value =
                        serde_json::from_slice(&bytes[split + 4..split + 4 + length]).unwrap();
                    assert_eq!(request["text"], PROMPT);
                    assert_eq!(request["requestId"], "rrflow-fixture");
                    break;
                }
            }
        }
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response.len()
        )
        .unwrap();
        stream.write_all(&response).unwrap();
    });
    let request =
        PreparedLfgObservation::new(&config(&url, &fixture), "rrflow-fixture", 1, PROMPT).unwrap();
    let control = EmbeddingExecutionControl::new(
        Instant::now() + Duration::from_secs(5),
        EmbeddingCancellation::default(),
    );
    let observed = request.execute(NetworkPolicy::Allow, &control).unwrap();
    assert_eq!(observed.observation.schema_version, "lfg.observation.v1");
    assert_eq!(observed.observation.intent.status, "inconclusive");
    assert_eq!(observed.request_sha256.len(), 64);
    assert_eq!(observed.response_sha256.len(), 64);
    worker.join().unwrap();
}

#[test]
#[ignore = "requires an explicitly launched LFG v1 service using the captured fixture configuration"]
fn real_lfg_service_returns_bound_advisory_data() {
    let fixture = fixture();
    let url = std::env::var("RRFLOW_TEST_LFG_URL").expect("configure the test LFG service origin");
    let request =
        PreparedLfgObservation::new(&config(&url, &fixture), "rrflow-fixture", 1, PROMPT).unwrap();
    let control = EmbeddingExecutionControl::new(
        Instant::now() + Duration::from_secs(10),
        EmbeddingCancellation::default(),
    );
    let observed = request.execute(NetworkPolicy::Allow, &control).unwrap();
    assert_eq!(
        observed.observation.input_sha256,
        fixture["request"]["inputSha256"]
    );
    assert_eq!(observed.observation.schema_version, "lfg.observation.v1");
    assert!(observed.observation.advisory);
}

#[test]
fn response_mismatch_is_an_uncertain_external_effect() {
    let fixture = fixture();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let mut response = fixture["observation"].clone();
    response["inputSha256"] = Value::String("a".repeat(64));
    let response = serde_json::to_vec(&response).unwrap();
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut buffer = [0u8; 8192];
        assert!(stream.read(&mut buffer).unwrap() > 0);
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response.len()
        )
        .unwrap();
        stream.write_all(&response).unwrap();
    });
    let request =
        PreparedLfgObservation::new(&config(&url, &fixture), "rrflow-fixture", 1, PROMPT).unwrap();
    let control = EmbeddingExecutionControl::new(
        Instant::now() + Duration::from_secs(5),
        EmbeddingCancellation::default(),
    );
    let error = request.execute(NetworkPolicy::Allow, &control).unwrap_err();
    assert!(error.execution_may_continue);
    worker.join().unwrap();
}
