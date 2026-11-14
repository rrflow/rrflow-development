//! The semantic commit plan: preparation and schema, reference and retirement validation.

use super::*;

/// A validated semantic commit plan that has not been published.
///
/// Encoded writes are private to ordinary repository callers, which can only
/// apply the complete plan. The temporary rrflowKV Raft bridge still receives
/// a raw operation vector so it can append coordinator metadata; that known
/// distributed-authority gap remains explicitly open and must be removed by
/// its owning convergence package.
#[derive(Debug)]
pub struct SemanticCommitPlan {
    outcome: RuntimeCommitOutcome,
    writes: TransactionWriteSet,
}

impl SemanticCommitPlan {
    pub fn outcome(&self) -> &RuntimeCommitOutcome {
        &self.outcome
    }

    pub(crate) fn apply(
        self,
        transaction: &mut dyn StorageTransaction,
    ) -> Result<RuntimeCommitOutcome> {
        let Self { outcome, writes } = self;
        for (key, value) in writes.into_mutations() {
            match value {
                Some(value) => transaction.put(key, value)?,
                None => transaction.delete(key)?,
            }
        }
        Ok(outcome)
    }

    pub(crate) fn into_encoded_writes(
        self,
    ) -> (RuntimeCommitOutcome, BTreeMap<Vec<u8>, Option<Vec<u8>>>) {
        (self.outcome, self.writes.into_mutations())
    }

    pub(crate) fn put_bytes(&mut self, space: Space, key: &[u8], value: Vec<u8>) -> Result<()> {
        self.writes.put(checked_key(space, key)?, value)
    }

    pub(crate) fn put_json<T: serde::Serialize>(
        &mut self,
        space: Space,
        key: &[u8],
        value: &T,
    ) -> Result<()> {
        self.put_bytes(space, key, serde_json::to_vec(value)?)
    }

    fn put_sequence(&mut self, key: &[u8], sequence: u64) -> Result<()> {
        self.put_bytes(keyspaces::META, key, sequence.to_string().into_bytes())
    }

    pub(crate) fn delete(&mut self, space: Space, key: &[u8]) -> Result<()> {
        self.writes.delete(checked_key(space, key)?)
    }
}

pub(crate) fn prepare_semantic_commit(
    reader: &(impl AccessRead + ?Sized),
    commit: &RuntimeCommit,
    read: Option<&ReadStamp>,
    archived_audit: Option<&AuditEnvelope>,
    function_receipts: Option<(&str, &[FunctionInvocationReceiptRecord])>,
) -> Result<SemanticCommitPlan> {
    commit.validate()?;
    if read.is_some() && archived_audit.is_some() {
        return Err(Error::Archive(
            "archive replay cannot also supply a live read stamp".into(),
        ));
    }
    if let Some(audit) = archived_audit {
        audit.validate()?;
        if let Some(read) = &audit.read {
            read.validate()?;
        }
    } else if let Some(read) = read {
        validate_read_stamp(reader, read)?;
    }

    let commit_id = commit.digest();
    let start = read_sequence(reader, &keyspaces::runtime_cursor_key())?;
    if start != commit.expected_cursor {
        return Err(Error::RuntimeConflict {
            expected: commit.expected_cursor,
            actual: start,
        });
    }
    let mut accumulator = current_accumulator(reader, start)?;
    validate_retirement_targets(reader, commit, start)?;
    let effective_schema = validate_schema_and_objects(reader, commit)?;
    validate_references(reader, commit)?;
    let catalogue_revision =
        read_sequence(reader, &keyspaces::catalog_revision_key(&commit.scope))?;

    let claim_count = commit
        .mutations
        .iter()
        .filter(|mutation| matches!(mutation, RuntimeMutation::Claim { .. }))
        .count();
    let claim_start = read_sequence(reader, &keyspaces::sequence_watermark_key())?;
    let mut claim_sequence = claim_start;
    let mut cursor = start;
    let mut previous_digest = get(
        reader,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    let previous_audit_digest = get(
        reader,
        keyspaces::META,
        &keyspaces::runtime_last_audit_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());

    let mut plan = SemanticCommitPlan {
        outcome: RuntimeCommitOutcome {
            commit_id: commit_id.clone(),
            first_cursor: 0,
            last_cursor: 0,
            count: commit.mutations.len(),
            first_claim_sequence: None,
            last_claim_sequence: None,
            outbox_count: 0,
        },
        writes: {
            let mut writes = TransactionWriteSet::default();
            if reader.measuring() {
                writes.measure_only()?;
            }
            writes
        },
    };
    let mut outbox_count = 0;
    let mut staged_relations = BTreeMap::<RuntimeRef, Option<RuntimeRelation>>::new();
    for (ordinal, mutation) in commit.mutations.iter().enumerate() {
        if let RuntimeMutation::Claim { claim } = mutation {
            claim.validate()?;
            claim_sequence = claim_sequence
                .checked_add(1)
                .ok_or(Error::SequenceOverflow)?;
            let claim_key = keyspaces::claim_key(
                &claim.subject,
                &claim.predicate,
                claim.valid_from,
                claim.tx_time,
            );
            plan.put_json(keyspaces::CLAIMS, &claim_key, claim)?;
            plan.put_bytes(
                keyspaces::SEQUENCE_INDEX,
                &keyspaces::sequence_key(claim_sequence),
                claim_key,
            )?;
        }

        cursor = cursor.checked_add(1).ok_or(Error::SequenceOverflow)?;
        let change = RuntimeChange::committed(
            cursor,
            commit,
            &commit_id,
            ordinal as u64,
            mutation.clone(),
            previous_digest.clone(),
        );
        plan.put_json(
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(cursor),
            &change,
        )?;
        for node in accumulator.append_change(&change)? {
            plan.put_bytes(
                keyspaces::META,
                &keyspaces::runtime_accumulator_node_key(node.level, node.index),
                node.digest.into_bytes(),
            )?;
        }
        if let Some(family) = projection_family(mutation) {
            let work = ProjectionWork::for_change(
                commit.scope.clone(),
                cursor,
                commit_id.clone(),
                ordinal as u64,
                family,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_PROJECTION_DELTAS,
                &keyspaces::runtime_projection_delta_key(&work),
                &work,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_OUTBOX,
                &keyspaces::runtime_outbox_key(cursor),
                &work,
            )?;
            outbox_count += 1;
        }
        encode_domain_effect(
            reader,
            &mut plan,
            &mut staged_relations,
            commit,
            mutation,
            &change,
        )?;
        previous_digest = Some(change.digest);
    }

    if let Some(schema) = effective_schema.as_ref() {
        super::super::encode_record_index_effects(
            reader,
            &mut plan,
            commit,
            schema,
            catalogue_revision,
            start,
        )?;
    }
    super::super::encode_vector_source_effects(reader, &mut plan, commit, start)?;
    if let Some((instance, receipts)) = function_receipts {
        super::super::encode_function_invocation_receipts(
            reader, &mut plan, instance, &commit_id, receipts,
        )?;
    }

    if claim_count > 0 {
        plan.put_sequence(&keyspaces::sequence_watermark_key(), claim_sequence)?;
    }
    plan.put_sequence(&keyspaces::runtime_cursor_key(), cursor)?;
    plan.put_bytes(
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
        previous_digest.as_deref().unwrap_or("").as_bytes().to_vec(),
    )?;
    plan.put_json(
        keyspaces::META,
        &keyspaces::runtime_accumulator_state_key(),
        &accumulator,
    )?;
    // Every semantic transaction writes the catalogue watermark it observed.
    // A concurrent catalogue transition therefore conflicts at physical
    // commit even for internal callers that did not supply a ReadStamp.
    plan.put_sequence(
        &keyspaces::catalog_revision_key(&commit.scope),
        catalogue_revision,
    )?;

    let audit_read = archived_audit
        .and_then(|audit| audit.read.as_ref())
        .or(read);
    let audit = AuditEnvelope::accepted_commit_at_read(
        commit,
        audit_read,
        &commit_id,
        cursor,
        previous_audit_digest,
    )?;
    if archived_audit.is_some_and(|expected| expected != &audit) {
        return Err(Error::Archive(format!(
            "runtime commit {commit_id} audit envelope differs from its archive"
        )));
    }
    plan.put_json(
        keyspaces::RUNTIME_AUDIT,
        &keyspaces::runtime_audit_key(&commit_id),
        &audit,
    )?;
    plan.put_bytes(
        keyspaces::META,
        &keyspaces::runtime_last_audit_digest_key(),
        audit.digest.as_bytes().to_vec(),
    )?;

    let outcome = RuntimeCommitOutcome {
        commit_id,
        first_cursor: start.checked_add(1).ok_or(Error::SequenceOverflow)?,
        last_cursor: cursor,
        count: commit.mutations.len(),
        first_claim_sequence: (claim_count > 0)
            .then_some(claim_start.checked_add(1).ok_or(Error::SequenceOverflow)?),
        last_claim_sequence: (claim_count > 0).then_some(claim_sequence),
        outbox_count,
    };
    plan.put_json(
        keyspaces::RUNTIME_COMMITS,
        &keyspaces::runtime_commit_key(&outcome.commit_id),
        &outcome,
    )?;
    plan.outcome = outcome;
    Ok(plan)
}

pub(super) fn validate_schema_and_objects(
    reader: &(impl AccessRead + ?Sized),
    commit: &RuntimeCommit,
) -> Result<Option<RuntimeSchemaRegistry>> {
    let previous_schema: Option<RuntimeSchemaRegistry> = get_json(
        reader,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(&commit.scope),
    )?;
    let proposed_schema = commit.mutations.iter().find_map(|mutation| match mutation {
        RuntimeMutation::Schema { registry } => Some(registry),
        _ => None,
    });
    let schema_free_claims = previous_schema.is_none()
        && proposed_schema.is_none()
        && commit
            .mutations
            .iter()
            .all(|mutation| matches!(mutation, RuntimeMutation::Claim { .. }));
    if schema_free_claims {
        return Ok(None);
    }
    let effective_schema = match (previous_schema.as_ref(), proposed_schema) {
        (None, Some(registry)) if registry.revision == 1 => registry,
        (None, Some(registry)) => {
            return Err(Error::RuntimeSchemaConflict {
                expected: 1,
                actual: registry.revision,
            });
        }
        (Some(previous), Some(registry))
            if registry.revision == previous.revision.saturating_add(1) =>
        {
            registry
        }
        (Some(previous), Some(registry)) => {
            return Err(Error::RuntimeSchemaConflict {
                expected: previous.revision.saturating_add(1),
                actual: registry.revision,
            });
        }
        (Some(previous), None) => previous,
        (None, None) => return Err(Error::RuntimeSchemaMissing(commit.scope.to_string())),
    };
    let (existing_records, existing_relations) =
        constraints::load(reader, commit, effective_schema, proposed_schema.is_some())?;
    effective_schema.validate_objects(&commit.mutations, &existing_records, &existing_relations)?;
    Ok(Some(effective_schema.clone()))
}

pub(super) fn validate_references(
    reader: &(impl AccessRead + ?Sized),
    commit: &RuntimeCommit,
) -> Result<()> {
    let new_records = commit
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Record { record } => Some(record.reference.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for mutation in &commit.mutations {
        let references: Vec<&RuntimeRef> = match mutation {
            RuntimeMutation::Relation { relation } => vec![&relation.from, &relation.to],
            RuntimeMutation::Event { event } => event.subject.iter().collect(),
            RuntimeMutation::Vector { vector } => vec![&vector.subject],
            RuntimeMutation::SeriesSample { sample } => vec![&sample.series],
            RuntimeMutation::Geo { geo } => vec![&geo.subject],
            RuntimeMutation::Object { object } => object.subject.iter().collect(),
            RuntimeMutation::Claim { .. }
            | RuntimeMutation::Schema { .. }
            | RuntimeMutation::Record { .. }
            | RuntimeMutation::Retire { .. } => Vec::new(),
        };
        for reference in references {
            if !new_records.contains(reference)
                && get(
                    reader,
                    keyspaces::RUNTIME_RECORDS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RECORDS,
                        &commit.scope,
                        reference,
                    ),
                )?
                .is_none()
            {
                return Err(Error::DanglingRuntimeReference(format!(
                    "{}/{} in scope {}",
                    reference.kind, reference.id, commit.scope
                )));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_retirement_targets(
    reader: &(impl AccessRead + ?Sized),
    commit: &RuntimeCommit,
    head: u64,
) -> Result<()> {
    let retirements = commit
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Retire { retirement } => Some(retirement),
            _ => None,
        })
        .collect::<Vec<_>>();
    if retirements.is_empty() {
        return Ok(());
    }
    let read = read_stamp_with(reader, &commit.scope)?;
    if read.commit_cursor != head {
        return Err(Error::ReadStampUnavailable(format!(
            "retirement validation for {} moved from cursor {head} to {}",
            commit.scope, read.commit_cursor
        )));
    }
    let mut sources = vec![RuntimeVersionedSource::Schema];
    sources.extend(
        retirements
            .iter()
            .map(|retirement| RuntimeVersionedSource::Identity {
                model: retirement.model,
                reference: retirement.reference.clone(),
            }),
    );
    let direct = read_versioned(
        reader,
        &read,
        &sources,
        RuntimeReadBudget::authenticated_history_integrity(head)?,
    )?;
    let schema = schema_at_read(&read, &direct.changes)?;
    let mut snapshots = BTreeMap::new();
    for retirement in retirements {
        let snapshot = match snapshots.entry(retirement.effective_at) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => entry.insert(
                RuntimeDataSnapshot::from_changes(
                    &direct.changes,
                    &schema,
                    commit.scope.clone(),
                    retirement.effective_at,
                    head,
                )
                .map_err(Error::from)?,
            ),
        };
        if !snapshot.contains(retirement.model, &retirement.reference) {
            return Err(Error::RuntimeTargetNotFound(format!(
                "{:?} {}/{} at {} in scope {}",
                retirement.model,
                retirement.reference.kind,
                retirement.reference.id,
                retirement.effective_at,
                commit.scope
            )));
        }
    }
    Ok(())
}
