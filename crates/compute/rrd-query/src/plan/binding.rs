//! Binding a parsed query to the schema and its index candidates.

use super::*;

pub fn bind(query: &Query, parameters: &Parameters, catalog: &Catalog) -> Result<BoundQuery> {
    let (bound, required) = prepare_binding(query, parameters, catalog)?;
    catalog.require_version_sources(&required)?;
    Ok(bound)
}

/// Resolve the one canonical binding and its catalogue requirements before
/// admitting source acquisition. The caller must satisfy `required` before
/// exposing or executing this binding.
pub(crate) fn prepare_binding(
    query: &Query,
    parameters: &Parameters,
    catalog: &Catalog,
) -> Result<(BoundQuery, Vec<RuntimeVersionedSource>)> {
    query
        .validate()
        .map_err(|error| Error::Binding(error.to_string()))?;
    let valid_at = resolve_u64_time(&query.temporal.valid_at, parameters, "valid time")?;
    let known_at_cursor = match &query.temporal.known_at {
        CursorExpr::Head => catalog.read.commit_cursor,
        CursorExpr::Literal(value) => *value,
        CursorExpr::Parameter(name) => parameter_u64(parameters, name, "known cursor")?,
    };
    if known_at_cursor > catalog.read.commit_cursor {
        return Err(Error::Binding(format!(
            "known cursor {known_at_cursor} exceeds captured head {}",
            catalog.read.commit_cursor
        )));
    }
    let schema = catalog.schema_at(known_at_cursor).ok_or_else(|| {
        Error::Binding(format!(
            "no schema was visible at known cursor {known_at_cursor}"
        ))
    })?;
    let source_fields = fields_for_source(&query.source, schema)?;
    let mut fields = if let Some(join) = &query.join {
        let right_fields = fields_for_source(&join.source, schema)?;
        let left_types = ensure_field(&join.left_field, &source_fields)?;
        let right_types = ensure_field(&join.right_field, &right_fields)?;
        if !left_types.iter().any(|left| {
            *left != RuntimeValueType::Null && right_types.iter().any(|right| left == right)
        }) {
            return Err(Error::Binding(format!(
                "join fields {:?} and {:?} have incompatible types",
                join.left_field, join.right_field
            )));
        }
        joined_field_catalog(&source_fields, &right_fields)
    } else {
        source_fields
    };
    for filter in &query.filters {
        let accepted = ensure_field(&filter.field, &fields)?;
        if let ValueExpr::Literal(value) = &filter.value {
            ensure_value_type(&filter.field, value, accepted)?;
        }
    }
    let filters = query
        .filters
        .iter()
        .map(|filter| {
            let value = match &filter.value {
                ValueExpr::Literal(value) => value.clone(),
                ValueExpr::Parameter(name) => parameters
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Error::Binding(format!("missing query parameter ${name}")))?,
            };
            ensure_value_type(
                &filter.field,
                &value,
                fields
                    .get(&filter.field)
                    .expect("filter field was checked before parameter resolution"),
            )?;
            ensure_comparison(&filter.field, filter.comparison, &value)?;
            Ok(BoundFilter {
                field: filter.field.clone(),
                comparison: filter.comparison,
                value,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    // Dynamic projections have no durable declared type. Admit them only
    // after filter binding so projection cannot authorize an untyped predicate.
    // Arrow materialization derives a lossless representation from this
    // request's bounded rows, without changing the authoritative schema.
    if let Projection::Fields(projected) = &query.projection {
        for field in projected {
            if !fields.contains_key(field) && is_schemaless_record_projection(field, query, schema)
            {
                fields.insert(field.clone(), Vec::new());
            }
            let _ = ensure_field(field, &fields)?;
        }
    }
    if query.join.is_some()
        && query
            .filters
            .iter()
            .any(|filter| filter.comparison.is_match())
    {
        return Err(Error::Binding(
            "MATCH scoring is not defined over joined row identities".into(),
        ));
    }
    let filter_fields = query
        .filters
        .iter()
        .map(|filter| filter.field.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let index_candidates = if query.join.is_none() {
        catalog
            .indexes
            .entries
            .values()
            .filter(|entry| entry.definition.source == query.source)
            .filter(|entry| entry.maintenance.is_some())
            .filter(|entry| {
                entry.result_identity_version
                    == Some(rrd_core::result_identity::RESULT_IDENTITY_VERSION)
            })
            .filter_map(|entry| {
                let matched_prefix = entry
                    .definition
                    .fields
                    .iter()
                    .take_while(|field| filter_fields.contains(field.as_str()))
                    .count();
                let predicates = entry
                    .definition
                    .filters
                    .iter()
                    .map(|filter| match &filter.value {
                        ValueExpr::Literal(value) => Some(BoundFilter {
                            field: filter.field.clone(),
                            comparison: filter.comparison,
                            value: value.clone(),
                        }),
                        ValueExpr::Parameter(_) => None,
                    })
                    .collect::<Option<Vec<_>>>()?;
                (matched_prefix > 0).then(|| BoundIndexCandidate {
                    id: entry.definition.id.clone(),
                    generation: entry.stamp.generation,
                    source_cursor: entry.stamp.source_cursor,
                    state: entry.stamp.state,
                    config_digest: entry.stamp.config_digest.clone(),
                    artifact_digest: entry.stamp.artifact_digest.clone(),
                    built_valid_at: entry.built_valid_at,
                    artifact_rows: entry.artifact_rows,
                    fields: entry.definition.fields.clone(),
                    matched_prefix,
                    kind: entry.definition.kind.clone(),
                    predicates,
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    let version_sources = bound_version_sources(query, &filters, schema)?;
    let commit_indexes = match (&query.source, &query.join) {
        (Source::Record { kind }, None) => {
            let model = schema
                .logical_model(kind)
                .map_err(|error| Error::Binding(error.to_string()))?;
            catalog
                .indexes
                .entries
                .values()
                .filter(|entry| {
                    entry.definition.source == query.source
                        && entry.definition.filters.is_empty()
                        && matches!(
                            entry.definition.kind,
                            IndexKind::Scalar | IndexKind::Bm25 { .. }
                        )
                })
                .map(|entry| BoundCommitIndex {
                    id: entry.definition.id.clone(),
                    kind: entry.definition.kind.clone(),
                    fields: entry.definition.fields.clone(),
                    model,
                })
                .collect()
        }
        _ => Vec::new(),
    };
    // A query a commit-bound index can serve at the head is addressed like an
    // identity read: its catalogue needs only the schema, because the index
    // names the identities to read, and the family is read in full only if
    // the index declines at execution.
    let commit_served = query.join.is_none()
        && known_at_cursor == catalog.read.commit_cursor
        && select_commit_index(
            &commit_indexes,
            &filters,
            query.limit,
            filters
                .iter()
                .find(|filter| filter.comparison.is_match())
                .map(|filter| filter.field.as_str()),
        )
        .is_some();
    let addressed = query.join.is_none()
        && (commit_served
            || matches!(
                version_sources.as_slice(),
                [RuntimeVersionedSource::Identity { .. }
                    | RuntimeVersionedSource::IdentityRange { .. }
                    | RuntimeVersionedSource::Adjacency { .. }]
            ));
    let source_cursor = catalog
        .source_watermarks
        .for_source_at(&query.source, known_at_cursor);
    let source_cursor = query.join.as_ref().map_or(source_cursor, |join| {
        source_cursor.max(
            catalog
                .source_watermarks
                .for_source_at(&join.source, known_at_cursor),
        )
    });
    let mut queried_sources = vec![query.source.clone()];
    if let Some(join) = &query.join {
        queried_sources.push(join.source.clone());
    }
    let required_catalogue_sources = if addressed {
        vec![RuntimeVersionedSource::Schema]
    } else {
        version_sources_for_sources(&queried_sources)
    };
    // Addressed direct paths do not consume a family index generation. Like
    // traversal, their freshness coordinate is the requested known prefix;
    // exact family index candidates still require actual source watermarks.
    let source_cursor = if addressed {
        known_at_cursor
    } else {
        source_cursor
    };
    Ok((
        BoundQuery {
            contract_version: QUERY_CONTRACT_VERSION,
            read: catalog.read.clone(),
            source: query.source.clone(),
            join: query.join.clone(),
            valid_at,
            known_at_cursor,
            source_cursor,
            version_sources,
            field_types: fields,
            filters,
            projection: query.projection.clone(),
            limit: query.limit,
            explain_contract: query.explain_contract,
            explain_analyze: query.explain_analyze,
            schema_revision: schema.revision,
            index_candidates,
            commit_indexes,
        },
        required_catalogue_sources,
    ))
}
