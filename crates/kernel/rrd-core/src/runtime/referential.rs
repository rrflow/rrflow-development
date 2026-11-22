//! Relation endpoint integrity at one knowledge coordinate, over valid time.
//!
//! Callers supply complete, authenticated histories for affected relations and
//! their endpoints. The kernel owns the verdict; storage owns range selection.
use super::*;
use crate::{RuntimeLogicalModel, RuntimeReferentialAction};
mod timeline;
use timeline::{first_gap, Timeline};

/// Resolve schema-authorized edge retirements. An empty result proves the
/// supplied relation histories remain endpoint-covered after this commit.
/// A nonempty result must be folded into the commit before publication; it is
/// never an instruction to write a second transaction or erase old versions.
pub fn relation_referential_actions(
    schema: &RuntimeSchemaRegistry,
    history: &[RuntimeChange],
    commit: &RuntimeCommit,
) -> Result<Vec<RuntimeRetirement>> {
    let mut records = BTreeMap::<RuntimeRef, Timeline<()>>::new();
    let mut relations = BTreeMap::<RuntimeRef, Timeline<RuntimeRelation>>::new();
    let mut deletions = BTreeMap::<RuntimeRef, u64>::new();
    for change in history
        .iter()
        .filter(|change| change.scope == commit.scope && change.cursor <= commit.expected_cursor)
    {
        add_version(
            &mut records,
            &mut relations,
            &change.mutation,
            change.cursor,
        );
    }
    for (index, mutation) in commit.mutations.iter().enumerate() {
        let cursor = commit
            .expected_cursor
            .checked_add(index as u64)
            .and_then(|value| value.checked_add(1))
            .ok_or(Error::SequenceOverflow)?;
        add_version(&mut records, &mut relations, mutation, cursor);
        if let RuntimeMutation::Retire { retirement } = mutation {
            if retirement.model.is_record_like() {
                deletions.insert(retirement.reference.clone(), retirement.effective_at);
            }
        }
    }
    let records = records
        .into_iter()
        .map(|(id, timeline)| (id, timeline.finish()))
        .collect::<BTreeMap<_, _>>();
    let mut retirements = Vec::new();
    for (reference, timeline) in relations {
        for part in timeline.finish() {
            let relation = &part.value;
            let rule = schema.relations.get(&reference.kind);
            let endpoints = [
                (&relation.from, rule.and_then(|rule| rule.on_delete_from)),
                (&relation.to, rule.and_then(|rule| rule.on_delete_to)),
            ];
            let gaps = endpoints
                .iter()
                .filter_map(|(endpoint, action)| {
                    let intervals = records.get(*endpoint).map(Vec::as_slice).unwrap_or(&[]);
                    first_gap(intervals, part.from, part.to).map(|at| {
                        let cascade = *action == Some(RuntimeReferentialAction::Cascade)
                            && deletions
                                .get(*endpoint)
                                .is_some_and(|retired| *retired <= at);
                        (at, *endpoint, cascade)
                    })
                })
                .collect::<Vec<_>>();
            let Some(first) = gaps.iter().map(|(at, _, _)| *at).min() else {
                continue;
            };
            if let Some((_, endpoint, _)) = gaps
                .iter()
                .find(|(at, _, cascade)| *at == first && !cascade)
            {
                return Err(Error::InvalidRuntime { reason: format!(
                    "relation {}/{} has unavailable endpoint {}/{} at valid time {first}; endpoint deletion requires CASCADE or an explicit atomic relation retirement",
                    reference.kind, reference.id, endpoint.kind, endpoint.id
                ) });
            }
            retirements.push(RuntimeRetirement {
                model: RuntimeLogicalModel::GraphRelation,
                reference: reference.clone(),
                effective_at: first,
            });
            // The new retirement masks all later versions already known for
            // this edge. Later commits can explicitly create it again.
            break;
        }
    }
    Ok(retirements)
}

fn add_version(
    records: &mut BTreeMap<RuntimeRef, Timeline<()>>,
    relations: &mut BTreeMap<RuntimeRef, Timeline<RuntimeRelation>>,
    mutation: &RuntimeMutation,
    cursor: u64,
) {
    match mutation {
        RuntimeMutation::Record { record } => records
            .entry(record.reference.clone())
            .or_default()
            .put(record.valid_from, record.valid_to, cursor, ()),
        RuntimeMutation::Relation { relation } => relations
            .entry(relation.reference.clone())
            .or_default()
            .put(
                relation.valid_from,
                relation.valid_to,
                cursor,
                relation.clone(),
            ),
        RuntimeMutation::Retire { retirement } if retirement.model.is_record_like() => {
            records
                .entry(retirement.reference.clone())
                .or_default()
                .retire(retirement.effective_at, cursor);
        }
        RuntimeMutation::Retire { retirement }
            if retirement.model == RuntimeLogicalModel::GraphRelation =>
        {
            relations
                .entry(retirement.reference.clone())
                .or_default()
                .retire(retirement.effective_at, cursor);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
