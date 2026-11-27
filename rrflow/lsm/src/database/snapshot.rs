//! Snapshot bundle export and install.

use super::*;

impl Database {
    /// Flushes the active WAL-backed memtable and captures the resulting
    /// immutable manifest closure. The published manifest points at an empty
    /// successor WAL, so every state byte required by the snapshot is carried
    /// by the authenticated segment inventory.
    pub fn export_snapshot_bundle(&mut self, at: u64) -> Result<SnapshotBundle> {
        self.flush_memtable(at)?;
        let segments = self
            .manifest
            .segments
            .iter()
            .map(|descriptor| {
                let bytes = self
                    .root
                    .join(SEGMENT_DIRECTORY)
                    .join(format!("{}.seg", descriptor.id))
                    .read()?;
                Ok(SnapshotSegment {
                    descriptor: descriptor.clone(),
                    bytes,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        SnapshotBundle::new(self.manifest.as_ref().clone(), segments)
    }

    /// Flushes and writes the authenticated snapshot bundle directly to a new
    /// file, retaining only a fixed copy buffer plus one segment during deep
    /// validation.
    pub fn export_snapshot_file(
        &mut self,
        at: u64,
        destination: impl crate::filesystem::AnchorInput,
    ) -> Result<SnapshotBundleFile> {
        self.flush_memtable(at)?;
        SnapshotBundleFile::create(
            self.manifest.as_ref().clone(),
            &self.root.join(SEGMENT_DIRECTORY),
            destination,
        )
    }

    pub fn export_snapshot_file_with_failure(
        &mut self,
        at: u64,
        destination: impl crate::filesystem::AnchorInput,
        boundary: SnapshotExportBoundary,
        mode: FailureMode,
    ) -> Result<SnapshotBundleFile> {
        self.flush_memtable(at)?;
        SnapshotBundleFile::create_with_hook(
            self.manifest.as_ref().clone(),
            &self.root.join(SEGMENT_DIRECTORY),
            destination,
            |reached| {
                if reached == boundary {
                    Err(Error::InjectedFailure {
                        mode: mode.as_str(),
                        boundary: boundary.as_str(),
                    })
                } else {
                    Ok(())
                }
            },
        )
    }

    /// Replaces the current logical database image with a newer authenticated
    /// snapshot. Segment files and an empty continuation WAL are durable before
    /// a single manifest-pointer publication makes the imported state visible.
    pub fn install_snapshot_bundle(
        &mut self,
        bundle: &SnapshotBundle,
        at: u64,
    ) -> Result<Manifest> {
        self.install_snapshot_bundle_inner(bundle, at, None)
    }

    /// Installs a file-backed bundle without decoding the whole transfer into
    /// one allocation. RRD LSM currently retains opened segments in memory, so
    /// the resident engine image remains a separately measured bound.
    pub fn install_snapshot_file(
        &mut self,
        bundle: &SnapshotBundleFile,
        at: u64,
    ) -> Result<Manifest> {
        self.install_snapshot_file_inner(bundle, at, None)
    }

    pub fn install_snapshot_file_with_failure(
        &mut self,
        bundle: &SnapshotBundleFile,
        at: u64,
        boundary: SnapshotInstallBoundary,
        mode: FailureMode,
    ) -> Result<Manifest> {
        self.install_snapshot_file_inner(bundle, at, Some((boundary, mode)))
    }

    pub(super) fn install_snapshot_file_inner(
        &mut self,
        bundle: &SnapshotBundleFile,
        at: u64,
        failure: Option<(SnapshotInstallBoundary, FailureMode)>,
    ) -> Result<Manifest> {
        self.ensure_writer_ready()?;
        self.ensure_no_active_transactions("snapshot installation")?;
        if bundle.source_manifest.application_format != self.manifest.application_format {
            return Err(Error::InvalidManifest(
                "snapshot application format differs from the target database".into(),
            ));
        }
        let source_sequence = bundle.source_manifest.durable_sequence;
        let current_sequence = self.snapshot().sequence;
        if source_sequence == current_sequence
            && self.manifest.segments == bundle.source_manifest.segments
        {
            return Ok(self.manifest.as_ref().clone());
        }
        if source_sequence <= current_sequence {
            return Err(Error::InvalidManifest(format!(
                "snapshot sequence {source_sequence} does not advance local sequence {current_sequence}"
            )));
        }

        let segment_directory = self.root.join(SEGMENT_DIRECTORY);
        let mut imported = Vec::with_capacity(bundle.source_manifest.segments.len());
        for (index, descriptor) in bundle.descriptors().enumerate() {
            let bytes = bundle.segment_bytes(index)?;
            imported.push(Arc::new(Segment::install_snapshot_bytes_with_cache(
                &segment_directory,
                descriptor,
                &bytes,
                Arc::clone(&self.page_cache),
                Arc::clone(&self.segment_io),
            )?));
        }
        inject_snapshot_install_failure(failure, SnapshotInstallBoundary::SegmentsSynced)?;

        let next_sequence = source_sequence
            .checked_add(1)
            .ok_or_else(|| Error::InvalidManifest("snapshot sequence overflowed".into()))?;
        let next_path = wal_path(&self.root, next_sequence);
        let successor = if next_path.try_exists()? {
            WalWriter::open_empty_at_anchored(&next_path, next_sequence)?
        } else {
            WalWriter::create_at_anchored(&next_path, next_sequence)?
        };
        inject_snapshot_install_failure(failure, SnapshotInstallBoundary::SuccessorWalSynced)?;

        let previous = self.manifest.digest.clone();
        let manifest = new_manifest(
            self.manifest
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::InvalidManifest("manifest generation overflowed".into()))?,
            Some(previous.clone()),
            at.max(self.manifest.created_at)
                .max(bundle.source_manifest.created_at),
            source_sequence,
            next_sequence,
            bundle.source_manifest.segments.clone(),
            self.manifest.application_format,
        )?;
        let point_index = PointIndex::new(&manifest.segments);
        self.manifests.publish(&manifest, Some(&previous))?;
        self.wal = successor;
        self.memtable = self.mutable_memory.empty(source_sequence);
        self.wal_payload_bytes = 0;
        self.segments = imported;
        self.point_index = point_index;
        self.manifest = Arc::new(manifest.clone());
        self.reclamation_pending = true;
        inject_snapshot_install_failure(failure, SnapshotInstallBoundary::ManifestPublished)?;
        Ok(manifest)
    }

    pub fn install_snapshot_bundle_with_failure(
        &mut self,
        bundle: &SnapshotBundle,
        at: u64,
        boundary: SnapshotInstallBoundary,
        mode: FailureMode,
    ) -> Result<Manifest> {
        self.install_snapshot_bundle_inner(bundle, at, Some((boundary, mode)))
    }

    pub(super) fn install_snapshot_bundle_inner(
        &mut self,
        bundle: &SnapshotBundle,
        at: u64,
        failure: Option<(SnapshotInstallBoundary, FailureMode)>,
    ) -> Result<Manifest> {
        self.ensure_writer_ready()?;
        self.ensure_no_active_transactions("snapshot installation")?;
        bundle.validate()?;
        if bundle.source_manifest.application_format != self.manifest.application_format {
            return Err(Error::InvalidManifest(
                "snapshot application format differs from the target database".into(),
            ));
        }
        let source_sequence = bundle.source_manifest.durable_sequence;
        let current_sequence = self.snapshot().sequence;
        if source_sequence == current_sequence
            && self.manifest.segments == bundle.source_manifest.segments
        {
            return Ok(self.manifest.as_ref().clone());
        }
        if source_sequence <= current_sequence {
            return Err(Error::InvalidManifest(format!(
                "snapshot sequence {source_sequence} does not advance local sequence {current_sequence}"
            )));
        }

        let segment_directory = self.root.join(SEGMENT_DIRECTORY);
        let mut imported = Vec::with_capacity(bundle.segments.len());
        for bundled in &bundle.segments {
            imported.push(Arc::new(Segment::install_snapshot_bytes_with_cache(
                &segment_directory,
                &bundled.descriptor,
                &bundled.bytes,
                Arc::clone(&self.page_cache),
                Arc::clone(&self.segment_io),
            )?));
        }
        inject_snapshot_install_failure(failure, SnapshotInstallBoundary::SegmentsSynced)?;

        let next_sequence = source_sequence
            .checked_add(1)
            .ok_or_else(|| Error::InvalidManifest("snapshot sequence overflowed".into()))?;
        let next_path = wal_path(&self.root, next_sequence);
        let successor = if next_path.try_exists()? {
            WalWriter::open_empty_at_anchored(&next_path, next_sequence)?
        } else {
            WalWriter::create_at_anchored(&next_path, next_sequence)?
        };
        inject_snapshot_install_failure(failure, SnapshotInstallBoundary::SuccessorWalSynced)?;

        let previous = self.manifest.digest.clone();
        let manifest = new_manifest(
            self.manifest
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::InvalidManifest("manifest generation overflowed".into()))?,
            Some(previous.clone()),
            at.max(self.manifest.created_at)
                .max(bundle.source_manifest.created_at),
            source_sequence,
            next_sequence,
            bundle.source_manifest.segments.clone(),
            self.manifest.application_format,
        )?;
        let point_index = PointIndex::new(&manifest.segments);
        self.manifests.publish(&manifest, Some(&previous))?;
        self.wal = successor;
        self.memtable = self.mutable_memory.empty(source_sequence);
        self.wal_payload_bytes = 0;
        self.segments = imported;
        self.point_index = point_index;
        self.manifest = Arc::new(manifest.clone());
        self.reclamation_pending = true;
        inject_snapshot_install_failure(failure, SnapshotInstallBoundary::ManifestPublished)?;
        Ok(manifest)
    }
}
