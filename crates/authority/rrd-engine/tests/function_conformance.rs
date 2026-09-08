use rrd_contract::{
    transaction_operation_sha256, BeginTransaction, CanonicalId, CommitTransaction, CorrelationId,
    CreateSession, ExecuteFunction, FunctionCatalogue, QueryValue, ReplaceFunctionCatalogue,
    RequestContext, SessionLease, SessionLimits, TransactionMutation,
};
use rrd_engine::RrdEngine;
use serde::Deserialize;

const TOKEN_KEY: [u8; 32] = [31; 32];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionConformanceFixture {
    catalogue: FunctionCatalogue,
    execute: ExecuteFunction,
    mutation: TransactionMutation,
    expected_output: QueryValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProfileEvidence {
    catalogue_sha256: String,
    output: QueryValue,
    runtime_commit_sha256: String,
    first_runtime_cursor: u64,
    last_runtime_cursor: u64,
    claim_mutation_count: u64,
}

fn fixture() -> FunctionConformanceFixture {
    serde_json::from_str(include_str!(
        "../../../../fixtures/rrd-function-conformance-v1.json"
    ))
    .unwrap()
}

fn id(value: &str) -> CorrelationId {
    CorrelationId::new(value).unwrap()
}

fn instance() -> CanonicalId {
    CanonicalId::new("function-conformance").unwrap()
}

fn install_and_execute(
    engine: &RrdEngine,
    fixture: &FunctionConformanceFixture,
) -> (SessionLease, ProfileEvidence) {
    let lease = engine
        .create_session(
            &CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 60_000,
                    absolute_timeout_ms: 300_000,
                    max_open_transactions: 4,
                },
            },
            &id("function-conformance-session"),
            1_000,
            "request-function-conformance-session",
            "operation-function-conformance-session",
        )
        .unwrap();
    let installed = engine
        .replace_function_catalogue(
            &lease.session_id,
            &lease.token,
            &ReplaceFunctionCatalogue {
                expected_revision: 0,
                catalogue: fixture.catalogue.clone(),
            },
            1_010,
            "request-function-catalogue-install",
            "operation-function-catalogue-install",
        )
        .unwrap();
    assert_eq!(installed, fixture.catalogue);
    assert_eq!(
        engine
            .function_catalogue(
                &lease.session_id,
                &lease.token,
                1_020,
                "request-function-catalogue-read",
                "operation-function-catalogue-read",
            )
            .unwrap(),
        fixture.catalogue
    );
    let execution = engine
        .execute_function(
            &lease.session_id,
            &lease.token,
            &fixture.execute,
            1_030,
            "request-function-execute",
            "operation-function-execute",
        )
        .unwrap();
    assert_eq!(execution.output, fixture.expected_output);

    let transaction = engine
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &BeginTransaction {
                scope: CanonicalId::new("claims").unwrap(),
                timeout_ms: 30_000,
            },
            &RequestContext {
                request_id: id("request-function-transaction-begin"),
                operation_id: id("operation-function-transaction-begin"),
                idempotency_key: Some(id("function-transaction-begin")),
                deadline_unix_ms: None,
            },
            1_040,
        )
        .unwrap();
    let mutations = vec![fixture.mutation.clone()];
    let request = CommitTransaction {
        operation_sha256: transaction_operation_sha256(&mutations),
        mutations,
    };
    let receipt = engine
        .commit_transaction(
            &lease.session_id,
            &lease.token,
            &transaction.transaction_id,
            &id("function-transaction-commit"),
            &request,
            1_050,
            "request-function-transaction-commit",
            "operation-function-transaction-commit",
        )
        .unwrap();
    assert_eq!(receipt.mutation_count, 1);
    assert!(!receipt.idempotent_replay);

    (
        lease,
        ProfileEvidence {
            catalogue_sha256: installed.sha256(),
            output: execution.output,
            runtime_commit_sha256: receipt.runtime_commit_sha256.unwrap(),
            first_runtime_cursor: receipt.first_runtime_cursor.unwrap(),
            last_runtime_cursor: receipt.last_runtime_cursor.unwrap(),
            claim_mutation_count: receipt.claim_mutation_count.unwrap(),
        },
    )
}

#[test]
fn canonical_function_corpus_matches_rrflow_mx_and_rrflow_kv_and_reopens() {
    let fixture = fixture();
    fixture.catalogue.validate().unwrap();
    fixture.execute.validate().unwrap();

    let mx = RrdEngine::rrflow_mx(instance(), TOKEN_KEY);
    let (_mx_lease, mx_evidence) = install_and_execute(&mx, &fixture);

    let root = tempfile::tempdir().unwrap();
    let kv = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    let (kv_lease, kv_evidence) = install_and_execute(&kv, &fixture);
    assert_eq!(mx_evidence, kv_evidence);

    drop(kv);
    let reopened = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    let catalogue = reopened
        .function_catalogue(
            &kv_lease.session_id,
            &kv_lease.token,
            1_060,
            "request-function-catalogue-reopen",
            "operation-function-catalogue-reopen",
        )
        .unwrap();
    assert_eq!(catalogue, fixture.catalogue);
    let execution = reopened
        .execute_function(
            &kv_lease.session_id,
            &kv_lease.token,
            &fixture.execute,
            1_070,
            "request-function-execute-reopen",
            "operation-function-execute-reopen",
        )
        .unwrap();
    assert_eq!(execution.output, fixture.expected_output);
    assert_eq!(execution.catalogue_sha256, kv_evidence.catalogue_sha256);
}
