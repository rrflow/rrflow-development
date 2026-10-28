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
