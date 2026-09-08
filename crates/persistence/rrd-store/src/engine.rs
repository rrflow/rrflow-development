//! The canonical storage port used by the RRFlow engine.
//!
//! The port owns semantics shared by the two RRFlow storage profiles:
//! persistent rrflowKV and volatile rrflowMX. Both profiles implement the same
//! claim, stamped transaction, snapshot, audit, and projection behavior. Only
//! rrflowKV adds WAL-backed durability and physical storage evidence.
//!
//! Cache and external database adapters compose above this boundary. They may
//! accelerate or supply data, but they cannot become a second state authority.

use crate::control::{
    validate_control_batch, validate_control_key, verify_control_page, verify_control_tail,
    ControlJournalEntry, ControlTransition,
};
use crate::error::{Error, Result};
use crate::keyspaces::Durability;
use crate::outcome::{AppendOutcome, IdempotentAppendOutcome};
use crate::projection::{
    difference, CurrentProjection, GroundedStamp, GroundingReport, ProjectionStatus,
    CURRENT_PROJECTION,
};
use crate::transaction::{
    validate_range, validate_read_key, StorageTransaction, TransactionCommit, TransactionRollback,
    TransactionWriteSet,
};
use rrd_core::reference::MemoryClaims;
use rrd_core::{
    projection_family, resolve_as_of, AuditEnvelope, Claim, ClaimSource, DataTransaction,
    DataTransactionView, Millis, ObjectReference, Predicate, ProjectionWork, ReadStamp, Reader,
    RetentionPin, RuntimeChange, RuntimeChangePage, RuntimeCommit, RuntimeCommitOutcome,
    RuntimeDataSnapshot, RuntimeGeo, RuntimeGraphSnapshot, RuntimeLogAccumulator, RuntimeMutation,
    RuntimeRecord, RuntimeRef, RuntimeRelation, RuntimeSchemaRegistry, RuntimeSeriesSample,
    RuntimeVector, ScopeId, SnapshotHandle, SnapshotId, Subject,
};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound::{Excluded, Included};
use std::sync::Mutex;

/// A read-only physical counter snapshot used to attribute bounded storage
/// work to one logical operation. Counters are cumulative; callers difference
/// two snapshots taken immediately around the work. Backends without stable
/// physical counters return `logical_only` evidence instead of inventing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalStoreEvidence {
    pub backend: String,
    pub evidence_level: String,
    pub physical_sequence: Option<u64>,
    pub manifest_generation: Option<u64>,
    pub durable_sequence: Option<u64>,
    pub memtable_versions: Option<u64>,
    pub memtable_bytes: Option<u64>,
    pub memtable_keys: Option<u64>,
    pub memtable_key_payload_bytes: Option<u64>,
    pub memtable_value_payload_bytes: Option<u64>,
    pub memtable_tombstones: Option<u64>,
    pub memtable_spilled_chains: Option<u64>,
    pub memtable_spilled_version_capacity: Option<u64>,
    pub memtable_version_record_bytes: Option<u64>,
    pub memtable_owned_bytes_lower_bound: Option<u64>,
    pub memtable_max_versions: Option<u64>,
    pub wal_payload_bytes: Option<u64>,
    pub wal_payload_max_bytes: Option<u64>,
    pub automatic_flushes: Option<u64>,
    pub maintenance_write_stalls: Option<u64>,
    pub failed_maintenance_flushes: Option<u64>,
    pub oversized_batches: Option<u64>,
    pub automatic_compactions: Option<u64>,
    pub failed_compactions: Option<u64>,
    pub compaction_input_bytes: Option<u64>,
    pub compaction_output_bytes: Option<u64>,
    pub peak_compaction_buffer_bytes: Option<u64>,
    pub l0_segment_count: Option<u64>,
    pub l0_compaction_trigger: Option<u64>,
    pub compaction_debt_segments: Option<u64>,
    pub compaction_target_segment_bytes: Option<u64>,
    pub segment_count: Option<u64>,
    pub segment_bytes: Option<u64>,
    pub cache_capacity_bytes: Option<u64>,
    pub cache_resident_bytes: Option<u64>,
    pub cache_entries: Option<u64>,
    pub cache_hits: Option<u64>,
    pub cache_misses: Option<u64>,
    pub cache_evictions: Option<u64>,
    pub block_loads: Option<u64>,
    pub block_bytes_loaded: Option<u64>,
    pub block_bytes_decoded: Option<u64>,
    pub filter_checks: Option<u64>,
    pub filter_negatives: Option<u64>,
    pub segment_io_requested_mode: Option<String>,
    pub segment_io_mmap_segments: Option<u64>,
    pub segment_io_uring_segments: Option<u64>,
    pub segment_io_bounded_segments: Option<u64>,
    pub segment_io_fallbacks: Option<u64>,
    pub segment_io_last_fallback: Option<String>,
    pub segment_io_read_operations: Option<u64>,
    pub segment_io_mmap_reads: Option<u64>,
    pub segment_io_uring_reads: Option<u64>,
    pub segment_io_bounded_reads: Option<u64>,
    pub segment_io_bytes_read: Option<u64>,
    pub segment_io_max_request_bytes: Option<u64>,
    pub segment_io_peak_request_bytes: Option<u64>,
}

impl PhysicalStoreEvidence {
    pub fn logical_only(backend: impl Into<String>) -> Self {
        Self {
            backend: backend.into(),
            evidence_level: "logical_only".into(),
            physical_sequence: None,
            manifest_generation: None,
            durable_sequence: None,
            memtable_versions: None,
            memtable_bytes: None,
            memtable_keys: None,
            memtable_key_payload_bytes: None,
            memtable_value_payload_bytes: None,
            memtable_tombstones: None,
            memtable_spilled_chains: None,
            memtable_spilled_version_capacity: None,
            memtable_version_record_bytes: None,
            memtable_owned_bytes_lower_bound: None,
            memtable_max_versions: None,
            wal_payload_bytes: None,
            wal_payload_max_bytes: None,
            automatic_flushes: None,
            maintenance_write_stalls: None,
            failed_maintenance_flushes: None,
            oversized_batches: None,
            automatic_compactions: None,
            failed_compactions: None,
            compaction_input_bytes: None,
            compaction_output_bytes: None,
            peak_compaction_buffer_bytes: None,
            l0_segment_count: None,
            l0_compaction_trigger: None,
            compaction_debt_segments: None,
            compaction_target_segment_bytes: None,
            segment_count: None,
            segment_bytes: None,
            cache_capacity_bytes: None,
            cache_resident_bytes: None,
            cache_entries: None,
            cache_hits: None,
            cache_misses: None,
            cache_evictions: None,
            block_loads: None,
            block_bytes_loaded: None,
            block_bytes_decoded: None,
            filter_checks: None,
            filter_negatives: None,
            segment_io_requested_mode: None,
            segment_io_mmap_segments: None,
            segment_io_uring_segments: None,
            segment_io_bounded_segments: None,
            segment_io_fallbacks: None,
            segment_io_last_fallback: None,
            segment_io_read_operations: None,
            segment_io_mmap_reads: None,
            segment_io_uring_reads: None,
            segment_io_bounded_reads: None,
            segment_io_bytes_read: None,
            segment_io_max_request_bytes: None,
            segment_io_peak_request_bytes: None,
        }
    }
}

pub trait StorageEngine: ClaimSource<Error = Error> {
    // ---- primitives every backend supplies ----

    /// Begins the profile-neutral physical transaction used by semantic
    /// repositories. `RrdEngine` remains the sole public mutation authority.
    fn begin_transaction(&self) -> Result<Box<dyn StorageTransaction + '_>>;

    /// Appends claims atomically with authoritative durability, advancing
    /// the sequence watermark in the same transaction.
    fn append_batch(&self, claims: &[Claim]) -> Result<AppendOutcome>;

    /// Atomically binds a client idempotency key to one operation digest and
    /// accepted claim-sequence interval. Retry survives process restart.
    fn append_batch_idempotent(
        &self,
        idempotency_key: &str,
        operation_sha256: &str,
        claims: &[Claim],
    ) -> Result<IdempotentAppendOutcome>;

    fn control_record(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Compare-and-swap one materialized control record and append its
    /// hash-chained journal event in the same authoritative commit.
    fn commit_control_transition(
        &self,
        transition: &ControlTransition,
    ) -> Result<ControlJournalEntry>;

    /// Atomically compares and replaces several distinct control records and
    /// appends their consecutively hash-chained journal entries. Either every
    /// transition publishes or none does.
    fn commit_control_batch(
        &self,
        transitions: &[ControlTransition],
    ) -> Result<Vec<ControlJournalEntry>>;

    /// Commits one catalogue mutation and advances the affected scope's
    /// catalogue revision in the same authoritative transaction. Catalogue
    /// state remains materialized as a control record, while the revision is
    /// part of every [`ReadStamp`] for that scope.
    fn commit_catalog_transition(
        &self,
        scope: &ScopeId,
        transition: &ControlTransition,
    ) -> Result<(u64, ControlJournalEntry)>;

    fn control_journal_since(&self, after: u64, limit: usize) -> Result<Vec<ControlJournalEntry>>;

    /// Current authoritative control-journal head. Callers use this with a
    /// runtime read stamp when one logical observation spans data and control
    /// catalogues.
    fn control_sequence(&self) -> Result<u64>;

    /// Current claim sequence watermark.
    fn sequence(&self) -> Result<u64>;

    /// Claims appended in `(from, to]`, in append order.
    fn claims_in_range(&self, from: u64, to: u64) -> Result<Vec<Claim>>;

    /// Every distinct subject with at least one claim, in key order.
    fn subjects(&self) -> Result<Vec<Subject>>;

    /// Records a read. Telemetry durability: loss on crash is acceptable.
    fn observe(
        &self,
        reader: &Reader,
        subject: &Subject,
        predicate: &Predicate,
        at: Millis,
    ) -> Result<()>;

    /// Loads a named projection blob. `None` means the caller rebuilds.
    fn get_projection(&self, name: &str) -> Result<Option<Vec<u8>>>;

    /// Stores a named projection blob with an explicit durability class.
    fn put_projection_with(&self, name: &str, bytes: &[u8], durability: Durability) -> Result<()>;

    /// Current global cursor of the authoritative typed runtime log.
    fn runtime_cursor(&self) -> Result<u64>;

    /// Latest authoritative schema registry for one runtime scope.
    fn runtime_schema(&self, scope: &ScopeId) -> Result<Option<RuntimeSchemaRegistry>>;

    /// Atomically captures the cursor, schema revision, and hash-chain head
    /// that make one logical read state reproducible across adapters.
    fn runtime_read_stamp(&self, scope: &ScopeId) -> Result<ReadStamp>;

    /// Persists a leased snapshot handle over one atomic read stamp.
    fn open_runtime_snapshot(
        &self,
        scope: &ScopeId,
        owner: &str,
        now: Millis,
        ttl: Millis,
    ) -> Result<SnapshotHandle>;

    /// Reads a bounded page that can never advance beyond the captured stamp.
    fn runtime_snapshot_changes(
        &self,
        snapshot: &SnapshotHandle,
        after: u64,
        limit: usize,
        now: Millis,
    ) -> Result<RuntimeChangePage>;

    /// Releases one persisted lease. Releasing an absent lease is idempotent.
    fn release_runtime_snapshot(&self, id: &SnapshotId) -> Result<bool>;

    /// Lists non-expired persisted leases in stable identity order.
    fn runtime_snapshots(&self, now: Millis) -> Result<Vec<SnapshotHandle>>;

    /// Lists the logical retention roots implied by every live snapshot.
    fn runtime_retention_pins(&self, now: Millis) -> Result<Vec<RetentionPin>>;

    /// Reads against an exact stamped state without creating a durable lease.
    /// This is reserved for the short lifetime of a data transaction; scans
    /// that outlive a transaction must use a persisted snapshot handle.
    fn runtime_read_changes(
        &self,
        read: &ReadStamp,
        after: u64,
        limit: usize,
    ) -> Result<RuntimeChangePage>;

    /// Reduces every logical model through one authenticated read stamp. The
    /// replay bound fails closed instead of returning a partial snapshot.
    fn runtime_data_snapshot(
        &self,
        scope: &ScopeId,
        valid_at: Millis,
        replay_limit: usize,
    ) -> Result<(ReadStamp, RuntimeDataSnapshot)> {
        if replay_limit == 0 {
            return Err(Error::Substrate(
                "runtime data snapshot replay limit must be greater than zero".into(),
            ));
        }
        let read = self.runtime_read_stamp(scope)?;
        let page = self.runtime_read_changes(&read, 0, replay_limit)?;
        if page.through_cursor != read.commit_cursor || page.has_more() {
            return Err(Error::Substrate(format!(
                "runtime data snapshot requires more than {replay_limit} retained changes"
            )));
        }
        let snapshot = if read.schema_revision.is_some() {
            let schema = schema_at_read(&read, &page)?;
            RuntimeDataSnapshot::from_changes(
                &page.changes,
                &schema,
                scope.clone(),
                valid_at,
                read.commit_cursor,
            )?
        } else {
            RuntimeDataSnapshot::from_claim_changes(
                &page.changes,
                scope.clone(),
                valid_at,
                read.commit_cursor,
            )?
        };
        Ok((read, snapshot))
    }

    /// Backend primitive that atomically commits typed runtime mutations and,
    /// for a data transaction, persists the exact validated read stamp in the
    /// accepted audit envelope. Callers use commit_runtime or
    /// commit_data_transaction.
    #[doc(hidden)]
    fn commit_runtime_at_read(
        &self,
        commit: &RuntimeCommit,
        read: Option<&ReadStamp>,
    ) -> Result<RuntimeCommitOutcome>;

    /// Atomically commits typed runtime mutations with exact-cursor conflict
    /// detection. Embedded claims join the same storage transaction.
    fn commit_runtime(&self, commit: &RuntimeCommit) -> Result<RuntimeCommitOutcome> {
        self.commit_runtime_at_read(commit, None)
    }

    /// Reads a bounded, resumable page of runtime changes.
    fn runtime_changes_since(
        &self,
        after: u64,
        limit: usize,
        scope: Option<&ScopeId>,
    ) -> Result<RuntimeChangePage>;

    /// Reads durable projection work after a source cursor.
    fn runtime_outbox_since(&self, after: u64, limit: usize) -> Result<Vec<ProjectionWork>>;

    /// Reads the accepted-operation audit envelope for one commit.
    fn runtime_audit(&self, commit_id: &str) -> Result<Option<AuditEnvelope>>;

    /// Looks up a durably accepted content identity for coordinator retry.
    fn runtime_commit_outcome(&self, commit_id: &str) -> Result<Option<RuntimeCommitOutcome>>;

    /// Captures cumulative, non-mutating physical counters. This deliberately
    /// does not emit a trace: callers bracket logical work and persist one
    /// bounded summary afterward, avoiding recursive tracing of the trace log.
    fn physical_store_evidence(&self) -> Result<PhysicalStoreEvidence> {
        Ok(PhysicalStoreEvidence::logical_only("unspecified"))
    }

    /// Commits a mutation envelope bound to its exact read stamp. The existing
    /// runtime CAS remains the final race-proof authority.
    fn commit_data_transaction(
        &self,
        transaction: &DataTransaction,
    ) -> Result<RuntimeCommitOutcome> {
        transaction.validate()?;
        self.commit_runtime_at_read(&transaction.commit, Some(&transaction.read))
    }

    /// Reconstructs the stamped base graph and overlays pending writes. The
    /// result is prospective and cannot be confused with committed evidence.
    fn preview_data_transaction(
        &self,
        transaction: &DataTransaction,
        valid_at: Millis,
    ) -> Result<DataTransactionView> {
        transaction.validate()?;
        let page = self.runtime_read_changes(&transaction.read, 0, usize::MAX)?;
        if page.through_cursor != transaction.read.commit_cursor {
            return Err(Error::Substrate(format!(
                "stamped runtime replay ended at {}, expected {}",
                page.through_cursor, transaction.read.commit_cursor
            )));
        }
        let base = RuntimeGraphSnapshot::from_changes(
            &page.changes,
            transaction.read.scope.clone(),
            valid_at,
            transaction.read.commit_cursor,
        );
        transaction.preview(&base).map_err(Error::from)
    }

    /// Reconstructs the complete all-model state at the transaction's exact
    /// read stamp, then overlays its pending mutations without publishing
    /// them. The returned cursor is prospective and cannot be used as durable
    /// evidence until the ordinary transaction commit succeeds.
    fn preview_data_snapshot(
        &self,
        transaction: &DataTransaction,
        valid_at: Millis,
        replay_limit: usize,
    ) -> Result<RuntimeDataSnapshot> {
        transaction.validate()?;
        if valid_at == 0 || replay_limit == 0 {
            return Err(Error::Substrate(
                "transaction data preview requires non-zero valid time and replay limit".into(),
            ));
        }
        let page = self.runtime_read_changes(&transaction.read, 0, replay_limit)?;
        if page.through_cursor != transaction.read.commit_cursor || page.has_more() {
            return Err(Error::Substrate(format!(
                "transaction data preview requires more than {replay_limit} retained changes"
            )));
        }
        let schema = transaction
            .commit
            .mutations
            .iter()
            .filter_map(|mutation| match mutation {
                RuntimeMutation::Schema { registry } => Some(registry.clone()),
                _ => None,
            })
            .next_back()
            .map(Ok)
            .unwrap_or_else(|| schema_at_read(&transaction.read, &page))?;
        schema.validate()?;

        let prospective_cursor = transaction
            .read
            .commit_cursor
            .checked_add(transaction.commit.mutations.len() as u64)
            .ok_or_else(|| Error::Substrate("transaction preview cursor overflowed".into()))?;
        let commit_id = transaction.commit.digest();
        let mut changes = page.changes;
        let mut previous_digest = changes.last().map(|change| change.digest.clone());
        for (ordinal, mutation) in transaction.commit.mutations.iter().cloned().enumerate() {
            let cursor = transaction.read.commit_cursor + ordinal as u64 + 1;
            let change = RuntimeChange::committed(
                cursor,
                &transaction.commit,
                &commit_id,
                ordinal as u64,
                mutation,
                previous_digest,
            );
            previous_digest = Some(change.digest.clone());
            changes.push(change);
        }
        RuntimeDataSnapshot::from_changes(
            &changes,
            &schema,
            transaction.read.scope.clone(),
            valid_at,
            prospective_cursor,
        )
        .map_err(Error::from)
    }

    // ---- provided: the semantic layer every engine inherits ----

    /// Appends a single claim. Equivalent to a batch of one.
    fn assert(&self, claim: &Claim) -> Result<AppendOutcome> {
        let candidates =
            self.versions_at_or_before(&claim.subject, &claim.predicate, claim.valid_from)?;
        let previous = resolve_as_of(&candidates, claim.valid_from).cloned();
        match previous {
            Some(previous) if previous.valid_from < claim.valid_from => {
                let pair = rrd_core::supersede(&previous, claim.clone())?;
                self.append_batch(&pair)
            }
            _ => self.append_batch(std::slice::from_ref(claim)),
        }
    }

    /// [`StorageEngine::put_projection_with`] at the Buffered default: a projection
    /// is derivable, so a crash-lost write costs a rebuild, never truth.
    fn put_projection(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.put_projection_with(name, bytes, Durability::Buffered)
    }

    /// Loads the current-state projection. Absence is the empty projection
    /// at watermark 0 — a recovery path, not an error.
    fn current_projection(&self) -> Result<CurrentProjection> {
        match self.get_projection(CURRENT_PROJECTION)? {
            Some(bytes) => Ok(CurrentProjection::from_stored_bytes(&bytes)?),
            None => Ok(CurrentProjection::empty()),
        }
    }

    /// §8.2: applies claims in `(watermark, current_sequence]` and advances
    /// the watermark in the same write as the projection. Refuses when
    /// quarantined — rebuilding on top of detected divergence would be the
    /// silent repair §8.3 forbids.
    #[tracing::instrument(level = "debug", skip_all)]
    fn rebuild_current(&self) -> Result<crate::projection::RebuildOutcome> {
        let mut projection = self.current_projection()?;
        if let ProjectionStatus::Quarantined { at, .. } = &projection.status {
            return Err(Error::Quarantined(format!(
                "projection `{CURRENT_PROJECTION}` quarantined at {at}; reset to recover"
            )));
        }
        let from = projection.watermark;
        let to = self.sequence()?;
        let interval = self.claims_in_range(from, to)?;
        let applied = interval.len();
        projection.apply(&interval);
        projection.watermark = to;
        self.put_projection_with(
            CURRENT_PROJECTION,
            &projection.to_stored_bytes()?,
            Durability::Buffered,
        )?;
        tracing::debug!(from, to, applied, "rebuild advanced the watermark");
        Ok(crate::projection::RebuildOutcome { from, to, applied })
    }

    /// §8.3: recomputes the projection from the sequence index at the
    /// projection's own watermark and differences the result against the
    /// incrementally maintained state. Empty differential stamps `grounded`;
    /// any difference quarantines the projection with Authoritative
    /// durability and reports it. Never repairs.
    #[tracing::instrument(level = "debug", skip_all)]
    fn ground_current(&self, at: Millis) -> Result<GroundingReport> {
        let mut projection = self.current_projection()?;
        if let ProjectionStatus::Quarantined { at, .. } = &projection.status {
            return Err(Error::Quarantined(format!(
                "projection `{CURRENT_PROJECTION}` quarantined at {at}; reset to recover"
            )));
        }

        let mut recomputed = CurrentProjection::empty();
        recomputed.apply(&self.claims_in_range(0, projection.watermark)?);

        let differences = difference(recomputed.entries(), projection.entries());
        if differences.is_empty() {
            let stamp = GroundedStamp {
                at,
                sequence: projection.watermark,
                digest: projection.digest()?,
            };
            projection.last_grounded = Some(stamp);
            self.put_projection_with(
                CURRENT_PROJECTION,
                &projection.to_stored_bytes()?,
                Durability::Buffered,
            )?;
            tracing::debug!(sequence = stamp.sequence, digest = stamp.digest, "grounded");
            return Ok(GroundingReport::Grounded(stamp));
        }

        projection.status = ProjectionStatus::Quarantined {
            at,
            differences: differences.clone(),
        };
        // The one derived-state write that pays for durability: a quarantine
        // a crash could forget would un-halt a diverged projection silently.
        self.put_projection_with(
            CURRENT_PROJECTION,
            &projection.to_stored_bytes()?,
            Durability::Authoritative,
        )?;
        tracing::warn!(
            differences = differences.len(),
            "divergence — projection quarantined"
        );
        Ok(GroundingReport::Divergence { differences })
    }

    /// Operator recovery: discards the projection and recomputes it from the
    /// log. The only exit from quarantine, and explicit — recomputation
    /// *becoming* the projection is a decision, not a background repair.
    /// Buffered: losing this write resurrects the quarantine, which fails
    /// closed.
    fn reset_current(&self) -> Result<crate::projection::RebuildOutcome> {
        let to = self.sequence()?;
        let mut projection = CurrentProjection::empty();
        let interval = self.claims_in_range(0, to)?;
        let applied = interval.len();
        projection.apply(&interval);
        projection.watermark = to;
        self.put_projection_with(
            CURRENT_PROJECTION,
            &projection.to_stored_bytes()?,
            Durability::Buffered,
        )?;
        Ok(crate::projection::RebuildOutcome {
            from: 0,
            to,
            applied,
        })
    }
}

/// Validate every retirement against one pre-commit authenticated snapshot.
/// The backend cursor CAS remains the final authority if another writer
/// advances after this read and before the physical transaction begins.
pub(crate) fn validate_retirement_targets<E: StorageEngine + ?Sized>(
    engine: &E,
    commit: &RuntimeCommit,
) -> Result<()> {
    let retirements = commit
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Retire { retirement } => Some(retirement),
            _ => None,
        })
        .collect::<Vec<_>>();
    if retirements.is_empty() {
        return Ok(());
    }

    let read = engine.runtime_read_stamp(&commit.scope)?;
    if read.commit_cursor != commit.expected_cursor {
        return Err(Error::RuntimeConflict {
            expected: commit.expected_cursor,
            actual: read.commit_cursor,
        });
    }
    let page = engine.runtime_read_changes(&read, 0, usize::MAX)?;
    if page.through_cursor != read.commit_cursor || page.has_more() {
        return Err(Error::ReadStampUnavailable(format!(
            "retirement validation for {}",
            commit.scope
        )));
    }
    let schema = schema_at_read(&read, &page)?;
    let mut snapshots = BTreeMap::new();
    for retirement in retirements {
        let snapshot = match snapshots.entry(retirement.effective_at) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => entry.insert(
                RuntimeDataSnapshot::from_changes(
                    &page.changes,
                    &schema,
                    commit.scope.clone(),
                    retirement.effective_at,
                    read.commit_cursor,
                )
                .map_err(Error::from)?,
            ),
        };
        if !snapshot.contains(retirement.model, &retirement.reference) {
            return Err(Error::RuntimeTargetNotFound(format!(
                "{:?} {}/{} at {} in scope {}",
                retirement.model,
                retirement.reference.kind,
                retirement.reference.id,
                retirement.effective_at,
                commit.scope
            )));
        }
    }
    Ok(())
}

fn schema_at_read(read: &ReadStamp, page: &RuntimeChangePage) -> Result<RuntimeSchemaRegistry> {
    let expected_revision = read
        .schema_revision
        .ok_or_else(|| Error::RuntimeSchemaMissing(read.scope.to_string()))?;
    let schema = page
        .changes
        .iter()
        .filter(|change| change.scope == read.scope && change.cursor <= read.commit_cursor)
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Schema { registry } => Some(registry),
            _ => None,
        })
        .next_back()
        .cloned()
        .ok_or_else(|| Error::RuntimeSchemaMissing(read.scope.to_string()))?;
    if schema.revision != expected_revision {
        return Err(Error::ReadStampUnavailable(format!(
            "schema revision {} for scope {} at cursor {}",
            expected_revision, read.scope, read.commit_cursor
        )));
    }
    Ok(schema)
}

/// rrflowMX: the process-local, volatile implementation of the complete
/// semantic storage port.
///
/// rrflowMX is also the reference side of storage conformance differentials,
/// but it is not a persistent RRFlow database: nothing here survives the
/// process. Arrow working sets and DataFusion execution are layered above this
/// port in `rrd-query`; they are not alternate state owned by rrflowMX.
#[derive(Default)]
pub struct RrflowMxStore {
    inner: Mutex<RrflowMxStoreInner>,
}

#[derive(Default)]
struct RrflowMxStoreInner {
    claims: MemoryClaims,
    /// Append order, so `claims_in_range` replays exactly like a log.
    order: Vec<Claim>,
    projections: BTreeMap<String, Vec<u8>>,
    observes: u64,
    runtime_changes: Vec<RuntimeChange>,
    runtime_accumulator: RuntimeLogAccumulator,
    runtime_merkle_nodes: BTreeMap<(u8, u64), String>,
    runtime_records: BTreeMap<(ScopeId, RuntimeRef), RuntimeRecord>,
    runtime_relations: BTreeMap<(ScopeId, RuntimeRef), RuntimeRelation>,
    runtime_vectors: BTreeMap<(ScopeId, RuntimeRef), RuntimeVector>,
    runtime_series: BTreeMap<(ScopeId, RuntimeRef), RuntimeSeriesSample>,
    runtime_geo: BTreeMap<(ScopeId, RuntimeRef), RuntimeGeo>,
    runtime_objects: BTreeMap<(ScopeId, RuntimeRef), ObjectReference>,
    runtime_outbox: BTreeMap<u64, ProjectionWork>,
    runtime_audit: BTreeMap<String, AuditEnvelope>,
    runtime_last_audit_digest: Option<String>,
    runtime_commits: BTreeMap<String, RuntimeCommitOutcome>,
    runtime_schemas: BTreeMap<ScopeId, RuntimeSchemaRegistry>,
    runtime_snapshots: BTreeMap<SnapshotId, SnapshotHandle>,
    accepted_appends: BTreeMap<String, IdempotentAppendOutcome>,
    control_records: BTreeMap<String, Vec<u8>>,
    control_journal: Vec<ControlJournalEntry>,
    catalog_revisions: BTreeMap<ScopeId, u64>,
    transaction_sequence: u64,
    transaction_values: BTreeMap<Vec<u8>, Vec<RrflowMxTransactionVersion>>,
}

#[derive(Debug)]
struct RrflowMxTransactionVersion {
    sequence: u64,
    value: Option<Vec<u8>>,
}

impl RrflowMxStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe calls recorded, for tests that assert telemetry flowed.
    pub fn observe_count(&self) -> u64 {
        self.inner.lock().expect("engine mutex").observes
    }
}

struct RrflowMxTransaction<'a> {
    store: &'a RrflowMxStore,
    snapshot_sequence: u64,
    writes: TransactionWriteSet,
}

impl StorageTransaction for RrflowMxTransaction<'_> {
    fn snapshot_sequence(&self) -> u64 {
        self.snapshot_sequence
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        validate_read_key(key)?;
        if let Some(value) = self.writes.get(key) {
            return Ok(value.map(<[u8]>::to_vec));
        }
        let inner = self.store.inner.lock().expect("engine mutex");
        Ok(inner
            .transaction_values
            .get(key)
            .and_then(|versions| {
                versions
                    .iter()
                    .rev()
                    .find(|version| version.sequence <= self.snapshot_sequence)
            })
            .and_then(|version| version.value.clone()))
    }

    fn scan(&self, start: &[u8], end: &[u8], limit: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        validate_range(start, end, limit)?;
        let inner = self.store.inner.lock().expect("engine mutex");
        let mut visible = inner
            .transaction_values
            .range::<[u8], _>((Included(start), Excluded(end)))
            .filter_map(|(key, versions)| {
                versions
                    .iter()
                    .rev()
                    .find(|version| version.sequence <= self.snapshot_sequence)
                    .and_then(|version| {
                        version
                            .value
                            .as_ref()
                            .map(|value| (key.clone(), value.clone()))
                    })
            })
            .collect::<BTreeMap<_, _>>();
        drop(inner);
        for (key, value) in self
            .writes
            .mutations()
            .range::<[u8], _>((Included(start), Excluded(end)))
        {
            match value {
                Some(value) => {
                    visible.insert(key.clone(), value.clone());
                }
                None => {
                    visible.remove(key.as_slice());
                }
            }
        }
        Ok(visible.into_iter().take(limit).collect())
    }

    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.writes.put(key, value)
    }

    fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        self.writes.delete(key)
    }

    fn commit(self: Box<Self>, _durability: Durability) -> Result<TransactionCommit> {
        let Self {
            store,
            snapshot_sequence,
            writes,
        } = *self;
        let mutation_count = writes.len();
        if mutation_count == 0 {
            tracing::debug!(
                target: "rrd_store::transaction",
                storage_profile = "rrflow_mx",
                snapshot_sequence,
                mutation_count,
                outcome = "read_only",
                "storage transaction committed without a write"
            );
            return Ok(TransactionCommit {
                snapshot_sequence,
                mutation_count,
                first_sequence: None,
                last_sequence: None,
            });
        }
        let batch = writes.into_batch()?;
        let mut inner = store.inner.lock().expect("engine mutex");
        for operation in batch.operations() {
            let key = operation.key();
            if let Some(conflicting_sequence) = inner
                .transaction_values
                .get(key)
                .and_then(|versions| versions.last())
                .map(|version| version.sequence)
                .filter(|sequence| *sequence > snapshot_sequence)
            {
                tracing::warn!(
                    target: "rrd_store::transaction",
                    storage_profile = "rrflow_mx",
                    snapshot_sequence,
                    conflicting_sequence,
                    mutation_count,
                    outcome = "conflict",
                    "storage transaction commit denied"
                );
                return Err(Error::TransactionConflict {
                    snapshot_sequence,
                    conflicting_sequence,
                });
            }
        }
        let mutation_count_u64 =
            u64::try_from(mutation_count).map_err(|_| Error::SequenceOverflow)?;
        let first_sequence = inner
            .transaction_sequence
            .checked_add(1)
            .ok_or(Error::SequenceOverflow)?;
        let last_sequence = inner
            .transaction_sequence
            .checked_add(mutation_count_u64)
            .ok_or(Error::SequenceOverflow)?;
        for (offset, operation) in batch.into_operations().into_iter().enumerate() {
            let (key, value) = match operation {
                rrd_lsm::Mutation::Put { key, value } => (key, Some(value)),
                rrd_lsm::Mutation::Delete { key } => (key, None),
            };
            inner
                .transaction_values
                .entry(key)
                .or_default()
                .push(RrflowMxTransactionVersion {
                    sequence: first_sequence + offset as u64,
                    value,
                });
        }
        inner.transaction_sequence = last_sequence;
        tracing::debug!(
            target: "rrd_store::transaction",
            storage_profile = "rrflow_mx",
            snapshot_sequence,
            mutation_count,
            first_sequence,
            last_sequence,
            outcome = "committed",
            "storage transaction committed"
        );
        Ok(TransactionCommit {
            snapshot_sequence,
            mutation_count,
            first_sequence: Some(first_sequence),
            last_sequence: Some(last_sequence),
        })
    }

    fn rollback(self: Box<Self>) -> Result<TransactionRollback> {
        let outcome = TransactionRollback {
            snapshot_sequence: self.snapshot_sequence,
            discarded_mutations: self.writes.len(),
        };
        tracing::debug!(
            target: "rrd_store::transaction",
            storage_profile = "rrflow_mx",
            snapshot_sequence = outcome.snapshot_sequence,
            discarded_mutations = outcome.discarded_mutations,
            outcome = "rolled_back",
            "storage transaction rolled back"
        );
        Ok(outcome)
    }
}

impl ClaimSource for RrflowMxStore {
    type Error = Error;

    fn versions_at_or_before(
        &self,
        subject: &Subject,
        predicate: &Predicate,
        as_of: Millis,
    ) -> Result<Vec<Claim>> {
        let inner = self.inner.lock().expect("engine mutex");
        Ok(infallible(
            inner
                .claims
                .versions_at_or_before(subject, predicate, as_of),
        ))
    }

    fn all_versions(&self, subject: &Subject, predicate: &Predicate) -> Result<Vec<Claim>> {
        let inner = self.inner.lock().expect("engine mutex");
        Ok(infallible(inner.claims.all_versions(subject, predicate)))
    }

    fn subject_versions(&self, subject: &Subject) -> Result<Vec<Claim>> {
        let inner = self.inner.lock().expect("engine mutex");
        Ok(infallible(inner.claims.subject_versions(subject)))
    }
}

fn infallible<T>(result: std::result::Result<T, std::convert::Infallible>) -> T {
    match result {
        Ok(value) => value,
    }
}

pub(crate) fn validate_idempotency(key: &str, digest: &str) -> Result<()> {
    if key.is_empty()
        || key.len() > 128
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(Error::Substrate("invalid idempotency key".into()));
    }
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(Error::Substrate(
            "operation digest must be lowercase SHA-256".into(),
        ));
    }
    Ok(())
}

impl StorageEngine for RrflowMxStore {
    fn begin_transaction(&self) -> Result<Box<dyn StorageTransaction + '_>> {
        let snapshot_sequence = self
            .inner
            .lock()
            .expect("engine mutex")
            .transaction_sequence;
        tracing::debug!(
            target: "rrd_store::transaction",
            storage_profile = "rrflow_mx",
            snapshot_sequence,
            outcome = "begun",
            "storage transaction began"
        );
        Ok(Box::new(RrflowMxTransaction {
            store: self,
            snapshot_sequence,
            writes: TransactionWriteSet::default(),
        }))
    }

    fn physical_store_evidence(&self) -> Result<PhysicalStoreEvidence> {
        Ok(PhysicalStoreEvidence::logical_only("rrflow_mx"))
    }

    fn append_batch(&self, claims: &[Claim]) -> Result<AppendOutcome> {
        let mut inner = self.inner.lock().expect("engine mutex");
        // Reject the whole batch before mutating either authoritative
        // collection if any member is invalid.
        for claim in claims {
            claim.validate()?;
        }
        let start = inner.order.len() as u64;
        for claim in claims {
            inner.claims.insert(claim.clone())?;
            inner.order.push(claim.clone());
        }
        Ok(AppendOutcome {
            first_sequence: start + 1,
            last_sequence: start + claims.len() as u64,
            count: claims.len(),
        })
    }

    fn append_batch_idempotent(
        &self,
        idempotency_key: &str,
        operation_sha256: &str,
        claims: &[Claim],
    ) -> Result<IdempotentAppendOutcome> {
        validate_idempotency(idempotency_key, operation_sha256)?;
        let mut inner = self.inner.lock().expect("engine mutex");
        if let Some(outcome) = inner.accepted_appends.get(idempotency_key) {
            if outcome.operation_sha256 != operation_sha256 {
                return Err(Error::IdempotencyConflict(idempotency_key.into()));
            }
            let mut replay = outcome.clone();
            replay.idempotent_replay = true;
            return Ok(replay);
        }
        for claim in claims {
            claim.validate()?;
        }
        if claims.is_empty() {
            return Err(Error::Substrate(
                "idempotent claim append must not be empty".into(),
            ));
        }
        let start = inner.order.len() as u64;
        for claim in claims {
            inner.claims.insert(claim.clone())?;
            inner.order.push(claim.clone());
        }
        let outcome = IdempotentAppendOutcome {
            operation_sha256: operation_sha256.into(),
            append: AppendOutcome {
                first_sequence: start + 1,
                last_sequence: start + claims.len() as u64,
                count: claims.len(),
            },
            idempotent_replay: false,
        };
        inner
            .accepted_appends
            .insert(idempotency_key.into(), outcome.clone());
        Ok(outcome)
    }

    fn control_record(&self, key: &str) -> Result<Option<Vec<u8>>> {
        validate_control_key(key)?;
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .control_records
            .get(key)
            .cloned())
    }

    fn commit_control_transition(
        &self,
        transition: &ControlTransition,
    ) -> Result<ControlJournalEntry> {
        let mut inner = self.inner.lock().expect("engine mutex");
        rrflow_mx_commit_control_transition(&mut inner, transition, None).map(|(_, entry)| entry)
    }

    fn commit_control_batch(
        &self,
        transitions: &[ControlTransition],
    ) -> Result<Vec<ControlJournalEntry>> {
        validate_control_batch(transitions)?;
        let mut inner = self.inner.lock().expect("engine mutex");
        for transition in transitions {
            if inner
                .control_records
                .get(&transition.key)
                .map(Vec::as_slice)
                != transition.expected.as_deref()
            {
                return Err(Error::ControlConflict(transition.key.clone()));
            }
        }
        let mut previous = inner
            .control_journal
            .last()
            .map(|entry| entry.digest.clone());
        verify_control_tail(
            inner.control_journal.len() as u64,
            previous.as_deref(),
            inner.control_journal.last(),
        )?;
        let current_sequence = inner.control_journal.len() as u64;
        let mut entries = Vec::with_capacity(transitions.len());
        for (offset, transition) in transitions.iter().enumerate() {
            let sequence = current_sequence
                .checked_add(offset as u64 + 1)
                .ok_or(Error::SequenceOverflow)?;
            let entry = ControlJournalEntry::committed(sequence, transition, previous.clone());
            previous = Some(entry.digest.clone());
            entries.push(entry);
        }
        for (transition, entry) in transitions.iter().zip(&entries) {
            match &transition.replacement {
                Some(value) => {
                    inner
                        .control_records
                        .insert(transition.key.clone(), value.clone());
                }
                None => {
                    inner.control_records.remove(&transition.key);
                }
            }
            inner.control_journal.push(entry.clone());
        }
        Ok(entries)
    }

    fn commit_catalog_transition(
        &self,
        scope: &ScopeId,
        transition: &ControlTransition,
    ) -> Result<(u64, ControlJournalEntry)> {
        let mut inner = self.inner.lock().expect("engine mutex");
        let (revision, entry) =
            rrflow_mx_commit_control_transition(&mut inner, transition, Some(scope))?;
        Ok((
            revision.expect("catalogue transition assigns a revision"),
            entry,
        ))
    }

    fn control_journal_since(&self, after: u64, limit: usize) -> Result<Vec<ControlJournalEntry>> {
        if limit == 0 {
            return Err(Error::Substrate(
                "control journal limit must be non-zero".into(),
            ));
        }
        let inner = self.inner.lock().expect("engine mutex");
        let entries = inner
            .control_journal
            .iter()
            .skip(after as usize)
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        let anchor_digest = after
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| inner.control_journal.get(index))
            .map(|entry| entry.digest.clone());
        verify_control_page(after, anchor_digest, &entries)?;
        Ok(entries)
    }

    fn control_sequence(&self) -> Result<u64> {
        u64::try_from(
            self.inner
                .lock()
                .expect("engine mutex")
                .control_journal
                .len(),
        )
        .map_err(|_| Error::SequenceOverflow)
    }

    fn sequence(&self) -> Result<u64> {
        Ok(self.inner.lock().expect("engine mutex").order.len() as u64)
    }

    fn claims_in_range(&self, from: u64, to: u64) -> Result<Vec<Claim>> {
        let inner = self.inner.lock().expect("engine mutex");
        let end = (to as usize).min(inner.order.len());
        if from as usize >= end {
            return Ok(Vec::new());
        }
        Ok(inner.order[from as usize..end].to_vec())
    }

    fn subjects(&self) -> Result<Vec<Subject>> {
        let inner = self.inner.lock().expect("engine mutex");
        let mut out: Vec<Subject> = Vec::new();
        for claim in inner.claims.iter() {
            if out.last().map(|s| s.as_str()) != Some(claim.subject.as_str()) {
                out.push(claim.subject.clone());
            }
        }
        out.dedup_by(|a, b| a.as_str() == b.as_str());
        Ok(out)
    }

    fn observe(&self, _: &Reader, _: &Subject, _: &Predicate, _: Millis) -> Result<()> {
        self.inner.lock().expect("engine mutex").observes += 1;
        Ok(())
    }

    fn get_projection(&self, name: &str) -> Result<Option<Vec<u8>>> {
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .projections
            .get(name)
            .cloned())
    }

    fn put_projection_with(&self, name: &str, bytes: &[u8], _: Durability) -> Result<()> {
        self.inner
            .lock()
            .expect("engine mutex")
            .projections
            .insert(name.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn runtime_cursor(&self) -> Result<u64> {
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .runtime_changes
            .len() as u64)
    }

    fn runtime_schema(&self, scope: &ScopeId) -> Result<Option<RuntimeSchemaRegistry>> {
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .runtime_schemas
            .get(scope)
            .cloned())
    }

    fn runtime_read_stamp(&self, scope: &ScopeId) -> Result<ReadStamp> {
        let inner = self.inner.lock().expect("engine mutex");
        rrflow_mx_read_stamp(&inner, scope)
    }

    fn open_runtime_snapshot(
        &self,
        scope: &ScopeId,
        owner: &str,
        now: Millis,
        ttl: Millis,
    ) -> Result<SnapshotHandle> {
        let mut inner = self.inner.lock().expect("engine mutex");
        let handle = SnapshotHandle::new(rrflow_mx_read_stamp(&inner, scope)?, owner, now, ttl)?;
        inner
            .runtime_snapshots
            .insert(handle.id.clone(), handle.clone());
        Ok(handle)
    }

    fn runtime_snapshot_changes(
        &self,
        snapshot: &SnapshotHandle,
        after: u64,
        limit: usize,
        now: Millis,
    ) -> Result<RuntimeChangePage> {
        snapshot.validate()?;
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime change page limit must be greater than zero".into(),
            ));
        }
        let inner = self.inner.lock().expect("engine mutex");
        let Some(persisted) = inner.runtime_snapshots.get(&snapshot.id) else {
            return Err(Error::SnapshotNotFound(snapshot.id.to_string()));
        };
        if persisted != snapshot {
            return Err(Error::SnapshotMismatch(snapshot.id.to_string()));
        }
        if snapshot.is_expired(now) {
            return Err(Error::SnapshotExpired {
                id: snapshot.id.to_string(),
                expired_at: snapshot.expires_at,
            });
        }
        Ok(rrflow_mx_change_page(
            &inner.runtime_changes,
            snapshot.read.commit_cursor,
            after,
            limit,
            Some(&snapshot.read.scope),
        ))
    }

    fn release_runtime_snapshot(&self, id: &SnapshotId) -> Result<bool> {
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .runtime_snapshots
            .remove(id)
            .is_some())
    }

    fn runtime_snapshots(&self, now: Millis) -> Result<Vec<SnapshotHandle>> {
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .runtime_snapshots
            .values()
            .filter(|snapshot| !snapshot.is_expired(now))
            .cloned()
            .collect())
    }

    fn runtime_retention_pins(&self, now: Millis) -> Result<Vec<RetentionPin>> {
        self.runtime_snapshots(now)?
            .iter()
            .map(RetentionPin::from_snapshot)
            .collect::<rrd_core::Result<Vec<_>>>()
            .map_err(Error::from)
    }

    fn runtime_read_changes(
        &self,
        read: &ReadStamp,
        after: u64,
        limit: usize,
    ) -> Result<RuntimeChangePage> {
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime change page limit must be greater than zero".into(),
            ));
        }
        let inner = self.inner.lock().expect("engine mutex");
        let validation = rrflow_mx_validate_read_stamp(&inner, read)?;
        if limit == 1 && after < read.commit_cursor {
            let mut page = rrflow_mx_authenticated_point_page(&inner, read, after + 1)?;
            if validation.method == "full_hash_chain_replay" {
                page.validation.method = "full_hash_chain_replay_then_rfc9162_inclusion".into();
                page.validation.change_reads = page
                    .validation
                    .change_reads
                    .saturating_add(validation.change_reads);
            }
            return Ok(page);
        }
        let mut page = rrflow_mx_change_page(
            &inner.runtime_changes,
            read.commit_cursor,
            after,
            limit,
            Some(&read.scope),
        );
        page.validation = validation;
        Ok(page)
    }

    fn commit_runtime_at_read(
        &self,
        commit: &RuntimeCommit,
        read: Option<&ReadStamp>,
    ) -> Result<RuntimeCommitOutcome> {
        commit.validate()?;
        validate_retirement_targets(self, commit)?;
        let mut inner = self.inner.lock().expect("engine mutex");
        if let Some(read) = read {
            rrflow_mx_validate_read_stamp(&inner, read)?;
        }
        let commit_id = commit.digest();
        let start = inner.runtime_changes.len() as u64;
        if start != commit.expected_cursor {
            return Err(Error::RuntimeConflict {
                expected: commit.expected_cursor,
                actual: start,
            });
        }
        let previous_schema = inner.runtime_schemas.get(&commit.scope);
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
            let effective_schema = match (previous_schema, proposed_schema) {
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
            let existing_records = inner
                .runtime_records
                .iter()
                .filter(|((scope, _), _)| scope == &commit.scope)
                .map(|(_, record)| record)
                .collect::<Vec<_>>();
            let existing_relations = inner
                .runtime_relations
                .iter()
                .filter(|((scope, _), _)| scope == &commit.scope)
                .map(|(_, relation)| relation)
                .collect::<Vec<_>>();
            effective_schema.validate_objects(
                &commit.mutations,
                existing_records,
                existing_relations,
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
                    && !inner
                        .runtime_records
                        .contains_key(&(commit.scope.clone(), reference.clone()))
                {
                    return Err(Error::DanglingRuntimeReference(format!(
                        "{}/{} in scope {}",
                        reference.kind, reference.id, commit.scope
                    )));
                }
            }
        }

        let claims = commit
            .mutations
            .iter()
            .filter_map(|mutation| match mutation {
                RuntimeMutation::Claim { claim } => Some(claim.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        for claim in &claims {
            claim.validate()?;
        }
        let claim_start = inner.order.len() as u64;
        for claim in claims.iter().cloned() {
            inner.claims.insert(claim.clone())?;
            inner.order.push(claim);
        }

        let mut previous_digest = inner
            .runtime_changes
            .last()
            .map(|change| change.digest.clone());
        let mut committed = Vec::with_capacity(commit.mutations.len());
        let mut accumulator = inner.runtime_accumulator.clone();
        let mut merkle_nodes = Vec::new();
        let mut pending_work = Vec::new();
        for (ordinal, mutation) in commit.mutations.iter().cloned().enumerate() {
            let cursor = start + ordinal as u64 + 1;
            if let Some(family) = projection_family(&mutation) {
                pending_work.push(ProjectionWork::for_change(
                    commit.scope.clone(),
                    cursor,
                    commit_id.clone(),
                    ordinal as u64,
                    family,
                )?);
            }
            let change = RuntimeChange::committed(
                cursor,
                commit,
                &commit_id,
                ordinal as u64,
                mutation,
                previous_digest.clone(),
            );
            previous_digest = Some(change.digest.clone());
            merkle_nodes.extend(accumulator.append_change(&change)?);
            committed.push(change);
        }
        let last_cursor = start + commit.mutations.len() as u64;
        let audit = AuditEnvelope::accepted_commit_at_read(
            commit,
            read,
            &commit_id,
            last_cursor,
            inner.runtime_last_audit_digest.clone(),
        )?;
        for mutation in &commit.mutations {
            match mutation {
                RuntimeMutation::Schema { registry } => {
                    inner
                        .runtime_schemas
                        .insert(commit.scope.clone(), registry.clone());
                }
                RuntimeMutation::Record { record } => {
                    inner.runtime_records.insert(
                        (commit.scope.clone(), record.reference.clone()),
                        record.clone(),
                    );
                }
                RuntimeMutation::Relation { relation } => {
                    inner.runtime_relations.insert(
                        (commit.scope.clone(), relation.reference.clone()),
                        relation.clone(),
                    );
                }
                RuntimeMutation::Vector { vector } => {
                    inner.runtime_vectors.insert(
                        (commit.scope.clone(), vector.reference.clone()),
                        vector.clone(),
                    );
                }
                RuntimeMutation::SeriesSample { sample } => {
                    inner.runtime_series.insert(
                        (commit.scope.clone(), sample.reference.clone()),
                        sample.clone(),
                    );
                }
                RuntimeMutation::Geo { geo } => {
                    inner
                        .runtime_geo
                        .insert((commit.scope.clone(), geo.reference.clone()), geo.clone());
                }
                RuntimeMutation::Object { object } => {
                    inner.runtime_objects.insert(
                        (commit.scope.clone(), object.reference.clone()),
                        object.clone(),
                    );
                }
                RuntimeMutation::Retire { retirement } => {
                    let key = (commit.scope.clone(), retirement.reference.clone());
                    if retirement.model.is_record_like() {
                        inner.runtime_records.remove(&key);
                    } else if retirement.model == rrd_core::RuntimeLogicalModel::GraphRelation {
                        inner.runtime_relations.remove(&key);
                    } else {
                        match retirement.model {
                            rrd_core::RuntimeLogicalModel::Vector => {
                                inner.runtime_vectors.remove(&key);
                            }
                            rrd_core::RuntimeLogicalModel::TimeSeries => {
                                inner.runtime_series.remove(&key);
                            }
                            rrd_core::RuntimeLogicalModel::Geo => {
                                inner.runtime_geo.remove(&key);
                            }
                            rrd_core::RuntimeLogicalModel::Object => {
                                inner.runtime_objects.remove(&key);
                            }
                            rrd_core::RuntimeLogicalModel::ReasoningClaim
                            | rrd_core::RuntimeLogicalModel::Document
                            | rrd_core::RuntimeLogicalModel::Relational
                            | rrd_core::RuntimeLogicalModel::GraphNode
                            | rrd_core::RuntimeLogicalModel::GraphRelation
                            | rrd_core::RuntimeLogicalModel::KeyValue
                            | rrd_core::RuntimeLogicalModel::Event
                            | rrd_core::RuntimeLogicalModel::ReasoningRecord
                            | rrd_core::RuntimeLogicalModel::ReasoningEvent
                            | rrd_core::RuntimeLogicalModel::LifecycleRecord
                            | rrd_core::RuntimeLogicalModel::LifecycleEvent => {}
                        }
                    }
                }
                RuntimeMutation::Claim { .. } | RuntimeMutation::Event { .. } => {}
            }
        }
        inner.runtime_changes.extend(committed);
        inner.runtime_accumulator = accumulator;
        for node in merkle_nodes {
            inner
                .runtime_merkle_nodes
                .insert((node.level, node.index), node.digest);
        }
        let outbox_count = pending_work.len();
        for work in pending_work {
            inner.runtime_outbox.insert(work.source_cursor, work);
        }
        inner.runtime_last_audit_digest = Some(audit.digest.clone());
        inner.runtime_audit.insert(commit_id.clone(), audit);
        let claim_count = claims.len();
        let outcome = RuntimeCommitOutcome {
            commit_id,
            first_cursor: start + 1,
            last_cursor: inner.runtime_changes.len() as u64,
            count: commit.mutations.len(),
            first_claim_sequence: (claim_count > 0).then_some(claim_start + 1),
            last_claim_sequence: (claim_count > 0).then_some(claim_start + claim_count as u64),
            outbox_count,
        };
        inner
            .runtime_commits
            .insert(outcome.commit_id.clone(), outcome.clone());
        Ok(outcome)
    }

    fn runtime_changes_since(
        &self,
        after: u64,
        limit: usize,
        scope: Option<&ScopeId>,
    ) -> Result<RuntimeChangePage> {
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime change page limit must be greater than zero".into(),
            ));
        }
        let inner = self.inner.lock().expect("engine mutex");
        Ok(rrflow_mx_change_page(
            &inner.runtime_changes,
            inner.runtime_changes.len() as u64,
            after,
            limit,
            scope,
        ))
    }

    fn runtime_outbox_since(&self, after: u64, limit: usize) -> Result<Vec<ProjectionWork>> {
        if limit == 0 {
            return Err(Error::Substrate(
                "runtime outbox page limit must be greater than zero".into(),
            ));
        }
        let inner = self.inner.lock().expect("engine mutex");
        Ok(inner
            .runtime_outbox
            .range((after.saturating_add(1))..)
            .take(limit)
            .map(|(_, work)| work.clone())
            .collect())
    }

    fn runtime_audit(&self, commit_id: &str) -> Result<Option<AuditEnvelope>> {
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .runtime_audit
            .get(commit_id)
            .cloned())
    }

    fn runtime_commit_outcome(&self, commit_id: &str) -> Result<Option<RuntimeCommitOutcome>> {
        Ok(self
            .inner
            .lock()
            .expect("engine mutex")
            .runtime_commits
            .get(commit_id)
            .cloned())
    }
}

/// One composition-root-owned RRFlow storage profile.
///
/// The upper engine composition root selects the profile once: rrflowKV is
/// persistent and rrflowMX is process-local. This enum owns no selection
/// heuristics and cannot open storage on its own.
pub enum StorageProfile {
    RrflowMx(RrflowMxStore),
    RrflowKv(crate::RrflowKvStore),
}

impl StorageProfile {
    pub fn rrflow_mx() -> Self {
        Self::RrflowMx(RrflowMxStore::new())
    }

    pub fn rrflow_kv(store: crate::RrflowKvStore) -> Self {
        Self::RrflowKv(store)
    }

    pub fn as_rrflow_kv(&self) -> Option<&crate::RrflowKvStore> {
        match self {
            Self::RrflowMx(_) => None,
            Self::RrflowKv(store) => Some(store),
        }
    }

    pub fn backend_name(&self) -> &'static str {
        match self {
            Self::RrflowMx(_) => "rrflow_mx",
            Self::RrflowKv(_) => "rrflow_kv",
        }
    }

    fn engine(&self) -> &(dyn StorageEngine + Send + Sync) {
        match self {
            Self::RrflowMx(engine) => engine,
            Self::RrflowKv(engine) => engine,
        }
    }
}

impl ClaimSource for StorageProfile {
    type Error = Error;

    fn versions_at_or_before(
        &self,
        subject: &Subject,
        predicate: &Predicate,
        as_of: Millis,
    ) -> Result<Vec<Claim>> {
        self.engine()
            .versions_at_or_before(subject, predicate, as_of)
    }

    fn all_versions(&self, subject: &Subject, predicate: &Predicate) -> Result<Vec<Claim>> {
        self.engine().all_versions(subject, predicate)
    }

    fn subject_versions(&self, subject: &Subject) -> Result<Vec<Claim>> {
        self.engine().subject_versions(subject)
    }
}

impl StorageEngine for StorageProfile {
    fn begin_transaction(&self) -> Result<Box<dyn StorageTransaction + '_>> {
        self.engine().begin_transaction()
    }

    fn append_batch(&self, claims: &[Claim]) -> Result<AppendOutcome> {
        self.engine().append_batch(claims)
    }

    fn append_batch_idempotent(
        &self,
        idempotency_key: &str,
        operation_sha256: &str,
        claims: &[Claim],
    ) -> Result<IdempotentAppendOutcome> {
        self.engine()
            .append_batch_idempotent(idempotency_key, operation_sha256, claims)
    }

    fn control_record(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.engine().control_record(key)
    }

    fn commit_control_transition(
        &self,
        transition: &ControlTransition,
    ) -> Result<ControlJournalEntry> {
        self.engine().commit_control_transition(transition)
    }

    fn commit_control_batch(
        &self,
        transitions: &[ControlTransition],
    ) -> Result<Vec<ControlJournalEntry>> {
        self.engine().commit_control_batch(transitions)
    }

    fn commit_catalog_transition(
        &self,
        scope: &ScopeId,
        transition: &ControlTransition,
    ) -> Result<(u64, ControlJournalEntry)> {
        self.engine().commit_catalog_transition(scope, transition)
    }

    fn control_journal_since(&self, after: u64, limit: usize) -> Result<Vec<ControlJournalEntry>> {
        self.engine().control_journal_since(after, limit)
    }

    fn control_sequence(&self) -> Result<u64> {
        self.engine().control_sequence()
    }

    fn sequence(&self) -> Result<u64> {
        self.engine().sequence()
    }

    fn claims_in_range(&self, from: u64, to: u64) -> Result<Vec<Claim>> {
        self.engine().claims_in_range(from, to)
    }

    fn subjects(&self) -> Result<Vec<Subject>> {
        self.engine().subjects()
    }

    fn observe(
        &self,
        reader: &Reader,
        subject: &Subject,
        predicate: &Predicate,
        at: Millis,
    ) -> Result<()> {
        self.engine().observe(reader, subject, predicate, at)
    }

    fn get_projection(&self, name: &str) -> Result<Option<Vec<u8>>> {
        self.engine().get_projection(name)
    }

    fn put_projection_with(&self, name: &str, bytes: &[u8], durability: Durability) -> Result<()> {
        self.engine().put_projection_with(name, bytes, durability)
    }

    fn runtime_cursor(&self) -> Result<u64> {
        self.engine().runtime_cursor()
    }

    fn runtime_schema(&self, scope: &ScopeId) -> Result<Option<RuntimeSchemaRegistry>> {
        self.engine().runtime_schema(scope)
    }

    fn runtime_read_stamp(&self, scope: &ScopeId) -> Result<ReadStamp> {
        self.engine().runtime_read_stamp(scope)
    }

    fn open_runtime_snapshot(
        &self,
        scope: &ScopeId,
        owner: &str,
        now: Millis,
        ttl: Millis,
    ) -> Result<SnapshotHandle> {
        self.engine().open_runtime_snapshot(scope, owner, now, ttl)
    }

    fn runtime_snapshot_changes(
        &self,
        snapshot: &SnapshotHandle,
        after: u64,
        limit: usize,
        now: Millis,
    ) -> Result<RuntimeChangePage> {
        self.engine()
            .runtime_snapshot_changes(snapshot, after, limit, now)
    }

    fn release_runtime_snapshot(&self, id: &SnapshotId) -> Result<bool> {
        self.engine().release_runtime_snapshot(id)
    }

    fn runtime_snapshots(&self, now: Millis) -> Result<Vec<SnapshotHandle>> {
        self.engine().runtime_snapshots(now)
    }

    fn runtime_retention_pins(&self, now: Millis) -> Result<Vec<RetentionPin>> {
        self.engine().runtime_retention_pins(now)
    }

    fn runtime_read_changes(
        &self,
        read: &ReadStamp,
        after: u64,
        limit: usize,
    ) -> Result<RuntimeChangePage> {
        self.engine().runtime_read_changes(read, after, limit)
    }

    fn commit_runtime_at_read(
        &self,
        commit: &RuntimeCommit,
        read: Option<&ReadStamp>,
    ) -> Result<RuntimeCommitOutcome> {
        self.engine().commit_runtime_at_read(commit, read)
    }

    fn runtime_changes_since(
        &self,
        after: u64,
        limit: usize,
        scope: Option<&ScopeId>,
    ) -> Result<RuntimeChangePage> {
        self.engine().runtime_changes_since(after, limit, scope)
    }

    fn runtime_outbox_since(&self, after: u64, limit: usize) -> Result<Vec<ProjectionWork>> {
        self.engine().runtime_outbox_since(after, limit)
    }

    fn runtime_audit(&self, commit_id: &str) -> Result<Option<AuditEnvelope>> {
        self.engine().runtime_audit(commit_id)
    }

    fn runtime_commit_outcome(&self, commit_id: &str) -> Result<Option<RuntimeCommitOutcome>> {
        self.engine().runtime_commit_outcome(commit_id)
    }

    fn physical_store_evidence(&self) -> Result<PhysicalStoreEvidence> {
        self.engine().physical_store_evidence()
    }
}

fn rrflow_mx_read_stamp(inner: &RrflowMxStoreInner, scope: &ScopeId) -> Result<ReadStamp> {
    ReadStamp::authenticated(
        scope.clone(),
        inner
            .runtime_schemas
            .get(scope)
            .map(|schema| schema.revision),
        inner
            .catalog_revisions
            .get(scope)
            .copied()
            .unwrap_or_default(),
        inner.runtime_changes.len() as u64,
        inner
            .runtime_changes
            .last()
            .map(|change| change.digest.clone()),
        inner.runtime_accumulator.root.clone(),
    )
    .map_err(Error::from)
}

fn rrflow_mx_validate_read_stamp(
    inner: &RrflowMxStoreInner,
    read: &ReadStamp,
) -> Result<rrd_core::RuntimeReadValidation> {
    read.validate()?;
    if read.commit_cursor > inner.runtime_changes.len() as u64 {
        return Err(Error::ReadStampUnavailable(read.manifest_id.clone()));
    }
    if read.commit_cursor == inner.runtime_changes.len() as u64 && read.accumulator_root.is_some() {
        let head_digest = inner
            .runtime_changes
            .last()
            .map(|change| change.digest.clone());
        let schema_revision = inner
            .runtime_schemas
            .get(&read.scope)
            .map(|schema| schema.revision);
        let catalog_revision = inner
            .catalog_revisions
            .get(&read.scope)
            .copied()
            .unwrap_or_default();
        if read.catalog_revision != catalog_revision
            || read.head_digest != head_digest
            || read.schema_revision != schema_revision
            || read.accumulator_root.as_deref() != Some(inner.runtime_accumulator.root.as_str())
        {
            return Err(Error::ReadStampMismatch(read.manifest_id.clone()));
        }
        return Ok(rrd_core::RuntimeReadValidation::new(
            "authenticated_current_head",
            0,
            0,
        ));
    }
    let head_digest = if read.commit_cursor == 0 {
        None
    } else {
        Some(
            inner.runtime_changes[read.commit_cursor as usize - 1]
                .digest
                .clone(),
        )
    };
    let schema_revision = inner
        .runtime_changes
        .iter()
        .take(read.commit_cursor as usize)
        .filter(|change| change.scope == read.scope)
        .filter_map(|change| match &change.mutation {
            RuntimeMutation::Schema { registry } => Some(registry.revision),
            _ => None,
        })
        .next_back();
    if let Some(root) = read.accumulator_root.as_deref() {
        RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
            Ok(inner.runtime_merkle_nodes.get(&(level, index)).cloned())
        })?;
    }
    let catalog_revision = inner
        .catalog_revisions
        .get(&read.scope)
        .copied()
        .unwrap_or_default();
    if read.catalog_revision != catalog_revision
        || read.head_digest != head_digest
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

fn rrflow_mx_commit_control_transition(
    inner: &mut RrflowMxStoreInner,
    transition: &ControlTransition,
    catalog_scope: Option<&ScopeId>,
) -> Result<(Option<u64>, ControlJournalEntry)> {
    transition.validate()?;
    if inner
        .control_records
        .get(&transition.key)
        .map(Vec::as_slice)
        != transition.expected.as_deref()
    {
        return Err(Error::ControlConflict(transition.key.clone()));
    }

    let catalog_revision = catalog_scope
        .map(|scope| {
            inner
                .catalog_revisions
                .get(scope)
                .copied()
                .unwrap_or_default()
                .checked_add(1)
                .ok_or(Error::SequenceOverflow)
        })
        .transpose()?;
    let current_sequence = inner.control_journal.len() as u64;
    let previous = inner
        .control_journal
        .last()
        .map(|entry| entry.digest.clone());
    verify_control_tail(
        current_sequence,
        previous.as_deref(),
        inner.control_journal.last(),
    )?;
    let sequence = current_sequence
        .checked_add(1)
        .ok_or(Error::SequenceOverflow)?;
    let entry = ControlJournalEntry::committed(sequence, transition, previous);

    match &transition.replacement {
        Some(value) => {
            inner
                .control_records
                .insert(transition.key.clone(), value.clone());
        }
        None => {
            inner.control_records.remove(&transition.key);
        }
    }
    if let (Some(scope), Some(revision)) = (catalog_scope, catalog_revision) {
        inner.catalog_revisions.insert(scope.clone(), revision);
    }
    inner.control_journal.push(entry.clone());
    Ok((catalog_revision, entry))
}

fn rrflow_mx_authenticated_point_page(
    inner: &RrflowMxStoreInner,
    read: &ReadStamp,
    cursor: u64,
) -> Result<RuntimeChangePage> {
    let root = read
        .accumulator_root
        .as_deref()
        .ok_or_else(|| Error::ReadStampMismatch(read.manifest_id.clone()))?;
    let accumulator = if inner.runtime_accumulator.tree_size == read.commit_cursor
        && inner.runtime_accumulator.root == root
    {
        inner.runtime_accumulator.clone()
    } else {
        RuntimeLogAccumulator::from_nodes(read.commit_cursor, root, |level, index| {
            Ok(inner.runtime_merkle_nodes.get(&(level, index)).cloned())
        })?
    };
    let change = inner
        .runtime_changes
        .get(cursor as usize - 1)
        .cloned()
        .ok_or_else(|| Error::ReadStampUnavailable(read.manifest_id.clone()))?;
    let proof = accumulator.inclusion_proof(cursor - 1, |level, index| {
        Ok(inner.runtime_merkle_nodes.get(&(level, index)).cloned())
    })?;
    let proof_nodes = proof.path.len();
    proof.verify_change(&change, root)?;
    let selected = (change.scope == read.scope).then_some(change);
    Ok(RuntimeChangePage {
        requested_after: cursor - 1,
        through_cursor: cursor,
        head_cursor: read.commit_cursor,
        validation: rrd_core::RuntimeReadValidation::new("rfc9162_inclusion_proof", 1, proof_nodes),
        changes: selected.into_iter().collect(),
    })
}

fn rrflow_mx_change_page(
    changes: &[RuntimeChange],
    head: u64,
    after: u64,
    limit: usize,
    scope: Option<&ScopeId>,
) -> RuntimeChangePage {
    if after == u64::MAX || after >= head {
        return RuntimeChangePage {
            requested_after: after,
            through_cursor: after,
            head_cursor: head,
            validation: rrd_core::RuntimeReadValidation::new("bounded_hash_chain_page", 0, 0),
            changes: Vec::new(),
        };
    }
    let end = (after as usize)
        .saturating_add(limit)
        .min(head as usize)
        .min(changes.len());
    let selected = changes[after as usize..end]
        .iter()
        .filter(|change| scope.is_none_or(|scope| scope == &change.scope))
        .cloned()
        .collect();
    RuntimeChangePage {
        requested_after: after,
        through_cursor: end as u64,
        head_cursor: head,
        validation: rrd_core::RuntimeReadValidation::new(
            "bounded_hash_chain_page",
            end.saturating_sub(after as usize) as u64,
            0,
        ),
        changes: selected,
    }
}
