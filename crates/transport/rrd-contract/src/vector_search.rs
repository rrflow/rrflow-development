//! Vector payload filters, vector search requests and their results.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "operator", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorPayloadOperator {
    Equals {
        value: QueryValue,
    },
    NotEquals {
        value: QueryValue,
    },
    In {
        values: Vec<QueryValue>,
    },
    Range {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gt: Option<QueryValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gte: Option<QueryValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lt: Option<QueryValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lte: Option<QueryValue>,
    },
    Exists {
        value: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorPayloadCondition {
    pub property: CanonicalId,
    pub operator: VectorPayloadOperator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorPayloadFilter {
    Condition { condition: VectorPayloadCondition },
    All { filters: Vec<VectorPayloadFilter> },
    Any { filters: Vec<VectorPayloadFilter> },
    Not { filter: Box<VectorPayloadFilter> },
}

impl VectorPayloadFilter {
    pub fn validate(&self) -> Result<()> {
        let mut nodes = 0_usize;
        self.validate_at(0, &mut nodes)
    }

    fn validate_at(&self, depth: usize, nodes: &mut usize) -> Result<()> {
        *nodes += 1;
        if depth > 32 || *nodes > 4_096 {
            return invalid("vector payload filter exceeds depth or node bounds");
        }
        match self {
            Self::Condition { condition } => match &condition.operator {
                VectorPayloadOperator::In { values } if values.is_empty() => {
                    invalid("vector payload in filter requires values")
                }
                VectorPayloadOperator::Range { gt, gte, lt, lte } => {
                    if (gt.is_some() && gte.is_some())
                        || (lt.is_some() && lte.is_some())
                        || (gt.is_none() && gte.is_none() && lt.is_none() && lte.is_none())
                    {
                        return invalid("vector payload range bounds are invalid");
                    }
                    Ok(())
                }
                _ => Ok(()),
            },
            Self::All { filters } | Self::Any { filters } => {
                if filters.is_empty() {
                    return invalid("vector payload all/any filter requires children");
                }
                for filter in filters {
                    filter.validate_at(depth + 1, nodes)?;
                }
                Ok(())
            }
            Self::Not { filter } => filter.validate_at(depth + 1, nodes),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorSearchQuery {
    Dense {
        values: Vec<f32>,
    },
    Sparse {
        dimensions: u32,
        indices: Vec<u32>,
        values: Vec<f32>,
    },
    MultiDense {
        dimensions: u32,
        vectors: Vec<Vec<f32>>,
        comparator: MultiVectorComparator,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorSearchMode {
    #[default]
    Exact,
    AllowApproximate {
        exact_rerank: u64,
        ef_search: u64,
    },
    RequireApproximate {
        exact_rerank: u64,
        ef_search: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchVectors {
    pub scope: String,
    pub valid_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<CanonicalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_name: Option<CanonicalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<CanonicalId>,
    pub query: VectorSearchQuery,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<VectorPayloadFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<VectorSearchMetric>,
    pub top_k: u64,
    #[serde(default)]
    pub mode: VectorSearchMode,
    pub max_storage_keys: u64,
}

impl SearchVectors {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        let collection_address = self.collection_id.is_some() && self.vector_name.is_some();
        let field_address = self.field.is_some() && self.metric.is_some();
        if collection_address == field_address
            || self.collection_id.is_some() != self.vector_name.is_some()
            || self.field.is_some() != self.metric.is_some()
        {
            return invalid(
                "vector search must use exactly one complete collection/vector or field/metric address",
            );
        }
        if self.valid_at == 0 {
            return invalid("vector search valid_at must be greater than zero");
        }
        if self.top_k == 0 || self.top_k > MAX_VECTOR_SEARCH_TOP_K {
            return invalid(format!(
                "vector search top_k must be in 1..={MAX_VECTOR_SEARCH_TOP_K}"
            ));
        }
        match self.mode {
            VectorSearchMode::Exact => {}
            VectorSearchMode::AllowApproximate {
                exact_rerank,
                ef_search,
            }
            | VectorSearchMode::RequireApproximate {
                exact_rerank,
                ef_search,
            } => {
                if exact_rerank < self.top_k
                    || exact_rerank > MAX_VECTOR_SEARCH_WORK
                    || ef_search < exact_rerank
                    || ef_search > MAX_VECTOR_SEARCH_WORK
                {
                    return invalid(
                        "approximate vector search requires top_k <= exact_rerank <= ef_search within the search bound",
                    );
                }
            }
        }
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_VECTOR_STORAGE_KEYS {
            return invalid(format!(
                "vector search max_storage_keys must be in 1..={MAX_VECTOR_STORAGE_KEYS}"
            ));
        }
        let vector = match &self.query {
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
        validate_data_vector(&vector)?;
        if let Some(filter) = &self.filter {
            filter.validate()?;
        }
        if self.metric == Some(VectorSearchMetric::Cosine) {
            let zero = match &self.query {
                VectorSearchQuery::Dense { values } | VectorSearchQuery::Sparse { values, .. } => {
                    values.iter().all(|value| *value == 0.0)
                }
                VectorSearchQuery::MultiDense { vectors, .. } => vectors
                    .iter()
                    .any(|values| values.iter().all(|value| *value == 0.0)),
            };
            if zero {
                return invalid("cosine vector search rejects zero-norm query rows");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorSearchHit {
    pub reference: DataReference,
    pub subject: DataReference,
    pub source_cursor: u64,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorSearchResult {
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<CanonicalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_name: Option<CanonicalId>,
    pub read_manifest_sha256: String,
    pub known_at_cursor: u64,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub plan_sha256: String,
    pub access_path: CanonicalId,
    pub exact: bool,
    pub resources: VectorSearchResourceEvidence,
    pub hits: Vec<VectorSearchHit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorSearchResourceEvidence {
    pub canonical_candidates: u64,
    pub selected_candidates: u64,
    pub ef_search: u64,
    pub exact_rerank: u64,
    pub overlay_candidates: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_generation: Option<u64>,
    pub loaded_artifact_bytes: u64,
    pub result_hits: u64,
    #[serde(default)]
    pub cold_source_hits: u64,
    #[serde(default)]
    pub rrflowcld_triggered: bool,
}
