use super::*;

fn assert_tree_excludes(root: &std::path::Path, secrets: &[&str]) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                let bytes = std::fs::read(entry.path()).unwrap();
                for secret in secrets {
                    assert!(
                        !bytes
                            .windows(secret.len())
                            .any(|window| window == secret.as_bytes()),
                        "raw session secret reached {}",
                        entry.path().display()
                    );
                }
            }
        }
    }
}

#[test]
fn journal_redacts_tokens_and_records_expiry_once() {
    let (_root, service) = isolated_engine();
    let lease = service
        .create_session(
            &session_request(1_000, 2),
            &id("create-key"),
            1_000,
            "request-create",
            "operation-create",
        )
        .unwrap();
    let transaction = service
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(&id("begin-key"), "request-begin", "operation-begin"),
            1_500,
        )
        .unwrap();

    let wrong_token = id("wrong-token");
    assert!(matches!(
        service.begin_transaction(
            &lease.session_id,
            &wrong_token,
            &begin_request(),
            &mutation_context(
                &id("wrong-token-key"),
                "request-wrong-token",
                "operation-wrong-token",
            ),
            1_600,
        ),
        Err(ServiceError::Unauthenticated)
    ));
    assert_eq!(
        service
            .storage
            .control()
            .journal_since(0, 10)
            .unwrap()
            .len(),
        2
    );

    assert!(matches!(
        service.begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(&id("expire-key"), "request-expire", "operation-expire"),
            2_500,
        ),
        Err(ServiceError::SessionExpired)
    ));
    assert!(matches!(
        service.begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(
                &id("expire-retry-key"),
                "request-expire-retry",
                "operation-expire-retry",
            ),
            2_600,
        ),
        Err(ServiceError::SessionExpired)
    ));

    let journal = service.storage.control().journal_since(0, 10).unwrap();
    assert_eq!(journal.len(), 4);
    assert_eq!(journal[0].action, "session.created");
    assert_eq!(journal[1].action, "transaction.began");
    assert_eq!(journal[2].action, "transaction.retained");
    assert_eq!(journal[3].action, "session.expired");
    assert!(journal.iter().all(|entry| entry.verify()));
    assert_eq!(
        journal[1].previous_digest.as_deref(),
        Some(journal[0].digest.as_str())
    );
    assert_eq!(
        journal[2].previous_digest.as_deref(),
        Some(journal[1].digest.as_str())
    );
    for entry in &journal {
        if let Some(replacement) = &entry.replacement {
            assert!(!String::from_utf8_lossy(replacement).contains(lease.token.as_str()));
        }
    }
    assert_eq!(
        journal[3].previous_digest.as_deref(),
        Some(journal[2].digest.as_str())
    );
    let state: Value = serde_json::from_slice(journal[3].replacement.as_ref().unwrap()).unwrap();
    assert_eq!(state["status"], "expired");
    assert!(state["transactions"].as_object().unwrap().is_empty());
    let (_, state) = service
        .load_authenticated(&lease.session_id, &lease.token)
        .unwrap();
    let record = service
        .transaction_record(&state, &transaction.transaction_id)
        .unwrap()
        .unwrap();
    assert_eq!(record.lease.state, TransactionState::Expired);
}

#[test]
fn quota_transaction_expiry_and_abort_are_authoritative_transitions() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("lifecycle");
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    let lease = service
        .create_session(
            &session_request(5_000, 1),
            &id("create-key"),
            1_000,
            "request-create",
            "operation-create",
        )
        .unwrap();
    let transaction = service
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(&id("begin-key"), "request-begin", "operation-begin"),
            1_100,
        )
        .unwrap();
    assert!(matches!(
        service.begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(&id("quota-key"), "request-quota", "operation-quota"),
            1_200,
        ),
        Err(ServiceError::TransactionQuota)
    ));
    assert_eq!(
        service
            .storage
            .control()
            .journal_since(0, 10)
            .unwrap()
            .len(),
        2
    );
    drop(service);
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    assert!(matches!(
        service.abort_transaction(
            &lease.session_id,
            &lease.token,
            &transaction.transaction_id,
            &AbortTransaction {},
            &mutation_context(
                &id("expire-abort-key"),
                "request-expire",
                "operation-expire",
            ),
            2_100,
        ),
        Err(ServiceError::TransactionExpired)
    ));
    let journal = service.storage.control().journal_since(0, 10).unwrap();
    assert_eq!(journal.last().unwrap().action, "transaction.expired");

    let second = service
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(&id("begin-key-2"), "request-begin-2", "operation-begin-2"),
            2_200,
        )
        .unwrap();
    drop(service);
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    let aborted = service
        .abort_transaction(
            &lease.session_id,
            &lease.token,
            &second.transaction_id,
            &AbortTransaction {},
            &mutation_context(&id("abort-key"), "request-abort", "operation-abort"),
            2_300,
        )
        .unwrap();
    assert_eq!(aborted.state, TransactionState::Aborted);
    drop(service);
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    let replay = service
        .abort_transaction(
            &lease.session_id,
            &lease.token,
            &second.transaction_id,
            &AbortTransaction {},
            &mutation_context(
                &id("abort-key"),
                "request-abort-replay",
                "operation-abort-replay",
            ),
            2_400,
        )
        .unwrap();
    assert_eq!(replay.state, TransactionState::Aborted);
    let journal = service.storage.control().journal_since(0, 10).unwrap();
    assert_eq!(journal.last().unwrap().action, "transaction.aborted");
}

#[test]
fn create_and_begin_replay_exact_responses_and_reject_collisions() {
    let (_root, service) = isolated_engine();
    let request = session_request(5_000, 2);
    let create_key = id("create-key");
    let first = service
        .create_session(
            &request,
            &create_key,
            1_000,
            "request-create",
            "operation-create",
        )
        .unwrap();
    let replay = service
        .create_session(
            &request,
            &create_key,
            2_000,
            "request-create-replay",
            "operation-create-replay",
        )
        .unwrap();
    assert_eq!(replay, first);
    assert_eq!(
        service
            .storage
            .control()
            .journal_since(0, 10)
            .unwrap()
            .len(),
        1
    );
    assert!(matches!(
        service.create_session(
            &session_request(4_000, 2),
            &create_key,
            2_000,
            "request-create-collision",
            "operation-create-collision",
        ),
        Err(ServiceError::IdempotencyConflict)
    ));

    let begin_key = id("begin-key");
    let transaction = service
        .begin_transaction(
            &first.session_id,
            &first.token,
            &begin_request(),
            &mutation_context(&begin_key, "request-begin", "operation-begin"),
            2_100,
        )
        .unwrap();
    let transaction_replay = service
        .begin_transaction(
            &first.session_id,
            &first.token,
            &begin_request(),
            &mutation_context(&begin_key, "request-begin-replay", "operation-begin-replay"),
            2_200,
        )
        .unwrap();
    assert_eq!(transaction_replay, transaction);
    let mut collision = begin_request();
    collision.timeout_ms = 2_000;
    assert!(matches!(
        service.begin_transaction(
            &first.session_id,
            &first.token,
            &collision,
            &mutation_context(
                &begin_key,
                "request-begin-collision",
                "operation-begin-collision",
            ),
            2_300,
        ),
        Err(ServiceError::IdempotencyConflict)
    ));
    assert_eq!(
        service
            .storage
            .control()
            .journal_since(0, 10)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn renewal_rotates_without_persisting_tokens_and_close_is_idempotent() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("lifecycle");
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    let lease = service
        .create_session(
            &session_request(5_000, 2),
            &id("create-key"),
            1_000,
            "request-create",
            "operation-create",
        )
        .unwrap();
    drop(service);
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    let renewal_key = id("renew-key");
    let renewed = service
        .renew_session(
            &lease.session_id,
            &lease.token,
            &RenewSession {},
            &renewal_key,
            1_100,
            "request-renew",
            "operation-renew",
        )
        .unwrap();
    assert_ne!(renewed.token, lease.token);
    drop(service);
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    let replay = service
        .renew_session(
            &lease.session_id,
            &lease.token,
            &RenewSession {},
            &renewal_key,
            1_200,
            "request-renew-replay",
            "operation-renew-replay",
        )
        .unwrap();
    assert_eq!(replay, renewed);
    assert!(matches!(
        service.begin_transaction(
            &lease.session_id,
            &lease.token,
            &begin_request(),
            &mutation_context(
                &id("old-token-begin"),
                "request-old-token",
                "operation-old-token",
            ),
            1_300,
        ),
        Err(ServiceError::Unauthenticated)
    ));
    let transaction = service
        .begin_transaction(
            &lease.session_id,
            &renewed.token,
            &begin_request(),
            &mutation_context(
                &id("new-token-begin"),
                "request-new-token",
                "operation-new-token",
            ),
            1_300,
        )
        .unwrap();
    let closed = service
        .close_session(
            &lease.session_id,
            &renewed.token,
            &CloseSession {},
            &id("close-key"),
            1_400,
            "request-close",
            "operation-close",
        )
        .unwrap();
    assert_eq!(closed.affected_open_transactions, 1);
    assert!(!closed.idempotent_replay);
    drop(service);
    let service = RrdEngine::open(&path, instance(), TOKEN_KEY).unwrap();
    let close_replay = service
        .close_session(
            &lease.session_id,
            &renewed.token,
            &CloseSession {},
            &id("close-key"),
            1_500,
            "request-close-replay",
            "operation-close-replay",
        )
        .unwrap();
    assert!(close_replay.idempotent_replay);
    assert_eq!(close_replay.ended_at_unix_ms, closed.ended_at_unix_ms);
    assert!(matches!(
        service.abort_transaction(
            &lease.session_id,
            &renewed.token,
            &transaction.transaction_id,
            &AbortTransaction {},
            &mutation_context(
                &id("abort-after-close"),
                "request-abort-after-close",
                "operation-abort-after-close",
            ),
            1_600,
        ),
        Err(ServiceError::SessionExpired)
    ));

    let journal = service.storage.control().journal_since(0, 10).unwrap();
    assert_eq!(
        journal
            .iter()
            .map(|entry| entry.action.as_str())
            .collect::<Vec<_>>(),
        [
            "session.created",
            "session.renewed",
            "transaction.began",
            "transaction.retained",
            "session.closed"
        ]
    );
    for entry in journal {
        let replacement = String::from_utf8_lossy(entry.replacement.as_ref().unwrap());
        assert!(!replacement.contains(lease.token.as_str()));
        assert!(!replacement.contains(renewed.token.as_str()));
    }
    drop(service);
    assert_tree_excludes(&path, &[lease.token.as_str(), renewed.token.as_str()]);
}

#[test]
fn concurrent_session_transactions_preserve_leases_and_release_quota() {
    let (_root, engine) = isolated_engine();
    let session = engine
        .create_session(
            &session_request(9_000, 8),
            &id("parallel-session"),
            1_000,
            "parallel-create",
            "parallel-create",
        )
        .unwrap();
    let barrier = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|worker| {
                let engine = &engine;
                let session = &session;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    for turn in 0..8 {
                        let name = format!("parallel-{worker}-{turn}");
                        let context = mutation_context(&id(&name), &name, &name);
                        let transaction = engine
                            .begin_transaction(
                                &session.session_id,
                                &session.token,
                                &begin_request(),
                                &context,
                                1_100,
                            )
                            .unwrap();
                        let aborted = engine
                            .abort_transaction(
                                &session.session_id,
                                &session.token,
                                &transaction.transaction_id,
                                &AbortTransaction {},
                                &context,
                                1_200,
                            )
                            .unwrap();
                        assert_eq!(aborted.state, TransactionState::Aborted);
                        assert_eq!(
                            engine
                                .begin_transaction(
                                    &session.session_id,
                                    &session.token,
                                    &begin_request(),
                                    &context,
                                    1_300
                                )
                                .unwrap(),
                            aborted
                        );
                    }
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
    });
    let admission = engine.transaction_admission_snapshot().unwrap();
    assert_eq!(admission.outstanding_requests, 0);
    assert_eq!(admission.logical_bytes, 0);
    assert!(!admission.active);
    let next = engine
        .begin_transaction(
            &session.session_id,
            &session.token,
            &begin_request(),
            &mutation_context(&id("after-parallel"), "after-parallel", "after-parallel"),
            1_400,
        )
        .unwrap();
    assert_eq!(next.state, TransactionState::Open);
}

#[test]
fn begin_timing_emits_after_admission_release_without_runtime_changes() {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct ObserveGate {
        engine: Arc<RrdEngine>,
        states: Arc<Mutex<Vec<bool>>>,
    }
    impl Write for ObserveGate {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let admission = self.engine.transaction_admission_snapshot().unwrap();
            self.states.lock().unwrap().push(admission.active);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let (_root, engine) = isolated_engine();
    let engine = Arc::new(engine);
    let session = engine
        .create_session(
            &session_request(9_000, 1),
            &id("timing-session"),
            1_000,
            "timing-session",
            "timing-session",
        )
        .unwrap();
    let before = engine.storage.runtime().cursor().unwrap();
    let states = Arc::new(Mutex::new(Vec::new()));
    let writer = ObserveGate {
        engine: engine.clone(),
        states: states.clone(),
    };
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("rrd_engine::transaction_diagnostic=debug")
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        let context = mutation_context(&id("timing-begin"), "timing-begin", "timing-begin");
        let created = engine
            .begin_transaction(
                &session.session_id,
                &session.token,
                &begin_request(),
                &context,
                1_100,
            )
            .unwrap();
        let replay = engine
            .begin_transaction(
                &session.session_id,
                &session.token,
                &begin_request(),
                &context,
                1_100,
            )
            .unwrap();
        assert_eq!(created, replay);
        let over_quota = mutation_context(&id("timing-quota"), "timing-quota", "timing-quota");
        assert!(matches!(
            engine.begin_transaction(
                &session.session_id,
                &session.token,
                &begin_request(),
                &over_quota,
                1_100
            ),
            Err(ServiceError::TransactionQuota)
        ));
    });
    let states = states.lock().unwrap();
    assert!(
        states.len() >= 3,
        "all outcomes must emit a diagnostic event"
    );
    assert!(
        states.iter().all(|active| !active),
        "trace writer observed an active permit"
    );
    assert_eq!(engine.storage.runtime().cursor().unwrap(), before);
}

#[test]
fn blocked_begin_trace_does_not_hold_transaction_admission() {
    use std::io::Write;
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::Duration;

    #[derive(Clone)]
    struct HeldWriter {
        entered: mpsc::SyncSender<()>,
        release: Arc<Mutex<mpsc::Receiver<()>>>,
    }
    impl Write for HeldWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.entered.send(()).map_err(std::io::Error::other)?;
            self.release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .map_err(std::io::Error::other)?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let (_root, engine) = isolated_engine();
    let engine = Arc::new(engine);
    let session = engine
        .create_session(
            &session_request(9_000, 2),
            &id("held-trace-session"),
            1_000,
            "held-trace-session",
            "held-trace-session",
        )
        .unwrap();
    let before = engine.storage.runtime().cursor().unwrap();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let writer = HeldWriter {
        entered: entered_tx,
        release: Arc::new(Mutex::new(release_rx)),
    };
    let first_engine = engine.clone();
    let first_session = session.clone();
    let first = std::thread::spawn(move || {
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter("rrd_engine::transaction_diagnostic=debug")
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            first_engine.begin_transaction(
                &first_session.session_id,
                &first_session.token,
                &begin_request(),
                &mutation_context(&id("held-first"), "held-first", "held-first"),
                1_100,
            )
        })
    });
    entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let (second_tx, second_rx) = mpsc::sync_channel(1);
    let second_engine = engine.clone();
    let second = std::thread::spawn(move || {
        let result = second_engine.begin_transaction(
            &session.session_id,
            &session.token,
            &begin_request(),
            &mutation_context(&id("held-second"), "held-second", "held-second"),
            1_100,
        );
        second_tx.send(result).unwrap();
    });
    let while_held = second_rx.recv_timeout(Duration::from_secs(5));
    release_tx.send(()).unwrap();
    let first = first.join().unwrap().unwrap();
    second.join().unwrap();
    let second = while_held
        .expect("second begin blocked behind the diagnostic writer")
        .unwrap();
    assert_ne!(first.transaction_id, second.transaction_id);
    assert_eq!(engine.storage.runtime().cursor().unwrap(), before);
}
