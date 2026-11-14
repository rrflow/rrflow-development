//! Projection, index-binding, term, index-entry and vector-delta keys.

use super::*;

pub(crate) fn runtime_projection_delta_start(cursor: u64) -> Vec<u8> {
    encode(RUNTIME_PROJECTION_DELTAS, &[KeyPart::U64(cursor)])
}

pub(crate) fn runtime_index_binding_key(scope: &ScopeId, index: &ProjectionId) -> Vec<u8> {
    encode(
        RUNTIME_INDEX_BINDINGS,
        &[KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())],
    )
}

pub(crate) fn runtime_index_binding_scope_prefix(scope: &ScopeId) -> Vec<u8> {
    encode(RUNTIME_INDEX_BINDINGS, &[KeyPart::Text(scope.as_str())])
}

pub(crate) fn runtime_index_binding_set_key(scope: &ScopeId) -> Vec<u8> {
    encode(RUNTIME_INDEX_BINDING_SETS, &[KeyPart::Text(scope.as_str())])
}

pub(crate) fn runtime_index_binding_version_key(
    scope: &ScopeId,
    index: &ProjectionId,
    catalogue_revision: u64,
    schema_revision: u64,
) -> Vec<u8> {
    encode(
        RUNTIME_INDEX_BINDING_VERSIONS,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(index.as_str()),
            KeyPart::U64(catalogue_revision),
            KeyPart::U64(schema_revision),
        ],
    )
}

pub(crate) fn runtime_index_entry_prefix(
    space: Space,
    scope: &ScopeId,
    index: &ProjectionId,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeScalarEntries | Space::RuntimeUniqueEntries
    ));
    encode(
        space,
        &[KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())],
    )
}

/// One document's postings for one term of a BM25 index.
pub(crate) fn runtime_term_posting_key(
    scope: &ScopeId,
    index: &ProjectionId,
    term: &str,
    reference: &RuntimeRef,
) -> Vec<u8> {
    encode(
        RUNTIME_TERM_POSTINGS,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(index.as_str()),
            KeyPart::Text(term),
            KeyPart::Text(reference.kind.as_str()),
            KeyPart::Text(reference.id.as_str()),
        ],
    )
}

/// Every posting of one term, or of the whole index when `term` is `None`.
pub(crate) fn runtime_term_posting_prefix(
    scope: &ScopeId,
    index: &ProjectionId,
    term: Option<&str>,
) -> Vec<u8> {
    let mut parts = vec![KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())];
    if let Some(term) = term {
        parts.push(KeyPart::Text(term));
    }
    encode(RUNTIME_TERM_POSTINGS, &parts)
}

/// One term's document frequency in a BM25 index.
pub(crate) fn runtime_term_dictionary_key(
    scope: &ScopeId,
    index: &ProjectionId,
    term: &str,
) -> Vec<u8> {
    encode(
        RUNTIME_TERM_DICTIONARY,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(index.as_str()),
            KeyPart::Text(term),
        ],
    )
}

pub(crate) fn runtime_term_dictionary_prefix(scope: &ScopeId, index: &ProjectionId) -> Vec<u8> {
    encode(
        RUNTIME_TERM_DICTIONARY,
        &[KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())],
    )
}

/// A BM25 index's document and token totals.
pub(crate) fn runtime_term_statistics_key(scope: &ScopeId, index: &ProjectionId) -> Vec<u8> {
    encode(
        RUNTIME_TERM_STATISTICS,
        &[KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())],
    )
}

pub(crate) fn runtime_index_value_prefix<'a>(
    space: Space,
    scope: &'a ScopeId,
    index: &'a ProjectionId,
    values: &'a [RuntimeValue],
) -> Result<Vec<u8>> {
    let mut parts = vec![KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())];
    append_index_values(&mut parts, values)?;
    Ok(encode(space, &parts))
}

pub(crate) fn runtime_index_entry_key<'a>(
    space: Space,
    scope: &'a ScopeId,
    index: &'a ProjectionId,
    values: &'a [RuntimeValue],
    reference: &'a RuntimeRef,
    valid_from: Millis,
) -> Result<Vec<u8>> {
    let mut parts = vec![KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())];
    append_index_values(&mut parts, values)?;
    parts.extend([
        KeyPart::Text(reference.kind.as_str()),
        KeyPart::Text(reference.id.as_str()),
        KeyPart::DescU64(valid_from),
    ]);
    Ok(encode(space, &parts))
}

pub(super) fn append_index_values<'a>(
    parts: &mut Vec<KeyPart<'a>>,
    values: &'a [RuntimeValue],
) -> Result<()> {
    for value in values {
        match value {
            RuntimeValue::Null => parts.push(KeyPart::U8(0)),
            RuntimeValue::Bool(value) => {
                parts.extend([KeyPart::U8(1), KeyPart::Bool(*value)]);
            }
            RuntimeValue::Integer(value) => {
                parts.extend([KeyPart::U8(2), KeyPart::I64(*value)]);
            }
            RuntimeValue::Unsigned(value) => {
                parts.extend([KeyPart::U8(3), KeyPart::U64(*value)]);
            }
            RuntimeValue::Decimal(value) => {
                parts.extend([KeyPart::U8(4), KeyPart::Text(value)]);
            }
            RuntimeValue::String(value) => {
                parts.extend([KeyPart::U8(5), KeyPart::Text(value)]);
            }
            RuntimeValue::Digest(value) => {
                parts.extend([KeyPart::U8(6), KeyPart::Text(value)]);
            }
            RuntimeValue::List(_) | RuntimeValue::Map(_) => {
                return Err(Error::Codec(
                    "scalar index keys cannot contain list or map values".into(),
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn runtime_index_source_delta_key(
    scope: &ScopeId,
    index: &ProjectionId,
    cursor: u64,
    ordinal: u64,
) -> Vec<u8> {
    encode(
        RUNTIME_INDEX_SOURCE_DELTAS,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(index.as_str()),
            KeyPart::U64(cursor),
            KeyPart::U64(ordinal),
        ],
    )
}

pub(crate) fn runtime_index_source_delta_start(
    scope: &ScopeId,
    index: &ProjectionId,
    cursor: u64,
) -> Vec<u8> {
    runtime_index_source_delta_key(scope, index, cursor, 0)
}

pub(crate) fn runtime_index_source_delta_prefix(scope: &ScopeId, index: &ProjectionId) -> Vec<u8> {
    encode(
        RUNTIME_INDEX_SOURCE_DELTAS,
        &[KeyPart::Text(scope.as_str()), KeyPart::Text(index.as_str())],
    )
}

pub(crate) fn runtime_vector_version_key(
    scope: &ScopeId,
    reference: &RuntimeRef,
    effective_at: Millis,
    cursor: u64,
) -> Vec<u8> {
    encode(
        RUNTIME_VECTOR_VERSIONS,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(reference.kind.as_str()),
            KeyPart::Text(reference.id.as_str()),
            KeyPart::DescU64(effective_at),
            KeyPart::DescU64(cursor),
        ],
    )
}

pub(crate) fn runtime_vector_source_delta_key(
    scope: &ScopeId,
    collection_id: &str,
    vector_name: &str,
    field: &str,
    cursor: u64,
    ordinal: u64,
) -> Result<Vec<u8>> {
    validate_vector_source_key_parts(collection_id, vector_name, field)?;
    Ok(encode(
        RUNTIME_VECTOR_SOURCE_DELTAS,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(collection_id),
            KeyPart::Text(vector_name),
            KeyPart::Text(field),
            KeyPart::U64(cursor),
            KeyPart::U64(ordinal),
        ],
    ))
}

pub(crate) fn runtime_vector_source_delta_start(
    scope: &ScopeId,
    collection_id: &str,
    vector_name: &str,
    field: &str,
    cursor: u64,
) -> Result<Vec<u8>> {
    runtime_vector_source_delta_key(scope, collection_id, vector_name, field, cursor, 0)
}

pub(super) fn validate_vector_source_key_parts(
    collection_id: &str,
    vector_name: &str,
    field: &str,
) -> Result<()> {
    if collection_id.trim().is_empty() || vector_name.trim().is_empty() || field.trim().is_empty() {
        return Err(Error::IndexConstraint(
            "vector source key identity must not contain an empty part".into(),
        ));
    }
    Ok(())
}

pub(super) fn projection_family_tag(family: ProjectionFamily) -> u8 {
    match family {
        ProjectionFamily::Scalar => 0,
        ProjectionFamily::Graph => 1,
        ProjectionFamily::Text => 2,
        ProjectionFamily::Vector => 3,
        ProjectionFamily::TimeSeries => 4,
        ProjectionFamily::Geo => 5,
        ProjectionFamily::Object => 6,
    }
}
