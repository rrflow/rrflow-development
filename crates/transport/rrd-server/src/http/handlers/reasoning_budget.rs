use super::super::*;
use rrd_contract::{
    ActivateBuiltinEmbedding, ConfigureTaskBudget, ExecuteTaskEmbedding, ReadTaskAttempt,
    ReadTaskBudget, StopTaskBudget,
};
impl AppState {
    pub(in crate::http) fn execute_task_worker(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<rrd_contract::TaskWorkerRequest, _, _>(
            headers,
            body,
            now,
            RrdOperation::WorkerExecute,
            None,
            |envelope, session, token| {
                self.service
                    .execute_task_worker(
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
    pub(in crate::http) fn propose_task_endpoint(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<rrd_contract::EndpointSelectionRequest, _, _>(
            headers,
            body,
            now,
            RrdOperation::EndpointPropose,
            None,
            |envelope, session, token| {
                self.service
                    .propose_task_endpoint(
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
    pub(in crate::http) fn activate_builtin_embedding(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ActivateBuiltinEmbedding, _, _>(
            headers,
            body,
            now,
            RrdOperation::ReasoningWrite,
            None,
            |envelope, session, token| {
                self.service
                    .activate_builtin_embedding(
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
    pub(in crate::http) fn configure_task_budget(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ConfigureTaskBudget, _, _>(
            headers,
            body,
            now,
            RrdOperation::ReasoningWrite,
            None,
            |envelope, session, token| {
                self.service
                    .configure_task_budget(
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
    pub(in crate::http) fn read_task_budget(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ReadTaskBudget, _, _>(
            headers,
            body,
            now,
            RrdOperation::ReasoningRead,
            None,
            |envelope, session, token| {
                self.service
                    .read_task_budget(
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
    pub(in crate::http) fn stop_task_budget(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<StopTaskBudget, _, _>(
            headers,
            body,
            now,
            RrdOperation::ReasoningWrite,
            None,
            |envelope, session, token| {
                self.service
                    .stop_task_budget(
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
    pub(in crate::http) fn execute_task_embedding(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ExecuteTaskEmbedding, _, _>(
            headers,
            body,
            now,
            RrdOperation::ReasoningWrite,
            None,
            |envelope, session, token| {
                self.service
                    .execute_task_embedding(
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
    pub(in crate::http) fn read_task_attempt(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ReadTaskAttempt, _, _>(
            headers,
            body,
            now,
            RrdOperation::ReasoningRead,
            None,
            |envelope, session, token| {
                self.service
                    .read_task_attempt(
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
