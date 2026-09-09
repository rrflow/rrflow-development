//! Child process for the durability test in `tests/durability.rs`.
//!
//! Writes claims through a [`ClaimBatchWriter`], optionally flushes, then announces
//! readiness and blocks. The parent terminates it with SIGKILL, so no
//! destructor, no `Drop`, and no shutdown path can contribute to what survives.
//!
//! ```text
//! durability-child <db-path> <count> <flush|noflush|rrflow-kv-transaction|rrflow-kv-lock>
//! ```

use rrd_core::{
    Claim, DataTransaction, GeoPoint, GeoValue, Predicate, Producer, RuntimeCommit, RuntimeEvent,
    RuntimeEventSchema, RuntimeGeo, RuntimeLogicalModel, RuntimeMutation, RuntimeProperties,
    RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeRelation, RuntimeRelationSchema,
    RuntimeSchemaRegistry, RuntimeSeriesSample, RuntimeTableSchema, RuntimeType, RuntimeVector,
    ScopeId, SeriesValue, Subject, VectorValue,
};
use rrd_store::{ClaimBatchWriter, ClaimBatchWriterConfig, RrflowKvStore, StorageEngine};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("db path");
    let count: usize = args.next().expect("count").parse().expect("count");
    let mode = args
        .next()
        .expect("flush|noflush|rrflow-kv-transaction|rrflow-kv-lock");
    let path = Path::new(&path);

    match mode.as_str() {
        "rrflow-kv-transaction" => rrflow_kv_transaction(path),
        "rrflow-kv-lock" => hold_rrflow_kv_lock(path),
        "flush" | "noflush" => {}
        other => panic!("unknown durability child mode {other:?}"),
    }

    let store = Arc::new(RrflowKvStore::open(path).expect("open store"));

    // In `noflush` mode the delay must exceed the lifetime of the process, so
    // that an absent claim proves the contract rather than losing a race with
    // the interval timer.
    let flush_delay = if mode == "flush" {
        Duration::from_millis(5)
    } else {
        Duration::from_secs(3600)
    };

    let writer = ClaimBatchWriter::spawn(
        Arc::clone(&store),
        ClaimBatchWriterConfig {
            flush_delay,
            max_batch: 1024,
            queue_capacity: 8192,
        },
    );

    for i in 0..count {
        let claim = Claim::new(
            Subject::new(format!("s{i}")).unwrap(),
            Predicate::new("status").unwrap(),
            "in_progress",
            100 + i as u64,
            100 + i as u64,
            Producer {
                actor: "durability-child".into(),
                on_behalf_of: None,
                session: None,
            },
        );
        writer.submit(claim).expect("submit");
    }

    if mode == "flush" {
        writer.flush().expect("flush");
    }

    // Leak the writer so that no unwinding or destructor can flush after the
    // readiness signal. The parent kills this process.
    std::mem::forget(writer);
    ready_and_wait()
}

fn rrflow_kv_transaction(path: &Path) -> ! {
    let engine = RrflowKvStore::open(path).expect("open rrflowKV store");
    let scope = ScopeId::new("instance:rrflow-kv-durability").unwrap();
    let read = engine.runtime().read_stamp(&scope).unwrap();
    let transaction = DataTransaction::new(
        read,
        RuntimeCommit {
            scope,
            at: 100,
            actor: "agent:durability-child".into(),
            expected_cursor: 0,
            mutations: rrflow_kv_mixed_family_mutations(),
        },
    )
    .unwrap();
    let outcome = engine
        .runtime()
        .commit_data_transaction(&transaction)
        .unwrap();
    assert_eq!(outcome.count, 9);
    ready_and_wait()
}

fn rrflow_kv_mixed_family_mutations() -> Vec<RuntimeMutation> {
    let entity = RuntimeType::new("entity").unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "rrflowKV durability schema");
    registry
        .define_record_table(
            entity.clone(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema::default(),
        )
        .unwrap();
    registry
        .define_relation_table(
            RuntimeType::new("links").unwrap(),
            RuntimeRelationSchema {
                from: BTreeSet::from([entity.clone()]),
                to: BTreeSet::from([entity.clone()]),
                ..RuntimeRelationSchema::default()
            },
        )
        .unwrap();
    registry
        .define_event_table(
            RuntimeType::new("observed").unwrap(),
            RuntimeLogicalModel::Event,
            RuntimeEventSchema {
                subject_required: true,
                subject_types: BTreeSet::from([entity.clone()]),
                properties: BTreeMap::new(),
                allow_additional_properties: false,
            },
        )
        .unwrap();
    for (kind, model) in [
        ("embedding", RuntimeLogicalModel::Vector),
        ("sample", RuntimeLogicalModel::TimeSeries),
        ("location", RuntimeLogicalModel::Geo),
    ] {
        registry.tables.insert(
            RuntimeType::new(kind).unwrap(),
            RuntimeTableSchema::schemaless(model),
        );
    }
    let a = RuntimeRef::new("entity", "a").unwrap();
    let b = RuntimeRef::new("entity", "b").unwrap();
    let record = |reference| RuntimeRecord {
        reference,
        valid_from: 100,
        valid_to: None,
        properties: RuntimeProperties::new(),
    };
    vec![
        RuntimeMutation::Schema { registry },
        RuntimeMutation::Record {
            record: record(a.clone()),
        },
        RuntimeMutation::Record {
            record: record(b.clone()),
        },
        RuntimeMutation::Relation {
            relation: RuntimeRelation {
                reference: RuntimeRef::new("links", "a-b").unwrap(),
                from: a.clone(),
                to: b,
                valid_from: 100,
                valid_to: None,
                properties: RuntimeProperties::new(),
            },
        },
        RuntimeMutation::Event {
            event: RuntimeEvent {
                kind: RuntimeType::new("observed").unwrap(),
                subject: Some(a.clone()),
                properties: RuntimeProperties::new(),
            },
        },
        RuntimeMutation::Vector {
            vector: RuntimeVector {
                reference: RuntimeRef::new("embedding", "a-title").unwrap(),
                subject: a.clone(),
                collection: rrd_core::VectorCollectionAddress {
                    collection_id: "documents".into(),
                    vector_name: "title".into(),
                },
                field: "title".into(),
                valid_from: 100,
                valid_to: None,
                value: VectorValue::Dense {
                    values: vec![0.0, 0.6, 0.8],
                },
                provenance: None,
                properties: RuntimeProperties::new(),
            },
        },
        RuntimeMutation::SeriesSample {
            sample: RuntimeSeriesSample {
                reference: RuntimeRef::new("sample", "a-temperature-100").unwrap(),
                series: a.clone(),
                observed_at: 100,
                value: SeriesValue::Decimal("21.5".into()),
                properties: RuntimeProperties::new(),
            },
        },
        RuntimeMutation::Geo {
            geo: RuntimeGeo {
                reference: RuntimeRef::new("location", "a-current").unwrap(),
                subject: a,
                field: "position".into(),
                valid_from: 100,
                valid_to: None,
                value: GeoValue::Point {
                    point: GeoPoint {
                        longitude: -122.4194,
                        latitude: 37.7749,
                    },
                },
                properties: RuntimeProperties::new(),
            },
        },
        RuntimeMutation::Claim {
            claim: Claim::new(
                Subject::new("entity:a").unwrap(),
                Predicate::new("status").unwrap(),
                "durable",
                100,
                100,
                Producer {
                    actor: "agent:durability-child".into(),
                    on_behalf_of: None,
                    session: Some("rrflow-kv-sigkill".into()),
                },
            ),
        },
    ]
}

fn hold_rrflow_kv_lock(path: &Path) -> ! {
    let _engine = RrflowKvStore::open(path).expect("open rrflowKV lock owner");
    ready_and_wait()
}

fn ready_and_wait() -> ! {
    println!("READY");
    std::io::stdout().flush().expect("flush stdout");
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
