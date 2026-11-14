//! Bound version, adjacency and identity-range sources, and the plan digest.

use super::*;

pub(super) fn bound_version_sources(
    query: &Query,
    filters: &[BoundFilter],
    schema: &RuntimeSchemaRegistry,
) -> Result<Vec<RuntimeVersionedSource>> {
    if query.join.is_none() {
        // ID equality addresses one temporal identity. Preserve every logical
        // predicate for evaluation after materialization. MATCH depends on
        // family-wide corpus statistics and must keep its original source.
        if !filters.iter().any(|filter| filter.comparison.is_match()) {
            if let Source::Record { kind } | Source::Relation { kind } = &query.source {
                if let Some(RuntimeValue::String(id)) = filters.iter().find_map(|filter| {
                    (filter.field == "id" && filter.comparison == ComparisonOperator::Equal)
                        .then_some(&filter.value)
                }) {
                    // An arbitrary comparison string need not be a valid
                    // runtime ID. Preserve its existing empty-result behavior.
                    if let Ok(reference) = rrd_core::RuntimeRef::parse(kind.to_string(), id.clone())
                    {
                        let model = schema
                            .logical_model(kind)
                            .map_err(|error| Error::Binding(error.to_string()))?;
                        return Ok(vec![RuntimeVersionedSource::Identity { model, reference }]);
                    }
                }
            }
            if let Some(source) = bound_adjacency_source(&query.source, filters, schema) {
                return Ok(vec![source]);
            }
            if let Some(source) = bound_identity_range(&query.source, filters, schema) {
                return Ok(vec![source]);
            }
        }
        if let Source::Event { kind } = &query.source {
            if let Some(cursor) = filters.iter().find_map(|filter| {
                (filter.field == "cursor" && filter.comparison == ComparisonOperator::Equal)
                    .then_some(&filter.value)
                    .and_then(|value| match value {
                        RuntimeValue::Unsigned(cursor) => Some(*cursor),
                        _ => None,
                    })
            }) {
                let model = schema
                    .logical_model(kind)
                    .map_err(|error| Error::Binding(error.to_string()))?;
                if !model.is_event_like() {
                    return Err(Error::Binding(format!(
                        "event source {kind} resolves to non-event model {model:?}"
                    )));
                }
                return Ok(vec![RuntimeVersionedSource::Identity {
                    model,
                    reference: rrd_core::RuntimeRef::event(kind.clone(), cursor),
                }]);
            }
        }
    }
    let mut sources = Vec::new();
    for source in std::iter::once(&query.source).chain(query.join.as_ref().map(|join| &join.source))
    {
        match source {
            Source::Traversal { start, .. } => {
                let model = schema
                    .logical_model(&start.kind)
                    .map_err(|error| Error::Binding(error.to_string()))?;
                if !model.is_record_like() {
                    return Err(Error::Binding(format!(
                        "traversal start type {} resolves to non-record model {model:?}",
                        start.kind
                    )));
                }
                sources.push(RuntimeVersionedSource::Identity {
                    model,
                    reference: start.clone(),
                });
            }
            source => sources.extend(version_sources_for_sources(std::slice::from_ref(source))),
        }
    }
    Ok(normalize_version_sources(sources))
}

/// Only a fully addressed endpoint can narrow the relation source. Multiple
/// allowed kinds without an explicit kind predicate retain the family scan.
pub(super) fn bound_adjacency_source(
    source: &Source,
    filters: &[BoundFilter],
    schema: &RuntimeSchemaRegistry,
) -> Option<RuntimeVersionedSource> {
    let Source::Relation { kind } = source else {
        return None;
    };
    let definition = schema.relations.get(kind)?;
    let equality = |field: &str| {
        filters.iter().find_map(|filter| {
            if filter.field == field && filter.comparison == ComparisonOperator::Equal {
                if let RuntimeValue::String(value) = &filter.value {
                    return Some(value.as_str());
                }
            }
            None
        })
    };
    for (id_field, kind_field, allowed, direction) in [
        (
            "from_id",
            "from_kind",
            &definition.from,
            rrd_store::RuntimeAdjacencyDirection::Outgoing,
        ),
        (
            "to_id",
            "to_kind",
            &definition.to,
            rrd_store::RuntimeAdjacencyDirection::Incoming,
        ),
    ] {
        let Some(id) = equality(id_field) else {
            continue;
        };
        let node_kind = equality(kind_field).or_else(|| {
            (allowed.len() == 1)
                .then(|| allowed.first().expect("one allowed endpoint kind").as_str())
        });
        let Some(node_kind) = node_kind else { continue };
        let Ok(node) = rrd_core::RuntimeRef::parse(node_kind, id) else {
            continue;
        };
        return Some(RuntimeVersionedSource::Adjacency {
            relation: kind.clone(),
            node,
            direction,
        });
    }
    None
}

pub(super) fn bound_identity_range(
    source: &Source,
    filters: &[BoundFilter],
    schema: &RuntimeSchemaRegistry,
) -> Option<RuntimeVersionedSource> {
    use std::ops::Bound;
    let (Source::Record { kind } | Source::Relation { kind }) = source else {
        return None;
    };
    let mut lower = Bound::Unbounded;
    let mut upper = Bound::Unbounded;
    for filter in filters.iter().filter(|filter| filter.field == "id") {
        let RuntimeValue::String(value) = &filter.value else {
            continue;
        };
        let Ok(id) = rrd_core::RuntimeId::parse(value.clone()) else {
            continue;
        };
        match filter.comparison {
            ComparisonOperator::GreaterThan | ComparisonOperator::GreaterThanOrEqual => {
                let strict = filter.comparison == ComparisonOperator::GreaterThan;
                let tighter = match &lower {
                    Bound::Unbounded => true,
                    Bound::Included(previous) => id > *previous || (id == *previous && strict),
                    Bound::Excluded(previous) => id > *previous,
                };
                if tighter {
                    lower = if strict {
                        Bound::Excluded(id)
                    } else {
                        Bound::Included(id)
                    };
                }
            }
            ComparisonOperator::LessThan | ComparisonOperator::LessThanOrEqual => {
                let strict = filter.comparison == ComparisonOperator::LessThan;
                let tighter = match &upper {
                    Bound::Unbounded => true,
                    Bound::Included(previous) => id < *previous || (id == *previous && strict),
                    Bound::Excluded(previous) => id < *previous,
                };
                if tighter {
                    upper = if strict {
                        Bound::Excluded(id)
                    } else {
                        Bound::Included(id)
                    };
                }
            }
            _ => {}
        }
    }
    if matches!((&lower, &upper), (Bound::Unbounded, Bound::Unbounded))
        || matches!((&lower, &upper),
            (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b) | Bound::Excluded(b)) if a > b)
    {
        return None;
    }
    Some(RuntimeVersionedSource::IdentityRange {
        model: schema.logical_model(kind).ok()?,
        kind: kind.clone(),
        lower,
        upper,
    })
}

impl PhysicalPlan {
    pub fn verify(&self) -> Result<()> {
        if self.logical.contract_version != QUERY_CONTRACT_VERSION {
            return Err(Error::Integrity(format!(
                "unsupported query contract version {}",
                self.logical.contract_version
            )));
        }
        if self.logical.result_identity_version
            != rrd_core::result_identity::RESULT_IDENTITY_VERSION
        {
            return Err(Error::Integrity(
                "physical plan has an unsupported result identity version".into(),
            ));
        }
        self.logical
            .read
            .validate()
            .map_err(|error| Error::Integrity(error.to_string()))?;
        let expected = plan_digest(&self.logical, &self.operators, &self.explanation)?;
        if self.digest != expected {
            return Err(Error::Integrity(
                "physical plan digest does not match its contract".into(),
            ));
        }
        if !self.explanation.contract.exact {
            return Err(Error::Integrity(
                "DataFusion executor requires an exact plan".into(),
            ));
        }
        Ok(())
    }
}

pub(super) fn plan_digest(
    logical: &LogicalPlan,
    physical: &[PhysicalOperator],
    explanation: &PlanExplanation,
) -> Result<String> {
    let bytes = serde_json::to_vec(&(logical, physical, explanation))?;
    Ok(digest::sha256_hex(&bytes))
}

pub(super) fn resolve_u64_time(
    value: &TimeExpr,
    parameters: &Parameters,
    label: &str,
) -> Result<u64> {
    match value {
        TimeExpr::Literal(value) => Ok(*value),
        TimeExpr::Parameter(name) => parameter_u64(parameters, name, label),
    }
}
