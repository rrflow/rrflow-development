//! Index definitions, entries, artifacts, catalogues and receipts.

use super::*;

pub const INDEX_CATALOGUE_CONTRACT_VERSION: u16 = 1;
pub const INDEX_ARTIFACT_CONTRACT_VERSION: u16 = 1;
pub(super) const MAX_INDEX_FIELDS: usize = 16;
pub(super) const MAX_INDEX_ARTIFACT_ROWS: usize = 1_000_000;
pub(super) const MAX_INDEX_OPERATION_RECEIPTS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IndexKind {
    #[default]
    Scalar,
    Count,
    Geo,
    MaterializedView,
    AggregateCount,
    Bm25 {
        #[serde(default)]
        config: Bm25Config,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexDefinition {
    pub id: ProjectionId,
    pub source: Source,
    pub fields: Vec<String>,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub kind: IndexKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Filter>,
}

impl IndexDefinition {
    pub fn validate(&self) -> Result<()> {
        if matches!(self.source, Source::Traversal { .. }) {
            return Err(Error::Catalog(
                "recursive traversal cannot be an index source".into(),
            ));
        }
        let allows_empty_fields = matches!(self.kind, IndexKind::Count);
        if (!allows_empty_fields && self.fields.is_empty()) || self.fields.len() > MAX_INDEX_FIELDS
        {
            return Err(Error::Catalog(format!(
                "this index kind requires 1..={MAX_INDEX_FIELDS} fields"
            )));
        }
        if matches!(self.kind, IndexKind::Bm25 { .. }) && self.fields.len() != 1 {
            return Err(Error::Catalog(
                "a BM25 index requires exactly one text field".into(),
            ));
        }
        if let IndexKind::Bm25 { config } = &self.kind {
            if self.unique {
                return Err(Error::Catalog("a BM25 index cannot be unique".into()));
            }
            config.validate()?;
        }
        if self.unique && !matches!(self.kind, IndexKind::Scalar) {
            return Err(Error::Catalog(
                "only scalar and compound scalar indexes can enforce uniqueness".into(),
            ));
        }
        if self.unique && !matches!(self.source, Source::Record { .. }) {
            return Err(Error::Catalog(
                "unique indexes require an authoritative record source".into(),
            ));
        }
        if matches!(self.kind, IndexKind::Geo) && !matches!(self.source, Source::Geo { .. }) {
            return Err(Error::Catalog("geo indexes require a geo source".into()));
        }
        if !self.filters.is_empty()
            && !matches!(
                self.kind,
                IndexKind::Count | IndexKind::MaterializedView | IndexKind::AggregateCount
            )
        {
            return Err(Error::Catalog(
                "only count, grouped-count, and materialized-view indexes accept predicates".into(),
            ));
        }
        let unique = self.fields.iter().collect::<BTreeSet<_>>();
        if unique.len() != self.fields.len() {
            return Err(Error::Catalog("index fields must be unique".into()));
        }
        Ok(())
    }

    pub fn config_digest(&self) -> Result<String> {
        self.validate()?;
        Ok(digest::sha256_hex(&serde_json::to_vec(self)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexEntry {
    pub definition: IndexDefinition,
    pub stamp: ProjectionStamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub built_valid_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_rows: Option<u64>,
    /// Absent on artifacts built before the canonical result identity format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_identity_version: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_artifact: Option<IndexArtifactReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintenance: Option<IndexMaintenanceEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analytics_total_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analytics_group_count: Option<u64>,
    /// A unique definition becomes an integrity constraint in the same
    /// transaction that installs its schema-bound commit projection. Artifact
    /// rebuild, quarantine, and read-path staleness do not disable it.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unique_validated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexArtifactReference {
    pub generation: u64,
    pub source_cursor: u64,
    pub valid_at: u64,
    pub artifact_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexMaintenanceEvidence {
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_source_cursor: Option<u64>,
    pub source_cursor: u64,
    pub inserted_rows: u64,
    pub updated_rows: u64,
    pub removed_rows: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexCountGroup {
    pub values: Vec<RuntimeValue>,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexAnalyticsArtifact {
    pub total_count: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<IndexCountGroup>,
}

impl IndexEntry {
    pub fn is_usable_at(&self, source_cursor: u64, valid_at: u64) -> bool {
        self.stamp.state == ProjectionState::Ready
            && self.result_identity_version == Some(RESULT_IDENTITY_VERSION)
            && self.stamp.source_cursor == source_cursor
            && self.built_valid_at == Some(valid_at)
            && self.artifact_rows.is_some()
            && self.maintenance.is_some()
            && self
                .definition
                .config_digest()
                .is_ok_and(|digest| self.stamp.config_digest == digest)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexArtifact {
    pub contract_version: u16,
    pub result_identity_version: u16,
    pub scope: ScopeId,
    pub definition: IndexDefinition,
    pub generation: u64,
    pub source_cursor: u64,
    pub read_cursor: u64,
    pub schema_revision: u64,
    pub valid_at: u64,
    pub rows: Vec<QueryRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bm25: Option<Bm25Artifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analytics: Option<IndexAnalyticsArtifact>,
    pub maintenance: IndexMaintenanceEvidence,
}

impl IndexArtifact {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != INDEX_ARTIFACT_CONTRACT_VERSION {
            return Err(Error::Integrity(format!(
                "unsupported index artifact contract version {}",
                self.contract_version
            )));
        }
        if self.result_identity_version != RESULT_IDENTITY_VERSION {
            return Err(Error::Integrity(format!(
                "unsupported index result identity version {}",
                self.result_identity_version
            )));
        }
        self.definition.validate()?;
        if self.generation == 0 || self.source_cursor == 0 || self.read_cursor < self.source_cursor
        {
            return Err(Error::Integrity(
                "index artifact generation/source cursor/read cursor are invalid".into(),
            ));
        }
        if self.rows.len() > MAX_INDEX_ARTIFACT_ROWS {
            return Err(Error::Integrity("index artifact row limit exceeded".into()));
        }
        if self
            .rows
            .windows(2)
            .any(|rows| rows[0].identity >= rows[1].identity)
        {
            return Err(Error::Integrity(
                "index artifact row identities must be unique and sorted".into(),
            ));
        }
        if self.maintenance.source_cursor != self.source_cursor
            || (self.maintenance.mode != "full_build"
                && self.maintenance.mode != "incremental_reconciliation")
        {
            return Err(Error::Integrity(
                "index maintenance evidence does not cover this artifact".into(),
            ));
        }
        match (&self.definition.kind, &self.bm25, &self.analytics) {
            (IndexKind::Scalar | IndexKind::Geo | IndexKind::MaterializedView, None, None) => {}
            (IndexKind::Count | IndexKind::AggregateCount, None, Some(analytics)) => {
                validate_analytics(&self.definition, analytics)?;
            }
            (IndexKind::Bm25 { config }, Some(artifact), None) => {
                artifact.validate()?;
                if &artifact.config != config
                    || artifact.source_cursor != self.source_cursor
                    || artifact.schema_revision != self.schema_revision
                    || artifact.valid_at != self.valid_at
                {
                    return Err(Error::Integrity(
                        "BM25 artifact coordinates differ from the containing query index".into(),
                    ));
                }
            }
            _ => {
                return Err(Error::Integrity(
                    "query index kind and specialized artifact disagree".into(),
                ))
            }
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        Ok(serde_json::to_vec(self)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let artifact: Self = serde_json::from_slice(bytes)?;
        artifact.validate()?;
        if artifact.encode()? != bytes {
            return Err(Error::Integrity(
                "index artifact bytes are not canonical".into(),
            ));
        }
        Ok(artifact)
    }

    pub fn digest(&self) -> Result<String> {
        Ok(digest::sha256_hex(&self.encode()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexCatalogue {
    pub contract_version: u16,
    pub scope: ScopeId,
    pub revision: u64,
    pub entries: BTreeMap<ProjectionId, IndexEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub operations: BTreeMap<String, IndexOperationReceipt>,
}

impl IndexCatalogue {
    pub(crate) fn empty(scope: ScopeId) -> Self {
        Self {
            contract_version: INDEX_CATALOGUE_CONTRACT_VERSION,
            scope,
            revision: 0,
            entries: BTreeMap::new(),
            operations: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.contract_version != INDEX_CATALOGUE_CONTRACT_VERSION {
            return Err(Error::Integrity(format!(
                "unsupported index catalogue contract version {}",
                self.contract_version
            )));
        }
        for (id, entry) in &self.entries {
            validate_index_entry(id, entry)?;
        }
        if self.operations.len() > MAX_INDEX_OPERATION_RECEIPTS {
            return Err(Error::Integrity(
                "index operation receipt limit exceeded".into(),
            ));
        }
        for (key, receipt) in &self.operations {
            if key.is_empty()
                || receipt.operation_digest.len() != 64
                || !receipt
                    .operation_digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err(Error::Integrity(
                    "index operation receipt identity is invalid".into(),
                ));
            }
            validate_index_entry(&receipt.entry.definition.id, &receipt.entry)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexOperationReceipt {
    pub operation_digest: String,
    pub entry: IndexEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexMutationContext {
    pub at: u64,
    pub actor: String,
    pub request_id: String,
    pub operation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexArtifactPublication {
    pub generation: u64,
    pub source_cursor: u64,
    pub valid_at: u64,
    pub artifact_rows: u64,
    pub artifact_digest: String,
    pub maintenance: IndexMaintenanceEvidence,
    pub analytics_total_count: Option<u64>,
    pub analytics_group_count: Option<u64>,
}

impl IndexMutationContext {
    pub(super) fn validate(&self) -> Result<()> {
        if self.at == 0
            || self.actor.is_empty()
            || self.request_id.is_empty()
            || self.operation_id.is_empty()
        {
            return Err(Error::Catalog(
                "index mutation context requires time, actor, request, and operation IDs".into(),
            ));
        }
        Ok(())
    }
}

pub struct IndexCatalogueRepository<'a, E: StorageEngine + ?Sized> {
    pub(super) engine: &'a E,
    pub(super) scope: ScopeId,
    pub(super) key: String,
}
