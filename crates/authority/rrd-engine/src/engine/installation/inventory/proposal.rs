//! Read-only Inventory preparation through the same installation source walk.
//! No stored outcome, source mutation, model call or metadata reuse is admitted here.
use super::*;
use rrd_attunement::inventory::{
    Builder, EntryKind, FileObservation, Limits, Proposal, Revisions, SourceScope,
};

pub(in crate::engine) fn prepare_project_inventory(
    root: &AnchoredPath,
    profile: &DefaultInstallProfile,
    scope: SourceScope,
    prior: Option<&Proposal>,
    check: &dyn Fn() -> Result<()>,
) -> Result<(String, Proposal)> {
    let (precondition, proposal) = scan_project(root, profile, None, Some((scope, prior)), check)?;
    let proposal =
        proposal.ok_or_else(|| ServiceError::Storage("Inventory proposal missing".into()))?;
    if !proposal.is_complete() {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    Ok((precondition, proposal))
}

pub(super) fn builder<'a>(
    scope: SourceScope,
    prior: Option<&'a Proposal>,
    policy: &CompiledPolicy<'_>,
    git: &GitInputs,
) -> Result<Builder<'a>> {
    let limits = &policy.source.limits;
    let effective_policy = rrd_core::digest::sha256_hex(
        &serde_json::to_vec(&(
            "rrflow-effective-inventory-policy-v1",
            &policy.sha256,
            git.identity()?,
        ))
        .map_err(storage_error)?,
    );
    Builder::new(
        scope,
        Revisions {
            policy_sha256: effective_policy,
            walker: "anchored-git-walk-v1".into(),
            detector: "content-only-v1".into(),
        },
        Limits {
            entries: limits.entries,
            depth: limits.depth,
            path_bytes: limits.path_bytes,
            metadata_bytes: limits.metadata_bytes,
            file_bytes: limits.file_bytes,
            content_bytes: limits.content_bytes,
        },
        prior,
    )
    .map_err(storage_error)
}

pub(super) fn ingest_leaf(
    builder: &mut Builder<'_>,
    relative: &str,
    entry: &WalkEntry,
) -> Result<()> {
    let kind = if entry.gitlink {
        EntryKind::Excluded {
            rule: "git-submodule".into(),
        }
    } else {
        match entry
            .observed
            .as_ref()
            .ok_or(ServiceError::ProjectBindingMismatch)?
        {
            SourceLeaf::Ordinary(obs) if obs.kind() == SourceKind::Directory => {
                return builder.directory(relative).map_err(storage_error)
            }
            SourceLeaf::Symlink => EntryKind::Symlink {
                rule: "record-without-following".into(),
            },
            SourceLeaf::Mount => EntryKind::Mount {
                rule: "record-without-following".into(),
            },
            SourceLeaf::Special => EntryKind::Excluded {
                rule: "special-object".into(),
            },
            _ => return Err(ServiceError::ProjectBindingMismatch),
        }
    };
    builder.leaf(relative, kind).map_err(storage_error)
}

pub(super) fn ingest_file(
    builder: &mut Builder<'_>,
    relative: &str,
    entry: &WalkEntry,
    budget: &WorkBudget<'_>,
) -> Result<String> {
    let Some(SourceLeaf::Ordinary(observation)) = &entry.observed else {
        return Err(ServiceError::ProjectBindingMismatch);
    };
    let path = &entry.path;
    let mut file = path.open_read().map_err(storage_error)?;
    if path
        .source_observation_for_file(&file)
        .map_err(storage_error)?
        != *observation
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let facts = FileObservation {
        bytes: observation.bytes(),
        executable: observation.executable_mode(),
        revision: observation.revision_sha256().map_err(storage_error)?,
        classification_sha256: rrd_core::digest::sha256_hex(b"rrflow-content-only-v1:unclassified"),
    };
    let mut remaining = observation.bytes();
    let mut failure = None;
    let chunks = std::iter::from_fn(|| {
        if remaining == 0 {
            return None;
        }
        if let Err(error) = budget.check() {
            remaining = 0;
            failure = Some(error);
            return Some(Err(()));
        }
        let mut bytes = vec![0; remaining.min(64 * 1024) as usize];
        match file.read(&mut bytes) {
            Ok(0) => {
                remaining = 0;
                failure = Some(ServiceError::ProjectBindingMismatch);
                Some(Err(()))
            }
            Ok(count) => {
                remaining -= count as u64;
                bytes.truncate(count);
                Some(Ok(bytes))
            }
            Err(error) => {
                remaining = 0;
                failure = Some(storage_error(error));
                Some(Err(()))
            }
        }
    });
    // The prior is used for changes, never as implicit native reuse authority.
    let result = builder.file_fallible(relative, facts, chunks, false);
    if let Some(error) = failure {
        return Err(error);
    }
    result.map_err(storage_error)?;
    budget.check()?;
    if path
        .source_observation_for_file(&file)
        .map_err(storage_error)?
        != *observation
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    builder
        .content_sha256(relative)
        .map(str::to_owned)
        .ok_or_else(|| ServiceError::Storage("Inventory file digest missing".into()))
}

pub(super) fn require_output_bound(value: &impl serde::Serialize, limit: u64) -> Result<()> {
    struct Counter {
        bytes: u64,
        limit: u64,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(bytes.len() as u64)
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| std::io::Error::other("Inventory output byte bound exceeded"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(&mut Counter { bytes: 0, limit }, value).map_err(|_| {
        ServiceError::ConfigurationLimit("Inventory output byte bound exceeded".into())
    })
}
