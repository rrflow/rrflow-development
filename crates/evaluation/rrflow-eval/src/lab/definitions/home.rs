//! The steps a person would otherwise repeat: where things live, which key signs, which number
//! comes next, and whether a patch applies. It holds no product's types.
use super::pack::{InstallOutcome, PackContent, PackStore, TrustedKeys};
use crate::lab::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NOT_BUILT: &str = "not built yet";

#[derive(Debug, Clone)]
pub struct DefinitionHome {
    root: PathBuf,
    product: String,
}

pub struct PublishOptions {
    pub days: u64,
    pub now_unix_s: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishKind {
    Published,
    Unchanged,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishOutcome {
    pub kind: PublishKind,
    pub sequence: u64,
    pub key_id: String,
    /// True when this publish made the key, so a person can be told to back it up.
    pub created_key: bool,
    /// False when only this machine trusts the key.
    pub key_pinned: bool,
    pub days_left: u64,
    pub full_pack: Option<PathBuf>,
    pub full_bytes: Option<u64>,
    pub patch_pack: Option<PathBuf>,
    pub patch_bytes: Option<u64>,
    pub install: Option<InstallOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IgnoredPack {
    pub file: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateOutcome {
    /// Absent when nothing newer applied.
    pub installed: Option<InstallOutcome>,
    pub current_sequence: Option<u64>,
    /// How many pack files the folder held.
    pub checked: usize,
    /// Files a person should know were not used, with why.
    pub ignored: Vec<IgnoredPack>,
}

impl DefinitionHome {
    pub fn at(root: PathBuf, product: &str) -> Self {
        Self {
            root,
            product: product.to_owned(),
        }
    }

    /// The override if there is one, or else this user's own data folder.
    pub fn resolve(
        _override_root: Option<PathBuf>,
        _product: &str,
        _variable: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self> {
        Err(NOT_BUILT.into())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn outbox(&self) -> PathBuf {
        self.root.join("outbox")
    }

    pub fn signing_key_path(&self) -> PathBuf {
        self.root.join("publisher").join("signing.key")
    }

    pub fn store(&self) -> Result<PackStore> {
        PackStore::open(self.root.join("store"), &self.product)
    }

    pub fn trusted_keys(&self, _pinned: &TrustedKeys) -> Result<TrustedKeys> {
        Err(NOT_BUILT.into())
    }

    pub fn add_trusted_keys(&self, _more: &TrustedKeys) -> Result<Vec<String>> {
        Err(NOT_BUILT.into())
    }

    pub fn publish(
        &self,
        _files: &BTreeMap<String, String>,
        _content: &dyn PackContent,
        _pinned: &TrustedKeys,
        _options: &PublishOptions,
    ) -> Result<PublishOutcome> {
        Err(NOT_BUILT.into())
    }

    pub fn update(
        &self,
        _from: &Path,
        _content: &dyn PackContent,
        _pinned: &TrustedKeys,
        _now_unix_s: u64,
    ) -> Result<UpdateOutcome> {
        Err(NOT_BUILT.into())
    }
}
