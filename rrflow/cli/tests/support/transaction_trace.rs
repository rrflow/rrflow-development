//! Bounded, redacted transaction diagnostics for an owned installed process.
//! This is characterization evidence, not durable audit or qualification.
use super::process::RedactedStderr;
use serde::Serialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

const MAX_LINE_BYTES: usize = 16 * 1024;
/// Fixed safety ceiling; callers can choose a smaller campaign-wide budget.
pub const MAX_RETAINED_EVENTS: usize = 1024;

#[derive(Clone)]
pub struct Capture {
    state: Arc<Mutex<Summary>>,
}

#[derive(Clone, Debug, Serialize)]
struct Event {
    pid: u32,
    observer_ordinal: u64,
    request_sha256: String,
    outcome: String,
    last_stage: String,
    validation_ns: u64,
    admission_ns: u64,
    authentication_ns: u64,
    replay_ns: u64,
    snapshot_ns: u64,
    publication_ns: u64,
    total_ns: u64,
}

#[derive(Clone, Default, Serialize)]
struct Summary {
    budget_events: usize,
    registered_observers: u64,
    complete_lines: u64,
    valid_events: u64,
    rejected_lines: u64,
    oversized_lines: u64,
    incomplete_lines: u64,
    finished_readers: u64,
    failed_readers: u64,
    dropped_events: u64,
    events: Vec<Event>,
}

impl Capture {
    pub fn new(budget_events: usize) -> Result<Self, &'static str> {
        if budget_events == 0 || budget_events > MAX_RETAINED_EVENTS {
            return Err("transaction trace event budget outside supported range");
        }
        Ok(Self {
            state: Arc::new(Mutex::new(Summary {
                budget_events,
                ..Summary::default()
            })),
        })
    }

    pub fn observer(&self) -> Box<dyn RedactedStderr> {
        let ordinal = {
            let mut state = self.state.lock().unwrap();
            state.registered_observers = state
                .registered_observers
                .checked_add(1)
                .expect("transaction trace observer ordinal exhausted");
            state.registered_observers
        };
        Box::new(Observer {
            capture: self.clone(),
            pid: 0,
            ordinal,
            line: Vec::new(),
            oversized: false,
        })
    }

    /// Snapshot after the owned readers have been reaped. This is not an
    /// assertion that the process emitted or the pipe delivered every event.
    pub fn report(&self) -> Value {
        serde_json::to_value(self.state.lock().unwrap().clone()).unwrap()
    }
}

struct Observer {
    capture: Capture,
    pid: u32,
    ordinal: u64,
    line: Vec<u8>,
    oversized: bool,
}

impl Observer {
    fn complete_line(&mut self) {
        let mut summary = self.capture.state.lock().unwrap();
        summary.complete_lines = summary.complete_lines.saturating_add(1);
        if self.oversized {
            summary.oversized_lines = summary.oversized_lines.saturating_add(1);
        } else if let Some(event) = parse_event(self.pid, self.ordinal, &self.line) {
            summary.valid_events = summary.valid_events.saturating_add(1);
            if summary.events.len() < summary.budget_events {
                summary.events.push(event);
            } else {
                summary.dropped_events = summary.dropped_events.saturating_add(1);
                // Keep the most useful slow samples without allowing output or
                // memory to grow with the number of requests or restarts.
                if let Some((index, minimum)) = summary
                    .events
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, value)| value.total_ns)
                {
                    if event.total_ns > minimum.total_ns {
                        summary.events[index] = event;
                    }
                }
            }
        } else {
            summary.rejected_lines = summary.rejected_lines.saturating_add(1);
        }
        self.line.clear();
        self.oversized = false;
    }
}

impl RedactedStderr for Observer {
    fn set_pid(&mut self, pid: u32) {
        self.pid = pid;
    }

    fn consume(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            if byte == b'\n' {
                self.complete_line();
            } else if !self.oversized {
                if self.line.len() == MAX_LINE_BYTES {
                    self.line.clear();
                    self.oversized = true;
                } else {
                    self.line.push(byte);
                }
            }
        }
    }

    fn finish(&mut self) {
        let mut summary = self.capture.state.lock().unwrap();
        summary.finished_readers = summary.finished_readers.saturating_add(1);
        if self.oversized || !self.line.is_empty() {
            summary.incomplete_lines = summary.incomplete_lines.saturating_add(1);
            if self.oversized {
                summary.oversized_lines = summary.oversized_lines.saturating_add(1);
            }
        }
        self.line.clear();
        self.oversized = false;
    }

    fn capture_failed(&mut self) {
        let mut summary = self.capture.state.lock().unwrap();
        summary.failed_readers = summary.failed_readers.saturating_add(1);
    }
}

fn parse_event(pid: u32, observer_ordinal: u64, bytes: &[u8]) -> Option<Event> {
    if pid == 0 {
        return None;
    }
    let value: Value = serde_json::from_slice(bytes).ok()?;
    if value.get("target")?.as_str()? != "rrd_engine::transaction_diagnostic" {
        return None;
    }
    let fields = value.get("fields")?.as_object()?;
    let digest = fields.get("request_sha256")?.as_str()?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let outcome = fields.get("outcome")?.as_str()?;
    if !["created", "replayed", "error"].contains(&outcome) {
        return None;
    }
    let stage = fields.get("last_stage")?.as_str()?;
    if ![
        "validation",
        "admission",
        "authentication",
        "replay",
        "snapshot",
        "publication",
    ]
    .contains(&stage)
    {
        return None;
    }
    let number = |key| fields.get(key)?.as_u64();
    Some(Event {
        pid,
        observer_ordinal,
        request_sha256: digest.to_owned(),
        outcome: outcome.to_owned(),
        last_stage: stage.to_owned(),
        validation_ns: number("validation_ns")?,
        admission_ns: number("admission_ns")?,
        authentication_ns: number("authentication_ns")?,
        replay_ns: number("replay_ns")?,
        snapshot_ns: number("snapshot_ns")?,
        publication_ns: number("publication_ns")?,
        total_ns: number("total_ns")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use std::time::Duration;

    fn event(total: u64) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "target":"rrd_engine::transaction_diagnostic",
            "fields":{"request_sha256":"a".repeat(64),"outcome":"created",
                "last_stage":"publication","validation_ns":1,"admission_ns":2,
                "authentication_ns":3,"replay_ns":4,"snapshot_ns":5,
                "publication_ns":6,"total_ns":total,"secret":"do-not-copy"}
        }))
        .unwrap()
    }

    #[test]
    fn bounded_across_processes_and_redacts_invalid_lines() {
        let capture = Capture::new(2).unwrap();
        let mut first = capture.observer();
        first.set_pid(11);
        for n in [10, 30] {
            let mut data = event(n);
            data.push(b'\n');
            first.consume(&data);
        }
        first.consume(b"foreign secret\n");
        first.consume(&vec![b'x'; MAX_LINE_BYTES + 1]);
        first.consume(b"\npartial secret");
        first.finish();
        let mut second = capture.observer();
        second.set_pid(12);
        let mut data = event(40);
        data.push(b'\n');
        second.consume(&data);
        second.finish();
        let report = capture.report();
        assert_eq!(report["valid_events"], 3);
        assert_eq!(report["dropped_events"], 1);
        assert_eq!(report["rejected_lines"], 1);
        assert_eq!(report["oversized_lines"], 1);
        assert_eq!(report["incomplete_lines"], 1);
        let text = report.to_string();
        assert!(!text.contains("secret"));
        let totals: Vec<_> = report["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["total_ns"].as_u64().unwrap())
            .collect();
        assert!(totals.contains(&30) && totals.contains(&40));
        assert!(report["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["pid"] == 12));
        assert!(Capture::new(0).is_err());
        assert!(Capture::new(MAX_RETAINED_EVENTS + 1).is_err());
    }

    #[test]
    fn rejects_wrong_target_digest_enums_and_non_numeric_timings() {
        let good: Value = serde_json::from_slice(&event(10)).unwrap();
        assert!(parse_event(3, 1, &serde_json::to_vec(&good).unwrap()).is_some());
        let mut bad = good.clone();
        bad["target"] = json!("other");
        assert!(parse_event(3, 1, &serde_json::to_vec(&bad).unwrap()).is_none());
        bad = good.clone();
        bad["fields"]["request_sha256"] = json!("g".repeat(64));
        assert!(parse_event(3, 1, &serde_json::to_vec(&bad).unwrap()).is_none());
        bad = good.clone();
        bad["fields"]["outcome"] = json!("secret");
        assert!(parse_event(3, 1, &serde_json::to_vec(&bad).unwrap()).is_none());
        bad = good.clone();
        bad["fields"]["total_ns"] = json!(-1);
        assert!(parse_event(3, 1, &serde_json::to_vec(&bad).unwrap()).is_none());
        assert!(parse_event(0, 1, &event(10)).is_none());
    }

    #[test]
    fn trace_stream_flood_fixture() {
        if std::env::var_os("RRFLOW_TEST_REDACTED_FLOOD").is_none() {
            return;
        }
        let mut stderr = std::io::stderr().lock();
        let mut line = event(10);
        line.push(b'\n');
        for _ in 0..6_000 {
            stderr.write_all(&line).unwrap();
        }
        stderr.flush().unwrap();
    }

    #[test]
    fn child_stream_drains_past_normal_capture_limit_without_retaining_raw_data() {
        assert!((event(10).len() + 1) * 6_000 > 1024 * 1024);
        let capture = Capture::new(2).unwrap();
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "transaction_trace::tests::trace_stream_flood_fixture",
                "--nocapture",
            ])
            .env("RRFLOW_TEST_REDACTED_FLOOD", "1");
        let mut child = super::super::process::Process::start_with_redacted_stderr(
            &mut command,
            Duration::from_secs(60),
            1024 * 1024,
            None,
            capture.observer(),
        )
        .unwrap();
        let output = child.finish().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let report = capture.report();
        assert_eq!(report["valid_events"], 6_000);
        assert_eq!(report["dropped_events"], 5_998);
        assert_eq!(report["failed_readers"], 0);
        assert_eq!(report["events"].as_array().unwrap().len(), 2);
        assert!(!report.to_string().contains("do-not-copy"));
    }

    #[test]
    fn observer_panic_fails_and_reaps_owned_child() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct PanickingObserver(Arc<AtomicBool>);
        impl RedactedStderr for PanickingObserver {
            fn set_pid(&mut self, _: u32) {}
            fn consume(&mut self, _: &[u8]) {
                panic!("injected observer failure");
            }
            fn finish(&mut self) {}
            fn capture_failed(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let failed = Arc::new(AtomicBool::new(false));
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "transaction_trace::tests::trace_stream_flood_fixture",
                "--nocapture",
            ])
            .env("RRFLOW_TEST_REDACTED_FLOOD", "1");
        let mut child = super::super::process::Process::start_with_redacted_stderr(
            &mut command,
            Duration::from_secs(60),
            1024 * 1024,
            None,
            Box::new(PanickingObserver(failed.clone())),
        )
        .unwrap();
        let error = child.finish().unwrap_err();
        assert!(error.contains("redacted stderr observer panicked"));
        assert!(failed.load(Ordering::Acquire));
        assert!(!child.running());
        assert!(!error.contains("do-not-copy"));
    }
}
