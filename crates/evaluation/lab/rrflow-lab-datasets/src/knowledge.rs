//! A repository's text, chunked by this harness so RRFlow's context tool has something to
//! search. RRFlow cannot do this itself yet, because attunement stops before reading content.
//! So results over this text measure lexical and graph retrieval, and not attunement,
//! parsing or embeddings. Every report using it says so.
use rrflow_evaluation_core::{sha256_hex, Result};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

pub const CHUNK_TABLE: &str = "lab_chunk";
pub const MAX_CHUNK_BYTES: usize = 2_048;
pub const MAX_FILE_BYTES: usize = 1024 * 1024;
const VALID_FROM: u64 = 100;
/// Lock files hold no reasoning, and a test's own answers must never be searchable.
const LEFT_OUT: [&str; 3] = [".lock", "eval/lab/tasks/", "eval/results/"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChunk {
    pub path: String,
    pub ordinal: u64,
    pub text: String,
}

impl TextChunk {
    /// Depends only on where the text sits, so a repeated import replaces rather than adds.
    pub fn reference_id(&self) -> String {
        let digest = sha256_hex(format!("{}\0{}", self.path, self.ordinal).as_bytes());
        format!("c{}", &digest[..32])
    }

    /// One public `put_record` mutation.
    pub fn mutation(&self) -> Value {
        json!({
            "mutation": "put_record",
            "reference": {"kind": CHUNK_TABLE, "id": self.reference_id()},
            "valid_from": VALID_FROM,
            "properties": {
                "path": {"type": "string", "value": self.path},
                "ordinal": {"type": "unsigned", "value": self.ordinal},
                "text": {"type": "string", "value": self.text}
            }
        })
    }
}

/// Whether a tracked file is worth searching as text.
pub fn importable(path: &str, bytes: &[u8]) -> bool {
    !LEFT_OUT.iter().any(|part| path.contains(part))
        && !bytes.is_empty()
        && bytes.len() <= MAX_FILE_BYTES
        && !bytes.contains(&0)
        && std::str::from_utf8(bytes).is_ok()
}

/// Splits on line boundaries, and inside a line only when one line is longer than a chunk.
pub fn chunk_text(path: &str, text: &str) -> Vec<TextChunk> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in text.split_inclusive('\n') {
        let mut rest = line;
        while !rest.is_empty() {
            let room = MAX_CHUNK_BYTES - current.len();
            if rest.len() <= room {
                current.push_str(rest);
                break;
            }
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
                continue;
            }
            let mut end = room;
            while !rest.is_char_boundary(end) {
                end -= 1;
            }
            chunks.push(rest[..end].to_string());
            rest = &rest[end..];
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
        .into_iter()
        .filter(|chunk| !chunk.trim().is_empty())
        .enumerate()
        .map(|(ordinal, text)| TextChunk {
            path: path.to_string(),
            ordinal: ordinal as u64,
            text,
        })
        .collect()
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RepositoryText {
    pub chunks: Vec<TextChunk>,
    pub files: u64,
    pub files_left_out: u64,
}

/// Every tracked text file in a checkout. Git says what is tracked, so build output and
/// anything else outside the repository's own history cannot reach the search.
pub fn read_repository_text(checkout: &Path) -> Result<RepositoryText> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|error| format!("could not list the checkout's files: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "could not list the checkout's files: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut text = RepositoryText::default();
    for path in String::from_utf8_lossy(&output.stdout).split('\0') {
        if path.is_empty() {
            continue;
        }
        // A tracked submodule is a directory, and it holds nothing of this repository's text.
        let Ok(bytes) = std::fs::read(checkout.join(path)) else {
            text.files_left_out += 1;
            continue;
        };
        if !importable(path, &bytes) {
            text.files_left_out += 1;
            continue;
        }
        text.files += 1;
        let content = String::from_utf8_lossy(&bytes);
        text.chunks.extend(chunk_text(path, &content));
    }
    Ok(text)
}
