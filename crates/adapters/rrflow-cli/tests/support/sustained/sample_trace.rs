//! Bounded, numeric-only attribution for slow completed resource acquisitions.
//! This does not replace resource samples or change their coverage validator.
use std::sync::atomic::{AtomicUsize, Ordering};

const SLOW_NS: u64 = 200_000_000;
const MAX_RECORDS: usize = 64;
static ADMITTED: AtomicUsize = AtomicUsize::new(0);

pub(super) struct Observation {
    pub pid: u32,
    pub elapsed_ns: u64,
    pub total_ns: u64,
    pub host_ns: u64,
    pub rpc_ns: u64,
    pub memory_ns: u64,
    pub enforcement_ns: u64,
}

#[derive(Debug, PartialEq, Eq)]
enum Admission {
    Skip,
    Record,
    Exhausted,
}

fn admit(counter: &AtomicUsize, total_ns: u64) -> Admission {
    if total_ns < SLOW_NS {
        return Admission::Skip;
    }
    match counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
        (count <= MAX_RECORDS).then_some(count + 1)
    }) {
        Ok(count) if count < MAX_RECORDS => Admission::Record,
        Ok(_) => Admission::Exhausted,
        Err(_) => Admission::Skip,
    }
}

pub(super) fn record(observation: Observation) {
    match admit(&ADMITTED, observation.total_ns) {
        Admission::Skip => {}
        Admission::Exhausted => eprintln!("c07_sample_phase_trace_exhausted limit={MAX_RECORDS}"),
        Admission::Record => eprintln!(
            "c07_sample_phase_trace pid={} elapsed_ns={} total_ns={} host_ns={} rpc_ns={} memory_ns={} enforcement_ns={}",
            observation.pid,
            observation.elapsed_ns,
            observation.total_ns,
            observation.host_ns,
            observation.rpc_ns,
            observation.memory_ns,
            observation.enforcement_ns,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_sample_trace_is_thresholded_and_bounded() {
        let count = AtomicUsize::new(0);
        assert_eq!(admit(&count, SLOW_NS - 1), Admission::Skip);
        for _ in 0..MAX_RECORDS {
            assert_eq!(admit(&count, SLOW_NS), Admission::Record);
        }
        assert_eq!(admit(&count, SLOW_NS), Admission::Exhausted);
        assert_eq!(admit(&count, SLOW_NS), Admission::Skip);
        assert_eq!(count.load(Ordering::Relaxed), MAX_RECORDS + 1);
    }

    #[test]
    fn concurrent_slow_samples_share_one_output_limit() {
        let count = AtomicUsize::new(0);
        let results = std::thread::scope(|scope| {
            let tasks: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| (0..16).map(|_| admit(&count, SLOW_NS)).collect::<Vec<_>>())
                })
                .collect();
            tasks
                .into_iter()
                .flat_map(|task| task.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(
            results.iter().filter(|v| **v == Admission::Record).count(),
            MAX_RECORDS
        );
        assert_eq!(
            results
                .iter()
                .filter(|v| **v == Admission::Exhausted)
                .count(),
            1
        );
    }
}
