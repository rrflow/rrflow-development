//! A signed definition pack and the store that installs it. It holds no product's types: a product
//! supplies its name, its schema and a check of its content.
//!
//! The rules come from how antivirus definitions and software updates stay safe (ClamAV's CVD and
//! `.cdiff`, The Update Framework). A pack is text with no code. Its manifest is parsed only after
//! its signature checks, and that manifest names every file by size and SHA-256. A lower sequence
//! is refused, so an old pack cannot replace a new one, and a pack expires, so one cannot be held
//! back forever. A patch carries only changed files and reuses the rest by digest, so there is no
//! patch language to get wrong. A failed install changes nothing.
use crate::lab::{require, sha256_hex, Result};
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const PACK_FORMAT: u16 = 1;
pub const MAX_PACK_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_PACK_FILES: usize = 4_096;
pub const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PATH_BYTES: usize = 200;
const MAX_REASON_BYTES: usize = 1_024;
const STORE_FORMAT: u16 = 1;
/// Keeps a signature made for a pack from meaning anything anywhere else.
const SIGNING_DOMAIN: &[u8] = b"definition-pack-v1\0";

/// What a product supplies. The envelope and the store need nothing else from it.
pub trait PackContent {
    fn product(&self) -> &str;
    /// The newest content schema this build understands.
    fn schema(&self) -> u16;
    /// Refuses content this build could not use. It runs before anything is stored.
    fn check(&self, files: &BTreeMap<String, String>) -> Result<()>;
    /// Plain lines for what an update changes that a person should look at before it runs.
    fn notable_changes(
        &self,
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
    ) -> Result<Vec<String>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedKey {
    pub id: String,
    pub public_key: String,
}

/// The publisher keys a store accepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedKeys {
    pub keys: Vec<TrustedKey>,
}

pub struct GeneratedKey {
    pub key_id: String,
    pub public_hex: String,
    pub private_hex: String,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex<const N: usize>(text: &str, what: &str) -> Result<[u8; N]> {
    let invalid = || format!("{what} is not {N} bytes of lowercase hexadecimal");
    if text.len() != N * 2
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid());
    }
    let mut bytes = [0_u8; N];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).map_err(|_| invalid())?;
    }
    Ok(bytes)
}

fn key_id(public: &VerifyingKey) -> String {
    sha256_hex(public.as_bytes())[..16].to_owned()
}

pub fn generate_key() -> Result<GeneratedKey> {
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|error| format!("no system randomness: {error}"))?;
    let signing = SigningKey::from_bytes(&seed);
    let public = signing.verifying_key();
    Ok(GeneratedKey {
        key_id: key_id(&public),
        public_hex: hex(public.as_bytes()),
        private_hex: hex(&seed),
    })
}

/// A path is lowercase and relative, and it names a JSON file. A store keeps files by digest, so
/// a path never reaches the file system, but a pack should still be unable to name anything else.
fn validate_path(path: &str) -> Result<()> {
    let refuse = || Err(format!("pack path {path:?} is not allowed"));
    if path.is_empty() || path.len() > MAX_PATH_BYTES || !path.ends_with(".json") {
        return refuse();
    }
    for segment in path.split('/') {
        let safe = segment.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        });
        if segment.is_empty() || segment.starts_with('.') || !safe {
            return refuse();
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

/// What the signature covers. It lists every file of the full set, including those a patch
/// leaves out, so the result of a patch is signed as exactly as a full pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    pub format: u16,
    pub product: String,
    pub content_schema: u16,
    pub sequence: u64,
    pub published_at_unix_s: u64,
    pub expires_at_unix_s: u64,
    pub files: Vec<PackFile>,
}

impl PackManifest {
    fn validate(&self) -> Result<()> {
        require(self.format == PACK_FORMAT, "unsupported pack format")?;
        require(!self.product.is_empty(), "a pack names its product")?;
        require(self.sequence > 0, "a pack sequence starts at 1")?;
        require(
            self.expires_at_unix_s > self.published_at_unix_s,
            "a pack expires after it is published",
        )?;
        require(
            self.files.len() <= MAX_PACK_FILES,
            "a pack lists too many files",
        )?;
        let mut previous: Option<&str> = None;
        for file in &self.files {
            validate_path(&file.path)?;
            require(
                previous.is_none_or(|before| before < file.path.as_str()),
                "a pack lists its files once each, in order",
            )?;
            previous = Some(&file.path);
            require(file.bytes <= MAX_FILE_BYTES, "a pack file is too large")?;
            unhex::<32>(&file.sha256, "a pack file digest")?;
        }
        Ok(())
    }

    fn file(&self, path: &str) -> Option<&PackFile> {
        self.files.iter().find(|file| file.path == path)
    }
}

/// One pack as it travels: one text file, readable by a person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedPack {
    pub format: u16,
    pub key_id: String,
    /// The exact text the signature covers.
    pub manifest: String,
    pub signature: String,
    /// The files this pack carries. A patch leaves out the ones that did not change.
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
    /// The pack this one patches. Its unchanged files travel as digests only.
    pub base: Option<&'a PackManifest>,
}

pub fn build_pack(request: &PackRequest<'_>) -> Result<SignedPack> {
    let mut files = Vec::with_capacity(request.files.len());
    let mut carried = BTreeMap::new();
    for (path, text) in request.files {
        validate_path(path)?;
        let entry = PackFile {
            path: path.clone(),
            bytes: text.len() as u64,
            sha256: sha256_hex(text.as_bytes()),
        };
        if request.base.and_then(|base| base.file(path)) != Some(&entry) {
            carried.insert(path.clone(), text.clone());
        }
        files.push(entry);
    }
    let manifest = PackManifest {
        format: PACK_FORMAT,
        product: request.product.to_owned(),
        content_schema: request.content_schema,
        sequence: request.sequence,
        published_at_unix_s: request.published_at_unix_s,
        expires_at_unix_s: request.expires_at_unix_s,
        files,
    };
    manifest.validate()?;
    let text = serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
    sign_manifest(text, carried, request.signing_key)
}

/// Signs manifest text exactly as given.
pub fn sign_manifest(
    manifest: String,
    files: BTreeMap<String, String>,
    signing_key: &str,
) -> Result<SignedPack> {
    let signing = SigningKey::from_bytes(&unhex::<32>(signing_key, "the signing key")?);
    let signature = signing.sign(&signed_bytes(&manifest));
    Ok(SignedPack {
        format: PACK_FORMAT,
        key_id: key_id(&signing.verifying_key()),
        manifest,
        signature: hex(&signature.to_bytes()),
        files,
    })
}

fn signed_bytes(manifest: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(SIGNING_DOMAIN.len() + manifest.len());
    bytes.extend_from_slice(SIGNING_DOMAIN);
    bytes.extend_from_slice(manifest.as_bytes());
    bytes
}

fn verified_manifest(signed: &SignedPack, keys: &TrustedKeys) -> Result<PackManifest> {
    require(signed.format == PACK_FORMAT, "unsupported pack format")?;
    let trusted = keys
        .keys
        .iter()
        .find(|key| key.id == signed.key_id)
        .ok_or_else(|| format!("pack key {} is not trusted", signed.key_id))?;
    let public = VerifyingKey::from_bytes(&unhex::<32>(&trusted.public_key, "a trusted key")?)
        .map_err(|error| format!("a trusted key is invalid: {error}"))?;
    let signature = Signature::from_bytes(&unhex::<64>(&signed.signature, "the pack signature")?);
    public
        .verify_strict(&signed_bytes(&signed.manifest), &signature)
        .map_err(|_| "the pack signature does not check".to_owned())?;
    // Parsed only now, so nothing unsigned is ever read as a manifest.
    let manifest: PackManifest = serde_json::from_str(&signed.manifest)
        .map_err(|error| format!("the signed manifest is invalid: {error}"))?;
    manifest.validate()?;
    Ok(manifest)
}

/// A pack whose signature checked and whose carried files match its manifest.
#[derive(Debug, Clone)]
pub struct VerifiedPack {
    pub manifest: PackManifest,
    pub manifest_sha256: String,
    pub carried: BTreeMap<String, String>,
}

fn check_text(file: &PackFile, text: &str) -> Result<()> {
    if text.len() as u64 != file.bytes || sha256_hex(text.as_bytes()) != file.sha256 {
        return Err(format!("{} differs from its signed digest", file.path));
    }
    Ok(())
}

pub fn read_pack(bytes: &[u8], keys: &TrustedKeys) -> Result<VerifiedPack> {
    if bytes.len() as u64 > MAX_PACK_BYTES {
        return Err("the pack exceeds its size bound".into());
    }
    let signed: SignedPack =
        serde_json::from_slice(bytes).map_err(|error| format!("not a pack: {error}"))?;
    let manifest = verified_manifest(&signed, keys)?;
    for (path, text) in &signed.files {
        let file = manifest
            .file(path)
            .ok_or_else(|| format!("the manifest does not list {path}"))?;
        check_text(file, text)?;
    }
    Ok(VerifiedPack {
        manifest_sha256: sha256_hex(signed.manifest.as_bytes()),
        manifest,
        carried: signed.files,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallKind {
    Installed,
    AlreadyInstalled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstallOutcome {
    pub kind: InstallKind,
    pub sequence: u64,
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub removed: Vec<String>,
    /// What the product says a person should look at before this runs.
    pub notable: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoreStatus {
    pub product: String,
    pub current_sequence: Option<u64>,
    /// The floor: nothing older can be installed, even after a revert.
    pub highest_sequence: u64,
    pub manifest_sha256: Option<String>,
    pub published_at_unix_s: Option<u64>,
    pub expires_at_unix_s: Option<u64>,
    /// It still works. Results made with it say it was stale.
    pub expired: bool,
    pub file_count: usize,
    pub reverted_because: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreState {
    format: u16,
    product: String,
    highest_sequence: u64,
    highest_manifest_sha256: Option<String>,
    current_sequence: Option<u64>,
    reverted_because: Option<String>,
    history: Vec<StoreEvent>,
}

/// What happened to the store, in order. A revert is here with its reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
enum StoreEvent {
    Installed {
        sequence: u64,
        manifest_sha256: String,
        at_unix_s: u64,
    },
    Reverted {
        from: u64,
        to: u64,
        reason: String,
        at_unix_s: u64,
    },
}

/// Files are kept by digest, one signed manifest per installed sequence, and one state file that
/// says which is current. The state file is written last and replaced atomically, so an
/// interrupted install leaves the previous pack in place.
pub struct PackStore {
    root: PathBuf,
    product: String,
}

impl PackStore {
    pub fn open(root: PathBuf, product: &str) -> Result<Self> {
        for directory in ["objects", "manifests"] {
            std::fs::create_dir_all(root.join(directory))
                .map_err(|error| format!("{}: {error}", root.display()))?;
        }
        let store = Self {
            root,
            product: product.to_owned(),
        };
        store.load_state()?;
        Ok(store)
    }

    fn state_path(&self) -> PathBuf {
        self.root.join("state.json")
    }

    fn object_path(&self, sha256: &str) -> PathBuf {
        self.root.join("objects").join(sha256)
    }

    fn manifest_path(&self, sequence: u64) -> PathBuf {
        self.root.join("manifests").join(format!("{sequence}.json"))
    }

    fn load_state(&self) -> Result<StoreState> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(StoreState {
                format: STORE_FORMAT,
                product: self.product.clone(),
                highest_sequence: 0,
                highest_manifest_sha256: None,
                current_sequence: None,
                reverted_because: None,
                history: Vec::new(),
            });
        }
        let bytes = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let state: StoreState =
            serde_json::from_slice(&bytes).map_err(|error| format!("the store state: {error}"))?;
        require(
            state.format == STORE_FORMAT,
            "unsupported definition store format",
        )?;
        if state.product != self.product {
            return Err(format!(
                "this store holds definitions for {}, not {}",
                state.product, self.product
            ));
        }
        Ok(state)
    }

    fn save_state(&self, state: &StoreState) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(state).map_err(|error| error.to_string())?;
        write_atomically(&self.state_path(), &bytes)
    }

    fn read_object(&self, file: &PackFile) -> Result<Option<String>> {
        let path = self.object_path(&file.sha256);
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    }

    fn load_manifest(&self, sequence: u64, keys: &TrustedKeys) -> Result<(PackManifest, String)> {
        let path = self.manifest_path(sequence);
        let bytes = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let signed: SignedPack = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let manifest = verified_manifest(&signed, keys)?;
        Ok((manifest, sha256_hex(signed.manifest.as_bytes())))
    }

    fn files_of(&self, manifest: &PackManifest) -> Result<BTreeMap<String, String>> {
        let mut files = BTreeMap::new();
        for file in &manifest.files {
            let text = self
                .read_object(file)?
                .ok_or_else(|| format!("the store has lost {}", file.path))?;
            check_text(file, &text)?;
            files.insert(file.path.clone(), text);
        }
        Ok(files)
    }

    /// The current definitions, each checked against the signed manifest as it is read.
    pub fn current_files(&self, keys: &TrustedKeys) -> Result<BTreeMap<String, String>> {
        let state = self.load_state()?;
        let sequence = state
            .current_sequence
            .ok_or("no definitions are installed")?;
        let (manifest, _) = self.load_manifest(sequence, keys)?;
        self.files_of(&manifest)
    }

    pub fn install(
        &self,
        bytes: &[u8],
        keys: &TrustedKeys,
        content: &dyn PackContent,
        now_unix_s: u64,
    ) -> Result<InstallOutcome> {
        let pack = read_pack(bytes, keys)?;
        let manifest = &pack.manifest;
        if manifest.product != self.product || manifest.product != content.product() {
            return Err(format!(
                "this pack is for product {}, not {}",
                manifest.product, self.product
            ));
        }
        if manifest.content_schema > content.schema() {
            return Err(format!(
                "this pack needs a newer build: content schema {} and this build reads {}",
                manifest.content_schema,
                content.schema()
            ));
        }
        let mut state = self.load_state()?;
        if manifest.sequence < state.highest_sequence {
            return Err(format!(
                "pack {} is older than pack {}, which this store has already seen",
                manifest.sequence, state.highest_sequence
            ));
        }
        if manifest.sequence == state.highest_sequence {
            if state.highest_manifest_sha256.as_deref() != Some(pack.manifest_sha256.as_str()) {
                return Err(format!(
                    "pack {} has different content from the pack {} already installed",
                    manifest.sequence, manifest.sequence
                ));
            }
            return Ok(InstallOutcome {
                kind: InstallKind::AlreadyInstalled,
                sequence: manifest.sequence,
                added: Vec::new(),
                changed: Vec::new(),
                removed: Vec::new(),
                notable: Vec::new(),
            });
        }
        if now_unix_s >= manifest.expires_at_unix_s {
            return Err(format!("pack {} has expired", manifest.sequence));
        }

        let mut files = BTreeMap::new();
        for file in &manifest.files {
            let text = match pack.carried.get(&file.path) {
                Some(text) => text.clone(),
                None => self.read_object(file)?.ok_or_else(|| {
                    format!(
                        "this patch needs {}, which this store does not have, so install a full pack",
                        file.path
                    )
                })?,
            };
            check_text(file, &text)?;
            files.insert(file.path.clone(), text);
        }
        content.check(&files)?;

        let before = match state.current_sequence {
            Some(sequence) => {
                let (current, _) = self.load_manifest(sequence, keys)?;
                self.files_of(&current)?
            }
            None => BTreeMap::new(),
        };
        let notable = content.notable_changes(&before, &files)?;
        let (added, changed, removed) = differences(&before, &files);

        for file in &manifest.files {
            let path = self.object_path(&file.sha256);
            if !path.exists() {
                write_atomically(&path, files[&file.path].as_bytes())?;
            }
        }
        let stored: SignedPack =
            serde_json::from_slice(bytes).map_err(|error| format!("not a pack: {error}"))?;
        let stored = SignedPack {
            files: BTreeMap::new(),
            ..stored
        };
        write_atomically(
            &self.manifest_path(manifest.sequence),
            &serde_json::to_vec_pretty(&stored).map_err(|error| error.to_string())?,
        )?;

        state.highest_sequence = manifest.sequence;
        state.highest_manifest_sha256 = Some(pack.manifest_sha256.clone());
        state.current_sequence = Some(manifest.sequence);
        state.reverted_because = None;
        state.history.push(StoreEvent::Installed {
            sequence: manifest.sequence,
            manifest_sha256: pack.manifest_sha256.clone(),
            at_unix_s: now_unix_s,
        });
        self.save_state(&state)?;
        Ok(InstallOutcome {
            kind: InstallKind::Installed,
            sequence: manifest.sequence,
            added,
            changed,
            removed,
            notable,
        })
    }

    pub fn status(&self, keys: &TrustedKeys, now_unix_s: u64) -> Result<StoreStatus> {
        let state = self.load_state()?;
        let current = state
            .current_sequence
            .map(|sequence| self.load_manifest(sequence, keys))
            .transpose()?;
        Ok(StoreStatus {
            product: state.product,
            current_sequence: state.current_sequence,
            highest_sequence: state.highest_sequence,
            manifest_sha256: current.as_ref().map(|(_, digest)| digest.clone()),
            published_at_unix_s: current.as_ref().map(|(m, _)| m.published_at_unix_s),
            expires_at_unix_s: current.as_ref().map(|(m, _)| m.expires_at_unix_s),
            expired: current
                .as_ref()
                .is_some_and(|(m, _)| now_unix_s >= m.expires_at_unix_s),
            file_count: current.as_ref().map_or(0, |(m, _)| m.files.len()),
            reverted_because: state.reverted_because,
        })
    }

    /// Steps back to a pack already installed here. It is an explicit, recorded act, and it does
    /// not lower the floor: to undo a bad pack for everyone, a publisher issues a higher sequence
    /// with the old content.
    pub fn revert(
        &self,
        sequence: u64,
        reason: &str,
        keys: &TrustedKeys,
        now_unix_s: u64,
    ) -> Result<()> {
        require(
            !reason.trim().is_empty() && reason.len() <= MAX_REASON_BYTES,
            "a revert states its reason",
        )?;
        let mut state = self.load_state()?;
        let from = state
            .current_sequence
            .ok_or("no definitions are installed")?;
        if !self.manifest_path(sequence).exists() {
            return Err(format!("pack {sequence} is not installed in this store"));
        }
        let (manifest, _) = self.load_manifest(sequence, keys)?;
        self.files_of(&manifest)?;
        state.current_sequence = Some(sequence);
        state.reverted_because = Some(reason.to_owned());
        state.history.push(StoreEvent::Reverted {
            from,
            to: sequence,
            reason: reason.to_owned(),
            at_unix_s: now_unix_s,
        });
        self.save_state(&state)
    }
}

fn differences(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let before_paths: BTreeSet<_> = before.keys().collect();
    let after_paths: BTreeSet<_> = after.keys().collect();
    let added = after_paths
        .difference(&before_paths)
        .map(|path| (*path).clone())
        .collect();
    let removed = before_paths
        .difference(&after_paths)
        .map(|path| (*path).clone())
        .collect();
    let changed = after
        .iter()
        .filter(|(path, text)| before.get(*path).is_some_and(|old| old != *text))
        .map(|(path, _)| path.clone())
        .collect();
    (added, changed, removed)
}

/// Replaces `path` whole or not at all.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("a store path has no parent")?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("a store path has no name")?;
    let temporary = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    let write = || -> std::io::Result<()> {
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    };
    write().map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        format!("{}: {error}", path.display())
    })
}
