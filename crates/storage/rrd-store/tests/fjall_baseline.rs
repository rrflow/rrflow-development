//! The Fjall comparison engine against rrflowMX and rrflowKV.
//!
//! The baseline is only useful if it is the same storage contract with a
//! different physical engine, so every check here compares it with the other
//! two through the one port rather than against an expectation of its own.

#![cfg(feature = "fjall-baseline")]

use rrd_store::{
    Durability, Error, FjallBaselineStore, GroundingReport, RrflowKvStore, RrflowMxStore,
    StorageEngine,
};
use rrflow_kernel::{Claim, Predicate, Producer, Subject};

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

fn grounded(engine: &dyn StorageEngine) -> (u64, u64) {
    engine.projections().rebuild_current().unwrap();
    match engine.projections().ground_current(500).unwrap() {
        GroundingReport::Grounded(stamp) => (stamp.sequence, stamp.digest),
        GroundingReport::Divergence { differences } => panic!("diverged: {differences:?}"),
    }
}

#[test]
fn fjall_reads_claims_and_grounds_exactly_as_rrflow_mx_and_rrflow_kv_do() {
    let kv_dir = tempfile::tempdir().unwrap();
    let fjall_dir = tempfile::tempdir().unwrap();
    let kv = RrflowKvStore::open(kv_dir.path()).unwrap();
    let fjall = FjallBaselineStore::open(fjall_dir.path()).unwrap();
    let memory = RrflowMxStore::new();
    for engine in [&kv as &dyn StorageEngine, &fjall, &memory] {
        engine.claims().append_batch(&corpus()).unwrap();
    }

    assert_eq!(
        fjall.claims().sequence().unwrap(),
        memory.claims().sequence().unwrap()
    );
    assert_eq!(
        fjall.claims().claims_in_range(2, 5).unwrap(),
        memory.claims().claims_in_range(2, 5).unwrap()
    );
    assert_eq!(
        fjall.claims().subjects().unwrap(),
        kv.claims().subjects().unwrap()
    );
    // The stamp names content, not the engine that computed it.
    assert_eq!(grounded(&fjall), grounded(&memory));
    assert_eq!(grounded(&fjall), grounded(&kv));
}

#[test]
fn a_rejected_batch_leaves_fjall_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let fjall = FjallBaselineStore::open(dir.path()).unwrap();
    let valid = claim("wp3", "status", "valid", 100);
    let mut invalid = claim("wp4", "status", "invalid", 200);
    invalid.valid_to = Some(200);
    assert!(fjall.claims().append_batch(&[valid, invalid]).is_err());
    assert_eq!(fjall.claims().sequence().unwrap(), 0);
    assert!(fjall.claims().subjects().unwrap().is_empty());
}

/// Every observable result of the byte-transaction contract, as text, so three
/// engines can be compared line for line.
fn transaction_transcript(engine: &dyn StorageEngine) -> Vec<String> {
    let mut log = Vec::new();
    let key = |name: &str| format!("contract/{name}").into_bytes();

    // A read-only commit carries no sequence interval.
    let empty = engine.begin_transaction().unwrap();
    let read_only = empty.commit(Durability::Authoritative).unwrap();
    log.push(format!("read-only: {read_only:?}"));

    // One sequence per mutation, assigned in key order within a batch.
    let mut tx = engine.begin_transaction().unwrap();
    for name in ["b", "a", "c"] {
        tx.put(key(name), name.as_bytes().to_vec()).unwrap();
    }
    log.push(format!(
        "first commit: {:?}",
        tx.commit(Durability::Authoritative).unwrap()
    ));

    // A transaction sees its snapshot, not later commits, and its own writes.
    let early = engine.begin_transaction().unwrap();
    let mut writer = engine.begin_transaction().unwrap();
    writer.put(key("a"), b"A2".to_vec()).unwrap();
    writer.delete(key("c")).unwrap();
    writer.put(key("d"), b"d".to_vec()).unwrap();
    log.push(format!(
        "second commit: {:?}",
        writer.commit(Durability::Buffered).unwrap()
    ));
    log.push(format!("early sees a: {:?}", early.get(&key("a")).unwrap()));
    log.push(format!("early sees c: {:?}", early.get(&key("c")).unwrap()));
    log.push(format!("early sees d: {:?}", early.get(&key("d")).unwrap()));
    log.push(format!("early snapshot: {}", early.snapshot_sequence()));
    drop(early);

    let mut late = engine.begin_transaction().unwrap();
    late.put(key("z"), b"z".to_vec()).unwrap();
    late.delete(key("b")).unwrap();
    log.push(format!(
        "own writes read back: z={:?} b={:?}",
        late.get(&key("z")).unwrap(),
        late.get(&key("b")).unwrap()
    ));

    // A bounded scan merges the snapshot with the transaction's own writes and
    // honours the limit after the merge, including when a deletion hides a row.
    log.push(format!(
        "scan all: {:?}",
        late.scan(b"contract/", b"contract0", 10).unwrap()
    ));
    log.push(format!(
        "scan limit 2: {:?}",
        late.scan(b"contract/", b"contract0", 2).unwrap()
    ));
    log.push(format!(
        "scan from b: {:?}",
        late.scan(&key("b"), b"contract0", 1).unwrap()
    ));
    // Presence and length follow the same snapshot and overlay as a read:
    // `a` is stored, `b` is deleted here, `z` is written here, `nope` never was.
    for name in ["a", "b", "z", "nope"] {
        log.push(format!(
            "{name}: contains {:?}, size {:?}",
            late.contains_key(&key(name)).unwrap(),
            late.size_of(&key(name)).unwrap()
        ));
    }
    // A reverse scan is the forward scan read backward, limit applied after the merge.
    log.push(format!(
        "reverse all: {:?}",
        late.scan_reverse(b"contract/", b"contract0", 10).unwrap()
    ));
    log.push(format!(
        "reverse limit 2: {:?}",
        late.scan_reverse(b"contract/", b"contract0", 2).unwrap()
    ));
    log.push(format!(
        "reverse below d: {:?}",
        late.scan_reverse(b"contract/", &key("d"), 1).unwrap()
    ));
    log.push(format!(
        "reverse of an empty range refused: {}",
        late.scan_reverse(&key("d"), &key("d"), 1).is_err()
    ));
    // take returns the value once and stages nothing for a key that holds none.
    let staged = late.staged_encoded_bytes();
    log.push(format!("take nope: {:?}", late.take(&key("nope")).unwrap()));
    log.push(format!(
        "take of an absent key staged nothing: {}",
        late.staged_encoded_bytes() == staged
    ));
    log.push(format!("take a: {:?}", late.take(&key("a")).unwrap()));
    log.push(format!("take a again: {:?}", late.take(&key("a")).unwrap()));
    log.push(format!(
        "after take: contains {:?}, reverse {:?}",
        late.contains_key(&key("a")).unwrap(),
        late.scan_reverse(b"contract/", b"contract0", 10).unwrap()
    ));
    log.push(format!("rollback: {:?}", late.rollback().unwrap()));

    // Two transactions from one snapshot that write one key: the second loses.
    let mut first = engine.begin_transaction().unwrap();
    let mut second = engine.begin_transaction().unwrap();
    first.put(key("race"), b"first".to_vec()).unwrap();
    second.put(key("race"), b"second".to_vec()).unwrap();
    second.put(key("other"), b"other".to_vec()).unwrap();
    log.push(format!(
        "winner: {:?}",
        first.commit(Durability::Authoritative).unwrap()
    ));
    match second.commit(Durability::Authoritative) {
        Err(Error::TransactionConflict {
            snapshot_sequence,
            conflicting_sequence,
        }) => log.push(format!(
            "loser conflicts: snapshot {snapshot_sequence}, with {conflicting_sequence}"
        )),
        other => log.push(format!("loser did not conflict: {other:?}")),
    }
    // A refused commit publishes none of its writes.
    let after = engine.begin_transaction().unwrap();
    log.push(format!(
        "loser's other key: {:?}",
        after.get(&key("other")).unwrap()
    ));
    log.push(format!(
        "race value: {:?}",
        after.get(&key("race")).unwrap()
    ));
    log
}

#[test]
fn the_byte_transaction_contract_reads_identically_in_all_three_engines() {
    let kv_dir = tempfile::tempdir().unwrap();
    let fjall_dir = tempfile::tempdir().unwrap();
    let kv = RrflowKvStore::open(kv_dir.path()).unwrap();
    let fjall = FjallBaselineStore::open(fjall_dir.path()).unwrap();
    let memory = RrflowMxStore::new();

    let reference = transaction_transcript(&memory);
    assert_eq!(
        transaction_transcript(&kv),
        reference,
        "rrflowKV departs from the reference"
    );
    assert_eq!(
        transaction_transcript(&fjall),
        reference,
        "the Fjall baseline departs from the reference"
    );
    assert!(
        reference
            .iter()
            .any(|line| line.starts_with("loser conflicts")),
        "the transcript must exercise a write-write conflict: {reference:#?}"
    );
}

#[test]
fn committed_data_and_the_sequence_survive_a_reopen_of_fjall() {
    let dir = tempfile::tempdir().unwrap();
    let last = {
        let fjall = FjallBaselineStore::open(dir.path()).unwrap();
        fjall.claims().append_batch(&corpus()).unwrap();
        fjall.maintain().unwrap();
        fjall.claims().sequence().unwrap()
    };
    assert_eq!(last, corpus().len() as u64);
    let reopened = FjallBaselineStore::open(dir.path()).unwrap();
    assert_eq!(reopened.claims().sequence().unwrap(), last);
    assert_eq!(
        reopened.claims().claims_in_range(0, last).unwrap().len(),
        corpus().len()
    );
    // New writes continue the sequence rather than restarting it.
    reopened
        .claims()
        .append_batch(&[claim("wp9", "status", "new", 400)])
        .unwrap();
    assert_eq!(reopened.claims().sequence().unwrap(), last + 1);
}

/// Presence, length and reverse reads of data each engine has flushed and
/// compacted by its own means, as text, so the engines compare line for line.
fn maintained_read_transcript(engine: &dyn StorageEngine) -> Vec<String> {
    let key = |index: usize| format!("maintained/{index:04}").into_bytes();
    let read = engine.begin_transaction().unwrap();
    let mut log = Vec::new();
    for index in [0, 1, 7, 250, 499, 500] {
        log.push(format!(
            "{index}: contains {:?}, size {:?}",
            read.contains_key(&key(index)).unwrap(),
            read.size_of(&key(index)).unwrap()
        ));
    }
    for (start, end, limit) in [(0, 500, 3), (100, 110, 20), (240, 260, 5), (498, 600, 4)] {
        log.push(format!(
            "reverse [{start}, {end}) limit {limit}: {:?}",
            read.scan_reverse(&key(start), &key(end), limit).unwrap()
        ));
    }
    log
}

#[test]
fn maintained_data_reads_identically_in_rrflow_kv_and_fjall() {
    let kv_dir = tempfile::tempdir().unwrap();
    let fjall_dir = tempfile::tempdir().unwrap();
    let kv = RrflowKvStore::open(kv_dir.path()).unwrap();
    let fjall = FjallBaselineStore::open(fjall_dir.path()).unwrap();
    let memory = RrflowMxStore::new();
    let key = |index: usize| format!("maintained/{index:04}").into_bytes();

    // Five hundred keys, every seventh then deleted and every eleventh rewritten
    // longer, so the maintained data holds tombstones and several versions.
    for engine in [&kv as &dyn StorageEngine, &fjall, &memory] {
        let mut write = engine.begin_transaction().unwrap();
        for index in 0..500 {
            write
                .put(key(index), format!("value-{index}").into_bytes())
                .unwrap();
        }
        write.commit(Durability::Authoritative).unwrap();
        let mut rewrite = engine.begin_transaction().unwrap();
        for index in (0..500).step_by(7) {
            rewrite.delete(key(index)).unwrap();
        }
        for index in (1..500).step_by(11) {
            rewrite
                .put(key(index), format!("rewritten-value-{index}").into_bytes())
                .unwrap();
        }
        rewrite.commit(Durability::Authoritative).unwrap();
    }
    kv.flush(1).unwrap();
    kv.compact(1, 1).unwrap();
    fjall.maintain().unwrap();

    let reference = maintained_read_transcript(&memory);
    assert_eq!(
        maintained_read_transcript(&kv),
        reference,
        "rrflowKV departs from the reference on maintained data"
    );
    assert_eq!(
        maintained_read_transcript(&fjall),
        reference,
        "the Fjall baseline departs from the reference on maintained data"
    );
    assert!(
        reference.iter().any(|line| line.contains("contains false")),
        "the transcript must read a deleted key: {reference:#?}"
    );
}
