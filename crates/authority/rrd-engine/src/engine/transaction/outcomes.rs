//! Definite transaction refusals release leases without obscuring uncertain writes.
use super::*;

impl RrdEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn finish_rejected_transaction(
        &self,
        session_id: &CorrelationId,
        transaction_id: &CorrelationId,
        bytes: Vec<u8>,
        state: &mut SessionState,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<()> {
        let record = state
            .transactions
            .get_mut(transaction_id)
            .ok_or(ServiceError::TransactionNotFound)?;
        record.commit_intent = None;
        record.lease.state = TransactionState::Aborted;
        self.replace_session(
            session_id,
            bytes,
            state,
            now,
            "transaction.commit_rejected",
            request_id,
            operation_id,
        )?;
        Ok(())
    }
}
