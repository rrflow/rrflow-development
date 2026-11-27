//! One bounded, hash-linked temporal selector for user and project configuration.
use super::*;

pub(super) fn decode(bytes: &[u8]) -> Result<ConfigurationRevision> {
    if bytes.len() > 64 * 1024 {
        return Err(unavailable("oversized configuration revision"));
    }
    let revision: ConfigurationRevision = serde_json::from_slice(bytes).map_err(contract_json)?;
    checked(revision.validate())?;
    Ok(revision)
}

pub(super) fn select(
    head: Option<ConfigurationRevision>,
    valid_at: u64,
    recorded_revision: Option<u64>,
    max_reads: u64,
    mut read: impl FnMut(u64) -> Result<Option<Vec<u8>>>,
) -> Result<(u64, Option<ConfigurationRevision>)> {
    let latest = head.as_ref().map_or(0, |head| head.revision);
    let known = recorded_revision.unwrap_or(latest);
    if known > latest || max_reads == 0 || max_reads > rrd_contract::MAX_CONFIGURATION_HISTORY_READS
    {
        return Err(unavailable(
            "requested configuration revision or history budget unavailable",
        ));
    }
    if known == 0 {
        return Ok((known, None));
    }
    let mut revision = head.ok_or_else(|| unavailable("configuration head missing"))?;
    for count in 0..max_reads {
        if revision.revision <= known && revision.change.contains(valid_at) {
            return Ok((known, Some(revision)));
        }
        if revision.revision == 1 {
            return Ok((known, None));
        }
        if count + 1 == max_reads {
            break;
        }
        let bytes = read(revision.revision - 1)?
            .ok_or_else(|| unavailable("configuration history incomplete"))?;
        let previous = decode(&bytes)?;
        if previous.source_id != revision.source_id
            || previous.revision + 1 != revision.revision
            || Some(&previous.revision_sha256) != revision.previous_sha256.as_ref()
            || previous.recorded_at_unix_ms > revision.recorded_at_unix_ms
        {
            return Err(unavailable("configuration history identity inconsistent"));
        }
        revision = previous;
    }
    Err(unavailable("configuration history read budget exhausted"))
}
