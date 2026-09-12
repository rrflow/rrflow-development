#![no_main]

use libfuzzer_sys::fuzz_target;
use rrd_lsm::Segment;

const MAX_INPUT_BYTES: usize = 128;
const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../fixtures/segment-v4.hex"
));

fuzz_target!(|input: &[u8]| {
    if input.len() > MAX_INPUT_BYTES {
        return;
    }

    let mut candidate = decode_fixture();
    mutate(&mut candidate, input);
    let directory = tempfile::tempdir().expect("temporary fuzz directory must be available");
    let path = directory.path().join("candidate.seg");
    std::fs::write(&path, candidate).expect("temporary segment write must succeed");

    if let Ok(segment) = Segment::open(&path) {
        let _ = segment.row_group_count();
        let _ = segment.open_evidence();
        segment
            .visible_versions(u64::MAX)
            .expect("an accepted segment must remain fully readable");
    }
});

fn decode_fixture() -> Vec<u8> {
    let digits = FIXTURE
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert!(digits.len().is_multiple_of(2), "fixture hex is incomplete");
    digits
        .chunks_exact(2)
        .map(|pair| (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]))
        .collect()
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("fixture contains a non-hex byte"),
    }
}

fn mutate(candidate: &mut Vec<u8>, input: &[u8]) {
    let Some((&mode, payload)) = input.split_first() else {
        return;
    };
    match mode % 4 {
        0 => {
            let retained = bounded_index(payload, candidate.len().saturating_add(1));
            candidate.truncate(retained);
        }
        1 => {
            for pair in payload.chunks(2).take(32) {
                let index = usize::from(pair[0]) % candidate.len();
                let mask = pair.get(1).copied().unwrap_or(0x80);
                candidate[index] ^= mask;
            }
        }
        2 => {
            if candidate.is_empty() {
                return;
            }
            let start = bounded_index(payload, candidate.len());
            let length = payload.get(1).copied().unwrap_or(1).clamp(1, 32) as usize;
            let replacement = payload.get(2).copied().unwrap_or(0);
            let end = start.saturating_add(length).min(candidate.len());
            candidate[start..end].fill(replacement);
        }
        3 => {
            for (index, byte) in payload.iter().copied().take(16).enumerate() {
                if index == candidate.len() {
                    break;
                }
                candidate[index] = byte;
            }
        }
        _ => unreachable!(),
    }
}

fn bounded_index(bytes: &[u8], upper: usize) -> usize {
    if upper == 0 {
        return 0;
    }
    bytes
        .iter()
        .take(std::mem::size_of::<usize>())
        .fold(0usize, |value, byte| {
            value.rotate_left(5) ^ usize::from(*byte)
        })
        % upper
}
