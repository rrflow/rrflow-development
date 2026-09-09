use rrd_core::{
    digest, Predicate, RuntimeCommit, RuntimeEvent, RuntimeEventSchema, RuntimeLogicalModel,
    RuntimeMutation, RuntimeProperties, RuntimePropertySchema, RuntimeRecord, RuntimeRecordSchema,
    RuntimeRef, RuntimeRelation, RuntimeRelationSchema, RuntimeSchemaRegistry, RuntimeType,
    RuntimeValue, RuntimeValueType, ScopeId,
};
use rrd_query::{
    bind, bind_graphql, derive_graphql_schema, lower_graphql, parse, Catalog, ExecutionBudget,
    GraphqlRequest, Parameters, Source, TraversalDirection,
};
use rrd_store::{RrflowMxStore, RuntimeReadBudget, StorageEngine};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EquivalenceFixture {
    contract_version: u16,
    cases: Vec<EquivalenceCase>,
    denials: Vec<DenialCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EquivalenceCase {
    name: String,
    graphql: GraphqlRequest,
    rrflowql: String,
    expected_schema_digest: String,
    expected_bound_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DenialCase {
    name: String,
    graphql: GraphqlRequest,
    error_contains: String,
}

fn fixture_path() -> String {
    format!(
        "{}/../../transport/rrd-contract/fixtures/graphql-equivalence-v1.json",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn catalog() -> Catalog {
    let scope = ScopeId::new("instance:graphql-equivalence").unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "GraphQL equivalence schema");
    let document_kind = RuntimeType::new("document").unwrap();
    registry
        .define_record_table(
            document_kind.clone(),
            RuntimeLogicalModel::Relational,
            RuntimeRecordSchema {
                allow_additional_properties: false,
                properties: BTreeMap::from([
                    (
                        "status".into(),
                        RuntimePropertySchema::required(RuntimeValueType::String),
                    ),
                    (
                        "title".into(),
                        RuntimePropertySchema::required(RuntimeValueType::String),
                    ),
                ]),
                ..RuntimeRecordSchema::default()
            },
        )
        .unwrap();
    registry
        .define_relation_table(
            RuntimeType::new("depends_on").unwrap(),
            RuntimeRelationSchema {
                from: BTreeSet::from([document_kind.clone()]),
                to: BTreeSet::from([document_kind]),
                properties: BTreeMap::from([(
                    "strength".into(),
                    RuntimePropertySchema::required(RuntimeValueType::String),
                )]),
                ..RuntimeRelationSchema::default()
            },
        )
        .unwrap();
    registry
        .define_event_table(
            RuntimeType::new("tool_result").unwrap(),
            RuntimeLogicalModel::Event,
            RuntimeEventSchema {
                properties: BTreeMap::from([(
                    "ok".into(),
                    RuntimePropertySchema::required(RuntimeValueType::Bool),
                )]),
                ..RuntimeEventSchema::default()
            },
        )
        .unwrap();
    let engine = RrflowMxStore::new();
    let document = RuntimeRef::new("document", "a").unwrap();
    engine
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope.clone(),
            at: 100,
            actor: "test:graphql-equivalence".into(),
            expected_cursor: 0,
            mutations: vec![
                RuntimeMutation::Schema { registry },
                RuntimeMutation::Record {
                    record: RuntimeRecord {
                        reference: document.clone(),
                        valid_from: 1,
                        valid_to: None,
                        properties: RuntimeProperties::from([
                            ("status".into(), RuntimeValue::String("open".into())),
                            ("title".into(), RuntimeValue::String("Alpha".into())),
                        ]),
                    },
                },
                RuntimeMutation::Relation {
                    relation: RuntimeRelation {
                        reference: RuntimeRef::new("depends_on", "a-a").unwrap(),
                        from: document.clone(),
                        to: document.clone(),
                        valid_from: 1,
                        valid_to: None,
                        properties: RuntimeProperties::from([(
                            "strength".into(),
                            RuntimeValue::String("hard".into()),
                        )]),
                    },
                },
                RuntimeMutation::Event {
                    event: RuntimeEvent {
                        kind: RuntimeType::new("tool_result").unwrap(),
                        subject: Some(document),
                        properties: RuntimeProperties::from([(
                            "ok".into(),
                            RuntimeValue::Bool(true),
                        )]),
                    },
                },
            ],
        })
        .unwrap();
    Catalog::capture_for_sources(
        &engine,
        &scope,
        &[
            Source::Record {
                kind: RuntimeType::new("document").unwrap(),
            },
            Source::Relation {
                kind: RuntimeType::new("depends_on").unwrap(),
            },
            Source::Event {
                kind: RuntimeType::new("tool_result").unwrap(),
            },
            Source::Traversal {
                relation: RuntimeType::new("depends_on").unwrap(),
                start: RuntimeRef::new("document", "a").unwrap(),
                direction: TraversalDirection::Outgoing,
                max_depth: 4,
            },
            Source::Claim {
                predicate: Some(Predicate::new("status").unwrap()),
            },
        ],
        RuntimeReadBudget::new(ExecutionBudget::default().max_storage_keys).unwrap(),
    )
    .unwrap()
}

fn fixture() -> EquivalenceFixture {
    serde_json::from_str(
        &std::fs::read_to_string(fixture_path()).expect("GraphQL equivalence fixture must exist"),
    )
    .expect("GraphQL equivalence fixture must be valid")
}

fn bound_digest(bound: &rrd_query::BoundQuery) -> String {
    digest::sha256_hex(&serde_json::to_vec(bound).unwrap())
}

#[test]
fn graphql_and_rrflowql_bind_to_identical_queries() {
    let catalog = catalog();
    let mut fixture = fixture();
    assert_eq!(fixture.contract_version, 1);
    for case in &mut fixture.cases {
        let lowered = lower_graphql(&case.graphql, &catalog)
            .unwrap_or_else(|error| panic!("{} GraphQL lowering failed: {error}", case.name));
        let graphql = bind_graphql(&case.graphql, &catalog)
            .unwrap_or_else(|error| panic!("{} GraphQL binding failed: {error}", case.name));
        let rrflowql = parse(&case.rrflowql)
            .and_then(|query| {
                bind(&query, &lowered.parameters, &catalog).map_err(|error| rrd_query::ParseError {
                    offset: 0,
                    message: error.to_string(),
                })
            })
            .unwrap_or_else(|error| panic!("{} rrflowQL binding failed: {error}", case.name));
        assert_eq!(graphql, rrflowql, "{} bound-query mismatch", case.name);
        if std::env::var_os("RRFLOW_UPDATE_GOLDENS").is_some() {
            case.expected_schema_digest = lowered.schema_digest;
            case.expected_bound_digest = bound_digest(&graphql);
        } else {
            assert_eq!(
                lowered.schema_digest, case.expected_schema_digest,
                "{} schema digest mismatch",
                case.name
            );
            assert_eq!(
                bound_digest(&graphql),
                case.expected_bound_digest,
                "{} bound digest mismatch",
                case.name
            );
        }
    }
    if std::env::var_os("RRFLOW_UPDATE_GOLDENS").is_some() {
        std::fs::write(
            fixture_path(),
            format!("{}\n", serde_json::to_string_pretty(&fixture).unwrap()),
        )
        .unwrap();
    }
}

#[test]
fn graphql_denials_fail_before_any_executor_exists() {
    let catalog = catalog();
    for denial in fixture().denials {
        let error = bind_graphql(&denial.graphql, &catalog).expect_err(&denial.name);
        assert!(
            error.to_string().contains(&denial.error_contains),
            "{}: expected {:?}, got {error}",
            denial.name,
            denial.error_contains
        );
    }
}

#[test]
fn schema_is_historical_digest_bound_and_rejects_unrepresentable_fields() {
    let catalog = catalog();
    let schema = derive_graphql_schema(&catalog, 4).unwrap();
    schema.validate().unwrap();
    assert_eq!(schema.read, catalog.read);
    assert_eq!(schema.schema_revision, 1);
    assert!(schema.sources.iter().any(|source| {
        source.kind.as_deref() == Some("document")
            && source.fields.iter().any(|field| field.name == "status")
    }));

    let mut invalid = catalog;
    invalid.schemas[0]
        .registry
        .records
        .get_mut(&RuntimeType::new("document").unwrap())
        .unwrap()
        .properties
        .insert(
            "not-a-graphql-name".into(),
            RuntimePropertySchema::required(RuntimeValueType::String),
        );
    let error = derive_graphql_schema(&invalid, 4).unwrap_err();
    assert!(error.to_string().contains("not a permitted GraphQL name"));
}

#[test]
fn variable_defaults_are_typed_and_do_not_create_another_parameter_model() {
    let catalog = catalog();
    let request = GraphqlRequest {
        contract_version: 1,
        document: r#"
            query Defaulted($when: Unsigned! = 100, $status: String! = "open") {
              record(
                kind: "document"
                validAt: $when
                knownAt: HEAD
                where: [{field: "status", comparison: EQ, value: $status}]
              ) { id status }
            }
        "#
        .into(),
        operation_name: Some("Defaulted".into()),
        variables: Parameters::new(),
    };
    let lowered = lower_graphql(&request, &catalog).unwrap();
    assert_eq!(
        lowered.parameters.get("when"),
        Some(&RuntimeValue::Unsigned(100))
    );
    assert_eq!(
        lowered.parameters.get("status"),
        Some(&RuntimeValue::String("open".into()))
    );
    bind_graphql(&request, &catalog).unwrap();
}
