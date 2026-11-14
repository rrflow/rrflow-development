//! Read-only inspection, tail repair and reconciliation evidence.

use super::*;

impl RrflowKvInspector {
    pub fn open(path: &(impl crate::filesystem::AnchorInput + ?Sized)) -> Result<Self> {
        Self::open_with_options(path, DatabaseOptions::default())
    }

    /// Inspect existing bytes using the same native replay limits as writer
    /// open. This neither repairs storage nor changes persisted configuration.
    pub fn open_with_options(
        path: &(impl crate::filesystem::AnchorInput + ?Sized),
        options: DatabaseOptions,
    ) -> Result<Self> {
        Self::open_anchored_with_options(&path.anchor()?, options)
    }

    pub fn open_anchored(path: &AnchoredPath) -> Result<Self> {
        Self::open_anchored_with_options(path, DatabaseOptions::default())
    }

    pub fn open_anchored_with_options(
        path: &AnchoredPath,
        options: DatabaseOptions,
    ) -> Result<Self> {
        let database =
            ReadOnlyDatabase::open_anchored_with_options(path, options).map_err(Error::from)?;
        let inspection = inspect_database(&database)?;
        Ok(Self {
            database,
            inspection,
        })
    }

    pub fn inspection(&self) -> &RrflowKvInspection {
        &self.inspection
    }

    pub fn control_record(&self, key: &str) -> Result<Option<Vec<u8>>> {
        crate::control::validate_control_key(key)?;
        let logical = keyspaces::control_record_key(key);
        let physical = crate::access::runtime_state::checked_key(keyspaces::META, &logical)?;
        self.database.get(&physical).map_err(Error::from)
    }

    /// Reconstructs the canonical semantic read identity from the same
    /// authenticated, read-only physical snapshot used by this inspector.
    pub fn runtime_read_stamp(&self, scope: &ScopeId) -> Result<ReadStamp> {
        crate::access::runtime_state::read_stamp_with(&self.database, scope)
    }
}

impl crate::access::runtime_state::AccessRead for ReadOnlyDatabase {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.get(key).map_err(Error::from)
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: &[u8],
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut rows = self.scan(start, Some(end)).map_err(Error::from)?;
        rows.truncate(limit);
        Ok(rows)
    }
}

pub(super) fn inspect_database(database: &ReadOnlyDatabase) -> Result<RrflowKvInspection> {
    if keyspaces::RrflowKvKeyCodec::from_application_format(database.manifest().application_format)
        .is_none()
    {
        return Err(Error::Substrate(format!(
            "unsupported rrflowKV application format {:?}",
            database.manifest().application_format
        )));
    }
    let claim_sequence = inspect_sequence(database, &keyspaces::sequence_watermark_key())?;
    let runtime_cursor = inspect_sequence(database, &keyspaces::runtime_cursor_key())?;
    let control_journal_sequence =
        inspect_sequence(database, &keyspaces::control_journal_sequence_key())?;
    let inspection = RrflowKvInspection {
        physical: database.inspection().clone(),
        claim_sequence,
        runtime_cursor,
        control_journal_sequence,
    };
    Ok(inspection)
}

/// Explicit offline repair owner; ordinary inspectors retain no mutation API.
pub struct RrflowKvTailRepair {
    repair: rrd_lsm::TailRepair,
    inspection: RrflowKvInspection,
}
impl RrflowKvTailRepair {
    pub fn open_anchored(path: &AnchoredPath, options: DatabaseOptions) -> Result<Self> {
        let repair = rrd_lsm::TailRepair::open_anchored(path, options)?;
        let inspection = inspect_database(repair.reader())?;
        Ok(Self { repair, inspection })
    }
    pub fn inspection(&self) -> &RrflowKvInspection {
        &self.inspection
    }
    pub fn control_record(&self, key: &str) -> Result<Option<Vec<u8>>> {
        crate::control::validate_control_key(key)?;
        let logical = keyspaces::control_record_key(key);
        let physical = crate::access::runtime_state::checked_key(keyspaces::META, &logical)?;
        self.repair.reader().get(&physical).map_err(Error::from)
    }
    pub fn plan(&self) -> Result<rrd_lsm::TailRepairPlan> {
        self.repair.plan().map_err(Error::from)
    }
    pub fn apply(
        self,
        plan: &rrd_lsm::TailRepairPlan,
        expect: &str,
    ) -> Result<rrd_lsm::TailRepairReceipt> {
        self.repair.apply(plan, expect).map_err(Error::from)
    }
}

pub(super) fn inspect_sequence(database: &ReadOnlyDatabase, key: &[u8]) -> Result<u64> {
    let physical = crate::access::runtime_state::checked_key(keyspaces::META, key)?;
    database
        .get(&physical)
        .map_err(Error::from)?
        .as_deref()
        .map(crate::access::runtime_state::decode_sequence)
        .transpose()
        .map(Option::unwrap_or_default)
}

pub(super) fn reconciliation_evidence(
    cache_before: &PageCacheStats,
    cache_after: &PageCacheStats,
    io_before: &SegmentIoStats,
    io_after: &SegmentIoStats,
) -> Result<RrflowKvReconciliationEvidence> {
    if cache_before.capacity_bytes != cache_after.capacity_bytes
        || io_before.requested_mode != io_after.requested_mode
        || io_before.configured_max_request_bytes != io_after.configured_max_request_bytes
        || io_before.configured_max_open_files != io_after.configured_max_open_files
        || io_before.mmap_segments != io_after.mmap_segments
        || io_before.io_uring_segments != io_after.io_uring_segments
        || io_before.bounded_segments != io_after.bounded_segments
    {
        return Err(Error::Substrate(
            "rrflowKV open counters changed outside startup reconciliation".into(),
        ));
    }
    let delta = |before: u64, after: u64, name: &str| {
        after.checked_sub(before).ok_or_else(|| {
            Error::Substrate(format!(
                "rrflowKV startup reconciliation counter regressed: {name}"
            ))
        })
    };
    let page_cache_hits = delta(cache_before.hits, cache_after.hits, "cache hits")?;
    let page_cache_misses = delta(cache_before.misses, cache_after.misses, "cache misses")?;
    let page_requests = page_cache_hits
        .checked_add(page_cache_misses)
        .ok_or_else(|| {
            Error::Substrate("rrflowKV startup reconciliation page requests overflowed".into())
        })?;
    let read_operations = delta(
        io_before.read_operations,
        io_after.read_operations,
        "read operations",
    )?;
    let mmap_read_operations = delta(
        io_before.mmap_read_operations,
        io_after.mmap_read_operations,
        "mmap read operations",
    )?;
    let io_uring_read_operations = delta(
        io_before.io_uring_read_operations,
        io_after.io_uring_read_operations,
        "io_uring read operations",
    )?;
    let bounded_read_operations = delta(
        io_before.bounded_read_operations,
        io_after.bounded_read_operations,
        "bounded read operations",
    )?;
    let classified_reads = mmap_read_operations
        .checked_add(io_uring_read_operations)
        .and_then(|value| value.checked_add(bounded_read_operations))
        .ok_or_else(|| {
            Error::Substrate("rrflowKV startup reconciliation read classes overflowed".into())
        })?;
    if classified_reads != read_operations {
        return Err(Error::Substrate(
            "rrflowKV startup reconciliation read classes do not match total reads".into(),
        ));
    }
    let evidence = RrflowKvReconciliationEvidence {
        contract_version: RRFLOW_KV_OPEN_EVIDENCE_VERSION,
        page_requests,
        page_cache_hits,
        page_cache_misses,
        page_cache_loads: delta(cache_before.loads, cache_after.loads, "cache loads")?,
        page_cache_evictions: delta(
            cache_before.evictions,
            cache_after.evictions,
            "cache evictions",
        )?,
        page_bytes_read: delta(
            cache_before.bytes_read,
            cache_after.bytes_read,
            "page bytes read",
        )?,
        page_bytes_decoded: delta(
            cache_before.bytes_decoded,
            cache_after.bytes_decoded,
            "page bytes decoded",
        )?,
        page_bytes_borrowed: delta(
            cache_before.bytes_borrowed,
            cache_after.bytes_borrowed,
            "page bytes borrowed",
        )?,
        page_bytes_allocated: delta(
            cache_before.bytes_allocated,
            cache_after.bytes_allocated,
            "page bytes allocated",
        )?,
        page_bytes_copied: delta(
            cache_before.bytes_copied,
            cache_after.bytes_copied,
            "page bytes copied",
        )?,
        page_bytes_decompressed: delta(
            cache_before.bytes_decompressed,
            cache_after.bytes_decompressed,
            "page bytes decompressed",
        )?,
        filter_checks: delta(
            cache_before.filter_checks,
            cache_after.filter_checks,
            "filter checks",
        )?,
        filter_negatives: delta(
            cache_before.filter_negatives,
            cache_after.filter_negatives,
            "filter negatives",
        )?,
        io_fallbacks: delta(
            io_before.fallback_count,
            io_after.fallback_count,
            "I/O fallbacks",
        )?,
        read_operations,
        mmap_read_operations,
        io_uring_read_operations,
        bounded_read_operations,
        bytes_read: delta(io_before.bytes_read, io_after.bytes_read, "I/O bytes read")?,
    };
    if evidence.page_cache_loads != evidence.read_operations
        || evidence.page_bytes_read != evidence.bytes_read
    {
        return Err(Error::Substrate(
            "rrflowKV startup reconciliation page loads do not match physical reads".into(),
        ));
    }
    Ok(evidence)
}
