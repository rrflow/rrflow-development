//! The supervised parser process for one batch. The engine owns the deadline:
//! the worker's own limits are advisory because a parser fault is exactly the
//! case they cannot be trusted in.

use super::{FileOutcome, FileRequest};
use crate::engine::ServiceError;
use rrd_attunement::parse::ParseLimits;
use rrd_attunement::worker::{
    read_response, write_request, WorkerRequest, WorkerResponse, WORKER_PROTOCOL_VERSION,
};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use super::super::linux_supervisor::spawn_worker;
#[cfg(windows)]
use super::super::windows_supervisor::spawn_worker;

type Result<T> = std::result::Result<T, ServiceError>;

/// How often the batch re-checks the job's lease and deadline while waiting.
const POLL: Duration = Duration::from_millis(100);
/// Added to the parser's own limit so the worker can report a limit itself.
const DEADLINE_MARGIN: Duration = Duration::from_secs(2);

fn request_for(file: &FileRequest<'_>, limits: &ParseLimits, chunk_bytes: u64) -> WorkerRequest {
    WorkerRequest {
        format_version: WORKER_PROTOCOL_VERSION,
        source_id: file.source_id.to_owned(),
        content_sha256: file.content_sha256.to_owned(),
        language: file.language.to_owned(),
        source_bytes: file.source.len() as u64,
        limits: limits.clone(),
        chunk_bytes,
    }
}

pub(super) fn run_batch(
    files: &[FileRequest<'_>],
    limits: &ParseLimits,
    chunk_bytes: u64,
    memory_bytes: u64,
    check: &dyn Fn() -> Result<()>,
) -> Result<Vec<FileOutcome>> {
    let (mut child, mut input, mut output) = spawn_worker(memory_bytes)
        .map_err(|error| ServiceError::Storage(format!("parser worker did not start: {error}")))?;
    let per_file = Duration::from_micros(limits.elapsed_micros).saturating_add(DEADLINE_MARGIN);
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        scope.spawn(move || {
            for file in files {
                let request = request_for(file, limits, chunk_bytes);
                let exchanged = write_request(&mut input, &request, file.source)
                    .and_then(|()| read_response(&mut output, &request));
                let failed = exchanged.is_err();
                if sender.send(exchanged).is_err() || failed {
                    break;
                }
            }
            // Dropping `input` here is the worker's clean end of batch.
        });

        let mut outcomes = Vec::with_capacity(files.len());
        let mut waiting_since = Instant::now();
        while outcomes.len() < files.len() {
            match receiver.recv_timeout(POLL) {
                Ok(Ok(WorkerResponse::Parsed { facts })) => {
                    outcomes.push(FileOutcome::Parsed(facts));
                    waiting_since = Instant::now();
                }
                Ok(Ok(WorkerResponse::Rejected { code })) => {
                    outcomes.push(FileOutcome::Rejected(code));
                    waiting_since = Instant::now();
                }
                Ok(Err(_)) | Err(RecvTimeoutError::Disconnected) => {
                    let _ = child.terminate();
                    outcomes.push(FileOutcome::Lost("worker_failed".into()));
                    return Ok(outcomes);
                }
                Err(RecvTimeoutError::Timeout) => {
                    if let Err(error) = check() {
                        let _ = child.terminate();
                        return Err(error);
                    }
                    if waiting_since.elapsed() > per_file {
                        let _ = child.terminate();
                        outcomes.push(FileOutcome::Lost("worker_timed_out".into()));
                        return Ok(outcomes);
                    }
                }
            }
        }
        Ok(outcomes)
    })
}
