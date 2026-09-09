//! Canonical vector heads, temporal versions, and materializer source deltas.
//!
//! The semantic transaction persists vector truth and an exact pointer delta.
//! Exact scan, TurboQuant, and HNSW consumers are downstream projections; no
//! accelerator is constructed or allowed to publish from this commit path.

use super::runtime_state::{get_json, scan_space_bounded_from};
use super::semantic_commit::SemanticCommitPlan;
use crate::keyspaces;
use crate::{Error, Result};
use rrd_core::{
    digest, RuntimeChange, RuntimeCommit, RuntimeMutation, RuntimeRef, RuntimeVector, ScopeId,
    VectorCollectionAddress,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const VECTOR_SOURCE_DELTA_CONTRACT_VERSION: u16 = 1;

/// The stable logical source consumed by one vector projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorSourceAddress {
    pub collection: VectorCollectionAddress,
    pub field: String,
}

impl VectorSourceAddress {
    pub fn validate(&self) -> Result<()> {
        if self.field.trim().is_empty() {
            return Err(Error::IndexConstraint(
                "vector source field must not be empty".into(),
            ));
        }
        self.collection.validate()?;
        Ok(())
    }

    fn from_vector(vector: &RuntimeVector) -> Self {
        Self {
            collection: vector.collection.clone(),
            field: vector.field.clone(),
        }
    }
}

/// Pointer to one immutable vector version in rrflowKV/rrflowMX.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorSourceVersion {
    pub reference: RuntimeRef,
    pub valid_from: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u64>,
    pub source_cursor: u64,
    pub commit_id: String,
    pub commit_ordinal: u64,
    pub value_sha256: String,
}

/// Atomic old/new projection input for exact, TurboQuant, or HNSW consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorSourceDelta {
    pub contract_version: u16,
    pub scope: ScopeId,
    pub source: VectorSourceAddress,
    pub source_cursor: u64,
    pub commit_id: String,
    pub commit_ordinal: u64,
    pub reference: RuntimeRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<VectorSourceVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<VectorSourceVersion>,
}

impl VectorSourceDelta {
    pub fn validate(&self) -> Result<()> {
        self.source.validate()?;
        if self.contract_version != VECTOR_SOURCE_DELTA_CONTRACT_VERSION
            || self.source_cursor == 0
            || (self.before.is_none() && self.after.is_none())
            || !is_sha256(&self.commit_id)
        {
            return Err(Error::IndexConstraint(
                "vector source delta is invalid".into(),
            ));
        }
        for version in [&self.before, &self.after].into_iter().flatten() {
            if version.reference != self.reference
                || version.source_cursor == 0
                || !is_sha256(&version.commit_id)
                || !is_sha256(&version.value_sha256)
                || version.source_cursor > self.source_cursor
            {
                return Err(Error::IndexConstraint(
                    "vector source delta contains an invalid version pointer".into(),
                ));
            }
        }
        if self.after.as_ref().is_some_and(|after| {
            after.source_cursor != self.source_cursor
                || after.commit_id != self.commit_id
                || after.commit_ordinal != self.commit_ordinal
        }) {
            return Err(Error::IndexConstraint(
                "vector source delta after-version is not its commit coordinate".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeVectorHead {
    vector: RuntimeVector,
    source_cursor: u64,
    commit_id: String,
    commit_ordinal: u64,
}

impl RuntimeVectorHead {
    fn version(&self) -> Result<VectorSourceVersion> {
        Ok(VectorSourceVersion {
            reference: self.vector.reference.clone(),
            valid_from: self.vector.valid_from,
            valid_to: self.vector.valid_to,
            source_cursor: self.source_cursor,
            commit_id: self.commit_id.clone(),
            commit_ordinal: self.commit_ordinal,
            value_sha256: digest::sha256_hex(&serde_json::to_vec(&self.vector.value)?),
        })
    }
}

#[derive(Debug, Clone)]
struct VectorTransition {
    before: Option<RuntimeVectorHead>,
    after: Option<RuntimeVectorHead>,
    source_cursor: u64,
    commit_ordinal: u64,
}

pub(crate) fn encode_vector_version(
    plan: &mut SemanticCommitPlan,
    scope: &ScopeId,
    vector: &RuntimeVector,
    change: &RuntimeChange,
) -> Result<()> {
    let head = RuntimeVectorHead {
        vector: vector.clone(),
        source_cursor: change.cursor,
        commit_id: change.commit_id.clone(),
        commit_ordinal: change.commit_ordinal,
    };
    plan.put_json(
        keyspaces::RUNTIME_VECTORS,
        &keyspaces::runtime_identity_key(keyspaces::RUNTIME_VECTORS, scope, &vector.reference),
        &head,
    )?;
    plan.put_json(
        keyspaces::RUNTIME_VECTOR_VERSIONS,
        &keyspaces::runtime_vector_version_key(
            scope,
            &vector.reference,
            vector.valid_from,
            change.cursor,
        ),
        change,
    )
}

pub(crate) fn encode_vector_retirement(
    plan: &mut SemanticCommitPlan,
    scope: &ScopeId,
    reference: &RuntimeRef,
    effective_at: u64,
    change: &RuntimeChange,
) -> Result<()> {
    plan.put_json(
        keyspaces::RUNTIME_VECTOR_VERSIONS,
        &keyspaces::runtime_vector_version_key(scope, reference, effective_at, change.cursor),
        change,
    )?;
    plan.delete(
        keyspaces::RUNTIME_VECTORS,
        &keyspaces::runtime_identity_key(keyspaces::RUNTIME_VECTORS, scope, reference),
    )
}

pub(crate) fn encode_vector_source_effects(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    plan: &mut SemanticCommitPlan,
    commit: &RuntimeCommit,
    starting_cursor: u64,
) -> Result<()> {
    let transitions = vector_transitions(reader, commit, starting_cursor)?;
    for (reference, transition) in transitions {
        let before_source = transition
            .before
            .as_ref()
            .map(|head| VectorSourceAddress::from_vector(&head.vector));
        let after_source = transition
            .after
            .as_ref()
            .map(|head| VectorSourceAddress::from_vector(&head.vector));
        let before = transition
            .before
            .as_ref()
            .map(RuntimeVectorHead::version)
            .transpose()?;
        let after = transition
            .after
            .as_ref()
            .map(RuntimeVectorHead::version)
            .transpose()?;
        if before == after {
            continue;
        }
        match (before_source, after_source) {
            (Some(before_source), Some(after_source)) if before_source == after_source => {
                put_delta(
                    plan,
                    commit,
                    &reference,
                    transition.source_cursor,
                    transition.commit_ordinal,
                    before_source,
                    before,
                    after,
                )?;
            }
            (before_source, after_source) => {
                if let Some(source) = before_source {
                    put_delta(
                        plan,
                        commit,
                        &reference,
                        transition.source_cursor,
                        transition.commit_ordinal,
                        source,
                        before,
                        None,
                    )?;
                }
                if let Some(source) = after_source {
                    put_delta(
                        plan,
                        commit,
                        &reference,
                        transition.source_cursor,
                        transition.commit_ordinal,
                        source,
                        None,
                        after,
                    )?;
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn vector_source_deltas(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    source: &VectorSourceAddress,
    after: u64,
    limit: usize,
) -> Result<Vec<VectorSourceDelta>> {
    source.validate()?;
    if limit == 0 {
        return Err(Error::Substrate(
            "vector source-delta page limit must be greater than zero".into(),
        ));
    }
    let start = keyspaces::runtime_vector_source_delta_start(
        scope,
        &source.collection.collection_id,
        &source.collection.vector_name,
        &source.field,
        after.checked_add(1).ok_or(Error::SequenceOverflow)?,
    )?;
    let prefix = keyspaces::runtime_vector_source_delta_prefix(
        scope,
        &source.collection.collection_id,
        &source.collection.vector_name,
        &source.field,
    )?;
    let mut deltas = scan_space_bounded_from(
        reader,
        keyspaces::RUNTIME_VECTOR_SOURCE_DELTAS,
        &prefix,
        &start,
        limit,
    )?
    .into_iter()
    .map(|(_, bytes)| serde_json::from_slice::<VectorSourceDelta>(&bytes).map_err(Error::from))
    .collect::<Result<Vec<_>>>()?;
    for delta in &deltas {
        delta.validate()?;
        if delta.scope != *scope || delta.source != *source {
            return Err(Error::IndexConstraint(
                "vector source-delta key escaped its requested identity".into(),
            ));
        }
    }
    deltas.sort_by_key(|delta| (delta.source_cursor, delta.commit_ordinal));
    Ok(deltas)
}

fn vector_transitions(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    commit: &RuntimeCommit,
    starting_cursor: u64,
) -> Result<BTreeMap<RuntimeRef, VectorTransition>> {
    let mut transitions = BTreeMap::new();
    for (ordinal, mutation) in commit.mutations.iter().enumerate() {
        let (reference, vector) = match mutation {
            RuntimeMutation::Vector { vector } => (&vector.reference, Some(vector.clone())),
            RuntimeMutation::Retire { retirement }
                if retirement.model == rrd_core::RuntimeLogicalModel::Vector =>
            {
                (&retirement.reference, None)
            }
            _ => continue,
        };
        let ordinal = u64::try_from(ordinal)
            .map_err(|_| Error::Substrate("commit ordinal exceeds u64".into()))?;
        let source_cursor = starting_cursor
            .checked_add(ordinal)
            .and_then(|cursor| cursor.checked_add(1))
            .ok_or(Error::SequenceOverflow)?;
        let transition = match transitions.entry(reference.clone()) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let before = get_json(
                    reader,
                    keyspaces::RUNTIME_VECTORS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_VECTORS,
                        &commit.scope,
                        reference,
                    ),
                )?;
                entry.insert(VectorTransition {
                    before,
                    after: None,
                    source_cursor,
                    commit_ordinal: ordinal,
                })
            }
        };
        transition.after = vector.map(|vector| RuntimeVectorHead {
            vector,
            source_cursor,
            commit_id: commit.digest(),
            commit_ordinal: ordinal,
        });
        transition.source_cursor = source_cursor;
        transition.commit_ordinal = ordinal;
    }
    Ok(transitions)
}

#[allow(clippy::too_many_arguments)]
fn put_delta(
    plan: &mut SemanticCommitPlan,
    commit: &RuntimeCommit,
    reference: &RuntimeRef,
    source_cursor: u64,
    commit_ordinal: u64,
    source: VectorSourceAddress,
    before: Option<VectorSourceVersion>,
    after: Option<VectorSourceVersion>,
) -> Result<()> {
    let delta = VectorSourceDelta {
        contract_version: VECTOR_SOURCE_DELTA_CONTRACT_VERSION,
        scope: commit.scope.clone(),
        source,
        source_cursor,
        commit_id: commit.digest(),
        commit_ordinal,
        reference: reference.clone(),
        before,
        after,
    };
    delta.validate()?;
    plan.put_json(
        keyspaces::RUNTIME_VECTOR_SOURCE_DELTAS,
        &keyspaces::runtime_vector_source_delta_key(
            &commit.scope,
            &delta.source.collection.collection_id,
            &delta.source.collection.vector_name,
            &delta.source.field,
            source_cursor,
            commit_ordinal,
        )?,
        &delta,
    )
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
