use rrd_attunement::parse::ParseLimits;
use rrd_attunement::worker::{
    read_response, run_worker, write_request, WorkerRequest, WorkerResponse,
};
use rrd_core::digest::sha256_hex;
use std::io::Cursor;

fn request(source: &[u8]) -> WorkerRequest {
    WorkerRequest {
        format_version: 1,
        source_id: "project/source.rs".into(),
        content_sha256: sha256_hex(source),
        language: "rust".into(),
        source_bytes: source.len() as u64,
        limits: ParseLimits {
            input_bytes: 4096,
            nodes: 512,
            depth: 64,
            output_bytes: 65536,
            elapsed_micros: 5_000_000,
        },
    }
}

#[test]
fn worker_binds_the_response_to_the_supplied_source() {
    let source = b"fn unique_symbol() {}";
    let request = request(source);
    let mut input = Vec::new();
    write_request(&mut input, &request, source).unwrap();
    let mut output = Vec::new();
    run_worker(&mut Cursor::new(input), &mut output).unwrap();
    let response = read_response(&mut Cursor::new(output), request.limits.output_bytes).unwrap();
    let WorkerResponse::Parsed { proposal } = response else {
        panic!("worker did not return a syntax proposal");
    };
    assert_eq!(proposal.source_id, request.source_id);
    assert_eq!(proposal.content_sha256, request.content_sha256);
    assert!(proposal.nodes.iter().any(|node| {
        &source[node.start_byte as usize..node.end_byte as usize] == b"unique_symbol"
    }));
}

#[test]
fn worker_rejects_truncated_and_oversized_frames_without_a_proposal() {
    let source = b"fn selected() {}";
    let request = request(source);
    let mut input = Vec::new();
    write_request(&mut input, &request, source).unwrap();
    input.pop();
    let mut output = Vec::new();
    assert!(run_worker(&mut Cursor::new(input), &mut output).is_err());
    assert!(output.is_empty());
    assert!(run_worker(&mut Cursor::new(u32::MAX.to_be_bytes()), &mut output).is_err());
    assert!(output.is_empty());
    assert!(read_response(&mut Cursor::new(u32::MAX.to_be_bytes()), 100).is_err());
}

#[test]
fn worker_returns_only_a_fixed_failure_for_changed_source() {
    let source = b"fn selected() {}";
    let mut request = request(source);
    request.content_sha256 = "0".repeat(64);
    let mut input = Vec::new();
    write_request(&mut input, &request, source).unwrap();
    let mut output = Vec::new();
    run_worker(&mut Cursor::new(input), &mut output).unwrap();
    assert_eq!(
        read_response(&mut Cursor::new(output), request.limits.output_bytes).unwrap(),
        WorkerResponse::Rejected {
            code: "source_digest_mismatch".into()
        }
    );
}

#[test]
fn worker_propagates_output_failure_and_rejects_bad_response_frames() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "closed",
            ))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let source = b"fn selected() {}";
    let mut input = Vec::new();
    write_request(&mut input, &request(source), source).unwrap();
    assert_eq!(
        run_worker(&mut Cursor::new(input), &mut Broken)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::BrokenPipe
    );
    let mut truncated = 10u32.to_be_bytes().to_vec();
    truncated.extend_from_slice(b"{}");
    assert!(read_response(&mut Cursor::new(truncated), 100).is_err());
    assert!(read_response(&mut Cursor::new(Vec::new()), u64::MAX).is_err());
}
