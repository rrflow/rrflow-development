//! Persistent rrflowKV implementation of the physical [`StorageEngine`] port.
//!
//! Semantic repositories encode logical keyspaces as stable byte prefixes and
//! publish through this store's snapshot transaction. The database's physical
//! MVCC sequence is deliberately independent of claim and runtime cursors
//! stored in those transactions.

use crate::access::{prepare_semantic_commit, SemanticCommitPlan};
use crate::engine::{PhysicalStoreEvidence, StorageEngine};
use crate::error::{Error, Result};
use crate::key_codec::prefix_end;
#[cfg(test)]
use crate::key_codec::KeyCodec;
use crate::keyspaces::{self, Durability};
use crate::transaction::{StorageTransaction, TransactionCommit, TransactionRollback};
use rrd_core::{
    AuditEnvelope, Millis, ObjectReference, ReadStamp, RuntimeCommit, RuntimeCommitOutcome,
    RuntimeLogAccumulator, RuntimeSchemaRegistry, ScopeId, SnapshotHandle, SnapshotId,
};
use rrd_lsm::{
    CompactionOutcome, Database, DatabaseOptions, GarbageCollectionReport, Manifest, Mutation,
    Snapshot, SnapshotBundleFile,
};
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

const RUNTIME_CHECKPOINT_PREFIX: &str = "runtime-";

pub struct RrflowKvStore {
    path: PathBuf,
    database: Mutex<Database>,
}

struct RrflowKvTransaction<'a> {
    store: &'a RrflowKvStore,
    transaction: rrd_lsm::Transaction,
    runtime_snapshot_writes: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
}

impl StorageTransaction for RrflowKvTransaction<'_> {
    fn snapshot_sequence(&self) -> u64 {
        self.transaction.snapshot().sequence
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let database = self.store.lock()?;
        self.transaction.get(&database, key).map_err(Error::from)
    }

    fn scan(&self, start: &[u8], end: &[u8], limit: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let database = self.store.lock()?;
        self.transaction
            .scan(&database, start, end, limit)
            .map_err(Error::from)
    }

    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.transaction.put(key.clone(), value.clone())?;
        if key.starts_with(&keyspaces::space_prefix(keyspaces::RUNTIME_SNAPSHOTS)) {
            self.runtime_snapshot_writes.insert(key, Some(value));
        }
        Ok(())
    }

    fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        self.transaction.delete(key.clone())?;
        if key.starts_with(&keyspaces::space_prefix(keyspaces::RUNTIME_SNAPSHOTS)) {
            self.runtime_snapshot_writes.insert(key, None);
        }
        Ok(())
    }

    fn commit(self: Box<Self>, durability: Durability) -> Result<TransactionCommit> {
        let Self {
            store,
            transaction,
            runtime_snapshot_writes,
        } = *self;
        let mut database = store.lock()?;
        let snapshot_handles = runtime_snapshot_writes
            .values()
            .flatten()
            .map(|bytes| {
                let handle: SnapshotHandle = serde_json::from_slice(bytes)?;
                handle.validate()?;
                Ok(handle)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut created_checkpoints = Vec::new();
        for handle in &snapshot_handles {
            match ensure_runtime_checkpoint(&mut database, handle, handle.created_at) {
                Ok(true) => created_checkpoints.push(runtime_checkpoint_name(&handle.id)),
                Ok(false) => {}
                Err(error) => {
                    for name in created_checkpoints {
                        let _ = database.release_checkpoint(&name);
                    }
                    return Err(error);
                }
            }
        }
        let outcome = match database.commit_transaction(
            transaction,
            match durability {
                Durability::Authoritative => rrd_lsm::Durability::Authoritative,
                Durability::Buffered => rrd_lsm::Durability::Buffered,
            },
        ) {
            Ok(outcome) => outcome,
            Err(error) => {
                for name in created_checkpoints {
                    let _ = database.release_checkpoint(&name);
                }
                return Err(error.into());
            }
        };
        if runtime_snapshot_writes.values().any(Option::is_none) {
            reconcile_runtime_checkpoints(&mut database, None, 0)?;
        }
        let (first_sequence, last_sequence) = outcome
            .receipt
            .as_ref()
            .map(|receipt| (Some(receipt.first_sequence), Some(receipt.last_sequence)))
            .unwrap_or((None, None));
        tracing::debug!(
            target: "rrd_store::transaction",
            storage_profile = "rrflow_kv",
            snapshot_sequence = outcome.snapshot_sequence,
            mutation_count = outcome.mutation_count,
            first_sequence,
            last_sequence,
            outcome = if outcome.receipt.is_some() { "committed" } else { "read_only" },
            "storage transaction completed"
        );
        Ok(TransactionCommit {
            snapshot_sequence: outcome.snapshot_sequence,
            mutation_count: outcome.mutation_count,
            first_sequence,
            last_sequence,
        })
    }

    fn rollback(self: Box<Self>) -> Result<TransactionRollback> {
        let outcome = self.transaction.rollback()?;
        Ok(TransactionRollback {
            snapshot_sequence: outcome.snapshot_sequence,
            discarded_mutations: outcome.discarded_mutations,
        })
    }
}

impl RrflowKvStore {
    /// Opens an existing rrflowKV database or creates one when `path` is absent.
    /// An existing but invalid directory fails closed rather than being
    /// silently reinitialized.
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_options(path, DatabaseOptions::default())
    }

    /// Opens with explicit rrflowKV cache and mutable-state bounds. Persistent
    /// format identity is unchanged; these are process-local operating limits.
    pub fn open_with_options(path: &Path, options: DatabaseOptions) -> Result<Self> {
        let total_started = Instant::now();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| Error::Substrate(error.to_string()))?;
            }
        }
        let empty = path.exists()
            && path.is_dir()
            && std::fs::read_dir(path)
                .map_err(|error| Error::Substrate(error.to_string()))?
                .next()
                .is_none();
        let database_started = Instant::now();
        let mut database = if !path.exists() || empty {
            Database::create_with_application_format(path, options, keyspaces::RRFLOW_KV_FORMAT)?
        } else {
            Database::open_with_options(path, options).map_err(|error| {
                Error::Substrate(format!(
                    "cannot open rrflowKV database {}: {error}",
                    path.display()
                ))
            })?
        };
        let database_open_ms = database_started.elapsed().as_millis() as u64;
        let _codec = keyspaces::RrflowKvKeyCodec::from_application_format(
            database.manifest().application_format,
        )
        .ok_or_else(|| {
            Error::Substrate(format!(
                "unsupported rrflowKV application format {:?}",
                database.manifest().application_format
            ))
        })?;
        let checkpoints_started = Instant::now();
        reconcile_runtime_checkpoints(&mut database, None, 0).map_err(|error| {
            Error::Substrate(format!(
                "cannot reconcile runtime checkpoints while opening {}: {error}",
                path.display()
            ))
        })?;
        let checkpoint_reconcile_ms = checkpoints_started.elapsed().as_millis() as u64;
        tracing::info!(
            target: "rrflow_kv::open",
            path = %path.display(),
            database_open_ms,
            checkpoint_reconcile_ms,
            total_ms = total_started.elapsed().as_millis() as u64,
            "rrflowKV store open phases completed"
        );
        let path = database.root().to_owned();
        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Publishes the current rrflowKV memtable as an immutable segment.
    pub fn flush(&self, at: Millis) -> Result<Option<Manifest>> {
        self.lock()?.flush_memtable(at).map_err(Error::from)
    }

    pub fn manifest(&self) -> Result<Manifest> {
        Ok(self.lock()?.manifest().clone())
    }

    /// Compacts rrflowKV state after reconciling physical manifest pins with the
    /// authoritative logical snapshot catalog.
    pub fn compact(&self, now: Millis, at: Millis) -> Result<Option<CompactionOutcome>> {
        let mut database = self.lock()?;
        reconcile_runtime_checkpoints(&mut database, Some(now), at)?;
        database.compact(&[], at).map_err(Error::from)
    }

    /// Reclaims only objects unreachable from `CURRENT` or a live runtime
    /// snapshot's physical checkpoint.
    pub fn garbage_collect(&self, now: Millis, at: Millis) -> Result<GarbageCollectionReport> {
        let mut database = self.lock()?;
        reconcile_runtime_checkpoints(&mut database, Some(now), at)?;
        database.garbage_collect().map_err(Error::from)
    }

    /// Replays one already-authenticated logical-archive commit while
    /// preserving its original audit envelope. This is deliberately crate
    /// private: ordinary callers must use the live `StorageEngine` transaction
    /// path, which validates a read stamp against the current database state.
    pub(crate) fn restore_runtime_commit(
        &self,
        commit: &RuntimeCommit,
        audit: &AuditEnvelope,
    ) -> Result<RuntimeCommitOutcome> {
        self.runtime().restore_commit(commit, audit)
    }

    fn lock(&self) -> Result<MutexGuard<'_, Database>> {
        self.database
            .lock()
            .map_err(|_| Error::Substrate("rrflowKV database mutex poisoned".into()))
    }
}

impl StorageEngine for RrflowKvStore {
    fn begin_transaction(&self) -> Result<Box<dyn StorageTransaction + '_>> {
        let transaction = {
            let database = self.lock()?;
            database.begin_transaction()?
        };
        tracing::debug!(
            target: "rrd_store::transaction",
            storage_profile = "rrflow_kv",
            snapshot_sequence = transaction.snapshot().sequence,
            outcome = "begun",
            "storage transaction began"
        );
        Ok(Box::new(RrflowKvTransaction {
            store: self,
            transaction,
            runtime_snapshot_writes: BTreeMap::new(),
        }))
    }

    fn claims(&self) -> crate::ClaimRepository<'_> {
        crate::ClaimRepository::new(self)
    }

    fn control(&self) -> crate::ControlRepository<'_> {
        crate::ControlRepository::new(self)
    }

    fn function_catalogue(&self) -> crate::FunctionCatalogueRepository<'_> {
        crate::FunctionCatalogueRepository::new(self)
    }

    fn projections(&self) -> crate::ProjectionRepository<'_> {
        crate::ProjectionRepository::new(self)
    }

    fn runtime(&self) -> crate::RuntimeRepository<'_> {
        crate::RuntimeRepository::new(self)
    }

    fn invocations(&self) -> crate::InvocationRepository<'_> {
        crate::InvocationRepository::new(self)
    }

    fn physical_store_evidence(&self) -> Result<PhysicalStoreEvidence> {
        let database = self.lock()?;
        let manifest = database.manifest();
        let cache = database.block_cache_stats();
        let segment_io = database.segment_io_stats();
        let maintenance = database.maintenance_policy();
        let compaction = database.compaction_policy();
        let maintenance_stats = database.maintenance_stats();
        let memtable = database.memtable().profile();
        Ok(PhysicalStoreEvidence {
            backend: "rrflow_kv".into(),
            evidence_level: "rrflow_kv_counters".into(),
            physical_sequence: Some(database.snapshot().sequence),
            manifest_generation: Some(manifest.generation),
            durable_sequence: Some(manifest.durable_sequence),
            memtable_versions: Some(database.memtable().version_count() as u64),
            memtable_bytes: Some(database.memtable().approximate_bytes() as u64),
            memtable_keys: Some(memtable.key_count as u64),
            memtable_key_payload_bytes: Some(memtable.key_payload_bytes as u64),
            memtable_value_payload_bytes: Some(memtable.value_payload_bytes as u64),
            memtable_tombstones: Some(memtable.tombstones as u64),
            memtable_spilled_chains: Some(memtable.spilled_chains as u64),
            memtable_spilled_version_capacity: Some(memtable.spilled_version_capacity as u64),
            memtable_version_record_bytes: Some(memtable.version_record_bytes as u64),
            memtable_owned_bytes_lower_bound: Some(memtable.owned_bytes_lower_bound as u64),
            memtable_max_versions: Some(maintenance.memtable_max_versions as u64),
            wal_payload_bytes: Some(database.wal_payload_bytes() as u64),
            wal_payload_max_bytes: Some(maintenance.wal_payload_max_bytes as u64),
            automatic_flushes: Some(maintenance_stats.automatic_flushes),
            maintenance_write_stalls: Some(maintenance_stats.write_stalls),
            failed_maintenance_flushes: Some(maintenance_stats.failed_flushes),
            oversized_batches: Some(maintenance_stats.oversized_batches),
            automatic_compactions: Some(maintenance_stats.automatic_compactions),
            failed_compactions: Some(maintenance_stats.failed_compactions),
            compaction_input_bytes: Some(maintenance_stats.compaction_input_bytes),
            compaction_output_bytes: Some(maintenance_stats.compaction_output_bytes),
            peak_compaction_buffer_bytes: Some(
                maintenance_stats.peak_compaction_buffer_bytes as u64,
            ),
            l0_segment_count: Some(database.l0_segment_count() as u64),
            l0_compaction_trigger: Some(compaction.l0_compaction_trigger as u64),
            compaction_debt_segments: Some(database.compaction_debt_segments() as u64),
            compaction_target_segment_bytes: Some(compaction.target_segment_bytes as u64),
            segment_count: Some(manifest.segments.len() as u64),
            segment_bytes: Some(manifest.segments.iter().map(|segment| segment.bytes).sum()),
            cache_capacity_bytes: Some(cache.capacity_bytes as u64),
            cache_resident_bytes: Some(cache.resident_bytes as u64),
            cache_entries: Some(cache.entries as u64),
            cache_hits: Some(cache.hits),
            cache_misses: Some(cache.misses),
            cache_evictions: Some(cache.evictions),
            block_loads: Some(cache.loads),
            block_bytes_loaded: Some(cache.bytes_loaded),
            block_bytes_decoded: Some(cache.bytes_decoded),
            filter_checks: Some(cache.filter_checks),
            filter_negatives: Some(cache.filter_negatives),
            segment_io_requested_mode: Some(segment_io.requested_mode.as_str().into()),
            segment_io_mmap_segments: Some(segment_io.mmap_segments),
            segment_io_uring_segments: Some(segment_io.io_uring_segments),
            segment_io_bounded_segments: Some(segment_io.bounded_segments),
            segment_io_fallbacks: Some(segment_io.fallback_count),
            segment_io_last_fallback: segment_io.last_fallback_reason,
            segment_io_read_operations: Some(segment_io.read_operations),
            segment_io_mmap_reads: Some(segment_io.mmap_read_operations),
            segment_io_uring_reads: Some(segment_io.io_uring_read_operations),
            segment_io_bounded_reads: Some(segment_io.bounded_read_operations),
            segment_io_bytes_read: Some(segment_io.bytes_read),
            segment_io_max_request_bytes: Some(segment_io.configured_max_request_bytes as u64),
            segment_io_peak_request_bytes: Some(segment_io.peak_request_bytes as u64),
        })
    }
}

/// Converts the one profile-neutral encoded semantic plan into the complete
/// rrflowKV mutation vector used by the temporary Raft application bridge.
///
/// The bridge appends coordinator metadata to this complete vector. Its raw
/// access remains an inventoried distributed-authority gap; centralizing the
/// semantic plan here removes the second planner but does not close that gap.
impl SemanticCommitPlan {
    pub fn into_parts_for(
        self,
        database: &Database,
    ) -> Result<(RuntimeCommitOutcome, Vec<Mutation>)> {
        database_codec(database)?;
        let (outcome, writes) = self.into_encoded_writes();
        let operations = writes
            .into_iter()
            .map(|(key, value)| match value {
                Some(value) => Mutation::Put { key, value },
                None => Mutation::Delete { key },
            })
            .collect();
        Ok((outcome, operations))
    }
}

struct RrflowKvSnapshotRead<'a> {
    database: &'a Database,
    snapshot: Snapshot,
}

impl crate::access::runtime_state::AccessRead for RrflowKvSnapshotRead<'_> {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.database.get(key, self.snapshot).map_err(Error::from)
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: &[u8],
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        Ok(self
            .database
            .scan(start, Some(end), self.snapshot)
            .map_err(Error::from)?
            .into_iter()
            .take(limit)
            .collect())
    }
}

/// Reads the exact rrflowKV cursor/schema pair used by the temporary Raft
/// application bridge before it submits the complete semantic plan.
pub fn rrflow_kv_commit_context(
    database: &Database,
    scope: &ScopeId,
) -> Result<(ReadStamp, Option<RuntimeSchemaRegistry>)> {
    let reader = RrflowKvSnapshotRead {
        database,
        snapshot: database.snapshot(),
    };
    let read = crate::access::runtime_state::read_stamp_with(&reader, scope)?;
    let schema = crate::access::runtime_state::get_json(
        &reader,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(scope),
    )?;
    if schema
        .as_ref()
        .map(|value: &RuntimeSchemaRegistry| value.revision)
        != read.schema_revision
    {
        return Err(Error::Substrate(
            "rrflowKV runtime schema differs from its read stamp".into(),
        ));
    }
    Ok((read, schema))
}

/// Validates and lowers one canonical [`RuntimeCommit`] into the same encoded
/// semantic plan used by rrflowMX and ordinary rrflowKV repositories. This
/// remains public only for the already-inventoried Raft application path.
pub fn prepare_rrflow_kv_commit(
    database: &Database,
    commit: &RuntimeCommit,
) -> Result<SemanticCommitPlan> {
    prepare_rrflow_kv_commit_at_read(database, commit, None, None)
}

fn prepare_rrflow_kv_commit_at_read(
    database: &Database,
    commit: &RuntimeCommit,
    read: Option<&ReadStamp>,
    archived_audit: Option<&AuditEnvelope>,
) -> Result<SemanticCommitPlan> {
    let reader = RrflowKvSnapshotRead {
        database,
        snapshot: database.snapshot(),
    };
    prepare_semantic_commit(&reader, commit, read, archived_audit, None)
}

/// Reads a previously accepted rrflowKV runtime outcome from a caller-held
/// database snapshot. Coordinators use this before planning so content-addressed
/// retries remain idempotent even when the transport request id changes.
pub fn rrflow_kv_commit_outcome(
    database: &Database,
    commit_id: &str,
) -> Result<Option<RuntimeCommitOutcome>> {
    let outcome: Option<RuntimeCommitOutcome> = get_json(
        database,
        database.snapshot(),
        keyspaces::RUNTIME_COMMITS,
        &keyspaces::runtime_commit_key(commit_id),
    )?;
    if outcome
        .as_ref()
        .is_some_and(|outcome| outcome.commit_id != commit_id)
    {
        return Err(Error::Substrate(
            "runtime commit outcome key does not match its content identity".into(),
        ));
    }
    Ok(outcome)
}

fn reconcile_runtime_checkpoints(
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
    let missing = desired.difference(&existing).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        let created_at = at.max(database.manifest().created_at);
        database.flush_memtable(created_at)?;
        for name in missing {
            database.checkpoint(&name, created_at)?;
        }
    }
    for name in existing.difference(&desired) {
        database.release_checkpoint(name)?;
    }
    Ok(())
}

/// Materializes the physical manifest pin before its logical snapshot handle
/// can commit. A failed semantic transaction removes checkpoints it created;
/// an interrupted pre-publication pin is reclaimed by open-time reconciliation.
fn ensure_runtime_checkpoint(
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

fn runtime_checkpoint_name(id: &SnapshotId) -> String {
    format!("{RUNTIME_CHECKPOINT_PREFIX}{}", id.as_str())
}

fn rrflow_kv_read_stamp(
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

fn rrflow_kv_snapshot_objects(
    bundle: &SnapshotBundleFile,
    required_scope: Option<&ScopeId>,
) -> Result<Vec<ObjectReference>> {
    let codec = snapshot_codec(bundle)?;
    let start = keyspaces::space_prefix(keyspaces::RUNTIME_OBJECTS);
    let end = prefix_end(&start);
    let values = bundle.scan(&start, end.as_deref()).map_err(Error::from)?;
    decode_snapshot_objects(codec, values, required_scope)
}

fn decode_snapshot_objects(
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

#[cfg(test)]
fn read_sequence(database: &Database, snapshot: Snapshot, key: &[u8]) -> Result<u64> {
    get(database, snapshot, keyspaces::META, key)?
        .as_deref()
        .map(decode_sequence)
        .transpose()
        .map(Option::unwrap_or_default)
}

fn decode_sequence(value: &[u8]) -> Result<u64> {
    std::str::from_utf8(value)
        .map_err(|error| Error::CorruptWatermark(error.to_string()))?
        .parse::<u64>()
        .map_err(|error| Error::CorruptWatermark(error.to_string()))
}

fn get(
    database: &Database,
    snapshot: Snapshot,
    space: keyspaces::Space,
    key: &[u8],
) -> Result<Option<Vec<u8>>> {
    database
        .get(&encoded_storage_key(database, space, key)?, snapshot)
        .map_err(Error::from)
}

fn get_json<T: DeserializeOwned>(
    database: &Database,
    snapshot: Snapshot,
    space: keyspaces::Space,
    key: &[u8],
) -> Result<Option<T>> {
    get(database, snapshot, space, key)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(Error::from))
        .transpose()
}

fn scan_space(
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
fn storage_key(space: keyspaces::Space, key: &[u8]) -> Vec<u8> {
    keyspaces::validate_space(KeyCodec, space, key)
        .expect("rrflowKV mutations use a canonical typed key");
    key.to_vec()
}

fn database_codec(database: &Database) -> Result<keyspaces::RrflowKvKeyCodec> {
    keyspaces::RrflowKvKeyCodec::from_application_format(database.manifest().application_format)
        .ok_or_else(|| {
            Error::Substrate(format!(
                "unsupported rrflowKV application format {:?}",
                database.manifest().application_format
            ))
        })
}

fn snapshot_codec(bundle: &SnapshotBundleFile) -> Result<keyspaces::RrflowKvKeyCodec> {
    keyspaces::RrflowKvKeyCodec::from_application_format(bundle.source_manifest.application_format)
        .ok_or_else(|| {
            Error::Substrate(format!(
                "unsupported rrflowKV snapshot application format {:?}",
                bundle.source_manifest.application_format
            ))
        })
}

fn encoded_storage_key(
    database: &Database,
    space: keyspaces::Space,
    key: &[u8],
) -> Result<Vec<u8>> {
    let codec = database_codec(database)?;
    keyspaces::validate_space(codec, space, key)?;
    Ok(key.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rrd_core::{
        Claim, DataTransaction, ObjectReceipt, Predicate, Producer, RuntimeEvent,
        RuntimeEventSchema, RuntimeMutation, RuntimeProperties, RuntimeRecord, RuntimeRecordSchema,
        RuntimeRef, RuntimeRelation, RuntimeRelationSchema, RuntimeType, Subject,
    };
    use rrd_lsm::{FailureMode, WriteBatch, WriteBoundary};
    use std::collections::BTreeMap;

    fn claim() -> Claim {
        Claim::new(
            rrd_core::Subject::new("test-subject").unwrap(),
            rrd_core::Predicate::new("test-predicate").unwrap(),
            "test-value",
            10,
            11,
            Producer {
                actor: "rrflow-kv-test".into(),
                on_behalf_of: None,
                session: None,
            },
        )
    }

    fn failure_transaction(database: &Database) -> DataTransaction {
        let scope = ScopeId::new("instance:rrflow-kv-failure").unwrap();
        let read = rrflow_kv_read_stamp(database, database.snapshot(), &scope).unwrap();
        let item_kind = RuntimeType::new("item").unwrap();
        let item = RuntimeRef::new("item", "one").unwrap();
        let mut registry = RuntimeSchemaRegistry::empty(1, "rrflowKV failure matrix");
        registry
            .records
            .insert(item_kind.clone(), RuntimeRecordSchema::default());
        registry.events.insert(
            RuntimeType::new("pulse").unwrap(),
            RuntimeEventSchema {
                subject_required: true,
                subject_types: BTreeSet::from([item_kind]),
                properties: BTreeMap::new(),
                allow_additional_properties: false,
            },
        );
        registry.relations.insert(
            RuntimeType::new("links").unwrap(),
            RuntimeRelationSchema {
                from: BTreeSet::from([RuntimeType::new("item").unwrap()]),
                to: BTreeSet::from([RuntimeType::new("item").unwrap()]),
                ..RuntimeRelationSchema::default()
            },
        );
        DataTransaction::new(
            read,
            RuntimeCommit {
                scope,
                at: 100,
                actor: "agent:rrflow-kv-failure".into(),
                expected_cursor: 0,
                mutations: vec![
                    RuntimeMutation::Schema { registry },
                    RuntimeMutation::Record {
                        record: RuntimeRecord {
                            reference: item.clone(),
                            valid_from: 100,
                            valid_to: None,
                            properties: RuntimeProperties::new(),
                        },
                    },
                    RuntimeMutation::Relation {
                        relation: RuntimeRelation {
                            reference: RuntimeRef::new("links", "one-self").unwrap(),
                            from: RuntimeRef::new("item", "one").unwrap(),
                            to: RuntimeRef::new("item", "one").unwrap(),
                            valid_from: 100,
                            valid_to: None,
                            properties: RuntimeProperties::new(),
                        },
                    },
                    RuntimeMutation::Event {
                        event: RuntimeEvent {
                            kind: RuntimeType::new("pulse").unwrap(),
                            subject: Some(item),
                            properties: RuntimeProperties::new(),
                        },
                    },
                    RuntimeMutation::Claim {
                        claim: Claim::new(
                            Subject::new("item:one").unwrap(),
                            Predicate::new("status").unwrap(),
                            "ready",
                            100,
                            100,
                            Producer {
                                actor: "agent:rrflow-kv-failure".into(),
                                on_behalf_of: None,
                                session: Some("rrflow-kv-failure".into()),
                            },
                        ),
                    },
                ],
            },
        )
        .unwrap()
    }

    #[test]
    fn rrflow_kv_multi_family_transaction_recovers_all_or_none_at_every_wal_boundary() {
        for mode in [FailureMode::Crash, FailureMode::StorageFull] {
            for boundary in [WriteBoundary::BeforeWalAppend, WriteBoundary::WalSynced] {
                let directory = tempfile::tempdir().unwrap();
                let root = directory.path().join("rrflow-kv-failure");
                let mut database = Database::create_with_application_format(
                    &root,
                    DatabaseOptions::default(),
                    keyspaces::RRFLOW_KV_FORMAT,
                )
                .unwrap();
                let transaction = failure_transaction(&database);
                let plan = prepare_rrflow_kv_commit_at_read(
                    &database,
                    &transaction.commit,
                    Some(&transaction.read),
                    None,
                )
                .unwrap();
                let expected = plan.outcome().clone();
                let (_, operations) = plan.into_parts_for(&database).unwrap();
                let error = database
                    .write_owned_with_failure(
                        WriteBatch::new(operations).unwrap(),
                        rrd_lsm::Durability::Authoritative,
                        boundary,
                        mode,
                    )
                    .unwrap_err();
                assert!(matches!(error, rrd_lsm::Error::InjectedFailure { .. }));
                drop(database);

                let mut recovered = Database::open(&root).unwrap();
                let snapshot = recovered.snapshot();
                let published = boundary == WriteBoundary::WalSynced;
                let expected_runtime_cursor = if published { 5 } else { 0 };
                let expected_claim_sequence = if published { 1 } else { 0 };
                assert_eq!(
                    read_sequence(&recovered, snapshot, &keyspaces::runtime_cursor_key(),).unwrap(),
                    expected_runtime_cursor,
                    "mode={mode:?} boundary={boundary:?}"
                );
                assert_eq!(
                    read_sequence(&recovered, snapshot, &keyspaces::sequence_watermark_key(),)
                        .unwrap(),
                    expected_claim_sequence,
                    "mode={mode:?} boundary={boundary:?}"
                );
                assert_eq!(
                    scan_space(&recovered, snapshot, keyspaces::RUNTIME_CHANGES, &[])
                        .unwrap()
                        .len(),
                    expected_runtime_cursor as usize
                );
                assert_eq!(
                    scan_space(&recovered, snapshot, keyspaces::RUNTIME_OUTBOX, &[])
                        .unwrap()
                        .len(),
                    if published { 3 } else { 0 }
                );
                assert_eq!(
                    scan_space(
                        &recovered,
                        snapshot,
                        keyspaces::RUNTIME_PROJECTION_DELTAS,
                        &[],
                    )
                    .unwrap()
                    .len(),
                    if published { 3 } else { 0 }
                );
                assert_eq!(
                    scan_space(
                        &recovered,
                        snapshot,
                        keyspaces::RUNTIME_RECORD_VERSIONS,
                        &[],
                    )
                    .unwrap()
                    .len(),
                    usize::from(published)
                );
                assert_eq!(
                    scan_space(
                        &recovered,
                        snapshot,
                        keyspaces::RUNTIME_RELATION_VERSIONS,
                        &[],
                    )
                    .unwrap()
                    .len(),
                    usize::from(published)
                );
                for adjacency in [
                    keyspaces::RUNTIME_OUTGOING_EDGES,
                    keyspaces::RUNTIME_INCOMING_EDGES,
                ] {
                    assert_eq!(
                        scan_space(&recovered, snapshot, adjacency, &[])
                            .unwrap()
                            .len(),
                        usize::from(published)
                    );
                }
                for adjacency_history in [
                    keyspaces::RUNTIME_OUTGOING_EDGE_VERSIONS,
                    keyspaces::RUNTIME_INCOMING_EDGE_VERSIONS,
                ] {
                    assert_eq!(
                        scan_space(&recovered, snapshot, adjacency_history, &[])
                            .unwrap()
                            .len(),
                        usize::from(published)
                    );
                }
                assert_eq!(
                    scan_space(&recovered, snapshot, keyspaces::CLAIMS, &[])
                        .unwrap()
                        .len(),
                    expected_claim_sequence as usize
                );

                let outcome: Option<RuntimeCommitOutcome> = get_json(
                    &recovered,
                    snapshot,
                    keyspaces::RUNTIME_COMMITS,
                    &keyspaces::runtime_commit_key(&expected.commit_id),
                )
                .unwrap();
                assert_eq!(outcome.as_ref(), published.then_some(&expected));
                let audit: Option<AuditEnvelope> = get_json(
                    &recovered,
                    snapshot,
                    keyspaces::RUNTIME_AUDIT,
                    &keyspaces::runtime_audit_key(&expected.commit_id),
                )
                .unwrap();
                assert_eq!(
                    audit.as_ref().and_then(|value| value.read.as_ref()),
                    published.then_some(&transaction.read)
                );
                assert_eq!(
                    audit.as_ref().and_then(|value| value.outcome_cursor),
                    published.then_some(5)
                );
                let schema: Option<RuntimeSchemaRegistry> = get_json(
                    &recovered,
                    snapshot,
                    keyspaces::RUNTIME_SCHEMAS,
                    &keyspaces::runtime_schema_key(&transaction.commit.scope),
                )
                .unwrap();
                assert_eq!(schema.is_some(), published);
                let record: Option<RuntimeRecord> = get_json(
                    &recovered,
                    snapshot,
                    keyspaces::RUNTIME_RECORDS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RECORDS,
                        &transaction.commit.scope,
                        &RuntimeRef::new("item", "one").unwrap(),
                    ),
                )
                .unwrap();
                assert_eq!(record.is_some(), published);
                let relation: Option<RuntimeRelation> = get_json(
                    &recovered,
                    snapshot,
                    keyspaces::RUNTIME_RELATIONS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RELATIONS,
                        &transaction.commit.scope,
                        &RuntimeRef::new("links", "one-self").unwrap(),
                    ),
                )
                .unwrap();
                assert_eq!(relation.is_some(), published);

                recovered
                    .write_owned(
                        WriteBatch::new(vec![Mutation::Put {
                            key: keyspaces::control_record_key("post-reopen"),
                            value: b"accepted".to_vec(),
                        }])
                        .unwrap(),
                        rrd_lsm::Durability::Authoritative,
                    )
                    .unwrap();
            }
        }
    }

    #[test]
    fn new_rrflow_kv_database_authenticates_and_reopens_typed_key_format() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("compact-rrflow-kv");
        let expected = claim();
        let engine = RrflowKvStore::open(&root).unwrap();
        assert_eq!(
            engine.manifest().unwrap().application_format,
            Some(keyspaces::RRFLOW_KV_FORMAT)
        );
        engine
            .claims()
            .append_batch(std::slice::from_ref(&expected))
            .unwrap();
        {
            let database = engine.lock().unwrap();
            let start = keyspaces::space_prefix(keyspaces::CLAIMS);
            let end = prefix_end(&start).unwrap();
            let rows = database
                .scan(&start, Some(&end), database.snapshot())
                .unwrap();
            assert_eq!(rows.len(), 1);
            keyspaces::parse_claim_key(KeyCodec, &rows[0].0).unwrap();
        }
        engine.flush(12).unwrap();
        drop(engine);

        let reopened = RrflowKvStore::open(&root).unwrap();
        assert_eq!(
            reopened.manifest().unwrap().application_format,
            Some(keyspaces::RRFLOW_KV_FORMAT)
        );
        assert_eq!(
            reopened.claims().claims_in_range(0, 1).unwrap(),
            vec![expected]
        );
    }

    #[test]
    fn rrflow_kv_open_denies_unknown_authenticated_application_format() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("unknown-rrflow-kv-format");
        drop(
            Database::create_with_application_format(&root, DatabaseOptions::default(), 7).unwrap(),
        );
        assert!(matches!(
            RrflowKvStore::open(&root),
            Err(Error::Substrate(reason)) if reason.contains("unsupported rrflowKV application format")
        ));
    }

    #[test]
    fn snapshot_object_closure_denies_foreign_project_references() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("multi-project-rrflow-kv");
        let mut database = Database::create_with_application_format(
            &root,
            DatabaseOptions::default(),
            keyspaces::RRFLOW_KV_FORMAT,
        )
        .unwrap();
        let first_scope = ScopeId::new("project:first").unwrap();
        let second_scope = ScopeId::new("project:second").unwrap();
        let object = |id: &str, bytes: &[u8]| {
            let sha256 = rrd_core::digest::sha256_hex(bytes);
            ObjectReference::for_bytes(
                id,
                None,
                "application/octet-stream",
                bytes,
                ObjectReceipt {
                    backend: "fixture".into(),
                    key: ObjectReference::canonical_key(&sha256).unwrap(),
                    version: None,
                    etag: None,
                },
            )
            .unwrap()
        };
        let first = object("first:bytes", b"first");
        let second = object("second:bytes", b"second");
        database
            .write_owned(
                WriteBatch::new(vec![
                    Mutation::Put {
                        key: storage_key(
                            keyspaces::RUNTIME_OBJECTS,
                            &keyspaces::runtime_identity_key(
                                keyspaces::RUNTIME_OBJECTS,
                                &first_scope,
                                &first.reference,
                            ),
                        ),
                        value: serde_json::to_vec(&first).unwrap(),
                    },
                    Mutation::Put {
                        key: storage_key(
                            keyspaces::RUNTIME_OBJECTS,
                            &keyspaces::runtime_identity_key(
                                keyspaces::RUNTIME_OBJECTS,
                                &second_scope,
                                &second.reference,
                            ),
                        ),
                        value: serde_json::to_vec(&second).unwrap(),
                    },
                ])
                .unwrap(),
                rrd_lsm::Durability::Authoritative,
            )
            .unwrap();
        let spool = directory.path().join("multi-project.snapshot");
        let bundle = database.export_snapshot_file(1, &spool).unwrap();

        let error = rrflow_kv_snapshot_object_references(&bundle, &first_scope).unwrap_err();
        assert!(error.to_string().contains("project scope"));
    }
}
