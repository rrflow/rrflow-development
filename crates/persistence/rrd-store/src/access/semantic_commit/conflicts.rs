//! Admission of document writes whose begin snapshot precedes publication.

use super::*;
use rrd_core::RuntimeLogicalModel;

pub(super) fn validate_document_writes(
    reader: &(impl AccessRead + ?Sized),
    commit: &RuntimeCommit,
    head_cursor: u64,
    live_read: Option<&ReadStamp>,
    archived_audit: Option<&AuditEnvelope>,
    function_receipts: Option<(&str, &[FunctionInvocationReceiptRecord])>,
) -> Result<()> {
    let conflict = || Error::RuntimeConflict {
        expected: commit.expected_cursor,
        actual: head_cursor,
    };
    let read = live_read
        .or_else(|| archived_audit.and_then(|audit| audit.read.as_ref()))
        .ok_or_else(conflict)?;
    if read.scope != commit.scope
        || read.commit_cursor != commit.expected_cursor
        || read.commit_cursor > head_cursor
        || function_receipts.is_some_and(|(_, receipts)| !receipts.is_empty())
    {
        return Err(conflict());
    }

    let mut sources = BTreeSet::new();
    for mutation in &commit.mutations {
        match mutation {
            RuntimeMutation::Record { record } if record.valid_to.is_none() => {
                sources.insert(record.reference.clone());
            }
            _ => return Err(conflict()),
        }
    }
    let head = read_stamp_with(reader, &commit.scope)?;
    // Logical archives preserve the source catalogue stamp in the audit;
    // only a live publication has the originating control catalogue available.
    if head.schema_revision != read.schema_revision
        || (live_read.is_some() && head.catalog_revision != read.catalog_revision)
    {
        return Err(conflict());
    }
    if archived_audit.is_some() {
        super::super::runtime_state::validate_archived_read(reader, read)?;
    }
    // All record-like table models share the record identity version range,
    // including tombstones. Compare commit coordinates across all valid times.
    let targets = sources
        .into_iter()
        .map(|reference| RuntimeVersionedSource::Identity {
            model: RuntimeLogicalModel::Document,
            reference,
        })
        .collect::<Vec<_>>();
    let versions = read_versioned(
        reader,
        &head,
        &targets,
        RuntimeReadBudget::authenticated_history_integrity(head_cursor)?,
    )?;
    if versions
        .changes
        .iter()
        .any(|change| change.cursor > read.commit_cursor)
    {
        return Err(conflict());
    }
    Ok(())
}
