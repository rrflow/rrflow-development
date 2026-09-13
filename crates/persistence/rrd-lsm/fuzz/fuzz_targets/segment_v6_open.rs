#![no_main]

use libfuzzer_sys::fuzz_target;
use rrd_lsm::Segment;

const MAX_INPUT_BYTES: usize = 128;
const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../fixtures/segment-v6.hex"
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
    match mode % 6 {
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
        4 => mutate_authenticated_metadata(candidate, payload),
        5 => mutate_authenticated_page(candidate, payload),
        _ => unreachable!(),
    }
}

fn mutate_authenticated_metadata(candidate: &mut Vec<u8>, payload: &[u8]) {
    let Some(descriptor) = first_page_descriptor(candidate) else {
        return;
    };
    let fields = [
        12,
        176,
        177,
        178,
        179,
        180,
        187,
        descriptor + 5,
        descriptor + 6,
        descriptor + 7,
        descriptor + 32,
        descriptor + 40,
    ];
    let selector = payload.first().copied().unwrap_or(0) as usize;
    let offset = fields[selector % fields.len()];
    let replacement = payload.get(1).copied().unwrap_or(0xff);
    if let Some(byte) = candidate.get_mut(offset) {
        *byte ^= replacement;
        rewrite_footer(candidate);
    }
}

fn mutate_authenticated_page(candidate: &mut Vec<u8>, payload: &[u8]) {
    let Some(first) = first_page_descriptor(candidate) else {
        return;
    };
    let page = payload.first().copied().unwrap_or(0) as usize % 6;
    let descriptor = first + page * 96;
    let Some(offset) = read_u64(candidate, descriptor + 24).and_then(|value| value.try_into().ok())
    else {
        return;
    };
    let Some(length) = read_u64(candidate, descriptor + 32).and_then(|value| value.try_into().ok())
    else {
        return;
    };
    if length == 0 {
        return;
    }
    let selected = payload.get(1).copied().unwrap_or(0) as usize % length;
    let Some(byte) = candidate.get_mut(offset + selected) else {
        return;
    };
    *byte ^= payload.get(2).copied().unwrap_or(0x80);
    if payload.get(3).copied().unwrap_or(0).is_multiple_of(2) {
        rewrite_page_digest(candidate, descriptor, offset, length);
    }
    rewrite_footer(candidate);
}

fn first_page_descriptor(candidate: &[u8]) -> Option<usize> {
    let index_offset: usize = read_u64(candidate, 48)?.try_into().ok()?;
    let group = index_offset.checked_add(32)?;
    let first_key_bytes: usize = read_u32(candidate, group.checked_add(12)?)?
        .try_into()
        .ok()?;
    let last_key_bytes: usize = read_u32(candidate, group.checked_add(16)?)?
        .try_into()
        .ok()?;
    let descriptor = group
        .checked_add(32)?
        .checked_add(first_key_bytes)?
        .checked_add(last_key_bytes)?;
    (descriptor.checked_add(6 * 96)? <= candidate.len()).then_some(descriptor)
}

fn rewrite_page_digest(candidate: &mut [u8], descriptor: usize, offset: usize, length: usize) {
    let Some(page) = candidate.get(offset..offset.saturating_add(length)) else {
        return;
    };
    let digest = ring::digest::digest(&ring::digest::SHA256, page);
    let Some(field) = candidate.get_mut(descriptor + 64..descriptor + 96) else {
        return;
    };
    field.copy_from_slice(digest.as_ref());
}

fn rewrite_footer(candidate: &mut Vec<u8>) {
    let Some(content_length) = candidate.len().checked_sub(64) else {
        return;
    };
    let digest = ring::digest::digest(&ring::digest::SHA256, &candidate[..content_length]);
    let mut encoded = [0u8; 64];
    for (index, byte) in digest.as_ref().iter().copied().enumerate() {
        encoded[index * 2] = hex_digit(byte >> 4);
        encoded[index * 2 + 1] = hex_digit(byte & 0x0f);
    }
    candidate[content_length..].copy_from_slice(&encoded);
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?,
    ))
}

fn hex_digit(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        10..=15 => b'a' + value - 10,
        _ => unreachable!("nibble is bounded"),
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
