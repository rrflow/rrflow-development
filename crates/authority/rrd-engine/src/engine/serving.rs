//! The served endpoint record: where a serving installed engram listens.
//!
//! `rrflow serve` publishes it once its listener is bound, so another local
//! process can find an instance started from the project folder. It is an
//! endpoint candidate, never readiness: a caller proves it with the
//! authenticated readiness challenge.
//!
//! The same directory carries the stop request. A principal who can write
//! `.rrflow` names the published record it wants stopped; the serving process
//! finds the request, shuts down the way an interrupt does, withdraws its
//! record and removes the request. The withdrawn record, not the absent
//! request, is what tells a requester that the process has finished: several
//! requesters share one request file. A request is never taken back. It stays
//! until its process has read it, or until the next process to serve the
//! engram discards it.

use super::engram_layout::{safe_join, EngramFilesystem};
use super::installation::{
    publish_new_file, read_locator, read_locator_anchored, read_regular_bounded,
};
use super::*;
use rrd_store::filesystem::AnchoredPath;
use std::net::SocketAddr;

pub(super) const SERVING_RECORD_RELATIVE_PATH: &str = ".rrflow/serving.json";
const SERVING_RECORD_PENDING: &str = ".rrflow/.serving.pending";
const STOP_REQUEST_RELATIVE_PATH: &str = ".rrflow/stop-request.json";
const STOP_REQUEST_PENDING: &str = ".rrflow/.stop-request.pending";
const SERVING_RECORD_FORMAT: u16 = 1;
const STOP_REQUEST_FORMAT: u16 = 1;
const MAX_SERVING_RECORD_BYTES: u64 = 4 * 1024;
const MAX_PRODUCT_VERSION_BYTES: usize = 64;

/// How often a serving process looks for a stop request, and how often
/// `rrflow stop` looks at what the project records, unless the caller chooses
/// another interval (`--stop-poll-interval-ms` on `rrflow serve` and
/// `rrd-server`, `--poll-interval-ms` on `rrflow stop`). The repository has no
/// filesystem notification dependency, so the request is found by looking.
/// Five looks a second cost an idle server one directory lookup each and keep
/// a stop within a delay a person does not notice.
pub const DEFAULT_STOP_POLL_INTERVAL_MS: u64 = 200;

/// How long `rrflow stop` waits for the serving record to be withdrawn unless
/// `--wait-ms` says otherwise. A clean shutdown drains open requests and
/// joins the attunement follower after its current pass, which the installed
/// budgets bound rather than this adapter, so the default leaves a minute.
pub const DEFAULT_STOP_WAIT_MS: u64 = 60_000;

/// Where one serving process listens. It holds no credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServingRecord {
    pub format_version: u16,
    pub instance_id: CanonicalId,
    pub url: String,
    pub process_id: u32,
    pub product_version: String,
}

impl ServingRecord {
    /// The loopback address the record names. A readiness proof sends the
    /// installed credential to it, so any other address is refused.
    pub fn address(&self) -> Result<SocketAddr> {
        let address: SocketAddr = self
            .url
            .strip_prefix("http://")
            .and_then(|authority| authority.parse().ok())
            .ok_or_else(|| record_error("does not name an http://host:port URL"))?;
        if !address.ip().is_loopback() {
            return Err(record_error("names a non-loopback address"));
        }
        if format!("http://{address}") != self.url {
            return Err(record_error("does not hold a canonical URL"));
        }
        Ok(address)
    }

    fn validate(&self, instance_id: &CanonicalId) -> Result<()> {
        if self.format_version != SERVING_RECORD_FORMAT {
            return Err(record_error("has an unsupported format version"));
        }
        if &self.instance_id != instance_id {
            return Err(record_error("names another instance"));
        }
        if self.product_version.is_empty() || self.product_version.len() > MAX_PRODUCT_VERSION_BYTES
        {
            return Err(record_error("has an invalid product version"));
        }
        self.address().map(|_| ())
    }
}

/// Publishes this process's endpoint for an open installed engram. Only an
/// open installed engine issues one, and that engine holds the engram's single
/// writer lock, so a record found at publication was left by a process that no
/// longer runs.
#[derive(Debug)]
pub struct ServingPublisher {
    project: EngramFilesystem,
    instance_id: CanonicalId,
    product_version: String,
}

/// A published record. Close it on clean shutdown. Dropping it withdraws the
/// record too, so no return path of the serving process leaves a record that
/// names a process which has gone; only a killed process leaves one.
#[derive(Debug)]
pub struct ServingRegistration {
    path: AnchoredPath,
    stop_path: AnchoredPath,
    bytes: Vec<u8>,
    record: ServingRecord,
}

/// The published record one stop request addresses. Naming the process and
/// URL keeps a request left for a process that has gone from stopping the
/// next one to serve the engram. It holds no credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StopRequest {
    format_version: u16,
    instance_id: CanonicalId,
    url: String,
    process_id: u32,
}

impl StopRequest {
    fn addressing(record: &ServingRecord) -> Self {
        Self {
            format_version: STOP_REQUEST_FORMAT,
            instance_id: record.instance_id.clone(),
            url: record.url.clone(),
            process_id: record.process_id,
        }
    }
}

/// A stop request this process published and is waiting on.
#[derive(Debug)]
pub struct ServingStopRequest {
    project: EngramFilesystem,
    path: AnchoredPath,
    temporary: AnchoredPath,
    bytes: Vec<u8>,
    record: ServingRecord,
}

/// What the project records now about the process a stop request addressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingStopState {
    /// The addressed record is still published: its process has not finished.
    Pending,
    /// No serving record is published: the addressed process withdrew its
    /// record after its engine closed, and nothing has served the engram since.
    Stopped,
    /// Another process published this record since. The addressed process no
    /// longer holds the engram, but the project is being served.
    Replaced(ServingRecord),
}

impl RrdEngine {
    pub fn serving_publisher(&self) -> Result<ServingPublisher> {
        let root = self.installed_project_root.clone().ok_or_else(|| {
            ServiceError::Contract("only an installed engram records a served endpoint".into())
        })?;
        let (project, locator, _) = read_locator_anchored(&EngramFilesystem::admit(root, false)?)?;
        if &locator.identity.instance_id != self.instance_id() {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        Ok(ServingPublisher {
            project,
            instance_id: locator.identity.instance_id,
            product_version: locator.product_version,
        })
    }

    /// Reads an installed project's served endpoint record, if one exists.
    /// Only the authenticated readiness challenge proves it.
    pub fn read_serving_record(project: &Path) -> Result<Option<ServingRecord>> {
        let (project, locator, _) = read_locator(project)?;
        read_record(&project, &locator.identity.instance_id)
    }

    /// Asks the process that published this project's serving record to stop.
    /// `None` means no record is published, so nothing is being served.
    ///
    /// The request is a file in the project's `.rrflow` directory, so the
    /// principals who may stop an instance are exactly those who may write
    /// that directory. A directory RRFlow created is private to its owner; one
    /// the project already owned is retained with the permissions it had.
    /// That is authorization by the filesystem, not by a credential, and it
    /// reaches only a process serving this project.
    pub fn request_serving_stop(project: &Path) -> Result<Option<ServingStopRequest>> {
        let (project, locator, _) = read_locator(project)?;
        let Some(record) = read_record(&project, &locator.identity.instance_id)? else {
            return Ok(None);
        };
        let request = ServingStopRequest {
            bytes: serde_json::to_vec(&StopRequest::addressing(&record))
                .map_err(|error| ServiceError::Contract(error.to_string()))?,
            path: safe_join(&project, STOP_REQUEST_RELATIVE_PATH)?,
            temporary: safe_join(&project, STOP_REQUEST_PENDING)?,
            project,
            record,
        };
        request.publish()?;
        Ok(Some(request))
    }
}

impl ServingPublisher {
    pub fn publish(&self, address: SocketAddr) -> Result<ServingRegistration> {
        // A reader may hold the stale record or request open while this
        // removes it, and a file whose removal has not finished can refuse to
        // be replaced. A failed publication is therefore made once more
        // before it is an error; an object this cannot remove fails both.
        self.publish_once(address)
            .or_else(|_| self.publish_once(address))
    }

    fn publish_once(&self, address: SocketAddr) -> Result<ServingRegistration> {
        let record = ServingRecord {
            format_version: SERVING_RECORD_FORMAT,
            instance_id: self.instance_id.clone(),
            url: format!("http://{address}"),
            process_id: std::process::id(),
            product_version: self.product_version.clone(),
        };
        record.validate(&self.instance_id)?;
        let bytes = serde_json::to_vec(&record)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let path = safe_join(&self.project, SERVING_RECORD_RELATIVE_PATH)?;
        let temporary = safe_join(&self.project, SERVING_RECORD_PENDING)?;
        let stop_path = safe_join(&self.project, STOP_REQUEST_RELATIVE_PATH)?;
        remove_stale(&temporary, Managed::ServingRecord)?;
        // A record found now belongs to a process that no longer runs. It is
        // admitted as a managed file here and replaced below in one step,
        // never removed first: a requester waiting on it must see it give way
        // to this record, not to an instant with no record, which is how a
        // finished stop looks.
        let stale = read_if_present(&path, Managed::ServingRecord)?.is_some();
        // A request found now addresses a record this publication replaces.
        remove_stale(
            &safe_join(&self.project, STOP_REQUEST_PENDING)?,
            Managed::StopRequestPending,
        )?;
        remove_stale(&stop_path, Managed::StopRequest)?;
        if stale {
            replace_file(&path, &temporary, &bytes)?;
        } else {
            publish_new_file(&path, &temporary, &bytes)?;
        }
        Ok(ServingRegistration {
            path,
            stop_path,
            bytes,
            record,
        })
    }
}

impl ServingRegistration {
    pub fn record(&self) -> &ServingRecord {
        &self.record
    }

    /// Removes the record if it is still the one this process published.
    pub fn withdraw(&self) -> Result<()> {
        if holds(&self.path, &self.bytes, Managed::ServingRecord)? {
            remove_if_present(&self.path)?;
        }
        Ok(())
    }

    /// The one clean shutdown of a published record, for an interrupt and for
    /// a stop request alike: withdraw the record, then remove the stop request
    /// addressed to it. True means this call removed a request. Call it once
    /// the engine has closed. A record that cannot be withdrawn keeps its
    /// request, so a requester goes on seeing the record it is waiting on.
    pub fn close(&self) -> Result<bool> {
        self.withdraw()?;
        // The withdrawn record is the whole of a finished shutdown. Removing
        // the request is tidying, and a requester that saw the record go is
        // removing the same file at this moment, which on some platforms
        // refuses a second removal until the first has finished. A request
        // left behind names a record that no longer exists, so the next
        // process to serve the engram discards it.
        Ok(self.acknowledge_stop().unwrap_or(false))
    }

    /// True when a stop request addressed to this record is present. A
    /// request that names another record, or that is not a request at all, is
    /// not this process's to act on.
    pub fn stop_requested(&self) -> Result<bool> {
        Ok(self.addressed_request()?.is_some())
    }

    /// Removes the stop request addressed to this record and says whether
    /// there was one. Call it after the record is withdrawn, so that a
    /// requester which has gone leaves no request behind.
    pub fn acknowledge_stop(&self) -> Result<bool> {
        if self.addressed_request()?.is_none() {
            return Ok(false);
        }
        // A requester that saw the record withdrawn removes the request too,
        // at the same moment; whoever comes second finds it gone.
        remove_if_present(&self.stop_path)
    }

    fn addressed_request(&self) -> Result<Option<StopRequest>> {
        let bytes = match read_if_present(&self.stop_path, Managed::StopRequest) {
            Ok(Some(bytes)) => bytes,
            // An object that is not a bounded regular file is not a request.
            Ok(None) | Err(ServiceError::Contract(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        Ok(serde_json::from_slice::<StopRequest>(&bytes)
            .ok()
            .filter(|request| request == &StopRequest::addressing(&self.record)))
    }
}

impl Drop for ServingRegistration {
    fn drop(&mut self) {
        // Withdrawal removes only the record this registration wrote, so a
        // second withdrawal finds nothing to do.
        let _ = self.withdraw();
    }
}

impl ServingStopRequest {
    /// The record whose process was asked to stop.
    pub fn record(&self) -> &ServingRecord {
        &self.record
    }

    /// Reads what the project records now. The serving record decides, never
    /// the request file: every requester of one record shares one request, so
    /// its absence only says that somebody removed it.
    ///
    /// Looking changes nothing while the addressed record is published. Once
    /// the record has gone or been replaced, a request left behind is removed:
    /// its process will never read it.
    pub fn observe(&self) -> Result<ServingStopState> {
        let state = match read_record(&self.project, &self.record.instance_id)? {
            Some(record) if record == self.record => return Ok(ServingStopState::Pending),
            Some(record) => ServingStopState::Replaced(record),
            None => ServingStopState::Stopped,
        };
        // Tidying, like the serving process's own removal of the request: the
        // answer does not depend on it, and whoever else is removing the same
        // file at this moment may make this removal fail.
        let _ = self.remove_leftover();
        Ok(state)
    }

    /// Looks every `poll_interval` until the addressed record is no longer
    /// published, for at most `limit`. `Pending` means the limit ended first.
    ///
    /// The serving process removes its record while this process reads it,
    /// and a file that is being removed can refuse to be opened for a moment.
    /// A look that fails is therefore not an answer. It is repeated at the
    /// next interval, and only a failure that is still there when the limit
    /// ends is returned.
    pub fn wait(&self, limit: Duration, poll_interval: Duration) -> Result<ServingStopState> {
        let started = Instant::now();
        loop {
            let look = self.observe();
            if matches!(
                look,
                Ok(ServingStopState::Stopped | ServingStopState::Replaced(_))
            ) {
                return look;
            }
            let waited = started.elapsed();
            if waited >= limit {
                return look;
            }
            std::thread::sleep(poll_interval.min(limit - waited));
        }
    }

    fn publish(&self) -> Result<()> {
        // Two requesters that start together write the same bytes through the
        // same pending file, and one of them can find it half written or
        // already moved. The request the other one published is this request
        // too, so a failed publication is looked at once more before it is an
        // error.
        self.publish_once().or_else(|_| self.publish_once())
    }

    fn publish_once(&self) -> Result<()> {
        if holds(&self.path, &self.bytes, Managed::StopRequest)? {
            return Ok(());
        }
        // A request for another record was left for a process that has gone.
        remove_unless(&self.temporary, &self.bytes, Managed::StopRequestPending)?;
        remove_unless(&self.path, &self.bytes, Managed::StopRequest)?;
        if let Err(error) = publish_new_file(&self.path, &self.temporary, &self.bytes) {
            if !holds(&self.path, &self.bytes, Managed::StopRequest)? {
                return Err(error);
            }
        }
        Ok(())
    }

    fn remove_leftover(&self) -> Result<bool> {
        if !holds(&self.path, &self.bytes, Managed::StopRequest)? {
            return Ok(false);
        }
        // The serving process removes the request it was sent as it closes,
        // and every requester removes one it finds left; whoever comes second
        // finds it gone.
        remove_if_present(&self.path)
    }
}

fn read_record(
    project: &EngramFilesystem,
    instance_id: &CanonicalId,
) -> Result<Option<ServingRecord>> {
    let path = safe_join(project, SERVING_RECORD_RELATIVE_PATH)?;
    let Some(bytes) = read_if_present(&path, Managed::ServingRecord)? else {
        return Ok(None);
    };
    let record: ServingRecord = serde_json::from_slice(&bytes)
        .map_err(|error| record_error(&format!("is malformed: {error}")))?;
    record.validate(instance_id)?;
    Ok(Some(record))
}

/// The files this module manages in `.rrflow`, so that a refusal names the
/// file the operator has to look at.
#[derive(Debug, Clone, Copy)]
enum Managed {
    ServingRecord,
    StopRequest,
    StopRequestPending,
}

/// True when the path holds exactly these bytes.
fn holds(path: &AnchoredPath, bytes: &[u8], managed: Managed) -> Result<bool> {
    Ok(read_if_present(path, managed)?.is_some_and(|held| held == bytes))
}

/// Removes a bounded regular file that holds anything but these bytes.
fn remove_unless(path: &AnchoredPath, bytes: &[u8], managed: Managed) -> Result<()> {
    match read_if_present(path, managed)? {
        Some(held) if held != bytes => remove_if_present(path).map(|_| ()),
        _ => Ok(()),
    }
}

/// Removes a record or request left by a process that no longer holds the
/// engram. Only a bounded regular file is removed; anything else is refused,
/// not replaced.
/// Writes `bytes` to `temporary` and renames it over `path` in one step, so
/// every reader of `path` sees the earlier content or this content.
fn replace_file(path: &AnchoredPath, temporary: &AnchoredPath, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let storage = |error: std::io::Error| ServiceError::Storage(error.to_string());
    let written = (|| -> std::io::Result<()> {
        let mut options = rrd_store::filesystem::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = options.open(temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        temporary.rename_to(path)
    })();
    if let Err(error) = written {
        let _ = temporary.remove_file();
        return Err(storage(error));
    }
    path.parent()
        .ok_or(ServiceError::ProjectBindingMismatch)?
        .sync_directory()
        .map_err(storage)
}

fn remove_stale(path: &AnchoredPath, managed: Managed) -> Result<()> {
    match read_if_present(path, managed)? {
        Some(_) => remove_if_present(path).map(|_| ()),
        None => Ok(()),
    }
}

/// Reads a managed file, or `None` when it is absent.
///
/// The serving process and its requesters remove these files while the others
/// read them. A file removed during a look fails that look: it was opened,
/// and its last link went before it could be admitted. That is an absent
/// file, not a fault, and a second look says so. Only what the second look
/// reports as well is an error.
fn read_if_present(path: &AnchoredPath, managed: Managed) -> Result<Option<Vec<u8>>> {
    read_once(path, managed).or_else(|_| read_once(path, managed))
}

fn read_once(path: &AnchoredPath, managed: Managed) -> Result<Option<Vec<u8>>> {
    let unusable = || {
        let reason = "is not a bounded regular file; remove it to serve this engram";
        match managed {
            Managed::ServingRecord => record_error(reason),
            Managed::StopRequest => ServiceError::Contract(format!(
                "the stop request at {STOP_REQUEST_RELATIVE_PATH} {reason}"
            )),
            Managed::StopRequestPending => ServiceError::Contract(format!(
                "the unfinished stop request at {STOP_REQUEST_PENDING} {reason}"
            )),
        }
    };
    // A directory cannot be opened for reading on every platform, so its
    // kind is refused by name before the read.
    match path.metadata() {
        Ok(metadata) if !metadata.is_file() => return Err(unusable()),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ServiceError::Storage(error.to_string())),
    }
    match read_regular_bounded(path, MAX_SERVING_RECORD_BYTES) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(ServiceError::ProjectBindingMismatch) => Err(unusable()),
        Err(error) => Err(error),
    }
}

/// Removes the file and says whether it was there to remove.
fn remove_if_present(path: &AnchoredPath) -> Result<bool> {
    match path.remove_file() {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(ServiceError::Storage(error.to_string())),
    }
    path.parent()
        .ok_or(ServiceError::ProjectBindingMismatch)?
        .sync_directory()
        .map_err(|error| ServiceError::Storage(error.to_string()))?;
    Ok(true)
}

fn record_error(reason: &str) -> ServiceError {
    ServiceError::Contract(format!("the served endpoint record {reason}"))
}
