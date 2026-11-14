//! Ephemeral, bounded inspection of an installed physical storage snapshot.
use crate::{invalid, CanonicalId, CorrelationId, ReadStorageDiagnostics, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectStorageCursor {
    pub scope: String,
    pub cursor_id: CorrelationId,
    pub action: StorageCursorAction,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StorageCursorAction {
    Open {
        batch_rows: u16,
        batch_bytes: u32,
        lifetime_ms: u32,
    },
    /// Retain a snapshot over the explicit half-open physical key range.
    OpenRange {
        #[schemars(range(min = 1, max = 4096))]
        batch_rows: u16,
        #[schemars(range(min = 1, max = 4194304))]
        batch_bytes: u32,
        #[schemars(range(min = 1, max = 60000))]
        lifetime_ms: u32,
        #[schemars(length(min = 1, max = 4096))]
        start: Vec<u8>,
        #[schemars(length(min = 1, max = 4096))]
        end: Vec<u8>,
    },
    /// Inspect current records of one kind in the authorized instance scope.
    /// The store derives the physical range; clients do not encode storage keys.
    OpenRecordKind {
        #[schemars(range(min = 1, max = 4096))]
        batch_rows: u16,
        #[schemars(range(min = 1, max = 4194304))]
        batch_bytes: u32,
        #[schemars(range(min = 1, max = 60000))]
        lifetime_ms: u32,
        record_kind: CanonicalId,
    },
    Observe {},
    Advance {
        expected_ordinal: u64,
    },
    CancelSource {},
    Release {},
}
impl InspectStorageCursor {
    pub fn validate(&self) -> Result<()> {
        ReadStorageDiagnostics {
            scope: self.scope.clone(),
        }
        .validate()?;
        let budgets = match &self.action {
            StorageCursorAction::Open {
                batch_rows,
                batch_bytes,
                lifetime_ms,
            }
            | StorageCursorAction::OpenRange {
                batch_rows,
                batch_bytes,
                lifetime_ms,
                ..
            }
            | StorageCursorAction::OpenRecordKind {
                batch_rows,
                batch_bytes,
                lifetime_ms,
                ..
            } => Some((*batch_rows, *batch_bytes, *lifetime_ms)),
            _ => None,
        };
        if let Some((rows, bytes, lifetime)) = budgets {
            if !(1..=4096).contains(&rows)
                || !(1..=4 * 1024 * 1024).contains(&bytes)
                || !(1..=60_000).contains(&lifetime)
            {
                return invalid("invalid storage cursor batch or lifetime budget");
            }
        }
        if let StorageCursorAction::OpenRange { start, end, .. } = &self.action {
            if start.is_empty()
                || end.is_empty()
                || start.len() > 4096
                || end.len() > 4096
                || start >= end
            {
                return invalid("invalid bounded storage cursor key range");
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StorageCursorSnapshot {
    pub cursor_id: CorrelationId,
    pub released: bool,
    pub source_open: bool,
    pub snapshot_sequence: u64,
    pub page_ordinal: u64,
    pub rows: u64,
    pub allocated_bytes: u64,
    pub page_sha256: String,
}
impl StorageCursorSnapshot {
    pub fn validate(&self) -> Result<()> {
        crate::validate_sha256(&self.page_sha256, "page_sha256")?;
        if self.rows > 4096
            || self.allocated_bytes > 4 * 1024 * 1024
            || (self.released && (self.source_open || self.rows != 0 || self.allocated_bytes != 0))
        {
            return invalid("invalid storage cursor ownership snapshot");
        }
        Ok(())
    }
}
