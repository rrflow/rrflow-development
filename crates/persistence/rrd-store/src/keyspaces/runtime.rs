//! Runtime identity, version, claim and adjacency keys.

use super::*;

pub(crate) fn projection_key(name: &str) -> Vec<u8> {
    encode(PROJECTIONS, &[KeyPart::Text(name)])
}

pub(crate) fn runtime_change_key(cursor: u64) -> Vec<u8> {
    encode(RUNTIME_CHANGES, &[KeyPart::U64(cursor)])
}

pub(crate) fn runtime_event_reference(event: &RuntimeEvent, cursor: u64) -> RuntimeRef {
    RuntimeRef::event(event.kind.clone(), cursor)
}

pub(crate) fn runtime_identity_key(
    space: Space,
    scope: &ScopeId,
    reference: &RuntimeRef,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeRecords
            | Space::RuntimeRelations
            | Space::RuntimeVectors
            | Space::RuntimeSeries
            | Space::RuntimeGeo
            | Space::RuntimeObjects
    ));
    encode(
        space,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(reference.kind.as_str()),
            KeyPart::Text(reference.id.as_str()),
        ],
    )
}

pub(crate) fn runtime_version_key(
    space: Space,
    scope: &ScopeId,
    reference: &RuntimeRef,
    effective_at: Millis,
    cursor: u64,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeRecordVersions
            | Space::RuntimeRelationVersions
            | Space::RuntimeEventVersions
            | Space::RuntimeSeriesVersions
            | Space::RuntimeGeoVersions
            | Space::RuntimeObjectVersions
    ));
    encode(
        space,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(reference.kind.as_str()),
            KeyPart::Text(reference.id.as_str()),
            KeyPart::DescU64(effective_at),
            KeyPart::DescU64(cursor),
        ],
    )
}

pub(crate) fn runtime_kind_prefix(
    space: Space,
    scope: &ScopeId,
    kind: &rrd_core::RuntimeType,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeRecords
            | Space::RuntimeRecordVersions
            | Space::RuntimeRelationVersions
            | Space::RuntimeEventVersions
            | Space::RuntimeVectorVersions
            | Space::RuntimeSeriesVersions
            | Space::RuntimeGeoVersions
            | Space::RuntimeObjectVersions
    ));
    encode(
        space,
        &[KeyPart::Text(scope.as_str()), KeyPart::Text(kind.as_str())],
    )
}

pub(crate) fn runtime_reference_prefix(
    space: Space,
    scope: &ScopeId,
    reference: &RuntimeRef,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeRecordVersions
            | Space::RuntimeRelationVersions
            | Space::RuntimeEventVersions
            | Space::RuntimeVectorVersions
            | Space::RuntimeSeriesVersions
            | Space::RuntimeGeoVersions
            | Space::RuntimeObjectVersions
    ));
    encode(
        space,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(reference.kind.as_str()),
            KeyPart::Text(reference.id.as_str()),
        ],
    )
}

pub(crate) fn runtime_claim_version_key(
    scope: &ScopeId,
    claim: &rrd_core::Claim,
    cursor: u64,
) -> Vec<u8> {
    encode(
        RUNTIME_CLAIM_VERSIONS,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(claim.predicate.as_str()),
            KeyPart::Text(claim.subject.as_str()),
            KeyPart::DescU64(claim.valid_from),
            KeyPart::DescU64(claim.tx_time),
            KeyPart::DescU64(cursor),
        ],
    )
}

pub(crate) fn runtime_claim_predicate_prefix(scope: &ScopeId, predicate: &Predicate) -> Vec<u8> {
    encode(
        RUNTIME_CLAIM_VERSIONS,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(predicate.as_str()),
        ],
    )
}

pub(crate) fn runtime_adjacency_key(
    space: Space,
    scope: &ScopeId,
    relation: &RuntimeRelation,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeOutgoingEdges | Space::RuntimeIncomingEdges
    ));
    let (source, target) = adjacency_endpoints(space, relation);
    encode(
        space,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(source.kind.as_str()),
            KeyPart::Text(source.id.as_str()),
            KeyPart::Text(relation.reference.kind.as_str()),
            KeyPart::Text(target.kind.as_str()),
            KeyPart::Text(target.id.as_str()),
            KeyPart::Text(relation.reference.id.as_str()),
        ],
    )
}

pub(crate) fn runtime_adjacency_version_key(
    space: Space,
    scope: &ScopeId,
    relation: &RuntimeRelation,
    effective_at: Millis,
    cursor: u64,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeOutgoingEdgeVersions | Space::RuntimeIncomingEdgeVersions
    ));
    let (source, target) = adjacency_endpoints(space, relation);
    encode(
        space,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(source.kind.as_str()),
            KeyPart::Text(source.id.as_str()),
            KeyPart::Text(relation.reference.kind.as_str()),
            KeyPart::Text(target.kind.as_str()),
            KeyPart::Text(target.id.as_str()),
            KeyPart::Text(relation.reference.id.as_str()),
            KeyPart::DescU64(effective_at),
            KeyPart::DescU64(cursor),
        ],
    )
}

/// Current or temporal adjacency range for one node and relation type.
/// Temporal ranges include every version and retirement at that endpoint.
pub(crate) fn runtime_adjacency_prefix(
    space: Space,
    scope: &ScopeId,
    node: &RuntimeRef,
    relation_kind: &rrd_core::RuntimeType,
) -> Vec<u8> {
    debug_assert!(matches!(
        space,
        Space::RuntimeOutgoingEdges
            | Space::RuntimeIncomingEdges
            | Space::RuntimeOutgoingEdgeVersions
            | Space::RuntimeIncomingEdgeVersions
    ));
    encode(
        space,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(node.kind.as_str()),
            KeyPart::Text(node.id.as_str()),
            KeyPart::Text(relation_kind.as_str()),
        ],
    )
}

/// Current outgoing identities for one typed endpoint pair.
pub(crate) fn runtime_adjacency_pair_prefix(
    scope: &ScopeId,
    relation: &RuntimeRelation,
) -> Vec<u8> {
    encode(
        RUNTIME_OUTGOING_EDGES,
        &[
            KeyPart::Text(scope.as_str()),
            KeyPart::Text(relation.from.kind.as_str()),
            KeyPart::Text(relation.from.id.as_str()),
            KeyPart::Text(relation.reference.kind.as_str()),
            KeyPart::Text(relation.to.kind.as_str()),
            KeyPart::Text(relation.to.id.as_str()),
        ],
    )
}

/// Strictly decoded temporal adjacency key. `node` is the space's near
/// endpoint and `other` the far endpoint recorded when the version was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeAdjacencyVersionKey {
    pub(crate) scope: ScopeId,
    pub(crate) node: RuntimeRef,
    pub(crate) relation: RuntimeRef,
    pub(crate) other: RuntimeRef,
    pub(crate) effective_at: Millis,
    pub(crate) cursor: u64,
}

pub(crate) fn parse_runtime_adjacency_version_key(
    codec: KeyCodec,
    space: Space,
    stored: &[u8],
) -> Result<RuntimeAdjacencyVersionKey> {
    debug_assert!(matches!(
        space,
        Space::RuntimeOutgoingEdgeVersions | Space::RuntimeIncomingEdgeVersions
    ));
    let [DecodedKeyPart::Text(scope), DecodedKeyPart::Text(node_kind), DecodedKeyPart::Text(node_id), DecodedKeyPart::Text(relation_kind), DecodedKeyPart::Text(other_kind), DecodedKeyPart::Text(other_id), DecodedKeyPart::Text(relation_id), DecodedKeyPart::DescU64(effective_at), DecodedKeyPart::DescU64(cursor)] =
        expect_parts(codec, space, stored)?
    else {
        return Err(Error::Codec(
            "runtime adjacency version key has invalid field types".into(),
        ));
    };
    Ok(RuntimeAdjacencyVersionKey {
        scope: ScopeId::new(scope)?,
        node: RuntimeRef::parse(node_kind, node_id)?,
        relation: RuntimeRef::parse(relation_kind, relation_id)?,
        other: RuntimeRef::parse(other_kind, other_id)?,
        effective_at,
        cursor,
    })
}

pub(super) fn adjacency_endpoints(
    space: Space,
    relation: &RuntimeRelation,
) -> (&RuntimeRef, &RuntimeRef) {
    match space {
        Space::RuntimeOutgoingEdges | Space::RuntimeOutgoingEdgeVersions => {
            (&relation.from, &relation.to)
        }
        Space::RuntimeIncomingEdges | Space::RuntimeIncomingEdgeVersions => {
            (&relation.to, &relation.from)
        }
        _ => panic!("adjacency key requires an outgoing or incoming edge space"),
    }
}

pub(crate) fn runtime_projection_delta_key(work: &ProjectionWork) -> Vec<u8> {
    encode(
        RUNTIME_PROJECTION_DELTAS,
        &[
            KeyPart::U64(work.source_cursor),
            KeyPart::U64(work.commit_ordinal),
            KeyPart::U8(projection_family_tag(work.family)),
            KeyPart::Text(work.scope.as_str()),
        ],
    )
}
