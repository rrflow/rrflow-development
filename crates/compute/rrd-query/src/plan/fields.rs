//! Parameters, field catalogues and type checks.

use super::*;

pub(super) fn parameter_u64(parameters: &Parameters, name: &str, label: &str) -> Result<u64> {
    match parameters.get(name) {
        Some(RuntimeValue::Unsigned(value)) => Ok(*value),
        Some(_) => Err(Error::Binding(format!(
            "{label} parameter ${name} must be unsigned"
        ))),
        None => Err(Error::Binding(format!("missing query parameter ${name}"))),
    }
}

pub(super) fn ensure_field<'a>(
    field: &str,
    fields: &'a FieldCatalog,
) -> Result<&'a [RuntimeValueType]> {
    fields.get(field).map(Vec::as_slice).ok_or_else(|| {
        Error::Binding(format!(
            "field {field:?} is not present in the selected source"
        ))
    })
}

pub(super) fn joined_field_catalog(left: &FieldCatalog, right: &FieldCatalog) -> FieldCatalog {
    left.iter()
        .map(|(field, types)| (format!("left.{field}"), types.clone()))
        .chain(
            right
                .iter()
                .map(|(field, types)| (format!("right.{field}"), types.clone())),
        )
        .collect()
}

pub(super) fn is_schemaless_record_projection(
    field: &str,
    query: &Query,
    schema: &RuntimeSchemaRegistry,
) -> bool {
    let (source, property) = match &query.join {
        Some(join) => match field.split_once('.') {
            Some(("left", property)) => (&query.source, property),
            Some(("right", property)) => (&join.source, property),
            _ => return false,
        },
        None => (&query.source, field),
    };
    if property.is_empty() || property.contains('.') {
        return false;
    }
    let Source::Record { kind } = source else {
        return false;
    };
    schema.tables.get(kind).is_some_and(|table| {
        table.model.is_record_like() && table.mode == rrd_core::RuntimeSchemaMode::Schemaless
    })
}

pub(super) fn ensure_value_type(
    field: &str,
    value: &RuntimeValue,
    accepted: &[RuntimeValueType],
) -> Result<()> {
    if accepted.iter().any(|expected| expected.matches(value)) {
        Ok(())
    } else {
        Err(Error::Binding(format!(
            "filter value for {field:?} does not match accepted types {accepted:?}"
        )))
    }
}

pub(super) fn ensure_comparison(
    field: &str,
    comparison: ComparisonOperator,
    value: &RuntimeValue,
) -> Result<()> {
    if comparison.is_match() && !matches!(value, RuntimeValue::String(_)) {
        return Err(Error::Binding(format!(
            "MATCH requires a string query for {field:?}"
        )));
    }
    if !comparison.is_ordering()
        || matches!(
            value,
            RuntimeValue::Integer(_) | RuntimeValue::Unsigned(_) | RuntimeValue::String(_)
        )
    {
        Ok(())
    } else {
        Err(Error::Binding(format!(
            "ordering comparison {} is not supported for the value type of {field:?}",
            comparison_label(comparison)
        )))
    }
}

pub(super) fn comparison_label(comparison: ComparisonOperator) -> &'static str {
    match comparison {
        ComparisonOperator::Equal => "=",
        ComparisonOperator::NotEqual => "!=",
        ComparisonOperator::LessThan => "<",
        ComparisonOperator::LessThanOrEqual => "<=",
        ComparisonOperator::GreaterThan => ">",
        ComparisonOperator::GreaterThanOrEqual => ">=",
        ComparisonOperator::Match => "MATCH",
    }
}

pub(crate) fn fields_for_source(
    source: &Source,
    schema: &RuntimeSchemaRegistry,
) -> Result<FieldCatalog> {
    let (builtins, properties): (BuiltinFields, Vec<(String, RuntimeValueType)>) = match source {
        Source::Record { kind } => {
            let properties = if let Some(definition) = schema.records.get(kind) {
                definition
                    .properties
                    .iter()
                    .map(|(name, rule)| (name.clone(), rule.value_type))
                    .collect()
            } else if schema.tables.get(kind).is_some_and(|table| {
                table.model.is_record_like()
                    && table.mode == rrd_core::RuntimeSchemaMode::Schemaless
            }) {
                Vec::new()
            } else {
                return Err(Error::Binding(format!(
                    "record type {kind} is not registered at this cursor"
                )));
            };
            (
                &[
                    ("id", &[RuntimeValueType::String]),
                    ("kind", &[RuntimeValueType::String]),
                    ("valid_from", &[RuntimeValueType::Unsigned]),
                    (
                        "valid_to",
                        &[RuntimeValueType::Null, RuntimeValueType::Unsigned],
                    ),
                ],
                properties,
            )
        }
        Source::Relation { kind } => {
            let definition = schema.relations.get(kind).ok_or_else(|| {
                Error::Binding(format!(
                    "relation type {kind} is not registered at this cursor"
                ))
            })?;
            (
                &[
                    ("id", &[RuntimeValueType::String]),
                    ("kind", &[RuntimeValueType::String]),
                    ("from_kind", &[RuntimeValueType::String]),
                    ("from_id", &[RuntimeValueType::String]),
                    ("to_kind", &[RuntimeValueType::String]),
                    ("to_id", &[RuntimeValueType::String]),
                    ("valid_from", &[RuntimeValueType::Unsigned]),
                    (
                        "valid_to",
                        &[RuntimeValueType::Null, RuntimeValueType::Unsigned],
                    ),
                ],
                definition
                    .properties
                    .iter()
                    .map(|(name, rule)| (name.clone(), rule.value_type))
                    .collect(),
            )
        }
        Source::Event { kind } => {
            let definition = schema.events.get(kind).ok_or_else(|| {
                Error::Binding(format!(
                    "event type {kind} is not registered at this cursor"
                ))
            })?;
            (
                &[
                    ("cursor", &[RuntimeValueType::Unsigned]),
                    ("kind", &[RuntimeValueType::String]),
                    (
                        "subject_kind",
                        &[RuntimeValueType::Null, RuntimeValueType::String],
                    ),
                    (
                        "subject_id",
                        &[RuntimeValueType::Null, RuntimeValueType::String],
                    ),
                    ("at", &[RuntimeValueType::Unsigned]),
                    ("actor", &[RuntimeValueType::String]),
                ],
                definition
                    .properties
                    .iter()
                    .map(|(name, rule)| (name.clone(), rule.value_type))
                    .collect(),
            )
        }
        Source::Series { .. } => (
            &[
                ("id", &[RuntimeValueType::String]),
                ("kind", &[RuntimeValueType::String]),
                ("series_kind", &[RuntimeValueType::String]),
                ("series_id", &[RuntimeValueType::String]),
                ("observed_at", &[RuntimeValueType::Unsigned]),
                (
                    "value",
                    &[
                        RuntimeValueType::Integer,
                        RuntimeValueType::Unsigned,
                        RuntimeValueType::Decimal,
                        RuntimeValueType::Bool,
                        RuntimeValueType::String,
                    ],
                ),
            ],
            Vec::new(),
        ),
        Source::Geo { .. } => (
            &[
                ("id", &[RuntimeValueType::String]),
                ("kind", &[RuntimeValueType::String]),
                ("subject_kind", &[RuntimeValueType::String]),
                ("subject_id", &[RuntimeValueType::String]),
                ("field", &[RuntimeValueType::String]),
                ("valid_from", &[RuntimeValueType::Unsigned]),
                (
                    "valid_to",
                    &[RuntimeValueType::Null, RuntimeValueType::Unsigned],
                ),
                ("geometry_kind", &[RuntimeValueType::String]),
                (
                    "longitude",
                    &[RuntimeValueType::Null, RuntimeValueType::Decimal],
                ),
                (
                    "latitude",
                    &[RuntimeValueType::Null, RuntimeValueType::Decimal],
                ),
                (
                    "southwest_longitude",
                    &[RuntimeValueType::Null, RuntimeValueType::Decimal],
                ),
                (
                    "southwest_latitude",
                    &[RuntimeValueType::Null, RuntimeValueType::Decimal],
                ),
                (
                    "northeast_longitude",
                    &[RuntimeValueType::Null, RuntimeValueType::Decimal],
                ),
                (
                    "northeast_latitude",
                    &[RuntimeValueType::Null, RuntimeValueType::Decimal],
                ),
            ],
            Vec::new(),
        ),
        Source::Traversal {
            relation, start, ..
        } => {
            let definition = schema.relations.get(relation).ok_or_else(|| {
                Error::Binding(format!(
                    "relation type {relation} is not registered at this cursor"
                ))
            })?;
            // A start is any record-like table in the catalogue; a schemaless
            // table is as much a graph node as one with a strict schema.
            let start_is_record = schema.records.contains_key(&start.kind)
                || schema
                    .tables
                    .get(&start.kind)
                    .is_some_and(|table| table.model.is_record_like());
            if !start_is_record {
                return Err(Error::Binding(format!(
                    "traversal start type {} is not registered at this cursor",
                    start.kind
                )));
            }
            if !definition.from.contains(&start.kind) && !definition.to.contains(&start.kind) {
                return Err(Error::Binding(format!(
                    "traversal start type {} is not an endpoint of relation {relation}",
                    start.kind
                )));
            }
            (
                &[
                    ("depth", &[RuntimeValueType::Unsigned]),
                    ("node_kind", &[RuntimeValueType::String]),
                    ("node_id", &[RuntimeValueType::String]),
                    ("relation_kind", &[RuntimeValueType::String]),
                    ("relation_id", &[RuntimeValueType::String]),
                    ("from_kind", &[RuntimeValueType::String]),
                    ("from_id", &[RuntimeValueType::String]),
                    ("to_kind", &[RuntimeValueType::String]),
                    ("to_id", &[RuntimeValueType::String]),
                    ("path", &[RuntimeValueType::List]),
                ],
                Vec::new(),
            )
        }
        Source::Claim { .. } => (
            &[
                ("subject", &[RuntimeValueType::String]),
                ("predicate", &[RuntimeValueType::String]),
                ("object", &[RuntimeValueType::String]),
                ("valid_from", &[RuntimeValueType::Unsigned]),
                (
                    "valid_to",
                    &[RuntimeValueType::Null, RuntimeValueType::Unsigned],
                ),
                ("tx_time", &[RuntimeValueType::Unsigned]),
                ("actor", &[RuntimeValueType::String]),
            ],
            Vec::new(),
        ),
    };
    Ok(builtins
        .iter()
        .map(|(field, accepted)| ((*field).to_owned(), accepted.to_vec()))
        .chain(
            properties
                .into_iter()
                .map(|(field, accepted)| (field, vec![accepted])),
        )
        .collect())
}

pub(crate) fn field_types_for_bound_source(
    source: &Source,
    catalog: &Catalog,
    field: &str,
) -> Result<Vec<RuntimeValueType>> {
    let schema = catalog
        .schema_at(catalog.read.commit_cursor)
        .ok_or_else(|| Error::Binding("no schema is visible at the catalogue head".into()))?;
    fields_for_source(source, schema)?
        .remove(field)
        .ok_or_else(|| Error::Binding(format!("field {field:?} is not present in the source")))
}
