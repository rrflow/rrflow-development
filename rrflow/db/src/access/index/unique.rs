//! Unique-value validation and shared entry helpers.

use super::*;

pub(super) fn validate_unique_transition(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    binding: &IndexCommitBinding,
    transitions: &[(&RuntimeRef, &RecordTransition)],
) -> Result<()> {
    let candidates = transitions
        .iter()
        .filter_map(|(reference, transition)| {
            transition.after.as_ref().map(|record| {
                (
                    (*reference).clone(),
                    source_row(record, &binding.definition.fields),
                )
            })
        })
        .filter(|(_, row)| !row.values.contains(&RuntimeValue::Null))
        .collect::<Vec<_>>();
    for (index, (left_ref, left)) in candidates.iter().enumerate() {
        if candidates[index + 1..].iter().any(|(right_ref, right)| {
            left_ref != right_ref
                && left.values == right.values
                && windows_overlap(
                    left.valid_from,
                    left.valid_to,
                    right.valid_from,
                    right.valid_to,
                )
        }) {
            return Err(unique_conflict(&binding.definition.id));
        }
    }
    let replaced = transitions
        .iter()
        .map(|(reference, _)| (*reference).clone())
        .collect::<BTreeSet<_>>();
    for (_, candidate) in &candidates {
        let prefix = keyspaces::runtime_index_value_prefix(
            keyspaces::RUNTIME_UNIQUE_ENTRIES,
            scope,
            &binding.definition.id,
            &candidate.values,
        )?;
        let existing = scan_space_bounded(
            reader,
            keyspaces::RUNTIME_UNIQUE_ENTRIES,
            &prefix,
            MAX_UNIQUE_VALUE_ENTRIES + 1,
        )?;
        if existing.len() > MAX_UNIQUE_VALUE_ENTRIES {
            return Err(Error::IndexConstraint(format!(
                "unique index {} value exceeds {MAX_UNIQUE_VALUE_ENTRIES} live windows",
                binding.definition.id
            )));
        }
        for (_, bytes) in existing {
            let existing: ScalarIndexEntry = serde_json::from_slice(&bytes)?;
            if !replaced.contains(&existing.record)
                && windows_overlap(
                    candidate.valid_from,
                    candidate.valid_to,
                    existing.valid_from,
                    existing.valid_to,
                )
            {
                return Err(unique_conflict(&binding.definition.id));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_unique_records(
    index: &ProjectionId,
    fields: &[String],
    records: &[&RuntimeRecord],
) -> Result<()> {
    let mut groups = BTreeMap::<Vec<u8>, Vec<&RuntimeRecord>>::new();
    for record in records {
        let row = source_row(record, fields);
        if row.values.contains(&RuntimeValue::Null) {
            continue;
        }
        groups
            .entry(serde_json::to_vec(&row.values)?)
            .or_default()
            .push(record);
    }
    for group in groups.values() {
        for (position, left) in group.iter().enumerate() {
            if group[position + 1..].iter().any(|right| {
                windows_overlap(
                    left.valid_from,
                    left.valid_to,
                    right.valid_from,
                    right.valid_to,
                )
            }) {
                return Err(unique_conflict(index));
            }
        }
    }
    Ok(())
}

pub(super) fn source_row(record: &RuntimeRecord, fields: &[String]) -> IndexSourceRow {
    let values = fields
        .iter()
        .map(|field| match field.as_str() {
            "id" => RuntimeValue::String(record.reference.id.to_string()),
            "kind" => RuntimeValue::String(record.reference.kind.to_string()),
            "valid_from" => RuntimeValue::Unsigned(record.valid_from),
            "valid_to" => record
                .valid_to
                .map_or(RuntimeValue::Null, RuntimeValue::Unsigned),
            _ => record
                .properties
                .get(field)
                .cloned()
                .unwrap_or(RuntimeValue::Null),
        })
        .collect();
    IndexSourceRow {
        values,
        valid_from: record.valid_from,
        valid_to: record.valid_to,
    }
}

pub(super) fn scalar_space(kind: IndexCommitBindingKind) -> Space {
    match kind {
        IndexCommitBindingKind::Scalar { unique: false } => keyspaces::RUNTIME_SCALAR_ENTRIES,
        IndexCommitBindingKind::Scalar { unique: true } => keyspaces::RUNTIME_UNIQUE_ENTRIES,
        IndexCommitBindingKind::Bm25 => {
            unreachable!("BM25 bindings do not own scalar entry keys")
        }
    }
}

pub(super) fn windows_overlap(
    left_from: u64,
    left_to: Option<u64>,
    right_from: u64,
    right_to: Option<u64>,
) -> bool {
    left_from < right_to.unwrap_or(u64::MAX) && right_from < left_to.unwrap_or(u64::MAX)
}

pub(super) fn unique_conflict(index: &ProjectionId) -> Error {
    Error::IndexConstraint(format!(
        "unique index {index} rejects overlapping duplicate records"
    ))
}

pub(super) fn validate_sha256(label: &str, value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(Error::IndexConstraint(format!(
            "{label} must be a lowercase SHA-256"
        )));
    }
    Ok(())
}

pub(super) fn put_json<T: Serialize>(
    transaction: &mut dyn StorageTransaction,
    space: Space,
    key: &[u8],
    value: &T,
) -> Result<()> {
    transaction.put(checked_key(space, key)?, serde_json::to_vec(value)?)
}
