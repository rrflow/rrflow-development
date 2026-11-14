use super::*;
use crate::key_codec::prefix_end;

const DIRECT_VERSION_KEY_FIXTURE: &str =
    include_str!("../../fixtures/rrflow-kv-direct-version-keys-v1.hex");

fn sp(subject: &str, predicate: &str) -> (Subject, Predicate) {
    (
        Subject::new(subject).unwrap(),
        Predicate::new(predicate).unwrap(),
    )
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[usize::from(byte >> 4)] as char);
        encoded.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

#[test]
fn spaces_have_frozen_unique_family_subspace_coordinates() {
    let spaces = [
        CLAIMS,
        SEQUENCE_INDEX,
        ACCESS,
        META,
        INVOCATIONS,
        FUNCTION_ARTIFACTS,
        FUNCTION_DEFINITIONS,
        TRANSACTION_FUNCTION_BINDINGS,
        FUNCTION_CATALOGUE_MEMBERSHIPS,
        FUNCTION_CATALOGUE_HEADS,
        PROJECTIONS,
        RUNTIME_CHANGES,
        RUNTIME_CLAIM_VERSIONS,
        RUNTIME_RECORDS,
        RUNTIME_RECORD_VERSIONS,
        RUNTIME_RELATIONS,
        RUNTIME_RELATION_VERSIONS,
        RUNTIME_EVENT_VERSIONS,
        RUNTIME_OUTGOING_EDGES,
        RUNTIME_OUTGOING_EDGE_VERSIONS,
        RUNTIME_INCOMING_EDGES,
        RUNTIME_INCOMING_EDGE_VERSIONS,
        RUNTIME_VECTORS,
        RUNTIME_VECTOR_VERSIONS,
        RUNTIME_SERIES,
        RUNTIME_SERIES_VERSIONS,
        RUNTIME_GEO,
        RUNTIME_GEO_VERSIONS,
        RUNTIME_OBJECTS,
        RUNTIME_OBJECT_VERSIONS,
        RUNTIME_PROJECTION_DELTAS,
        RUNTIME_INDEX_SOURCE_DELTAS,
        RUNTIME_VECTOR_SOURCE_DELTAS,
        RUNTIME_INDEX_BINDINGS,
        RUNTIME_INDEX_BINDING_VERSIONS,
        RUNTIME_INDEX_BINDING_SETS,
        RUNTIME_SCALAR_ENTRIES,
        RUNTIME_UNIQUE_ENTRIES,
        RUNTIME_TERM_DICTIONARY,
        RUNTIME_TERM_STATISTICS,
        RUNTIME_TERM_POSTINGS,
        RUNTIME_OUTBOX,
        RUNTIME_AUDIT,
        RUNTIME_COMMITS,
        RUNTIME_SCHEMAS,
        RUNTIME_SCHEMA_VERSIONS,
        RUNTIME_SNAPSHOTS,
    ];
    let mut coordinates = spaces
        .map(|space| (space.family() as u8, space.subspace()))
        .to_vec();
    coordinates.sort_unstable();
    coordinates.dedup();
    assert_eq!(coordinates.len(), spaces.len());
}

#[test]
fn claim_keys_are_bitemporal_newest_first_and_isolated() {
    let codec = KeyCodec;
    let (subject, predicate) = sp("wp3", "status");
    let original = claim_key(&subject, &predicate, 100, 100);
    let correction = claim_key(&subject, &predicate, 100, 200);
    let newer = claim_key(&subject, &predicate, 300, 300);
    assert!(newer < correction && correction < original);
    assert_eq!(
        parse_claim_key(codec, &correction).unwrap(),
        (subject.clone(), predicate.clone(), 100, 200)
    );

    let prefix = claim_version_prefix(&subject, &predicate);
    let end = prefix_end(&prefix).unwrap();
    assert!(correction >= prefix && correction < end);
    for (other_subject, other_predicate) in
        [("wp3x", "status"), ("wp3", "statusx"), ("wp", "status")]
    {
        let (other_subject, other_predicate) = sp(other_subject, other_predicate);
        let neighbour = claim_key(&other_subject, &other_predicate, 100, 100);
        assert!(neighbour < prefix || neighbour >= end);
    }
}

#[test]
fn seek_ordering_matches_as_of_semantics() {
    let (subject, predicate) = sp("wp3", "status");
    let v1 = claim_key(&subject, &predicate, 100, 100);
    let v2 = claim_key(&subject, &predicate, 200, 200);
    let seek = claim_seek_key(&subject, &predicate, 150);
    assert!(v2 < seek);
    assert!(seek <= v1);
}

#[test]
fn access_and_sequence_keys_round_trip_in_order() {
    let codec = KeyCodec;
    let reader = Reader::new("agent:clyffy/worker").unwrap();
    let (subject, predicate) = sp("a-b", "c");
    let access = access_key(500, &reader, &subject, &predicate);
    assert_eq!(
        parse_access_key(codec, &access).unwrap(),
        (500, reader, subject, predicate)
    );
    assert!(
        access_key(
            9,
            &Reader::new("r").unwrap(),
            &Subject::new("s").unwrap(),
            &Predicate::new("p").unwrap()
        ) < access_bound(10)
    );
    assert!(sequence_key(9) < sequence_key(10));
    assert_eq!(parse_sequence_key(codec, &sequence_key(42)).unwrap(), 42);
}

#[test]
fn malformed_semantic_shapes_are_rejected() {
    let codec = KeyCodec;
    assert!(parse_claim_key(codec, &space_prefix(CLAIMS)).is_err());
    assert!(parse_access_key(codec, &sequence_key(1)).is_err());
    let malformed = KeyCodec.encode(
        KeyAddress::GLOBAL,
        KeyFamily::Temporal,
        &[KeyPart::U8(Space::Claims.subspace()), KeyPart::U64(1)],
    );
    assert!(parse_claim_key(codec, &malformed).is_err());
}

#[test]
fn temporal_and_adjacency_shapes_preserve_direction_and_newest_first_order() {
    let scope = ScopeId::new("project:graph-order").unwrap();
    let relation = RuntimeRelation {
        reference: RuntimeRef::new("imports", "a-b").unwrap(),
        from: RuntimeRef::new("file", "src-a.rs").unwrap(),
        to: RuntimeRef::new("file", "src-b.rs").unwrap(),
        valid_from: 100,
        valid_to: None,
        properties: Default::default(),
    };
    let outgoing = runtime_adjacency_key(RUNTIME_OUTGOING_EDGES, &scope, &relation);
    let incoming = runtime_adjacency_key(RUNTIME_INCOMING_EDGES, &scope, &relation);
    assert_ne!(outgoing, incoming);
    let outgoing_history = space_prefix(RUNTIME_OUTGOING_EDGE_VERSIONS);
    let incoming_history = space_prefix(RUNTIME_INCOMING_EDGE_VERSIONS);
    assert_ne!(space_prefix(RUNTIME_OUTGOING_EDGES), outgoing_history);
    assert_ne!(space_prefix(RUNTIME_INCOMING_EDGES), incoming_history);
    assert!(
        runtime_adjacency_version_key(RUNTIME_OUTGOING_EDGE_VERSIONS, &scope, &relation, 200, 9,)
            < runtime_adjacency_version_key(
                RUNTIME_OUTGOING_EDGE_VERSIONS,
                &scope,
                &relation,
                100,
                8,
            )
    );
    assert!(
        runtime_version_key(
            RUNTIME_RELATION_VERSIONS,
            &scope,
            &relation.reference,
            200,
            9,
        ) < runtime_version_key(
            RUNTIME_RELATION_VERSIONS,
            &scope,
            &relation.reference,
            100,
            8,
        )
    );
}

#[test]
fn adjacency_prefixes_isolate_node_and_relation_type_and_keys_round_trip() {
    let codec = KeyCodec;
    let scope = ScopeId::new("project:adjacency-prefix").unwrap();
    let relation = RuntimeRelation {
        reference: RuntimeRef::new("imports", "a-b").unwrap(),
        from: RuntimeRef::new("file", "a").unwrap(),
        to: RuntimeRef::new("module", "b").unwrap(),
        valid_from: 100,
        valid_to: None,
        properties: Default::default(),
    };
    let imports = rrd_core::RuntimeType::new("imports").unwrap();
    for (space, near, far) in [
        (RUNTIME_OUTGOING_EDGE_VERSIONS, &relation.from, &relation.to),
        (RUNTIME_INCOMING_EDGE_VERSIONS, &relation.to, &relation.from),
    ] {
        let key = runtime_adjacency_version_key(space, &scope, &relation, 100, 7);
        let prefix = runtime_adjacency_prefix(space, &scope, near, &imports);
        let end = prefix_end(&prefix).unwrap();
        assert!(key >= prefix && key < end);
        for (node_id, kind) in [
            (format!("{}x", near.id), "imports"),
            (format!("{}-b", near.id), "imports"),
            (near.id.to_string(), "importsx"),
            (near.id.to_string(), "import"),
        ] {
            let mut neighbour = relation.clone();
            neighbour.reference = RuntimeRef::new(kind, "a-b").unwrap();
            let endpoint = RuntimeRef::new(near.kind.as_str(), node_id.as_str()).unwrap();
            if space == RUNTIME_OUTGOING_EDGE_VERSIONS {
                neighbour.from = endpoint;
            } else {
                neighbour.to = endpoint;
            }
            let neighbour = runtime_adjacency_version_key(space, &scope, &neighbour, 100, 7);
            assert!(neighbour < prefix || neighbour >= end, "{node_id} {kind}");
        }
        assert_eq!(
            parse_runtime_adjacency_version_key(codec, space, &key).unwrap(),
            RuntimeAdjacencyVersionKey {
                scope: scope.clone(),
                node: near.clone(),
                relation: relation.reference.clone(),
                other: far.clone(),
                effective_at: 100,
                cursor: 7,
            }
        );
        assert!(parse_runtime_adjacency_version_key(codec, space, &prefix).is_err());
    }
    assert!(parse_runtime_adjacency_version_key(
        codec,
        RUNTIME_INCOMING_EDGE_VERSIONS,
        &runtime_adjacency_version_key(RUNTIME_OUTGOING_EDGE_VERSIONS, &scope, &relation, 100, 7),
    )
    .is_err());
}

#[test]
fn every_direct_version_family_is_scope_bound_and_newest_first() {
    let scope = ScopeId::new("project:direct-version-keys").unwrap();
    let other = ScopeId::new("project:direct-version-keys-other").unwrap();
    let reference = RuntimeRef::new("document", "alpha").unwrap();
    for space in [
        RUNTIME_RECORD_VERSIONS,
        RUNTIME_RELATION_VERSIONS,
        RUNTIME_EVENT_VERSIONS,
        RUNTIME_SERIES_VERSIONS,
        RUNTIME_GEO_VERSIONS,
        RUNTIME_OBJECT_VERSIONS,
    ] {
        let newer = runtime_version_key(space, &scope, &reference, 200, 9);
        let older = runtime_version_key(space, &scope, &reference, 100, 8);
        assert!(newer < older);
        assert!(newer.starts_with(&runtime_kind_prefix(space, &scope, &reference.kind)));
        assert!(!newer.starts_with(&runtime_scope_prefix(space, &other)));
    }

    let claim = rrd_core::Claim::new(
        Subject::new("document-alpha").unwrap(),
        Predicate::new("status").unwrap(),
        "ready",
        100,
        200,
        rrd_core::Producer {
            actor: "test:direct-version-keys".into(),
            on_behalf_of: None,
            session: None,
        },
    );
    let claim_key = runtime_claim_version_key(&scope, &claim, 9);
    assert!(claim_key.starts_with(&runtime_claim_predicate_prefix(&scope, &claim.predicate)));
    assert!(!claim_key.starts_with(&runtime_scope_prefix(RUNTIME_CLAIM_VERSIONS, &other)));
    assert!(runtime_schema_version_key(&scope, 9) < runtime_schema_version_key(&scope, 8));
    assert!(runtime_schema_version_key(&scope, 9)
        .starts_with(&runtime_scope_prefix(RUNTIME_SCHEMA_VERSIONS, &scope)));
}

#[test]
fn direct_version_key_shapes_match_the_frozen_fixture() {
    let scope = ScopeId::new("project:version-fixture").unwrap();
    let reference = RuntimeRef::new("document", "alpha").unwrap();
    let claim = rrd_core::Claim::new(
        Subject::new("document-alpha").unwrap(),
        Predicate::new("status").unwrap(),
        "ready",
        100,
        200,
        rrd_core::Producer {
            actor: "test:version-fixture".into(),
            on_behalf_of: None,
            session: None,
        },
    );
    let entries = [
        ("schema", runtime_schema_version_key(&scope, 1)),
        ("claim", runtime_claim_version_key(&scope, &claim, 2)),
        (
            "record",
            runtime_version_key(RUNTIME_RECORD_VERSIONS, &scope, &reference, 100, 3),
        ),
        (
            "relation",
            runtime_version_key(RUNTIME_RELATION_VERSIONS, &scope, &reference, 100, 4),
        ),
        (
            "event",
            runtime_version_key(RUNTIME_EVENT_VERSIONS, &scope, &reference, 100, 5),
        ),
        (
            "vector",
            runtime_vector_version_key(&scope, &reference, 100, 6),
        ),
        (
            "series",
            runtime_version_key(RUNTIME_SERIES_VERSIONS, &scope, &reference, 100, 7),
        ),
        (
            "geo",
            runtime_version_key(RUNTIME_GEO_VERSIONS, &scope, &reference, 100, 8),
        ),
        (
            "object",
            runtime_version_key(RUNTIME_OBJECT_VERSIONS, &scope, &reference, 100, 9),
        ),
    ];
    let actual = entries
        .into_iter()
        .map(|(label, key)| format!("{label} {}", hex(&key)))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(actual, DIRECT_VERSION_KEY_FIXTURE.trim_end());
}

#[test]
fn native_index_and_vector_delta_keys_are_typed_ordered_and_source_isolated() {
    let scope = ScopeId::new("project:index-key-order").unwrap();
    let index = ProjectionId::new("by-score").unwrap();
    let reference = RuntimeRef::new("document", "alpha").unwrap();
    let negative = vec![RuntimeValue::Integer(-7)];
    let positive = vec![RuntimeValue::Integer(9)];
    let negative_key = runtime_index_entry_key(
        RUNTIME_SCALAR_ENTRIES,
        &scope,
        &index,
        &negative,
        &reference,
        10,
    )
    .unwrap();
    let positive_key = runtime_index_entry_key(
        RUNTIME_SCALAR_ENTRIES,
        &scope,
        &index,
        &positive,
        &reference,
        10,
    )
    .unwrap();
    assert!(negative_key < positive_key);

    let unique_prefix =
        runtime_index_value_prefix(RUNTIME_UNIQUE_ENTRIES, &scope, &index, &positive).unwrap();
    let unique_key = runtime_index_entry_key(
        RUNTIME_UNIQUE_ENTRIES,
        &scope,
        &index,
        &positive,
        &reference,
        10,
    )
    .unwrap();
    assert!(unique_key.starts_with(&unique_prefix));
    assert!(
        runtime_index_source_delta_key(&scope, &index, 9, 0)
            < runtime_index_source_delta_key(&scope, &index, 10, 0)
    );
    assert!(
        runtime_vector_version_key(&scope, &reference, 20, 9)
            < runtime_vector_version_key(&scope, &reference, 10, 8)
    );
    let start = |collection: &str, name: &str, cursor: u64| {
        runtime_vector_source_delta_start(&scope, collection, name, "title", cursor).unwrap()
    };
    let documents = start("documents", "semantic", 1);
    assert_ne!(documents, start("archive", "semantic", 1));
    assert_ne!(documents, start("documents", "summary", 1));
    // A cursor interval of one source orders every ordinal of its cursors
    // inside it and no other source between its ends.
    let key =
        runtime_vector_source_delta_key(&scope, "documents", "semantic", "title", 7, 3).unwrap();
    assert!(start("documents", "semantic", 7) <= key && key < start("documents", "semantic", 8));
    assert!(
        !(start("documents", "semantic", 1)..start("documents", "semantic", u64::MAX))
            .contains(&start("documents-archive", "semantic", 7))
    );
    assert!(runtime_vector_source_delta_start(&scope, "", "semantic", "title", 1).is_err());
    assert!(runtime_vector_source_delta_start(&scope, "documents", "", "title", 1).is_err());
}
