//! Native source metadata observations, not filesystem snapshots or grants.
//!
//! Capture uses admitted handles and reads no content. Equality is a necessary
//! reuse condition, never sufficient proof of unchanged bytes: the engine must
//! also enforce its filesystem timestamp-resolution/racy-observation policy.

use super::{cap, native, AnchoredPath};
use cap_fs_ext::OpenOptionsFollowExt;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    File,
    Directory,
}

/// A metadata-only leaf classification. Non-ordinary variants carry neither
/// target paths nor target content and never grant traversal/read authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "observation", rename_all = "snake_case")]
pub enum SourceLeaf {
    Ordinary(SourceObservation),
    /// Includes non-followed Windows directory junctions/reparse aliases.
    Symlink,
    Mount,
    Special,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Timestamp {
    seconds: i64,
    nanos: u32,
}

impl Timestamp {
    fn total_nanos(&self) -> i128 {
        i128::from(self.seconds) * 1_000_000_000 + i128::from(self.nanos)
    }
}

/// An opaque, versioned observation of one ordinary native filesystem object.
/// Host identity and timestamps are freshness evidence, not portable source IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceObservation {
    format_version: u32,
    platform: &'static str,
    object: [u8; 24],
    kind: SourceKind,
    bytes: u64,
    modified: Timestamp,
    changed: Timestamp,
    mode: u32,
    owner: Option<u64>,
    group: Option<u64>,
    links: u64,
}

impl SourceObservation {
    pub fn kind(&self) -> SourceKind {
        self.kind
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Observed POSIX execute-mode bits. Windows has no POSIX execute bit;
    /// this does not claim whether a process may execute a file under its ACL.
    pub fn executable_mode(&self) -> bool {
        #[cfg(unix)]
        {
            self.mode & 0o111 != 0
        }
        #[cfg(windows)]
        {
            false
        }
    }

    /// Digest grammar includes platform, object identity, size, mtime, ctime,
    /// permissions/attributes, ownership and links. Last-access time is omitted.
    pub fn revision_sha256(&self) -> io::Result<String> {
        let bytes = serde_json::to_vec(&("rrflow-source-observation-v1", self))
            .map_err(io::Error::other)?;
        Ok(rrd_core::digest::sha256_hex(&bytes))
    }

    /// Conservative timestamp-age predicate only. The engine supplies a known
    /// resolution margin for the admitted filesystem and compares observations
    /// before reuse. Unknown resolution, zero margin, future timestamps, clocks
    /// without a trusted bound or changed observations require content reads.
    pub fn predates(&self, boundary: SystemTime, resolution_margin: Duration) -> bool {
        if resolution_margin.is_zero() {
            return false;
        }
        let boundary = match boundary.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos() as i128,
            Err(error) => -(error.duration().as_nanos() as i128),
        };
        let cutoff = boundary - resolution_margin.as_nanos() as i128;
        self.modified.total_nanos() < cutoff && self.changed.total_nanos() < cutoff
    }

    fn capture(file: &File) -> io::Result<Self> {
        native::validate_object(file)?;
        let metadata = file.metadata()?;
        let kind = if metadata.is_dir() {
            SourceKind::Directory
        } else {
            native::validate_file(file)?;
            SourceKind::File
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let mut object = [0; 24];
            object[..8].copy_from_slice(&metadata.dev().to_le_bytes());
            object[8..16].copy_from_slice(&metadata.ino().to_le_bytes());
            let timestamp = |seconds, nanos| {
                let nanos = u32::try_from(nanos)
                    .ok()
                    .filter(|n| *n < 1_000_000_000)
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "invalid native timestamp")
                    })?;
                Ok::<_, io::Error>(Timestamp { seconds, nanos })
            };
            Ok(Self {
                format_version: 1,
                platform: "unix",
                object,
                kind,
                bytes: metadata.len(),
                modified: timestamp(metadata.mtime(), metadata.mtime_nsec())?,
                changed: timestamp(metadata.ctime(), metadata.ctime_nsec())?,
                mode: metadata.mode(),
                owner: Some(u64::from(metadata.uid())),
                group: Some(u64::from(metadata.gid())),
                links: metadata.nlink(),
            })
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::{
                FileBasicInfo, FileIdInfo, FileStandardInfo, GetFileInformationByHandleEx,
                FILE_BASIC_INFO, FILE_ID_INFO, FILE_STANDARD_INFO,
            };
            let mut basic = FILE_BASIC_INFO::default();
            let mut id = FILE_ID_INFO::default();
            let mut standard = FILE_STANDARD_INFO::default();
            // Each output is the documented structure for its information
            // class. Inspect only after success; the live handle stays owned.
            for (class, pointer, bytes) in [
                (
                    FileBasicInfo,
                    (&mut basic as *mut FILE_BASIC_INFO).cast(),
                    std::mem::size_of::<FILE_BASIC_INFO>(),
                ),
                (
                    FileIdInfo,
                    (&mut id as *mut FILE_ID_INFO).cast(),
                    std::mem::size_of::<FILE_ID_INFO>(),
                ),
                (
                    FileStandardInfo,
                    (&mut standard as *mut FILE_STANDARD_INFO).cast(),
                    std::mem::size_of::<FILE_STANDARD_INFO>(),
                ),
            ] {
                if unsafe {
                    GetFileInformationByHandleEx(file.as_raw_handle(), class, pointer, bytes as u32)
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
            }
            let mut object = [0; 24];
            object[..8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
            object[8..].copy_from_slice(&id.FileId.Identifier);
            let timestamp = |ticks: i64| {
                // Windows FILETIME uses 100ns ticks since 1601-01-01.
                let ticks = i128::from(ticks) - 116_444_736_000_000_000;
                Timestamp {
                    seconds: ticks.div_euclid(10_000_000) as i64,
                    nanos: (ticks.rem_euclid(10_000_000) * 100) as u32,
                }
            };
            Ok(Self {
                format_version: 1,
                platform: "windows",
                object,
                kind,
                bytes: u64::try_from(standard.EndOfFile).map_err(io::Error::other)?,
                modified: timestamp(basic.LastWriteTime),
                changed: timestamp(basic.ChangeTime),
                mode: basic.FileAttributes,
                owner: None,
                group: None,
                links: u64::from(standard.NumberOfLinks),
            })
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (kind, metadata);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native source observation unavailable",
            ))
        }
    }
}

fn stable_capture(file: &File) -> io::Result<SourceObservation> {
    let first = SourceObservation::capture(file)?;
    if first != SourceObservation::capture(file)? {
        return Err(changed());
    }
    Ok(first)
}

fn changed() -> io::Error {
    io::Error::new(
        io::ErrorKind::WouldBlock,
        "source object changed during observation",
    )
}

impl AnchoredPath {
    /// Classifies a leaf relative to retained, no-follow parent authority.
    /// Links and special objects are identified by no-follow metadata without
    /// opening their content. Ordinary metadata handles are checked against the
    /// admitted mount before an observation is returned. Parent aliases and
    /// hardlinked files remain errors, never implicit grants.
    pub fn source_leaf(&self) -> io::Result<SourceLeaf> {
        if self.relative.as_os_str().is_empty() {
            return stable_capture(&self.root.file).map(SourceLeaf::Ordinary);
        }
        let (parent, name, _ancestors) = self.parent_handle()?;
        let metadata = cap::stat(&parent, Path::new(&name), cap::FollowSymlinks::No)?;
        if metadata.is_symlink() {
            return Ok(SourceLeaf::Symlink);
        }
        if !metadata.is_file() && !metadata.is_dir() {
            return Ok(SourceLeaf::Special);
        }
        let mut options = cap::OpenOptions::new();
        options.read(true).follow(cap::FollowSymlinks::No);
        native::metadata_options(&mut options);
        let file = cap::open(&parent, Path::new(&name), &options)?;
        if native::mount_identity(&file)? != self.root.mount {
            return Ok(SourceLeaf::Mount);
        }
        stable_capture(&file).map(SourceLeaf::Ordinary)
    }

    /// Opens only metadata through this root's no-follow/mount admission.
    /// Directory observations are supported; symlinks, special files and hard
    /// linked regular files retain the existing containment denial.
    pub fn source_observation(&self) -> io::Result<SourceObservation> {
        if self.relative.as_os_str().is_empty() {
            return stable_capture(&self.root.file);
        }
        let (parent, name, _ancestors) = self.parent_handle()?;
        let mut options = cap::OpenOptions::new();
        options.read(true).follow(cap::FollowSymlinks::No);
        native::metadata_options(&mut options);
        let file = cap::open(&parent, Path::new(&name), &options)?;
        self.admit_mount(&file)?;
        stable_capture(&file)
    }

    /// Revalidates the supplied live read handle against this admitted name
    /// without reading bytes or moving the file cursor. Call around a bounded
    /// content read; mismatches must not receive an accepted digest.
    pub fn source_observation_for_file(&self, file: &File) -> io::Result<SourceObservation> {
        self.admit_mount(file)?;
        self.validate_file(file)?;
        let observed = stable_capture(file)?;
        if observed != self.source_observation()? {
            return Err(changed());
        }
        Ok(observed)
    }
}
