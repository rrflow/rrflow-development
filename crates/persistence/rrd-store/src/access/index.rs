//! Transaction-bound scalar, unique, and search-source index effects.
//!
//! The query catalogue remains the definition authority. This module stores a
//! digest-bound physical projection of the definition fields required during a
//! semantic commit, so rrflowMX and rrflowKV can derive the same old/new
//! effects without depending on rrflowQL or replaying the runtime log.

use super::super::keyspaces::{self, Space};
use super::runtime_state::{
    checked_key, get, get_json, read_sequence, scan_space_bounded, scan_space_bounded_from,
};
use super::semantic_commit::SemanticCommitPlan;
use crate::{Error, Result, StorageTransaction};
use rrd_core::{
    digest, ProjectionId, RuntimeCommit, RuntimeMutation, RuntimeRecord, RuntimeRef,
    RuntimeSchemaRegistry, RuntimeType, RuntimeValue, RuntimeValueType, ScopeId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const INDEX_COMMIT_BINDING_CONTRACT_VERSION: u16 = 1;
pub const INDEX_SOURCE_DELTA_CONTRACT_VERSION: u16 = 1;
const MAX_INDEX_FIELDS: usize = 16;
const MAX_INDEX_BACKFILL_RECORDS: usize = 1_000_000;
const MAX_INDEX_COMMIT_BINDINGS: usize = 65_536;
const MAX_INDEX_ENTRIES_PER_DEFINITION: usize = 1_000_000;
const MAX_UNIQUE_VALUE_ENTRIES: usize = 1_000_000;

/// The commit-time behavior projected from one rrflowQL index definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexCommitBindingKind {
    Scalar { unique: bool },
    Bm25,
}

/// Provider-neutral definition data needed by the storage transaction.
///
/// The owning catalogue bytes and accepted revisions are attached inside the
/// same control transaction; callers cannot claim those coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexCommitBindingDefinition {
    pub id: ProjectionId,
    pub record_kind: RuntimeType,
    pub fields: Vec<String>,
    pub kind: IndexCommitBindingKind,
    pub configuration_sha256: String,
}

impl IndexCommitBindingDefinition {
    pub fn validate(&self) -> Result<()> {
        if self.fields.is_empty() || self.fields.len() > MAX_INDEX_FIELDS {
            return Err(Error::IndexConstraint(format!(
                "index {} requires 1..={MAX_INDEX_FIELDS} commit-bound fields",
                self.id
            )));
        }
        if matches!(self.kind, IndexCommitBindingKind::Bm25) && self.fields.len() != 1 {
            return Err(Error::IndexConstraint(format!(
                "BM25 index {} requires exactly one commit-bound field",
                self.id
            )));
        }
        if self.fields.iter().any(|field| field.trim().is_empty())
            || self.fields.iter().collect::<BTreeSet<_>>().len() != self.fields.len()
        {
            return Err(Error::IndexConstraint(format!(
                "index {} fields must be non-empty and unique",
                self.id
            )));
        }
        validate_sha256("index configuration", &self.configuration_sha256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexCommitBinding {
    contract_version: u16,
    scope: ScopeId,
    definition: IndexCommitBindingDefinition,
    schema_revision: u64,
    catalogue_revision: u64,
    source_control_key: String,
    source_control_sha256: String,
}

impl IndexCommitBinding {
    fn validate(&self) -> Result<()> {
        if self.contract_version != INDEX_COMMIT_BINDING_CONTRACT_VERSION
            || self.schema_revision == 0
            || self.catalogue_revision == 0
            || self.source_control_key.is_empty()
        {
            return Err(Error::IndexConstraint(format!(
                "index {} has an invalid commit binding",
                self.definition.id
            )));
        }
        if self.source_control_key != format!("server/state/index-catalogue/{}", self.scope) {
            return Err(Error::IndexConstraint(format!(
                "index {} commit binding names a foreign catalogue key",
                self.definition.id
            )));
        }
        self.definition.validate()?;
        validate_sha256("index source catalogue", &self.source_control_sha256)
    }
}

/// Scope-level integrity head for the complete commit-binding projection.
///
/// The authoritative catalogue remains a control record. This head proves
/// that the whole derived binding set came from the exact current catalogue
/// bytes rather than from a partial, stale, or generic control transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexCommitBindingSet {
    contract_version: u16,
    scope: ScopeId,
    schema_revision: u64,
    catalogue_revision: u64,
    source_control_key: String,
    source_control_sha256: String,
    definition_count: u64,
    definitions_sha256: String,
}

impl IndexCommitBindingSet {
    fn validate(&self) -> Result<()> {
        let expected_key = format!("server/state/index-catalogue/{}", self.scope);
        if self.contract_version != INDEX_COMMIT_BINDING_CONTRACT_VERSION
            || self.schema_revision == 0
            || self.catalogue_revision == 0
            || self.source_control_key != expected_key
            || self.definition_count > MAX_INDEX_COMMIT_BINDINGS as u64
        {
            return Err(Error::IndexConstraint(format!(
                "scope {} has an invalid index commit-binding set",
                self.scope
            )));
        }
        validate_sha256("index source catalogue", &self.source_control_sha256)?;
        validate_sha256("index binding definitions", &self.definitions_sha256)
    }
}

/// Exact field values and validity window on one side of a committed change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexSourceRow {
    pub values: Vec<RuntimeValue>,
    pub valid_from: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u64>,
}

/// Durable old/new input for scalar, BM25, and later index materializers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexSourceDelta {
    pub contract_version: u16,
    pub scope: ScopeId,
    pub index_id: ProjectionId,
    pub binding_kind: IndexCommitBindingKind,
    pub configuration_sha256: String,
    pub source_control_sha256: String,
    pub schema_revision: u64,
    pub catalogue_revision: u64,
    pub source_cursor: u64,
    pub commit_id: String,
    pub commit_ordinal: u64,
    pub record: RuntimeRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<IndexSourceRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<IndexSourceRow>,
}

impl IndexSourceDelta {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != INDEX_SOURCE_DELTA_CONTRACT_VERSION
            || self.schema_revision == 0
            || self.catalogue_revision == 0
            || self.source_cursor == 0
            || (self.before.is_none() && self.after.is_none())
        {
            return Err(Error::IndexConstraint(format!(
                "index source delta for {} is invalid",
                self.index_id
            )));
        }
        validate_sha256("index source commit", &self.commit_id)?;
        validate_sha256("index configuration", &self.configuration_sha256)?;
        validate_sha256("index source catalogue", &self.source_control_sha256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScalarIndexEntry {
    index_id: ProjectionId,
    record: RuntimeRef,
    values: Vec<RuntimeValue>,
    valid_from: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    valid_to: Option<u64>,
    source_cursor: u64,
}

#[derive(Debug, Clone)]
struct RecordTransition {
    before: Option<RuntimeRecord>,
    after: Option<RuntimeRecord>,
    source_cursor: u64,
    commit_ordinal: u64,
}

pub(crate) fn synchronize_index_commit_bindings(
    transaction: &mut dyn StorageTransaction,
    scope: &ScopeId,
    expected_schema_revision: u64,
    catalogue_revision: u64,
    source_control_key: &str,
    source_control_sha256: &str,
    definitions: &[IndexCommitBindingDefinition],
) -> Result<()> {
    validate_sha256("index source catalogue", source_control_sha256)?;
    let schema: RuntimeSchemaRegistry = get_json(
        &*transaction,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(scope),
    )?
    .ok_or_else(|| Error::RuntimeSchemaMissing(scope.to_string()))?;
    if schema.revision != expected_schema_revision {
        return Err(Error::RuntimeSchemaConflict {
            expected: expected_schema_revision,
            actual: schema.revision,
        });
    }

    let mut requested = BTreeMap::new();
    for definition in definitions {
        definition.validate()?;
        validate_binding_schema(definition, &schema)?;
        if requested
            .insert(definition.id.clone(), definition.clone())
            .is_some()
        {
            return Err(Error::IndexConstraint(format!(
                "duplicate commit binding for index {}",
                definition.id
            )));
        }
    }
    let existing = raw_bindings_for_scope(&*transaction, scope)?
        .into_iter()
        .map(|binding| (binding.definition.id.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    if let Some(foreign) = existing
        .values()
        .find(|binding| binding.source_control_key != source_control_key)
    {
        return Err(Error::IndexConstraint(format!(
            "index {} is bound to a different catalogue authority",
            foreign.definition.id
        )));
    }

    for (id, prior) in &existing {
        if !requested.contains_key(id) {
            clear_index_entries(transaction, scope, prior)?;
            transaction.delete(checked_key(
                keyspaces::RUNTIME_INDEX_BINDINGS,
                &keyspaces::runtime_index_binding_key(scope, id),
            )?)?;
        }
    }

    let canonical_definitions = requested.values().cloned().collect::<Vec<_>>();
    for definition in requested.into_values() {
        let binding = IndexCommitBinding {
            contract_version: INDEX_COMMIT_BINDING_CONTRACT_VERSION,
            scope: scope.clone(),
            definition,
            schema_revision: expected_schema_revision,
            catalogue_revision,
            source_control_key: source_control_key.to_owned(),
            source_control_sha256: source_control_sha256.to_owned(),
        };
        binding.validate()?;
        let requires_backfill = existing
            .get(&binding.definition.id)
            .is_none_or(|prior| prior.definition != binding.definition);
        if let Some(prior) = existing.get(&binding.definition.id) {
            if requires_backfill {
                clear_index_entries(transaction, scope, prior)?;
            }
        }
        if requires_backfill {
            backfill_index_entries(transaction, &binding)?;
        }
        put_json(
            transaction,
            keyspaces::RUNTIME_INDEX_BINDINGS,
            &keyspaces::runtime_index_binding_key(scope, &binding.definition.id),
            &binding,
        )?;
        put_json(
            transaction,
            keyspaces::RUNTIME_INDEX_BINDING_VERSIONS,
            &keyspaces::runtime_index_binding_version_key(
                scope,
                &binding.definition.id,
                catalogue_revision,
                expected_schema_revision,
            ),
            &binding,
        )?;
    }
    let binding_set = IndexCommitBindingSet {
        contract_version: INDEX_COMMIT_BINDING_CONTRACT_VERSION,
        scope: scope.clone(),
        schema_revision: expected_schema_revision,
        catalogue_revision,
        source_control_key: source_control_key.to_owned(),
        source_control_sha256: source_control_sha256.to_owned(),
        definition_count: u64::try_from(canonical_definitions.len())
            .map_err(|_| Error::IndexConstraint("index binding count exceeds u64".into()))?,
        definitions_sha256: definitions_sha256(&canonical_definitions)?,
    };
    binding_set.validate()?;
    put_json(
        transaction,
        keyspaces::RUNTIME_INDEX_BINDING_SETS,
        &keyspaces::runtime_index_binding_set_key(scope),
        &binding_set,
    )?;
    Ok(())
}

pub(crate) fn encode_record_index_effects(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    plan: &mut SemanticCommitPlan,
    commit: &RuntimeCommit,
    schema: &RuntimeSchemaRegistry,
    catalogue_revision: u64,
    starting_cursor: u64,
) -> Result<()> {
    let source_control_key = format!("server/state/index-catalogue/{}", commit.scope);
    let source_control = get(
        reader,
        keyspaces::META,
        &keyspaces::control_record_key(&source_control_key),
    )?;
    let mut binding_set: Option<IndexCommitBindingSet> = get_json(
        reader,
        keyspaces::RUNTIME_INDEX_BINDING_SETS,
        &keyspaces::runtime_index_binding_set_key(&commit.scope),
    )?;
    let mut bindings = raw_bindings_for_scope(reader, &commit.scope)?;
    match (&source_control, &binding_set) {
        (None, None) if bindings.is_empty() => return Ok(()),
        (None, None) => {
            return Err(Error::IndexConstraint(format!(
                "scope {} has orphaned index commit bindings",
                commit.scope
            )));
        }
        (Some(_), None) => {
            return Err(Error::IndexConstraint(format!(
                "scope {} index catalogue has no atomic commit-binding set",
                commit.scope
            )));
        }
        (None, Some(_)) => {
            return Err(Error::IndexConstraint(format!(
                "scope {} index commit-binding set has no source catalogue",
                commit.scope
            )));
        }
        (Some(_), Some(_)) => {}
    }
    let source_control = source_control.expect("matched source catalogue bytes");
    let binding_set = binding_set
        .as_mut()
        .expect("matched index commit-binding set");
    binding_set.validate()?;
    if binding_set.scope != commit.scope
        || binding_set.source_control_key != source_control_key
        || binding_set.catalogue_revision > catalogue_revision
        || binding_set.schema_revision > schema.revision
        || binding_set.source_control_sha256 != digest::sha256_hex(&source_control)
    {
        return Err(Error::IndexConstraint(format!(
            "scope {} index commit-binding set does not match its accepted catalogue and schema",
            commit.scope
        )));
    }
    let definitions = bindings
        .iter()
        .map(|binding| binding.definition.clone())
        .collect::<Vec<_>>();
    let binding_count = u64::try_from(bindings.len())
        .map_err(|_| Error::IndexConstraint("index binding count exceeds u64".into()))?;
    if binding_set.definition_count != binding_count
        || binding_set.definitions_sha256 != definitions_sha256(&definitions)?
    {
        return Err(Error::IndexConstraint(format!(
            "scope {} index commit-binding projection is incomplete",
            commit.scope
        )));
    }
    for binding in &mut bindings {
        binding.validate()?;
        if binding.scope != commit.scope
            || binding.schema_revision != binding_set.schema_revision
            || binding.catalogue_revision != binding_set.catalogue_revision
            || binding.source_control_key != binding_set.source_control_key
            || binding.source_control_sha256 != binding_set.source_control_sha256
        {
            return Err(Error::IndexConstraint(format!(
                "index {} escaped its accepted commit-binding set",
                binding.definition.id
            )));
        }
        if binding.catalogue_revision > catalogue_revision {
            return Err(Error::IndexConstraint(format!(
                "index {} commit binding names a future catalogue revision",
                binding.definition.id
            )));
        }
        validate_binding_schema(&binding.definition, schema)?;
        if binding.schema_revision != schema.revision {
            binding.schema_revision = schema.revision;
            binding.catalogue_revision = catalogue_revision;
            plan.put_json(
                keyspaces::RUNTIME_INDEX_BINDINGS,
                &keyspaces::runtime_index_binding_key(&commit.scope, &binding.definition.id),
                binding,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_INDEX_BINDING_VERSIONS,
                &keyspaces::runtime_index_binding_version_key(
                    &commit.scope,
                    &binding.definition.id,
                    catalogue_revision,
                    schema.revision,
                ),
                binding,
            )?;
        }
    }
    if binding_set.schema_revision != schema.revision {
        binding_set.schema_revision = schema.revision;
        binding_set.catalogue_revision = catalogue_revision;
        plan.put_json(
            keyspaces::RUNTIME_INDEX_BINDING_SETS,
            &keyspaces::runtime_index_binding_set_key(&commit.scope),
            binding_set,
        )?;
    }

    let transitions = record_transitions(reader, commit, starting_cursor)?;
    for binding in bindings {
        let relevant = transitions
            .iter()
            .filter(|(reference, _)| reference.kind == binding.definition.record_kind)
            .collect::<Vec<_>>();
        if relevant.is_empty() {
            continue;
        }
        if matches!(
            binding.definition.kind,
            IndexCommitBindingKind::Scalar { unique: true }
        ) {
            validate_unique_transition(reader, &commit.scope, &binding, &relevant)?;
        }
        for (reference, transition) in relevant {
            let before = transition
                .before
                .as_ref()
                .map(|record| source_row(record, &binding.definition.fields));
            let after = transition
                .after
                .as_ref()
                .map(|record| source_row(record, &binding.definition.fields));
            if before == after {
                continue;
            }
            if let IndexCommitBindingKind::Scalar { unique } = binding.definition.kind {
                let space = if unique {
                    keyspaces::RUNTIME_UNIQUE_ENTRIES
                } else {
                    keyspaces::RUNTIME_SCALAR_ENTRIES
                };
                if let Some(row) = &before {
                    if !unique || !row.values.contains(&RuntimeValue::Null) {
                        plan.delete(
                            space,
                            &keyspaces::runtime_index_entry_key(
                                space,
                                &commit.scope,
                                &binding.definition.id,
                                &row.values,
                                reference,
                                row.valid_from,
                            )?,
                        )?;
                    }
                }
                if let Some(row) = &after {
                    if !unique || !row.values.contains(&RuntimeValue::Null) {
                        let entry = ScalarIndexEntry {
                            index_id: binding.definition.id.clone(),
                            record: (*reference).clone(),
                            values: row.values.clone(),
                            valid_from: row.valid_from,
                            valid_to: row.valid_to,
                            source_cursor: transition.source_cursor,
                        };
                        plan.put_json(
                            space,
                            &keyspaces::runtime_index_entry_key(
                                space,
                                &commit.scope,
                                &binding.definition.id,
                                &row.values,
                                reference,
                                row.valid_from,
                            )?,
                            &entry,
                        )?;
                    }
                }
            }
            let delta = IndexSourceDelta {
                contract_version: INDEX_SOURCE_DELTA_CONTRACT_VERSION,
                scope: commit.scope.clone(),
                index_id: binding.definition.id.clone(),
                binding_kind: binding.definition.kind,
                configuration_sha256: binding.definition.configuration_sha256.clone(),
                source_control_sha256: binding.source_control_sha256.clone(),
                schema_revision: schema.revision,
                catalogue_revision,
                source_cursor: transition.source_cursor,
                commit_id: commit.digest(),
                commit_ordinal: transition.commit_ordinal,
                record: (*reference).clone(),
                before,
                after,
            };
            delta.validate()?;
            plan.put_json(
                keyspaces::RUNTIME_INDEX_SOURCE_DELTAS,
                &keyspaces::runtime_index_source_delta_key(
                    &commit.scope,
                    &binding.definition.id,
                    transition.source_cursor,
                    transition.commit_ordinal,
                ),
                &delta,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn index_source_deltas(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    index: &ProjectionId,
    after: u64,
    limit: usize,
) -> Result<Vec<IndexSourceDelta>> {
    if limit == 0 {
        return Err(Error::Substrate(
            "index source-delta page limit must be greater than zero".into(),
        ));
    }
    let start = keyspaces::runtime_index_source_delta_start(
        scope,
        index,
        after.checked_add(1).ok_or(Error::SequenceOverflow)?,
    );
    let prefix = keyspaces::runtime_index_source_delta_prefix(scope, index);
    let mut deltas = scan_space_bounded_from(
        reader,
        keyspaces::RUNTIME_INDEX_SOURCE_DELTAS,
        &prefix,
        &start,
        limit,
    )?
    .into_iter()
    .map(|(_, bytes)| serde_json::from_slice::<IndexSourceDelta>(&bytes).map_err(Error::from))
    .collect::<Result<Vec<_>>>()?;
    for delta in &deltas {
        delta.validate()?;
        if delta.scope != *scope || delta.index_id != *index {
            return Err(Error::IndexConstraint(
                "index source-delta key escaped its requested identity".into(),
            ));
        }
    }
    deltas.sort_by_key(|delta| (delta.source_cursor, delta.commit_ordinal));
    Ok(deltas)
}

fn raw_bindings_for_scope(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
) -> Result<Vec<IndexCommitBinding>> {
    let prefix = keyspaces::runtime_index_binding_scope_prefix(scope);
    let encoded = scan_space_bounded(
        reader,
        keyspaces::RUNTIME_INDEX_BINDINGS,
        &prefix,
        MAX_INDEX_COMMIT_BINDINGS + 1,
    )?;
    if encoded.len() > MAX_INDEX_COMMIT_BINDINGS {
        return Err(Error::IndexConstraint(format!(
            "scope {scope} exceeds {MAX_INDEX_COMMIT_BINDINGS} commit-bound indexes"
        )));
    }
    encoded
        .into_iter()
        .map(|(_, bytes)| {
            let binding: IndexCommitBinding = serde_json::from_slice(&bytes)?;
            binding.validate()?;
            if binding.scope != *scope {
                return Err(Error::IndexConstraint(
                    "index commit binding escaped its scope key".into(),
                ));
            }
            Ok(binding)
        })
        .collect()
}

fn definitions_sha256(definitions: &[IndexCommitBindingDefinition]) -> Result<String> {
    let mut canonical = definitions.to_vec();
    canonical.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(digest::sha256_hex(&serde_json::to_vec(&canonical)?))
}

fn validate_binding_schema(
    definition: &IndexCommitBindingDefinition,
    schema: &RuntimeSchemaRegistry,
) -> Result<()> {
    let record = schema.records.get(&definition.record_kind).ok_or_else(|| {
        Error::IndexConstraint(format!(
            "index {} references unknown record type {}",
            definition.id, definition.record_kind
        ))
    })?;
    for field in &definition.fields {
        let value_type = match field.as_str() {
            "id" | "kind" => RuntimeValueType::String,
            "valid_from" | "valid_to" => RuntimeValueType::Unsigned,
            _ => record
                .properties
                .get(field)
                .map(|property| property.value_type)
                .ok_or_else(|| {
                    Error::IndexConstraint(format!(
                        "index {} references untyped field {field:?}",
                        definition.id
                    ))
                })?,
        };
        if matches!(value_type, RuntimeValueType::List | RuntimeValueType::Map) {
            return Err(Error::IndexConstraint(format!(
                "index {} cannot encode non-scalar field {field:?}",
                definition.id
            )));
        }
        if matches!(definition.kind, IndexCommitBindingKind::Bm25)
            && value_type != RuntimeValueType::String
        {
            return Err(Error::IndexConstraint(format!(
                "BM25 index {} requires a string field",
                definition.id
            )));
        }
    }
    Ok(())
}

fn clear_index_entries(
    transaction: &mut dyn StorageTransaction,
    scope: &ScopeId,
    binding: &IndexCommitBinding,
) -> Result<()> {
    if !matches!(
        binding.definition.kind,
        IndexCommitBindingKind::Scalar { .. }
    ) {
        return Ok(());
    }
    let space = scalar_space(binding.definition.kind);
    let prefix = keyspaces::runtime_index_entry_prefix(space, scope, &binding.definition.id);
    let entries = scan_space_bounded(
        &*transaction,
        space,
        &prefix,
        MAX_INDEX_ENTRIES_PER_DEFINITION + 1,
    )?;
    if entries.len() > MAX_INDEX_ENTRIES_PER_DEFINITION {
        return Err(Error::IndexConstraint(format!(
            "index {} exceeds {MAX_INDEX_ENTRIES_PER_DEFINITION} removable entries",
            binding.definition.id
        )));
    }
    for (key, _) in entries {
        transaction.delete(checked_key(space, &key)?)?;
    }
    Ok(())
}

fn backfill_index_entries(
    transaction: &mut dyn StorageTransaction,
    binding: &IndexCommitBinding,
) -> Result<()> {
    if !matches!(
        binding.definition.kind,
        IndexCommitBindingKind::Scalar { .. }
    ) {
        return Ok(());
    }
    let prefix = keyspaces::runtime_scope_prefix(keyspaces::RUNTIME_RECORDS, &binding.scope);
    let records = scan_space_bounded(
        &*transaction,
        keyspaces::RUNTIME_RECORDS,
        &prefix,
        MAX_INDEX_BACKFILL_RECORDS + 1,
    )?;
    if records.len() > MAX_INDEX_BACKFILL_RECORDS {
        return Err(Error::IndexConstraint(format!(
            "index {} backfill exceeds {MAX_INDEX_BACKFILL_RECORDS} records",
            binding.definition.id
        )));
    }
    let records = records
        .into_iter()
        .map(|(_, bytes)| serde_json::from_slice::<RuntimeRecord>(&bytes).map_err(Error::from))
        .collect::<Result<Vec<_>>>()?;
    let source_cursor = read_sequence(transaction, &keyspaces::runtime_cursor_key())?;
    let relevant = records
        .iter()
        .filter(|record| record.reference.kind == binding.definition.record_kind)
        .collect::<Vec<_>>();
    if matches!(
        binding.definition.kind,
        IndexCommitBindingKind::Scalar { unique: true }
    ) {
        validate_unique_records(
            &binding.definition.id,
            &binding.definition.fields,
            &relevant,
        )?;
    }
    let space = scalar_space(binding.definition.kind);
    for record in relevant {
        let row = source_row(record, &binding.definition.fields);
        if matches!(
            binding.definition.kind,
            IndexCommitBindingKind::Scalar { unique: true }
        ) && row.values.contains(&RuntimeValue::Null)
        {
            continue;
        }
        let entry = ScalarIndexEntry {
            index_id: binding.definition.id.clone(),
            record: record.reference.clone(),
            values: row.values.clone(),
            valid_from: row.valid_from,
            valid_to: row.valid_to,
            source_cursor,
        };
        put_json(
            transaction,
            space,
            &keyspaces::runtime_index_entry_key(
                space,
                &binding.scope,
                &binding.definition.id,
                &row.values,
                &record.reference,
                row.valid_from,
            )?,
            &entry,
        )?;
    }
    Ok(())
}

fn record_transitions(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    commit: &RuntimeCommit,
    starting_cursor: u64,
) -> Result<BTreeMap<RuntimeRef, RecordTransition>> {
    let mut transitions = BTreeMap::new();
    for (ordinal, mutation) in commit.mutations.iter().enumerate() {
        let (reference, after) = match mutation {
            RuntimeMutation::Record { record } => (&record.reference, Some(record.clone())),
            RuntimeMutation::Retire { retirement } if retirement.model.is_record_like() => {
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
                    keyspaces::RUNTIME_RECORDS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RECORDS,
                        &commit.scope,
                        reference,
                    ),
                )?;
                entry.insert(RecordTransition {
                    before,
                    after: None,
                    source_cursor,
                    commit_ordinal: ordinal,
                })
            }
        };
        transition.after = after;
        transition.source_cursor = source_cursor;
        transition.commit_ordinal = ordinal;
    }
    Ok(transitions)
}

fn validate_unique_transition(
    reader: &(impl super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    binding: &IndexCommitBinding,
    transitions: &[(&RuntimeRef, &RecordTransition)],
) -> Result<()> {
    let candidates = transitions
        .iter()
        .filter_map(|(reference, transition)| {
            transition.after.as_ref().map(|record| {
                (
                    (*reference).clone(),
                    source_row(record, &binding.definition.fields),
                )
            })
        })
        .filter(|(_, row)| !row.values.contains(&RuntimeValue::Null))
        .collect::<Vec<_>>();
    for (index, (left_ref, left)) in candidates.iter().enumerate() {
        if candidates[index + 1..].iter().any(|(right_ref, right)| {
            left_ref != right_ref
                && left.values == right.values
                && windows_overlap(
                    left.valid_from,
                    left.valid_to,
                    right.valid_from,
                    right.valid_to,
                )
        }) {
            return Err(unique_conflict(&binding.definition.id));
        }
    }
    let replaced = transitions
        .iter()
        .map(|(reference, _)| (*reference).clone())
        .collect::<BTreeSet<_>>();
    for (_, candidate) in &candidates {
        let prefix = keyspaces::runtime_index_value_prefix(
            keyspaces::RUNTIME_UNIQUE_ENTRIES,
            scope,
            &binding.definition.id,
            &candidate.values,
        )?;
        let existing = scan_space_bounded(
            reader,
            keyspaces::RUNTIME_UNIQUE_ENTRIES,
            &prefix,
            MAX_UNIQUE_VALUE_ENTRIES + 1,
        )?;
        if existing.len() > MAX_UNIQUE_VALUE_ENTRIES {
            return Err(Error::IndexConstraint(format!(
                "unique index {} value exceeds {MAX_UNIQUE_VALUE_ENTRIES} live windows",
                binding.definition.id
            )));
        }
        for (_, bytes) in existing {
            let existing: ScalarIndexEntry = serde_json::from_slice(&bytes)?;
            if !replaced.contains(&existing.record)
                && windows_overlap(
                    candidate.valid_from,
                    candidate.valid_to,
                    existing.valid_from,
                    existing.valid_to,
                )
            {
                return Err(unique_conflict(&binding.definition.id));
            }
        }
    }
    Ok(())
}

fn validate_unique_records(
    index: &ProjectionId,
    fields: &[String],
    records: &[&RuntimeRecord],
) -> Result<()> {
    let mut groups = BTreeMap::<Vec<u8>, Vec<&RuntimeRecord>>::new();
    for record in records {
        let row = source_row(record, fields);
        if row.values.contains(&RuntimeValue::Null) {
            continue;
        }
        groups
            .entry(serde_json::to_vec(&row.values)?)
            .or_default()
            .push(record);
    }
    for group in groups.values() {
        for (position, left) in group.iter().enumerate() {
            if group[position + 1..].iter().any(|right| {
                windows_overlap(
                    left.valid_from,
                    left.valid_to,
                    right.valid_from,
                    right.valid_to,
                )
            }) {
                return Err(unique_conflict(index));
            }
        }
    }
    Ok(())
}

fn source_row(record: &RuntimeRecord, fields: &[String]) -> IndexSourceRow {
    let values = fields
        .iter()
        .map(|field| match field.as_str() {
            "id" => RuntimeValue::String(record.reference.id.to_string()),
            "kind" => RuntimeValue::String(record.reference.kind.to_string()),
            "valid_from" => RuntimeValue::Unsigned(record.valid_from),
            "valid_to" => record
                .valid_to
                .map_or(RuntimeValue::Null, RuntimeValue::Unsigned),
            _ => record
                .properties
                .get(field)
                .cloned()
                .unwrap_or(RuntimeValue::Null),
        })
        .collect();
    IndexSourceRow {
        values,
        valid_from: record.valid_from,
        valid_to: record.valid_to,
    }
}

fn scalar_space(kind: IndexCommitBindingKind) -> Space {
    match kind {
        IndexCommitBindingKind::Scalar { unique: false } => keyspaces::RUNTIME_SCALAR_ENTRIES,
        IndexCommitBindingKind::Scalar { unique: true } => keyspaces::RUNTIME_UNIQUE_ENTRIES,
        IndexCommitBindingKind::Bm25 => {
            unreachable!("BM25 bindings do not own scalar entry keys")
        }
    }
}

fn windows_overlap(
    left_from: u64,
    left_to: Option<u64>,
    right_from: u64,
    right_to: Option<u64>,
) -> bool {
    left_from < right_to.unwrap_or(u64::MAX) && right_from < left_to.unwrap_or(u64::MAX)
}

fn unique_conflict(index: &ProjectionId) -> Error {
    Error::IndexConstraint(format!(
        "unique index {index} rejects overlapping duplicate records"
    ))
}

fn validate_sha256(label: &str, value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(Error::IndexConstraint(format!(
            "{label} must be a lowercase SHA-256"
        )));
    }
    Ok(())
}

fn put_json<T: Serialize>(
    transaction: &mut dyn StorageTransaction,
    space: Space,
    key: &[u8],
    value: &T,
) -> Result<()> {
    transaction.put(checked_key(space, key)?, serde_json::to_vec(value)?)
}
