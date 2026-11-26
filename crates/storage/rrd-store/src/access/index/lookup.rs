//! Lexical posting backfill, binding lookup and scalar index reads.

use super::*;

/// Indexes every current record of a BM25 binding's kind when the binding is
/// created or redefined.
pub(super) fn backfill_postings(
    transaction: &mut dyn StorageTransaction,
    binding: &IndexCommitBinding,
    config: &crate::lexical::Bm25Config,
) -> Result<()> {
    let prefix = keyspaces::runtime_scope_prefix(keyspaces::RUNTIME_RECORDS, &binding.scope);
    let records = scan_space_bounded(
        &*transaction,
        keyspaces::RUNTIME_RECORDS,
        &prefix,
        MAX_INDEX_BACKFILL_RECORDS + 1,
    )?;
    if records.len() > MAX_INDEX_BACKFILL_RECORDS {
        return Err(Error::IndexConstraint(format!(
            "index {} backfill exceeds {MAX_INDEX_BACKFILL_RECORDS} records",
            binding.definition.id
        )));
    }
    let mut changes = super::super::lexical_index::LexicalChanges::new(
        &binding.scope,
        &binding.definition.id,
        config,
        binding.definition.fields[0].as_str(),
    );
    for (_, bytes) in records {
        let record = serde_json::from_slice::<RuntimeRecord>(&bytes)?;
        if record.reference.kind == binding.definition.record_kind {
            changes.add(&record)?;
        }
    }
    let mut writes = Vec::new();
    changes.write(&*transaction, &mut |space, key, value| {
        writes.push((space, key, value));
        Ok(())
    })?;
    for (space, key, value) in writes {
        match value {
            Some(value) => transaction.put(checked_key(space, &key)?, value)?,
            None => transaction.delete(checked_key(space, &key)?)?,
        }
    }
    Ok(())
}

/// The lexical configuration, field and record kind of a BM25 commit binding.
pub(crate) fn lexical_binding(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    index: &ProjectionId,
) -> Result<Option<(crate::lexical::Bm25Config, String, RuntimeType)>> {
    let binding: Option<IndexCommitBinding> = get_json(
        reader,
        keyspaces::RUNTIME_INDEX_BINDINGS,
        &keyspaces::runtime_index_binding_key(scope, index),
    )?;
    let Some(binding) = binding else {
        return Ok(None);
    };
    binding.validate()?;
    if binding.scope != *scope || binding.definition.id != *index {
        return Err(Error::IndexConstraint(
            "index commit binding escaped its key".into(),
        ));
    }
    Ok(binding.definition.lexical.map(|config| {
        (
            config,
            binding.definition.fields[0].clone(),
            binding.definition.record_kind,
        )
    }))
}

/// The valid time a transition's prior version stopped being visible: its
/// retirement, or the valid time of the version that replaced it; `None` for
/// a record that had no prior version.
pub(super) fn removed_at(
    removals: &BTreeMap<&RuntimeRef, u64>,
    reference: &RuntimeRef,
    transition: &RecordTransition,
) -> Option<u64> {
    transition.before.as_ref()?;
    removals
        .get(reference)
        .copied()
        .or_else(|| transition.after.as_ref().map(|record| record.valid_from))
}

/// The definition of a commit-bound index, when one is bound.
pub(crate) fn commit_binding(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    index: &ProjectionId,
) -> Result<Option<IndexCommitBindingDefinition>> {
    let binding: Option<IndexCommitBinding> = get_json(
        reader,
        keyspaces::RUNTIME_INDEX_BINDINGS,
        &keyspaces::runtime_index_binding_key(scope, index),
    )?;
    let Some(binding) = binding else {
        return Ok(None);
    };
    binding.validate()?;
    if binding.scope != *scope || binding.definition.id != *index {
        return Err(Error::IndexConstraint(
            "index commit binding escaped its key".into(),
        ));
    }
    Ok(Some(binding.definition))
}

/// Current records whose leading indexed values equal `values` and whose
/// version is visible at `valid_at`, from a scalar binding's entries.
pub(crate) fn scalar_lookup(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    definition: &IndexCommitBindingDefinition,
    values: &[RuntimeValue],
    valid_at: u64,
    max_entries: usize,
) -> Result<(Vec<RuntimeRef>, u64)> {
    let IndexCommitBindingKind::Scalar { unique } = definition.kind else {
        return Err(Error::IndexConstraint(format!(
            "index {} is not a scalar index",
            definition.id
        )));
    };
    if values.is_empty() || values.len() > definition.fields.len() {
        return Err(Error::IndexConstraint(format!(
            "a lookup in index {} binds 1..={} leading values",
            definition.id,
            definition.fields.len()
        )));
    }
    let space = scalar_space(IndexCommitBindingKind::Scalar { unique });
    let prefix = keyspaces::runtime_index_value_prefix(space, scope, &definition.id, values)?;
    let entries = scan_space_bounded(reader, space, &prefix, max_entries.saturating_add(1))?;
    if entries.len() > max_entries {
        return Err(Error::IndexConstraint(format!(
            "lookup in index {} matches more than {max_entries} entries",
            definition.id
        )));
    }
    let keys = entries.len() as u64;
    let mut references = Vec::new();
    for (_, bytes) in entries {
        let entry: ScalarIndexEntry = serde_json::from_slice(&bytes)?;
        if entry.index_id != definition.id || entry.values.get(..values.len()) != Some(values) {
            return Err(Error::IndexConstraint(format!(
                "index {} entry escaped its key",
                definition.id
            )));
        }
        if entry.valid_from <= valid_at && entry.valid_to.is_none_or(|end| valid_at < end) {
            references.push(entry.record);
        }
    }
    references.sort();
    references.dedup();
    Ok((references, keys))
}
