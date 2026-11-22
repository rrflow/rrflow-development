//! Definite transaction refusals release leases without obscuring uncertain writes.
use super::*;

impl RrdEngine {
    pub(super) fn finish_rejected_transaction(
        &self,
        transaction_id: &CorrelationId,
        bytes: Vec<u8>,
        state: &mut SessionState,
        context: &EngineControlContext<'_>,
    ) -> Result<()> {
        let record = state
            .transactions
            .get_mut(transaction_id)
            .ok_or(ServiceError::TransactionNotFound)?;
        record.commit_intent = None;
        record.lease.state = TransactionState::Aborted;
        self.replace_session(
            context.session_id,
            bytes,
            state,
            context.now,
            context.action,
            context.request_id,
            context.operation_id,
        )?;
        Ok(())
    }
}
