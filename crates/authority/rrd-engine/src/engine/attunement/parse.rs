//! Durable, resumable Parse. Each Step parses a bounded batch of the Inventory
//! files that have no facts for their current content and commits those facts
//! with the job transition, so a restart resumes where the last commit left off
//! and an edit re-parses only the edited file.

use super::inventory::{load_parse_inventory, Work};
use super::*;
use crate::engine::transaction_admission::TransactionPermit;
use rrd_attunement::inventory::{Entry, EntryKind, Proposal};
use rrd_attunement::parse::{parser_revision, ParseLimits, SUPPORTED_LANGUAGES};
use std::collections::BTreeMap;

#[cfg(any(windows, target_os = "linux"))]
mod host;
mod records;

use records::{FileState, ParseCommit, Status};

/// Target size of one chunk's text. Small enough to embed and to rank
/// precisely, large enough to hold a typical function with its context.
const CHUNK_BYTES: u64 = 1_500;
/// Share of the commit bound one Step spends on source; facts about the source
/// and the text of its chunks roughly triple what is read.
const INPUT_SHARE: u64 = 3;
/// A physical write batch is capped at 16 MiB and carries more than the logical
/// JSON a commit is measured by, so a Step targets half of the policy's bound.
const COMMIT_SHARE: u64 = 2;

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

struct Candidate<'a> {
    entry: &'a Entry,
    language: String,
    content_sha256: &'a str,
    bytes: u64,
}

/// Inventory files the policy binds to a grammar this engine links.
fn candidates<'a>(proposal: &'a Proposal, policy: &SourceParsePolicy) -> Vec<Candidate<'a>> {
    let bindings: BTreeMap<&str, &str> = policy
        .bindings
        .iter()
        .filter(|binding| SUPPORTED_LANGUAGES.contains(&binding.language.as_str()))
        .map(|binding| (binding.extension.as_str(), binding.language.as_str()))
        .collect();
    proposal
        .entries()
        .iter()
        .filter_map(|entry| {
            let EntryKind::File {
                observation,
                content_sha256,
            } = &entry.kind
            else {
                return None;
            };
            let extension = entry.path.rsplit_once('.')?.1;
            let language = *bindings.get(extension)?;
            Some(Candidate {
                entry,
                language: language.to_owned(),
                content_sha256,
                bytes: observation.bytes,
            })
        })
        .collect()
}

fn is_current(state: &FileState, candidate: &Candidate<'_>) -> bool {
    state.content_sha256 == candidate.content_sha256
        && state.parser_revision
            == parser_revision(&candidate.language).unwrap_or_default()
        && state.chunk_bytes == CHUNK_BYTES
}

pub(super) fn execute(
    engine: &RrdEngine,
    work: Work<'_>,
    permit: TransactionPermit<'_>,
) -> Result<AttunementControlResult> {
    let holder = worker(work.session)?;
    let invocation = engine.invocation_admission(
        work.context.request_id.as_str(),
        work.context.operation_id.as_str(),
    )?;
    let check = || {
        if let Some(invocation) = &invocation {
            invocation.check()?;
        }
        let at = elapsed_time(work.now, work.started)?;
        check_deadline(work.context, at)?;
        require_worker(&work.before, &holder, work.lease_generation, at)
    };
    check()?;
    let selected = work
        .plan
        .phases
        .get(work.before.checkpoints.len())
        .filter(|step| step.phase == AttunementPhase::Parse)
        .ok_or_else(|| contract("selected phase is not Parse"))?;
    let policy = selected
        .parse_policy
        .as_ref()
        .ok_or_else(|| contract("Parse step has no selected policy"))?;
    let root = engine
        .installed_project_root
        .as_ref()
        .ok_or(ServiceError::ProjectBindingMismatch)?;
    let mut remaining_keys = engine.estate_configuration.query.max_storage_keys;
    let proposal = load_parse_inventory(engine, &work.before, &work.plan, &mut remaining_keys)?;
    let all = candidates(&proposal, policy);
    if all.len() as u64 > policy.max_files {
        return Err(ServiceError::ConfigurationLimit(
            "project has more parseable files than the selected Parse policy admits".into(),
        ));
    }
    let scope = ScopeId::new(format!("instance:{}", engine.instance)).map_err(contract)?;
    let read = engine.storage.runtime().read_stamp(&scope)?;
    let mut states = records::load_states(engine, &read, &mut remaining_keys)?;

    let wanted: std::collections::BTreeSet<&str> =
        all.iter().map(|file| file.entry.path.as_str()).collect();
    let stale: Vec<FileState> = states
        .values()
        .filter(|state| !wanted.contains(state.path.as_str()))
        .cloned()
        .collect();
    let pending: Vec<&Candidate<'_>> = all
        .iter()
        .filter(|file| {
            states
                .get(&file.entry.path)
                .is_none_or(|state| !is_current(state, file))
        })
        .collect();

    let commit_limit = policy
        .max_commit_bytes
        .min(
            engine
                .estate_configuration
                .storage
                .transaction_admission_bytes,
        )
        / COMMIT_SHARE;
    let input_limit = commit_limit / INPUT_SHARE;
    let limits = ParseLimits {
        input_bytes: policy.max_file_bytes,
        nodes: policy.max_nodes,
        depth: policy.max_depth,
        output_bytes: policy.max_output_bytes,
        elapsed_micros: policy.max_elapsed_ms.saturating_mul(1_000),
    };

    let mut parse = ParseCommit::new(engine, &scope, &read, commit_limit)?;
    for state in &stale {
        parse.retire_file(state, elapsed_time(work.now, work.started)?)?;
        states.remove(&state.path);
    }

    // Too-large files are decided without being read.
    let mut selected_files: Vec<&Candidate<'_>> = Vec::new();
    let mut input_bytes = 0u64;
    let mut remaining_input = policy.max_input_bytes;
    for file in &pending {
        if file.bytes > policy.max_file_bytes {
            let at = elapsed_time(work.now, work.started)?;
            let state = parse.reject(file.entry, file.content_sha256, &file.language, "file_too_large", CHUNK_BYTES, states.get(&file.entry.path), at)?;
            states.insert(state.path.clone(), state);
            continue;
        }
        if !selected_files.is_empty() && input_bytes.saturating_add(file.bytes) > input_limit {
            break;
        }
        input_bytes = input_bytes.saturating_add(file.bytes);
        selected_files.push(*file);
    }

    if !selected_files.is_empty() {
        let mut sources = Vec::with_capacity(selected_files.len());
        for file in &selected_files {
            let mut bytes = Vec::with_capacity(usize::try_from(file.bytes).unwrap_or(0));
            super::super::installation::read_selected_source(
                root,
                &proposal,
                file.entry,
                policy.max_file_bytes,
                &mut remaining_input,
                &mut bytes,
                &check,
            )?;
            sources.push(bytes);
        }
        let requests: Vec<FileRequest<'_>> = selected_files
            .iter()
            .zip(&sources)
            .map(|(file, source)| FileRequest {
                source_id: &file.entry.path,
                content_sha256: file.content_sha256,
                language: &file.language,
                source,
            })
            .collect();
        let outcomes = run_batch(&requests, &limits, policy.max_memory_bytes, &check)?;
        for ((file, source), outcome) in selected_files.iter().zip(&sources).zip(outcomes) {
            let at = elapsed_time(work.now, work.started)?;
            let previous = states.get(&file.entry.path);
            let state = match outcome {
                FileOutcome::Parsed(facts) => parse.publish(file.entry, source, &facts, CHUNK_BYTES, previous, at)?,
                FileOutcome::Rejected(code) => parse.reject(file.entry, file.content_sha256, &file.language, &code, CHUNK_BYTES, previous, at)?,
                FileOutcome::Lost(code) => parse.reject(file.entry, file.content_sha256, &file.language, &code, CHUNK_BYTES, previous, at)?,
            };
            let Some(state) = state.admitted() else {
                break;
            };
            states.insert(state.path.clone(), state);
        }
    }

    let finished = all.iter().all(|file| {
        states
            .get(&file.entry.path)
            .is_some_and(|state| is_current(state, file))
    });
    parse.commit(&work, permit, states, finished, &check)
}

impl Status {
    pub(super) fn parsed(&self) -> bool {
        matches!(self, Status::Parsed)
    }
}
