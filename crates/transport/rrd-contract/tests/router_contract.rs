use rrd_contract::{
    route_step_decision_sha256, route_step_request_sha256, router_backend_descriptor_sha256,
    CanonicalId, CorrelationId, DataReference, ReasoningCursorAdvance, ReasoningTree,
    RouteContextAllowance, RouteContextBudget, RouteDecisionKind, RouteParameterValue, RouteSignal,
    RouteSignalValue, RouteStepDecision, RouteStepRequest, RouterBackendDescriptor,
    RouterBackendLimits, ROUTER_CONTRACT_VERSION,
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const REQUESTED_AT: u64 = 1_800_000_100_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReasoningGolden {
    tree: ReasoningTree,
    advance: ReasoningCursorAdvance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GoldenRouterContract {
    backend: RouterBackendDescriptor,
    request: RouteStepRequest,
    decisions: Vec<RouteStepDecision>,
}

fn canonical_id(value: &str) -> CanonicalId {
    CanonicalId::new(value).unwrap()
}

fn digest(value: u64) -> String {
    format!("{value:064x}")
}

fn reasoning_golden() -> ReasoningGolden {
    serde_json::from_str(include_str!(
        "../../../kernel/rrd-core/tests/fixtures/reasoning-tree-v1.json"
    ))
    .unwrap()
}

fn backend() -> RouterBackendDescriptor {
    let mut descriptor = RouterBackendDescriptor {
        contract_version: ROUTER_CONTRACT_VERSION,
        id: canonical_id("reference-router"),
        revision: 1,
        model_manifest_id: canonical_id("reference-router-model"),
        model_manifest_revision: 1,
        model_manifest_sha256: digest(10),
        decisions: BTreeSet::from([
            RouteDecisionKind::SelectRecipe,
            RouteDecisionKind::AdvanceBranch,
            RouteDecisionKind::RequestContext,
        ]),
        limits: RouterBackendLimits {
            maximum_request_bytes: 64 * 1024,
            maximum_response_bytes: 32 * 1024,
            maximum_intent_bytes: 4 * 1024,
            maximum_signals: 16,
            maximum_recipe_candidates: 16,
            maximum_branch_candidates: 16,
            maximum_context_seeds: 16,
            maximum_parameters: 16,
            maximum_execution_ms: 5_000,
        },
        descriptor_sha256: digest(0),
    };
    descriptor.descriptor_sha256 = router_backend_descriptor_sha256(&descriptor).unwrap();
    descriptor
}

fn request(backend: &RouterBackendDescriptor) -> RouteStepRequest {
    let reasoning = reasoning_golden();
    let mut request = RouteStepRequest {
        contract_version: ROUTER_CONTRACT_VERSION,
        id: CorrelationId::new("route-01").unwrap(),
        backend_id: backend.id.clone(),
        backend_revision: backend.revision,
        backend_descriptor_sha256: backend.descriptor_sha256.clone(),
        cursor: reasoning.advance.before,
        intent: "select the next bounded workspace-context action".into(),
        signals: vec![
            RouteSignal {
                id: canonical_id("context-required"),
                value: RouteSignalValue::Bool(true),
                evidence_sha256: digest(1),
            },
            RouteSignal {
                id: canonical_id("unresolved-diagnostics"),
                value: RouteSignalValue::Unsigned(2),
                evidence_sha256: digest(2),
            },
        ],
        recipe_candidates: reasoning.tree.recipes,
        branch_candidates: reasoning.tree.edges,
        context: Some(RouteContextAllowance {
            scope: "instance:golden".into(),
            valid_at: 1_300,
            maximum_query_bytes: 4_096,
            eligible_seeds: vec![DataReference {
                kind: canonical_id("source-file"),
                id: canonical_id("src-lib-rs"),
            }],
            budget: RouteContextBudget {
                max_graph_depth: 4,
                max_items: 64,
                max_output_bytes: 131_072,
                max_storage_keys: 100_000,
            },
        }),
        allowed_decisions: backend.decisions.clone(),
        requested_at_unix_ms: REQUESTED_AT,
        deadline_unix_ms: REQUESTED_AT + backend.limits.maximum_execution_ms,
        request_sha256: digest(0),
    };
    request.request_sha256 = route_step_request_sha256(&request).unwrap();
    request
}

fn seal_decision(mut decision: RouteStepDecision) -> RouteStepDecision {
    let sha256 = route_step_decision_sha256(&decision).unwrap();
    match &mut decision {
        RouteStepDecision::SelectRecipe {
            decision_sha256, ..
        }
        | RouteStepDecision::AdvanceBranch {
            decision_sha256, ..
        }
        | RouteStepDecision::RequestContext {
            decision_sha256, ..
        } => *decision_sha256 = sha256,
    }
    decision
}

fn select_recipe(request: &RouteStepRequest) -> RouteStepDecision {
    seal_decision(RouteStepDecision::SelectRecipe {
        contract_version: ROUTER_CONTRACT_VERSION,
        request_id: request.id.clone(),
        request_sha256: request.request_sha256.clone(),
        recipe_id: canonical_id("workspace-retrieval"),
        recipe_revision: 3,
        parameters: BTreeMap::from([(
            canonical_id("query-intent"),
            RouteParameterValue::String("find relevant workspace context".into()),
        )]),
        decided_at_unix_ms: REQUESTED_AT + 1,
        decision_sha256: digest(0),
    })
}

fn advance_branch(request: &RouteStepRequest) -> RouteStepDecision {
    seal_decision(RouteStepDecision::AdvanceBranch {
        contract_version: ROUTER_CONTRACT_VERSION,
        request_id: request.id.clone(),
        request_sha256: request.request_sha256.clone(),
        edge_id: canonical_id("retrieval-satisfied"),
        decided_at_unix_ms: REQUESTED_AT + 2,
        decision_sha256: digest(0),
    })
}

fn request_context(request: &RouteStepRequest) -> RouteStepDecision {
    seal_decision(RouteStepDecision::RequestContext {
        contract_version: ROUTER_CONTRACT_VERSION,
        request_id: request.id.clone(),
        request_sha256: request.request_sha256.clone(),
        query: "find exported authentication utilities matching the active interface".into(),
        seeds: request.context.as_ref().unwrap().eligible_seeds.clone(),
        budget: RouteContextBudget {
            max_graph_depth: 2,
            max_items: 32,
            max_output_bytes: 65_536,
            max_storage_keys: 50_000,
        },
        decided_at_unix_ms: REQUESTED_AT + 3,
        decision_sha256: digest(0),
    })
}

fn golden_contract() -> GoldenRouterContract {
    let backend = backend();
    let request = request(&backend);
    let decisions = vec![
        select_recipe(&request),
        advance_branch(&request),
        request_context(&request),
    ];
    GoldenRouterContract {
        backend,
        request,
        decisions,
    }
}

#[test]
fn router_contract_matches_model_neutral_golden_vectors() {
    let fixture = golden_contract();
    fixture.backend.validate().unwrap();
    fixture.request.validate_for(&fixture.backend).unwrap();
    for decision in &fixture.decisions {
        decision
            .validate_for(&fixture.request, &fixture.backend)
            .unwrap();
    }

    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/router-contract-v1.json")).unwrap();
    let actual = serde_json::to_value(&fixture).unwrap();
    assert_eq!(
        actual,
        expected,
        "{}",
        serde_json::to_string_pretty(&actual).unwrap()
    );
    let reopened: GoldenRouterContract = serde_json::from_value(expected).unwrap();
    assert_eq!(reopened, fixture);
}

#[test]
fn generated_schema_is_closed_and_exposes_exactly_three_decisions() {
    let schema = serde_json::to_value(schema_for!(GoldenRouterContract)).unwrap();
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    let definitions = schema["$defs"].as_object().unwrap();
    for name in [
        "RouteContextAllowance",
        "RouteContextBudget",
        "RouteSignal",
        "RouteStepRequest",
        "RouterBackendDescriptor",
        "RouterBackendLimits",
    ] {
        assert!(
            definitions.contains_key(name),
            "missing generated schema {name}"
        );
        assert_eq!(
            definitions[name]["additionalProperties"],
            serde_json::json!(false),
            "schema {name} must reject unknown fields"
        );
    }
    for name in [
        "RouteParameterValue",
        "RouteSignalValue",
        "RouteStepDecision",
    ] {
        assert!(definitions.contains_key(name));
        let options = definitions[name]["oneOf"].as_array().unwrap();
        assert!(
            options
                .iter()
                .all(|option| option["additionalProperties"] == serde_json::json!(false)),
            "enum schema {name} must reject unknown fields"
        );
    }
    let decision_options = definitions["RouteStepDecision"]["oneOf"]
        .as_array()
        .unwrap();
    assert_eq!(decision_options.len(), 3);
    let decision_schema = serde_json::to_string(&definitions["RouteStepDecision"]).unwrap();
    for decision in ["select_recipe", "advance_branch", "request_context"] {
        assert!(decision_schema.contains(decision));
    }

    let mut descriptor = serde_json::to_value(backend()).unwrap();
    descriptor["provider"] = serde_json::json!("openai");
    assert!(serde_json::from_value::<RouterBackendDescriptor>(descriptor).is_err());

    let fixture_json = serde_json::to_string(&golden_contract()).unwrap();
    assert!(!fixture_json.contains("provider"));
    assert!(!fixture_json.contains("model_sha256"));
    assert!(!fixture_json.contains("physical_index"));
    assert!(!fixture_json.contains("storage_backend"));
    assert!(!fixture_json.contains("keyspace"));
}

#[test]
fn requests_are_bounded_stamped_and_candidate_closed() {
    let backend = backend();
    let request = request(&backend);

    let mut oversized = request.clone();
    oversized.intent = "x".repeat(backend.limits.maximum_intent_bytes as usize + 1);
    oversized.request_sha256 = route_step_request_sha256(&oversized).unwrap();
    assert!(oversized.validate_for(&backend).is_err());

    let mut stale_digest = request.clone();
    stale_digest.intent.push_str(" with changed content");
    assert!(stale_digest.validate_for(&backend).is_err());

    let mut unsorted = request.clone();
    unsorted.signals.reverse();
    unsorted.request_sha256 = route_step_request_sha256(&unsorted).unwrap();
    assert!(unsorted.validate_for(&backend).is_err());

    let mut inconsistent = request.clone();
    inconsistent
        .allowed_decisions
        .remove(&RouteDecisionKind::SelectRecipe);
    inconsistent.request_sha256 = route_step_request_sha256(&inconsistent).unwrap();
    assert!(inconsistent.validate_for(&backend).is_err());

    let mut wrong_scope = request.clone();
    wrong_scope.context.as_mut().unwrap().scope = "instance:other".into();
    wrong_scope.request_sha256 = route_step_request_sha256(&wrong_scope).unwrap();
    assert!(wrong_scope.validate_for(&backend).is_err());

    let mut excessive_limits = backend.clone();
    excessive_limits.limits.maximum_execution_ms += 60_000;
    excessive_limits.descriptor_sha256 =
        router_backend_descriptor_sha256(&excessive_limits).unwrap();
    assert!(excessive_limits.validate().is_err());

    let mut tiny_response_backend = backend.clone();
    tiny_response_backend.limits.maximum_response_bytes = 1;
    tiny_response_backend.descriptor_sha256 =
        router_backend_descriptor_sha256(&tiny_response_backend).unwrap();
    let tiny_response_request = self::request(&tiny_response_backend);
    assert!(select_recipe(&tiny_response_request)
        .validate_for(&tiny_response_request, &tiny_response_backend)
        .is_err());
}

#[test]
fn decisions_cannot_escape_candidates_budgets_or_request_binding() {
    let backend = backend();
    let request = request(&backend);

    let mut unknown_recipe = select_recipe(&request);
    if let RouteStepDecision::SelectRecipe { recipe_id, .. } = &mut unknown_recipe {
        *recipe_id = canonical_id("unlisted-recipe");
    }
    unknown_recipe = seal_decision(unknown_recipe);
    assert!(unknown_recipe.validate_for(&request, &backend).is_err());

    let mut unknown_edge = advance_branch(&request);
    if let RouteStepDecision::AdvanceBranch { edge_id, .. } = &mut unknown_edge {
        *edge_id = canonical_id("unlisted-edge");
    }
    unknown_edge = seal_decision(unknown_edge);
    assert!(unknown_edge.validate_for(&request, &backend).is_err());

    let mut excessive_context = request_context(&request);
    if let RouteStepDecision::RequestContext { budget, .. } = &mut excessive_context {
        budget.max_items = request.context.as_ref().unwrap().budget.max_items + 1;
    }
    excessive_context = seal_decision(excessive_context);
    assert!(excessive_context.validate_for(&request, &backend).is_err());

    let mut foreign_seed = request_context(&request);
    if let RouteStepDecision::RequestContext { seeds, .. } = &mut foreign_seed {
        *seeds = vec![DataReference {
            kind: canonical_id("source-file"),
            id: canonical_id("not-offered"),
        }];
    }
    foreign_seed = seal_decision(foreign_seed);
    assert!(foreign_seed.validate_for(&request, &backend).is_err());

    let mut rebound = advance_branch(&request);
    if let RouteStepDecision::AdvanceBranch { request_sha256, .. } = &mut rebound {
        *request_sha256 = digest(999);
    }
    rebound = seal_decision(rebound);
    assert!(rebound.validate_for(&request, &backend).is_err());

    let mut late = advance_branch(&request);
    if let RouteStepDecision::AdvanceBranch {
        decided_at_unix_ms, ..
    } = &mut late
    {
        *decided_at_unix_ms = request.deadline_unix_ms + 1;
    }
    late = seal_decision(late);
    assert!(late.validate_for(&request, &backend).is_err());
}

#[test]
fn nested_recipe_parameters_fail_closed_at_the_depth_bound() {
    let backend = backend();
    let request = request(&backend);
    let mut value = RouteParameterValue::String("leaf".into());
    for _ in 0..17 {
        value = RouteParameterValue::List(vec![value]);
    }
    let mut decision = select_recipe(&request);
    if let RouteStepDecision::SelectRecipe { parameters, .. } = &mut decision {
        *parameters = BTreeMap::from([(canonical_id("nested"), value)]);
    }
    decision = seal_decision(decision);
    assert!(decision.validate_for(&request, &backend).is_err());
}

#[test]
fn unknown_decision_fields_are_rejected_before_validation() {
    let fixture = golden_contract();
    let mut decision = serde_json::to_value(&fixture.decisions[2]).unwrap();
    decision["vector_index"] = serde_json::json!("workspace-ann");
    assert!(serde_json::from_value::<RouteStepDecision>(decision).is_err());

    let mut parameter_decision = serde_json::to_value(&fixture.decisions[0]).unwrap();
    parameter_decision["parameters"]["query-intent"]["provider_payload"] = serde_json::json!(true);
    assert!(serde_json::from_value::<RouteStepDecision>(parameter_decision).is_err());
}
