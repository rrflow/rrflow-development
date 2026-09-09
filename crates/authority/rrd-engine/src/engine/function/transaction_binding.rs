use super::super::security::AuditEvent;
use super::super::*;
use super::execution::{
    encode_receipt_record, execute_definition, invocation_id, json_to_query_value,
};

pub(in crate::engine) struct PreparedTransactionFunctions {
    pub commit: RuntimeCommit,
    pub receipts: Vec<FunctionInvocationReceipt>,
}

impl RrdEngine {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine) fn prepare_transaction_function_bindings(
        &self,
        catalogue: &FunctionCatalogue,
        request: &CommitTransaction,
        transaction_id: &CorrelationId,
        mut commit: RuntimeCommit,
        principal_id: Option<CanonicalId>,
        at: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<PreparedTransactionFunctions> {
        let mut receipts = Vec::new();
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
                let artifact = catalogue
                    .artifacts
                    .get(definition.runtime.artifact_sha256())
                    .ok_or_else(|| ServiceError::Storage("function artifact is missing".into()))?;
                super::runtime_profile::validate_runtime_identity(&definition.runtime)?;
                let input = transaction_binding_input(index, mutation, at)?;
                let binding_operation_id = format!(
                    "{operation_id}:transaction-function-binding:{}",
                    binding.binding_id
                );
                let input_bytes = serde_json::to_vec(&input).map_err(contract_json)?;
                let input_sha256 = digest::sha256_hex(&input_bytes);
                let content = artifact
                    .decoded_content()
                    .map_err(|error| ServiceError::Contract(error.to_string()))?;
                let executed = match execute_definition(definition, &content, &input) {
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
                let output_bytes = serde_json::to_vec(&executed.output).map_err(contract_json)?;
                let output_sha256 = digest::sha256_hex(&output_bytes);
                let proposal = apply_transaction_binding_effect(
                    binding,
                    executed.output.clone(),
                    &mut commit,
                    &output_sha256,
                )?;
                let mutation_index: u32 = index.try_into().map_err(|_| {
                    ServiceError::Contract(
                        "transaction function binding mutation index exceeds u32".into(),
                    )
                })?;
                let invocation = invocation_id(&[
                    transaction_id.as_str(),
                    request.operation_sha256.as_str(),
                    binding.binding_id.as_str(),
                    &mutation_index.to_string(),
                    &input_sha256,
                ])?;
                receipts.push(FunctionInvocationReceipt {
                    contract_version: rrd_contract::FUNCTION_CONTRACT_VERSION,
                    invocation_id: invocation,
                    catalogue_revision: catalogue.revision,
                    catalogue_sha256: catalogue.sha256(),
                    function_id: definition.function_id.clone(),
                    function_definition_sha256: definition.sha256(),
                    binding_id: Some(binding.binding_id.clone()),
                    mutation_index: Some(mutation_index),
                    runtime: definition.runtime.kind(),
                    artifact_sha256: definition.runtime.artifact_sha256().into(),
                    runtime_profile: definition.runtime.runtime_profile().clone(),
                    runtime_build_sha256: definition.runtime.runtime_build_sha256().into(),
                    input_schema_sha256: definition.input_schema_sha256.clone(),
                    output_schema_sha256: definition.output_schema_sha256.clone(),
                    input_sha256,
                    output_sha256,
                    output: executed.output,
                    proposal,
                    runtime_commit_sha256: None,
                    limits: definition.limits.clone(),
                    attempts: 1,
                    interrupts_consumed: executed.interrupts_consumed,
                    fuel_consumed: executed.fuel_consumed,
                    receipt_sha256: String::new(),
                });
            }
        }
        commit.validate().map_err(core_contract)?;
        let commit_sha256 = commit.digest();
        let receipts = receipts
            .into_iter()
            .map(|mut receipt| {
                receipt.runtime_commit_sha256 = Some(commit_sha256.clone());
                receipt
                    .seal()
                    .map_err(|error| ServiceError::Contract(error.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(PreparedTransactionFunctions { commit, receipts })
    }

    pub(in crate::engine) fn apply_prepared_transaction_function_receipts(
        &self,
        catalogue: &FunctionCatalogue,
        request: &CommitTransaction,
        transaction_id: &CorrelationId,
        mut commit: RuntimeCommit,
        receipts: &[FunctionInvocationReceipt],
        at: u64,
    ) -> Result<RuntimeCommit> {
        let mut receipt_index = 0_usize;
        for (index, mutation) in request.mutations.iter().enumerate() {
            for binding in catalogue
                .transaction_bindings
                .values()
                .filter(|binding| transaction_binding_matches(binding, mutation))
            {
                let receipt = receipts.get(receipt_index).ok_or_else(|| {
                    ServiceError::Contract(
                        "prepared transaction is missing a function receipt".into(),
                    )
                })?;
                receipt_index += 1;
                receipt
                    .validate()
                    .map_err(|error| ServiceError::Contract(error.to_string()))?;
                let definition = catalogue
                    .functions
                    .get(&binding.function_id)
                    .ok_or(ServiceError::FunctionNotFound)?;
                let input = transaction_binding_input(index, mutation, at)?;
                let input_sha256 =
                    digest::sha256_hex(&serde_json::to_vec(&input).map_err(contract_json)?);
                definition
                    .input_schema
                    .validate_value(&input)
                    .map_err(|error| ServiceError::Contract(error.to_string()))?;
                let mutation_index: u32 = index.try_into().map_err(|_| {
                    ServiceError::Contract(
                        "transaction function binding mutation index exceeds u32".into(),
                    )
                })?;
                let expected_invocation = invocation_id(&[
                    transaction_id.as_str(),
                    request.operation_sha256.as_str(),
                    binding.binding_id.as_str(),
                    &mutation_index.to_string(),
                    &input_sha256,
                ])?;
                if receipt.invocation_id != expected_invocation
                    || receipt.catalogue_revision != catalogue.revision
                    || receipt.catalogue_sha256 != catalogue.sha256()
                    || receipt.function_id != definition.function_id
                    || receipt.function_definition_sha256 != definition.sha256()
                    || receipt.binding_id.as_ref() != Some(&binding.binding_id)
                    || receipt.mutation_index != Some(mutation_index)
                    || receipt.runtime != definition.runtime.kind()
                    || receipt.artifact_sha256 != definition.runtime.artifact_sha256()
                    || receipt.runtime_profile != *definition.runtime.runtime_profile()
                    || receipt.runtime_build_sha256 != definition.runtime.runtime_build_sha256()
                    || receipt.input_schema_sha256 != definition.input_schema_sha256
                    || receipt.output_schema_sha256 != definition.output_schema_sha256
                    || receipt.input_sha256 != input_sha256
                {
                    return Err(ServiceError::OperationDigestMismatch);
                }
                definition
                    .output_schema
                    .validate_value(&receipt.output)
                    .map_err(|error| ServiceError::Contract(error.to_string()))?;
                apply_prepared_proposal(binding, receipt, &mut commit)?;
            }
        }
        if receipt_index != receipts.len() {
            return Err(ServiceError::Contract(
                "prepared transaction contains unmatched function receipts".into(),
            ));
        }
        commit.validate().map_err(core_contract)?;
        let commit_sha256 = commit.digest();
        if receipts
            .iter()
            .any(|receipt| receipt.runtime_commit_sha256.as_deref() != Some(commit_sha256.as_str()))
        {
            return Err(ServiceError::OperationDigestMismatch);
        }
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

    pub(in crate::engine) fn require_committed_transaction_function_receipts(
        &self,
        receipts: &[FunctionInvocationReceipt],
    ) -> Result<()> {
        for receipt in receipts {
            let expected = encode_receipt_record(receipt)?;
            let stored = self
                .storage
                .function_catalogue()
                .invocation_receipt(self.instance.as_str(), receipt.invocation_id.as_str())?
                .ok_or(ServiceError::OperationDigestMismatch)?;
            if stored != expected {
                return Err(ServiceError::OperationDigestMismatch);
            }
        }
        Ok(())
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
    output_sha256: &str,
) -> Result<FunctionInvocationProposal> {
    match &binding.effect {
        TransactionFunctionEffect::RequireTrue if output == QueryValue::Bool(true) => {
            Ok(FunctionInvocationProposal::RequireTrue)
        }
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
            Ok(FunctionInvocationProposal::AppendEvent {
                kind: kind.clone(),
                properties_sha256: output_sha256.into(),
            })
        }
    }
}

fn apply_prepared_proposal(
    binding: &TransactionFunctionBinding,
    receipt: &FunctionInvocationReceipt,
    commit: &mut RuntimeCommit,
) -> Result<()> {
    match (&binding.effect, &receipt.proposal) {
        (TransactionFunctionEffect::RequireTrue, FunctionInvocationProposal::RequireTrue) => Ok(()),
        (
            TransactionFunctionEffect::AppendEvent { kind },
            FunctionInvocationProposal::AppendEvent {
                kind: receipt_kind,
                properties_sha256,
            },
        ) if kind == receipt_kind && properties_sha256 == &receipt.output_sha256 => {
            let QueryValue::Map(properties) = &receipt.output else {
                return Err(ServiceError::OperationDigestMismatch);
            };
            commit.mutations.push(RuntimeMutation::Event {
                event: RuntimeEvent {
                    kind: RuntimeType::new(kind.as_str()).map_err(core_contract)?,
                    subject: None,
                    properties: runtime_properties(properties)?,
                },
            });
            Ok(())
        }
        _ => Err(ServiceError::OperationDigestMismatch),
    }
}
