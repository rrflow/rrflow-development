//! RRFlow operator surface.
//!
//! Runtime operator invocations that use the legacy command path are recorded
//! with their trigger, arguments, outcome, and duration. Installed-lifecycle
//! bootstrap and read-only commands do not open a second/default database only
//! to create that record: install apply persists its engine-owned receipt,
//! authenticated service requests are audited by `RrdEngine`, and process
//! lifecycle uses opt-in traces until D-01 adds durable start/stop receipts.
//!
//! Recording for the legacy runtime path wraps execution in one place.
//!
//! This is an outer adapter and therefore may read a clock, which the kernel
//! must not. The clock is read once, here, and passed inward.

mod command;
mod dev;
mod installed;

use clap::Parser;
use command::Cli;
use rrd_engine::{OperatorInvocationInput, Reader, RrdEngine};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the Unix epoch")
        .as_millis() as u64
}

/// Step T: traces are enableable, off by default, and free when off — no
/// subscriber is installed unless `RRFLOW_TRACE` is set, and the `tracing`
/// macros compile to a branch on a static in that case. Output goes to
/// stderr always: stdout is the answer channel and must never carry
/// diagnostics. `RRFLOW_TRACE_FORMAT=json`
/// switches to machine-readable lines for the observatory path.
fn install_tracing() {
    let Ok(filter) = std::env::var("RRFLOW_TRACE") else {
        return;
    };
    let builder = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr);
    if std::env::var("RRFLOW_TRACE_FORMAT").as_deref() == Ok("json") {
        builder.json().init();
    } else {
        builder.init();
    }
}

fn main() -> std::process::ExitCode {
    let process_started = Instant::now();
    install_tracing();
    let parse_started = Instant::now();
    let cli = Cli::parse();
    let parse_ms = parse_started.elapsed().as_millis() as u64;
    let now = now_millis();

    if let Some(result) = installed::execute(&cli.command, now, cli.json) {
        return finish_unrecorded(result);
    }

    let db = cli
        .db
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from(".rrflow/rrd"));

    if let Some(result) = command::execute_offline(&db, &cli.command, now, cli.json) {
        return finish_unrecorded(result);
    }

    let open_started = Instant::now();
    let store = match RrdEngine::open_project_store(&db) {
        Ok(store) => store,
        Err(error) => {
            tracing::error!(
                target: "rrflow::control_plane",
                phase = "store.open",
                elapsed_ms = open_started.elapsed().as_millis() as u64,
                total_ms = process_started.elapsed().as_millis() as u64,
                path = %db.display(),
                error = %error,
                "RRFlow control-plane phase failed"
            );
            eprintln!("cannot open database at {}: {error}", db.display());
            return std::process::ExitCode::from(2);
        }
    };
    let open_ms = open_started.elapsed().as_millis() as u64;

    let reader = match Reader::new(cli.reader.clone()) {
        Ok(reader) => reader,
        Err(error) => {
            eprintln!("invalid reader identity: {error}");
            return std::process::ExitCode::from(2);
        }
    };

    let execute_started = Instant::now();
    let result = command::execute(&store, &cli.command, &reader, now, cli.json);
    let execute_ms = execute_started.elapsed().as_millis() as u64;
    // The durable invocation duration includes parsing and database startup.
    // Excluding store open hid the dominant control-plane cost on real estates.
    let duration_ms = process_started.elapsed().as_millis() as u64;

    let (outcome, error_detail) = command::outcome_of(&result);
    // A successful execution may still carry a detail line; a failure's
    // detail is the error.
    let detail = error_detail.or_else(|| result.as_ref().ok().and_then(|e| e.detail.clone()));

    // The invocation is recorded whether the command succeeded or failed. A log
    // containing only successes would misrepresent operator activity.
    let record_started = Instant::now();
    if let Err(error) = store.record_operator_invocation(OperatorInvocationInput {
        at: now,
        trigger: cli.command.trigger(),
        command: cli.command.name(),
        arguments: &cli.command.arguments(),
        outcome,
        duration_ms,
        detail,
    }) {
        // Recording is the point of this surface, so a failure to record is
        // reported rather than swallowed, even when the command itself worked.
        eprintln!("warning: invocation was not recorded: {error}");
    }
    let record_ms = record_started.elapsed().as_millis() as u64;
    tracing::info!(
        target: "rrflow::control_plane",
        command = cli.command.name(),
        parse_ms,
        open_ms,
        execute_ms,
        record_ms,
        total_ms = process_started.elapsed().as_millis() as u64,
        "RRFlow control-plane phases completed"
    );

    match result {
        Ok(execution) => {
            // An empty answer prints nothing at all.
            if !execution.text.is_empty() {
                println!("{}", execution.text);
            }
            if execution.success {
                std::process::ExitCode::SUCCESS
            } else {
                std::process::ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn finish_unrecorded(
    result: Result<command::Execution, Box<dyn std::error::Error>>,
) -> std::process::ExitCode {
    match result {
        Ok(execution) => {
            if !execution.text.is_empty() {
                println!("{}", execution.text);
            }
            if execution.success {
                std::process::ExitCode::SUCCESS
            } else {
                std::process::ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
