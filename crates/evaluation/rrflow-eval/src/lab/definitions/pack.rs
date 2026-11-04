//! Not built yet. The tests in `tests/lab_definitions.rs` come first.
use crate::lab::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const MAX_PACK_BYTES: u64 = 16 * 1024 * 1024;

pub trait PackContent {
    fn product(&self) -> &str;
    fn schema(&self) -> u16;
    fn check(&self, files: &BTreeMap<String, String>) -> Result<()>;
    fn notable_changes(
        &self,
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
    ) -> Result<Vec<String>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedKey {
    pub id: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedKeys {
    pub keys: Vec<TrustedKey>,
}

pub struct GeneratedKey {
    pub key_id: String,
    pub public_hex: String,
    pub private_hex: String,
}

pub fn generate_key() -> Result<GeneratedKey> {
    Ok(GeneratedKey {
        key_id: String::new(),
        public_hex: String::new(),
        private_hex: String::new(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPack {
    pub key_id: String,
    pub manifest: String,
    pub signature: String,
    pub files: BTreeMap<String, String>,
}

pub struct PackRequest<'a> {
    pub product: &'a str,
    pub content_schema: u16,
    pub sequence: u64,
    pub published_at_unix_s: u64,
    pub expires_at_unix_s: u64,
    pub files: &'a BTreeMap<String, String>,
    pub signing_key: &'a str,
    pub base: Option<&'a PackManifest>,
}

pub fn build_pack(_request: &PackRequest<'_>) -> Result<SignedPack> {
    Err("not built yet".into())
}

pub fn sign_manifest(
    _manifest: String,
    _files: BTreeMap<String, String>,
    _signing_key: &str,
) -> Result<SignedPack> {
    Err("not built yet".into())
}

#[derive(Debug)]
pub struct VerifiedPack {
    pub manifest: PackManifest,
}

pub fn read_pack(_bytes: &[u8], _keys: &TrustedKeys) -> Result<VerifiedPack> {
    Err("not built yet".into())
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstallKind {
    Installed,
    AlreadyInstalled,
}

#[derive(Debug)]
pub struct InstallOutcome {
    pub kind: InstallKind,
    pub sequence: u64,
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub removed: Vec<String>,
    pub notable: Vec<String>,
}

#[derive(Debug)]
pub struct StoreStatus {
    pub current_sequence: Option<u64>,
    pub highest_sequence: u64,
    pub file_count: usize,
    pub expired: bool,
    pub reverted_because: Option<String>,
}

pub struct PackStore {
    _root: PathBuf,
}

impl PackStore {
    pub fn open(root: PathBuf, _product: &str) -> Result<Self> {
        Ok(Self { _root: root })
    }

    pub fn install(
        &self,
        _bytes: &[u8],
        _keys: &TrustedKeys,
        _content: &dyn PackContent,
        _now_unix_s: u64,
    ) -> Result<InstallOutcome> {
        Err("not built yet".into())
    }

    pub fn current_files(&self, _keys: &TrustedKeys) -> Result<BTreeMap<String, String>> {
        Err("not built yet".into())
    }

    pub fn status(&self, _keys: &TrustedKeys, _now_unix_s: u64) -> Result<StoreStatus> {
        Err("not built yet".into())
    }

    pub fn revert(
        &self,
        _sequence: u64,
        _reason: &str,
        _keys: &TrustedKeys,
        _now_unix_s: u64,
    ) -> Result<()> {
        Err("not built yet".into())
    }
}
