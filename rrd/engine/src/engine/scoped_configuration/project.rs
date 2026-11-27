//! Authenticated, digest-approved project policy publication; no external effects.
use super::*;
use rrd_contract::{
    ApplyProjectConfiguration, PreviewProjectConfiguration, ProjectConfiguration,
    ProjectConfigurationPlan, UserConfigurationAdmission,
};

impl RrdEngine {
    pub fn read_project_configuration(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        context: &RequestContext,
        now: u64,
    ) -> Result<ProjectConfiguration> {
        self.authorize(
            session,
            token,
            SecurityAction::EngramRead,
            now,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        Ok(self.project_policy_head()?.1)
    }

    pub fn preview_project_configuration(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &PreviewProjectConfiguration,
        context: &RequestContext,
        now: u64,
    ) -> Result<ProjectConfigurationPlan> {
        self.authorize(
            session,
            token,
            SecurityAction::EngramAdmin,
            now,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        checked(request.validate())?;
        let now = current_time()?;
        let (_, current) = self.project_policy_head()?;
        if current.revision() != request.expected_revision {
            return Err(conflict(
                "project policy changed; preview the current revision",
            ));
        }
        // Current/future edits preview their active point; a purely historical
        // correction previews its first valid instant. Admission still checks now.
        let evaluated_at = if request.change.contains(now) {
            now
        } else {
            request.change.valid_from_unix_ms
        };
        let (user, values) = self.preview_project_values(&request.change, evaluated_at, now)?;
        let mut plan = ProjectConfigurationPlan {
            format_version: SCOPED_CONFIGURATION_VERSION,
            current,
            change: request.change.clone(),
            evaluated_at_unix_ms: evaluated_at,
            user,
            values,
            issued_at_unix_ms: now,
            expires_at_unix_ms: now
                .checked_add(CONFIGURATION_PLAN_TTL_MS)
                .ok_or_else(|| unavailable("configuration preview clock overflow"))?,
            plan_sha256: String::new(),
        };
        plan.plan_sha256 = checked(plan.sha256())?;
        checked(plan.validate())?;
        Ok(plan)
    }

    pub fn apply_project_configuration(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &ApplyProjectConfiguration,
        context: &RequestContext,
        now: u64,
    ) -> Result<ProjectConfiguration> {
        self.authorize(
            session,
            token,
            SecurityAction::EngramAdmin,
            now,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        checked(request.validate())?;
        let _permit = self.admit_transaction(
            request,
            context.request_id.as_str(),
            context.operation_id.as_str(),
        )?;
        let now = current_time()?;
        let (expected, current) = self.project_policy_head()?;
        let plan = &request.plan;
        if current.identity != plan.current.identity
            || current.installed_configuration_sha256 != plan.current.installed_configuration_sha256
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        if now < plan.issued_at_unix_ms {
            return Err(ServiceError::DeadlineExceeded);
        }
        if current
            .head
            .as_ref()
            .is_some_and(|head| head.applied_plan_sha256 == plan.plan_sha256)
        {
            return Ok(current);
        }
        if now >= plan.expires_at_unix_ms {
            return Err(ServiceError::DeadlineExceeded);
        }
        if current != plan.current {
            return Err(conflict("project policy changed; preview again"));
        }
        let (user, values) =
            self.preview_project_values(&plan.change, plan.evaluated_at_unix_ms, now)?;
        if user != plan.user || values != plan.values {
            return Err(conflict(
                "user policy or composition changed; preview again",
            ));
        }
        // A future activation boundary crossed since preview must be checked too.
        if plan.change.contains(now) {
            self.preview_project_values(&plan.change, now, now)?;
        }
        let mut revision = ConfigurationRevision {
            format_version: SCOPED_CONFIGURATION_VERSION,
            source_id: current.identity.instance_id.clone(),
            revision: current.revision() + 1,
            recorded_at_unix_ms: now,
            previous_sha256: current
                .head
                .as_ref()
                .map(|head| head.revision_sha256.clone()),
            change: plan.change.clone(),
            applied_plan_sha256: plan.plan_sha256.clone(),
            revision_sha256: String::new(),
        };
        revision.revision_sha256 = checked(revision.sha256())?;
        self.publish_project_policy(expected, &revision, session, context)?;
        Ok(ProjectConfiguration {
            head: Some(revision),
            ..current
        })
    }

    fn preview_project_values(
        &self,
        change: &ConfigurationChange,
        valid_at: u64,
        now: u64,
    ) -> Result<(
        Option<UserConfigurationAdmission>,
        BTreeMap<ConfigurationSetting, EffectiveConfigurationValue>,
    )> {
        let (known, user) = resolution::load_user(&self.engram_configuration, valid_at, None, now)?;
        let values = resolution::resolve(
            &self.engram_configuration,
            user.as_ref(),
            &change.settings,
            &BTreeMap::new(),
            ConfigurationSetting::ALL,
        )?;
        let user = user.map(|user| UserConfigurationAdmission {
            source_id: user.source_id,
            read_revision: known.expect("bound user source"),
            selected_revision: user.revision,
            revision_sha256: user.revision_sha256,
        });
        Ok((user, values))
    }
}
