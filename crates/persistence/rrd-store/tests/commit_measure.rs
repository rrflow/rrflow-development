//! A runtime commit's physical size can be measured exactly before it is
//! committed (POAM-065): measurement stages the same writes, reports what the
//! payload ceiling admits, and leaves no state behind.
use rrd_core::{Claim, Predicate, Producer, RuntimeCommit, RuntimeMutation, ScopeId, Subject};
use rrd_store::{
    ControlTransition, Error, RrflowKvStore, StorageEngine, PHYSICAL_BATCH_PAYLOAD_BYTES,
};

fn commit(scope: &ScopeId, cursor: u64, claims: usize) -> RuntimeCommit {
    RuntimeCommit {
        scope: scope.clone(),
        at: 100 + cursor,
        actor: "measure-oracle".into(),
        expected_cursor: cursor,
        mutations: (0..claims)
            .map(|index| RuntimeMutation::Claim {
                claim: Claim::new(
                    Subject::new(format!("document-{index}")).unwrap(),
                    Predicate::new("status").unwrap(),
                    "acknowledged",
                    100,
                    100,
                    Producer {
                        actor: "measure-oracle".into(),
                        on_behalf_of: None,
                        session: None,
                    },
                ),
            })
            .collect(),
    }
}

fn transitions(batch: &str, count: usize, bytes: usize) -> Vec<ControlTransition> {
    (0..count)
        .map(|index| ControlTransition {
            key: format!("server/state/measure/{batch}/{index}"),
            expected: None,
            replacement: Some(vec![b'x'; bytes]),
            at: 100,
            actor: "measure-oracle".into(),
            action: "measure.seed".into(),
            request_id: format!("seed-{index}"),
            operation_id: format!("seed-{index}"),
        })
        .collect()
}

#[test]
fn a_measured_commit_is_admitted_exactly_when_it_fits_the_payload_ceiling() {
    let directory = tempfile::tempdir().unwrap();
    let store = RrflowKvStore::create_new(&directory.path().join("measure")).unwrap();
    let scope = ScopeId::new("instance:measure").unwrap();

    let small = commit(&scope, 0, 3);
    let small_control = transitions("small", 2, 1024);
    let measured = store
        .runtime()
        .measure_commit_with_control(&small, |_, _| Ok::<_, Error>(small_control.clone()))
        .unwrap();
    assert!(measured > 2 * 1024 && measured <= PHYSICAL_BATCH_PAYLOAD_BYTES);
    // Measuring changed nothing: the cursor and the control keys are untouched.
    assert_eq!(store.runtime().read_stamp(&scope).unwrap().commit_cursor, 0);
    assert_eq!(
        store.control().get("server/state/measure/small/0").unwrap(),
        None
    );
    store
        .runtime()
        .commit_with_control_transitions(&small, &small_control)
        .unwrap();
    let cursor = store.runtime().read_stamp(&scope).unwrap().commit_cursor;
    assert!(cursor > 0);

    // Growing batches: every one that measures within the ceiling commits,
    // and every one that measures over it is refused and changes nothing.
    let mut cursor = cursor;
    let (mut admitted, mut refused) = (0, 0);
    let mut previous = 0;
    for megabytes in 1..=6 {
        let request = commit(&scope, cursor, 1);
        let control = transitions(&format!("grow-{megabytes}"), megabytes, 1024 * 1024);
        let measured = store
            .runtime()
            .measure_commit_with_control(&request, |_, _| Ok::<_, Error>(control.clone()))
            .unwrap();
        assert!(measured > previous, "a larger batch measures larger");
        previous = measured;
        let outcome = store
            .runtime()
            .commit_with_control_transitions(&request, &control);
        if measured <= PHYSICAL_BATCH_PAYLOAD_BYTES {
            outcome.unwrap();
            cursor = store.runtime().read_stamp(&scope).unwrap().commit_cursor;
            admitted += 1;
        } else {
            assert!(outcome.is_err(), "{measured} bytes were admitted");
            assert_eq!(
                store.runtime().read_stamp(&scope).unwrap().commit_cursor,
                cursor
            );
            refused += 1;
        }
    }
    assert!(
        admitted > 0 && refused > 0,
        "admitted {admitted}, refused {refused}"
    );
}

/// A read stamp taken before a catalogue change stays a valid historical
/// stamp after it: the store records the runtime cursor at which each
/// catalogue revision took effect. Before this, any catalogue change
/// invalidated every older stamp, and Parse could no longer read the
/// Inventory its job was planned from once Parse declared its indexes.
#[test]
fn a_historical_stamp_survives_a_later_catalogue_revision() {
    let directory = tempfile::tempdir().unwrap();
    let store = RrflowKvStore::create_new(&directory.path().join("catalogue")).unwrap();
    let scope = ScopeId::new("instance:catalogue-history").unwrap();
    store.runtime().commit(&commit(&scope, 0, 1)).unwrap();
    let before = store.runtime().read_stamp(&scope).unwrap();
    let (revision, _) = store
        .control()
        .commit_catalog(
            &scope,
            &ControlTransition {
                key: format!("server/state/catalogue/{scope}"),
                expected: None,
                replacement: Some(br#"{"contract":"history-fixture-v1"}"#.to_vec()),
                at: 200,
                actor: "measure-oracle".into(),
                action: "catalogue.revised".into(),
                request_id: "catalogue".into(),
                operation_id: "catalogue".into(),
            },
        )
        .unwrap();
    let revised = store.runtime().read_stamp(&scope).unwrap();
    assert_eq!(revised.catalog_revision, revision);
    assert_ne!(revised.catalog_revision, before.catalog_revision);
    store
        .runtime()
        .commit(&commit(&scope, revised.commit_cursor, 1))
        .unwrap();
    // Both stamps are now historical, each naming the revision in effect at
    // its cursor.
    store.runtime().validate_read_stamp(&before).unwrap();
    store.runtime().validate_read_stamp(&revised).unwrap();
}
