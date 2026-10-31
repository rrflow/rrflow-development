//! Pure Git metadata interpretation. The engine supplies already authorized,
//! bounded, stable bytes and applies safety exclusions BEFORE these decisions.
//! No repository discovery, filesystem access, subprocess, config includes or
//! ambient global ignore lookup is performed here. Git object IDs never stand
//! in for a digest of the current working file's content.

use gix_index::hash;
use rrd_core::digest::sha256_hex;
use serde::Serialize;
use std::{collections::BTreeMap, fmt};

mod configuration;
pub use configuration::RepositoryConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidLimits,
    InputLimit,
    AllocationLimit,
    EntryLimit,
    PathLimit,
    InvalidPath,
    InvalidIndex,
    ChecksumMismatch,
    SplitIndex,
    SparseIndex,
    InvalidMode,
    DuplicateSource,
    PatternLimit,
    InvalidPatterns,
    InvalidConfiguration,
    UnsupportedConfiguration,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Upstream errors can contain paths or input. Keep diagnostics redacted.
        write!(f, "Git inventory input: {self:?}")
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy)]
pub struct ParseLimits {
    pub input_bytes: usize,
    pub entries: usize,
    pub path_bytes: usize,
    pub depth: usize,
    /// Decoded path-byte ceiling and upstream initial-allocation guard.
    /// Collection overhead/capacity and aggregate resident memory are separate.
    pub allocation_bytes: usize,
}

impl ParseLimits {
    fn validate(self) -> Result<(), Error> {
        if self.input_bytes == 0
            || self.entries == 0
            || self.path_bytes == 0
            || self.depth == 0
            || self.allocation_bytes == 0
        {
            return Err(Error::InvalidLimits);
        }
        Ok(())
    }

    fn path(self, path: &str) -> Result<(), Error> {
        if path.len() > self.path_bytes || path.split('/').count() > self.depth {
            return Err(Error::PathLimit);
        }
        if path.is_empty()
            || path.contains(['\\', ':'])
            || path.chars().any(char::is_control)
            || path
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
        {
            return Err(Error::InvalidPath);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFormat {
    Sha1,
    Sha256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackedKind {
    File,
    Symlink,
    Submodule,
    /// Conflict stages disagree about the object kind; no implicit traversal.
    Conflicted,
}

/// Tracking facts only. Index metadata is never a content-cache validity proof.
#[derive(Debug)]
pub struct TrackedPaths {
    paths: BTreeMap<String, TrackedKind>,
    folded_paths: BTreeMap<String, TrackedKind>,
    input_sha256: String,
    checksum_present: bool,
}

impl TrackedPaths {
    pub fn from_bytes(
        bytes: &[u8],
        format: ObjectFormat,
        limits: ParseLimits,
    ) -> Result<Self, Error> {
        limits.validate()?;
        if bytes.len() > limits.input_bytes {
            return Err(Error::InputLimit);
        }
        let kind = match format {
            ObjectFormat::Sha1 => hash::Kind::Sha1,
            ObjectFormat::Sha256 => hash::Kind::Sha256,
        };
        if bytes.len() < 12 + kind.len_in_bytes() || &bytes[..4] != b"DIRC" {
            return Err(Error::InvalidIndex);
        }
        let count =
            u32::from_be_bytes(bytes[8..12].try_into().map_err(|_| Error::InvalidIndex)?) as usize;
        if count > limits.entries {
            return Err(Error::EntryLimit);
        }
        let (payload, trailer) = bytes.split_at(bytes.len() - kind.len_in_bytes());
        let checksum_present = trailer.iter().any(|b| *b != 0);
        if checksum_present {
            let mut digest = hash::hasher(kind);
            digest.update(payload);
            if digest
                .try_finalize()
                .map_err(|_| Error::ChecksumMismatch)?
                .as_slice()
                != trailer
            {
                return Err(Error::ChecksumMismatch);
            }
        }
        bound_index_paths(payload, kind.len_in_bytes(), count, limits)?;
        let (state, _) = gix_index::State::from_bytes(
            // Required parser bookkeeping only; never exposed as observed time
            // or used to infer working-file freshness.
            bytes,
            filetime::FileTime::from_unix_time(0, 0),
            kind,
            gix_index::decode::Options {
                thread_limit: Some(1),
                min_extension_block_in_bytes_for_threading: usize::MAX,
                expected_checksum: None,
                alloc_limit_bytes: Some(limits.allocation_bytes),
            },
        )
        .map_err(|_| Error::InvalidIndex)?;
        // These formats need additional admitted sources, not a partial path set.
        if state.link().is_some() {
            return Err(Error::SplitIndex);
        }
        if state.is_sparse() {
            return Err(Error::SparseIndex);
        }
        state.verify_entries().map_err(|_| Error::InvalidIndex)?;
        let mut paths = BTreeMap::new();
        for entry in state.entries() {
            let path = std::str::from_utf8(entry.path(&state)).map_err(|_| Error::InvalidPath)?;
            limits.path(path)?;
            let mode = match entry.mode {
                gix_index::entry::Mode::FILE | gix_index::entry::Mode::FILE_EXECUTABLE => {
                    TrackedKind::File
                }
                gix_index::entry::Mode::SYMLINK => TrackedKind::Symlink,
                gix_index::entry::Mode::COMMIT => TrackedKind::Submodule,
                _ => return Err(Error::InvalidMode),
            };
            paths
                .entry(path.to_owned())
                .and_modify(|old| {
                    if *old != mode {
                        *old = TrackedKind::Conflicted;
                    }
                })
                .or_insert(mode);
        }
        let mut folded_paths = BTreeMap::new();
        for (path, kind) in &paths {
            folded_paths
                .entry(path.to_ascii_lowercase())
                .and_modify(|existing| {
                    if existing != kind {
                        *existing = TrackedKind::Conflicted;
                    }
                })
                .or_insert(*kind);
        }
        Ok(Self {
            paths,
            folded_paths,
            input_sha256: sha256_hex(bytes),
            checksum_present,
        })
    }

    pub fn kind(&self, path: &str, ignore_case: bool) -> Option<TrackedKind> {
        if ignore_case {
            self.folded_paths.get(&path.to_ascii_lowercase()).copied()
        } else {
            self.paths.get(path).copied()
        }
    }
    pub fn len(&self) -> usize {
        self.paths.len()
    }
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
    pub fn input_sha256(&self) -> &str {
        &self.input_sha256
    }
    pub fn checksum_present(&self) -> bool {
        self.checksum_present
    }

    /// An ignored directory may still contain tracked files. Descending for
    /// those files must not make its untracked children eligible.
    pub fn has_descendants(&self, directory: &str, ignore_case: bool) -> bool {
        if directory.is_empty() {
            return !self.paths.is_empty();
        }
        let (paths, prefix) = if ignore_case {
            (
                &self.folded_paths,
                format!("{}/", directory.to_ascii_lowercase()),
            )
        } else {
            (&self.paths, format!("{directory}/"))
        };
        paths
            .range(prefix.clone()..)
            .next()
            .is_some_and(|(path, _)| path.starts_with(&prefix))
    }
}

/// Git v4 delta compression can expand paths beyond gix-index's initial
/// allocation estimate. Check lengths without constructing paths first. This
/// is only a resource preflight; the pinned decoder still validates the format.
fn bound_index_paths(
    payload: &[u8],
    hash_bytes: usize,
    count: usize,
    limits: ParseLimits,
) -> Result<(), Error> {
    let version = u32::from_be_bytes(payload[4..8].try_into().map_err(|_| Error::InvalidIndex)?);
    if !(2..=4).contains(&version) {
        return Err(Error::InvalidIndex);
    }
    let mut cursor = 12usize;
    let mut previous = 0usize;
    let mut total = 0usize;
    for _ in 0..count {
        let start = cursor;
        let fixed = 40 + hash_bytes + 2;
        let header = payload
            .get(cursor..cursor.checked_add(fixed).ok_or(Error::InvalidIndex)?)
            .ok_or(Error::InvalidIndex)?;
        let flags = u16::from_be_bytes([header[fixed - 2], header[fixed - 1]]);
        cursor += fixed;
        if flags & 0x4000 != 0 {
            if version == 2 {
                return Err(Error::InvalidIndex);
            }
            cursor = cursor.checked_add(2).ok_or(Error::InvalidIndex)?;
        }
        let mut prefix = 0usize;
        if version == 4 {
            let mut byte = *payload.get(cursor).ok_or(Error::InvalidIndex)?;
            cursor += 1;
            let mut remove = usize::from(byte & 0x7f);
            while byte & 0x80 != 0 {
                byte = *payload.get(cursor).ok_or(Error::InvalidIndex)?;
                cursor += 1;
                remove = remove
                    .checked_add(1)
                    .and_then(|v| v.checked_mul(128))
                    .and_then(|v| v.checked_add(usize::from(byte & 0x7f)))
                    .ok_or(Error::InvalidIndex)?;
            }
            prefix = previous.checked_sub(remove).ok_or(Error::InvalidIndex)?;
        }
        let remaining = payload.get(cursor..).ok_or(Error::InvalidIndex)?;
        let suffix = remaining
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(Error::InvalidIndex)?;
        let length = prefix.checked_add(suffix).ok_or(Error::PathLimit)?;
        if length > limits.path_bytes {
            return Err(Error::PathLimit);
        }
        total = total.checked_add(length).ok_or(Error::AllocationLimit)?;
        if total > limits.allocation_bytes {
            return Err(Error::AllocationLimit);
        }
        cursor = cursor.checked_add(suffix + 1).ok_or(Error::InvalidIndex)?;
        if version != 4 {
            cursor = cursor
                .checked_add((8 - (cursor - start) % 8) % 8)
                .ok_or(Error::InvalidIndex)?;
        }
        if cursor > payload.len() {
            return Err(Error::InvalidIndex);
        }
        previous = length;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IgnoreOrigin {
    /// Explicitly admitted bytes; never an ambient home/config path.
    Global,
    Repository,
    /// Root-relative directory containing this .gitignore; empty for the root.
    Worktree {
        directory: String,
    },
}

pub struct IgnoreInput<'a> {
    pub origin: IgnoreOrigin,
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub struct IgnoreLimits {
    pub parse: ParseLimits,
    pub sources: usize,
    pub patterns: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IgnoreMatch {
    pub origin: IgnoreOrigin,
    pub input_sha256: String,
    pub line: usize,
}

struct Layer {
    origin: IgnoreOrigin,
    search: gix_ignore::Search,
    input_sha256: String,
}

/// Canonical precedence independent of enumeration order. The engine must
/// supply all applicable admitted ignore sources, revalidate them before commit,
/// and bind this input digest into its inventory policy/source observation.
pub struct IgnoreRules {
    layers: Vec<Layer>,
    limits: ParseLimits,
    case: gix_ignore::glob::pattern::Case,
    input_sha256: String,
}

impl IgnoreRules {
    pub fn from_inputs(
        inputs: &[IgnoreInput<'_>],
        limits: IgnoreLimits,
        ignore_case: bool,
    ) -> Result<Self, Error> {
        limits.parse.validate()?;
        if limits.sources == 0 || limits.patterns == 0 {
            return Err(Error::InvalidLimits);
        }
        if inputs.len() > limits.sources {
            return Err(Error::InputLimit);
        }
        let mut total_bytes = 0usize;
        let mut total_patterns = 0usize;
        let mut layers = BTreeMap::new();
        for input in inputs {
            if let IgnoreOrigin::Worktree { directory } = &input.origin {
                if !directory.is_empty() {
                    limits.parse.path(directory)?;
                }
            }
            total_bytes = total_bytes
                .checked_add(input.bytes.len())
                .ok_or(Error::InputLimit)?;
            if total_bytes > limits.parse.input_bytes {
                return Err(Error::InputLimit);
            }
            // Count before allocating pattern lists. Empty/comment lines do not
            // allocate patterns; parser scanning is bounded by input_bytes.
            let count = gix_ignore::parse(input.bytes, false).count();
            total_patterns = total_patterns
                .checked_add(count)
                .ok_or(Error::PatternLimit)?;
            if total_patterns > limits.patterns {
                return Err(Error::PatternLimit);
            }
            let mut search = gix_ignore::Search::default();
            search
                .add_patterns_buffer(
                    input.bytes,
                    ".gitignore",
                    None,
                    gix_ignore::search::Ignore {
                        support_precious: false,
                    },
                )
                .map_err(|_| Error::InvalidPatterns)?;
            let layer = Layer {
                origin: input.origin.clone(),
                search,
                input_sha256: sha256_hex(input.bytes),
            };
            let sort_key = match &input.origin {
                IgnoreOrigin::Worktree { directory } if ignore_case => IgnoreOrigin::Worktree {
                    directory: directory.to_ascii_lowercase(),
                },
                origin => origin.clone(),
            };
            if layers.insert(sort_key, layer).is_some() {
                return Err(Error::DuplicateSource);
            }
        }
        // For any queried path its ancestor directories sort in shallow-to-deep
        // order. Unrelated directories cannot match. Global < info < worktree.
        let layers: Vec<_> = layers.into_values().collect();
        let identities: Vec<_> = layers
            .iter()
            .map(|l| (&l.origin, &l.input_sha256))
            .collect();
        let input_sha256 = sha256_hex(
            &serde_json::to_vec(&(1u32, ignore_case, identities)).expect("serializable identities"),
        );
        Ok(Self {
            layers,
            limits: limits.parse,
            case: if ignore_case {
                gix_ignore::glob::pattern::Case::Fold
            } else {
                gix_ignore::glob::pattern::Case::Sensitive
            },
            input_sha256,
        })
    }

    pub fn input_sha256(&self) -> &str {
        &self.input_sha256
    }

    /// Call only for eligible UNTRACKED paths, after engine safety exclusion.
    /// Ancestor exclusions cannot be undone by a pattern for a child.
    pub fn ignored(&self, path: &str, is_directory: bool) -> Result<Option<IgnoreMatch>, Error> {
        self.limits.path(path)?;
        for (index, _) in path.match_indices('/') {
            if let Some(rule) = self.match_one(&path[..index], true) {
                return Ok(Some(rule));
            }
        }
        Ok(self.match_one(path, is_directory))
    }

    fn match_one(&self, path: &str, is_directory: bool) -> Option<IgnoreMatch> {
        for layer in self.layers.iter().rev() {
            let relative = match &layer.origin {
                IgnoreOrigin::Worktree { directory } if !directory.is_empty() => {
                    let prefix_matches =
                        path.get(..directory.len())
                            .is_some_and(|prefix| match self.case {
                                gix_ignore::glob::pattern::Case::Fold => {
                                    prefix.eq_ignore_ascii_case(directory)
                                }
                                gix_ignore::glob::pattern::Case::Sensitive => prefix == directory,
                            });
                    let Some(relative) = prefix_matches
                        .then(|| path.get(directory.len()..))
                        .flatten()
                        .and_then(|p| p.strip_prefix('/'))
                    else {
                        continue;
                    };
                    relative
                }
                _ => path,
            };
            if let Some(matched) = layer.search.pattern_matching_relative_path(
                relative.into(),
                Some(is_directory),
                self.case,
            ) {
                return (!matched.pattern.is_negative()).then(|| IgnoreMatch {
                    origin: layer.origin.clone(),
                    input_sha256: layer.input_sha256.clone(),
                    line: matched.sequence_number,
                });
            }
        }
        None
    }
}
