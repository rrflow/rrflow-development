use rrd_contract::{
    CanonicalId, ExecuteFunction, FunctionCapability, FunctionCatalogue, FunctionDefinition,
    FunctionLimits, FunctionRuntime, ListFunctionCatalogue, QueryValue, ReplaceFunctionCatalogue,
    TransactionFunctionBinding, TransactionFunctionEffect, TransactionMutationKind,
    FUNCTION_CONTRACT_VERSION,
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GoldenFunctionContract {
    catalogue: FunctionCatalogue,
    catalogue_sha256: String,
    replace: ReplaceFunctionCatalogue,
    list: ListFunctionCatalogue,
    execute: ExecuteFunction,
}

fn canonical_id(value: &str) -> CanonicalId {
    CanonicalId::new(value).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn golden_contract() -> GoldenFunctionContract {
    let source = "(input) => input.allowed === true";
    let function = FunctionDefinition {
        function_id: canonical_id("validate-record"),
        runtime: FunctionRuntime::JavaScriptEs2020 {
            source: source.into(),
            source_sha256: sha256(source.as_bytes()),
        },
        limits: FunctionLimits {
            max_input_bytes: 4_096,
            max_output_bytes: 4_096,
            memory_bytes: 1_048_576,
            stack_bytes: 65_536,
            max_interrupts: 1_000,
            max_fuel: 100_000,
        },
        capabilities: BTreeSet::<FunctionCapability>::new(),
    };
    let binding = TransactionFunctionBinding {
        binding_id: canonical_id("person-record-policy"),
        function_id: function.function_id.clone(),
        mutation: TransactionMutationKind::PutRecord,
        kind: Some(canonical_id("person")),
        effect: TransactionFunctionEffect::RequireTrue,
        max_attempts: 1,
    };
    let catalogue = FunctionCatalogue {
        contract_version: FUNCTION_CONTRACT_VERSION,
        revision: 1,
        functions: BTreeMap::from([(function.function_id.clone(), function)]),
        transaction_bindings: BTreeMap::from([(binding.binding_id.clone(), binding)]),
    };
    GoldenFunctionContract {
        catalogue_sha256: catalogue.sha256(),
        replace: ReplaceFunctionCatalogue {
            expected_revision: 0,
            catalogue: catalogue.clone(),
        },
        catalogue,
        list: ListFunctionCatalogue {},
        execute: ExecuteFunction {
            function_id: canonical_id("validate-record"),
            input: QueryValue::Map(BTreeMap::from([("allowed".into(), QueryValue::Bool(true))])),
        },
    }
}

#[test]
fn function_contract_matches_the_canonical_golden_fixture() {
    let actual = golden_contract();
    actual.catalogue.validate().unwrap();
    actual.replace.validate().unwrap();
    actual.execute.validate().unwrap();

    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/function-contract-v1.json")).unwrap();
    let actual_value = serde_json::to_value(&actual).unwrap();
    assert_eq!(
        actual_value,
        expected,
        "{}",
        serde_json::to_string_pretty(&actual_value).unwrap()
    );
    let reopened: GoldenFunctionContract = serde_json::from_value(expected).unwrap();
    assert_eq!(reopened, actual);
}

#[test]
fn function_contract_schema_is_closed_and_old_shapes_fail() {
    let schema = serde_json::to_value(schema_for!(GoldenFunctionContract)).unwrap();
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    let definitions = schema["$defs"].as_object().unwrap();
    for name in [
        "ExecuteFunction",
        "FunctionCatalogue",
        "FunctionDefinition",
        "FunctionLimits",
        "ListFunctionCatalogue",
        "ReplaceFunctionCatalogue",
        "TransactionFunctionBinding",
    ] {
        assert_eq!(
            definitions[name]["additionalProperties"],
            serde_json::json!(false),
            "schema {name} must reject unknown fields"
        );
    }

    let canonical = serde_json::to_value(golden_contract()).unwrap();
    let encoded = serde_json::to_string(&canonical).unwrap();
    for retired in [
        ["Automation", "Catalogue"].concat(),
        ["Function", "Trigger"].concat(),
        ["trigger", "_id"].concat(),
        ["trig", "gers"].concat(),
    ] {
        assert!(
            !encoded.contains(&retired),
            "retired name survived: {retired}"
        );
    }

    let mut old_catalogue_field = canonical.clone();
    let catalogue = old_catalogue_field["catalogue"].as_object_mut().unwrap();
    let bindings = catalogue.remove("transaction_bindings").unwrap();
    catalogue.insert(["trig", "gers"].concat(), bindings);
    assert!(serde_json::from_value::<GoldenFunctionContract>(old_catalogue_field).is_err());

    let mut old_binding_field = canonical;
    let bindings = old_binding_field["catalogue"]["transaction_bindings"]
        .as_object_mut()
        .unwrap();
    let binding = bindings
        .get_mut("person-record-policy")
        .unwrap()
        .as_object_mut()
        .unwrap();
    let binding_id = binding.remove("binding_id").unwrap();
    binding.insert(["trigger", "_id"].concat(), binding_id);
    assert!(serde_json::from_value::<GoldenFunctionContract>(old_binding_field).is_err());
}
