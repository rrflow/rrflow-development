//! Bounded installation preconditions and read-only Inventory preparation.
//!
//! Version-three preconditions bind admitted policy/Git inputs and exclude safety paths
//! before content reads. Enumeration admits entries before retaining them, and
//! hashing reads at most the admitted file length. Metadata drift fails closed;
//! this is not an atomic
//! filesystem snapshot or evidence for metadata-only content reuse.

use super::policy::CompiledPolicy;
use super::DefaultInstallProfile;
use crate::engine::{Result, ServiceError};
use rrd_core::digest::Sha256 as IncrementalSha256;
use rrd_store::filesystem::{AnchoredPath, SourceKind, SourceLeaf, SourceObservation};

mod git_inputs;
use git_inputs::GitInputs;

struct WalkEntry {
    path: AnchoredPath,
    observed: Option<SourceLeaf>,
    gitlink: bool,
    excluded: Option<String>,
}

use std::fs::Metadata;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

struct WorkBudget<'a> {
    check_external: &'a dyn Fn() -> Result<()>,
    started: Instant,
    duration: Duration,
}

impl WorkBudget<'_> {
    fn new(elapsed_ms: u64) -> Self {
        Self {
            check_external: &|| Ok(()),
            started: Instant::now(),
            duration: Duration::from_millis(elapsed_ms),
        }
    }
    fn check(&self) -> Result<()> {
        (self.check_external)()?;
        if self.started.elapsed() >= self.duration {
            return Err(ServiceError::DeadlineExceeded);
        }
        Ok(())
    }
}

pub(in crate::engine) fn project_inventory_sha256(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
) -> Result<String> {
    project_inventory_sha256_excluding(project, profile, None)
}

pub(super) fn project_inventory_sha256_excluding(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
    excluded_relative: Option<&Path>,
) -> Result<String> {
    scan_project(project, profile, excluded_relative, None, &|| Ok(())).map(|(digest, _)| digest)
}

mod proposal;
pub(in crate::engine) use proposal::prepare_project_inventory;

fn scan_project(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
    excluded_relative: Option<&Path>,
    proposal: Option<(
        rrd_attunement::inventory::SourceScope,
        Option<&rrd_attunement::inventory::Proposal>,
    )>,
    check: &dyn Fn() -> Result<()>,
) -> Result<(String, Option<rrd_attunement::inventory::Proposal>)> {
    let policy = profile.inventory_policy.compile()?;
    let budget = WorkBudget {
        check_external: check,
        ..WorkBudget::new(policy.source.limits.elapsed_ms)
    };
    let root_observation = project.source_observation().map_err(storage_error)?;
    let mut git = GitInputs::read(project, &policy, &budget)?;
    let mut entries = Vec::<WalkEntry>::new();
    let admitted_lease = super::lease::admit_existing(project)?;
    collect_project_entries(
        project,
        WalkOptions {
            excluded_relative,
            lease_admitted: admitted_lease.is_some(),
            record_exclusions: proposal.is_some(),
        },
        &policy,
        &budget,
        &mut git,
        &mut entries,
    )?;
    let mut builder = proposal
        .map(|(scope, prior)| proposal::builder(scope, prior, &policy, &git))
        .transpose()?;
    entries.sort_by(|left, right| left.path.relative_path().cmp(right.path.relative_path()));
    let mut aggregate = IncrementalSha256::new();
    aggregate.update(b"rrflow-project-precondition-v3\0");
    aggregate.update(policy.sha256.as_bytes());
    aggregate.update(git.identity()?.as_bytes());
    let mut total = 0u64;
    for entry in &entries {
        let path = &entry.path;
        budget.check()?;
        let relative = path.relative_path();
        let relative = relative
            .to_str()
            .ok_or(ServiceError::ProjectBindingMismatch)?
            .replace('\\', "/");
        if let Some(rule) = &entry.excluded {
            if let Some(builder) = &mut builder {
                builder
                    .leaf(
                        &relative,
                        rrd_attunement::inventory::EntryKind::Excluded { rule: rule.clone() },
                    )
                    .map_err(storage_error)?;
            }
            continue;
        }
        let (tag, file) = match entry
            .observed
            .as_ref()
            .ok_or(ServiceError::ProjectBindingMismatch)?
        {
            _ if entry.gitlink => (b"g\0", false),
            SourceLeaf::Ordinary(obs) if obs.kind() == SourceKind::Directory => (b"d\0", false),
            SourceLeaf::Ordinary(_) => (b"f\0", true),
            SourceLeaf::Symlink => (b"l\0", false),
            SourceLeaf::Mount => (b"m\0", false),
            SourceLeaf::Special => (b"s\0", false),
        };
        aggregate.update(tag);
        aggregate.update(&(relative.len() as u64).to_be_bytes());
        aggregate.update(relative.as_bytes());
        if file {
            let metadata = path
                .metadata()
                .map_err(|error| ServiceError::Storage(error.to_string()))?;
            total = total.checked_add(metadata.len()).ok_or_else(|| {
                ServiceError::Contract("project inventory byte count overflowed".into())
            })?;
            if metadata.len() > policy.source.limits.file_bytes {
                // The file changed since enumeration and cannot silently enter
                // the oversized-file exclusion after its path was admitted.
                return Err(ServiceError::ProjectBindingMismatch);
            }
            if total > policy.source.limits.content_bytes {
                return Err(ServiceError::Contract(
                    "project inventory exceeds the configured byte bound".into(),
                ));
            }
            aggregate.update(&metadata.len().to_be_bytes());
            if path.source_leaf().map_err(storage_error)?
                != entry
                    .observed
                    .as_ref()
                    .ok_or(ServiceError::ProjectBindingMismatch)?
                    .clone()
            {
                return Err(ServiceError::ProjectBindingMismatch);
            }
            let digest = if let Some(builder) = &mut builder {
                proposal::ingest_file(builder, &relative, entry, &budget)?
            } else {
                hash_inventory_file(path, &metadata, &budget)?
            };
            aggregate.update(digest.as_bytes());
        } else if let Some(builder) = &mut builder {
            proposal::ingest_leaf(builder, &relative, entry)?;
        }
    }
    revalidate_observations(project, &root_observation, &entries, &git, &budget)?;
    let proposed = builder
        .map(|builder| builder.finish(true).map_err(storage_error))
        .transpose()?;
    if let Some(proposed) = &proposed {
        proposal::require_output_bound(proposed, policy.source.limits.output_bytes)?;
    }
    Ok((aggregate.finalize_hex(), proposed))
}

// Share the same source-observation checks with non-authoritative discovery.
// Neither this check nor an unchanged directory is an atomic filesystem snapshot.
fn revalidate_observations(
    project: &AnchoredPath,
    root_observation: &SourceObservation,
    entries: &[WalkEntry],
    git: &GitInputs,
    budget: &WorkBudget,
) -> Result<()> {
    for entry in entries {
        budget.check()?;
        if entry.excluded.is_none()
            && entry.path.source_leaf().map_err(storage_error)?
                != entry
                    .observed
                    .as_ref()
                    .ok_or(ServiceError::ProjectBindingMismatch)?
                    .clone()
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
    }
    git.revalidate(budget)?;
    if &project.source_observation().map_err(storage_error)? != root_observation {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    budget.check()
}

/// Read only the admitted parent tree and return present tracked gitlink
/// directories as provisional candidates. Child metadata is never opened.
pub(super) fn discover_gitlinks(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
    profile_sha256: String,
) -> Result<super::NestedRepositoryDiscovery> {
    let policy = profile.inventory_policy.compile()?;
    let budget = WorkBudget::new(policy.source.limits.elapsed_ms);
    let root_observation = project.source_observation().map_err(storage_error)?;
    let mut git = GitInputs::read(project, &policy, &budget)?;
    let mut entries = Vec::new();
    collect_project_entries(
        project,
        WalkOptions {
            excluded_relative: None,
            lease_admitted: false,
            record_exclusions: false,
        },
        &policy,
        &budget,
        &mut git,
        &mut entries,
    )?;
    let mut candidates = Vec::new();
    for entry in &entries {
        if entry.gitlink
            && matches!(&entry.observed, Some(SourceLeaf::Ordinary(obs)) if obs.kind() == SourceKind::Directory)
        {
            let path = entry
                .path
                .relative_path()
                .to_str()
                .ok_or(ServiceError::ProjectBindingMismatch)?
                .replace('\\', "/");
            candidates.push(path);
        }
    }
    candidates.sort();
    candidates.dedup();
    let result = super::NestedRepositoryDiscovery {
        format_version: 1,
        profile_sha256,
        git_inputs_sha256: git.identity()?,
        git_index_verified: git.has_verified_index(),
        coverage: "present_tracked_gitlink_directories_only",
        recommended_layout: "separate_projects",
        selection_available: false,
        candidates,
    };
    let output = serde_json::to_string_pretty(&result).map_err(storage_error)?;
    if output.len() as u64 > policy.source.limits.output_bytes {
        return Err(ServiceError::ConfigurationLimit(
            "repository discovery output byte bound exceeded".into(),
        ));
    }
    revalidate_observations(project, &root_observation, &entries, &git, &budget)?;
    Ok(result)
}

struct WalkOptions<'a> {
    excluded_relative: Option<&'a Path>,
    lease_admitted: bool,
    record_exclusions: bool,
}

fn collect_project_entries(
    root: &AnchoredPath,
    options: WalkOptions<'_>,
    policy: &CompiledPolicy<'_>,
    budget: &WorkBudget,
    git: &mut GitInputs,
    output: &mut Vec<WalkEntry>,
) -> Result<()> {
    let WalkOptions {
        excluded_relative,
        lease_admitted,
        record_exclusions,
    } = options;
    let mut pending = vec![root.to_owned()];
    let mut visited = 0u64;
    let mut metadata_bytes = 0u64;
    while let Some(current) = pending.pop() {
        budget.check()?;
        git.enter(&current, policy, budget)?;
        let remaining_metadata = policy
            .source
            .limits
            .metadata_bytes
            .checked_sub(git.metadata_bytes)
            .filter(|remaining| *remaining >= metadata_bytes)
            .ok_or_else(|| {
                ServiceError::ConfigurationLimit("inventory metadata byte bound exceeded".into())
            })?;
        let entries = current.entries().map_err(storage_error)?;
        // Do not collect or sort a whole directory before applying the bound.
        // The admitted output is sorted once, by the digest builder.
        for entry in entries {
            budget.check()?;
            if visited >= policy.source.limits.entries {
                return Err(ServiceError::Contract(
                    "project inventory exceeds the configured file bound".into(),
                ));
            }
            visited += 1;
            let path = entry.map_err(storage_error)?;
            let relative = path.relative_path();
            if excluded_relative == Some(relative)
                || lease_admitted && relative == Path::new(super::lease::RELATIVE_PATH)
            {
                continue;
            }
            let normalized = relative
                .to_str()
                .ok_or(ServiceError::ProjectBindingMismatch)?
                .replace('\\', "/");
            // Non-directory-only rules can exclude a leaf without even opening
            // its metadata handle (important for locked secret files).
            let excluded = policy.excluded(&normalized, false)?.map(str::to_owned);
            let excluded = match excluded {
                Some(rule) => Some(rule),
                None if git.excluded_without_metadata(&normalized)? => Some("git-ignore".into()),
                None => None,
            };
            if let Some(rule) = excluded {
                if record_exclusions {
                    account_metadata(&path, &mut metadata_bytes, remaining_metadata)?;
                    output.push(WalkEntry {
                        path,
                        observed: None,
                        gitlink: false,
                        excluded: Some(rule),
                    });
                }
                continue;
            }
            let observed = path.source_leaf().map_err(storage_error)?;
            let directory = matches!(&observed, SourceLeaf::Ordinary(obs) if obs.kind() == SourceKind::Directory);
            let excluded = policy.excluded(&normalized, directory)?.map(str::to_owned);
            let excluded = match excluded {
                Some(rule) => Some(rule),
                None if !git.admits(&normalized, directory)? => Some("git-ignore".into()),
                None => None,
            };
            if let Some(rule) = excluded {
                if record_exclusions {
                    account_metadata(&path, &mut metadata_bytes, remaining_metadata)?;
                    output.push(WalkEntry {
                        path,
                        observed: Some(observed),
                        gitlink: false,
                        excluded: Some(rule),
                    });
                }
                continue;
            }
            let gitlink = git.is_submodule(&normalized);
            if let SourceLeaf::Ordinary(obs) = &observed {
                if obs.kind() == SourceKind::File && obs.bytes() > policy.source.limits.file_bytes {
                    if record_exclusions {
                        account_metadata(&path, &mut metadata_bytes, remaining_metadata)?;
                        output.push(WalkEntry {
                            path,
                            observed: Some(observed),
                            gitlink: false,
                            excluded: Some("file-byte-limit".into()),
                        });
                    }
                    continue;
                }
            }
            account_metadata(&path, &mut metadata_bytes, remaining_metadata)?;
            if directory && !gitlink {
                pending.push(path.clone());
            }
            output.push(WalkEntry {
                path,
                observed: Some(observed),
                gitlink,
                excluded: None,
            });
        }
    }
    Ok(())
}

fn account_metadata(path: &AnchoredPath, accounted: &mut u64, ceiling: u64) -> Result<()> {
    // Retained logical metadata accounting, not an RSS measurement. Include
    // both path buffers and possible pending-directory copies before retention.
    let bytes = 2
        * (std::mem::size_of::<WalkEntry>()
            + path.relative_path().as_os_str().as_encoded_bytes().len()
            + path.diagnostic_path().as_os_str().as_encoded_bytes().len()) as u64;
    *accounted = accounted.checked_add(bytes).ok_or_else(|| {
        ServiceError::ConfigurationLimit("inventory metadata accounting overflow".into())
    })?;
    if *accounted > ceiling {
        return Err(ServiceError::ConfigurationLimit(
            "inventory metadata byte bound exceeded".into(),
        ));
    }
    Ok(())
}

fn hash_inventory_file(
    path: &AnchoredPath,
    admitted: &Metadata,
    budget: &WorkBudget,
) -> Result<String> {
    budget.check()?;
    if admitted.file_type().is_symlink() || !admitted.is_file() {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let before = path.source_observation().map_err(storage_error)?;
    let mut file = path.open_read().map_err(storage_error)?;
    if before
        != path
            .source_observation_for_file(&file)
            .map_err(storage_error)?
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    require_same_observation(admitted, &file.metadata().map_err(storage_error)?)?;
    let digest = hash_exact_contents(&mut file, admitted.len(), budget)?;
    require_same_observation(admitted, &file.metadata().map_err(storage_error)?)?;
    require_same_observation(admitted, &path.metadata().map_err(storage_error)?)?;
    if before
        != path
            .source_observation_for_file(&file)
            .map_err(storage_error)?
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    budget.check()?;
    Ok(digest)
}

fn require_same_observation(admitted: &Metadata, observed: &Metadata) -> Result<()> {
    if observed.file_type().is_symlink()
        || !observed.is_file()
        || observed.len() != admitted.len()
        || observed.modified().map_err(storage_error)?
            != admitted.modified().map_err(storage_error)?
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    Ok(())
}

fn hash_exact_contents(
    reader: &mut impl Read,
    admitted_bytes: u64,
    budget: &WorkBudget,
) -> Result<String> {
    let mut bounded = reader.take(admitted_bytes);
    let mut hasher = IncrementalSha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        budget.check()?;
        let read = match bounded.read(&mut buffer) {
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(storage_error(error)),
        };
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if bounded.limit() != 0 {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    budget.check()?;
    Ok(hasher.finalize_hex())
}

fn storage_error(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    fn hash_exact_contents(reader: &mut impl Read, admitted_bytes: u64) -> Result<String> {
        super::hash_exact_contents(reader, admitted_bytes, &WorkBudget::new(30_000))
    }
    fn hash_inventory_file(path: &AnchoredPath, admitted: &Metadata) -> Result<String> {
        super::hash_inventory_file(path, admitted, &WorkBudget::new(30_000))
    }
    fn project_inventory_sha256(path: &Path, profile: &DefaultInstallProfile) -> Result<String> {
        super::project_inventory_sha256(
            &AnchoredPath::open_root(path).map_err(storage_error)?,
            profile,
        )
    }
    fn project_inventory_sha256_excluding(
        path: &Path,
        profile: &DefaultInstallProfile,
        exclude: Option<&Path>,
    ) -> Result<String> {
        super::project_inventory_sha256_excluding(
            &AnchoredPath::open_root(path).map_err(storage_error)?,
            profile,
            exclude,
        )
    }
    fn collect_project_entries(
        path: &Path,
        exclude: Option<&Path>,
        max: u64,
        output: &mut Vec<(AnchoredPath, bool)>,
    ) -> Result<()> {
        let (mut profile, _) = DefaultInstallProfile::load("default")?;
        profile.inventory_policy.limits.entries = max;
        let policy = profile.inventory_policy.compile()?;
        let root = AnchoredPath::open_root(path).map_err(storage_error)?;
        let budget = WorkBudget::new(policy.source.limits.elapsed_ms);
        let mut git = GitInputs::read(&root, &policy, &budget)?;
        let mut entries = Vec::new();
        let result = super::collect_project_entries(
            &root,
            WalkOptions {
                excluded_relative: exclude,
                lease_admitted: false,
                record_exclusions: false,
            },
            &policy,
            &budget,
            &mut git,
            &mut entries,
        );
        output.extend(entries.into_iter().map(|entry| {
            let directory = matches!(&entry.observed, Some(SourceLeaf::Ordinary(obs)) if obs.kind() == SourceKind::Directory);
            (entry.path, directory)
        }));
        result
    }

    #[test]
    fn project_precondition_binds_version_three_policy_git_inputs_and_eligible_source() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::create_dir(root.path().join(".git")).unwrap();
        std::fs::create_dir(root.path().join(".rrflow")).unwrap();
        std::fs::write(root.path().join("src/main.rs"), b"fn main() {}\n").unwrap();
        std::fs::write(root.path().join("empty"), b"").unwrap();
        std::fs::write(root.path().join("README.md"), b"RRFlow\n").unwrap();
        std::fs::write(
            root.path().join(".git/config"),
            b"[core]\nrepositoryFormatVersion = 0\n",
        )
        .unwrap();
        std::fs::write(root.path().join(".rrflow/cache"), b"excluded").unwrap();
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        assert_eq!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            "31a1b80977ce83f586733848513b6d9079fecf3337034f4a4e6ad05208c9af9a"
        );
    }

    #[test]
    fn enumeration_never_retains_more_than_the_admitted_entries() {
        let root = tempfile::tempdir().unwrap();
        let mut directory = root.path().to_owned();
        for _ in 0..24 {
            directory.push("d");
            std::fs::create_dir(&directory).unwrap();
        }
        std::fs::write(directory.join("leaf"), b"source").unwrap();
        for bound in [1, 7, 24] {
            let mut entries = Vec::new();
            let error =
                collect_project_entries(root.path(), None, bound, &mut entries).unwrap_err();
            assert!(error.to_string().contains("file bound"));
            assert_eq!(entries.len() as u64, bound);
        }
        let mut entries = Vec::new();
        collect_project_entries(root.path(), None, 25, &mut entries).unwrap();
        assert_eq!(entries.len(), 25);
    }

    #[test]
    fn exclusions_and_creation_order_do_not_change_the_digest_or_admitted_count() {
        let (mut profile, _) = DefaultInstallProfile::load("default").unwrap();
        profile.inventory_policy.limits.entries = 6;
        profile.inventory_policy.limits.content_bytes = 3;
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        for (root, names) in [
            (first.path(), ["alpha", "empty", "unicode-\u{3bb}"]),
            (second.path(), ["unicode-\u{3bb}", "empty", "alpha"]),
        ] {
            for name in names {
                std::fs::write(root.join(name), if name == "empty" { "" } else { "a" }).unwrap();
            }
            for name in [".git", ".rrflow"] {
                std::fs::create_dir(root.join(name)).unwrap();
                std::fs::write(root.join(name).join("not-source"), b"excluded").unwrap();
            }
        }
        let digest = project_inventory_sha256(first.path(), &profile).unwrap();
        assert_eq!(
            project_inventory_sha256(second.path(), &profile).unwrap(),
            digest
        );
        std::fs::write(first.path().join("intent"), b"owned intent").unwrap();
        assert_eq!(
            project_inventory_sha256_excluding(first.path(), &profile, Some(Path::new("intent")))
                .unwrap(),
            digest
        );
        assert!(project_inventory_sha256(first.path(), &profile).is_err());
        std::fs::remove_file(first.path().join("intent")).unwrap();
        profile.inventory_policy.limits.content_bytes = 1;
        assert!(project_inventory_sha256(first.path(), &profile)
            .unwrap_err()
            .to_string()
            .contains("byte bound"));
    }

    #[test]
    fn git_walk_retains_tracked_ignored_source_and_binds_metadata_inputs() {
        let root = tempfile::tempdir().unwrap();
        for directory in [".git/info", "src", "ignored", ".github/workflows"] {
            std::fs::create_dir_all(root.path().join(directory)).unwrap();
        }
        std::fs::write(
            root.path().join(".git/config"),
            b"[core]\nrepositoryFormatVersion=0\nignoreCase=false\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join(".git/index"),
            include_bytes!(
                "../../../../../compute/rrd-attunement/tests/fixtures/git-index/sha1-v2.index"
            ),
        )
        .unwrap();
        std::fs::write(root.path().join(".gitignore"), b"ignored/\n*.log\n").unwrap();
        std::fs::write(root.path().join(".git/info/exclude"), b"*.bak\n").unwrap();
        std::fs::write(root.path().join("src/.gitignore"), b"!keep.log\n").unwrap();
        for path in [
            "src/main.rs",
            "ignored/tracked.txt",
            ".github/workflows/check.yml",
            "src/keep.log",
        ] {
            std::fs::write(root.path().join(path), b"source").unwrap();
        }
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        let first = project_inventory_sha256(root.path(), &profile).unwrap();
        for path in [
            "ignored/untracked.txt",
            "other.log",
            "other.bak",
            "src/drop.log",
            ".env",
        ] {
            std::fs::write(root.path().join(path), b"excluded").unwrap();
        }
        assert_eq!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            first
        );
        for path in [
            "src/main.rs",
            "ignored/tracked.txt",
            ".github/workflows/check.yml",
            "src/keep.log",
        ] {
            std::fs::write(root.path().join(path), b"changed").unwrap();
            assert_ne!(
                project_inventory_sha256(root.path(), &profile).unwrap(),
                first,
                "{path}"
            );
            std::fs::write(root.path().join(path), b"source").unwrap();
        }
        std::fs::write(root.path().join(".git/info/exclude"), b"*.bak\n*.new\n").unwrap();
        assert_ne!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            first
        );
        std::fs::write(
            root.path().join(".git/config"),
            b"[include]\npath=../outside\n",
        )
        .unwrap();
        assert!(project_inventory_sha256(root.path(), &profile).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn git_ignored_locked_file_is_never_opened() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".git")).unwrap();
        std::fs::write(root.path().join(".gitignore"), b"*.log\n").unwrap();
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        let first = project_inventory_sha256(root.path(), &profile).unwrap();
        let locked = File::options()
            .create_new(true)
            .read(true)
            .write(true)
            .share_mode(0)
            .open(root.path().join("private.log"))
            .unwrap();
        assert_eq!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            first
        );
        drop(locked);
    }

    #[test]
    fn git_inputs_detect_appearance_change_and_bound_bytes() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".git")).unwrap();
        let (mut profile, _) = DefaultInstallProfile::load("default").unwrap();
        let anchor = AnchoredPath::open_root(root.path()).unwrap();
        let budget = WorkBudget::new(30_000);
        let policy = profile.inventory_policy.compile().unwrap();
        let mut git = GitInputs::read(&anchor, &policy, &budget).unwrap();
        git.enter(&anchor, &policy, &budget).unwrap();
        git.revalidate(&budget).unwrap();
        std::fs::write(root.path().join(".gitignore"), b"*.log\n").unwrap();
        assert!(git.revalidate(&budget).is_err());
        let mut git = GitInputs::read(&anchor, &policy, &budget).unwrap();
        git.enter(&anchor, &policy, &budget).unwrap();
        std::fs::write(root.path().join(".gitignore"), b"*.bak\n").unwrap();
        assert!(git.revalidate(&budget).is_err());
        std::fs::write(root.path().join(".git/config"), vec![b' '; 2048]).unwrap();
        profile.inventory_policy.limits.git_input_bytes = 1024;
        assert!(project_inventory_sha256(root.path(), &profile)
            .unwrap_err()
            .to_string()
            .contains("Git input byte bound"));
    }

    #[test]
    fn directory_alias_is_a_leaf_and_target_content_is_not_read() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("private"), b"first").unwrap();
        let link = root.path().join("alias");
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        #[cfg(windows)]
        {
            let result = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(&link)
                .arg(outside.path())
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        let first = project_inventory_sha256(root.path(), &profile).unwrap();
        std::fs::write(outside.path().join("private"), b"changed outside content").unwrap();
        assert_eq!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            first
        );
        let anchored = AnchoredPath::open_root(root.path()).unwrap();
        let policy = profile.inventory_policy.compile().unwrap();
        let budget = WorkBudget::new(30_000);
        let mut git = GitInputs::read(&anchored, &policy, &budget).unwrap();
        let mut entries = Vec::new();
        super::collect_project_entries(
            &anchored,
            WalkOptions {
                excluded_relative: None,
                lease_admitted: false,
                record_exclusions: false,
            },
            &policy,
            &budget,
            &mut git,
            &mut entries,
        )
        .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].observed, Some(SourceLeaf::Symlink));
    }

    #[test]
    fn discovery_reports_real_gitlinks_without_following_child_metadata() {
        use std::process::Command;
        let root = tempfile::tempdir().unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .current_dir(root.path())
                .args(args)
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .env_remove("GIT_INDEX_FILE")
                .env_remove("GIT_COMMON_DIR")
                .env_remove("GIT_TEMPLATE_DIR")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env(
                    "GIT_CONFIG_GLOBAL",
                    if cfg!(windows) { "NUL" } else { "/dev/null" },
                )
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init", "--quiet", "--object-format=sha1"]);
        for child in ["z-child", "a-child"] {
            std::fs::create_dir(root.path().join(child)).unwrap();
            std::fs::write(
                root.path().join(child).join(".git"),
                b"gitdir: ../outside\n",
            )
            .unwrap();
            std::fs::write(root.path().join(child).join(".gitignore"), b"first\n").unwrap();
            git(&[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("160000,1111111111111111111111111111111111111111,{child}"),
            ]);
        }
        std::fs::create_dir_all(root.path().join("untracked/.git")).unwrap();
        let (profile, digest) = DefaultInstallProfile::load("default").unwrap();
        let anchor = AnchoredPath::open_root(root.path()).unwrap();
        let first = super::discover_gitlinks(&anchor, &profile, digest.clone()).unwrap();
        assert!(first.git_index_verified);
        assert_eq!(first.candidates, ["a-child", "z-child"]);
        assert!(!first.selection_available);
        assert_eq!(first.coverage, "present_tracked_gitlink_directories_only");
        assert!(!root.path().join(".rrflow").exists());
        std::fs::write(root.path().join("a-child/.git"), b"gitdir: ../elsewhere\n").unwrap();
        std::fs::write(root.path().join("a-child/.gitignore"), b"later\n").unwrap();
        let second = super::discover_gitlinks(&anchor, &profile, digest.clone()).unwrap();
        assert_eq!(first, second, "child metadata must not be read");
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000,2222222222222222222222222222222222222222,a-child",
        ]);
        let third = super::discover_gitlinks(&anchor, &profile, digest).unwrap();
        assert_eq!(first.candidates, third.candidates);
        assert_ne!(first.git_inputs_sha256, third.git_inputs_sha256);
        assert!(!root.path().join(".rrflow").exists());
    }

    #[test]
    fn discovery_classifies_missing_index_and_enforces_existing_bounds() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("source"), b"source").unwrap();
        let anchor = AnchoredPath::open_root(root.path()).unwrap();
        let (mut profile, digest) = DefaultInstallProfile::load("default").unwrap();
        let result = super::discover_gitlinks(&anchor, &profile, digest.clone()).unwrap();
        assert!(!result.git_index_verified);
        assert!(result.candidates.is_empty());
        assert!(!result.selection_available);
        profile.inventory_policy.limits.output_bytes = 1;
        assert!(super::discover_gitlinks(&anchor, &profile, digest)
            .unwrap_err()
            .to_string()
            .contains("discovery output byte bound"));
        assert!(!root.path().join(".rrflow").exists());
    }

    #[test]
    fn linked_worktree_metadata_requires_explicit_admission() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join(".git"), b"gitdir: ../outside\n").unwrap();
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        assert!(project_inventory_sha256(root.path(), &profile)
            .unwrap_err()
            .to_string()
            .contains("requires admission"));
    }

    struct EndlessReader {
        consumed: usize,
        interrupt_once: bool,
    }

    impl Read for EndlessReader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if self.interrupt_once {
                self.interrupt_once = false;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            buffer.fill(b'a');
            self.consumed += buffer.len();
            Ok(buffer.len())
        }
    }

    #[test]
    fn hashing_cannot_read_past_admission_even_when_the_source_never_ends() {
        for length in [0, 1, 65_537] {
            let mut reader = EndlessReader {
                consumed: 0,
                interrupt_once: true,
            };
            let digest = hash_exact_contents(&mut reader, length).unwrap();
            assert_eq!(reader.consumed as u64, length);
            assert_eq!(
                digest,
                rrd_core::digest::sha256_hex(&vec![b'a'; length as usize])
            );
        }
        let mut short = std::io::Cursor::new(b"abc");
        assert!(hash_exact_contents(&mut short, 4).is_err());
    }

    #[test]
    fn observed_file_growth_shrinkage_and_timestamp_changes_fail_closed() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source");
        std::fs::write(&path, b"abc").unwrap();
        let admitted = std::fs::metadata(&path).unwrap();
        assert_eq!(
            hash_inventory_file(&AnchoredPath::from_path(&path).unwrap(), &admitted).unwrap(),
            rrd_core::digest::sha256_hex(b"abc")
        );
        std::fs::write(&path, b"abcdef").unwrap();
        assert!(hash_inventory_file(&AnchoredPath::from_path(&path).unwrap(), &admitted).is_err());
        std::fs::write(&path, b"a").unwrap();
        assert!(hash_inventory_file(&AnchoredPath::from_path(&path).unwrap(), &admitted).is_err());
        std::fs::write(&path, b"abc").unwrap();
        let file = File::options().write(true).open(&path).unwrap();
        let modified = admitted.modified().unwrap() + std::time::Duration::from_secs(60);
        file.set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        assert!(hash_inventory_file(&AnchoredPath::from_path(&path).unwrap(), &admitted).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn secret_exclusions_precede_actual_file_opening() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = tempfile::tempdir().unwrap();
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        std::fs::write(root.path().join("source.rs"), b"source").unwrap();
        let expected = project_inventory_sha256(root.path(), &profile).unwrap();
        std::fs::create_dir(root.path().join(".swag")).unwrap();
        let secrets: Vec<_> = [".env", "server.env", ".swag/session"]
            .into_iter()
            .map(|name| {
                File::options()
                    .create_new(true)
                    .write(true)
                    .read(true)
                    .share_mode(0)
                    .open(root.path().join(name))
                    .unwrap()
            })
            .collect();
        assert_eq!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            expected
        );
        let eligible = File::options()
            .create_new(true)
            .write(true)
            .read(true)
            .share_mode(0)
            .open(root.path().join("ordinary.rs"))
            .unwrap();
        assert!(project_inventory_sha256(root.path(), &profile).is_err());
        drop(eligible);
        drop(secrets);
    }

    #[test]
    fn generated_and_oversized_files_are_excluded_but_hidden_configuration_matters() {
        let root = tempfile::tempdir().unwrap();
        let (mut profile, _) = DefaultInstallProfile::load("default").unwrap();
        profile.inventory_policy.limits.file_bytes = 8;
        let expected = project_inventory_sha256(root.path(), &profile).unwrap();
        std::fs::create_dir(root.path().join("target")).unwrap();
        std::fs::write(root.path().join("target/generated"), b"generated").unwrap();
        std::fs::write(root.path().join("large"), b"oversized source").unwrap();
        assert_eq!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            expected
        );
        std::fs::create_dir(root.path().join(".github")).unwrap();
        std::fs::write(root.path().join(".github/config"), b"config").unwrap();
        let hidden = project_inventory_sha256(root.path(), &profile).unwrap();
        assert_ne!(hidden, expected);
        profile.inventory_policy.limits.file_bytes += 1;
        assert_ne!(
            project_inventory_sha256(root.path(), &profile).unwrap(),
            hidden
        );
    }

    #[test]
    fn metadata_and_depth_budgets_reject_before_complete_precondition() {
        let root = tempfile::tempdir().unwrap();
        let (mut profile, _) = DefaultInstallProfile::load("default").unwrap();
        for number in 0..16 {
            std::fs::write(root.path().join(format!("source-{number}")), b"a").unwrap();
        }
        profile.inventory_policy.limits.metadata_bytes = 1024;
        profile.inventory_policy.limits.git_input_bytes = 1024;
        profile.inventory_policy.limits.path_bytes = 512;
        assert!(project_inventory_sha256(root.path(), &profile)
            .unwrap_err()
            .to_string()
            .contains("metadata byte bound"));
        let (mut profile, _) = DefaultInstallProfile::load("default").unwrap();
        profile.inventory_policy.limits.depth = 1;
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::write(root.path().join("src/nested"), b"a").unwrap();
        assert!(project_inventory_sha256(root.path(), &profile)
            .unwrap_err()
            .to_string()
            .contains("PathLimit"));
    }

    #[test]
    fn expired_budget_performs_no_content_read() {
        let mut reader = EndlessReader {
            consumed: 0,
            interrupt_once: false,
        };
        assert!(matches!(
            super::hash_exact_contents(&mut reader, 10, &WorkBudget::new(0)),
            Err(ServiceError::DeadlineExceeded)
        ));
        assert_eq!(reader.consumed, 0);
    }
}
