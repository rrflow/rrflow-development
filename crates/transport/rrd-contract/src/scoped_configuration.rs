//! Exact, scoped query/recall configuration. These are capacity settings, not
//! promises about answer quality or a schema for private reasoning.
use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod project;
mod publication;
pub use project::*;
pub use publication::*;

pub const SCOPED_CONFIGURATION_VERSION: u16 = 1;
pub const MAX_CONFIGURATION_HISTORY_READS: u64 = 4096;

/// The exact source admission used by an operation; independent of its data read stamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationAdmission {
    pub project_revision: u64,
    pub project_configuration_sha256: String,
    pub evaluated_at_unix_ms: u64,
    pub user: Option<UserConfigurationAdmission>,
    pub project_policy: Option<ConfigurationRevisionAdmission>,
    pub project_read_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UserConfigurationAdmission {
    pub source_id: CanonicalId,
    pub read_revision: u64,
    pub selected_revision: u64,
    pub revision_sha256: String,
}

impl ConfigurationAdmission {
    pub fn validate(&self) -> Result<()> {
        if self.project_revision == 0 || self.evaluated_at_unix_ms == 0 {
            return invalid("invalid configuration admission coordinate");
        }
        crate::validate_sha256(&self.project_configuration_sha256, "project configuration")?;
        if let Some(user) = &self.user {
            if user.selected_revision == 0 || user.read_revision < user.selected_revision {
                return invalid("invalid configuration admission user revision");
            }
            crate::validate_sha256(&user.revision_sha256, "user configuration")?;
        }
        if let Some(policy) = &self.project_policy {
            policy.validate()?;
            if policy.read_revision != self.project_read_revision {
                return invalid("project policy read revision mismatch");
            }
        }
        Ok(())
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationSetting {
    QueryStorageKeys,
    QueryRows,
    QueryOutputBytes,
    QueryBatchRows,
    QueryMemoryBytes,
    QuerySpillBytes,
    QueryElapsedMs,
    RecallGraphDepth,
    RecallItems,
    RecallOutputBytes,
    RecallStorageKeys,
}

impl ConfigurationSetting {
    pub const ALL: [Self; 11] = [
        Self::QueryStorageKeys,
        Self::QueryRows,
        Self::QueryOutputBytes,
        Self::QueryBatchRows,
        Self::QueryMemoryBytes,
        Self::QuerySpillBytes,
        Self::QueryElapsedMs,
        Self::RecallGraphDepth,
        Self::RecallItems,
        Self::RecallOutputBytes,
        Self::RecallStorageKeys,
    ];

    pub fn unit(self) -> &'static str {
        match self {
            Self::QueryOutputBytes
            | Self::QueryMemoryBytes
            | Self::QuerySpillBytes
            | Self::RecallOutputBytes => "bytes",
            Self::QueryElapsedMs => "milliseconds",
            Self::RecallGraphDepth => "hops",
            Self::QueryStorageKeys | Self::RecallStorageKeys => "keys",
            Self::QueryRows | Self::QueryBatchRows => "rows",
            Self::RecallItems => "items",
        }
    }

    pub fn bounds(self) -> (u64, u64) {
        (
            u64::from(self != Self::RecallGraphDepth),
            match self {
                Self::QueryStorageKeys => MAX_QUERY_STORAGE_KEYS,
                Self::QueryRows => MAX_QUERY_ROWS,
                Self::QueryOutputBytes => MAX_QUERY_OUTPUT_BYTES,
                Self::QueryBatchRows => MAX_QUERY_BATCH_ROWS,
                Self::QueryMemoryBytes => MAX_QUERY_MEMORY_BYTES,
                Self::QuerySpillBytes => MAX_QUERY_SPILL_BYTES,
                Self::QueryElapsedMs => MAX_QUERY_ELAPSED_MS,
                Self::RecallGraphDepth => u64::from(MAX_CONTEXT_GRAPH_DEPTH),
                Self::RecallItems => MAX_CONTEXT_ITEMS,
                Self::RecallOutputBytes => MAX_CONTEXT_OUTPUT_BYTES,
                Self::RecallStorageKeys => MAX_CONTEXT_STORAGE_KEYS,
            },
        )
    }

    pub fn installed_value(self, configuration: &EngramConfiguration) -> u64 {
        match self {
            Self::QueryStorageKeys => configuration.query.max_storage_keys,
            Self::QueryRows => configuration.query.max_rows,
            Self::QueryOutputBytes => configuration.query.max_output_bytes,
            Self::QueryBatchRows => configuration.query.max_batch_rows,
            Self::QueryMemoryBytes => configuration.query.max_memory_bytes,
            Self::QuerySpillBytes => configuration.query.max_spill_bytes,
            Self::QueryElapsedMs => configuration.query.max_elapsed_ms,
            Self::RecallGraphDepth => u64::from(configuration.recall.max_graph_depth),
            Self::RecallItems => configuration.recall.max_items,
            Self::RecallOutputBytes => configuration.recall.max_output_bytes,
            Self::RecallStorageKeys => configuration.recall.max_storage_keys,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NumericConfiguration {
    pub minimum: Option<u64>,
    pub maximum: Option<u64>,
    pub preference: Option<u64>,
}

pub type ConfigurationSettings = BTreeMap<ConfigurationSetting, NumericConfiguration>;

pub fn validate_configuration_settings(settings: &ConfigurationSettings) -> Result<()> {
    for (setting, rule) in settings {
        let (floor, ceiling) = setting.bounds();
        let minimum = rule.minimum.unwrap_or(floor);
        let maximum = rule.maximum.unwrap_or(ceiling);
        if minimum < floor
            || maximum > ceiling
            || minimum > maximum
            || rule
                .preference
                .is_some_and(|value| value < minimum || value > maximum)
        {
            return invalid(format!("invalid scoped configuration for {setting:?}"));
        }
    }
    Ok(())
}

/// Explicit local-owner read binding, sealed in the project's installation.
/// It grants no global write or project-to-project knowledge access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UserConfigurationBinding {
    pub source_id: CanonicalId,
    pub root: String,
    pub max_history_reads: u64,
}

impl UserConfigurationBinding {
    pub fn validate(&self) -> Result<()> {
        if self.root.is_empty()
            || self.root.len() > 4096
            || self.root.bytes().any(|byte| byte < 32)
            || !std::path::Path::new(&self.root).is_absolute()
            || self.max_history_reads == 0
            || self.max_history_reads > MAX_CONFIGURATION_HISTORY_READS
        {
            return invalid(
                "user configuration requires an absolute local root and bounded history reads",
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopedConfiguration {
    pub user: Option<UserConfigurationBinding>,
    #[serde(default)]
    pub settings: ConfigurationSettings,
}

impl ScopedConfiguration {
    pub fn validate(&self) -> Result<()> {
        if let Some(user) = &self.user {
            user.validate()?;
        }
        validate_configuration_settings(&self.settings)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadEffectiveConfiguration {
    /// Diagnostic historical reads only; runtime admission always uses current time.
    pub valid_at_unix_ms: Option<u64>,
    pub recorded_revision: Option<u64>,
    pub project_recorded_revision: Option<u64>,
    #[serde(default)]
    pub requested: BTreeMap<ConfigurationSetting, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationValueSource {
    Request,
    ProjectPreference,
    UserPreference,
    InstalledDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveConfigurationValue {
    pub value: u64,
    pub unit: String,
    pub source: ConfigurationValueSource,
    pub minimum: u64,
    pub maximum: u64,
    pub installed_ceiling: u64,
    pub user_rule: Option<NumericConfiguration>,
    pub project_rule: Option<NumericConfiguration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveConfiguration {
    pub format_version: u16,
    pub identity: InstalledEngramIdentity,
    pub project_revision: u64,
    pub project_configuration_sha256: String,
    pub valid_at_unix_ms: u64,
    pub project_read_revision: u64,
    pub project_policy: Option<ConfigurationRevision>,
    pub user_read_revision: Option<u64>,
    pub user_revision: Option<ConfigurationRevision>,
    pub values: BTreeMap<ConfigurationSetting, EffectiveConfigurationValue>,
}

impl EffectiveConfiguration {
    pub fn validate(&self) -> Result<()> {
        self.identity.validate()?;
        crate::validate_sha256(&self.project_configuration_sha256, "project configuration")?;
        if self.format_version != SCOPED_CONFIGURATION_VERSION
            || self.project_revision == 0
            || self.values.len() != ConfigurationSetting::ALL.len()
            || self.user_read_revision.is_some() != self.user_revision.is_some()
        {
            return invalid("invalid effective configuration identity");
        }
        if let Some(user) = &self.user_revision {
            user.validate()?;
            if self
                .user_read_revision
                .is_none_or(|known| known < user.revision)
                || !user.change.contains(self.valid_at_unix_ms)
            {
                return invalid("invalid effective configuration temporal identity");
            }
        }
        if let Some(policy) = &self.project_policy {
            policy.validate()?;
            if policy.source_id != self.identity.instance_id
                || policy.revision > self.project_read_revision
                || !policy.change.contains(self.valid_at_unix_ms)
            {
                return invalid("invalid effective project policy identity");
            }
        }
        validate_effective_values(&self.values)?;
        Ok(())
    }
}

fn validate_effective_values(
    values: &BTreeMap<ConfigurationSetting, EffectiveConfigurationValue>,
) -> Result<()> {
    if values.len() != ConfigurationSetting::ALL.len() {
        return invalid("incomplete configuration values");
    }
    for (setting, value) in values {
        let (floor, ceiling) = setting.bounds();
        if value.minimum < floor
            || value.maximum > ceiling
            || value.minimum > value.maximum
            || value.maximum > value.installed_ceiling
            || value.installed_ceiling > ceiling
            || value.value < value.minimum
            || value.value > value.maximum
            || value.unit != setting.unit()
        {
            return invalid("invalid effective configuration value");
        }
    }
    Ok(())
}
