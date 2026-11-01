//! Actual primary-executable parser pipe tests; native process isolation is
//! separately required before the engine can activate this worker.
use rrd_core::digest::sha256_hex;
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

fn input(source: &[u8], digest: &str) -> Vec<u8> {
    let header = serde_json::to_vec(&json!({
        "format_version": 1,
        "source_id": "project/source.rs",
        "content_sha256": digest,
        "language": "rust",
        "source_bytes": source.len(),
        "limits": {"input_bytes": 4096, "nodes": 512, "depth": 64,
                   "output_bytes": 65536, "elapsed_micros": 5_000_000}
    }))
    .unwrap();
    let mut bytes = (header.len() as u32).to_be_bytes().to_vec();
    bytes.extend(header);
    bytes.extend(source);
    bytes
}

fn run(bytes: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rrflow"))
        .arg("--rrflow-internal-parse-worker-v1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    child.wait_with_output().unwrap()
}

fn response(output: &[u8]) -> Value {
    let length = u32::from_be_bytes(output[..4].try_into().unwrap()) as usize;
    assert_eq!(length, output.len() - 4);
    serde_json::from_slice(&output[4..]).unwrap()
}

#[test]
fn primary_executable_parses_only_the_supplied_source() {
    let source = b"fn actual_process_symbol() {}";
    let output = run(&input(source, &sha256_hex(source)));
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let response = response(&output.stdout);
    assert_eq!(response["status"], "parsed");
    assert_eq!(response["proposal"]["content_sha256"], sha256_hex(source));
    assert!(response["proposal"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| {
            let start = node["start_byte"].as_u64().unwrap() as usize;
            let end = node["end_byte"].as_u64().unwrap() as usize;
            source.get(start..end) == Some(b"actual_process_symbol")
        }));
}

#[test]
fn primary_executable_refuses_bad_digest_and_truncated_input() {
    let source = b"fn selected() {}";
    let output = run(&input(source, &"0".repeat(64)));
    assert!(output.status.success());
    assert_eq!(response(&output.stdout)["code"], "source_digest_mismatch");
    assert!(output.stderr.is_empty());

    let mut truncated = input(source, &sha256_hex(source));
    truncated.pop();
    let output = run(&truncated);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());

    let output = run(&u32::MAX.to_be_bytes());
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
