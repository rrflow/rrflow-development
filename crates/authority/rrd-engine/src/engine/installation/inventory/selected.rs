//! Reading and revalidating admitted and selected inventory paths, and the change fingerprint.

use super::*;

/// Read one already admitted inventory file through its anchored parent. The
/// caller supplies only the engine-resolved relative path and committed file
/// facts; neither a client path nor ambient filesystem I/O is accepted here.
pub(in crate::engine) fn read_admitted_inventory_file(
    root: &AnchoredPath,
    relative: &Path,
    expected: &rrd_attunement::inventory::FileObservation,
    max_bytes: u64,
    elapsed_ms: u64,
    check: &dyn Fn() -> Result<()>,
) -> Result<(Vec<u8>, String)> {
    let path = root.join(relative);
    let budget = WorkBudget {
        check_external: check,
        ..WorkBudget::new(elapsed_ms)
    };
    budget.check()?;
    let before = match path.source_leaf().map_err(storage_error)? {
        SourceLeaf::Ordinary(observation) if observation.kind() == SourceKind::File => observation,
        _ => return Err(ServiceError::ProjectBindingMismatch),
    };
    if before.kind() != SourceKind::File
        || before.bytes() != expected.bytes
        || before.executable_mode() != expected.executable
        || before.revision_sha256().map_err(storage_error)? != expected.revision
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let admitted = path.metadata().map_err(storage_error)?;
    if admitted.file_type().is_symlink() || !admitted.is_file() || admitted.len() > max_bytes {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let mut file = path.open_read().map_err(storage_error)?;
    if before
        != path
            .source_observation_for_file(&file)
            .map_err(storage_error)?
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    require_same_observation(&admitted, &file.metadata().map_err(storage_error)?)?;
    let capacity = usize::try_from(admitted.len()).map_err(|_| {
        ServiceError::ConfigurationLimit("selected source exceeds addressable memory".into())
    })?;
    let mut bytes = Vec::with_capacity(capacity);
    let mut buffer = [0u8; 16 * 1024];
    loop {
        budget.check()?;
        let read = file.read(&mut buffer).map_err(storage_error)?;
        if read == 0 {
            break;
        }
        if (bytes.len() as u64).saturating_add(read as u64) > max_bytes
            || bytes.len().saturating_add(read) > capacity
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    require_same_observation(&admitted, &file.metadata().map_err(storage_error)?)?;
    require_same_observation(&admitted, &path.metadata().map_err(storage_error)?)?;
    if bytes.len() != capacity
        || before
            != path
                .source_observation_for_file(&file)
                .map_err(storage_error)?
        || !matches!(
            path.source_leaf().map_err(storage_error)?,
            SourceLeaf::Ordinary(observation) if observation == before
        )
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    budget.check()?;
    let content_sha256 = rrd_core::digest::sha256_hex(&bytes);
    Ok((bytes, content_sha256))
}

/// Re-evaluate whether one previously selected root-relative path is currently
/// eligible under the same static exclusions and Git traversal rules as a full
/// inventory. This is intentionally path-scoped: unrelated index entries and
/// ignore files do not invalidate the selection, while every applicable
/// ancestor ignore input and tracked override is freshly observed.
pub(in crate::engine) fn validate_selected_inventory_path(
    root: &AnchoredPath,
    profile: &DefaultInstallProfile,
    relative: &Path,
    maximum_bytes: u64,
    check: &dyn Fn() -> Result<()>,
) -> Result<()> {
    retry_selected_path_observation(|| {
        validate_selected_inventory_path_once(root, profile, relative, maximum_bytes, check)
    })
}

pub(super) fn retry_selected_path_observation(
    mut observe: impl FnMut() -> Result<()>,
) -> Result<()> {
    for attempt in 0..3 {
        match observe() {
            Err(ServiceError::AttunementSourceChanged) if attempt < 2 => continue,
            result => return result,
        }
    }
    Err(ServiceError::AttunementSourceChanged)
}

pub(super) fn validate_selected_inventory_path_once(
    root: &AnchoredPath,
    profile: &DefaultInstallProfile,
    relative: &Path,
    maximum_bytes: u64,
    check: &dyn Fn() -> Result<()>,
) -> Result<()> {
    use std::path::Component;

    let policy = profile.inventory_policy.compile()?;
    let normalized = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => value.to_str().ok_or(ServiceError::ProjectBindingMismatch),
            _ => Err(ServiceError::ProjectBindingMismatch),
        })
        .collect::<Result<Vec<_>>>()?
        .join("/");
    if normalized.is_empty()
        || normalized.len() as u64 > policy.source.limits.path_bytes
        || normalized.split('/').count() as u64 > policy.source.limits.depth
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let parts: Vec<_> = normalized.split('/').collect();
    let mut budget = WorkBudget::new(policy.source.limits.elapsed_ms);
    budget.check_external = check;
    let mut git = GitInputs::read_for_selected_path(root, &policy, &budget)?;
    let mut metadata_bytes = 0u64;
    let mut current = root.to_owned();
    git.enter_selected_path_directory(&current, &policy, &budget)?;

    for part in &parts[..parts.len() - 1] {
        budget.check()?;
        let child = current.join(part);
        let path = child
            .relative_path()
            .to_str()
            .ok_or(ServiceError::ProjectBindingMismatch)?
            .replace('\\', "/");
        if policy.excluded(&path, false)?.is_some() || git.excluded_without_metadata(&path)? {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let observed = child.source_leaf().map_err(storage_error)?;
        let metadata_ceiling = policy
            .source
            .limits
            .metadata_bytes
            .checked_sub(git.metadata_bytes)
            .ok_or_else(|| {
                ServiceError::ConfigurationLimit("inventory metadata byte bound exceeded".into())
            })?;
        account_metadata(&child, &mut metadata_bytes, metadata_ceiling)?;
        let SourceLeaf::Ordinary(metadata) = &observed else {
            return Err(ServiceError::ProjectBindingMismatch);
        };
        let is_directory = metadata.kind() == SourceKind::Directory;
        if !is_directory
            || policy.excluded(&path, true)?.is_some()
            || !git.admits(&path, true)?
            || git.is_submodule(&path)
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        current = child;
        git.enter_selected_path_directory(&current, &policy, &budget)?;
    }

    budget.check()?;
    let child = current.join(parts[parts.len() - 1]);
    let path = child
        .relative_path()
        .to_str()
        .ok_or(ServiceError::ProjectBindingMismatch)?
        .replace('\\', "/");
    if policy.excluded(&path, false)?.is_some() || git.excluded_without_metadata(&path)? {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let observed = child.source_leaf().map_err(storage_error)?;
    let metadata_ceiling = policy
        .source
        .limits
        .metadata_bytes
        .checked_sub(git.metadata_bytes)
        .ok_or_else(|| {
            ServiceError::ConfigurationLimit("inventory metadata byte bound exceeded".into())
        })?;
    account_metadata(&child, &mut metadata_bytes, metadata_ceiling)?;
    let SourceLeaf::Ordinary(metadata) = observed else {
        return Err(ServiceError::ProjectBindingMismatch);
    };
    if metadata.kind() != SourceKind::File
        || metadata.bytes() > maximum_bytes.min(policy.source.limits.file_bytes)
        || policy.excluded(&path, false)?.is_some()
        || !git.admits(&path, false)?
        || git.is_submodule(&path)
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    git.revalidate_selected(&budget)?;
    budget.check()?;
    Ok(())
}

/// A cheap fingerprint of the project as the Inventory would see it: the same
/// walk, exclusions and Git inputs, hashing each admitted entry's path and
/// filesystem observation (size, times, permissions and object identity)
/// instead of its content. It changes whenever a file is added, removed,
/// renamed or written; equal fingerprints may still hide an edit that kept
/// every observation, which the content-addressed Inventory would catch.
pub(in crate::engine) fn project_change_fingerprint(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
) -> Result<String> {
    let policy = profile.inventory_policy.compile()?;
    let budget = WorkBudget::new(policy.source.limits.elapsed_ms);
    let mut git = GitInputs::read(project, &policy, &budget)?;
    let mut entries = Vec::<WalkEntry>::new();
    let admitted_lease = super::super::lease::admit_existing(project)?;
    collect_project_entries(
        project,
        WalkOptions {
            excluded_relative: None,
            lease_admitted: admitted_lease.is_some(),
            record_exclusions: false,
        },
        &policy,
        &budget,
        &mut git,
        &mut entries,
    )?;
    entries.sort_by(|left, right| left.path.relative_path().cmp(right.path.relative_path()));
    let mut aggregate = IncrementalSha256::new();
    aggregate.update(b"rrflow-project-change-fingerprint-v1\0");
    aggregate.update(policy.sha256.as_bytes());
    aggregate.update(git.identity()?.as_bytes());
    for entry in entries.iter().filter(|entry| entry.excluded.is_none()) {
        budget.check()?;
        let relative = entry.path.relative_path();
        let relative = relative.to_string_lossy();
        aggregate.update(&(relative.len() as u64).to_be_bytes());
        aggregate.update(relative.as_bytes());
        let observation = match entry.observed.as_ref() {
            Some(SourceLeaf::Ordinary(observation)) => {
                observation.revision_sha256().map_err(storage_error)?
            }
            Some(other) => format!("{other:?}"),
            None => String::new(),
        };
        aggregate.update(observation.as_bytes());
    }
    Ok(aggregate.finalize_hex())
}

pub(in super::super) fn project_inventory_sha256_excluding(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
    excluded_relative: Option<&Path>,
) -> Result<String> {
    scan_project(project, profile, excluded_relative, None, &|| Ok(())).map(|(digest, _)| digest)
}
