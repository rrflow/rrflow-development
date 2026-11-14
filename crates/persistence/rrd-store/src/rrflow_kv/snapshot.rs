//! Runtime checkpoints, read stamps and snapshot artifact views.

use super::*;

pub(super) fn reconcile_runtime_checkpoints(
    database: &mut Database,
    now: Option<Millis>,
    at: Millis,
) -> Result<()> {
    let snapshot = database.snapshot();
    let handles = scan_space(database, snapshot, keyspaces::RUNTIME_SNAPSHOTS, &[])?
        .into_iter()
        .map(|(_, bytes)| serde_json::from_slice::<SnapshotHandle>(&bytes).map_err(Error::from))
        .collect::<Result<Vec<_>>>()?;
    for handle in &handles {
        handle.validate()?;
    }
    let desired = handles
        .iter()
        .filter(|handle| now.is_none_or(|now| !handle.is_expired(now)))
        .map(|handle| runtime_checkpoint_name(&handle.id))
        .collect::<BTreeSet<_>>();
    let checkpoints = database.checkpoints()?;
    let existing = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.name.starts_with(RUNTIME_CHECKPOINT_PREFIX))
        .map(|checkpoint| checkpoint.name.clone())
        .collect::<BTreeSet<_>>();
    // Canonical live handles decide reachability. Release only obsolete runtime
    // pins before admitting missing pins so the quota cannot block reconciliation.
    for name in existing.difference(&desired) {
        database.release_checkpoint(name)?;
    }
    let missing = desired.difference(&existing).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        let created_at = at.max(database.manifest().created_at);
        database.flush_memtable(created_at)?;
        for name in missing {
            database.checkpoint(&name, created_at)?;
        }
    }
    Ok(())
}

/// Materializes the physical manifest pin before its logical snapshot handle
/// can commit. A failed semantic transaction removes checkpoints it created;
/// an interrupted pre-publication pin is reclaimed by open-time reconciliation.
pub(super) fn ensure_runtime_checkpoint(
    database: &mut Database,
    handle: &SnapshotHandle,
    at: Millis,
) -> Result<bool> {
    let name = runtime_checkpoint_name(&handle.id);
    if database
        .checkpoints()?
        .iter()
        .any(|checkpoint| checkpoint.name == name)
    {
        return Ok(false);
    }
    let created_at = at.max(database.manifest().created_at);
    database.flush_memtable(created_at)?;
    database.checkpoint(&name, created_at)?;
    Ok(true)
}

pub(super) fn runtime_checkpoint_name(id: &SnapshotId) -> String {
    format!("{RUNTIME_CHECKPOINT_PREFIX}{}", id.as_str())
}

pub(super) fn rrflow_kv_read_stamp(
    database: &Database,
    snapshot: Snapshot,
    scope: &ScopeId,
) -> Result<ReadStamp> {
    crate::access::runtime_state::read_stamp_with(
        &RrflowKvSnapshotRead { database, snapshot },
        scope,
    )
}

/// Reads the exact immutable-object closure for one scope from an authenticated
/// physical snapshot before that snapshot is installed on a replica.
pub fn rrflow_kv_snapshot_object_references(
    bundle: &SnapshotBundleFile,
    scope: &ScopeId,
) -> Result<Vec<ObjectReference>> {
    rrflow_kv_snapshot_artifact_view(bundle, scope).map(|(_, objects)| objects)
}

/// Reads the exact project read stamp and immutable-object closure directly
/// from an authenticated physical snapshot.
pub fn rrflow_kv_snapshot_artifact_view(
    bundle: &SnapshotBundleFile,
    scope: &ScopeId,
) -> Result<(ReadStamp, Vec<ObjectReference>)> {
    snapshot_codec(bundle)?;
    let cursor_key = keyspaces::runtime_cursor_key();
    let digest_key = keyspaces::runtime_last_digest_key();
    let accumulator_key = keyspaces::runtime_accumulator_state_key();
    let schema_key = keyspaces::runtime_schema_key(scope);
    let values = bundle
        .get_many(&[&cursor_key, &digest_key, &accumulator_key, &schema_key])
        .map_err(Error::from)?;
    let commit_cursor = values[0]
        .as_deref()
        .map(decode_sequence)
        .transpose()?
        .unwrap_or_default();
    let head_digest = values[1]
        .clone()
        .map(String::from_utf8)
        .transpose()
        .map_err(|error| Error::CorruptWatermark(error.to_string()))?
        .filter(|digest| !digest.is_empty());
    let accumulator = values[2]
        .as_deref()
        .map(serde_json::from_slice::<RuntimeLogAccumulator>)
        .transpose()?;
    if let Some(accumulator) = &accumulator {
        accumulator.validate()?;
        if accumulator.tree_size != commit_cursor {
            return Err(Error::Substrate(format!(
                "snapshot runtime accumulator size {} differs from cursor {commit_cursor}",
                accumulator.tree_size
            )));
        }
    }
    let schema_revision = values[3]
        .as_deref()
        .map(serde_json::from_slice::<RuntimeSchemaRegistry>)
        .transpose()?
        .map(|schema| schema.revision);
    let read = match accumulator {
        Some(accumulator) => ReadStamp::authenticated(
            scope.clone(),
            schema_revision,
            0,
            commit_cursor,
            head_digest,
            accumulator.root,
        )?,
        None if commit_cursor == 0 => ReadStamp::authenticated(
            scope.clone(),
            schema_revision,
            0,
            commit_cursor,
            head_digest,
            RuntimeLogAccumulator::new().root,
        )?,
        None => ReadStamp::new(
            scope.clone(),
            schema_revision,
            0,
            commit_cursor,
            head_digest,
        )?,
    };
    let objects = rrflow_kv_snapshot_objects(bundle, Some(scope))?;
    Ok((read, objects))
}

/// Reads the project artifact view from a live rrflowKV database snapshot. This
/// is used by the cluster adapter without reopening a second database handle.
pub fn rrflow_kv_database_artifact_view(
    database: &Database,
    scope: &ScopeId,
) -> Result<(ReadStamp, Vec<ObjectReference>)> {
    let snapshot = database.snapshot();
    let read = rrflow_kv_read_stamp(database, snapshot, scope)?;
    let rows = scan_space(database, snapshot, keyspaces::RUNTIME_OBJECTS, &[])?;
    let objects = decode_snapshot_objects(database_codec(database)?, rows, Some(scope))?;
    Ok((read, objects))
}

/// Reads every immutable reference in a physical snapshot. Unlike the
/// project-specific transfer view, this permits multiple scopes and is the
/// final target-side activation gate.
pub fn rrflow_kv_snapshot_all_object_references(
    bundle: &SnapshotBundleFile,
) -> Result<Vec<ObjectReference>> {
    rrflow_kv_snapshot_objects(bundle, None)
}

pub(super) fn rrflow_kv_snapshot_objects(
    bundle: &SnapshotBundleFile,
    required_scope: Option<&ScopeId>,
) -> Result<Vec<ObjectReference>> {
    let codec = snapshot_codec(bundle)?;
    let start = keyspaces::space_prefix(keyspaces::RUNTIME_OBJECTS);
    let end = prefix_end(&start);
    let values = bundle.scan(&start, end.as_deref()).map_err(Error::from)?;
    decode_snapshot_objects(codec, values, required_scope)
}

pub(super) fn decode_snapshot_objects(
    codec: keyspaces::RrflowKvKeyCodec,
    values: Vec<(Vec<u8>, Vec<u8>)>,
    required_scope: Option<&ScopeId>,
) -> Result<Vec<ObjectReference>> {
    if values.len() > 1_000_000 {
        return Err(Error::Substrate(
            "rrflowKV snapshot object-reference limit exceeded".into(),
        ));
    }
    let mut objects = values
        .into_iter()
        .map(|(stored_key, value)| {
            let object: ObjectReference = serde_json::from_slice(&value)?;
            object.validate()?;
            let (encoded_scope, encoded_reference) = keyspaces::parse_runtime_identity_key(
                codec,
                keyspaces::RUNTIME_OBJECTS,
                &stored_key,
            )?;
            if required_scope.is_some_and(|scope| scope != &encoded_scope) {
                return Err(Error::Substrate(
                    "rrflowKV snapshot object project scope differs from the transfer".into(),
                ));
            }
            let expected = keyspaces::runtime_identity_key(
                keyspaces::RUNTIME_OBJECTS,
                &encoded_scope,
                &object.reference,
            );
            if stored_key != expected || encoded_reference != object.reference {
                return Err(Error::Substrate(
                    "rrflowKV snapshot object key/value identity differs from its canonical reference"
                        .into(),
                ));
            }
            Ok(object)
        })
        .collect::<Result<Vec<_>>>()?;
    objects.sort_by(|left, right| left.reference.cmp(&right.reference));
    if required_scope.is_some()
        && objects
            .windows(2)
            .any(|pair| pair[0].reference >= pair[1].reference)
    {
        return Err(Error::Substrate(
            "rrflowKV snapshot contains duplicate object references".into(),
        ));
    }
    Ok(objects)
}
