//! Bookkeeping/freshness tests in isolated engine estates. The supplied output
//! tuple projects retained provider bytes onto a fresh task solely to test the
//! ledger; it is not evidence of a provider call for that packet, nor learning data.
use super::*;
use crate::engine::reasoning_budget::{
    AttemptAuthorityGuard, Reservation, TaskAttemptAdmission, TaskAttemptOutput,
};
use crate::engine::{operation_digest, Result};
use rrd_contract::{ReadTaskAttempt, TaskAttemptInvalidation, TaskAttemptStatus};

fn context(lease: &SessionLease) -> EngineControlContext<'_> {
    EngineControlContext {
        now: 50,
        session_id: &lease.session_id,
        action: "test-proposal-completion",
        request_id: "proposal-ledger",
        operation_id: "proposal-ledger",
    }
}

fn reserve(
    engine: &RrdEngine,
    lease: &SessionLease,
    request: &rrd_contract::EndpointSelectionRequest,
    cancellation: &EmbeddingCancellation,
    at: u64,
) -> Result<Reservation> {
    engine.reserve_admitted_task_attempt(
        &TaskAttemptAdmission {
            started: Instant::now(),
            scope: &request.scope,
            task_id: &request.task_id,
            attempt_id: &request.attempt_id,
            request_sha256: &operation_digest(request).unwrap(),
            cancellation,
        },
        || panic!("no repetition policy or adapter expected"),
        &FixedClock::at(at),
        &context(lease),
    )
}

fn read(
    engine: &RrdEngine,
    lease: &SessionLease,
    request: &rrd_contract::EndpointSelectionRequest,
) -> rrd_contract::TaskAttemptView {
    engine
        .read_task_attempt(
            &lease.session_id,
            &lease.token,
            &ReadTaskAttempt {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
            },
            100,
            "read-proposal-ledger",
            "read-proposal-ledger",
        )
        .unwrap()
}

#[test]
fn supplied_proposal_output_is_atomic_readable_after_reopen_and_never_reexecuted() {
    let (root, engine, lease, _, request, _, proposal) = proposal::tuple_fixture();
    let cancellation = EmbeddingCancellation::default();
    let reservation = reserve(&engine, &lease, &request, &cancellation, 1_000).unwrap();
    let key = crate::engine::session::session_key(&engine.instance, &lease.session_id);
    let expected = engine.storage.control().get(&key).unwrap().unwrap();
    engine
        .finish_admitted_task_attempt(
            reservation,
            Ok(proposal.clone().into()),
            |_| {
                Ok(vec![AttemptAuthorityGuard {
                    key: key.clone(),
                    expected: expected.clone(),
                }])
            },
            &FixedClock::at(1_100),
            &context(&lease),
        )
        .unwrap();
    assert_eq!(
        engine.storage.control().get(&key).unwrap().unwrap(),
        expected
    );
    let before = read(&engine, &lease, &request);
    let attempt = before.attempt.as_ref().unwrap();
    assert_eq!(attempt.status, TaskAttemptStatus::Accepted);
    assert!(attempt.result.is_none());
    assert_eq!(attempt.endpoint_proposal, Some(proposal.clone()));
    assert_eq!(
        attempt.outcome_sha256,
        Some(operation_digest(&proposal).unwrap())
    );
    let budget = state(&engine, &lease, &request.task_id);
    assert_eq!(budget.reserved_steps, 1);
    assert!(budget.charged_elapsed_ms > 0);
    assert!(budget.active_attempt.is_none());
    assert!(!budget.execution_may_continue);
    drop(engine);
    let mut reopened = RrdEngine::open_existing(root.path(), instance(), TOKEN_KEY).unwrap();
    configure(&mut reopened, 3);
    assert_eq!(read(&reopened, &lease, &request), before);
    assert_eq!(state(&reopened, &lease, &request.task_id), budget);
    assert!(matches!(
        reserve(&reopened, &lease, &request, &cancellation, 1_200),
        Err(ServiceError::ReasoningAttemptRecorded {
            outcome_sha256: Some(_)
        })
    ));
    let mut corrupt = before;
    corrupt
        .attempt
        .as_mut()
        .unwrap()
        .endpoint_proposal
        .as_mut()
        .unwrap()
        .usage = None;
    assert!(corrupt.validate().is_err());
}

#[test]
fn invalidation_charges_spent_effort_without_reset_or_permanent_task_stop() {
    let (root, engine, lease, _, request, _, proposal) = proposal::tuple_fixture();
    let cancellation = EmbeddingCancellation::default();
    let reservation = reserve(&engine, &lease, &request, &cancellation, 1_000).unwrap();
    let original = state(&engine, &lease, &request.task_id);
    assert!(matches!(
        engine.finish_admitted_task_attempt(
            reservation,
            Ok(proposal.into()),
            |_| Err(ServiceError::TaskAttemptInvalidated(
                TaskAttemptInvalidation::InputsChanged
            )),
            &FixedClock::at(1_100),
            &context(&lease)
        ),
        Err(ServiceError::TaskAttemptInvalidated(
            TaskAttemptInvalidation::InputsChanged
        ))
    ));
    let receipt = read(&engine, &lease, &request).attempt.unwrap();
    assert_eq!(receipt.status, TaskAttemptStatus::Invalidated);
    assert_eq!(
        receipt.invalidation,
        Some(TaskAttemptInvalidation::InputsChanged)
    );
    assert!(receipt.endpoint_proposal.is_none() && receipt.outcome_sha256.is_none());
    let spent = state(&engine, &lease, &request.task_id);
    assert!(spent.stopped.is_none() && spent.active_attempt.is_none());
    assert!(spent.charged_elapsed_ms > original.charged_elapsed_ms);
    assert_eq!(spent.reserved_steps, 1);
    drop(engine);
    let mut engine = RrdEngine::open_existing(root.path(), instance(), TOKEN_KEY).unwrap();
    configure(&mut engine, 3);
    assert_eq!(state(&engine, &lease, &request.task_id), spent);
    assert!(matches!(
        reserve(&engine, &lease, &request, &cancellation, 1_200),
        Err(ServiceError::ReasoningAttemptRecorded {
            outcome_sha256: None
        })
    ));
    let mut successor = request.clone();
    successor.attempt_id = id("successor-proposal-ledger-test");
    drop(reserve(&engine, &lease, &successor, &cancellation, 1_200).unwrap());
    let next = state(&engine, &lease, &request.task_id);
    assert_eq!(next.deadline_unix_ms, original.deadline_unix_ms);
    assert_eq!(next.started_at_unix_ms, original.started_at_unix_ms);
    assert_eq!(next.charged_elapsed_ms, spent.charged_elapsed_ms);
    assert_eq!(next.reserved_steps, 2);
}

#[test]
fn session_change_between_guard_read_and_commit_cannot_publish_accepted_output() {
    let (_root, engine, lease, _, request, _, proposal) = proposal::tuple_fixture();
    let reservation = reserve(
        &engine,
        &lease,
        &request,
        &EmbeddingCancellation::default(),
        1_000,
    )
    .unwrap();
    let key = crate::engine::session::session_key(&engine.instance, &lease.session_id);
    let expected = engine.storage.control().get(&key).unwrap().unwrap();
    let mut checks = 0;
    let result = engine.finish_admitted_task_attempt(
        reservation,
        Ok(proposal.into()),
        |_| {
            checks += 1;
            if engine.storage.control().get(&key)?.as_deref() != Some(expected.as_slice()) {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::AuthorizationChanged,
                ));
            }
            // A real session transition occurs after the check and before the
            // completion CAS. The callback deliberately returns the now-stale guard.
            engine.close_session(
                &lease.session_id,
                &lease.token,
                &rrd_contract::CloseSession {},
                &id("guard-session-close"),
                60,
                "guard-session-close",
                "guard-session-close",
            )?;
            Ok(vec![AttemptAuthorityGuard {
                key: key.clone(),
                expected: expected.clone(),
            }])
        },
        &FixedClock::at(1_100),
        &context(&lease),
    );
    assert!(matches!(
        result,
        Err(ServiceError::TaskAttemptInvalidated(
            TaskAttemptInvalidation::AuthorizationChanged
        ))
    ));
    assert_eq!(
        checks, 2,
        "stale control guard must conflict and re-admit completion"
    );
    assert_ne!(
        engine.storage.control().get(&key).unwrap().unwrap(),
        expected
    );
    let reader = engine
        .create_session(
            &session_request(9_000, 4),
            &id("guard-reader"),
            70,
            "guard-reader",
            "guard-reader",
        )
        .unwrap();
    let attempt = read(&engine, &reader, &request).attempt.unwrap();
    assert_eq!(attempt.status, TaskAttemptStatus::Invalidated);
    assert!(attempt.endpoint_proposal.is_none());
    assert!(state(&engine, &reader, &request.task_id).stopped.is_none());
}

#[test]
fn task_closure_between_freshness_read_and_commit_invalidates_the_attempt() {
    let (_root, engine, lease, _, request, _, proposal) = proposal::tuple_fixture();
    let reservation = reserve(
        &engine,
        &lease,
        &request,
        &EmbeddingCancellation::default(),
        1_000,
    )
    .unwrap();
    let mut checks = 0;
    let result = engine.finish_admitted_task_attempt(
        reservation,
        Ok(proposal.into()),
        |read| {
            checks += 1;
            if matches!(
                engine.open_task_at(read, &request.scope, &request.task_id),
                Err(ServiceError::TaskClosed)
            ) {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::InputsChanged,
                ));
            }
            engine.close_task(
                &lease.session_id,
                &lease.token,
                &rrd_contract::CloseTask {
                    scope: request.scope.clone(),
                    task_id: request.task_id.clone(),
                    status: rrd_contract::TaskStatus::Abandoned,
                    outcome: None,
                },
                60,
                "guard-task-close",
                "guard-task-close",
            )?;
            Ok(Vec::new())
        },
        &FixedClock::at(1_100),
        &context(&lease),
    );
    assert!(matches!(
        result,
        Err(ServiceError::TaskAttemptInvalidated(
            TaskAttemptInvalidation::InputsChanged
        ))
    ));
    assert_eq!(checks, 2);
    assert_eq!(
        read(&engine, &lease, &request).attempt.unwrap().status,
        TaskAttemptStatus::Invalidated
    );
}

#[test]
fn uncertain_failure_and_cancellation_never_retain_successful_output() {
    for cancel in [false, true] {
        let (_root, engine, lease, _, request, _, proposal) = proposal::tuple_fixture();
        let cancellation = EmbeddingCancellation::default();
        let reservation = reserve(&engine, &lease, &request, &cancellation, 1_000).unwrap();
        let result = if cancel {
            cancellation.cancel();
            Ok(TaskAttemptOutput::from(proposal))
        } else {
            // Controlled classification input, not a newly observed provider failure.
            Err(ServiceError::EndpointCallFailed {
                reason: "test unresolved transport outcome",
                execution_may_continue: true,
            })
        };
        assert!(engine
            .finish_admitted_task_attempt(
                reservation,
                result,
                |_| if cancel {
                    Err(ServiceError::TaskAttemptInvalidated(
                        TaskAttemptInvalidation::InputsChanged,
                    ))
                } else {
                    Ok(Vec::new())
                },
                &FixedClock::at(1_100),
                &context(&lease)
            )
            .is_err());
        let attempt = read(&engine, &lease, &request).attempt.unwrap();
        assert_eq!(attempt.status, TaskAttemptStatus::Stopped);
        assert!(attempt.endpoint_proposal.is_none() && attempt.outcome_sha256.is_none());
        let budget = state(&engine, &lease, &request.task_id);
        assert_eq!(budget.execution_may_continue, !cancel);
        assert_eq!(
            budget.stopped,
            Some(if cancel {
                ReasoningBudgetStop::Cancelled
            } else {
                ReasoningBudgetStop::AttemptFailed
            })
        );
    }
}
