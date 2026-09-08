//! Black-box evidence that rrflowKV persists the C-01 typed-key format.

use rrd_core::{Claim, Predicate, Producer, Subject};
use rrd_lsm::Database;
use rrd_store::{RrflowKvStore, StorageEngine};

const FORMAT: u64 = u64::from_be_bytes(*b"RRKV0001");
const GOLDEN_LAYOUT: &str = include_str!("../fixtures/rrflow-kv-store-layout-v1.hex");

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[usize::from(byte >> 4)] as char);
        encoded.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

fn claim() -> Claim {
    Claim::new(
        Subject::new("file:a").unwrap(),
        Predicate::new("imports").unwrap(),
        "file:b",
        100,
        200,
        Producer {
            actor: "codec-proof".into(),
            on_behalf_of: None,
            session: Some("c-01".into()),
        },
    )
}

#[test]
fn public_rrflow_kv_writes_frozen_typed_keys_and_reopens_them() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("rrflow-kv");
    let expected_claim = claim();

    let store = RrflowKvStore::open(&root).unwrap();
    assert_eq!(store.manifest().unwrap().application_format, Some(FORMAT));
    StorageEngine::append_batch(&store, std::slice::from_ref(&expected_claim)).unwrap();
    drop(store);

    let database = Database::open(&root).unwrap();
    assert_eq!(database.manifest().application_format, Some(FORMAT));
    let rows = database
        .scan(&[], None, database.snapshot())
        .unwrap()
        .into_iter()
        .map(|(key, _)| hex(&key))
        .collect::<Vec<_>>();
    let labels = ["claim", "sequence_1", "sequence_watermark"];
    assert_eq!(rows.len(), labels.len());
    let actual = labels
        .iter()
        .zip(rows)
        .map(|(label, key)| format!("{label} {key}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(actual, GOLDEN_LAYOUT.trim_end());
    drop(database);

    let reopened = RrflowKvStore::open(&root).unwrap();
    assert_eq!(
        StorageEngine::claims_in_range(&reopened, 0, 1).unwrap(),
        vec![expected_claim]
    );
}
