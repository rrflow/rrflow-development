//! Any other system joins through this small HTTP protocol, usually behind a thin
//! shim. The types below own the protocol; its schema is generated from them.
use super::adapter::{wait_until, CollectionSpec, Http, PointBatch, SearchRequest, SystemAdapter};
use super::manifest::{IndexSpec, LabMetric, SearchSetting, SystemBinding, WorkloadSpec};
use super::Result;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
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

/// What served the searches, in the system's own words. A system without an answer
/// replies 404 to this route, and the report then shows no evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomEvidence {
    pub evidence: String,
}

/// The whole protocol in one generated schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CustomProtocol {
    pub evidence_response: CustomEvidence,
    pub health_response: CustomHealth,
    pub prepare_request: CustomPrepare,
    pub ingest_request: CustomIngest,
    pub finish_response: CustomReady,
    pub search_request: CustomSearch,
    pub search_response: CustomSearchResult,
}

pub struct CustomAdapter {
    http: Http,
}

impl CustomAdapter {
    pub fn new(binding: &SystemBinding, workload: &WorkloadSpec) -> Self {
        Self {
            http: Http::new(&binding.endpoint, workload, Vec::new()),
        }
    }

    fn decode<T: DeserializeOwned>(value: Value) -> Result<T> {
        serde_json::from_value(value).map_err(|error| format!("protocol violation: {error}"))
    }

    fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T> {
        let body = serde_json::to_value(body).map_err(|error| error.to_string())?;
        Self::decode(self.http.json("POST", path, Some(&body))?)
    }
}

impl SystemAdapter for CustomAdapter {
    fn observed_version(&self) -> Result<String> {
        let health: CustomHealth = Self::decode(self.http.json("GET", "/health", None)?)?;
        if health.protocol_version != CUSTOM_PROTOCOL_VERSION {
            return Err(format!(
                "custom system speaks protocol {}, this harness speaks {CUSTOM_PROTOCOL_VERSION}",
                health.protocol_version
            ));
        }
        Ok(format!("{} {}", health.name, health.version))
    }

    fn prepare(&self, spec: &CollectionSpec<'_>) -> Result<()> {
        let _: Value = self.post(
            "/prepare",
            &CustomPrepare {
                name: spec.name.to_string(),
                dimensions: spec.dimensions,
                metric: spec.metric,
                categories: spec.categories,
                index: spec.index.clone(),
            },
        )?;
        Ok(())
    }

    fn ingest(&self, batch: &PointBatch<'_>) -> Result<()> {
        let _: Value = self.post(
            "/ingest",
            &CustomIngest {
                ids: batch.ids.to_vec(),
                vectors: (0..batch.ids.len())
                    .map(|index| batch.vector(index).to_vec())
                    .collect(),
                categories: batch.categories.to_vec(),
            },
        )?;
        Ok(())
    }

    fn finish_ingest(&self, deadline: Instant) -> Result<()> {
        wait_until(deadline, "custom system", || {
            let ready: CustomReady = self.post("/finish", &json!({}))?;
            Ok(ready.ready)
        })
    }

    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        let result: CustomSearchResult = self.post(
            "/search",
            &CustomSearch {
                vector: request.vector.to_vec(),
                k: request.k,
                category: request.category,
                setting: request.setting.clone(),
            },
        )?;
        Ok(result.ids)
    }

    fn index_evidence(&self) -> Result<Option<String>> {
        let (status, text) = self.http.raw("GET", "/evidence", None)?;
        if status == 404 {
            return Ok(None);
        }
        if !(200..300).contains(&status) {
            return Err(format!("GET /evidence: HTTP {status}: {text}"));
        }
        let value =
            serde_json::from_str(&text).map_err(|error| format!("protocol violation: {error}"))?;
        let evidence: CustomEvidence = Self::decode(value)?;
        Ok(Some(evidence.evidence))
    }

    fn teardown(&self) -> Result<()> {
        let _: Value = self.post("/teardown", &json!({}))?;
        Ok(())
    }
}
