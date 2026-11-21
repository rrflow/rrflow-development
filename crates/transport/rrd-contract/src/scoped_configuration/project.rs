//! Governed project edits share the same numeric and temporal semantics as user policy.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationRevisionAdmission {
    pub read_revision: u64,
    pub selected_revision: u64,
    pub revision_sha256: String,
}
impl ConfigurationRevisionAdmission {
    pub fn validate(&self) -> Result<()> {
        if self.selected_revision == 0 || self.read_revision < self.selected_revision {
            return invalid("invalid project policy admission revision");
        }
        crate::validate_sha256(&self.revision_sha256, "project policy")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadProjectConfiguration {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfiguration {
    pub identity: InstalledEngramIdentity,
    pub installed_configuration_sha256: String,
    pub installed_settings: ConfigurationSettings,
    /// Latest recorded edit, including a future-dated edit. Effective reads select by valid time.
    pub head: Option<ConfigurationRevision>,
}
impl ProjectConfiguration {
    pub fn validate(&self) -> Result<()> {
        self.identity.validate()?;
        crate::validate_sha256(
            &self.installed_configuration_sha256,
            "installed configuration",
        )?;
        validate_configuration_settings(&self.installed_settings)?;
        if let Some(head) = &self.head {
            head.validate()?;
            if head.source_id != self.identity.instance_id {
                return invalid("project policy belongs to another instance");
            }
        }
        Ok(())
    }
    pub fn revision(&self) -> u64 {
        self.head.as_ref().map_or(0, |head| head.revision)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreviewProjectConfiguration {
    pub expected_revision: u64,
    pub change: ConfigurationChange,
}
impl PreviewProjectConfiguration {
    pub fn validate(&self) -> Result<()> {
        self.change.validate()?;
        if self.expected_revision == u64::MAX {
            return invalid("project policy revision exhausted");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfigurationPlan {
    pub format_version: u16,
    pub current: ProjectConfiguration,
    pub change: ConfigurationChange,
    /// Composition is checked at this valid-time coordinate, not all future policy states.
    pub evaluated_at_unix_ms: u64,
    pub user: Option<UserConfigurationAdmission>,
    pub values: BTreeMap<ConfigurationSetting, EffectiveConfigurationValue>,
    pub issued_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub plan_sha256: String,
}
impl ProjectConfigurationPlan {
    pub fn sha256(&self) -> Result<String> {
        let mut value = self.clone();
        value.plan_sha256.clear();
        configuration_digest(&value)
    }
    pub fn validate(&self) -> Result<()> {
        self.current.validate()?;
        validate_effective_values(&self.values)?;
        self.change.validate()?;
        if self.format_version != SCOPED_CONFIGURATION_VERSION
            || self.current.revision() == u64::MAX
            || self.issued_at_unix_ms == 0
            || self
                .issued_at_unix_ms
                .checked_add(CONFIGURATION_PLAN_TTL_MS)
                != Some(self.expires_at_unix_ms)
            || !self.change.contains(self.evaluated_at_unix_ms)
            || self.values.len() != ConfigurationSetting::ALL.len()
            || self.plan_sha256 != self.sha256()?
        {
            return invalid("invalid project configuration plan");
        }
        if let Some(user) = &self.user {
            ConfigurationRevisionAdmission {
                read_revision: user.read_revision,
                selected_revision: user.selected_revision,
                revision_sha256: user.revision_sha256.clone(),
            }
            .validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApplyProjectConfiguration {
    pub plan: ProjectConfigurationPlan,
    pub approved_plan_sha256: String,
}
impl ApplyProjectConfiguration {
    pub fn validate(&self) -> Result<()> {
        self.plan.validate()?;
        if self.approved_plan_sha256 != self.plan.plan_sha256 {
            return invalid("exact project configuration plan approval is required");
        }
        Ok(())
    }
}
