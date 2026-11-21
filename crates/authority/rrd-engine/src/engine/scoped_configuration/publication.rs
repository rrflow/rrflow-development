use super::*;
use store::UserStore;

impl RrdEngine {
    /// Pure local-owner preview. An absent root is not created by preview.
    pub fn preview_user_configuration(
        binding: &UserConfigurationBinding,
        expected_revision: u64,
        change: ConfigurationChange,
    ) -> Result<UserConfigurationPlan> {
        checked(binding.validate())?;
        checked(change.validate())?;
        let now = current_time()?;
        let root = rrd_store::filesystem::AnchoredPath::from_path(Path::new(&binding.root))
            .map_err(|error| unavailable(error.to_string()))?;
        let head = if root
            .try_exists()
            .map_err(|error| unavailable(error.to_string()))?
        {
            UserStore::open(binding, false)?.head
        } else {
            None
        };
        if head.as_ref().map_or(0, |head| head.revision) != expected_revision {
            return Err(conflict(
                "user configuration changed; preview its current revision",
            ));
        }
        if head
            .as_ref()
            .is_some_and(|head| head.recorded_at_unix_ms > now)
        {
            return Err(unavailable(
                "host clock precedes recorded user configuration",
            ));
        }
        let mut plan = UserConfigurationPlan {
            format_version: SCOPED_CONFIGURATION_VERSION,
            binding: binding.clone(),
            expected_revision,
            expected_sha256: head.map(|head| head.revision_sha256),
            change,
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

    /// OS-owner operation; project bearer authority cannot call this via HTTP.
    pub fn apply_user_configuration(
        plan: &UserConfigurationPlan,
        approved_sha256: &str,
    ) -> Result<ConfigurationRevision> {
        Self::apply_user_configuration_at(plan, approved_sha256, current_time()?)
    }

    pub(super) fn apply_user_configuration_at(
        plan: &UserConfigurationPlan,
        approved_sha256: &str,
        now: u64,
    ) -> Result<ConfigurationRevision> {
        checked(plan.validate())?;
        if approved_sha256 != plan.plan_sha256 {
            return Err(ServiceError::PermissionDenied);
        }
        if now < plan.issued_at_unix_ms {
            return Err(ServiceError::DeadlineExceeded);
        }
        let expired = now >= plan.expires_at_unix_ms;
        // An expired plan can read back its acknowledged publication, never create state.
        let source = if expired {
            UserStore::open(&plan.binding, false).map_err(|_| ServiceError::DeadlineExceeded)?
        } else {
            UserStore::open(&plan.binding, plan.expected_revision == 0)?
        };
        if let Some(head) = &source.head {
            if head.recorded_at_unix_ms > now {
                return Err(unavailable(
                    "host clock precedes recorded user configuration",
                ));
            }
            if head.applied_plan_sha256 == plan.plan_sha256 {
                return Ok(head.clone());
            }
        }
        if expired {
            return Err(ServiceError::DeadlineExceeded);
        }
        if source.head.as_ref().map_or(0, |head| head.revision) != plan.expected_revision
            || source.head.as_ref().map(|head| &head.revision_sha256)
                != plan.expected_sha256.as_ref()
        {
            return Err(conflict(
                "user configuration changed; preview its current revision",
            ));
        }
        let mut revision = ConfigurationRevision {
            format_version: SCOPED_CONFIGURATION_VERSION,
            source_id: plan.binding.source_id.clone(),
            revision: plan.expected_revision + 1,
            recorded_at_unix_ms: now,
            previous_sha256: plan.expected_sha256.clone(),
            change: plan.change.clone(),
            applied_plan_sha256: plan.plan_sha256.clone(),
            revision_sha256: String::new(),
        };
        revision.revision_sha256 = checked(revision.sha256())?;
        source.publish(&revision)?;
        Ok(revision)
    }

    pub fn read_user_configuration(
        binding: &UserConfigurationBinding,
        valid_at_unix_ms: Option<u64>,
        recorded_revision: Option<u64>,
    ) -> Result<ConfigurationRevision> {
        let now = current_time()?;
        let source = UserStore::open(binding, false)?;
        if source
            .head
            .as_ref()
            .is_some_and(|head| head.recorded_at_unix_ms > now)
        {
            return Err(unavailable(
                "host clock precedes recorded user configuration",
            ));
        }
        source
            .select(binding, valid_at_unix_ms.unwrap_or(now), recorded_revision)
            .map(|(_, value)| value)
    }
}
