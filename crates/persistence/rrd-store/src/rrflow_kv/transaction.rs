//! The rrflowKV storage transaction: staged reads, writes, measuring and commit.

use super::*;

impl StorageTransaction for RrflowKvTransaction<'_> {
    fn snapshot_sequence(&self) -> u64 {
        self.transaction.snapshot().sequence
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let database = self.store.read_lock()?;
        self.transaction.get(&database, key).map_err(Error::from)
    }

    fn scan(&self, start: &[u8], end: &[u8], limit: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let database = self.store.read_lock()?;
        self.transaction
            .scan(&database, start, end, limit)
            .map_err(Error::from)
    }

    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.transaction.put(key, value).map_err(Error::from)
    }

    fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        self.transaction.delete(key).map_err(Error::from)
    }

    fn measure_only(&mut self) -> Result<()> {
        self.transaction.measure_only().map_err(Error::from)
    }

    fn staged_encoded_bytes(&self) -> usize {
        self.transaction.encoded_len()
    }

    fn is_measuring(&self) -> bool {
        self.transaction.is_measuring()
    }

    fn commit(self: Box<Self>, durability: Durability) -> Result<TransactionCommit> {
        let Self {
            store,
            transaction,
            begin_mutex_wait_ns,
        } = *self;
        let diagnostics_enabled = store
            .native_write_path_diagnostics_enabled
            .load(Ordering::Acquire);
        let commit_lock_started = diagnostics_enabled.then(Instant::now);
        let mut database = store.lock()?;
        let commit_mutex_wait_ns = commit_lock_started.map_or(0, |started| {
            u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
        });
        let snapshot_prefix = keyspaces::space_prefix(keyspaces::RUNTIME_SNAPSHOTS);
        let removes_snapshot = transaction
            .pending_prefix(&snapshot_prefix)
            .any(|(_, value)| value.is_none());
        let snapshot_handles = transaction
            .pending_prefix(&snapshot_prefix)
            .filter_map(|(_, value)| value)
            .map(|bytes| {
                let handle: SnapshotHandle = serde_json::from_slice(bytes)?;
                handle.validate()?;
                Ok(handle)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut created_checkpoints = Vec::new();
        for handle in &snapshot_handles {
            match ensure_runtime_checkpoint(&mut database, handle, handle.created_at) {
                Ok(true) => created_checkpoints.push(runtime_checkpoint_name(&handle.id)),
                Ok(false) => {}
                Err(error) => {
                    for name in created_checkpoints {
                        let _ = database.release_checkpoint(&name);
                    }
                    return Err(error);
                }
            }
        }
        let physical_durability = match durability {
            Durability::Authoritative => rrd_lsm::Durability::Authoritative,
            Durability::Buffered => rrd_lsm::Durability::Buffered,
        };
        // No untracked physical Snapshot escapes this store. Foreground and
        // idle maintenance use the same registered-reader retention contract.
        let commit = database.commit_transaction_with_protected_snapshots(
            transaction,
            physical_durability,
            &[],
            WriteLockWait {
                begin_mutex_wait_ns,
                commit_mutex_wait_ns,
            },
        );
        let outcome = match commit {
            Ok(outcome) => outcome,
            Err(error) => {
                for name in created_checkpoints {
                    let _ = database.release_checkpoint(&name);
                }
                return Err(error.into());
            }
        };
        if removes_snapshot {
            reconcile_runtime_checkpoints(&mut database, None, 0)?;
        }
        let (first_sequence, last_sequence) = outcome
            .receipt
            .as_ref()
            .map(|receipt| (Some(receipt.first_sequence), Some(receipt.last_sequence)))
            .unwrap_or((None, None));
        tracing::debug!(
            target: "rrd_store::transaction",
            storage_profile = "rrflow_kv",
            snapshot_sequence = outcome.snapshot_sequence,
            mutation_count = outcome.mutation_count,
            first_sequence,
            last_sequence,
            outcome = if outcome.receipt.is_some() { "committed" } else { "read_only" },
            "storage transaction completed"
        );
        Ok(TransactionCommit {
            snapshot_sequence: outcome.snapshot_sequence,
            mutation_count: outcome.mutation_count,
            first_sequence,
            last_sequence,
        })
    }

    fn rollback(self: Box<Self>) -> Result<TransactionRollback> {
        let outcome = self.transaction.rollback()?;
        Ok(TransactionRollback {
            snapshot_sequence: outcome.snapshot_sequence,
            discarded_mutations: outcome.discarded_mutations,
        })
    }
}
