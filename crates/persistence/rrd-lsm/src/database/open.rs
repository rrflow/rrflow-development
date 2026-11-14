//! Creating and opening a database: layout validation, manifest and WAL recovery.

use super::*;

pub(super) fn new_manifest(
    generation: u64,
    parent: Option<String>,
    created_at: u64,
    durable_sequence: u64,
    wal_start_sequence: u64,
    segments: Vec<crate::SegmentDescriptor>,
    application_format: Option<u64>,
) -> Result<Manifest> {
    match application_format {
        Some(application_format) => Manifest::new_with_application_format(
            generation,
            parent,
            created_at,
            durable_sequence,
            wal_start_sequence,
            segments,
            application_format,
        ),
        None => Manifest::new(
            generation,
            parent,
            created_at,
            durable_sequence,
            wal_start_sequence,
            segments,
        ),
    }
}

pub(super) fn validate_existing_database_layout(root: &AnchoredPath) -> Result<()> {
    for relative in ["", WAL_DIRECTORY, SEGMENT_DIRECTORY] {
        let path = root.join(relative);
        let metadata = path.metadata().map_err(|error| {
            Error::io(
                "validating database directory",
                path.diagnostic_path(),
                error,
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(Error::InvalidManifest(format!(
                "database directory {} is missing, symbolic, or invalid",
                path.display()
            )));
        }
    }
    Ok(())
}

pub(super) fn require_existing_regular_file(path: &AnchoredPath, object: &str) -> Result<()> {
    let metadata = path
        .metadata()
        .map_err(|error| Error::io("validating database file", path.diagnostic_path(), error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Error::InvalidManifest(format!(
            "{object} {} is symbolic or invalid",
            path.display()
        )));
    }
    Ok(())
}

pub(crate) fn wal_path(root: &AnchoredPath, starting_sequence: u64) -> AnchoredPath {
    root.join(WAL_DIRECTORY)
        .join(format!("{starting_sequence:020}.wal"))
}

impl Database {
    pub fn create(root: &Path) -> Result<Self> {
        Self::create_with_options(root, DatabaseOptions::default())
    }

    pub fn create_with_page_cache(root: &Path, page_cache_bytes: usize) -> Result<Self> {
        Self::create_with_options(
            root,
            DatabaseOptions {
                page_cache_bytes,
                ..DatabaseOptions::default()
            },
        )
    }

    pub fn create_with_options(root: &Path, options: DatabaseOptions) -> Result<Self> {
        Self::create_with_options_and_application_format(root, options, None)
    }

    /// Creates a database whose manifests authenticate the higher-level key
    /// encoding interpreted by its owner. The physical engine treats this
    /// non-zero identity as opaque and preserves it across every publication.
    pub fn create_with_application_format(
        root: &Path,
        options: DatabaseOptions,
        application_format: u64,
    ) -> Result<Self> {
        if application_format == 0 {
            return Err(Error::InvalidConfiguration(
                "application format identity must be non-zero".into(),
            ));
        }
        Self::create_with_options_and_application_format(root, options, Some(application_format))
    }

    pub(super) fn create_with_options_and_application_format(
        root: &Path,
        options: DatabaseOptions,
        application_format: Option<u64>,
    ) -> Result<Self> {
        Self::create_anchored(&AnchoredPath::from_path(root)?, options, application_format)
    }

    pub fn create_anchored(
        root: &AnchoredPath,
        options: DatabaseOptions,
        application_format: Option<u64>,
    ) -> Result<Self> {
        if application_format == Some(0) {
            return Err(Error::InvalidConfiguration(
                "application format identity must be non-zero".into(),
            ));
        }
        let options = options.validate()?;
        let segment_io = IoContext::new(options.segment_io)?;
        match root.metadata() {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || !metadata.is_dir()
                    || root.entries()?.next().is_some()
                {
                    return Err(Error::InvalidManifest(
                        "new database path exists and is not an empty non-symlink directory".into(),
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                root.create_dir()?;
            }
            Err(error) => return Err(error.into()),
        }
        let root = root.pin_directory()?;
        let root = root.with_recovery_reserve(options.recovery_reserve_bytes)?;
        let manifests = ManifestStore::create_anchored(&root)?
            .with_checkpoint_limits(options.checkpoint_entries, options.reclamation)?;
        root.join(WAL_DIRECTORY).create_dir()?;
        root.join(SEGMENT_DIRECTORY).create_dir()?;
        let manifest = new_manifest(1, None, 0, 0, 1, Vec::new(), application_format)?;
        let wal = WalWriter::create_at_anchored(&wal_path(&root, 1), 1)?;
        manifests.publish(&manifest, None)?;
        let mutable_memory = crate::MutableMemory::new(options.mutable_retention);
        Ok(Self {
            root: root.to_owned(),
            manifests,
            point_index: PointIndex::new(&manifest.segments),
            manifest: Arc::new(manifest),
            wal,
            memtable: mutable_memory.empty(0),
            mutable_memory,
            segments: Vec::new(),
            page_cache: new_page_cache_with_limits(
                options.page_cache_bytes,
                options.page_cache_policy,
                options.source_page_bytes,
            )?,
            segment_io,
            segment_open_evidence: SegmentOpenEvidence::default(),
            segment_row_group_budget: options.segment_row_group_budget,
            segment_compression: options.segment_compression,
            maintenance: options.maintenance,
            mutable_limits: options.mutable_limits,
            compaction: options.compaction,
            compaction_limits: options.compaction_limits,
            reclamation: options.reclamation,
            reclamation_pending: true,
            maintenance_stats: MaintenanceStats::default(),
            wal_payload_bytes: 0,
            writer_requires_reopen: None,
            transaction_snapshots: Arc::new(crate::transaction::TransactionSnapshots::new(
                options.transaction_limits,
            )),
            active_read_views: Arc::new(std::sync::Mutex::new(ActiveReadViews::default())),
            active_read_view_limit: options.active_read_views,
            pinned_read_byte_limit: options.pinned_read_bytes,
            projected_output: crate::ProjectedOutputMemory::new(options.projected_output_bytes),
            write_path_diagnostics: None,
        })
    }

    pub fn open(root: &Path) -> Result<Self> {
        Self::open_with_options(root, DatabaseOptions::default())
    }

    pub fn open_with_page_cache(root: &Path, page_cache_bytes: usize) -> Result<Self> {
        Self::open_with_options(
            root,
            DatabaseOptions {
                page_cache_bytes,
                ..DatabaseOptions::default()
            },
        )
    }

    pub fn open_with_options(root: &Path, options: DatabaseOptions) -> Result<Self> {
        Self::open_anchored(&AnchoredPath::open_root(root)?, options)
    }

    pub fn open_anchored(root: &AnchoredPath, options: DatabaseOptions) -> Result<Self> {
        let root = root.pin_directory()?;
        let total_started = Instant::now();
        validate_existing_database_layout(&root)?;
        let options = options.validate()?;
        let root = root.with_recovery_reserve(options.recovery_reserve_bytes)?;
        let mut recovery_budget = crate::recovery_budget::RecoveryBudget::new(options.recovery);
        let segment_io = IoContext::new(options.segment_io)?;
        let manifest_started = Instant::now();
        let manifests = ManifestStore::open_existing_anchored(&root)?
            .with_checkpoint_limits(options.checkpoint_entries, options.reclamation)?;
        let (_, manifest) = manifests
            .current_bounded(&mut recovery_budget)?
            .ok_or_else(|| Error::InvalidManifest("database has no CURRENT manifest".into()))?;
        #[cfg(all(feature = "crash-test", not(test)))]
        crate::crash_test::boundary(
            crate::crash_test::Point::RecoveryManifest,
            root.diagnostic_path(),
        );
        let manifest_ms = manifest_started.elapsed().as_millis() as u64;
        let page_cache = new_page_cache_with_limits(
            options.page_cache_bytes,
            options.page_cache_policy,
            options.source_page_bytes,
        )?;
        let mut segment_bytes = 0u64;
        for segment in &manifest.segments {
            recovery_budget.file(segment.bytes)?;
            segment_bytes = segment_bytes.checked_add(segment.bytes).ok_or_else(|| {
                Error::InvalidManifest("recovery segment byte count overflow".into())
            })?;
        }
        let segments_started = Instant::now();
        let mut segments = Vec::with_capacity(manifest.segments.len());
        let mut segment_open_evidence = SegmentOpenEvidence::default();
        for expected in &manifest.segments {
            recovery_budget.tick()?;
            let segment_path = root
                .join(SEGMENT_DIRECTORY)
                .join(format!("{}.seg", expected.id));
            require_existing_regular_file(&segment_path, "segment")?;
            let segment = Segment::open_expected_with_cache_and_io(
                &segment_path,
                expected,
                Arc::clone(&page_cache),
                Arc::clone(&segment_io),
            )?;
            if &segment.descriptor != expected {
                return Err(Error::InvalidManifest(format!(
                    "segment {} does not match its manifest descriptor",
                    expected.id
                )));
            }
            segment_open_evidence.merge(segment.open_evidence())?;
            segments.push(segment);
        }
        let point_index = PointIndex::new(&manifest.segments);
        recovery_budget.tick()?;
        let segments_ms = segments_started.elapsed().as_millis() as u64;
        let path = wal_path(&root, manifest.wal_start_sequence);
        require_existing_regular_file(&path, "active WAL")?;
        let mut memtable = Memtable::at_sequence(manifest.durable_sequence);
        let wal_started = Instant::now();
        let mut wal_file = OpenOptions::new().read(true).write(true).open(&path)?;
        let recovery = crate::wal::replay_file_bounded(
            &mut wal_file,
            manifest.wal_start_sequence,
            &mut recovery_budget,
            |batch| {
                #[cfg(all(feature = "crash-test", not(test)))]
                crate::crash_test::boundary(
                    crate::crash_test::Point::RecoveryRead,
                    root.diagnostic_path(),
                );
                memtable.apply_owned_recovered(batch, options.mutable_limits)
            },
        )?;
        if let Some(offset) = recovery.torn_tail {
            return Err(Error::TornTail { offset });
        }
        let wal_recovery_ms = wal_started.elapsed().as_millis() as u64;
        let wal_payload_bytes = recovery.payload_bytes;
        let maintenance_stats = MaintenanceStats {
            peak_wal_payload_bytes: wal_payload_bytes,
            peak_memtable_versions: memtable.version_count(),
            peak_accounted_memtable_bytes: memtable.approximate_bytes(),
            ..MaintenanceStats::default()
        };
        let wal = WalWriter::from_recovered_file(&path, wal_file, recovery)?;
        tracing::info!(
            target: "rrd_lsm::open",
            path = %root.display(),
            manifest_ms,
            segment_count = manifest.segments.len(),
            segment_bytes,
            segments_ms,
            wal_recovery_ms,
            wal_payload_bytes,
            page_cache_policy = options.page_cache_policy.kind(),
            page_cache_protected_capacity_basis_points =
                options.page_cache_policy.protected_capacity_basis_points(),
            page_cache_capacity_bytes = options.page_cache_bytes,
            row_group_target_bytes = options.segment_row_group_budget.target_bytes,
            row_group_max_rows = options.segment_row_group_budget.max_rows,
            segment_compression = options.segment_compression.kind(),
            compression_minimum_savings_basis_points =
                options.segment_compression.minimum_savings_basis_points(),
            compression_maximum_page_logical_bytes =
                options.segment_compression.maximum_page_logical_bytes(),
            total_ms = total_started.elapsed().as_millis() as u64,
            "rrflowKV LSM open phases completed"
        );
        let mutable_memory = crate::MutableMemory::new(options.mutable_retention);
        Ok(Self {
            root: root.to_owned(),
            manifests,
            point_index,
            manifest: Arc::new(manifest),
            wal,
            memtable: Arc::new(mutable_memory.wrap(memtable)?),
            mutable_memory,
            segments: segments.into_iter().map(Arc::new).collect(),
            page_cache,
            segment_io,
            segment_open_evidence,
            segment_row_group_budget: options.segment_row_group_budget,
            segment_compression: options.segment_compression,
            maintenance: options.maintenance,
            mutable_limits: options.mutable_limits,
            compaction: options.compaction,
            compaction_limits: options.compaction_limits,
            reclamation: options.reclamation,
            reclamation_pending: true,
            maintenance_stats,
            wal_payload_bytes,
            writer_requires_reopen: None,
            transaction_snapshots: Arc::new(crate::transaction::TransactionSnapshots::new(
                options.transaction_limits,
            )),
            active_read_views: Arc::new(std::sync::Mutex::new(ActiveReadViews::default())),
            active_read_view_limit: options.active_read_views,
            pinned_read_byte_limit: options.pinned_read_bytes,
            projected_output: crate::ProjectedOutputMemory::new(options.projected_output_bytes),
            write_path_diagnostics: None,
        })
    }
}
