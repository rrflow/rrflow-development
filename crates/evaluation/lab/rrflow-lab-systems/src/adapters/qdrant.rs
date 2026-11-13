//! Qdrant over its public REST API, as probed on v1.20.0.
use crate::adapter::{
    describe, wait_until, CollectionSpec, ConnectionLimits, Http, PointBatch, SearchRequest,
    SystemAdapter,
};
use crate::binding::{IndexSpec, SystemBinding};
use rrflow_evaluation_core::LabMetric;
use rrflow_evaluation_core::Result;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::Instant;

pub struct QdrantAdapter {
    http: Http,
    collection: OnceLock<String>,
    exact: OnceLock<bool>,
}

impl QdrantAdapter {
    pub fn new(binding: &SystemBinding, limits: &ConnectionLimits) -> Self {
        Self {
            http: Http::new(&binding.endpoint, limits, Vec::new()),
            collection: OnceLock::new(),
            exact: OnceLock::new(),
        }
    }

    fn path(&self, suffix: &str) -> Result<String> {
        let name = self.collection.get().ok_or("collection is not prepared")?;
        Ok(format!("/collections/{name}{suffix}"))
    }
}

/// Status alone is not enough: Qdrant reports green during a gap between optimizer
/// tasks, so readiness needs several consecutive quiet observations.
const QUIET_OBSERVATIONS: u32 = 8;

impl SystemAdapter for QdrantAdapter {
    fn observed_version(&self) -> Result<String> {
        let value = self.http.json("GET", "/", None)?;
        value["version"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("no version in {}", describe(&value)))
    }

    fn prepare(&self, spec: &CollectionSpec<'_>) -> Result<()> {
        let _ = self.collection.set(spec.name.to_string());
        let _ = self.exact.set(matches!(spec.index, IndexSpec::Exact));
        let path = self.path("")?;
        self.http.json("DELETE", &path, None)?;
        let distance = match spec.metric {
            LabMetric::Cosine => "Cosine",
            LabMetric::Dot => "Dot",
            LabMetric::Euclidean => "Euclid",
        };
        let mut body = json!({"vectors": {"size": spec.dimensions, "distance": distance}});
        if let IndexSpec::Hnsw { m, ef_construction } = spec.index {
            body["hnsw_config"] = json!({"m": m, "ef_construct": ef_construction});
        }
        self.http.json("PUT", &path, Some(&body))?;
        self.http.json(
            "PUT",
            &self.path("/index?wait=true")?,
            Some(&json!({"field_name": "category", "field_schema": "integer"})),
        )?;
        Ok(())
    }

    fn ingest(&self, batch: &PointBatch<'_>) -> Result<()> {
        let points: Vec<Value> = (0..batch.ids.len())
            .map(|index| {
                json!({
                    "id": batch.ids[index],
                    "vector": batch.vector(index),
                    "payload": {"category": batch.categories[index]}
                })
            })
            .collect();
        self.http.json(
            "PUT",
            &self.path("/points?wait=true")?,
            Some(&json!({"points": points})),
        )?;
        Ok(())
    }

    fn finish_ingest(&self, deadline: Instant) -> Result<()> {
        let path = self.path("")?;
        let mut quiet = 0;
        wait_until(deadline, "Qdrant indexing", || {
            let value = self.http.json("GET", &path, None)?;
            let result = &value["result"];
            let settled = result["status"] == "green" && result["optimizer_status"] == "ok";
            quiet = if settled { quiet + 1 } else { 0 };
            Ok(quiet >= QUIET_OBSERVATIONS)
        })
    }

    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        let mut params = json!({});
        if *self.exact.get().unwrap_or(&false) {
            params["exact"] = json!(true);
        } else if let Some(ef) = request.setting.ef {
            params["hnsw_ef"] = json!(ef);
        }
        let mut body = json!({
            "query": request.vector,
            "limit": request.k,
            "params": params,
            "with_payload": false
        });
        if let Some(category) = request.category {
            body["filter"] = json!({"must": [{"key": "category", "match": {"value": category}}]});
        }
        let value = self
            .http
            .json("POST", &self.path("/points/query")?, Some(&body))?;
        let points = value["result"]["points"]
            .as_array()
            .ok_or_else(|| format!("no points in {}", describe(&value)))?;
        points
            .iter()
            .map(|point| {
                point["id"]
                    .as_u64()
                    .ok_or_else(|| format!("invalid point id in {}", describe(point)))
            })
            .collect()
    }

    fn index_evidence(&self) -> Result<Option<String>> {
        let value = self.http.json("GET", &self.path("")?, None)?;
        let result = &value["result"];
        Ok(Some(format!(
            "{} of {} points indexed across {} segments",
            result["indexed_vectors_count"], result["points_count"], result["segments_count"]
        )))
    }

    fn teardown(&self) -> Result<()> {
        self.http.json("DELETE", &self.path("")?, None)?;
        Ok(())
    }
}
