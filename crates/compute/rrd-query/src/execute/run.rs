//! Executing a physical plan and the read paths it can take.

use super::*;

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

pub(super) enum ReadPath {
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

pub(super) struct LoadedAccess {
    pub(super) rows: Vec<QueryRow>,
    pub(super) right_rows: Option<Vec<QueryRow>>,
    pub(super) bm25: Option<Bm25Artifact>,
    /// Scores a commit-bound BM25 index computed over every current document.
    pub(super) ranked: Option<Ranked>,
    pub(super) selected_versions: usize,
    pub(super) read_evidence: RuntimeReadEvidence,
}

/// A commit-bound BM25 ranking: scores and matched terms by row identity, and
/// the configuration that analysed them.
pub(super) struct Ranked {
    pub(super) config: Bm25Config,
    pub(super) hits: BTreeMap<String, (f64, Vec<String>)>,
}
