//! One file per system and one file per test, so a new system or a newer pin takes a file and
//! not code. A manifest is generated from them, so a report's digests stay what they were. How a
//! system starts is data too: `run_system.sh` asks for its arguments and has no case per system.
use super::manifest::VectorComparisonManifest;
use super::manifest::{CorpusSpec, QualitySpec, WorkloadSpec};
use super::{require, Result};
use crate::storage::Id;
use rrflow_evaluation_core::report::TestIdentity;
use rrflow_evaluation_core::Availability;
use rrflow_lab_systems::binding::{IndexSpec, SearchSetting, SystemBinding};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CATALOG_SCHEMA_VERSION: u16 = 1;
const MAX_READY_SECONDS: u32 = 3_600;

/// A folder inside the run's private directory, mounted into a container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VolumeMount {
    pub container_path: String,
}

/// How to tell a system is up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadyCheck {
    pub url: String,
    /// Text the reply must hold, for a route that answers before the system is ready.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contains: Option<String>,
    pub seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LaunchSpec {
    /// An image pinned by digest. The container is removed after the run.
    Container {
        image: String,
        name: String,
        /// `127.0.0.1:HOST:CONTAINER`. Nothing is published beyond this machine.
        ports: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        volume: Option<VolumeMount>,
        /// Plain settings, never secrets.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        env: BTreeMap<String, String>,
        /// Names in the container, each given a value generated for this run under the host's name.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        generated_secrets: BTreeMap<String, String>,
        /// Fixed host variables a client needs, such as a user name. Never secrets.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        host_env: BTreeMap<String, String>,
        /// Container settings passed by name from `host_env` or `generated_secrets`.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        env_from_host: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        user: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<String>,
        ready: ReadyCheck,
    },
    /// A local process. `{lab}` is `eval/lab`, `{data}` the run's data folder, and `{env:NAME}`
    /// a variable that must be set.
    Process {
        command: Vec<String>,
        port: u16,
        /// Written under the run's folder for a probe to find the process.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pid_file: Option<String>,
        /// A harness that embeds the system needs no server from here.
        #[serde(default)]
        embedded_under_vectordbbench: bool,
        ready: ReadyCheck,
    },
    /// RRFlow's own install and serve path.
    RrflowEngine { port: u16 },
    /// A declared gap, with nothing to start.
    None,
}

impl LaunchSpec {
    /// What this starts that the machine was not already running, in words a person can check.
    pub fn summary(&self) -> Option<String> {
        match self {
            Self::Container { image, .. } => Some(format!("container {image}")),
            Self::Process { command, .. } => Some(format!("program {}", command.join(" "))),
            Self::RrflowEngine { .. } | Self::None => None,
        }
    }

    /// The environment variables a run needs for this to start.
    pub fn needed_variables(&self) -> Vec<String> {
        let mut names = Vec::new();
        match self {
            Self::Process { command, .. } => {
                for word in command {
                    let mut rest = word.as_str();
                    while let Some((_, after)) = rest.split_once("{env:") {
                        let Some((name, tail)) = after.split_once('}') else {
                            break;
                        };
                        if !names.iter().any(|known| known == name) {
                            names.push(name.to_owned());
                        }
                        rest = tail;
                    }
                }
            }
            Self::RrflowEngine { .. } => names.push("RRFLOW_BIN".to_owned()),
            Self::Container { .. } | Self::None => {}
        }
        names
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SystemDefinition {
    pub schema_version: u16,
    pub binding: SystemBinding,
    pub launch: LaunchSpec,
    /// Arguments that drive it under VectorDBBench. `{ef}` is the search setting and `{data}`
    /// the run's data folder.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vectordbbench: Vec<String>,
}

/// One system in one test. Anything left out comes from the system's own file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestSystem {
    pub system: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<IndexSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_ladder: Option<Vec<SearchSetting>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availability: Option<Availability>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestDefinition {
    pub schema_version: u16,
    pub test: TestIdentity,
    /// The name others publish this data under, so a published result can be matched to it.
    /// Generated data has none. It is outside every digest, because it changes no measurement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_dataset: Option<String>,
    pub corpus: CorpusSpec,
    pub workload: WorkloadSpec,
    pub quality: QualitySpec,
    pub systems: Vec<TestSystem>,
}

fn is_loopback_port(mapping: &str) -> bool {
    let parts: Vec<&str> = mapping.split(':').collect();
    parts.len() == 3
        && parts[0] == "127.0.0.1"
        && parts[1].parse::<u16>().is_ok()
        && parts[2].parse::<u16>().is_ok()
}

fn is_digest_pinned(image: &str) -> bool {
    image.split_once("@sha256:").is_some_and(|(name, digest)| {
        !name.is_empty()
            && digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn is_variable_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn looks_secret(name: &str) -> bool {
    ["SECRET", "PASS", "TOKEN", "KEY", "CREDENTIAL"]
        .iter()
        .any(|word| name.contains(word))
}

fn validate_ready(id: &str, ready: &ReadyCheck) -> Result<()> {
    require(
        ready.url.starts_with("http://127.0.0.1:"),
        &format!("{id}'s readiness check must stay on this machine"),
    )?;
    require(
        (1..=MAX_READY_SECONDS).contains(&ready.seconds),
        &format!("{id}'s readiness wait is outside bounds"),
    )
}

impl SystemDefinition {
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == CATALOG_SCHEMA_VERSION,
            "unsupported system definition schema version",
        )?;
        let id = self.binding.id.as_str();
        let declared_gap = self.binding.availability != Availability::Available;
        match &self.launch {
            LaunchSpec::Container {
                image,
                name,
                ports,
                env,
                generated_secrets,
                host_env,
                env_from_host,
                ready,
                ..
            } => {
                require(
                    is_digest_pinned(image),
                    &format!("{id}'s image must be pinned by digest"),
                )?;
                require(
                    !name.is_empty()
                        && name
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'),
                    &format!("{id}'s container name is invalid"),
                )?;
                require(!ports.is_empty(), &format!("{id} publishes no port"))?;
                for mapping in ports {
                    require(
                        is_loopback_port(mapping),
                        &format!("{id} must publish {mapping} on loopback only"),
                    )?;
                }
                for names in [env.keys(), host_env.keys()] {
                    for name in names {
                        require(
                            is_variable_name(name) && !looks_secret(name),
                            &format!("{id} cannot hold {name} in a file: generate it for the run"),
                        )?;
                    }
                }
                for (inside, outside) in generated_secrets.iter().chain(env_from_host.iter()) {
                    require(
                        is_variable_name(inside) && is_variable_name(outside),
                        &format!("{id} names an invalid variable"),
                    )?;
                }
                validate_ready(id, ready)?;
            }
            LaunchSpec::Process { command, ready, .. } => {
                require(!command.is_empty(), &format!("{id} has no command"))?;
                validate_ready(id, ready)?;
            }
            LaunchSpec::RrflowEngine { .. } => {}
            LaunchSpec::None => {
                require(
                    declared_gap,
                    &format!("{id} can run, so it has to say how to start"),
                )?;
            }
        }
        Ok(())
    }
}

/// Builds a manifest from a test and the systems it names. A test may set only a system's index,
/// search settings and availability, so everything else has one owner.
pub fn resolve_test(
    test: &TestDefinition,
    systems: &BTreeMap<Id, SystemDefinition>,
) -> Result<VectorComparisonManifest> {
    require(
        test.schema_version == CATALOG_SCHEMA_VERSION,
        "unsupported test definition schema version",
    )?;
    let mut seen = BTreeSet::new();
    let mut bindings = Vec::with_capacity(test.systems.len());
    for entry in &test.systems {
        let name = entry.system.as_str();
        require(seen.insert(name), &format!("{name} is listed twice"))?;
        let definition = systems
            .get(&entry.system)
            .ok_or_else(|| format!("no eval/lab/systems/{name}.json exists"))?;
        definition.validate()?;
        require(
            definition.binding.id == entry.system,
            &format!("{name}'s file binds a different system"),
        )?;
        let mut binding = definition.binding.clone();
        if let Some(index) = &entry.index {
            binding.index = index.clone();
        }
        if let Some(ladder) = &entry.search_ladder {
            binding.search_ladder = ladder.clone();
        }
        if let Some(availability) = &entry.availability {
            binding.availability = availability.clone();
        }
        bindings.push(binding);
    }
    let manifest = VectorComparisonManifest {
        schema_version: super::manifest::MANIFEST_SCHEMA_VERSION,
        test: test.test.clone(),
        corpus: test.corpus.clone(),
        workload: test.workload.clone(),
        quality: test.quality.clone(),
        systems: bindings,
    };
    manifest.validate()?;
    Ok(manifest)
}

fn substitute(
    text: &str,
    lab: &str,
    data: &str,
    ef: &str,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<String> {
    let mut out = text
        .replace("{lab}", lab)
        .replace("{data}", data)
        .replace("{ef}", ef);
    while let Some(start) = out.find("{env:") {
        let end = out[start..]
            .find('}')
            .ok_or_else(|| format!("unterminated variable in {text}"))?
            + start;
        let name = out[start + 5..end].to_string();
        let value = env(&name).ok_or_else(|| format!("set {name} first"))?;
        out.replace_range(start..=end, &value);
    }
    Ok(out)
}

/// A run's pod: one container's network namespace, which every system container joins, so
/// each serves on the lab's own loopback and nothing is published on this machine. Its data
/// lives in a named Docker volume, because a folder inside the lab's container is not one the
/// Docker host can mount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pod<'a> {
    pub container: &'a str,
    pub volume: &'a str,
}

/// A `127.0.0.1:HOST:CONTAINER` mapping's two ports.
fn mapped_ports(mapping: &str) -> Option<(&str, &str)> {
    let mut parts = mapping.split(':').skip(1);
    Some((parts.next()?, parts.next()?))
}

/// What `run_system.sh` needs to know, as plain `key=value` lines with no secret in them.
pub fn launch_info(definition: &SystemDefinition) -> Vec<(String, String)> {
    let mut info = Vec::new();
    let mut add = |key: &str, value: String| info.push((key.to_string(), value));
    match &definition.launch {
        LaunchSpec::Container {
            name,
            ports,
            generated_secrets,
            host_env,
            env_from_host,
            ready,
            ..
        } => {
            add("kind", "container".to_string());
            add("container", name.clone());
            add(
                "ports",
                ports
                    .iter()
                    .filter_map(|mapping| mapping.split(':').nth(1))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            let pairs = |map: &BTreeMap<String, String>| {
                map.iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            add("secrets", pairs(generated_secrets));
            add("env_from_host", pairs(env_from_host));
            add("host_env", pairs(host_env));
            add("ready_url", ready.url.clone());
            add("ready_contains", ready.contains.clone().unwrap_or_default());
            add("ready_seconds", ready.seconds.to_string());
        }
        LaunchSpec::Process {
            port,
            pid_file,
            embedded_under_vectordbbench,
            ready,
            ..
        } => {
            add("kind", "process".to_string());
            add("ports", port.to_string());
            add("pid_file", pid_file.clone().unwrap_or_default());
            add(
                "embedded_under_vectordbbench",
                embedded_under_vectordbbench.to_string(),
            );
            add("ready_url", ready.url.clone());
            add("ready_contains", ready.contains.clone().unwrap_or_default());
            add("ready_seconds", ready.seconds.to_string());
        }
        LaunchSpec::RrflowEngine { port } => {
            add("kind", "rrflow_engine".to_string());
            add("ports", port.to_string());
        }
        LaunchSpec::None => add("kind", "none".to_string()),
    }
    info
}

/// The arguments after `docker run -d`, or a process's whole command. Separate words, so nothing
/// here passes through a shell. A secret goes by name only and never appears in them. In a pod a
/// container publishes nothing, so it must listen on the port its endpoint names.
pub fn launch_arguments(
    definition: &SystemDefinition,
    data_directory: &str,
    lab_directory: &str,
    cpuset: Option<&str>,
    pod: Option<Pod<'_>>,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<String>> {
    definition.validate()?;
    match &definition.launch {
        LaunchSpec::Container {
            image,
            name,
            ports,
            volume,
            env: settings,
            generated_secrets,
            env_from_host,
            user,
            arguments,
            ..
        } => {
            let id = definition.binding.id.as_str();
            // Two runs on one host each get their own containers.
            let name = match pod {
                Some(pod) => format!("{}-{name}", pod.container),
                None => name.clone(),
            };
            let mut words = vec!["--name".to_string(), name];
            if let Some(cpuset) = cpuset {
                words.extend(["--cpuset-cpus".to_string(), cpuset.to_string()]);
            }
            for (key, value) in settings {
                words.extend(["-e".to_string(), format!("{key}={value}")]);
            }
            for inside in generated_secrets.keys().chain(env_from_host.keys()) {
                words.extend(["-e".to_string(), inside.clone()]);
            }
            match pod {
                Some(pod) => {
                    for mapping in ports {
                        let (outside, inside) = mapped_ports(mapping)
                            .ok_or_else(|| format!("{id} has an invalid port mapping"))?;
                        require(
                            outside == inside,
                            &format!(
                                "{id} listens on {inside} but is reached on {outside}; \
                                 a pod maps no ports, so it must listen on {outside}"
                            ),
                        )?;
                    }
                    words.extend([
                        "--network".to_string(),
                        format!("container:{}", pod.container),
                    ]);
                }
                None => {
                    for mapping in ports {
                        words.extend(["-p".to_string(), mapping.clone()]);
                    }
                }
            }
            if let Some(user) = user {
                words.extend(["--user".to_string(), user.clone()]);
            }
            if let Some(volume) = volume {
                words.extend(match pod {
                    Some(pod) => [
                        "--mount".to_string(),
                        format!(
                            "type=volume,src={},dst={}",
                            pod.volume, volume.container_path
                        ),
                    ],
                    None => [
                        "-v".to_string(),
                        format!("{data_directory}:{}", volume.container_path),
                    ],
                });
            }
            words.push(image.clone());
            words.extend(arguments.iter().cloned());
            Ok(words)
        }
        LaunchSpec::Process { command, .. } => command
            .iter()
            .map(|word| substitute(word, lab_directory, data_directory, "", env))
            .collect(),
        LaunchSpec::RrflowEngine { .. } | LaunchSpec::None => Ok(Vec::new()),
    }
}

/// The arguments that drive this system under VectorDBBench.
pub fn vectordbbench_arguments(
    definition: &SystemDefinition,
    data_directory: &str,
    ef: &str,
) -> Result<Vec<String>> {
    require(
        !definition.vectordbbench.is_empty(),
        &format!(
            "no VectorDBBench client is set up for {}",
            definition.binding.id.as_str()
        ),
    )?;
    definition
        .vectordbbench
        .iter()
        .map(|word| substitute(word, "", data_directory, ef, &|_| None))
        .collect()
}
