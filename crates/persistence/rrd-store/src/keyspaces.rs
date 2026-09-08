//! Canonical semantic key shapes layered on the rrflowKV tuple codec.
//!
//! Callers name a semantic shape; they never concatenate physical delimiters.
//! The codec places every shape below the manifest-authenticated
//! `format / tenant / scope / family` prefix and this module freezes the first
//! component that subdivides a family.

use crate::error::{Error, Result};
use crate::key_codec::{
    CatalogueSubfamily, DecodedKeyPart, KeyAddress, KeyCodec, KeyFamily, KeyPart,
};
use rrd_core::{Millis, Predicate, Reader, RuntimeRef, ScopeId, Subject};

pub(crate) use crate::key_codec::APPLICATION_FORMAT as RRFLOW_KV_FORMAT;
pub(crate) type RrflowKvKeyCodec = KeyCodec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Space {
    Claims,
    SequenceIndex,
    Access,
    System,
    Invocations,
    Projections,
    RuntimeChanges,
    RuntimeRecords,
    RuntimeRelations,
    RuntimeVectors,
    RuntimeSeries,
    RuntimeGeo,
    RuntimeObjects,
    RuntimeOutbox,
    RuntimeAudit,
    RuntimeCommits,
    RuntimeSchemas,
    RuntimeSnapshots,
}

pub(crate) const CLAIMS: Space = Space::Claims;
pub(crate) const SEQUENCE_INDEX: Space = Space::SequenceIndex;
pub(crate) const ACCESS: Space = Space::Access;
pub(crate) const META: Space = Space::System;
pub(crate) const INVOCATIONS: Space = Space::Invocations;
pub(crate) const PROJECTIONS: Space = Space::Projections;
pub(crate) const RUNTIME_CHANGES: Space = Space::RuntimeChanges;
pub(crate) const RUNTIME_RECORDS: Space = Space::RuntimeRecords;
pub(crate) const RUNTIME_RELATIONS: Space = Space::RuntimeRelations;
pub(crate) const RUNTIME_VECTORS: Space = Space::RuntimeVectors;
pub(crate) const RUNTIME_SERIES: Space = Space::RuntimeSeries;
pub(crate) const RUNTIME_GEO: Space = Space::RuntimeGeo;
pub(crate) const RUNTIME_OBJECTS: Space = Space::RuntimeObjects;
pub(crate) const RUNTIME_OUTBOX: Space = Space::RuntimeOutbox;
pub(crate) const RUNTIME_AUDIT: Space = Space::RuntimeAudit;
pub(crate) const RUNTIME_COMMITS: Space = Space::RuntimeCommits;
pub(crate) const RUNTIME_SCHEMAS: Space = Space::RuntimeSchemas;
pub(crate) const RUNTIME_SNAPSHOTS: Space = Space::RuntimeSnapshots;

impl Space {
    const fn family(self) -> KeyFamily {
        match self {
            Self::Claims | Self::SequenceIndex => KeyFamily::Temporal,
            Self::Access | Self::RuntimeAudit => KeyFamily::Audit,
            Self::System => KeyFamily::System,
            Self::Invocations | Self::RuntimeSchemas | Self::RuntimeSnapshots => {
                KeyFamily::Catalogue
            }
            Self::Projections => KeyFamily::ProjectionDelta,
            Self::RuntimeChanges => KeyFamily::EngineEvent,
            Self::RuntimeRecords
            | Self::RuntimeRelations
            | Self::RuntimeSeries
            | Self::RuntimeGeo
            | Self::RuntimeObjects => KeyFamily::Current,
            Self::RuntimeVectors => KeyFamily::Vector,
            Self::RuntimeOutbox => KeyFamily::Outbox,
            Self::RuntimeCommits => KeyFamily::RuntimeCommit,
        }
    }

    const fn subspace(self) -> u8 {
        match self {
            Self::Claims => 0x01,
            Self::SequenceIndex => 0x02,
            Self::Access => 0x01,
            Self::System => 0x01,
            // This is the canonical catalogue receipt subfamily, not a second
            // invocation namespace.
            Self::Invocations => CatalogueSubfamily::InvocationReceipt as u8,
            Self::RuntimeSchemas => 0x20,
            Self::RuntimeSnapshots => 0x21,
            Self::Projections => 0x01,
            Self::RuntimeChanges => 0x01,
            Self::RuntimeRecords => 0x01,
            Self::RuntimeRelations => 0x02,
            Self::RuntimeSeries => 0x03,
            Self::RuntimeGeo => 0x04,
            Self::RuntimeObjects => 0x05,
            Self::RuntimeVectors => 0x01,
            Self::RuntimeOutbox => 0x01,
            Self::RuntimeAudit => 0x02,
            Self::RuntimeCommits => 0x01,
        }
    }
}

fn encode(space: Space, parts: &[KeyPart<'_>]) -> Vec<u8> {
    let mut tuple = Vec::with_capacity(parts.len() + 1);
    tuple.push(KeyPart::U8(space.subspace()));
    tuple.extend_from_slice(parts);
    KeyCodec.encode(KeyAddress::GLOBAL, space.family(), &tuple)
}

pub(crate) fn space_prefix(space: Space) -> Vec<u8> {
    encode(space, &[])
}

fn decode_space(codec: KeyCodec, space: Space, stored: &[u8]) -> Result<Vec<DecodedKeyPart>> {
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

fn expect_parts<const N: usize>(
    codec: KeyCodec,
    space: Space,
    stored: &[u8],
) -> Result<[DecodedKeyPart; N]> {
    decode_space(codec, space, stored)?
        .try_into()
        .map_err(|_| Error::Codec(format!("{space:?} key has the wrong field count")))
}

pub(crate) fn claim_key(
    subject: &Subject,
    predicate: &Predicate,
    valid_from: Millis,
    tx_time: Millis,
) -> Vec<u8> {
    encode(
        CLAIMS,
        &[
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
            KeyPart::DescU64(valid_from),
            KeyPart::DescU64(tx_time),
        ],
    )
}

pub(crate) fn claim_subject_prefix(subject: &Subject) -> Vec<u8> {
    encode(CLAIMS, &[KeyPart::Text(subject.as_str())])
}

pub(crate) fn claim_version_prefix(subject: &Subject, predicate: &Predicate) -> Vec<u8> {
    encode(
        CLAIMS,
        &[
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
        ],
    )
}

pub(crate) fn claim_seek_key(subject: &Subject, predicate: &Predicate, as_of: Millis) -> Vec<u8> {
    encode(
        CLAIMS,
        &[
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
            KeyPart::DescU64(as_of),
        ],
    )
}

pub(crate) fn parse_claim_key(
    codec: KeyCodec,
    stored: &[u8],
) -> Result<(Subject, Predicate, Millis, Millis)> {
    let [DecodedKeyPart::Text(subject), DecodedKeyPart::Text(predicate), DecodedKeyPart::DescU64(valid_from), DecodedKeyPart::DescU64(tx_time)] =
        expect_parts(codec, CLAIMS, stored)?
    else {
        return Err(Error::Codec("claim key has invalid field types".into()));
    };
    Ok((
        Subject::new(subject)?,
        Predicate::new(predicate)?,
        valid_from,
        tx_time,
    ))
}

pub(crate) fn sequence_key(sequence: u64) -> Vec<u8> {
    encode(SEQUENCE_INDEX, &[KeyPart::U64(sequence)])
}

pub(crate) fn parse_sequence_key(codec: KeyCodec, stored: &[u8]) -> Result<u64> {
    let [DecodedKeyPart::U64(sequence)] = expect_parts(codec, SEQUENCE_INDEX, stored)? else {
        return Err(Error::Codec(
            "claim sequence key has invalid field types".into(),
        ));
    };
    Ok(sequence)
}

pub(crate) fn access_key(
    at: Millis,
    reader: &Reader,
    subject: &Subject,
    predicate: &Predicate,
) -> Vec<u8> {
    encode(
        ACCESS,
        &[
            KeyPart::U64(at),
            KeyPart::Text(reader.as_str()),
            KeyPart::Text(subject.as_str()),
            KeyPart::Text(predicate.as_str()),
        ],
    )
}

pub(crate) fn access_bound(at: Millis) -> Vec<u8> {
    encode(ACCESS, &[KeyPart::U64(at)])
}

pub(crate) fn parse_access_key(
    codec: KeyCodec,
    stored: &[u8],
) -> Result<(Millis, Reader, Subject, Predicate)> {
    let [DecodedKeyPart::U64(at), DecodedKeyPart::Text(reader), DecodedKeyPart::Text(subject), DecodedKeyPart::Text(predicate)] =
        expect_parts(codec, ACCESS, stored)?
    else {
        return Err(Error::Codec("access key has invalid field types".into()));
    };
    Ok((
        at,
        Reader::new(reader)?,
        Subject::new(subject)?,
        Predicate::new(predicate)?,
    ))
}

pub(crate) fn invocation_key(at: Millis, ordinal: u64) -> Vec<u8> {
    encode(INVOCATIONS, &[KeyPart::U64(at), KeyPart::U64(ordinal)])
}

pub(crate) fn invocation_bound(at: Millis) -> Vec<u8> {
    encode(INVOCATIONS, &[KeyPart::U64(at)])
}

pub(crate) fn projection_key(name: &str) -> Vec<u8> {
    encode(PROJECTIONS, &[KeyPart::Text(name)])
}

pub(crate) fn runtime_change_key(cursor: u64) -> Vec<u8> {
    encode(RUNTIME_CHANGES, &[KeyPart::U64(cursor)])
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

pub(crate) fn runtime_scope_prefix(space: Space, scope: &ScopeId) -> Vec<u8> {
    encode(space, &[KeyPart::Text(scope.as_str())])
}

pub(crate) fn parse_runtime_identity_key(
    codec: KeyCodec,
    space: Space,
    stored: &[u8],
) -> Result<(ScopeId, RuntimeRef)> {
    let [DecodedKeyPart::Text(scope), DecodedKeyPart::Text(kind), DecodedKeyPart::Text(id)] =
        expect_parts(codec, space, stored)?
    else {
        return Err(Error::Codec(
            "runtime identity key has invalid field types".into(),
        ));
    };
    Ok((ScopeId::new(scope)?, RuntimeRef::new(kind, id)?))
}

pub(crate) fn runtime_outbox_key(cursor: u64) -> Vec<u8> {
    encode(RUNTIME_OUTBOX, &[KeyPart::U64(cursor)])
}

pub(crate) fn runtime_audit_key(commit_id: &str) -> Vec<u8> {
    encode(RUNTIME_AUDIT, &[KeyPart::Text(commit_id)])
}

pub(crate) fn runtime_commit_key(commit_id: &str) -> Vec<u8> {
    encode(RUNTIME_COMMITS, &[KeyPart::Text(commit_id)])
}

pub(crate) fn runtime_schema_key(scope: &ScopeId) -> Vec<u8> {
    encode(RUNTIME_SCHEMAS, &[KeyPart::Text(scope.as_str())])
}

pub(crate) fn runtime_snapshot_key(id: &str) -> Vec<u8> {
    encode(RUNTIME_SNAPSHOTS, &[KeyPart::Text(id)])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum SystemEntry {
    ClaimSequence = 0x01,
    AcceptedClaimAppend = 0x02,
    ControlValue = 0x03,
    ControlJournalSequence = 0x04,
    ControlJournalLastDigest = 0x05,
    ControlJournalEntry = 0x06,
    CatalogueRevision = 0x07,
    InvocationOrdinal = 0x08,
    RuntimeCursor = 0x09,
    RuntimeLastDigest = 0x0a,
    RuntimeLastAuditDigest = 0x0b,
    RuntimeAccumulatorState = 0x0c,
    RuntimeAccumulatorNode = 0x0d,
}

fn system_key(entry: SystemEntry, parts: &[KeyPart<'_>]) -> Vec<u8> {
    let mut tuple = Vec::with_capacity(parts.len() + 1);
    tuple.push(KeyPart::U8(entry as u8));
    tuple.extend_from_slice(parts);
    encode(META, &tuple)
}

pub(crate) fn sequence_watermark_key() -> Vec<u8> {
    system_key(SystemEntry::ClaimSequence, &[])
}

pub(crate) fn accepted_append_key(idempotency_key: &str) -> Vec<u8> {
    system_key(
        SystemEntry::AcceptedClaimAppend,
        &[KeyPart::Text(idempotency_key)],
    )
}

pub(crate) fn control_record_key(key: &str) -> Vec<u8> {
    system_key(SystemEntry::ControlValue, &[KeyPart::Text(key)])
}

pub(crate) fn control_journal_sequence_key() -> Vec<u8> {
    system_key(SystemEntry::ControlJournalSequence, &[])
}

pub(crate) fn control_journal_last_digest_key() -> Vec<u8> {
    system_key(SystemEntry::ControlJournalLastDigest, &[])
}

pub(crate) fn control_journal_key(sequence: u64) -> Vec<u8> {
    system_key(SystemEntry::ControlJournalEntry, &[KeyPart::U64(sequence)])
}

pub(crate) fn is_control_journal_key(codec: KeyCodec, stored: &[u8]) -> bool {
    matches!(
        decode_space(codec, META, stored).as_deref(),
        Ok([DecodedKeyPart::U8(tag), DecodedKeyPart::U64(_)])
            if *tag == SystemEntry::ControlJournalEntry as u8
    )
}

pub(crate) fn catalog_revision_key(scope: &ScopeId) -> Vec<u8> {
    system_key(
        SystemEntry::CatalogueRevision,
        &[KeyPart::Text(scope.as_str())],
    )
}

pub(crate) fn invocation_watermark_key() -> Vec<u8> {
    system_key(SystemEntry::InvocationOrdinal, &[])
}

pub(crate) fn runtime_cursor_key() -> Vec<u8> {
    system_key(SystemEntry::RuntimeCursor, &[])
}

pub(crate) fn runtime_last_digest_key() -> Vec<u8> {
    system_key(SystemEntry::RuntimeLastDigest, &[])
}

pub(crate) fn runtime_last_audit_digest_key() -> Vec<u8> {
    system_key(SystemEntry::RuntimeLastAuditDigest, &[])
}

pub(crate) fn runtime_accumulator_state_key() -> Vec<u8> {
    system_key(SystemEntry::RuntimeAccumulatorState, &[])
}

pub(crate) fn runtime_accumulator_node_key(level: u8, index: u64) -> Vec<u8> {
    system_key(
        SystemEntry::RuntimeAccumulatorNode,
        &[KeyPart::U8(level), KeyPart::U64(index)],
    )
}

/// Persistence policy for a write transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// Authoritative writes. Journal is flushed and synced before commit returns.
    Authoritative,
    /// Telemetry writes. Buffered; a periodic flush or a later authoritative
    /// write carries them to disk.
    Buffered,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_codec::prefix_end;

    fn sp(subject: &str, predicate: &str) -> (Subject, Predicate) {
        (
            Subject::new(subject).unwrap(),
            Predicate::new(predicate).unwrap(),
        )
    }

    #[test]
    fn spaces_have_frozen_unique_family_subspace_coordinates() {
        let spaces = [
            CLAIMS,
            SEQUENCE_INDEX,
            ACCESS,
            META,
            INVOCATIONS,
            PROJECTIONS,
            RUNTIME_CHANGES,
            RUNTIME_RECORDS,
            RUNTIME_RELATIONS,
            RUNTIME_VECTORS,
            RUNTIME_SERIES,
            RUNTIME_GEO,
            RUNTIME_OBJECTS,
            RUNTIME_OUTBOX,
            RUNTIME_AUDIT,
            RUNTIME_COMMITS,
            RUNTIME_SCHEMAS,
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
        let (subject, predicate) = sp("a/b", "c");
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
}
