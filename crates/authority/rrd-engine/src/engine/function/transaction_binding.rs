use super::super::security::AuditEvent;
use super::super::*;
use super::execution::{execute_definition, json_to_query_value};

impl RrdEngine {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine) fn apply_transaction_function_bindings(
        &self,
        catalogue: &FunctionCatalogue,
        request: &CommitTransaction,
        mut commit: RuntimeCommit,
        principal_id: Option<CanonicalId>,
        at: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<RuntimeCommit> {
        if catalogue.transaction_bindings.is_empty() {
            return Ok(commit);
        }
        for (index, mutation) in request.mutations.iter().enumerate() {
            for binding in catalogue
                .transaction_bindings
                .values()
                .filter(|binding| transaction_binding_matches(binding, mutation))
            {
                let definition = catalogue
                    .functions
                    .get(&binding.function_id)
                    .ok_or(ServiceError::FunctionNotFound)?;
                let input = transaction_binding_input(index, mutation, at)?;
                let binding_operation_id = format!(
                    "{operation_id}:transaction-function-binding:{}",
                    binding.binding_id
                );
                let input_sha256 =
                    digest::sha256_hex(&serde_json::to_vec(&input).map_err(contract_json)?);
                self.append_audit(AuditEvent {
                    at_unix_ms: at,
                    attempt: 1,
                    principal_id: principal_id.clone(),
                    action: SecurityAction::FunctionExecute,
                    resource: self.instance_resource(),
                    request_id: request_id.into(),
                    operation_id: binding_operation_id.clone(),
                    phase: AuditPhase::Authorized,
                    decision: AuditDecision::Allowed,
                    status_code: 100,
                    request_sha256: input_sha256.clone(),
                    response_sha256: digest::sha256_hex(b"rrd-audit-completion-pending"),
                })?;
                let executed = match execute_definition(definition, &input) {
                    Ok(executed) => executed,
                    Err(error) => {
                        self.append_audit(AuditEvent {
                            at_unix_ms: at,
                            attempt: 1,
                            principal_id: principal_id.clone(),
                            action: SecurityAction::FunctionExecute,
                            resource: self.instance_resource(),
                            request_id: request_id.into(),
                            operation_id: binding_operation_id,
                            phase: AuditPhase::Completed,
                            decision: AuditDecision::Failed,
                            status_code: 422,
                            request_sha256: input_sha256,
                            response_sha256: digest::sha256_hex(error.to_string().as_bytes()),
                        })?;
                        return Err(error);
                    }
                };
                apply_transaction_binding_effect(binding, executed.output.clone(), &mut commit)?;
                self.append_audit(AuditEvent {
                    at_unix_ms: at,
                    attempt: 1,
                    principal_id: principal_id.clone(),
                    action: SecurityAction::FunctionExecute,
                    resource: self.instance_resource(),
                    request_id: request_id.into(),
                    operation_id: binding_operation_id,
                    phase: AuditPhase::Completed,
                    decision: AuditDecision::Allowed,
                    status_code: 200,
                    request_sha256: input_sha256,
                    response_sha256: digest::sha256_hex(
                        &serde_json::to_vec(&executed.output).map_err(contract_json)?,
                    ),
                })?;
            }
        }
        commit.validate().map_err(core_contract)?;
        Ok(commit)
    }

    pub(in crate::engine) fn authorize_transaction_function_bindings(
        &self,
        state: &SessionState,
        catalogue: &FunctionCatalogue,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<()> {
        if catalogue.transaction_bindings.is_empty() {
            return Ok(());
        }
        match self.authorize_session_policy(state, SecurityAction::FunctionExecute, now) {
            Ok(()) => Ok(()),
            Err(error) => {
                let (decision, status_code) = super::super::invocation::audit_failure(&error);
                self.append_audit(AuditEvent {
                    at_unix_ms: now,
                    attempt: 1,
                    principal_id: state.principal_id.clone(),
                    action: SecurityAction::FunctionExecute,
                    resource: self.instance_resource(),
                    request_id: request_id.into(),
                    operation_id: format!(
                        "{operation_id}:transaction-function-binding-authorization"
                    ),
                    phase: AuditPhase::Completed,
                    decision,
                    status_code,
                    request_sha256: digest::sha256_hex(
                        &serde_json::to_vec(&(catalogue.revision, catalogue.sha256()))
                            .map_err(contract_json)?,
                    ),
                    response_sha256: digest::sha256_hex(error.to_string().as_bytes()),
                })?;
                Err(error)
            }
        }
    }
}

fn transaction_binding_matches(
    binding: &TransactionFunctionBinding,
    mutation: &TransactionMutation,
) -> bool {
    let (mutation_kind, kind) = mutation_coordinates(mutation);
    binding.mutation == mutation_kind
        && binding
            .kind
            .as_ref()
            .is_none_or(|expected| kind.is_some_and(|actual| actual == expected))
}

fn mutation_coordinates(
    mutation: &TransactionMutation,
) -> (TransactionMutationKind, Option<&CanonicalId>) {
    match mutation {
        TransactionMutation::AssertClaim { .. } => (TransactionMutationKind::AssertClaim, None),
        TransactionMutation::PutSchema { .. } => (TransactionMutationKind::PutSchema, None),
        TransactionMutation::PutRecord { reference, .. } => {
            (TransactionMutationKind::PutRecord, Some(&reference.kind))
        }
        TransactionMutation::PutRelation { reference, .. } => {
            (TransactionMutationKind::PutRelation, Some(&reference.kind))
        }
        TransactionMutation::AppendEvent { kind, .. } => {
            (TransactionMutationKind::AppendEvent, Some(kind))
        }
        TransactionMutation::PutVector { reference, .. } => {
            (TransactionMutationKind::PutVector, Some(&reference.kind))
        }
        TransactionMutation::AppendSeriesSample { reference, .. } => (
            TransactionMutationKind::AppendSeriesSample,
            Some(&reference.kind),
        ),
        TransactionMutation::PutGeo { reference, .. } => {
            (TransactionMutationKind::PutGeo, Some(&reference.kind))
        }
        TransactionMutation::PublishObjectReference { reference, .. } => (
            TransactionMutationKind::PublishObjectReference,
            Some(&reference.kind),
        ),
        TransactionMutation::RetireData { .. } => (TransactionMutationKind::RetireData, None),
    }
}

fn transaction_binding_input(
    index: usize,
    mutation: &TransactionMutation,
    at: u64,
) -> Result<QueryValue> {
    let mutation_value = serde_json::to_value(mutation).map_err(contract_json)?;
    Ok(QueryValue::Map(BTreeMap::from([
        ("at_unix_ms".into(), QueryValue::Unsigned(at)),
        (
            "mutation_index".into(),
            QueryValue::Unsigned(index.try_into().map_err(|_| {
                ServiceError::Contract(
                    "transaction function binding mutation index overflow".into(),
                )
            })?),
        ),
        ("mutation".into(), json_to_query_value(mutation_value)?),
    ])))
}

fn apply_transaction_binding_effect(
    binding: &TransactionFunctionBinding,
    output: QueryValue,
    commit: &mut RuntimeCommit,
) -> Result<()> {
    match &binding.effect {
        TransactionFunctionEffect::RequireTrue if output == QueryValue::Bool(true) => Ok(()),
        TransactionFunctionEffect::RequireTrue => Err(ServiceError::Function(format!(
            "transaction function binding {} rejected the transaction",
            binding.binding_id
        ))),
        TransactionFunctionEffect::AppendEvent { kind } => {
            let QueryValue::Map(properties) = output else {
                return Err(ServiceError::Function(format!(
                    "transaction function binding {} append_event output must be an object",
                    binding.binding_id
                )));
            };
            commit.mutations.push(RuntimeMutation::Event {
                event: RuntimeEvent {
                    kind: RuntimeType::new(kind.as_str()).map_err(core_contract)?,
                    subject: None,
                    properties: runtime_properties(&properties)?,
                },
            });
            Ok(())
        }
    }
}
