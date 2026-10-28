//! H-04q1: build, activate and retire are replay-safe. The same idempotency
//! key and request return the recorded operation without a second effect;
//! another request under that key conflicts.

use super::*;
use rrd_contract::{BuildVectorQuantizationArtifactResult, SessionLease};

const DIMENSIONS: usize = 8;

fn scope() -> String {
    format!("instance:{}", instance())
}

fn values(seed: usize) -> Vec<f32> {
    (0..DIMENSIONS)
        .map(|dimension| ((seed * 13 + dimension * 5) % 41) as f32 / 20.0 - 1.0)
        .collect()
}

/// Six documents in `documents`, with no quantization yet.
fn fixture() -> (tempfile::TempDir, RrdEngine, SessionLease) {
    let root = tempfile::tempdir().unwrap();
    let engine = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    let lease = engine
        .create_session(
            &session_request(9_000, 8),
            &id("quantization-replay-session"),
            100,
            "request-quantization-replay-session",
            "operation-quantization-replay-session",
        )
        .unwrap();
    engine
        .ensure_vector_collection(
            &lease.session_id,
            &lease.token,
            &EnsureVectorCollection {
                scope: scope(),
                collection_id: CanonicalId::new("documents").unwrap(),
                vectors: vec![NamedVectorDefinition {
                    name: CanonicalId::new("body").unwrap(),
                    field: CanonicalId::new("body-embedding").unwrap(),
                    kind: VectorValueKind::Dense,
                    dimensions: DIMENSIONS as u32,
                    metric: VectorSearchMetric::Cosine,
                    embedding_model: None,
                    memory_tier: VectorMemoryTier::Cached,
                }],
            },
            &mutation_context(
                &id("quantization-replay-collection"),
                "request-quantization-replay-collection",
                "operation-quantization-replay-collection",
            ),
            200,
        )
        .unwrap();
    let mut mutations = vec![TransactionMutation::PutSchema {
        registry: vector_fixture_schema(
            "install quantization replay fixture",
            DataRecordSchema {
                properties: BTreeMap::from([(
                    "body".into(),
                    DataPropertySchema {
                        value_type: DataValueType::String,
                        required: false,
                    },
                )]),
                allow_additional_properties: true,
                ..DataRecordSchema::default()
            },
        ),
    }];
    for row in 0..6 {
        let id = format!("r{row}");
        mutations.push(document_mutation(&id, "replay fixture"));
        mutations.push(TransactionMutation::PutVector {
            reference: reference("embedding", &id),
            subject: reference("document", &id),
            collection_id: CanonicalId::new("documents").unwrap(),
            vector_name: CanonicalId::new("body").unwrap(),
            field: CanonicalId::new("body-embedding").unwrap(),
            valid_from: 1,
            valid_to: None,
            value: DataVectorValue::Dense {
                values: values(row),
            },
            provenance: None,
            properties: DataProperties::new(),
        });
    }
    commit_vectors(&engine, &lease, 300, mutations);
    (root, engine, lease)
}

fn build_request(seed: u64) -> BuildVectorQuantizationArtifact {
    BuildVectorQuantizationArtifact {
        scope: scope(),
        collection_id: CanonicalId::new("documents").unwrap(),
        vector_name: CanonicalId::new("body").unwrap(),
        method: VectorQuantizationMethod::TurboQuant {
            bits: VectorQuantizationBits::Bits4,
            seed,
        },
        filter_properties: Vec::new(),
        max_storage_keys: 100_000,
    }
}

fn context(key: &str) -> RequestContext {
    mutation_context(
        &id(key),
        &format!("request-{key}"),
        &format!("operation-{key}"),
    )
}

fn build(
    engine: &RrdEngine,
    lease: &SessionLease,
    key: &str,
    seed: u64,
    now: u64,
) -> crate::Result<BuildVectorQuantizationArtifactResult> {
    engine.build_vector_quantization_artifact(
        &lease.session_id,
        &lease.token,
        &build_request(seed),
        &context(key),
        now,
    )
}

fn lifecycle_revision(engine: &RrdEngine, lease: &SessionLease) -> (u64, usize) {
    let listed = engine
        .list_vector_quantization_artifacts(
            &lease.session_id,
            &lease.token,
            &ListVectorQuantizationArtifacts {
                scope: scope(),
                collection_id: None,
                vector_name: None,
                max_artifacts: 100,
            },
            900,
            "request-quantization-replay-list",
            "operation-quantization-replay-list",
        )
        .unwrap();
    (listed.lifecycle_revision, listed.artifacts.len())
}

fn activate(
    engine: &RrdEngine,
    lease: &SessionLease,
    key: &str,
    generation: u64,
) -> crate::Result<rrd_contract::ActivateVectorQuantizationArtifactResult> {
    engine.activate_vector_quantization_artifact(
        &lease.session_id,
        &lease.token,
        &ActivateVectorQuantizationArtifact {
            scope: scope(),
            artifact_id: CanonicalId::new("quant-turboquant-documents-body").unwrap(),
            generation,
        },
        &context(key),
        500,
    )
}

fn retire(
    engine: &RrdEngine,
    lease: &SessionLease,
    key: &str,
    generation: u64,
) -> crate::Result<rrd_contract::RetireVectorQuantizationArtifactResult> {
    engine.retire_vector_quantization_artifact(
        &lease.session_id,
        &lease.token,
        &RetireVectorQuantizationArtifact {
            scope: scope(),
            artifact_id: CanonicalId::new("quant-turboquant-documents-body").unwrap(),
            generation,
        },
        &context(key),
        600,
    )
}

#[test]
fn a_repeated_build_is_one_effect() {
    let (_root, engine, lease) = fixture();
    let first = build(&engine, &lease, "build-1", 29, 400).unwrap();
    assert!(!first.idempotent_replay);
    assert_eq!(first.artifact.generation, 1);
    let recorded = lifecycle_revision(&engine, &lease);

    let again = build(&engine, &lease, "build-1", 29, 410).unwrap();
    assert!(again.idempotent_replay);
    assert_eq!(again.artifact, first.artifact);
    assert_eq!(lifecycle_revision(&engine, &lease), recorded);
    // The replay read the catalogue under the request's budget and no vector.
    assert!(again.read_evidence.keys_examined > 0);
    assert!(again
        .read_evidence
        .paths
        .iter()
        .all(|path| path.path != ReadAccessPath::VectorVersions));

    let changed = build(&engine, &lease, "build-1", 30, 420).unwrap_err();
    assert_eq!(changed.kind(), crate::ServiceErrorKind::Conflict);
    assert_eq!(lifecycle_revision(&engine, &lease), recorded);

    let next = build(&engine, &lease, "build-2", 30, 430).unwrap();
    assert!(!next.idempotent_replay);
    assert_eq!(next.artifact.generation, 2);
}

#[test]
fn activate_and_retire_replay_their_recorded_operation() {
    let (_root, engine, lease) = fixture();
    build(&engine, &lease, "build-1", 29, 400).unwrap();

    let activated = activate(&engine, &lease, "activate-1", 1).unwrap();
    assert!(!activated.idempotent_replay);
    assert_eq!(
        activated.artifact.state,
        VectorQuantizationArtifactState::Active
    );
    let recorded = lifecycle_revision(&engine, &lease);
    let again = activate(&engine, &lease, "activate-1", 1).unwrap();
    assert!(again.idempotent_replay);
    assert_eq!(again.artifact, activated.artifact);
    assert_eq!(lifecycle_revision(&engine, &lease), recorded);

    let retired = retire(&engine, &lease, "retire-1", 1).unwrap();
    assert!(!retired.idempotent_replay);
    assert_eq!(
        retired.artifact.state,
        VectorQuantizationArtifactState::Retired
    );
    let recorded = lifecycle_revision(&engine, &lease);
    assert!(
        retire(&engine, &lease, "retire-1", 1)
            .unwrap()
            .idempotent_replay
    );
    // A replay reports the artifact as it is now.
    let late = activate(&engine, &lease, "activate-1", 1).unwrap();
    assert!(late.idempotent_replay);
    assert_eq!(
        late.artifact.state,
        VectorQuantizationArtifactState::Retired
    );
    assert_eq!(lifecycle_revision(&engine, &lease), recorded);

    // Another request under a recorded key conflicts, across kinds too.
    for error in [
        activate(&engine, &lease, "activate-1", 2).unwrap_err(),
        retire(&engine, &lease, "activate-1", 1).unwrap_err(),
        activate(&engine, &lease, "build-1", 1).unwrap_err(),
    ] {
        assert_eq!(error.kind(), crate::ServiceErrorKind::Conflict);
    }
    assert_eq!(lifecycle_revision(&engine, &lease), recorded);
}

#[test]
fn a_lifecycle_operation_requires_an_idempotency_key() {
    let (_root, engine, lease) = fixture();
    let mut unkeyed = context("build-1");
    unkeyed.idempotency_key = None;
    let error = engine
        .build_vector_quantization_artifact(
            &lease.session_id,
            &lease.token,
            &build_request(29),
            &unkeyed,
            400,
        )
        .unwrap_err();
    assert_eq!(error.kind(), crate::ServiceErrorKind::InvalidArgument);
    assert_eq!(lifecycle_revision(&engine, &lease), (0, 0));
}

#[test]
fn a_recorded_operation_replays_after_reopen_and_lease_expiry() {
    let (root, engine, lease) = fixture();
    let first = build(&engine, &lease, "build-1", 29, 400).unwrap();
    drop(engine);
    let reopened = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    // The session's absolute lease ended at 10,100.
    let replayed = build(&reopened, &lease, "build-1", 29, 20_000).unwrap();
    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.artifact, first.artifact);
    let expired = build(&reopened, &lease, "build-2", 29, 20_000).unwrap_err();
    assert_eq!(expired.kind(), crate::ServiceErrorKind::FailedPrecondition);
}

#[test]
fn concurrent_builds_with_one_key_publish_one_generation() {
    let (_root, engine, lease) = fixture();
    let results = std::thread::scope(|threads| {
        // Each request has its own time, so the commits differ and only one
        // can land; the others retry and find the operation recorded.
        let handles = (0..4)
            .map(|thread| {
                let (engine, lease) = (&engine, &lease);
                threads.spawn(move || build(engine, lease, "build-1", 29, 400 + thread))
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap().unwrap())
            .collect::<Vec<_>>()
    });
    // Every caller sees the one generation, and every one succeeds: a commit
    // that loses its cursor retries and then finds the operation recorded.
    assert!(results
        .iter()
        .all(|result| result.artifact == results[0].artifact));
    assert_eq!(results[0].artifact.generation, 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| !result.idempotent_replay)
            .count(),
        1
    );
    assert_eq!(lifecycle_revision(&engine, &lease), (1, 1));
    // Every version of every lifecycle event record, at one stamp: one.
    let read = engine
        .storage
        .runtime()
        .read_stamp(&rrd_core::ScopeId::new(scope()).unwrap())
        .unwrap();
    let events = engine
        .storage
        .runtime()
        .read_versioned(
            &read,
            &[rrd_store::RuntimeVersionedSource::Records {
                kind: Some(
                    rrd_core::RuntimeType::new(rrd_vector::QUANTIZATION_LIFECYCLE_RECORD_TYPE)
                        .unwrap(),
                ),
            }],
            rrd_store::RuntimeReadBudget::new(10_000).unwrap(),
        )
        .unwrap();
    assert_eq!(events.changes.len(), 1);
}

#[test]
fn a_denied_lifecycle_operation_is_audited_and_replays_nothing() {
    let (_root, engine, lease) = fixture();
    build(&engine, &lease, "build-1", 29, 400).unwrap();
    let principal_id = CanonicalId::new("lifecycle-denied").unwrap();
    let resource = ResourcePath {
        segments: vec![ResourceId::new(ResourceKind::Instance, instance().as_str()).unwrap()],
    };
    SecurityRepository::new(&engine.storage, instance())
        .initialize(
            SecurityState {
                format_version: rrd_security::SECURITY_FORMAT,
                revision: 1,
                principals: [(
                    principal_id.clone(),
                    Principal {
                        id: principal_id.clone(),
                        kind: PrincipalKind::Service,
                        credential_sha256: digest::sha256_hex(b"lifecycle-denied-key"),
                        credential_revision: 1,
                        not_before_unix_ms: 1,
                        expires_at_unix_ms: u64::MAX,
                        disabled: false,
                        role_ids: Default::default(),
                        grants: vec![ResourceGrant {
                            action: SecurityAction::SessionCreate,
                            resource_prefix: resource,
                            data_policy: None,
                        }],
                    },
                )]
                .into_iter()
                .collect(),
                roles: Default::default(),
                identity_bindings: Default::default(),
                jwt_issuers: Default::default(),
            },
            450,
            "security-test",
            "request-security",
            "operation-security",
        )
        .unwrap();
    let denied = engine
        .create_authenticated_session(
            &principal_id,
            b"lifecycle-denied-key",
            &session_request(5_000, 2),
            &id("lifecycle-denied-session"),
            500,
            "request-lifecycle-denied-session",
            "operation-lifecycle-denied-session",
        )
        .unwrap();
    let scope = rrd_core::ScopeId::new(scope()).unwrap();
    let revision = crate::quantization_artifact_catalogue(&engine.storage, &scope)
        .unwrap()
        .revision;
    // Neither a new build nor the recorded one is returned to a caller
    // without the grant, and each refusal is audited.
    for key in ["build-2", "build-1"] {
        let error = build(&engine, &denied, key, 29, 600).unwrap_err();
        assert!(matches!(error, ServiceError::PermissionDenied));
        let audit = SecurityRepository::new(&engine.storage, instance())
            .audit_since(0, 64)
            .unwrap()
            .records
            .into_iter()
            .map(|(_, record)| record)
            .find(|record| {
                record.request_id == format!("request-{key}") && record.at_unix_ms == 600
            })
            .expect("the denied lifecycle operation is audited");
        assert_eq!(audit.action, SecurityAction::VectorCollectionEnsure);
        assert_eq!(audit.decision, AuditDecision::Denied);
    }
    assert_eq!(
        crate::quantization_artifact_catalogue(&engine.storage, &scope)
            .unwrap()
            .revision,
        revision
    );
}
