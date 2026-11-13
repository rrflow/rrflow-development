//! What this machine chooses to do with installed definitions: which systems and tests are on,
//! what is tuned, and which launches a person has approved. It lives apart from the signed files,
//! so a toggle needs no key and a new pack keeps it.
//!
//! No choice here can flatter a result. A system turned off stays in the report with its reason. A
//! tuned system has a different binding digest and a limit line that says what changed, so it
//! cannot be read as the baseline. A launch a person has not approved cannot start, because the
//! files a run reads say there is nothing to start.
use super::{canonical_json, sha256_hex, Result};
use crate::storage::Id;
use rrflow_evaluation_core::Availability;
use rrflow_lab_benchmarks::catalog::{LaunchSpec, SystemDefinition, TestDefinition};
use rrflow_lab_benchmarks::manifest::VectorComparisonManifest;
use rrflow_lab_datasets::corpus::CorpusSource;
use rrflow_lab_datasets::corpus::CORPUS_DIRECTORY_ENV;
use rrflow_lab_definitions::content::{stem, systems_of};
use rrflow_lab_systems::binding::{IndexSpec, SearchSetting, SystemBinding};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const SETTINGS_SCHEMA_VERSION: u16 = 1;
pub const TURNED_OFF_REASON: &str = "turned off in this machine's lab settings";
const HELD_REASON: &str = "its start command is waiting for approval on this machine";
const MAX_SETTINGS_BYTES: u64 = 1024 * 1024;

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

fn whole_number<T: std::str::FromStr>(name: &str, value: &str) -> Result<T> {
    value
        .parse()
        .map_err(|_| format!("{name} must be a whole number, not {value:?}"))
}

fn rungs(name: &str, value: &str) -> Result<Vec<u32>> {
    value
        .split(',')
        .map(|rung| whole_number(name, rung.trim()))
        .collect()
}

fn joined(values: &[u32]) -> String {
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

impl SystemTuning {
    /// Sets one `name` from `value`, as a person types them.
    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        match name {
            "m" => self.m = Some(whole_number(name, value)?),
            "ef_construction" => self.ef_construction = Some(whole_number(name, value)?),
            "seed" => self.seed = Some(whole_number(name, value)?),
            "partitions" => self.partitions = Some(whole_number(name, value)?),
            "bits" if !value.is_empty() => self.bits = Some(value.to_owned()),
            "ef" => self.ef = Some(rungs(name, value)?),
            "nprobes" => self.nprobes = Some(rungs(name, value)?),
            _ => {
                let Some(setting) = name.strip_prefix("engine.").filter(|setting| {
                    !setting.is_empty()
                        && setting.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                        })
                }) else {
                    return Err(format!(
                        "there is no setting named {name:?}. These can be tuned: m, \
                         ef_construction, ef, bits, seed, partitions, nprobes, engine.NAME"
                    ));
                };
                self.engine_settings
                    .insert(setting.to_owned(), whole_number(name, value)?);
            }
        }
        Ok(())
    }

    /// One `name=value` for each thing set.
    pub fn describe(&self) -> Vec<String> {
        let mut said = Vec::new();
        if let Some(m) = self.m {
            said.push(format!("m={m}"));
        }
        if let Some(ef_construction) = self.ef_construction {
            said.push(format!("ef_construction={ef_construction}"));
        }
        if let Some(bits) = &self.bits {
            said.push(format!("bits={bits}"));
        }
        if let Some(seed) = self.seed {
            said.push(format!("seed={seed}"));
        }
        if let Some(partitions) = self.partitions {
            said.push(format!("partitions={partitions}"));
        }
        if let Some(ef) = &self.ef {
            said.push(format!("ef={}", joined(ef)));
        }
        if let Some(nprobes) = &self.nprobes {
            said.push(format!("nprobes={}", joined(nprobes)));
        }
        for (name, value) in &self.engine_settings {
            said.push(format!("engine.{name}={value}"));
        }
        said
    }

    /// Replaces the pack's values in a binding. A value this index does not have is refused,
    /// since quietly ignoring it would label a run as tuned when nothing changed.
    fn apply(&self, binding: &mut SystemBinding) -> Result<()> {
        let id = binding.id.as_str().to_owned();
        let refuse = |name: &str| Err(format!("{id}'s index has no setting named {name}"));
        match &mut binding.index {
            IndexSpec::Hnsw { m, ef_construction } => {
                if let Some(value) = self.m {
                    *m = value;
                }
                if let Some(value) = self.ef_construction {
                    *ef_construction = value;
                }
            }
            _ if self.m.is_some() => return refuse("m"),
            _ if self.ef_construction.is_some() => return refuse("ef_construction"),
            _ => {}
        }
        match &mut binding.index {
            IndexSpec::TurboQuant { bits, seed } => {
                if let Some(value) = &self.bits {
                    bits.clone_from(value);
                }
                if let Some(value) = self.seed {
                    *seed = value;
                }
            }
            _ if self.bits.is_some() => return refuse("bits"),
            _ if self.seed.is_some() => return refuse("seed"),
            _ => {}
        }
        match &mut binding.index {
            IndexSpec::IvfRq { partitions } => {
                if let Some(value) = self.partitions {
                    *partitions = value;
                }
            }
            _ if self.partitions.is_some() => return refuse("partitions"),
            _ => {}
        }
        let by_ef = matches!(
            binding.index,
            IndexSpec::Hnsw { .. } | IndexSpec::TurboQuant { .. }
        );
        let by_partitions = matches!(binding.index, IndexSpec::IvfRq { .. });
        if let Some(rungs) = &self.ef {
            if !by_ef {
                return refuse("ef");
            }
            binding.search_ladder = retuned_ladder(&binding.search_ladder, rungs, true, &id)?;
        }
        if let Some(rungs) = &self.nprobes {
            if !by_partitions {
                return refuse("nprobes");
            }
            binding.search_ladder = retuned_ladder(&binding.search_ladder, rungs, false, &id)?;
        }
        binding.engine_settings.extend(
            self.engine_settings
                .iter()
                .map(|(name, value)| (name.clone(), *value)),
        );
        let said = self.describe();
        if !said.is_empty() {
            binding.limits.push(format!(
                "Tuned on this machine, so this is not the published setup: {}.",
                said.join(", ")
            ));
        }
        Ok(())
    }
}

/// New rungs with the shape the ladder already has: a rerank that follows `ef` still follows it,
/// and one held fixed stays fixed.
fn retuned_ladder(
    ladder: &[SearchSetting],
    rungs: &[u32],
    by_ef: bool,
    system: &str,
) -> Result<Vec<SearchSetting>> {
    let first = ladder
        .first()
        .ok_or_else(|| format!("{system} has no search ladder to tune"))?;
    let (prefix, _) = first
        .id
        .as_str()
        .rsplit_once('-')
        .unwrap_or((if by_ef { "ef" } else { "nprobes" }, ""));
    let rerank_follows = first.exact_rerank.is_some() && first.exact_rerank == first.ef;
    rungs
        .iter()
        .map(|rung| {
            Ok(SearchSetting {
                id: Id::try_from(format!("{prefix}-{rung}"))?,
                ef: by_ef.then_some(*rung),
                exact_rerank: if rerank_follows {
                    Some(*rung)
                } else {
                    first.exact_rerank
                },
                nprobes: (!by_ef).then_some(*rung),
            })
        })
        .collect()
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
    pub fn load(path: &Path) -> Result<Self> {
        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        if metadata.len() > MAX_SETTINGS_BYTES {
            return Err(format!("{} is larger than settings may be", path.display()));
        }
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        // The version is read first, so a newer file gets a useful answer and not a field error.
        #[derive(Deserialize)]
        struct Version {
            schema_version: u16,
        }
        let version: Version = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if version.schema_version > SETTINGS_SCHEMA_VERSION {
            return Err(format!(
                "{} was written by a newer build, version {}",
                path.display(),
                version.schema_version
            ));
        }
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let parent = path.parent().ok_or("a settings path has no parent")?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("{}: {error}", parent.display()))?;
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        rrflow_lab_definitions::pack::write_atomically(path, &bytes)
    }
}

/// An installed pack's definitions, read and checked.
#[derive(Debug, Clone)]
pub struct Definitions {
    pub systems: BTreeMap<Id, SystemDefinition>,
    pub tests: BTreeMap<Id, TestDefinition>,
}

impl Definitions {
    pub fn read(files: &BTreeMap<String, String>) -> Result<Self> {
        let systems = systems_of(files)?;
        let mut tests = BTreeMap::new();
        for (path, text) in files {
            if let Some(name) = stem(path, "tests") {
                let test: TestDefinition =
                    serde_json::from_str(text).map_err(|error| format!("{path}: {error}"))?;
                if test.test.id.as_str() != name {
                    return Err(format!(
                        "{path} names itself {}, and a test's file is named for it",
                        test.test.id.as_str()
                    ));
                }
                tests.insert(test.test.id.clone(), test);
            }
        }
        Ok(Self { systems, tests })
    }
}

/// Whether a person must approve this system's launch before it starts on their machine.
/// RRFlow's own engine is the binary under test, so it brings nothing new with it.
pub fn needs_approval(system: &SystemDefinition) -> bool {
    matches!(
        system.launch,
        LaunchSpec::Container { .. } | LaunchSpec::Process { .. }
    )
}

pub fn launch_digest(system: &SystemDefinition) -> Result<String> {
    Ok(sha256_hex(&canonical_json(&system.launch)?))
}

fn is_approved(settings: &LabSettings, id: &Id, system: &SystemDefinition) -> Result<bool> {
    Ok(!needs_approval(system)
        || settings.approved_launches.get(id).map(String::as_str)
            == Some(launch_digest(system)?.as_str()))
}

/// Whether this system can run in at least one test. A test may declare a system available that
/// its own file does not, as the custom reference is.
pub fn runs_somewhere(definitions: &Definitions, id: &Id, system: &SystemDefinition) -> bool {
    let is_available = |availability: &Availability| *availability == Availability::Available;
    definitions.tests.values().any(|test| {
        test.systems.iter().any(|entry| {
            entry.system == *id
                && entry
                    .availability
                    .as_ref()
                    .map_or(is_available(&system.binding.availability), is_available)
        })
    })
}

/// The systems a person still has to look at, whichever test would use them.
pub fn waiting_for_approval(definitions: &Definitions, settings: &LabSettings) -> Result<Vec<Id>> {
    let mut waiting = Vec::new();
    for (id, system) in &definitions.systems {
        if runs_somewhere(definitions, id, system)
            && !settings.off.contains(id)
            && !is_approved(settings, id, system)?
        {
            waiting.push(id.clone());
        }
    }
    Ok(waiting)
}

/// Marks every launch in these definitions approved, and returns what changed.
pub fn approve_all(settings: &mut LabSettings, definitions: &Definitions) -> Result<Vec<Id>> {
    let mut changed = Vec::new();
    for (id, system) in &definitions.systems {
        if needs_approval(system) {
            let digest = launch_digest(system)?;
            if settings
                .approved_launches
                .insert(id.clone(), digest.clone())
                != Some(digest)
            {
                changed.push(id.clone());
            }
        }
    }
    Ok(changed)
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
    /// Environment variables a run needs for this system to start.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
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
    /// Environment variables a run needs for this test's data.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
    pub systems: Vec<PlannedSystem>,
    /// What a run of this test reads. Absent when the test is off or has a problem.
    #[serde(skip)]
    pub manifest: Option<VectorComparisonManifest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct LabPlan {
    pub tests: Vec<PlannedTest>,
}

fn reason_of(availability: &Availability) -> Option<String> {
    match availability {
        Availability::Available => None,
        Availability::Unavailable { reason }
        | Availability::Unimplemented { reason }
        | Availability::NotApplicable { reason } => Some(reason.clone()),
    }
}

/// One test as this machine would run it, or why it cannot.
fn plan_systems(
    definitions: &Definitions,
    settings: &LabSettings,
    manifest: &mut VectorComparisonManifest,
) -> Result<Vec<PlannedSystem>> {
    let mut planned = Vec::with_capacity(manifest.systems.len());
    for binding in &mut manifest.systems {
        let system = definitions
            .systems
            .get(&binding.id)
            .ok_or_else(|| format!("no definition for {}", binding.id.as_str()))?;
        let mut entry = PlannedSystem {
            id: binding.id.clone(),
            label: binding.label.clone(),
            state: SystemState::WillRun,
            reason: None,
            tuned: Vec::new(),
            starts: None,
            needs: system.launch.needed_variables(),
        };
        if binding.availability != Availability::Available {
            // A declared gap stays one: nothing this machine says can make it run.
            entry.state = SystemState::NotAvailable;
            entry.reason = reason_of(&binding.availability);
        } else if settings.off.contains(&binding.id) {
            entry.state = SystemState::TurnedOff;
            entry.reason = Some(TURNED_OFF_REASON.to_owned());
            binding.availability = Availability::Unavailable {
                reason: TURNED_OFF_REASON.to_owned(),
            };
        } else if !is_approved(settings, &binding.id, system)? {
            entry.state = SystemState::NeedsApproval;
            entry.reason = Some(HELD_REASON.to_owned());
            entry.starts = system.launch.summary();
            binding.availability = Availability::Unavailable {
                reason: HELD_REASON.to_owned(),
            };
        } else if let Some(tuning) = settings.tuning.get(&binding.id) {
            tuning.apply(binding)?;
            entry.tuned = tuning.describe();
        }
        planned.push(entry);
    }
    Ok(planned)
}

/// Works out what would run. It starts nothing and writes nothing.
pub fn build_plan(definitions: &Definitions, settings: &LabSettings) -> Result<LabPlan> {
    let mut tests = Vec::with_capacity(definitions.tests.len());
    for (id, test) in &definitions.tests {
        let mut planned = PlannedTest {
            id: id.clone(),
            title: test.test.title.clone(),
            on: !settings.off.contains(id),
            points: test.corpus.points,
            dimensions: test.corpus.dimensions,
            queries: test.workload.queries,
            k: test.workload.k,
            recall_target: test.quality.recall_target,
            problem: None,
            needs: match test.corpus.source {
                CorpusSource::Imported { .. } => vec![CORPUS_DIRECTORY_ENV.to_owned()],
                CorpusSource::GaussianMixture { .. } => Vec::new(),
            },
            systems: Vec::new(),
            manifest: None,
        };
        let mut manifest =
            rrflow_lab_benchmarks::catalog::resolve_test(test, &definitions.systems)?;
        match plan_systems(definitions, settings, &mut manifest)
            .and_then(|systems| manifest.validate().map(|()| systems))
        {
            Ok(systems) => {
                planned.systems = systems;
                if planned.on {
                    planned.manifest = Some(manifest);
                }
            }
            Err(problem) => planned.problem = Some(problem),
        }
        tests.push(planned);
    }
    Ok(LabPlan { tests })
}

/// JSON with a final newline, as the checked-in definitions have.
fn pretty(value: &impl Serialize) -> Result<String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    text.push('\n');
    Ok(text)
}

/// The files a run reads, by relative path. A system that is off or held is written with nothing
/// to start, so a script given these files cannot start it.
pub fn planned_files(
    definitions: &Definitions,
    settings: &LabSettings,
    plan: &LabPlan,
    test: &str,
) -> Result<BTreeMap<String, String>> {
    let planned = plan
        .tests
        .iter()
        .find(|entry| entry.id.as_str() == test)
        .ok_or_else(|| format!("no test is named {test}"))?;
    if let Some(problem) = &planned.problem {
        return Err(format!("{test} cannot run as configured: {problem}"));
    }
    if !planned.on {
        return Err(format!(
            "{test} is turned off. Turn it on with: rrflow-lab settings on {test}"
        ));
    }
    let manifest = planned
        .manifest
        .as_ref()
        .ok_or_else(|| format!("{test} has nothing to run"))?;
    let mut files = BTreeMap::new();
    files.insert(format!("manifests/{test}.json"), pretty(manifest)?);
    for (id, system) in &definitions.systems {
        let mut written = system.clone();
        let reason = if settings.off.contains(id) {
            Some(TURNED_OFF_REASON)
        } else if !is_approved(settings, id, system)? {
            Some(HELD_REASON)
        } else {
            None
        };
        if let Some(reason) = reason {
            written.launch = LaunchSpec::None;
            written.binding.availability = Availability::Unavailable {
                reason: reason.to_owned(),
            };
        }
        files.insert(format!("systems/{}.json", id.as_str()), pretty(&written)?);
    }
    let mut list = String::new();
    for system in &planned.systems {
        if system.state == SystemState::WillRun {
            list.push_str(system.id.as_str());
            list.push('\n');
        }
    }
    files.insert("run-list.txt".to_owned(), list);
    Ok(files)
}
