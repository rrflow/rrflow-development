//! Primary-binary user publication and two authenticated, isolated project servers.
use super::*;
use rrd_contract::{
    ConfigurationSetting as Setting, NumericConfiguration, ReadEffectiveConfiguration,
    ScopedConfiguration, UserConfigurationBinding, UserConfigurationChange,
    UserConfigurationRevision,
};

fn options() -> RequestOptions {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    RequestOptions::read(
        &format!("scoped-request-{id}"),
        &format!("scoped-operation-{id}"),
    )
    .unwrap()
}

fn publish(
    root: &Path,
    source: &Path,
    expected: u64,
    rows: u64,
    items: u64,
) -> UserConfigurationRevision {
    let change = UserConfigurationChange {
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
    };
    let input = root.join(format!("user-input-{expected}.json"));
    std::fs::write(&input, serde_json::to_vec(&change).unwrap()).unwrap();
    let preview = invoke(&[
        "user-configuration",
        "plan",
        "--root",
        source.to_str().unwrap(),
        "--source",
        "shared-user",
        "--expected-revision",
        &expected.to_string(),
        "--input",
        input.to_str().unwrap(),
    ]);
    assert_success(&preview, "preview user configuration");
    if expected == 0 {
        assert!(!source.exists(), "preview wrote the global source");
    }
    let plan: rrd_contract::UserConfigurationPlan =
        serde_json::from_slice(&preview.stdout).unwrap();
    let path = root.join(format!("user-plan-{expected}.json"));
    std::fs::write(&path, preview.stdout).unwrap();
    let applied = invoke(&[
        "user-configuration",
        "apply",
        "--plan",
        path.to_str().unwrap(),
        "--expect",
        &plan.plan_sha256,
    ]);
    assert_success(&applied, "apply user configuration");
    serde_json::from_slice(&applied.stdout).unwrap()
}

fn install(root: &Path, source: &Path, name: &str, preference: Option<u64>) -> PathBuf {
    let project = root.join(name);
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("README.md"), name).unwrap();
    let default = rrd_contract::EngramConfiguration::default();
    let mut scoped = ScopedConfiguration {
        user: Some(UserConfigurationBinding {
            source_id: CanonicalId::new("shared-user").unwrap(),
            root: source.to_str().unwrap().into(),
            max_history_reads: 32,
        }),
        settings: BTreeMap::new(),
    };
    if let Some(preference) = preference {
        scoped.settings.insert(
            Setting::QueryRows,
            NumericConfiguration {
                minimum: None,
                maximum: None,
                preference: Some(preference),
            },
        );
    }
    let input = rrd_contract::EngramConfigurationInput {
        scoped,
        format_version: default.format_version,
        clock: default.clock,
        storage: default.storage,
        reasoning: default.reasoning,
        recall: default.recall,
        query: default.query,
        attunement_follow: None,
    };
    let configuration_path = root.join(format!("{name}-configuration.toml"));
    std::fs::write(&configuration_path, toml::to_string(&input).unwrap()).unwrap();
    let identity = invoke(&["install", "identity"]);
    assert_success(&identity, "installation identity");
    let identity: serde_json::Value = serde_json::from_slice(&identity.stdout).unwrap();
    let preview = invoke(&[
        "install",
        "plan",
        "--project",
        project.to_str().unwrap(),
        "--identity",
        identity["installation_identity"].as_str().unwrap(),
        "--configuration",
        configuration_path.to_str().unwrap(),
    ]);
    assert_success(&preview, "install preview with global binding");
    let plan: rrd_engine::InstallationPreview = serde_json::from_slice(&preview.stdout).unwrap();
    let path = root.join(format!("{name}-install-plan.json"));
    std::fs::write(&path, preview.stdout).unwrap();
    let applied = invoke(&[
        "install",
        "apply",
        "--project",
        project.to_str().unwrap(),
        "--plan",
        path.to_str().unwrap(),
        "--expect",
        &plan.installation.plan_sha256,
    ]);
    assert_success(&applied, "install apply with global binding");
    project
}

async fn connect(project: &Path, address: &str, name: &str) -> (RrdClient, rrd_client::Session) {
    let locator = RrdEngine::read_project_locator(project).unwrap();
    let credential = RrdEngine::read_installed_api_key(project).unwrap();
    let client = RrdClient::connect_local(
        address.parse().unwrap(),
        locator.identity.instance_id,
        ClientConfig {
            request_timeout: Duration::from_secs(30),
            ..ClientConfig::default()
        },
    )
    .unwrap();
    let session = client
        .create_session(
            credential.principal_id.clone(),
            credential.credential(),
            CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 60_000,
                    absolute_timeout_ms: 300_000,
                    max_open_transactions: 1,
                },
            },
            RequestOptions::mutation(name, name, name).unwrap(),
        )
        .await
        .unwrap();
    (client, session)
}

async fn seed(client: &RrdClient, session: &rrd_client::Session, name: &str) {
    let transaction = client
        .begin_transaction(
            session,
            BeginTransaction {
                scope: CanonicalId::new("claims").unwrap(),
                timeout_ms: 10_000,
            },
            RequestOptions::mutation(
                &format!("begin-{name}"),
                &format!("begin-{name}"),
                &format!("begin-{name}"),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let mutations = vec![TransactionMutation::AssertClaim {
        subject: CanonicalId::new(name).unwrap(),
        predicate: CanonicalId::new("status").unwrap(),
        object: format!("{name} evidence"),
        valid_from: 1,
        tx_time: 1,
        producer: CanonicalId::new("local-operator").unwrap(),
        confidence: Some(1.0),
    }];
    client
        .commit_transaction(
            session,
            &transaction.transaction_id,
            CommitTransaction {
                operation_sha256: transaction_operation_sha256(&mutations),
                mutations,
            },
            RequestOptions::new(
                &format!("commit-{name}"),
                &format!("commit-{name}"),
                Some(&format!("commit-{name}")),
                Some(u64::MAX),
            )
            .unwrap(),
        )
        .await
        .unwrap();
}

fn query(client: &RrdClient, rows: u64) -> ExecuteQuery {
    ExecuteQuery {
        scope: format!("instance:{}", client.instance_id()),
        query: "FROM claim:status AT VALID 1 KNOWN HEAD PROJECT subject, object EXPLAIN CONTRACT"
            .into(),
        parameters: BTreeMap::new(),
        budget: QueryBudget {
            max_rows: rows,
            ..QueryBudget::default()
        },
    }
}

fn recall(client: &RrdClient, items: u64) -> rrd_contract::AssembleContext {
    rrd_contract::AssembleContext {
        scope: format!("instance:{}", client.instance_id()),
        query: "evidence".into(),
        valid_at: 1,
        seeds: Vec::new(),
        max_graph_depth: 0,
        max_items: items,
        max_output_bytes: 32768,
        max_storage_keys: 10000,
    }
}

#[test]
fn scoped_configuration_two_installed_projects_observe_corrections_without_restart() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("user-configuration");
    let first = publish(root.path(), &source, 0, 20, 20);
    let a = install(root.path(), &source, "alpha", Some(8));
    let b = install(root.path(), &source, "beta", None);
    let mut captured = Vec::new();
    let (mut a_server, a_address) = RunningEngine::start(&a, &mut captured);
    let (mut b_server, b_address) = RunningEngine::start(&b, &mut captured);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let (a_client, a_session) = connect(&a, &a_address, "alpha-session").await;
        let (b_client, b_session) = connect(&b, &b_address, "beta-session").await;
        seed(&a_client, &a_session, "alpha").await;
        seed(&b_client, &b_session, "beta").await;
        let a_effective = a_client.read_effective_configuration(&a_session, ReadEffectiveConfiguration::default(), options()).await.unwrap();
        let b_effective = b_client.read_effective_configuration(&b_session, ReadEffectiveConfiguration::default(), options()).await.unwrap();
        assert_eq!(a_effective.values[&Setting::QueryRows].value, 8);
        assert_eq!(b_effective.values[&Setting::QueryRows].value, 20);
        assert_eq!(a_effective.user_revision.as_ref().unwrap(), &first);
        assert_ne!(a_effective.identity, b_effective.identity);
        for (client, session, name) in [(&a_client, &a_session, "alpha"), (&b_client, &b_session, "beta")] {
            let result = client.execute_query(session, query(client, 15), options()).await.unwrap();
            assert_eq!(result.rows.len(), 1);
            assert_eq!(result.configuration.as_ref().unwrap().user.as_ref().unwrap().read_revision, 1);
            assert_eq!(result.rows[0].values["subject"], QueryValue::String(name.into()));
            let packet = client.assemble_context(session, recall(client, 15), options()).await.unwrap();
            assert!(!packet.items.is_empty());
            assert_eq!(packet.configuration.as_ref().unwrap().user.as_ref().unwrap().revision_sha256, first.revision_sha256);
        }
        publish(root.path(), &source, 1, 10, 5);
        for (client, session) in [(&a_client, &a_session), (&b_client, &b_session)] {
            assert!(matches!(client.execute_query(session, query(client, 15), options()).await,
                Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::ResourceExhausted));
            assert!(matches!(client.assemble_context(session, recall(client, 15), options()).await,
                Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::ResourceExhausted));
            let admitted = client.execute_query(session, query(client, 4), options()).await.unwrap();
            assert_eq!(admitted.rows.len(), 1);
            assert_eq!(admitted.configuration.unwrap().user.unwrap().read_revision, 2);
            let historical = client.read_effective_configuration(session, ReadEffectiveConfiguration {
                valid_at_unix_ms: Some(1), recorded_revision: Some(1), requested: BTreeMap::new(),
            }, options()).await.unwrap();
            assert_eq!(historical.values[&Setting::QueryRows].maximum, 20);
            let current = client.read_effective_configuration(session, ReadEffectiveConfiguration::default(), options()).await.unwrap();
            assert_eq!(current.values[&Setting::QueryRows].maximum, 10);
            assert_eq!(current.user_read_revision, Some(2));
        }
        let mut foreign = query(&a_client, 4);
        foreign.scope = format!("instance:{}", b_client.instance_id());
        assert!(a_client.execute_query(&a_session, foreign, options()).await.is_err());
        a_server.stop(&mut captured);
        publish(root.path(), &source, 2, 6, 3);
        let (mut reopened, address) = RunningEngine::start(&a, &mut captured);
        let (client, session) = connect(&a, &address, "alpha-reopened-session").await;
        assert!(matches!(client.read_effective_configuration(&session, ReadEffectiveConfiguration::default(), options()).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::Conflict), "stale stored preference must not be silently clamped");
        let valid = ReadEffectiveConfiguration { requested: BTreeMap::from([(Setting::QueryRows, 4)]), ..ReadEffectiveConfiguration::default() };
        let current = client.read_effective_configuration(&session, valid, options()).await.unwrap();
        assert_eq!(current.user_read_revision, Some(3));
        assert_eq!(client.execute_query(&session, query(&client, 4), options()).await.unwrap().rows.len(), 1);
        let offline = root.path().join("temporarily-unavailable-source");
        std::fs::rename(&source, &offline).unwrap();
        assert!(matches!(client.execute_query(&session, query(&client, 4), options()).await,
            Err(rrd_client::Error::Api { error, .. }) if error.code == ErrorCode::FailedPrecondition));
        std::fs::rename(&offline, &source).unwrap();
        assert_eq!(b_client.execute_query(&b_session, query(&b_client, 4), options()).await.unwrap().rows.len(), 1);
        reopened.stop(&mut captured);
    });
    b_server.stop(&mut captured);
    let binding = UserConfigurationBinding {
        source_id: CanonicalId::new("shared-user").unwrap(),
        root: source.to_str().unwrap().into(),
        max_history_reads: 32,
    };
    assert_eq!(
        RrdEngine::read_user_configuration(&binding, Some(1), None)
            .unwrap()
            .revision,
        3,
        "global configuration survives both project shutdowns"
    );
}
