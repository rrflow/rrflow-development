//! Engine-owned user and project policy. User publication requires local owner
//! access; installed project edits require authenticated administration.
use super::*;
use rrd_contract::{
    ConfigurationChange, ConfigurationRevision, ConfigurationSetting, ConfigurationSettings,
    ConfigurationValueSource, EffectiveConfiguration, EffectiveConfigurationValue,
    ReadEffectiveConfiguration, UserConfigurationBinding, UserConfigurationPlan,
    CONFIGURATION_PLAN_TTL_MS, SCOPED_CONFIGURATION_VERSION,
};

mod history;
mod project;
mod project_store;
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
