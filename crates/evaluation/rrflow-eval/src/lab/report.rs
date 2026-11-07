//! The test-agnostic report. Any test can write it, and a client renders it from
//! the metrics, dimensions and views it declares, so a new test needs no new
//! components. It has no winner field: verdicts answer declared questions.
use super::{require, Result};
use crate::storage::{Digest, Id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const REPORT_SCHEMA_VERSION: u16 = 1;
pub const MAX_REPORT_SYSTEMS: usize = 64;
pub const MAX_REPORT_OBSERVATIONS: usize = 200_000;
pub const MAX_REPORT_TEXT_BYTES: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestIdentity {
    pub id: Id,
    pub title: String,
    pub family: Id,
    pub revision: u32,
}

/// Only diagnostic evidence exists today. J-04 acceptance requires its own
/// qualification and cannot be claimed through this report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LabEvidenceClass {
    Diagnostic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemStatus {
    Completed,
    Partial,
    Failed,
    Unavailable,
    Unimplemented,
    NotApplicable,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReportSystem {
    pub id: Id,
    pub label: String,
    pub kind: String,
    pub status: SystemStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_version: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub configuration: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    Higher,
    Lower,
    Neither,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricDefinition {
    pub id: Id,
    pub label: String,
    pub unit: String,
    pub better: MetricDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DimensionDefinition {
    pub id: Id,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub system: Id,
    pub metric: Id,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub case: BTreeMap<String, String>,
    pub value: f64,
    pub samples: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerdictOutcome {
    Met,
    NotMet,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Verdict {
    pub id: Id,
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<Id>,
    pub outcome: VerdictOutcome,
    pub basis: String,
}

/// What a client should draw. `fixed` selects observations by case dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ViewDefinition {
    /// One line per system, joining the two metrics on identical cases.
    Curve {
        id: Id,
        title: String,
        x: Id,
        y: Id,
        #[serde(default)]
        fixed: BTreeMap<String, String>,
    },
    Bars {
        id: Id,
        title: String,
        metric: Id,
        #[serde(default)]
        fixed: BTreeMap<String, String>,
    },
    Table {
        id: Id,
        title: String,
        metrics: Vec<Id>,
        rows: Vec<Id>,
        #[serde(default)]
        fixed: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReference {
    pub path: String,
    pub sha256: Digest,
    pub bytes: u64,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabReport {
    pub schema_version: u16,
    pub test: TestIdentity,
    pub evidence_class: LabEvidenceClass,
    pub generated_by: String,
    pub generated_at_unix_ms: u64,
    pub environment: BTreeMap<String, String>,
    pub identities: BTreeMap<String, Digest>,
    pub systems: Vec<ReportSystem>,
    pub dimensions: Vec<DimensionDefinition>,
    pub metrics: Vec<MetricDefinition>,
    pub observations: Vec<Observation>,
    pub verdicts: Vec<Verdict>,
    pub views: Vec<ViewDefinition>,
    pub limits: Vec<String>,
    pub artifacts: Vec<ArtifactReference>,
}

fn bounded(text: &str, label: &str) -> Result<()> {
    require(
        !text.trim().is_empty() && text.len() <= MAX_REPORT_TEXT_BYTES,
        &format!("{label} must be nonempty and bounded"),
    )
}

impl LabReport {
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == REPORT_SCHEMA_VERSION,
            "unsupported report schema version",
        )?;
        bounded(&self.test.title, "test title")?;
        require(
            !self.systems.is_empty() && self.systems.len() <= MAX_REPORT_SYSTEMS,
            "report system count is outside bounds",
        )?;
        require(
            self.observations.len() <= MAX_REPORT_OBSERVATIONS,
            "report observation count exceeds its bound",
        )?;
        let mut systems = BTreeSet::new();
        for system in &self.systems {
            require(systems.insert(&system.id), "duplicate report system")?;
            bounded(&system.label, "system label")?;
            match (system.status, &system.reason) {
                (SystemStatus::Completed, _) => {}
                (_, Some(reason)) => bounded(reason, "system reason")?,
                (_, None) => return Err(format!("{} needs a reason", system.id.as_str())),
            }
        }
        let metrics: BTreeSet<_> = self.metrics.iter().map(|metric| &metric.id).collect();
        require(
            metrics.len() == self.metrics.len(),
            "duplicate report metric",
        )?;
        let dimensions: BTreeSet<_> = self
            .dimensions
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        require(
            dimensions.len() == self.dimensions.len(),
            "duplicate report dimension",
        )?;
        let mut observed = BTreeSet::new();
        for observation in &self.observations {
            require(
                systems.contains(&observation.system),
                "observation names an undeclared system",
            )?;
            require(
                metrics.contains(&observation.metric),
                "observation names an undeclared metric",
            )?;
            require(
                observation
                    .case
                    .keys()
                    .all(|key| dimensions.contains(key.as_str())),
                "observation names an undeclared dimension",
            )?;
            require(
                observation.value.is_finite()
                    && observation.low.is_none_or(f64::is_finite)
                    && observation.high.is_none_or(f64::is_finite),
                "observation values must be finite",
            )?;
            observed.insert(&observation.system);
        }
        for system in &self.systems {
            match system.status {
                SystemStatus::Completed | SystemStatus::Partial => require(
                    observed.contains(&system.id),
                    "a completed system needs at least one observation",
                )?,
                SystemStatus::Failed => {}
                _ => require(
                    !observed.contains(&system.id),
                    "a system that did not run cannot have observations",
                )?,
            }
        }
        for verdict in &self.verdicts {
            bounded(&verdict.question, "verdict question")?;
            bounded(&verdict.basis, "verdict basis")?;
            if let Some(system) = &verdict.system {
                require(
                    systems.contains(system),
                    "verdict names an undeclared system",
                )?;
            }
        }
        for view in &self.views {
            let (referenced, fixed): (Vec<&Id>, _) = match view {
                ViewDefinition::Curve { x, y, fixed, .. } => (vec![x, y], fixed),
                ViewDefinition::Bars { metric, fixed, .. } => (vec![metric], fixed),
                ViewDefinition::Table {
                    metrics: columns,
                    rows,
                    fixed,
                    ..
                } => {
                    require(
                        rows.iter().all(|row| dimensions.contains(row.as_str())),
                        "view row names an undeclared dimension",
                    )?;
                    (columns.iter().collect(), fixed)
                }
            };
            require(
                referenced.iter().all(|metric| metrics.contains(metric)),
                "view names an undeclared metric",
            )?;
            require(
                fixed.keys().all(|key| dimensions.contains(key.as_str())),
                "view fixes an undeclared dimension",
            )?;
        }
        Ok(())
    }
}
