use super::*;
use rrd_contract::{
    ApplyProjectConfiguration, ConfigurationChange, ConfigurationSetting, InstalledEngramIdentity,
    NumericConfiguration, PreviewProjectConfiguration, SessionLease,
};
use std::collections::{BTreeMap, BTreeSet};

fn bind(engine: &mut RrdEngine) {
    engine.installed_engram = Some(InstalledEngramIdentity {
        project_id: CanonicalId::new("policy-project").unwrap(),
        engram_id: CanonicalId::new("policy-engram").unwrap(),
        instance_id: instance(),
    });
}
fn context(name: &str) -> RequestContext {
    mutation_context(&id(name), name, name)
}
fn request(rows: u64) -> PreviewProjectConfiguration {
    PreviewProjectConfiguration {
        expected_revision: 0,
        change: ConfigurationChange {
            valid_from_unix_ms: 0,
            valid_to_unix_ms: None,
            settings: BTreeMap::from([(
                ConfigurationSetting::QueryRows,
                NumericConfiguration {
                    minimum: None,
                    maximum: Some(rows),
                    preference: Some(rows),
                },
            )]),
        },
    }
}
fn session(engine: &RrdEngine) -> SessionLease {
    engine
        .create_session(
            &session_request(5000, 2),
            &id("policy-session"),
            1000,
            "session",
            "session",
        )
        .unwrap()
}
fn apply(
    engine: &RrdEngine,
    lease: &SessionLease,
    plan: rrd_contract::ProjectConfigurationPlan,
    key: &str,
) -> crate::Result<rrd_contract::ProjectConfiguration> {
    engine.apply_project_configuration(
        &lease.session_id,
        &lease.token,
        &ApplyProjectConfiguration {
            approved_plan_sha256: plan.plan_sha256.clone(),
            plan,
        },
        &context(key),
        1002,
    )
}

#[test]
fn project_configuration_concurrent_publish_has_one_winner_and_atomic_history() {
    let (_root, mut engine) = isolated_engine();
    bind(&mut engine);
    let lease = session(&engine);
    let a = engine
        .preview_project_configuration(
            &lease.session_id,
            &lease.token,
            &request(7),
            &context("preview-a"),
            1001,
        )
        .unwrap();
    let b = engine
        .preview_project_configuration(
            &lease.session_id,
            &lease.token,
            &request(8),
            &context("preview-b"),
            1001,
        )
        .unwrap();
    let barrier = std::sync::Barrier::new(2);
    let results = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            barrier.wait();
            apply(&engine, &lease, a, "a")
        });
        let right = scope.spawn(|| {
            barrier.wait();
            apply(&engine, &lease, b, "b")
        });
        [left.join().unwrap(), right.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(ServiceError::StorageConflict(_))))
            .count(),
        1
    );
    let winner = results.into_iter().find_map(Result::ok).unwrap();
    let head = winner.head.unwrap();
    let selected = engine
        .read_effective_configuration(
            &lease.session_id,
            &lease.token,
            &rrd_contract::ReadEffectiveConfiguration {
                project_recorded_revision: Some(1),
                ..Default::default()
            },
            &context("read"),
            1003,
        )
        .unwrap();
    assert_eq!(selected.project_policy, Some(head));
}

#[test]
fn project_configuration_expired_or_tampered_plan_cannot_publish() {
    let (_root, mut engine) = isolated_engine();
    bind(&mut engine);
    let lease = session(&engine);
    let plan = engine
        .preview_project_configuration(
            &lease.session_id,
            &lease.token,
            &request(7),
            &context("preview"),
            1001,
        )
        .unwrap();
    let mut expired = plan.clone();
    expired.issued_at_unix_ms = 1;
    expired.expires_at_unix_ms = 1 + rrd_contract::CONFIGURATION_PLAN_TTL_MS;
    expired.plan_sha256 = expired.sha256().unwrap();
    assert!(matches!(
        apply(&engine, &lease, expired, "expired"),
        Err(ServiceError::DeadlineExceeded)
    ));
    let mut changed = plan;
    changed.change = request(9).change;
    assert!(apply(&engine, &lease, changed, "tampered").is_err());
    assert_eq!(
        engine
            .read_project_configuration(&lease.session_id, &lease.token, &context("read"), 1003)
            .unwrap()
            .revision(),
        0
    );
}

#[test]
fn project_configuration_reader_cannot_preview_or_publish() {
    let mut engine = RrdEngine::rrflow_mx(instance(), TOKEN_KEY);
    bind(&mut engine);
    let owner = session(&engine);
    let plan = engine
        .preview_project_configuration(
            &owner.session_id,
            &owner.token,
            &request(7),
            &context("preview"),
            1001,
        )
        .unwrap();
    let principal_id = CanonicalId::new("policy-reader").unwrap();
    let resource = ResourcePath {
        segments: vec![ResourceId::new(ResourceKind::Instance, instance().as_str()).unwrap()],
    };
    let principal = Principal {
        id: principal_id.clone(),
        kind: PrincipalKind::Service,
        credential_sha256: digest::sha256_hex(b"reader"),
        credential_revision: 1,
        not_before_unix_ms: 1,
        expires_at_unix_ms: u64::MAX,
        disabled: false,
        role_ids: BTreeSet::new(),
        grants: [SecurityAction::SessionCreate, SecurityAction::EngramRead]
            .into_iter()
            .map(|action| ResourceGrant {
                action,
                resource_prefix: resource.clone(),
                data_policy: None,
            })
            .collect(),
    };
    SecurityRepository::new(&engine.storage, instance())
        .initialize(
            SecurityState {
                format_version: rrflow_security::SECURITY_FORMAT,
                revision: 1,
                principals: BTreeMap::from([(principal_id.clone(), principal)]),
                roles: BTreeMap::new(),
                identity_bindings: BTreeMap::new(),
                jwt_issuers: BTreeMap::new(),
            },
            1000,
            "test",
            "security",
            "security",
        )
        .unwrap();
    let reader = engine
        .create_authenticated_session(
            &principal_id,
            b"reader",
            &session_request(5000, 2),
            &id("reader"),
            1000,
            "reader",
            "reader",
        )
        .unwrap();
    assert_eq!(
        engine
            .read_project_configuration(&reader.session_id, &reader.token, &context("read"), 1001)
            .unwrap()
            .revision(),
        0
    );
    assert!(engine
        .preview_project_configuration(
            &reader.session_id,
            &reader.token,
            &request(7),
            &context("denied"),
            1001
        )
        .is_err());
    assert!(apply(&engine, &reader, plan, "denied-apply").is_err());
}
