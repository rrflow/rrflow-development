use super::super::*;
use rrd_contract::{
    AttunementCancelResult, AttunementReceipt, AttunementReceiptReadResult,
    AttunementSourceVerification, CancelAttunement, ReadAttunement, ReadAttunementReceipt,
};
use rrd_engine::AttunementCommand;

impl AppState {
    pub(in crate::http) fn cancel_attunement(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<CancelAttunement, _, _>(
            headers,
            body,
            now,
            RrdOperation::EstateAdmin,
            None,
            |envelope, session, token| {
                self.service
                    .control_attunement(
                        session,
                        token,
                        &AttunementCommand::Cancel {
                            request: envelope.payload.clone(),
                        },
                        &envelope.context,
                        now,
                    )
                    .map(|result| AttunementCancelResult {
                        status: result.status,
                        idempotent_replay: result.idempotent_replay,
                    })
                    .map_err(api_error)
            },
        )
    }
    pub(in crate::http) fn read_attunement_receipt(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ReadAttunementReceipt, _, _>(
            headers,
            body,
            now,
            RrdOperation::EstateRead,
            None,
            |envelope, session, token| {
                envelope.payload.validate().map_err(|error| {
                    ApiError::new(ErrorCode::InvalidArgument, error.to_string(), false)
                })?;
                self.service
                    .read_attunement_receipt(
                        session,
                        token,
                        &envelope.payload.job_id,
                        &envelope.payload.idempotency_key,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map(|receipt| AttunementReceiptReadResult {
                        receipt: receipt.map(|view| AttunementReceipt {
                            principal_id: view.principal_id,
                            operation_sha256: view.operation_sha256,
                            plan: view.plan,
                            status: view.status,
                            source_verification: AttunementSourceVerification::NotChecked,
                        }),
                    })
                    .map_err(api_error)
            },
        )
    }
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
