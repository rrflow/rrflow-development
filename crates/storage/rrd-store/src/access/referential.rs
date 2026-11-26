//! Authenticated, bounded dependency selection for temporal relation integrity.
use super::runtime_state::AccessRead;
use super::{
    read_versioned_expanding, schema_at_read, RuntimeAdjacencyDirection, RuntimeReadBudget,
    RuntimeReadEvidence, RuntimeVersionedSource,
};
use crate::{Error, Result};
use rrflow_kernel::{
    ReadStamp, RuntimeChange, RuntimeCommit, RuntimeLogicalModel, RuntimeMutation, RuntimeRef,
    RuntimeRetirement, RuntimeSchemaRegistry,
};
use std::collections::BTreeSet;

pub(crate) fn relation_actions(
    reader: &(impl AccessRead + ?Sized),
    read: &ReadStamp,
    commit: &RuntimeCommit,
    budget: RuntimeReadBudget,
) -> Result<(Vec<RuntimeRetirement>, Option<RuntimeReadEvidence>)> {
    if !commit.mutations.iter().any(|mutation| match mutation {
        // An open-ended put only adds endpoint coverage under the canonical
        // newest-known-version reducer. It cannot invalidate an existing edge.
        RuntimeMutation::Record { record } => record.valid_to.is_some(),
        RuntimeMutation::Relation { .. } => true,
        RuntimeMutation::Retire { retirement } => {
            retirement.model.is_record_like()
                || retirement.model == RuntimeLogicalModel::GraphRelation
        }
        _ => false,
    }) {
        return Ok((Vec::new(), None));
    }
    let mut selection = Selection {
        schema: None,
        records: BTreeSet::new(),
        relations: BTreeSet::new(),
        sources: 1,
        limit: budget.max_keys,
    };
    let mut failure = None;
    let versions = read_versioned_expanding(
        reader,
        read,
        &[RuntimeVersionedSource::Schema],
        budget,
        &mut |changes| match selection.expand(read, commit, changes) {
            Ok(sources) => sources,
            Err(error) => {
                failure = Some(error);
                Vec::new()
            }
        },
    )?;
    if let Some(error) = failure {
        return Err(error);
    }
    let schema = selection
        .schema
        .ok_or_else(|| Error::RuntimeSchemaMissing(read.scope.to_string()))?;
    let actions = rrflow_kernel::relation_referential_actions(&schema, &versions.changes, commit)
        .map_err(|error| match error {
        rrflow_kernel::Error::InvalidRuntime { reason } => Error::DanglingRuntimeReference(reason),
        other => Error::Kernel(other),
    })?;
    Ok((actions, Some(versions.evidence)))
}

struct Selection {
    schema: Option<RuntimeSchemaRegistry>,
    records: BTreeSet<RuntimeRef>,
    relations: BTreeSet<RuntimeRef>,
    sources: u64,
    limit: u64,
}

impl Selection {
    fn expand(
        &mut self,
        read: &ReadStamp,
        commit: &RuntimeCommit,
        changes: &[RuntimeChange],
    ) -> Result<Vec<RuntimeVersionedSource>> {
        let mut next = Vec::new();
        if self.schema.is_none() {
            self.schema = Some(
                match commit.mutations.iter().find_map(|mutation| match mutation {
                    RuntimeMutation::Schema { registry } => Some(registry.clone()),
                    _ => None,
                }) {
                    Some(schema) => schema,
                    None => schema_at_read(read, changes)?,
                },
            );
            let mut affected = BTreeSet::new();
            for mutation in &commit.mutations {
                match mutation {
                    RuntimeMutation::Record { record } if record.valid_to.is_some() => {
                        affected.insert(record.reference.clone());
                    }
                    RuntimeMutation::Retire { retirement } if retirement.model.is_record_like() => {
                        affected.insert(retirement.reference.clone());
                    }
                    RuntimeMutation::Relation { relation } => {
                        self.relation(&mut next, &relation.reference)?;
                        self.record(&mut next, &relation.from)?;
                        self.record(&mut next, &relation.to)?;
                    }
                    RuntimeMutation::Retire { retirement }
                        if retirement.model == RuntimeLogicalModel::GraphRelation =>
                    {
                        self.relation(&mut next, &retirement.reference)?;
                    }
                    _ => {}
                }
            }
            // Only directly changed endpoints fan out. Reading their opposite
            // endpoints below must not recursively scan the entire graph.
            for node in affected {
                self.record(&mut next, &node)?;
                for direction in [
                    RuntimeAdjacencyDirection::Outgoing,
                    RuntimeAdjacencyDirection::Incoming,
                ] {
                    self.push(
                        &mut next,
                        RuntimeVersionedSource::Incident {
                            node: node.clone(),
                            direction,
                        },
                    )?;
                }
            }
        } else {
            for change in changes {
                if let RuntimeMutation::Relation { relation } = &change.mutation {
                    // An adjacency version is only discovery. Complete identity
                    // history includes later moves to other endpoints.
                    self.relation(&mut next, &relation.reference)?;
                    self.record(&mut next, &relation.from)?;
                    self.record(&mut next, &relation.to)?;
                }
            }
        }
        Ok(next)
    }

    fn record(
        &mut self,
        next: &mut Vec<RuntimeVersionedSource>,
        reference: &RuntimeRef,
    ) -> Result<()> {
        if !self.records.insert(reference.clone()) {
            return Ok(());
        }
        let model = self
            .schema
            .as_ref()
            .expect("schema selected")
            .tables
            .get(&reference.kind)
            .filter(|table| table.model.is_record_like())
            .map(|table| table.model)
            .ok_or_else(|| {
                Error::DanglingRuntimeReference(format!(
                    "endpoint {}/{} is not a record table",
                    reference.kind, reference.id
                ))
            })?;
        self.push(
            next,
            RuntimeVersionedSource::Identity {
                model,
                reference: reference.clone(),
            },
        )
    }

    fn relation(
        &mut self,
        next: &mut Vec<RuntimeVersionedSource>,
        reference: &RuntimeRef,
    ) -> Result<()> {
        if !self.relations.insert(reference.clone()) {
            return Ok(());
        }
        self.push(
            next,
            RuntimeVersionedSource::Identity {
                model: RuntimeLogicalModel::GraphRelation,
                reference: reference.clone(),
            },
        )
    }

    fn push(
        &mut self,
        next: &mut Vec<RuntimeVersionedSource>,
        source: RuntimeVersionedSource,
    ) -> Result<()> {
        self.sources = self.sources.checked_add(1).ok_or(Error::SequenceOverflow)?;
        if self.sources > self.limit {
            return Err(Error::RuntimeReadBudgetExceeded {
                limit: self.limit,
                observed: self.sources,
            });
        }
        next.push(source);
        Ok(())
    }
}
