use super::*;
use rrd_contract::NumericConfiguration;

fn binding(root: &Path) -> UserConfigurationBinding {
    UserConfigurationBinding {
        source_id: CanonicalId::new("user-policies").unwrap(),
        root: root.to_str().unwrap().into(),
        max_history_reads: 32,
    }
}

fn change(from: u64, to: Option<u64>, maximum: u64) -> UserConfigurationChange {
    UserConfigurationChange {
        valid_from_unix_ms: from,
        valid_to_unix_ms: to,
        settings: BTreeMap::from([(
            ConfigurationSetting::QueryRows,
            NumericConfiguration {
                minimum: Some(2),
                maximum: Some(maximum),
                preference: Some(maximum),
            },
        )]),
    }
}

fn publish(
    binding: &UserConfigurationBinding,
    expected: u64,
    change: UserConfigurationChange,
) -> UserConfigurationRevision {
    let plan = RrdEngine::preview_user_configuration(binding, expected, change).unwrap();
    RrdEngine::apply_user_configuration(&plan, &plan.plan_sha256).unwrap()
}

#[test]
fn scoped_configuration_native_history_preserves_late_corrections_and_cas_after_reopen() {
    let root = tempfile::tempdir().unwrap();
    let binding = binding(&root.path().join("user"));
    let initial = RrdEngine::preview_user_configuration(&binding, 0, change(0, None, 20)).unwrap();
    assert!(
        !Path::new(&binding.root).exists(),
        "preview created a store"
    );
    assert!(RrdEngine::apply_user_configuration(&initial, &"0".repeat(64)).is_err());
    assert!(
        !Path::new(&binding.root).exists(),
        "unapproved apply created a store"
    );
    let first = RrdEngine::apply_user_configuration(&initial, &initial.plan_sha256).unwrap();
    assert_eq!(
        first,
        RrdEngine::apply_user_configuration(&initial, &initial.plan_sha256).unwrap()
    );
    let stale = RrdEngine::preview_user_configuration(&binding, 1, change(0, None, 18)).unwrap();
    let second = publish(&binding, 1, change(10, Some(30), 8));
    assert_eq!(second.previous_sha256, Some(first.revision_sha256.clone()));
    assert!(matches!(
        RrdEngine::apply_user_configuration(&stale, &stale.plan_sha256),
        Err(ServiceError::StorageConflict(_))
    ));
    assert!(
        RrdEngine::apply_user_configuration(&initial, &initial.plan_sha256).is_err(),
        "stale replay resurrected history"
    );
    for (valid, known, expected) in [(20, 1, 20), (20, 2, 8), (9, 2, 20), (30, 2, 20)] {
        let result =
            RrdEngine::read_user_configuration(&binding, Some(valid), Some(known)).unwrap();
        assert_eq!(
            result.change.settings[&ConfigurationSetting::QueryRows].maximum,
            Some(expected)
        );
    }
    let third = publish(&binding, 2, change(20, Some(25), 20));
    assert_eq!(third.revision, 3, "returning to an old value is a new fact");
    assert_eq!(
        RrdEngine::read_user_configuration(&binding, Some(22), Some(3))
            .unwrap()
            .revision,
        3
    );
    let mut foreign = binding.clone();
    foreign.source_id = CanonicalId::new("foreign-user").unwrap();
    assert!(matches!(
        RrdEngine::read_user_configuration(&foreign, Some(22), None),
        Err(ServiceError::ProjectBindingMismatch)
    ));
    let mut bounded = binding.clone();
    bounded.max_history_reads = 1;
    assert!(RrdEngine::read_user_configuration(&bounded, Some(9), None).is_err());
    let held = store::UserStore::open(&binding, false).unwrap();
    assert!(
        RrdEngine::read_user_configuration(&binding, Some(22), None).is_err(),
        "contended source must not disappear into defaults"
    );
    drop(held);
    assert_eq!(
        RrdEngine::read_user_configuration(&binding, Some(22), None)
            .unwrap()
            .revision,
        3
    );
}

#[test]
fn scoped_configuration_exact_resolution_keeps_all_policies_and_reports_preference_source() {
    let root = tempfile::tempdir().unwrap();
    let binding = binding(&root.path().join("user"));
    let user = publish(&binding, 0, change(0, None, 20));
    let mut project = EngramConfiguration::default();
    project.scoped.settings.insert(
        ConfigurationSetting::QueryRows,
        NumericConfiguration {
            minimum: Some(5),
            maximum: Some(30),
            preference: Some(12),
        },
    );
    let requested = BTreeMap::new();
    let resolve = |project: &EngramConfiguration,
                   requested: &BTreeMap<ConfigurationSetting, u64>| {
        resolution::resolve(
            project,
            Some(&user),
            requested,
            [ConfigurationSetting::QueryRows],
        )
    };
    let result = resolve(&project, &requested).unwrap();
    let value = &result[&ConfigurationSetting::QueryRows];
    assert_eq!((value.minimum, value.maximum, value.value), (5, 20, 12));
    assert_eq!(value.source, ConfigurationValueSource::ProjectPreference);
    assert!(matches!(
        resolve(
            &project,
            &BTreeMap::from([(ConfigurationSetting::QueryRows, 21)])
        ),
        Err(ServiceError::ConfigurationLimit(_))
    ));
    project
        .scoped
        .settings
        .get_mut(&ConfigurationSetting::QueryRows)
        .unwrap()
        .minimum = Some(21);
    assert!(matches!(
        resolve(&project, &requested),
        Err(ServiceError::ConfigurationConflict(_))
    ));
    project.scoped.settings.clear();
    assert_eq!(
        resolve(&project, &requested).unwrap()[&ConfigurationSetting::QueryRows].source,
        ConfigurationValueSource::UserPreference
    );
}

#[test]
fn scoped_configuration_expiry_cannot_write_and_future_policy_is_not_current() {
    let root = tempfile::tempdir().unwrap();
    let binding = binding(&root.path().join("user"));
    let plan = RrdEngine::preview_user_configuration(&binding, 0, change(0, None, 20)).unwrap();
    assert!(matches!(
        RrdEngine::apply_user_configuration_at(&plan, &plan.plan_sha256, plan.expires_at_unix_ms),
        Err(ServiceError::DeadlineExceeded)
    ));
    assert!(!Path::new(&binding.root).exists());
    let first = RrdEngine::apply_user_configuration(&plan, &plan.plan_sha256).unwrap();
    assert_eq!(
        RrdEngine::apply_user_configuration_at(&plan, &plan.plan_sha256, plan.expires_at_unix_ms)
            .unwrap(),
        first
    );
    let future = current_time().unwrap() + 60_000;
    publish(&binding, 1, change(future, None, 6));
    assert_eq!(
        RrdEngine::read_user_configuration(&binding, None, None)
            .unwrap()
            .revision,
        1
    );
    assert_eq!(
        RrdEngine::read_user_configuration(&binding, Some(future), None)
            .unwrap()
            .revision,
        2
    );
    assert_eq!(
        RrdEngine::read_user_configuration(&binding, Some(future), Some(1))
            .unwrap()
            .revision,
        1
    );
}

#[cfg(unix)]
#[test]
fn scoped_configuration_rejects_publicly_accessible_source_directory() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let binding = binding(&root.path().join("user"));
    publish(&binding, 0, change(0, None, 20));
    std::fs::set_permissions(&binding.root, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(RrdEngine::read_user_configuration(&binding, None, None).is_err());
    std::fs::set_permissions(&binding.root, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(RrdEngine::read_user_configuration(&binding, None, None).is_ok());
}
