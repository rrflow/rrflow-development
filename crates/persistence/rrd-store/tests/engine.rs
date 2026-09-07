//! The storage contract proven across rrflowMX and rrflowKV.
//!
//! Both profiles must produce identical claim reads, projections, and
//! grounding stamps through the same engine boundary.

use rrd_core::{Claim, ClaimReader, Predicate, Producer, Subject};
use rrd_store::{GroundingReport, RrflowKvStore, RrflowMxStore, StorageEngine};

fn claim(subject: &str, predicate: &str, object: &str, from: u64) -> Claim {
    Claim::new(
        Subject::new(subject).unwrap(),
        Predicate::new(predicate).unwrap(),
        object,
        from,
        from,
        Producer {
            actor: "test".into(),
            on_behalf_of: None,
            session: None,
        },
    )
}

fn corpus() -> Vec<Claim> {
    vec![
        claim("wp3", "status", "planned", 100),
        claim("wp3", "status", "active", 200),
        claim("wp3", "owner", "ada", 150),
        claim("wp4", "status", "planned", 120),
        claim("wp3", "status", "done", 300),
        claim("wp4", "owner", "lin", 180),
    ]
}

#[test]
fn rrflow_kv_physical_maintenance_preserves_exact_semantics_across_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("rrflow-kv");
    let store = RrflowKvStore::open(&root).unwrap();
    StorageEngine::append_batch(&store, &corpus()).unwrap();
    let before = StorageEngine::claims_in_range(&store, 0, corpus().len() as u64).unwrap();
    store.flush(1_000).unwrap();
    store.compact(1_001, 1_001).unwrap();
    assert_eq!(
        StorageEngine::claims_in_range(&store, 0, corpus().len() as u64).unwrap(),
        before
    );
    drop(store);

    let reopened = RrflowKvStore::open(&root).unwrap();
    assert_eq!(
        StorageEngine::claims_in_range(&reopened, 0, corpus().len() as u64).unwrap(),
        before
    );
}

#[test]
fn all_engines_are_indistinguishable_through_the_port() {
    let dir = tempfile::tempdir().unwrap();
    let rrflow_kv = RrflowKvStore::open(dir.path()).unwrap();
    let memory = RrflowMxStore::new();

    for engine in [&rrflow_kv as &dyn AnyEngine, &memory as &dyn AnyEngine] {
        engine.load(&corpus());
    }

    // Same sequence, same interval replay, same subjects.
    assert_eq!(
        StorageEngine::sequence(&rrflow_kv).unwrap(),
        StorageEngine::sequence(&memory).unwrap()
    );
    assert_eq!(
        StorageEngine::claims_in_range(&rrflow_kv, 2, 5).unwrap(),
        StorageEngine::claims_in_range(&memory, 2, 5).unwrap()
    );
    assert_eq!(
        StorageEngine::subjects(&rrflow_kv).unwrap(),
        StorageEngine::subjects(&memory).unwrap()
    );

    // Same projection after rebuild, and the SAME grounding digest — the
    // stamp names content, not the engine that computed it.
    let ga = match (
        rrflow_kv.rebuild_current().unwrap(),
        rrflow_kv.ground_current(500).unwrap(),
    ) {
        (_, GroundingReport::Grounded(stamp)) => stamp,
        (_, GroundingReport::Divergence { differences }) => {
            panic!("rrflowKV diverged: {differences:?}")
        }
    };
    let gb = match (
        memory.rebuild_current().unwrap(),
        memory.ground_current(500).unwrap(),
    ) {
        (_, GroundingReport::Grounded(stamp)) => stamp,
        (_, GroundingReport::Divergence { differences }) => {
            panic!("memory diverged: {differences:?}")
        }
    };
    assert_eq!(ga.sequence, gb.sequence);
    assert_eq!(
        ga.digest, gb.digest,
        "grounding digests agree across engines"
    );

    // The quarantine semantics ride the trait too: corrupt the reference
    // engine's stored blob and the provided ground_current halts it.
    let bytes = StorageEngine::get_projection(&memory, rrd_store::CURRENT_PROJECTION)
        .unwrap()
        .unwrap();
    let corrupted = String::from_utf8(bytes)
        .unwrap()
        .replacen("\"done\"", "\"drifted\"", 1);
    StorageEngine::put_projection(&memory, rrd_store::CURRENT_PROJECTION, corrupted.as_bytes())
        .unwrap();
    assert!(matches!(
        memory.ground_current(600).unwrap(),
        GroundingReport::Divergence { .. }
    ));
    assert!(matches!(
        memory.rebuild_current(),
        Err(rrd_store::Error::Quarantined(_))
    ));
}

#[test]
fn a_rejected_batch_is_atomic_in_all_engines() {
    let dir = tempfile::tempdir().unwrap();
    let rrflow_kv = RrflowKvStore::open(dir.path()).unwrap();
    let memory = RrflowMxStore::new();
    let valid = claim("wp3", "status", "valid", 100);
    let mut invalid = claim("wp4", "status", "invalid", 200);
    invalid.valid_to = Some(200);
    assert!(StorageEngine::append_batch(&rrflow_kv, &[valid.clone(), invalid.clone()]).is_err());
    assert!(StorageEngine::append_batch(&memory, &[valid, invalid]).is_err());
    assert_eq!(StorageEngine::sequence(&rrflow_kv).unwrap(), 0);
    assert_eq!(StorageEngine::sequence(&memory).unwrap(), 0);
    assert!(StorageEngine::subjects(&rrflow_kv).unwrap().is_empty());
    assert!(StorageEngine::subjects(&memory).unwrap().is_empty());
}

#[test]
fn assert_retires_the_previous_claim_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let store = RrflowKvStore::open(dir.path()).unwrap();
    let first = claim("wp3", "status", "failing", 100);
    let mut second = claim("wp3", "status", "passing", 200);
    second.tx_time = 250;
    StorageEngine::assert(&store, &first).unwrap();
    StorageEngine::assert(&store, &second).unwrap();
    let history = store.history(&first.subject, &first.predicate).unwrap();
    let retired = history
        .iter()
        .find(|candidate| candidate.object == "failing" && candidate.valid_to == Some(200))
        .expect("retirement correction is retained in history");
    assert_eq!(retired.tx_time, 250);
    assert_eq!(
        store
            .as_of(&first.subject, &first.predicate, 150)
            .unwrap()
            .unwrap()
            .object,
        "failing"
    );
    assert_eq!(
        store
            .as_of(&first.subject, &first.predicate, 250)
            .unwrap()
            .unwrap()
            .object,
        "passing"
    );
}

/// Object-safe loading helper so both engines take the corpus through the
/// same call shape.
trait AnyEngine {
    fn load(&self, claims: &[Claim]);
}
impl<E: StorageEngine> AnyEngine for E {
    fn load(&self, claims: &[Claim]) {
        StorageEngine::append_batch(self, claims).unwrap();
    }
}
