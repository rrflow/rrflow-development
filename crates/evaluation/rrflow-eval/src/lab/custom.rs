//! Any other system joins through this small HTTP protocol, usually behind a thin
//! shim. The types below own the protocol; its schema is generated from them.
use super::adapter::{CollectionSpec, PointBatch, SearchRequest, SystemAdapter};
use super::manifest::{IndexSpec, LabMetric, SearchSetting, SystemBinding, WorkloadSpec};
use super::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub const CUSTOM_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomHealth {
    pub protocol_version: u16,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomPrepare {
    pub name: String,
    pub dimensions: u32,
    pub metric: LabMetric,
    pub categories: u32,
    pub index: IndexSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomIngest {
    pub ids: Vec<u64>,
    pub vectors: Vec<Vec<f32>>,
    pub categories: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomReady {
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomSearch {
    pub vector: Vec<f32>,
    pub k: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<u32>,
    pub setting: SearchSetting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomSearchResult {
    pub ids: Vec<u64>,
}

/// The whole protocol in one generated schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomProtocol {
    pub health_response: CustomHealth,
    pub prepare_request: CustomPrepare,
    pub ingest_request: CustomIngest,
    pub finish_response: CustomReady,
    pub search_request: CustomSearch,
    pub search_response: CustomSearchResult,
}

pub struct CustomAdapter {
    _disabled: (),
}

impl CustomAdapter {
    pub fn new(_binding: &SystemBinding, _workload: &WorkloadSpec) -> Self {
        Self { _disabled: () }
    }
}

impl SystemAdapter for CustomAdapter {
    fn observed_version(&self) -> Result<String> {
        Err(ownership_error())
    }

    fn prepare(&self, _spec: &CollectionSpec<'_>) -> Result<()> {
        Err(ownership_error())
    }

    fn ingest(&self, _batch: &PointBatch<'_>) -> Result<()> {
        Err(ownership_error())
    }

    fn finish_ingest(&self, _deadline: Instant) -> Result<()> {
        Err(ownership_error())
    }

    fn search(&self, _request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        Err(ownership_error())
    }

    fn teardown(&self) -> Result<()> {
        Ok(())
    }
}

fn ownership_error() -> String {
    "custom protocol v1 has no scoped resource ownership; adapter disabled".into()
}
