//! Runtime commits, measured commits, data transactions, restore and previews.

use super::*;

impl<'a> RuntimeRepository<'a> {
    pub fn commit(&self, commit: &RuntimeCommit) -> Result<RuntimeCommitOutcome> {
        self.commit_at_read(commit, None, None, None)
    }

    /// Composition boundary for already encoded runtime and control effects.
    /// Both become visible through one authoritative commit or not at all.
    pub fn commit_with_control_transitions(
        &self,
        commit: &RuntimeCommit,
        transitions: &[ControlTransition],
    ) -> Result<(RuntimeCommitOutcome, Vec<ControlJournalEntry>)> {
        self.commit_with_control_builder(commit, |_, _| Ok::<_, Error>((transitions, ())))
            .map(|(outcome, entries, ())| (outcome, entries))
    }

    /// Builds control checkpoints against the resulting runtime read identity
    /// inside the same physical transaction. The builder receives validated,
    /// staged state, not a published receipt. It must only construct/validate
    /// transitions: external effects or an independent storage commit are not
    /// permitted. It runs once, with no automatic retry.
    ///
    /// Builder, control-CAS or storage-admission rejection drops all staged
    /// effects. Only authoritative commit success returns the caller's value;
    /// uncertain native I/O errors still require ordinary outcome reconciliation.
    pub fn commit_with_control_builder<T, E, C>(
        &self,
        commit: &RuntimeCommit,
        build: impl FnOnce(&ReadStamp, &RuntimeCommitOutcome) -> std::result::Result<(C, T), E>,
    ) -> std::result::Result<(RuntimeCommitOutcome, Vec<ControlJournalEntry>, T), E>
    where
        E: From<Error>,
        C: AsRef<[ControlTransition]>,
    {
        commit.validate().map_err(Error::from)?;
        let mut transaction = self.storage.begin_transaction()?;
        let plan = prepare_semantic_commit(&*transaction, commit, None, None, None)?;
        let outcome = plan.apply(&mut *transaction)?;
        let read = read_stamp_with(&*transaction, &commit.scope)?;
        let (transitions, value) = build(&read, &outcome)?;
        let entries =
            super::super::control::prepare_control_batch(&mut *transaction, transitions.as_ref())?;
        match transaction.commit(Durability::Authoritative) {
            Ok(_) => Ok((outcome, entries, value)),
            Err(Error::TransactionConflict { .. }) => Err(Error::RuntimeConflict {
                expected: commit.expected_cursor,
                actual: self.cursor()?,
            }
            .into()),
            Err(error) => Err(error.into()),
        }
    }

    /// Exact physical size of the batch that
    /// [`Self::commit_with_control_builder`] would write for this commit and
    /// these control transitions, measured by staging both in a transaction
    /// that is never committed. The batch is admitted when the size is at
    /// most [`crate::PHYSICAL_BATCH_PAYLOAD_BYTES`]. Writes that would be refused for
    /// any other reason are refused here too.
    pub fn measure_commit_with_control<E, C>(
        &self,
        commit: &RuntimeCommit,
        build: impl FnOnce(&ReadStamp, &RuntimeCommitOutcome) -> std::result::Result<C, E>,
    ) -> std::result::Result<usize, E>
    where
        E: From<Error>,
        C: AsRef<[ControlTransition]>,
    {
        commit.validate().map_err(Error::from)?;
        let mut transaction = self.storage.begin_transaction()?;
        transaction.measure_only()?;
        let plan = prepare_semantic_commit(&*transaction, commit, None, None, None)?;
        let outcome = plan.apply(&mut *transaction)?;
        let read = read_stamp_with(&*transaction, &commit.scope)?;
        let transitions = build(&read, &outcome)?;
        super::super::control::prepare_control_batch(&mut *transaction, transitions.as_ref())?;
        let bytes = transaction.staged_encoded_bytes();
        transaction.rollback()?;
        Ok(bytes)
    }

    pub fn commit_data_transaction(
        &self,
        transaction: &DataTransaction,
    ) -> Result<RuntimeCommitOutcome> {
        transaction.validate()?;
        self.commit_at_read(&transaction.commit, Some(&transaction.read), None, None)
    }

    /// Commits prepared governed-function receipts through the same physical
    /// transaction as their domain, graph, index, event, outbox, audit, cursor,
    /// and outcome effects. The receipt records are already validated and
    /// runtime-build-bound by `RrdEngine`; this boundary additionally proves
    /// that each one names this exact runtime commit.
    pub fn commit_data_transaction_with_function_receipts(
        &self,
        transaction: &DataTransaction,
        instance: &str,
        receipts: &[FunctionInvocationReceiptRecord],
    ) -> Result<RuntimeCommitOutcome> {
        transaction.validate()?;
        self.commit_at_read(
            &transaction.commit,
            Some(&transaction.read),
            None,
            Some((instance, receipts)),
        )
    }

    pub(crate) fn restore_commit(
        &self,
        commit: &RuntimeCommit,
        audit: &AuditEnvelope,
    ) -> Result<RuntimeCommitOutcome> {
        self.commit_at_read(commit, None, Some(audit), None)
    }

    pub(super) fn commit_at_read(
        &self,
        commit: &RuntimeCommit,
        read: Option<&ReadStamp>,
        archived_audit: Option<&AuditEnvelope>,
        function_receipts: Option<(&str, &[FunctionInvocationReceiptRecord])>,
    ) -> Result<RuntimeCommitOutcome> {
        commit.validate()?;
        let mut transaction = self.storage.begin_transaction()?;
        let plan = prepare_semantic_commit(
            &*transaction,
            commit,
            read,
            archived_audit,
            function_receipts,
        )?;
        let outcome = plan.apply(&mut *transaction)?;
        match transaction.commit(Durability::Authoritative) {
            Ok(_) => Ok(outcome),
            Err(Error::TransactionConflict { .. }) => Err(Error::RuntimeConflict {
                expected: commit.expected_cursor,
                actual: self.cursor()?,
            }),
            Err(error) => Err(error),
        }
    }

    pub fn commit_outcome(&self, commit_id: &str) -> Result<Option<RuntimeCommitOutcome>> {
        let transaction = self.storage.begin_transaction()?;
        get_json(
            &*transaction,
            keyspaces::RUNTIME_COMMITS,
            &keyspaces::runtime_commit_key(commit_id),
        )
    }

    pub fn preview_transaction(
        &self,
        transaction: &DataTransaction,
        valid_at: Millis,
        max_keys: usize,
    ) -> Result<DataTransactionView> {
        transaction.validate()?;
        let storage = self.storage.begin_transaction()?;
        let sources = RuntimeVersionedSource::all();
        let direct = read_versioned_access(
            &*storage,
            &transaction.read,
            &sources,
            read_budget(max_keys, "transaction graph preview")?,
        )?;
        let base = RuntimeGraphSnapshot::from_changes(
            &direct.changes,
            transaction.read.scope.clone(),
            valid_at,
            transaction.read.commit_cursor,
        );
        transaction.preview(&base).map_err(Error::from)
    }

    pub fn preview_data_snapshot(
        &self,
        transaction: &DataTransaction,
        valid_at: Millis,
        max_keys: usize,
    ) -> Result<RuntimeDataSnapshotRead> {
        transaction.validate()?;
        if valid_at == 0 {
            return Err(Error::Substrate(
                "transaction data preview requires non-zero valid time".into(),
            ));
        }
        let storage = self.storage.begin_transaction()?;
        let sources = RuntimeVersionedSource::all();
        let direct = read_versioned_access(
            &*storage,
            &transaction.read,
            &sources,
            read_budget(max_keys, "transaction data preview")?,
        )?;
        let schema = transaction
            .commit
            .mutations
            .iter()
            .filter_map(|mutation| match mutation {
                RuntimeMutation::Schema { registry } => Some(registry.clone()),
                _ => None,
            })
            .next_back()
            .map(Ok)
            .or_else(|| {
                transaction
                    .read
                    .schema_revision
                    .map(|_| schema_at_read(&transaction.read, &direct.changes))
            })
            .transpose()?;
        if let Some(schema) = &schema {
            schema.validate()?;
        } else if !transaction
            .commit
            .mutations
            .iter()
            .all(|mutation| matches!(mutation, RuntimeMutation::Claim { .. }))
        {
            return Err(Error::Substrate(
                "schema-bound transaction preview requires a schema".into(),
            ));
        }
        let prospective_cursor = transaction
            .read
            .commit_cursor
            .checked_add(transaction.commit.mutations.len() as u64)
            .ok_or_else(|| Error::Substrate("transaction preview cursor overflowed".into()))?;
        let commit_id = transaction.commit.digest();
        let selected_versions = selected_version_count(&direct.changes)?;
        let mut changes = direct.changes;
        let mut previous_digest = transaction.read.head_digest.clone();
        for (ordinal, mutation) in transaction.commit.mutations.iter().cloned().enumerate() {
            let cursor = transaction.read.commit_cursor + ordinal as u64 + 1;
            let change = RuntimeChange::committed(
                cursor,
                &transaction.commit,
                &commit_id,
                ordinal as u64,
                mutation,
                previous_digest,
            );
            previous_digest = Some(change.digest.clone());
            changes.push(change);
        }
        let snapshot = if let Some(schema) = &schema {
            RuntimeDataSnapshot::from_changes(
                &changes,
                schema,
                transaction.read.scope.clone(),
                valid_at,
                prospective_cursor,
            )?
        } else {
            RuntimeDataSnapshot::from_claim_changes(
                &changes,
                transaction.read.scope.clone(),
                valid_at,
                prospective_cursor,
            )?
        };
        Ok(RuntimeDataSnapshotRead {
            read: transaction.read.clone(),
            selected_versions,
            read_evidence: direct.evidence,
            snapshot,
        })
    }
}
