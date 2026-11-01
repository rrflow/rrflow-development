//! Versioned bundled inventory policy, admitted by the existing installation
//! profile digest. This is not a mutable settings or external-root grant API.

use super::{contract_json, CanonicalId, Result, ServiceError};
use rrd_attunement::git::{IgnoreInput, IgnoreLimits, IgnoreOrigin, IgnoreRules, ParseLimits};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::engine) struct InventoryPolicy {
    pub format_version: u16,
    pub logical_root: CanonicalId,
    pub git: GitPolicy,
    pub symlinks: BoundaryPolicy,
    pub mounts: BoundaryPolicy,
    pub reuse: ReusePolicy,
    pub limits: InventoryLimits,
    pub exclusions: Vec<ExclusionRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::engine) enum GitPolicy {
    /// Inspect only metadata under the admitted project root. External worktree
    /// metadata and ambient global configuration are not implicitly authorized.
    DetectAtRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::engine) enum BoundaryPolicy {
    RecordWithoutFollowing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::engine) enum ReusePolicy {
    /// Requires engine proof of stable native identity and a usable filesystem
    /// timestamp resolution/clock boundary; ambiguity always forces a read.
    ConservativeNativeObservation,
    ReadContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::engine) struct InventoryLimits {
    pub entries: u64,
    pub depth: u64,
    pub path_bytes: u64,
    pub metadata_bytes: u64,
    pub file_bytes: u64,
    pub content_bytes: u64,
    pub output_bytes: u64,
    pub elapsed_ms: u64,
    pub open_files: u64,
    pub workers: u64,
    pub attempts: u64,
    pub git_input_bytes: u64,
    pub git_entries: u64,
    pub git_allocation_bytes: u64,
    pub ignore_sources: u64,
    pub ignore_patterns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::engine) struct ExclusionRule {
    pub id: CanonicalId,
    /// One positive Git-style pattern. No negation can override a safety rule.
    pub pattern: String,
}

pub(super) struct CompiledPolicy<'a> {
    pub source: &'a InventoryPolicy,
    safety: IgnoreRules,
    pub sha256: String,
}

impl InventoryPolicy {
    pub(super) fn validate(&self) -> Result<()> {
        if self.format_version != 1 || self.exclusions.is_empty() {
            return Err(invalid("unsupported format or missing safety rules"));
        }
        // Protocol ceilings prevent conversions/aggregate bounds from depending
        // on host pointer width. Effective operating values live in profile data.
        let limits = serde_json::to_value(&self.limits).map_err(contract_json)?;
        for (name, value) in limits.as_object().expect("limits serialize as object") {
            let value = value.as_u64().expect("limits are u64");
            if value == 0 || value > u64::from(u32::MAX) {
                return Err(invalid(&format!(
                    "{name} is outside the positive protocol range"
                )));
            }
        }
        if self.limits.workers > self.limits.open_files
            || self.limits.path_bytes > self.limits.metadata_bytes
            || self.limits.git_input_bytes > self.limits.metadata_bytes
            || self.exclusions.len() as u64 > self.limits.ignore_patterns
        {
            return Err(invalid("inconsistent resource bounds"));
        }
        let mut ids = BTreeSet::new();
        let mut bytes = 0u64;
        for rule in &self.exclusions {
            if !ids.insert(&rule.id)
                || rule.pattern.trim().is_empty()
                || rule.pattern.trim() != rule.pattern
                || rule.pattern.starts_with(['!', '#'])
                || rule.pattern.trim_matches('/').is_empty()
                || rule.pattern.chars().any(char::is_control)
                || rule.pattern.len() as u64 > self.limits.path_bytes
            {
                return Err(invalid("invalid, duplicate or non-positive safety rule"));
            }
            bytes = bytes
                .checked_add(rule.pattern.len() as u64 + 1)
                .ok_or_else(|| invalid("rule byte count overflow"))?;
        }
        if bytes > self.limits.git_input_bytes {
            return Err(invalid("safety rules exceed metadata input bound"));
        }
        Ok(())
    }

    pub(super) fn compile(&self) -> Result<CompiledPolicy<'_>> {
        self.validate()?;
        let bytes = self
            .exclusions
            .iter()
            .map(|rule| format!("{}\n", rule.pattern))
            .collect::<String>();
        let limits = &self.limits;
        let safety = IgnoreRules::from_inputs(
            &[IgnoreInput {
                origin: IgnoreOrigin::Global,
                bytes: bytes.as_bytes(),
            }],
            IgnoreLimits {
                parse: ParseLimits {
                    input_bytes: limits.git_input_bytes as usize,
                    entries: limits.git_entries as usize,
                    path_bytes: limits.path_bytes as usize,
                    depth: limits.depth as usize,
                    allocation_bytes: limits.git_allocation_bytes as usize,
                },
                sources: limits.ignore_sources as usize,
                patterns: limits.ignore_patterns as usize,
            },
            // Safety exclusions are case-insensitive on every platform. Git
            // repository matching has its own separately admitted case setting.
            true,
        )
        .map_err(|_| invalid("safety rule compilation failed"))?;
        Ok(CompiledPolicy {
            source: self,
            safety,
            sha256: super::json_sha256(self)?,
        })
    }
}

impl CompiledPolicy<'_> {
    pub fn excluded(&self, path: &str, directory: bool) -> Result<Option<&str>> {
        self.safety
            .ignored(path, directory)
            .map_err(|error| invalid(&error.to_string()))?
            .map(|matched| {
                self.source
                    .exclusions
                    .get(matched.line.saturating_sub(1))
                    .map(|rule| rule.id.as_str())
                    .ok_or_else(|| invalid("invalid safety rule identity"))
            })
            .transpose()
    }
}

fn invalid(message: &str) -> ServiceError {
    ServiceError::Contract(format!("inventory policy: {message}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn bundled_policy_preserves_exclusion_identity_and_rejects_invalid_rules() {
        let (profile, _) = super::super::DefaultInstallProfile::load("default").unwrap();
        let policy = &profile.inventory_policy;
        let compiled = policy.compile().unwrap();
        assert_eq!(
            compiled.excluded(".ENV", false).unwrap(),
            Some("environment-secrets")
        );
        assert_eq!(
            compiled.excluded("config/server.env", false).unwrap(),
            Some("environment-suffix")
        );
        assert_eq!(
            compiled.excluded(".swag/session", false).unwrap(),
            Some("swag-local-state")
        );
        assert_eq!(
            compiled.excluded("src/private.key", false).unwrap(),
            Some("private-keys")
        );
        assert_eq!(
            compiled.excluded("target/output", false).unwrap(),
            Some("rust-build")
        );
        assert_eq!(
            compiled
                .excluded(".github/workflows/check.yml", false)
                .unwrap(),
            None
        );
        for pattern in ["!*.key", "#comment", "", "a\nb", " ", "/"] {
            let mut changed = policy.clone();
            changed.exclusions[0].pattern = pattern.into();
            assert!(changed.validate().is_err(), "{pattern:?}");
        }
        let mut changed = policy.clone();
        changed.exclusions.push(changed.exclusions[0].clone());
        assert!(changed.validate().is_err());
        changed = policy.clone();
        changed.limits.entries = 0;
        assert!(changed.validate().is_err());
        changed = policy.clone();
        changed.limits.workers = changed.limits.open_files + 1;
        assert!(changed.validate().is_err());
    }
}
