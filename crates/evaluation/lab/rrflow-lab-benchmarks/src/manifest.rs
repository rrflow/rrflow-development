//! The frozen, typed definition of one vector comparison: its corpus, workload,
//! quality target and the systems it runs against.
use rrflow_evaluation_core::report::TestIdentity;
use rrflow_evaluation_core::{require, Result};
use rrflow_lab_datasets::corpus::{CorpusSource, CorpusSpec};
use rrflow_lab_systems::adapter::ConnectionLimits;
use rrflow_lab_systems::binding::SystemBinding;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::Duration;

pub const MANIFEST_SCHEMA_VERSION: u16 = 1;
pub const MAX_LAB_POINTS: u64 = 10_000_000;
pub const MAX_LAB_DIMENSIONS: u32 = 65_536;
pub const MAX_LAB_QUERIES: u32 = 100_000;
pub const MAX_LAB_SYSTEMS: usize = 64;
pub const MAX_LAB_CONCURRENCY: u32 = 1_024;

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
            binding.validate(workload.k)?;
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

impl WorkloadSpec {
    /// The limits a system adapter applies to each request and to readiness.
    pub fn connection_limits(&self) -> ConnectionLimits {
        ConnectionLimits {
            request_timeout: Duration::from_millis(self.request_timeout_ms),
            readiness_timeout: Duration::from_millis(self.readiness_timeout_ms),
        }
    }
}
