//! Ending a served engine: an interrupt or the engine's stop request.
//!
//! Every process that serves an installed engram ends the same way. It serves
//! until it is interrupted or until a stop request addressed to the serving
//! record it published appears, then closes that record. `rrflow serve` and
//! `rrd-server` both call this one function; neither has a stop of its own.

use crate::{HttpError, RrdHttpServer};
use rrd_engine::{ServiceError, ServingRegistration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// The process interrupt, listened for from the moment this is created.
///
/// A server creates one before it announces that it is listening or publishes
/// readiness. An interrupt sent the instant a supervisor reads that line is
/// then already caught, and ends the server through its one clean shutdown.
/// Waiting for the interrupt only once serving starts leaves a gap in which
/// it kills the process outright, with its serving record still published.
pub struct Interrupt {
    #[cfg(unix)]
    signal: tokio::signal::unix::Signal,
    #[cfg(windows)]
    signal: tokio::signal::windows::CtrlC,
}

impl Interrupt {
    /// Starts listening. Call it inside the Tokio runtime that will serve.
    pub fn listen() -> std::io::Result<Self> {
        #[cfg(unix)]
        let signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
        #[cfg(windows)]
        let signal = tokio::signal::windows::ctrl_c()?;
        Ok(Self { signal })
    }

    /// Completes when the process has been interrupted since `listen`.
    pub async fn received(&mut self) {
        let _ = self.signal.recv().await;
    }
}

/// How one published serving ended.
#[derive(Debug)]
pub struct PublishedServingEnd {
    /// True when a stop request ended serving, false for an interrupt.
    pub stop_requested: bool,
    pub served: Result<(), HttpError>,
    /// The record's close: an error if it could not be withdrawn, otherwise
    /// whether a stop request addressed to it was there to remove.
    pub closed: Result<bool, ServiceError>,
}

/// Serves until `interrupt` is received or a stop request addressed to
/// `registration` appears, then runs the one clean shutdown either way: the
/// engine closes with the server, the record is withdrawn and the request
/// addressed to it is removed.
///
/// The repository has no filesystem notification dependency, so the request
/// is found by looking every `stop_poll_interval`. Callers take the interval
/// from a flag whose default is `rrd_engine::DEFAULT_STOP_POLL_INTERVAL_MS`.
pub async fn serve_published(
    server: RrdHttpServer,
    registration: Arc<ServingRegistration>,
    stop_poll_interval: Duration,
    interrupt: Interrupt,
) -> PublishedServingEnd {
    let stop_requested = Arc::new(AtomicBool::new(false));
    let served = server
        .serve_until(interrupt_or_stop_request(
            Arc::clone(&registration),
            stop_poll_interval,
            Arc::clone(&stop_requested),
            interrupt,
        ))
        .await;
    PublishedServingEnd {
        stop_requested: stop_requested.load(Ordering::Relaxed),
        served,
        closed: registration.close(),
    }
}

async fn interrupt_or_stop_request(
    registration: Arc<ServingRegistration>,
    poll_interval: Duration,
    stop_requested: Arc<AtomicBool>,
    mut interrupt: Interrupt,
) {
    let stop_request = async {
        let mut reported = false;
        loop {
            match registration.stop_requested() {
                Ok(true) => return,
                Ok(false) => {}
                Err(error) if !reported => {
                    reported = true;
                    tracing::warn!(
                        target: "rrd_server::stop",
                        error = %error,
                        "the stop request could not be read; an interrupt still stops this server"
                    );
                }
                Err(_) => {}
            }
            tokio::time::sleep(poll_interval).await;
        }
    };
    tokio::select! {
        _ = interrupt.received() => {}
        _ = stop_request => stop_requested.store(true, Ordering::Relaxed),
    }
}
