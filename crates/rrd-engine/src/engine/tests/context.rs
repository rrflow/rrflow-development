use super::*;
use rrd_contract::{
    AssembleContext, ContextEvidenceKind, DataCatalogueIdentity, DataLogicalModel, DataReference,
    DataSchemaMode, DataSchemaRegistry, DataTableSchema, EmbeddingInput, EmbeddingNetworkPolicy,
    EnsureVectorCollection, GenerateEmbeddings, ListEmbeddingModels, NamedVectorDefinition,
    QueryValue, VectorEmbeddingModel, VectorMemoryTier, VectorSearchMetric, VectorValueKind,
};
use std::collections::BTreeMap;

fn reference(kind: &str, id: &str) -> DataReference {
    DataReference {
        kind: CanonicalId::new(kind).unwrap(),
        id: CanonicalId::new(id).unwrap(),
    }
}

#[test]
fn context_flows_through_one_engine_stamp_and_survives_reopen() {
    let (root, engine) = isolated_engine();
    let lease = engine
        .create_session(
            &session_request(5_000, 2),
            &id("context-session"),
            1_000,
            "request-context-session",
            "operation-context-session",
        )
        .unwrap();
    let alpha = reference("note", "alpha");
    let beta = reference("note", "beta");
    let gamma = reference("note", "gamma");
    let relation_kind = CanonicalId::new("supports").unwrap();
    let mutations = vec![
        TransactionMutation::PutSchema {
            registry: DataSchemaRegistry {
                revision: 1,
                migration: "install context graph fixture".into(),
                catalogue: DataCatalogueIdentity::default(),
                tables: BTreeMap::from([
                    (
                        alpha.kind.clone(),
                        DataTableSchema {
                            model: DataLogicalModel::GraphNode,
                            mode: DataSchemaMode::Schemaless,
                            properties: BTreeMap::new(),
                            allow_additional_properties: false,
                        },
                    ),
                    (
                        relation_kind.clone(),
                        DataTableSchema {
                            model: DataLogicalModel::GraphRelation,
                            mode: DataSchemaMode::Schemaless,
                            properties: BTreeMap::new(),
                            allow_additional_properties: false,
                        },
                    ),
                ]),
                records: BTreeMap::new(),
                relations: BTreeMap::new(),
                events: BTreeMap::new(),
            },
        },
        TransactionMutation::PutRecord {
            reference: alpha.clone(),
            valid_from: 1_100,
            valid_to: None,
            properties: BTreeMap::from([
                (
                    "title".into(),
                    QueryValue::String("Authentication session recovery".into()),
                ),
                (
                    "body".into(),
                    QueryValue::String("Restore the durable session from its token.".into()),
                ),
            ]),
        },
        TransactionMutation::PutRecord {
            reference: beta.clone(),
            valid_from: 1_100,
            valid_to: None,
            properties: BTreeMap::from([(
                "body".into(),
                QueryValue::String("Rotate credentials only after recovery succeeds.".into()),
            )]),
        },
        TransactionMutation::PutRecord {
            reference: gamma,
            valid_from: 1_100,
            valid_to: None,
            properties: BTreeMap::from([(
                "body".into(),
                QueryValue::String("Unrelated rendering preferences.".into()),
            )]),
        },
        TransactionMutation::PutRelation {
            reference: reference("supports", "alpha-beta"),
            from: alpha,
            to: beta,
            valid_from: 1_100,
            valid_to: None,
            properties: BTreeMap::new(),
        },
    ];
    let transaction = engine
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &BeginTransaction {
                scope: CanonicalId::new("data").unwrap(),
                timeout_ms: 1_000,
            },
            &mutation_context(
                &id("context-begin"),
                "request-context-begin",
                "operation-context-begin",
            ),
            1_050,
        )
        .unwrap();
    let receipt = engine
        .commit_transaction(
            &lease.session_id,
            &lease.token,
            &transaction.transaction_id,
            &id("context-commit"),
            &CommitTransaction {
                operation_sha256: rrd_contract::transaction_operation_sha256(&mutations),
                mutations,
            },
            1_100,
            "request-context-commit",
            "operation-context-commit",
        )
        .unwrap();
    let request = AssembleContext {
        scope: "instance:test-instance".into(),
        query: "authentication session".into(),
        valid_at: 1_200,
        seeds: Vec::new(),
        max_graph_depth: 1,
        max_items: 8,
        max_output_bytes: 64 * 1024,
        max_scanned_changes: 128,
    };
    let packet = engine
        .assemble_context(
            &lease.session_id,
            &lease.token,
            &request,
            1_200,
            "request-context",
            "operation-context",
        )
        .unwrap();
    packet.validate().unwrap();
    assert_eq!(
        packet.read.runtime_cursor,
        receipt.last_runtime_cursor.unwrap()
    );
    let alpha = packet
        .items
        .iter()
        .find(|item| item.identity == "record:note:alpha")
        .unwrap();
    assert!(alpha
        .evidence
        .iter()
        .any(|evidence| evidence.kind == ContextEvidenceKind::Text));
    let beta = packet
        .items
        .iter()
        .find(|item| item.identity == "record:note:beta")
        .unwrap();
    assert!(beta
        .evidence
        .iter()
        .any(|evidence| evidence.kind == ContextEvidenceKind::Graph));
    assert!(packet
        .items
        .iter()
        .all(|item| item.identity != "record:note:gamma"));

    let mut bounded_request = request.clone();
    bounded_request.max_items = 1;
    bounded_request.max_output_bytes = 4 * 1024;
    let bounded = engine
        .assemble_context(
            &lease.session_id,
            &lease.token,
            &bounded_request,
            1_200,
            "request-context-bounded",
            "operation-context-bounded",
        )
        .unwrap();
    assert_eq!(bounded.items.len(), 1);
    assert!(bounded.truncated);
    assert!(bounded.output_bytes <= bounded_request.max_output_bytes);

    let mut before_validity = request.clone();
    before_validity.valid_at = 1_050;
    let before_validity = engine
        .assemble_context(
            &lease.session_id,
            &lease.token,
            &before_validity,
            1_200,
            "request-context-before-validity",
            "operation-context-before-validity",
        )
        .unwrap();
    assert!(before_validity.items.is_empty());

    drop(engine);
    let reopened = RrdEngine::open(root.path(), instance(), TOKEN_KEY).unwrap();
    let reopened_packet = reopened
        .assemble_context(
            &lease.session_id,
            &lease.token,
            &request,
            1_201,
            "request-context-reopen",
            "operation-context-reopen",
        )
        .unwrap();
    assert_eq!(reopened_packet, packet);
}

#[test]
fn context_discovers_matching_local_vector_retrieval_without_caller_wiring() {
    let (_root, engine) = isolated_engine();
    let lease = engine
        .create_session(
            &session_request(5_000, 2),
            &id("vector-context-session"),
            1_000,
            "request-vector-context-session",
            "operation-vector-context-session",
        )
        .unwrap();
    engine.install_feature_hash_embedding(16, 7).unwrap();
    let catalogue = engine
        .list_embedding_models(
            &lease.session_id,
            &lease.token,
            &ListEmbeddingModels {
                scope: "instance:test-instance".into(),
            },
            1_010,
            "request-vector-context-models",
            "operation-vector-context-models",
        )
        .unwrap();
    let backend = catalogue.backends.into_iter().next().unwrap();
    let collection_id = CanonicalId::new("context-memory").unwrap();
    let vector_name = CanonicalId::new("semantic").unwrap();
    engine
        .ensure_vector_collection(
            &lease.session_id,
            &lease.token,
            &EnsureVectorCollection {
                scope: "instance:test-instance".into(),
                collection_id: collection_id.clone(),
                vectors: vec![NamedVectorDefinition {
                    name: vector_name.clone(),
                    field: CanonicalId::new("embedding").unwrap(),
                    kind: VectorValueKind::Dense,
                    dimensions: backend.dimensions,
                    metric: VectorSearchMetric::Cosine,
                    embedding_model: Some(VectorEmbeddingModel {
                        name: format!(
                            "{}/{}@{}",
                            backend.provider, backend.model, backend.revision
                        ),
                        digest: backend.model_sha256.clone(),
                    }),
                    memory_tier: VectorMemoryTier::Cached,
                }],
            },
            &mutation_context(
                &id("vector-context-collection"),
                "request-vector-context-collection",
                "operation-vector-context-collection",
            ),
            1_020,
        )
        .unwrap();

    let note = reference("note", "semantic-alpha");
    let embedding = reference("embedding", "semantic-alpha");
    let initial = vec![
        TransactionMutation::PutSchema {
            registry: DataSchemaRegistry {
                revision: 1,
                migration: "install semantic context fixture".into(),
                catalogue: DataCatalogueIdentity::default(),
                tables: BTreeMap::from([
                    (
                        note.kind.clone(),
                        DataTableSchema {
                            model: DataLogicalModel::GraphNode,
                            mode: DataSchemaMode::Schemaless,
                            properties: BTreeMap::new(),
                            allow_additional_properties: false,
                        },
                    ),
                    (
                        embedding.kind.clone(),
                        DataTableSchema {
                            model: DataLogicalModel::Vector,
                            mode: DataSchemaMode::Schemaless,
                            properties: BTreeMap::new(),
                            allow_additional_properties: false,
                        },
                    ),
                ]),
                records: BTreeMap::new(),
                relations: BTreeMap::new(),
                events: BTreeMap::new(),
            },
        },
        TransactionMutation::PutRecord {
            reference: note.clone(),
            valid_from: 1_100,
            valid_to: None,
            properties: BTreeMap::from([(
                "body".into(),
                QueryValue::String("Recover durable authentication state".into()),
            )]),
        },
    ];
    let transaction = engine
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &BeginTransaction {
                scope: CanonicalId::new("data").unwrap(),
                timeout_ms: 1_000,
            },
            &mutation_context(
                &id("vector-context-begin-record"),
                "request-vector-context-begin-record",
                "operation-vector-context-begin-record",
            ),
            1_050,
        )
        .unwrap();
    engine
        .commit_transaction(
            &lease.session_id,
            &lease.token,
            &transaction.transaction_id,
            &id("vector-context-commit-record"),
            &CommitTransaction {
                operation_sha256: rrd_contract::transaction_operation_sha256(&initial),
                mutations: initial,
            },
            1_100,
            "request-vector-context-commit-record",
            "operation-vector-context-commit-record",
        )
        .unwrap();
    let generated = engine
        .generate_embeddings(
            &lease.session_id,
            &lease.token,
            &GenerateEmbeddings {
                scope: "instance:test-instance".into(),
                backend_id: backend.id,
                network_policy: EmbeddingNetworkPolicy::Deny,
                inputs: vec![EmbeddingInput {
                    id: CanonicalId::new("semantic-alpha").unwrap(),
                    media_type: "text/plain; charset=utf-8".into(),
                    bytes: b"Recover durable authentication state".to_vec(),
                }],
            },
            1_110,
            "request-vector-context-embedding",
            "operation-vector-context-embedding",
        )
        .unwrap()
        .embeddings
        .into_iter()
        .next()
        .unwrap();
    let vector_mutations = vec![TransactionMutation::PutVector {
        reference: embedding,
        subject: note,
        collection_id: Some(collection_id),
        vector_name: Some(vector_name),
        field: CanonicalId::new("embedding").unwrap(),
        valid_from: 1_120,
        valid_to: None,
        value: generated.value,
        provenance: Some(generated.provenance),
        properties: BTreeMap::new(),
    }];
    let transaction = engine
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &BeginTransaction {
                scope: CanonicalId::new("data").unwrap(),
                timeout_ms: 1_000,
            },
            &mutation_context(
                &id("vector-context-begin-vector"),
                "request-vector-context-begin-vector",
                "operation-vector-context-begin-vector",
            ),
            1_115,
        )
        .unwrap();
    engine
        .commit_transaction(
            &lease.session_id,
            &lease.token,
            &transaction.transaction_id,
            &id("vector-context-commit-vector"),
            &CommitTransaction {
                operation_sha256: rrd_contract::transaction_operation_sha256(&vector_mutations),
                mutations: vector_mutations,
            },
            1_120,
            "request-vector-context-commit-vector",
            "operation-vector-context-commit-vector",
        )
        .unwrap();

    let packet = engine
        .assemble_context(
            &lease.session_id,
            &lease.token,
            &AssembleContext {
                scope: "instance:test-instance".into(),
                query: "authentication recovery".into(),
                valid_at: 1_200,
                seeds: Vec::new(),
                max_graph_depth: 1,
                max_items: 8,
                max_output_bytes: 64 * 1024,
                max_scanned_changes: 128,
            },
            1_200,
            "request-vector-context",
            "operation-vector-context",
        )
        .unwrap();
    let item = packet
        .items
        .iter()
        .find(|item| item.identity == "record:note:semantic-alpha")
        .unwrap();
    assert!(item
        .evidence
        .iter()
        .any(|evidence| evidence.kind == ContextEvidenceKind::Vector));
}

#[test]
fn context_reads_active_claims_from_the_same_runtime_snapshot() {
    let (_root, engine) = isolated_engine();
    let lease = engine
        .create_session(
            &session_request(5_000, 2),
            &id("claim-context-session"),
            1_000,
            "request-claim-context-session",
            "operation-claim-context-session",
        )
        .unwrap();
    let transaction = engine
        .begin_transaction(
            &lease.session_id,
            &lease.token,
            &BeginTransaction {
                scope: CanonicalId::new("data").unwrap(),
                timeout_ms: 1_000,
            },
            &mutation_context(
                &id("claim-context-begin"),
                "request-claim-context-begin",
                "operation-claim-context-begin",
            ),
            1_050,
        )
        .unwrap();
    let mutations = vec![
        TransactionMutation::PutSchema {
            registry: DataSchemaRegistry {
                revision: 1,
                migration: "install claim context fixture".into(),
                catalogue: DataCatalogueIdentity::default(),
                tables: BTreeMap::from([(
                    CanonicalId::new("claim").unwrap(),
                    DataTableSchema {
                        model: DataLogicalModel::ReasoningClaim,
                        mode: DataSchemaMode::Schemaless,
                        properties: BTreeMap::new(),
                        allow_additional_properties: false,
                    },
                )]),
                records: BTreeMap::new(),
                relations: BTreeMap::new(),
                events: BTreeMap::new(),
            },
        },
        TransactionMutation::AssertClaim {
            subject: CanonicalId::new("project-alpha").unwrap(),
            predicate: CanonicalId::new("status").unwrap(),
            object: "Durable authentication recovery is operational".into(),
            valid_from: 1_100,
            tx_time: 1_100,
            producer: CanonicalId::new("agent-test").unwrap(),
            confidence: Some(0.9),
        },
    ];
    let receipt = engine
        .commit_transaction(
            &lease.session_id,
            &lease.token,
            &transaction.transaction_id,
            &id("claim-context-commit"),
            &CommitTransaction {
                operation_sha256: rrd_contract::transaction_operation_sha256(&mutations),
                mutations,
            },
            1_100,
            "request-claim-context-commit",
            "operation-claim-context-commit",
        )
        .unwrap();
    let packet = engine
        .assemble_context(
            &lease.session_id,
            &lease.token,
            &AssembleContext {
                scope: "instance:test-instance".into(),
                query: "authentication recovery".into(),
                valid_at: 1_200,
                seeds: Vec::new(),
                max_graph_depth: 1,
                max_items: 8,
                max_output_bytes: 64 * 1024,
                max_scanned_changes: 128,
            },
            1_200,
            "request-claim-context",
            "operation-claim-context",
        )
        .unwrap();
    assert_eq!(
        packet.read.runtime_cursor,
        receipt.last_runtime_cursor.unwrap()
    );
    let item = packet
        .items
        .iter()
        .find(|item| {
            item.values.get("subject") == Some(&QueryValue::String("project-alpha".into()))
        })
        .unwrap();
    assert!(item.identity.starts_with("claim:"));
    assert!(item
        .evidence
        .iter()
        .any(|evidence| evidence.kind == ContextEvidenceKind::Text));
}
