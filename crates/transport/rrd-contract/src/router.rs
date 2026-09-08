//! Model- and provider-neutral reasoning-tree routing contracts.
//!
//! A router proposes one bounded decision. It does not authorize work,
//! evaluate deterministic edge conditions, select physical query paths, or
//! mutate the active cursor. `RrdEngine` validates and applies any proposal.

use crate::{
    invalid, sha256_bytes, validate_sha256, AssembleContext, CanonicalId, CorrelationId,
    DataReference, ReasoningActiveCursor, ReasoningEdge, ReasoningRecipe, Result,
    MAX_CONTEXT_GRAPH_DEPTH, MAX_CONTEXT_ITEMS, MAX_CONTEXT_OUTPUT_BYTES, MAX_CONTEXT_QUERY_BYTES,
    MAX_CONTEXT_SCANNED_CHANGES, MAX_CONTEXT_SEEDS, MAX_REASONING_EDGE_CONDITIONS,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const ROUTER_CONTRACT_VERSION: u16 = 1;
pub const MAX_ROUTE_REQUEST_BYTES: u64 = 512 * 1024;
pub const MAX_ROUTE_RESPONSE_BYTES: u64 = 128 * 1024;
pub const MAX_ROUTE_INTENT_BYTES: u64 = 32 * 1024;
pub const MAX_ROUTE_SIGNAL_BYTES: usize = 4 * 1024;
pub const MAX_ROUTE_SIGNALS: u32 = 256;
pub const MAX_ROUTE_RECIPE_CANDIDATES: u32 = 256;
pub const MAX_ROUTE_BRANCH_CANDIDATES: u32 = 256;
pub const MAX_ROUTE_PARAMETERS: u32 = 128;
pub const MAX_ROUTE_PARAMETER_DEPTH: usize = 16;
pub const MAX_ROUTE_PARAMETER_ITEMS: usize = 1_024;
pub const MAX_ROUTE_EXECUTION_MS: u64 = 60_000;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RouteDecisionKind {
    SelectRecipe,
    AdvanceBranch,
    RequestContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterBackendLimits {
    pub maximum_request_bytes: u64,
    pub maximum_response_bytes: u64,
    pub maximum_intent_bytes: u64,
    pub maximum_signals: u32,
    pub maximum_recipe_candidates: u32,
    pub maximum_branch_candidates: u32,
    pub maximum_context_seeds: u32,
    pub maximum_parameters: u32,
    pub maximum_execution_ms: u64,
}

impl RouterBackendLimits {
    pub fn validate(&self) -> Result<()> {
        validate_limit(
            self.maximum_request_bytes,
            MAX_ROUTE_REQUEST_BYTES,
            "router maximum_request_bytes",
        )?;
        validate_limit(
            self.maximum_response_bytes,
            MAX_ROUTE_RESPONSE_BYTES,
            "router maximum_response_bytes",
        )?;
        validate_limit(
            self.maximum_intent_bytes,
            MAX_ROUTE_INTENT_BYTES,
            "router maximum_intent_bytes",
        )?;
        validate_limit(
            u64::from(self.maximum_signals),
            u64::from(MAX_ROUTE_SIGNALS),
            "router maximum_signals",
        )?;
        validate_limit(
            u64::from(self.maximum_recipe_candidates),
            u64::from(MAX_ROUTE_RECIPE_CANDIDATES),
            "router maximum_recipe_candidates",
        )?;
        validate_limit(
            u64::from(self.maximum_branch_candidates),
            u64::from(MAX_ROUTE_BRANCH_CANDIDATES),
            "router maximum_branch_candidates",
        )?;
        validate_limit(
            u64::from(self.maximum_context_seeds),
            MAX_CONTEXT_SEEDS as u64,
            "router maximum_context_seeds",
        )?;
        validate_limit(
            u64::from(self.maximum_parameters),
            u64::from(MAX_ROUTE_PARAMETERS),
            "router maximum_parameters",
        )?;
        validate_limit(
            self.maximum_execution_ms,
            MAX_ROUTE_EXECUTION_MS,
            "router maximum_execution_ms",
        )?;
        if self.maximum_intent_bytes > self.maximum_request_bytes {
            return invalid("router intent byte limit exceeds its request byte limit");
        }
        Ok(())
    }
}

/// Capabilities and hard dispatch limits for one replaceable router backend.
/// The descriptor binds one exact model manifest; B-03 validates that
/// manifest's artifacts and runtime separately before any backend is loaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterBackendDescriptor {
    pub contract_version: u16,
    pub id: CanonicalId,
    pub revision: u64,
    pub model_manifest_id: CanonicalId,
    pub model_manifest_revision: u64,
    pub model_manifest_sha256: String,
    pub decisions: BTreeSet<RouteDecisionKind>,
    pub limits: RouterBackendLimits,
    pub descriptor_sha256: String,
}

impl RouterBackendDescriptor {
    pub fn validate(&self) -> Result<()> {
        validate_contract_version(self.contract_version)?;
        if self.revision == 0 {
            return invalid("router backend revision must be greater than zero");
        }
        if self.model_manifest_revision == 0 {
            return invalid("router backend model manifest revision must be greater than zero");
        }
        validate_sha256(
            &self.model_manifest_sha256,
            "router backend model_manifest_sha256",
        )?;
        if self.decisions.is_empty() {
            return invalid("router backend must support at least one routing decision");
        }
        self.limits.validate()?;
        validate_sha256(&self.descriptor_sha256, "router backend descriptor_sha256")?;
        if self.descriptor_sha256 != router_backend_descriptor_sha256(self)? {
            return invalid("router backend descriptor digest does not match its content");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum RouteSignalValue {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Decimal(String),
    String(String),
    Digest(String),
}

/// Strict JSON-like recipe parameters. This is separate from rrflowQL values:
/// a routing model cannot smuggle query or storage semantics into a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum RouteParameterValue {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Decimal(String),
    String(String),
    Digest(String),
    List(Vec<RouteParameterValue>),
    Map(BTreeMap<CanonicalId, RouteParameterValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouteSignal {
    pub id: CanonicalId,
    pub value: RouteSignalValue,
    pub evidence_sha256: String,
}

impl RouteSignal {
    fn validate(&self) -> Result<()> {
        match &self.value {
            RouteSignalValue::Decimal(value) => {
                validate_bounded_text(value, "route decimal signal", MAX_ROUTE_SIGNAL_BYTES)?;
            }
            RouteSignalValue::String(value) => {
                validate_bounded_text(value, "route string signal", MAX_ROUTE_SIGNAL_BYTES)?;
            }
            RouteSignalValue::Digest(value) => {
                validate_sha256(value, "route digest signal")?;
            }
            RouteSignalValue::Null
            | RouteSignalValue::Bool(_)
            | RouteSignalValue::Integer(_)
            | RouteSignalValue::Unsigned(_) => {}
        }
        validate_sha256(&self.evidence_sha256, "route signal evidence_sha256")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouteContextBudget {
    pub max_graph_depth: u8,
    pub max_items: u64,
    pub max_output_bytes: u64,
    pub max_scanned_changes: u64,
}

impl RouteContextBudget {
    pub fn validate(&self) -> Result<()> {
        if self.max_graph_depth > MAX_CONTEXT_GRAPH_DEPTH {
            return invalid("route context graph depth exceeds its hard bound");
        }
        validate_limit(self.max_items, MAX_CONTEXT_ITEMS, "route context max_items")?;
        validate_limit(
            self.max_output_bytes,
            MAX_CONTEXT_OUTPUT_BYTES,
            "route context max_output_bytes",
        )?;
        validate_limit(
            self.max_scanned_changes,
            MAX_CONTEXT_SCANNED_CHANGES,
            "route context max_scanned_changes",
        )
    }

    fn fits_within(&self, allowance: &Self) -> bool {
        self.max_graph_depth <= allowance.max_graph_depth
            && self.max_items <= allowance.max_items
            && self.max_output_bytes <= allowance.max_output_bytes
            && self.max_scanned_changes <= allowance.max_scanned_changes
    }
}

/// The only context coordinates a router may influence. Scope, valid time,
/// authorization, and physical access-path selection remain engine inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouteContextAllowance {
    pub scope: String,
    pub valid_at: u64,
    pub maximum_query_bytes: u64,
    #[serde(default)]
    pub eligible_seeds: Vec<DataReference>,
    pub budget: RouteContextBudget,
}

impl RouteContextAllowance {
    fn validate(&self, limits: &RouterBackendLimits) -> Result<()> {
        validate_scope(&self.scope)?;
        if self.valid_at == 0 {
            return invalid("route context valid_at must be greater than zero");
        }
        validate_limit(
            self.maximum_query_bytes,
            MAX_CONTEXT_QUERY_BYTES as u64,
            "route context maximum_query_bytes",
        )?;
        if self.eligible_seeds.len() > limits.maximum_context_seeds as usize {
            return invalid("route context eligible seeds exceed the backend limit");
        }
        if !strictly_sorted_by(&self.eligible_seeds, |seed| seed) {
            return invalid("route context eligible seeds must be unique and sorted");
        }
        self.budget.validate()
    }
}

/// One bounded, stamped routing input. It projects the existing reasoning
/// cursor and candidate contracts; it does not create another tree schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouteStepRequest {
    pub contract_version: u16,
    pub id: CorrelationId,
    pub backend_id: CanonicalId,
    pub backend_revision: u64,
    pub backend_descriptor_sha256: String,
    pub cursor: ReasoningActiveCursor,
    pub intent: String,
    #[serde(default)]
    pub signals: Vec<RouteSignal>,
    #[serde(default)]
    pub recipe_candidates: Vec<ReasoningRecipe>,
    #[serde(default)]
    pub branch_candidates: Vec<ReasoningEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RouteContextAllowance>,
    pub allowed_decisions: BTreeSet<RouteDecisionKind>,
    pub requested_at_unix_ms: u64,
    pub deadline_unix_ms: u64,
    pub request_sha256: String,
}

impl RouteStepRequest {
    pub fn validate_for(&self, backend: &RouterBackendDescriptor) -> Result<()> {
        backend.validate()?;
        validate_contract_version(self.contract_version)?;
        if self.backend_id != backend.id
            || self.backend_revision != backend.revision
            || self.backend_descriptor_sha256 != backend.descriptor_sha256
        {
            return invalid("route request does not bind its backend descriptor");
        }
        validate_sha256(
            &self.backend_descriptor_sha256,
            "route request backend_descriptor_sha256",
        )?;
        self.cursor.validate()?;
        validate_bounded_text(
            &self.intent,
            "route intent",
            backend.limits.maximum_intent_bytes as usize,
        )?;
        if self.signals.len() > backend.limits.maximum_signals as usize {
            return invalid("route signals exceed the backend limit");
        }
        if !strictly_sorted_by(&self.signals, |signal| &signal.id) {
            return invalid("route signals must be unique and sorted by id");
        }
        for signal in &self.signals {
            signal.validate()?;
        }
        if self.recipe_candidates.len() > backend.limits.maximum_recipe_candidates as usize {
            return invalid("route recipe candidates exceed the backend limit");
        }
        if !strictly_sorted_by(&self.recipe_candidates, |recipe| &recipe.id) {
            return invalid("route recipe candidates must be unique and sorted by id");
        }
        for recipe in &self.recipe_candidates {
            recipe.validate()?;
        }
        if self.branch_candidates.len() > backend.limits.maximum_branch_candidates as usize {
            return invalid("route branch candidates exceed the backend limit");
        }
        if !strictly_sorted_by(&self.branch_candidates, |edge| &edge.id) {
            return invalid("route branch candidates must be unique and sorted by id");
        }
        for edge in &self.branch_candidates {
            validate_branch_candidate(edge, &self.cursor.node_id)?;
        }
        if self.allowed_decisions.is_empty()
            || !self.allowed_decisions.is_subset(&backend.decisions)
        {
            return invalid("route request allowed decisions are empty or unsupported");
        }
        validate_candidate_presence(
            self.allowed_decisions
                .contains(&RouteDecisionKind::SelectRecipe),
            !self.recipe_candidates.is_empty(),
            "select_recipe",
        )?;
        validate_candidate_presence(
            self.allowed_decisions
                .contains(&RouteDecisionKind::AdvanceBranch),
            !self.branch_candidates.is_empty(),
            "advance_branch",
        )?;
        validate_candidate_presence(
            self.allowed_decisions
                .contains(&RouteDecisionKind::RequestContext),
            self.context.is_some(),
            "request_context",
        )?;
        if let Some(context) = &self.context {
            context.validate(&backend.limits)?;
            if context.scope != self.cursor.read.scope {
                return invalid("route context allowance differs from the cursor read scope");
            }
        }
        if self.requested_at_unix_ms == 0
            || self.deadline_unix_ms <= self.requested_at_unix_ms
            || self.deadline_unix_ms - self.requested_at_unix_ms
                > backend.limits.maximum_execution_ms
        {
            return invalid("route request deadline exceeds the backend execution limit");
        }
        validate_sha256(&self.request_sha256, "route request_sha256")?;
        if self.request_sha256 != route_step_request_sha256(self)? {
            return invalid("route request digest does not match its content");
        }
        validate_encoded_size(self, backend.limits.maximum_request_bytes, "route request")
    }
}

/// Exactly one grammar-constrained proposal from a router backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "decision", rename_all = "snake_case", deny_unknown_fields)]
pub enum RouteStepDecision {
    SelectRecipe {
        contract_version: u16,
        request_id: CorrelationId,
        request_sha256: String,
        recipe_id: CanonicalId,
        recipe_revision: u64,
        #[serde(default)]
        parameters: BTreeMap<CanonicalId, RouteParameterValue>,
        decided_at_unix_ms: u64,
        decision_sha256: String,
    },
    AdvanceBranch {
        contract_version: u16,
        request_id: CorrelationId,
        request_sha256: String,
        edge_id: CanonicalId,
        decided_at_unix_ms: u64,
        decision_sha256: String,
    },
    RequestContext {
        contract_version: u16,
        request_id: CorrelationId,
        request_sha256: String,
        query: String,
        #[serde(default)]
        seeds: Vec<DataReference>,
        budget: RouteContextBudget,
        decided_at_unix_ms: u64,
        decision_sha256: String,
    },
}

impl RouteStepDecision {
    pub const fn kind(&self) -> RouteDecisionKind {
        match self {
            Self::SelectRecipe { .. } => RouteDecisionKind::SelectRecipe,
            Self::AdvanceBranch { .. } => RouteDecisionKind::AdvanceBranch,
            Self::RequestContext { .. } => RouteDecisionKind::RequestContext,
        }
    }

    pub fn validate_for(
        &self,
        request: &RouteStepRequest,
        backend: &RouterBackendDescriptor,
    ) -> Result<()> {
        request.validate_for(backend)?;
        let binding = self.binding();
        validate_contract_version(binding.contract_version)?;
        if binding.request_id != &request.id || binding.request_sha256 != request.request_sha256 {
            return invalid("route decision does not bind its request identity and digest");
        }
        validate_sha256(binding.request_sha256, "route decision request_sha256")?;
        if binding.decided_at_unix_ms < request.requested_at_unix_ms
            || binding.decided_at_unix_ms > request.deadline_unix_ms
        {
            return invalid("route decision lies outside its request time window");
        }
        if !request.allowed_decisions.contains(&self.kind())
            || !backend.decisions.contains(&self.kind())
        {
            return invalid("route decision kind was not allowed by request and backend");
        }
        match self {
            Self::SelectRecipe {
                recipe_id,
                recipe_revision,
                parameters,
                ..
            } => {
                if !request.recipe_candidates.iter().any(|candidate| {
                    candidate.id == *recipe_id && candidate.revision == *recipe_revision
                }) {
                    return invalid("route decision selected an unknown recipe revision");
                }
                if parameters.len() > backend.limits.maximum_parameters as usize {
                    return invalid("route recipe parameters exceed the backend limit");
                }
                validate_parameters(parameters)?;
            }
            Self::AdvanceBranch { edge_id, .. } => {
                if !request
                    .branch_candidates
                    .iter()
                    .any(|candidate| candidate.id == *edge_id)
                {
                    return invalid("route decision selected an unknown branch");
                }
            }
            Self::RequestContext {
                query,
                seeds,
                budget,
                ..
            } => validate_context_decision(query, seeds, budget, request, backend)?,
        }
        validate_sha256(binding.decision_sha256, "route decision_sha256")?;
        if binding.decision_sha256 != route_step_decision_sha256(self)? {
            return invalid("route decision digest does not match its content");
        }
        validate_encoded_size(
            self,
            backend.limits.maximum_response_bytes,
            "route decision",
        )
    }

    fn binding(&self) -> RouteDecisionBinding<'_> {
        match self {
            Self::SelectRecipe {
                contract_version,
                request_id,
                request_sha256,
                decided_at_unix_ms,
                decision_sha256,
                ..
            }
            | Self::AdvanceBranch {
                contract_version,
                request_id,
                request_sha256,
                decided_at_unix_ms,
                decision_sha256,
                ..
            }
            | Self::RequestContext {
                contract_version,
                request_id,
                request_sha256,
                decided_at_unix_ms,
                decision_sha256,
                ..
            } => RouteDecisionBinding {
                contract_version: *contract_version,
                request_id,
                request_sha256,
                decided_at_unix_ms: *decided_at_unix_ms,
                decision_sha256,
            },
        }
    }
}

struct RouteDecisionBinding<'a> {
    contract_version: u16,
    request_id: &'a CorrelationId,
    request_sha256: &'a str,
    decided_at_unix_ms: u64,
    decision_sha256: &'a str,
}

pub fn router_backend_descriptor_sha256(descriptor: &RouterBackendDescriptor) -> Result<String> {
    digest_json(
        b"rrflow-router-backend-descriptor-v1",
        &(
            descriptor.contract_version,
            &descriptor.id,
            descriptor.revision,
            &descriptor.model_manifest_id,
            descriptor.model_manifest_revision,
            &descriptor.model_manifest_sha256,
            &descriptor.decisions,
            &descriptor.limits,
        ),
    )
}

/// Digest of the exact closed JSON Schema that constrained router output must
/// satisfy. Canonical object ordering prevents workspace feature unification
/// from changing this identity.
pub fn route_step_decision_schema_sha256() -> Result<String> {
    let schema =
        serde_json::to_value(schemars::schema_for!(RouteStepDecision)).map_err(|error| {
            crate::ContractError(format!("router decision schema encoding failed: {error}"))
        })?;
    let encoded = serde_json::to_vec(&crate::canonical_json(schema)).map_err(|error| {
        crate::ContractError(format!(
            "router decision schema serialization failed: {error}"
        ))
    })?;
    Ok(sha256_bytes(&encoded))
}

pub fn route_step_request_sha256(request: &RouteStepRequest) -> Result<String> {
    digest_json(
        b"rrflow-route-step-request-v1",
        &(
            request.contract_version,
            &request.id,
            &request.backend_id,
            request.backend_revision,
            &request.backend_descriptor_sha256,
            &request.cursor,
            &request.intent,
            &request.signals,
            &request.recipe_candidates,
            &request.branch_candidates,
            &request.context,
            &request.allowed_decisions,
            request.requested_at_unix_ms,
            request.deadline_unix_ms,
        ),
    )
}

pub fn route_step_decision_sha256(decision: &RouteStepDecision) -> Result<String> {
    match decision {
        RouteStepDecision::SelectRecipe {
            contract_version,
            request_id,
            request_sha256,
            recipe_id,
            recipe_revision,
            parameters,
            decided_at_unix_ms,
            ..
        } => digest_json(
            b"rrflow-route-step-decision-v1",
            &(
                "select_recipe",
                contract_version,
                request_id,
                request_sha256,
                recipe_id,
                recipe_revision,
                parameters,
                decided_at_unix_ms,
            ),
        ),
        RouteStepDecision::AdvanceBranch {
            contract_version,
            request_id,
            request_sha256,
            edge_id,
            decided_at_unix_ms,
            ..
        } => digest_json(
            b"rrflow-route-step-decision-v1",
            &(
                "advance_branch",
                contract_version,
                request_id,
                request_sha256,
                edge_id,
                decided_at_unix_ms,
            ),
        ),
        RouteStepDecision::RequestContext {
            contract_version,
            request_id,
            request_sha256,
            query,
            seeds,
            budget,
            decided_at_unix_ms,
            ..
        } => digest_json(
            b"rrflow-route-step-decision-v1",
            &(
                "request_context",
                contract_version,
                request_id,
                request_sha256,
                query,
                seeds,
                budget,
                decided_at_unix_ms,
            ),
        ),
    }
}

fn validate_context_decision(
    query: &str,
    seeds: &[DataReference],
    budget: &RouteContextBudget,
    request: &RouteStepRequest,
    backend: &RouterBackendDescriptor,
) -> Result<()> {
    let Some(allowance) = &request.context else {
        return invalid("route context decision has no request allowance");
    };
    if query.len() as u64 > allowance.maximum_query_bytes {
        return invalid("route context query exceeds its request allowance");
    }
    if seeds.len() > backend.limits.maximum_context_seeds as usize
        || !strictly_sorted_by(seeds, |seed| seed)
        || !seeds
            .iter()
            .all(|seed| allowance.eligible_seeds.binary_search(seed).is_ok())
    {
        return invalid("route context seeds are unsorted, duplicated, or ineligible");
    }
    budget.validate()?;
    if !budget.fits_within(&allowance.budget) {
        return invalid("route context decision exceeds its resource allowance");
    }
    AssembleContext {
        scope: allowance.scope.clone(),
        query: query.into(),
        valid_at: allowance.valid_at,
        seeds: seeds.to_vec(),
        max_graph_depth: budget.max_graph_depth,
        max_items: budget.max_items,
        max_output_bytes: budget.max_output_bytes,
        max_scanned_changes: budget.max_scanned_changes,
    }
    .validate()
}

fn validate_branch_candidate(edge: &ReasoningEdge, node_id: &CanonicalId) -> Result<()> {
    if &edge.from != node_id || edge.to == edge.from {
        return invalid("route branch candidate is not an outbound reasoning edge");
    }
    if edge.conditions.len() > MAX_REASONING_EDGE_CONDITIONS {
        return invalid("route branch candidate exceeds its condition bound");
    }
    if !strictly_sorted_by(&edge.conditions, |condition| &condition.id) {
        return invalid("route branch conditions must be unique and sorted by id");
    }
    for condition in &edge.conditions {
        condition.validate()?;
    }
    Ok(())
}

fn validate_candidate_presence(allowed: bool, present: bool, name: &str) -> Result<()> {
    if allowed != present {
        return invalid(format!(
            "route {name} candidates must be present exactly when the decision is allowed"
        ));
    }
    Ok(())
}

fn validate_parameters(parameters: &BTreeMap<CanonicalId, RouteParameterValue>) -> Result<()> {
    let mut items = 0_usize;
    for value in parameters.values() {
        validate_parameter_value(value, 1, &mut items)?;
    }
    Ok(())
}

fn validate_parameter_value(
    value: &RouteParameterValue,
    depth: usize,
    items: &mut usize,
) -> Result<()> {
    if depth > MAX_ROUTE_PARAMETER_DEPTH {
        return invalid("route recipe parameters exceed their nesting-depth limit");
    }
    *items = items
        .checked_add(1)
        .ok_or_else(|| crate::ContractError("route parameter item count overflowed".into()))?;
    if *items > MAX_ROUTE_PARAMETER_ITEMS {
        return invalid("route recipe parameters exceed their item limit");
    }
    match value {
        RouteParameterValue::Decimal(value) | RouteParameterValue::String(value) => {
            validate_bounded_text(value, "route recipe parameter", MAX_ROUTE_SIGNAL_BYTES)
        }
        RouteParameterValue::Digest(value) => {
            validate_sha256(value, "route recipe parameter digest")
        }
        RouteParameterValue::List(values) => {
            for value in values {
                validate_parameter_value(value, depth + 1, items)?;
            }
            Ok(())
        }
        RouteParameterValue::Map(values) => {
            for value in values.values() {
                validate_parameter_value(value, depth + 1, items)?;
            }
            Ok(())
        }
        RouteParameterValue::Null
        | RouteParameterValue::Bool(_)
        | RouteParameterValue::Integer(_)
        | RouteParameterValue::Unsigned(_) => Ok(()),
    }
}

fn validate_scope(scope: &str) -> Result<()> {
    if scope.is_empty() || scope.len() > 256 || scope.as_bytes().contains(&0) {
        return invalid("route context scope must be a non-empty bounded string without NUL");
    }
    Ok(())
}

fn validate_contract_version(version: u16) -> Result<()> {
    if version != ROUTER_CONTRACT_VERSION {
        return invalid(format!(
            "unsupported router contract version {version}; expected {ROUTER_CONTRACT_VERSION}"
        ));
    }
    Ok(())
}

fn validate_limit(value: u64, maximum: u64, field: &str) -> Result<()> {
    if value == 0 || value > maximum {
        return invalid(format!("{field} must be in 1..={maximum}"));
    }
    Ok(())
}

fn validate_bounded_text(value: &str, field: &str, maximum: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > maximum || value.as_bytes().contains(&0) {
        return invalid(format!(
            "{field} must be non-empty, contain no NUL, and use at most {maximum} bytes"
        ));
    }
    Ok(())
}

fn validate_encoded_size<T: Serialize>(value: &T, maximum: u64, field: &str) -> Result<()> {
    let encoded = serde_json::to_vec(value)
        .map_err(|error| crate::ContractError(format!("{field} encoding failed: {error}")))?;
    let length = u64::try_from(encoded.len())
        .map_err(|_| crate::ContractError(format!("{field} byte length exceeds u64")))?;
    if length > maximum {
        return invalid(format!("{field} exceeds its encoded-byte limit"));
    }
    Ok(())
}

fn strictly_sorted_by<T, K: Ord + ?Sized>(values: &[T], key: impl Fn(&T) -> &K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn digest_json<T: Serialize>(domain: &[u8], value: &T) -> Result<String> {
    let encoded = serde_json::to_vec(value).map_err(|error| {
        crate::ContractError(format!("contract digest encoding failed: {error}"))
    })?;
    let mut bytes = Vec::with_capacity(domain.len() + 1 + encoded.len());
    bytes.extend_from_slice(domain);
    bytes.push(0);
    bytes.extend_from_slice(&encoded);
    Ok(sha256_bytes(&bytes))
}
