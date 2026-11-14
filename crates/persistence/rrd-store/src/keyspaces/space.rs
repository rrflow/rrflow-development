//! Key spaces, their tags and the shared key encoding.

use super::*;

pub(crate) type RrflowKvKeyCodec = KeyCodec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Space {
    Claims,
    SequenceIndex,
    Access,
    System,
    Invocations,
    FunctionArtifacts,
    FunctionDefinitions,
    TransactionFunctionBindings,
    FunctionCatalogueMemberships,
    FunctionCatalogueHeads,
    Projections,
    RuntimeChanges,
    RuntimeClaimVersions,
    RuntimeRecords,
    RuntimeRecordVersions,
    RuntimeRelations,
    RuntimeRelationVersions,
    RuntimeEventVersions,
    RuntimeOutgoingEdges,
    RuntimeOutgoingEdgeVersions,
    RuntimeIncomingEdges,
    RuntimeIncomingEdgeVersions,
    RuntimeVectors,
    RuntimeVectorVersions,
    RuntimeSeries,
    RuntimeSeriesVersions,
    RuntimeGeo,
    RuntimeGeoVersions,
    RuntimeObjects,
    RuntimeObjectVersions,
    RuntimeProjectionDeltas,
    RuntimeIndexSourceDeltas,
    RuntimeVectorSourceDeltas,
    RuntimeIndexBindings,
    RuntimeIndexBindingVersions,
    RuntimeIndexBindingSets,
    RuntimeScalarEntries,
    RuntimeUniqueEntries,
    RuntimeTermDictionary,
    RuntimeTermStatistics,
    RuntimeTermPostings,
    RuntimeOutbox,
    RuntimeAudit,
    RuntimeCommits,
    RuntimeSchemas,
    RuntimeSchemaVersions,
    RuntimeSnapshots,
}

pub(crate) const CLAIMS: Space = Space::Claims;
pub(crate) const SEQUENCE_INDEX: Space = Space::SequenceIndex;
pub(crate) const ACCESS: Space = Space::Access;
pub(crate) const META: Space = Space::System;
pub(crate) const INVOCATIONS: Space = Space::Invocations;
pub(crate) const FUNCTION_ARTIFACTS: Space = Space::FunctionArtifacts;
pub(crate) const FUNCTION_DEFINITIONS: Space = Space::FunctionDefinitions;
pub(crate) const TRANSACTION_FUNCTION_BINDINGS: Space = Space::TransactionFunctionBindings;
pub(crate) const FUNCTION_CATALOGUE_MEMBERSHIPS: Space = Space::FunctionCatalogueMemberships;
pub(crate) const FUNCTION_CATALOGUE_HEADS: Space = Space::FunctionCatalogueHeads;
pub(crate) const PROJECTIONS: Space = Space::Projections;
pub(crate) const RUNTIME_CHANGES: Space = Space::RuntimeChanges;
pub(crate) const RUNTIME_CLAIM_VERSIONS: Space = Space::RuntimeClaimVersions;
pub(crate) const RUNTIME_RECORDS: Space = Space::RuntimeRecords;
pub(crate) const RUNTIME_RECORD_VERSIONS: Space = Space::RuntimeRecordVersions;
pub(crate) const RUNTIME_RELATIONS: Space = Space::RuntimeRelations;
pub(crate) const RUNTIME_RELATION_VERSIONS: Space = Space::RuntimeRelationVersions;
pub(crate) const RUNTIME_EVENT_VERSIONS: Space = Space::RuntimeEventVersions;
pub(crate) const RUNTIME_OUTGOING_EDGES: Space = Space::RuntimeOutgoingEdges;
pub(crate) const RUNTIME_OUTGOING_EDGE_VERSIONS: Space = Space::RuntimeOutgoingEdgeVersions;
pub(crate) const RUNTIME_INCOMING_EDGES: Space = Space::RuntimeIncomingEdges;
pub(crate) const RUNTIME_INCOMING_EDGE_VERSIONS: Space = Space::RuntimeIncomingEdgeVersions;
pub(crate) const RUNTIME_VECTORS: Space = Space::RuntimeVectors;
pub(crate) const RUNTIME_VECTOR_VERSIONS: Space = Space::RuntimeVectorVersions;
pub(crate) const RUNTIME_SERIES: Space = Space::RuntimeSeries;
pub(crate) const RUNTIME_SERIES_VERSIONS: Space = Space::RuntimeSeriesVersions;
pub(crate) const RUNTIME_GEO: Space = Space::RuntimeGeo;
pub(crate) const RUNTIME_GEO_VERSIONS: Space = Space::RuntimeGeoVersions;
pub(crate) const RUNTIME_OBJECTS: Space = Space::RuntimeObjects;
pub(crate) const RUNTIME_OBJECT_VERSIONS: Space = Space::RuntimeObjectVersions;
pub(crate) const RUNTIME_PROJECTION_DELTAS: Space = Space::RuntimeProjectionDeltas;
pub(crate) const RUNTIME_INDEX_SOURCE_DELTAS: Space = Space::RuntimeIndexSourceDeltas;
pub(crate) const RUNTIME_VECTOR_SOURCE_DELTAS: Space = Space::RuntimeVectorSourceDeltas;
pub(crate) const RUNTIME_INDEX_BINDINGS: Space = Space::RuntimeIndexBindings;
pub(crate) const RUNTIME_INDEX_BINDING_VERSIONS: Space = Space::RuntimeIndexBindingVersions;
pub(crate) const RUNTIME_INDEX_BINDING_SETS: Space = Space::RuntimeIndexBindingSets;
pub(crate) const RUNTIME_SCALAR_ENTRIES: Space = Space::RuntimeScalarEntries;
pub(crate) const RUNTIME_UNIQUE_ENTRIES: Space = Space::RuntimeUniqueEntries;
pub(crate) const RUNTIME_TERM_DICTIONARY: Space = Space::RuntimeTermDictionary;
pub(crate) const RUNTIME_TERM_STATISTICS: Space = Space::RuntimeTermStatistics;
pub(crate) const RUNTIME_TERM_POSTINGS: Space = Space::RuntimeTermPostings;
pub(crate) const RUNTIME_OUTBOX: Space = Space::RuntimeOutbox;
pub(crate) const RUNTIME_AUDIT: Space = Space::RuntimeAudit;
pub(crate) const RUNTIME_COMMITS: Space = Space::RuntimeCommits;
pub(crate) const RUNTIME_SCHEMAS: Space = Space::RuntimeSchemas;
pub(crate) const RUNTIME_SCHEMA_VERSIONS: Space = Space::RuntimeSchemaVersions;
pub(crate) const RUNTIME_SNAPSHOTS: Space = Space::RuntimeSnapshots;

impl Space {
    pub(super) const fn family(self) -> KeyFamily {
        match self {
            Self::Claims
            | Self::SequenceIndex
            | Self::RuntimeClaimVersions
            | Self::RuntimeRecordVersions
            | Self::RuntimeRelationVersions
            | Self::RuntimeSeriesVersions
            | Self::RuntimeGeoVersions
            | Self::RuntimeObjectVersions => KeyFamily::Temporal,
            Self::Access | Self::RuntimeAudit => KeyFamily::Audit,
            Self::System => KeyFamily::System,
            Self::Invocations
            | Self::FunctionArtifacts
            | Self::FunctionDefinitions
            | Self::TransactionFunctionBindings
            | Self::FunctionCatalogueMemberships
            | Self::FunctionCatalogueHeads
            | Self::RuntimeSchemas
            | Self::RuntimeSchemaVersions
            | Self::RuntimeSnapshots
            | Self::RuntimeIndexBindings
            | Self::RuntimeIndexBindingVersions
            | Self::RuntimeIndexBindingSets => KeyFamily::Catalogue,
            Self::Projections
            | Self::RuntimeProjectionDeltas
            | Self::RuntimeIndexSourceDeltas
            | Self::RuntimeVectorSourceDeltas => KeyFamily::ProjectionDelta,
            Self::RuntimeChanges | Self::RuntimeEventVersions => KeyFamily::EngineEvent,
            Self::RuntimeRecords
            | Self::RuntimeRelations
            | Self::RuntimeSeries
            | Self::RuntimeGeo
            | Self::RuntimeObjects => KeyFamily::Current,
            Self::RuntimeVectors | Self::RuntimeVectorVersions => KeyFamily::Vector,
            Self::RuntimeScalarEntries => KeyFamily::Scalar,
            Self::RuntimeUniqueEntries => KeyFamily::Unique,
            Self::RuntimeTermDictionary => KeyFamily::TermDictionary,
            Self::RuntimeTermStatistics => KeyFamily::TermStatistic,
            Self::RuntimeTermPostings => KeyFamily::TermPosting,
            Self::RuntimeOutgoingEdges | Self::RuntimeOutgoingEdgeVersions => {
                KeyFamily::OutgoingEdge
            }
            Self::RuntimeIncomingEdges | Self::RuntimeIncomingEdgeVersions => {
                KeyFamily::IncomingEdge
            }
            Self::RuntimeOutbox => KeyFamily::Outbox,
            Self::RuntimeCommits => KeyFamily::RuntimeCommit,
        }
    }

    pub(super) const fn subspace(self) -> u8 {
        match self {
            Self::Claims => 0x01,
            Self::SequenceIndex => 0x02,
            Self::RuntimeRecordVersions => 0x03,
            Self::RuntimeRelationVersions => 0x04,
            Self::RuntimeClaimVersions => 0x05,
            Self::RuntimeSeriesVersions => 0x06,
            Self::RuntimeGeoVersions => 0x07,
            Self::RuntimeObjectVersions => 0x08,
            Self::Access => 0x01,
            Self::System => 0x01,
            // This is the canonical catalogue receipt subfamily, not a second
            // invocation namespace.
            Self::Invocations => CatalogueSubfamily::InvocationReceipt as u8,
            Self::FunctionArtifacts => CatalogueSubfamily::FunctionArtifact as u8,
            Self::FunctionDefinitions => CatalogueSubfamily::FunctionDefinition as u8,
            Self::TransactionFunctionBindings => {
                CatalogueSubfamily::TransactionFunctionBinding as u8
            }
            Self::FunctionCatalogueMemberships => CatalogueSubfamily::Membership as u8,
            Self::FunctionCatalogueHeads => CatalogueSubfamily::Head as u8,
            Self::RuntimeSchemas => 0x20,
            Self::RuntimeSnapshots => 0x21,
            Self::RuntimeSchemaVersions => 0x22,
            Self::RuntimeIndexBindings => 0x30,
            Self::RuntimeIndexBindingVersions => 0x31,
            Self::RuntimeIndexBindingSets => 0x32,
            Self::Projections => 0x01,
            Self::RuntimeProjectionDeltas => 0x02,
            Self::RuntimeIndexSourceDeltas => 0x03,
            Self::RuntimeVectorSourceDeltas => 0x04,
            Self::RuntimeChanges => 0x01,
            Self::RuntimeEventVersions => 0x02,
            Self::RuntimeRecords => 0x01,
            Self::RuntimeRelations => 0x02,
            Self::RuntimeSeries => 0x03,
            Self::RuntimeGeo => 0x04,
            Self::RuntimeObjects => 0x05,
            Self::RuntimeVectors => 0x01,
            Self::RuntimeVectorVersions => 0x02,
            Self::RuntimeScalarEntries | Self::RuntimeUniqueEntries => 0x01,
            Self::RuntimeTermDictionary
            | Self::RuntimeTermStatistics
            | Self::RuntimeTermPostings => 0x01,
            Self::RuntimeOutgoingEdges | Self::RuntimeIncomingEdges => 0x01,
            Self::RuntimeOutgoingEdgeVersions | Self::RuntimeIncomingEdgeVersions => 0x02,
            Self::RuntimeOutbox => 0x01,
            Self::RuntimeAudit => 0x02,
            Self::RuntimeCommits => 0x01,
        }
    }
}

pub(super) fn encode(space: Space, parts: &[KeyPart<'_>]) -> Vec<u8> {
    let mut tuple = Vec::with_capacity(parts.len() + 1);
    tuple.push(KeyPart::U8(space.subspace()));
    tuple.extend_from_slice(parts);
    KeyCodec.encode(KeyAddress::GLOBAL, space.family(), &tuple)
}

pub(crate) fn space_prefix(space: Space) -> Vec<u8> {
    encode(space, &[])
}

pub(super) fn decode_space(
    codec: KeyCodec,
    space: Space,
    stored: &[u8],
) -> Result<Vec<DecodedKeyPart>> {
    let decoded = codec.decode(stored)?;
    if decoded.tenant.is_some()
        || decoded.scope.is_some()
        || decoded.family != space.family()
        || decoded.parts.first() != Some(&DecodedKeyPart::U8(space.subspace()))
    {
        return Err(Error::Codec(format!(
            "key escaped canonical rrflowKV space {space:?}"
        )));
    }
    Ok(decoded.parts.into_iter().skip(1).collect())
}

pub(crate) fn validate_space(codec: KeyCodec, space: Space, stored: &[u8]) -> Result<()> {
    decode_space(codec, space, stored).map(|_| ())
}

pub(super) fn expect_parts<const N: usize>(
    codec: KeyCodec,
    space: Space,
    stored: &[u8],
) -> Result<[DecodedKeyPart; N]> {
    decode_space(codec, space, stored)?
        .try_into()
        .map_err(|_| Error::Codec(format!("{space:?} key has the wrong field count")))
}
