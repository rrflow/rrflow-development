//! The steps a person would otherwise repeat: where things live, which key signs, which number
//! comes next, and whether a patch applies. It holds no product's types.
//!
//! A home is one folder per user. It keeps the installed store, this machine's added trust, and,
//! on a machine that publishes, the signing key and an outbox to send from. The key is never in a
//! repository and never in the outbox. A folder of packs never brings its own trust: its public
//! keys travel beside the packs only so a person can choose to trust them.
use super::pack::{
    build_pack, generate_key, key_from_private, read_pack, write_atomically, InstallKind,
    InstallOutcome, PackContent, PackRequest, PackStore, TrustedKey, TrustedKeys, MAX_PACK_BYTES,
    MAX_PACK_FILES,
};
use crate::lab::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// A pack with fewer days than this left is published again even with nothing changed, so it
/// does not expire on the machines using it.
const REFRESH_WITHIN_DAYS: u64 = 30;
const DAY_SECONDS: u64 = 86_400;
const PACK_SUFFIX: &str = ".pack.json";
const TRUST_FILE: &str = "trusted-keys.json";

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

struct Candidate {
    file: String,
    bytes: Vec<u8>,
    sequence: u64,
    carried: usize,
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
        override_root: Option<PathBuf>,
        product: &str,
        variable: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self> {
        if let Some(root) = override_root {
            return Ok(Self::at(root, product));
        }
        let base = user_data_folder(variable)
            .ok_or("cannot find this user's data folder; name one with the home override")?;
        Ok(Self::at(base.join(product).join("definitions"), product))
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

    fn trust_path(&self) -> PathBuf {
        self.root.join(TRUST_FILE)
    }

    fn own_trust(&self) -> Result<TrustedKeys> {
        let path = self.trust_path();
        match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|error| format!("{}: {error}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(TrustedKeys { keys: Vec::new() })
            }
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    }

    /// The keys pinned in this build, and any this machine was told to trust.
    pub fn trusted_keys(&self, pinned: &TrustedKeys) -> Result<TrustedKeys> {
        let mut keys = pinned.keys.clone();
        for key in self.own_trust()?.keys {
            if !keys.iter().any(|known| known.id == key.id) {
                keys.push(key);
            }
        }
        Ok(TrustedKeys { keys })
    }

    /// Adds keys to this machine's trust, and returns the ids that were new.
    pub fn add_trusted_keys(&self, more: &TrustedKeys) -> Result<Vec<String>> {
        let mut own = self.own_trust()?;
        let mut added = Vec::new();
        for key in &more.keys {
            let derived = key.checked_id()?;
            if derived != key.id {
                return Err(format!(
                    "key {} is named for a different public key, {derived}",
                    key.id
                ));
            }
            if !own.keys.iter().any(|known| known.id == key.id) {
                own.keys.push(key.clone());
                added.push(key.id.clone());
            }
        }
        if !added.is_empty() {
            write_json(&self.trust_path(), &own)?;
        }
        Ok(added)
    }

    /// This machine's signing key, made on first use. Returns the key, its public half, and
    /// whether it was just made.
    fn signing_key(&self) -> Result<(String, TrustedKey, bool)> {
        let path = self.signing_key_path();
        if path.exists() {
            refuse_if_others_can_read(&path)?;
            let private = std::fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            let private = private.trim().to_owned();
            let public = key_from_private(&private)?;
            return Ok((private, public, false));
        }
        let key = generate_key()?;
        create_private(&path, &key.private_hex)?;
        let public = TrustedKey {
            id: key.key_id,
            public_key: key.public_hex,
        };
        Ok((key.private_hex, public, true))
    }

    /// The next number nothing has used, whether or not the outbox is still here.
    fn next_sequence(&self, store_highest: u64) -> Result<u64> {
        let mut highest = store_highest;
        if let Ok(entries) = std::fs::read_dir(self.outbox()) {
            for entry in entries {
                let name = entry.map_err(|error| error.to_string())?.file_name();
                let name = name.to_string_lossy();
                let number = name
                    .strip_prefix(&format!("{}-", self.product))
                    .and_then(|rest| rest.split(['.', '-']).next())
                    .and_then(|digits| digits.parse::<u64>().ok());
                highest = highest.max(number.unwrap_or(0));
            }
        }
        highest
            .checked_add(1)
            .ok_or_else(|| "the pack sequence is exhausted".to_owned())
    }

    /// Signs what changed, under the next number, and installs it here. With nothing changed and
    /// time left on the current pack, it publishes nothing.
    pub fn publish(
        &self,
        files: &BTreeMap<String, String>,
        content: &dyn PackContent,
        pinned: &TrustedKeys,
        options: &PublishOptions,
    ) -> Result<PublishOutcome> {
        content.check(files)?;
        let (private, public, created_key) = self.signing_key()?;
        let key_pinned = pinned.keys.iter().any(|key| key.id == public.id);
        self.add_trusted_keys(&TrustedKeys {
            keys: vec![public.clone()],
        })?;
        let keys = self.trusted_keys(pinned)?;
        let store = self.store()?;
        let status = store.status(&keys, options.now_unix_s)?;
        let base = store.highest_manifest(&keys)?;

        if let Some(previous) = &base {
            let days_left = previous
                .expires_at_unix_s
                .saturating_sub(options.now_unix_s)
                / DAY_SECONDS;
            if previous.same_files_as(files) && days_left >= REFRESH_WITHIN_DAYS {
                return Ok(PublishOutcome {
                    kind: PublishKind::Unchanged,
                    sequence: previous.sequence,
                    key_id: public.id,
                    created_key,
                    key_pinned,
                    days_left,
                    full_pack: None,
                    full_bytes: None,
                    patch_pack: None,
                    patch_bytes: None,
                    install: None,
                });
            }
        }

        let sequence = self.next_sequence(status.highest_sequence)?;
        let expires = options
            .days
            .checked_mul(DAY_SECONDS)
            .and_then(|seconds| seconds.checked_add(options.now_unix_s))
            .ok_or("the pack's lifetime is too long")?;
        let request = |base| PackRequest {
            product: &self.product,
            content_schema: content.schema(),
            sequence,
            published_at_unix_s: options.now_unix_s,
            expires_at_unix_s: expires,
            files,
            signing_key: &private,
            base,
        };
        let encode = |pack: &_| serde_json::to_vec_pretty(pack).map_err(|error| error.to_string());
        let full = encode(&build_pack(&request(None))?)?;
        let patch = base
            .as_ref()
            .map(|previous| encode(&build_pack(&request(Some(previous)))?))
            .transpose()?;

        // A pack that would not install here never reaches another machine.
        let install = store.install(&full, &keys, content, options.now_unix_s)?;

        let outbox = self.outbox();
        std::fs::create_dir_all(&outbox)
            .map_err(|error| format!("{}: {error}", outbox.display()))?;
        write_json(
            &outbox.join(format!("{}-{TRUST_FILE}", self.product)),
            &TrustedKeys {
                keys: vec![public.clone()],
            },
        )?;
        let full_pack = outbox.join(format!("{}-{sequence}{PACK_SUFFIX}", self.product));
        write_atomically(&full_pack, &full)?;
        let patch_pack = match (&patch, &base) {
            (Some(bytes), Some(previous)) => {
                let path = outbox.join(format!(
                    "{}-{sequence}-from-{}{PACK_SUFFIX}",
                    self.product, previous.sequence
                ));
                write_atomically(&path, bytes)?;
                Some(path)
            }
            _ => None,
        };
        Ok(PublishOutcome {
            kind: PublishKind::Published,
            sequence,
            key_id: public.id,
            created_key,
            key_pinned,
            days_left: options.days,
            full_bytes: Some(full.len() as u64),
            full_pack: Some(full_pack),
            patch_bytes: patch.as_ref().map(|bytes| bytes.len() as u64),
            patch_pack,
            install: Some(install),
        })
    }

    /// Reads the packs in a folder, and installs the newest one that checks. A patch comes first
    /// at each number, since it needs less, and a full pack covers a patch that cannot apply.
    /// Nothing in the folder can make itself trusted, and nothing here is fetched.
    pub fn update(
        &self,
        from: &Path,
        content: &dyn PackContent,
        pinned: &TrustedKeys,
        now_unix_s: u64,
    ) -> Result<UpdateOutcome> {
        let keys = self.trusted_keys(pinned)?;
        let store = self.store()?;
        let highest = store.status(&keys, now_unix_s)?.highest_sequence;
        let entries =
            std::fs::read_dir(from).map_err(|error| format!("{}: {error}", from.display()))?;

        let mut candidates = Vec::new();
        let mut ignored = Vec::new();
        let mut checked = 0;
        for entry in entries {
            let path = entry.map_err(|error| error.to_string())?.path();
            let Some(file) = path
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| name.ends_with(PACK_SUFFIX))
                .map(str::to_owned)
            else {
                continue;
            };
            checked += 1;
            if checked > MAX_PACK_FILES {
                return Err(format!("{} holds too many packs", from.display()));
            }
            let mut refuse = |reason: String| {
                ignored.push(IgnoredPack {
                    file: file.clone(),
                    reason,
                });
            };
            let metadata = std::fs::metadata(&path).map_err(|error| error.to_string())?;
            if metadata.len() > MAX_PACK_BYTES {
                refuse("it is larger than a pack may be".to_owned());
                continue;
            }
            let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
            match read_pack(&bytes, &keys) {
                Ok(pack) if pack.manifest.product != self.product => {
                    refuse(format!("it is for {}", pack.manifest.product));
                }
                Ok(pack) if pack.manifest.sequence < highest => {}
                Ok(pack) => candidates.push(Candidate {
                    file,
                    sequence: pack.manifest.sequence,
                    carried: pack.carried.len(),
                    bytes,
                }),
                Err(reason) => refuse(reason),
            }
        }

        candidates.sort_by(|left, right| {
            right
                .sequence
                .cmp(&left.sequence)
                .then(left.carried.cmp(&right.carried))
        });
        let mut installed = None;
        let mut failed: Vec<(u64, IgnoredPack)> = Vec::new();
        for candidate in &candidates {
            match store.install(&candidate.bytes, &keys, content, now_unix_s) {
                Ok(outcome) => {
                    // A patch that could not apply is routine once its full pack has.
                    failed.retain(|(sequence, _)| *sequence != candidate.sequence);
                    if outcome.kind == InstallKind::Installed {
                        installed = Some(outcome);
                    }
                    break;
                }
                Err(reason) => failed.push((
                    candidate.sequence,
                    IgnoredPack {
                        file: candidate.file.clone(),
                        reason,
                    },
                )),
            }
        }
        ignored.extend(failed.into_iter().map(|(_, pack)| pack));
        ignored.sort_by(|left, right| left.file.cmp(&right.file));
        Ok(UpdateOutcome {
            installed,
            current_sequence: store.status(&keys, now_unix_s)?.current_sequence,
            checked,
            ignored,
        })
    }
}

/// A folder already private to this user, so a key there is not on a shared path.
fn user_data_folder(variable: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    if cfg!(windows) {
        return variable("LOCALAPPDATA").map(PathBuf::from);
    }
    variable("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| variable("HOME").map(|home| PathBuf::from(home).join(".local/share")))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path.parent().ok_or("a home path has no parent")?;
    std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    write_atomically(path, &bytes)
}

/// Writes a new key and never replaces one, so a key cannot be overwritten by accident.
fn create_private(path: &Path, text: &str) -> Result<()> {
    let parent = path.parent().ok_or("a key path has no parent")?;
    std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| format!("{}: {error}", parent.display()))?;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(unix)]
fn refuse_if_others_can_read(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = std::fs::metadata(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .permissions()
        .mode();
    if mode & 0o077 != 0 {
        return Err(format!(
            "{} can be read by others. Anyone holding it can publish tests your machines trust, so \
             make it private, and replace it if others may have read it",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn refuse_if_others_can_read(_path: &Path) -> Result<()> {
    Ok(())
}
