//! Validate retirement targets against history and preceding mutations in the same batch.
use super::*;

pub(super) fn validate_retirement_targets(
    reader: &(impl AccessRead + ?Sized),
    commit: &RuntimeCommit,
    head: u64,
) -> Result<BTreeMap<usize, RuntimeRelation>> {
    let retirements = commit
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Retire { retirement } => Some(retirement),
            _ => None,
        })
        .collect::<Vec<_>>();
    if retirements.is_empty() {
        return Ok(BTreeMap::new());
    }
    let read = read_stamp_with(reader, &commit.scope)?;
    if read.commit_cursor != head {
        return Err(Error::ReadStampUnavailable(format!(
            "retirement validation for {} moved from cursor {head} to {}",
            commit.scope, read.commit_cursor
        )));
    }
    let mut sources = vec![RuntimeVersionedSource::Schema];
    sources.extend(
        retirements
            .iter()
            .map(|retirement| RuntimeVersionedSource::Identity {
                model: retirement.model,
                reference: retirement.reference.clone(),
            }),
    );
    let direct = read_versioned(
        reader,
        &read,
        &sources,
        RuntimeReadBudget::authenticated_history_integrity(head)?,
    )?;
    let schema = match commit.mutations.iter().find_map(|mutation| match mutation {
        RuntimeMutation::Schema { registry } => Some(registry.clone()),
        _ => None,
    }) {
        Some(schema) => schema,
        None => schema_at_read(&read, &direct.changes)?,
    };
    // A retirement may target an identity created earlier in this same atomic
    // batch. Evaluate the preceding proposed mutations in their commit order;
    // a later put cannot retroactively authorize deletion of a missing target.
    let mut retired_relations = BTreeMap::new();
    let mut histories = BTreeMap::<RuntimeRef, Vec<RuntimeChange>>::new();
    for change in direct.changes {
        if let Some(reference) = identity(&change) {
            histories.entry(reference).or_default().push(change);
        }
    }
    let commit_id = commit.digest();
    for (ordinal, mutation) in commit.mutations.iter().enumerate() {
        let cursor = head
            .checked_add(ordinal as u64)
            .ok_or(Error::SequenceOverflow)?;
        if let RuntimeMutation::Retire { retirement } = mutation {
            let snapshot = RuntimeDataSnapshot::from_changes(
                histories
                    .get(&retirement.reference)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                &schema,
                commit.scope.clone(),
                retirement.effective_at,
                cursor,
            )?;
            if !snapshot.contains(retirement.model, &retirement.reference) {
                return Err(Error::RuntimeTargetNotFound(format!(
                    "{:?} {}/{} at {} in scope {}",
                    retirement.model,
                    retirement.reference.kind,
                    retirement.reference.id,
                    retirement.effective_at,
                    commit.scope
                )));
            }
            if retirement.model == rrd_core::RuntimeLogicalModel::GraphRelation {
                let relation = snapshot
                    .relations
                    .into_iter()
                    .find(|value| value.value.reference == retirement.reference)
                    .expect("target checked");
                retired_relations.insert(ordinal, relation.value);
            }
        }
        let change = RuntimeChange::committed(
            cursor.checked_add(1).ok_or(Error::SequenceOverflow)?,
            commit,
            &commit_id,
            ordinal as u64,
            mutation.clone(),
            None,
        );
        if let Some(reference) = identity(&change) {
            histories.entry(reference).or_default().push(change);
        }
    }
    Ok(retired_relations)
}

fn identity(change: &RuntimeChange) -> Option<RuntimeRef> {
    Some(match &change.mutation {
        RuntimeMutation::Record { record } => record.reference.clone(),
        RuntimeMutation::Relation { relation } => relation.reference.clone(),
        RuntimeMutation::Event { event } => {
            keyspaces::runtime_event_reference(event, change.cursor)
        }
        RuntimeMutation::Vector { vector } => vector.reference.clone(),
        RuntimeMutation::SeriesSample { sample } => sample.reference.clone(),
        RuntimeMutation::Geo { geo } => geo.reference.clone(),
        RuntimeMutation::Object { object } => object.reference.clone(),
        RuntimeMutation::Retire { retirement } => retirement.reference.clone(),
        RuntimeMutation::Schema { .. } | RuntimeMutation::Claim { .. } => return None,
    })
}
