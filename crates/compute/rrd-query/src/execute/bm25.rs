//! BM25 scoring, ranked scores, evidence offsets and projection helpers.

use super::*;

pub(super) fn bm25_scores(
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
pub(super) fn ranked_scores(
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
pub(super) struct Bm25Evidence {
    pub(super) score: f64,
    pub(super) matched_terms: Vec<String>,
    pub(super) offsets: Vec<Bm25Offset>,
    pub(super) highlight: String,
}

pub(super) fn public_bm25_offset(offset: &Bm25Offset) -> RuntimeValue {
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

pub(super) fn apply_projection(row: &mut QueryRow, projection: &Projection) {
    if let Projection::Fields(fields) = projection {
        row.values.retain(|field, _| fields.contains(field));
    }
}

pub(crate) fn string(value: String) -> RuntimeValue {
    RuntimeValue::String(value)
}

pub(super) fn optional_u64(value: Option<u64>) -> RuntimeValue {
    value.map_or(RuntimeValue::Null, RuntimeValue::Unsigned)
}
