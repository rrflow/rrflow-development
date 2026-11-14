//! Vector collections, payload indexes, vector indexes and quantization artifacts.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VectorSearchMetric {
    Cosine,
    Dot,
    Euclidean,
    Manhattan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MultiVectorComparator {
    MaxSim,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VectorValueKind {
    Dense,
    Sparse,
    MultiDense,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VectorMemoryTier {
    Pinned,
    Cached,
    Cold,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorEmbeddingModel {
    pub name: String,
    pub digest: String,
}

impl VectorEmbeddingModel {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.name.len() > MAX_MESSAGE_BYTES
            || self.name.contains('\0')
        {
            return invalid("vector embedding model name is invalid");
        }
        validate_sha256(&self.digest, "vector embedding model digest")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NamedVectorDefinition {
    pub name: CanonicalId,
    pub field: CanonicalId,
    pub kind: VectorValueKind,
    pub dimensions: u32,
    pub metric: VectorSearchMetric,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<VectorEmbeddingModel>,
    pub memory_tier: VectorMemoryTier,
}

impl NamedVectorDefinition {
    fn validate(&self) -> Result<()> {
        if self.dimensions == 0 || u64::from(self.dimensions) > MAX_VECTOR_DIMENSIONS as u64 {
            return invalid(format!(
                "named-vector dimensions must be in 1..={MAX_VECTOR_DIMENSIONS}"
            ));
        }
        if let Some(model) = &self.embedding_model {
            model.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureVectorCollection {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub vectors: Vec<NamedVectorDefinition>,
}

impl EnsureVectorCollection {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.vectors.is_empty() || self.vectors.len() > 64 {
            return invalid("vector collection must contain 1..=64 named vectors");
        }
        let mut names = BTreeSet::new();
        let mut fields = BTreeSet::new();
        for vector in &self.vectors {
            vector.validate()?;
            if !names.insert(vector.name.as_str()) || !fields.insert(vector.field.as_str()) {
                return invalid("vector collection names and fields must be unique");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListVectorCollections {
    pub scope: String,
}

impl ListVectorCollections {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorCollectionSnapshot {
    pub collection_id: CanonicalId,
    pub vectors: Vec<NamedVectorDefinition>,
    #[serde(default)]
    pub payload_indexes: Vec<VectorPayloadIndexSnapshot>,
    pub generation: u64,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub configuration_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorCollectionCatalogueSnapshot {
    pub scope: String,
    pub revision: u64,
    pub collections: Vec<VectorCollectionSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureVectorCollectionResult {
    pub collection: VectorCollectionSnapshot,
    pub catalogue_revision: u64,
    pub idempotent_replay: bool,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VectorPayloadIndexKind {
    Boolean,
    Integer,
    Unsigned,
    Decimal,
    Keyword,
    Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorPayloadIndexSnapshot {
    pub field: CanonicalId,
    pub kind: VectorPayloadIndexKind,
    pub generation: u64,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub configuration_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureVectorPayloadIndex {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub field: CanonicalId,
    pub kind: VectorPayloadIndexKind,
}

impl EnsureVectorPayloadIndex {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteVectorPayloadIndex {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub field: CanonicalId,
}

impl DeleteVectorPayloadIndex {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListVectorPayloadIndexes {
    pub scope: String,
    pub collection_id: CanonicalId,
}

impl ListVectorPayloadIndexes {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorPayloadIndexCatalogueSnapshot {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub collection_generation: u64,
    pub catalogue_revision: u64,
    pub indexes: Vec<VectorPayloadIndexSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureVectorPayloadIndexResult {
    pub collection: VectorCollectionSnapshot,
    pub index: VectorPayloadIndexSnapshot,
    pub catalogue_revision: u64,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteVectorPayloadIndexResult {
    pub collection: VectorCollectionSnapshot,
    pub deleted_index: VectorPayloadIndexSnapshot,
    pub catalogue_revision: u64,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteVectorCollection {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub valid_at: u64,
    pub max_storage_keys: u64,
}

impl DeleteVectorCollection {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.valid_at == 0 {
            return invalid("vector collection delete valid_at must be greater than zero");
        }
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_VECTOR_STORAGE_KEYS {
            return invalid(format!(
                "vector collection delete max_storage_keys must be in 1..={MAX_VECTOR_STORAGE_KEYS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteVectorCollectionResult {
    pub deleted_collection: VectorCollectionSnapshot,
    pub catalogue_revision: u64,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorIndexConfiguration {
    Hnsw {
        m: u32,
        ef_construction: u64,
        max_level: u8,
        seed: u64,
        #[serde(default)]
        filter_properties: Vec<CanonicalId>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VectorQuantizationBits {
    Bits4,
    Bits2,
    Bits1_5,
    Bits1,
}

impl VectorIndexConfiguration {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Hnsw {
                m,
                ef_construction,
                max_level,
                filter_properties,
                ..
            } => {
                if !(2..=128).contains(m)
                    || *ef_construction < u64::from(*m)
                    || *ef_construction > 1_000_000
                    || !(1..=32).contains(max_level)
                {
                    return invalid("HNSW index parameters are outside supported bounds");
                }
                let unique = filter_properties
                    .iter()
                    .map(CanonicalId::as_str)
                    .collect::<BTreeSet<_>>();
                if unique.len() != filter_properties.len() {
                    return invalid("HNSW filter properties must be unique");
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorIndexBuildResourceEvidence {
    pub input_vectors: u64,
    pub dimensions: u64,
    pub input_values: u64,
    pub cpu_artifact_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accelerator_artifact_bytes: Option<u64>,
    pub semantic_probe_queries: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VectorIndexDifferentialStatus {
    NotRun,
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorIndexBuildTarget {
    Cpu,
    Gpu { platform: String, device: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorIndexBuildEvidence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_backend_id: Option<CanonicalId>,
    pub selected_backend_id: String,
    pub selected_target: VectorIndexBuildTarget,
    pub used_fallback: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub byte_differential: VectorIndexDifferentialStatus,
    pub semantic_differential: VectorIndexDifferentialStatus,
    pub resources: VectorIndexBuildResourceEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorIndexBuildPolicy {
    #[default]
    Cpu,
    PreferGpu {
        backend_id: CanonicalId,
        allow_cpu_fallback: bool,
    },
    RequireGpu {
        backend_id: CanonicalId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureVectorIndex {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub configuration: VectorIndexConfiguration,
    #[serde(default)]
    pub build_policy: VectorIndexBuildPolicy,
    pub max_storage_keys: u64,
}

impl EnsureVectorIndex {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        self.configuration.validate()?;
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_VECTOR_STORAGE_KEYS {
            return invalid(format!(
                "vector index max_storage_keys must be in 1..={MAX_VECTOR_STORAGE_KEYS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorIndexMaintenanceSnapshot {
    pub mode: VectorIndexMaintenanceMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_generation: Option<u64>,
    pub indexed_delta_vectors: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VectorIndexMaintenanceMode {
    FullBuild,
    Incremental,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorIndexSnapshot {
    pub index_id: CanonicalId,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub kind: CanonicalId,
    pub generation: u64,
    pub source_cursor: u64,
    pub indexed_vectors: u64,
    pub maintenance: VectorIndexMaintenanceSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packed_vector_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_precision_vector_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_evidence: Option<VectorIndexBuildEvidence>,
    pub configuration_sha256: String,
    pub artifact_sha256: String,
    pub object_sha256: String,
    pub catalogue_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnsureVectorIndexResult {
    pub index: VectorIndexSnapshot,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VectorProductCompression {
    X4,
    X8,
    X16,
    X32,
    X64,
}

impl VectorProductCompression {
    pub const fn ratio(self) -> u64 {
        match self {
            Self::X4 => 4,
            Self::X8 => 8,
            Self::X16 => 16,
            Self::X32 => 32,
            Self::X64 => 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorQuantizationMethod {
    Scalar,
    Product {
        compression: VectorProductCompression,
    },
    Binary,
    TurboQuant {
        bits: VectorQuantizationBits,
        seed: u64,
        /// Absent means `mse`, as before the inner-product estimator existed.
        #[serde(default, skip_serializing_if = "VectorQuantizationEstimator::is_mse")]
        estimator: VectorQuantizationEstimator,
    },
}

impl VectorQuantizationMethod {
    pub const fn maximum_compression_ratio(&self) -> u64 {
        match self {
            Self::Scalar => 4,
            Self::Product { compression } => compression.ratio(),
            Self::Binary
            | Self::TurboQuant {
                bits: VectorQuantizationBits::Bits1,
                ..
            } => 32,
            Self::TurboQuant {
                bits: VectorQuantizationBits::Bits1_5,
                ..
            } => 21,
            Self::TurboQuant {
                bits: VectorQuantizationBits::Bits2,
                ..
            } => 16,
            Self::TurboQuant {
                bits: VectorQuantizationBits::Bits4,
                ..
            } => 8,
        }
    }
}

/// How a TurboQuant generation estimates a query's inner products.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VectorQuantizationEstimator {
    /// TurboQuant_mse: each coordinate's nearest codebook value, rescaled to
    /// the vector's length.
    #[default]
    Mse,
    /// TurboQuant_prod: an MSE stage one bit narrower plus one sign bit per
    /// coordinate of its residual's Gaussian projection, an unbiased estimate
    /// of inner products.
    InnerProduct,
}

impl VectorQuantizationEstimator {
    pub fn is_mse(&self) -> bool {
        *self == Self::Mse
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VectorQuantizationArtifactState {
    Ready,
    Active,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildVectorQuantizationArtifact {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub method: VectorQuantizationMethod,
    #[serde(default)]
    pub filter_properties: Vec<CanonicalId>,
    pub max_storage_keys: u64,
}

impl BuildVectorQuantizationArtifact {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if let VectorQuantizationMethod::TurboQuant {
            bits: VectorQuantizationBits::Bits1_5,
            estimator: VectorQuantizationEstimator::InnerProduct,
            ..
        } = self.method
        {
            return invalid("1.5-bit TurboQuant has no inner-product estimator");
        }
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_VECTOR_STORAGE_KEYS {
            return invalid(format!(
                "quantization build max_storage_keys must be in 1..={MAX_VECTOR_STORAGE_KEYS}"
            ));
        }
        let unique = self
            .filter_properties
            .iter()
            .map(CanonicalId::as_str)
            .collect::<BTreeSet<_>>();
        if unique.len() != self.filter_properties.len() {
            return invalid("quantization filter properties must be unique");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListVectorQuantizationArtifacts {
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<CanonicalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_name: Option<CanonicalId>,
    pub max_artifacts: u64,
}

impl ListVectorQuantizationArtifacts {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.max_artifacts == 0 || self.max_artifacts > 100_000 {
            return invalid("quantization list max_artifacts must be in 1..=100000");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActivateVectorQuantizationArtifact {
    pub scope: String,
    pub artifact_id: CanonicalId,
    pub generation: u64,
}

impl ActivateVectorQuantizationArtifact {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.generation == 0 {
            return invalid("quantization activation generation must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetireVectorQuantizationArtifact {
    pub scope: String,
    pub artifact_id: CanonicalId,
    pub generation: u64,
}

impl RetireVectorQuantizationArtifact {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.generation == 0 {
            return invalid("quantization retirement generation must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorQuantizationArtifactSnapshot {
    pub artifact_id: CanonicalId,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub method: VectorQuantizationMethod,
    pub state: VectorQuantizationArtifactState,
    pub generation: u64,
    pub source_cursor: u64,
    pub indexed_vectors: u64,
    pub packed_vector_bytes: u64,
    pub full_precision_vector_bytes: u64,
    pub auxiliary_bytes: u64,
    pub maximum_compression_ratio: u64,
    pub configuration_sha256: String,
    pub artifact_sha256: String,
    pub object_sha256: String,
    pub object_length: u64,
    pub built_at_unix_ms: u64,
    pub lifecycle_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildVectorQuantizationArtifactResult {
    pub artifact: VectorQuantizationArtifactSnapshot,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    /// The generation this build extended with the vectors changed since it;
    /// absent when the build encoded every version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_generation: Option<u64>,
    /// Vector versions this build encoded. An extended generation carries the
    /// rest from its base; a replay encodes none.
    pub encoded_versions: u64,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListVectorQuantizationArtifactsResult {
    pub scope: String,
    pub lifecycle_revision: u64,
    pub artifacts: Vec<VectorQuantizationArtifactSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActivateVectorQuantizationArtifactResult {
    pub artifact: VectorQuantizationArtifactSnapshot,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetireVectorQuantizationArtifactResult {
    pub artifact: VectorQuantizationArtifactSnapshot,
    pub idempotent_replay: bool,
}
