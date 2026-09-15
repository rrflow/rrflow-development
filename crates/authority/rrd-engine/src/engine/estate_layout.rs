//! One canonical project-local filesystem layout for an installed RRFlow estate.

use super::*;
use rrd_contract::{InstallationManagedPath, InstallationManagedPathKind, InstallationRemovalRule};

pub(super) const ESTATE_DIRECTORY: &str = ".rrflow";
pub(super) const STORAGE_DIRECTORY: &str = ".rrflow/rrd";
pub(super) const STORAGE_ROOTS_DIRECTORY: &str = ".rrflow/rrd/roots";
pub(super) const CREDENTIAL_DIRECTORY: &str = ".rrflow/credentials";
pub(super) const LOCATOR_RELATIVE_PATH: &str = ".rrflow/config.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EstateLayout {
    pub storage_root: String,
    pub token_key: String,
    pub operator_credential: String,
}

impl EstateLayout {
    pub fn new(storage_root_id: &CanonicalId, principal_id: &CanonicalId) -> Result<Self> {
        let layout = Self {
            storage_root: format!("{STORAGE_ROOTS_DIRECTORY}/{storage_root_id}"),
            token_key: format!("{STORAGE_ROOTS_DIRECTORY}/{storage_root_id}/RRD.TOKEN"),
            operator_credential: format!("{CREDENTIAL_DIRECTORY}/{principal_id}.json"),
        };
        for relative in [
            layout.storage_root.as_str(),
            layout.token_key.as_str(),
            layout.operator_credential.as_str(),
            LOCATOR_RELATIVE_PATH,
        ] {
            validate_managed_relative(relative)?;
        }
        Ok(layout)
    }

    pub fn managed_paths(&self, absent_sha256: &str) -> Vec<InstallationManagedPath> {
        [
            (
                InstallationManagedPathKind::EstateDirectory,
                ESTATE_DIRECTORY.to_owned(),
                InstallationRemovalRule::RemoveIfOwnedAndEmpty,
            ),
            (
                InstallationManagedPathKind::StorageDirectory,
                STORAGE_DIRECTORY.to_owned(),
                InstallationRemovalRule::RemoveIfOwnedAndEmpty,
            ),
            (
                InstallationManagedPathKind::StorageRootsDirectory,
                STORAGE_ROOTS_DIRECTORY.to_owned(),
                InstallationRemovalRule::RemoveIfOwnedAndEmpty,
            ),
            (
                InstallationManagedPathKind::CredentialDirectory,
                CREDENTIAL_DIRECTORY.to_owned(),
                InstallationRemovalRule::RemoveIfOwnedAndEmpty,
            ),
            (
                InstallationManagedPathKind::StorageRoot,
                self.storage_root.clone(),
                InstallationRemovalRule::RemoveIfOwnedTreeDigestMatches,
            ),
            (
                InstallationManagedPathKind::TokenKey,
                self.token_key.clone(),
                InstallationRemovalRule::RemoveIfOwnedDigestMatches,
            ),
            (
                InstallationManagedPathKind::OperatorCredential,
                self.operator_credential.clone(),
                InstallationRemovalRule::RemoveIfOwnedDigestMatches,
            ),
            (
                InstallationManagedPathKind::ProjectLocator,
                LOCATOR_RELATIVE_PATH.to_owned(),
                InstallationRemovalRule::RemoveIfOwnedDigestMatches,
            ),
        ]
        .into_iter()
        .map(
            |(kind, relative_path, removal_rule)| InstallationManagedPath {
                kind,
                relative_path,
                precondition_sha256: absent_sha256.to_owned(),
                removal_rule,
            },
        )
        .collect()
    }
}

/// A new install may not coexist with either an old `.rrflow/instance.toml`
/// estate or an unrecognized `.rrflow` tree. Replay is resolved from the
/// canonical locator before this check is called.
pub(super) fn reject_existing_estate(project: &Path) -> Result<()> {
    match std::fs::symlink_metadata(project.join(ESTATE_DIRECTORY)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(ServiceError::ProjectBindingMismatch),
        Err(error) => Err(ServiceError::Storage(error.to_string())),
    }
}

pub(super) fn create_estate_directories(project: &Path) -> Result<()> {
    for relative in [
        ESTATE_DIRECTORY,
        STORAGE_DIRECTORY,
        STORAGE_ROOTS_DIRECTORY,
        CREDENTIAL_DIRECTORY,
    ] {
        let path = project.join(relative);
        if std::fs::symlink_metadata(&path).is_ok() {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let parent = path.parent().ok_or(ServiceError::ProjectBindingMismatch)?;
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt as _;
            builder.mode(0o700);
        }
        builder
            .create(&path)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
        rrd_store::sync_directory_metadata(parent)
            .map_err(|error| ServiceError::Storage(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn reject_existing_symlink_path(project: &Path, path: &Path) -> Result<()> {
    let relative = path
        .strip_prefix(project)
        .map_err(|_| ServiceError::ProjectBindingMismatch)?;
    let mut current = project.to_owned();
    for component in relative.components() {
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ServiceError::ProjectBindingMismatch);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(ServiceError::Storage(error.to_string())),
        }
    }
    Ok(())
}

pub(super) fn validate_managed_relative(relative: &str) -> Result<()> {
    if relative.is_empty()
        || relative.len() > 4_096
        || relative.starts_with('/')
        || relative.contains('\\')
        || relative
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
        || !(relative == ESTATE_DIRECTORY || relative.starts_with(".rrflow/"))
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    Ok(())
}

pub(super) fn safe_join(project: &Path, relative: &str) -> Result<PathBuf> {
    validate_managed_relative(relative)?;
    let path = project.join(relative);
    reject_existing_symlink_path(project, &path)?;
    Ok(path)
}
