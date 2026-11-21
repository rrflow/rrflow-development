//! User configuration is engine-owned control data, independent of a project
//! engram. Local owner access is required; project sessions receive read only.
use super::*;
use rrd_contract::{
    ConfigurationSetting, ConfigurationSettings, ConfigurationValueSource, EffectiveConfiguration,
    EffectiveConfigurationValue, ReadEffectiveConfiguration, UserConfigurationBinding,
    UserConfigurationChange, UserConfigurationPlan, UserConfigurationRevision,
    SCOPED_CONFIGURATION_VERSION, USER_CONFIGURATION_PLAN_TTL_MS,
};

mod publication;
mod resolution;
mod store;
#[cfg(test)]
mod tests;

fn checked<T>(result: rrd_contract::Result<T>) -> Result<T> {
    result.map_err(|error| ServiceError::Contract(error.to_string()))
}

fn current_time() -> Result<u64> {
    Ok(
        super::clock::observe_required(&super::clock::HostSystemClock, "configuration.resolve")?
            .observed_at_unix_ms,
    )
}

fn conflict(message: impl Into<String>) -> ServiceError {
    ServiceError::StorageConflict(message.into())
}

fn unavailable(message: impl Into<String>) -> ServiceError {
    ServiceError::ConfigurationUnavailable(message.into())
}
