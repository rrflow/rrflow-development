//! Deterministic failure injection at each write, flush, compaction and snapshot boundary.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureMode {
    Crash,
    StorageFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteBoundary {
    Prepared,
    WalAppended,
    WalSynced,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlushBoundary {
    WalSynced,
    SegmentSynced,
    SuccessorWalSynced,
    ManifestPublished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionBoundary {
    SegmentSynced,
    ManifestPublished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotInstallBoundary {
    SegmentsSynced,
    SuccessorWalSynced,
    ManifestPublished,
}

pub(super) fn inject_failure(
    configured: Option<(FlushBoundary, FailureMode)>,
    reached: FlushBoundary,
) -> Result<()> {
    if let Some((boundary, mode)) = configured.filter(|(boundary, _)| *boundary == reached) {
        return Err(Error::InjectedFailure {
            mode: mode.as_str(),
            boundary: boundary.as_str(),
        });
    }
    Ok(())
}

pub(super) fn inject_write_failure(
    configured: Option<(WriteBoundary, FailureMode)>,
    reached: WriteBoundary,
) -> Result<()> {
    if let Some((boundary, mode)) = configured.filter(|(boundary, _)| *boundary == reached) {
        return Err(Error::InjectedFailure {
            mode: mode.as_str(),
            boundary: boundary.as_str(),
        });
    }
    Ok(())
}

pub(super) fn append_durability(
    requested: Durability,
    failure: Option<(WriteBoundary, FailureMode)>,
) -> Durability {
    if matches!(failure, Some((WriteBoundary::WalAppended, _))) {
        // The injected boundary must occur after a complete frame write but
        // before an authoritative sync. Recovery may therefore observe the
        // complete frame, but can never observe only part of its operations.
        Durability::Buffered
    } else {
        requested
    }
}

pub(super) fn inject_snapshot_install_failure(
    configured: Option<(SnapshotInstallBoundary, FailureMode)>,
    reached: SnapshotInstallBoundary,
) -> Result<()> {
    if let Some((boundary, mode)) = configured.filter(|(boundary, _)| *boundary == reached) {
        return Err(Error::InjectedFailure {
            mode: mode.as_str(),
            boundary: boundary.as_str(),
        });
    }
    Ok(())
}

pub(super) fn inject_compaction_failure(
    configured: Option<(CompactionBoundary, FailureMode)>,
    reached: CompactionBoundary,
) -> Result<()> {
    if let Some((boundary, mode)) = configured.filter(|(boundary, _)| *boundary == reached) {
        return Err(Error::InjectedFailure {
            mode: mode.as_str(),
            boundary: boundary.as_str(),
        });
    }
    Ok(())
}

impl Database {
    pub(super) fn inject_post_append_failure(
        &mut self,
        failure: Option<(WriteBoundary, FailureMode)>,
    ) -> Result<()> {
        let Some((WriteBoundary::WalAppended, mode)) = failure else {
            return Ok(());
        };
        let boundary = WriteBoundary::WalAppended.as_str();
        self.writer_requires_reopen = Some(boundary);
        Err(Error::InjectedFailure {
            mode: mode.as_str(),
            boundary,
        })
    }

    pub(super) fn inject_post_wal_failure(
        &mut self,
        failure: Option<(WriteBoundary, FailureMode)>,
        receipt: &AppendReceipt,
    ) -> Result<()> {
        let Some((WriteBoundary::WalSynced, mode)) = failure else {
            return Ok(());
        };
        if !receipt.durable {
            self.wal.sync()?;
        }
        let boundary = WriteBoundary::WalSynced.as_str();
        self.writer_requires_reopen = Some(boundary);
        Err(Error::InjectedFailure {
            mode: mode.as_str(),
            boundary,
        })
    }

    pub(super) fn inject_post_visibility_failure(
        &mut self,
        failure: Option<(WriteBoundary, FailureMode)>,
    ) -> Result<()> {
        #[cfg(test)]
        crate::fault_io::crash::visible(self.root.diagnostic_path());
        #[cfg(all(feature = "crash-test", not(test)))]
        crate::crash_test::boundary(
            crate::crash_test::Point::Visible,
            self.root.diagnostic_path(),
        );
        let Some((WriteBoundary::Visible, mode)) = failure else {
            return Ok(());
        };
        let boundary = WriteBoundary::Visible.as_str();
        self.writer_requires_reopen = Some(boundary);
        Err(Error::InjectedFailure {
            mode: mode.as_str(),
            boundary,
        })
    }
}
