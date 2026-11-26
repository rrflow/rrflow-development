use super::*;
use rrd_contract::ProjectConfiguration;
use rrd_store::Repositories;

impl RrdEngine {
    fn project_policy_key(&self, suffix: &str) -> String {
        format!(
            "server/state/{}/project-configuration/{suffix}",
            self.instance
        )
    }

    pub(super) fn project_policy_head(&self) -> Result<(Option<Vec<u8>>, ProjectConfiguration)> {
        let identity = self
            .installed_engram_identity()
            .cloned()
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        let bytes = self
            .storage
            .control()
            .get(&self.project_policy_key("head"))?;
        let head = bytes.as_deref().map(history::decode).transpose()?;
        let state = ProjectConfiguration {
            identity,
            installed_configuration_sha256: self.engram_configuration.configuration_sha256.clone(),
            installed_settings: self.engram_configuration.scoped.settings.clone(),
            head,
        };
        checked(state.validate())?;
        let now = current_time()?;
        if state
            .head
            .as_ref()
            .is_some_and(|head| head.recorded_at_unix_ms > now)
        {
            return Err(unavailable(
                "host clock precedes recorded project configuration",
            ));
        }
        Ok((bytes, state))
    }

    pub(super) fn select_project_policy(
        &self,
        valid_at: u64,
        known: Option<u64>,
    ) -> Result<(u64, Option<ConfigurationRevision>)> {
        // Component engines have no installed project scope and cannot publish policy.
        if self.installed_engram_identity().is_none() {
            if known.is_some() {
                return Err(ServiceError::ProjectBindingMismatch);
            }
            return Ok((0, None));
        }
        let (_, state) = self.project_policy_head()?;
        history::select(
            state.head,
            valid_at,
            known,
            rrd_contract::MAX_CONFIGURATION_HISTORY_READS,
            |revision| {
                self.storage
                    .control()
                    .get(&self.project_policy_key(&format!("revision/{revision:020}")))
                    .map_err(Into::into)
            },
        )
    }

    pub(super) fn publish_project_policy(
        &self,
        expected: Option<Vec<u8>>,
        revision: &ConfigurationRevision,
        session: &CorrelationId,
        context: &RequestContext,
    ) -> Result<()> {
        checked(revision.validate())?;
        let bytes = serde_json::to_vec(revision).map_err(contract_json)?;
        let transition = |suffix: &str, expected| ControlTransition {
            key: self.project_policy_key(suffix),
            expected,
            replacement: Some(bytes.clone()),
            at: revision.recorded_at_unix_ms,
            actor: session.as_str().to_owned(),
            action: "project.configuration.applied".into(),
            request_id: context.request_id.as_str().to_owned(),
            operation_id: context.operation_id.as_str().to_owned(),
        };
        self.storage.control().commit_batch(&[
            transition("head", expected),
            transition(&format!("revision/{:020}", revision.revision), None),
        ])?;
        Ok(())
    }
}
