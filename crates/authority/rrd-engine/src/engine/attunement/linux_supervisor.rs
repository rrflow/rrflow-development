//! Linux direct-child containment for the Parse worker: `/proc/self/exe` runs
//! the exact image this engine started from, the environment is cleared, the
//! address space is capped and the child dies with its supervising thread. It
//! is not a filesystem sandbox.

use super::WORKER_FLAG;
use std::ffi::OsStr;
use std::io;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Owns one direct child. RLIMIT_AS is virtual address space, not RSS or a
/// process-tree memory limit; parent-death signal is not a filesystem sandbox.
#[allow(dead_code)] // The Parse executor launches the worker.
pub(super) struct ManagedChild {
    child: Option<Child>,
    keep_alive: Option<mpsc::Sender<()>>,
    _keeper: JoinHandle<()>,
}

#[allow(dead_code)] // The Parse executor launches the worker.
/// Starts the private parser worker from the running image.
pub(in crate::engine::attunement) fn spawn_worker(
    limit: u64,
) -> io::Result<(ManagedChild, ChildStdin, ChildStdout)> {
    let mut child = ManagedChild::launch(
        Path::new("/proc/self/exe"),
        &[OsStr::new(WORKER_FLAG)],
        limit,
        Stdio::piped(),
        Stdio::piped(),
    )?;
    let running = child.child.as_mut().expect("child is present until drop");
    let input = running.stdin.take().expect("stdin was piped");
    let output = running.stdout.take().expect("stdout was piped");
    Ok((child, input, output))
}

#[allow(dead_code)]
impl ManagedChild {
    pub(super) fn spawn(executable: &Path, args: &[&OsStr], limit: u64) -> io::Result<Self> {
        Self::launch(executable, args, limit, Stdio::null(), Stdio::null())
    }

    fn launch(
        executable: &Path,
        args: &[&OsStr],
        limit: u64,
        stdin: Stdio,
        stdout: Stdio,
    ) -> io::Result<Self> {
        if limit == 0 || !executable.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid child admission",
            ));
        }
        let bound = libc::rlim_t::try_from(limit)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "address limit overflow"))?;
        let limits = libc::rlimit {
            rlim_cur: bound,
            rlim_max: bound,
        };
        // SAFETY: getpid has no arguments and cannot invalidate memory.
        let expected_parent = unsafe { libc::getpid() };
        let mut command = Command::new(executable);
        command
            .args(args)
            .env_clear()
            .stdin(stdin)
            .stdout(stdout)
            .stderr(Stdio::null());
        // SAFETY: this closure runs after fork and before exec. It uses only
        // native system calls and constructs OS error codes without allocation.
        // The parent PID check closes the process-death-before-prctl window;
        // privileged exec and parent-thread edge cases still require admission.
        unsafe {
            command.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::getppid() != expected_parent {
                    return Err(io::Error::from_raw_os_error(libc::ESRCH));
                }
                if libc::setrlimit(libc::RLIMIT_AS, &limits) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        // PR_SET_PDEATHSIG follows the creating thread. Keep the launch thread
        // alive while the owner holds this child, even if the requesting thread
        // returns to a pool. This is not yet a bounded worker scheduler.
        let (ready_tx, ready_rx) = mpsc::sync_channel::<io::Result<Child>>(1);
        let (keep_alive, shutdown) = mpsc::channel::<()>();
        let keeper = std::thread::Builder::new()
            .name("rrflow-parser-supervisor".into())
            .spawn(move || {
                if let Err(error) = ready_tx.send(command.spawn()) {
                    if let Ok(mut child) = error.0 {
                        let _ = child.kill();
                        let started = Instant::now();
                        while started.elapsed() < Duration::from_secs(5) {
                            if !matches!(child.try_wait(), Ok(None)) {
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    }
                    return;
                }
                let _ = shutdown.recv();
            })?;
        let child = ready_rx.recv().map_err(|_| {
            io::Error::new(io::ErrorKind::BrokenPipe, "supervisor exited during launch")
        })??;
        Ok(Self {
            child: Some(child),
            keep_alive: Some(keep_alive),
            _keeper: keeper,
        })
    }

    pub(super) fn wait(&mut self, timeout: Duration) -> io::Result<Option<u32>> {
        let started = Instant::now();
        let child = self.child.as_mut().expect("child is present until drop");
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(Some(match status.code() {
                    Some(code) => code as u32,
                    None => 128 + status.signal().unwrap_or(0) as u32,
                }));
            }
            if started.elapsed() >= timeout {
                return Ok(None);
            }
            std::thread::sleep(
                Duration::from_millis(10).min(timeout.saturating_sub(started.elapsed())),
            );
        }
    }

    pub(super) fn terminate(&mut self) -> io::Result<()> {
        self.child
            .as_mut()
            .expect("child is present until drop")
            .kill()
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        let _ = self.terminate();
        let exited = matches!(self.wait(Duration::from_secs(5)), Ok(Some(_)));
        // A noninterruptible kernel wait can outlast the bound. The child is
        // already killed; this thread reaps it whenever the kernel lets it go.
        if let (false, Some(mut child)) = (exited, self.child.take()) {
            let _ = std::thread::Builder::new()
                .name("rrflow-parser-reaper".into())
                .spawn(move || {
                    let _ = child.wait();
                });
        }
        // Disconnecting releases the keeper only after the termination attempt.
        self.keep_alive.take();
    }
}

#[cfg(test)]
#[path = "linux_supervisor/tests.rs"]
mod tests;
