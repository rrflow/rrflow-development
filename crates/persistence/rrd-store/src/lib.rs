//! # rrd-store
//!
//! The semantic storage boundary shared by persistent rrflowKV and volatile
//! rrflowMX.
//!
//! rrflowKV is the only durable implementation. rrflowMX is the non-durable
//! conformance and embedded-memory profile. Both expose the same stamped
//! transactions, temporal claims, snapshots, audit chain, and projection work;
//! callers select a profile only at the engine composition root.

mod access;
mod archive;
mod backup;
mod control;
mod ds;
mod engine;
mod error;
mod footprint;
mod gc;
mod invocation;
mod key_codec;
mod keyspaces;
mod object;
mod outcome;
mod projection;
mod repository;
mod rrflow_kv;
mod s3;
mod transaction;
mod writer;

pub use access::SemanticCommitPlan;
pub use archive::{
    export_logical_archive, export_logical_archive_with_progress, inspect_logical_archive,
    restore_logical_archive_to_new_root, restore_logical_archive_to_new_root_with,
    restore_logical_archive_to_new_root_with_progress, LogicalArchiveCheckpoint,
    LogicalArchiveInventory, LogicalArchiveOperation, LogicalArchiveProgress, LogicalRestoreReport,
    LOGICAL_ARCHIVE_VERSION,
};
pub use backup::{
    create_application_backup, create_logical_backup, load_backup_catalogue,
    prune_backup_catalogue, restore_catalogued_backup, verify_backup_catalogue,
    verify_restored_backup_objects, BackupCatalogue, BackupCoverage, BackupEntry,
    BackupPruneOutcome, BackupPrunePlan, BackupPruneReceipt, CatalogueManifestInventory,
    ObjectPayloadManifestInventory, BACKUP_CATALOGUE_VERSION, MAX_BACKUP_PRUNE_RECEIPTS,
};
pub use control::{ControlJournalEntry, ControlTransition};
pub use ds::{DataRuntime, DataRuntimeAccess, DataRuntimeRef, DataRuntimeStep};
pub use engine::{PhysicalStoreEvidence, RrflowMxStore, StorageEngine, StorageProfile};
pub use error::{Error, Result};
pub use footprint::{measure_storage_footprint, FootprintBytes, StorageFootprint};
pub use gc::{PairStatus, RemovalReport, Verdict};
pub use invocation::{Invocation, InvocationInput, Outcome, Trigger};
pub use keyspaces::Durability;
pub use object::{
    ImmutableObjectStore, LocalObjectStore, MemoryObjectStore, ObjectInventory,
    ObjectInventoryEntry, ObjectInventoryState, ObjectStep, ObjectStoreBox, VerifiedObject,
};
pub use outcome::{AppendOutcome, IdempotentAppendOutcome};
pub use projection::{
    CurrentProjection, GroundedStamp, GroundingReport, ProjectionStatus, RebuildOutcome,
    CURRENT_PROJECTION,
};
pub use repository::{
    ClaimRepository, ControlRepository, InvocationRepository, ProjectionRepository,
    RuntimeRepository,
};
pub use rrd_core::{
    DataTransaction, DataTransactionView, ReadStamp, RetentionPin, RetentionPinId,
    RuntimeChangePage, RuntimeCommitOutcome, SnapshotHandle, SnapshotId,
};
pub use rrd_lsm::{
    publish_rename as publish_durable_rename, sync_directory as sync_directory_metadata,
};
pub use rrflow_kv::{
    prepare_rrflow_kv_commit, rrflow_kv_commit_context, rrflow_kv_commit_outcome,
    rrflow_kv_database_artifact_view, rrflow_kv_snapshot_all_object_references,
    rrflow_kv_snapshot_artifact_view, rrflow_kv_snapshot_object_references, RrflowKvStore,
};
pub use s3::{
    ConditionalPut, S3Authentication, S3CompatibleObjectStore, S3MultipartUpload, S3ObjectClient,
    S3ObjectMetadata, S3TransferPolicy, S3TransportCapabilities, S3UploadedPart,
    DEFAULT_S3_RANGE_BYTES, S3_MAX_PARTS, S3_MAX_PART_BYTES, S3_MIN_PART_BYTES,
};
pub use transaction::{StorageTransaction, TransactionCommit, TransactionRollback};
pub use writer::{ClaimBatchWriter, ClaimBatchWriterConfig, ClaimBatchWriterStats};
