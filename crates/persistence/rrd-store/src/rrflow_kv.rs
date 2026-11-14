//! Persistent rrflowKV implementation of the physical [`StorageEngine`] port.
//!
//! Semantic repositories encode logical keyspaces as stable byte prefixes and
//! publish through this store's snapshot transaction. The database's physical
//! MVCC sequence is deliberately independent of claim and runtime cursors
//! stored in those transactions.

use crate::access::{prepare_semantic_commit, SemanticCommitPlan};
use crate::engine::{PhysicalStoreEvidence, StorageEngine, RRFLOW_KV_COUNTER_EVIDENCE};
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
use rrd_lsm::filesystem::AnchoredPath;
use rrd_lsm::{
    CompactionOutcome, Database, DatabaseInspection, DatabaseOptions, GarbageCollectionReport,
    Manifest, Mutation, PageCacheStats, ReadOnlyDatabase, SegmentIoStats, SegmentOpenEvidence,
    Snapshot, SnapshotBundleFile, WriteLockWait, WritePathDiagnostics,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Instant;

mod inspection_cursor;
pub use inspection_cursor::{StorageInspectionCursor, StorageInspectionPage};

mod codec;
mod commit;
mod engine;
mod inspection;
mod snapshot;
mod store;
mod transaction;
mod types;

use codec::*;
pub use commit::*;
pub use inspection::*;
pub use snapshot::*;
pub use store::*;
pub use types::*;

#[cfg(test)]
mod tests;
