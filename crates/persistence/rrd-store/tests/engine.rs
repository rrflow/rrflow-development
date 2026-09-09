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
    store.claims().append_batch(&corpus()).unwrap();
    let before = store
        .claims()
        .claims_in_range(0, corpus().len() as u64)
        .unwrap();
    store.flush(1_000).unwrap();
    store.compact(1_001, 1_001).unwrap();
    assert_eq!(
        store
            .claims()
            .claims_in_range(0, corpus().len() as u64)
            .unwrap(),
        before
    );
    drop(store);

    let reopened = RrflowKvStore::open(&root).unwrap();
    assert_eq!(
        reopened
            .claims()
            .claims_in_range(0, corpus().len() as u64)
            .unwrap(),
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
        rrflow_kv.claims().sequence().unwrap(),
        memory.claims().sequence().unwrap()
    );
    assert_eq!(
        rrflow_kv.claims().claims_in_range(2, 5).unwrap(),
        memory.claims().claims_in_range(2, 5).unwrap()
    );
    assert_eq!(
        rrflow_kv.claims().subjects().unwrap(),
        memory.claims().subjects().unwrap()
    );

    // Same projection after rebuild, and the SAME grounding digest — the
    // stamp names content, not the engine that computed it.
    let ga = match (
        rrflow_kv.projections().rebuild_current().unwrap(),
        rrflow_kv.projections().ground_current(500).unwrap(),
    ) {
        (_, GroundingReport::Grounded(stamp)) => stamp,
        (_, GroundingReport::Divergence { differences }) => {
            panic!("rrflowKV diverged: {differences:?}")
        }
    };
    let gb = match (
        memory.projections().rebuild_current().unwrap(),
        memory.projections().ground_current(500).unwrap(),
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
    let bytes = memory
        .projections()
        .get(rrd_store::CURRENT_PROJECTION)
        .unwrap()
        .unwrap();
    let corrupted = String::from_utf8(bytes)
        .unwrap()
        .replacen("\"done\"", "\"drifted\"", 1);
    memory
        .projections()
        .put(rrd_store::CURRENT_PROJECTION, corrupted.as_bytes())
        .unwrap();
    assert!(matches!(
        memory.projections().ground_current(600).unwrap(),
        GroundingReport::Divergence { .. }
    ));
    assert!(matches!(
        memory.projections().rebuild_current(),
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
    assert!(rrflow_kv
        .claims()
        .append_batch(&[valid.clone(), invalid.clone()])
        .is_err());
    assert!(memory.claims().append_batch(&[valid, invalid]).is_err());
    assert_eq!(rrflow_kv.claims().sequence().unwrap(), 0);
    assert_eq!(memory.claims().sequence().unwrap(), 0);
    assert!(rrflow_kv.claims().subjects().unwrap().is_empty());
    assert!(memory.claims().subjects().unwrap().is_empty());
}

#[test]
fn assert_retires_the_previous_claim_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let store = RrflowKvStore::open(dir.path()).unwrap();
    let first = claim("wp3", "status", "failing", 100);
    let mut second = claim("wp3", "status", "passing", 200);
    second.tx_time = 250;
    store.claims().assert(&first).unwrap();
    store.claims().assert(&second).unwrap();
    let history = store
        .claims()
        .history(&first.subject, &first.predicate)
        .unwrap();
    let retired = history
        .iter()
        .find(|candidate| candidate.object == "failing" && candidate.valid_to == Some(200))
        .expect("retirement correction is retained in history");
    assert_eq!(retired.tx_time, 250);
    assert_eq!(
        store
            .claims()
            .as_of(&first.subject, &first.predicate, 150)
            .unwrap()
            .unwrap()
            .object,
        "failing"
    );
    assert_eq!(
        store
            .claims()
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
        self.claims().append_batch(claims).unwrap();
    }
}
