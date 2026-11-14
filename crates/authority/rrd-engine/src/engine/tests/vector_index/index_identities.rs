//! POAM-046: a scope that holds vector index generations keeps its public
//! changefeed, because every identity the catalogue writes is public.

use super::*;
use rrd_contract::{ChangeMutationSnapshot, ReadChangefeed, SessionLease};
use rrd_core::{ProjectionId, ScopeId};
use std::collections::BTreeSet;

fn scope() -> String {
    format!("instance:{}", instance())
}

fn fixture() -> (tempfile::TempDir, RrdEngine, SessionLease) {
    let root = tempfile::tempdir().unwrap();
    let engine = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    let lease = engine
        .create_session(
            &session_request(9_000, 8),
            &id("index-identities-session"),
            100,
            "request-index-identities-session",
            "operation-index-identities-session",
        )
        .unwrap();
    ensure_documents_memory_tier(
        &engine,
        &lease,
        VectorMemoryTier::Cached,
        "index-identities-collection",
        200,
    );
    commit_vectors(
        &engine,
        &lease,
        300,
        vec![
            TransactionMutation::PutSchema {
                registry: vector_fixture_schema(
                    "install index identities fixture",
                    DataRecordSchema {
                        allow_additional_properties: true,
                        ..DataRecordSchema::default()
                    },
                ),
            },
            document_mutation("a", "alpha"),
            document_mutation("b", "beta"),
            vector_mutation("a", vec![1.0, 0.0]),
            vector_mutation("b", vec![0.0, 1.0]),
        ],
    );
    (root, engine, lease)
}

/// Publishes an exact segment over the scope's current vectors, the way the
/// catalogue's other artifact kinds are published.
fn publish_exact_segment(engine: &RrdEngine, expected_revision: u64) {
    let scope = ScopeId::new(scope()).unwrap();
    let read = engine.storage.runtime().read_stamp(&scope).unwrap();
    let direct = crate::engine::vector::read_vector_versions(engine, &read, 10_000).unwrap();
    let canonical = rrd_vector::candidates_from_changes(&direct.changes, &scope);
    let data = rrd_store::DataRuntimeRef::new(&engine.storage, &engine.objects);
    let mut runtime = crate::reopen_vector_runtime(&data, &scope, canonical.clone()).unwrap();
    let segment = rrd_vector::ImmutableVectorSegment::build(
        rrd_vector::VectorSegmentConfig {
            id: ProjectionId::new("exact-documents-body").unwrap(),
            scope: scope.clone(),
            field: "body-embedding".into(),
            dimensions: 2,
            metric: rrd_vector::ScoreMetric::Dot,
            embedding_model: None,
            filter_properties: BTreeSet::new(),
        },
        1,
        read.commit_cursor,
        canonical,
    )
    .unwrap();
    crate::publish_traced_vector_artifact(
        &data,
        &mut runtime,
        expected_revision,
        rrd_vector::VectorArtifact::ExactSegment(segment),
        "session:index-identities",
        450,
    )
    .unwrap();
}

/// The closure oracle of POAM-046: after an HNSW and an exact-segment
/// publication, the changefeed projects every change, including each
/// artifact's record and object.
#[test]
fn a_scope_with_index_generations_keeps_its_public_changefeed() {
    let (_root, engine, lease) = fixture();
    engine
        .ensure_vector_index(
            &lease.session_id,
            &lease.token,
            &index_request(),
            &mutation_context(
                &id("operation-index-identities-build"),
                "request-index-identities-build",
                "operation-index-identities-build",
            ),
            400,
        )
        .unwrap();
    publish_exact_segment(&engine, 1);

    let page = engine
        .read_changefeed_page(&ReadChangefeed {
            scope: scope(),
            after_cursor: 0,
            limit: 1_000,
        })
        .unwrap();
    let identities = page
        .changes
        .iter()
        .filter_map(|change| match &change.mutation {
            ChangeMutationSnapshot::Data {
                mutation:
                    TransactionMutation::PutRecord { reference, .. }
                    | TransactionMutation::PublishObjectReference { reference, .. },
            } => Some(format!("{}:{}", reference.kind, reference.id)),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for artifact in ["hnsw-documents-body.1", "exact-documents-body.1"] {
        assert!(
            identities.contains(&format!("vector_artifact:{artifact}")),
            "{artifact} has no record in {identities:?}"
        );
    }
    let objects = page
        .changes
        .iter()
        .filter(|change| {
            matches!(
                &change.mutation,
                ChangeMutationSnapshot::Data {
                    mutation: TransactionMutation::PublishObjectReference { .. },
                }
            )
        })
        .count();
    assert_eq!(objects, 2, "each artifact publishes one object");
}

fn ensure_under_key(
    engine: &RrdEngine,
    lease: &SessionLease,
    request: &EnsureVectorIndex,
    key: &str,
    now: u64,
) -> Result<rrd_contract::EnsureVectorIndexResult, ServiceError> {
    engine.ensure_vector_index(
        &lease.session_id,
        &lease.token,
        request,
        &mutation_context(
            &id(key),
            &format!("request-{now}"),
            &format!("operation-{now}"),
        ),
        now,
    )
}

/// POAM-053: a retried public build finds the generation it published, even
/// after the source moved on, and survives a restart. Another request under
/// that key is a conflict, and a new key builds the next generation.
#[test]
fn an_index_build_honours_its_retry_key() {
    let (root, engine, lease) = fixture();
    let first = ensure_under_key(&engine, &lease, &index_request(), "build-key", 400).unwrap();
    assert!(!first.idempotent_replay);
    assert_eq!(first.index.generation, 1);

    commit_vectors(
        &engine,
        &lease,
        500,
        vec![
            document_mutation("c", "gamma"),
            vector_mutation("c", vec![0.5, 0.5]),
        ],
    );
    let retried = ensure_under_key(&engine, &lease, &index_request(), "build-key", 510).unwrap();
    assert!(retried.idempotent_replay);
    assert_eq!(retried.index.generation, 1);
    assert_eq!(retried.index.artifact_sha256, first.index.artifact_sha256);

    drop(engine);
    let engine = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    let reopened = ensure_under_key(&engine, &lease, &index_request(), "build-key", 520).unwrap();
    assert!(reopened.idempotent_replay);
    assert_eq!(reopened.index.generation, 1);

    let mut other = index_request();
    other.max_storage_keys += 1;
    assert!(matches!(
        ensure_under_key(&engine, &lease, &other, "build-key", 530),
        Err(ServiceError::IdempotencyConflict)
    ));

    let next = ensure_under_key(&engine, &lease, &index_request(), "next-key", 540).unwrap();
    assert!(!next.idempotent_replay);
    assert_eq!(next.index.generation, 2);
}
