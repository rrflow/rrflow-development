//! The per-user instance registry: the projects this user installed or served.
//!
//! A project's own `.rrflow` directory says everything about one instance, but
//! nothing says which projects to look in. The registry is that list: one
//! small private file per instance in this user's state directory. It is a
//! candidate list and never authority. It holds no credential, the project
//! remains the owner of every fact a listing reports, and only the
//! authenticated readiness challenge proves that an instance is live.
//!
//! One file per instance and project root means a writer never rewrites
//! another entry, and a copied project that shares its original's identity is
//! listed beside it instead of replacing it. A retained lock file serializes
//! writers and listers, each entry is written beside its final name and moved
//! into place, and a damaged entry is reported without hiding the others.
//!
//! Nothing here waits without a bound. A lock that is not released within the
//! registry's lock wait makes a writer fail with a reason its caller reports,
//! and makes a lister read without the lock and say so.

use super::engram_layout::{safe_join, LOCATOR_RELATIVE_PATH};
use super::installation::{read_locator, MAX_LOCATOR_BYTES};
use super::serving::SERVING_RECORD_RELATIVE_PATH;
use super::token_key::{
    create_private_directory_anchored, open_private_empty_anchored, read_private_anchored,
    write_private_anchored,
};
use super::*;
use rrd_store::filesystem::AnchoredPath;
use std::ffi::OsString;
use std::fs::{File, TryLockError};
use std::time::Duration;

/// Names the registry directory when no explicit override is given. The
/// `rrflow` command's `--instance-registry` flag takes precedence over it.
pub const INSTANCE_REGISTRY_ENVIRONMENT: &str = "RRFLOW_INSTANCE_REGISTRY";

/// Names the lock wait in milliseconds when no explicit override is given.
/// The `--instance-registry-lock-wait-ms` flag takes precedence over it.
pub const INSTANCE_REGISTRY_LOCK_WAIT_ENVIRONMENT: &str = "RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS";

/// How long a writer or lister waits for the registry lock unless the flag or
/// the variable says otherwise. A holder keeps the lock for one small file
/// write and one rename, so five seconds is far longer than a healthy volume
/// needs and short enough that `install` and `serve` report a registry they
/// could not update instead of appearing to hang behind a stuck holder.
pub const DEFAULT_INSTANCE_REGISTRY_LOCK_WAIT_MS: u64 = 5_000;

const INSTANCE_ENTRY_FORMAT: u16 = 1;
/// An entry holds one project path and three short values; the locator's
/// bound is the engine's existing ceiling for a small non-secret record.
const MAX_INSTANCE_ENTRY_BYTES: u64 = MAX_LOCATOR_BYTES;
const ENTRY_SUFFIX: &str = ".json";
const LOCK_FILE: &str = ".lock";
const PRODUCT_DIRECTORY: &str = "rrflow";
const REGISTRY_DIRECTORY: &str = "instances";

/// One installed or served project. It holds no credential and no endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceRegistryEntry {
    pub format_version: u16,
    pub instance_id: CanonicalId,
    /// The canonical project root when the entry was written.
    pub project_root: String,
    pub product_version: String,
    /// Whether a serving record was published when the entry was last
    /// written. A killed server never withdraws, so this is history, not
    /// liveness.
    pub serving_published: bool,
}

/// What the project itself says about one registered instance, read when the
/// registry is listed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstanceStatus {
    #[serde(flatten)]
    pub entry: InstanceRegistryEntry,
    /// False when the registered project folder no longer exists.
    pub project_present: bool,
    pub locator_present: bool,
    pub serving_record_present: bool,
    /// The recorded endpoint: a candidate until `rrflow ready` proves it.
    pub endpoint: Option<String>,
    pub process_id: Option<u32>,
    /// Why this entry cannot be used as it stands, in plain words.
    pub problems: Vec<String>,
}

/// One entry written to the registry, and what it found there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRegistration {
    pub entry: InstanceRegistryEntry,
    /// Other project roots registered under this entry's instance id. A copy
    /// of a project folder carries its original's identity, so both are kept.
    pub other_project_roots: Vec<String>,
}

/// A registry file that is not a usable entry. It is reported, never dropped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnreadableInstanceEntry {
    pub file: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstanceRegistryListing {
    pub registry_root: String,
    /// False when nothing has written the registry yet.
    pub registry_present: bool,
    /// Why the entries were read without the registry lock, when they were.
    /// Every entry is moved into place whole, so such a read still sees whole
    /// entries; one being replaced at that moment may be reported unreadable.
    pub read_without_lock: Option<String>,
    pub instances: Vec<InstanceStatus>,
    pub unreadable: Vec<UnreadableInstanceEntry>,
}

/// The host conventions for a per-user state directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostPlatform {
    Windows,
    MacOs,
    Unix,
}

impl HostPlatform {
    fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Unix
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRegistry {
    root: PathBuf,
    lock_wait: Duration,
}

impl InstanceRegistry {
    /// A registry in exactly this directory, with the default lock wait.
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            lock_wait: Duration::from_millis(DEFAULT_INSTANCE_REGISTRY_LOCK_WAIT_MS),
        }
    }

    /// The same registry with another bound on waiting for its lock. Zero
    /// means one attempt and no wait.
    pub fn with_lock_wait(mut self, lock_wait: Duration) -> Self {
        self.lock_wait = lock_wait;
        self
    }

    /// The explicit override if there is one, else the directory
    /// `RRFLOW_INSTANCE_REGISTRY` names, else this user's state directory:
    /// `%LOCALAPPDATA%\rrflow\instances` on Windows,
    /// `~/Library/Application Support/rrflow/instances` on macOS, and
    /// `$XDG_STATE_HOME/rrflow/instances` or `~/.local/state/rrflow/instances`
    /// elsewhere. The override is resolved against the working directory, as
    /// any path argument is. The variable must name an absolute directory: a
    /// relative one would silently name a different registry in every folder
    /// a command runs in.
    ///
    /// The lock wait is the explicit override if there is one, else the
    /// milliseconds `RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS` names, else
    /// [`DEFAULT_INSTANCE_REGISTRY_LOCK_WAIT_MS`].
    pub fn resolve(
        override_root: Option<&Path>,
        override_lock_wait_ms: Option<u64>,
        variable: &dyn Fn(&str) -> Option<OsString>,
    ) -> Result<Self> {
        Self::resolve_for(
            HostPlatform::current(),
            override_root,
            override_lock_wait_ms,
            variable,
        )
    }

    pub(super) fn resolve_for(
        platform: HostPlatform,
        override_root: Option<&Path>,
        override_lock_wait_ms: Option<u64>,
        variable: &dyn Fn(&str) -> Option<OsString>,
    ) -> Result<Self> {
        let named = |name: &str| variable(name).filter(|value| !value.is_empty());
        let lock_wait_ms = match (
            override_lock_wait_ms,
            named(INSTANCE_REGISTRY_LOCK_WAIT_ENVIRONMENT),
        ) {
            (Some(milliseconds), _) => milliseconds,
            (None, Some(value)) => value
                .to_str()
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| {
                    ServiceError::Contract(format!(
                        "{INSTANCE_REGISTRY_LOCK_WAIT_ENVIRONMENT} must be a whole number of milliseconds"
                    ))
                })?,
            (None, None) => DEFAULT_INSTANCE_REGISTRY_LOCK_WAIT_MS,
        };
        let found =
            |root: PathBuf| Self::at(root).with_lock_wait(Duration::from_millis(lock_wait_ms));
        if let Some(root) = override_root {
            return std::path::absolute(root).map(found).map_err(storage);
        }
        if let Some(root) = named(INSTANCE_REGISTRY_ENVIRONMENT).map(PathBuf::from) {
            if !root.is_absolute() {
                return Err(ServiceError::Contract(format!(
                    "{INSTANCE_REGISTRY_ENVIRONMENT} must name an absolute directory; a relative one names a different registry in every working directory"
                )));
            }
            return Ok(found(root));
        }
        // A relative base would depend on the working directory, so it is
        // ignored exactly as the XDG base directory specification requires.
        let absolute = |name: &str| {
            named(name)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };
        let base = match platform {
            HostPlatform::Windows => absolute("LOCALAPPDATA"),
            HostPlatform::MacOs => {
                absolute("HOME").map(|home| home.join("Library").join("Application Support"))
            }
            HostPlatform::Unix => absolute("XDG_STATE_HOME")
                .or_else(|| absolute("HOME").map(|home| home.join(".local").join("state"))),
        };
        let base = base.ok_or_else(|| {
            ServiceError::Contract(format!(
                "this user's state directory cannot be found; name the instance registry with --instance-registry or {INSTANCE_REGISTRY_ENVIRONMENT}"
            ))
        })?;
        Ok(found(base.join(PRODUCT_DIRECTORY).join(REGISTRY_DIRECTORY)))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn lock_wait(&self) -> Duration {
        self.lock_wait
    }

    /// Records an installed project. Whether a serving record is published is
    /// read from the project, so recording never invents or erases one.
    pub fn record_installed(&self, project: &Path) -> Result<InstanceRegistration> {
        self.record(project, None)
    }

    /// Records that this project's serving record was published or withdrawn.
    pub fn record_serving(&self, project: &Path, published: bool) -> Result<InstanceRegistration> {
        self.record(project, Some(published))
    }

    fn record(&self, project: &Path, serving: Option<bool>) -> Result<InstanceRegistration> {
        let (filesystem, locator, _) = read_locator(project)?;
        let serving_record = safe_join(&filesystem, SERVING_RECORD_RELATIVE_PATH)?;
        let project_root = std::fs::canonicalize(project)
            .map_err(storage)?
            .into_os_string()
            .into_string()
            .map_err(|_| ServiceError::Contract("the project root is not valid Unicode".into()))?;
        let mut entry = InstanceRegistryEntry {
            format_version: INSTANCE_ENTRY_FORMAT,
            instance_id: locator.identity.instance_id,
            project_root,
            product_version: locator.product_version,
            serving_published: serving.unwrap_or(false),
        };
        let other_project_roots = self.write(&mut entry, |entry| {
            if serving.is_none() {
                entry.serving_published = serving_record.try_exists().map_err(storage)?;
            }
            Ok(())
        })?;
        Ok(InstanceRegistration {
            entry,
            other_project_roots,
        })
    }

    /// Writes one entry and returns the other project roots registered under
    /// its instance id. `settle` runs once the lock is held, so what it reads
    /// from the project is what the entry says when the next writer takes its
    /// turn: a writer that waited does not write what it saw before the wait.
    fn write(
        &self,
        entry: &mut InstanceRegistryEntry,
        settle: impl FnOnce(&mut InstanceRegistryEntry) -> Result<()>,
    ) -> Result<Vec<String>> {
        let directory = self.open(true)?.ok_or_else(|| {
            ServiceError::Storage("the instance registry directory is absent".into())
        })?;
        let Some(_lock) = RegistryLock::acquire(
            open_lock_file(&directory, true)?,
            LockMode::Exclusive,
            self.lock_wait,
        )?
        else {
            return Err(self.busy());
        };
        settle(entry)?;
        let entry = &*entry;
        let bytes =
            serde_json::to_vec(entry).map_err(|error| ServiceError::Contract(error.to_string()))?;
        if bytes.len() as u64 > MAX_INSTANCE_ENTRY_BYTES {
            return Err(ServiceError::Contract(
                "the instance registry entry exceeds its byte bound".into(),
            ));
        }
        let name = entry_file_name(entry);
        let path = directory.join(&name);
        let temporary = directory.join(format!(".{name}.pending"));
        // Writers hold the lock, so a pending file found now was left by a
        // writer that died before it could move its entry into place.
        match temporary.remove_file() {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(storage(error)),
        }
        let published =
            write_private_anchored(&temporary, &bytes).and_then(|()| temporary.rename_to(&path));
        if published.is_err() {
            let _ = temporary.remove_file();
        }
        published.map_err(storage)?;
        // Read under the same lock, so the answer is what this write left. The
        // entry is written; a directory that cannot be read back now is not a
        // failed write, and the listing reports shared identities as well.
        let entries = read_entries(&directory)
            .map(|(entries, _)| entries)
            .unwrap_or_default();
        Ok(entries
            .into_iter()
            .filter(|other| {
                other.instance_id == entry.instance_id && other.project_root != entry.project_root
            })
            .map(|other| other.project_root)
            .collect())
    }

    fn busy(&self) -> ServiceError {
        ServiceError::StorageConflict(format!(
            "the instance registry is busy: its lock was not released within {} ms (--instance-registry-lock-wait-ms or {INSTANCE_REGISTRY_LOCK_WAIT_ENVIRONMENT} sets the wait)",
            self.lock_wait.as_millis()
        ))
    }

    /// Every registered instance with what its project says now. A registry
    /// nothing has written yet lists no instances; it is not an error.
    pub fn list(&self) -> Result<InstanceRegistryListing> {
        let mut listing = InstanceRegistryListing {
            registry_root: self.root.display().to_string(),
            registry_present: false,
            read_without_lock: None,
            instances: Vec::new(),
            unreadable: Vec::new(),
        };
        let Some(directory) = self.open(false)? else {
            return Ok(listing);
        };
        listing.registry_present = true;
        // Listing changes nothing: the lock file is opened, never created, so
        // a registry this user can only read is still listed.
        let lock = match open_lock_file(&directory, false) {
            Ok(file) => RegistryLock::acquire(file, LockMode::Shared, self.lock_wait)
                .map_err(|error| format!("the registry lock cannot be taken: {error}"))
                .and_then(|lock| {
                    lock.ok_or_else(|| {
                        format!(
                            "the registry lock was not released within {} ms",
                            self.lock_wait.as_millis()
                        )
                    })
                }),
            Err(error) => Err(format!("the registry lock cannot be opened: {error}")),
        };
        let (entries, unreadable) = match lock {
            Ok(_lock) => read_entries(&directory)?,
            Err(reason) => {
                listing.read_without_lock = Some(reason);
                read_entries(&directory)?
            }
        };
        listing.unreadable = unreadable;
        // The project is inspected after the lock is released: it is the
        // project's state, and a slow volume must not hold up a writer.
        listing.instances = entries.into_iter().map(inspect).collect();
        report_shared_identities(&mut listing.instances);
        listing.instances.sort_by(|left, right| {
            (&left.entry.project_root, &left.entry.instance_id)
                .cmp(&(&right.entry.project_root, &right.entry.instance_id))
        });
        listing
            .unreadable
            .sort_by(|left, right| left.file.cmp(&right.file));
        Ok(listing)
    }

    /// The registry directory, created private when `create` is set and it is
    /// absent. A directory that already exists is used as it is: the override
    /// may name one the user made, and every entry in it is a private file.
    fn open(&self, create: bool) -> Result<Option<AnchoredPath>> {
        let root = AnchoredPath::from_path(&self.root).map_err(storage)?;
        match root.pin_directory() {
            Ok(directory) => return Ok(Some(directory)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(storage(error)),
        }
        if !create {
            return Ok(None);
        }
        if let Some(parent) = root.parent() {
            parent.create_dir_all().map_err(storage)?;
        }
        match create_private_directory_anchored(&root) {
            Ok(directory) => Ok(Some(directory)),
            // Another writer created it between the two looks.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                root.pin_directory().map(Some).map_err(storage)
            }
            Err(error) => Err(storage(error)),
        }
    }
}

/// Opens the retained empty lock file, creating it only for a writer. It is
/// never removed: unlinking it would let two writers lock two different
/// files, as the installation lease explains.
fn open_lock_file(directory: &AnchoredPath, create: bool) -> Result<File> {
    let path = directory.join(LOCK_FILE);
    if create {
        match open_private_empty_anchored(&path, true) {
            Ok(file) => return Ok(file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(storage(error)),
        }
    }
    open_private_empty_anchored(&path, false).map_err(storage)
}

#[derive(Debug, Clone, Copy)]
enum LockMode {
    Shared,
    Exclusive,
}

/// A held registry lock. Dropping it releases the lock explicitly, as the
/// installation lease does, before the retained handle closes.
struct RegistryLock(File);

impl Drop for RegistryLock {
    fn drop(&mut self) {
        let _ = File::unlock(&self.0);
    }
}

impl RegistryLock {
    /// Takes the lock, waiting at most `wait` for another holder. `None`
    /// means it was still held when the wait ended.
    ///
    /// The operating system offers a lock that waits without a bound and one
    /// that does not wait at all. A free lock is taken at once. A held one is
    /// awaited on a helper thread while this thread waits for the helper with
    /// the bound, so no retry interval has to be chosen. A helper that is
    /// still waiting when the bound passes takes the lock whenever its holder
    /// lets go and releases it at once, because nothing is left to receive it.
    fn acquire(file: File, mode: LockMode, wait: Duration) -> Result<Option<Self>> {
        let attempt = match mode {
            LockMode::Shared => file.try_lock_shared(),
            LockMode::Exclusive => file.try_lock(),
        };
        match attempt {
            Ok(()) => return Ok(Some(Self(file))),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(error)) => return Err(storage(error)),
        }
        if wait.is_zero() {
            return Ok(None);
        }
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("rrflow-instance-registry-lock".into())
            .spawn(move || {
                let locked = match mode {
                    LockMode::Shared => file.lock_shared(),
                    LockMode::Exclusive => file.lock(),
                };
                // A lock nobody receives is dropped with the message, which
                // releases it.
                let _ = send.send(locked.map(|()| Self(file)));
            })
            .map_err(storage)?;
        match receive.recv_timeout(wait) {
            Ok(locked) => locked.map(Some).map_err(storage),
            Err(_) => Ok(None),
        }
    }
}

/// An entry is named by its instance and by a digest of its project root, so
/// two folders that share one identity keep one entry each.
fn entry_file_name(entry: &InstanceRegistryEntry) -> String {
    // A canonical id is lowercase ASCII, digits, '-', '_' and '.', and never
    // starts with '.', so the name cannot collide with the lock or a pending
    // file. Its bound and the digest keep the name within one path component.
    format!(
        "{}.{}{ENTRY_SUFFIX}",
        entry.instance_id,
        digest::sha256_hex(entry.project_root.as_bytes())
    )
}

fn read_entries(
    directory: &AnchoredPath,
) -> Result<(Vec<InstanceRegistryEntry>, Vec<UnreadableInstanceEntry>)> {
    let mut entries = Vec::new();
    let mut unreadable = Vec::new();
    for path in directory.entries().map_err(storage)? {
        let path = path.map_err(storage)?;
        let Some(name) = path.relative_path().file_name() else {
            continue;
        };
        let name = name.to_string_lossy().into_owned();
        // The lock and pending files are the registry's own bookkeeping.
        if name.starts_with('.') {
            continue;
        }
        match read_entry(&path, &name) {
            Ok(entry) => entries.push(entry),
            Err(reason) => unreadable.push(UnreadableInstanceEntry { file: name, reason }),
        }
    }
    Ok((entries, unreadable))
}

fn read_entry(
    path: &AnchoredPath,
    name: &str,
) -> std::result::Result<InstanceRegistryEntry, String> {
    if !name.ends_with(ENTRY_SUFFIX) {
        return Err("is not an instance registry entry".into());
    }
    let bytes = read_private_anchored(path, MAX_INSTANCE_ENTRY_BYTES)
        .map_err(|error| format!("cannot be read as a private entry: {error}"))?;
    let entry: InstanceRegistryEntry =
        serde_json::from_slice(&bytes).map_err(|error| format!("is malformed: {error}"))?;
    if entry.format_version != INSTANCE_ENTRY_FORMAT {
        return Err("has an unsupported format version".into());
    }
    if entry_file_name(&entry) != name {
        return Err("names another instance or project root than its file".into());
    }
    if entry.product_version.is_empty() || !Path::new(&entry.project_root).is_absolute() {
        return Err("does not name a product version and an absolute project root".into());
    }
    Ok(entry)
}

/// Reads what the registered project says now. Nothing here opens the engram
/// or contacts the endpoint.
fn inspect(entry: InstanceRegistryEntry) -> InstanceStatus {
    let project = PathBuf::from(&entry.project_root);
    let mut status = InstanceStatus {
        project_present: project.is_dir(),
        locator_present: false,
        serving_record_present: false,
        endpoint: None,
        process_id: None,
        problems: Vec::new(),
        entry,
    };
    if !status.project_present {
        status
            .problems
            .push("the project folder no longer exists".into());
        return status;
    }
    status.locator_present = is_file(&below(&project, LOCATOR_RELATIVE_PATH));
    status.serving_record_present = is_file(&below(&project, SERVING_RECORD_RELATIVE_PATH));
    if !status.locator_present {
        status
            .problems
            .push("the project holds no RRFlow locator; it is no longer installed".into());
        return status;
    }
    match RrdEngine::read_project_locator(&project) {
        Ok(locator) if locator.identity.instance_id != status.entry.instance_id => {
            status.problems.push(format!(
                "the project is now installed as instance {}",
                locator.identity.instance_id
            ));
            return status;
        }
        Ok(_) => {}
        Err(error) => {
            status
                .problems
                .push(format!("the project locator cannot be used: {error}"));
            return status;
        }
    }
    match RrdEngine::read_serving_record(&project) {
        Ok(Some(record)) => {
            status.serving_record_present = true;
            status.endpoint = Some(record.url);
            status.process_id = Some(record.process_id);
        }
        Ok(None) => status.serving_record_present = false,
        Err(error) => status.problems.push(error.to_string()),
    }
    status
}

/// Two folders that are both installed as one instance cannot both be served
/// under it, and a client that keys on the instance id would confuse them.
/// Both are listed, each naming the other.
fn report_shared_identities(instances: &mut [InstanceStatus]) {
    let installed: Vec<(CanonicalId, String)> = instances
        .iter()
        .filter(|status| status.locator_present && status.problems.is_empty())
        .map(|status| {
            (
                status.entry.instance_id.clone(),
                status.entry.project_root.clone(),
            )
        })
        .collect();
    for status in instances
        .iter_mut()
        .filter(|status| status.locator_present && status.problems.is_empty())
    {
        let others: Vec<&str> = installed
            .iter()
            .filter(|(instance_id, root)| {
                instance_id == &status.entry.instance_id && root != &status.entry.project_root
            })
            .map(|(_, root)| root.as_str())
            .collect();
        if !others.is_empty() {
            status.problems.push(format!(
                "this instance id is also installed at {}; a copied project folder keeps its original's identity",
                others.join(", ")
            ));
        }
    }
}

/// Joins a managed `/`-separated relative path one component at a time. A
/// canonical Windows root is a verbatim path, in which `/` is not a separator.
fn below(project: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(project.to_owned(), |path, component| path.join(component))
}

/// True for an ordinary file; a link is not followed.
fn is_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file())
}

fn storage(error: std::io::Error) -> ServiceError {
    ServiceError::Storage(error.to_string())
}
