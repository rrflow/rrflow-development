//! Hybrid search and the composable retrieval query.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HybridFusion {
    ReciprocalRank {
        rank_constant: u32,
        text_weight_millionths: u32,
        vector_weight_millionths: u32,
    },
}

impl Default for HybridFusion {
    fn default() -> Self {
        Self::ReciprocalRank {
            rank_constant: 60,
            text_weight_millionths: 1_000_000,
            vector_weight_millionths: 1_000_000,
        }
    }
}

impl HybridFusion {
    fn validate(self) -> Result<()> {
        match self {
            Self::ReciprocalRank {
                rank_constant,
                text_weight_millionths,
                vector_weight_millionths,
            } => {
                if !(1..=10_000).contains(&rank_constant)
                    || !(1..=1_000_000).contains(&text_weight_millionths)
                    || !(1..=1_000_000).contains(&vector_weight_millionths)
                {
                    return invalid(
                        "hybrid reciprocal-rank parameters exceed their supported bounds",
                    );
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchHybrid {
    pub scope: String,
    pub valid_at: u64,
    pub document_kind: CanonicalId,
    pub text_field: CanonicalId,
    pub text_query: String,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub vector_query: VectorSearchQuery,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_filter: Option<VectorPayloadFilter>,
    #[serde(default)]
    pub vector_mode: VectorSearchMode,
    #[serde(default)]
    pub fusion: HybridFusion,
    pub top_k: u64,
    pub candidate_k: u64,
    pub max_storage_keys: u64,
}

impl SearchHybrid {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.valid_at == 0 {
            return invalid("hybrid search valid_at must be greater than zero");
        }
        if self.text_query.trim().is_empty() || self.text_query.len() > MAX_MESSAGE_BYTES {
            return invalid(format!(
                "hybrid text query length must be in 1..={MAX_MESSAGE_BYTES} bytes"
            ));
        }
        if self.top_k == 0
            || self.top_k > MAX_VECTOR_SEARCH_TOP_K
            || self.candidate_k < self.top_k
            || self.candidate_k > MAX_VECTOR_SEARCH_TOP_K
        {
            return invalid(
                "hybrid search requires 1 <= top_k <= candidate_k within the vector result bound",
            );
        }
        self.fusion.validate()?;
        SearchVectors {
            scope: self.scope.clone(),
            valid_at: self.valid_at,
            collection_id: Some(self.collection_id.clone()),
            vector_name: Some(self.vector_name.clone()),
            field: None,
            query: self.vector_query.clone(),
            filter: self.vector_filter.clone(),
            metric: None,
            top_k: self.candidate_k,
            mode: self.vector_mode,
            max_storage_keys: self.max_storage_keys,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HybridSearchHit {
    pub subject: DataReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_reference: Option<DataReference>,
    pub fused_score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_rank: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_rank: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HybridSearchResult {
    pub scope: String,
    pub read_manifest_sha256: String,
    pub known_at_cursor: u64,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub text_plan_sha256: String,
    pub vector_plan_sha256: String,
    pub fusion_plan_sha256: String,
    pub text_access_path: CanonicalId,
    pub vector_access_path: CanonicalId,
    pub vector_exact: bool,
    pub text_candidates: u64,
    pub vector_candidates: u64,
    pub fusion: HybridFusion,
    pub hits: Vec<HybridSearchHit>,
}

pub(crate) const MAX_RETRIEVAL_QUERY_DEPTH: usize = 8;
pub(crate) const MAX_RETRIEVAL_QUERY_NODES: usize = 128;
pub(crate) const MAX_RETRIEVAL_PREFETCHES: usize = 16;
pub(crate) const MAX_RETRIEVAL_EXAMPLES: usize = 256;
pub(crate) const MAX_RETRIEVAL_RERANK_STAGES: usize = 16;

/// A stored vector version or caller-supplied vector used by recommendation
/// and discovery queries. Stored examples are vector-reference addressed so a
/// point with several named modalities is never ambiguous.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RetrievalVectorExample {
    Reference { reference: DataReference },
    Vector { value: VectorSearchQuery },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalContextPair {
    pub positive: RetrievalVectorExample,
    pub negative: RetrievalVectorExample,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalRecommendStrategy {
    AverageVector,
    BestScore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RetrievalFusion {
    ReciprocalRank {
        rank_constant: u32,
        weights_millionths: Vec<u32>,
    },
}

impl RetrievalFusion {
    fn validate(&self, branches: usize) -> Result<()> {
        match self {
            Self::ReciprocalRank {
                rank_constant,
                weights_millionths,
            } => {
                if !(1..=10_000).contains(rank_constant)
                    || weights_millionths.len() != branches
                    || weights_millionths
                        .iter()
                        .any(|weight| !(1..=1_000_000).contains(weight))
                {
                    return invalid(
                        "retrieval RRF requires one bounded positive weight per prefetch",
                    );
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RetrievalRerankStage {
    ScoreBoost {
        filter: VectorPayloadFilter,
        add_millionths: i32,
        multiply_millionths: u32,
    },
    Exact {
        using: CanonicalId,
        query: VectorSearchQuery,
    },
    Model {
        using: CanonicalId,
        model: VectorEmbeddingModel,
        query: VectorSearchQuery,
    },
    Mmr {
        using: CanonicalId,
        diversity_millionths: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalPrefetch {
    pub query: Box<RetrievalQuery>,
    pub limit: u64,
}

/// Recursive retrieval algebra. Leaf sources read one authoritative snapshot;
/// fusion and reranking consume only their declared bounded prefetch results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RetrievalQuery {
    Nearest {
        using: CanonicalId,
        query: VectorSearchQuery,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<VectorPayloadFilter>,
        #[serde(default)]
        mode: VectorSearchMode,
    },
    Keyword {
        document_kind: CanonicalId,
        text_field: CanonicalId,
        query: String,
    },
    Recommend {
        using: CanonicalId,
        positive: Vec<RetrievalVectorExample>,
        #[serde(default)]
        negative: Vec<RetrievalVectorExample>,
        strategy: RetrievalRecommendStrategy,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<VectorPayloadFilter>,
    },
    Discover {
        using: CanonicalId,
        target: RetrievalVectorExample,
        context: Vec<RetrievalContextPair>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<VectorPayloadFilter>,
    },
    Context {
        using: CanonicalId,
        context: Vec<RetrievalContextPair>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<VectorPayloadFilter>,
    },
    Fusion {
        prefetch: Vec<RetrievalPrefetch>,
        fusion: RetrievalFusion,
    },
    Rerank {
        prefetch: Box<RetrievalPrefetch>,
        stages: Vec<RetrievalRerankStage>,
    },
}

impl RetrievalQuery {
    fn validate_at(
        &self,
        request: &ExecuteRetrievalQuery,
        limit: u64,
        depth: usize,
        nodes: &mut usize,
    ) -> Result<()> {
        *nodes += 1;
        if depth > MAX_RETRIEVAL_QUERY_DEPTH || *nodes > MAX_RETRIEVAL_QUERY_NODES {
            return invalid("retrieval query exceeds its depth or node bound");
        }
        let validate_vector = |using: &CanonicalId,
                               query: &VectorSearchQuery,
                               filter: Option<VectorPayloadFilter>,
                               mode: VectorSearchMode| {
            SearchVectors {
                scope: request.scope.clone(),
                valid_at: request.valid_at,
                collection_id: Some(request.collection_id.clone()),
                vector_name: Some(using.clone()),
                field: None,
                query: query.clone(),
                filter,
                metric: None,
                top_k: limit,
                mode,
                max_storage_keys: request.max_storage_keys,
            }
            .validate()
        };
        let validate_example = validate_retrieval_example;
        let validate_filter = |filter: &Option<VectorPayloadFilter>| {
            if let Some(filter) = filter {
                filter.validate()?;
            }
            Ok(())
        };
        match self {
            Self::Nearest {
                using,
                query,
                filter,
                mode,
            } => validate_vector(using, query, filter.clone(), *mode),
            Self::Keyword { query, .. } => {
                if query.trim().is_empty() || query.len() > MAX_MESSAGE_BYTES {
                    return invalid("retrieval keyword query is empty or exceeds its byte bound");
                }
                Ok(())
            }
            Self::Recommend {
                positive,
                negative,
                filter,
                ..
            } => {
                if positive.is_empty()
                    || positive.len().saturating_add(negative.len()) > MAX_RETRIEVAL_EXAMPLES
                {
                    return invalid(
                        "retrieval recommendation requires bounded positive/negative examples",
                    );
                }
                for example in positive.iter().chain(negative) {
                    validate_example(example)?;
                }
                validate_filter(filter)
            }
            Self::Discover {
                target,
                context,
                filter,
                ..
            } => {
                validate_example(target)?;
                validate_context(context, &validate_example)?;
                validate_filter(filter)
            }
            Self::Context {
                context, filter, ..
            } => {
                validate_context(context, &validate_example)?;
                validate_filter(filter)
            }
            Self::Fusion { prefetch, fusion } => {
                if !(2..=MAX_RETRIEVAL_PREFETCHES).contains(&prefetch.len()) {
                    return invalid("retrieval fusion requires 2..=16 prefetch branches");
                }
                fusion.validate(prefetch.len())?;
                for branch in prefetch {
                    branch.validate_at(request, depth + 1, nodes)?;
                }
                Ok(())
            }
            Self::Rerank { prefetch, stages } => {
                if stages.is_empty() || stages.len() > MAX_RETRIEVAL_RERANK_STAGES {
                    return invalid("retrieval rerank requires 1..=16 governed stages");
                }
                prefetch.validate_at(request, depth + 1, nodes)?;
                for stage in stages {
                    match stage {
                        RetrievalRerankStage::ScoreBoost {
                            filter,
                            add_millionths,
                            multiply_millionths,
                        } => {
                            filter.validate()?;
                            if add_millionths.unsigned_abs() > 10_000_000
                                || *multiply_millionths > 10_000_000
                            {
                                return invalid(
                                    "retrieval score boost exceeds its fixed-point bound",
                                );
                            }
                        }
                        RetrievalRerankStage::Exact { using, query } => {
                            validate_vector(using, query, None, VectorSearchMode::Exact)?;
                        }
                        RetrievalRerankStage::Model {
                            using,
                            model,
                            query,
                        } => {
                            model.validate()?;
                            validate_vector(using, query, None, VectorSearchMode::Exact)?;
                        }
                        RetrievalRerankStage::Mmr {
                            diversity_millionths,
                            ..
                        } if *diversity_millionths > 1_000_000 => {
                            return invalid("retrieval MMR diversity must be in 0..=1000000");
                        }
                        RetrievalRerankStage::Mmr { .. } => {}
                    }
                }
                Ok(())
            }
        }
    }
}

impl RetrievalPrefetch {
    fn validate_at(
        &self,
        request: &ExecuteRetrievalQuery,
        depth: usize,
        nodes: &mut usize,
    ) -> Result<()> {
        if self.limit == 0 || self.limit > request.candidate_limit {
            return invalid("retrieval prefetch limit must be in 1..=candidate_limit");
        }
        self.query.validate_at(request, self.limit, depth, nodes)
    }
}

pub(crate) fn validate_context(
    context: &[RetrievalContextPair],
    validate_example: &dyn Fn(&RetrievalVectorExample) -> Result<()>,
) -> Result<()> {
    if context.is_empty() || context.len().saturating_mul(2) > MAX_RETRIEVAL_EXAMPLES {
        return invalid("retrieval context requires a bounded non-empty pair set");
    }
    for pair in context {
        validate_example(&pair.positive)?;
        validate_example(&pair.negative)?;
    }
    Ok(())
}

pub(crate) fn validate_retrieval_example(example: &RetrievalVectorExample) -> Result<()> {
    let RetrievalVectorExample::Vector { value } = example else {
        return Ok(());
    };
    let value = match value {
        VectorSearchQuery::Dense { values } => DataVectorValue::Dense {
            values: values.clone(),
        },
        VectorSearchQuery::Sparse {
            dimensions,
            indices,
            values,
        } => DataVectorValue::Sparse {
            dimensions: *dimensions,
            indices: indices.clone(),
            values: values.clone(),
        },
        VectorSearchQuery::MultiDense {
            dimensions,
            vectors,
            ..
        } => DataVectorValue::MultiDense {
            dimensions: *dimensions,
            vectors: vectors.clone(),
        },
    };
    validate_data_vector(&value).map(|_| ())
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RetrievalResultShape {
    #[default]
    Points,
    Groups {
        property: CanonicalId,
        max_groups: u64,
        hits_per_group: u64,
    },
    Facets {
        property: CanonicalId,
        limit: u64,
    },
    Matrix {
        using: CanonicalId,
        sample: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecuteRetrievalQuery {
    pub scope: String,
    pub valid_at: u64,
    pub collection_id: CanonicalId,
    pub query: RetrievalQuery,
    #[serde(default)]
    pub result_shape: RetrievalResultShape,
    pub limit: u64,
    pub candidate_limit: u64,
    pub max_storage_keys: u64,
}

impl ExecuteRetrievalQuery {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.valid_at == 0 {
            return invalid("retrieval query valid_at must be greater than zero");
        }
        if self.limit == 0
            || self.limit > MAX_VECTOR_SEARCH_TOP_K
            || self.candidate_limit < self.limit
            || self.candidate_limit > MAX_VECTOR_SEARCH_TOP_K
        {
            return invalid(
                "retrieval query requires 1 <= limit <= candidate_limit within the result bound",
            );
        }
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_VECTOR_STORAGE_KEYS {
            return invalid("retrieval query max_storage_keys exceeds its bound");
        }
        match self.result_shape {
            RetrievalResultShape::Points => {}
            RetrievalResultShape::Groups {
                max_groups,
                hits_per_group,
                ..
            } => {
                if max_groups == 0
                    || hits_per_group == 0
                    || max_groups
                        .checked_mul(hits_per_group)
                        .is_none_or(|rows| rows > self.candidate_limit)
                {
                    return invalid("retrieval group result exceeds its candidate bound");
                }
            }
            RetrievalResultShape::Facets { limit, .. } => {
                if limit == 0 || limit > self.candidate_limit {
                    return invalid("retrieval facet limit exceeds its candidate bound");
                }
            }
            RetrievalResultShape::Matrix { sample, .. } => {
                if !(2..=1_024).contains(&sample) || sample > self.candidate_limit {
                    return invalid("retrieval matrix sample must be in 2..=1024 and bounded");
                }
            }
        }
        let mut nodes = 0;
        self.query
            .validate_at(self, self.candidate_limit, 0, &mut nodes)?;
        if self
            .candidate_limit
            .checked_mul(nodes as u64)
            .is_none_or(|work| work > MAX_VECTOR_SEARCH_WORK)
        {
            return invalid("retrieval query candidate-stage work exceeds its bound");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalContribution {
    pub stage_ordinal: u32,
    pub rank: u64,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalHit {
    pub subject: DataReference,
    #[serde(default)]
    pub vector_references: Vec<DataReference>,
    pub score: f64,
    #[serde(default)]
    pub contributions: Vec<RetrievalContribution>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalGroup {
    pub value: QueryValue,
    pub hits: Vec<RetrievalHit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalFacet {
    pub value: QueryValue,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalMatrixCell {
    pub left: DataReference,
    pub right: DataReference,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RetrievalOutput {
    Points {
        hits: Vec<RetrievalHit>,
    },
    Groups {
        groups: Vec<RetrievalGroup>,
    },
    Facets {
        facets: Vec<RetrievalFacet>,
    },
    Matrix {
        subjects: Vec<DataReference>,
        cells: Vec<RetrievalMatrixCell>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalStageEvidence {
    pub ordinal: u32,
    pub kind: CanonicalId,
    pub input_candidates: u64,
    pub output_candidates: u64,
    pub exact: bool,
    pub plan_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalQueryResult {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub read_manifest_sha256: String,
    pub known_at_cursor: u64,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub query_plan_sha256: String,
    pub stages: Vec<RetrievalStageEvidence>,
    pub output: RetrievalOutput,
}
