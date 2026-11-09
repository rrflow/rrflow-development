//! The one boundary every compared system implements. Each adapter speaks only
//! the system's public API, so no comparator code enters this repository.
use super::manifest::{
    IndexSpec, LabMetric, SearchSetting, SystemBinding, SystemKind, WorkloadSpec,
};
use super::Result;
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CollectionSpec<'a> {
    pub name: &'a str,
    pub dimensions: u32,
    pub metric: LabMetric,
    pub categories: u32,
    pub index: &'a IndexSpec,
}

#[derive(Debug, Clone, Copy)]
pub struct PointBatch<'a> {
    pub ids: &'a [u64],
    /// Flat, `dimensions` values per point.
    pub vectors: &'a [f32],
    pub categories: &'a [u32],
    pub dimensions: usize,
}

impl<'a> PointBatch<'a> {
    pub fn vector(&self, index: usize) -> &'a [f32] {
        &self.vectors[index * self.dimensions..(index + 1) * self.dimensions]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchRequest<'a> {
    pub vector: &'a [f32],
    pub k: u32,
    pub category: Option<u32>,
    pub setting: &'a SearchSetting,
}

pub trait SystemAdapter: Sync {
    /// What the running server reports about itself, not what the manifest says.
    fn observed_version(&self) -> Result<String>;
    /// Prepares a scratch collection. Provider-specific ownership constraints apply.
    fn prepare(&self, spec: &CollectionSpec<'_>) -> Result<()>;
    fn ingest(&self, batch: &PointBatch<'_>) -> Result<()>;
    /// Returns once indexes are built and search reflects every point.
    fn finish_ingest(&self, deadline: Instant) -> Result<()>;
    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>>;
    /// What the system itself says served the searches. A configured index can go
    /// unused: at small sizes some systems scan instead, and a result must show that.
    fn index_evidence(&self) -> Result<Option<String>> {
        Ok(None)
    }
    fn teardown(&self) -> Result<()>;
}

/// Resource identifiers enter URL paths and SQL identifiers; never accept arbitrary syntax.
pub(crate) fn validate_scratch_name(name: &str) -> Result<()> {
    if name.len() > 80
        || !name.starts_with("lab_")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err("invalid scratch resource name".into());
    }
    Ok(())
}

/// Groups consecutive items so each group fits a server's request-body limit. A single
/// item over the limit gets its own group, and the server's refusal then reports it.
pub(crate) fn split_by_size(
    sizes: impl IntoIterator<Item = usize>,
    max_bytes: usize,
    max_items: usize,
) -> Vec<std::ops::Range<usize>> {
    let mut groups = Vec::new();
    let (mut start, mut bytes, mut index) = (0, 0, 0);
    for size in sizes {
        if index > start && (bytes + size > max_bytes || index - start >= max_items) {
            groups.push(start..index);
            start = index;
            bytes = 0;
        }
        bytes += size;
        index += 1;
    }
    if index > start {
        groups.push(start..index);
    }
    groups
}

/// A small blocking client. Responses are bounded and server error bodies are
/// excluded from diagnostics because they may contain credentials or user data.
pub(crate) struct Http {
    agent: ureq::Agent,
    base: String,
    headers: Vec<(String, String)>,
}

const MAX_RESPONSE_BYTES: u64 = 256 * 1024 * 1024;

impl Http {
    pub(crate) fn new(base: &str, workload: &WorkloadSpec, headers: Vec<(String, String)>) -> Self {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_millis(workload.request_timeout_ms)))
            .build()
            .new_agent();
        Self {
            agent,
            base: base.trim_end_matches('/').to_string(),
            headers,
        }
    }

    pub(crate) fn raw(
        &self,
        method: &str,
        path: &str,
        body: Option<(&str, Vec<u8>)>,
    ) -> Result<(u16, String)> {
        let url = format!("{}{path}", self.base);
        let label = format!("{method} {path}");
        let mut builder = ureq::http::Request::builder().method(method).uri(&url);
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }
        let response = match body {
            Some((content_type, bytes)) => {
                let request = builder
                    .header("Content-Type", content_type)
                    .body(bytes)
                    .map_err(|_| format!("{label}: invalid request"))?;
                self.agent.run(request)
            }
            None => {
                let request = builder
                    .body(())
                    .map_err(|_| format!("{label}: invalid request"))?;
                self.agent.run(request)
            }
        };
        let mut response = response.map_err(|_| format!("{label}: transport failed"))?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES)
            .read_to_string()
            .map_err(|_| format!("{label}: response read failed"))?;
        Ok((status, text))
    }

    /// Sends JSON, requiring a 2xx response that parses as JSON.
    pub(crate) fn json(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
        let payload = match body {
            Some(value) => Some((
                "application/json",
                serde_json::to_vec(value).map_err(|error| error.to_string())?,
            )),
            None => None,
        };
        let (status, text) = self.raw(method, path, payload)?;
        if !(200..300).contains(&status) {
            return Err(format!("{method} {path}: HTTP {status} (body omitted)"));
        }
        serde_json::from_str(&text)
            .map_err(|_| format!("{method} {path}: invalid JSON (body omitted)"))
    }
}

pub(crate) fn describe(_value: &Value) -> String {
    "response body omitted".to_string()
}

pub(crate) fn wait_until(
    deadline: Instant,
    label: &str,
    mut ready: impl FnMut() -> Result<bool>,
) -> Result<()> {
    loop {
        if ready()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "{label} did not finish before the readiness deadline"
            ));
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

fn environment(name: &Option<String>, id: &str, what: &str) -> Result<String> {
    let Some(name) = name else {
        return Err(format!("{id} needs a {what} environment reference"));
    };
    std::env::var(name).map_err(|_| format!("{id}: environment variable {name} is not set"))
}

pub(crate) fn required_credentials(binding: &SystemBinding) -> Result<(String, String)> {
    let id = binding.id.as_str();
    Ok((
        environment(&binding.credentials.principal_env, id, "principal")?,
        environment(&binding.credentials.secret_env, id, "secret")?,
    ))
}

pub(crate) fn required_instance(binding: &SystemBinding) -> Result<String> {
    environment(
        &binding.credentials.instance_env,
        binding.id.as_str(),
        "instance",
    )
}

pub fn connect(binding: &SystemBinding, workload: &WorkloadSpec) -> Result<Box<dyn SystemAdapter>> {
    Ok(match binding.kind {
        SystemKind::Rrflow => Box::new(super::rrflow::RrflowAdapter::connect(binding, workload)?),
        SystemKind::Qdrant => Box::new(super::qdrant::QdrantAdapter::new(binding, workload)),
        SystemKind::Surrealdb => {
            Box::new(super::surrealdb::SurrealAdapter::new(binding, workload)?)
        }
        SystemKind::Milvus => Box::new(super::milvus::MilvusAdapter::new(binding, workload)),
        SystemKind::Custom => Box::new(super::custom::CustomAdapter::new(binding, workload)),
    })
}

#[cfg(test)]
mod tests {
    use super::split_by_size;

    #[test]
    fn groups_respect_both_limits_and_cover_every_item_once() {
        assert_eq!(split_by_size([4, 4, 4], 8, 10), vec![0..2, 2..3]);
        assert_eq!(split_by_size([1, 1, 1], 100, 2), vec![0..2, 2..3]);
        assert_eq!(split_by_size([9, 1], 8, 10), vec![0..1, 1..2]);
        assert_eq!(split_by_size([1, 9, 1], 8, 10), vec![0..1, 1..2, 2..3]);
        assert!(split_by_size([], 8, 10).is_empty());
    }
}
