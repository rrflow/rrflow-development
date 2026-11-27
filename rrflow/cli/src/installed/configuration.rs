//! The CLI translates arguments; publication and history remain engine-owned.
use super::*;
use crate::command::ConfigurationAction;
use rrd_contract::{ConfigurationChange, UserConfigurationBinding, UserConfigurationPlan};

pub(super) fn execute(action: &ConfigurationAction) -> Result<Execution, BoxError> {
    let value = match action {
        ConfigurationAction::Plan {
            root,
            source,
            expected_revision,
            input,
            max_history_reads,
        } => {
            let binding = binding(root, source, *max_history_reads)?;
            let change: ConfigurationChange = read(input)?;
            serde_json::to_value(RrdEngine::preview_user_configuration(
                &binding,
                *expected_revision,
                change,
            )?)?
        }
        ConfigurationAction::Apply { plan, expect } => {
            let plan: UserConfigurationPlan = read(plan)?;
            serde_json::to_value(RrdEngine::apply_user_configuration(&plan, expect)?)?
        }
        ConfigurationAction::Read {
            root,
            source,
            valid_at_unix_ms,
            recorded_revision,
            max_history_reads,
        } => {
            let binding = binding(root, source, *max_history_reads)?;
            serde_json::to_value(RrdEngine::read_user_configuration(
                &binding,
                *valid_at_unix_ms,
                *recorded_revision,
            )?)?
        }
    };
    Ok(serde_json::to_string_pretty(&value)?.into())
}

fn binding(
    root: &Path,
    source: &str,
    max_history_reads: u64,
) -> Result<UserConfigurationBinding, BoxError> {
    let root = std::path::absolute(root)?;
    let binding = UserConfigurationBinding {
        source_id: CanonicalId::new(source)?,
        root: root
            .to_str()
            .ok_or("configuration root must be UTF-8")?
            .into(),
        max_history_reads,
    };
    binding.validate()?;
    Ok(binding)
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, BoxError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
        return Err("configuration input must be a regular file of at most 64 KiB".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(64 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 {
        return Err("configuration input exceeds 64 KiB".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}
