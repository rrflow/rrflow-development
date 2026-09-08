use super::*;
use rrd_contract::{
    CloseSubscription, OpenSubscription, SubscriptionAcknowledgement, SubscriptionDelivery,
    SubscriptionLeaseCoordinate, SubscriptionPoll, SubscriptionResume, SubscriptionStatus,
    SubscriptionStream,
};
use rrd_core::{
    RuntimeCommit, RuntimeMutation, RuntimeProperties, RuntimePropertySchema, RuntimeRecord,
    RuntimeRecordSchema, RuntimeRef, RuntimeSchemaRegistry, RuntimeType, RuntimeValue,
    RuntimeValueType, ScopeId,
};
use std::collections::BTreeMap;

fn commit_claim(engine: &RrdEngine, lease: &rrd_contract::SessionLease, suffix: &str, now: u64) {
    let begun = engine
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(
                &id(&format!("begin-subscription-{suffix}")),
                &format!("request-begin-subscription-{suffix}"),
                &format!("operation-begin-subscription-{suffix}"),
            ),
            now,
        )
        .unwrap();
    engine
        .commit_transaction(
            &lease.session_id,
            &lease.token,
            &begun.transaction_id,
            &id(&format!("commit-subscription-{suffix}")),
            &commit_request(suffix),
            now,
            &format!("request-commit-subscription-{suffix}"),
            &format!("operation-commit-subscription-{suffix}"),
        )
        .unwrap();
}

fn changefeed_subscription(name: &str, after_cursor: u64, retention: u64) -> OpenSubscription {
    OpenSubscription {
        subscription_id: id(name),
        stream: SubscriptionStream::Changefeed {
            scope: format!("instance:{}", instance()),
        },
        after_cursor,
        batch_size: 1,
        max_in_flight: 2,
        retention_cursor_window: retention,
        lease_ms: 5_000,
        heartbeat_interval_ms: 100,
    }
}

#[test]
fn durable_subscription_fences_connections_bounds_delivery_and_replays_after_reopen() {
    let (root, engine) = isolated_engine();
    let lease = engine
        .create_session(
            &session_request(8_000, 8),
            &id("subscription-session"),
            1_000,
            "request-subscription-session",
            "operation-subscription-session",
        )
        .unwrap();
    commit_claim(&engine, &lease, "one", 1_010);
    commit_claim(&engine, &lease, "two", 1_020);
    commit_claim(&engine, &lease, "three", 1_030);

    let request = changefeed_subscription("subscription-durable", 0, 100);
    let opened = engine
        .open_subscription(
            &lease.session_id,
            &lease.token,
            &id("open-subscription-durable"),
            &request,
            1_040,
            "request-open-subscription-durable",
            "operation-open-subscription-durable",
        )
        .unwrap();
    assert!(!opened.idempotent_replay);
    assert_eq!(opened.subscription.head_cursor, 3);
    assert_eq!(opened.subscription.connection_generation, 0);
    let replayed_open = engine
        .open_subscription(
            &lease.session_id,
            &lease.token,
            &id("open-subscription-durable"),
            &request,
            1_041,
            "request-open-subscription-durable-replay",
            "operation-open-subscription-durable-replay",
        )
        .unwrap();
    assert!(replayed_open.idempotent_replay);

    let connected = engine
        .connect_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionResume::from_snapshot(&opened.subscription),
            1_050,
            "request-connect-subscription-durable",
            "operation-connect-subscription-durable",
        )
        .unwrap();
    assert_eq!(connected.connection_generation, 1);
    let first = engine
        .next_subscription_delivery(
            &lease.session_id,
            &lease.token,
            &request.subscription_id,
            1,
            1_051,
            "request-next-subscription-one",
            "operation-next-subscription-one",
        )
        .unwrap();
    let second = engine
        .next_subscription_delivery(
            &lease.session_id,
            &lease.token,
            &request.subscription_id,
            1,
            1_052,
            "request-next-subscription-two",
            "operation-next-subscription-two",
        )
        .unwrap();
    let (second_sequence, second_cursor) = match (&first, &second) {
        (
            SubscriptionPoll::Delivery { batch: first },
            SubscriptionPoll::Delivery { batch: second },
        ) => {
            let SubscriptionDelivery::Changefeed { page: first_page } = &first.delivery else {
                panic!("expected first changefeed delivery")
            };
            let SubscriptionDelivery::Changefeed { page: second_page } = &second.delivery else {
                panic!("expected second changefeed delivery")
            };
            assert_eq!(first_page.changes[0].cursor, 1);
            assert_eq!(second_page.changes[0].cursor, 2);
            assert_eq!(first_page.changes[0].commit_ordinal, 0);
            assert_eq!(second_page.changes[0].commit_ordinal, 0);
            (second.delivery_sequence, second.through_cursor)
        }
        other => panic!("unexpected subscription frames: {other:?}"),
    };
    assert!(matches!(
        engine.next_subscription_delivery(
            &lease.session_id,
            &lease.token,
            &request.subscription_id,
            1,
            1_053,
            "request-next-subscription-blocked",
            "operation-next-subscription-blocked",
        ),
        Err(ServiceError::SubscriptionBackpressure)
    ));
    let acknowledged = engine
        .acknowledge_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionAcknowledgement {
                subscription_id: request.subscription_id.clone(),
                connection_generation: 1,
                delivery_sequence: second_sequence,
                through_cursor: second_cursor,
            },
            1_054,
            "request-ack-subscription-two",
            "operation-ack-subscription-two",
        )
        .unwrap();
    assert_eq!(acknowledged.acknowledged_cursor, 2);

    drop(engine);
    let reopened = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    let reconnected = reopened
        .connect_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionResume::from_snapshot(&acknowledged),
            1_060,
            "request-reconnect-subscription",
            "operation-reconnect-subscription",
        )
        .unwrap();
    assert_eq!(reconnected.connection_generation, 2);
    assert_eq!(reconnected.acknowledged_cursor, 2);
    assert!(matches!(
        reopened.next_subscription_delivery(
            &lease.session_id,
            &lease.token,
            &request.subscription_id,
            1,
            1_061,
            "request-stale-subscription",
            "operation-stale-subscription",
        ),
        Err(ServiceError::SubscriptionConnectionReplaced)
    ));
    let third = reopened
        .next_subscription_delivery(
            &lease.session_id,
            &lease.token,
            &request.subscription_id,
            2,
            1_062,
            "request-replayed-subscription-three",
            "operation-replayed-subscription-three",
        )
        .unwrap();
    assert!(matches!(
        third,
        SubscriptionPoll::Delivery { ref batch }
            if batch.from_cursor == 2 && batch.through_cursor == 3
                && matches!(batch.delivery, SubscriptionDelivery::Changefeed { .. })
    ));

    let closed = reopened
        .close_subscription(
            &lease.session_id,
            &lease.token,
            &id("close-subscription-durable"),
            &CloseSubscription {
                subscription_id: request.subscription_id.clone(),
            },
            1_070,
            "request-close-subscription",
            "operation-close-subscription",
        )
        .unwrap();
    assert_eq!(closed.subscription.status, SubscriptionStatus::Closed);
    assert!(matches!(
        reopened.connect_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionResume::from_snapshot(&closed.subscription),
            1_071,
            "request-connect-closed-subscription",
            "operation-connect-closed-subscription",
        ),
        Err(ServiceError::SubscriptionClosed)
    ));
}

#[test]
fn resume_and_lease_coordinates_reject_stale_or_drifted_state() {
    let (_root, engine) = isolated_engine();
    let lease = engine
        .create_session(
            &session_request(8_000, 4),
            &id("subscription-coordinate-session"),
            1_000,
            "request-subscription-coordinate-session",
            "operation-subscription-coordinate-session",
        )
        .unwrap();
    commit_claim(&engine, &lease, "coordinate", 1_010);
    let request = changefeed_subscription("subscription-coordinate", 0, 100);
    let opened = engine
        .open_subscription(
            &lease.session_id,
            &lease.token,
            &id("open-subscription-coordinate"),
            &request,
            1_020,
            "request-open-subscription-coordinate",
            "operation-open-subscription-coordinate",
        )
        .unwrap();
    let connected = engine
        .connect_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionResume::from_snapshot(&opened.subscription),
            1_030,
            "request-connect-subscription-coordinate",
            "operation-connect-subscription-coordinate",
        )
        .unwrap();

    assert!(matches!(
        engine.connect_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionResume::from_snapshot(&opened.subscription),
            1_031,
            "request-stale-resume",
            "operation-stale-resume",
        ),
        Err(ServiceError::Subscription(_))
    ));
    let mut drifted_resume = SubscriptionResume::from_snapshot(&connected);
    drifted_resume.stream_sha256 = "f".repeat(64);
    assert!(matches!(
        engine.connect_subscription(
            &lease.session_id,
            &lease.token,
            &drifted_resume,
            1_032,
            "request-drifted-resume",
            "operation-drifted-resume",
        ),
        Err(ServiceError::Subscription(_))
    ));

    let coordinate = SubscriptionLeaseCoordinate::from_snapshot(&connected);
    let mut wrong_generation = coordinate.clone();
    wrong_generation.connection_generation += 1;
    assert!(matches!(
        engine.renew_subscription_lease(
            &lease.session_id,
            &lease.token,
            &wrong_generation,
            1_033,
            "request-wrong-generation-heartbeat",
            "operation-wrong-generation-heartbeat",
        ),
        Err(ServiceError::SubscriptionConnectionReplaced)
    ));
    let mut wrong_cursor = coordinate.clone();
    wrong_cursor.acknowledged_cursor += 1;
    assert!(matches!(
        engine.renew_subscription_lease(
            &lease.session_id,
            &lease.token,
            &wrong_cursor,
            1_034,
            "request-wrong-cursor-heartbeat",
            "operation-wrong-cursor-heartbeat",
        ),
        Err(ServiceError::Subscription(_))
    ));
    let renewed = engine
        .renew_subscription_lease(
            &lease.session_id,
            &lease.token,
            &coordinate,
            1_035,
            "request-valid-heartbeat",
            "operation-valid-heartbeat",
        )
        .unwrap();
    assert_eq!(
        renewed.connection_generation,
        connected.connection_generation
    );
    assert_eq!(renewed.acknowledged_cursor, connected.acknowledged_cursor);
    assert!(renewed.lease_expires_at_unix_ms > connected.lease_expires_at_unix_ms);
}

#[test]
fn subscription_retention_floor_and_owner_are_fail_closed() {
    let (_root, engine) = isolated_engine();
    let owner = engine
        .create_session(
            &session_request(8_000, 8),
            &id("subscription-owner-session"),
            1_000,
            "request-subscription-owner",
            "operation-subscription-owner",
        )
        .unwrap();
    let stranger = engine
        .create_session(
            &session_request(8_000, 8),
            &id("subscription-stranger-session"),
            1_001,
            "request-subscription-stranger",
            "operation-subscription-stranger",
        )
        .unwrap();
    for (suffix, now) in [("a", 1_010), ("b", 1_020), ("c", 1_030)] {
        commit_claim(&engine, &owner, suffix, now);
    }
    let expired = changefeed_subscription("subscription-expired", 0, 1);
    assert!(matches!(
        engine.open_subscription(
            &owner.session_id,
            &owner.token,
            &id("open-subscription-expired"),
            &expired,
            1_040,
            "request-open-subscription-expired",
            "operation-open-subscription-expired",
        ),
        Err(ServiceError::SubscriptionExpired {
            requested: 0,
            retention_floor: 2,
            head: 3
        })
    ));

    let request = changefeed_subscription("subscription-owned", 2, 1);
    let opened = engine
        .open_subscription(
            &owner.session_id,
            &owner.token,
            &id("open-subscription-owned"),
            &request,
            1_041,
            "request-open-subscription-owned",
            "operation-open-subscription-owned",
        )
        .unwrap();
    assert!(matches!(
        engine.connect_subscription(
            &stranger.session_id,
            &stranger.token,
            &SubscriptionResume::from_snapshot(&opened.subscription),
            1_042,
            "request-steal-subscription",
            "operation-steal-subscription",
        ),
        Err(ServiceError::PermissionDenied)
    ));
}

#[test]
fn global_commit_cursor_orders_interleaved_writers_for_one_distributed_feed() {
    let (_root, engine) = isolated_engine();
    let first_writer = engine
        .create_session(
            &session_request(8_000, 8),
            &id("distributed-writer-one"),
            1_000,
            "request-distributed-writer-one",
            "operation-distributed-writer-one",
        )
        .unwrap();
    let second_writer = engine
        .create_session(
            &session_request(8_000, 8),
            &id("distributed-writer-two"),
            1_001,
            "request-distributed-writer-two",
            "operation-distributed-writer-two",
        )
        .unwrap();
    commit_claim(&engine, &first_writer, "writer-one-a", 1_010);
    commit_claim(&engine, &second_writer, "writer-two", 1_020);
    commit_claim(&engine, &first_writer, "writer-one-b", 1_030);

    let mut request = changefeed_subscription("distributed-feed", 0, 128);
    request.batch_size = 3;
    let opened = engine
        .open_subscription(
            &first_writer.session_id,
            &first_writer.token,
            &id("open-distributed-feed"),
            &request,
            1_040,
            "request-open-distributed-feed",
            "operation-open-distributed-feed",
        )
        .unwrap();
    let connected = engine
        .connect_subscription(
            &first_writer.session_id,
            &first_writer.token,
            &SubscriptionResume::from_snapshot(&opened.subscription),
            1_041,
            "request-connect-distributed-feed",
            "operation-connect-distributed-feed",
        )
        .unwrap();
    let frame = engine
        .next_subscription_delivery(
            &first_writer.session_id,
            &first_writer.token,
            &request.subscription_id,
            connected.connection_generation,
            1_042,
            "request-next-distributed-feed",
            "operation-next-distributed-feed",
        )
        .unwrap();
    let page = match frame {
        SubscriptionPoll::Delivery { batch } => match batch.delivery {
            SubscriptionDelivery::Changefeed { page } => page,
            delivery => panic!("expected distributed changefeed delivery, got {delivery:?}"),
        },
        frame => panic!("expected distributed changefeed frame, got {frame:?}"),
    };
    assert_eq!(
        page.changes
            .iter()
            .map(|change| change.cursor)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        page.changes[0].actor,
        format!("session:{}", first_writer.session_id.as_str())
    );
    assert_eq!(
        page.changes[1].actor,
        format!("session:{}", second_writer.session_id.as_str())
    );
    assert_eq!(
        page.changes[2].actor,
        format!("session:{}", first_writer.session_id.as_str())
    );
    assert!(page
        .changes
        .windows(2)
        .all(|pair| pair[1].previous_change_sha256.as_deref() == Some(&pair[0].change_sha256)));
}

#[test]
fn subscription_open_requires_both_subscription_and_stream_permissions() {
    let (_root, engine) = isolated_engine();
    let principal_id = CanonicalId::new("subscription-policy-reader").unwrap();
    let resource = ResourcePath {
        segments: vec![ResourceId::new(ResourceKind::Instance, instance().as_str()).unwrap()],
    };
    let principal = Principal {
        id: principal_id.clone(),
        kind: PrincipalKind::Service,
        credential_sha256: digest::sha256_hex(b"subscription-policy-key"),
        credential_revision: 1,
        not_before_unix_ms: 1,
        expires_at_unix_ms: u64::MAX,
        disabled: false,
        role_ids: Default::default(),
        grants: [
            SecurityAction::SessionCreate,
            SecurityAction::SubscriptionOpen,
        ]
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
                format_version: rrd_security::SECURITY_FORMAT,
                revision: 1,
                principals: [(principal_id.clone(), principal)].into_iter().collect(),
                roles: Default::default(),
                identity_bindings: Default::default(),
                jwt_issuers: Default::default(),
            },
            1,
            "subscription-policy-test",
            "request-subscription-policy-bootstrap",
            "operation-subscription-policy-bootstrap",
        )
        .unwrap();
    let lease = engine
        .create_authenticated_session(
            &principal_id,
            b"subscription-policy-key",
            &session_request(5_000, 2),
            &id("subscription-policy-session"),
            1_000,
            "request-subscription-policy-session",
            "operation-subscription-policy-session",
        )
        .unwrap();
    let request = changefeed_subscription("subscription-policy-denied", 0, 128);
    assert!(matches!(
        engine.open_subscription(
            &lease.session_id,
            &lease.token,
            &id("subscription-policy-open"),
            &request,
            1_010,
            "request-subscription-policy-open",
            "operation-subscription-policy-open",
        ),
        Err(ServiceError::PermissionDenied)
    ));
    assert!(engine
        .storage
        .control_record(&format!(
            "server/state/{}/subscription/{}",
            instance(),
            request.subscription_id.as_str()
        ))
        .unwrap()
        .is_none());
}

#[test]
fn live_query_subscription_pushes_semantic_delta_on_the_runtime_cursor() {
    let (_root, engine) = isolated_engine();
    let scope = ScopeId::new(format!("instance:{}", instance())).unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "subscription live query fixture");
    registry.records.insert(
        RuntimeType::new("document").unwrap(),
        RuntimeRecordSchema {
            properties: BTreeMap::from([(
                "status".into(),
                RuntimePropertySchema::required(RuntimeValueType::String),
            )]),
            ..RuntimeRecordSchema::default()
        },
    );
    let record = |status: &str| RuntimeRecord {
        reference: RuntimeRef::new("document", "alpha").unwrap(),
        valid_from: 10,
        valid_to: None,
        properties: RuntimeProperties::from([(
            "status".into(),
            RuntimeValue::String(status.into()),
        )]),
    };
    engine
        .storage
        .commit_runtime(&RuntimeCommit {
            scope: scope.clone(),
            at: 10,
            actor: "node-one".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema {
                    registry: registry.clone(),
                },
                RuntimeMutation::Record {
                    record: record("open"),
                },
            ],
        })
        .unwrap();
    engine
        .storage
        .commit_runtime(&RuntimeCommit {
            scope: scope.clone(),
            at: 20,
            actor: "node-two".into(),
            expected_cursor: 2,
            mutations: vec![RuntimeMutation::Record {
                record: record("closed"),
            }],
        })
        .unwrap();
    let lease = engine
        .create_session(
            &session_request(8_000, 2),
            &id("live-subscription-session"),
            1_000,
            "request-live-subscription-session",
            "operation-live-subscription-session",
        )
        .unwrap();
    let request = OpenSubscription {
        subscription_id: id("live-subscription"),
        stream: SubscriptionStream::LiveQuery {
            scope: format!("instance:{}", instance()),
            query: "FROM record:document AT VALID 100 KNOWN HEAD PROJECT id, status".into(),
            parameters: BTreeMap::new(),
            budget: rrd_contract::QueryBudget::default(),
            max_delta_rows: 16,
        },
        after_cursor: 2,
        batch_size: 1,
        max_in_flight: 1,
        retention_cursor_window: 128,
        lease_ms: 5_000,
        heartbeat_interval_ms: 100,
    };
    let opened = engine
        .open_subscription(
            &lease.session_id,
            &lease.token,
            &id("open-live-subscription"),
            &request,
            1_010,
            "request-open-live-subscription",
            "operation-open-live-subscription",
        )
        .unwrap();
    let connected = engine
        .connect_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionResume::from_snapshot(&opened.subscription),
            1_011,
            "request-connect-live-subscription",
            "operation-connect-live-subscription",
        )
        .unwrap();
    let frame = engine
        .next_subscription_delivery(
            &lease.session_id,
            &lease.token,
            &request.subscription_id,
            connected.connection_generation,
            1_012,
            "request-next-live-subscription",
            "operation-next-live-subscription",
        )
        .unwrap();
    let (delivery_sequence, through_cursor) = match frame {
        SubscriptionPoll::Delivery { batch } => {
            let delivery_sequence = batch.delivery_sequence;
            let through_cursor = batch.through_cursor;
            let SubscriptionDelivery::LiveQuery { delta } = batch.delivery else {
                panic!("expected live query delivery")
            };
            assert_eq!(delta.from_cursor, 2);
            assert_eq!(delta.through_cursor, 3);
            assert_eq!(delta.updated.len(), 1);
            assert_eq!(
                delta.updated[0].before.values["status"],
                rrd_contract::QueryValue::String("open".into())
            );
            assert_eq!(
                delta.updated[0].after.values["status"],
                rrd_contract::QueryValue::String("closed".into())
            );
            (delivery_sequence, through_cursor)
        }
        frame => panic!("expected live query frame, got {frame:?}"),
    };
    engine
        .acknowledge_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionAcknowledgement {
                subscription_id: request.subscription_id.clone(),
                connection_generation: connected.connection_generation,
                delivery_sequence,
                through_cursor,
            },
            1_013,
            "request-ack-live-subscription",
            "operation-ack-live-subscription",
        )
        .unwrap();

    // A commit can advance the read stamp without changing the query result.
    // That cursor-only interval must still be delivered and acknowledged so a
    // resumed subscription never replays it forever.
    registry.revision = 2;
    registry.migration = "subscription cursor-only schema revision".into();
    engine
        .storage
        .commit_runtime(&RuntimeCommit {
            scope,
            at: 30,
            actor: "node-three".into(),
            expected_cursor: 3,
            mutations: vec![RuntimeMutation::Schema { registry }],
        })
        .unwrap();
    let cursor_only = engine
        .next_subscription_delivery(
            &lease.session_id,
            &lease.token,
            &request.subscription_id,
            connected.connection_generation,
            1_014,
            "request-next-cursor-only-subscription",
            "operation-next-cursor-only-subscription",
        )
        .unwrap();
    let SubscriptionPoll::Delivery { batch } = cursor_only else {
        panic!("expected cursor-only live-query delivery")
    };
    batch.validate().unwrap();
    assert_eq!(batch.from_cursor, 3);
    assert_eq!(batch.through_cursor, 4);
    let SubscriptionDelivery::LiveQuery { delta } = &batch.delivery else {
        panic!("expected cursor-only live-query delivery")
    };
    assert!(delta.added.is_empty());
    assert!(delta.updated.is_empty());
    assert!(delta.removed.is_empty());
    let acknowledged = engine
        .acknowledge_subscription(
            &lease.session_id,
            &lease.token,
            &SubscriptionAcknowledgement {
                subscription_id: request.subscription_id,
                connection_generation: connected.connection_generation,
                delivery_sequence: batch.delivery_sequence,
                through_cursor: batch.through_cursor,
            },
            1_015,
            "request-ack-cursor-only-subscription",
            "operation-ack-cursor-only-subscription",
        )
        .unwrap();
    assert_eq!(acknowledged.acknowledged_cursor, 4);
}
