//! One selected model activity under the existing task attempt transaction.
//! Accepted output is a draft, never a project mutation or correctness label.
use super::clock::{observe_required, ClockSource, HostSystemClock};
use super::endpoint_proposal::{capacity::WorkerCapacityLease, endpoint_failure};
use super::reasoning_budget::{
    ActiveTaskCancellation, AttemptAuthorityGuard, TaskAttemptAdmission, TaskAttemptOutput,
    TaskCompletionAdmission,
};
use super::*;
use rrd_contract::{
    ContextPacket, ContextReadStamp, EndpointSelectionChoice, EndpointSelectionTask, ReadTask,
    ReadTaskAttempt, TaskAttemptInvalidation, TaskAttemptStatus, TaskWorkerContext,
    TaskWorkerOutput, TaskWorkerRequest, ENDPOINT_WORKER_VERSION,
};
use rrd_inference::endpoint_chat::{ChatMessage, ChatRole, ModelChatRequest, PreparedModelChat};
use rrd_inference::{EmbeddingCancellation, NetworkPolicy};

const ACTIONS: [SecurityAction; 4] = [
    SecurityAction::WorkerExecute,
    SecurityAction::TaskRead,
    SecurityAction::ReasoningRead,
    SecurityAction::ReasoningWrite,
];

#[derive(Serialize)]
struct WorkerPacket {
    contract_version: u16,
    task: EndpointSelectionTask,
    selection_sha256: String,
    choice: EndpointSelectionChoice,
    configuration_sha256: String,
    read: ContextReadStamp,
    context: Option<ContextPacket>,
}

struct WorkerAuthority {
    guards: Vec<(AttemptAuthorityGuard, TaskAttemptInvalidation)>,
}

impl RrdEngine {
    pub fn execute_task_worker(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &TaskWorkerRequest,
        request_id: &str,
        operation_id: &str,
    ) -> Result<TaskWorkerOutput> {
        self.execute_task_worker_with_clock(
            session,
            token,
            request,
            request_id,
            operation_id,
            &HostSystemClock,
        )
    }

    pub(super) fn execute_task_worker_with_clock(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &TaskWorkerRequest,
        request_id: &str,
        operation_id: &str,
        clock: &dyn ClockSource,
    ) -> Result<TaskWorkerOutput> {
        let started = Instant::now();
        request.validate().map_err(worker_contract)?;
        let now = observe_required(clock, "worker-authorize")?.observed_at_unix_ms;
        let (security_key, security_bytes) =
            rrd_security::SecurityRepository::new(&self.storage, self.instance.clone())
                .authority_snapshot()?
                .ok_or(ServiceError::PermissionDenied)?;
        let session_bytes =
            self.authorize_worker(session, token, request, now, request_id, operation_id)?;
        let (configuration_bytes, configuration) = self.load_endpoint_configuration()?;
        if configuration.sha256().map_err(worker_contract)? != request.expected_configuration_sha256
        {
            return Err(ServiceError::TaskAttemptInvalidated(
                TaskAttemptInvalidation::InputsChanged,
            ));
        }
        let profile = configuration
            .profile
            .as_ref()
            .ok_or_else(|| ServiceError::Contract("endpoint profile is absent".into()))?;
        let policy = profile
            .worker_policy
            .as_ref()
            .ok_or_else(|| ServiceError::Contract("worker policy is absent".into()))?;
        if !policy.allow_network
            || !policy.allow_context_export
            || !policy.allow_unverified_identity
        {
            return Err(ServiceError::PermissionDenied);
        }
        let (selection_guard, retained) = self.read_task_attempt_record(&ReadTaskAttempt {
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            attempt_id: request.selection_attempt_id.clone(),
        })?;
        let selection = retained
            .attempt
            .filter(|attempt| {
                attempt.status == TaskAttemptStatus::Accepted
                    && attempt.outcome_sha256.as_ref() == Some(&request.expected_selection_sha256)
            })
            .and_then(|attempt| attempt.endpoint_proposal)
            .ok_or_else(|| {
                ServiceError::Contract("accepted selection does not match worker request".into())
            })?;
        if selection.task_sha256 != request.expected_task_sha256
            || selection.configuration_sha256 != request.expected_configuration_sha256
        {
            return Err(ServiceError::TaskAttemptInvalidated(
                TaskAttemptInvalidation::InputsChanged,
            ));
        }
        let selector = profile
            .selection_policy
            .as_ref()
            .ok_or_else(|| ServiceError::Contract("selector policy is absent".into()))?;
        let allowance = selector
            .allowance(&configuration)
            .map_err(worker_contract)?;
        if selection.allowance_sha256
            != allowance.sha256(&configuration).map_err(worker_contract)?
        {
            return Err(ServiceError::TaskAttemptInvalidated(
                TaskAttemptInvalidation::InputsChanged,
            ));
        }
        EndpointSelectionChoice::decode(
            &serde_json::to_vec(&selection.choice).map_err(contract_json)?,
            &allowance,
            &configuration,
        )
        .map_err(worker_contract)?;
        let EndpointSelectionChoice::Select { binding_id, effort } = &selection.choice else {
            return Err(ServiceError::Contract(
                "abstention cannot dispatch a worker".into(),
            ));
        };
        let worker = policy
            .bindings
            .iter()
            .find(|worker| &worker.binding_id == binding_id)
            .ok_or_else(|| ServiceError::Contract("selected worker is not configured".into()))?;
        let binding = profile
            .bindings
            .iter()
            .find(|binding| &binding.id == binding_id)
            .ok_or_else(|| ServiceError::Contract("selected binding is absent".into()))?;
        let read = self
            .storage
            .runtime()
            .read_stamp(&self.query_scope(&request.scope)?)?;
        let task = self.read_task_record_at(&read, &worker_task_read(request))?;
        let task = EndpointSelectionTask::from_task(&task).map_err(worker_contract)?;
        if task.task_sha256 != request.expected_task_sha256 {
            return Err(ServiceError::TaskAttemptInvalidated(
                TaskAttemptInvalidation::InputsChanged,
            ));
        }
        let context = request
            .context
            .as_ref()
            .map(|context| {
                self.assemble_context(session, token, context, now, request_id, operation_id)
            })
            .transpose()?;
        let context_reference = context
            .as_ref()
            .zip(request.context.as_ref())
            .map(|(packet, request)| {
                Ok::<_, ServiceError>(TaskWorkerContext {
                    request_sha256: rrd_contract::context_request_sha256(request)
                        .map_err(worker_contract)?,
                    packet_sha256: rrd_contract::context_packet_sha256(packet)
                        .map_err(worker_contract)?,
                    read: packet.read.clone(),
                })
            })
            .transpose()?;
        let packet = WorkerPacket {
            contract_version: ENDPOINT_WORKER_VERSION,
            task,
            selection_sha256: request.expected_selection_sha256.clone(),
            choice: selection.choice.clone(),
            configuration_sha256: request.expected_configuration_sha256.clone(),
            read: ContextReadStamp {
                runtime_manifest_sha256: read.manifest_id.clone(),
                runtime_cursor: read.commit_cursor,
                schema_revision: read.schema_revision,
                catalogue_revision: read.catalog_revision,
            },
            context,
        };
        let packet_bytes = serde_json::to_vec(&packet).map_err(contract_json)?;
        if packet_bytes.len() as u64 > rrd_contract::MAX_ROUTE_REQUEST_BYTES {
            return Err(ServiceError::ConfigurationLimit(
                "worker packet exceeds byte bound".into(),
            ));
        }
        let packet_sha256 = digest::sha256_hex(&packet_bytes);
        let chat = ModelChatRequest {
            model: binding.service_id.clone(),
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: worker.system_prompt.clone(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: String::from_utf8(packet_bytes)
                        .map_err(|_| ServiceError::Contract("worker packet encoding".into()))?,
                },
            ],
            settings: worker.generation.clone(),
            reasoning_effort: worker
                .wire_effort(effort.as_deref())
                .map_err(worker_contract)?
                .map(str::to_owned),
        };
        let prepared = PreparedModelChat::new(
            &configuration,
            binding_id,
            &chat,
            worker.maximum_response_bytes,
        )
        .map_err(endpoint_failure)?;
        let authority = WorkerAuthority {
            guards: vec![
                (
                    AttemptAuthorityGuard {
                        key: security_key,
                        expected: security_bytes,
                    },
                    TaskAttemptInvalidation::AuthorizationChanged,
                ),
                (
                    AttemptAuthorityGuard {
                        key: super::session::session_key(&self.instance, session),
                        expected: session_bytes,
                    },
                    TaskAttemptInvalidation::AuthorizationChanged,
                ),
                (
                    AttemptAuthorityGuard {
                        key: self.endpoint_configuration_key(),
                        expected: configuration_bytes.ok_or_else(|| {
                            ServiceError::Contract("endpoint configuration is absent".into())
                        })?,
                    },
                    TaskAttemptInvalidation::InputsChanged,
                ),
                (
                    selection_guard
                        .ok_or_else(|| ServiceError::Storage("selection disappeared".into()))?,
                    TaskAttemptInvalidation::InputsChanged,
                ),
            ],
        };
        let mut admission = self.worker_completion_admission(&authority, request, &read)?;
        if let Some(reason) = admission.invalidation {
            return Err(ServiceError::TaskAttemptInvalidated(reason));
        }
        if binding.credential.as_ref().is_some_and(|reference| {
            !self
                .endpoint_credentials
                .as_ref()
                .is_some_and(|credentials| credentials.contains(reference))
        }) {
            return Err(ServiceError::EndpointCallFailed {
                reason: "configured endpoint credential is unavailable",
                execution_may_continue: false,
            });
        }
        let lease = WorkerCapacityLease {
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            attempt_id: request.attempt_id.clone(),
            binding_id: binding_id.clone(),
        };
        admission.writes.push(self.worker_capacity_transition(
            &lease,
            Some((
                policy.maximum_concurrent_calls,
                binding.limits.max_concurrency,
            )),
        )?);
        let cancellation = EmbeddingCancellation::default();
        let control_context = EngineControlContext {
            now,
            session_id: session,
            action: "task-worker",
            request_id,
            operation_id,
        };
        let request_sha256 = operation_digest(request)?;
        let key = self.budget_key(&request.scope, &request.task_id);
        let reservation = {
            let mut active = self
                .active_task_cancellations
                .lock()
                .map_err(|_| ServiceError::Storage("task cancellation lock poisoned".into()))?;
            let reservation = self.reserve_controlled_task_attempt(
                &TaskAttemptAdmission {
                    started,
                    scope: &request.scope,
                    task_id: &request.task_id,
                    attempt_id: &request.attempt_id,
                    request_sha256: &request_sha256,
                    cancellation: &cancellation,
                },
                // Budget commits advance the read cursor themselves. Equivalent
                // work binds semantic inputs, not that incidental cursor or a
                // newly numbered selection receipt. Exact packet provenance is
                // still retained separately in the accepted output.
                || {
                    operation_digest(&(
                        "task-worker",
                        &packet.task,
                        &packet.configuration_sha256,
                        &packet.choice,
                        &request.context,
                        packet
                            .context
                            .as_ref()
                            .map(|context| (&context.items, context.truncated)),
                    ))
                },
                &admission,
                clock,
                &control_context,
            )?;
            active.insert(key.clone(), (request.attempt_id.clone(), cancellation));
            reservation
        };
        let _active = ActiveTaskCancellation {
            engine: self,
            key,
            attempt_id: request.attempt_id.clone(),
        };
        let result = (|| {
            let at = observe_required(clock, "worker-dispatch")?.observed_at_unix_ms;
            self.authorize_worker(session, token, request, at, request_id, operation_id)?;
            let current = self
                .storage
                .runtime()
                .read_stamp(&self.query_scope(&request.scope)?)?;
            if let Some(reason) = self
                .worker_completion_admission(&authority, request, &current)?
                .invalidation
            {
                return Err(ServiceError::TaskAttemptInvalidated(reason));
            }
            let observed = prepared
                .execute(
                    NetworkPolicy::Allow,
                    reservation.execution_control(),
                    |reference, _| {
                        self.endpoint_credentials
                            .as_ref()
                            .and_then(|credentials| credentials.resolve(reference))
                            .map(str::to_owned)
                            .ok_or_else(|| "configured endpoint credential is unavailable".into())
                    },
                )
                .map_err(endpoint_failure)?;
            let output = TaskWorkerOutput {
                contract_version: ENDPOINT_WORKER_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
                selection_attempt_id: request.selection_attempt_id.clone(),
                selection_sha256: request.expected_selection_sha256.clone(),
                task_sha256: request.expected_task_sha256.clone(),
                configuration_sha256: observed.configuration_sha256,
                binding_id: binding_id.clone(),
                service_id: CanonicalId::new(observed.service_id).map_err(worker_contract)?,
                packet_sha256: packet_sha256.clone(),
                request_sha256: observed.request_sha256,
                response_sha256: observed.response_sha256,
                content_sha256: observed.content_sha256,
                content: observed.content,
                read: packet.read.clone(),
                context: context_reference,
                usage: observed.usage,
                elapsed_ms: observed.elapsed_ms,
                http_status: observed.http_status,
            };
            output.validate().map_err(worker_contract)?;
            Ok(TaskAttemptOutput::Worker(Box::new(output)))
        })();
        let uncertain = matches!(
            &result,
            Err(ServiceError::EndpointCallFailed {
                execution_may_continue: true,
                ..
            })
        );
        match self.finish_controlled_task_attempt(
            reservation,
            result,
            |read| {
                let mut admission = self.worker_completion_admission(&authority, request, read)?;
                let at = observe_required(clock, "worker-completion")?.observed_at_unix_ms;
                if self
                    .authorize_worker(session, token, request, at, request_id, operation_id)
                    .is_err()
                {
                    admission.invalidation = Some(TaskAttemptInvalidation::AuthorizationChanged);
                }
                if !uncertain {
                    admission
                        .writes
                        .push(self.worker_capacity_transition(&lease, None)?);
                }
                Ok(admission)
            },
            clock,
            &control_context,
        )? {
            TaskAttemptOutput::Worker(output) => Ok(*output),
            _ => Err(ServiceError::Storage("unexpected task output kind".into())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn authorize_worker(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &TaskWorkerRequest,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<Vec<u8>> {
        let mut session_bytes = None;
        for action in ACTIONS {
            let (bytes, _) =
                self.authorize(session, token, action, now, request_id, operation_id)?;
            if session_bytes.as_ref().is_some_and(|prior| prior != &bytes) {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::AuthorizationChanged,
                ));
            }
            session_bytes = Some(bytes);
        }
        if request.context.is_some() {
            let (bytes, _, _) = self.authorize_resource_without_data_policy(
                session,
                token,
                SecurityAction::MemoryContextRead,
                &self.instance_resource(),
                now,
                request_id,
                operation_id,
            )?;
            if session_bytes.as_ref() != Some(&bytes) {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::AuthorizationChanged,
                ));
            }
        }
        session_bytes.ok_or(ServiceError::PermissionDenied)
    }

    fn worker_completion_admission(
        &self,
        authority: &WorkerAuthority,
        request: &TaskWorkerRequest,
        read: &ReadStamp,
    ) -> Result<TaskCompletionAdmission> {
        let mut admission = TaskCompletionAdmission::default();
        for (guard, reason) in &authority.guards {
            if self.storage.control().get(&guard.key)?.as_ref() != Some(&guard.expected) {
                admission.invalidation = Some(*reason);
            } else {
                admission.guards.push(AttemptAuthorityGuard {
                    key: guard.key.clone(),
                    expected: guard.expected.clone(),
                });
            }
        }
        let task = self.read_task_record_at(read, &worker_task_read(request))?;
        if EndpointSelectionTask::from_task(&task).map_or(true, |task| {
            task.task_sha256 != request.expected_task_sha256
        }) {
            admission.invalidation = Some(TaskAttemptInvalidation::InputsChanged);
        }
        Ok(admission)
    }
}

fn worker_task_read(request: &TaskWorkerRequest) -> ReadTask {
    ReadTask {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        max_storage_keys: request.max_storage_keys,
    }
}
fn worker_contract(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::Contract(error.to_string())
}
