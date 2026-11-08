//! POAM-054: a versioned read holds one batch of stored rows at a time. What
//! it returns and what it proves must not depend on where a batch ends.

use super::*;
use crate::{RrflowKvStore, RrflowMxStore, StorageEngine};
use rrd_core::{
    RuntimeCommit, RuntimeRecord, RuntimeSchemaRegistry, RuntimeTableSchema, RuntimeVector,
    ScopeId, VectorCollectionAddress, VectorValue,
};
use std::cell::RefCell;
use std::ops::Bound::{Excluded, Included};

const BATCH: usize = VERSIONED_SCAN_BATCH_ROWS;
const POINTS: usize = 2 * BATCH + BATCH / 2;
const COMMIT_POINTS: usize = 500;
const BUDGET: u64 = 1_000_000;

struct CountingRows {
    rows: BTreeMap<Vec<u8>, Vec<u8>>,
    requests: RefCell<Vec<usize>>,
}

impl CountingRows {
    fn new(rows: BTreeMap<Vec<u8>, Vec<u8>>) -> Self {
        Self {
            rows,
            requests: RefCell::new(Vec::new()),
        }
    }
}

impl AccessRead for CountingRows {
    fn read_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.rows.get(key).cloned())
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: &[u8],
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        assert!(start < end, "storage refuses an empty range");
        self.requests.borrow_mut().push(limit);
        Ok(self
            .rows
            .range::<[u8], _>((Included(start), Excluded(end)))
            .take(limit)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect())
    }
}

/// Every batch of an even count ends on a key that is the prefix of the next
/// key, which is the only case a wrong successor could lose.
fn rows(count: usize) -> BTreeMap<Vec<u8>, Vec<u8>> {
    (0..count)
        .map(|row| {
            let mut key = format!("k{:07}", row.div_ceil(2)).into_bytes();
            if (row + 1) % 2 == 1 {
                key.push(0);
            }
            (key, row.to_be_bytes().to_vec())
        })
        .collect()
}

#[test]
fn a_scan_asks_for_one_batch_at_a_time_and_loses_no_row() {
    for count in [
        0,
        1,
        BATCH - 1,
        BATCH,
        BATCH + 1,
        2 * BATCH,
        2 * BATCH + 1,
        3 * BATCH,
    ] {
        let expected = rows(count);
        let reader = CountingRows::new(expected.clone());
        let mut seen = Vec::new();
        scan_in_batches(&reader, b"k".to_vec(), b"l", &mut |key, value| {
            seen.push((key, value));
            Ok(())
        })
        .unwrap();
        assert_eq!(
            seen,
            expected.into_iter().collect::<Vec<_>>(),
            "{count} rows"
        );
        let requests = reader.requests.into_inner();
        assert!(
            requests.iter().all(|limit| *limit == BATCH),
            "{count} rows asked for {requests:?}"
        );
        assert_eq!(requests.len(), count / BATCH + 1, "{count} rows");
    }
}

#[test]
fn a_range_ending_on_a_batch_does_not_ask_past_its_end() {
    let expected = rows(BATCH);
    let mut end = expected.keys().next_back().unwrap().clone();
    end.push(0);
    let reader = CountingRows::new(expected);
    let mut seen = 0;
    scan_in_batches(&reader, b"k".to_vec(), &end, &mut |_, _| {
        seen += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(seen, BATCH);
    assert_eq!(reader.requests.into_inner().len(), 1);
}

#[test]
fn a_row_the_caller_refuses_stops_the_scan() {
    let reader = CountingRows::new(rows(3 * BATCH));
    let mut seen = 0;
    let refused = scan_in_batches(&reader, b"k".to_vec(), b"l", &mut |_, _| {
        seen += 1;
        if seen == BATCH + 1 {
            return Err(Error::Substrate("refused".into()));
        }
        Ok(())
    });
    assert!(matches!(refused, Err(Error::Substrate(_))));
    assert_eq!(seen, BATCH + 1);
    assert_eq!(reader.requests.into_inner().len(), 2);
}

fn scope() -> ScopeId {
    ScopeId::new("project:versioned-read-batches").unwrap()
}

fn commit(engine: &dyn StorageEngine, at: u64, mutations: Vec<RuntimeMutation>) {
    let runtime = engine.runtime();
    let expected_cursor = runtime.cursor().unwrap();
    runtime
        .commit(&RuntimeCommit {
            scope: scope(),
            at,
            actor: "test:versioned-read-batches".into(),
            expected_cursor,
            mutations,
        })
        .unwrap();
}

fn corpus(engine: &dyn StorageEngine) {
    let mut schema = RuntimeSchemaRegistry::empty(1, "versioned read batches");
    for (kind, model) in [
        ("note", RuntimeLogicalModel::Document),
        ("embedding", RuntimeLogicalModel::Vector),
    ] {
        schema.tables.insert(
            RuntimeType::new(kind).unwrap(),
            RuntimeTableSchema::schemaless(model),
        );
    }
    let subject = RuntimeRef::new("note", "subject").unwrap();
    commit(
        engine,
        1,
        vec![
            RuntimeMutation::Schema { registry: schema },
            RuntimeMutation::Record {
                record: RuntimeRecord {
                    reference: subject.clone(),
                    valid_from: 1,
                    valid_to: None,
                    properties: Default::default(),
                },
            },
        ],
    );
    let points = (0..POINTS)
        .map(|point| RuntimeMutation::Vector {
            vector: RuntimeVector {
                reference: RuntimeRef::new("embedding", format!("point-{point:06}")).unwrap(),
                subject: subject.clone(),
                collection: VectorCollectionAddress {
                    collection_id: "notes".into(),
                    vector_name: "semantic".into(),
                },
                field: "body".into(),
                valid_from: 2,
                valid_to: None,
                value: VectorValue::Dense {
                    values: (0..32)
                        .map(|axis| ((point * 31 + axis) as f32 * 0.37).sin())
                        .collect(),
                },
                provenance: None,
                properties: Default::default(),
            },
        })
        .collect::<Vec<_>>();
    for (commit_index, chunk) in points.chunks(COMMIT_POINTS).enumerate() {
        commit(engine, 2 + commit_index as u64, chunk.to_vec());
    }
}

const VECTORS: [RuntimeVersionedSource; 1] = [RuntimeVersionedSource::Vectors { kind: None }];

fn read_vectors(engine: &dyn StorageEngine, budget: u64) -> Result<RuntimeVersionedRead> {
    let read = engine.runtime().read_stamp(&scope())?;
    engine
        .runtime()
        .read_versioned(&read, &VECTORS, RuntimeReadBudget::new(budget)?)
}

fn stored_vector_rows(engine: &dyn StorageEngine) -> Vec<(Vec<u8>, Vec<u8>)> {
    let (_, start, end) = source_interval(&VECTORS[0], &scope()).unwrap();
    let transaction = engine.begin_transaction().unwrap();
    transaction.scan(&start, &end, usize::MAX).unwrap()
}

/// The largest bytes one batch holds, and the bytes of the whole read.
fn stored_sizes(engine: &dyn StorageEngine) -> (usize, usize) {
    let rows = stored_vector_rows(engine);
    assert_eq!(rows.len(), POINTS);
    let size = |rows: &[(Vec<u8>, Vec<u8>)]| {
        rows.iter()
            .map(|(key, value)| key.len() + value.len())
            .sum::<usize>()
    };
    let batch = rows.chunks(BATCH).map(size).max().unwrap();
    (batch, size(&rows))
}

fn store_with_output_limit(path: &std::path::Path, limit: usize) -> RrflowKvStore {
    RrflowKvStore::open_with_options(
        path,
        rrd_lsm::DatabaseOptions {
            projected_output_bytes: limit,
            ..rrd_lsm::DatabaseOptions::default()
        },
    )
    .unwrap()
}

#[test]
fn a_read_larger_than_the_output_limit_returns_every_change() {
    let oracle = RrflowMxStore::new();
    corpus(&oracle);
    let expected = read_vectors(&oracle, BUDGET).unwrap();
    assert_eq!(expected.changes.len(), POINTS);
    let (batch, total) = stored_sizes(&oracle);
    // Room for one batch and what the scan's own row list costs, and not the read.
    let limit = batch * 2;
    assert!(total > limit, "{total} bytes fit in the limit of {limit}");

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv");
    let store = store_with_output_limit(&path, limit);
    corpus(&store);
    let mutable = read_vectors(&store, BUDGET).unwrap();
    assert_eq!(mutable.changes, expected.changes);
    assert_eq!(mutable.evidence, expected.evidence);
    assert!(mutable.evidence.range_scans > 1);

    store.flush(300).unwrap();
    drop(store);
    let reopened = store_with_output_limit(&path, limit);
    let immutable = read_vectors(&reopened, BUDGET).unwrap();
    assert_eq!(immutable.changes, expected.changes);
    assert_eq!(immutable.evidence, expected.evidence);
}

#[test]
fn a_batch_that_cannot_fit_is_still_refused() {
    let oracle = RrflowMxStore::new();
    corpus(&oracle);
    let (batch, _) = stored_sizes(&oracle);

    let directory = tempfile::tempdir().unwrap();
    let store = store_with_output_limit(&directory.path().join("rrflow-kv"), batch / 2);
    corpus(&store);
    let refused = read_vectors(&store, BUDGET).map(|_| ()).unwrap_err();
    assert!(
        matches!(
            refused,
            Error::ProjectedReadLimit {
                resource: crate::ProjectedReadResource::OutputBufferBytes,
                ..
            }
        ),
        "{refused:?}"
    );
}

/// The row a later batch reads, in a store where it can be rewritten.
fn row_in_a_later_batch(engine: &RrflowMxStore) -> (Vec<u8>, RuntimeChange) {
    corpus(engine);
    read_vectors(engine, BUDGET).unwrap();
    let (key, stored) = stored_vector_rows(engine)[BATCH + 1].clone();
    // Writing the same bytes back is not what a read objects to.
    rewrite(engine, &key, stored.clone());
    read_vectors(engine, BUDGET).unwrap();
    (key, serde_json::from_slice(&stored).unwrap())
}

fn rewrite(engine: &RrflowMxStore, key: &[u8], value: Vec<u8>) {
    let mut transaction = engine.begin_transaction().unwrap();
    transaction.put(key.to_vec(), value).unwrap();
    transaction
        .commit(crate::Durability::Authoritative)
        .unwrap();
}

fn with_changed_vector(mutation: &RuntimeMutation) -> RuntimeMutation {
    let mut changed = mutation.clone();
    let RuntimeMutation::Vector { vector } = &mut changed else {
        panic!("the row at this position is a vector");
    };
    let VectorValue::Dense { values } = &mut vector.value else {
        panic!("the corpus is dense");
    };
    values[0] += 1.0;
    changed
}

#[test]
fn a_changed_row_in_a_later_batch_fails_its_digest() {
    let engine = RrflowMxStore::new();
    let (key, mut forged) = row_in_a_later_batch(&engine);
    forged.mutation = with_changed_vector(&forged.mutation);
    rewrite(&engine, &key, serde_json::to_vec(&forged).unwrap());
    let refused = read_vectors(&engine, BUDGET).map(|_| ()).unwrap_err();
    assert!(
        refused.to_string().contains("failed digest verification"),
        "{refused:?}"
    );
}

/// Only the proof can catch this one, because the forged row's own digest is
/// right. It is what a batched read must not let through.
#[test]
fn a_changed_row_with_a_recomputed_digest_in_a_later_batch_fails_its_proof() {
    let engine = RrflowMxStore::new();
    let (key, stored) = row_in_a_later_batch(&engine);
    let commit = RuntimeCommit {
        scope: stored.scope.clone(),
        at: stored.at,
        actor: stored.actor.clone(),
        expected_cursor: 0,
        mutations: Vec::new(),
    };
    let forged = RuntimeChange::committed(
        stored.cursor,
        &commit,
        &stored.commit_id,
        stored.commit_ordinal,
        with_changed_vector(&stored.mutation),
        stored.previous_digest.clone(),
    );
    assert!(forged.verify_digest());
    assert_ne!(forged.digest, stored.digest);
    rewrite(&engine, &key, serde_json::to_vec(&forged).unwrap());
    let refused = read_vectors(&engine, BUDGET).map(|_| ()).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("inclusion proof does not match its accumulator root"),
        "{refused:?}"
    );
}

#[test]
fn the_key_budget_still_binds_across_batches() {
    let engine = RrflowMxStore::new();
    corpus(&engine);
    let limit = (BATCH + BATCH / 2) as u64;
    let refused = read_vectors(&engine, limit).map(|_| ()).unwrap_err();
    assert!(
        matches!(
            refused,
            Error::RuntimeReadBudgetExceeded { limit: bound, observed }
                if bound == limit && observed == limit + 1
        ),
        "{refused:?}"
    );
}
