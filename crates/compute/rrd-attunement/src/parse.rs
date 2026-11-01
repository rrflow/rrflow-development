//! Bounded, pure syntax proposals over bytes supplied by the engine.
//!
//! This module neither opens a project file nor establishes that a source is
//! authorized or committed. The caller must bind the supplied source identity
//! and digest to an accepted Inventory checkpoint before publishing anything.

use rrd_core::digest::sha256_hex;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tree_sitter::{Language, Parser};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParseLimits {
    pub input_bytes: u64,
    /// Maximum syntax nodes visited, including nodes omitted from the output.
    pub nodes: u64,
    pub depth: u64,
    pub output_bytes: u64,
    pub elapsed_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnsupportedLanguage,
    SourceDigestMismatch,
    InvalidSourceIdentity,
    InvalidEncoding,
    LimitExceeded,
    Cancelled,
    ParserUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub error: bool,
    pub missing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseProposal {
    pub format_version: u16,
    pub source_id: String,
    pub content_sha256: String,
    pub language: String,
    pub parser_revision: String,
    pub nodes: Vec<SyntaxNode>,
}

/// The selected language is explicit; this function never guesses from a path.
/// Grammars are linked build inputs and are not fetched during execution.
pub fn parse_source(
    source_id: &str,
    expected_sha256: &str,
    language: &str,
    source: &[u8],
    limits: &ParseLimits,
    cancelled: impl Fn() -> bool,
) -> Result<ParseProposal, ParseError> {
    let started = Instant::now();
    if source_id.is_empty() || source_id.len() > 256 {
        return Err(ParseError::InvalidSourceIdentity);
    }
    if limits.input_bytes == 0
        || limits.nodes == 0
        || limits.depth == 0
        || limits.output_bytes == 0
        || limits.elapsed_micros == 0
        || source.len() as u64 > limits.input_bytes
        || source.len() > u32::MAX as usize
    {
        return Err(ParseError::LimitExceeded);
    }
    if cancelled() {
        return Err(ParseError::Cancelled);
    }
    if sha256_hex(source) != expected_sha256 {
        return Err(ParseError::SourceDigestMismatch);
    }
    std::str::from_utf8(source).map_err(|_| ParseError::InvalidEncoding)?;
    let (grammar, revision): (Language, &str) = match language {
        "rust" => (
            tree_sitter_rust::LANGUAGE.into(),
            "tree-sitter-0.24.7/rust-0.23.3",
        ),
        "python" => (
            tree_sitter_python::LANGUAGE.into(),
            "tree-sitter-0.24.7/python-0.23.6",
        ),
        "go" => (
            tree_sitter_go::LANGUAGE.into(),
            "tree-sitter-0.24.7/go-0.23.4",
        ),
        _ => return Err(ParseError::UnsupportedLanguage),
    };
    let deadline = Duration::from_micros(limits.elapsed_micros);
    if started.elapsed() >= deadline {
        return Err(ParseError::LimitExceeded);
    }
    let mut parser = Parser::new();
    parser
        .set_language(&grammar)
        .map_err(|_| ParseError::ParserUnavailable)?;
    let remaining = deadline.saturating_sub(started.elapsed()).as_micros();
    if remaining == 0 {
        return Err(ParseError::LimitExceeded);
    }
    parser.set_timeout_micros(u64::try_from(remaining).unwrap_or(u64::MAX));
    let mut stopped = None;
    let tree = parser.parse_with(
        &mut |offset, _| {
            if stopped.is_some() {
                &source[0..0]
            } else {
                // Tree-sitter invokes this closure from a C callback. Do not
                // allow an operator-supplied cancellation check to unwind
                // across the FFI boundary.
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(&cancelled)) {
                    Ok(true) => {
                        stopped = Some(ParseError::Cancelled);
                        &source[0..0]
                    }
                    Err(_) => {
                        stopped = Some(ParseError::ParserUnavailable);
                        &source[0..0]
                    }
                    Ok(false) if started.elapsed() >= deadline => {
                        stopped = Some(ParseError::LimitExceeded);
                        &source[0..0]
                    }
                    Ok(false) => source.get(offset..).unwrap_or_default(),
                }
            }
        },
        None,
    );
    if cancelled() {
        return Err(ParseError::Cancelled);
    }
    if let Some(reason) = stopped {
        return Err(reason);
    }
    if started.elapsed() >= deadline {
        return Err(ParseError::LimitExceeded);
    }
    let tree = tree.ok_or(ParseError::LimitExceeded)?;
    let mut cursor = tree.walk();
    let mut depth = 1u64;
    let mut visited = 0u64;
    let mut output = ParseProposal {
        format_version: 1,
        source_id: source_id.into(),
        content_sha256: expected_sha256.into(),
        language: language.into(),
        parser_revision: revision.into(),
        nodes: Vec::new(),
    };
    let mut output_bytes = serde_json::to_vec(&output)
        .map_err(|_| ParseError::LimitExceeded)?
        .len() as u64;
    if output_bytes > limits.output_bytes {
        return Err(ParseError::LimitExceeded);
    }
    loop {
        if cancelled() {
            return Err(ParseError::Cancelled);
        }
        if started.elapsed() >= deadline || depth > limits.depth {
            return Err(ParseError::LimitExceeded);
        }
        visited = visited.checked_add(1).ok_or(ParseError::LimitExceeded)?;
        if visited > limits.nodes {
            return Err(ParseError::LimitExceeded);
        }
        let node = cursor.node();
        if node.is_named() || node.is_error() || node.is_missing() {
            let start = node.start_byte();
            let end = node.end_byte();
            if start > end || end > source.len() {
                return Err(ParseError::ParserUnavailable);
            }
            let proposed = SyntaxNode {
                kind: node.kind().into(),
                start_byte: start as u64,
                end_byte: end as u64,
                error: node.is_error(),
                missing: node.is_missing(),
            };
            let node_bytes = serde_json::to_vec(&proposed)
                .map_err(|_| ParseError::LimitExceeded)?
                .len() as u64;
            output_bytes = output_bytes
                .checked_add(node_bytes)
                .and_then(|bytes| bytes.checked_add(u64::from(!output.nodes.is_empty())))
                .ok_or(ParseError::LimitExceeded)?;
            if output_bytes > limits.output_bytes {
                return Err(ParseError::LimitExceeded);
            }
            output.nodes.push(proposed);
        }
        if cursor.goto_first_child() {
            depth = depth.checked_add(1).ok_or(ParseError::LimitExceeded)?;
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                if cancelled() {
                    return Err(ParseError::Cancelled);
                }
                if started.elapsed() >= deadline {
                    return Err(ParseError::LimitExceeded);
                }
                return Ok(output);
            }
            depth -= 1;
        }
    }
}
