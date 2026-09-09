//! Shared encoded reads over one captured storage snapshot.

use crate::key_codec::{prefix_end, KeyCodec};
use crate::keyspaces::{self, Space};
use crate::{Error, Result, StorageTransaction};
use rrd_core::{
    ReadStamp, RuntimeChange, RuntimeChangePage, RuntimeLogAccumulator, RuntimeMerkleNode,
    RuntimeMutation, RuntimeReadValidation, RuntimeSchemaRegistry, ScopeId,
};
use serde::de::DeserializeOwned;

pub(crate) fn checked_key(space: Space, key: &[u8]) -> Result<Vec<u8>> {
    keyspaces::validate_space(KeyCodec, space, key)?;
    Ok(key.to_vec())
}

pub(crate) trait AccessRead {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    fn scan_range(&self, start: &[u8], end: &[u8], limit: usize)
        -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

impl AccessRead for dyn StorageTransaction + '_ {
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

pub(crate) fn get(
    reader: &(impl AccessRead + ?Sized),
    space: Space,
    key: &[u8],
) -> Result<Option<Vec<u8>>> {
    reader.read_key(&checked_key(space, key)?)
}

pub(crate) fn get_json<T: DeserializeOwned>(
    reader: &(impl AccessRead + ?Sized),
    space: Space,
    key: &[u8],
) -> Result<Option<T>> {
    get(reader, space, key)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(Error::from))
        .transpose()
}

pub(crate) fn read_sequence(reader: &(impl AccessRead + ?Sized), key: &[u8]) -> Result<u64> {
    get(reader, keyspaces::META, key)?
        .as_deref()
        .map(decode_sequence)
        .transpose()
        .map(Option::unwrap_or_default)
}

pub(crate) fn decode_sequence(value: &[u8]) -> Result<u64> {
    std::str::from_utf8(value)
        .map_err(|error| Error::CorruptWatermark(error.to_string()))?
        .parse::<u64>()
        .map_err(|error| Error::CorruptWatermark(error.to_string()))
}

pub(crate) fn put_sequence(
    transaction: &mut dyn StorageTransaction,
    key: &[u8],
    sequence: u64,
) -> Result<()> {
    transaction.put(
        checked_key(keyspaces::META, key)?,
        sequence.to_string().into_bytes(),
    )
}

pub(crate) fn scan_space(
    reader: &(impl AccessRead + ?Sized),
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
    reader.scan_range(&start, &end, usize::MAX)
}

pub(crate) fn scan_space_from(
    reader: &(impl AccessRead + ?Sized),
    space: Space,
    from: &[u8],
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let start = checked_key(space, from)?;
    let prefix = keyspaces::space_prefix(space);
    let end = prefix_end(&prefix)
        .ok_or_else(|| Error::Substrate("canonical key space has no upper bound".into()))?;
    reader.scan_range(&start, &end, usize::MAX)
}

pub(crate) fn read_stamp_with(
    reader: &(impl AccessRead + ?Sized),
    scope: &ScopeId,
) -> Result<ReadStamp> {
    let commit_cursor = read_sequence(reader, &keyspaces::runtime_cursor_key())?;
    let schema_revision = get_json::<RuntimeSchemaRegistry>(
        reader,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(scope),
    )?
    .map(|schema| schema.revision);
    let catalog_revision = read_sequence(reader, &keyspaces::catalog_revision_key(scope))?;
    let head_digest = get(
        reader,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    match load_accumulator(reader, commit_cursor)? {
        Some(accumulator) => ReadStamp::authenticated(
            scope.clone(),
            schema_revision,
            catalog_revision,
            commit_cursor,
            head_digest,
            accumulator.root,
        ),
        None => ReadStamp::new(
            scope.clone(),
            schema_revision,
            catalog_revision,
            commit_cursor,
            head_digest,
        ),
    }
    .map_err(Error::from)
}

pub(crate) fn validate_read_stamp(
    reader: &(impl AccessRead + ?Sized),
    read: &ReadStamp,
) -> Result<RuntimeReadValidation> {
    read.validate()?;
    let current = read_sequence(reader, &keyspaces::runtime_cursor_key())?;
    if read.commit_cursor > current {
        return Err(Error::ReadStampUnavailable(read.manifest_id.clone()));
    }
    if read.commit_cursor == current && read.accumulator_root.is_some() {
        let accumulator = load_accumulator(reader, current)?
            .ok_or_else(|| Error::ReadStampMismatch(read.manifest_id.clone()))?;
        let head_digest = get(
            reader,
            keyspaces::META,
            &keyspaces::runtime_last_digest_key(),
        )?
        .map(String::from_utf8)
        .transpose()
        .map_err(|error| Error::CorruptWatermark(error.to_string()))?
        .filter(|digest| !digest.is_empty());
        let schema_revision = get_json::<RuntimeSchemaRegistry>(
            reader,
            keyspaces::RUNTIME_SCHEMAS,
            &keyspaces::runtime_schema_key(&read.scope),
        )?
        .map(|schema| schema.revision);
        let catalog_revision =
            read_sequence(reader, &keyspaces::catalog_revision_key(&read.scope))?;
        if read.catalog_revision != catalog_revision
            || read.head_digest != head_digest
            || read.schema_revision != schema_revision
            || read.accumulator_root.as_deref() != Some(accumulator.root.as_str())
        {
            return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
        }
        return Ok(RuntimeReadValidation::new(
            "authenticated_current_head",
            0,
            0,
        ));
    }
    let retained_head = if read.commit_cursor == 0 {
        None
    } else {
        let change: RuntimeChange = get_json(
            reader,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(read.commit_cursor),
        )?
        .ok_or_else(|| Error::ReadStampUnavailable(read.manifest_id.clone()))?;
        if !change.verify_digest() {
            return Err(Error::Substrate(format!(
                "runtime change {} failed digest verification",
                read.commit_cursor
            )));
        }
        Some(change.digest)
    };
    let page = change_page(reader, read.commit_cursor, 0, usize::MAX, Some(&read.scope))?;
    let schema_revision = page
        .changes
        .iter()
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Schema { registry } => Some(registry.revision),
            _ => None,
        })
        .next_back();
    let catalog_revision = read_sequence(reader, &keyspaces::catalog_revision_key(&read.scope))?;
    if let Some(root) = read.accumulator_root.as_deref() {
        RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
            read_accumulator_node(reader, level, index)
        })?;
    }
    if read.catalog_revision != catalog_revision
        || read.head_digest != retained_head
        || read.schema_revision != schema_revision
    {
        return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
    }
    Ok(RuntimeReadValidation::new(
        "full_hash_chain_replay",
        read.commit_cursor,
        0,
    ))
}

pub(crate) fn accumulator_with(
    reader: &(impl AccessRead + ?Sized),
    expected_size: u64,
) -> Result<(RuntimeLogAccumulator, Vec<RuntimeMerkleNode>)> {
    if let Some(accumulator) = load_accumulator(reader, expected_size)? {
        return Ok((accumulator, Vec::new()));
    }
    let page = change_page(reader, expected_size, 0, usize::MAX, None)?;
    let mut accumulator = RuntimeLogAccumulator::new();
    let mut nodes = Vec::new();
    for change in &page.changes {
        nodes.extend(accumulator.append_change(change)?);
    }
    if accumulator.tree_size != expected_size {
        return Err(Error::Substrate(
            "runtime accumulator bootstrap did not cover the full log".into(),
        ));
    }
    Ok((accumulator, nodes))
}

fn load_accumulator(
    reader: &(impl AccessRead + ?Sized),
    expected_size: u64,
) -> Result<Option<RuntimeLogAccumulator>> {
    let stored: Option<RuntimeLogAccumulator> = get_json(
        reader,
        keyspaces::META,
        &keyspaces::runtime_accumulator_state_key(),
    )?;
    match stored {
        Some(accumulator) => {
            accumulator.validate()?;
            if accumulator.tree_size != expected_size {
                return Err(Error::Substrate(format!(
                    "runtime accumulator size {} differs from cursor {expected_size}",
                    accumulator.tree_size
                )));
            }
            Ok(Some(accumulator))
        }
        None if expected_size == 0 => Ok(Some(RuntimeLogAccumulator::new())),
        None => Ok(None),
    }
}

pub(crate) fn authenticated_point_page(
    reader: &(impl AccessRead + ?Sized),
    read: &ReadStamp,
    cursor: u64,
) -> Result<RuntimeChangePage> {
    let root = read
        .accumulator_root
        .as_deref()
        .ok_or_else(|| Error::ReadStampMismatch(read.manifest_id.clone()))?;
    let accumulator = match load_accumulator(reader, read.commit_cursor) {
        Ok(Some(accumulator)) if accumulator.root == root => accumulator,
        Ok(_) | Err(_) => {
            RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
                read_accumulator_node(reader, level, index)
            })?
        }
    };
    let change: RuntimeChange = get_json(
        reader,
        keyspaces::RUNTIME_CHANGES,
        &keyspaces::runtime_change_key(cursor),
    )?
    .ok_or_else(|| Error::ReadStampUnavailable(read.manifest_id.clone()))?;
    let proof = accumulator.inclusion_proof(cursor - 1, |level, index| {
        read_accumulator_node(reader, level, index)
    })?;
    let proof_nodes = proof.path.len();
    proof.verify_change(&change, root)?;
    let selected = (change.scope == read.scope).then_some(change);
    Ok(RuntimeChangePage {
        requested_after: cursor - 1,
        through_cursor: cursor,
        head_cursor: read.commit_cursor,
        validation: RuntimeReadValidation::new("rfc9162_inclusion_proof", 1, proof_nodes),
        changes: selected.into_iter().collect(),
    })
}

fn read_accumulator_node(
    reader: &(impl AccessRead + ?Sized),
    level: u8,
    index: u64,
) -> rrd_core::Result<Option<String>> {
    get(
        reader,
        keyspaces::META,
        &keyspaces::runtime_accumulator_node_key(level, index),
    )
    .map_err(|error| rrd_core::Error::InvalidRuntime {
        reason: format!("cannot read runtime accumulator node: {error}"),
    })?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| rrd_core::Error::InvalidRuntime {
        reason: format!("runtime accumulator node is not UTF-8: {error}"),
    })
}

pub(crate) fn change_page(
    reader: &(impl AccessRead + ?Sized),
    head: u64,
    after: u64,
    limit: usize,
    scope: Option<&ScopeId>,
) -> Result<RuntimeChangePage> {
    if limit == 0 {
        return Err(Error::Substrate(
            "runtime change page limit must be greater than zero".into(),
        ));
    }
    if after == u64::MAX || after >= head {
        return Ok(RuntimeChangePage {
            requested_after: after,
            through_cursor: after,
            head_cursor: head,
            validation: RuntimeReadValidation::new("bounded_hash_chain_page", 0, 0),
            changes: Vec::new(),
        });
    }
    let mut previous_digest = if after == 0 {
        None
    } else {
        let prior: RuntimeChange = get_json(
            reader,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(after),
        )?
        .ok_or_else(|| Error::Substrate(format!("runtime log is missing cursor {after}")))?;
        if !prior.verify_digest() {
            return Err(Error::Substrate(format!(
                "runtime change {after} failed digest verification"
            )));
        }
        Some(prior.digest)
    };
    let mut through = after;
    let mut selected = Vec::new();
    for expected in after + 1..=head {
        if through.saturating_sub(after) as usize >= limit {
            break;
        }
        let change: RuntimeChange = get_json(
            reader,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(expected),
        )?
        .ok_or_else(|| Error::Substrate(format!("runtime log is missing cursor {expected}")))?;
        if change.cursor != expected
            || change.previous_digest != previous_digest
            || !change.verify_digest()
        {
            return Err(Error::Substrate(format!(
                "runtime change {expected} failed cursor/hash-chain verification"
            )));
        }
        through = expected;
        previous_digest = Some(change.digest.clone());
        if scope.is_none_or(|scope| scope == &change.scope) {
            selected.push(change);
        }
    }
    Ok(RuntimeChangePage {
        requested_after: after,
        through_cursor: through,
        head_cursor: head,
        validation: RuntimeReadValidation::new(
            "bounded_hash_chain_page",
            through.saturating_sub(after),
            0,
        ),
        changes: selected,
    })
}

pub(crate) fn values_for_scope<T: DeserializeOwned>(
    reader: &(impl AccessRead + ?Sized),
    space: Space,
    scope: &ScopeId,
) -> Result<Vec<T>> {
    let prefix = keyspaces::runtime_scope_prefix(space, scope);
    scan_space(reader, space, &prefix)?
        .into_iter()
        .map(|(_, value)| serde_json::from_slice(&value).map_err(Error::from))
        .collect()
}
