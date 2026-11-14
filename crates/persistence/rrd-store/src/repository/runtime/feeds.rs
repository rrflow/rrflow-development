//! Change, outbox, projection, index-source and vector delta feeds, and the audit page.

use super::*;

impl<'a> RuntimeRepository<'a> {
    pub fn changes_since(
        &self,
        after: u64,
        limit: usize,
        scope: Option<&ScopeId>,
    ) -> Result<RuntimeChangePage> {
        let transaction = self.storage.begin_transaction()?;
        let head = read_sequence(&*transaction, &keyspaces::runtime_cursor_key())?;
        change_page(&*transaction, head, after, limit, scope)
    }

    pub fn outbox_since(&self, after: u64, limit: usize) -> Result<Vec<ProjectionWork>> {
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime outbox page limit must be greater than zero".into(),
            ));
        }
        let transaction = self.storage.begin_transaction()?;
        let start =
            keyspaces::runtime_outbox_key(after.checked_add(1).ok_or(Error::SequenceOverflow)?);
        scan_space_from(&*transaction, keyspaces::RUNTIME_OUTBOX, &start)?
            .into_iter()
            .take(limit)
            .map(|(_, bytes)| {
                let work: ProjectionWork = serde_json::from_slice(&bytes)?;
                work.validate()?;
                Ok(work)
            })
            .collect()
    }

    /// Durable projection-work identities committed with their canonical
    /// mutations. These are distinct from outbox delivery records even though
    /// both initially carry the same validated work identity; C-03b adds the
    /// family-specific old/new index deltas behind these identities.
    pub fn projection_deltas_since(&self, after: u64, limit: usize) -> Result<Vec<ProjectionWork>> {
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime projection-delta page limit must be greater than zero".into(),
            ));
        }
        let transaction = self.storage.begin_transaction()?;
        let start = keyspaces::runtime_projection_delta_start(
            after.checked_add(1).ok_or(Error::SequenceOverflow)?,
        );
        scan_space_from(&*transaction, keyspaces::RUNTIME_PROJECTION_DELTAS, &start)?
            .into_iter()
            .take(limit)
            .map(|(_, bytes)| {
                let work: ProjectionWork = serde_json::from_slice(&bytes)?;
                work.validate()?;
                Ok(work)
            })
            .collect()
    }

    /// Exact old/new record fields committed for one schema-bound native
    /// index. Consumers checkpoint `source_cursor`; they never replay the full
    /// runtime log to infer maintenance work.
    pub fn index_source_deltas_since(
        &self,
        scope: &ScopeId,
        index: &rrd_core::ProjectionId,
        after: u64,
        limit: usize,
    ) -> Result<Vec<IndexSourceDelta>> {
        let transaction = self.storage.begin_transaction()?;
        index_source_deltas(&*transaction, scope, index, after, limit)
    }

    /// One collection/name/field source's exact vector-version pointer
    /// changes in (`after`, read cursor], with the authenticated history of
    /// every identity they name, under one read stamp and key budget.
    /// Accelerator builders and the vector planner consume this outside the
    /// commit path; every pointer is checked against the version it names.
    pub fn read_vector_deltas(
        &self,
        read: &ReadStamp,
        source: &VectorSourceAddress,
        after: u64,
        budget: RuntimeReadBudget,
    ) -> Result<RuntimeVectorDeltaRead> {
        let transaction = self.storage.begin_transaction()?;
        read_vector_deltas_access(&*transaction, read, source, after, budget)
    }

    pub fn audit(&self, commit_id: &str) -> Result<Option<AuditEnvelope>> {
        let transaction = self.storage.begin_transaction()?;
        let audit: Option<AuditEnvelope> = get_json(
            &*transaction,
            keyspaces::RUNTIME_AUDIT,
            &keyspaces::runtime_audit_key(commit_id),
        )?;
        if let Some(value) = &audit {
            value.validate()?;
        }
        Ok(audit)
    }
}
