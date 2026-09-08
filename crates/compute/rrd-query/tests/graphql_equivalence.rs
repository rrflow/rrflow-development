use rrd_core::{
    digest, ReadStamp, RuntimeEventSchema, RuntimePropertySchema, RuntimeRecordSchema,
    RuntimeRelationSchema, RuntimeSchemaRegistry, RuntimeType, RuntimeValue, RuntimeValueType,
    ScopeId,
};
use rrd_query::{
    bind, bind_graphql, derive_graphql_schema, lower_graphql, parse, Catalog, GraphqlRequest,
    IndexCatalogueRepository, Parameters, SchemaVersion, SourceWatermarks,
};
use rrd_store::RrflowMxStore;
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
    let read = ReadStamp::new(scope.clone(), Some(1), 1, 4, Some("31".repeat(32))).unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "GraphQL equivalence schema");
    registry.records.insert(
        RuntimeType::new("document").unwrap(),
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
    );
    registry.relations.insert(
        RuntimeType::new("depends_on").unwrap(),
        RuntimeRelationSchema {
            from: BTreeSet::from([RuntimeType::new("document").unwrap()]),
            to: BTreeSet::from([RuntimeType::new("document").unwrap()]),
            properties: BTreeMap::from([(
                "strength".into(),
                RuntimePropertySchema::required(RuntimeValueType::String),
            )]),
            ..RuntimeRelationSchema::default()
        },
    );
    registry.events.insert(
        RuntimeType::new("tool_result").unwrap(),
        RuntimeEventSchema {
            properties: BTreeMap::from([(
                "ok".into(),
                RuntimePropertySchema::required(RuntimeValueType::Bool),
            )]),
            ..RuntimeEventSchema::default()
        },
    );
    let engine = RrflowMxStore::new();
    Catalog {
        read,
        schemas: vec![SchemaVersion {
            cursor: 1,
            registry,
        }],
        indexes: IndexCatalogueRepository::new(&engine, scope)
            .load()
            .unwrap(),
        source_watermarks: SourceWatermarks {
            schema: 1,
            any_record: 2,
            records: BTreeMap::from([(RuntimeType::new("document").unwrap(), 2)]),
            relations: BTreeMap::from([(RuntimeType::new("depends_on").unwrap(), 3)]),
            events: BTreeMap::from([(RuntimeType::new("tool_result").unwrap(), 4)]),
            schema_history: vec![1],
            any_record_history: vec![2],
            record_history: BTreeMap::from([(RuntimeType::new("document").unwrap(), vec![2])]),
            relation_history: BTreeMap::from([(RuntimeType::new("depends_on").unwrap(), vec![3])]),
            event_history: BTreeMap::from([(RuntimeType::new("tool_result").unwrap(), vec![4])]),
            ..SourceWatermarks::default()
        },
    }
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
