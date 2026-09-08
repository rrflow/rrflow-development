use rrd_store::{Durability, Error, RrflowKvStore, StorageEngine, StorageProfile};
use std::sync::{Arc, Barrier};
use std::thread;

#[derive(Debug, PartialEq, Eq)]
struct ConformanceResult {
    ordered_items: Vec<(Vec<u8>, Vec<u8>)>,
    doctors: Vec<(Vec<u8>, Vec<u8>)>,
    retry_value: Option<Vec<u8>>,
    final_sequence: u64,
}

/// One behavioral corpus is deliberately invoked through both profiles. It
/// tests the transaction port itself rather than profile-specific helpers.
fn exercise(engine: &dyn StorageEngine) -> ConformanceResult {
    let mut seed = engine.begin_transaction().unwrap();
    seed.put(b"item:a".to_vec(), b"one".to_vec()).unwrap();
    seed.put(b"item:c".to_vec(), b"three".to_vec()).unwrap();
    let seed_outcome = seed.commit(Durability::Authoritative).unwrap();
    assert_eq!(seed_outcome.mutation_count, 2);
    assert_eq!(seed_outcome.first_sequence, Some(1));
    assert_eq!(seed_outcome.last_sequence, Some(2));

    let mut observer = engine.begin_transaction().unwrap();
    let frozen = observer.scan(b"item:", b"item;", 16).unwrap();
    assert_eq!(
        frozen,
        vec![
            (b"item:a".to_vec(), b"one".to_vec()),
            (b"item:c".to_vec(), b"three".to_vec()),
        ]
    );

    let mut concurrent = engine.begin_transaction().unwrap();
    concurrent.put(b"item:b".to_vec(), b"two".to_vec()).unwrap();
    concurrent.commit(Durability::Authoritative).unwrap();
    assert_eq!(
        observer.scan(b"item:", b"item;", 16).unwrap(),
        frozen,
        "the captured range must not observe a concurrent phantom"
    );
    observer
        .put(b"decision:observer".to_vec(), b"accepted".to_vec())
        .unwrap();
    observer.commit(Durability::Authoritative).unwrap();

    let mut local = engine.begin_transaction().unwrap();
    local.put(b"item:a".to_vec(), b"updated".to_vec()).unwrap();
    local.delete(b"item:c".to_vec()).unwrap();
    local.put(b"item:d".to_vec(), b"four".to_vec()).unwrap();
    assert_eq!(local.get(b"item:a").unwrap(), Some(b"updated".to_vec()));
    assert_eq!(local.get(b"item:c").unwrap(), None);
    assert_eq!(
        local.scan(b"item:", b"item;", 3).unwrap(),
        vec![
            (b"item:a".to_vec(), b"updated".to_vec()),
            (b"item:b".to_vec(), b"two".to_vec()),
            (b"item:d".to_vec(), b"four".to_vec()),
        ]
    );
    local.commit(Durability::Authoritative).unwrap();

    let before_rollback = engine.begin_transaction().unwrap().snapshot_sequence();
    let mut rolled_back = engine.begin_transaction().unwrap();
    rolled_back
        .put(b"discarded".to_vec(), b"never-visible".to_vec())
        .unwrap();
    let rollback = rolled_back.rollback().unwrap();
    assert_eq!(rollback.snapshot_sequence, before_rollback);
    assert_eq!(rollback.discarded_mutations, 1);
    let read_only = engine.begin_transaction().unwrap();
    assert_eq!(read_only.get(b"discarded").unwrap(), None);
    let read_only = read_only.commit(Durability::Authoritative).unwrap();
    assert_eq!(read_only.mutation_count, 0);
    assert_eq!(read_only.first_sequence, None);
    assert_eq!(read_only.last_sequence, None);

    let mut put = engine.begin_transaction().unwrap();
    let mut delete = engine.begin_transaction().unwrap();
    put.put(b"shared".to_vec(), b"put".to_vec()).unwrap();
    delete.delete(b"shared".to_vec()).unwrap();
    put.commit(Durability::Authoritative).unwrap();
    assert!(matches!(
        delete.commit(Durability::Authoritative),
        Err(Error::TransactionConflict { .. })
    ));

    let mut delete = engine.begin_transaction().unwrap();
    let mut update = engine.begin_transaction().unwrap();
    delete.delete(b"shared".to_vec()).unwrap();
    update
        .put(b"shared".to_vec(), b"stale-update".to_vec())
        .unwrap();
    delete.commit(Durability::Authoritative).unwrap();
    assert!(matches!(
        update.commit(Durability::Authoritative),
        Err(Error::TransactionConflict { .. })
    ));

    let mut stale_retry = engine.begin_transaction().unwrap();
    let mut winner = engine.begin_transaction().unwrap();
    stale_retry
        .put(b"retry".to_vec(), b"desired".to_vec())
        .unwrap();
    winner
        .put(b"retry".to_vec(), b"intervening".to_vec())
        .unwrap();
    winner.commit(Durability::Authoritative).unwrap();
    assert!(matches!(
        stale_retry.commit(Durability::Authoritative),
        Err(Error::TransactionConflict { .. })
    ));
    let mut retry = engine.begin_transaction().unwrap();
    assert_eq!(retry.get(b"retry").unwrap(), Some(b"intervening".to_vec()));
    retry.put(b"retry".to_vec(), b"desired".to_vec()).unwrap();
    retry.commit(Durability::Authoritative).unwrap();

    let mut doctors = engine.begin_transaction().unwrap();
    doctors
        .put(b"doctor:alice".to_vec(), b"on".to_vec())
        .unwrap();
    doctors.put(b"doctor:bob".to_vec(), b"on".to_vec()).unwrap();
    doctors.commit(Durability::Authoritative).unwrap();
    let mut alice = engine.begin_transaction().unwrap();
    let mut bob = engine.begin_transaction().unwrap();
    assert_eq!(alice.get(b"doctor:bob").unwrap(), Some(b"on".to_vec()));
    assert_eq!(bob.get(b"doctor:alice").unwrap(), Some(b"on".to_vec()));
    alice
        .put(b"doctor:alice".to_vec(), b"off".to_vec())
        .unwrap();
    bob.put(b"doctor:bob".to_vec(), b"off".to_vec()).unwrap();
    alice.commit(Durability::Authoritative).unwrap();
    bob.commit(Durability::Authoritative).unwrap();

    let final_read = engine.begin_transaction().unwrap();
    let result = ConformanceResult {
        ordered_items: final_read.scan(b"item:", b"item;", 16).unwrap(),
        doctors: final_read.scan(b"doctor:", b"doctor;", 16).unwrap(),
        retry_value: final_read.get(b"retry").unwrap(),
        final_sequence: final_read.snapshot_sequence(),
    };
    final_read.rollback().unwrap();
    result
}

#[test]
fn rrflow_mx_and_rrflow_kv_execute_the_identical_transaction_corpus() {
    let directory = tempfile::tempdir().unwrap();
    let rrflow_mx = StorageProfile::rrflow_mx();
    let rrflow_kv = StorageProfile::rrflow_kv(
        RrflowKvStore::open(&directory.path().join("rrflow-kv")).unwrap(),
    );

    let mx = exercise(&rrflow_mx);
    let kv = exercise(&rrflow_kv);
    assert_eq!(mx, kv);
    assert_eq!(
        mx.ordered_items,
        vec![
            (b"item:a".to_vec(), b"updated".to_vec()),
            (b"item:b".to_vec(), b"two".to_vec()),
            (b"item:d".to_vec(), b"four".to_vec()),
        ]
    );
    assert_eq!(
        mx.doctors,
        vec![
            (b"doctor:alice".to_vec(), b"off".to_vec()),
            (b"doctor:bob".to_vec(), b"off".to_vec()),
        ],
        "successful write skew records the accepted snapshot-isolation policy"
    );
    assert_eq!(mx.retry_value, Some(b"desired".to_vec()));
}

#[test]
fn only_rrflow_kv_reopens_transaction_commits() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv-reopen");
    {
        let store = RrflowKvStore::open(&path).unwrap();
        let mut transaction = store.begin_transaction().unwrap();
        transaction
            .put(b"persistent".to_vec(), b"value".to_vec())
            .unwrap();
        transaction.commit(Durability::Authoritative).unwrap();
    }

    let reopened = RrflowKvStore::open(&path).unwrap();
    let transaction = reopened.begin_transaction().unwrap();
    assert_eq!(
        transaction.get(b"persistent").unwrap(),
        Some(b"value".to_vec())
    );
    transaction.rollback().unwrap();
}

fn assert_concurrent_conflict<E>(engine: Arc<E>)
where
    E: StorageEngine + Send + Sync + 'static,
{
    let barrier = Arc::new(Barrier::new(3));
    let workers = [b"left".to_vec(), b"right".to_vec()]
        .into_iter()
        .map(|value| {
            let engine = Arc::clone(&engine);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut transaction = engine.begin_transaction().unwrap();
                transaction.put(b"contended".to_vec(), value).unwrap();
                barrier.wait();
                transaction.commit(Durability::Authoritative)
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let results = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(Error::TransactionConflict { .. })))
            .count(),
        1
    );
}

#[test]
fn concurrent_same_key_writers_conflict_on_both_profiles() {
    let directory = tempfile::tempdir().unwrap();
    assert_concurrent_conflict(Arc::new(rrd_store::RrflowMxStore::new()));
    assert_concurrent_conflict(Arc::new(
        RrflowKvStore::open(&directory.path().join("rrflow-kv-concurrent")).unwrap(),
    ));
}
