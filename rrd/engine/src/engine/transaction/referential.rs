//! Close schema-derived relation effects before intent and receipt publication.
use super::*;

impl RrdEngine {
    pub(in crate::engine) fn resolve_transaction_relation_actions(
        &self,
        read: &ReadStamp,
        commit: RuntimeCommit,
    ) -> Result<RuntimeCommit> {
        let requested = commit.mutations.len();
        let transaction = DataTransaction::new(read.clone(), commit).map_err(core_contract)?;
        let (resolved, evidence) = self
            .storage
            .runtime()
            .resolve_relation_actions(
                &transaction,
                rrflow_db::RuntimeReadBudget::relation_integrity(read.commit_cursor, requested)?,
            )
            .map_err(classify_rejected_commit)?;
        if let Some(evidence) = evidence {
            tracing::debug!(
                source_cursor = read.commit_cursor,
                keys_examined = evidence.keys_examined,
                decoded_bytes = evidence.decoded_bytes,
                requested_mutations = requested,
                resolved_mutations = resolved.mutations.len(),
                "resolved temporal relation actions"
            );
        }
        Ok(resolved)
    }
}
