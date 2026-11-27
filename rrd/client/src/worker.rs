//! A selected-worker effect is sent once; recovery reads the retained attempt.
use crate::operations::TaskWorkerExecute;
use crate::{RequestOptions, Result, RrdClient, Session};
use rrd_contract::{TaskWorkerOutput, TaskWorkerRequest};

impl RrdClient {
    pub async fn execute_task_worker(
        &self,
        session: &Session,
        request: TaskWorkerRequest,
        options: RequestOptions,
    ) -> Result<TaskWorkerOutput> {
        self.call_operation::<TaskWorkerExecute>(session, request, options)
            .await
    }
}
