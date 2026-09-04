use rrd_core::{
    ReadStamp, ReasoningActiveCursor, ReasoningCondition, ReasoningConditionEvaluation,
    ReasoningConditionPredicate, ReasoningCursorAdvance, ReasoningDecisionEvidence, ReasoningEdge,
    ReasoningEvidence, ReasoningNode, ReasoningRecipe, ReasoningRecipeSelection, ReasoningTree,
    ReasoningVerificationResult, ReasoningVerificationStatus, RuntimeId, RuntimeType, ScopeId,
    REASONING_TREE_CONTRACT_VERSION,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldenReasoningContract {
    tree: ReasoningTree,
    advance: ReasoningCursorAdvance,
}

fn runtime_id(value: &str) -> RuntimeId {
    RuntimeId::new(value).unwrap()
}

fn runtime_type(value: &str) -> RuntimeType {
    RuntimeType::new(value).unwrap()
}

fn evidence(kind: &str, source: &str, digest: char, observed_at: u64) -> ReasoningEvidence {
    ReasoningEvidence {
        kind: runtime_type(kind),
        source: source.into(),
        content_sha256: digest.to_string().repeat(64),
        observed_at,
        summary: "the observed artifact satisfied its declared check".into(),
    }
}

fn fixture() -> GoldenReasoningContract {
    let tree = ReasoningTree {
        contract_version: REASONING_TREE_CONTRACT_VERSION,
        id: runtime_id("workspace-context"),
        revision: 1,
        root: runtime_id("route-context"),
        recipes: vec![ReasoningRecipe {
            id: runtime_id("workspace-retrieval"),
            revision: 3,
            kind: runtime_type("context-retrieval"),
            instruction_sha256: "6".repeat(64),
            parameter_schema_sha256: "7".repeat(64),
            output_schema_sha256: "8".repeat(64),
        }],
        nodes: vec![
            ReasoningNode {
                id: runtime_id("route-context"),
                kind: runtime_type("recipe-router"),
                recipe_id: Some(runtime_id("workspace-retrieval")),
                terminal: false,
                verification_requirement: None,
            },
            ReasoningNode {
                id: runtime_id("context-ready"),
                kind: runtime_type("context-result"),
                recipe_id: None,
                terminal: true,
                verification_requirement: Some(runtime_id("context-verified")),
            },
        ],
        edges: vec![ReasoningEdge {
            id: runtime_id("retrieval-satisfied"),
            kind: runtime_type("branch"),
            from: runtime_id("route-context"),
            to: runtime_id("context-ready"),
            conditions: vec![ReasoningCondition {
                id: runtime_id("relevant-context-found"),
                predicate: ReasoningConditionPredicate::Snapshot {
                    evaluator: runtime_type("rrflowql"),
                    expression_sha256: "2".repeat(64),
                },
            }],
        }],
    };
    let read = ReadStamp::new(
        ScopeId::new("instance:golden").unwrap(),
        Some(3),
        2,
        7,
        Some("1".repeat(64)),
    )
    .unwrap();
    let before = ReasoningActiveCursor {
        contract_version: REASONING_TREE_CONTRACT_VERSION,
        id: runtime_id("cursor-01"),
        tree_id: tree.id.clone(),
        tree_revision: tree.revision,
        node_id: tree.root.clone(),
        step: 0,
        read: read.clone(),
    };
    let after = ReasoningActiveCursor {
        node_id: runtime_id("context-ready"),
        step: 1,
        ..before.clone()
    };
    let decision_evidence = evidence("query-result", "rrflow://golden/query/result-7", '3', 1_300);
    let verification_evidence = evidence("test-result", "cargo test -p rrd-core", '9', 1_302);
    let advance = ReasoningCursorAdvance {
        contract_version: REASONING_TREE_CONTRACT_VERSION,
        before: before.clone(),
        edge_id: runtime_id("retrieval-satisfied"),
        decision: ReasoningDecisionEvidence {
            id: runtime_id("decision-01"),
            actor_kind: runtime_type("local-model"),
            actor_id: runtime_id("lfg-router"),
            cursor_id: before.id.clone(),
            cursor_step: before.step,
            read_manifest_sha256: before.read.manifest_id.clone(),
            selected_edge: runtime_id("retrieval-satisfied"),
            recipe: Some(ReasoningRecipeSelection {
                recipe_id: runtime_id("workspace-retrieval"),
                recipe_revision: 3,
                parameters_sha256: "5".repeat(64),
            }),
            input_sha256: "4".repeat(64),
            decided_at: 1_301,
            condition_evaluations: vec![ReasoningConditionEvaluation {
                condition_id: runtime_id("relevant-context-found"),
                satisfied: true,
                evidence: vec![decision_evidence.clone()],
            }],
            evidence: vec![decision_evidence],
        },
        verifications: vec![ReasoningVerificationResult {
            id: runtime_id("verification-01"),
            requirement: runtime_id("context-verified"),
            status: ReasoningVerificationStatus::Passed,
            checked_at: 1_302,
            evidence: vec![verification_evidence],
        }],
        after,
    };
    GoldenReasoningContract { tree, advance }
}

#[test]
fn reasoning_tree_and_advance_match_the_v1_golden_contract() {
    let actual = fixture();
    actual.tree.validate().unwrap();
    actual.advance.validate(&actual.tree).unwrap();

    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/reasoning-tree-v1.json")).unwrap();
    assert_eq!(serde_json::to_value(&actual).unwrap(), expected);

    let reopened: GoldenReasoningContract = serde_json::from_value(expected).unwrap();
    reopened.tree.validate().unwrap();
    reopened.advance.validate(&reopened.tree).unwrap();
    assert_eq!(reopened, actual);
}

#[test]
fn reasoning_tree_wire_objects_reject_unknown_fields_and_versions() {
    let actual = fixture();
    let mut encoded = serde_json::to_value(&actual).unwrap();
    encoded["tree"]["nodes"][0]["implicit_lifecycle"] = serde_json::json!("fixed-stage-chain");
    assert!(serde_json::from_value::<GoldenReasoningContract>(encoded).is_err());

    let mut nested = serde_json::to_value(&actual).unwrap();
    nested["advance"]["before"]["read"]["unstamped_state"] = serde_json::json!(true);
    assert!(serde_json::from_value::<GoldenReasoningContract>(nested).is_err());

    let mut wrong_version = actual.tree;
    wrong_version.contract_version = 2;
    assert!(wrong_version.validate().is_err());
}

#[test]
fn reasoning_tree_rejects_invalid_edge_topology() {
    let actual = fixture();

    let mut unknown_target = actual.tree.clone();
    unknown_target.edges[0].to = runtime_id("missing-node");
    assert!(unknown_target.validate().is_err());

    let mut terminal_outgoing = actual.tree;
    terminal_outgoing.edges.push(ReasoningEdge {
        id: runtime_id("illegal-loop"),
        kind: runtime_type("branch"),
        from: runtime_id("context-ready"),
        to: runtime_id("route-context"),
        conditions: Vec::new(),
    });
    assert!(terminal_outgoing.validate().is_err());
}

#[test]
fn cursor_advance_rejects_missing_or_mismatched_proof() {
    let actual = fixture();

    let mut missing_condition = actual.advance.clone();
    missing_condition.decision.condition_evaluations.clear();
    assert!(missing_condition.validate(&actual.tree).is_err());

    let mut missing_evidence = actual.advance.clone();
    missing_evidence.decision.condition_evaluations[0]
        .evidence
        .clear();
    assert!(missing_evidence.validate(&actual.tree).is_err());

    let mut missing_verification = actual.advance.clone();
    missing_verification.verifications.clear();
    assert!(missing_verification.validate(&actual.tree).is_err());

    let mut wrong_edge = actual.advance.clone();
    wrong_edge.decision.selected_edge = runtime_id("some-other-edge");
    assert!(wrong_edge.validate(&actual.tree).is_err());

    let mut changed_stamp = actual.advance;
    changed_stamp.after.read = ReadStamp::new(
        ScopeId::new("instance:golden").unwrap(),
        Some(3),
        2,
        8,
        Some("a".repeat(64)),
    )
    .unwrap();
    assert!(changed_stamp.validate(&actual.tree).is_err());
}
