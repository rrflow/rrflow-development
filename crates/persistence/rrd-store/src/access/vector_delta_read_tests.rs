//! E-04a: one vector source's committed deltas, read at a stamp under the
//! request's budget and checked against the authenticated versions they name.

use super::*;
use crate::{RrflowKvStore, RrflowMxStore, StorageEngine};
use rrd_core::{
    RuntimeCommit, RuntimeRecord, RuntimeRetirement, RuntimeSchemaRegistry, RuntimeTableSchema,
    RuntimeVector, ScopeId, VectorCollectionAddress, VectorValue,
};

fn scope() -> ScopeId {
    ScopeId::new("project:vector-delta-reads").unwrap()
}

fn embedding(id: &str) -> RuntimeRef {
    RuntimeRef::new("embedding", id).unwrap()
}

fn address(collection: &str) -> VectorSourceAddress {
    VectorSourceAddress {
        collection: VectorCollectionAddress {
            collection_id: collection.into(),
            vector_name: "semantic".into(),
        },
        field: "body".into(),
    }
}

fn note(id: &str) -> RuntimeMutation {
    RuntimeMutation::Record {
        record: RuntimeRecord {
            reference: RuntimeRef::new("note", id).unwrap(),
            valid_from: 1,
            valid_to: None,
            properties: Default::default(),
        },
    }
}

fn vector(id: &str, collection: &str, value: f32, valid_from: u64) -> RuntimeMutation {
    RuntimeMutation::Vector {
        vector: RuntimeVector {
            reference: embedding(id),
            subject: RuntimeRef::new("note", id).unwrap(),
            collection: address(collection).collection,
            field: "body".into(),
            valid_from,
            valid_to: None,
            value: VectorValue::Dense {
                values: vec![value, 1.0 - value],
            },
            provenance: None,
            properties: Default::default(),
        },
    }
}

fn retire(id: &str, effective_at: u64) -> RuntimeMutation {
    RuntimeMutation::Retire {
        retirement: RuntimeRetirement {
            model: RuntimeLogicalModel::Vector,
            reference: embedding(id),
            effective_at,
        },
    }
}

fn commit(engine: &dyn StorageEngine, at: u64, mutations: Vec<RuntimeMutation>) -> u64 {
    let runtime = engine.runtime();
    let expected_cursor = runtime.cursor().unwrap();
    runtime
        .commit(&RuntimeCommit {
            scope: scope(),
            at,
            actor: "test:vector-delta-reads".into(),
            expected_cursor,
            mutations,
        })
        .unwrap();
    runtime.cursor().unwrap()
}

fn install(engine: &dyn StorageEngine) -> u64 {
    let mut schema = RuntimeSchemaRegistry::empty(1, "vector delta fixture");
    for (kind, model) in [
        ("note", RuntimeLogicalModel::Document),
        ("embedding", RuntimeLogicalModel::Vector),
    ] {
        schema.tables.insert(
            RuntimeType::new(kind).unwrap(),
            RuntimeTableSchema::schemaless(model),
        );
    }
    let mut mutations = vec![RuntimeMutation::Schema { registry: schema }];
    mutations.extend(["a", "b", "c", "d"].map(note));
    commit(engine, 1, mutations)
}

/// Puts, an update, a move out of and into the source, a retirement, an
/// update retired in the same commit, and unrelated vectors in another
/// collection.
fn history(engine: &dyn StorageEngine, unrelated: usize) -> Vec<u64> {
    let mut cursors = vec![install(engine)];
    cursors.push(commit(
        engine,
        2,
        vec![vector("a", "notes", 0.1, 10), vector("b", "notes", 0.2, 10)],
    ));
    for index in 0..unrelated {
        let id = format!("other-{index}");
        cursors.push(commit(
            engine,
            3,
            vec![note(&id), vector(&id, "archive", 0.5, 10)],
        ));
    }
    cursors.push(commit(engine, 4, vec![vector("a", "notes", 0.3, 20)]));
    cursors.push(commit(engine, 5, vec![vector("b", "archive", 0.4, 30)]));
    cursors.push(commit(engine, 6, vec![vector("c", "archive", 0.6, 30)]));
    cursors.push(commit(engine, 7, vec![vector("c", "notes", 0.7, 40)]));
    cursors.push(commit(engine, 8, vec![retire("a", 50)]));
    cursors.push(commit(engine, 9, vec![vector("d", "notes", 0.8, 60)]));
    cursors.push(commit(
        engine,
        10,
        vec![vector("d", "notes", 0.9, 61), retire("d", 62)],
    ));
    cursors
}

fn read_deltas(
    engine: &dyn StorageEngine,
    read: &ReadStamp,
    source: &VectorSourceAddress,
    after: u64,
    max_keys: u64,
) -> Result<RuntimeVectorDeltaRead> {
    engine
        .runtime()
        .read_vector_deltas(read, source, after, RuntimeReadBudget::new(max_keys)?)
}

/// One delta's shape: its cursor, reference, and the cursors of the versions
/// it moved between.
type Shape = (u64, RuntimeRef, Option<u64>, Option<u64>);

fn shapes(read: &RuntimeVectorDeltaRead) -> Vec<Shape> {
    let mut shapes = read
        .deltas
        .iter()
        .map(|delta| {
            (
                delta.source_cursor,
                delta.reference.clone(),
                delta.before.as_ref().map(|version| version.source_cursor),
                delta.after.as_ref().map(|version| version.source_cursor),
            )
        })
        .collect::<Vec<_>>();
    shapes.sort();
    shapes
}

/// The expected deltas, derived independently from a full exact read of
/// every vector version: per commit and identity, the head before the commit
/// and the commit's last mutation, kept where either lies in the source.
fn expected(
    engine: &dyn StorageEngine,
    read: &ReadStamp,
    source: &VectorSourceAddress,
    after: u64,
) -> Vec<Shape> {
    let full = engine
        .runtime()
        .read_versioned(
            read,
            &[RuntimeVersionedSource::Vectors { kind: None }],
            RuntimeReadBudget::new(1_000_000).unwrap(),
        )
        .unwrap();
    let in_source = |change: &&RuntimeChange| match &change.mutation {
        RuntimeMutation::Vector { vector } => {
            vector.collection == source.collection && vector.field == source.field
        }
        _ => false,
    };
    let mut by_commit = BTreeMap::<u64, Vec<RuntimeChange>>::new();
    for change in &full.changes {
        by_commit
            .entry(change.cursor - change.commit_ordinal)
            .or_default()
            .push(change.clone());
    }
    let mut heads = BTreeMap::<RuntimeRef, RuntimeChange>::new();
    let mut shapes = Vec::new();
    for changes in by_commit.into_values() {
        let mut last = BTreeMap::<RuntimeRef, RuntimeChange>::new();
        for change in changes {
            let reference = match &change.mutation {
                RuntimeMutation::Vector { vector } => vector.reference.clone(),
                RuntimeMutation::Retire { retirement } => retirement.reference.clone(),
                _ => continue,
            };
            last.insert(reference, change);
        }
        for (reference, change) in last {
            let before = heads
                .get(&reference)
                .filter(|head| matches!(head.mutation, RuntimeMutation::Vector { .. }));
            let after_head = Some(&change).filter(|head| in_source(head));
            let before_here = before.filter(in_source);
            if change.cursor > after && (before_here.is_some() || after_head.is_some()) {
                shapes.push((
                    change.cursor,
                    reference.clone(),
                    before_here.map(|head| head.cursor),
                    after_head.map(|head| head.cursor),
                ));
            }
            heads.insert(reference, change);
        }
    }
    shapes.sort();
    shapes
}

#[test]
fn stamped_deltas_equal_the_full_exact_derivation_on_both_profiles() {
    let directory = tempfile::tempdir().unwrap();
    let kv = RrflowKvStore::open(&directory.path().join("rrflow-kv")).unwrap();
    let mx = RrflowMxStore::new();
    let mut results = Vec::new();
    for engine in [&mx as &dyn StorageEngine, &kv] {
        let cursors = history(engine, 2);
        let read = engine.runtime().read_stamp(&scope()).unwrap();
        for source in [address("notes"), address("archive")] {
            for after in std::iter::once(0).chain(cursors.iter().copied()) {
                let deltas = read_deltas(engine, &read, &source, after, 100_000).unwrap();
                assert_eq!(
                    shapes(&deltas),
                    expected(engine, &read, &source, after),
                    "{source:?} after {after}"
                );
                let scans = deltas
                    .evidence
                    .paths
                    .iter()
                    .find(|path| path.path == RuntimeReadAccessPath::VectorSourceDeltas)
                    .map_or(0, |path| path.range_scans);
                assert_eq!(scans, u64::from(after < read.commit_cursor));
            }
        }
        let all = read_deltas(engine, &read, &address("notes"), 0, 100_000).unwrap();
        // The move out, the move in, and the retirement each appear; an
        // update retired in its own commit records only the retirement.
        let named = |id: &str| {
            all.deltas
                .iter()
                .filter(|delta| delta.reference == embedding(id))
                .collect::<Vec<_>>()
        };
        assert!(named("b").iter().any(|delta| delta.after.is_none()));
        assert!(named("c").iter().any(|delta| delta.before.is_none()));
        assert!(named("a").iter().any(|delta| delta.after.is_none()));
        assert_eq!(named("d").len(), 2);
        assert!(named("d").iter().any(|delta| delta.after.is_none()));
        results.push(shapes(&all));
    }
    assert_eq!(results[0], results[1], "rrflowMX and rrflowKV agree");
}

#[test]
fn an_older_stamp_excludes_later_commits_on_both_profiles_and_after_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rrflow-kv");
    let shapes_at = |engine: &dyn StorageEngine, read: &ReadStamp| {
        shapes(&read_deltas(engine, read, &address("notes"), 0, 100_000).unwrap())
    };
    let (earlier, at_earlier, current, at_current) = {
        let kv = RrflowKvStore::open(&path).unwrap();
        let mx = RrflowMxStore::new();
        let mut seen = Vec::new();
        for engine in [&mx as &dyn StorageEngine, &kv] {
            install(engine);
            commit(engine, 2, vec![vector("a", "notes", 0.1, 10)]);
            let earlier = engine.runtime().read_stamp(&scope()).unwrap();
            commit(engine, 3, vec![vector("a", "notes", 0.2, 20)]);
            commit(engine, 4, vec![vector("b", "notes", 0.3, 20)]);
            let current = engine.runtime().read_stamp(&scope()).unwrap();
            let at_earlier = shapes_at(engine, &earlier);
            assert_eq!(at_earlier.len(), 1, "only the first put is visible");
            assert!(at_earlier
                .iter()
                .all(|shape| shape.0 <= earlier.commit_cursor));
            let at_current = shapes_at(engine, &current);
            assert_eq!(at_current.len(), 3);
            assert!(
                read_deltas(
                    engine,
                    &earlier,
                    &address("notes"),
                    current.commit_cursor,
                    100
                )
                .is_err(),
                "a read cannot start after its stamp"
            );
            seen.push((earlier, at_earlier, current, at_current));
        }
        assert_eq!(
            (&seen[0].1, &seen[0].3),
            (&seen[1].1, &seen[1].3),
            "rrflowMX and rrflowKV agree"
        );
        seen.pop().unwrap()
    };
    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_eq!(shapes_at(&reopened, &earlier), at_earlier);
    assert_eq!(shapes_at(&reopened, &current), at_current);
}

#[test]
fn the_key_budget_binds_and_the_evidence_counts_it() {
    let engine = RrflowMxStore::new();
    history(&engine, 2);
    let read = engine.runtime().read_stamp(&scope()).unwrap();
    let full = read_deltas(&engine, &read, &address("notes"), 0, 100_000).unwrap();
    let examined = full.evidence.keys_examined;
    let exact = read_deltas(&engine, &read, &address("notes"), 0, examined).unwrap();
    assert_eq!(shapes(&exact), shapes(&full));
    assert!(matches!(
        read_deltas(&engine, &read, &address("notes"), 0, examined - 1),
        Err(Error::RuntimeReadBudgetExceeded { limit, .. }) if limit == examined - 1
    ));
}

#[test]
fn one_source_costs_the_same_however_many_unrelated_vectors_exist() {
    let examined = |unrelated: usize| {
        let engine = RrflowMxStore::new();
        history(&engine, unrelated);
        let read = engine.runtime().read_stamp(&scope()).unwrap();
        let deltas = read_deltas(&engine, &read, &address("notes"), 0, 1_000_000).unwrap();
        let path = deltas
            .evidence
            .paths
            .iter()
            .find(|path| path.path == RuntimeReadAccessPath::VectorSourceDeltas)
            .unwrap()
            .clone();
        (
            shapes(&deltas).len(),
            path.keys_examined,
            path.decoded_bytes,
        )
    };
    assert_eq!(examined(1), examined(40));
}

#[test]
fn a_delta_that_disagrees_with_its_version_or_goes_missing_fails_closed() {
    let engine = RrflowMxStore::new();
    history(&engine, 0);
    let read = engine.runtime().read_stamp(&scope()).unwrap();
    let source = address("notes");
    let honest = read_deltas(&engine, &read, &source, 0, 100_000).unwrap();
    let update = honest
        .deltas
        .iter()
        .find(|delta| delta.before.is_some() && delta.after.is_some())
        .expect("the update of a")
        .clone();
    let key = keyspaces::runtime_vector_source_delta_key(
        &scope(),
        &source.collection.collection_id,
        &source.collection.vector_name,
        &source.field,
        update.source_cursor,
        update.commit_ordinal,
    )
    .unwrap();
    let rewrite = |value: Option<Vec<u8>>| {
        let mut transaction = engine.begin_transaction().unwrap();
        match value {
            Some(value) => transaction.put(key.clone(), value).unwrap(),
            None => transaction.delete(key.clone()).unwrap(),
        }
        transaction
            .commit(crate::Durability::Authoritative)
            .unwrap();
    };
    let mut forged = update.clone();
    forged.after.as_mut().unwrap().value_sha256 = "0".repeat(64);
    rewrite(Some(serde_json::to_vec(&forged).unwrap()));
    assert!(matches!(
        read_deltas(&engine, &read, &source, 0, 100_000),
        Err(Error::IndexConstraint(message)) if message.contains("after version")
    ));
    rewrite(None);
    assert!(matches!(
        read_deltas(&engine, &read, &source, 0, 100_000),
        Err(Error::IndexConstraint(message)) if message.contains("is missing")
    ));
    rewrite(Some(serde_json::to_vec(&update).unwrap()));
    assert_eq!(
        shapes(&read_deltas(&engine, &read, &source, 0, 100_000).unwrap()),
        shapes(&honest)
    );
}
