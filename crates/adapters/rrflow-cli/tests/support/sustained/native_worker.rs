//! Owned native resource acquisition for the installed sustained harness.
//!
//! The helper is this test executable, not an installed product service. Its
//! blocking platform calls never run on the driver's current-thread runtime.
use super::{native, DriverResult};
use rrflow_eval::storage::{AllocationEnforcement, NativeAllocationReceipt, NativeMetric};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const INPUT_ENV: &str = "RRFLOW_NATIVE_SAMPLE_WORKER_V1";
const MARKER: &str = "rrflow_native_sample_v1=";
const HELPER: &str = "sustained::native_worker::native_sample_helper";
const OUTPUT_LIMIT: usize = 64 * 1024;
const DEADLINE: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    pid: u32,
    harness_pid: u32,
    allocation: Option<NativeAllocationReceipt>,
    // Only explicitly constructed harness tests supply this private gate.
    hold_marker: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Observation {
    pid: u32,
    harness_pid: u32,
    worker_pid: u32,
    pub resident_bytes: u64,
    pub host_resident_bytes: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
    pub cpu_time_ns: u64,
    pub private_bytes: NativeMetric,
    pub mapped_bytes: NativeMetric,
    pub enforcement: AllocationEnforcement,
    pub host_ns: u64,
    pub memory_ns: u64,
    pub enforcement_ns: u64,
}

struct Active {
    cancel: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

#[derive(Clone, Default)]
pub(super) struct Owner {
    active: Arc<Mutex<Option<Active>>>,
    hold: Arc<Mutex<Option<(usize, PathBuf)>>>,
}

struct CancelOnDrop(Arc<AtomicBool>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

impl Owner {
    pub(super) async fn acquire(
        &self,
        pid: u32,
        allocation: &Option<NativeAllocationReceipt>,
    ) -> DriverResult<Observation> {
        let hold_marker = {
            let mut hold = self
                .hold
                .lock()
                .map_err(|_| "native test gate lock poisoned")?;
            match hold.as_mut() {
                Some((remaining, _)) if *remaining > 0 => {
                    *remaining -= 1;
                    None
                }
                Some(_) => hold.take().map(|(_, marker)| marker),
                None => None,
            }
        };
        self.acquire_request(Request {
            pid,
            harness_pid: std::process::id(),
            allocation: allocation.clone(),
            hold_marker,
        })
        .await
    }

    /// Private installed-test gate; production campaign inputs cannot set it.
    pub(super) fn hold_after(&self, skip: usize, marker: PathBuf) {
        *self.hold.lock().expect("native test gate lock") = Some((skip, marker));
    }

    async fn acquire_request(&self, request: Request) -> DriverResult<Observation> {
        let expected_pid = request.pid;
        let expected_harness = request.harness_pid;
        let input = serde_json::to_string(&request).map_err(|error| error.to_string())?;
        if input.len() > 8192 || expected_pid == 0 || expected_harness == 0 {
            return Err(
                "native sample request exceeds bounds or has invalid process identity".into(),
            );
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = tokio::sync::oneshot::channel();
        {
            let mut active = self
                .active
                .lock()
                .map_err(|_| "native worker lock poisoned")?;
            if active
                .as_ref()
                .is_some_and(|worker| !worker.thread.is_finished())
            {
                return Err("previous native sample is still active or being reaped".into());
            }
            if let Some(previous) = active.take() {
                previous
                    .thread
                    .join()
                    .map_err(|_| "native worker monitor panicked")?;
            }
            let token = cancel.clone();
            let thread = thread::spawn(move || {
                let result = run_helper(&input, token);
                let _ = sender.send(result);
            });
            *active = Some(Active {
                cancel: cancel.clone(),
                thread,
            });
        }
        let _cancel_on_drop = CancelOnDrop(cancel.clone());
        let observation = receiver.await.map_err(|_| "native worker monitor stopped");
        self.join_finished(Some(&cancel)).await?;
        let observation = observation??;
        if observation.pid != expected_pid
            || observation.harness_pid != expected_harness
            || observation.worker_pid == 0
            || observation.worker_pid == expected_harness
        {
            return Err("native worker returned mismatched process identities".into());
        }
        Ok(observation)
    }

    async fn join_finished(&self, expected: Option<&Arc<AtomicBool>>) -> DriverResult<()> {
        let deadline = tokio::time::Instant::now() + DEADLINE;
        loop {
            {
                let mut active = self
                    .active
                    .lock()
                    .map_err(|_| "native worker lock poisoned")?;
                if active.as_ref().is_none_or(|worker| {
                    expected.is_some_and(|token| !Arc::ptr_eq(token, &worker.cancel))
                        || worker.thread.is_finished()
                }) {
                    if active.as_ref().is_some_and(|worker| {
                        expected.is_some_and(|token| !Arc::ptr_eq(token, &worker.cancel))
                    }) {
                        return Ok(());
                    }
                    if let Some(worker) = active.take() {
                        worker
                            .thread
                            .join()
                            .map_err(|_| "native worker monitor panicked")?;
                    }
                    return Ok(());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Err("native worker cleanup exceeded its deadline".into());
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    pub(super) async fn shutdown(&self) -> DriverResult<()> {
        let token = {
            let active = self
                .active
                .lock()
                .map_err(|_| "native worker lock poisoned")?;
            active.as_ref().map(|worker| {
                worker.cancel.store(true, Ordering::Release);
                worker.cancel.clone()
            })
        };
        if let Some(token) = token {
            self.join_finished(Some(&token)).await?;
        }
        Ok(())
    }
}

fn run_helper(input: &str, cancel: Arc<AtomicBool>) -> DriverResult<Observation> {
    let mut command = Command::new(std::env::current_exe().map_err(|error| error.to_string())?);
    command
        .args(["--exact", HELPER, "--ignored", "--nocapture", "--quiet"])
        .env(INPUT_ENV, input);
    let mut process = super::super::process::Process::start_cancellable(
        &mut command,
        DEADLINE,
        OUTPUT_LIMIT,
        cancel,
    )
    .map_err(|error| error.to_string())?;
    let worker_pid = process.id();
    let output = process.finish()?;
    if !output.status.success() {
        return Err("native sample helper exited unsuccessfully".into());
    }
    let text =
        std::str::from_utf8(&output.stdout).map_err(|_| "native helper output is not UTF-8")?;
    let records: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix(MARKER))
        .collect();
    if records.len() != 1 {
        return Err("native helper did not return exactly one observation".into());
    }
    let observation: Observation = serde_json::from_str(records[0])
        .map_err(|error| format!("invalid native observation: {error}"))?;
    if observation.worker_pid != worker_pid {
        return Err("native observation does not belong to the owned worker".into());
    }
    Ok(observation)
}

fn elapsed(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

#[test]
#[ignore = "private test-executable helper; invoked by the installed harness"]
fn native_sample_helper() {
    let raw = std::env::var(INPUT_ENV).expect("explicit bounded native sample request");
    assert!(raw.len() <= 8192);
    let request: Request = serde_json::from_str(&raw).unwrap();
    assert!(request.pid != 0 && request.harness_pid != 0);
    if let Some(marker) = &request.hold_marker {
        std::fs::write(marker, std::process::id().to_string()).unwrap();
        // This private test gate deliberately blocks the helper, never the driver.
        let release = marker.with_extension("release");
        let deadline = Instant::now() + Duration::from_secs(30);
        while !release.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(release.exists(), "native test gate was not released");
    }
    let started = Instant::now();
    let mut system = sysinfo::System::new();
    let process_id = sysinfo::Pid::from_u32(request.pid);
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[process_id]), true);
    system.refresh_memory();
    let process = system
        .process(process_id)
        .expect("owned engine exited during native sampling");
    let io = process.disk_usage();
    let host_ns = elapsed(started);
    let memory_started = Instant::now();
    let (private_bytes, mapped_bytes) = native::memory(request.pid);
    let memory_ns = elapsed(memory_started);
    let enforcement_started = Instant::now();
    let enforcement = native::enforcement_for(
        request.pid,
        request.harness_pid,
        request.allocation.as_ref(),
    )
    .unwrap();
    let result = Observation {
        pid: request.pid,
        harness_pid: request.harness_pid,
        worker_pid: std::process::id(),
        resident_bytes: process.memory(),
        host_resident_bytes: system.used_memory(),
        read_bytes: io.total_read_bytes,
        written_bytes: io.total_written_bytes,
        cpu_time_ns: process
            .accumulated_cpu_time()
            .checked_mul(1_000_000)
            .expect("CPU time overflow"),
        private_bytes,
        mapped_bytes,
        enforcement,
        host_ns,
        memory_ns,
        enforcement_ns: elapsed(enforcement_started),
    };
    println!("{MARKER}{}", serde_json::to_string(&result).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires an explicitly launcher-owned native allocation"]
    fn native_job_contains_engine_harness_and_worker() {
        let raw = std::env::var("RRFLOW_SUSTAINED_ALLOCATION_RECEIPT").unwrap();
        let supplied: NativeAllocationReceipt = serde_json::from_str(&raw).unwrap();
        assert_eq!(supplied.engine_bytes.get(), 2 * 1024 * 1024 * 1024);
        assert_eq!(supplied.host_bytes.get(), 4 * 1024 * 1024 * 1024);
        let mut campaign = super::super::smoke_campaign();
        campaign.allocation.engine_bytes = supplied.engine_bytes;
        campaign.allocation.host_bytes = supplied.host_bytes;
        campaign.allocation.vcpus = supplied.vcpus;
        let receipt = native::receipt(&campaign).unwrap().unwrap();
        let expected = receipt.enforcement().unwrap();
        let estate = super::super::Estate::install("fresh");
        let clock = Instant::now();
        let (mut engine, _client, _) = estate.start_for(clock, Duration::from_secs(120));
        let engine_pid = engine.id();
        assert_ne!(engine_pid, std::process::id());
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let owner = Owner::default();
                let observed = owner
                    .acquire(engine_pid, &Some(receipt.clone()))
                    .await
                    .unwrap();
                assert_eq!(observed.enforcement, expected);
                assert_ne!(observed.worker_pid, engine_pid);
                assert_ne!(observed.worker_pid, std::process::id());
                let directory = tempfile::tempdir().unwrap();
                let marker = directory.path().join("job-held-worker");
                let worker_owner = owner.clone();
                let worker_receipt = receipt.clone();
                let worker_marker = marker.clone();
                let held = tokio::spawn(async move {
                    worker_owner
                        .acquire_request(Request {
                            pid: engine_pid,
                            harness_pid: std::process::id(),
                            allocation: Some(worker_receipt),
                            hold_marker: Some(worker_marker),
                        })
                        .await
                });
                let helper_pid = tokio::time::timeout(Duration::from_secs(6), async {
                    loop {
                        if let Ok(raw) = std::fs::read_to_string(&marker) {
                            if let Ok(pid) = raw.parse::<u32>() {
                                break pid;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await
                .unwrap();
                assert_eq!(
                    native::enforcement_for(helper_pid, std::process::id(), Some(&receipt))
                        .unwrap(),
                    expected
                );
                held.abort();
                assert!(held.await.is_err());
                owner.shutdown().await.unwrap();
                let mut processes = sysinfo::System::new();
                let helper = sysinfo::Pid::from_u32(helper_pid);
                processes.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[helper]), true);
                assert!(
                    processes.process(helper).is_none(),
                    "native Job helper survived cancellation"
                );
                let wrong_harness = owner
                    .acquire_request(Request {
                        pid: engine_pid,
                        harness_pid: u32::MAX,
                        allocation: Some(receipt.clone()),
                        hold_marker: None,
                    })
                    .await;
                assert!(wrong_harness.is_err());
                let resumed = owner.acquire(engine_pid, &Some(receipt)).await.unwrap();
                assert_eq!(resumed.enforcement, expected);
                owner.shutdown().await.unwrap();
            });
        engine.terminate();
        engine.finish().expect("reap owned installed engine");
    }

    #[test]
    fn held_native_worker_is_cancelled_reaped_and_owner_reused() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let directory = tempfile::tempdir().unwrap();
                let marker = directory.path().join("native-worker-entered");
                let owner = Owner::default();
                let held_owner = owner.clone();
                let held_marker = marker.clone();
                let held = tokio::spawn(async move {
                    held_owner
                        .acquire_request(Request {
                            pid: std::process::id(),
                            harness_pid: std::process::id(),
                            allocation: None,
                            hold_marker: Some(held_marker),
                        })
                        .await
                });
                let helper_pid = tokio::time::timeout(Duration::from_secs(6), async {
                    loop {
                        if let Ok(value) = std::fs::read_to_string(&marker) {
                            if let Ok(pid) = value.parse::<u32>() {
                                break pid;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await
                .unwrap();
                assert!(owner.acquire(std::process::id(), &None).await.is_err());
                // The held helper must not block the single-thread runtime.
                tokio::time::timeout(Duration::from_millis(500), async {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                })
                .await
                .unwrap();
                held.abort();
                assert!(held.await.is_err());
                owner.shutdown().await.unwrap();
                let mut system = sysinfo::System::new();
                let helper = sysinfo::Pid::from_u32(helper_pid);
                system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[helper]), true);
                assert!(
                    system.process(helper).is_none(),
                    "owned helper survived cancellation"
                );
                let observation = owner.acquire(std::process::id(), &None).await.unwrap();
                assert_eq!(observation.pid, std::process::id());
                assert_eq!(observation.harness_pid, std::process::id());
                assert_ne!(observation.worker_pid, observation.harness_pid);
                assert!(matches!(
                    observation.enforcement,
                    AllocationEnforcement::ObservedOnly
                ));
                owner.shutdown().await.unwrap();
                // A dead target must fail without manufacturing an observation,
                // and its failed worker must not poison the next acquisition.
                assert!(owner.acquire(u32::MAX, &None).await.is_err());
                owner.shutdown().await.unwrap();
                assert!(owner.acquire(std::process::id(), &None).await.is_ok());
                owner.shutdown().await.unwrap();
            });
    }
}
