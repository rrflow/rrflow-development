//! A selected-worker effect is sent once; recovery reads the retained attempt.
use crate::error::contract;
use crate::generated_operations as generated;
use crate::{Error, RequestOptions, Result, RrdClient, Session};
use rrd_contract::{TaskWorkerOutput, TaskWorkerRequest};

impl RrdClient {
    pub async fn execute_task_worker(
        &self,
        session: &Session,
        request: TaskWorkerRequest,
        options: RequestOptions,
    ) -> Result<TaskWorkerOutput> {
        request.validate().map_err(contract)?;
        // Reuse the existing transport and catalogue projection. The clone shares
        // its connection pool; this effect never inherits automatic retries.
        let mut once = self.clone();
        once.config.max_attempts = 1;
        let output = once
            .generated_attunement_call::<generated::TaskWorkerExecute>(session, &request, options)
            .await?;
        output.validate().map_err(contract)?;
        if output.scope != request.scope
            || output.task_id != request.task_id
            || output.attempt_id != request.attempt_id
            || output.selection_attempt_id != request.selection_attempt_id
            || output.selection_sha256 != request.expected_selection_sha256
            || output.task_sha256 != request.expected_task_sha256
            || output.configuration_sha256 != request.expected_configuration_sha256
        {
            return Err(Error::ResponseIdentityMismatch);
        }
        Ok(output)
    }
}
