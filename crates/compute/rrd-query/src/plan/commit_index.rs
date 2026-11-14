//! Selecting a commit-time index for a query.

use super::*;

/// The commit-bound index a query can narrow its read with: a BM25 index when
/// its only predicate is a MATCH on the index's field, otherwise the scalar
/// index whose leading fields the query binds by equality the furthest.
pub(super) fn select_commit_index<'a>(
    commit_indexes: &'a [BoundCommitIndex],
    filters: &[BoundFilter],
    limit: Option<usize>,
    match_field: Option<&str>,
) -> Option<(&'a BoundCommitIndex, CommitIndexProbe)> {
    if let Some(field) = match_field {
        let [filter] = filters else {
            return None;
        };
        let RuntimeValue::String(query) = &filter.value else {
            return None;
        };
        return commit_indexes.iter().find_map(|index| match &index.kind {
            IndexKind::Bm25 { config } if index.fields.as_slice() == [field] => Some((
                index,
                CommitIndexProbe::Bm25 {
                    query: query.clone(),
                    config: config.clone(),
                    top_k: limit
                        .unwrap_or(MAX_COMMIT_INDEX_TOP_K)
                        .min(MAX_COMMIT_INDEX_TOP_K),
                },
            )),
            _ => None,
        });
    }
    commit_indexes
        .iter()
        .filter(|index| matches!(index.kind, IndexKind::Scalar))
        .filter_map(|index| {
            let values = index
                .fields
                .iter()
                .map_while(|field| {
                    filters.iter().find_map(|filter| {
                        (filter.field == *field && filter.comparison == ComparisonOperator::Equal)
                            .then(|| filter.value.clone())
                    })
                })
                .collect::<Vec<_>>();
            (!values.is_empty()).then_some((index, values))
        })
        .max_by(|(left, left_values), (right, right_values)| {
            left_values
                .len()
                .cmp(&right_values.len())
                .then_with(|| right.id.cmp(&left.id))
        })
        .map(|(index, values)| (index, CommitIndexProbe::Scalar { values }))
}

/// Documents a commit-bound BM25 read ranks at most.
pub const MAX_COMMIT_INDEX_TOP_K: usize = 10_000;
