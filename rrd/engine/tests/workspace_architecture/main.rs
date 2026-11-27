use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct WorkspaceMetadata {
    root: PathBuf,
    packages: BTreeMap<String, PackageDependencies>,
    target_sources: BTreeSet<PathBuf>,
    dependency_inputs: Vec<DependencyInput>,
}

impl WorkspaceMetadata {
    /// A path inside a workspace crate, relative to the repository root and found by package
    /// name, so that moving a crate to another folder changes no rule in this test.
    fn in_crate(&self, package: &str, relative: &str) -> &'static str {
        let manifest = &self
            .packages
            .get(package)
            .unwrap_or_else(|| panic!("{package} is not a workspace package"))
            .manifest;
        let directory = manifest
            .parent()
            .expect("a package manifest has a directory")
            .to_string_lossy()
            .replace('\\', "/");
        let path = if relative.is_empty() {
            directory
        } else {
            format!("{directory}/{relative}")
        };
        Box::leak(path.into_boxed_str())
    }

    /// The crates `architecture.toml` lists above `rrd-engine`, without the daemon's own
    /// crates: the components an outward consumer reaches only through the engine.
    fn components_below_the_engine(&self) -> BTreeSet<String> {
        let source = fs::read_to_string(self.root.join("architecture.toml"))
            .expect("the architecture manifest must be readable");
        let manifest: toml::Value =
            toml::from_str(&source).expect("the architecture manifest must be TOML");
        let names = manifest["crate"]
            .as_array()
            .expect("crates must be a list")
            .iter()
            .map(|entry| entry["name"].as_str().expect("a crate name is text"))
            .collect::<Vec<_>>();
        let engine = names
            .iter()
            .position(|name| *name == "rrd-engine")
            .expect("rrd-engine must be listed");
        names[..engine]
            .iter()
            .filter(|name| !name.starts_with("rrd-"))
            .map(|name| (*name).to_owned())
            .collect()
    }
}

struct PackageDependencies {
    manifest: PathBuf,
    description: String,
    features: BTreeMap<String, BTreeSet<String>>,
    workspace: BTreeSet<String>,
    all: BTreeSet<String>,
    targets: BTreeSet<String>,
}

struct DependencyInput {
    owner: String,
    name: String,
    kind: String,
    optional: bool,
    features: BTreeSet<String>,
    path: Option<PathBuf>,
    source: Option<String>,
}

struct GitIndexEntry {
    mode: String,
    path: PathBuf,
}

mod dependencies;
mod retired_and_inputs;
mod storage;
mod submodules;
mod support;
mod text;

use submodules::*;
use support::*;
