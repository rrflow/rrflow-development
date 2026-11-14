//! Vector point scroll, retrieve and batch.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScrollVectorPoints {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub valid_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_reference: Option<DataReference>,
    pub limit: u64,
    pub max_storage_keys: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<VectorPayloadFilter>,
}

impl ScrollVectorPoints {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.valid_at == 0 {
            return invalid("vector point scroll valid_at must be greater than zero");
        }
        if self.limit == 0 || self.limit > MAX_VECTOR_POINT_PAGE {
            return invalid(format!(
                "vector point scroll limit must be in 1..={MAX_VECTOR_POINT_PAGE}"
            ));
        }
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_VECTOR_STORAGE_KEYS {
            return invalid(format!(
                "vector point scroll max_storage_keys must be in 1..={MAX_VECTOR_STORAGE_KEYS}"
            ));
        }
        if let Some(filter) = &self.filter {
            filter.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorPointSnapshot {
    pub reference: DataReference,
    pub subject: DataReference,
    pub source_cursor: u64,
    pub value: DataVectorValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<DataEmbeddingProvenance>,
    pub payload: DataProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorPointPage {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub read_manifest_sha256: String,
    pub known_at_cursor: u64,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub points: Vec<VectorPointSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_after: Option<DataReference>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrieveVectorPoints {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub valid_at: u64,
    pub references: Vec<DataReference>,
    pub max_storage_keys: u64,
}

impl RetrieveVectorPoints {
    pub fn validate(&self) -> Result<()> {
        validate_vector_scope(&self.scope)?;
        if self.valid_at == 0 {
            return invalid("vector point retrieve valid_at must be greater than zero");
        }
        if self.references.is_empty() || self.references.len() > MAX_VECTOR_POINT_PAGE as usize {
            return invalid(format!(
                "vector point retrieve references must contain 1..={MAX_VECTOR_POINT_PAGE} identities"
            ));
        }
        if self.references.iter().collect::<BTreeSet<_>>().len() != self.references.len() {
            return invalid("vector point retrieve references must be unique");
        }
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_VECTOR_STORAGE_KEYS {
            return invalid(format!(
                "vector point retrieve max_storage_keys must be in 1..={MAX_VECTOR_STORAGE_KEYS}"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorPointBatch {
    pub scope: String,
    pub collection_id: CanonicalId,
    pub vector_name: CanonicalId,
    pub read_manifest_sha256: String,
    pub known_at_cursor: u64,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub points: Vec<VectorPointSnapshot>,
    pub missing: Vec<DataReference>,
}

pub(crate) fn validate_vector_scope(scope: &str) -> Result<()> {
    if scope.is_empty() || scope.len() > MAX_ID_BYTES || scope.as_bytes().contains(&0) {
        return invalid("vector scope is invalid");
    }
    Ok(())
}
