use super::super::*;

fn storage_diagnostic_span(request_id: &str) -> tracing::Span {
    if tracing::span_enabled!(target: "rrd_server::diagnostic", tracing::Level::DEBUG, request_sha256)
    {
        tracing::debug_span!(
            target: "rrd_server::diagnostic",
            "storage_diagnostics",
            request_sha256 = %sha256_hex(request_id.as_bytes())
        )
    } else {
        tracing::Span::none()
    }
}

impl AppState {
    pub(in crate::http) fn inspect_storage_cursor(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<rrd_contract::InspectStorageCursor, _, _>(
            headers,
            body,
            now,
            RrdOperation::DiagnosticsRead,
            None,
            |envelope, session, token| {
                self.service
                    .inspect_storage_cursor(
                        session,
                        token,
                        &envelope.payload,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn read_storage_diagnostics(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<rrd_contract::ReadStorageDiagnostics, _, _>(
            headers,
            body,
            now,
            RrdOperation::DiagnosticsRead,
            None,
            |envelope, session, token| {
                // Request identifiers are caller supplied; retain only their
                // digest in the opt-in trace used to correlate store timings.
                let span = storage_diagnostic_span(envelope.context.request_id.as_str());
                let _entered = span.enter();
                let timed = tracing::event_enabled!(target: "rrd_server::diagnostic", tracing::Level::DEBUG, service_elapsed_ns, success, message);
                let started = timed.then(std::time::Instant::now);
                let result = self.service.read_storage_diagnostics(
                    session,
                    token,
                    &envelope.payload,
                    now,
                    envelope.context.request_id.as_str(),
                    envelope.context.operation_id.as_str(),
                );
                if let Some(started) = started {
                    tracing::debug!(
                        target: "rrd_server::diagnostic",
                        service_elapsed_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
                        success = result.is_ok(),
                        "storage diagnostic service returned"
                    );
                }
                result.map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn read_diagnostic_snapshot(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ReadDiagnosticSnapshot, _, _>(
            headers,
            body,
            now,
            RrdOperation::DiagnosticsRead,
            None,
            |envelope, session, token| {
                self.service
                    .read_diagnostic_snapshot(
                        session,
                        token,
                        &envelope.payload,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rrd_store::{RrflowKvStore, StorageEngine};
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn diagnostic_span_correlates_real_store_event_without_raw_request_id() {
        let directory = tempfile::tempdir().unwrap();
        let store = RrflowKvStore::open(&directory.path().join("trace")).unwrap();
        let output = Arc::new(Mutex::new(Vec::new()));
        let writer_output = output.clone();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(move || Capture(writer_output.clone()))
            .finish();
        let id = "secret-request-sentinel";
        tracing::subscriber::with_default(subscriber, || {
            assert!(tracing::enabled!(target: "rrd_server::diagnostic", tracing::Level::DEBUG));
            assert!(tracing::enabled!(target: "rrd_store::diagnostic", tracing::Level::DEBUG));
            let span = storage_diagnostic_span(id);
            let _entered = span.enter();
            store.physical_store_evidence().unwrap();
        });
        let bytes = output.lock().unwrap().clone();
        let raw = std::str::from_utf8(&bytes).unwrap();
        assert!(!raw.contains(id));
        let event: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(event["span"]["request_sha256"], sha256_hex(id.as_bytes()));
        assert_eq!(event["target"], "rrd_store::diagnostic");
        assert!(event["fields"]["mutex_wait_ns"].is_u64());
    }
}
