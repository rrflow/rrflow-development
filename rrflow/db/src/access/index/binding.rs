//! Index commit-binding definitions, binding sets, source deltas and entry types.

use super::*;

pub const INDEX_COMMIT_BINDING_CONTRACT_VERSION: u16 = 1;
pub const INDEX_SOURCE_DELTA_CONTRACT_VERSION: u16 = 1;
pub(super) const MAX_INDEX_FIELDS: usize = 16;
pub(super) const MAX_INDEX_BACKFILL_RECORDS: usize = 1_000_000;
pub(super) const MAX_INDEX_COMMIT_BINDINGS: usize = 65_536;
pub(super) const MAX_INDEX_ENTRIES_PER_DEFINITION: usize = 1_000_000;
pub(super) const MAX_UNIQUE_VALUE_ENTRIES: usize = 1_000_000;

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
    /// The text pipeline of a BM25 binding, which maintains its postings at
    /// commit. Present exactly when `kind` is BM25.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexical: Option<crate::lexical::Bm25Config>,
}

impl IndexCommitBindingDefinition {
    pub fn validate(&self) -> Result<()> {
        if self.fields.is_empty() || self.fields.len() > MAX_INDEX_FIELDS {
            return Err(Error::IndexConstraint(format!(
                "index {} requires 1..={MAX_INDEX_FIELDS} commit-bound fields",
                self.id
            )));
        }
        match (self.kind, &self.lexical) {
            (IndexCommitBindingKind::Bm25, Some(config)) => config.validate()?,
            (IndexCommitBindingKind::Bm25, None) => {
                return Err(Error::IndexConstraint(format!(
                    "BM25 index {} requires its lexical configuration",
                    self.id
                )));
            }
            (IndexCommitBindingKind::Scalar { .. }, Some(_)) => {
                return Err(Error::IndexConstraint(format!(
                    "scalar index {} cannot carry a lexical configuration",
                    self.id
                )));
            }
            (IndexCommitBindingKind::Scalar { .. }, None) => {}
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
pub(super) struct IndexCommitBinding {
    pub(super) contract_version: u16,
    pub(super) scope: ScopeId,
    pub(super) definition: IndexCommitBindingDefinition,
    pub(super) schema_revision: u64,
    pub(super) catalogue_revision: u64,
    pub(super) source_control_key: String,
    pub(super) source_control_sha256: String,
}

impl IndexCommitBinding {
    pub(super) fn validate(&self) -> Result<()> {
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
pub(super) struct IndexCommitBindingSet {
    pub(super) contract_version: u16,
    pub(super) scope: ScopeId,
    pub(super) schema_revision: u64,
    pub(super) catalogue_revision: u64,
    pub(super) source_control_key: String,
    pub(super) source_control_sha256: String,
    pub(super) definition_count: u64,
    pub(super) definitions_sha256: String,
}

impl IndexCommitBindingSet {
    pub(super) fn validate(&self) -> Result<()> {
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
pub(super) struct ScalarIndexEntry {
    pub(super) index_id: ProjectionId,
    pub(super) record: RuntimeRef,
    pub(super) values: Vec<RuntimeValue>,
    pub(super) valid_from: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) valid_to: Option<u64>,
    pub(super) source_cursor: u64,
}

#[derive(Debug, Clone)]
pub(super) struct RecordTransition {
    pub(super) before: Option<RuntimeRecord>,
    pub(super) after: Option<RuntimeRecord>,
    pub(super) source_cursor: u64,
    pub(super) commit_ordinal: u64,
}
