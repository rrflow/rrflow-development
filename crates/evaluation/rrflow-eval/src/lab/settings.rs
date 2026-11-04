//! What this machine chooses to do with installed definitions: which systems and tests are on,
//! what is tuned, and which launches a person has approved. It lives apart from the signed files,
//! so a toggle needs no key and a new pack keeps it.
use super::catalog::{SystemDefinition, TestDefinition};
use super::manifest::VectorComparisonManifest;
use super::Result;
use crate::storage::Id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const SETTINGS_SCHEMA_VERSION: u16 = 1;
pub const TURNED_OFF_REASON: &str = "turned off in this machine's lab settings";

const NOT_BUILT: &str = "not built yet";

/// Only what a person may change without changing what a pack means. Each value replaces the
/// pack's own, and a run made with one says so in its report.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SystemTuning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ef_construction: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bits: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partitions: Option<u32>,
    /// The search ladder's rungs, for an index searched by `ef`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ef: Option<Vec<u32>>,
    /// The search ladder's rungs, for an index searched by partitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nprobes: Option<Vec<u32>>,
    /// RRFlow `[storage]` settings, added to the system's own.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub engine_settings: BTreeMap<String, u64>,
}

impl SystemTuning {
    /// Sets one `name` from `value`, as a person types them.
    pub fn set(&mut self, _name: &str, _value: &str) -> Result<()> {
        Err(NOT_BUILT.into())
    }

    /// One `name=value` for each thing set.
    pub fn describe(&self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabSettings {
    pub schema_version: u16,
    /// The folder `run_all.sh` updates from. Without one, this machine's own outbox.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_from: Option<String>,
    /// Systems and tests turned off.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub off: BTreeSet<Id>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tuning: BTreeMap<Id, SystemTuning>,
    /// The launch digest a person approved for each system.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub approved_launches: BTreeMap<Id, String>,
}

impl Default for LabSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            update_from: None,
            off: BTreeSet::new(),
            tuning: BTreeMap::new(),
            approved_launches: BTreeMap::new(),
        }
    }
}

impl LabSettings {
    /// A missing file means nothing has been changed.
    pub fn load(_path: &Path) -> Result<Self> {
        Err(NOT_BUILT.into())
    }

    pub fn save(&self, _path: &Path) -> Result<()> {
        Err(NOT_BUILT.into())
    }
}

/// An installed pack's definitions, read and checked.
#[derive(Debug, Clone)]
pub struct Definitions {
    pub systems: BTreeMap<Id, SystemDefinition>,
    pub tests: BTreeMap<Id, TestDefinition>,
}

impl Definitions {
    pub fn read(_files: &BTreeMap<String, String>) -> Result<Self> {
        Err(NOT_BUILT.into())
    }
}

/// Whether a person must approve this system's launch before it starts on their machine.
pub fn needs_approval(_system: &SystemDefinition) -> bool {
    false
}

pub fn launch_digest(_system: &SystemDefinition) -> Result<String> {
    Err(NOT_BUILT.into())
}

/// Marks every launch in these definitions approved, and returns what changed.
pub fn approve_all(_settings: &mut LabSettings, _definitions: &Definitions) -> Result<Vec<Id>> {
    Err(NOT_BUILT.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemState {
    WillRun,
    TurnedOff,
    NeedsApproval,
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct PlannedSystem {
    pub id: Id,
    pub label: String,
    pub state: SystemState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// What this machine tuned, as `name=value`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tuned: Vec<String>,
    /// What approving would allow to start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct PlannedTest {
    pub id: Id,
    pub title: String,
    pub on: bool,
    pub points: u64,
    pub dimensions: u32,
    pub queries: u32,
    pub k: u32,
    pub recall_target: f64,
    /// Why this test cannot run as configured, such as tuning it refuses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    pub systems: Vec<PlannedSystem>,
    /// What a run of this test reads. Absent when the test is off or has a problem.
    #[serde(skip)]
    pub manifest: Option<VectorComparisonManifest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct LabPlan {
    pub tests: Vec<PlannedTest>,
}

/// Works out what would run. It starts nothing and writes nothing.
pub fn build_plan(_definitions: &Definitions, _settings: &LabSettings) -> Result<LabPlan> {
    Err(NOT_BUILT.into())
}

/// The files a run reads, by relative path. A system that is off or held is written with nothing
/// to start, so a script given these files cannot start it.
pub fn planned_files(
    _definitions: &Definitions,
    _settings: &LabSettings,
    _plan: &LabPlan,
    _test: &str,
) -> Result<BTreeMap<String, String>> {
    Err(NOT_BUILT.into())
}
