use super::*;

impl RrdEngine {
    /// The installed read boundary exposes exact policies, independent source
    /// revisions and selected defaults. It never executes a historical effect.
    pub fn read_effective_configuration(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &ReadEffectiveConfiguration,
        context: &RequestContext,
        now: u64,
    ) -> Result<EffectiveConfiguration> {
        self.authorize(
            session,
            token,
            SecurityAction::EngramRead,
            now,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let identity = self
            .installed_engram_identity()
            .cloned()
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        let now = current_time()?;
        let valid_at = request.valid_at_unix_ms.unwrap_or(now);
        let (known, user) = load_user(
            &self.engram_configuration,
            valid_at,
            request.recorded_revision,
            now,
        )?;
        let values = resolve(
            &self.engram_configuration,
            user.as_ref(),
            &request.requested,
            ConfigurationSetting::ALL,
        )?;
        Ok(EffectiveConfiguration {
            format_version: SCOPED_CONFIGURATION_VERSION,
            identity,
            project_revision: self.engram_configuration.revision,
            project_configuration_sha256: self.engram_configuration.configuration_sha256.clone(),
            valid_at_unix_ms: valid_at,
            user_read_revision: known,
            user_revision: user,
            values,
        })
    }

    pub(in crate::engine) fn enforce_scoped_query(
        &self,
        requested: &rrd_contract::QueryBudget,
    ) -> Result<Option<rrd_contract::ConfigurationAdmission>> {
        use ConfigurationSetting::*;
        self.enforce_scoped_settings(BTreeMap::from([
            (QueryStorageKeys, requested.max_storage_keys),
            (QueryRows, requested.max_rows),
            (QueryOutputBytes, requested.max_output_bytes),
            (QueryBatchRows, requested.max_batch_rows),
            (QueryMemoryBytes, requested.max_memory_bytes),
            (QuerySpillBytes, requested.max_spill_bytes),
            (QueryElapsedMs, requested.max_elapsed_ms),
        ]))
    }

    pub(in crate::engine) fn enforce_scoped_recall(
        &self,
        requested: &rrd_contract::AssembleContext,
    ) -> Result<Option<rrd_contract::ConfigurationAdmission>> {
        use ConfigurationSetting::*;
        self.enforce_scoped_settings(BTreeMap::from([
            (RecallGraphDepth, u64::from(requested.max_graph_depth)),
            (RecallItems, requested.max_items),
            (RecallOutputBytes, requested.max_output_bytes),
            (RecallStorageKeys, requested.max_storage_keys),
        ]))
    }

    fn enforce_scoped_settings(
        &self,
        requested: BTreeMap<ConfigurationSetting, u64>,
    ) -> Result<Option<rrd_contract::ConfigurationAdmission>> {
        if self.engram_configuration.scoped.user.is_none()
            && self.engram_configuration.scoped.settings.is_empty()
        {
            return Ok(None);
        }
        let now = current_time()?;
        let (known, user) = load_user(&self.engram_configuration, now, None, now)?;
        resolve(
            &self.engram_configuration,
            user.as_ref(),
            &requested,
            requested.keys().copied(),
        )?;
        Ok(Some(rrd_contract::ConfigurationAdmission {
            project_revision: self.engram_configuration.revision,
            project_configuration_sha256: self.engram_configuration.configuration_sha256.clone(),
            evaluated_at_unix_ms: now,
            user: user.map(|revision| rrd_contract::UserConfigurationAdmission {
                source_id: revision.source_id,
                read_revision: known.expect("bound user read"),
                selected_revision: revision.revision,
                revision_sha256: revision.revision_sha256,
            }),
        }))
    }

    pub(in crate::engine) fn validate_user_configuration_binding(
        configuration: &EngramConfiguration,
    ) -> Result<()> {
        let now = current_time()?;
        // Binding admission reads the exact source identity, not a discovered candidate.
        load_user(configuration, now, None, now)?;
        Ok(())
    }
}

fn load_user(
    configuration: &EngramConfiguration,
    valid_at: u64,
    known: Option<u64>,
    now: u64,
) -> Result<(Option<u64>, Option<UserConfigurationRevision>)> {
    let Some(binding) = &configuration.scoped.user else {
        if known.is_some() {
            return Err(unavailable("no user configuration source is bound"));
        }
        return Ok((None, None));
    };
    let source = store::UserStore::open(binding, false)?;
    if source
        .head
        .as_ref()
        .is_some_and(|head| head.recorded_at_unix_ms > now)
    {
        return Err(unavailable(
            "host clock precedes recorded user configuration",
        ));
    }
    let (known, revision) = source.select(binding, valid_at, known)?;
    Ok((Some(known), Some(revision)))
}

pub(super) fn resolve(
    configuration: &EngramConfiguration,
    user: Option<&UserConfigurationRevision>,
    requested: &BTreeMap<ConfigurationSetting, u64>,
    settings: impl IntoIterator<Item = ConfigurationSetting>,
) -> Result<BTreeMap<ConfigurationSetting, EffectiveConfigurationValue>> {
    let empty = ConfigurationSettings::new();
    let user_settings = user.map_or(&empty, |user| &user.change.settings);
    let mut values = BTreeMap::new();
    for setting in settings {
        let user_rule = user_settings.get(&setting);
        let project_rule = configuration.scoped.settings.get(&setting);
        let installed_ceiling = setting.installed_value(configuration);
        let (mut minimum, _) = setting.bounds();
        let mut maximum = installed_ceiling;
        for rule in [user_rule, project_rule].into_iter().flatten() {
            minimum = minimum.max(rule.minimum.unwrap_or(minimum));
            maximum = maximum.min(rule.maximum.unwrap_or(maximum));
        }
        let (value, source) = if let Some(value) = requested.get(&setting) {
            (*value, ConfigurationValueSource::Request)
        } else if let Some(value) = project_rule.and_then(|rule| rule.preference) {
            (value, ConfigurationValueSource::ProjectPreference)
        } else if let Some(value) = user_rule.and_then(|rule| rule.preference) {
            (value, ConfigurationValueSource::UserPreference)
        } else {
            (
                installed_ceiling,
                ConfigurationValueSource::InstalledDefault,
            )
        };
        if minimum > maximum || value < minimum || value > maximum {
            let reason = format!(
                "{setting:?} configuration conflict: {source:?} value {value} outside [{minimum}, {maximum}] {}",
                setting.unit());
            return Err(if minimum > maximum || !requested.contains_key(&setting) {
                ServiceError::ConfigurationConflict(reason)
            } else {
                ServiceError::ConfigurationLimit(reason)
            });
        }
        values.insert(
            setting,
            EffectiveConfigurationValue {
                value,
                unit: setting.unit().into(),
                source,
                minimum,
                maximum,
                installed_ceiling,
                user_rule: user_rule.cloned(),
                project_rule: project_rule.cloned(),
            },
        );
    }
    Ok(values)
}
