//! Child process for the durability tests in `tests/durability.rs`.
//!
//! Every mode announces readiness only after its documented durable boundary,
//! then blocks so the parent can terminate the process without destructors.

use rrd_core::{
    Claim, DataTransaction, GeoPoint, GeoValue, Predicate, Producer, RuntimeCommit, RuntimeEvent,
    RuntimeEventSchema, RuntimeGeo, RuntimeMutation, RuntimeProperties, RuntimeRecord,
    RuntimeRecordSchema, RuntimeRef, RuntimeRelation, RuntimeRelationSchema, RuntimeSchemaRegistry,
    RuntimeSeriesSample, RuntimeType, RuntimeVector, ScopeId, SeriesValue, Subject, VectorValue,
};
use rrd_store::{Engine, NativeEngine, Store, Writer, WriterConfig};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("db path");
    let count: usize = args.next().expect("count").parse().expect("count");
    let mode = args.next().expect("durability mode");

    match mode.as_str() {
        "flush" | "noflush" => run_compatibility_writer(&path, count, &mode),
        "native-transaction" => run_native_transaction(&path),
        "native-lock" => {
            let engine = NativeEngine::open(std::path::Path::new(&path)).expect("open native");
            hold_after_ready(engine)
        }
        _ => panic!("unknown durability mode {mode:?}"),
    }
}

fn run_compatibility_writer(path: &str, count: usize, mode: &str) -> ! {
    let store = Arc::new(Store::open(std::path::Path::new(path)).expect("open store"));
    let flush_delay = if mode == "flush" {
        Duration::from_millis(5)
    } else {
        Duration::from_secs(3600)
    };
    let writer = Writer::spawn(
        Arc::clone(&store),
        WriterConfig {
            flush_delay,
            max_batch: 1024,
            queue_capacity: 8192,
        },
    );
    for i in 0..count {
        writer
            .submit(Claim::new(
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
            ))
            .expect("submit");
    }
    if mode == "flush" {
        writer.flush().expect("flush");
    }
    hold_after_ready(writer)
}

fn run_native_transaction(path: &str) -> ! {
    let engine = NativeEngine::open(std::path::Path::new(path)).expect("open native");
    let scope = ScopeId::new("instance:native-durability").unwrap();
    let read = engine.runtime_read_stamp(&scope).unwrap();
    let commit = RuntimeCommit {
        scope: scope.clone(),
        at: 100,
        actor: "agent:durability-child".into(),
        expected_cursor: 0,
        mutations: native_mixed_family_mutations(),
    };
    engine
        .commit_data_transaction(&DataTransaction::new(read, commit).unwrap())
        .expect("commit native transaction");
    hold_after_ready(engine)
}

fn native_mixed_family_mutations() -> Vec<RuntimeMutation> {
    let entity = RuntimeType::new("entity").unwrap();
    let mut schema = RuntimeSchemaRegistry::empty(1, "native durability schema");
    schema
        .records
        .insert(entity.clone(), RuntimeRecordSchema::default());
    schema.relations.insert(
        RuntimeType::new("links").unwrap(),
        RuntimeRelationSchema {
            from: BTreeSet::from([entity.clone()]),
            to: BTreeSet::from([entity.clone()]),
            ..RuntimeRelationSchema::default()
        },
    );
    schema.events.insert(
        RuntimeType::new("observed").unwrap(),
        RuntimeEventSchema {
            subject_required: true,
            subject_types: BTreeSet::from([entity]),
            properties: BTreeMap::new(),
            allow_additional_properties: false,
        },
    );
    let a = RuntimeRef::new("entity", "a").unwrap();
    let b = RuntimeRef::new("entity", "b").unwrap();
    let record = |reference| RuntimeRecord {
        reference,
        valid_from: 100,
        valid_to: None,
        properties: RuntimeProperties::new(),
    };
    vec![
        RuntimeMutation::Schema { registry: schema },
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
                collection: None,
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
                    session: Some("native-sigkill".into()),
                },
            ),
        },
    ]
}

fn hold_after_ready<T>(owner: T) -> ! {
    println!("READY");
    std::io::stdout().flush().expect("flush stdout");
    std::mem::forget(owner);
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
