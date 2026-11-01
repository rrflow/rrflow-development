use super::super::*;
use rrd_contract::{DeliverObservation, PrepareObservationDelivery, ReadObservationDelivery};

impl AppState {
    pub(in crate::http) fn prepare_observation_delivery(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<PrepareObservationDelivery, _, _>(
            headers,
            body,
            now,
            RrdOperation::ObservationPrepare,
            None,
            |envelope, session, token| {
                self.service
                    .prepare_observation_delivery(
                        session,
                        token,
                        &envelope.payload,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }
    pub(in crate::http) fn deliver_observation(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<DeliverObservation, _, _>(
            headers,
            body,
            now,
            RrdOperation::ObservationDeliver,
            None,
            |envelope, session, token| {
                self.service
                    .deliver_observation(
                        session,
                        token,
                        &envelope.payload,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }
    pub(in crate::http) fn read_observation_delivery(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ReadObservationDelivery, _, _>(
            headers,
            body,
            now,
            RrdOperation::ObservationRead,
            None,
            |envelope, session, token| {
                self.service
                    .read_observation_delivery(
                        session,
                        token,
                        &envelope.payload,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }
}
