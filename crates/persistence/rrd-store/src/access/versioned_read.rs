//! Read-stamp-bound access to canonical semantic version entries.
//!
//! This is the normal state-read substrate shared by rrflowMX and rrflowKV.
//! It selects scope-bound typed version ranges, validates every returned key
//! against its value, authenticates each value against the stamped RFC 9162
//! root, and reports the complete bounded access shape. The authoritative
//! change log is not a range source here; replay, changefeed, archive, and
//! recovery keep their explicit paged-log APIs.

use super::runtime_state::{read_accumulator_node, scan_space, validate_read_stamp, AccessRead};
use crate::keyspaces::{self, Space};
use crate::{Error, Result, StorageTransaction};
use rrd_core::{
    Predicate, ReadStamp, RuntimeChange, RuntimeLogAccumulator, RuntimeLogicalModel,
    RuntimeMutation, RuntimeReadValidation, RuntimeRef, RuntimeType,
};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

pub const RUNTIME_VERSIONED_READ_CONTRACT_VERSION: u16 = 1;

/// Hard key-examination bound for one direct semantic read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeReadBudget {
    pub max_keys: u64,
}

impl RuntimeReadBudget {
    pub fn new(max_keys: u64) -> Result<Self> {
        let budget = Self { max_keys };
        budget.validate()?;
        Ok(budget)
    }

    pub fn validate(self) -> Result<()> {
        if self.max_keys == 0 {
            return Err(Error::Substrate(
                "runtime versioned-read key budget must be greater than zero".into(),
            ));
        }
        Ok(())
    }
}

/// One typed semantic version range. An absent discriminator selects the
/// complete family within the stamped scope; a present discriminator narrows
/// the physical prefix before any value is decoded.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "family", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeVersionedSource {
    Schema,
    Claims {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        predicate: Option<Predicate>,
    },
    Records {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<RuntimeType>,
    },
    Relations {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<RuntimeType>,
    },
    Events {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<RuntimeType>,
    },
    Vectors {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<RuntimeType>,
    },
    Series {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<RuntimeType>,
    },
    Geo {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<RuntimeType>,
    },
    Objects {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<RuntimeType>,
    },
}

impl RuntimeVersionedSource {
    pub const fn all() -> [Self; 9] {
        [
            Self::Schema,
            Self::Claims { predicate: None },
            Self::Records { kind: None },
            Self::Relations { kind: None },
            Self::Events { kind: None },
            Self::Vectors { kind: None },
            Self::Series { kind: None },
            Self::Geo { kind: None },
            Self::Objects { kind: None },
        ]
    }

    fn path(&self) -> RuntimeReadAccessPath {
        match self {
            Self::Schema => RuntimeReadAccessPath::SchemaVersions,
            Self::Claims { .. } => RuntimeReadAccessPath::ClaimVersions,
            Self::Records { .. } => RuntimeReadAccessPath::RecordVersions,
            Self::Relations { .. } => RuntimeReadAccessPath::RelationVersions,
            Self::Events { .. } => RuntimeReadAccessPath::EventVersions,
            Self::Vectors { .. } => RuntimeReadAccessPath::VectorVersions,
            Self::Series { .. } => RuntimeReadAccessPath::SeriesVersions,
            Self::Geo { .. } => RuntimeReadAccessPath::GeoVersions,
            Self::Objects { .. } => RuntimeReadAccessPath::ObjectVersions,
        }
    }

    fn is_unbounded_family(&self) -> bool {
        match self {
            Self::Schema => true,
            Self::Claims { predicate } => predicate.is_none(),
            Self::Records { kind }
            | Self::Relations { kind }
            | Self::Events { kind }
            | Self::Vectors { kind }
            | Self::Series { kind }
            | Self::Geo { kind }
            | Self::Objects { kind } => kind.is_none(),
        }
    }
}

/// Closed physical path vocabulary for direct-read evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeReadAccessPath {
    ReadStamp,
    SchemaVersions,
    ClaimVersions,
    RecordVersions,
    RelationVersions,
    EventVersions,
    VectorVersions,
    SeriesVersions,
    GeoVersions,
    ObjectVersions,
    AccumulatorProof,
}

/// Counters attributed to one closed direct-read access path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeReadPathEvidence {
    pub path: RuntimeReadAccessPath,
    pub point_reads: u64,
    pub range_scans: u64,
    pub keys_examined: u64,
    pub values_decoded: u64,
    pub decoded_bytes: u64,
}

/// Complete logical I/O evidence for one stamped direct read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeReadEvidence {
    pub contract_version: u16,
    pub key_budget: u64,
    pub point_reads: u64,
    pub range_scans: u64,
    pub keys_examined: u64,
    pub values_decoded: u64,
    pub decoded_bytes: u64,
    pub stamp_validation: RuntimeReadValidation,
    pub paths: Vec<RuntimeReadPathEvidence>,
}

impl RuntimeReadEvidence {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != RUNTIME_VERSIONED_READ_CONTRACT_VERSION
            || self.key_budget == 0
            || self.keys_examined > self.key_budget
            || self.values_decoded > self.keys_examined
            || self.paths.is_empty()
        {
            return Err(Error::Substrate(
                "runtime versioned-read evidence header is invalid".into(),
            ));
        }
        let mut prior = None;
        let mut totals = PathCounters::default();
        for path in &self.paths {
            if prior.is_some_and(|value| value >= path.path)
                || path.values_decoded > path.keys_examined
            {
                return Err(Error::Substrate(
                    "runtime versioned-read path evidence is invalid".into(),
                ));
            }
            prior = Some(path.path);
            totals.add(
                path.point_reads,
                path.range_scans,
                path.keys_examined,
                path.values_decoded,
                path.decoded_bytes,
            )?;
        }
        if totals.point_reads != self.point_reads
            || totals.range_scans != self.range_scans
            || totals.keys_examined != self.keys_examined
            || totals.values_decoded != self.values_decoded
            || totals.decoded_bytes != self.decoded_bytes
        {
            return Err(Error::Substrate(
                "runtime versioned-read evidence totals do not match its paths".into(),
            ));
        }
        Ok(())
    }
}

/// Authenticated semantic versions returned from exactly one read stamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeVersionedRead {
    pub read: ReadStamp,
    pub changes: Vec<RuntimeChange>,
    pub evidence: RuntimeReadEvidence,
}

pub(crate) fn read_versioned(
    transaction: &dyn StorageTransaction,
    read: &ReadStamp,
    sources: &[RuntimeVersionedSource],
    budget: RuntimeReadBudget,
) -> Result<RuntimeVersionedRead> {
    budget.validate()?;
    let sources = validate_sources(sources)?;
    if read.accumulator_root.is_none() {
        return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
    }

    let reader = MeteredRead::new(transaction, budget);
    let stamp_validation = reader.with_path(RuntimeReadAccessPath::ReadStamp, || {
        validate_read_stamp(&reader, read)
    })?;
    let mut changes = BTreeMap::<u64, RuntimeChange>::new();
    for source in sources {
        let encoded = reader.with_path(source.path(), || scan_source(&reader, read, &source))?;
        for (stored_key, bytes) in encoded {
            let change: RuntimeChange = serde_json::from_slice(&bytes)?;
            if change.cursor > read.commit_cursor {
                continue;
            }
            validate_version_entry(&source, read, &stored_key, &change)?;
            if let Some(prior) = changes.insert(change.cursor, change.clone()) {
                if prior != change {
                    return Err(Error::Substrate(format!(
                        "runtime cursor {} resolved to conflicting semantic versions",
                        change.cursor
                    )));
                }
            }
        }
    }
    reader.with_path(RuntimeReadAccessPath::AccumulatorProof, || {
        authenticate_changes(&reader, read, changes.values())
    })?;
    let evidence = reader.finish(stamp_validation)?;
    let result = RuntimeVersionedRead {
        read: read.clone(),
        changes: changes.into_values().collect(),
        evidence,
    };
    result.evidence.validate()?;
    Ok(result)
}

fn validate_sources(
    sources: &[RuntimeVersionedSource],
) -> Result<BTreeSet<RuntimeVersionedSource>> {
    if sources.is_empty() {
        return Err(Error::Substrate(
            "runtime versioned read requires at least one typed source".into(),
        ));
    }
    let unique = sources.iter().cloned().collect::<BTreeSet<_>>();
    for left in &unique {
        for right in &unique {
            if left < right
                && left.path() == right.path()
                && (left.is_unbounded_family() || right.is_unbounded_family())
            {
                return Err(Error::Substrate(format!(
                    "runtime versioned read contains overlapping {:?} ranges",
                    left.path()
                )));
            }
        }
    }
    Ok(unique)
}

fn scan_source(
    reader: &(impl AccessRead + ?Sized),
    read: &ReadStamp,
    source: &RuntimeVersionedSource,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let (space, prefix) = match source {
        RuntimeVersionedSource::Schema => (
            keyspaces::RUNTIME_SCHEMA_VERSIONS,
            keyspaces::runtime_scope_prefix(keyspaces::RUNTIME_SCHEMA_VERSIONS, &read.scope),
        ),
        RuntimeVersionedSource::Claims { predicate } => (
            keyspaces::RUNTIME_CLAIM_VERSIONS,
            predicate.as_ref().map_or_else(
                || keyspaces::runtime_scope_prefix(keyspaces::RUNTIME_CLAIM_VERSIONS, &read.scope),
                |predicate| keyspaces::runtime_claim_predicate_prefix(&read.scope, predicate),
            ),
        ),
        RuntimeVersionedSource::Records { kind } => version_range(
            keyspaces::RUNTIME_RECORD_VERSIONS,
            &read.scope,
            kind.as_ref(),
        ),
        RuntimeVersionedSource::Relations { kind } => version_range(
            keyspaces::RUNTIME_RELATION_VERSIONS,
            &read.scope,
            kind.as_ref(),
        ),
        RuntimeVersionedSource::Events { kind } => version_range(
            keyspaces::RUNTIME_EVENT_VERSIONS,
            &read.scope,
            kind.as_ref(),
        ),
        RuntimeVersionedSource::Vectors { kind } => version_range(
            keyspaces::RUNTIME_VECTOR_VERSIONS,
            &read.scope,
            kind.as_ref(),
        ),
        RuntimeVersionedSource::Series { kind } => version_range(
            keyspaces::RUNTIME_SERIES_VERSIONS,
            &read.scope,
            kind.as_ref(),
        ),
        RuntimeVersionedSource::Geo { kind } => {
            version_range(keyspaces::RUNTIME_GEO_VERSIONS, &read.scope, kind.as_ref())
        }
        RuntimeVersionedSource::Objects { kind } => version_range(
            keyspaces::RUNTIME_OBJECT_VERSIONS,
            &read.scope,
            kind.as_ref(),
        ),
    };
    scan_space(reader, space, &prefix)
}

fn version_range(
    space: Space,
    scope: &rrd_core::ScopeId,
    kind: Option<&RuntimeType>,
) -> (Space, Vec<u8>) {
    let prefix = kind.map_or_else(
        || keyspaces::runtime_scope_prefix(space, scope),
        |kind| keyspaces::runtime_kind_prefix(space, scope, kind),
    );
    (space, prefix)
}

fn validate_version_entry(
    source: &RuntimeVersionedSource,
    read: &ReadStamp,
    stored_key: &[u8],
    change: &RuntimeChange,
) -> Result<()> {
    if change.scope != read.scope || change.cursor == 0 || change.cursor > read.commit_cursor {
        return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
    }
    if !change.verify_digest() {
        return Err(Error::Substrate(format!(
            "runtime semantic version {} failed digest verification",
            change.cursor
        )));
    }
    let expected = expected_version_key(source, change)?;
    if stored_key != expected {
        return Err(Error::Codec(format!(
            "runtime semantic version {} does not match its typed key",
            change.cursor
        )));
    }
    Ok(())
}

fn expected_version_key(
    source: &RuntimeVersionedSource,
    change: &RuntimeChange,
) -> Result<Vec<u8>> {
    let scope = &change.scope;
    let cursor = change.cursor;
    match (source, &change.mutation) {
        (RuntimeVersionedSource::Schema, RuntimeMutation::Schema { .. }) => {
            Ok(keyspaces::runtime_schema_version_key(scope, cursor))
        }
        (RuntimeVersionedSource::Claims { predicate }, RuntimeMutation::Claim { claim })
            if predicate
                .as_ref()
                .is_none_or(|value| value == &claim.predicate) =>
        {
            Ok(keyspaces::runtime_claim_version_key(scope, claim, cursor))
        }
        (RuntimeVersionedSource::Records { kind }, RuntimeMutation::Record { record })
            if kind
                .as_ref()
                .is_none_or(|value| value == &record.reference.kind) =>
        {
            Ok(keyspaces::runtime_version_key(
                keyspaces::RUNTIME_RECORD_VERSIONS,
                scope,
                &record.reference,
                record.valid_from,
                cursor,
            ))
        }
        (RuntimeVersionedSource::Relations { kind }, RuntimeMutation::Relation { relation })
            if kind
                .as_ref()
                .is_none_or(|value| value == &relation.reference.kind) =>
        {
            Ok(keyspaces::runtime_version_key(
                keyspaces::RUNTIME_RELATION_VERSIONS,
                scope,
                &relation.reference,
                relation.valid_from,
                cursor,
            ))
        }
        (RuntimeVersionedSource::Events { kind }, RuntimeMutation::Event { event })
            if kind.as_ref().is_none_or(|value| value == &event.kind) =>
        {
            let reference = keyspaces::runtime_event_reference(event, cursor)?;
            Ok(keyspaces::runtime_version_key(
                keyspaces::RUNTIME_EVENT_VERSIONS,
                scope,
                &reference,
                change.at,
                cursor,
            ))
        }
        (RuntimeVersionedSource::Vectors { kind }, RuntimeMutation::Vector { vector })
            if kind
                .as_ref()
                .is_none_or(|value| value == &vector.reference.kind) =>
        {
            Ok(keyspaces::runtime_vector_version_key(
                scope,
                &vector.reference,
                vector.valid_from,
                cursor,
            ))
        }
        (RuntimeVersionedSource::Series { kind }, RuntimeMutation::SeriesSample { sample })
            if kind
                .as_ref()
                .is_none_or(|value| value == &sample.reference.kind) =>
        {
            Ok(keyspaces::runtime_version_key(
                keyspaces::RUNTIME_SERIES_VERSIONS,
                scope,
                &sample.reference,
                sample.observed_at,
                cursor,
            ))
        }
        (RuntimeVersionedSource::Geo { kind }, RuntimeMutation::Geo { geo })
            if kind
                .as_ref()
                .is_none_or(|value| value == &geo.reference.kind) =>
        {
            Ok(keyspaces::runtime_version_key(
                keyspaces::RUNTIME_GEO_VERSIONS,
                scope,
                &geo.reference,
                geo.valid_from,
                cursor,
            ))
        }
        (RuntimeVersionedSource::Objects { kind }, RuntimeMutation::Object { object })
            if kind
                .as_ref()
                .is_none_or(|value| value == &object.reference.kind) =>
        {
            Ok(keyspaces::runtime_version_key(
                keyspaces::RUNTIME_OBJECT_VERSIONS,
                scope,
                &object.reference,
                change.at,
                cursor,
            ))
        }
        (source, RuntimeMutation::Retire { retirement })
            if retirement_matches(source, retirement.model, &retirement.reference) =>
        {
            let space = version_space(source)?;
            if space == keyspaces::RUNTIME_VECTOR_VERSIONS {
                Ok(keyspaces::runtime_vector_version_key(
                    scope,
                    &retirement.reference,
                    retirement.effective_at,
                    cursor,
                ))
            } else {
                Ok(keyspaces::runtime_version_key(
                    space,
                    scope,
                    &retirement.reference,
                    retirement.effective_at,
                    cursor,
                ))
            }
        }
        _ => Err(Error::Codec(format!(
            "runtime semantic version {} escaped its selected {:?} family",
            cursor,
            source.path()
        ))),
    }
}

fn retirement_matches(
    source: &RuntimeVersionedSource,
    model: RuntimeLogicalModel,
    reference: &RuntimeRef,
) -> bool {
    let discriminator_matches = match source {
        RuntimeVersionedSource::Records { kind }
        | RuntimeVersionedSource::Relations { kind }
        | RuntimeVersionedSource::Events { kind }
        | RuntimeVersionedSource::Vectors { kind }
        | RuntimeVersionedSource::Series { kind }
        | RuntimeVersionedSource::Geo { kind }
        | RuntimeVersionedSource::Objects { kind } => {
            kind.as_ref().is_none_or(|value| value == &reference.kind)
        }
        RuntimeVersionedSource::Schema | RuntimeVersionedSource::Claims { .. } => false,
    };
    discriminator_matches
        && match source {
            RuntimeVersionedSource::Records { .. } => model.is_record_like(),
            RuntimeVersionedSource::Relations { .. } => model == RuntimeLogicalModel::GraphRelation,
            RuntimeVersionedSource::Events { .. } => model.is_event_like(),
            RuntimeVersionedSource::Vectors { .. } => model == RuntimeLogicalModel::Vector,
            RuntimeVersionedSource::Series { .. } => model == RuntimeLogicalModel::TimeSeries,
            RuntimeVersionedSource::Geo { .. } => model == RuntimeLogicalModel::Geo,
            RuntimeVersionedSource::Objects { .. } => model == RuntimeLogicalModel::Object,
            RuntimeVersionedSource::Schema | RuntimeVersionedSource::Claims { .. } => false,
        }
}

fn version_space(source: &RuntimeVersionedSource) -> Result<Space> {
    match source {
        RuntimeVersionedSource::Schema | RuntimeVersionedSource::Claims { .. } => Err(
            Error::Codec("schema and claim values cannot carry typed retirements".into()),
        ),
        RuntimeVersionedSource::Records { .. } => Ok(keyspaces::RUNTIME_RECORD_VERSIONS),
        RuntimeVersionedSource::Relations { .. } => Ok(keyspaces::RUNTIME_RELATION_VERSIONS),
        RuntimeVersionedSource::Events { .. } => Ok(keyspaces::RUNTIME_EVENT_VERSIONS),
        RuntimeVersionedSource::Vectors { .. } => Ok(keyspaces::RUNTIME_VECTOR_VERSIONS),
        RuntimeVersionedSource::Series { .. } => Ok(keyspaces::RUNTIME_SERIES_VERSIONS),
        RuntimeVersionedSource::Geo { .. } => Ok(keyspaces::RUNTIME_GEO_VERSIONS),
        RuntimeVersionedSource::Objects { .. } => Ok(keyspaces::RUNTIME_OBJECT_VERSIONS),
    }
}

fn authenticate_changes<'a>(
    reader: &(impl AccessRead + ?Sized),
    read: &ReadStamp,
    changes: impl Iterator<Item = &'a RuntimeChange>,
) -> Result<()> {
    let root = read
        .accumulator_root
        .as_deref()
        .ok_or_else(|| Error::ReadStampMismatch(read.manifest_id.clone()))?;
    let accumulator =
        RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
            read_accumulator_node(reader, level, index)
        })?;
    for change in changes {
        let proof = accumulator.inclusion_proof(change.cursor - 1, |level, index| {
            read_accumulator_node(reader, level, index)
        })?;
        proof.verify_change(change, root)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
struct PathCounters {
    point_reads: u64,
    range_scans: u64,
    keys_examined: u64,
    values_decoded: u64,
    decoded_bytes: u64,
}

impl PathCounters {
    fn add(
        &mut self,
        point_reads: u64,
        range_scans: u64,
        keys_examined: u64,
        values_decoded: u64,
        decoded_bytes: u64,
    ) -> Result<()> {
        self.point_reads = checked_add(self.point_reads, point_reads)?;
        self.range_scans = checked_add(self.range_scans, range_scans)?;
        self.keys_examined = checked_add(self.keys_examined, keys_examined)?;
        self.values_decoded = checked_add(self.values_decoded, values_decoded)?;
        self.decoded_bytes = checked_add(self.decoded_bytes, decoded_bytes)?;
        Ok(())
    }
}

struct MeteredRead<'a> {
    inner: &'a dyn StorageTransaction,
    budget: RuntimeReadBudget,
    path: Cell<RuntimeReadAccessPath>,
    totals: RefCell<PathCounters>,
    paths: RefCell<BTreeMap<RuntimeReadAccessPath, PathCounters>>,
}

impl<'a> MeteredRead<'a> {
    fn new(inner: &'a dyn StorageTransaction, budget: RuntimeReadBudget) -> Self {
        Self {
            inner,
            budget,
            path: Cell::new(RuntimeReadAccessPath::ReadStamp),
            totals: RefCell::new(PathCounters::default()),
            paths: RefCell::new(BTreeMap::new()),
        }
    }

    fn with_path<T>(&self, path: RuntimeReadAccessPath, operation: impl FnOnce() -> T) -> T {
        let prior = self.path.replace(path);
        let result = operation();
        self.path.set(prior);
        result
    }

    fn record(
        &self,
        point_reads: u64,
        range_scans: u64,
        keys_examined: u64,
        values_decoded: u64,
        decoded_bytes: u64,
    ) -> Result<()> {
        self.totals.borrow_mut().add(
            point_reads,
            range_scans,
            keys_examined,
            values_decoded,
            decoded_bytes,
        )?;
        self.paths
            .borrow_mut()
            .entry(self.path.get())
            .or_default()
            .add(
                point_reads,
                range_scans,
                keys_examined,
                values_decoded,
                decoded_bytes,
            )
    }

    fn finish(self, stamp_validation: RuntimeReadValidation) -> Result<RuntimeReadEvidence> {
        let totals = self.totals.into_inner();
        let paths = self
            .paths
            .into_inner()
            .into_iter()
            .map(|(path, counters)| RuntimeReadPathEvidence {
                path,
                point_reads: counters.point_reads,
                range_scans: counters.range_scans,
                keys_examined: counters.keys_examined,
                values_decoded: counters.values_decoded,
                decoded_bytes: counters.decoded_bytes,
            })
            .collect();
        Ok(RuntimeReadEvidence {
            contract_version: RUNTIME_VERSIONED_READ_CONTRACT_VERSION,
            key_budget: self.budget.max_keys,
            point_reads: totals.point_reads,
            range_scans: totals.range_scans,
            keys_examined: totals.keys_examined,
            values_decoded: totals.values_decoded,
            decoded_bytes: totals.decoded_bytes,
            stamp_validation,
            paths,
        })
    }
}

impl AccessRead for MeteredRead<'_> {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let observed = checked_add(self.totals.borrow().keys_examined, 1)?;
        if observed > self.budget.max_keys {
            return Err(Error::RuntimeReadBudgetExceeded {
                limit: self.budget.max_keys,
                observed,
            });
        }
        let value = self.inner.get(key)?;
        let (decoded, bytes) = value.as_ref().map_or((0, 0), |value| {
            (1, u64::try_from(value.len()).unwrap_or(u64::MAX))
        });
        self.record(1, 0, 1, decoded, bytes)?;
        Ok(value)
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: &[u8],
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let examined = self.totals.borrow().keys_examined;
        let remaining = self.budget.max_keys.saturating_sub(examined);
        let probe_limit = usize::try_from(remaining.saturating_add(1))
            .unwrap_or(usize::MAX)
            .min(limit);
        let rows = self.inner.scan(start, end, probe_limit)?;
        let row_count = u64::try_from(rows.len()).unwrap_or(u64::MAX);
        let observed = checked_add(examined, row_count)?;
        if observed > self.budget.max_keys {
            return Err(Error::RuntimeReadBudgetExceeded {
                limit: self.budget.max_keys,
                observed,
            });
        }
        let decoded_bytes = rows.iter().try_fold(0_u64, |total, (_, value)| {
            checked_add(total, u64::try_from(value.len()).unwrap_or(u64::MAX))
        })?;
        self.record(0, 1, row_count, row_count, decoded_bytes)?;
        Ok(rows)
    }
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right).ok_or(Error::SequenceOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rrd_core::{RuntimeCommit, RuntimeSchemaRegistry, ScopeId};

    fn schema_change() -> (ReadStamp, RuntimeChange) {
        let scope = ScopeId::new("project:version-entry-validation").unwrap();
        let commit = RuntimeCommit {
            scope: scope.clone(),
            at: 100,
            actor: "test:version-entry-validation".into(),
            expected_cursor: 0,
            mutations: vec![RuntimeMutation::Schema {
                registry: RuntimeSchemaRegistry::empty(1, "version entry validation"),
            }],
        };
        let change = RuntimeChange::committed(
            1,
            &commit,
            &commit.digest(),
            0,
            commit.mutations[0].clone(),
            None,
        );
        let read = ReadStamp::authenticated(
            scope,
            Some(1),
            0,
            1,
            Some(change.digest.clone()),
            "11".repeat(32),
        )
        .unwrap();
        (read, change)
    }

    #[test]
    fn a_version_value_cannot_move_to_another_typed_key_or_family() {
        let (read, change) = schema_change();
        let mut key = keyspaces::runtime_schema_version_key(&read.scope, change.cursor);
        validate_version_entry(&RuntimeVersionedSource::Schema, &read, &key, &change).unwrap();

        *key.last_mut().unwrap() ^= 1;
        assert!(matches!(
            validate_version_entry(&RuntimeVersionedSource::Schema, &read, &key, &change,),
            Err(Error::Codec(_))
        ));
        assert!(matches!(
            validate_version_entry(
                &RuntimeVersionedSource::Records { kind: None },
                &read,
                &keyspaces::runtime_schema_version_key(&read.scope, change.cursor),
                &change,
            ),
            Err(Error::Codec(_))
        ));
    }

    #[test]
    fn overlapping_family_ranges_are_rejected_before_storage_access() {
        assert!(validate_sources(&[
            RuntimeVersionedSource::Records { kind: None },
            RuntimeVersionedSource::Records {
                kind: Some(RuntimeType::new("document").unwrap()),
            },
        ])
        .is_err());
    }
}
