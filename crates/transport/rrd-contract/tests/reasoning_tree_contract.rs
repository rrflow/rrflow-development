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
        "../../../kernel/rrd-core/tests/fixtures/reasoning-tree-v1.json"
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
    assert_eq!(
        rrd_contract::REASONING_TREE_CONTRACT_VERSION,
        rrd_core::REASONING_TREE_CONTRACT_VERSION
    );
    assert_eq!(
        rrd_contract::MAX_REASONING_TREE_NODES,
        rrd_core::MAX_REASONING_TREE_NODES
    );
    assert_eq!(
        rrd_contract::MAX_REASONING_TREE_EDGES,
        rrd_core::MAX_REASONING_TREE_EDGES
    );
    assert_eq!(
        rrd_contract::MAX_REASONING_TREE_RECIPES,
        rrd_core::MAX_REASONING_TREE_RECIPES
    );
}

#[test]
fn public_reasoning_schema_is_closed_and_versioned() {
    let schema = serde_json::to_value(schema_for!(ReasoningTree)).unwrap();
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    assert!(schema["properties"]["contract_version"].is_object());

    let mut expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../kernel/rrd-core/tests/fixtures/reasoning-tree-v1.json"
    ))
    .unwrap();
    expected["tree"]["nodes"][0]["provider_hook"] = serde_json::json!("session-start");
    assert!(serde_json::from_value::<GoldenReasoningContract>(expected).is_err());
}

#[test]
fn public_validation_rejects_invalid_edges_and_unverifiable_advances() {
    let golden: serde_json::Value = serde_json::from_str(include_str!(
        "../../../kernel/rrd-core/tests/fixtures/reasoning-tree-v1.json"
    ))
    .unwrap();
    let contract: GoldenReasoningContract = serde_json::from_value(golden.clone()).unwrap();

    let mut invalid_tree_json = golden["tree"].clone();
    invalid_tree_json["edges"][0]["to"] = serde_json::json!("missing-node");
    let invalid_public_tree: ReasoningTree =
        serde_json::from_value(invalid_tree_json.clone()).unwrap();
    let invalid_kernel_tree: rrd_core::ReasoningTree =
        serde_json::from_value(invalid_tree_json).unwrap();
    assert!(invalid_public_tree.validate().is_err());
    assert!(invalid_kernel_tree.validate().is_err());

    let mut invalid_advance_json = golden["advance"].clone();
    invalid_advance_json["decision"]["condition_evaluations"] = serde_json::json!([]);
    let invalid_public_advance: ReasoningCursorAdvance =
        serde_json::from_value(invalid_advance_json.clone()).unwrap();
    let invalid_kernel_advance: rrd_core::ReasoningCursorAdvance =
        serde_json::from_value(invalid_advance_json).unwrap();
    let kernel_tree: rrd_core::ReasoningTree =
        serde_json::from_value(golden["tree"].clone()).unwrap();
    assert!(invalid_public_advance.validate(&contract.tree).is_err());
    assert!(invalid_kernel_advance.validate(&kernel_tree).is_err());
}
