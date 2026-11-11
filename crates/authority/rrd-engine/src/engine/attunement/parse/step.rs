//! The Parse Step: parse a bounded batch of the Inventory files whose facts are
//! missing or stale, and commit those facts with the job's transition, so a
//! restart resumes from the last commit and an unchanged file is never parsed
//! twice.

use super::super::inventory::Work;
use super::publish::publish;
use super::records::{self, FileState, Outcome, Status};
use super::worker::{Answer, Worker};
use super::*;
use crate::engine::transaction_admission::TransactionPermit;
use rrd_attunement::inventory::{Entry, EntryKind};
use rrd_attunement::parse::{parser_revision, ParseLimits};
use rrd_attunement::worker::{WorkerRequest, WORKER_PROTOCOL_VERSION};
use rrd_contract::SourceParsePolicy;
use std::collections::BTreeMap;
use std::time::Duration;

/// AST chunk bound in bytes. Fixed until the parse policy carries it.
const CHUNK_BYTES: u64 = 1_500;
/// Workers started in one Step; files after the last loss wait for the next Step.
const MAX_WORKERS: u32 = 3;

#[cfg(test)]
use super::worker::in_process as start_worker;
#[cfg(not(test))]
use super::worker::start as start_worker;

/// A file the policy binds to a grammar, with the state it should reach.
struct Wanted<'a> {
    entry: &'a Entry,
    bytes: u64,
    state: FileState,
}

fn wanted<'a>(entries: &'a [Entry], policy: &SourceParsePolicy, job_id: &str) -> Vec<Wanted<'a>> {
    let bound: BTreeMap<&str, &str> = policy
        .bindings
        .iter()
        .map(|binding| (binding.extension.as_str(), binding.language.as_str()))
        .collect();
    entries
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
            let language = *bound.get(extension)?;
            let revision = parser_revision(language)?;
            Some(Wanted {
                entry,
                bytes: observation.bytes,
                state: FileState {
                    path: entry.path.clone(),
                    content_sha256: content_sha256.clone(),
                    language: language.into(),
                    parser_revision: revision.into(),
                    chunk_bytes: CHUNK_BYTES,
                    job_id: job_id.into(),
                    status: Status::Lost,
                    reason: String::new(),
                    facts_sha256: String::new(),
                    symbols: 0,
                    spans: 0,
                    chunks: 0,
                    nodes: 0,
                    error_nodes: 0,
                },
            })
        })
        .collect()
}

/// True when this job need not parse the file again: its content already has a
/// final answer, or the worker was lost on it in this job, which a later job retries.
fn settled(current: Option<&FileState>, wanted: &FileState) -> bool {
    current.is_some_and(|current| {
        current.content_sha256 == wanted.content_sha256
            && current.parser_revision == wanted.parser_revision
            && current.chunk_bytes == wanted.chunk_bytes
            && match current.status {
                Status::Parsed | Status::Rejected => true,
                Status::Lost => current.job_id == wanted.job_id,
            }
    })
}

fn encoded_len(value: &impl Serialize) -> Result<u64> {
    Ok(serde_json::to_vec(value).map_err(contract)?.len() as u64)
}

pub(in crate::engine::attunement) fn execute(
    engine: &RrdEngine,
    work: Work<'_>,
    mut permit: TransactionPermit<'_>,
) -> Result<AttunementControlResult> {
    let holder = super::super::worker(work.session)?;
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
    let step = work
        .plan
        .phases
        .get(work.before.checkpoints.len())
        .filter(|step| step.phase == AttunementPhase::Parse)
        .ok_or_else(|| contract("selected phase is not Parse"))?;
    let policy = step
        .parse_policy
        .as_ref()
        .ok_or_else(|| contract("the Parse step has no parse policy"))?;
    let mut remaining_keys = engine.estate_configuration.query.max_storage_keys;
    let proposal = load_parse_inventory(engine, &work.before, &work.plan, &mut remaining_keys)?;
    let root = engine
        .installed_project_root
        .as_ref()
        .ok_or(ServiceError::ProjectBindingMismatch)?;
    let scope = ScopeId::new(format!("instance:{}", engine.instance)).map_err(contract)?;
    let read = engine.storage.runtime().read_stamp(&scope)?;
    let mut states = records::load_states(engine, &read, &mut remaining_keys)?;
    let wanted = wanted(proposal.entries(), policy, work.before.id.as_str());
    let at = elapsed_time(work.now, work.started)?;
    let budget = policy.max_commit_bytes.min(
        engine
            .estate_configuration
            .storage
            .transaction_admission_bytes,
    );

    let mut mutations: Vec<RuntimeMutation> = records::schema_mutation(engine, &scope, &read)?
        .into_iter()
        .collect();
    let kept: std::collections::BTreeSet<&str> =
        wanted.iter().map(|file| file.state.path.as_str()).collect();
    for state in states
        .values()
        .filter(|state| !kept.contains(state.path.as_str()))
    {
        mutations.extend(records::retire_file(state, at)?);
    }
    let mut used = encoded_len(&mutations)?;
    let pending: Vec<&Wanted<'_>> = wanted
        .iter()
        .filter(|file| !settled(states.get(&file.state.path), &file.state))
        .collect();
    let mut remaining_input = policy.max_input_bytes;
    let mut worker: Option<Worker> = None;
    let mut started_workers = 0;
    let mut done = 0;
    let timeout = Duration::from_millis(policy.max_elapsed_ms);
    for file in &pending {
        if done as u64 >= policy.max_files {
            break;
        }
        check()?;
        let previous = states.get(&file.state.path);
        let next = file.state.clone();
        let mut source = Vec::new();
        let outcome_mutations = if file.bytes > policy.max_file_bytes {
            records::file_mutations(
                previous,
                next.clone(),
                Outcome::Rejected("file_too_large".into()),
                &source,
                at,
            )?
        } else {
            if file.bytes > remaining_input {
                break;
            }
            crate::engine::installation::read_selected_source(
                root,
                &proposal,
                file.entry,
                policy.max_file_bytes,
                &mut remaining_input,
                &mut source,
                &check,
            )?;
            if worker.is_none() {
                if started_workers >= MAX_WORKERS {
                    break;
                }
                started_workers += 1;
                worker = Some(start_worker(policy.max_memory_bytes).map_err(|error| {
                    ServiceError::Storage(format!("parse worker did not start: {error}"))
                })?);
            }
            let request = WorkerRequest {
                format_version: WORKER_PROTOCOL_VERSION,
                source_id: file.entry.identity.clone(),
                content_sha256: next.content_sha256.clone(),
                language: next.language.clone(),
                source_bytes: source.len() as u64,
                limits: ParseLimits {
                    input_bytes: policy.max_file_bytes,
                    nodes: policy.max_nodes,
                    depth: policy.max_depth,
                    output_bytes: policy.max_output_bytes,
                    elapsed_micros: policy.max_elapsed_ms.saturating_mul(1_000),
                },
                chunk_bytes: CHUNK_BYTES,
            };
            let answer = worker.as_mut().expect("a worker was started above").parse(
                request,
                source.clone(),
                timeout,
            );
            let outcome = match &answer {
                Answer::Parsed(facts) => Outcome::Parsed(facts),
                Answer::Rejected(code) => Outcome::Rejected(code.clone()),
                Answer::Lost(reason) => {
                    worker = None;
                    Outcome::Lost(reason.clone())
                }
            };
            records::file_mutations(previous, next.clone(), outcome, &source, at)?
        };
        let size = encoded_len(&outcome_mutations)?;
        let outcome_mutations = if used.saturating_add(size) <= budget {
            outcome_mutations
        } else if done == 0 {
            // One file whose facts cannot fit any commit is refused, not retried.
            records::file_mutations(
                previous,
                next,
                Outcome::Rejected("facts_exceed_commit_budget".into()),
                &source,
                at,
            )?
        } else {
            break;
        };
        used = used.saturating_add(encoded_len(&outcome_mutations)?);
        for mutation in &outcome_mutations {
            if let Some(state) = records::state_of(mutation)? {
                states.insert(state.path.clone(), state);
            }
        }
        mutations.extend(outcome_mutations);
        done += 1;
    }
    drop(worker);
    states.retain(|path, _| kept.contains(path.as_str()));
    let finished = done == pending.len();
    let summary_sha256 = records::summary_sha256(&states)?;
    permit.charge(
        &mutations,
        engine
            .estate_configuration
            .storage
            .transaction_admission_bytes,
    )?;
    publish(
        engine,
        &work,
        &check,
        mutations,
        &scope,
        finished,
        &summary_sha256,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(content: &str, status: Status, job: &str) -> FileState {
        FileState {
            path: "src/lib.rs".into(),
            content_sha256: content.into(),
            language: "rust".into(),
            parser_revision: "r1".into(),
            chunk_bytes: CHUNK_BYTES,
            job_id: job.into(),
            status,
            reason: String::new(),
            facts_sha256: String::new(),
            symbols: 0,
            spans: 0,
            chunks: 0,
            nodes: 0,
            error_nodes: 0,
        }
    }

    #[test]
    fn a_final_answer_for_the_same_content_is_not_parsed_again() {
        let wanted = state("c1", Status::Lost, "job-2");
        assert!(!settled(None, &wanted), "a new file is parsed");
        assert!(settled(
            Some(&state("c1", Status::Parsed, "job-1")),
            &wanted
        ));
        assert!(settled(
            Some(&state("c1", Status::Rejected, "job-1")),
            &wanted
        ));
        assert!(
            !settled(Some(&state("c0", Status::Parsed, "job-1")), &wanted),
            "an edit is parsed"
        );
    }

    #[test]
    fn a_lost_file_waits_for_the_next_job_instead_of_looping() {
        let wanted = state("c1", Status::Lost, "job-2");
        assert!(settled(Some(&state("c1", Status::Lost, "job-2")), &wanted));
        assert!(!settled(Some(&state("c1", Status::Lost, "job-1")), &wanted));
    }
}
