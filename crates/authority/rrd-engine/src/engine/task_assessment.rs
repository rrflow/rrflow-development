//! One source-bound controller assessment under the existing task ledger.
//! Provider output remains an advisory report, never a task completion or effect.
use super::clock::{observe_required, ClockSource, HostSystemClock};
use super::endpoint_proposal::capacity::AssessmentCapacityLease;
use super::reasoning_budget::{
    ActiveTaskCancellation, AttemptAuthorityGuard, TaskAttemptAdmission, TaskAttemptOutput,
    TaskCompletionAdmission,
};
use super::*;
use rrd_contract::{
    ReadTask, ReadTaskAssessment, TaskAssessmentOutput, TaskAssessmentPacket, TaskAssessmentReply,
    TaskAssessmentRequest, TaskAssessmentView, TaskAttemptInvalidation, TASK_ASSESSMENT_VERSION,
};
use rrd_inference::endpoint_chat::{ChatMessage, ChatRole, ModelChatRequest, PreparedModelChat};
use rrd_inference::{EmbeddingCancellation, NetworkPolicy};

struct AssessmentAuthority {
    guards: Vec<(AttemptAuthorityGuard, TaskAttemptInvalidation)>,
}

impl RrdEngine {
    pub fn execute_task_assessment(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &TaskAssessmentRequest,
        request_id: &str,
        operation_id: &str,
    ) -> Result<TaskAssessmentOutput> {
        self.execute_task_assessment_with_clock(
            session,
            token,
            request,
            request_id,
            operation_id,
            &HostSystemClock,
        )
    }

    pub(super) fn execute_task_assessment_with_clock(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &TaskAssessmentRequest,
        request_id: &str,
        operation_id: &str,
        clock: &dyn ClockSource,
    ) -> Result<TaskAssessmentOutput> {
        let started = Instant::now();
        request.validate().map_err(assessment_contract)?;
        let now = observe_required(clock, "task-assessment-authorize")?.observed_at_unix_ms;
        let authority = rrd_security::SecurityRepository::new(&self.storage, self.instance.clone())
            .authority_snapshot()?
            .ok_or(ServiceError::PermissionDenied)?;
        let (security_key, security_bytes) = authority;
        let session_bytes = self.authorize_assessment_request(
            session,
            token,
            request,
            now,
            request_id,
            operation_id,
        )?;
        let (configuration_bytes, configuration) = self.load_endpoint_configuration()?;
        if configuration.sha256().map_err(assessment_contract)?
            != request.expected_configuration_sha256
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
            .assessment_policy
            .as_ref()
            .ok_or(ServiceError::PermissionDenied)?;
        if !policy.allow_network || !policy.allow_source_export || !policy.allow_context_export {
            return Err(ServiceError::PermissionDenied);
        }
        let selector = profile
            .selection_policy
            .as_ref()
            .ok_or_else(|| ServiceError::Contract("selector policy is absent".into()))?;
        let controller_id = &selector.controller_binding_id;
        let controller = profile
            .bindings
            .iter()
            .find(|binding| &binding.id == controller_id)
            .ok_or_else(|| ServiceError::Contract("assessment controller is absent".into()))?;
        let source_capture = self.capture_worker_source(
            &request.source,
            policy.maximum_input_bytes,
            clock,
            &|| Ok(()),
        )?;
        let read = self
            .storage
            .runtime()
            .read_stamp(&self.query_scope(&request.scope)?)?;
        let task = self.read_task_record_at(
            &read,
            &ReadTask {
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                max_storage_keys: request.max_storage_keys,
            },
        )?;
        let task =
            rrd_contract::EndpointSelectionTask::from_task(&task).map_err(assessment_contract)?;
        if task.task_sha256 != request.expected_task_sha256 {
            return Err(ServiceError::TaskAttemptInvalidated(
                TaskAttemptInvalidation::InputsChanged,
            ));
        }
        let packet = TaskAssessmentPacket {
            contract_version: TASK_ASSESSMENT_VERSION,
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            task_sha256: task.task_sha256.clone(),
            configuration_sha256: request.expected_configuration_sha256.clone(),
            descriptor: request.descriptor.clone(),
            practice_identity: request.practice_identity.clone(),
            practice_sha256: request.practice_sha256().map_err(assessment_contract)?,
            practice_text: request.practice_text.clone(),
            source_content: source_capture.packet.content.clone(),
            source: source_capture.packet.provenance.clone(),
            read: super::endpoint_worker::context_read_stamp(&read),
        };
        packet.validate().map_err(assessment_contract)?;
        let packet_bytes = serde_json::to_vec(&packet).map_err(contract_json)?;
        if packet_bytes.len() as u64 > policy.maximum_input_bytes {
            return Err(ServiceError::ConfigurationLimit(
                "assessment packet exceeds its configured input bound".into(),
            ));
        }
        let packet_sha256 = digest::sha256_hex(&packet_bytes);
        let chat = ModelChatRequest {
            model: controller.service_id.clone(),
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: policy.system_prompt.clone(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: String::from_utf8(packet_bytes)
                        .map_err(|_| ServiceError::Contract("assessment packet encoding".into()))?,
                },
            ],
            settings: policy.generation.clone(),
            reasoning_effort: None,
        };
        let prepared = PreparedModelChat::new(
            &configuration,
            controller_id,
            &chat,
            policy.maximum_response_bytes,
        )
        .map_err(super::endpoint_proposal::endpoint_failure)?;
        let request_sha256 = operation_digest(request)?;
        let capacity = AssessmentCapacityLease {
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            attempt_id: request.attempt_id.clone(),
            binding_id: controller_id.clone(),
        };
        let authority = AssessmentAuthority {
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
                        expected: session_bytes.clone(),
                    },
                    TaskAttemptInvalidation::AuthorizationChanged,
                ),
                (
                    AttemptAuthorityGuard {
                        key: self.endpoint_configuration_key(),
                        expected: configuration_bytes.clone().ok_or_else(|| {
                            ServiceError::Contract("endpoint configuration is absent".into())
                        })?,
                    },
                    TaskAttemptInvalidation::InputsChanged,
                ),
            ],
        };
        let mut admission = TaskCompletionAdmission {
            guards: authority
                .guards
                .iter()
                .map(|(guard, _)| guard.clone())
                .collect(),
            ..Default::default()
        };
        admission.writes.push(self.assessment_capacity_transition(
            &capacity,
            Some((
                selector.maximum_concurrent_proposals,
                controller.limits.max_concurrency,
            )),
        )?);
        if controller.credential.as_ref().is_some_and(|reference| {
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
        let cancellation = EmbeddingCancellation::default();
        let control_context = EngineControlContext {
            now,
            session_id: session,
            action: "task-assessment",
            request_id,
            operation_id,
        };
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
                || assessment_repetition_identity(&packet),
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
            let at = observe_required(clock, "task-assessment-dispatch")?.observed_at_unix_ms;
            self.authorize_assessment_request(
                session,
                token,
                request,
                at,
                request_id,
                operation_id,
            )?;
            let (_, current_configuration) = self.load_endpoint_configuration()?;
            if current_configuration
                .sha256()
                .map_err(assessment_contract)?
                != request.expected_configuration_sha256
            {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::InputsChanged,
                ));
            }
            let current_source = self.capture_worker_source(
                &request.source,
                policy.maximum_input_bytes,
                clock,
                &|| Ok(()),
            )?;
            if !super::endpoint_worker::equivalent_worker_source(
                source_capture.provenance(),
                current_source.provenance(),
            ) {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::InputsChanged,
                ));
            }
            let current_read = self
                .storage
                .runtime()
                .read_stamp(&self.query_scope(&request.scope)?)?;
            let current_task = self.read_task_record_at(
                &current_read,
                &ReadTask {
                    scope: request.scope.clone(),
                    task_id: request.task_id.clone(),
                    max_storage_keys: request.max_storage_keys,
                },
            )?;
            if rrd_contract::EndpointSelectionTask::from_task(&current_task)
                .map_or(true, |current| {
                    current.task_sha256 != request.expected_task_sha256
                })
            {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::InputsChanged,
                ));
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
                .map_err(super::endpoint_proposal::endpoint_failure)?;
            if observed.configuration_sha256 != request.expected_configuration_sha256
                || observed.service_id != controller.service_id.as_str()
            {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::InputsChanged,
                ));
            }
            let reply = TaskAssessmentReply::decode(
                observed.content.as_bytes(),
                &request.descriptor,
                &source_capture.packet.content,
                &request.practice_text,
            )
            .map_err(assessment_contract)?;
            let output = TaskAssessmentOutput {
                contract_version: TASK_ASSESSMENT_VERSION,
                scope: request.scope.clone(),
                task_id: request.task_id.clone(),
                attempt_id: request.attempt_id.clone(),
                task_sha256: task.task_sha256.clone(),
                configuration_sha256: observed.configuration_sha256,
                packet_sha256: packet_sha256.clone(),
                request_sha256: observed.request_sha256,
                response_sha256: observed.response_sha256,
                content_sha256: observed.content_sha256,
                controller_binding_id: controller_id.clone(),
                service_id: CanonicalId::new(observed.service_id).map_err(assessment_contract)?,
                descriptor: request.descriptor.clone(),
                practice_identity: request.practice_identity.clone(),
                practice_sha256: packet.practice_sha256.clone(),
                source_bytes: source_capture.packet.content.len() as u64,
                practice_bytes: request.practice_text.len() as u64,
                source: source_capture.packet.provenance.clone(),
                read: packet.read.clone(),
                decision: reply.decision,
                references: reply.references,
                usage: observed.usage,
                elapsed_ms: observed.elapsed_ms,
                http_status: observed.http_status,
            };
            output.validate().map_err(assessment_contract)?;
            Ok(TaskAttemptOutput::Assessment(Box::new(output)))
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
                let mut admission =
                    assessment_completion_admission(self, &authority, request, read)?;
                let at = observe_required(clock, "task-assessment-completion")?.observed_at_unix_ms;
                let session_now = self.authorize_assessment_request(
                    session,
                    token,
                    request,
                    at,
                    request_id,
                    operation_id,
                );
                if session_now.is_err() {
                    admission.invalidation = Some(TaskAttemptInvalidation::AuthorizationChanged);
                }
                if session_now.is_ok() {
                    match self.capture_worker_source(
                        &request.source,
                        policy.maximum_input_bytes,
                        clock,
                        &|| Ok(()),
                    ) {
                        Ok(current)
                            if super::endpoint_worker::equivalent_worker_source(
                                source_capture.provenance(),
                                current.provenance(),
                            ) =>
                        {
                            admission.guards.push(AttemptAuthorityGuard {
                                key: current.head_key,
                                expected: current.head_bytes,
                            });
                            let mut provenance = source_capture.packet.provenance.clone();
                            provenance.validated_snapshot_sha256 =
                                current.packet.provenance.captured_snapshot_sha256;
                            provenance.validated_read = current.packet.provenance.captured_read;
                            provenance.validated_at_unix_ms =
                                current.packet.provenance.captured_at_unix_ms;
                            admission.assessment_source = Some(provenance);
                        }
                        Ok(_) => {
                            admission.invalidation = Some(TaskAttemptInvalidation::InputsChanged);
                        }
                        Err(ServiceError::ProjectBindingMismatch)
                        | Err(ServiceError::Contract(_))
                        | Err(ServiceError::ConfigurationLimit(_)) => {
                            admission.invalidation = Some(TaskAttemptInvalidation::InputsChanged);
                        }
                        Err(error) => return Err(error),
                    }
                }
                if !uncertain {
                    admission
                        .writes
                        .push(self.assessment_capacity_transition(&capacity, None)?);
                }
                Ok(admission)
            },
            clock,
            &control_context,
        )? {
            TaskAttemptOutput::Assessment(output) => Ok(*output),
            _ => Err(ServiceError::Storage(
                "unexpected assessment output kind".into(),
            )),
        }
    }

    fn authorize_assessment_request(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &TaskAssessmentRequest,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<Vec<u8>> {
        let mut session_bytes = None;
        for action in [
            SecurityAction::EndpointPropose,
            SecurityAction::TaskRead,
            SecurityAction::ReasoningWrite,
        ] {
            let (bytes, _) =
                self.authorize(session, token, action, now, request_id, operation_id)?;
            if session_bytes.as_ref().is_some_and(|prior| prior != &bytes) {
                return Err(ServiceError::TaskAttemptInvalidated(
                    TaskAttemptInvalidation::AuthorizationChanged,
                ));
            }
            session_bytes = Some(bytes);
        }
        let installed = self.attunement_installed_record()?;
        let source = super::endpoint_worker::worker_source_resource(
            &self.instance,
            &installed.installation.target.project_id,
            &request.source.entry_identity,
        )?;
        let (source_authority, _, _) = self.authorize_resource_without_data_policy(
            session,
            token,
            SecurityAction::ProjectSourceRead,
            &source,
            now,
            request_id,
            operation_id,
        )?;
        let (context_authority, _, _) = self.authorize_resource_without_data_policy(
            session,
            token,
            SecurityAction::MemoryContextRead,
            &self.instance_resource(),
            now,
            request_id,
            operation_id,
        )?;
        if session_bytes.as_ref() != Some(&source_authority)
            || session_bytes.as_ref() != Some(&context_authority)
        {
            return Err(ServiceError::TaskAttemptInvalidated(
                TaskAttemptInvalidation::AuthorizationChanged,
            ));
        }
        let configuration = self.load_endpoint_configuration()?.1;
        if !configuration
            .profile
            .as_ref()
            .and_then(|profile| profile.assessment_policy.as_ref())
            .is_some_and(|policy| {
                policy.allow_network && policy.allow_source_export && policy.allow_context_export
            })
        {
            return Err(ServiceError::PermissionDenied);
        }
        session_bytes.ok_or(ServiceError::PermissionDenied)
    }

    pub fn read_task_assessment(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        request: &ReadTaskAssessment,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<TaskAssessmentView> {
        request.validate().map_err(assessment_contract)?;
        self.authorize(
            session,
            token,
            SecurityAction::ReasoningRead,
            now,
            request_id,
            operation_id,
        )?;
        self.authorize(
            session,
            token,
            SecurityAction::TaskRead,
            now,
            request_id,
            operation_id,
        )?;
        self.query_scope(&request.scope)?;
        let view = self.read_task_assessment_record(request)?;
        if let Some(output) = &view.assessment {
            self.authorize_assessment_source_readback(
                session,
                token,
                &output.source,
                now,
                request_id,
                operation_id,
            )?;
        }
        view.validate().map_err(assessment_contract)?;
        Ok(view)
    }
}

fn assessment_completion_admission(
    engine: &RrdEngine,
    authority: &AssessmentAuthority,
    request: &TaskAssessmentRequest,
    read: &ReadStamp,
) -> Result<TaskCompletionAdmission> {
    let mut admission = TaskCompletionAdmission::default();
    for (guard, reason) in &authority.guards {
        if engine.storage.control().get(&guard.key)?.as_ref() != Some(&guard.expected) {
            admission.invalidation = Some(*reason);
        } else {
            admission.guards.push(AttemptAuthorityGuard {
                key: guard.key.clone(),
                expected: guard.expected.clone(),
            });
        }
    }
    let task = engine.read_task_record_at(
        read,
        &ReadTask {
            scope: request.scope.clone(),
            task_id: request.task_id.clone(),
            max_storage_keys: request.max_storage_keys,
        },
    )?;
    if rrd_contract::EndpointSelectionTask::from_task(&task).map_or(true, |task| {
        task.task_sha256 != request.expected_task_sha256
    }) {
        admission.invalidation = Some(TaskAttemptInvalidation::InputsChanged);
    }
    Ok(admission)
}

fn assessment_repetition_identity(packet: &TaskAssessmentPacket) -> Result<String> {
    let source = &packet.source;
    operation_digest(&(
        "task-assessment-v1",
        &packet.scope,
        &packet.task_id,
        &packet.task_sha256,
        &packet.configuration_sha256,
        &packet.descriptor,
        &packet.practice_identity,
        &packet.practice_sha256,
        (
            &source.project_id,
            &source.entry_identity,
            &source.path_sha256,
            &source.content_sha256,
            &source.profile_sha256,
        ),
    ))
}

fn assessment_contract(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::Contract(error.to_string())
}
