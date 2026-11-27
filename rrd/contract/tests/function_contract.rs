use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use rrd_contract::{
    CanonicalId, ExecuteFunction, FunctionArtifact, FunctionArtifactMediaType, FunctionCapability,
    FunctionCatalogue, FunctionDefinition, FunctionLimits, FunctionRuntime, FunctionValueSchema,
    FunctionValueShape, ListFunctionCatalogue, QueryValue, ReplaceFunctionCatalogue,
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
    let artifact_sha256 = sha256(source.as_bytes());
    let artifact = FunctionArtifact {
        content_sha256: artifact_sha256.clone(),
        media_type: FunctionArtifactMediaType::JavaScriptUtf8,
        byte_length: source.len().try_into().unwrap(),
        content_base64: STANDARD.encode(source),
    };
    let input_schema = FunctionValueSchema {
        schema_id: canonical_id("function-input"),
        revision: 1,
        shape: FunctionValueShape::CanonicalValueV1,
    };
    let output_schema = FunctionValueSchema {
        schema_id: canonical_id("function-output"),
        revision: 1,
        shape: FunctionValueShape::Boolean,
    };
    let function = FunctionDefinition {
        function_id: canonical_id("validate-record"),
        revision: 1,
        predecessor_sha256: None,
        runtime: FunctionRuntime::JavaScriptEs2020 {
            artifact_sha256: artifact_sha256.clone(),
            runtime_profile: canonical_id("javascript-es2020-json-v1"),
            runtime_build_sha256:
                "7b565bf62da8b17fcb258102bf025eb9fc94e530a20f4d937317de81831a8474".into(),
        },
        input_schema_sha256: input_schema.sha256(),
        input_schema,
        output_schema_sha256: output_schema.sha256(),
        output_schema,
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
        revision: 1,
        predecessor_sha256: None,
        function_id: function.function_id.clone(),
        function_definition_sha256: function.sha256(),
        mutation: TransactionMutationKind::PutRecord,
        kind: Some(canonical_id("person")),
        effect: TransactionFunctionEffect::RequireTrue,
        max_attempts: 1,
    };
    let catalogue = FunctionCatalogue {
        contract_version: FUNCTION_CONTRACT_VERSION,
        revision: 1,
        artifacts: BTreeMap::from([(artifact_sha256, artifact)]),
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
        "FunctionArtifact",
        "FunctionCatalogue",
        "FunctionDefinition",
        "FunctionLimits",
        "FunctionValueSchema",
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

    let mut inline_source = serde_json::to_value(golden_contract()).unwrap();
    inline_source["catalogue"]["functions"]["validate-record"]["runtime"]["source"] =
        serde_json::json!("() => true");
    assert!(serde_json::from_value::<GoldenFunctionContract>(inline_source).is_err());

    let mut missing_artifact = golden_contract();
    missing_artifact.catalogue.artifacts.clear();
    assert!(missing_artifact.catalogue.validate().is_err());

    let mut omitted_artifacts = serde_json::to_value(golden_contract()).unwrap();
    omitted_artifacts["catalogue"]
        .as_object_mut()
        .unwrap()
        .remove("artifacts");
    assert!(serde_json::from_value::<GoldenFunctionContract>(omitted_artifacts).is_err());
}
