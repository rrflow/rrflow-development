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
    TaskWorkerOutput, TaskWorkerRequest, TaskWorkerSourceProvenance, TaskWorkerSourceRequest,
    ENDPOINT_WORKER_VERSION, TASK_WORKER_SOURCE_VERSION,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<WorkerSourcePacket>,
}

#[derive(Clone, Serialize)]
struct WorkerSourcePacket {
    content: String,
    provenance: TaskWorkerSourceProvenance,
}

struct WorkerSourceCapture {
    packet: WorkerSourcePacket,
    head_key: String,
    head_bytes: Vec<u8>,
}

impl WorkerSourceCapture {
    fn provenance(&self) -> &TaskWorkerSourceProvenance {
        &self.packet.provenance
    }
}

struct WorkerAuthority {
    guards: Vec<(AttemptAuthorityGuard, TaskAttemptInvalidation)>,
}

impl RrdEngine {
    pub(in crate::engine) fn authorize_worker_source_readback(
        &self,
        session: &CorrelationId,
        token: &CorrelationId,
        source: &TaskWorkerSourceProvenance,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<()> {
        let resource =
            worker_source_resource(&self.instance, &source.project_id, &source.entry_identity)?;
        self.authorize_resource_without_data_policy(
            session,
            token,
            SecurityAction::ProjectSourceRead,
            &resource,
            now,
            request_id,
            operation_id,
        )?;
        self.authorize_resource_without_data_policy(
            session,
            token,
            SecurityAction::MemoryContextRead,
            &self.instance_resource(),
            now,
            request_id,
            operation_id,
        )?;

        let configuration = self.load_endpoint_configuration()?.1;
        if !configuration
            .profile
            .as_ref()
            .and_then(|profile| profile.worker_policy.as_ref())
            .is_some_and(|policy| policy.allow_context_export && policy.allow_source_export)
        {
            return Err(ServiceError::PermissionDenied);
        }

        let installed = self.attunement_installed_record()?;
        let (profile, profile_sha256) = super::installation::DefaultInstallProfile::load(
            installed.installation.profile_id.as_str(),
        )?;
        if profile_sha256 != installed.installation.profile_sha256
            || profile_sha256 != source.profile_sha256
            || installed.installation.target.instance_id != self.instance
            || installed.installation.target.project_id != source.project_id
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let root = self
            .installed_project_root
            .as_ref()
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        let limits = &profile.inventory_policy.limits;
        let accepted = super::attunement::load_accepted_source(
            self,
            &source.project_id,
            limits.output_bytes,
            limits.entries,
        )?;
        let entry = accepted
            .proposal
            .entries()
            .iter()
            .find(|entry| entry.identity == source.entry_identity)
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        if digest::sha256_hex(entry.path.as_bytes()) != source.path_sha256 {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        super::installation::validate_selected_inventory_path(
            root,
            &profile,
            std::path::Path::new(&entry.path),
            limits.file_bytes,
            &|| Ok(()),
        )
    }

    fn capture_worker_source(
        &self,
        request: &TaskWorkerSourceRequest,
        maximum_input_bytes: u64,
        clock: &dyn ClockSource,
        check: &dyn Fn() -> Result<()>,
    ) -> Result<WorkerSourceCapture> {
        let installed = self.attunement_installed_record()?;
        let (profile, profile_sha256) = super::installation::DefaultInstallProfile::load(
            installed.installation.profile_id.as_str(),
        )?;
        if profile_sha256 != installed.installation.profile_sha256
            || installed.installation.target.instance_id != self.instance
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let root = self
            .installed_project_root
            .as_ref()
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        let limits = &profile.inventory_policy.limits;
        let accepted = super::attunement::load_accepted_source(
            self,
            &installed.installation.target.project_id,
            limits.output_bytes,
            limits.entries,
        )?;
        let proposal = &accepted.proposal;
        let scope = proposal.scope();
        if accepted.project_id != installed.installation.target.project_id
            || scope.project != installed.installation.target.project_id.as_str()
            || scope.estate != installed.installation.target.estate_id.as_str()
            || scope.root != profile.inventory_policy.logical_root.as_str()
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let entry = proposal
            .entries()
            .iter()
            .find(|entry| entry.identity == request.entry_identity)
            .ok_or(ServiceError::ProjectBindingMismatch)?;
        let rrd_attunement::inventory::EntryKind::File {
            observation,
            content_sha256,
        } = &entry.kind
        else {
            return Err(ServiceError::ProjectBindingMismatch);
        };
        let read_limit = request
            .maximum_input_bytes
            .min(maximum_input_bytes)
            .min(limits.file_bytes);
        if content_sha256 != &request.expected_content_sha256
            || observation.bytes > read_limit
            || observation.classification_sha256
                != digest::sha256_hex(b"rrflow-content-only-v1:unclassified")
        {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        check()?;
        super::installation::validate_selected_inventory_path(
            root,
            &profile,
            std::path::Path::new(&entry.path),
            read_limit,
            check,
        )?;
        let (bytes, actual_digest) = super::installation::read_admitted_inventory_file(
            root,
            std::path::Path::new(&entry.path),
            observation,
            read_limit,
            limits.elapsed_ms,
            check,
        )?;
        if actual_digest != *content_sha256 {
            return Err(ServiceError::ProjectBindingMismatch);
        }
        let content = String::from_utf8(bytes)
            .map_err(|_| ServiceError::Contract("selected source is not UTF-8".into()))?;
        let observed_at_unix_ms =
            observe_required(clock, "worker-source-observed")?.observed_at_unix_ms;
        let head_read = context_read_stamp(&accepted.read);
        let path_sha256 = digest::sha256_hex(entry.path.as_bytes());
        let provenance = TaskWorkerSourceProvenance {
            contract_version: TASK_WORKER_SOURCE_VERSION,
            captured_snapshot_sha256: accepted.proposal_sha256.clone(),
            captured_read: head_read.clone(),
            project_id: installed.installation.target.project_id.clone(),
            entry_identity: entry.identity.clone(),
            path_sha256,
            content_sha256: actual_digest,
            policy_sha256: proposal.revisions().policy_sha256.clone(),
            profile_sha256,
            captured_at_unix_ms: observed_at_unix_ms,
            validated_snapshot_sha256: proposal.proposal_sha256().to_owned(),
            validated_read: head_read,
            validated_at_unix_ms: observed_at_unix_ms,
        };
        provenance.validate().map_err(worker_contract)?;
        Ok(WorkerSourceCapture {
            packet: WorkerSourcePacket {
                content,
                provenance,
            },
            head_key: super::attunement::source_head_key(&self.instance),
            head_bytes: accepted.head_bytes,
        })
    }

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
        if request.source.is_some() && !policy.allow_source_export {
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
        let source_capture = request
            .source
            .as_ref()
            .map(|source| {
                self.capture_worker_source(
                    source,
                    binding.limits.max_input_bytes,
                    clock,
                    &|| Ok(()),
                )
            })
            .transpose()?;
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
            source: source_capture
                .as_ref()
                .map(|capture| &capture.packet)
                .cloned(),
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
                || equivalent_worker_work_digest(&packet, request, source_capture.as_ref()),
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
            if let (Some(source_request), Some(captured)) =
                (request.source.as_ref(), source_capture.as_ref())
            {
                let current = self.capture_worker_source(
                    source_request,
                    binding.limits.max_input_bytes,
                    clock,
                    &|| Ok(()),
                )?;
                if !equivalent_worker_source(captured.provenance(), current.provenance()) {
                    return Err(ServiceError::TaskAttemptInvalidated(
                        TaskAttemptInvalidation::InputsChanged,
                    ));
                }
            }
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
                source: source_capture
                    .as_ref()
                    .map(|capture| capture.packet.provenance.clone()),
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
                let currently_authorized = self
                    .authorize_worker(session, token, request, at, request_id, operation_id)
                    .is_ok();
                if !currently_authorized {
                    admission.invalidation = Some(TaskAttemptInvalidation::AuthorizationChanged);
                }
                if currently_authorized {
                    if let (Some(source_request), Some(captured)) =
                        (request.source.as_ref(), source_capture.as_ref())
                    {
                        match self.capture_worker_source(
                            source_request,
                            binding.limits.max_input_bytes,
                            clock,
                            &|| Ok(()),
                        ) {
                            Ok(current)
                                if equivalent_worker_source(
                                    captured.provenance(),
                                    current.provenance(),
                                ) =>
                            {
                                admission.guards.push(AttemptAuthorityGuard {
                                    key: current.head_key,
                                    expected: current.head_bytes,
                                });
                                let mut provenance = captured.packet.provenance.clone();
                                provenance.validated_snapshot_sha256 =
                                    current.packet.provenance.captured_snapshot_sha256;
                                provenance.validated_read = current.packet.provenance.captured_read;
                                provenance.validated_at_unix_ms =
                                    current.packet.provenance.captured_at_unix_ms;
                                admission.worker_source = Some(provenance);
                            }
                            Ok(_) => {
                                admission.invalidation =
                                    Some(TaskAttemptInvalidation::InputsChanged);
                            }
                            Err(ServiceError::ProjectBindingMismatch)
                            | Err(ServiceError::Contract(_))
                            | Err(ServiceError::ConfigurationLimit(_)) => {
                                admission.invalidation =
                                    Some(TaskAttemptInvalidation::InputsChanged);
                            }
                            Err(error) => return Err(error),
                        }
                    }
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
        if let Some(source) = &request.source {
            let installed = self.attunement_installed_record()?;
            let resource = worker_source_resource(
                &self.instance,
                &installed.installation.target.project_id,
                &source.entry_identity,
            )?;
            let (bytes, _, _) = self.authorize_resource_without_data_policy(
                session,
                token,
                SecurityAction::ProjectSourceRead,
                &resource,
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
        if request.context.is_some() || request.source.is_some() {
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

fn context_read_stamp(read: &ReadStamp) -> ContextReadStamp {
    ContextReadStamp {
        runtime_manifest_sha256: read.manifest_id.clone(),
        runtime_cursor: read.commit_cursor,
        schema_revision: read.schema_revision,
        catalogue_revision: read.catalog_revision,
    }
}

fn worker_source_resource(
    instance: &CanonicalId,
    project: &CanonicalId,
    entry_identity: &str,
) -> Result<ResourcePath> {
    Ok(ResourcePath {
        segments: vec![
            ResourceId::new(ResourceKind::Instance, instance.as_str().to_owned())
                .map_err(worker_contract)?,
            ResourceId::new(ResourceKind::Project, project.as_str().to_owned())
                .map_err(worker_contract)?,
            ResourceId::new(ResourceKind::Record, entry_identity.to_owned())
                .map_err(worker_contract)?,
        ],
    })
}

fn equivalent_worker_source(
    captured: &TaskWorkerSourceProvenance,
    validated: &TaskWorkerSourceProvenance,
) -> bool {
    captured.project_id == validated.project_id
        && captured.entry_identity == validated.entry_identity
        && captured.path_sha256 == validated.path_sha256
        && captured.content_sha256 == validated.content_sha256
        && captured.profile_sha256 == validated.profile_sha256
}

fn equivalent_worker_work_digest(
    packet: &WorkerPacket,
    request: &TaskWorkerRequest,
    source: Option<&WorkerSourceCapture>,
) -> Result<String> {
    let context_identity = packet
        .context
        .as_ref()
        .map(|context| (&context.items, context.truncated));
    if let Some(source) = source {
        let provenance = source.provenance();
        operation_digest(&(
            "task-worker-source-v1",
            &packet.task,
            &packet.configuration_sha256,
            &packet.choice,
            &request.context,
            (
                &provenance.project_id,
                &provenance.entry_identity,
                &provenance.path_sha256,
                &provenance.content_sha256,
                &provenance.profile_sha256,
            ),
            context_identity,
        ))
    } else {
        // Preserve the exact legacy serialization and repetition identity for
        // no-source worker calls. Existing budget records use this digest.
        operation_digest(&(
            "task-worker",
            &packet.task,
            &packet.configuration_sha256,
            &packet.choice,
            &request.context,
            context_identity,
        ))
    }
}

fn worker_contract(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::Contract(error.to_string())
}

#[cfg(test)]
mod legacy_fingerprint_tests {
    use super::*;

    #[test]
    fn no_source_worker_fingerprint_keeps_pre_source_tuple_bytes() {
        let packet = WorkerPacket {
            contract_version: ENDPOINT_WORKER_VERSION,
            task: EndpointSelectionTask {
                scope: "instance:test".into(),
                task_id: CanonicalId::new("task-a").unwrap(),
                title: "Review".into(),
                goal: "Inspect".into(),
                task_sha256: "1".repeat(64),
                known_at_cursor: 7,
            },
            selection_sha256: "2".repeat(64),
            choice: EndpointSelectionChoice::Select {
                binding_id: CanonicalId::new("binding-a").unwrap(),
                effort: None,
            },
            configuration_sha256: "3".repeat(64),
            read: ContextReadStamp {
                runtime_manifest_sha256: "4".repeat(64),
                runtime_cursor: 8,
                schema_revision: None,
                catalogue_revision: 1,
            },
            context: None,
            source: None,
        };
        let request = TaskWorkerRequest {
            scope: "instance:test".into(),
            task_id: CanonicalId::new("task-a").unwrap(),
            attempt_id: rrd_contract::CorrelationId::new("attempt-a").unwrap(),
            selection_attempt_id: rrd_contract::CorrelationId::new("selection-a").unwrap(),
            expected_selection_sha256: "2".repeat(64),
            expected_task_sha256: "1".repeat(64),
            expected_configuration_sha256: "3".repeat(64),
            max_storage_keys: 100,
            context: None,
            source: None,
        };
        let legacy = (
            "task-worker",
            &packet.task,
            &packet.configuration_sha256,
            &packet.choice,
            &request.context,
            None::<(&Vec<rrd_contract::ContextItem>, bool)>,
        );
        let bytes = serde_json::to_vec(&legacy).unwrap();
        let expected_bytes = br#"["task-worker",{"scope":"instance:test","task_id":"task-a","title":"Review","goal":"Inspect","task_sha256":"1111111111111111111111111111111111111111111111111111111111111111","known_at_cursor":7},"3333333333333333333333333333333333333333333333333333333333333333",{"kind":"select","binding_id":"binding-a","effort":null},null,null]"#;
        assert_eq!(bytes, expected_bytes);
        assert_eq!(
            equivalent_worker_work_digest(&packet, &request, None).unwrap(),
            digest::sha256_hex(expected_bytes)
        );
    }

    #[test]
    fn source_repetition_fingerprint_ignores_global_policy_and_snapshot_refreshes() {
        let packet = WorkerPacket {
            contract_version: ENDPOINT_WORKER_VERSION,
            task: EndpointSelectionTask {
                scope: "instance:test".into(),
                task_id: CanonicalId::new("task-a").unwrap(),
                title: "Review".into(),
                goal: "Inspect".into(),
                task_sha256: "1".repeat(64),
                known_at_cursor: 7,
            },
            selection_sha256: "2".repeat(64),
            choice: EndpointSelectionChoice::Select {
                binding_id: CanonicalId::new("binding-a").unwrap(),
                effort: None,
            },
            configuration_sha256: "3".repeat(64),
            read: ContextReadStamp {
                runtime_manifest_sha256: "4".repeat(64),
                runtime_cursor: 8,
                schema_revision: None,
                catalogue_revision: 1,
            },
            context: None,
            source: None,
        };
        let request = TaskWorkerRequest {
            scope: "instance:test".into(),
            task_id: CanonicalId::new("task-a").unwrap(),
            attempt_id: rrd_contract::CorrelationId::new("attempt-a").unwrap(),
            selection_attempt_id: rrd_contract::CorrelationId::new("selection-a").unwrap(),
            expected_selection_sha256: "2".repeat(64),
            expected_task_sha256: "1".repeat(64),
            expected_configuration_sha256: "3".repeat(64),
            max_storage_keys: 100,
            context: None,
            source: None,
        };
        let capture = |snapshot: &str, policy: &str, at| WorkerSourceCapture {
            packet: WorkerSourcePacket {
                content: "selected source bytes".into(),
                provenance: TaskWorkerSourceProvenance {
                    contract_version: TASK_WORKER_SOURCE_VERSION,
                    captured_snapshot_sha256: snapshot.into(),
                    captured_read: ContextReadStamp {
                        runtime_manifest_sha256: "5".repeat(64),
                        runtime_cursor: 12,
                        schema_revision: None,
                        catalogue_revision: 2,
                    },
                    project_id: CanonicalId::new("project-a").unwrap(),
                    entry_identity: "6".repeat(64),
                    path_sha256: "7".repeat(64),
                    content_sha256: digest::sha256_hex(b"selected source bytes"),
                    policy_sha256: policy.into(),
                    profile_sha256: "8".repeat(64),
                    captured_at_unix_ms: at,
                    validated_snapshot_sha256: "9".repeat(64),
                    validated_read: ContextReadStamp {
                        runtime_manifest_sha256: "a".repeat(64),
                        runtime_cursor: 12,
                        schema_revision: None,
                        catalogue_revision: 2,
                    },
                    validated_at_unix_ms: at + 1,
                },
            },
            head_key: "head".into(),
            head_bytes: vec![1],
        };
        let before = capture(&"b".repeat(64), &"c".repeat(64), 10);
        let after = capture(&"d".repeat(64), &"e".repeat(64), 20);
        assert_eq!(
            equivalent_worker_work_digest(&packet, &request, Some(&before)).unwrap(),
            equivalent_worker_work_digest(&packet, &request, Some(&after)).unwrap()
        );
    }
}
