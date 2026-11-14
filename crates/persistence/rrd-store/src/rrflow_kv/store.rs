//! Creating, opening and maintaining an rrflowKV store.

use super::*;

/// Read-only projection of the formats owned by the initial rrflowKV writer.
/// Installation can bind these without copying physical-format constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RrflowKvInitializationFormats {
    pub application_format: u64,
    pub manifest_format: u16,
    pub wal_format: u16,
    pub batch_format: u16,
}

impl RrflowKvStore {
    pub const fn initialization_formats() -> RrflowKvInitializationFormats {
        RrflowKvInitializationFormats {
            application_format: keyspaces::RRFLOW_KV_FORMAT,
            manifest_format: rrd_lsm::MANIFEST_FORMAT_VERSION,
            wal_format: rrd_lsm::WAL_FORMAT_VERSION,
            batch_format: rrd_lsm::BATCH_FORMAT_VERSION,
        }
    }

    /// Creates exactly one new rrflowKV root. Parent-directory creation is an
    /// installer effect and therefore remains outside this constructor.
    pub fn create_new(path: &Path) -> Result<Self> {
        Self::create_new_with_options(path, DatabaseOptions::default())
    }

    pub fn create_new_with_options(path: &Path, options: DatabaseOptions) -> Result<Self> {
        Self::create_new_anchored(&AnchoredPath::from_path(path)?, options)
    }

    pub fn create_new_anchored(path: &AnchoredPath, options: DatabaseOptions) -> Result<Self> {
        let database = Database::create_anchored(path, options, Some(keyspaces::RRFLOW_KV_FORMAT))?;
        Self::from_explicit_database(database, true)
    }

    /// Opens an existing rrflowKV root without creating paths or reconciling
    /// checkpoints. Recovery validates bytes but never repairs a torn tail.
    pub fn open_existing(path: &Path) -> Result<Self> {
        Self::open_existing_with_options(path, DatabaseOptions::default())
    }

    pub fn open_existing_with_options(path: &Path, options: DatabaseOptions) -> Result<Self> {
        Self::open_existing_anchored(&AnchoredPath::open_root(path)?, options)
    }

    pub fn open_existing_anchored(path: &AnchoredPath, options: DatabaseOptions) -> Result<Self> {
        let database = Database::open_anchored(path, options).map_err(|error| {
            if matches!(
                error,
                rrd_lsm::Error::MutableLimit { .. }
                    | rrd_lsm::Error::SegmentHandleLimit { .. }
                    | rrd_lsm::Error::SourcePageLimit { .. }
            ) {
                return Error::from(error);
            }
            Error::Substrate(format!(
                "cannot open existing rrflowKV database {}: {error}",
                path.display()
            ))
        })?;
        Self::from_explicit_database(database, false)
    }

    fn from_explicit_database(database: Database, created: bool) -> Result<Self> {
        let _codec = keyspaces::RrflowKvKeyCodec::from_application_format(
            database.manifest().application_format,
        )
        .ok_or_else(|| {
            Error::Substrate(format!(
                "unsupported rrflowKV application format {:?}",
                database.manifest().application_format
            ))
        })?;
        let path = database.anchored_root().clone();
        Ok(Self {
            path,
            open_evidence: RrflowKvOpenEvidence {
                contract_version: RRFLOW_KV_OPEN_EVIDENCE_VERSION,
                created,
                segment_validation: database.segment_open_evidence().clone(),
                reconciliation: RrflowKvReconciliationEvidence::default(),
            },
            database: RwLock::new(database),
            native_write_path_diagnostics_enabled: AtomicBool::new(false),
        })
    }

    /// Opens an existing rrflowKV database or creates one when `path` is absent.
    /// An existing but invalid directory fails closed rather than being
    /// silently reinitialized.
    pub fn open(path: &(impl crate::filesystem::AnchorInput + ?Sized)) -> Result<Self> {
        Self::open_with_options(path, DatabaseOptions::default())
    }

    /// Opens with explicit rrflowKV cache and mutable-state bounds. Persistent
    /// format identity is unchanged; these are process-local operating limits.
    pub fn open_with_options(
        path: &(impl crate::filesystem::AnchorInput + ?Sized),
        options: DatabaseOptions,
    ) -> Result<Self> {
        let path = &path.anchor()?;
        let total_started = Instant::now();
        if !path.try_exists()? {
            if let Some(parent) = path.parent() {
                parent
                    .create_dir_all()
                    .map_err(|error| Error::Substrate(error.to_string()))?;
            }
        }
        let empty = path.try_exists()?
            && path.metadata()?.is_dir()
            && path
                .entries()
                .map_err(|error| Error::Substrate(error.to_string()))?
                .next()
                .is_none();
        let created = !path.try_exists()? || empty;
        let database_started = Instant::now();
        let mut database = if created {
            Database::create_anchored(path, options, Some(keyspaces::RRFLOW_KV_FORMAT))?
        } else {
            Database::open_anchored(path, options).map_err(|error| {
                if matches!(
                    error,
                    rrd_lsm::Error::MutableLimit { .. }
                        | rrd_lsm::Error::SegmentHandleLimit { .. }
                        | rrd_lsm::Error::SourcePageLimit { .. }
                ) {
                    return Error::from(error);
                }
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
        let segment_validation = database.segment_open_evidence().clone();
        let cache_before = database.page_cache_stats();
        let io_before = database.segment_io_stats();
        let checkpoints_started = Instant::now();
        reconcile_runtime_checkpoints(&mut database, None, 0).map_err(|error| {
            Error::Substrate(format!(
                "cannot reconcile runtime checkpoints while opening {}: {error}",
                path.display()
            ))
        })?;
        let checkpoint_reconcile_ms = checkpoints_started.elapsed().as_millis() as u64;
        let reconciliation = reconciliation_evidence(
            &cache_before,
            &database.page_cache_stats(),
            &io_before,
            &database.segment_io_stats(),
        )?;
        let open_evidence = RrflowKvOpenEvidence {
            contract_version: RRFLOW_KV_OPEN_EVIDENCE_VERSION,
            created,
            segment_validation,
            reconciliation,
        };
        tracing::info!(
            target: "rrflow_kv::open",
            path = %path.display(),
            created,
            database_open_ms,
            checkpoint_reconcile_ms,
            validated_segments = open_evidence.segment_validation.segment_count,
            validation_bytes = open_evidence.segment_validation.full_checksum_bytes,
            persisted_filter_count = open_evidence.segment_validation.persisted_filter_count,
            persisted_filter_bytes = open_evidence.segment_validation.persisted_filter_bytes,
            none_policy_segments = open_evidence.segment_validation.none_policy_segment_count,
            adaptive_lz4_policy_segments = open_evidence
                .segment_validation
                .adaptive_lz4_policy_segment_count,
            raw_pages = open_evidence.segment_validation.raw_page_count,
            compressed_pages = open_evidence.segment_validation.compressed_page_count,
            stored_page_bytes = open_evidence.segment_validation.stored_page_bytes,
            logical_page_bytes = open_evidence.segment_validation.logical_page_bytes,
            reconciliation_page_requests = open_evidence.reconciliation.page_requests,
            reconciliation_read_operations = open_evidence.reconciliation.read_operations,
            reconciliation_bytes_read = open_evidence.reconciliation.bytes_read,
            total_ms = total_started.elapsed().as_millis() as u64,
            "rrflowKV store open phases completed"
        );
        let path = database.anchored_root().clone();
        Ok(Self {
            path,
            database: RwLock::new(database),
            open_evidence,
            native_write_path_diagnostics_enabled: AtomicBool::new(false),
        })
    }

    pub fn path(&self) -> &Path {
        self.path.diagnostic_path()
    }

    pub fn anchored_path(&self) -> &AnchoredPath {
        &self.path
    }

    /// Attempt native space admission without queuing behind another owner.
    pub fn try_admit_immutable_file(&self, bytes: u64) -> Result<bool> {
        match self.database.try_write() {
            Ok(database) => {
                database.admit_immutable_file(bytes).map_err(Error::from)?;
                Ok(true)
            }
            Err(std::sync::TryLockError::WouldBlock) => Ok(false),
            Err(std::sync::TryLockError::Poisoned(_)) => {
                Err(Error::Substrate("rrflowKV database lock poisoned".into()))
            }
        }
    }

    pub fn open_evidence(&self) -> &RrflowKvOpenEvidence {
        &self.open_evidence
    }

    /// Enables bounded native write-path diagnostics for subsequent commits.
    /// This process-local evidence is disabled by default and is never stored.
    pub fn enable_native_write_path_diagnostics(&self, capacity: usize) -> Result<()> {
        self.lock()?.enable_write_path_diagnostics(capacity)?;
        self.native_write_path_diagnostics_enabled
            .store(true, Ordering::Release);
        Ok(())
    }

    /// Disables and drains the current native write-path diagnostic buffer.
    pub fn take_native_write_path_diagnostics(&self) -> Result<Option<WritePathDiagnostics>> {
        self.native_write_path_diagnostics_enabled
            .store(false, Ordering::Release);
        Ok(self.lock()?.take_write_path_diagnostics())
    }

    /// Publishes the current rrflowKV memtable as an immutable segment.
    pub fn flush(&self, at: Millis) -> Result<Option<Manifest>> {
        self.lock()?.flush_memtable(at).map_err(Error::from)
    }

    /// Offer one physical maintenance quantum without queuing behind a reader
    /// or writer. Busy is a skipped offer, not a second worker or a retry loop.
    pub fn try_maintenance_step(&self) -> Result<bool> {
        match self.database.try_write() {
            // The private database never exposes an untracked Snapshot.
            // Transactions own registered leases; inspection streams and
            // runtime checkpoints retain physical generations independently.
            // Logical versioned keys are data, not superseded MVCC versions.
            Ok(mut database) => database
                .maintenance_step_with_protected_snapshots(&[])
                .map_err(Error::from),
            Err(std::sync::TryLockError::WouldBlock) => Ok(false),
            Err(std::sync::TryLockError::Poisoned(_)) => {
                Err(Error::Substrate("rrflowKV database lock poisoned".into()))
            }
        }
    }

    pub fn manifest(&self) -> Result<Manifest> {
        Ok(self.read_lock()?.manifest().clone())
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

    pub(super) fn lock(&self) -> Result<RwLockWriteGuard<'_, Database>> {
        self.database
            .write()
            .map_err(|_| Error::Substrate("rrflowKV database lock poisoned".into()))
    }

    pub(super) fn read_lock(&self) -> Result<RwLockReadGuard<'_, Database>> {
        self.database
            .read()
            .map_err(|_| Error::Substrate("rrflowKV database lock poisoned".into()))
    }
}
