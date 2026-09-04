use rrd_contract::{ReasoningCursorAdvance, ReasoningTree};
use schemars::schema_for;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldenReasoningContract {
    tree: ReasoningTree,
    advance: ReasoningCursorAdvance,
}

#[test]
fn public_reasoning_schema_round_trips_the_kernel_golden() {
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../rrd-core/tests/fixtures/reasoning-tree-v1.json"
    ))
    .unwrap();
    let contract: GoldenReasoningContract = serde_json::from_value(expected.clone()).unwrap();
    contract.tree.validate().unwrap();
    contract.advance.validate(&contract.tree).unwrap();
    assert_eq!(serde_json::to_value(&contract).unwrap(), expected);

    let kernel: rrd_core::ReasoningTree = serde_json::from_value(expected["tree"].clone()).unwrap();
    kernel.validate().unwrap();
    assert_eq!(
        serde_json::to_value(&contract.tree).unwrap(),
        serde_json::to_value(kernel).unwrap()
    );
}

#[test]
fn public_reasoning_schema_is_closed_and_versioned() {
    let schema = serde_json::to_value(schema_for!(ReasoningTree)).unwrap();
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    assert!(schema["properties"]["contract_version"].is_object());

    let mut expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../rrd-core/tests/fixtures/reasoning-tree-v1.json"
    ))
    .unwrap();
    expected["tree"]["nodes"][0]["provider_hook"] = serde_json::json!("session-start");
    assert!(serde_json::from_value::<GoldenReasoningContract>(expected).is_err());
}

#[test]
fn public_validation_rejects_invalid_edges_and_unverifiable_advances() {
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../rrd-core/tests/fixtures/reasoning-tree-v1.json"
    ))
    .unwrap();
    let contract: GoldenReasoningContract = serde_json::from_value(expected).unwrap();

    let mut invalid_tree = contract.tree.clone();
    invalid_tree.edges[0].to = rrd_contract::CanonicalId::new("missing-node").unwrap();
    assert!(invalid_tree.validate().is_err());

    let mut invalid_advance = contract.advance;
    invalid_advance.decision.condition_evaluations.clear();
    assert!(invalid_advance.validate(&contract.tree).is_err());
}
