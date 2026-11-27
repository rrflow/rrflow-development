//! Protocol and digest validation shared by every module.

use crate::*;
use sha2::{Digest, Sha256};

pub fn validate_protocol(protocol: &str, version: u16) -> Result<()> {
    if protocol != PROTOCOL {
        return invalid(format!(
            "unsupported protocol {protocol:?}; expected {PROTOCOL:?}"
        ));
    }
    if version != PROTOCOL_VERSION {
        return invalid(format!(
            "unsupported protocol version {version}; expected {PROTOCOL_VERSION}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_sha256(value: &str, field: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return invalid(format!("{field} must be lowercase SHA-256 hex"));
    }
    Ok(())
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        output.push(HEX[usize::from(byte >> 4)] as char);
        output.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    output
}

pub(crate) fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(ContractError(message.into()))
}
