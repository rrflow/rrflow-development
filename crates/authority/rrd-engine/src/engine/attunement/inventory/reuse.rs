//! Reuse only equal current authenticated facts, never a prior-head assumption.
use super::*;
use rrd_store::{RuntimeReadBudget, RuntimeVersionedSource};
use std::collections::BTreeMap;

pub(super) fn filter(
    engine: &RrdEngine,
    read: &ReadStamp,
    mutations: Vec<RuntimeMutation>,
    remaining_keys: &mut u64,
    check: &dyn Fn() -> Result<()>,
) -> Result<(Vec<RuntimeMutation>, String)> {
    let mut parts = records::partition(mutations)?;
    // An empty runtime has no facts to reuse; an uninitialized accumulator
    // cannot authenticate reuse. Neither needs identity-range lookups.
    if read.commit_cursor != 0 && read.accumulator_root.is_some() && !parts.groups.is_empty() {
        let mut admitted = BTreeMap::new();
        for group in &parts.groups {
            check()?;
            for mutation in group {
                let (reference, model) = identity(mutation)?;
                if admitted.insert(reference.clone(), model).is_some() {
                    return Err(invalid());
                }
            }
        }
        let sources = admitted
            .iter()
            .map(|(reference, model)| RuntimeVersionedSource::Identity {
                model: *model,
                reference: reference.clone(),
            })
            .collect::<Vec<_>>();
        check()?;
        let current = engine.storage.runtime().read_versioned(
            read,
            &sources,
            RuntimeReadBudget::new(*remaining_keys)?,
        )?;
        *remaining_keys = remaining_keys
            .checked_sub(current.evidence.keys_examined)
            .ok_or_else(invalid)?;
        check()?;
        let mut latest = BTreeMap::<RuntimeRef, (u64, RuntimeMutation)>::new();
        for change in current.changes {
            let (reference, model) = identity(&change.mutation)?;
            if admitted.get(reference) != Some(&model) {
                return Err(invalid());
            }
            if latest
                .get(reference)
                .is_none_or(|(cursor, _)| change.cursor > *cursor)
            {
                latest.insert(reference.clone(), (change.cursor, change.mutation));
            }
        }
        let mut changed = Vec::new();
        for group in parts.groups {
            check()?;
            let equal = group.iter().all(|desired| {
                identity(desired).is_ok_and(|(reference, _)| {
                    latest
                        .get(reference)
                        .is_some_and(|(_, existing)| same_facts(existing, desired))
                })
            });
            // An entry and its containment are one publication group.
            if !equal {
                changed.push(group);
            }
        }
        parts.groups = changed;
    }
    check()?;
    let publication_sha256 = work_set_digest(&parts.groups)?;
    Ok((
        parts
            .schema
            .into_iter()
            .chain(parts.groups.into_iter().flatten())
            .chain(parts.complete)
            .collect(),
        publication_sha256,
    ))
}

fn identity(mutation: &RuntimeMutation) -> Result<(&RuntimeRef, RuntimeLogicalModel)> {
    match mutation {
        RuntimeMutation::Record { record } => {
            Ok((&record.reference, RuntimeLogicalModel::GraphNode))
        }
        RuntimeMutation::Relation { relation } => {
            Ok((&relation.reference, RuntimeLogicalModel::GraphRelation))
        }
        RuntimeMutation::Retire { retirement } => Ok((&retirement.reference, retirement.model)),
        _ => Err(invalid()),
    }
}

fn same_facts(existing: &RuntimeMutation, desired: &RuntimeMutation) -> bool {
    match (existing, desired) {
        (RuntimeMutation::Record { record: old }, RuntimeMutation::Record { record: new }) => {
            old.reference == new.reference
                && old.properties == new.properties
                && old.valid_to == new.valid_to
                && old.valid_from <= new.valid_from
        }
        (
            RuntimeMutation::Relation { relation: old },
            RuntimeMutation::Relation { relation: new },
        ) => {
            old.reference == new.reference
                && old.from == new.from
                && old.to == new.to
                && old.properties == new.properties
                && old.valid_to == new.valid_to
                && old.valid_from <= new.valid_from
        }
        // A retirement, missing row or different value cannot authorize reuse.
        _ => false,
    }
}

fn work_set_digest(groups: &[Vec<RuntimeMutation>]) -> Result<String> {
    let mut digest = digest::Sha256::new();
    digest.update(b"rrflow-inventory-publication-work-v1");
    for group in groups {
        let mut group_digest = digest::Sha256::new();
        for mutation in group {
            // Invocation timestamps may change on resume; desired facts and
            // ordered group boundaries must not change under a reused offset.
            let bytes = match mutation {
                RuntimeMutation::Record { record } => serde_json::to_vec(&(
                    "record",
                    &record.reference,
                    record.valid_to,
                    &record.properties,
                )),
                RuntimeMutation::Relation { relation } => serde_json::to_vec(&(
                    "relation",
                    &relation.reference,
                    &relation.from,
                    &relation.to,
                    relation.valid_to,
                    &relation.properties,
                )),
                _ => return Err(invalid()),
            }
            .map_err(contract)?;
            group_digest.update(&(bytes.len() as u64).to_be_bytes());
            group_digest.update(&bytes);
        }
        digest.update(&group_digest.finalize());
    }
    Ok(digest.finalize_hex())
}

fn invalid() -> ServiceError {
    ServiceError::Storage("invalid Inventory reuse observation".into())
}
