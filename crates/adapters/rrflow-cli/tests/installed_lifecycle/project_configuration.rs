//! Real installed HTTP editing, isolation and correction readback.
use super::scoped_configuration::{connect, install, options, publish, query, recall, seed};
use super::*;
use rrd_contract::{
    ApplyProjectConfiguration, ConfigurationChange, ConfigurationSetting as Setting,
    NumericConfiguration, PreviewProjectConfiguration, ReadEffectiveConfiguration,
    ReadProjectConfiguration,
};

fn change(rows: u64, items: u64) -> ConfigurationChange {
    ConfigurationChange {
        valid_from_unix_ms: 0,
        valid_to_unix_ms: None,
        settings: BTreeMap::from([
            (
                Setting::QueryRows,
                NumericConfiguration {
                    minimum: Some(2),
                    maximum: Some(rows),
                    preference: Some(rows),
                },
            ),
            (
                Setting::RecallItems,
                NumericConfiguration {
                    minimum: Some(1),
                    maximum: Some(items),
                    preference: Some(items),
                },
            ),
        ]),
    }
}
fn mutation(name: &str) -> RequestOptions {
    RequestOptions::mutation(name, name, name).unwrap()
}
fn apply(plan: rrd_contract::ProjectConfigurationPlan) -> ApplyProjectConfiguration {
    ApplyProjectConfiguration {
        approved_plan_sha256: plan.plan_sha256.clone(),
        plan,
    }
}

#[test]
fn project_policy_edits_are_live_isolated_and_temporal_after_restart() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("user");
    publish(root.path(), &source, 0, 20, 20);
    let a = install(root.path(), &source, "alpha", Some(8));
    let b = install(root.path(), &source, "beta", None);
    let mut captured = Vec::new();
    let (mut server, address) = RunningEngine::start(&a, &mut captured);
    let (mut other_server, other_address) = RunningEngine::start(&b, &mut captured);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let (client, session) = connect(&a, &address, "edit-session").await;
        let (other, other_session) = connect(&b, &other_address, "other-session").await;
        seed(&client, &session, "alpha").await;
        seed(&other, &other_session, "beta").await;
        let initial = client.read_project_configuration(&session, ReadProjectConfiguration {}, options()).await.unwrap();
        assert_eq!(initial.revision(), 0);
        let request = PreviewProjectConfiguration { expected_revision: 0, change: change(8, 5) };
        let plan = client.preview_project_configuration(&session, request.clone(), options()).await.unwrap();
        assert_eq!(plan.values[&Setting::QueryRows].value, 8);
        assert_eq!(client.read_project_configuration(&session, ReadProjectConfiguration {}, options()).await.unwrap(), initial, "preview mutated state");
        let mut unapproved = apply(plan.clone()); unapproved.approved_plan_sha256 = "0".repeat(64);
        assert!(client.apply_project_configuration(&session, unapproved, mutation("unapproved")).await.is_err());
        assert!(other.apply_project_configuration(&other_session, apply(plan.clone()), mutation("foreign")).await.is_err());
        publish(root.path(), &source, 1, 10, 10);
        assert!(matches!(client.apply_project_configuration(&session, apply(plan), mutation("changed-user")).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::Conflict));
        let plan = client.preview_project_configuration(&session, request, options()).await.unwrap();
        let accepted = apply(plan);
        let first = client.apply_project_configuration(&session, accepted.clone(), mutation("first")).await.unwrap();
        assert_eq!(first.revision(), 1);
        assert_eq!(client.apply_project_configuration(&session, accepted.clone(), mutation("retry-first")).await.unwrap(), first);
        assert!(matches!(client.execute_query(&session, query(&client, 9), options()).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::ResourceExhausted));
        assert_eq!(other.execute_query(&other_session, query(&other, 9), options()).await.unwrap().rows.len(), 1);
        assert!(matches!(client.assemble_context(&session, recall(&client, 6), options()).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::ResourceExhausted));
        let result = client.execute_query(&session, query(&client, 4), options()).await.unwrap();
        assert_eq!(result.configuration.unwrap().project_policy.unwrap().revision_sha256, first.head.as_ref().unwrap().revision_sha256);
        let packet = client.assemble_context(&session, recall(&client, 4), options()).await.unwrap();
        assert_eq!(packet.configuration.unwrap().project_policy.unwrap().selected_revision, 1);

        let mut correction = change(6, 4); correction.valid_from_unix_ms = 1; correction.valid_to_unix_ms = Some(2);
        let plan = client.preview_project_configuration(&session, PreviewProjectConfiguration { expected_revision: 1, change: correction }, options()).await.unwrap();
        let competing = client.preview_project_configuration(&session, PreviewProjectConfiguration { expected_revision: 1, change: change(7, 4) }, options()).await.unwrap();
        client.apply_project_configuration(&session, apply(plan), mutation("correction")).await.unwrap();
        assert!(matches!(client.apply_project_configuration(&session, apply(competing), mutation("stale-project")).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::Conflict));
        assert!(client.apply_project_configuration(&session, accepted, mutation("stale-retry")).await.is_err());
        for (known, rows) in [(0, 8), (1, 8), (2, 6)] {
            let result = client.read_effective_configuration(&session, ReadEffectiveConfiguration {
                valid_at_unix_ms: Some(1), recorded_revision: Some(1), project_recorded_revision: Some(known), requested: BTreeMap::new(),
            }, options()).await.unwrap();
            assert_eq!(result.values[&Setting::QueryRows].value, rows);
            assert_eq!(result.project_read_revision, known);
        }
        publish(root.path(), &source, 2, 3, 3);
        assert!(matches!(client.read_effective_configuration(&session, ReadEffectiveConfiguration::default(), options()).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::Conflict));
        let state = client.read_project_configuration(&session, ReadProjectConfiguration {}, options()).await.unwrap();
        assert_eq!(state.revision(), 2, "conflict must not block repair-state reads");
        assert!(matches!(client.preview_project_configuration(&session, PreviewProjectConfiguration { expected_revision: 2, change: change(4, 3) }, options()).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::Conflict));
        let repair = client.preview_project_configuration(&session, PreviewProjectConfiguration { expected_revision: 2, change: change(3, 3) }, options()).await.unwrap();
        client.apply_project_configuration(&session, apply(repair), mutation("repair")).await.unwrap();
        assert_eq!(client.read_effective_configuration(&session, ReadEffectiveConfiguration::default(), options()).await.unwrap().values[&Setting::QueryRows].value, 3);
        server.stop(&mut captured);
        let (mut reopened, address) = RunningEngine::start(&a, &mut captured);
        let (client, session) = connect(&a, &address, "reopened-edit").await;
        let result = client.execute_query(&session, query(&client, 3), options()).await.unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.configuration.unwrap().project_policy.unwrap().selected_revision, 3);
        let historical = client.read_effective_configuration(&session, ReadEffectiveConfiguration {
            valid_at_unix_ms: Some(1), recorded_revision: Some(1), project_recorded_revision: Some(2), requested: BTreeMap::new(),
        }, options()).await.unwrap();
        assert_eq!(historical.values[&Setting::QueryRows].value, 6);
        assert_eq!(other.read_project_configuration(&other_session, ReadProjectConfiguration {}, options()).await.unwrap().revision(), 0);
        reopened.stop(&mut captured);
    });
    other_server.stop(&mut captured);
}
