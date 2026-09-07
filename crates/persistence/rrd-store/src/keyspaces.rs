//! Canonical rrflowKV keyspaces, physical tags, and durability classes.

/// Authoritative claims.
pub const CLAIMS: &str = "claims";
/// Sequence index: append sequence to canonical claim key.
pub const SEQUENCE_INDEX: &str = "sequence_index";
/// Read telemetry. Loss on crash is acceptable.
pub const ACCESS: &str = "access";
/// Watermarks and idempotency keys.
pub const META: &str = "meta";
/// Recorded operator invocations (`SPEC.md` §13).
pub const INVOCATIONS: &str = "invocations";
/// Derived projections (the routing index among them). Derivable from their
/// sources by construction, so loss on crash costs a rebuild, not truth.
pub const PROJECTIONS: &str = "projections";
/// Authoritative append-only runtime change log, keyed by global cursor.
pub const RUNTIME_CHANGES: &str = "runtime_changes";
/// Latest persisted version of each typed runtime record. Updated atomically
/// with the authoritative change log and used for reference-integrity checks.
pub const RUNTIME_RECORDS: &str = "runtime_records";
/// Latest persisted version of each typed runtime relation.
pub const RUNTIME_RELATIONS: &str = "runtime_relations";
/// Latest canonical vector values. Vector indexes remain projections.
pub const RUNTIME_VECTORS: &str = "runtime_vectors";
/// Latest canonical time-series samples. Time indexes remain projections.
pub const RUNTIME_SERIES: &str = "runtime_series";
/// Latest canonical WGS84 values. Spatial indexes remain projections.
pub const RUNTIME_GEO: &str = "runtime_geo";
/// Canonical visibility records for verified immutable object bytes.
pub const RUNTIME_OBJECTS: &str = "runtime_objects";
/// Transactional projection work keyed by the source runtime cursor.
pub const RUNTIME_OUTBOX: &str = "runtime_outbox";
/// Accepted-operation audit envelopes keyed by commit identity.
pub const RUNTIME_AUDIT: &str = "runtime_audit";
/// Accepted transaction outcomes keyed by content identity for idempotent retry.
pub const RUNTIME_COMMITS: &str = "runtime_commits";
/// Latest authoritative schema registry for each runtime scope. Every update
/// is also present in the hash-chained runtime change log.
pub const RUNTIME_SCHEMAS: &str = "runtime_schemas";
/// Persisted leased read stamps. rrflowKV uses the same catalogue to pin
/// physical manifests.
pub const RUNTIME_SNAPSHOTS: &str = "runtime_snapshots";

#[cfg(test)]
const ALL: [&str; 18] = [
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

/// Manifest-authenticated rrflowKV application format (`RRDSK002`).
pub(crate) const RRFLOW_KV_FORMAT: u64 = 0x5252_4453_4b30_3032;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RrflowKvKeyCodec;

impl RrflowKvKeyCodec {
    pub(crate) fn from_application_format(format: Option<u64>) -> Option<Self> {
        (format == Some(RRFLOW_KV_FORMAT)).then_some(Self)
    }

    pub(crate) fn encode(self, space: &str, key: &[u8]) -> Option<Vec<u8>> {
        let mut stored = Vec::with_capacity(1 + key.len());
        stored.push(rrflow_kv_keyspace_tag(space)?);
        stored.extend_from_slice(key);
        Some(stored)
    }

    pub(crate) fn strip<'a>(self, space: &str, stored: &'a [u8]) -> Option<&'a [u8]> {
        stored.strip_prefix(&[rrflow_kv_keyspace_tag(space)?])
    }
}

pub(crate) fn rrflow_kv_keyspace_tag(space: &str) -> Option<u8> {
    match space {
        CLAIMS => Some(1),
        SEQUENCE_INDEX => Some(2),
        ACCESS => Some(3),
        META => Some(4),
        INVOCATIONS => Some(5),
        PROJECTIONS => Some(6),
        RUNTIME_CHANGES => Some(7),
        RUNTIME_RECORDS => Some(8),
        RUNTIME_RELATIONS => Some(9),
        RUNTIME_VECTORS => Some(10),
        RUNTIME_SERIES => Some(11),
        RUNTIME_GEO => Some(12),
        RUNTIME_OBJECTS => Some(13),
        RUNTIME_OUTBOX => Some(14),
        RUNTIME_AUDIT => Some(15),
        RUNTIME_COMMITS => Some(16),
        RUNTIME_SCHEMAS => Some(17),
        RUNTIME_SNAPSHOTS => Some(18),
        _ => None,
    }
}

#[cfg(test)]
mod rrflow_kv_key_codec_tests {
    use super::*;

    #[test]
    fn rrflow_kv_keyspace_tags_are_frozen_unique_and_canonical() {
        let tags = ALL
            .iter()
            .map(|space| rrflow_kv_keyspace_tag(space).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(tags, (1_u8..=18).collect::<Vec<_>>());
        assert_eq!(RrflowKvKeyCodec::from_application_format(None), None);
        assert_eq!(
            RrflowKvKeyCodec::from_application_format(Some(RRFLOW_KV_FORMAT)),
            Some(RrflowKvKeyCodec)
        );
        assert_eq!(RrflowKvKeyCodec::from_application_format(Some(7)), None);
        assert_eq!(rrflow_kv_keyspace_tag("unknown"), None);
    }

    #[test]
    fn rrflow_kv_key_codec_round_trips_frozen_wire_bytes() {
        let logical = b"subject/predicate";
        let compact = RrflowKvKeyCodec.encode(CLAIMS, logical).unwrap();
        assert_eq!(compact, b"\x01subject/predicate");
        assert_eq!(
            RrflowKvKeyCodec.strip(CLAIMS, &compact),
            Some(logical.as_slice())
        );

        assert_eq!(RrflowKvKeyCodec.encode("unknown", logical), None);
        assert_eq!(RrflowKvKeyCodec.strip(SEQUENCE_INDEX, &compact), None);
    }
}

/// Key under which the claim sequence watermark is recorded.
pub const SEQUENCE_WATERMARK: &[u8] = b"watermark/claims/sequence";

pub(crate) fn accepted_append_key(idempotency_key: &str) -> Vec<u8> {
    let mut key = b"accepted/claim-append/".to_vec();
    key.extend_from_slice(idempotency_key.as_bytes());
    key
}

pub(crate) const CONTROL_JOURNAL_SEQUENCE: &[u8] = b"server/journal/sequence";
pub(crate) const CONTROL_JOURNAL_LAST_DIGEST: &[u8] = b"server/journal/last-digest";

const CATALOG_REVISION_PREFIX: &[u8] = b"watermark/catalogue/revision/";

/// Per-scope monotonic revision for catalogue state that is stored outside the
/// typed runtime log. The scope suffix keeps unrelated project/instance
/// catalogues from invalidating one another's read stamps.
pub(crate) fn catalog_revision_key(scope: &str) -> Vec<u8> {
    let mut key = Vec::with_capacity(CATALOG_REVISION_PREFIX.len() + scope.len());
    key.extend_from_slice(CATALOG_REVISION_PREFIX);
    key.extend_from_slice(scope.as_bytes());
    key
}

pub(crate) fn control_journal_key(sequence: u64) -> Vec<u8> {
    format!("server/journal/entries/{sequence:020}").into_bytes()
}

/// Key under which the invocation ordinal watermark is recorded.
pub const INVOCATION_WATERMARK: &[u8] = b"watermark/invocations/ordinal";

/// Global cursor and hash-chain head for typed runtime changes.
pub const RUNTIME_CURSOR: &[u8] = b"watermark/runtime/cursor";
pub const RUNTIME_LAST_DIGEST: &[u8] = b"watermark/runtime/last-digest";
pub const RUNTIME_LAST_AUDIT_DIGEST: &[u8] = b"watermark/runtime/last-audit-digest";
/// Versioned RFC 9162 compact frontier for the global runtime log. Complete
/// subtree nodes share this META keyspace so snapshots cannot accidentally
/// omit proof state.
pub const RUNTIME_ACCUMULATOR_STATE: &[u8] = b"runtime/merkle/v1/state";
const RUNTIME_ACCUMULATOR_NODE_PREFIX: &[u8] = b"runtime/merkle/v1/node/";

pub fn runtime_accumulator_node_key(level: u8, index: u64) -> Vec<u8> {
    let mut key = Vec::with_capacity(RUNTIME_ACCUMULATOR_NODE_PREFIX.len() + 9);
    key.extend_from_slice(RUNTIME_ACCUMULATOR_NODE_PREFIX);
    key.push(level);
    key.extend_from_slice(&index.to_be_bytes());
    key
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
