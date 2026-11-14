//! Bounded, numeric-only attribution for slow completed resource acquisitions.
//! This does not replace resource samples or change their coverage validator.
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

const SLOW_NS: u64 = 200_000_000;
const MAX_RECORDS: usize = 64;
static ADMITTED: AtomicUsize = AtomicUsize::new(0);
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(0);

pub(super) fn next_request_id() -> Result<String, &'static str> {
    request_id(&NEXT_REQUEST)
}

fn request_id(counter: &AtomicU64) -> Result<String, &'static str> {
    let ordinal = counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| "resource sample request counter exhausted")?;
    Ok(format!("capacity-resource-{ordinal:016x}"))
}

pub(super) struct Observation<'a> {
    pub request_id: &'a str,
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

pub(super) fn record(observation: Observation<'_>) {
    match admit(&ADMITTED, observation.total_ns) {
        Admission::Skip => {}
        Admission::Exhausted => eprintln!("c07_sample_phase_trace_exhausted limit={MAX_RECORDS}"),
        Admission::Record => eprintln!("{}", format_record(&observation)),
    }
}

fn format_record(observation: &Observation<'_>) -> String {
    let request_sha256 = rrd_core::digest::sha256_hex(observation.request_id.as_bytes());
    format!(
            "c07_sample_phase_trace request_sha256={} pid={} elapsed_ns={} total_ns={} host_ns={} rpc_ns={} memory_ns={} enforcement_ns={}",
            request_sha256,
            observation.pid,
            observation.elapsed_ns,
            observation.total_ns,
            observation.host_ns,
            observation.rpc_ns,
            observation.memory_ns,
            observation.enforcement_ns,
        )
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

    #[test]
    fn concurrent_request_ids_are_unique_and_exhaustion_fails_closed() {
        let counter = AtomicU64::new(0);
        let ids = std::thread::scope(|scope| {
            let tasks: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        (0..100)
                            .map(|_| request_id(&counter).unwrap())
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            tasks
                .into_iter()
                .flat_map(|task| task.join().unwrap())
                .collect::<std::collections::BTreeSet<_>>()
        });
        assert_eq!(ids.len(), 800);
        assert_eq!(counter.load(Ordering::Relaxed), 800);
        let exhausted = AtomicU64::new(u64::MAX);
        assert!(request_id(&exhausted).is_err());
    }

    #[test]
    fn slow_sample_record_contains_only_the_server_compatible_request_digest() {
        let id = request_id(&AtomicU64::new(0)).unwrap();
        let line = format_record(&Observation {
            request_id: &id,
            pid: 1,
            elapsed_ns: 2,
            total_ns: 3,
            host_ns: 4,
            rpc_ns: 5,
            memory_ns: 6,
            enforcement_ns: 7,
        });
        assert!(line.contains(
            "request_sha256=503bb869c71edbc697d959533faf8a411a9a6ba85b2a73e0af9430d2f4d51fb2"
        ));
        assert!(!line.contains(&id));
    }
}
