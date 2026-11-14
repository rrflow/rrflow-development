//! Reading one selected source file under its budget.

use super::*;

/// Stream exactly one regular file from an authenticated Inventory proposal.
/// The caller must bind the proposal to the job's committed checkpoint, authorize
/// the operation, and recheck the project source and lease before publication.
/// Reserved input remains charged after an attempted read, including failures.
/// The sink must be disposable: on any error its partial bytes are unverified and
/// the caller must discard them and terminate any worker receiving them.
#[allow(dead_code)] // Used by the Parse executor once isolated execution is admitted.
pub(in crate::engine) fn read_selected_source(
    root: &AnchoredPath,
    proposal: &rrd_attunement::inventory::Proposal,
    selected: &rrd_attunement::inventory::Entry,
    max_file_bytes: u64,
    remaining_input_bytes: &mut u64,
    output: &mut dyn std::io::Write,
    check: &dyn Fn() -> Result<()>,
) -> Result<()> {
    use rrd_attunement::inventory::EntryKind;
    use std::path::Component;

    check()?;
    if !proposal.is_complete() {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    let entries = proposal.entries();
    let entry = entries
        .binary_search_by(|entry| entry.path.cmp(&selected.path))
        .ok()
        .and_then(|index| entries.get(index))
        .filter(|entry| *entry == selected)
        .ok_or(ServiceError::ProjectBindingMismatch)?;
    let EntryKind::File {
        observation,
        content_sha256,
    } = &entry.kind
    else {
        return Err(ServiceError::ProjectBindingMismatch);
    };
    let relative = Path::new(&entry.path);
    if entry.path.is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(ServiceError::ProjectBindingMismatch);
    }
    if observation.bytes > max_file_bytes || observation.bytes > *remaining_input_bytes {
        return Err(ServiceError::ConfigurationLimit(
            "selected source exceeds parse input budget".into(),
        ));
    }
    let path = root.join(relative);
    let source = match path.source_leaf().map_err(storage_error)? {
        SourceLeaf::Ordinary(source) if source.kind() == SourceKind::File => source,
        _ => return Err(ServiceError::AttunementSourceChanged),
    };
    if source.bytes() != observation.bytes
        || source.executable_mode() != observation.executable
        || source.revision_sha256().map_err(storage_error)? != observation.revision
    {
        return Err(ServiceError::AttunementSourceChanged);
    }
    let mut file = path.open_read().map_err(storage_error)?;
    if path
        .source_observation_for_file(&file)
        .map_err(storage_error)?
        != source
    {
        return Err(ServiceError::AttunementSourceChanged);
    }
    *remaining_input_bytes -= observation.bytes;
    let mut digest = IncrementalSha256::new();
    let mut remaining = observation.bytes;
    let mut buffer = [0u8; 64 * 1024];
    while remaining > 0 {
        check()?;
        let allowed = usize::try_from(remaining.min(buffer.len() as u64)).map_err(storage_error)?;
        let read = match file.read(&mut buffer[..allowed]) {
            Ok(0) => return Err(ServiceError::AttunementSourceChanged),
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(storage_error(error)),
        };
        remaining -= read as u64;
        digest.update(&buffer[..read]);
        output.write_all(&buffer[..read]).map_err(storage_error)?;
    }
    check()?;
    if path
        .source_observation_for_file(&file)
        .map_err(storage_error)?
        != source
        || digest.finalize_hex() != *content_sha256
    {
        return Err(ServiceError::AttunementSourceChanged);
    }
    Ok(())
}
