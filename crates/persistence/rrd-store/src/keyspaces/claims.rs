//! Claim, sequence, access, invocation and function keys.

use super::*;

pub(crate) fn claim_key(
    subject: &Subject,
    predicate: &Predicate,
    valid_from: Millis,
    tx_time: Millis,
) -> Vec<u8> {
    encode(
        CLAIMS,
        &[
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
            KeyPart::DescU64(valid_from),
            KeyPart::DescU64(tx_time),
        ],
    )
}

pub(crate) fn claim_subject_prefix(subject: &Subject) -> Vec<u8> {
    encode(CLAIMS, &[KeyPart::Text(subject.as_str())])
}

pub(crate) fn claim_version_prefix(subject: &Subject, predicate: &Predicate) -> Vec<u8> {
    encode(
        CLAIMS,
        &[
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
        ],
    )
}

pub(crate) fn claim_seek_key(subject: &Subject, predicate: &Predicate, as_of: Millis) -> Vec<u8> {
    encode(
        CLAIMS,
        &[
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
            KeyPart::DescU64(as_of),
        ],
    )
}

pub(crate) fn parse_claim_key(
    codec: KeyCodec,
    stored: &[u8],
) -> Result<(Subject, Predicate, Millis, Millis)> {
    let [DecodedKeyPart::Text(subject), DecodedKeyPart::Text(predicate), DecodedKeyPart::DescU64(valid_from), DecodedKeyPart::DescU64(tx_time)] =
        expect_parts(codec, CLAIMS, stored)?
    else {
        return Err(Error::Codec("claim key has invalid field types".into()));
    };
    Ok((
        Subject::new(subject)?,
        Predicate::new(predicate)?,
        valid_from,
        tx_time,
    ))
}

pub(crate) fn sequence_key(sequence: u64) -> Vec<u8> {
    encode(SEQUENCE_INDEX, &[KeyPart::U64(sequence)])
}

pub(crate) fn parse_sequence_key(codec: KeyCodec, stored: &[u8]) -> Result<u64> {
    let [DecodedKeyPart::U64(sequence)] = expect_parts(codec, SEQUENCE_INDEX, stored)? else {
        return Err(Error::Codec(
            "claim sequence key has invalid field types".into(),
        ));
    };
    Ok(sequence)
}

pub(crate) fn access_key(
    at: Millis,
    reader: &Reader,
    subject: &Subject,
    predicate: &Predicate,
) -> Vec<u8> {
    encode(
        ACCESS,
        &[
            KeyPart::U64(at),
            KeyPart::Text(reader.as_str()),
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
        ],
    )
}

pub(crate) fn access_bound(at: Millis) -> Vec<u8> {
    encode(ACCESS, &[KeyPart::U64(at)])
}

pub(crate) fn parse_access_key(
    codec: KeyCodec,
    stored: &[u8],
) -> Result<(Millis, Reader, Subject, Predicate)> {
    let [DecodedKeyPart::U64(at), DecodedKeyPart::Text(reader), DecodedKeyPart::Text(subject), DecodedKeyPart::Text(predicate)] =
        expect_parts(codec, ACCESS, stored)?
    else {
        return Err(Error::Codec("access key has invalid field types".into()));
    };
    Ok((
        at,
        Reader::new(reader)?,
        Subject::new(subject)?,
        Predicate::new(predicate)?,
    ))
}

pub(crate) fn invocation_key(at: Millis, ordinal: u64) -> Vec<u8> {
    encode(INVOCATIONS, &[KeyPart::U64(at), KeyPart::U64(ordinal)])
}

pub(crate) fn invocation_bound(at: Millis) -> Vec<u8> {
    encode(INVOCATIONS, &[KeyPart::U64(at)])
}

pub(crate) fn function_artifact_key(instance: &str, content_sha256: &str) -> Result<Vec<u8>> {
    let digest = decode_sha256(content_sha256)?;
    Ok(encode(
        FUNCTION_ARTIFACTS,
        &[KeyPart::Text(instance), KeyPart::Bytes(&digest)],
    ))
}

pub(crate) fn function_definition_key(
    instance: &str,
    function_id: &str,
    definition_sha256: &str,
) -> Result<Vec<u8>> {
    let digest = decode_sha256(definition_sha256)?;
    Ok(encode(
        FUNCTION_DEFINITIONS,
        &[
            KeyPart::Text(instance),
            KeyPart::Text(function_id),
            KeyPart::Bytes(&digest),
        ],
    ))
}

pub(crate) fn transaction_function_binding_key(
    instance: &str,
    binding_id: &str,
    binding_sha256: &str,
) -> Result<Vec<u8>> {
    let digest = decode_sha256(binding_sha256)?;
    Ok(encode(
        TRANSACTION_FUNCTION_BINDINGS,
        &[
            KeyPart::Text(instance),
            KeyPart::Text(binding_id),
            KeyPart::Bytes(&digest),
        ],
    ))
}

pub(crate) fn function_catalogue_membership_key(instance: &str, revision: u64) -> Vec<u8> {
    encode(
        FUNCTION_CATALOGUE_MEMBERSHIPS,
        &[KeyPart::Text(instance), KeyPart::U64(revision)],
    )
}

pub(crate) fn function_catalogue_head_key(instance: &str) -> Vec<u8> {
    encode(FUNCTION_CATALOGUE_HEADS, &[KeyPart::Text(instance)])
}

pub(crate) fn function_invocation_receipt_key(instance: &str, invocation_id: &str) -> Vec<u8> {
    encode(
        INVOCATIONS,
        &[KeyPart::Text(instance), KeyPart::Text(invocation_id)],
    )
}

pub(super) fn decode_sha256(value: &str) -> Result<[u8; 32]> {
    if value.len() != 64 {
        return Err(Error::Codec(
            "SHA-256 key coordinate has invalid length".into(),
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        digest[index] = (decode_hex_digit(pair[0])? << 4) | decode_hex_digit(pair[1])?;
    }
    Ok(digest)
}

pub(super) fn decode_hex_digit(value: u8) -> Result<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(Error::Codec(
            "SHA-256 key coordinate is not lowercase hexadecimal".into(),
        )),
    }
}
