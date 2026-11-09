//! Milvus over its public REST API, as probed on 3.0.5.
use super::adapter::{
    describe, wait_until, CollectionSpec, Http, PointBatch, SearchRequest, SystemAdapter,
};
use super::manifest::{SystemBinding, WorkloadSpec};
use super::Result;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::Instant;

const VECTOR_FIELD: &str = "v";

pub struct MilvusAdapter {
    http: Http,
    collection: OnceLock<String>,
}

impl MilvusAdapter {
    pub fn new(binding: &SystemBinding, workload: &WorkloadSpec) -> Self {
        Self {
            http: Http::new(&binding.endpoint, workload, Vec::new()),
            collection: OnceLock::new(),
        }
    }

    fn name(&self) -> Result<&str> {
        self.collection
            .get()
            .map(String::as_str)
            .ok_or_else(|| "collection is not prepared".into())
    }

    /// Milvus reports failures as HTTP 200 with a nonzero code, so the code decides.
    fn call(&self, path: &str, body: &Value) -> Result<Value> {
        let value = self
            .http
            .json("POST", &format!("/v2/vectordb{path}"), Some(body))?;
        if value["code"].as_i64() == Some(0) {
            Ok(value)
        } else {
            Err(match value["code"].as_i64() {
                Some(code) => format!("{path}: Milvus code {code} (body omitted)"),
                None => format!("{path}: missing Milvus result code (body omitted)"),
            })
        }
    }
}

fn parse_id(entry: &Value) -> Result<u64> {
    let id = &entry["id"];
    id.as_u64()
        .or_else(|| id.as_str().and_then(|text| text.parse().ok()))
        .ok_or_else(|| format!("invalid id in {}", describe(entry)))
}

impl SystemAdapter for MilvusAdapter {
    fn observed_version(&self) -> Result<String> {
        // The REST API has no version route, so this only proves reachability; the
        // pinned image digest in the manifest carries the version.
        self.call("/collections/list", &json!({}))
            .map(|_| "reachable; version from pinned image".to_string())
    }

    fn prepare(&self, _spec: &CollectionSpec<'_>) -> Result<()> {
        // Milvus 3.0.5 may report success when a collection already exists.
        // A random name is not authority to write or delete an existing one.
        Err(
            "Milvus lab mutations require an owned isolated service; admission is not implemented"
                .into(),
        )
    }

    fn ingest(&self, batch: &PointBatch<'_>) -> Result<()> {
        let data: Vec<Value> = (0..batch.ids.len())
            .map(|index| {
                json!({
                    "id": batch.ids[index],
                    VECTOR_FIELD: batch.vector(index),
                    "category": batch.categories[index]
                })
            })
            .collect();
        self.call(
            "/entities/insert",
            &json!({"collectionName": self.name()?, "data": data}),
        )?;
        Ok(())
    }

    fn finish_ingest(&self, deadline: Instant) -> Result<()> {
        let name = self.name()?;
        self.call("/collections/flush", &json!({"collectionName": name}))?;
        wait_until(deadline, "Milvus indexing", || {
            let value = self.call(
                "/indexes/describe",
                &json!({"collectionName": name, "indexName": VECTOR_FIELD}),
            )?;
            let index = &value["data"][0];
            if index["indexState"] == "Failed" {
                return Err("index build failed (server detail omitted)".into());
            }
            Ok(index["indexState"] == "Finished"
                && index["pendingRows"].as_u64() == Some(0)
                && index["indexedRows"] == index["totalRows"])
        })?;
        // A collection created with an index is already loaded, and a later handoff moves its
        // search off growing segments. Reloading means every measured search uses the index.
        self.call("/collections/release", &json!({"collectionName": name}))?;
        self.call("/collections/load", &json!({"collectionName": name}))?;
        wait_until(deadline, "Milvus load", || {
            let value = self.call(
                "/collections/get_load_state",
                &json!({"collectionName": name}),
            )?;
            Ok(value["data"]["loadState"] == "LoadStateLoaded")
        })
    }

    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        let mut body = json!({
            "collectionName": self.name()?,
            "data": [request.vector],
            "annsField": VECTOR_FIELD,
            "limit": request.k,
            "consistencyLevel": "Strong"
        });
        if let Some(ef) = request.setting.ef {
            body["searchParams"] = json!({"params": {"ef": ef}});
        }
        if let Some(category) = request.category {
            body["filter"] = json!(format!("category == {category}"));
        }
        let value = self.call("/entities/search", &body)?;
        let hits = value["data"]
            .as_array()
            .ok_or_else(|| format!("no hits in {}", describe(&value)))?;
        hits.iter().map(parse_id).collect()
    }

    fn index_evidence(&self) -> Result<Option<String>> {
        let value = self.call(
            "/indexes/describe",
            &json!({"collectionName": self.name()?, "indexName": VECTOR_FIELD}),
        )?;
        let index = &value["data"][0];
        Ok(Some(format!(
            "{} {}, {} of {} rows indexed",
            index["indexType"].as_str().unwrap_or("unknown"),
            index["indexState"].as_str().unwrap_or("unknown"),
            index["indexedRows"],
            index["totalRows"]
        )))
    }

    fn teardown(&self) -> Result<()> {
        self.call(
            "/collections/drop",
            &json!({"collectionName": self.name()?}),
        )?;
        Ok(())
    }
}
