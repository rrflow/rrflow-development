use super::super::*;
use rrd_contract::ReadAttunement;

impl AppState {
    pub(in crate::http) fn read_attunement(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ReadAttunement, _, _>(
            headers,
            body,
            now,
            RrdOperation::EstateRead,
            None,
            |envelope, session, token| {
                self.service
                    .read_attunement_detail(
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
