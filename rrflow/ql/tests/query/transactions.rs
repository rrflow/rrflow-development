use rrflow_core::{DataTransaction, RuntimeCommit, RuntimeCommitOutcome};
use rrflow_db::{Repositories, RuntimeReadBudget, StorageEngine};

/// Publish the fixture at its declared read cursor, including schema-authorized edge effects.
pub fn publish(storage: &dyn StorageEngine, commit: RuntimeCommit) -> RuntimeCommitOutcome {
    let runtime = storage.runtime();
    let read = runtime.read_stamp(&commit.scope).unwrap();
    let budget =
        RuntimeReadBudget::relation_integrity(read.commit_cursor, commit.mutations.len()).unwrap();
    let mut transaction = DataTransaction::new(read, commit).unwrap();
    let (resolved, _) = runtime
        .resolve_relation_actions(&transaction, budget)
        .unwrap();
    transaction.commit = resolved;
    runtime.commit_data_transaction(&transaction).unwrap()
}
