//! Deterministic rrflowKV physical-storage differential against an exact model.
//!
//! This intentionally exercises raw put/update/delete semantics: the typed
//! runtime contract does not yet define referentially safe entity deletion.

use rrd_core::digest::Sha256;
use rrd_lsm::{Database, Durability, Mutation, WriteBatch};
use std::collections::BTreeMap;

const OPERATIONS: usize = 20_000;
const BATCH: usize = 250;
const KEY_CARDINALITY: u64 = 2_048;
const SEED: u64 = 0x6a09_e667_f3bc_c909;

#[test]
fn mixed_put_update_delete_reopen_compaction_matches_exact_model() {
    let root = tempfile::tempdir().unwrap();
    let rrflow_kv_path = root.path().join("rrflow-kv");
    let mut rrflow_kv = Database::create(&rrflow_kv_path).unwrap();
    let mut model = BTreeMap::<Vec<u8>, Vec<u8>>::new();
    let mut random = SEED;
    let mut puts = 0u64;
    let mut deletes = 0u64;
    let mut updates = 0u64;
    let mut reopen_count = 0u64;
    let mut compaction_count = 0u64;

    for (epoch, offset) in (0..OPERATIONS).step_by(BATCH).enumerate() {
        let mut rrflow_kv_ops = Vec::with_capacity(BATCH);
        for operation in offset..(offset + BATCH).min(OPERATIONS) {
            random = random
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let key = format!("key/{:04}", random % KEY_CARDINALITY).into_bytes();
            if (random >> 17).is_multiple_of(5) {
                model.remove(&key);
                rrflow_kv_ops.push(Mutation::Delete { key });
                deletes += 1;
            } else {
                if model.contains_key(&key) {
                    updates += 1;
                } else {
                    puts += 1;
                }
                let value = format!("value/{operation:08}/{random:016x}").into_bytes();
                model.insert(key.clone(), value.clone());
                rrflow_kv_ops.push(Mutation::Put { key, value });
            }
        }
        rrflow_kv
            .write_owned(
                WriteBatch::new(rrflow_kv_ops).unwrap(),
                Durability::Authoritative,
            )
            .unwrap();
        rrflow_kv.flush_memtable((epoch + 1) as u64).unwrap();

        if (epoch + 1) % 10 == 0 {
            rrflow_kv.compact(&[], (epoch + 1) as u64).unwrap();
            compaction_count += 1;
        }
        if (epoch + 1) % 8 == 0 {
            drop(rrflow_kv);
            rrflow_kv = Database::open(&rrflow_kv_path).unwrap();
            reopen_count += 1;
        }
        if (epoch + 1) % 5 == 0 {
            assert_matches_model(&rrflow_kv, &model);
        }
    }

    assert_matches_model(&rrflow_kv, &model);
    let digest = model_digest(&model);
    let actual = serde_json::json!({
        "contract": "rrflow-kv-exact-model-soak-v1",
        "seed": format!("0x{SEED:016x}"),
        "operations": OPERATIONS,
        "inserts": puts,
        "updates": updates,
        "deletes": deletes,
        "key_cardinality": KEY_CARDINALITY,
        "visible_keys": model.len(),
        "reopens": reopen_count,
        "compactions": compaction_count,
        "final_sha256": digest,
        "result": "matches_exact_model",
    });
    eprintln!("{actual}");
    let checked: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../docs/evidence/m4-storage-mixed-soak.json"
    ))
    .unwrap();
    for field in [
        "seed",
        "operations",
        "inserts",
        "updates",
        "deletes",
        "key_cardinality",
        "visible_keys",
        "reopens",
        "compactions",
        "final_sha256",
    ] {
        assert_eq!(
            checked[field], actual[field],
            "stale evidence field {field}"
        );
    }
}

fn assert_matches_model(rrflow_kv: &Database, model: &BTreeMap<Vec<u8>, Vec<u8>>) {
    let stored_values: BTreeMap<_, _> = rrflow_kv
        .scan(&[], None, rrflow_kv.snapshot())
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(
        &stored_values, model,
        "rrflowKV differs from the exact reference model"
    );
}

fn model_digest(model: &BTreeMap<Vec<u8>, Vec<u8>>) -> String {
    let mut digest = Sha256::new();
    for (key, value) in model {
        digest.update(&(key.len() as u64).to_be_bytes());
        digest.update(key);
        digest.update(&(value.len() as u64).to_be_bytes());
        digest.update(value);
    }
    digest.finalize_hex()
}
