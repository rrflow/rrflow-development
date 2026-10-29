//! Frozen canonical identities of schema registries that use the default
//! organisation. They were written before estate organisation existed, so
//! every registry an estate already persisted keeps its digest, and with it
//! its commit identities and history proofs. Never regenerate this fixture.

use rrd_core::{
    RuntimeCommit, RuntimeEventSchema, RuntimeLogicalModel, RuntimeMutation, RuntimePropertySchema,
    RuntimeRecordSchema, RuntimeRelationSchema, RuntimeSchemaRegistry, RuntimeTableSchema,
    RuntimeType, RuntimeValueType, ScopeId,
};
use std::collections::{BTreeMap, BTreeSet};

fn kind(value: &str) -> RuntimeType {
    RuntimeType::new(value).unwrap()
}

/// One registry that exercises every logical model and specialized schema.
fn every_model() -> RuntimeSchemaRegistry {
    let mut registry = RuntimeSchemaRegistry::empty(3, "every logical model");
    registry
        .define_record_table(
            kind("person"),
            RuntimeLogicalModel::GraphNode,
            RuntimeRecordSchema {
                properties: BTreeMap::from([
                    (
                        "email".into(),
                        RuntimePropertySchema::required(RuntimeValueType::String),
                    ),
                    (
                        "age".into(),
                        RuntimePropertySchema::optional(RuntimeValueType::Unsigned),
                    ),
                ]),
                allow_additional_properties: false,
                unique_properties: BTreeSet::from(["email".to_string()]),
            },
        )
        .unwrap();
    registry
        .define_relation_table(
            kind("knows"),
            RuntimeRelationSchema {
                from: BTreeSet::from([kind("person")]),
                to: BTreeSet::from([kind("person")]),
                properties: BTreeMap::new(),
                allow_additional_properties: true,
                unique_pair: true,
                max_outgoing: Some(64),
                max_incoming: None,
            },
        )
        .unwrap();
    registry
        .define_event_table(
            kind("visit"),
            RuntimeLogicalModel::Event,
            RuntimeEventSchema {
                subject_required: true,
                subject_types: BTreeSet::from([kind("person")]),
                properties: BTreeMap::new(),
                allow_additional_properties: false,
            },
        )
        .unwrap();
    for (name, model) in [
        ("documents", RuntimeLogicalModel::Document),
        ("rows", RuntimeLogicalModel::Relational),
        ("entries", RuntimeLogicalModel::KeyValue),
        ("vectors", RuntimeLogicalModel::Vector),
        ("samples", RuntimeLogicalModel::TimeSeries),
        ("places", RuntimeLogicalModel::Geo),
        ("objects", RuntimeLogicalModel::Object),
        ("claims", RuntimeLogicalModel::ReasoningClaim),
        ("reasoning", RuntimeLogicalModel::ReasoningRecord),
        ("reasoning_events", RuntimeLogicalModel::ReasoningEvent),
        ("lifecycles", RuntimeLogicalModel::LifecycleRecord),
        ("lifecycle_events", RuntimeLogicalModel::LifecycleEvent),
    ] {
        registry
            .tables
            .insert(kind(name), RuntimeTableSchema::schemaless(model));
    }
    registry.validate().unwrap();
    registry
}

/// The engine-owned memory estate kinds, which share the `rrflow-` prefix.
fn engine_owned() -> RuntimeSchemaRegistry {
    let mut registry = RuntimeSchemaRegistry::empty(1, "memory estate bootstrap");
    registry
        .define_record_table(
            kind("rrflow-seat"),
            RuntimeLogicalModel::GraphNode,
            RuntimeRecordSchema::default(),
        )
        .unwrap();
    registry
        .define_record_table(
            kind("rrflow-provider-identity"),
            RuntimeLogicalModel::GraphNode,
            RuntimeRecordSchema::default(),
        )
        .unwrap();
    registry
        .define_relation_table(
            kind("rrflow-represents"),
            RuntimeRelationSchema {
                from: BTreeSet::from([kind("rrflow-provider-identity")]),
                to: BTreeSet::from([kind("rrflow-seat")]),
                ..RuntimeRelationSchema::default()
            },
        )
        .unwrap();
    registry.validate().unwrap();
    registry
}

fn commit(registry: RuntimeSchemaRegistry) -> RuntimeCommit {
    let commit = RuntimeCommit {
        scope: ScopeId::new("instance:golden").unwrap(),
        at: 1_100,
        actor: "agent:golden".into(),
        expected_cursor: 7,
        mutations: vec![RuntimeMutation::Schema { registry }],
    };
    commit.validate().unwrap();
    commit
}

fn contract() -> serde_json::Value {
    let every_model = every_model();
    let engine_owned = engine_owned();
    serde_json::json!({
        "comment": "default-organisation registries; written before estate organisation and never regenerated",
        "every_model": {
            "registry": every_model,
            "commit_digest": commit(every_model.clone()).digest(),
        },
        "engine_owned": {
            "registry": engine_owned,
            "commit_digest": commit(engine_owned.clone()).digest(),
        },
    })
}

#[test]
fn default_organisation_keeps_every_existing_registry_identity() {
    let actual = contract();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/schema-registry-default-digest-v1.json"
    );
    if std::env::var_os("GOLDEN_WRITE").is_some() {
        std::fs::write(
            path,
            format!("{}\n", serde_json::to_string_pretty(&actual).unwrap()),
        )
        .unwrap();
        return;
    }
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}
