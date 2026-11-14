//! Walking the tree, accounting metadata and hashing files.

use super::*;

pub(super) struct WalkOptions<'a> {
    pub(super) excluded_relative: Option<&'a Path>,
    pub(super) lease_admitted: bool,
    pub(super) record_exclusions: bool,
}

pub(super) fn collect_project_entries(
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
                || lease_admitted && relative == Path::new(super::super::lease::RELATIVE_PATH)
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

pub(super) fn account_metadata(
    path: &AnchoredPath,
    accounted: &mut u64,
    ceiling: u64,
) -> Result<()> {
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

pub(super) fn hash_inventory_file(
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

pub(super) fn require_same_observation(admitted: &Metadata, observed: &Metadata) -> Result<()> {
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

pub(super) fn hash_exact_contents(
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

pub(super) fn storage_error(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::Storage(error.to_string())
}
