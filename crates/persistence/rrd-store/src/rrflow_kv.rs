//! Persistent rrflowKV implementation of the physical [`StorageEngine`] port.
//!
//! Semantic repositories encode logical keyspaces as stable byte prefixes and
//! publish through this store's snapshot transaction. The database's physical
//! MVCC sequence is deliberately independent of claim and runtime cursors
//! stored in those transactions.

use crate::engine::{PhysicalStoreEvidence, StorageEngine};
use crate::error::{Error, Result};
use crate::key_codec::{prefix_end, KeyCodec};
use crate::keyspaces::{self, Durability};
use crate::transaction::{StorageTransaction, TransactionCommit, TransactionRollback};
use rrd_core::{
    projection_family, AuditEnvelope, Millis, ObjectReference, ProjectionWork, ReadStamp,
    RuntimeChange, RuntimeChangePage, RuntimeCommit, RuntimeCommitOutcome, RuntimeLogAccumulator,
    RuntimeMerkleNode, RuntimeMutation, RuntimeRecord, RuntimeRef, RuntimeRelation,
    RuntimeSchemaRegistry, ScopeId, SnapshotHandle, SnapshotId,
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

/// A validated rrflowKV runtime transaction that has not yet crossed the WAL
/// durability boundary. The caller may append metadata operations and publish
/// the combined vector as one rrflowKV [`WriteBatch`].
///
/// Planning reads the supplied database's current snapshot. Correct callers
/// therefore hold the database's exclusive writer guard from planning through
/// publication; `RrflowKvStore` does this internally and the Raft adapter uses
/// the same discipline.
#[derive(Debug)]
pub struct RrflowKvCommitPlan {
    outcome: RuntimeCommitOutcome,
    operations: Vec<Mutation>,
}

/// Reads the exact rrflowKV cursor/schema pair needed to prepare a runtime
/// transaction outside `RrflowKvStore` while retaining one database snapshot.
/// Coordinators use this before submitting the resulting commit through their
/// own durability boundary (for example, a Raft log).
pub fn rrflow_kv_commit_context(
    database: &Database,
    scope: &ScopeId,
) -> Result<(ReadStamp, Option<RuntimeSchemaRegistry>)> {
    let snapshot = database.snapshot();
    let read = rrflow_kv_read_stamp(database, snapshot, scope)?;
    let schema = get_json(
        database,
        snapshot,
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

impl RrflowKvCommitPlan {
    pub fn outcome(&self) -> &RuntimeCommitOutcome {
        &self.outcome
    }

    /// Verifies the target database format before an external coordinator
    /// combines these canonical mutations with its metadata in one batch.
    pub fn into_parts_for(
        self,
        database: &Database,
    ) -> Result<(RuntimeCommitOutcome, Vec<Mutation>)> {
        database_codec(database)?;
        Ok((self.outcome, self.operations))
    }
}

/// Validates and lowers one canonical [`RuntimeCommit`] into rrflowKV
/// mutations without writing them. This is the composition boundary used when
/// a coordinator must atomically include its own durable metadata.
pub fn prepare_rrflow_kv_commit(
    database: &Database,
    commit: &RuntimeCommit,
) -> Result<RrflowKvCommitPlan> {
    prepare_rrflow_kv_commit_at_read(database, commit, None, None)
}

fn prepare_rrflow_kv_commit_at_read(
    database: &Database,
    commit: &RuntimeCommit,
    read: Option<&ReadStamp>,
    archived_audit: Option<&AuditEnvelope>,
) -> Result<RrflowKvCommitPlan> {
    commit.validate()?;
    let snapshot = database.snapshot();
    if read.is_some() && archived_audit.is_some() {
        return Err(Error::Archive(
            "archive replay cannot also supply a live read stamp".into(),
        ));
    }
    if let Some(audit) = archived_audit {
        audit.validate()?;
        if let Some(read) = &audit.read {
            read.validate()?;
        }
    } else if let Some(read) = read {
        validate_rrflow_kv_read_stamp(database, snapshot, read)?;
    }
    let commit_id = commit.digest();
    let start = read_sequence(database, snapshot, &keyspaces::runtime_cursor_key())?;
    if start != commit.expected_cursor {
        return Err(Error::RuntimeConflict {
            expected: commit.expected_cursor,
            actual: start,
        });
    }
    let (mut accumulator, bootstrap_nodes) =
        rrflow_kv_runtime_accumulator_with(database, snapshot, start)?;

    let previous_schema: Option<RuntimeSchemaRegistry> = get_json(
        database,
        snapshot,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(&commit.scope),
    )?;
    let proposed_schema = commit.mutations.iter().find_map(|mutation| match mutation {
        RuntimeMutation::Schema { registry } => Some(registry),
        _ => None,
    });
    let schema_free_claims = previous_schema.is_none()
        && proposed_schema.is_none()
        && commit
            .mutations
            .iter()
            .all(|mutation| matches!(mutation, RuntimeMutation::Claim { .. }));
    if !schema_free_claims {
        let effective_schema = match (previous_schema.as_ref(), proposed_schema) {
            (None, Some(registry)) if registry.revision == 1 => registry,
            (None, Some(registry)) => {
                return Err(Error::RuntimeSchemaConflict {
                    expected: 1,
                    actual: registry.revision,
                });
            }
            (Some(previous), Some(registry))
                if registry.revision == previous.revision.saturating_add(1) =>
            {
                registry
            }
            (Some(previous), Some(registry)) => {
                return Err(Error::RuntimeSchemaConflict {
                    expected: previous.revision.saturating_add(1),
                    actual: registry.revision,
                });
            }
            (Some(previous), None) => previous,
            (None, None) => {
                return Err(Error::RuntimeSchemaMissing(commit.scope.to_string()));
            }
        };
        let existing_records = if effective_schema
            .records
            .values()
            .any(|schema| !schema.unique_properties.is_empty())
        {
            rrflow_kv_values_for_scope::<RuntimeRecord>(
                database,
                snapshot,
                keyspaces::RUNTIME_RECORDS,
                &commit.scope,
            )?
        } else {
            Vec::new()
        };
        let existing_relations = if effective_schema.relations.values().any(|schema| {
            schema.unique_pair || schema.max_outgoing.is_some() || schema.max_incoming.is_some()
        }) {
            rrflow_kv_values_for_scope::<RuntimeRelation>(
                database,
                snapshot,
                keyspaces::RUNTIME_RELATIONS,
                &commit.scope,
            )?
        } else {
            Vec::new()
        };
        effective_schema.validate_objects(
            &commit.mutations,
            &existing_records,
            &existing_relations,
        )?;
    }

    let new_records = commit
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Record { record } => Some(record.reference.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for mutation in &commit.mutations {
        let references: Vec<&RuntimeRef> = match mutation {
            RuntimeMutation::Relation { relation } => vec![&relation.from, &relation.to],
            RuntimeMutation::Event { event } => event.subject.iter().collect(),
            RuntimeMutation::Vector { vector } => vec![&vector.subject],
            RuntimeMutation::SeriesSample { sample } => vec![&sample.series],
            RuntimeMutation::Geo { geo } => vec![&geo.subject],
            RuntimeMutation::Object { object } => object.subject.iter().collect(),
            RuntimeMutation::Claim { .. }
            | RuntimeMutation::Schema { .. }
            | RuntimeMutation::Record { .. }
            | RuntimeMutation::Retire { .. } => Vec::new(),
        };
        for reference in references {
            if !new_records.contains(reference)
                && get(
                    database,
                    snapshot,
                    keyspaces::RUNTIME_RECORDS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RECORDS,
                        &commit.scope,
                        reference,
                    ),
                )?
                .is_none()
            {
                return Err(Error::DanglingRuntimeReference(format!(
                    "{}/{} in scope {}",
                    reference.kind, reference.id, commit.scope
                )));
            }
        }
    }

    let claim_count = commit
        .mutations
        .iter()
        .filter(|mutation| matches!(mutation, RuntimeMutation::Claim { .. }))
        .count();
    let claim_start = read_sequence(database, snapshot, &keyspaces::sequence_watermark_key())?;
    let mut claim_sequence = claim_start;
    let mut cursor = start;
    let mut previous_digest = get(
        database,
        snapshot,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    let previous_audit_digest = get(
        database,
        snapshot,
        keyspaces::META,
        &keyspaces::runtime_last_audit_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    let mut operations = Vec::new();
    for node in bootstrap_nodes {
        put(
            &mut operations,
            keyspaces::META,
            &keyspaces::runtime_accumulator_node_key(node.level, node.index),
            node.digest.into_bytes(),
        );
    }
    let mut outbox_count = 0;

    for (ordinal, mutation) in commit.mutations.iter().cloned().enumerate() {
        if let RuntimeMutation::Claim { claim } = &mutation {
            claim.validate()?;
            claim_sequence = claim_sequence
                .checked_add(1)
                .ok_or(Error::SequenceOverflow)?;
            let claim_key = keyspaces::claim_key(
                &claim.subject,
                &claim.predicate,
                claim.valid_from,
                claim.tx_time,
            );
            put(
                &mut operations,
                keyspaces::SEQUENCE_INDEX,
                &keyspaces::sequence_key(claim_sequence),
                claim_key.clone(),
            );
            put(
                &mut operations,
                keyspaces::CLAIMS,
                &claim_key,
                serde_json::to_vec(claim)?,
            );
        }

        cursor = cursor.checked_add(1).ok_or(Error::SequenceOverflow)?;
        let change = RuntimeChange::committed(
            cursor,
            commit,
            &commit_id,
            ordinal as u64,
            mutation.clone(),
            previous_digest.clone(),
        );
        put(
            &mut operations,
            keyspaces::RUNTIME_CHANGES,
            &keyspaces::runtime_change_key(cursor),
            serde_json::to_vec(&change)?,
        );
        for node in accumulator.append_change(&change)? {
            put(
                &mut operations,
                keyspaces::META,
                &keyspaces::runtime_accumulator_node_key(node.level, node.index),
                node.digest.into_bytes(),
            );
        }
        if let Some(family) = projection_family(&mutation) {
            let work = ProjectionWork::for_change(
                commit.scope.clone(),
                cursor,
                commit_id.clone(),
                ordinal as u64,
                family,
            )?;
            put(
                &mut operations,
                keyspaces::RUNTIME_OUTBOX,
                &keyspaces::runtime_outbox_key(cursor),
                serde_json::to_vec(&work)?,
            );
            outbox_count += 1;
        }
        match mutation {
            RuntimeMutation::Schema { registry } => put(
                &mut operations,
                keyspaces::RUNTIME_SCHEMAS,
                &keyspaces::runtime_schema_key(&commit.scope),
                serde_json::to_vec(&registry)?,
            ),
            RuntimeMutation::Record { record } => put(
                &mut operations,
                keyspaces::RUNTIME_RECORDS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_RECORDS,
                    &commit.scope,
                    &record.reference,
                ),
                serde_json::to_vec(&record)?,
            ),
            RuntimeMutation::Relation { relation } => put(
                &mut operations,
                keyspaces::RUNTIME_RELATIONS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_RELATIONS,
                    &commit.scope,
                    &relation.reference,
                ),
                serde_json::to_vec(&relation)?,
            ),
            RuntimeMutation::Vector { vector } => put(
                &mut operations,
                keyspaces::RUNTIME_VECTORS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_VECTORS,
                    &commit.scope,
                    &vector.reference,
                ),
                serde_json::to_vec(&vector)?,
            ),
            RuntimeMutation::SeriesSample { sample } => put(
                &mut operations,
                keyspaces::RUNTIME_SERIES,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_SERIES,
                    &commit.scope,
                    &sample.reference,
                ),
                serde_json::to_vec(&sample)?,
            ),
            RuntimeMutation::Geo { geo } => put(
                &mut operations,
                keyspaces::RUNTIME_GEO,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_GEO,
                    &commit.scope,
                    &geo.reference,
                ),
                serde_json::to_vec(&geo)?,
            ),
            RuntimeMutation::Object { object } => put(
                &mut operations,
                keyspaces::RUNTIME_OBJECTS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_OBJECTS,
                    &commit.scope,
                    &object.reference,
                ),
                serde_json::to_vec(&object)?,
            ),
            RuntimeMutation::Retire { retirement } => {
                let space = if retirement.model.is_record_like() {
                    Some(keyspaces::RUNTIME_RECORDS)
                } else {
                    match retirement.model {
                        rrd_core::RuntimeLogicalModel::GraphRelation => {
                            Some(keyspaces::RUNTIME_RELATIONS)
                        }
                        rrd_core::RuntimeLogicalModel::Vector => Some(keyspaces::RUNTIME_VECTORS),
                        rrd_core::RuntimeLogicalModel::TimeSeries => {
                            Some(keyspaces::RUNTIME_SERIES)
                        }
                        rrd_core::RuntimeLogicalModel::Geo => Some(keyspaces::RUNTIME_GEO),
                        rrd_core::RuntimeLogicalModel::Object => Some(keyspaces::RUNTIME_OBJECTS),
                        rrd_core::RuntimeLogicalModel::ReasoningClaim
                        | rrd_core::RuntimeLogicalModel::Document
                        | rrd_core::RuntimeLogicalModel::Relational
                        | rrd_core::RuntimeLogicalModel::GraphNode
                        | rrd_core::RuntimeLogicalModel::KeyValue
                        | rrd_core::RuntimeLogicalModel::Event
                        | rrd_core::RuntimeLogicalModel::ReasoningRecord
                        | rrd_core::RuntimeLogicalModel::ReasoningEvent
                        | rrd_core::RuntimeLogicalModel::LifecycleRecord
                        | rrd_core::RuntimeLogicalModel::LifecycleEvent => None,
                    }
                };
                if let Some(space) = space {
                    let key = keyspaces::runtime_identity_key(
                        space,
                        &commit.scope,
                        &retirement.reference,
                    );
                    delete(&mut operations, space, &key);
                }
            }
            RuntimeMutation::Claim { .. } | RuntimeMutation::Event { .. } => {}
        }
        previous_digest = Some(change.digest);
    }
    if claim_count > 0 {
        put_sequence(
            &mut operations,
            &keyspaces::sequence_watermark_key(),
            claim_sequence,
        );
    }
    put_sequence(&mut operations, &keyspaces::runtime_cursor_key(), cursor);
    put(
        &mut operations,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
        previous_digest.as_deref().unwrap_or("").as_bytes().to_vec(),
    );
    put(
        &mut operations,
        keyspaces::META,
        &keyspaces::runtime_accumulator_state_key(),
        serde_json::to_vec(&accumulator)?,
    );
    let audit_read = archived_audit
        .and_then(|audit| audit.read.as_ref())
        .or(read);
    let audit = AuditEnvelope::accepted_commit_at_read(
        commit,
        audit_read,
        &commit_id,
        cursor,
        previous_audit_digest,
    )?;
    if archived_audit.is_some_and(|expected| expected != &audit) {
        return Err(Error::Archive(format!(
            "runtime commit {commit_id} audit envelope differs from its archive"
        )));
    }
    put(
        &mut operations,
        keyspaces::RUNTIME_AUDIT,
        &keyspaces::runtime_audit_key(&commit_id),
        serde_json::to_vec(&audit)?,
    );
    put(
        &mut operations,
        keyspaces::META,
        &keyspaces::runtime_last_audit_digest_key(),
        audit.digest.as_bytes().to_vec(),
    );
    let outcome = RuntimeCommitOutcome {
        commit_id,
        first_cursor: start + 1,
        last_cursor: cursor,
        count: commit.mutations.len(),
        first_claim_sequence: (claim_count > 0).then_some(claim_start + 1),
        last_claim_sequence: (claim_count > 0).then_some(claim_sequence),
        outbox_count,
    };
    put(
        &mut operations,
        keyspaces::RUNTIME_COMMITS,
        &keyspaces::runtime_commit_key(&outcome.commit_id),
        serde_json::to_vec(&outcome)?,
    );
    Ok(RrflowKvCommitPlan {
        outcome,
        operations,
    })
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
    let commit_cursor = read_sequence(database, snapshot, &keyspaces::runtime_cursor_key())?;
    let schema_revision = get_json::<RuntimeSchemaRegistry>(
        database,
        snapshot,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(scope),
    )?
    .map(|schema| schema.revision);
    let catalog_revision =
        read_sequence(database, snapshot, &keyspaces::catalog_revision_key(scope))?;
    let head_digest = get(
        database,
        snapshot,
        keyspaces::META,
        &keyspaces::runtime_last_digest_key(),
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|error| Error::CorruptWatermark(error.to_string()))?
    .filter(|digest| !digest.is_empty());
    match load_rrflow_kv_runtime_accumulator(database, snapshot, commit_cursor)? {
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

fn validate_rrflow_kv_read_stamp(
    database: &Database,
    snapshot: Snapshot,
    read: &ReadStamp,
) -> Result<rrd_core::RuntimeReadValidation> {
    read.validate()?;
    let current = read_sequence(database, snapshot, &keyspaces::runtime_cursor_key())?;
    if read.commit_cursor > current {
        return Err(Error::ReadStampUnavailable(read.manifest_id.clone()));
    }
    if read.commit_cursor == current && read.accumulator_root.is_some() {
        let accumulator = load_rrflow_kv_runtime_accumulator(database, snapshot, current)?
            .ok_or_else(|| Error::ReadStampMismatch(read.manifest_id.clone()))?;
        let head_digest = get(
            database,
            snapshot,
            keyspaces::META,
            &keyspaces::runtime_last_digest_key(),
        )?
        .map(String::from_utf8)
        .transpose()
        .map_err(|error| Error::CorruptWatermark(error.to_string()))?
        .filter(|digest| !digest.is_empty());
        let schema_revision = get_json::<RuntimeSchemaRegistry>(
            database,
            snapshot,
            keyspaces::RUNTIME_SCHEMAS,
            &keyspaces::runtime_schema_key(&read.scope),
        )?
        .map(|schema| schema.revision);
        let catalog_revision = read_sequence(
            database,
            snapshot,
            &keyspaces::catalog_revision_key(&read.scope),
        )?;
        if read.catalog_revision != catalog_revision
            || read.head_digest != head_digest
            || read.schema_revision != schema_revision
            || read.accumulator_root.as_deref() != Some(accumulator.root.as_str())
        {
            return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
        }
        return Ok(rrd_core::RuntimeReadValidation::new(
            "authenticated_current_head",
            0,
            0,
        ));
    }
    let retained_head = if read.commit_cursor == 0 {
        None
    } else {
        let change: RuntimeChange = get_json(
            database,
            snapshot,
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
    let page = rrflow_kv_change_page(
        database,
        snapshot,
        read.commit_cursor,
        0,
        usize::MAX,
        Some(&read.scope),
    )?;
    let schema_revision = page
        .changes
        .iter()
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Schema { registry } => Some(registry.revision),
            _ => None,
        })
        .next_back();
    let catalog_revision = read_sequence(
        database,
        snapshot,
        &keyspaces::catalog_revision_key(&read.scope),
    )?;
    if let Some(root) = read.accumulator_root.as_deref() {
        RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
            read_rrflow_kv_accumulator_node(database, snapshot, level, index)
        })?;
    }
    if read.catalog_revision != catalog_revision
        || read.head_digest != retained_head
        || read.schema_revision != schema_revision
    {
        return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
    }
    Ok(rrd_core::RuntimeReadValidation::new(
        "full_hash_chain_replay",
        read.commit_cursor,
        0,
    ))
}

fn load_rrflow_kv_runtime_accumulator(
    database: &Database,
    snapshot: Snapshot,
    expected_size: u64,
) -> Result<Option<RuntimeLogAccumulator>> {
    let stored: Option<RuntimeLogAccumulator> = get_json(
        database,
        snapshot,
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

fn rrflow_kv_runtime_accumulator_with(
    database: &Database,
    snapshot: Snapshot,
    expected_size: u64,
) -> Result<(RuntimeLogAccumulator, Vec<RuntimeMerkleNode>)> {
    if let Some(accumulator) =
        load_rrflow_kv_runtime_accumulator(database, snapshot, expected_size)?
    {
        return Ok((accumulator, Vec::new()));
    }
    let page = rrflow_kv_change_page(database, snapshot, expected_size, 0, usize::MAX, None)?;
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

fn read_rrflow_kv_accumulator_node(
    database: &Database,
    snapshot: Snapshot,
    level: u8,
    index: u64,
) -> rrd_core::Result<Option<String>> {
    get(
        database,
        snapshot,
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

fn rrflow_kv_change_page(
    database: &Database,
    snapshot: Snapshot,
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
            validation: rrd_core::RuntimeReadValidation::new("bounded_hash_chain_page", 0, 0),
            changes: Vec::new(),
        });
    }
    let mut previous_digest = if after == 0 {
        None
    } else {
        let prior: RuntimeChange = get_json(
            database,
            snapshot,
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
            database,
            snapshot,
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
        validation: rrd_core::RuntimeReadValidation::new(
            "bounded_hash_chain_page",
            through.saturating_sub(after),
            0,
        ),
        changes: selected,
    })
}

fn rrflow_kv_values_for_scope<T: DeserializeOwned>(
    database: &Database,
    snapshot: Snapshot,
    space: keyspaces::Space,
    scope: &ScopeId,
) -> Result<Vec<T>> {
    let prefix = keyspaces::runtime_scope_prefix(space, scope);
    scan_space(database, snapshot, space, &prefix)?
        .into_iter()
        .map(|(_, value)| serde_json::from_slice(&value).map_err(Error::from))
        .collect()
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

fn put(operations: &mut Vec<Mutation>, space: keyspaces::Space, key: &[u8], value: Vec<u8>) {
    operations.push(Mutation::Put {
        key: storage_key(space, key),
        value,
    });
}

fn delete(operations: &mut Vec<Mutation>, space: keyspaces::Space, key: &[u8]) {
    operations.push(Mutation::Delete {
        key: storage_key(space, key),
    });
}

fn put_sequence(operations: &mut Vec<Mutation>, key: &[u8], sequence: u64) {
    put(
        operations,
        keyspaces::META,
        key,
        sequence.to_string().into_bytes(),
    );
}

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
        RuntimeEventSchema, RuntimeProperties, RuntimeRecordSchema, RuntimeType, Subject,
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
                let expected_runtime_cursor = if published { 4 } else { 0 };
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
                    if published { 2 } else { 0 }
                );
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
                    published.then_some(4)
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
