//! Garbage collection of files no manifest retains.

use super::*;

pub(super) fn retain_manifest(
    manifest: &Manifest,
    manifests: &mut BTreeSet<String>,
    segments: &mut BTreeMap<String, u64>,
    wals: &mut BTreeSet<String>,
    budget: &mut crate::reclamation::ReclamationBudget,
) -> Result<()> {
    budget.entry()?;
    if manifests.contains(&manifest.digest) {
        return Ok(());
    }
    budget.charge_entries(manifest.segments.len() as u64)?;
    manifest.validate()?;
    manifests.insert(manifest.digest.clone());
    wals.insert(format!("{:020}.wal", manifest.wal_start_sequence));
    for segment in &manifest.segments {
        budget.tick()?;
        if let Some(previous) = segments.insert(segment.id.clone(), segment.bytes) {
            if previous != segment.bytes {
                return Err(Error::InvalidManifest(
                    "protected manifests disagree about segment length".into(),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn unreachable_files(
    directory: &AnchoredPath,
    extension: &str,
    retained: &BTreeSet<String>,
    budget: &mut crate::reclamation::ReclamationBudget,
) -> Result<Vec<(AnchoredPath, String, u64)>> {
    let mut candidates = Vec::new();
    for entry in directory.entries()? {
        budget.entry()?;
        let entry = entry?;
        let metadata = entry.metadata()?;
        if !metadata.is_file()
            || entry
                .relative_path()
                .extension()
                .and_then(|value| value.to_str())
                != Some(extension)
        {
            continue;
        }
        let file_name = entry
            .relative_path()
            .file_name()
            .ok_or_else(|| Error::InvalidManifest("missing file name".into()))?
            .to_string_lossy()
            .into_owned();
        let identity = entry
            .relative_path()
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::InvalidManifest("non-UTF-8 storage object name".into()))?;
        if !retained.contains(if extension == "wal" {
            file_name.as_str()
        } else {
            identity
        }) {
            candidates.push((entry, file_name, metadata.len()));
        }
    }
    candidates.sort_by(|left, right| left.1.cmp(&right.1));
    Ok(candidates)
}

impl Database {
    /// Reclaims a bounded prefix only after validating the complete admitted
    /// current/view/checkpoint reachability closure and candidate inventory.
    pub fn garbage_collect(&self) -> Result<GarbageCollectionReport> {
        self.ensure_writer_ready()?;
        let mut budget = crate::reclamation::ReclamationBudget::new(self.reclamation);
        let mut retained_manifests = BTreeSet::new();
        let mut retained_segments = BTreeMap::new();
        let mut retained_wals = BTreeSet::new();
        retain_manifest(
            &self.manifest,
            &mut retained_manifests,
            &mut retained_segments,
            &mut retained_wals,
            &mut budget,
        )?;
        // A projected stream owns selected immutable segments directly. An old
        // manifest is not a durable checkpoint and need not retain unrelated files.
        ActiveReadViews::visit_segments(&self.active_read_views, |segment| {
            budget.entry()?;
            if let Some(previous) = retained_segments.insert(segment.id.clone(), segment.bytes) {
                if previous != segment.bytes {
                    return Err(Error::InvalidManifest(
                        "protected segment pins disagree about length".into(),
                    ));
                }
            }
            Ok(())
        })?;
        for checkpoint in self.manifests.checkpoints_bounded(&mut budget)? {
            if !retained_manifests.contains(&checkpoint.manifest) {
                let manifest = self
                    .manifests
                    .load_bounded(&checkpoint.manifest, &mut budget)?;
                retain_manifest(
                    &manifest,
                    &mut retained_manifests,
                    &mut retained_segments,
                    &mut retained_wals,
                    &mut budget,
                )?;
            }
        }
        let manifest_directory = self.root.join("manifests");
        let segment_directory = self.root.join(SEGMENT_DIRECTORY);
        let wal_directory = self.root.join(WAL_DIRECTORY);
        // A missing protected object is damage, not permission to clean up the
        // remaining evidence. Validate references before the first delete.
        for identity in &retained_manifests {
            self.manifests.load_bounded(identity, &mut budget)?;
        }
        for (identity, bytes) in &retained_segments {
            budget.entry()?;
            let metadata = segment_directory
                .join(format!("{identity}.seg"))
                .metadata()?;
            if !metadata.is_file() || metadata.len() != *bytes {
                return Err(Error::InvalidManifest("protected reclamation segment is missing, non-regular or has a different length".into()));
            }
        }
        for name in &retained_wals {
            budget.entry()?;
            if !wal_directory.join(name).metadata()?.is_file() {
                return Err(Error::InvalidManifest(
                    "protected reclamation WAL is not a regular file".into(),
                ));
            }
        }
        let segment_ids = retained_segments.into_keys().collect::<BTreeSet<_>>();
        let manifest_candidates = unreachable_files(
            &manifest_directory,
            "json",
            &retained_manifests,
            &mut budget,
        )?;
        let segment_candidates =
            unreachable_files(&segment_directory, "seg", &segment_ids, &mut budget)?;
        let wal_candidates = unreachable_files(&wal_directory, "wal", &retained_wals, &mut budget)?;
        budget.tick()?;
        let mut report = GarbageCollectionReport {
            retained_manifests: retained_manifests.into_iter().collect(),
            retained_segments: segment_ids.into_iter().collect(),
            retained_wals: retained_wals.into_iter().collect(),
            ..GarbageCollectionReport::default()
        };
        report.has_more |= remove_candidates(
            &manifest_directory,
            manifest_candidates,
            &mut report.removed_manifests,
            &mut budget,
        )?;
        report.has_more |= remove_candidates(
            &segment_directory,
            segment_candidates,
            &mut report.removed_segments,
            &mut budget,
        )?;
        report.has_more |= remove_candidates(
            &wal_directory,
            wal_candidates,
            &mut report.removed_wals,
            &mut budget,
        )?;
        report.examined_entries = budget.entries;
        report.metadata_bytes = budget.metadata_bytes;
        report.removed_bytes = budget.deleted_bytes;
        Ok(report)
    }
}
