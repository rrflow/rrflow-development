use rrd_contract::*;
use std::collections::BTreeMap;

#[test]
fn scoped_configuration_rejects_unknown_effects_and_impossible_rules() {
    assert!(serde_json::from_str::<ConfigurationSettings>(
        r#"{"unknown_effect":{"minimum":null,"maximum":3,"preference":null}}"#
    )
    .is_err());
    for rule in [
        NumericConfiguration {
            minimum: Some(4),
            maximum: Some(3),
            preference: None,
        },
        NumericConfiguration {
            minimum: None,
            maximum: Some(3),
            preference: Some(4),
        },
        NumericConfiguration {
            minimum: Some(0),
            maximum: None,
            preference: None,
        },
    ] {
        assert!(validate_configuration_settings(&BTreeMap::from([(
            ConfigurationSetting::QueryRows,
            rule
        )]))
        .is_err());
    }
    let zero_hops = BTreeMap::from([(
        ConfigurationSetting::RecallGraphDepth,
        NumericConfiguration {
            minimum: Some(0),
            maximum: Some(0),
            preference: Some(0),
        },
    )]);
    validate_configuration_settings(&zero_hops).unwrap();
}

#[test]
fn scoped_configuration_changes_bind_half_open_valid_intervals_and_tamper_evidence() {
    let change = UserConfigurationChange {
        valid_from_unix_ms: 20,
        valid_to_unix_ms: Some(40),
        settings: BTreeMap::new(),
    };
    assert!(!change.contains(19));
    assert!(change.contains(20));
    assert!(change.contains(39));
    assert!(!change.contains(40));
    let mut revision = UserConfigurationRevision {
        format_version: SCOPED_CONFIGURATION_VERSION,
        source_id: CanonicalId::new("user-source").unwrap(),
        revision: 1,
        recorded_at_unix_ms: 100,
        previous_sha256: None,
        change,
        applied_plan_sha256: "a".repeat(64),
        revision_sha256: String::new(),
    };
    revision.revision_sha256 = revision.sha256().unwrap();
    revision.validate().unwrap();
    revision.change.valid_from_unix_ms = 10;
    assert!(revision.validate().is_err());
}
