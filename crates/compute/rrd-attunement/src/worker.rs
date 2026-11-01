//! Private framed protocol for a single isolated parser invocation.
//!
//! This protocol does not provide isolation. The engine must start this worker
//! under an enforced native process memory limit and deadline, validate the
//! selected source, and discard its output on any failed source recheck.
//! Neither a successful frame nor a parsed proposal authorizes publication.

use crate::parse::{
    parse_source, parser_revision, ParseError, ParseLimits, ParseProposal, PARSE_PROPOSAL_VERSION,
};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

/// Bound protocol metadata independently of the configurable source budget.
const MAX_HEADER_BYTES: u32 = 16 * 1024;
/// JSON response envelope and fixed rejection codes add only bounded metadata
/// around the parser's separately accounted output.
const RESPONSE_OVERHEAD_BYTES: u64 = 1024;
pub const WORKER_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRequest {
    pub format_version: u16,
    pub source_id: String,
    pub content_sha256: String,
    pub language: String,
    pub source_bytes: u64,
    pub limits: ParseLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerResponse {
    Parsed { proposal: ParseProposal },
    Rejected { code: String },
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn frame<R: Read>(reader: &mut R, limit: u64) -> io::Result<Vec<u8>> {
    let mut prefix = [0; 4];
    reader.read_exact(&mut prefix)?;
    let length = u64::from(u32::from_be_bytes(prefix));
    if length == 0 || length > limit {
        return Err(invalid("invalid worker frame length"));
    }
    let size = usize::try_from(length).map_err(|_| invalid("worker frame too large"))?;
    let mut bytes = vec![0; size];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_frame<W: Write>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
    let length = u32::try_from(bytes.len()).map_err(|_| invalid("worker frame too large"))?;
    if length == 0 {
        return Err(invalid("empty worker frame"));
    }
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(bytes)
}

fn validate_request(request: &WorkerRequest) -> io::Result<()> {
    if request.format_version != WORKER_PROTOCOL_VERSION
        || request.source_id.is_empty()
        || request.source_id.len() > 256
        || request.source_id.as_bytes().contains(&0)
        || request.content_sha256.len() != 64
        || !request
            .content_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || request.limits.input_bytes == 0
        || request.limits.nodes == 0
        || request.limits.depth == 0
        || request.limits.output_bytes == 0
        || request.limits.elapsed_micros == 0
        || request.source_bytes > request.limits.input_bytes
        || request.source_bytes > u64::from(u32::MAX)
    {
        return Err(invalid("invalid parser worker request"));
    }
    Ok(())
}

/// Send only the bounded metadata. The engine can then stream the selected
/// source from its anchored reader without buffering the entire file.
pub fn write_request_header<W: Write>(writer: &mut W, request: &WorkerRequest) -> io::Result<()> {
    validate_request(request)?;
    let header = serde_json::to_vec(request).map_err(|_| invalid("invalid worker header"))?;
    if header.len() > MAX_HEADER_BYTES as usize {
        return Err(invalid("worker header too large"));
    }
    write_frame(writer, &header)
}

/// Encode metadata followed by the exact source body. The caller must not use
/// this convenience function in place of the engine's anchored source reader.
pub fn write_request<W: Write>(
    writer: &mut W,
    request: &WorkerRequest,
    source: &[u8],
) -> io::Result<()> {
    if source.len() as u64 != request.source_bytes {
        return Err(invalid("parser worker source length mismatch"));
    }
    write_request_header(writer, request)?;
    writer.write_all(source)
}

/// Read one response, refusing a frame larger than the admitted output budget
/// plus bounded protocol overhead before allocating its body.
pub fn read_response<R: Read>(
    reader: &mut R,
    request: &WorkerRequest,
) -> io::Result<WorkerResponse> {
    validate_request(request)?;
    let max = request
        .limits
        .output_bytes
        .checked_add(RESPONSE_OVERHEAD_BYTES)
        .ok_or_else(|| invalid("worker response limit overflow"))?;
    let bytes = frame(reader, max)?;
    let response =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid worker response"))?;
    validate_response(request, &response)?;
    Ok(response)
}

/// Check a decoded, bounded response against the exact admitted request.
/// This is not authorization, source freshness, or proof of parser correctness.
pub fn validate_response(request: &WorkerRequest, response: &WorkerResponse) -> io::Result<()> {
    validate_request(request)?;
    match response {
        WorkerResponse::Parsed { proposal } => {
            if proposal.format_version != PARSE_PROPOSAL_VERSION
                || proposal.source_id != request.source_id
                || proposal.content_sha256 != request.content_sha256
                || proposal.language != request.language
                || parser_revision(&request.language) != Some(proposal.parser_revision.as_str())
                || proposal.nodes.len() as u64 > request.limits.nodes
                || proposal.nodes.iter().any(|node| {
                    node.kind.is_empty()
                        || node.start_byte > node.end_byte
                        || node.end_byte > request.source_bytes
                })
            {
                return Err(invalid("parser response does not match request"));
            }
            let bytes =
                serde_json::to_vec(proposal).map_err(|_| invalid("invalid parser proposal"))?;
            if bytes.len() as u64 > request.limits.output_bytes {
                return Err(invalid("parser proposal exceeds output limit"));
            }
            Ok(())
        }
        WorkerResponse::Rejected { code } => {
            if matches!(
                code.as_str(),
                "unsupported_language"
                    | "source_digest_mismatch"
                    | "invalid_source_identity"
                    | "invalid_encoding"
                    | "limit_exceeded"
                    | "cancelled"
                    | "parser_unavailable"
            ) {
                Ok(())
            } else {
                Err(invalid("unknown parser rejection"))
            }
        }
    }
}

/// Handle one framed request on supplied pipes. The child opens no source path
/// and writes no durable state. Invalid or truncated framing returns an error
/// without a response; parse refusals return only a fixed code.
pub fn run_worker<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let header = frame(reader, u64::from(MAX_HEADER_BYTES))?;
    let request: WorkerRequest =
        serde_json::from_slice(&header).map_err(|_| invalid("invalid worker header"))?;
    validate_request(&request)?;
    let size = usize::try_from(request.source_bytes).map_err(|_| invalid("source too large"))?;
    let mut source = vec![0; size];
    reader.read_exact(&mut source)?;
    let response = match parse_source(
        &request.source_id,
        &request.content_sha256,
        &request.language,
        &source,
        &request.limits,
        || false,
    ) {
        Ok(proposal) => WorkerResponse::Parsed { proposal },
        Err(error) => WorkerResponse::Rejected {
            code: error_code(error).into(),
        },
    };
    let bytes =
        serde_json::to_vec(&response).map_err(|_| invalid("cannot encode worker response"))?;
    let limit = request
        .limits
        .output_bytes
        .checked_add(RESPONSE_OVERHEAD_BYTES)
        .ok_or_else(|| invalid("worker response limit overflow"))?;
    if bytes.len() as u64 > limit {
        return Err(invalid("worker response exceeds limit"));
    }
    write_frame(writer, &bytes)?;
    writer.flush()
}

fn error_code(error: ParseError) -> &'static str {
    match error {
        ParseError::UnsupportedLanguage => "unsupported_language",
        ParseError::SourceDigestMismatch => "source_digest_mismatch",
        ParseError::InvalidSourceIdentity => "invalid_source_identity",
        ParseError::InvalidEncoding => "invalid_encoding",
        ParseError::LimitExceeded => "limit_exceeded",
        ParseError::Cancelled => "cancelled",
        ParseError::ParserUnavailable => "parser_unavailable",
    }
}
