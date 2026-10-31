//! Durable task effort at the existing control/runtime transaction boundary.
//! An acknowledged reservation is never permission to repeat an uncertain call.

use super::clock::{observe_required, ClockSource, HostSystemClock};
use super::*;
use rrd_inference::{EmbeddingCancellation, EmbeddingExecutionControl};

mod persistence;
mod public;
mod repetition;

pub use rrd_contract::{ReasoningBudgetStop, TaskReasoningBudget};

/// Attempt identity is stable across a lost response or reconnect. A new
/// identity spends another step; it cannot change this task's original budget.
pub struct TaskEmbeddingAttempt<'a> {
    pub task_id: &'a CanonicalId,
    pub attempt_id: &'a CorrelationId,
    pub request: &'a GenerateEmbeddings,
    pub cancellation: &'a EmbeddingCancellation,
}

/// Engine-internal effect identity for the one task budget. Callers must
/// authorize and derive this identity before reserving; it is not a public
/// capability or permission to dispatch an effect.
pub(super) struct TaskAttemptAdmission<'a> {
    /// Captured before effect-specific identity preparation, never reset by it.
    pub started: Instant,
    pub scope: &'a str,
    pub task_id: &'a CanonicalId,
    pub attempt_id: &'a CorrelationId,
    pub request_sha256: &'a str,
    pub cancellation: &'a EmbeddingCancellation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttemptRecord {
    format_version: u16,
    attempt_id: CorrelationId,
    request_sha256: String,
    ordinal: u64,
    read_manifest_sha256: String,
    read_cursor: u64,
    outcome: AttemptOutcome,
    outcome_sha256: Option<String>,
    result: Option<GenerateEmbeddingsResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    endpoint_proposal: Option<rrd_contract::EndpointSelectionProposal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worker_output: Option<rrd_contract::TaskWorkerOutput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    invalidation: Option<rrd_contract::TaskAttemptInvalidation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AttemptOutcome {
    Reserved,
    Accepted,
    Stopped,
    Invalidated,
}

/// Internal typed completion; public callers cannot inject a retained output.
#[derive(Debug, Clone)]
pub(super) enum TaskAttemptOutput {
    Embedding(Box<GenerateEmbeddingsResult>),
    EndpointProposal(Box<rrd_contract::EndpointSelectionProposal>),
    Worker(Box<rrd_contract::TaskWorkerOutput>),
}

impl From<rrd_contract::EndpointSelectionProposal> for TaskAttemptOutput {
    fn from(output: rrd_contract::EndpointSelectionProposal) -> Self {
        Self::EndpointProposal(Box::new(output))
    }
}

impl TaskAttemptOutput {
    fn bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::Embedding(output) => serde_json::to_vec(output),
            Self::EndpointProposal(output) => serde_json::to_vec(output),
            Self::Worker(output) => serde_json::to_vec(output),
        }
        .map_err(contract_json)
    }

    fn validate_for(&self, budget: &TaskReasoningBudget, attempt: &AttemptRecord) -> Result<()> {
        match self {
            Self::Worker(output) => {
                output
                    .validate()
                    .map_err(|e| ServiceError::Contract(e.to_string()))?;
                if output.scope != budget.scope
                    || output.task_id != budget.task_id
                    || output.attempt_id != attempt.attempt_id
                {
                    return Err(ServiceError::Contract(
                        "task worker output identity mismatch".into(),
                    ));
                }
            }
            Self::Embedding(output) => {
                output
                    .validate()
                    .map_err(|e| ServiceError::Contract(e.to_string()))?;
                if output.scope != budget.scope
                    || self.bytes()?.len() > rrd_contract::MAX_TASK_EMBEDDING_OUTPUT_BYTES
                {
                    return Err(ServiceError::ConfigurationLimit(
                        "task embedding output scope or bound mismatch".into(),
                    ));
                }
            }
            Self::EndpointProposal(output) => {
                output
                    .validate()
                    .map_err(|e| ServiceError::Contract(e.to_string()))?;
                if output.scope != budget.scope
                    || output.task_id != budget.task_id
                    || output.attempt_id != attempt.attempt_id
                {
                    return Err(ServiceError::Contract(
                        "task proposal output identity mismatch".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Engine-owned control bytes that must still match at the completion commit.
/// These are compare-only semantic guards: their replacement is the same bytes.
/// They never accept client-supplied keys or modify another authority's value.
pub(super) struct AttemptAuthorityGuard {
    pub key: String,
    pub expected: Vec<u8>,
}

/// Effect-specific admission state committed with the existing attempt ledger.
/// Only engine composition supplies these keys; callers cannot inject writes.
pub(super) struct AttemptControlWrite {
    pub key: String,
    pub expected: Option<Vec<u8>>,
    pub replacement: Vec<u8>,
}

#[derive(Default)]
pub(super) struct TaskCompletionAdmission {
    pub guards: Vec<AttemptAuthorityGuard>,
    pub writes: Vec<AttemptControlWrite>,
    pub invalidation: Option<rrd_contract::TaskAttemptInvalidation>,
}

pub(super) struct Reservation {
    budget: TaskReasoningBudget,
    attempt: AttemptRecord,
    read: ReadStamp,
    control: EmbeddingExecutionControl,
    started: Instant,
}

impl Reservation {
    pub(super) fn execution_control(&self) -> &EmbeddingExecutionControl {
        &self.control
    }
}

impl RrdEngine {
    /// Execute one embedding attempt under the task's original durable effort
    /// allowance. This entry point requires ReasoningWrite AND EmbeddingGenerate.
    /// No automatic retries; a recorded attempt returns a receipt digest or
    /// unresolved status rather than re-executing or fabricating lost output.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_task_embeddings(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        attempt: &TaskEmbeddingAttempt<'_>,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<GenerateEmbeddingsResult> {
        self.generate_task_embeddings_with_clock(
            session_id,
            token,
            attempt,
            now,
            request_id,
            operation_id,
            &HostSystemClock,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn generate_task_embeddings_with_clock(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        attempt: &TaskEmbeddingAttempt<'_>,
        now: u64,
        request_id: &str,
        operation_id: &str,
        clock: &dyn ClockSource,
    ) -> Result<GenerateEmbeddingsResult> {
        attempt
            .request
            .validate()
            .map_err(|e| ServiceError::Contract(e.to_string()))?;
        for action in [
            SecurityAction::ReasoningWrite,
            SecurityAction::EmbeddingGenerate,
        ] {
            self.authorize(session_id, token, action, now, request_id, operation_id)?;
        }
        let context = EngineControlContext {
            now,
            session_id,
            action: "reasoning-attempt",
            request_id,
            operation_id,
        };
        // Registration and durable Stop share this short lock. No provider
        // computation holds it; the durable budget remains the only authority.
        let key = self.budget_key(&attempt.request.scope, attempt.task_id);
        let reservation = {
            let mut active = self
                .active_task_cancellations
                .lock()
                .map_err(|_| ServiceError::Storage("task cancellation lock poisoned".into()))?;
            let reservation = self.reserve_task_attempt(attempt, clock, &context)?;
            active.insert(
                key.clone(),
                (attempt.attempt_id.clone(), attempt.cancellation.clone()),
            );
            reservation
        };
        let _active = ActiveTaskCancellation {
            engine: self,
            key,
            attempt_id: attempt.attempt_id.clone(),
        };
        let result = self.generate_embeddings_at_with_control(
            attempt.request,
            reservation.read.clone(),
            &reservation.control,
        );
        self.finish_task_attempt(reservation, result, clock, &context)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inspect_task_reasoning_budget(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        scope: &str,
        task_id: &CanonicalId,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<Option<TaskReasoningBudget>> {
        self.authorize(
            session_id,
            token,
            SecurityAction::ReasoningRead,
            now,
            request_id,
            operation_id,
        )?;
        self.query_scope(scope)?;
        self.load_task_budget(scope, task_id)
            .map(|(_, budget)| budget)
    }

    pub(super) fn reserve_task_attempt(
        &self,
        input: &TaskEmbeddingAttempt<'_>,
        clock: &dyn ClockSource,
        context: &EngineControlContext<'_>,
    ) -> Result<Reservation> {
        let started = Instant::now();
        let request_sha256 = operation_digest(input.request)?;
        self.reserve_admitted_task_attempt(
            &TaskAttemptAdmission {
                started,
                scope: &input.request.scope,
                task_id: input.task_id,
                attempt_id: input.attempt_id,
                request_sha256: &request_sha256,
                cancellation: input.cancellation,
            },
            || self.embedding_repetition_identity(input.request),
            clock,
            context,
        )
    }

    /// Shared reservation, with deferred effect-specific equivalence. Replay,
    /// prior stops and exhausted budgets must not resolve an adapter again.
    pub(super) fn reserve_admitted_task_attempt(
        &self,
        input: &TaskAttemptAdmission<'_>,
        repetition_identity: impl FnOnce() -> Result<String>,
        clock: &dyn ClockSource,
        context: &EngineControlContext<'_>,
    ) -> Result<Reservation> {
        self.reserve_controlled_task_attempt(
            input,
            repetition_identity,
            &TaskCompletionAdmission::default(),
            clock,
            context,
        )
    }

    pub(super) fn reserve_controlled_task_attempt(
        &self,
        input: &TaskAttemptAdmission<'_>,
        repetition_identity: impl FnOnce() -> Result<String>,
        admission: &TaskCompletionAdmission,
        clock: &dyn ClockSource,
        context: &EngineControlContext<'_>,
    ) -> Result<Reservation> {
        if let Some(reason) = admission.invalidation {
            return Err(ServiceError::TaskAttemptInvalidated(reason));
        }
        let started = input.started;
        validate_attempt_digest(input.request_sha256)?;
        let at = observe_required(clock, "reasoning-reserve")?.observed_at_unix_ms;
        let scope = self.query_scope(input.scope)?;
        let read = self.storage.runtime().read_stamp(&scope)?;
        // The following semantic/control commit also checks this runtime cursor:
        // concurrent task closure cannot publish a new reservation.
        self.open_task_at(&read, input.scope, input.task_id)?;
        let (expected, existing) = self.load_task_budget(input.scope, input.task_id)?;
        let attempt_key = self.attempt_key(input.scope, input.task_id, input.attempt_id);
        if let Some(bytes) = self.storage.control().get(&attempt_key)? {
            let previous: AttemptRecord = serde_json::from_slice(&bytes).map_err(contract_json)?;
            if previous.format_version != 1 || previous.attempt_id != *input.attempt_id {
                return Err(ServiceError::Storage(
                    "reasoning attempt identity is corrupt".into(),
                ));
            }
            if previous.request_sha256 != input.request_sha256 {
                return Err(ServiceError::IdempotencyConflict);
            }
            return Err(ServiceError::ReasoningAttemptRecorded {
                outcome_sha256: previous.outcome_sha256,
            });
        }
        let limits = self.estate_configuration.reasoning.clone();
        let mut budget = existing.unwrap_or(TaskReasoningBudget {
            format_version: 1,
            instance_id: self.instance.clone(),
            scope: input.scope.into(),
            task_id: input.task_id.clone(),
            configuration_sha256: self.estate_configuration.configuration_sha256.clone(),
            deadline_unix_ms: at
                .checked_add(limits.max_run_elapsed_ms)
                .ok_or(ServiceError::DeadlineExceeded)?,
            limits,
            started_at_unix_ms: at,
            last_observed_at_unix_ms: at,
            charged_elapsed_ms: 0,
            reserved_steps: 0,
            active_attempt: None,
            execution_may_continue: false,
            stopped: None,
            stop_expected_sha256: None,
            repetition_policy: None,
            equivalent_attempts: BTreeMap::new(),
        });
        if let Some(reason) = budget.stopped {
            return Err(ServiceError::ReasoningBudgetStopped(reason));
        }
        if budget.active_attempt.is_some() {
            return Err(ServiceError::ReasoningBudgetStopped(
                ReasoningBudgetStop::UnresolvedAttempt,
            ));
        }
        let mut stop = if at < budget.last_observed_at_unix_ms {
            Some(ReasoningBudgetStop::ClockRollback)
        } else if budget.configuration_sha256 != self.estate_configuration.configuration_sha256 {
            Some(ReasoningBudgetStop::ConfigurationChanged)
        } else if input.cancellation.is_cancelled() {
            Some(ReasoningBudgetStop::Cancelled)
        } else if at >= budget.deadline_unix_ms
            || budget.charged_elapsed_ms >= budget.limits.max_run_elapsed_ms
        {
            Some(ReasoningBudgetStop::Deadline)
        } else if budget.reserved_steps >= budget.limits.max_steps {
            Some(ReasoningBudgetStop::StepLimit)
        } else {
            None
        };
        let identity = if stop.is_none() && budget.repetition_policy.is_some() {
            let identity = repetition_identity()?;
            validate_attempt_digest(&identity)?;
            Some(identity)
        } else {
            None
        };
        if let Some(identity) = &identity {
            stop = repetition::check(&budget, identity);
        }
        budget.last_observed_at_unix_ms = budget.last_observed_at_unix_ms.max(at);
        if let Some(reason) = stop {
            budget.stopped = Some(reason);
            self.commit_budget(&read, expected, &budget, None, context)?;
            return Err(ServiceError::ReasoningBudgetStopped(reason));
        }
        let remaining = (budget.deadline_unix_ms - at)
            .min(budget.limits.max_run_elapsed_ms - budget.charged_elapsed_ms)
            .min(budget.limits.max_step_elapsed_ms);
        let deadline = started
            .checked_add(Duration::from_millis(remaining))
            .ok_or(ServiceError::DeadlineExceeded)?;
        budget.reserved_steps += 1;
        if let Some(identity) = identity {
            *budget.equivalent_attempts.entry(identity).or_default() += 1;
        }
        budget.active_attempt = Some(input.attempt_id.clone());
        budget.execution_may_continue = true;
        let attempt = AttemptRecord {
            format_version: 1,
            attempt_id: input.attempt_id.clone(),
            request_sha256: input.request_sha256.into(),
            ordinal: budget.reserved_steps,
            read_manifest_sha256: read.manifest_id.clone(),
            read_cursor: read.commit_cursor,
            outcome: AttemptOutcome::Reserved,
            outcome_sha256: None,
            result: None,
            endpoint_proposal: None,
            worker_output: None,
            invalidation: None,
        };
        self.commit_budget_controlled(
            &read,
            expected,
            &budget,
            Some((None, &attempt)),
            admission,
            context,
        )?;
        Ok(Reservation {
            budget,
            attempt,
            read,
            started,
            control: EmbeddingExecutionControl::new(deadline, input.cancellation.clone()),
        })
    }

    fn finish_task_attempt(
        &self,
        reservation: Reservation,
        result: Result<GenerateEmbeddingsResult>,
        clock: &dyn ClockSource,
        context: &EngineControlContext<'_>,
    ) -> Result<GenerateEmbeddingsResult> {
        match self.finish_admitted_task_attempt(
            reservation,
            result.map(|output| TaskAttemptOutput::Embedding(Box::new(output))),
            |_| Ok(Vec::new()),
            clock,
            context,
        )? {
            TaskAttemptOutput::Embedding(output) => Ok(*output),
            TaskAttemptOutput::EndpointProposal(_) | TaskAttemptOutput::Worker(_) => {
                Err(ServiceError::Storage("unexpected task output kind".into()))
            }
        }
    }

    pub(super) fn finish_admitted_task_attempt(
        &self,
        reservation: Reservation,
        result: Result<TaskAttemptOutput>,
        mut admit_completion: impl FnMut(&ReadStamp) -> Result<Vec<AttemptAuthorityGuard>>,
        clock: &dyn ClockSource,
        context: &EngineControlContext<'_>,
    ) -> Result<TaskAttemptOutput> {
        self.finish_controlled_task_attempt(
            reservation,
            result,
            |read| match admit_completion(read) {
                Ok(guards) => Ok(TaskCompletionAdmission {
                    guards,
                    ..Default::default()
                }),
                Err(ServiceError::TaskAttemptInvalidated(reason)) => Ok(TaskCompletionAdmission {
                    invalidation: Some(reason),
                    ..Default::default()
                }),
                Err(error) => Err(error),
            },
            clock,
            context,
        )
    }

    pub(super) fn finish_controlled_task_attempt(
        &self,
        reservation: Reservation,
        mut result: Result<TaskAttemptOutput>,
        mut admit_completion: impl FnMut(&ReadStamp) -> Result<TaskCompletionAdmission>,
        clock: &dyn ClockSource,
        context: &EngineControlContext<'_>,
    ) -> Result<TaskAttemptOutput> {
        let Reservation {
            budget: reserved,
            mut attempt,
            control,
            started,
            ..
        } = reservation;
        let expected_attempt = Some(serde_json::to_vec(&attempt).map_err(contract_json)?);
        if let Ok(output) = &result {
            if let Err(error) = output.validate_for(&reserved, &attempt) {
                result = Err(error);
            }
        }
        for index in 0..3 {
            let (expected_budget, current) =
                self.load_task_budget(&reserved.scope, &reserved.task_id)?;
            let mut budget = current
                .ok_or_else(|| ServiceError::Storage("reserved budget disappeared".into()))?;
            if budget.active_attempt.as_ref() != Some(&attempt.attempt_id)
                || budget.reserved_steps != reserved.reserved_steps
                || budget.limits != reserved.limits
                || budget.configuration_sha256 != reserved.configuration_sha256
            {
                return Err(ServiceError::StorageConflict(
                    "attempt no longer owns completion".into(),
                ));
            }
            let scope = self.query_scope(&budget.scope)?;
            let read = self.storage.runtime().read_stamp(&scope)?;
            // Cleanup also runs for known failures. If admission cannot be read,
            // leave the reservation unresolved rather than releasing capacity.
            let admission = admit_completion(&read)?;
            if result.is_ok() {
                if let Some(reason) = admission.invalidation {
                    result = Err(ServiceError::TaskAttemptInvalidated(reason));
                }
            }
            // Reobserve after admission and on every conflict retry. A slow
            // freshness check cannot extend the original deadline/effort.
            let at = observe_required(clock, "reasoning-complete")?.observed_at_unix_ms;
            let elapsed = u64::try_from(started.elapsed().as_millis())
                .unwrap_or(u64::MAX)
                .saturating_add(1);
            budget.charged_elapsed_ms = reserved.charged_elapsed_ms.saturating_add(elapsed);
            let stop = budget.stopped.or_else(|| {
                if at < budget.last_observed_at_unix_ms {
                    Some(ReasoningBudgetStop::ClockRollback)
                } else if at >= budget.deadline_unix_ms
                    || budget.charged_elapsed_ms >= budget.limits.max_run_elapsed_ms
                    || control.check().is_err()
                {
                    Some(match control.check() {
                        Err(rrd_inference::EmbeddingDispatchError::Cancelled { .. }) => {
                            ReasoningBudgetStop::Cancelled
                        }
                        _ => ReasoningBudgetStop::Deadline,
                    })
                } else if result.is_err()
                    && !matches!(result, Err(ServiceError::TaskAttemptInvalidated(_)))
                {
                    Some(ReasoningBudgetStop::AttemptFailed)
                } else {
                    None
                }
            });
            if let Some(reason) = stop {
                budget.stopped = Some(reason);
                if result.as_ref().err().is_none_or(ServiceError::retryable)
                    || matches!(result, Err(ServiceError::TaskAttemptInvalidated(_)))
                {
                    result = Err(ServiceError::ReasoningBudgetStopped(reason));
                }
            }
            budget.last_observed_at_unix_ms = budget.last_observed_at_unix_ms.max(at);
            budget.active_attempt = None;
            budget.execution_may_continue = match &result {
                Err(ServiceError::InferenceDeadlineExceeded {
                    execution_may_continue,
                })
                | Err(ServiceError::InferenceCancelled {
                    execution_may_continue,
                })
                | Err(ServiceError::EndpointCallFailed {
                    execution_may_continue,
                    ..
                }) => *execution_may_continue,
                Err(ServiceError::Storage(_)) => true,
                _ => false,
            };
            attempt.invalidation = match &result {
                Err(ServiceError::TaskAttemptInvalidated(reason)) => Some(*reason),
                _ => None,
            };
            attempt.outcome = if result.is_ok() {
                AttemptOutcome::Accepted
            } else if attempt.invalidation.is_some() {
                AttemptOutcome::Invalidated
            } else {
                AttemptOutcome::Stopped
            };
            attempt.outcome_sha256 = result
                .as_ref()
                .ok()
                .map(|output| output.bytes().map(|bytes| digest::sha256_hex(&bytes)))
                .transpose()?;
            attempt.result = match &result {
                Ok(TaskAttemptOutput::Embedding(output)) => Some((**output).clone()),
                _ => None,
            };
            attempt.endpoint_proposal = match &result {
                Ok(TaskAttemptOutput::EndpointProposal(output)) => Some((**output).clone()),
                _ => None,
            };
            attempt.worker_output = match &result {
                Ok(TaskAttemptOutput::Worker(output)) => Some((**output).clone()),
                _ => None,
            };
            match self.commit_budget_controlled(
                &read,
                expected_budget,
                &budget,
                Some((expected_attempt.clone(), &attempt)),
                &admission,
                context,
            ) {
                Err(ServiceError::StorageConflict(_)) if index < 2 => continue,
                outcome => {
                    outcome?;
                    break;
                }
            }
        }
        if result.is_ok() {
            control
                .check()
                .map_err(super::inference::embedding_dispatch_error)?;
        }
        result
    }
}

fn validate_attempt_digest(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ServiceError::Contract(
            "task attempt identity must be a lowercase SHA-256 digest".into(),
        ));
    }
    Ok(())
}

pub(super) struct ActiveTaskCancellation<'a> {
    pub engine: &'a RrdEngine,
    pub key: String,
    pub attempt_id: CorrelationId,
}
impl Drop for ActiveTaskCancellation<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.engine.active_task_cancellations.lock() {
            // Completion may have admitted the next attempt before this guard
            // runs. An old guard must never unregister the next attempt's Stop.
            if active
                .get(&self.key)
                .is_some_and(|(id, _)| id == &self.attempt_id)
            {
                active.remove(&self.key);
            }
        }
    }
}
