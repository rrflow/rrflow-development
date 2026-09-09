use crate::key_codec::{prefix_end, KeyCodec};
use crate::keyspaces::{self, Space};
use crate::{Error, Result, StorageTransaction};
use serde::de::DeserializeOwned;

pub(super) fn checked_key(space: Space, key: &[u8]) -> Result<Vec<u8>> {
    keyspaces::validate_space(KeyCodec, space, key)?;
    Ok(key.to_vec())
}

pub(crate) trait RepositoryRead {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    fn scan_range(&self, start: &[u8], end: &[u8], limit: usize)
        -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

impl RepositoryRead for dyn StorageTransaction + '_ {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.get(key)
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: &[u8],
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.scan(start, end, limit)
    }
}

pub(super) fn get(
    transaction: &(impl RepositoryRead + ?Sized),
    space: Space,
    key: &[u8],
) -> Result<Option<Vec<u8>>> {
    transaction.read_key(&checked_key(space, key)?)
}

pub(super) fn get_json<T: DeserializeOwned>(
    transaction: &(impl RepositoryRead + ?Sized),
    space: Space,
    key: &[u8],
) -> Result<Option<T>> {
    get(transaction, space, key)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(Error::from))
        .transpose()
}

pub(super) fn read_sequence(
    transaction: &(impl RepositoryRead + ?Sized),
    key: &[u8],
) -> Result<u64> {
    get(transaction, keyspaces::META, key)?
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

pub(super) fn put_sequence(
    transaction: &mut dyn StorageTransaction,
    key: &[u8],
    sequence: u64,
) -> Result<()> {
    transaction.put(
        checked_key(keyspaces::META, key)?,
        sequence.to_string().into_bytes(),
    )
}

pub(super) fn scan_space(
    transaction: &(impl RepositoryRead + ?Sized),
    space: Space,
    prefix: &[u8],
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let start = if prefix.is_empty() {
        keyspaces::space_prefix(space)
    } else {
        checked_key(space, prefix)?
    };
    let end = prefix_end(&start)
        .ok_or_else(|| Error::Substrate("canonical key prefix has no upper bound".into()))?;
    transaction.scan_range(&start, &end, usize::MAX)
}

pub(super) fn scan_space_from(
    transaction: &(impl RepositoryRead + ?Sized),
    space: Space,
    from: &[u8],
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let start = checked_key(space, from)?;
    let prefix = keyspaces::space_prefix(space);
    let end = prefix_end(&prefix)
        .ok_or_else(|| Error::Substrate("canonical key space has no upper bound".into()))?;
    transaction.scan_range(&start, &end, usize::MAX)
}
