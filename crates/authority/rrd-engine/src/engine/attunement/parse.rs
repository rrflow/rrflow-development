//! Durable, resumable Parse. Each Step parses a bounded batch of the Inventory
//! files that have no facts for their current content and commits those facts
//! with the job transition, so a restart resumes where the last commit left off
//! and an edit re-parses only the edited file.

// The executor that drives these pieces lands in the next pull request.
#![allow(dead_code)]

use super::*;
use rrd_attunement::parse::ParseLimits;

#[cfg(all(not(test), any(windows, target_os = "linux")))]
mod host;
mod records;

/// Target size of one chunk's text. Small enough to embed and to rank
/// precisely, large enough to hold a typical function with its context.
const CHUNK_BYTES: u64 = 1_500;
/// Share of the commit bound one Step spends on source; facts about the source
/// and the text of its chunks roughly triple what is read.
const INPUT_SHARE: u64 = 3;
/// A physical write batch is capped at 16 MiB and carries about twice the
/// logical JSON a commit is measured by, so a Step targets a quarter of the
/// policy's bound.
const COMMIT_SHARE: u64 = 4;
/// Physical write conflicts with background work retried before reporting one.
const MAX_PHYSICAL_RETRIES: u32 = 3;
/// Smallest batch a refused commit is rebuilt at before the refusal stands.
const MIN_COMMIT_BYTES: u64 = 64 * 1024;

pub(super) struct FileRequest<'a> {
    pub source_id: &'a str,
    pub content_sha256: &'a str,
    pub language: &'a str,
    pub source: &'a [u8],
}

pub(super) enum FileOutcome {
    Parsed(rrd_attunement::facts::SourceFacts),
    /// The parser refused this file with a fixed reason; retrying cannot help.
    Rejected(String),
    /// The worker died or timed out while holding this file.
    #[cfg_attr(test, allow(dead_code))]
    Lost(String),
}

/// Runs the batch in the supervised worker. Stops early after a lost worker, so
/// fewer outcomes than requests means the rest are still pending.
#[cfg(all(not(test), any(windows, target_os = "linux")))]
fn run_batch(
    requests: &[FileRequest<'_>],
    limits: &ParseLimits,
    memory_bytes: u64,
    check: &dyn Fn() -> Result<()>,
) -> Result<Vec<FileOutcome>> {
    host::run_batch(requests, limits, CHUNK_BYTES, memory_bytes, check)
}

#[cfg(all(not(test), not(any(windows, target_os = "linux"))))]
fn run_batch(
    _: &[FileRequest<'_>],
    _: &ParseLimits,
    _: u64,
    _: &dyn Fn() -> Result<()>,
) -> Result<Vec<FileOutcome>> {
    Err(ServiceError::ConfigurationLimit(
        "this platform has no supervised parser; Parse is not an available capability".into(),
    ))
}

/// Unit tests run the parser in process: they exercise publication and resume,
/// and the supervised path has its own tests against the real binary.
#[cfg(test)]
fn run_batch(
    requests: &[FileRequest<'_>],
    limits: &ParseLimits,
    _: u64,
    check: &dyn Fn() -> Result<()>,
) -> Result<Vec<FileOutcome>> {
    requests
        .iter()
        .map(|request| {
            check()?;
            Ok(
                match rrd_attunement::facts::source_facts(
                    request.source_id,
                    request.content_sha256,
                    request.language,
                    request.source,
                    limits,
                    CHUNK_BYTES,
                    || false,
                ) {
                    Ok(facts) => FileOutcome::Parsed(facts),
                    Err(_) => FileOutcome::Rejected("rejected".into()),
                },
            )
        })
        .collect()
}
