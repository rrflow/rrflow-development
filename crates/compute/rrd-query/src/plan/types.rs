//! Bound queries, logical and physical operators, candidates and explanations.

use super::*;

pub type Parameters = BTreeMap<String, RuntimeValue>;
pub(super) type FieldTypes = Vec<RuntimeValueType>;
pub type QueryFieldTypes = BTreeMap<String, FieldTypes>;
pub(super) type FieldCatalog = QueryFieldTypes;
pub(super) type BuiltinFields = &'static [(&'static str, &'static [RuntimeValueType])];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundFilter {
    pub field: String,
    #[serde(default, skip_serializing_if = "ComparisonOperator::is_equal")]
    pub comparison: ComparisonOperator,
    pub value: RuntimeValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundQuery {
    pub contract_version: u16,
    pub read: ReadStamp,
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join: Option<Join>,
    pub valid_at: u64,
    pub known_at_cursor: u64,
    pub source_cursor: u64,
    pub version_sources: Vec<RuntimeVersionedSource>,
    pub field_types: QueryFieldTypes,
    pub filters: Vec<BoundFilter>,
    pub projection: Projection,
    pub limit: Option<usize>,
    pub explain_contract: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub explain_analyze: bool,
    pub schema_revision: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub index_candidates: Vec<BoundIndexCandidate>,
    /// Scalar and BM25 indexes over this record source. Each has a commit
    /// binding from the moment it is defined, so its entries or postings are
    /// current at the head without a built artifact.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commit_indexes: Vec<BoundCommitIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundCommitIndex {
    pub id: ProjectionId,
    pub kind: IndexKind,
    pub fields: Vec<String>,
    pub model: rrd_core::RuntimeLogicalModel,
}

/// What a commit-bound index is asked for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "probe", rename_all = "snake_case")]
pub enum CommitIndexProbe {
    /// Equality on the index's leading fields.
    Scalar { values: Vec<RuntimeValue> },
    /// The query's only predicate, a MATCH on the index's field, ranked over
    /// every current document and cut at `top_k`.
    Bm25 {
        query: String,
        config: crate::Bm25Config,
        top_k: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundIndexCandidate {
    pub id: ProjectionId,
    pub generation: u64,
    pub source_cursor: u64,
    pub state: ProjectionState,
    pub config_digest: String,
    pub artifact_digest: String,
    pub built_valid_at: Option<u64>,
    pub artifact_rows: Option<u64>,
    pub fields: Vec<String>,
    pub matched_prefix: usize,
    pub kind: IndexKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<BoundFilter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operator", rename_all = "snake_case")]
pub enum LogicalOperator {
    Scan {
        source: Source,
    },
    Join {
        source: Source,
        left_field: String,
        right_field: String,
    },
    Temporal {
        valid_at: u64,
        known_at_cursor: u64,
    },
    Filter {
        predicates: Vec<BoundFilter>,
    },
    Project {
        projection: Projection,
    },
    Limit {
        rows: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalPlan {
    pub contract_version: u16,
    pub result_identity_version: u16,
    pub read: ReadStamp,
    pub schema_revision: u64,
    pub source_cursor: u64,
    pub version_sources: Vec<RuntimeVersionedSource>,
    pub field_types: QueryFieldTypes,
    pub operators: Vec<LogicalOperator>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub explain_analyze: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operator", rename_all = "snake_case")]
pub enum PhysicalOperator {
    VersionedRead {
        sources: Vec<RuntimeVersionedSource>,
        exact: bool,
        stable_order: String,
    },
    /// Reads `sources` (each traversal's start identity plus any joined
    /// family), then expands every bounded traversal one breadth-first level
    /// at a time through temporal adjacency ranges and the identity
    /// histories of the relations they name.
    AdjacencyTraversal {
        sources: Vec<RuntimeVersionedSource>,
        exact: bool,
        stable_order: String,
    },
    MaterializedIndex {
        id: ProjectionId,
        kind: IndexKind,
        generation: u64,
        source_cursor: u64,
        schema_revision: u64,
        valid_at: u64,
        config_digest: String,
        artifact_digest: String,
        artifact_rows: u64,
        exact: bool,
        stable_order: String,
    },
    /// Narrows the read to the identities a commit-bound index selects at the
    /// head, then reads those identities' authenticated versions; every
    /// predicate is still evaluated. When the index cannot answer for this
    /// stamp or valid time, `fallback` is read instead.
    CommitIndexRead {
        id: ProjectionId,
        probe: CommitIndexProbe,
        model: rrd_core::RuntimeLogicalModel,
        fallback: Vec<RuntimeVersionedSource>,
        exact: bool,
        stable_order: String,
    },
    DataFusionEvaluate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidatePath {
    pub name: String,
    pub selected: bool,
    pub exact: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionContract {
    pub read_manifest: String,
    pub scope: String,
    pub valid_at: u64,
    pub known_at_cursor: u64,
    pub source_cursor: u64,
    pub schema_revision: u64,
    pub exact: bool,
    pub deterministic_order: String,
    pub read_authentication: String,
    pub read_budget_unit: String,
    pub network_required: bool,
    pub gpu_required: bool,
    pub authorization_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanExplanation {
    pub contract: ExecutionContract,
    pub candidates: Vec<CandidatePath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalPlan {
    pub logical: LogicalPlan,
    pub operators: Vec<PhysicalOperator>,
    pub explanation: PlanExplanation,
    pub digest: String,
}
