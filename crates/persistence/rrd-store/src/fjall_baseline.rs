//! A Fjall-backed storage engine kept only as a comparison baseline.
//!
//! It exists so the same semantic workload and the same storage differentials
//! can run against a second, independent log-structured engine, and so
//! rrflowKV's results can be stated against one. It is compiled only with the
//! `fjall-baseline` feature, no production crate enables that feature, and
//! [`crate::StorageProfile`] cannot select it: the engine composition root
//! chooses rrflowKV or rrflowMX and nothing else.
//!
//! It implements the same profile-neutral byte transaction as rrflowMX and
//! rrflowKV, with identical observable semantics: one sequence number per
//! mutation assigned in batch order, a snapshot read plus the transaction's own
//! writes, a write-write conflict when any written key was last written after
//! the snapshot, and a read-only commit that carries no sequence interval.
//!
//! Physical layout: every logical key and value is stored unchanged in one
//! Fjall keyspace, so both engines see identical bytes. Two small keyspaces
//! hold what the contract needs and Fjall does not offer directly: the last
//! write sequence of every key (for conflict detection) and the global sequence.

use crate::transaction::{validate_range, validate_read_key, TransactionWriteSet};
use crate::{
    Durability, Error, PhysicalStoreEvidence, Result, StorageEngine, StorageTransaction,
    TransactionCommit, TransactionRollback,
};
use fjall::{
    KeyspaceCreateOptions, PersistMode, Readable, SingleWriterTxDatabase, SingleWriterTxKeyspace,
    Snapshot,
};
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};
use std::path::{Path, PathBuf};

const DATA: &str = "data";
const LAST_WRITE: &str = "last_write";
const META: &str = "meta";
const SEQUENCE_KEY: &[u8] = b"sequence";

fn fjall_error(error: impl std::fmt::Display) -> Error {
    Error::Substrate(format!("fjall baseline: {error}"))
}

fn decode_sequence(bytes: &[u8]) -> Result<u64> {
    let array: [u8; 8] = bytes
        .try_into()
        .map_err(|_| fjall_error("a stored sequence is not eight bytes"))?;
    Ok(u64::from_be_bytes(array))
}

/// The persist mode that makes a commit as durable as the contract requires.
/// The same mapping the original comparison used: authoritative writes are
/// synced before commit returns, buffered writes are not.
fn persist_mode(durability: Durability) -> PersistMode {
    match durability {
        Durability::Authoritative => PersistMode::SyncAll,
        Durability::Buffered => PersistMode::Buffer,
    }
}

/// The Fjall comparison engine. See the module documentation.
pub struct FjallBaselineStore {
    path: PathBuf,
    database: SingleWriterTxDatabase,
    data: SingleWriterTxKeyspace,
    last_write: SingleWriterTxKeyspace,
    meta: SingleWriterTxKeyspace,
}

impl FjallBaselineStore {
    /// Opens or creates a store at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        let database = SingleWriterTxDatabase::builder(path)
            .open()
            .map_err(fjall_error)?;
        let data = database
            .keyspace(DATA, KeyspaceCreateOptions::default)
            .map_err(fjall_error)?;
        let last_write = database
            .keyspace(LAST_WRITE, KeyspaceCreateOptions::default)
            .map_err(fjall_error)?;
        let meta = database
            .keyspace(META, KeyspaceCreateOptions::default)
            .map_err(fjall_error)?;
        Ok(Self {
            path: path.to_path_buf(),
            database,
            data,
            last_write,
            meta,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Flushes every keyspace, compacts it fully and syncs the journal: the
    /// maintenance boundary the benchmarks measure as "maintained".
    pub fn maintain(&self) -> Result<()> {
        for keyspace in [&self.data, &self.last_write, &self.meta] {
            keyspace
                .as_ref()
                .rotate_memtable_and_wait()
                .map_err(fjall_error)?;
            keyspace.as_ref().major_compact().map_err(fjall_error)?;
        }
        self.database
            .persist(PersistMode::SyncAll)
            .map_err(fjall_error)
    }

    fn sequence_in(&self, reader: &impl Readable) -> Result<u64> {
        reader
            .get(&self.meta, SEQUENCE_KEY)
            .map_err(fjall_error)?
            .map_or(Ok(0), |bytes| decode_sequence(&bytes))
    }
}

struct FjallBaselineTransaction<'a> {
    store: &'a FjallBaselineStore,
    snapshot: Snapshot,
    snapshot_sequence: u64,
    writes: TransactionWriteSet,
}

impl StorageTransaction for FjallBaselineTransaction<'_> {
    fn snapshot_sequence(&self) -> u64 {
        self.snapshot_sequence
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        validate_read_key(key)?;
        if let Some(value) = self.writes.get(key) {
            return Ok(value.map(<[u8]>::to_vec));
        }
        Ok(self
            .snapshot
            .get(&self.store.data, key)
            .map_err(fjall_error)?
            .map(|value| value.to_vec()))
    }

    fn scan(&self, start: &[u8], end: &[u8], limit: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        validate_range(start, end, limit)?;
        let overlay = self
            .writes
            .mutations()
            .range::<[u8], _>((Included(start), Excluded(end)))
            .collect::<Vec<_>>();
        // The transaction's own deletions can hide at most `overlay.len()`
        // snapshot rows, so this many are always enough to fill `limit`.
        let wanted = limit.saturating_add(overlay.len());
        let mut visible = BTreeMap::new();
        for guard in self
            .snapshot
            .range(&self.store.data, start.to_vec()..end.to_vec())
            .take(wanted)
        {
            let (key, value) = guard.into_inner().map_err(fjall_error)?;
            visible.insert(key.to_vec(), value.to_vec());
        }
        for (key, value) in overlay {
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

    fn contains_key(&self, key: &[u8]) -> Result<bool> {
        validate_read_key(key)?;
        if let Some(value) = self.writes.get(key) {
            return Ok(value.is_some());
        }
        self.snapshot
            .contains_key(&self.store.data, key)
            .map_err(fjall_error)
    }

    fn size_of(&self, key: &[u8]) -> Result<Option<u64>> {
        validate_read_key(key)?;
        if let Some(value) = self.writes.get(key) {
            return Ok(value.map(|value| value.len() as u64));
        }
        Ok(self
            .snapshot
            .size_of(&self.store.data, key)
            .map_err(fjall_error)?
            .map(u64::from))
    }

    fn scan_reverse(
        &self,
        start: &[u8],
        end: &[u8],
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        validate_range(start, end, limit)?;
        let overlay = self
            .writes
            .mutations()
            .range::<[u8], _>((Included(start), Excluded(end)))
            .collect::<Vec<_>>();
        // As in `scan`: the overlay hides at most `overlay.len()` stored rows.
        let wanted = limit.saturating_add(overlay.len());
        let mut visible = BTreeMap::new();
        for guard in self
            .snapshot
            .range(&self.store.data, start.to_vec()..end.to_vec())
            .rev()
            .take(wanted)
        {
            let (key, value) = guard.into_inner().map_err(fjall_error)?;
            visible.insert(key.to_vec(), value.to_vec());
        }
        for (key, value) in overlay {
            match value {
                Some(value) => {
                    visible.insert(key.clone(), value.clone());
                }
                None => {
                    visible.remove(key.as_slice());
                }
            }
        }
        Ok(visible.into_iter().rev().take(limit).collect())
    }

    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.writes.put(key, value)
    }

    fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        self.writes.delete(key)
    }

    fn measure_only(&mut self) -> Result<()> {
        self.writes.measure_only()
    }

    fn staged_encoded_bytes(&self) -> usize {
        self.writes.encoded_len()
    }

    fn is_measuring(&self) -> bool {
        self.writes.is_measuring()
    }

    fn commit(self: Box<Self>, durability: Durability) -> Result<TransactionCommit> {
        if self.writes.is_measuring() {
            return Err(Error::Substrate(
                "a measuring transaction cannot commit".into(),
            ));
        }
        let Self {
            store,
            snapshot_sequence,
            writes,
            ..
        } = *self;
        let mutation_count = writes.len();
        if mutation_count == 0 {
            return Ok(TransactionCommit {
                snapshot_sequence,
                mutation_count,
                first_sequence: None,
                last_sequence: None,
            });
        }
        let batch = writes.into_batch()?;
        // Fjall's single-writer lock serializes commits, so the conflict check,
        // the sequence assignment and the writes below are one atomic step.
        let mut tx = store
            .database
            .write_tx()
            .durability(Some(persist_mode(durability)));
        for operation in batch.operations() {
            if let Some(bytes) = tx
                .get(&store.last_write, operation.key())
                .map_err(fjall_error)?
            {
                let conflicting_sequence = decode_sequence(&bytes)?;
                if conflicting_sequence > snapshot_sequence {
                    return Err(Error::TransactionConflict {
                        snapshot_sequence,
                        conflicting_sequence,
                    });
                }
            }
        }
        let current = store.sequence_in(&tx)?;
        let mutation_count_u64 =
            u64::try_from(mutation_count).map_err(|_| Error::SequenceOverflow)?;
        let first_sequence = current.checked_add(1).ok_or(Error::SequenceOverflow)?;
        let last_sequence = current
            .checked_add(mutation_count_u64)
            .ok_or(Error::SequenceOverflow)?;
        for (offset, operation) in batch.into_operations().into_iter().enumerate() {
            let sequence = first_sequence + offset as u64;
            let key = match operation {
                rrd_lsm::Mutation::Put { key, value } => {
                    tx.insert(&store.data, key.clone(), value);
                    key
                }
                rrd_lsm::Mutation::Delete { key } => {
                    tx.remove(&store.data, key.clone());
                    key
                }
            };
            tx.insert(&store.last_write, key, sequence.to_be_bytes().to_vec());
        }
        tx.insert(
            &store.meta,
            SEQUENCE_KEY.to_vec(),
            last_sequence.to_be_bytes().to_vec(),
        );
        tx.commit().map_err(fjall_error)?;
        Ok(TransactionCommit {
            snapshot_sequence,
            mutation_count,
            first_sequence: Some(first_sequence),
            last_sequence: Some(last_sequence),
        })
    }

    fn rollback(self: Box<Self>) -> Result<TransactionRollback> {
        Ok(TransactionRollback {
            snapshot_sequence: self.snapshot_sequence,
            discarded_mutations: self.writes.len(),
        })
    }
}

impl StorageEngine for FjallBaselineStore {
    fn begin_transaction(&self) -> Result<Box<dyn StorageTransaction + '_>> {
        let snapshot = self.database.read_tx();
        let snapshot_sequence = self.sequence_in(&snapshot)?;
        Ok(Box::new(FjallBaselineTransaction {
            store: self,
            snapshot,
            snapshot_sequence,
            writes: TransactionWriteSet::default(),
        }))
    }

    fn claims(&self) -> crate::ClaimRepository<'_> {
        crate::ClaimRepository::new(self)
    }

    fn control(&self) -> crate::ControlRepository<'_> {
        crate::ControlRepository::new(self)
    }

    fn function_catalogue(&self) -> crate::FunctionCatalogueRepository<'_> {
        crate::FunctionCatalogueRepository::new(self)
    }

    fn projections(&self) -> crate::ProjectionRepository<'_> {
        crate::ProjectionRepository::new(self)
    }

    fn runtime(&self) -> crate::RuntimeRepository<'_> {
        crate::RuntimeRepository::new(self)
    }

    fn invocations(&self) -> crate::InvocationRepository<'_> {
        crate::InvocationRepository::new(self)
    }

    fn physical_store_evidence(&self) -> Result<PhysicalStoreEvidence> {
        Ok(PhysicalStoreEvidence::logical_only("fjall_baseline"))
    }
}
