//! Entry, field and analytics validation, commit bindings and maintenance evidence.

use super::*;

pub(super) fn validate_index_entry(id: &ProjectionId, entry: &IndexEntry) -> Result<()> {
    entry.definition.validate()?;
    entry
        .stamp
        .validate()
        .map_err(|error| Error::Integrity(format!("invalid index stamp for {id}: {error}")))?;
    if id != &entry.definition.id
        || id != &entry.stamp.id
        || entry.stamp.config_digest != entry.definition.config_digest()?
    {
        return Err(Error::Integrity(format!(
            "index identity or configuration digest disagrees for {id}"
        )));
    }
    if entry.stamp.state == ProjectionState::Ready
        && (entry.built_valid_at.is_none() || entry.artifact_rows.is_none())
    {
        return Err(Error::Integrity(format!(
            "ready index {id} is missing artifact coverage"
        )));
    }
    let analytics_kind = matches!(
        entry.definition.kind,
        IndexKind::Count | IndexKind::AggregateCount
    );
    if entry.stamp.state == ProjectionState::Ready
        && entry.maintenance.is_some()
        && (analytics_kind
            != (entry.analytics_total_count.is_some() && entry.analytics_group_count.is_some()))
    {
        return Err(Error::Integrity(format!(
            "ready index {id} has inconsistent analytics summaries"
        )));
    }
    if entry.stamp.state != ProjectionState::Building && entry.prior_artifact.is_some() {
        return Err(Error::Integrity(format!(
            "non-building index {id} retained a prior artifact pointer"
        )));
    }
    if entry.unique_validated && !entry.definition.unique {
        return Err(Error::Integrity(format!(
            "non-unique index {id} claims unique-constraint validation"
        )));
    }
    Ok(())
}

pub(crate) fn index_artifact_name(
    scope: &ScopeId,
    id: &ProjectionId,
    generation: u64,
    artifact_digest: &str,
) -> String {
    format!("query-index/{scope}/{id}/{generation}/{artifact_digest}")
}

pub(super) fn validate_fields(catalogue: &Catalog, definition: &IndexDefinition) -> Result<()> {
    let mut query = Query::new(
        definition.source.clone(),
        TemporalSelector {
            valid_at: TimeExpr::Literal(0),
            known_at: CursorExpr::Head,
        },
    );
    query.filters = definition.filters.clone();
    query.projection = if definition.fields.is_empty() {
        Projection::All
    } else {
        Projection::Fields(definition.fields.clone())
    };
    let bound = bind(&query, &Parameters::new(), catalogue)?;
    if let IndexKind::Bm25 { .. } = definition.kind {
        let field = &definition.fields[0];
        let accepted =
            super::super::plan::field_types_for_bound_source(&bound.source, catalogue, field)?;
        if !accepted.contains(&rrd_core::RuntimeValueType::String) {
            return Err(Error::Catalog(format!(
                "BM25 field {field:?} is not a string field"
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_unique_rows(definition: &IndexDefinition, rows: &[QueryRow]) -> Result<()> {
    if !definition.unique {
        return Ok(());
    }
    let mut keys = BTreeMap::<Vec<u8>, &str>::new();
    for row in rows {
        let values = definition
            .fields
            .iter()
            .map(|field| row.values.get(field).cloned().unwrap_or(RuntimeValue::Null))
            .collect::<Vec<_>>();
        if values.contains(&RuntimeValue::Null) {
            continue;
        }
        let key = serde_json::to_vec(&values)?;
        if let Some(existing) = keys.insert(key, &row.identity) {
            return Err(Error::Catalog(format!(
                "unique index {} rejects duplicate rows {existing:?} and {:?}",
                definition.id, row.identity
            )));
        }
    }
    Ok(())
}

pub(super) fn commit_binding_definitions(
    catalogue: &IndexCatalogue,
) -> Result<Vec<IndexCommitBindingDefinition>> {
    catalogue
        .entries
        .values()
        .filter_map(|entry| {
            let Source::Record { kind: record_kind } = &entry.definition.source else {
                return None;
            };
            let binding_kind = match entry.definition.kind {
                IndexKind::Scalar => IndexCommitBindingKind::Scalar {
                    unique: entry.definition.unique,
                },
                IndexKind::Bm25 { .. } => IndexCommitBindingKind::Bm25,
                IndexKind::Count
                | IndexKind::Geo
                | IndexKind::MaterializedView
                | IndexKind::AggregateCount => return None,
            };
            Some(
                entry
                    .definition
                    .config_digest()
                    .map(|configuration_sha256| IndexCommitBindingDefinition {
                        id: entry.definition.id.clone(),
                        record_kind: record_kind.clone(),
                        fields: entry.definition.fields.clone(),
                        kind: binding_kind,
                        configuration_sha256,
                        lexical: match &entry.definition.kind {
                            IndexKind::Bm25 { config } => Some(config.clone()),
                            _ => None,
                        },
                    }),
            )
        })
        .collect()
}

pub(super) fn is_false(value: &bool) -> bool {
    !*value
}

pub(super) fn build_analytics(
    definition: &IndexDefinition,
    rows: &[QueryRow],
) -> Result<IndexAnalyticsArtifact> {
    let total_count =
        u64::try_from(rows.len()).map_err(|_| Error::Budget("index count exceeds u64".into()))?;
    let groups = if matches!(definition.kind, IndexKind::AggregateCount) {
        let mut grouped = BTreeMap::<Vec<u8>, (Vec<RuntimeValue>, u64)>::new();
        for row in rows {
            let values = definition
                .fields
                .iter()
                .map(|field| row.values.get(field).cloned().unwrap_or(RuntimeValue::Null))
                .collect::<Vec<_>>();
            let key = serde_json::to_vec(&values)?;
            let group = grouped.entry(key).or_insert((values, 0));
            group.1 = group
                .1
                .checked_add(1)
                .ok_or_else(|| Error::Budget("grouped count overflow".into()))?;
        }
        grouped
            .into_values()
            .map(|(values, count)| IndexCountGroup { values, count })
            .collect()
    } else {
        Vec::new()
    };
    Ok(IndexAnalyticsArtifact {
        total_count,
        groups,
    })
}

pub(super) fn validate_analytics(
    definition: &IndexDefinition,
    analytics: &IndexAnalyticsArtifact,
) -> Result<()> {
    if matches!(definition.kind, IndexKind::Count) && !analytics.groups.is_empty() {
        return Err(Error::Integrity(
            "a count index cannot contain groups".into(),
        ));
    }
    if matches!(definition.kind, IndexKind::AggregateCount) {
        let grouped_total = analytics.groups.iter().try_fold(0_u64, |sum, group| {
            if group.values.len() != definition.fields.len() || group.count == 0 {
                return Err(Error::Integrity("grouped-count entry is invalid".into()));
            }
            sum.checked_add(group.count)
                .ok_or_else(|| Error::Integrity("grouped-count total overflow".into()))
        })?;
        if grouped_total != analytics.total_count {
            return Err(Error::Integrity(
                "grouped-count total does not match its groups".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn maintenance_evidence<E: StorageEngine>(
    engine: &E,
    scope: &ScopeId,
    id: &ProjectionId,
    prior: Option<&IndexArtifactReference>,
    source_cursor: u64,
    rows: &[QueryRow],
) -> Result<IndexMaintenanceEvidence> {
    let Some(prior) = prior else {
        return Ok(IndexMaintenanceEvidence {
            mode: "full_build".into(),
            prior_source_cursor: None,
            source_cursor,
            inserted_rows: u64::try_from(rows.len())
                .map_err(|_| Error::Budget("index row count exceeds u64".into()))?,
            updated_rows: 0,
            removed_rows: 0,
        });
    };
    let name = index_artifact_name(scope, id, prior.generation, &prior.artifact_digest);
    let bytes = engine
        .projections()
        .get(&name)?
        .ok_or_else(|| Error::Integrity("prior index artifact is missing".into()))?;
    let artifact = IndexArtifact::decode(&bytes)?;
    if artifact.source_cursor != prior.source_cursor || artifact.valid_at != prior.valid_at {
        return Err(Error::Integrity(
            "prior index artifact coordinates disagree with the catalogue".into(),
        ));
    }
    let old = artifact
        .rows
        .iter()
        .map(|row| (row.identity.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let new = rows
        .iter()
        .map(|row| (row.identity.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let inserted_rows = new
        .keys()
        .filter(|identity| !old.contains_key(*identity))
        .count();
    let removed_rows = old
        .keys()
        .filter(|identity| !new.contains_key(*identity))
        .count();
    let updated_rows = new
        .iter()
        .filter(|(identity, row)| old.get(*identity).is_some_and(|old| *old != **row))
        .count();
    Ok(IndexMaintenanceEvidence {
        mode: "incremental_reconciliation".into(),
        prior_source_cursor: Some(prior.source_cursor),
        source_cursor,
        inserted_rows: u64::try_from(inserted_rows)
            .map_err(|_| Error::Budget("inserted row count exceeds u64".into()))?,
        updated_rows: u64::try_from(updated_rows)
            .map_err(|_| Error::Budget("updated row count exceeds u64".into()))?,
        removed_rows: u64::try_from(removed_rows)
            .map_err(|_| Error::Budget("removed row count exceeds u64".into()))?,
    })
}
