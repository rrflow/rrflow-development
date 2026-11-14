//! Outbox, audit, commit, schema, snapshot, system and control-journal keys.

use super::*;

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
    Ok((ScopeId::new(scope)?, RuntimeRef::parse(kind, id)?))
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

pub(crate) fn runtime_schema_version_key(scope: &ScopeId, cursor: u64) -> Vec<u8> {
    encode(
        RUNTIME_SCHEMA_VERSIONS,
        &[KeyPart::Text(scope.as_str()), KeyPart::DescU64(cursor)],
    )
}

pub(crate) fn runtime_snapshot_key(id: &str) -> Vec<u8> {
    encode(RUNTIME_SNAPSHOTS, &[KeyPart::Text(id)])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum SystemEntry {
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
    ControlRestoreProgress = 0x0e,
    CatalogueRevisionCursor = 0x0f,
}

pub(super) fn system_key(entry: SystemEntry, parts: &[KeyPart<'_>]) -> Vec<u8> {
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

pub(crate) fn control_journal_prefix() -> Vec<u8> {
    system_key(SystemEntry::ControlJournalEntry, &[])
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

/// The runtime cursor at which a scope's catalogue revision took effect, so a
/// historical read stamp can prove the revision it names was current then.
pub(crate) fn catalog_revision_cursor_key(scope: &ScopeId, revision: u64) -> Vec<u8> {
    system_key(
        SystemEntry::CatalogueRevisionCursor,
        &[KeyPart::Text(scope.as_str()), KeyPart::U64(revision)],
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

/// Progress of a control-journal archive replay into this root. Written in
/// the same physical transaction as each replayed control transition, so a
/// resumed restore never re-applies or skips an entry.
pub(crate) fn control_restore_progress_key() -> Vec<u8> {
    system_key(SystemEntry::ControlRestoreProgress, &[])
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
