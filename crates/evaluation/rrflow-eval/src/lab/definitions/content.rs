//! What RRFlow's lab carries in a definition pack: its systems, its tests and its list of outside
//! sources, each checked the way the lab itself would read it.
use super::pack::{PackContent, TrustedKeys};
use crate::lab::catalog::{resolve_test, SystemDefinition, TestDefinition};
use crate::lab::references::ReferenceFile;
use crate::lab::sources::SourceList;
use crate::lab::Result;
use crate::storage::Id;
use std::collections::BTreeMap;
use std::path::Path;

pub const RRFLOW_PRODUCT: &str = "rrflow";
pub const RRFLOW_CONTENT_SCHEMA: u16 = 2;
const SOURCES: &str = "sources.json";

pub struct RrflowContent;

/// The publisher keys this build trusts with nothing else set up.
pub fn pinned_keys() -> Result<TrustedKeys> {
    serde_json::from_str(include_str!("../../../assets/lab-trusted-keys.json"))
        .map_err(|error| format!("the pinned keys: {error}"))
}

pub(crate) fn stem<'a>(path: &'a str, folder: &str) -> Option<&'a str> {
    path.strip_prefix(folder)?
        .strip_prefix('/')?
        .strip_suffix(".json")
        .filter(|name| !name.contains('/'))
}

fn parse_system(path: &str, text: &str) -> Result<SystemDefinition> {
    let system: SystemDefinition =
        serde_json::from_str(text).map_err(|error| format!("{path}: {error}"))?;
    system
        .validate()
        .map_err(|error| format!("{path}: {error}"))?;
    Ok(system)
}

pub(crate) fn systems_of(
    files: &BTreeMap<String, String>,
) -> Result<BTreeMap<Id, SystemDefinition>> {
    let mut systems = BTreeMap::new();
    for (path, text) in files {
        if let Some(name) = stem(path, "systems") {
            let system = parse_system(path, text)?;
            if system.binding.id.as_str() != name {
                return Err(format!(
                    "{path} names itself {}, and a system's file is named for it",
                    system.binding.id.as_str()
                ));
            }
            systems.insert(system.binding.id.clone(), system);
        }
    }
    Ok(systems)
}

impl PackContent for RrflowContent {
    fn product(&self) -> &str {
        RRFLOW_PRODUCT
    }

    fn schema(&self) -> u16 {
        RRFLOW_CONTENT_SCHEMA
    }

    fn check(&self, files: &BTreeMap<String, String>) -> Result<()> {
        let systems = systems_of(files)?;
        for (path, text) in files {
            if stem(path, "systems").is_some() {
                continue;
            }
            if path == SOURCES {
                let list: SourceList =
                    serde_json::from_str(text).map_err(|error| format!("{path}: {error}"))?;
                list.validate()
                    .map_err(|error| format!("{path}: {error}"))?;
            } else if let Some(name) = stem(path, "tests") {
                let test: TestDefinition =
                    serde_json::from_str(text).map_err(|error| format!("{path}: {error}"))?;
                if test.test.id.as_str() != name {
                    return Err(format!(
                        "{path} names itself {}, and a test's file is named for it",
                        test.test.id.as_str()
                    ));
                }
                resolve_test(&test, &systems).map_err(|error| format!("{path}: {error}"))?;
            } else if let Some(name) = stem(path, "references") {
                let reference = read_reference(path, text)?;
                if reference.source.id.as_str() != name {
                    return Err(format!(
                        "{path} names its source {}, and a source's file is named for it",
                        reference.source.id.as_str()
                    ));
                }
            } else {
                return Err(format!("{path} is not a definition file"));
            }
        }
        Ok(())
    }

    /// A pack can change what starts on this machine, so it says so. Installing runs nothing.
    fn notable_changes(
        &self,
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
    ) -> Result<Vec<String>> {
        let old = systems_of(before)?;
        let mut notable = Vec::new();
        for (id, system) in systems_of(after)? {
            match old.get(&id) {
                None => {
                    if let Some(summary) = system.launch.summary() {
                        notable.push(format!("new system {} would start {summary}", id.as_str()));
                    }
                }
                // A change gets every detail, since it may be the one thing that differs.
                Some(previous) if previous.launch != system.launch => {
                    let launch =
                        serde_json::to_string(&system.launch).map_err(|error| error.to_string())?;
                    notable.push(format!(
                        "system {} now starts something different: {launch}",
                        id.as_str()
                    ));
                }
                Some(_) => {}
            }
        }
        Ok(notable)
    }
}

fn read_reference(path: &str, text: &str) -> Result<ReferenceFile> {
    let reference: ReferenceFile =
        serde_json::from_str(text).map_err(|error| format!("{path}: {error}"))?;
    reference
        .validate()
        .map_err(|error| format!("{path}: {error}"))?;
    Ok(reference)
}

/// The published results a pack carries, each source checked as it is read.
pub fn references_of(files: &BTreeMap<String, String>) -> Result<Vec<ReferenceFile>> {
    files
        .iter()
        .filter(|(path, _)| stem(path, "references").is_some())
        .map(|(path, text)| read_reference(path, text))
        .collect()
}

/// The definition files in a lab folder, by the path a pack carries them under.
pub fn lab_files(lab: &Path) -> Result<BTreeMap<String, String>> {
    let read = |path: &Path| {
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
    };
    let mut files = BTreeMap::new();
    files.insert(SOURCES.to_owned(), read(&lab.join(SOURCES))?);
    for folder in ["systems", "tests", "references"] {
        let directory = lab.join(folder);
        if folder == "references" && !directory.exists() {
            continue;
        }
        let entries = std::fs::read_dir(&directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        for entry in entries {
            let path = entry.map_err(|error| error.to_string())?.path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("a definition file has no name")?;
            if name.ends_with(".json") {
                files.insert(format!("{folder}/{name}"), read(&path)?);
            }
        }
    }
    Ok(files)
}
