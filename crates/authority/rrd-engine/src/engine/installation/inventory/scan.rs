//! Scanning a project, revalidating observations and discovering gitlinks.

use super::*;

pub(super) fn scan_project(
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
    let admitted_lease = super::super::lease::admit_existing(project)?;
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
pub(super) fn revalidate_observations(
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
pub(in super::super) fn discover_gitlinks(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
    profile_sha256: String,
) -> Result<super::super::NestedRepositoryDiscovery> {
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
    let result = super::super::NestedRepositoryDiscovery {
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
