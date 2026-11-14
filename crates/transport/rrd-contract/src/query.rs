//! rrflowQL requests, results, live queries and query indexes.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum QueryValue {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Decimal(String),
    String(String),
    Digest(String),
    List(Vec<QueryValue>),
    Map(BTreeMap<String, QueryValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryBudget {
    pub max_storage_keys: u64,
    pub max_rows: u64,
    pub max_output_bytes: u64,
    pub max_batch_rows: u64,
    #[serde(default = "default_query_memory_bytes")]
    pub max_memory_bytes: u64,
    #[serde(default = "default_query_spill_bytes")]
    pub max_spill_bytes: u64,
    #[serde(default = "default_query_elapsed_ms")]
    pub max_elapsed_ms: u64,
}

impl Default for QueryBudget {
    fn default() -> Self {
        Self {
            max_storage_keys: 100_000,
            max_rows: 10_000,
            max_output_bytes: 512 * 1024,
            max_batch_rows: 256,
            max_memory_bytes: default_query_memory_bytes(),
            max_spill_bytes: default_query_spill_bytes(),
            max_elapsed_ms: default_query_elapsed_ms(),
        }
    }
}

impl QueryBudget {
    pub fn validate(&self) -> Result<()> {
        for (name, value, maximum) in [
            (
                "max_storage_keys",
                self.max_storage_keys,
                MAX_QUERY_STORAGE_KEYS,
            ),
            ("max_rows", self.max_rows, MAX_QUERY_ROWS),
            (
                "max_output_bytes",
                self.max_output_bytes,
                MAX_QUERY_OUTPUT_BYTES,
            ),
            ("max_batch_rows", self.max_batch_rows, MAX_QUERY_BATCH_ROWS),
            (
                "max_memory_bytes",
                self.max_memory_bytes,
                MAX_QUERY_MEMORY_BYTES,
            ),
            (
                "max_spill_bytes",
                self.max_spill_bytes,
                MAX_QUERY_SPILL_BYTES,
            ),
            ("max_elapsed_ms", self.max_elapsed_ms, MAX_QUERY_ELAPSED_MS),
        ] {
            if value == 0 || value > maximum {
                return invalid(format!("query {name} must be in 1..={maximum}"));
            }
        }
        Ok(())
    }
}

pub(crate) const fn default_query_memory_bytes() -> u64 {
    64 * 1024 * 1024
}

pub(crate) const fn default_query_spill_bytes() -> u64 {
    256 * 1024 * 1024
}

pub(crate) const fn default_query_elapsed_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecuteQuery {
    pub scope: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, QueryValue>,
    #[serde(default)]
    pub budget: QueryBudget,
}

impl ExecuteQuery {
    pub fn validate(&self) -> Result<()> {
        if self.scope.is_empty() || self.scope.len() > MAX_ID_BYTES || self.scope.contains('\0') {
            return invalid(format!(
                "query scope length must be in 1..={MAX_ID_BYTES} bytes and contain no NUL"
            ));
        }
        if self.query.trim().is_empty() || self.query.len() > MAX_QUERY_BYTES {
            return invalid(format!(
                "query text length must be in 1..={MAX_QUERY_BYTES} bytes"
            ));
        }
        if self.parameters.len() > MAX_QUERY_PARAMETERS {
            return invalid(format!(
                "query parameters may contain at most {MAX_QUERY_PARAMETERS} entries"
            ));
        }
        if self.parameters.values().any(|value| {
            !matches!(
                value,
                QueryValue::Null
                    | QueryValue::Bool(_)
                    | QueryValue::Integer(_)
                    | QueryValue::Unsigned(_)
                    | QueryValue::String(_)
            )
        }) {
            return invalid(
                "query parameters support only null, boolean, integer, unsigned, and string values",
            );
        }
        let parameter_bytes = serde_json::to_vec(&self.parameters)
            .map_err(|error| ContractError(error.to_string()))?
            .len();
        if parameter_bytes > MAX_QUERY_PARAMETER_BYTES {
            return invalid(format!(
                "query parameters may encode at most {MAX_QUERY_PARAMETER_BYTES} bytes"
            ));
        }
        self.budget.validate()
    }
}

/// A bounded rrflowQL transaction program plus canonical typed mutation
/// bindings. The program selects ordering and commit/cancel disposition; the
/// bindings reuse the one public transaction mutation vocabulary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecuteQueryTransaction {
    pub scope: String,
    pub program: String,
    pub mutation_bindings: BTreeMap<String, TransactionMutation>,
    pub timeout_ms: u64,
}

impl ExecuteQueryTransaction {
    pub fn validate(&self) -> Result<()> {
        if self.scope.is_empty() || self.scope.len() > MAX_ID_BYTES || self.scope.contains('\0') {
            return invalid(format!(
                "query transaction scope length must be in 1..={MAX_ID_BYTES} bytes and contain no NUL"
            ));
        }
        if self.program.trim().is_empty() || self.program.len() > MAX_QUERY_BYTES {
            return invalid(format!(
                "query transaction program length must be in 1..={MAX_QUERY_BYTES} bytes"
            ));
        }
        if self.mutation_bindings.is_empty()
            || self.mutation_bindings.len() > MAX_QUERY_TRANSACTION_MUTATIONS
        {
            return invalid(format!(
                "query transaction binding count must be in 1..={MAX_QUERY_TRANSACTION_MUTATIONS}"
            ));
        }
        if self.mutation_bindings.keys().any(|name| {
            name.is_empty()
                || name.len() > MAX_ID_BYTES
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
        }) {
            return invalid("query transaction binding names must be bounded identifiers");
        }
        for mutation in self.mutation_bindings.values() {
            mutation.validate()?;
        }
        let bytes = serde_json::to_vec(&self.mutation_bindings)
            .map_err(|error| ContractError(error.to_string()))?
            .len();
        if bytes > MAX_QUERY_TRANSACTION_BINDING_BYTES {
            return invalid(format!(
                "query transaction bindings may encode at most {MAX_QUERY_TRANSACTION_BINDING_BYTES} bytes"
            ));
        }
        if !(MIN_LEASE_MS..=MAX_LEASE_MS).contains(&self.timeout_ms) {
            return invalid(format!(
                "query transaction timeout_ms must be in {MIN_LEASE_MS}..={MAX_LEASE_MS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryTransactionResult {
    pub canonical_program: String,
    pub transaction_id: CorrelationId,
    pub read_cursor: u64,
    pub state: TransactionState,
    pub mutation_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<CommitReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryPlanCandidate {
    pub name: String,
    pub selected: bool,
    pub exact: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryPlanSnapshot {
    pub plan_sha256: String,
    /// Revision of the security authority compiled before query binding. Zero
    /// denotes an explicitly unsecured loopback development engine.
    pub security_policy_revision: u64,
    pub authorization_sha256: String,
    pub exact: bool,
    pub deterministic_order: String,
    pub authorization_boundary: String,
    pub candidates: Vec<QueryPlanCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryRowSnapshot {
    pub identity: String,
    pub values: BTreeMap<String, QueryValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryExecutionSnapshot {
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub returned_rows: u64,
    pub output_bytes: u64,
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<QueryExecutionAnalysisSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryExecutionAnalysisSnapshot {
    pub engine: String,
    pub provider_scans: u64,
    pub input_rows: u64,
    pub input_batches: u64,
    pub input_memory_bytes: u64,
    pub output_batches: u64,
    pub projection_pushdown: String,
    pub filter_pushdown: String,
    pub limit_pushdown: String,
    pub physical_operators: u64,
    pub peak_memory_bytes: u64,
    pub spill_count: u64,
    pub spilled_bytes: u64,
    pub spilled_rows: u64,
    pub elapsed_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryResult {
    pub canonical_query: String,
    pub scope: String,
    pub read_manifest_sha256: String,
    pub known_at_cursor: u64,
    pub result_identity_version: u16,
    pub schema_revision: u64,
    pub plan: QueryPlanSnapshot,
    pub execution: QueryExecutionSnapshot,
    pub rows: Vec<QueryRowSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PollLiveQuery {
    pub scope: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, QueryValue>,
    pub after_cursor: u64,
    /// Identity version of the caller's retained snapshot; required on resume.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_identity_version: Option<u16>,
    #[serde(default)]
    pub budget: QueryBudget,
    pub max_delta_rows: u64,
    #[serde(default)]
    pub wait_timeout_ms: u64,
}

impl PollLiveQuery {
    pub fn validate(&self) -> Result<()> {
        ExecuteQuery {
            scope: self.scope.clone(),
            query: self.query.clone(),
            parameters: self.parameters.clone(),
            budget: self.budget.clone(),
        }
        .validate()?;
        if self.result_identity_version == Some(0) {
            return invalid("live query result identity version must be nonzero");
        }
        if self.max_delta_rows == 0 || self.max_delta_rows > MAX_LIVE_QUERY_DELTA_ROWS {
            return invalid(format!(
                "live query max_delta_rows must be in 1..={MAX_LIVE_QUERY_DELTA_ROWS}"
            ));
        }
        if self.wait_timeout_ms > MAX_LIVE_QUERY_WAIT_MS {
            return invalid(format!(
                "live query wait_timeout_ms must be in 0..={MAX_LIVE_QUERY_WAIT_MS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiveQueryRowChange {
    pub before: QueryRowSnapshot,
    pub after: QueryRowSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiveQueryDeltaResult {
    pub timed_out: bool,
    pub waited_ms: u64,
    pub query_sha256: String,
    pub result_identity_version: u16,
    pub from_cursor: u64,
    pub through_cursor: u64,
    pub head_cursor: u64,
    pub added: Vec<QueryRowSnapshot>,
    pub updated: Vec<LiveQueryRowChange>,
    pub removed: Vec<QueryRowSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureQueryIndex {
    pub scope: String,
    pub index_id: CanonicalId,
    pub definition_query: String,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub kind: QueryIndexKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_text: Option<QueryFullTextConfiguration>,
    #[serde(default)]
    pub budget: QueryBudget,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryIndexKind {
    #[default]
    Scalar,
    Count,
    Geo,
    MaterializedView,
    AggregateCount,
    Bm25,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryTextAnalyzer {
    #[default]
    UnicodeLowercase,
    UnicodeCaseSensitive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryTextTokenizer {
    #[default]
    UnicodeAlphanumeric,
    Whitespace,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryTextStemmer {
    #[default]
    None,
    English,
}

/// How a code identifier such as `read_stamp` or `RrdEngine` is indexed.
/// `pieces` keeps only the tokenizer's pieces; `whole_and_parts` also indexes
/// each `_`-joined identifier whole and the case-change parts of a
/// mixed-case piece, so a question can name an identifier whole or by its
/// words.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryTextIdentifiers {
    #[default]
    Pieces,
    WholeAndParts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryFullTextConfiguration {
    #[serde(default)]
    pub analyzer: QueryTextAnalyzer,
    #[serde(default)]
    pub tokenizer: QueryTextTokenizer,
    #[serde(default)]
    pub ascii_folding: bool,
    #[serde(default)]
    pub stop_words: BTreeSet<String>,
    #[serde(default = "default_query_min_token_chars")]
    pub min_token_chars: u16,
    #[serde(default = "default_query_max_token_chars")]
    pub max_token_chars: u16,
    #[serde(default)]
    pub stemmer: QueryTextStemmer,
    #[serde(default)]
    pub identifiers: QueryTextIdentifiers,
    #[serde(default = "default_query_bm25_k1_micros")]
    pub k1_micros: u32,
    #[serde(default = "default_query_bm25_b_micros")]
    pub b_micros: u32,
}

impl Default for QueryFullTextConfiguration {
    fn default() -> Self {
        Self {
            analyzer: QueryTextAnalyzer::default(),
            tokenizer: QueryTextTokenizer::default(),
            ascii_folding: false,
            stop_words: BTreeSet::new(),
            min_token_chars: default_query_min_token_chars(),
            max_token_chars: default_query_max_token_chars(),
            stemmer: QueryTextStemmer::default(),
            identifiers: QueryTextIdentifiers::default(),
            k1_micros: default_query_bm25_k1_micros(),
            b_micros: default_query_bm25_b_micros(),
        }
    }
}

impl QueryFullTextConfiguration {
    fn validate(&self) -> Result<()> {
        if self.min_token_chars == 0
            || self.min_token_chars > self.max_token_chars
            || self.max_token_chars > 40
            || self.k1_micros == 0
            || self.k1_micros > 10_000_000
            || self.b_micros > 1_000_000
            || self.stop_words.len() > 10_000
            || self.stop_words.iter().any(|word| word.is_empty())
        {
            return invalid("full-text index configuration is outside supported bounds");
        }
        Ok(())
    }
}

pub(crate) const fn default_query_min_token_chars() -> u16 {
    1
}
pub(crate) const fn default_query_max_token_chars() -> u16 {
    40
}
pub(crate) const fn default_query_bm25_k1_micros() -> u32 {
    1_200_000
}
pub(crate) const fn default_query_bm25_b_micros() -> u32 {
    750_000
}

impl EnsureQueryIndex {
    pub fn validate(&self) -> Result<()> {
        match (self.kind, &self.full_text) {
            (QueryIndexKind::Bm25, Some(config)) => config.validate()?,
            (QueryIndexKind::Bm25, None) | (_, None) => {}
            (_, Some(_)) => return invalid("full_text configuration requires kind bm25"),
        }
        ExecuteQuery {
            scope: self.scope.clone(),
            query: self.definition_query.clone(),
            parameters: BTreeMap::new(),
            budget: self.budget.clone(),
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListQueryIndexes {
    pub scope: String,
}

impl ListQueryIndexes {
    pub fn validate(&self) -> Result<()> {
        if self.scope.is_empty() || self.scope.len() > MAX_ID_BYTES || self.scope.contains('\0') {
            return invalid(format!(
                "query index scope length must be in 1..={MAX_ID_BYTES} bytes and contain no NUL"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryIndexState {
    Building,
    Ready,
    Quarantined,
    Retiring,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryIndexSnapshot {
    pub index_id: CanonicalId,
    pub definition_query: String,
    pub unique: bool,
    #[serde(default)]
    pub kind: QueryIndexKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_text: Option<QueryFullTextConfiguration>,
    pub generation: u64,
    pub source_cursor: u64,
    pub built_valid_at: Option<u64>,
    pub artifact_rows: Option<u64>,
    /// Identity encoding of the derived rows, absent for older artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_identity_version: Option<u16>,
    /// The stored index is not eligible until rebuilt with current identities.
    #[serde(default)]
    pub identity_rebuild_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analytics_total_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analytics_group_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintenance: Option<QueryIndexMaintenanceSnapshot>,
    pub configuration_sha256: String,
    pub artifact_sha256: String,
    pub state: QueryIndexState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryIndexMaintenanceSnapshot {
    pub mode: String,
    pub prior_source_cursor: Option<u64>,
    pub source_cursor: u64,
    pub inserted_rows: u64,
    pub updated_rows: u64,
    pub removed_rows: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryIndexCatalogueSnapshot {
    pub scope: String,
    pub revision: u64,
    pub indexes: Vec<QueryIndexSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureQueryIndexResult {
    pub index: QueryIndexSnapshot,
    pub catalogue_revision: u64,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub idempotent_replay: bool,
}
