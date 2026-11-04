//! The frozen, typed definition of one vector comparison. It names credentials
//! only by environment variable, so a manifest can be published with its results.
use super::report::TestIdentity;
use super::{require, Result};
use crate::storage::{Digest, Id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MANIFEST_SCHEMA_VERSION: u16 = 1;
pub const MAX_LAB_POINTS: u64 = 10_000_000;
pub const MAX_LAB_DIMENSIONS: u32 = 65_536;
pub const MAX_LAB_QUERIES: u32 = 100_000;
pub const MAX_LAB_SYSTEMS: usize = 64;
pub const MAX_LAB_CONCURRENCY: u32 = 1_024;
pub const MAX_LADDER_SETTINGS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LabMetric {
    Cosine,
    Dot,
    Euclidean,
}

/// Generated from fixed arithmetic so its digest matches on every platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "generator", rename_all = "snake_case", deny_unknown_fields)]
pub enum CorpusSource {
    GaussianMixture {
        seed: u64,
        clusters: u32,
        /// Standard deviation around a cluster center, in thousandths.
        spread_milli: u32,
    },
    /// Real vectors prepared by `import_corpus.py`, kept outside the repository and named
    /// by digest. Each file holds little-endian 32-bit floats.
    Imported {
        /// Where the data came from, for a reader of the report.
        origin: String,
        license: String,
        points_sha256: Digest,
        queries_sha256: Digest,
        /// The data has no filter field, so categories are assigned from this seed.
        category_seed: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CorpusSpec {
    pub source: CorpusSource,
    pub points: u64,
    pub dimensions: u32,
    pub metric: LabMetric,
    /// Filtered search selects one of these, so selectivity is about 1/categories.
    pub categories: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkloadSpec {
    pub queries: u32,
    pub k: u32,
    pub warmup_queries: u32,
    pub ingest_batch: u32,
    pub concurrency: Vec<u32>,
    pub filtered: bool,
    pub request_timeout_ms: u64,
    pub readiness_timeout_ms: u64,
    /// A case stops here and reports the samples it reached.
    pub max_case_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualitySpec {
    /// Speed is compared only at or above this recall, declared before running.
    pub recall_target: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemKind {
    Rrflow,
    Qdrant,
    Surrealdb,
    Milvus,
    Custom,
}

/// A reader must be able to see when a result crossed a network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemLocation {
    Loopback,
    Lan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "index", rename_all = "snake_case", deny_unknown_fields)]
pub enum IndexSpec {
    Exact,
    Hnsw { m: u32, ef_construction: u32 },
    TurboQuant { bits: String, seed: u64 },
    IvfRq { partitions: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchSetting {
    pub id: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ef: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_rerank: Option<u32>,
    /// How many partitions an IVF index probes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nprobes: Option<u32>,
}

/// A declared gap is reported as such. It can never read as a passing result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Availability {
    Available,
    Unavailable { reason: String },
    Unimplemented { reason: String },
    NotApplicable { reason: String },
}

/// Environment variable names only. The values never enter a file.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CredentialReferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_env: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SystemBinding {
    pub id: Id,
    pub label: String,
    pub kind: SystemKind,
    pub location: SystemLocation,
    pub endpoint: String,
    /// Pinned image or build identity, compared with what the server reports.
    pub artifact: String,
    /// What backs the data, because memory and durable disk are different claims.
    pub storage_profile: String,
    pub availability: Availability,
    pub index: IndexSpec,
    pub search_ladder: Vec<SearchSetting>,
    #[serde(default)]
    pub credentials: CredentialReferences,
    /// Commands that print one integer byte count on standard output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_probe: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_probe: Option<Vec<String>>,
    /// RRFlow `[storage]` settings the runner writes before the engine starts, so a run
    /// away from the installed defaults says so in the manifest it is bound to.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub engine_settings: BTreeMap<String, u64>,
    /// What a reader must know about this system's results, repeated in the report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorComparisonManifest {
    pub schema_version: u16,
    pub test: TestIdentity,
    pub corpus: CorpusSpec,
    pub workload: WorkloadSpec,
    pub quality: QualitySpec,
    pub systems: Vec<SystemBinding>,
}

fn valid_env_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_endpoint(binding: &SystemBinding) -> Result<()> {
    let id = binding.id.as_str();
    let Some(address) = binding.endpoint.strip_prefix("http://") else {
        return Err(format!(
            "{id} endpoint must use http:// on a trusted network"
        ));
    };
    require(
        !address.contains('@'),
        &format!("{id} endpoint cannot carry credentials; use environment references"),
    )?;
    let host = address.split([':', '/']).next().unwrap_or_default();
    let loopback = matches!(host, "127.0.0.1" | "localhost" | "[::1]");
    match binding.location {
        SystemLocation::Loopback => require(
            loopback,
            &format!("{id} is declared loopback but its endpoint is not"),
        ),
        SystemLocation::Lan => require(
            !loopback,
            &format!("{id} is declared lan but its endpoint is loopback"),
        ),
    }
}

impl SystemBinding {
    fn validate(&self, workload: &WorkloadSpec) -> Result<()> {
        require(
            self.engine_settings.is_empty() || self.kind == SystemKind::Rrflow,
            &format!(
                "{} declares engine settings, which only RRFlow takes",
                self.id.as_str()
            ),
        )?;
        require(
            self.engine_settings.keys().all(|name| {
                !name.is_empty()
                    && name.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            }),
            &format!("{} has an invalid engine setting name", self.id.as_str()),
        )?;
        let id = self.id.as_str();
        require(
            !self.label.trim().is_empty() && self.label.len() <= 128,
            &format!("{id} label is invalid"),
        )?;
        require(
            !self.storage_profile.trim().is_empty() && !self.artifact.trim().is_empty(),
            &format!("{id} must declare its artifact and storage profile"),
        )?;
        validate_endpoint(self)?;
        for name in [
            &self.credentials.principal_env,
            &self.credentials.secret_env,
            &self.credentials.instance_env,
        ]
        .into_iter()
        .flatten()
        {
            require(
                valid_env_name(name),
                &format!("{id} credential reference must be an environment variable name"),
            )?;
        }
        if let Availability::Unavailable { reason }
        | Availability::Unimplemented { reason }
        | Availability::NotApplicable { reason } = &self.availability
        {
            return require(
                !reason.trim().is_empty(),
                &format!("{id} must say why it does not run"),
            );
        }
        require(
            !self.search_ladder.is_empty() && self.search_ladder.len() <= MAX_LADDER_SETTINGS,
            &format!("{id} search ladder size is outside bounds"),
        )?;
        let mut settings = BTreeSet::new();
        for setting in &self.search_ladder {
            require(
                settings.insert(&setting.id),
                &format!("{id} repeats a search setting"),
            )?;
            if let Some(ef) = setting.ef {
                require(
                    ef >= workload.k,
                    &format!("{id} ef must be at least k, or some systems reject it"),
                )?;
            }
            if let Some(rerank) = setting.exact_rerank {
                require(
                    rerank >= workload.k && setting.ef.is_none_or(|ef| rerank <= ef),
                    &format!("{id} rerank must be at least k and at most ef"),
                )?;
            }
            if matches!(
                self.index,
                IndexSpec::Hnsw { .. } | IndexSpec::TurboQuant { .. }
            ) {
                require(
                    setting.ef.is_some(),
                    &format!("{id} approximate search needs ef"),
                )?;
            }
            if let IndexSpec::IvfRq { partitions } = self.index {
                require(
                    setting
                        .nprobes
                        .is_some_and(|nprobes| (1..=partitions).contains(&nprobes)),
                    &format!("{id} IVF search needs nprobes within its partitions"),
                )?;
            }
        }
        match (&self.kind, &self.index) {
            (SystemKind::Custom, IndexSpec::IvfRq { partitions }) => require(
                *partitions > 0,
                &format!("{id} IVF index needs at least one partition"),
            ),
            (_, IndexSpec::IvfRq { .. }) => Err(format!(
                "{id}: an IVF index is available only through a custom system"
            )),
            (SystemKind::Rrflow, _) | (SystemKind::Custom, _) | (_, IndexSpec::Hnsw { .. }) => {
                Ok(())
            }
            (_, IndexSpec::Exact) => Ok(()),
            (_, IndexSpec::TurboQuant { .. }) => {
                Err(format!("{id}: TurboQuant is available only for RRFlow"))
            }
        }
    }
}

impl VectorComparisonManifest {
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == MANIFEST_SCHEMA_VERSION,
            "unsupported manifest schema version",
        )?;
        require(
            !self.test.title.trim().is_empty() && self.test.title.len() <= 256,
            "test title is invalid",
        )?;
        let corpus = &self.corpus;
        require(
            (1..=MAX_LAB_POINTS).contains(&corpus.points),
            "corpus point count is outside bounds",
        )?;
        require(
            (1..=MAX_LAB_DIMENSIONS).contains(&corpus.dimensions),
            "corpus dimensions are outside bounds",
        )?;
        require(corpus.categories > 0, "corpus needs at least one category")?;
        match &corpus.source {
            CorpusSource::GaussianMixture {
                clusters,
                spread_milli,
                ..
            } => require(
                *clusters > 0 && u64::from(*clusters) <= corpus.points && *spread_milli > 0,
                "corpus mixture is invalid",
            )?,
            CorpusSource::Imported {
                origin, license, ..
            } => require(
                !origin.trim().is_empty() && !license.trim().is_empty(),
                "an imported corpus must say where it came from and under what license",
            )?,
        }
        let workload = &self.workload;
        require(
            (1..=MAX_LAB_QUERIES).contains(&workload.queries),
            "query count is outside bounds",
        )?;
        require(
            workload.k > 0 && u64::from(workload.k) <= corpus.points,
            "k is outside bounds",
        )?;
        require(workload.ingest_batch > 0, "ingest batch must be positive")?;
        require(
            !workload.concurrency.is_empty()
                && workload
                    .concurrency
                    .iter()
                    .all(|count| (1..=MAX_LAB_CONCURRENCY).contains(count)),
            "concurrency ladder is outside bounds",
        )?;
        require(
            workload.request_timeout_ms > 0
                && workload.readiness_timeout_ms > 0
                && workload.max_case_seconds > 0,
            "workload limits must be positive",
        )?;
        require(
            self.quality.recall_target > 0.0 && self.quality.recall_target <= 1.0,
            "recall target must be in (0, 1]",
        )?;
        require(
            !self.systems.is_empty() && self.systems.len() <= MAX_LAB_SYSTEMS,
            "system count is outside bounds",
        )?;
        let mut systems = BTreeSet::new();
        for binding in &self.systems {
            require(systems.insert(&binding.id), "duplicate system identity")?;
            binding.validate(workload)?;
        }
        Ok(())
    }

    pub fn binding(&self, id: &str) -> Result<&SystemBinding> {
        self.systems
            .iter()
            .find(|binding| binding.id.as_str() == id)
            .ok_or_else(|| format!("manifest has no system named {id}"))
    }
}
