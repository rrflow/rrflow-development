//! The commit-time BM25 stage over persistent postings.

use super::*;

/// What the commit-maintained BM25 indexes answered for a context request.
#[derive(Default)]
pub(super) struct CommitLexical {
    /// Record kinds a BM25 index ranked.
    pub(super) covered: BTreeSet<RuntimeType>,
    /// Record kinds the snapshot reads only by identity: those ranked from
    /// postings, and those whose only indexes are scalar lookups.
    pub(super) excluded: BTreeSet<RuntimeType>,
    /// The hits' identities, read at the request's stamp.
    pub(super) identities: Vec<rrd_store::RuntimeVersionedSource>,
    pub(super) hits: Vec<(rrd_core::ProjectionId, Vec<rrd_store::LexicalHit>)>,
    pub(super) keys_read: usize,
}

impl RrdEngine {
    /// Ranks every record kind that has a commit-bound BM25 index over a text
    /// field from its postings at `stamp`. A kind whose index cannot answer
    /// for this stamp or valid time is left to the snapshot.
    pub(super) fn commit_lexical(
        &self,
        request: &AssembleContext,
        stamp: &ReadStamp,
        key_budget: usize,
    ) -> Result<CommitLexical> {
        let mut lexical = CommitLexical::default();
        if request.query.trim().is_empty() {
            return Ok(lexical);
        }
        let catalogue =
            rrd_query::IndexCatalogueRepository::new(&self.storage, stamp.scope.clone())
                .load()
                .map_err(|error| ServiceError::Query(error.to_string()))?;
        let schema = self.storage.runtime().schema(&stamp.scope)?;
        let top_k = usize::try_from(request.max_items)
            .map_err(|_| ServiceError::Query("context item bound exceeds usize".into()))?
            .saturating_add(1);
        for entry in catalogue.entries.values() {
            let (rrd_query::IndexKind::Bm25 { .. }, rrd_query::Source::Record { kind }) =
                (&entry.definition.kind, &entry.definition.source)
            else {
                continue;
            };
            if lexical.covered.contains(kind) {
                continue;
            }
            let Some(model) = schema
                .as_ref()
                .and_then(|schema| schema.logical_model(kind).ok())
            else {
                continue;
            };
            let remaining = key_budget.saturating_sub(lexical.keys_read) as u64;
            match self.storage.runtime().lexical_search(
                stamp,
                rrd_store::LexicalQuery {
                    index: &entry.definition.id,
                    text: &request.query,
                    valid_at: request.valid_at,
                    top_k,
                    max_keys: remaining,
                },
            )? {
                rrd_store::LexicalSearch::Hits {
                    hits, keys_read, ..
                } => {
                    lexical.keys_read = lexical
                        .keys_read
                        .saturating_add(usize::try_from(keys_read).unwrap_or(usize::MAX));
                    lexical.covered.insert(kind.clone());
                    lexical.identities.extend(hits.iter().map(|hit| {
                        rrd_store::RuntimeVersionedSource::Identity {
                            model,
                            reference: hit.record.clone(),
                        }
                    }));
                    lexical.hits.push((entry.definition.id.clone(), hits));
                }
                rrd_store::LexicalSearch::NotApplicable(_) => {}
            }
        }
        // A kind reached only through scalar lookups is read by identity on
        // demand, not scanned; a kind whose BM25 index could not answer is
        // read and ranked from the snapshot.
        let lexical_kinds = catalogue
            .entries
            .values()
            .filter(|entry| matches!(entry.definition.kind, rrd_query::IndexKind::Bm25 { .. }))
            .filter_map(|entry| match &entry.definition.source {
                rrd_query::Source::Record { kind } => Some(kind.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        lexical.excluded = catalogue
            .entries
            .values()
            .filter(|entry| matches!(entry.definition.kind, rrd_query::IndexKind::Scalar))
            .filter_map(|entry| match &entry.definition.source {
                rrd_query::Source::Record { kind } if !lexical_kinds.contains(kind) => {
                    Some(kind.clone())
                }
                _ => None,
            })
            .chain(lexical.covered.iter().cloned())
            .collect();
        Ok(lexical)
    }
}
