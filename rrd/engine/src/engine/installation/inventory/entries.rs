//! Project entries, the work budget and the inventory digest.

use super::*;

pub(super) struct WalkEntry {
    pub(super) path: AnchoredPath,
    pub(super) observed: Option<SourceLeaf>,
    pub(super) gitlink: bool,
    pub(super) excluded: Option<String>,
}

pub(super) struct WorkBudget<'a> {
    pub(super) check_external: &'a dyn Fn() -> Result<()>,
    pub(super) started: Instant,
    pub(super) duration: Duration,
}

impl WorkBudget<'_> {
    pub(super) fn new(elapsed_ms: u64) -> Self {
        Self {
            check_external: &|| Ok(()),
            started: Instant::now(),
            duration: Duration::from_millis(elapsed_ms),
        }
    }
    pub(super) fn check(&self) -> Result<()> {
        (self.check_external)()?;
        if self.started.elapsed() >= self.duration {
            return Err(ServiceError::DeadlineExceeded);
        }
        Ok(())
    }
}

/// One metadata-only entry admitted by the installed project's inventory
/// policy. This projection carries no file content or permission to mutate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::engine) struct ProjectEntry {
    pub relative: String,
    pub directory: bool,
    pub bytes: u64,
}

/// Return the currently admitted project paths using the same bounded,
/// `.gitignore`-aware traversal as installation preconditions.
pub(in crate::engine) fn project_entries(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
) -> Result<Vec<ProjectEntry>> {
    let policy = profile.inventory_policy.compile()?;
    let budget = WorkBudget::new(policy.source.limits.elapsed_ms);
    let mut git = GitInputs::read(project, &policy, &budget)?;
    let mut entries = Vec::new();
    collect_project_entries(
        project,
        WalkOptions {
            excluded_relative: None,
            lease_admitted: super::super::lease::admit_existing(project)?.is_some(),
            record_exclusions: false,
        },
        &policy,
        &budget,
        &mut git,
        &mut entries,
    )?;
    entries.sort_by(|left, right| left.path.relative_path().cmp(right.path.relative_path()));
    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let observed = entry.observed?;
            let SourceLeaf::Ordinary(observed) = observed else {
                return None;
            };
            let relative = entry.path.relative_path().to_str()?.replace('\\', "/");
            Some(ProjectEntry {
                relative,
                directory: observed.kind() == SourceKind::Directory || entry.gitlink,
                bytes: if observed.kind() == SourceKind::File {
                    observed.bytes()
                } else {
                    0
                },
            })
        })
        .collect::<Vec<_>>())
}

pub(in crate::engine) fn project_inventory_sha256(
    project: &AnchoredPath,
    profile: &DefaultInstallProfile,
) -> Result<String> {
    project_inventory_sha256_excluding(project, profile, None)
}
