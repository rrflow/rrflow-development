use super::*;

pub const CONFIGURATION_PLAN_TTL_MS: u64 = 300_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationChange {
    pub valid_from_unix_ms: u64,
    pub valid_to_unix_ms: Option<u64>,
    pub settings: ConfigurationSettings,
}

impl ConfigurationChange {
    pub fn validate(&self) -> Result<()> {
        if self
            .valid_to_unix_ms
            .is_some_and(|end| end <= self.valid_from_unix_ms)
        {
            return invalid("configuration valid interval must be nonempty and half-open");
        }
        validate_configuration_settings(&self.settings)
    }

    pub fn contains(&self, valid_at: u64) -> bool {
        valid_at >= self.valid_from_unix_ms
            && self.valid_to_unix_ms.is_none_or(|end| valid_at < end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationRevision {
    pub format_version: u16,
    pub source_id: CanonicalId,
    pub revision: u64,
    pub recorded_at_unix_ms: u64,
    pub previous_sha256: Option<String>,
    pub change: ConfigurationChange,
    pub applied_plan_sha256: String,
    pub revision_sha256: String,
}

impl ConfigurationRevision {
    pub fn sha256(&self) -> Result<String> {
        let mut value = self.clone();
        value.revision_sha256.clear();
        configuration_digest(&value)
    }

    pub fn validate(&self) -> Result<()> {
        self.change.validate()?;
        if self.format_version != SCOPED_CONFIGURATION_VERSION
            || self.revision == 0
            || self.recorded_at_unix_ms == 0
            || (self.revision == 1) != self.previous_sha256.is_none()
            || !is_digest(&self.applied_plan_sha256)
            || self
                .previous_sha256
                .as_ref()
                .is_some_and(|digest| !is_digest(digest))
            || self.revision_sha256 != self.sha256()?
        {
            return invalid("invalid configuration revision");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UserConfigurationPlan {
    pub format_version: u16,
    pub binding: UserConfigurationBinding,
    pub expected_revision: u64,
    pub expected_sha256: Option<String>,
    pub change: ConfigurationChange,
    pub issued_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub plan_sha256: String,
}

impl UserConfigurationPlan {
    pub fn sha256(&self) -> Result<String> {
        let mut value = self.clone();
        value.plan_sha256.clear();
        configuration_digest(&value)
    }

    pub fn validate(&self) -> Result<()> {
        self.binding.validate()?;
        self.change.validate()?;
        if self.format_version != SCOPED_CONFIGURATION_VERSION
            || self.expected_revision == u64::MAX
            || (self.expected_revision == 0) != self.expected_sha256.is_none()
            || self
                .expected_sha256
                .as_ref()
                .is_some_and(|digest| !is_digest(digest))
            || self.issued_at_unix_ms == 0
            || self
                .issued_at_unix_ms
                .checked_add(CONFIGURATION_PLAN_TTL_MS)
                != Some(self.expires_at_unix_ms)
            || self.plan_sha256 != self.sha256()?
        {
            return invalid("invalid user configuration plan");
        }
        Ok(())
    }
}

pub fn configuration_digest(value: &impl Serialize) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|error| ContractError(error.to_string()))?;
    if bytes.len() > 64 * 1024 {
        return invalid("configuration exceeds 64 KiB");
    }
    Ok(sha256_bytes(&bytes))
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
