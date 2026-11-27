//! Search results, versions and internal vector filters.

use super::*;

/// Whether an active generation can answer for this collection. A descriptor names a field, which
/// two collections may share, so a generation built for one must not answer for the other: it
/// would propose only identities the other does not hold, and an empty answer would read as a real
/// one. A quantized generation's lifecycle names its collection, and a graph's nodes do. An exact
/// segment confirms what it finds against the canonical versions, so it needs neither.
pub(super) fn generation_serves(
    lifecycle: &rrflow_vector::QuantizationArtifactCatalogue,
    binding: &crate::VectorArtifactBinding,
    artifact: &rrflow_vector::VectorArtifact,
    (collection_id, vector_name): (&CanonicalId, &CanonicalId),
) -> bool {
    // A generation bound to a named source serves that source and nothing else,
    // however many nodes it holds: an empty bound graph must not answer for a
    // populated collection. Only a legacy unbound generation is judged by its
    // contents.
    if let Some(source) = &binding.source {
        return *source
            == rrflow_core::VectorCollectionAddress {
                collection_id: collection_id.as_str().into(),
                vector_name: vector_name.as_str().into(),
            };
    }
    match artifact {
        rrflow_vector::VectorArtifact::Hnsw(index) => {
            index.serves_collection(&rrflow_core::VectorCollectionAddress {
                collection_id: collection_id.as_str().into(),
                vector_name: vector_name.as_str().into(),
            })
        }
        rrflow_vector::VectorArtifact::TurboQuant(_)
        | rrflow_vector::VectorArtifact::Quantized(_) => lifecycle.active_entries().any(|entry| {
            entry.descriptor == binding.descriptor
                && entry.collection_id.as_str() == collection_id.as_str()
                && entry.vector_name.as_str() == vector_name.as_str()
        }),
        rrflow_vector::VectorArtifact::ExactSegment(_)
        | rrflow_vector::VectorArtifact::CompactDense(_) => true,
    }
}

/// What one search read and hydrated, for its public result.
pub(super) struct Searched {
    pub(super) selected_versions: u64,
    pub(super) read_evidence: rrd_contract::ReadEvidence,
    pub(super) canonical_candidates: u64,
    pub(super) loaded_artifact_bytes: u64,
    pub(super) cold_source_hits: u64,
}

pub(super) fn search_result(
    request: &SearchVectors,
    read: ReadStamp,
    searched: Searched,
    prepared: &rrflow_vector::PreparedVectorSearch,
    execution: rrflow_vector::SearchExecution,
) -> Result<VectorSearchResult> {
    let access_path = match execution.plan.selected.kind {
        rrflow_vector::AccessPathKind::ExactScan => "exact_scan",
        rrflow_vector::AccessPathKind::ExactSegment => "exact_segment",
        rrflow_vector::AccessPathKind::Hnsw => "hnsw",
        rrflow_vector::AccessPathKind::ScalarQuantized => "scalar_quantized",
        rrflow_vector::AccessPathKind::ProductQuantized => "product_quantized",
        rrflow_vector::AccessPathKind::BinaryQuantized => "binary_quantized",
        rrflow_vector::AccessPathKind::TurboQuant => "turboquant",
    };
    let resources = VectorSearchResourceEvidence {
        canonical_candidates: searched.canonical_candidates,
        selected_candidates: execution.plan.selected.estimated_candidates,
        ef_search: u64::try_from(prepared.ef_search())
            .map_err(|_| ServiceError::Vector("vector ef_search exceeds u64".into()))?,
        exact_rerank: u64::try_from(execution.plan.selected.exact_rerank)
            .map_err(|_| ServiceError::Vector("vector exact_rerank exceeds u64".into()))?,
        overlay_candidates: execution.plan.selected.overlay_candidates,
        selected_generation: (!matches!(
            execution.plan.selected.kind,
            rrflow_vector::AccessPathKind::ExactScan
        ))
        .then_some(execution.plan.selected.generation),
        loaded_artifact_bytes: searched.loaded_artifact_bytes,
        result_hits: u64::try_from(execution.hits.len())
            .map_err(|_| ServiceError::Vector("vector result count exceeds u64".into()))?,
        cold_source_hits: searched.cold_source_hits,
        rrflowcld_triggered: searched.cold_source_hits > 0,
    };
    Ok(VectorSearchResult {
        scope: request.scope.clone(),
        collection_id: request.collection_id.clone(),
        vector_name: request.vector_name.clone(),
        read_manifest_sha256: read.manifest_id,
        known_at_cursor: read.commit_cursor,
        selected_versions: searched.selected_versions,
        read_evidence: searched.read_evidence,
        plan_sha256: prepared.plan_digest().into(),
        access_path: CanonicalId::new(access_path)
            .map_err(|error| ServiceError::Vector(error.to_string()))?,
        exact: !execution.plan.selected.kind.is_approximate(),
        resources,
        hits: execution
            .hits
            .into_iter()
            .map(|hit| {
                Ok(VectorSearchHit {
                    reference: public_data_ref(&hit.reference)?,
                    subject: public_data_ref(&hit.subject)?,
                    source_cursor: hit.source_cursor,
                    score: hit.score,
                })
            })
            .collect::<Result<_>>()?,
    })
}

/// The visibility a search resolves its candidates at. The payload filter
/// is left to the runtime, which applies it to the version that resolves.
pub(super) fn search_visibility(
    search: &rrflow_vector::SearchRequest,
) -> rrflow_vector::VectorVisibilityRequest {
    rrflow_vector::VectorVisibilityRequest {
        scope: search.scope.clone(),
        read: search.read.clone(),
        valid_at: search.valid_at,
        field: search.field.clone(),
        embedding_model: search.embedding_model.clone(),
        filter: None,
    }
}

/// What remains of one request's key budget after `examined` keys. An
/// exhausted budget is the same denial a read past it raises.
pub(super) fn remaining_budget(budget: u64, examined: u64) -> Result<rrflow_db::RuntimeReadBudget> {
    match budget
        .checked_sub(examined)
        .filter(|remaining| *remaining > 0)
    {
        Some(remaining) => Ok(rrflow_db::RuntimeReadBudget::new(remaining)?),
        None => Err(ServiceError::from(
            rrflow_db::Error::RuntimeReadBudgetExceeded {
                limit: budget,
                observed: examined.saturating_add(1),
            },
        )),
    }
}

pub(super) fn internal_search_mode(
    mode: VectorSearchMode,
) -> Result<(rrflow_vector::SearchMode, usize)> {
    let to_usize = |value: u64, label: &str| {
        usize::try_from(value)
            .map_err(|_| ServiceError::Vector(format!("vector {label} exceeds usize")))
    };
    Ok(match mode {
        VectorSearchMode::Exact => (rrflow_vector::SearchMode::Exact, 1),
        VectorSearchMode::AllowApproximate {
            exact_rerank,
            ef_search,
        } => (
            rrflow_vector::SearchMode::AllowApproximate {
                exact_rerank: to_usize(exact_rerank, "exact_rerank")?,
            },
            to_usize(ef_search, "ef_search")?,
        ),
        VectorSearchMode::RequireApproximate {
            exact_rerank,
            ef_search,
        } => (
            rrflow_vector::SearchMode::RequireApproximate {
                exact_rerank: to_usize(exact_rerank, "exact_rerank")?,
            },
            to_usize(ef_search, "ef_search")?,
        ),
    })
}

pub(super) fn vector_artifact_relevant(
    descriptor: &rrflow_vector::VectorProjectionDescriptor,
    request: &rrflow_vector::SearchRequest,
) -> bool {
    let candidate = descriptor.candidate_path(1);
    if descriptor.scope() != &request.scope
        || candidate.stamp.state != ProjectionState::Ready
        || candidate.field != request.field
        || candidate.dimensions != request.query.dimensions()
        || candidate.metric != request.metric
        || candidate.embedding_model != request.embedding_model
        || request.filter.as_ref().is_some_and(|filter| {
            filter
                .referenced_properties()
                .iter()
                .any(|property| !candidate.filter_properties.contains(property))
        })
    {
        return false;
    }
    match request.mode {
        rrflow_vector::SearchMode::Exact => !candidate.kind.is_approximate(),
        rrflow_vector::SearchMode::AllowApproximate { .. } => true,
        rrflow_vector::SearchMode::RequireApproximate { .. } => candidate.kind.is_approximate(),
    }
}

pub(in crate::engine) fn internal_vector_query(
    query: &VectorSearchQuery,
) -> rrflow_vector::VectorQuery {
    match query {
        VectorSearchQuery::Dense { values } => rrflow_vector::VectorQuery::Dense {
            values: values.clone(),
        },
        VectorSearchQuery::Sparse {
            dimensions,
            indices,
            values,
        } => rrflow_vector::VectorQuery::Sparse {
            dimensions: *dimensions,
            indices: indices.clone(),
            values: values.clone(),
        },
        VectorSearchQuery::MultiDense {
            dimensions,
            vectors,
            comparator: _,
        } => rrflow_vector::VectorQuery::MultiDense {
            dimensions: *dimensions,
            vectors: vectors.clone(),
            comparator: rrflow_vector::MultiVectorComparator::MaxSim,
        },
    }
}

pub(in crate::engine) fn internal_vector_filter(
    filter: &VectorPayloadFilter,
) -> Result<rrflow_vector::FilterExpression> {
    Ok(match filter {
        VectorPayloadFilter::Condition { condition } => {
            rrflow_vector::FilterExpression::Condition {
                condition: rrflow_vector::FilterCondition {
                    property: condition.property.as_str().into(),
                    operator: match &condition.operator {
                        VectorPayloadOperator::Equals { value } => {
                            rrflow_vector::FilterOperator::Equals {
                                value: runtime_value(value)?,
                            }
                        }
                        VectorPayloadOperator::NotEquals { value } => {
                            rrflow_vector::FilterOperator::NotEquals {
                                value: runtime_value(value)?,
                            }
                        }
                        VectorPayloadOperator::In { values } => rrflow_vector::FilterOperator::In {
                            values: values.iter().map(runtime_value).collect::<Result<_>>()?,
                        },
                        VectorPayloadOperator::Range { gt, gte, lt, lte } => {
                            rrflow_vector::FilterOperator::Range {
                                gt: gt.as_ref().map(runtime_value).transpose()?,
                                gte: gte.as_ref().map(runtime_value).transpose()?,
                                lt: lt.as_ref().map(runtime_value).transpose()?,
                                lte: lte.as_ref().map(runtime_value).transpose()?,
                            }
                        }
                        VectorPayloadOperator::Exists { value } => {
                            rrflow_vector::FilterOperator::Exists { value: *value }
                        }
                    },
                },
            }
        }
        VectorPayloadFilter::All { filters } => rrflow_vector::FilterExpression::All {
            filters: filters
                .iter()
                .map(internal_vector_filter)
                .collect::<Result<_>>()?,
        },
        VectorPayloadFilter::Any { filters } => rrflow_vector::FilterExpression::Any {
            filters: filters
                .iter()
                .map(internal_vector_filter)
                .collect::<Result<_>>()?,
        },
        VectorPayloadFilter::Not { filter } => rrflow_vector::FilterExpression::Not {
            filter: Box::new(internal_vector_filter(filter)?),
        },
    })
}
