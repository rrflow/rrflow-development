//! One supervised parse worker serving a batch of files. The worker runs the
//! engine's own executable under the native supervisor. Two threads drive its
//! pipes, so a child that stalls, dies or breaks the protocol ends that file as
//! `Lost` at the deadline instead of blocking the Step.

use rrd_attunement::facts::SourceFacts;
use rrd_attunement::worker::{read_response, write_request, WorkerRequest, WorkerResponse};
use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

/// What the worker said about one file.
#[derive(Debug)]
pub(super) enum Answer {
    Parsed(SourceFacts),
    /// The parser refused the content, with its fixed code.
    Rejected(String),
    /// The worker stalled, exited or broke the protocol, and is unusable after this.
    Lost(String),
}

/// Ends the child process when a file is lost or the batch is dropped.
pub(super) trait Stop {
    fn stop(&mut self);
}

pub(super) struct Worker {
    child: Box<dyn Stop>,
    requests: Option<Sender<(WorkerRequest, Vec<u8>)>>,
    responses: Receiver<io::Result<WorkerResponse>>,
    usable: bool,
}

impl Worker {
    /// Drives a worker through its request and response pipes.
    pub(super) fn over(
        child: Box<dyn Stop>,
        mut input: impl Write + Send + 'static,
        mut output: impl Read + Send + 'static,
    ) -> io::Result<Self> {
        let (requests, queued) = mpsc::channel::<(WorkerRequest, Vec<u8>)>();
        let (sent, expected) = mpsc::channel::<WorkerRequest>();
        let (answered, responses) = mpsc::channel();
        std::thread::Builder::new()
            .name("rrflow-parse-writer".into())
            .spawn(move || {
                for (request, source) in queued {
                    let written =
                        write_request(&mut input, &request, &source).and_then(|()| input.flush());
                    if written.is_err() || sent.send(request).is_err() {
                        return;
                    }
                }
                // Returning drops `input`, closing the pipe at a frame boundary.
            })?;
        std::thread::Builder::new()
            .name("rrflow-parse-reader".into())
            .spawn(move || {
                for request in expected {
                    let response = read_response(&mut output, &request);
                    let failed = response.is_err();
                    if answered.send(response).is_err() || failed {
                        return;
                    }
                }
            })?;
        Ok(Self {
            child,
            requests: Some(requests),
            responses,
            usable: true,
        })
    }

    /// Sends one verified source and waits up to `timeout` for its answer.
    pub(super) fn parse(
        &mut self,
        request: WorkerRequest,
        source: Vec<u8>,
        timeout: Duration,
    ) -> Answer {
        if !self.usable {
            return Answer::Lost("worker_unusable".into());
        }
        let delivered = self
            .requests
            .as_ref()
            .is_some_and(|requests| requests.send((request, source)).is_ok());
        let answer = if delivered {
            self.responses.recv_timeout(timeout)
        } else {
            Err(RecvTimeoutError::Disconnected)
        };
        match answer {
            Ok(Ok(WorkerResponse::Parsed { facts })) => Answer::Parsed(facts),
            Ok(Ok(WorkerResponse::Rejected { code })) => Answer::Rejected(code),
            Ok(Err(_)) => self.lose("worker_protocol"),
            Err(RecvTimeoutError::Timeout) => self.lose("worker_deadline"),
            Err(RecvTimeoutError::Disconnected) => self.lose("worker_exited"),
        }
    }

    fn lose(&mut self, reason: &str) -> Answer {
        self.usable = false;
        self.requests = None;
        self.child.stop();
        Answer::Lost(reason.into())
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Closing the requests lets a healthy worker finish; stop it regardless.
        self.requests = None;
        self.child.stop();
    }
}

// Engine tests drive an in-process worker instead.
#[cfg(any(windows, target_os = "linux"))]
#[cfg_attr(test, allow(dead_code))]
mod native {
    #[cfg(target_os = "linux")]
    use super::super::super::linux_supervisor as supervisor;
    #[cfg(windows)]
    use super::super::super::windows_supervisor as supervisor;
    use super::*;

    struct Supervised(supervisor::ManagedChild);

    impl Stop for Supervised {
        fn stop(&mut self) {
            let _ = self.0.terminate();
            let _ = self.0.wait(Duration::from_secs(5));
        }
    }

    /// Starts the engine's own executable as a worker within `memory_limit`.
    pub(in crate::engine::attunement) fn start(memory_limit: u64) -> io::Result<Worker> {
        let (child, input, output) = supervisor::spawn_worker(memory_limit)?;
        Worker::over(Box::new(Supervised(child)), input, output)
    }
}

#[cfg(any(windows, target_os = "linux"))]
#[allow(unused_imports)] // The Step uses this once it dispatches, in the next pull request.
pub(in crate::engine::attunement) use native::start;

/// No native supervisor contains the worker on this platform, so Parse cannot run.
#[cfg(not(any(windows, target_os = "linux")))]
#[cfg_attr(test, allow(dead_code))]
pub(in crate::engine::attunement) fn start(_memory_limit: u64) -> io::Result<Worker> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "no native parse supervisor on this platform",
    ))
}

/// A worker on the calling process, for engine tests that cannot start the
/// primary executable.
#[cfg(test)]
pub(super) fn in_process(_memory_limit: u64) -> io::Result<Worker> {
    struct Unsupervised;
    impl Stop for Unsupervised {
        fn stop(&mut self) {}
    }
    let (mut request_reader, request_writer) = io::pipe()?;
    let (response_reader, mut response_writer) = io::pipe()?;
    std::thread::spawn(move || {
        let _ = rrd_attunement::worker::run_worker(&mut request_reader, &mut response_writer);
    });
    Worker::over(Box::new(Unsupervised), request_writer, response_reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rrd_attunement::parse::ParseLimits;
    use rrd_attunement::worker::{run_worker, WORKER_PROTOCOL_VERSION};
    use rrd_core::digest::sha256_hex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct Counted(Arc<AtomicUsize>);

    impl Stop for Counted {
        fn stop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn request(source: &[u8]) -> WorkerRequest {
        WorkerRequest {
            format_version: WORKER_PROTOCOL_VERSION,
            source_id: "src/lib.rs".into(),
            content_sha256: sha256_hex(source),
            language: "rust".into(),
            source_bytes: source.len() as u64,
            limits: ParseLimits {
                input_bytes: 1 << 20,
                nodes: 1 << 20,
                depth: 256,
                output_bytes: 1 << 20,
                elapsed_micros: 5_000_000,
            },
            chunk_bytes: 1_500,
        }
    }

    /// A worker whose far end is `serve`, running on its own thread.
    fn worker(
        serve: impl FnOnce(io::PipeReader, io::PipeWriter) + Send + 'static,
    ) -> (Worker, Arc<AtomicUsize>) {
        let (request_reader, request_writer) = io::pipe().unwrap();
        let (response_reader, response_writer) = io::pipe().unwrap();
        std::thread::spawn(move || serve(request_reader, response_writer));
        let stops = Arc::new(AtomicUsize::new(0));
        let worker = Worker::over(
            Box::new(Counted(stops.clone())),
            request_writer,
            response_reader,
        )
        .unwrap();
        (worker, stops)
    }

    #[test]
    fn one_worker_parses_a_batch_of_files() {
        let (mut worker, _) = worker(|mut input, mut output| {
            run_worker(&mut input, &mut output).unwrap();
        });
        for name in ["alpha", "beta"] {
            let source = format!("fn {name}() {{}}\n").into_bytes();
            let Answer::Parsed(facts) =
                worker.parse(request(&source), source.clone(), Duration::from_secs(30))
            else {
                panic!("{name} was not parsed");
            };
            assert_eq!(facts.content_sha256, sha256_hex(&source));
        }
    }

    #[test]
    fn a_stalled_worker_is_lost_at_the_deadline_and_stopped() {
        let (mut worker, stops) = worker(|input, output| {
            std::thread::sleep(Duration::from_secs(5));
            drop((input, output));
        });
        let source = b"fn alpha() {}\n".to_vec();
        let started = std::time::Instant::now();
        let answer = worker.parse(request(&source), source.clone(), Duration::from_millis(200));
        assert!(matches!(answer, Answer::Lost(ref reason) if reason == "worker_deadline"));
        assert!(started.elapsed() < Duration::from_secs(4));
        assert_eq!(stops.load(Ordering::SeqCst), 1);
        let again = worker.parse(request(&source), source, Duration::from_secs(1));
        assert!(matches!(again, Answer::Lost(ref reason) if reason == "worker_unusable"));
    }

    #[test]
    fn a_worker_that_exits_or_breaks_the_protocol_is_lost() {
        let source = b"fn alpha() {}\n".to_vec();
        let (mut exited, _) = worker(|input, output| drop((input, output)));
        assert!(matches!(
            exited.parse(request(&source), source.clone(), Duration::from_secs(5)),
            Answer::Lost(_)
        ));
        let (mut garbled, _) = worker(|mut input, mut output| {
            let mut prefix = [0; 4];
            input.read_exact(&mut prefix).unwrap();
            output.write_all(&[0, 0, 0, 3, b'b', b'a', b'd']).unwrap();
            std::thread::sleep(Duration::from_secs(2));
        });
        assert!(matches!(
            garbled.parse(request(&source), source, Duration::from_secs(5)),
            Answer::Lost(ref reason) if reason == "worker_protocol"
        ));
    }
}
