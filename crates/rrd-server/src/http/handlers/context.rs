use super::super::*;

impl AppState {
    pub(in crate::http) fn assemble_context(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<AssembleContext, _, _>(
            headers,
            body,
            now,
            RrdOperation::MemoryContextRead,
            None,
            |envelope, session, token| {
                self.service
                    .assemble_context(
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
