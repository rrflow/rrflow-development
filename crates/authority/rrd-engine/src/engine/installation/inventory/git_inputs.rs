//! Engine-owned, root-relative Git inputs. The compute crate only sees bytes.
use super::{storage_error, CompiledPolicy, Result, ServiceError, WorkBudget};
use rrd_attunement::git::{
    IgnoreInput, IgnoreLimits, IgnoreOrigin, IgnoreRules, ParseLimits, RepositoryConfig,
    TrackedKind, TrackedPaths,
};
use rrd_core::digest::sha256_hex;
use rrd_store::filesystem::{AnchoredPath, SourceKind, SourceLeaf};
use std::io::{ErrorKind, Read};

pub(super) struct GitInputs {
    config: Option<RepositoryConfig>,
    tracked: Option<TrackedPaths>,
    layers: Vec<(IgnoreOrigin, Vec<u8>)>,
    rules: IgnoreRules,
    observations: Vec<(AnchoredPath, Option<SourceLeaf>)>,
    identities: Vec<(String, Option<String>)>,
    bytes: u64,
    pub metadata_bytes: u64,
    metadata_ceiling: u64,
    limits: IgnoreLimits,
    retry_observed_drift: bool,
}

impl GitInputs {
    pub fn read(
        root: &AnchoredPath,
        policy: &CompiledPolicy<'_>,
        budget: &WorkBudget,
    ) -> Result<Self> {
        Self::read_with_drift_mode(root, policy, budget, false)
    }

    pub fn read_for_selected_path(
        root: &AnchoredPath,
        policy: &CompiledPolicy<'_>,
        budget: &WorkBudget,
    ) -> Result<Self> {
        Self::read_with_drift_mode(root, policy, budget, true)
    }

    fn read_with_drift_mode(
        root: &AnchoredPath,
        policy: &CompiledPolicy<'_>,
        budget: &WorkBudget,
        retry_observed_drift: bool,
    ) -> Result<Self> {
        let bound = &policy.source.limits;
        let limits = IgnoreLimits {
            parse: ParseLimits {
                input_bytes: bound.git_input_bytes as usize,
                entries: bound.git_entries as usize,
                path_bytes: bound.path_bytes as usize,
                depth: bound.depth as usize,
                allocation_bytes: bound.git_allocation_bytes as usize,
            },
            sources: bound.ignore_sources as usize,
            patterns: bound.ignore_patterns as usize,
        };
        let mut input = Self {
            config: None,
            tracked: None,
            layers: Vec::new(),
            rules: IgnoreRules::from_inputs(&[], limits, false).map_err(storage_error)?,
            observations: Vec::new(),
            identities: Vec::new(),
            bytes: 0,
            metadata_bytes: 0,
            metadata_ceiling: bound.metadata_bytes,
            limits,
            retry_observed_drift,
        };
        budget.check()?;
        let marker = root.join(".git");
        let observed = observe(&marker)?;
        let present = match &observed {
            None => false,
            Some(SourceLeaf::Ordinary(obs)) if obs.kind() == SourceKind::Directory => true,
            _ => {
                return Err(unsupported(
                    "external or non-directory Git metadata requires admission",
                ))
            }
        };
        super::account_metadata(&marker, &mut input.metadata_bytes, input.metadata_ceiling)?;
        input.observations.push((marker, observed));
        if !present {
            return Ok(input);
        }
        let config = input
            .read_optional(&root.join(".git/config"), budget)?
            .unwrap_or_default();
        let config = RepositoryConfig::from_bytes(&config, limits.parse).map_err(storage_error)?;
        if let Some(index) = input.read_optional(&root.join(".git/index"), budget)? {
            let paths = TrackedPaths::from_bytes(&index, config.object_format, limits.parse)
                .map_err(storage_error)?;
            if !paths.checksum_present() {
                return Err(unsupported("Git index has no verified checksum"));
            }
            input.tracked = Some(paths);
        }
        // A common directory/config-worktree requires a separately admitted root
        // and configuration model. Never silently interpret it as this repository.
        for name in [".git/commondir", ".git/config.worktree"] {
            if input.read_optional(&root.join(name), budget)?.is_some() {
                return Err(unsupported(
                    "additional Git configuration requires admission",
                ));
            }
        }
        if let Some(bytes) = input.read_optional(&root.join(".git/info/exclude"), budget)? {
            input.layers.push((IgnoreOrigin::Repository, bytes));
        }
        input.config = Some(config);
        input.compile()?;
        Ok(input)
    }

    pub fn enter(
        &mut self,
        directory: &AnchoredPath,
        policy: &CompiledPolicy<'_>,
        budget: &WorkBudget,
    ) -> Result<()> {
        self.enter_with_drift_mode(directory, policy, budget, false)
    }

    pub fn enter_selected_path_directory(
        &mut self,
        directory: &AnchoredPath,
        policy: &CompiledPolicy<'_>,
        budget: &WorkBudget,
    ) -> Result<()> {
        self.enter_with_drift_mode(directory, policy, budget, true)
    }

    fn enter_with_drift_mode(
        &mut self,
        directory: &AnchoredPath,
        policy: &CompiledPolicy<'_>,
        budget: &WorkBudget,
        retry_observed_drift: bool,
    ) -> Result<()> {
        if retry_observed_drift != self.retry_observed_drift {
            return Err(unsupported("Git input observation mode changed"));
        }
        if self.config.is_none() {
            return Ok(());
        }
        let path = directory.join(".gitignore");
        let relative = normalized(&path)?;
        if policy.excluded(&relative, false)?.is_some() {
            return Ok(());
        }
        if let Some(bytes) = self.read_optional(&path, budget)? {
            if self.layers.len() >= self.limits.sources {
                return Err(unsupported("Git ignore source bound exceeded"));
            }
            self.layers.push((
                IgnoreOrigin::Worktree {
                    directory: normalized(directory)?,
                },
                bytes,
            ));
            self.compile()?;
        }
        Ok(())
    }

    fn compile(&mut self) -> Result<()> {
        let inputs: Vec<_> = self
            .layers
            .iter()
            .map(|(origin, bytes)| IgnoreInput {
                origin: origin.clone(),
                bytes,
            })
            .collect();
        self.rules = IgnoreRules::from_inputs(&inputs, self.limits, self.ignore_case())
            .map_err(storage_error)?;
        Ok(())
    }

    fn ignore_case(&self) -> bool {
        self.config.as_ref().is_some_and(|c| c.ignore_case)
    }

    pub fn admits(&self, path: &str, directory: bool) -> Result<bool> {
        if self.config.is_none() {
            return Ok(true);
        }
        if let Some(tracked) = &self.tracked {
            match tracked.kind(path, self.ignore_case()) {
                Some(TrackedKind::Conflicted) => {
                    return Err(unsupported("conflicted Git source requires admission"));
                }
                Some(_) => return Ok(true),
                None => {}
            }
            if directory && tracked.has_descendants(path, self.ignore_case()) {
                return Ok(true);
            }
        }
        Ok(self
            .rules
            .ignored(path, directory)
            .map_err(storage_error)?
            .is_none())
    }

    pub fn excluded_without_metadata(&self, path: &str) -> Result<bool> {
        if self.config.is_none() {
            return Ok(false);
        }
        if self.tracked.as_ref().is_some_and(|t| {
            t.kind(path, self.ignore_case()).is_some()
                || t.has_descendants(path, self.ignore_case())
        }) {
            return Ok(false);
        }
        Ok(self
            .rules
            .ignored(path, false)
            .map_err(storage_error)?
            .is_some())
    }

    /// True only when the root index was present and its checksum verified.
    pub fn has_verified_index(&self) -> bool {
        self.tracked.is_some()
    }

    pub fn is_submodule(&self, path: &str) -> bool {
        self.tracked
            .as_ref()
            .and_then(|t| t.kind(path, self.ignore_case()))
            == Some(TrackedKind::Submodule)
    }

    pub fn identity(&self) -> Result<String> {
        let mut identities: Vec<_> = self.identities.iter().collect();
        identities.sort_by(|a, b| a.0.cmp(&b.0));
        let encoded =
            serde_json::to_vec(&("rrflow-git-inputs-v1", self.config.is_some(), &identities))
                .map_err(storage_error)?;
        Ok(sha256_hex(&encoded))
    }

    pub fn revalidate(&self, budget: &WorkBudget) -> Result<()> {
        for (path, before) in &self.observations {
            budget.check()?;
            if observe(path)? != *before {
                return Err(ServiceError::ProjectBindingMismatch);
            }
        }
        Ok(())
    }

    /// Selected-path admission may retry a bounded local re-evaluation when a
    /// Git input changes during observation. A stable exclusion remains a
    /// denial; it is never converted into an eligibility retry.
    pub fn revalidate_selected(&self, budget: &WorkBudget) -> Result<()> {
        if !self.retry_observed_drift {
            return Err(unsupported("selected Git validation mode is absent"));
        }
        for (path, before) in &self.observations {
            budget.check()?;
            if observe(path)? != *before {
                return Err(ServiceError::AttunementSourceChanged);
            }
        }
        Ok(())
    }

    fn read_optional(
        &mut self,
        path: &AnchoredPath,
        budget: &WorkBudget,
    ) -> Result<Option<Vec<u8>>> {
        self.read_optional_after_observation(path, budget, || Ok(()))
    }

    fn read_optional_after_observation(
        &mut self,
        path: &AnchoredPath,
        budget: &WorkBudget,
        after_observation: impl FnOnce() -> Result<()>,
    ) -> Result<Option<Vec<u8>>> {
        self.read_optional_with_after_open(path, budget, after_observation, || Ok(()))
    }

    fn read_optional_with_after_open(
        &mut self,
        path: &AnchoredPath,
        budget: &WorkBudget,
        after_observation: impl FnOnce() -> Result<()>,
        after_open: impl FnOnce() -> Result<()>,
    ) -> Result<Option<Vec<u8>>> {
        budget.check()?;
        // Missing inputs count too: repeated absent ignore files cannot consume
        // an unbounded observation vector or silently appear during the walk.
        if self.observations.len() >= self.limits.sources + 6 {
            return Err(unsupported("Git input observation bound exceeded"));
        }
        super::account_metadata(path, &mut self.metadata_bytes, self.metadata_ceiling)?;
        let before = observe(path)?;
        after_observation()?;
        let bytes = match &before {
            None => None,
            Some(SourceLeaf::Ordinary(obs)) if obs.kind() == SourceKind::File => {
                self.bytes = self
                    .bytes
                    .checked_add(obs.bytes())
                    .filter(|n| *n <= self.limits.parse.input_bytes as u64)
                    .ok_or_else(|| unsupported("Git input byte bound exceeded"))?;
                let mut file = path.open_read().map_err(storage_error)?;
                if path
                    .source_observation_for_file(&file)
                    .map_err(storage_error)?
                    != *obs
                {
                    return Err(self.observed_drift_error());
                }
                after_open()?;
                let mut bytes = vec![0; obs.bytes() as usize];
                // Bound each read and recheck elapsed work between chunks.
                for chunk in bytes.chunks_mut(64 * 1024) {
                    budget.check()?;
                    if let Err(error) = file.read_exact(chunk) {
                        // A short read is retryable only when the anchored path
                        // now positively observes a different Git input. Keep
                        // stable I/O failures as storage errors.
                        if observe(path)? != before {
                            return Err(self.observed_drift_error());
                        }
                        return Err(storage_error(error));
                    }
                }
                if path
                    .source_observation_for_file(&file)
                    .map_err(storage_error)?
                    != *obs
                {
                    return Err(self.observed_drift_error());
                }
                Some(bytes)
            }
            _ => return Err(unsupported("Git input is not an admitted ordinary file")),
        };
        if observe(path)? != before {
            return Err(self.observed_drift_error());
        }
        self.identities
            .push((normalized(path)?, bytes.as_ref().map(|b| sha256_hex(b))));
        self.observations.push((path.clone(), before));
        Ok(bytes)
    }

    fn observed_drift_error(&self) -> ServiceError {
        if self.retry_observed_drift {
            ServiceError::AttunementSourceChanged
        } else {
            ServiceError::ProjectBindingMismatch
        }
    }
}

fn observe(path: &AnchoredPath) -> Result<Option<SourceLeaf>> {
    match path.source_leaf() {
        Ok(observed) => Ok(Some(observed)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(storage_error(error)),
    }
}

fn normalized(path: &AnchoredPath) -> Result<String> {
    path.relative_path()
        .to_str()
        .map(|p| p.replace('\\', "/"))
        .ok_or(ServiceError::ProjectBindingMismatch)
}

fn unsupported(message: &str) -> ServiceError {
    ServiceError::Contract(format!("inventory Git input: {message}"))
}

#[cfg(test)]
mod tests {
    use super::super::{DefaultInstallProfile, WorkBudget};
    use super::*;

    #[test]
    fn selected_git_read_observation_drift_is_retryable() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join(".git")).unwrap();
        std::fs::write(
            directory.path().join(".git/config"),
            b"[core]\nrepositoryFormatVersion=0\n",
        )
        .unwrap();
        std::fs::write(directory.path().join(".gitignore"), b"alpha.txt\n").unwrap();
        let root = AnchoredPath::open_root(directory.path()).unwrap();
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        let policy = profile.inventory_policy.compile().unwrap();
        let budget = WorkBudget::new(policy.source.limits.elapsed_ms);
        let mut git = GitInputs::read_for_selected_path(&root, &policy, &budget).unwrap();
        let error = git
            .read_optional_after_observation(&root.join(".gitignore"), &budget, || {
                std::fs::write(directory.path().join(".gitignore"), b"alpha.tmp\n").unwrap();
                Ok(())
            })
            .unwrap_err();
        assert!(matches!(error, ServiceError::AttunementSourceChanged));

        let mut ordinary = GitInputs::read(&root, &policy, &budget).unwrap();
        let error = ordinary
            .read_optional_after_observation(&root.join(".gitignore"), &budget, || {
                std::fs::write(directory.path().join(".gitignore"), b"alpha.bak\n").unwrap();
                Ok(())
            })
            .unwrap_err();
        assert!(matches!(error, ServiceError::ProjectBindingMismatch));
    }

    #[test]
    fn selected_git_short_read_retries_only_after_observed_input_drift() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join(".git")).unwrap();
        std::fs::write(
            directory.path().join(".git/config"),
            b"[core]\nrepositoryFormatVersion=0\n",
        )
        .unwrap();
        std::fs::write(directory.path().join(".gitignore"), b"alpha.txt\n").unwrap();
        let root = AnchoredPath::open_root(directory.path()).unwrap();
        let (profile, _) = DefaultInstallProfile::load("default").unwrap();
        let policy = profile.inventory_policy.compile().unwrap();
        let budget = WorkBudget::new(policy.source.limits.elapsed_ms);
        let mut selected = GitInputs::read_for_selected_path(&root, &policy, &budget).unwrap();
        let error = selected
            .read_optional_with_after_open(
                &root.join(".gitignore"),
                &budget,
                || Ok(()),
                || {
                    std::fs::write(directory.path().join(".gitignore"), b"").unwrap();
                    Ok(())
                },
            )
            .unwrap_err();
        assert!(matches!(error, ServiceError::AttunementSourceChanged));

        std::fs::write(directory.path().join(".gitignore"), b"alpha.txt\n").unwrap();
        let mut ordinary = GitInputs::read(&root, &policy, &budget).unwrap();
        let error = ordinary
            .read_optional_with_after_open(
                &root.join(".gitignore"),
                &budget,
                || Ok(()),
                || {
                    std::fs::write(directory.path().join(".gitignore"), b"").unwrap();
                    Ok(())
                },
            )
            .unwrap_err();
        assert!(matches!(error, ServiceError::ProjectBindingMismatch));
    }
}
