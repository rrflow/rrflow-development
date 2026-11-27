//! Binding-schema validation, clearing and backfilling index entries.

use super::*;

pub(super) fn definitions_sha256(definitions: &[IndexCommitBindingDefinition]) -> Result<String> {
    let mut canonical = definitions.to_vec();
    canonical.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(digest::sha256_hex(&serde_json::to_vec(&canonical)?))
}

pub(super) fn validate_binding_schema(
    definition: &IndexCommitBindingDefinition,
    schema: &RuntimeSchemaRegistry,
) -> Result<()> {
    let record = schema.records.get(&definition.record_kind).ok_or_else(|| {
        Error::IndexConstraint(format!(
            "index {} references unknown record type {}",
            definition.id, definition.record_kind
        ))
    })?;
    for field in &definition.fields {
        let value_type = match field.as_str() {
            "id" | "kind" => RuntimeValueType::String,
            "valid_from" | "valid_to" => RuntimeValueType::Unsigned,
            _ => record
                .properties
                .get(field)
                .map(|property| property.value_type)
                .ok_or_else(|| {
                    Error::IndexConstraint(format!(
                        "index {} references untyped field {field:?}",
                        definition.id
                    ))
                })?,
        };
        if matches!(value_type, RuntimeValueType::List | RuntimeValueType::Map) {
            return Err(Error::IndexConstraint(format!(
                "index {} cannot encode non-scalar field {field:?}",
                definition.id
            )));
        }
        if matches!(definition.kind, IndexCommitBindingKind::Bm25)
            && value_type != RuntimeValueType::String
        {
            return Err(Error::IndexConstraint(format!(
                "BM25 index {} requires a string field",
                definition.id
            )));
        }
    }
    Ok(())
}

pub(super) fn clear_index_entries(
    transaction: &mut dyn StorageTransaction,
    scope: &ScopeId,
    binding: &IndexCommitBinding,
) -> Result<()> {
    if matches!(binding.definition.kind, IndexCommitBindingKind::Bm25) {
        return super::super::lexical_index::clear(transaction, scope, &binding.definition.id);
    }
    if !matches!(
        binding.definition.kind,
        IndexCommitBindingKind::Scalar { .. }
    ) {
        return Ok(());
    }
    let space = scalar_space(binding.definition.kind);
    let prefix = keyspaces::runtime_index_entry_prefix(space, scope, &binding.definition.id);
    let entries = scan_space_bounded(
        &*transaction,
        space,
        &prefix,
        MAX_INDEX_ENTRIES_PER_DEFINITION + 1,
    )?;
    if entries.len() > MAX_INDEX_ENTRIES_PER_DEFINITION {
        return Err(Error::IndexConstraint(format!(
            "index {} exceeds {MAX_INDEX_ENTRIES_PER_DEFINITION} removable entries",
            binding.definition.id
        )));
    }
    for (key, _) in entries {
        transaction.delete(checked_key(space, &key)?)?;
    }
    Ok(())
}

pub(super) fn backfill_index_entries(
    transaction: &mut dyn StorageTransaction,
    binding: &IndexCommitBinding,
) -> Result<()> {
    if let Some(config) = &binding.definition.lexical {
        return backfill_postings(transaction, binding, config);
    }
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
    let records = records
        .into_iter()
        .map(|(_, bytes)| serde_json::from_slice::<RuntimeRecord>(&bytes).map_err(Error::from))
        .collect::<Result<Vec<_>>>()?;
    let source_cursor = read_sequence(transaction, &keyspaces::runtime_cursor_key())?;
    let relevant = records
        .iter()
        .filter(|record| record.reference.kind == binding.definition.record_kind)
        .collect::<Vec<_>>();
    if matches!(
        binding.definition.kind,
        IndexCommitBindingKind::Scalar { unique: true }
    ) {
        validate_unique_records(
            &binding.definition.id,
            &binding.definition.fields,
            &relevant,
        )?;
    }
    let space = scalar_space(binding.definition.kind);
    for record in relevant {
        let row = source_row(record, &binding.definition.fields);
        if matches!(
            binding.definition.kind,
            IndexCommitBindingKind::Scalar { unique: true }
        ) && row.values.contains(&RuntimeValue::Null)
        {
            continue;
        }
        let entry = ScalarIndexEntry {
            index_id: binding.definition.id.clone(),
            record: record.reference.clone(),
            values: row.values.clone(),
            valid_from: row.valid_from,
            valid_to: row.valid_to,
            source_cursor,
        };
        put_json(
            transaction,
            space,
            &keyspaces::runtime_index_entry_key(
                space,
                &binding.scope,
                &binding.definition.id,
                &row.values,
                &record.reference,
                row.valid_from,
            )?,
            &entry,
        )?;
    }
    Ok(())
}

pub(super) fn record_transitions(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    commit: &RuntimeCommit,
    starting_cursor: u64,
) -> Result<BTreeMap<RuntimeRef, RecordTransition>> {
    let mut transitions = BTreeMap::new();
    for (ordinal, mutation) in commit.mutations.iter().enumerate() {
        let (reference, after) = match mutation {
            RuntimeMutation::Record { record } => (&record.reference, Some(record.clone())),
            RuntimeMutation::Retire { retirement } if retirement.model.is_record_like() => {
                (&retirement.reference, None)
            }
            _ => continue,
        };
        let ordinal = u64::try_from(ordinal)
            .map_err(|_| Error::Substrate("commit ordinal exceeds u64".into()))?;
        let source_cursor = starting_cursor
            .checked_add(ordinal)
            .and_then(|cursor| cursor.checked_add(1))
            .ok_or(Error::SequenceOverflow)?;
        let transition = match transitions.entry(reference.clone()) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let before = get_json(
                    reader,
                    keyspaces::RUNTIME_RECORDS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RECORDS,
                        &commit.scope,
                        reference,
                    ),
                )?;
                entry.insert(RecordTransition {
                    before,
                    after: None,
                    source_cursor,
                    commit_ordinal: ordinal,
                })
            }
        };
        transition.after = after;
        transition.source_cursor = source_cursor;
        transition.commit_ordinal = ordinal;
    }
    Ok(transitions)
}
