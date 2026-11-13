use crate::physical::graph::{AdjacencyExpansion, TraversalPlan};
use crate::{
    execute_snapshot, highlight_offsets, index::index_artifact_name, rows_to_arrow_snapshot,
    ArrowReadStamp, Bm25Artifact, Bm25Config, Bm25Offset, BoundFilter, Error, FusionAnalysis,
    FusionBudget, IndexDefinition, IndexKind, IndexMaintenanceEvidence, LogicalOperator,
    PhysicalOperator, PhysicalPlan, Result, INDEX_ARTIFACT_CONTRACT_VERSION,
};
use crate::{Join, Projection, Source};
use rrd_core::result_identity::{self, ResultIdentityFamily, RESULT_IDENTITY_VERSION};
use rrd_core::{
    resolve_as_of, Claim, GeoValue, RuntimeChange, RuntimeGeo, RuntimeMutation, RuntimeRecord,
    RuntimeRelation, RuntimeValue, SeriesValue,
};
use rrd_store::{RuntimeReadBudget, RuntimeReadEvidence, RuntimeVersionedSource, StorageEngine};
use serde::de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBudget {
    pub max_storage_keys: u64,
    pub max_input_rows: usize,
    pub max_rows: usize,
    pub max_output_bytes: usize,
    pub max_batch_rows: usize,
    pub max_memory_bytes: usize,
    pub max_spill_bytes: usize,
    pub max_elapsed_ms: u64,
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self {
            max_storage_keys: 100_000,
            max_input_rows: 100_000,
            max_rows: 10_000,
            max_output_bytes: 8 * 1024 * 1024,
            max_batch_rows: 256,
            max_memory_bytes: 64 * 1024 * 1024,
            max_spill_bytes: 256 * 1024 * 1024,
            max_elapsed_ms: 30_000,
        }
    }
}

impl ExecutionBudget {
    fn validate(&self) -> Result<()> {
        if self.max_storage_keys == 0
            || self.max_input_rows == 0
            || self.max_rows == 0
            || self.max_output_bytes == 0
            || self.max_batch_rows == 0
            || self.max_memory_bytes == 0
            || self.max_spill_bytes == 0
            || self.max_elapsed_ms == 0
        {
            return Err(Error::Budget(
                "all execution budgets must be greater than zero".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryRow {
    pub identity: String,
    pub values: BTreeMap<String, RuntimeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryBatch {
    pub ordinal: usize,
    pub rows: Vec<QueryRow>,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryExecution {
    pub plan_digest: String,
    pub read_manifest: String,
    pub valid_at: u64,
    pub known_at_cursor: u64,
    /// Number of authenticated semantic versions selected before temporal
    /// reduction. Index-backed paths report zero because their source rows are
    /// carried by the separately verified immutable artifact.
    pub selected_versions: usize,
    pub read_evidence: RuntimeReadEvidence,
    pub returned_rows: usize,
    pub output_bytes: usize,
    pub truncated: bool,
    pub batches: Vec<QueryBatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<FusionAnalysis>,
}

pub fn execute<E: StorageEngine>(
    engine: &E,
    plan: &PhysicalPlan,
    budget: &ExecutionBudget,
) -> Result<QueryExecution> {
    plan.verify()?;
    budget.validate()?;
    let contract = &plan.explanation.contract;
    if contract.read_manifest != plan.logical.read.manifest_id {
        return Err(Error::Integrity(
            "plan explanation names a different read stamp".into(),
        ));
    }
    let shape = PlanShape::read(plan)?;
    let access = ReadPath::from_plan(plan, &shape)?;
    let mut loaded = access.load(
        engine,
        &plan.logical.read,
        &shape,
        RuntimeReadBudget::new(budget.max_storage_keys)?,
    )?;
    let input_rows = loaded
        .rows
        .len()
        .saturating_add(loaded.right_rows.as_ref().map_or(0, std::vec::Vec::len));
    if input_rows > budget.max_input_rows {
        return Err(Error::Budget(format!(
            "query selected {input_rows} input rows, budget allows {}",
            budget.max_input_rows
        )));
    }
    if let Some(join) = &shape.join {
        let right = loaded.right_rows.take().ok_or_else(|| {
            Error::Integrity("join plan did not load its right-hand source".into())
        })?;
        loaded.rows = exact_join_rows(&loaded.rows, &right, join, budget.max_rows)?;
    } else if loaded.right_rows.is_some() {
        return Err(Error::Integrity(
            "single-source plan unexpectedly loaded join rows".into(),
        ));
    }
    // BM25 corpus statistics are calculated from the complete authoritative
    // source snapshot, before relational predicates narrow the result set.
    let bm25_scores = match loaded.ranked.take() {
        Some(ranked) => ranked_scores(&loaded.rows, &shape, ranked)?,
        None => bm25_scores(&loaded.rows, &shape, loaded.bm25)?,
    };
    let has_match = shape
        .filters
        .iter()
        .any(|filter| filter.comparison.is_match());
    let snapshot = rows_to_arrow_snapshot(
        &loaded.rows,
        &plan.logical.field_types,
        ArrowReadStamp {
            read_manifest: plan.logical.read.manifest_id.clone(),
            scope: plan.logical.read.scope.to_string(),
            valid_at: shape.valid_at,
            known_at_cursor: shape.known_at_cursor,
            source_cursor: plan.logical.source_cursor,
            schema_revision: plan.logical.schema_revision,
        },
        budget.max_batch_rows,
    )?;
    loaded.rows.clear();
    loaded.rows.shrink_to_fit();
    let fusion = execute_snapshot(
        snapshot,
        &shape.filters,
        &shape.projection,
        shape.limit,
        has_match,
        &FusionBudget {
            max_batch_rows: budget.max_batch_rows,
            max_memory_bytes: budget.max_memory_bytes,
            max_spill_bytes: budget.max_spill_bytes,
            max_elapsed_ms: budget.max_elapsed_ms,
        },
    )?;
    let analysis = plan.logical.explain_analyze.then_some(fusion.analysis);
    let mut rows = fusion.rows;
    if has_match {
        rows.retain(|row| bm25_scores.contains_key(&row.identity));
        rows.sort_by(|left, right| {
            bm25_scores[&right.identity]
                .score
                .total_cmp(&bm25_scores[&left.identity].score)
                .then_with(|| left.identity.cmp(&right.identity))
        });
        for row in &mut rows {
            let evidence = &bm25_scores[&row.identity];
            row.values.insert(
                "_score".into(),
                RuntimeValue::Decimal(format!("{:.17}", evidence.score)),
            );
            row.values.insert(
                "_matched_terms".into(),
                RuntimeValue::List(
                    evidence
                        .matched_terms
                        .iter()
                        .cloned()
                        .map(RuntimeValue::String)
                        .collect(),
                ),
            );
            row.values.insert(
                "_highlight_offsets".into(),
                RuntimeValue::List(evidence.offsets.iter().map(public_bm25_offset).collect()),
            );
            row.values.insert(
                "_highlight".into(),
                RuntimeValue::String(evidence.highlight.clone()),
            );
        }
        if let Some(query_limit) = shape.limit {
            rows.truncate(query_limit);
        }
    }
    let semantic_rows = rows.len();
    rows.truncate(budget.max_rows);
    let row_truncated = rows.len() < semantic_rows;
    let mut accepted = Vec::with_capacity(rows.len());
    let mut output_bytes = 0usize;
    let mut byte_truncated = false;
    for mut row in rows {
        apply_projection(&mut row, &shape.projection);
        let bytes = serde_json::to_vec(&row)?.len();
        if output_bytes.saturating_add(bytes) > budget.max_output_bytes {
            byte_truncated = true;
            break;
        }
        output_bytes += bytes;
        accepted.push(row);
    }
    let returned_rows = accepted.len();
    let truncated = byte_truncated || row_truncated;
    let chunk_count = returned_rows.div_ceil(budget.max_batch_rows);
    let batches = accepted
        .chunks(budget.max_batch_rows)
        .enumerate()
        .map(|(ordinal, rows)| QueryBatch {
            ordinal,
            rows: rows.to_vec(),
            done: ordinal + 1 == chunk_count,
        })
        .collect();
    Ok(QueryExecution {
        plan_digest: plan.digest.clone(),
        read_manifest: plan.logical.read.manifest_id.clone(),
        valid_at: contract.valid_at,
        known_at_cursor: contract.known_at_cursor,
        selected_versions: loaded.selected_versions,
        read_evidence: loaded.read_evidence,
        returned_rows,
        output_bytes,
        truncated,
        batches,
        analysis,
    })
}

enum ReadPath {
    Versioned {
        sources: Vec<RuntimeVersionedSource>,
    },
    Adjacency {
        sources: Vec<RuntimeVersionedSource>,
        traversals: Vec<TraversalPlan>,
    },
    Index {
        id: rrd_core::ProjectionId,
        kind: IndexKind,
        generation: u64,
        source_cursor: u64,
        schema_revision: u64,
        valid_at: u64,
        config_digest: String,
        artifact_digest: String,
        artifact_rows: u64,
    },
    CommitIndex {
        id: rrd_core::ProjectionId,
        probe: crate::plan::CommitIndexProbe,
        model: rrd_core::RuntimeLogicalModel,
        fallback: Vec<RuntimeVersionedSource>,
    },
}

struct LoadedAccess {
    rows: Vec<QueryRow>,
    right_rows: Option<Vec<QueryRow>>,
    bm25: Option<Bm25Artifact>,
    /// Scores a commit-bound BM25 index computed over every current document.
    ranked: Option<Ranked>,
    selected_versions: usize,
    read_evidence: RuntimeReadEvidence,
}

/// A commit-bound BM25 ranking: scores and matched terms by row identity, and
/// the configuration that analysed them.
struct Ranked {
    config: Bm25Config,
    hits: BTreeMap<String, (f64, Vec<String>)>,
}

impl ReadPath {
    fn from_plan(plan: &PhysicalPlan, shape: &PlanShape) -> Result<Self> {
        let contract = &plan.explanation.contract;
        if contract.read_manifest != plan.logical.read.manifest_id
            || contract.scope != plan.logical.read.scope.as_str()
            || contract.valid_at != shape.valid_at
            || contract.known_at_cursor != shape.known_at_cursor
            || contract.source_cursor != plan.logical.source_cursor
            || contract.schema_revision != plan.logical.schema_revision
            || contract.authorization_boundary != format!("scope:{}", plan.logical.read.scope)
            || contract.read_authentication != "rfc9162_authenticated_semantic_versions"
            || contract.read_budget_unit != "physical_keys_examined"
        {
            return Err(Error::Integrity(
                "logical plan, read stamp, and execution contract disagree".into(),
            ));
        }
        let selected = plan
            .explanation
            .candidates
            .iter()
            .filter(|candidate| candidate.selected)
            .collect::<Vec<_>>();
        if selected.len() != 1 || !selected[0].exact {
            return Err(Error::Integrity(
                "physical plan must report exactly one exact selected path".into(),
            ));
        }
        let [access, PhysicalOperator::DataFusionEvaluate] = plan.operators.as_slice() else {
            return Err(Error::Integrity(
                "physical plan must contain one authoritative access path followed by DataFusion evaluation"
                    .into(),
            ));
        };
        let traversals = std::iter::once(&shape.source)
            .chain(shape.join.as_ref().map(|join| &join.source))
            .filter_map(TraversalPlan::from_source)
            .collect::<Vec<_>>();
        match access {
            PhysicalOperator::VersionedRead {
                sources,
                exact,
                stable_order,
            } if *exact
                && traversals.is_empty()
                && stable_order == "global_cursor"
                && sources == &plan.logical.version_sources
                && selected[0].name
                    == if matches!(
                        sources.as_slice(),
                        [RuntimeVersionedSource::Identity { .. }]
                    ) {
                        "versioned_identity_read"
                    } else if matches!(
                        sources.as_slice(),
                        [RuntimeVersionedSource::IdentityRange { .. }]
                    ) {
                        "versioned_identity_range_read"
                    } else if matches!(
                        sources.as_slice(),
                        [RuntimeVersionedSource::Adjacency { .. }]
                    ) {
                        "versioned_adjacency_read"
                    } else {
                        "versioned_source_read"
                    } =>
            {
                Ok(Self::Versioned {
                    sources: sources.clone(),
                })
            }
            PhysicalOperator::AdjacencyTraversal {
                sources,
                exact,
                stable_order,
            } if *exact
                && !traversals.is_empty()
                && stable_order == "global_cursor"
                && sources == &plan.logical.version_sources
                && selected[0].name == "adjacency_traversal"
                && traversals.iter().all(|traversal| {
                    sources.iter().any(|source| match source {
                        RuntimeVersionedSource::Identity { model, reference } => {
                            model.is_record_like() && reference == &traversal.start
                        }
                        RuntimeVersionedSource::Records { kind } => kind
                            .as_ref()
                            .is_none_or(|kind| kind == &traversal.start.kind),
                        _ => false,
                    })
                }) =>
            {
                Ok(Self::Adjacency {
                    sources: sources.clone(),
                    traversals,
                })
            }
            PhysicalOperator::CommitIndexRead {
                id,
                probe,
                model,
                fallback,
                exact,
                stable_order,
            } if *exact
                && traversals.is_empty()
                && shape.join.is_none()
                && fallback == &plan.logical.version_sources
                && stable_order
                    == if matches!(probe, crate::plan::CommitIndexProbe::Bm25 { .. }) {
                        "bm25_score_desc_identity_ascending"
                    } else {
                        "global_cursor"
                    }
                && selected[0].name == format!("commit_index:{id}") =>
            {
                Ok(Self::CommitIndex {
                    id: id.clone(),
                    probe: probe.clone(),
                    model: *model,
                    fallback: fallback.clone(),
                })
            }
            PhysicalOperator::MaterializedIndex {
                id,
                kind,
                generation,
                source_cursor,
                schema_revision,
                valid_at,
                config_digest,
                artifact_digest,
                artifact_rows,
                exact,
                stable_order,
            } if *source_cursor == plan.logical.source_cursor
                && *schema_revision == plan.logical.schema_revision
                && *valid_at == shape.valid_at
                && *exact
                && stable_order
                    == if matches!(kind, IndexKind::Bm25 { .. }) {
                        "bm25_score_desc_identity_ascending"
                    } else {
                        "identity_ascending"
                    }
                && selected[0].name == format!("index:{id}") =>
            {
                Ok(Self::Index {
                    id: id.clone(),
                    kind: kind.clone(),
                    generation: *generation,
                    source_cursor: *source_cursor,
                    schema_revision: *schema_revision,
                    valid_at: *valid_at,
                    config_digest: config_digest.clone(),
                    artifact_digest: artifact_digest.clone(),
                    artifact_rows: *artifact_rows,
                })
            }
            _ => Err(Error::Integrity(
                "physical access path does not match the stamped logical query".into(),
            )),
        }
    }

    fn load<E: StorageEngine>(
        &self,
        engine: &E,
        stamp: &rrd_core::ReadStamp,
        shape: &PlanShape,
        budget: RuntimeReadBudget,
    ) -> Result<LoadedAccess> {
        if let Self::CommitIndex {
            id,
            probe,
            model,
            fallback,
        } = self
        {
            let max_keys = budget.max_keys;
            let (selected, ranked, index_keys) = match probe {
                crate::plan::CommitIndexProbe::Scalar { values } => {
                    match engine.runtime().index_lookup(
                        stamp,
                        id,
                        values,
                        shape.valid_at,
                        usize::try_from(max_keys).unwrap_or(usize::MAX),
                    )? {
                        rrd_store::IndexLookup::Records {
                            references,
                            keys_read,
                        } => (Some(references), None, keys_read),
                        rrd_store::IndexLookup::NotApplicable(_) => (None, None, 0),
                    }
                }
                crate::plan::CommitIndexProbe::Bm25 {
                    query,
                    config,
                    top_k,
                } => match engine.runtime().lexical_search(
                    stamp,
                    id,
                    query,
                    shape.valid_at,
                    *top_k,
                    max_keys,
                )? {
                    rrd_store::LexicalSearch::Hits {
                        hits, keys_read, ..
                    } => {
                        let references = hits.iter().map(|hit| hit.record.clone()).collect();
                        let ranked = Ranked {
                            config: config.clone(),
                            hits: hits
                                .into_iter()
                                .map(|hit| {
                                    (
                                        result_identity::typed(
                                            ResultIdentityFamily::Record,
                                            &hit.record,
                                        ),
                                        (hit.score, hit.matched_terms),
                                    )
                                })
                                .collect(),
                        };
                        (Some(references), Some(ranked), keys_read)
                    }
                    rrd_store::LexicalSearch::NotApplicable(_) => (None, None, 0),
                },
            };
            // The index's keys count against the same read budget.
            let budget = RuntimeReadBudget::new(
                max_keys
                    .checked_sub(index_keys)
                    .filter(|keys| *keys > 0)
                    .ok_or_else(|| {
                        Error::Budget(format!(
                            "index {id} read {index_keys} keys of a {max_keys}-key budget"
                        ))
                    })?,
            )?;
            let Some(references) = selected else {
                return Self::Versioned {
                    sources: fallback.clone(),
                }
                .load(engine, stamp, shape, budget);
            };
            if references.is_empty() {
                let validation = engine.runtime().read_versioned(
                    stamp,
                    &[RuntimeVersionedSource::Schema],
                    budget,
                )?;
                return Ok(LoadedAccess {
                    rows: Vec::new(),
                    right_rows: None,
                    bm25: None,
                    ranked: ranked.map(|ranked| Ranked {
                        hits: BTreeMap::new(),
                        ..ranked
                    }),
                    selected_versions: 0,
                    read_evidence: validation.evidence,
                });
            }
            let sources = references
                .into_iter()
                .map(|reference| RuntimeVersionedSource::Identity {
                    model: *model,
                    reference,
                })
                .collect::<Vec<_>>();
            let mut loaded = Self::Versioned { sources }.load(engine, stamp, shape, budget)?;
            loaded.ranked = ranked;
            return Ok(loaded);
        }
        if let Self::Index {
            id,
            kind,
            generation,
            source_cursor,
            schema_revision,
            valid_at,
            config_digest,
            artifact_digest,
            artifact_rows,
        } = self
        {
            let validation = engine.runtime().read_versioned(
                stamp,
                &[RuntimeVersionedSource::Schema],
                budget,
            )?;
            let name = index_artifact_name(&stamp.scope, id, *generation, artifact_digest);
            let bytes = engine
                .projections()
                .get(&name)?
                .ok_or_else(|| Error::Integrity(format!("index artifact {id} is missing")))?;
            if rrd_core::digest::sha256_hex(&bytes) != *artifact_digest {
                return Err(Error::Integrity(format!(
                    "index artifact {id} digest does not verify"
                )));
            }
            let row_limit = if shape.filters.is_empty()
                && !matches!(kind, IndexKind::Bm25 { .. })
                && shape.join.is_none()
            {
                shape.limit
            } else {
                None
            };
            let artifact = decode_index_artifact_for_read(&bytes, row_limit)?;
            if artifact.scope != stamp.scope
                || artifact.definition.id != *id
                || artifact.definition.kind != *kind
                || artifact.definition.source != shape.source
                || artifact.definition.config_digest()? != *config_digest
                || artifact.generation != *generation
                || artifact.source_cursor != *source_cursor
                || artifact.read_cursor > shape.known_at_cursor
                || artifact.schema_revision != *schema_revision
                || artifact.valid_at != *valid_at
                || artifact.total_rows
                    != usize::try_from(*artifact_rows)
                        .map_err(|_| Error::Budget("index artifact rows exceed usize".into()))?
            {
                return Err(Error::Integrity(format!(
                    "index artifact {id} does not satisfy the physical plan"
                )));
            }
            return Ok(LoadedAccess {
                rows: artifact.rows,
                right_rows: None,
                bm25: artifact.bm25,
                ranked: None,
                selected_versions: 0,
                read_evidence: validation.evidence,
            });
        }

        struct LoadedIndexArtifact {
            scope: rrd_core::ScopeId,
            definition: IndexDefinition,
            generation: u64,
            source_cursor: u64,
            read_cursor: u64,
            schema_revision: u64,
            valid_at: u64,
            rows: Vec<QueryRow>,
            total_rows: usize,
            bm25: Option<Bm25Artifact>,
        }

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct IndexArtifactEnvelope<'a> {
            contract_version: u16,
            result_identity_version: u16,
            scope: rrd_core::ScopeId,
            definition: IndexDefinition,
            generation: u64,
            source_cursor: u64,
            read_cursor: u64,
            schema_revision: u64,
            valid_at: u64,
            #[serde(borrow)]
            rows: &'a RawValue,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            bm25: Option<Bm25Artifact>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            analytics: Option<crate::IndexAnalyticsArtifact>,
            maintenance: IndexMaintenanceEvidence,
        }

        fn decode_index_artifact_for_read(
            bytes: &[u8],
            row_limit: Option<usize>,
        ) -> Result<LoadedIndexArtifact> {
            let envelope: IndexArtifactEnvelope<'_> = serde_json::from_slice(bytes)?;
            if envelope.contract_version != INDEX_ARTIFACT_CONTRACT_VERSION {
                return Err(Error::Integrity(format!(
                    "unsupported index artifact contract version {}",
                    envelope.contract_version
                )));
            }
            if envelope.result_identity_version != RESULT_IDENTITY_VERSION {
                return Err(Error::Integrity(format!(
                    "unsupported index result identity version {}",
                    envelope.result_identity_version
                )));
            }
            envelope.definition.validate()?;
            if envelope.generation == 0
                || envelope.source_cursor == 0
                || envelope.read_cursor < envelope.source_cursor
            {
                return Err(Error::Integrity(
                    "index artifact generation/source cursor/read cursor are invalid".into(),
                ));
            }
            if envelope.maintenance.source_cursor != envelope.source_cursor
                || (envelope.maintenance.mode != "full_build"
                    && envelope.maintenance.mode != "incremental_reconciliation")
            {
                return Err(Error::Integrity(
                    "index maintenance evidence does not cover this artifact".into(),
                ));
            }
            match (
                &envelope.definition.kind,
                &envelope.bm25,
                &envelope.analytics,
            ) {
                (IndexKind::Scalar | IndexKind::Geo | IndexKind::MaterializedView, None, None) => {}
                (IndexKind::Count | IndexKind::AggregateCount, None, Some(analytics)) => {
                    validate_count_analytics(&envelope.definition, analytics)?;
                }
                (IndexKind::Bm25 { config }, Some(artifact), None) => {
                    artifact.validate()?;
                    if &artifact.config != config
                        || artifact.source_cursor != envelope.source_cursor
                        || artifact.schema_revision != envelope.schema_revision
                        || artifact.valid_at != envelope.valid_at
                    {
                        return Err(Error::Integrity(
                            "BM25 artifact coordinates differ from the containing query index"
                                .into(),
                        ));
                    }
                }
                _ => {
                    return Err(Error::Integrity(
                        "query index kind and specialized artifact disagree".into(),
                    ))
                }
            }
            let (rows, total_rows) = decode_index_rows(envelope.rows.get(), row_limit)?;
            if total_rows > 1_000_000 {
                return Err(Error::Integrity("index artifact row limit exceeded".into()));
            }
            Ok(LoadedIndexArtifact {
                scope: envelope.scope,
                definition: envelope.definition,
                generation: envelope.generation,
                source_cursor: envelope.source_cursor,
                read_cursor: envelope.read_cursor,
                schema_revision: envelope.schema_revision,
                valid_at: envelope.valid_at,
                rows,
                total_rows,
                bm25: envelope.bm25,
            })
        }

        fn decode_index_rows(
            rows_json: &str,
            row_limit: Option<usize>,
        ) -> Result<(Vec<QueryRow>, usize)> {
            let mut deserializer = serde_json::Deserializer::from_str(rows_json);
            let seed = RowCollector { row_limit };
            let (rows, total) = seed.deserialize(&mut deserializer)?;
            deserializer.end()?;
            Ok((rows, total))
        }

        struct RowCollector {
            row_limit: Option<usize>,
        }

        impl<'de> DeserializeSeed<'de> for RowCollector {
            type Value = (Vec<QueryRow>, usize);

            fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(RowCollectorVisitor {
                    row_limit: self.row_limit,
                })
            }
        }

        struct RowCollectorVisitor {
            row_limit: Option<usize>,
        }

        impl<'de> Visitor<'de> for RowCollectorVisitor {
            type Value = (Vec<QueryRow>, usize);

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an array of query rows")
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut rows = Vec::new();
                let mut total = 0usize;
                let mut previous_identity: Option<String> = None;
                while self.row_limit.is_none_or(|limit| rows.len() < limit) {
                    let Some(row) = seq.next_element::<QueryRow>()? else {
                        return Ok((rows, total));
                    };
                    total = total.saturating_add(1);
                    if previous_identity
                        .as_ref()
                        .is_some_and(|previous| previous >= &row.identity)
                    {
                        return Err(serde::de::Error::custom(
                            "index artifact row identities must be unique and sorted",
                        ));
                    }
                    previous_identity = Some(row.identity.clone());
                    rows.push(row);
                }
                while seq.next_element::<IgnoredAny>()?.is_some() {
                    total = total.saturating_add(1);
                }
                Ok((rows, total))
            }
        }

        fn validate_count_analytics(
            definition: &IndexDefinition,
            analytics: &crate::IndexAnalyticsArtifact,
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
        let selected = match self {
            Self::Versioned { sources } => {
                if matches!(
                    sources.as_slice(),
                    [RuntimeVersionedSource::Adjacency { .. }]
                ) {
                    let mut expanded = false;
                    engine.runtime().read_versioned_expanding(
                        stamp,
                        sources,
                        budget,
                        &mut |changes| {
                            if std::mem::replace(&mut expanded, true) {
                                return Vec::new();
                            }
                            // An adjacency entry names an identity, not its final
                            // temporal state. Later endpoint moves and retirements
                            // outside this adjacency must still govern visibility.
                            changes
                                .iter()
                                .filter_map(|change| match &change.mutation {
                                    RuntimeMutation::Relation { relation } => {
                                        Some(relation.reference.clone())
                                    }
                                    RuntimeMutation::Retire { retirement }
                                        if retirement.model
                                            == rrd_core::RuntimeLogicalModel::GraphRelation =>
                                    {
                                        Some(retirement.reference.clone())
                                    }
                                    _ => None,
                                })
                                .collect::<std::collections::BTreeSet<_>>()
                                .into_iter()
                                .map(|reference| RuntimeVersionedSource::Identity {
                                    model: rrd_core::RuntimeLogicalModel::GraphRelation,
                                    reference,
                                })
                                .collect()
                        },
                    )?
                } else {
                    engine.runtime().read_versioned(stamp, sources, budget)?
                }
            }
            Self::Adjacency {
                sources,
                traversals,
            } => {
                let mut expansion = AdjacencyExpansion::new(
                    &stamp.scope,
                    shape.valid_at,
                    shape.known_at_cursor,
                    sources,
                    traversals,
                );
                engine.runtime().read_versioned_expanding(
                    stamp,
                    sources,
                    budget,
                    &mut |changes| expansion.next(changes),
                )?
            }
            Self::Index { .. } | Self::CommitIndex { .. } => {
                unreachable!("indexes return above")
            }
        };
        let selected_versions = selected.changes.len();
        let rows = rows_for_source(
            &selected.changes,
            &stamp.scope,
            &shape.source,
            shape.valid_at,
            shape.known_at_cursor,
        );
        let right_rows = shape.join.as_ref().map(|join| {
            rows_for_source(
                &selected.changes,
                &stamp.scope,
                &join.source,
                shape.valid_at,
                shape.known_at_cursor,
            )
        });
        Ok(LoadedAccess {
            rows,
            right_rows,
            bm25: None,
            ranked: None,
            selected_versions,
            read_evidence: selected.evidence,
        })
    }
}

struct PlanShape {
    source: Source,
    join: Option<Join>,
    valid_at: u64,
    known_at_cursor: u64,
    filters: Vec<BoundFilter>,
    projection: Projection,
    limit: Option<usize>,
    schema_revision: u64,
    source_cursor: u64,
}

impl PlanShape {
    fn read(plan: &PhysicalPlan) -> Result<Self> {
        let mut source = None;
        let mut temporal = None;
        let mut join = None;
        let mut filters = Vec::new();
        let mut projection = None;
        let mut limit = None;
        for operator in &plan.logical.operators {
            match operator {
                LogicalOperator::Scan { source: value } => {
                    if source.replace(value.clone()).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one scan".into(),
                        ));
                    }
                }
                LogicalOperator::Join {
                    source,
                    left_field,
                    right_field,
                } => {
                    if join
                        .replace(Join {
                            source: source.clone(),
                            left_field: left_field.clone(),
                            right_field: right_field.clone(),
                        })
                        .is_some()
                    {
                        return Err(Error::Integrity(
                            "logical plan contains more than one join".into(),
                        ));
                    }
                }
                LogicalOperator::Filter { predicates } => filters.extend(predicates.clone()),
                LogicalOperator::Project { projection: value } => {
                    if projection.replace(value.clone()).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one projection".into(),
                        ));
                    }
                }
                LogicalOperator::Limit { rows } => {
                    if limit.replace(*rows).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one limit".into(),
                        ));
                    }
                }
                LogicalOperator::Temporal {
                    valid_at,
                    known_at_cursor,
                } => {
                    if temporal.replace((*valid_at, *known_at_cursor)).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one temporal selector".into(),
                        ));
                    }
                }
            }
        }
        let (valid_at, known_at_cursor) = temporal
            .ok_or_else(|| Error::Integrity("logical plan has no temporal selector".into()))?;
        Ok(Self {
            source: source.ok_or_else(|| Error::Integrity("logical plan has no scan".into()))?,
            join,
            valid_at,
            known_at_cursor,
            filters,
            projection: projection
                .ok_or_else(|| Error::Integrity("logical plan has no projection".into()))?,
            limit,
            schema_revision: plan.logical.schema_revision,
            source_cursor: plan.logical.source_cursor,
        })
    }
}

fn exact_join_rows(
    left: &[QueryRow],
    right: &[QueryRow],
    join: &Join,
    max_rows: usize,
) -> Result<Vec<QueryRow>> {
    let mut right_by_key = BTreeMap::<String, Vec<&QueryRow>>::new();
    for row in right {
        let Some(value) = row.values.get(&join.right_field) else {
            return Err(Error::Integrity(format!(
                "right join row {} lacks bound field {:?}",
                row.identity, join.right_field
            )));
        };
        if !matches!(value, RuntimeValue::Null) {
            right_by_key
                .entry(serde_json::to_string(value)?)
                .or_default()
                .push(row);
        }
    }

    let mut joined = Vec::new();
    for left_row in left {
        let Some(value) = left_row.values.get(&join.left_field) else {
            return Err(Error::Integrity(format!(
                "left join row {} lacks bound field {:?}",
                left_row.identity, join.left_field
            )));
        };
        if matches!(value, RuntimeValue::Null) {
            continue;
        }
        let key = serde_json::to_string(value)?;
        for right_row in right_by_key.get(&key).into_iter().flatten() {
            if joined.len() == max_rows {
                return Err(Error::Budget(format!(
                    "join produces more than the {max_rows}-row execution bound"
                )));
            }
            let values = left_row
                .values
                .iter()
                .map(|(field, value)| (format!("left.{field}"), value.clone()))
                .chain(
                    right_row
                        .values
                        .iter()
                        .map(|(field, value)| (format!("right.{field}"), value.clone())),
                )
                .collect();
            joined.push(QueryRow {
                identity: format!(
                    "join:{}:{}{}",
                    left_row.identity.len(),
                    left_row.identity,
                    right_row.identity
                ),
                values,
            });
        }
    }
    Ok(joined)
}

fn rows_for_source(
    changes: &[RuntimeChange],
    scope: &rrd_core::ScopeId,
    source: &Source,
    valid_at: u64,
    known_at_cursor: u64,
) -> Vec<QueryRow> {
    match source {
        Source::Record { kind } => active_graph_values(changes, scope, valid_at, known_at_cursor)
            .0
            .into_values()
            .filter(|record| &record.reference.kind == kind)
            .map(|record| {
                let identity =
                    result_identity::typed(ResultIdentityFamily::Record, &record.reference);
                let mut values = record.properties;
                values.insert("id".into(), string(record.reference.id.to_string()));
                values.insert("kind".into(), string(record.reference.kind.to_string()));
                values.insert(
                    "valid_from".into(),
                    RuntimeValue::Unsigned(record.valid_from),
                );
                values.insert("valid_to".into(), optional_u64(record.valid_to));
                QueryRow { identity, values }
            })
            .collect(),
        Source::Relation { kind } => active_graph_values(changes, scope, valid_at, known_at_cursor)
            .1
            .into_values()
            .filter(|relation| &relation.reference.kind == kind)
            .map(|relation| {
                let identity =
                    result_identity::typed(ResultIdentityFamily::Relation, &relation.reference);
                let mut values = relation.properties;
                values.insert("id".into(), string(relation.reference.id.to_string()));
                values.insert("kind".into(), string(relation.reference.kind.to_string()));
                values.insert("from_kind".into(), string(relation.from.kind.to_string()));
                values.insert("from_id".into(), string(relation.from.id.to_string()));
                values.insert("to_kind".into(), string(relation.to.kind.to_string()));
                values.insert("to_id".into(), string(relation.to.id.to_string()));
                values.insert(
                    "valid_from".into(),
                    RuntimeValue::Unsigned(relation.valid_from),
                );
                values.insert("valid_to".into(), optional_u64(relation.valid_to));
                QueryRow { identity, values }
            })
            .collect(),
        Source::Event { kind } => {
            let mut active = BTreeMap::new();
            for change in changes
                .iter()
                .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
            {
                match &change.mutation {
                    RuntimeMutation::Event { event }
                        if &event.kind == kind && change.at <= valid_at =>
                    {
                        let reference =
                            rrd_core::RuntimeRef::event(event.kind.clone(), change.cursor);
                        active.insert(reference, (change, event));
                    }
                    RuntimeMutation::Retire { retirement }
                        if retirement.model.is_event_like()
                            && &retirement.reference.kind == kind
                            && retirement.effective_at <= valid_at =>
                    {
                        active.remove(&retirement.reference);
                    }
                    _ => {}
                }
            }
            active
                .into_values()
                .map(|(change, event)| {
                    let mut values = event.properties.clone();
                    values.insert("cursor".into(), RuntimeValue::Unsigned(change.cursor));
                    values.insert("kind".into(), string(event.kind.to_string()));
                    values.insert("at".into(), RuntimeValue::Unsigned(change.at));
                    values.insert("actor".into(), string(change.actor.clone()));
                    values.insert(
                        "subject_kind".into(),
                        event
                            .subject
                            .as_ref()
                            .map_or(RuntimeValue::Null, |subject| {
                                string(subject.kind.to_string())
                            }),
                    );
                    values.insert(
                        "subject_id".into(),
                        event
                            .subject
                            .as_ref()
                            .map_or(RuntimeValue::Null, |subject| string(subject.id.to_string())),
                    );
                    QueryRow {
                        identity: result_identity::encode(
                            ResultIdentityFamily::Event,
                            &[event.kind.as_str(), &change.cursor.to_string()],
                        ),
                        values,
                    }
                })
                .collect()
        }
        Source::Series { kind } => {
            let mut active = BTreeMap::new();
            for change in changes
                .iter()
                .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
            {
                match &change.mutation {
                    RuntimeMutation::SeriesSample { sample }
                        if &sample.series.kind == kind && sample.observed_at <= valid_at =>
                    {
                        active.insert(sample.reference.clone(), sample);
                    }
                    RuntimeMutation::Retire { retirement }
                        if retirement.model == rrd_core::RuntimeLogicalModel::TimeSeries
                            && retirement.effective_at <= valid_at =>
                    {
                        active.remove(&retirement.reference);
                    }
                    _ => {}
                }
            }
            active
                .into_values()
                .map(|sample| {
                    let mut values = sample.properties.clone();
                    values.insert("id".into(), string(sample.reference.id.to_string()));
                    values.insert("kind".into(), string(sample.reference.kind.to_string()));
                    values.insert("series_kind".into(), string(sample.series.kind.to_string()));
                    values.insert("series_id".into(), string(sample.series.id.to_string()));
                    values.insert(
                        "observed_at".into(),
                        RuntimeValue::Unsigned(sample.observed_at),
                    );
                    values.insert("value".into(), series_value(&sample.value));
                    QueryRow {
                        identity: result_identity::typed(
                            ResultIdentityFamily::Series,
                            &sample.reference,
                        ),
                        values,
                    }
                })
                .collect()
        }
        Source::Geo { kind } => geo_rows(changes, scope, kind, valid_at, known_at_cursor),
        Source::Traversal {
            relation,
            start,
            direction,
            max_depth,
        } => crate::physical::graph::traversal_rows(
            changes,
            scope,
            relation,
            start,
            *direction,
            *max_depth,
            valid_at,
            known_at_cursor,
        ),
        Source::Claim { predicate } => claim_rows(
            changes,
            scope,
            predicate.as_ref(),
            valid_at,
            known_at_cursor,
        ),
    }
}

pub(crate) fn active_graph_values<'a>(
    changes: impl IntoIterator<Item = &'a RuntimeChange>,
    scope: &rrd_core::ScopeId,
    valid_at: u64,
    known_at_cursor: u64,
) -> (
    BTreeMap<rrd_core::RuntimeRef, RuntimeRecord>,
    BTreeMap<rrd_core::RuntimeRef, RuntimeRelation>,
) {
    let mut state =
        crate::physical::graph::state::GraphState::new(scope, valid_at, known_at_cursor);
    for change in changes {
        state.apply(change);
    }
    state.into_values()
}

fn geo_rows(
    changes: &[RuntimeChange],
    scope: &rrd_core::ScopeId,
    kind: &rrd_core::RuntimeType,
    valid_at: u64,
    known_at_cursor: u64,
) -> Vec<QueryRow> {
    let mut active = BTreeMap::<rrd_core::RuntimeRef, (u64, RuntimeGeo)>::new();
    for change in changes
        .iter()
        .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
    {
        match &change.mutation {
            RuntimeMutation::Geo { geo }
                if &geo.reference.kind == kind
                    && geo.valid_from <= valid_at
                    && geo.valid_to.is_none_or(|end| valid_at < end) =>
            {
                let replace = active.get(&geo.reference).is_none_or(|(cursor, current)| {
                    geo.valid_from > current.valid_from
                        || (geo.valid_from == current.valid_from && change.cursor > *cursor)
                });
                if replace {
                    active.insert(geo.reference.clone(), (change.cursor, geo.clone()));
                }
            }
            RuntimeMutation::Retire { retirement }
                if retirement.model == rrd_core::RuntimeLogicalModel::Geo
                    && &retirement.reference.kind == kind
                    && retirement.effective_at <= valid_at =>
            {
                active.remove(&retirement.reference);
            }
            _ => {}
        }
    }
    active
        .into_values()
        .map(|(_, geo)| {
            let mut values = geo.properties;
            values.insert("id".into(), string(geo.reference.id.to_string()));
            values.insert("kind".into(), string(geo.reference.kind.to_string()));
            values.insert("subject_kind".into(), string(geo.subject.kind.to_string()));
            values.insert("subject_id".into(), string(geo.subject.id.to_string()));
            values.insert("field".into(), string(geo.field));
            values.insert("valid_from".into(), RuntimeValue::Unsigned(geo.valid_from));
            values.insert("valid_to".into(), optional_u64(geo.valid_to));
            insert_geo_value(&mut values, &geo.value);
            QueryRow {
                identity: result_identity::typed(ResultIdentityFamily::Geo, &geo.reference),
                values,
            }
        })
        .collect()
}

fn series_value(value: &SeriesValue) -> RuntimeValue {
    match value {
        SeriesValue::Integer(value) => RuntimeValue::Integer(*value),
        SeriesValue::Unsigned(value) => RuntimeValue::Unsigned(*value),
        SeriesValue::Decimal(value) => RuntimeValue::Decimal(value.clone()),
        SeriesValue::Bool(value) => RuntimeValue::Bool(*value),
        SeriesValue::String(value) => RuntimeValue::String(value.clone()),
    }
}

fn insert_geo_value(values: &mut BTreeMap<String, RuntimeValue>, geo: &GeoValue) {
    for field in [
        "longitude",
        "latitude",
        "southwest_longitude",
        "southwest_latitude",
        "northeast_longitude",
        "northeast_latitude",
    ] {
        values.insert(field.into(), RuntimeValue::Null);
    }
    match geo {
        GeoValue::Point { point } => {
            values.insert("geometry_kind".into(), string("point".into()));
            values.insert(
                "longitude".into(),
                RuntimeValue::Decimal(point.longitude.to_string()),
            );
            values.insert(
                "latitude".into(),
                RuntimeValue::Decimal(point.latitude.to_string()),
            );
        }
        GeoValue::BoundingBox {
            southwest,
            northeast,
        } => {
            values.insert("geometry_kind".into(), string("bounding_box".into()));
            values.insert(
                "southwest_longitude".into(),
                RuntimeValue::Decimal(southwest.longitude.to_string()),
            );
            values.insert(
                "southwest_latitude".into(),
                RuntimeValue::Decimal(southwest.latitude.to_string()),
            );
            values.insert(
                "northeast_longitude".into(),
                RuntimeValue::Decimal(northeast.longitude.to_string()),
            );
            values.insert(
                "northeast_latitude".into(),
                RuntimeValue::Decimal(northeast.latitude.to_string()),
            );
        }
    }
}

fn claim_rows(
    changes: &[RuntimeChange],
    scope: &rrd_core::ScopeId,
    predicate: Option<&rrd_core::Predicate>,
    valid_at: u64,
    known_at_cursor: u64,
) -> Vec<QueryRow> {
    let mut groups = BTreeMap::<(String, String), Vec<(u64, Claim)>>::new();
    for change in changes
        .iter()
        .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
    {
        let RuntimeMutation::Claim { claim } = &change.mutation else {
            continue;
        };
        if predicate.is_some_and(|expected| &claim.predicate != expected) {
            continue;
        }
        groups
            .entry((claim.subject.to_string(), claim.predicate.to_string()))
            .or_default()
            .push((change.cursor, claim.clone()));
    }
    groups
        .into_iter()
        .filter_map(|((subject, predicate), mut versions)| {
            versions.sort_by_key(|version| std::cmp::Reverse(version.0));
            let candidates = versions
                .into_iter()
                .map(|(_, claim)| claim)
                .collect::<Vec<_>>();
            let claim = resolve_as_of(&candidates, valid_at)?;
            let mut values = BTreeMap::new();
            values.insert("subject".into(), string(subject.clone()));
            values.insert("predicate".into(), string(predicate.clone()));
            values.insert("object".into(), string(claim.object.clone()));
            values.insert(
                "valid_from".into(),
                RuntimeValue::Unsigned(claim.valid_from),
            );
            values.insert("valid_to".into(), optional_u64(claim.valid_to));
            values.insert("tx_time".into(), RuntimeValue::Unsigned(claim.tx_time));
            values.insert("actor".into(), string(claim.producer.actor.clone()));
            Some(QueryRow {
                identity: result_identity::claim(&claim.subject, &claim.predicate),
                values,
            })
        })
        .collect()
}

fn bm25_scores(
    rows: &[QueryRow],
    shape: &PlanShape,
    artifact: Option<Bm25Artifact>,
) -> Result<BTreeMap<String, Bm25Evidence>> {
    let mut matches = shape
        .filters
        .iter()
        .filter(|filter| filter.comparison.is_match());
    let Some(filter) = matches.next() else {
        if artifact.is_some() {
            return Err(Error::Integrity(
                "a BM25 artifact was selected for a query without MATCH".into(),
            ));
        }
        return Ok(BTreeMap::new());
    };
    if matches.next().is_some() {
        return Err(Error::Binding(
            "BM25 v1 supports exactly one MATCH predicate".into(),
        ));
    }
    let RuntimeValue::String(query) = &filter.value else {
        return Err(Error::Binding("MATCH requires a string query".into()));
    };
    if rows.is_empty() {
        return Ok(BTreeMap::new());
    }
    let artifact = match artifact {
        Some(artifact) => {
            if artifact.source_cursor != shape.source_cursor || artifact.valid_at != shape.valid_at
            {
                return Err(Error::Integrity(
                    "selected BM25 artifact is not fresh for the bound query".into(),
                ));
            }
            artifact
        }
        None => {
            let documents = rows
                .iter()
                .filter_map(|row| match row.values.get(&filter.field) {
                    Some(RuntimeValue::String(text)) => {
                        Some(Ok((row.identity.clone(), text.clone())))
                    }
                    Some(RuntimeValue::Null) | None => None,
                    Some(_) => Some(Err(Error::Binding(format!(
                        "MATCH field {:?} produced a non-string value",
                        filter.field
                    )))),
                })
                .collect::<Result<Vec<_>>>()?;
            if documents.is_empty() {
                return Ok(BTreeMap::new());
            }
            Bm25Artifact::build(
                Bm25Config::default(),
                shape.source_cursor,
                shape.schema_revision,
                shape.valid_at,
                documents,
            )?
        }
    };
    let text_by_identity = rows
        .iter()
        .filter_map(|row| match row.values.get(&filter.field) {
            Some(RuntimeValue::String(text)) => Some((row.identity.as_str(), text.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    artifact
        .search(query, rows.len().max(1))?
        .into_iter()
        .map(|hit| {
            let text = text_by_identity.get(hit.identity.as_str()).ok_or_else(|| {
                Error::Integrity(format!(
                    "BM25 hit {} has no authoritative source text",
                    hit.identity
                ))
            })?;
            let evidence = Bm25Evidence {
                score: hit.score,
                matched_terms: hit.matched_terms,
                highlight: highlight_offsets(text, &hit.offsets, "<em>", "</em>")?,
                offsets: hit.offsets,
            };
            Ok((hit.identity, evidence))
        })
        .collect()
}

/// Evidence for rows a commit-bound BM25 index ranked: its scores and
/// matched terms, with highlights recomputed from each row's own text.
fn ranked_scores(
    rows: &[QueryRow],
    shape: &PlanShape,
    ranked: Ranked,
) -> Result<BTreeMap<String, Bm25Evidence>> {
    let Some(filter) = shape
        .filters
        .iter()
        .find(|filter| filter.comparison.is_match())
    else {
        return Err(Error::Integrity(
            "a commit-bound BM25 ranking was loaded for a query without MATCH".into(),
        ));
    };
    rows.iter()
        .filter_map(|row| {
            let (score, matched_terms) = ranked.hits.get(&row.identity)?;
            let Some(RuntimeValue::String(text)) = row.values.get(&filter.field) else {
                return Some(Err(Error::Integrity(format!(
                    "BM25 hit {} has no authoritative source text",
                    row.identity
                ))));
            };
            Some((|| {
                let offsets = crate::bm25::analyze_with_offsets(&ranked.config, text)?
                    .into_iter()
                    .filter(|token| matched_terms.contains(&token.term))
                    .map(|token| token.offset)
                    .collect::<Vec<_>>();
                Ok((
                    row.identity.clone(),
                    Bm25Evidence {
                        score: *score,
                        matched_terms: matched_terms.clone(),
                        highlight: highlight_offsets(text, &offsets, "<em>", "</em>")?,
                        offsets,
                    },
                ))
            })())
        })
        .collect()
}

#[derive(Debug)]
struct Bm25Evidence {
    score: f64,
    matched_terms: Vec<String>,
    offsets: Vec<Bm25Offset>,
    highlight: String,
}

fn public_bm25_offset(offset: &Bm25Offset) -> RuntimeValue {
    RuntimeValue::Map(BTreeMap::from([
        (
            "token_ordinal".into(),
            RuntimeValue::Unsigned(u64::from(offset.token_ordinal)),
        ),
        (
            "byte_start".into(),
            RuntimeValue::Unsigned(u64::from(offset.byte_start)),
        ),
        (
            "byte_end".into(),
            RuntimeValue::Unsigned(u64::from(offset.byte_end)),
        ),
    ]))
}

fn apply_projection(row: &mut QueryRow, projection: &Projection) {
    if let Projection::Fields(fields) = projection {
        row.values.retain(|field, _| fields.contains(field));
    }
}

pub(crate) fn string(value: String) -> RuntimeValue {
    RuntimeValue::String(value)
}

fn optional_u64(value: Option<u64>) -> RuntimeValue {
    value.map_or(RuntimeValue::Null, RuntimeValue::Unsigned)
}
