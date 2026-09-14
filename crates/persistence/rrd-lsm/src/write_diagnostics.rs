//! Explicit, bounded diagnostics for accepted native write batches.
//!
//! The collector is disabled by default and is never persisted. It records
//! physical dimensions and timing only; keys and values are deliberately
//! excluded. These samples are diagnostic evidence, not storage authority.

use crate::database::MaintenanceStats;
use crate::{AppendReceipt, Durability, Error, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub const WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION: u16 = 1;
pub const MAX_WRITE_PATH_DIAGNOSTIC_SAMPLES: usize = 65_536;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WritePathPhaseTiming {
    pub wall_ns: u64,
    pub thread_cpu_ns: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteLockWait {
    pub begin_mutex_wait_ns: u64,
    pub commit_mutex_wait_ns: u64,
}

impl WriteLockWait {
    pub fn total_ns(self) -> u64 {
        self.begin_mutex_wait_ns
            .saturating_add(self.commit_mutex_wait_ns)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteThreadResourceDelta {
    pub minor_page_faults: u64,
    pub major_page_faults: u64,
    pub block_input_operations: u64,
    pub block_output_operations: u64,
    pub voluntary_context_switches: u64,
    pub involuntary_context_switches: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteMaintenanceDelta {
    pub automatic_flushes: u64,
    pub write_stalls: u64,
    pub failed_flushes: u64,
    pub oversized_batches: u64,
    pub automatic_compactions: u64,
    pub failed_compactions: u64,
}

impl WriteMaintenanceDelta {
    fn between(before: MaintenanceStats, after: MaintenanceStats) -> Self {
        Self {
            automatic_flushes: after
                .automatic_flushes
                .saturating_sub(before.automatic_flushes),
            write_stalls: after.write_stalls.saturating_sub(before.write_stalls),
            failed_flushes: after.failed_flushes.saturating_sub(before.failed_flushes),
            oversized_batches: after
                .oversized_batches
                .saturating_sub(before.oversized_batches),
            automatic_compactions: after
                .automatic_compactions
                .saturating_sub(before.automatic_compactions),
            failed_compactions: after
                .failed_compactions
                .saturating_sub(before.failed_compactions),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WritePathPhases {
    pub transaction_validation: WritePathPhaseTiming,
    pub batch_prepare: WritePathPhaseTiming,
    pub batch_encode: WritePathPhaseTiming,
    pub maintenance: WritePathPhaseTiming,
    pub wal_record_prepare: WritePathPhaseTiming,
    pub wal_initial_reservation: WritePathPhaseTiming,
    pub wal_write: WritePathPhaseTiming,
    pub wal_sync: WritePathPhaseTiming,
    pub memtable_apply: WritePathPhaseTiming,
    pub bookkeeping: WritePathPhaseTiming,
    pub physical_total: WritePathPhaseTiming,
}

impl WritePathPhases {
    /// Sum of the mutually exclusive measured sub-phases. The enclosing total
    /// additionally contains measurement calls and short transitions.
    pub fn measured_wall_ns(self) -> u64 {
        [
            self.transaction_validation.wall_ns,
            self.batch_prepare.wall_ns,
            self.batch_encode.wall_ns,
            self.maintenance.wall_ns,
            self.wal_record_prepare.wall_ns,
            self.wal_initial_reservation.wall_ns,
            self.wal_write.wall_ns,
            self.wal_sync.wall_ns,
            self.memtable_apply.wall_ns,
            self.bookkeeping.wall_ns,
        ]
        .into_iter()
        .fold(0u64, u64::saturating_add)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WritePathSample {
    pub ordinal: u64,
    pub durability: Durability,
    pub mutation_count: usize,
    pub encoded_payload_bytes: usize,
    pub wal_frame_bytes: usize,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub memtable_bytes_added: usize,
    pub lock_wait: WriteLockWait,
    pub maintenance: WriteMaintenanceDelta,
    pub phases: WritePathPhases,
    pub thread_resources: Option<WriteThreadResourceDelta>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WritePathDiagnostics {
    pub contract_version: u16,
    pub capacity: usize,
    pub observed_samples: u64,
    pub dropped_samples: u64,
    pub thread_resource_scope: String,
    pub samples: Vec<WritePathSample>,
}

pub(crate) struct WritePathCollector {
    capacity: usize,
    observed_samples: u64,
    dropped_samples: u64,
    samples: Vec<WritePathSample>,
}

impl WritePathCollector {
    pub(crate) fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 || capacity > MAX_WRITE_PATH_DIAGNOSTIC_SAMPLES {
            return Err(Error::InvalidConfiguration(format!(
                "write diagnostic capacity must be in 1..={MAX_WRITE_PATH_DIAGNOSTIC_SAMPLES}"
            )));
        }
        let mut samples = Vec::new();
        samples.try_reserve_exact(capacity).map_err(|error| {
            Error::InvalidConfiguration(format!(
                "cannot reserve write diagnostic capacity {capacity}: {error}"
            ))
        })?;
        Ok(Self {
            capacity,
            observed_samples: 0,
            dropped_samples: 0,
            samples,
        })
    }

    pub(crate) fn record(&mut self, mut sample: WritePathSample) {
        sample.ordinal = self.observed_samples;
        self.observed_samples = self.observed_samples.saturating_add(1);
        if self.samples.len() < self.capacity {
            self.samples.push(sample);
        } else {
            self.dropped_samples = self.dropped_samples.saturating_add(1);
        }
    }

    pub(crate) fn finish(self) -> WritePathDiagnostics {
        WritePathDiagnostics {
            contract_version: WRITE_PATH_DIAGNOSTICS_CONTRACT_VERSION,
            capacity: self.capacity,
            observed_samples: self.observed_samples,
            dropped_samples: self.dropped_samples,
            thread_resource_scope: thread_resource_scope().into(),
            samples: self.samples,
        }
    }
}

pub(crate) struct PendingWriteDiagnostics {
    total: PhaseClock,
    resources_before: Option<ThreadResourceSnapshot>,
    maintenance_before: MaintenanceStats,
    memtable_bytes_before: usize,
    lock_wait: WriteLockWait,
    pub(crate) phases: WritePathPhases,
}

impl PendingWriteDiagnostics {
    pub(crate) fn start(
        lock_wait: WriteLockWait,
        maintenance_before: MaintenanceStats,
        memtable_bytes_before: usize,
    ) -> Self {
        Self {
            total: PhaseClock::start(),
            resources_before: thread_resource_snapshot(),
            maintenance_before,
            memtable_bytes_before,
            lock_wait,
            phases: WritePathPhases::default(),
        }
    }

    pub(crate) fn finish(
        mut self,
        durability: Durability,
        mutation_count: usize,
        encoded_payload_bytes: usize,
        receipt: &AppendReceipt,
        maintenance_after: MaintenanceStats,
        memtable_bytes_after: usize,
    ) -> WritePathSample {
        let thread_resources = self
            .resources_before
            .zip(thread_resource_snapshot())
            .map(|(before, after)| after.delta(before));
        self.phases.physical_total = self.total.finish();
        WritePathSample {
            ordinal: 0,
            durability,
            mutation_count,
            encoded_payload_bytes,
            wal_frame_bytes: usize::try_from(receipt.end_offset.saturating_sub(receipt.offset))
                .unwrap_or(usize::MAX),
            first_sequence: receipt.first_sequence,
            last_sequence: receipt.last_sequence,
            memtable_bytes_added: memtable_bytes_after.saturating_sub(self.memtable_bytes_before),
            lock_wait: self.lock_wait,
            maintenance: WriteMaintenanceDelta::between(self.maintenance_before, maintenance_after),
            phases: self.phases,
            thread_resources,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WalAppendPhases {
    pub(crate) record_prepare: WritePathPhaseTiming,
    pub(crate) initial_reservation: WritePathPhaseTiming,
    pub(crate) write: WritePathPhaseTiming,
    pub(crate) sync: WritePathPhaseTiming,
}

pub(crate) struct PhaseClock {
    wall_started: Instant,
    thread_cpu_started_ns: Option<u64>,
}

impl PhaseClock {
    pub(crate) fn start() -> Self {
        Self {
            wall_started: Instant::now(),
            thread_cpu_started_ns: thread_cpu_ns(),
        }
    }

    pub(crate) fn finish(self) -> WritePathPhaseTiming {
        let thread_cpu_ns = self
            .thread_cpu_started_ns
            .zip(thread_cpu_ns())
            .map(|(before, after)| after.saturating_sub(before));
        WritePathPhaseTiming {
            wall_ns: duration_ns(self.wall_started.elapsed()),
            thread_cpu_ns,
        }
    }
}

fn duration_ns(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[derive(Debug, Clone, Copy)]
struct ThreadResourceSnapshot {
    minor_page_faults: u64,
    major_page_faults: u64,
    block_input_operations: u64,
    block_output_operations: u64,
    voluntary_context_switches: u64,
    involuntary_context_switches: u64,
}

impl ThreadResourceSnapshot {
    fn delta(self, before: Self) -> WriteThreadResourceDelta {
        WriteThreadResourceDelta {
            minor_page_faults: self
                .minor_page_faults
                .saturating_sub(before.minor_page_faults),
            major_page_faults: self
                .major_page_faults
                .saturating_sub(before.major_page_faults),
            block_input_operations: self
                .block_input_operations
                .saturating_sub(before.block_input_operations),
            block_output_operations: self
                .block_output_operations
                .saturating_sub(before.block_output_operations),
            voluntary_context_switches: self
                .voluntary_context_switches
                .saturating_sub(before.voluntary_context_switches),
            involuntary_context_switches: self
                .involuntary_context_switches
                .saturating_sub(before.involuntary_context_switches),
        }
    }
}

#[cfg(target_os = "linux")]
fn thread_cpu_ns() -> Option<u64> {
    let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: clock_gettime initializes the pointed-to timespec on success.
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, value.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: the successful call above initialized every timespec field.
    let value = unsafe { value.assume_init() };
    let seconds = u64::try_from(value.tv_sec).ok()?;
    let nanoseconds = u64::try_from(value.tv_nsec).ok()?;
    seconds.checked_mul(1_000_000_000)?.checked_add(nanoseconds)
}

#[cfg(not(target_os = "linux"))]
fn thread_cpu_ns() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn thread_resource_snapshot() -> Option<ThreadResourceSnapshot> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: getrusage initializes the pointed-to rusage structure on success.
    if unsafe { libc::getrusage(libc::RUSAGE_THREAD, usage.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: the successful call above initialized every rusage field.
    let usage = unsafe { usage.assume_init() };
    Some(ThreadResourceSnapshot {
        minor_page_faults: nonnegative_counter(usage.ru_minflt),
        major_page_faults: nonnegative_counter(usage.ru_majflt),
        block_input_operations: nonnegative_counter(usage.ru_inblock),
        block_output_operations: nonnegative_counter(usage.ru_oublock),
        voluntary_context_switches: nonnegative_counter(usage.ru_nvcsw),
        involuntary_context_switches: nonnegative_counter(usage.ru_nivcsw),
    })
}

#[cfg(not(target_os = "linux"))]
fn thread_resource_snapshot() -> Option<ThreadResourceSnapshot> {
    None
}

#[cfg(target_os = "linux")]
fn nonnegative_counter(value: libc::c_long) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

#[cfg(target_os = "linux")]
const fn thread_resource_scope() -> &'static str {
    "linux_rusage_thread"
}

#[cfg(not(target_os = "linux"))]
const fn thread_resource_scope() -> &'static str {
    "unavailable"
}
