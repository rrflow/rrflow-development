//! Sequence, key and codec helpers.

use super::*;

#[cfg(test)]
pub(super) fn read_sequence(database: &Database, snapshot: Snapshot, key: &[u8]) -> Result<u64> {
    get(database, snapshot, keyspaces::META, key)?
        .as_deref()
        .map(decode_sequence)
        .transpose()
        .map(Option::unwrap_or_default)
}

pub(super) fn decode_sequence(value: &[u8]) -> Result<u64> {
    std::str::from_utf8(value)
        .map_err(|error| Error::CorruptWatermark(error.to_string()))?
        .parse::<u64>()
        .map_err(|error| Error::CorruptWatermark(error.to_string()))
}

pub(super) fn get(
    database: &Database,
    snapshot: Snapshot,
    space: keyspaces::Space,
    key: &[u8],
) -> Result<Option<Vec<u8>>> {
    database
        .get(&encoded_storage_key(database, space, key)?, snapshot)
        .map_err(Error::from)
}

pub(super) fn get_json<T: DeserializeOwned>(
    database: &Database,
    snapshot: Snapshot,
    space: keyspaces::Space,
    key: &[u8],
) -> Result<Option<T>> {
    get(database, snapshot, space, key)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(Error::from))
        .transpose()
}

pub(super) fn scan_space(
    database: &Database,
    snapshot: Snapshot,
    space: keyspaces::Space,
    prefix: &[u8],
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let start = if prefix.is_empty() {
        database_codec(database)?;
        keyspaces::space_prefix(space)
    } else {
        encoded_storage_key(database, space, prefix)?
    };
    let end = prefix_end(&start);
    database
        .scan(&start, end.as_deref(), snapshot)
        .map_err(Error::from)
}

#[cfg(test)]
pub(super) fn storage_key(space: keyspaces::Space, key: &[u8]) -> Vec<u8> {
    keyspaces::validate_space(KeyCodec, space, key)
        .expect("rrflowKV mutations use a canonical typed key");
    key.to_vec()
}

pub(super) fn database_codec(database: &Database) -> Result<keyspaces::RrflowKvKeyCodec> {
    keyspaces::RrflowKvKeyCodec::from_application_format(database.manifest().application_format)
        .ok_or_else(|| {
            Error::Substrate(format!(
                "unsupported rrflowKV application format {:?}",
                database.manifest().application_format
            ))
        })
}

pub(super) fn snapshot_codec(bundle: &SnapshotBundleFile) -> Result<keyspaces::RrflowKvKeyCodec> {
    keyspaces::RrflowKvKeyCodec::from_application_format(bundle.source_manifest.application_format)
        .ok_or_else(|| {
            Error::Substrate(format!(
                "unsupported rrflowKV snapshot application format {:?}",
                bundle.source_manifest.application_format
            ))
        })
}

pub(super) fn encoded_storage_key(
    database: &Database,
    space: keyspaces::Space,
    key: &[u8],
) -> Result<Vec<u8>> {
    let codec = database_codec(database)?;
    keyspaces::validate_space(codec, space, key)?;
    Ok(key.to_vec())
}
