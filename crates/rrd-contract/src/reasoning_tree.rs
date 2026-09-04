//! Versioned public schema for generic RRFlow reasoning trees.
//!
//! This transport boundary mirrors the kernel semantics without importing a
//! runtime implementation. Cross-boundary conformance tests feed the same
//! golden and rejection corpus to this module and `rrd-core`.

use crate::{invalid, sha256_bytes, validate_sha256, CanonicalId, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const REASONING_TREE_CONTRACT_VERSION: u16 = 1;
pub const MAX_REASONING_TREE_NODES: usize = 4_096;
pub const MAX_REASONING_TREE_EDGES: usize = 16_384;
pub const MAX_REASONING_TREE_RECIPES: usize = 4_096;
pub const MAX_REASONING_EDGE_CONDITIONS: usize = 64;
pub const MAX_REASONING_EVIDENCE_ITEMS: usize = 64;
pub const MAX_REASONING_EVIDENCE_SOURCE_BYTES: usize = 4_096;
pub const MAX_REASONING_EVIDENCE_SUMMARY_BYTES: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningRecipe {
    pub id: CanonicalId,
    pub revision: u64,
    pub kind: CanonicalId,
    pub instruction_sha256: String,
    pub parameter_schema_sha256: String,
    pub output_schema_sha256: String,
}

impl ReasoningRecipe {
    pub fn validate(&self) -> Result<()> {
        if self.revision == 0 {
            return invalid("reasoning recipe revision must be greater than zero");
        }
        validate_sha256(
            &self.instruction_sha256,
            "reasoning recipe instruction_sha256",
        )?;
        validate_sha256(
            &self.parameter_schema_sha256,
            "reasoning recipe parameter_schema_sha256",
        )?;
        validate_sha256(
            &self.output_schema_sha256,
            "reasoning recipe output_schema_sha256",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningNode {
    pub id: CanonicalId,
    pub kind: CanonicalId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<CanonicalId>,
    #[serde(default)]
    pub terminal: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_requirement: Option<CanonicalId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "predicate", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReasoningConditionPredicate {
    Always,
    Snapshot {
        evaluator: CanonicalId,
        expression_sha256: String,
    },
    VerificationPassed {
        requirement: CanonicalId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningCondition {
    pub id: CanonicalId,
    pub predicate: ReasoningConditionPredicate,
}

impl ReasoningCondition {
    pub fn validate(&self) -> Result<()> {
        if let ReasoningConditionPredicate::Snapshot {
            expression_sha256, ..
        } = &self.predicate
        {
            validate_sha256(expression_sha256, "reasoning condition expression_sha256")?;
        }
        Ok(())
    }

    fn requires_evidence(&self) -> bool {
        !matches!(self.predicate, ReasoningConditionPredicate::Always)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningEdge {
    pub id: CanonicalId,
    pub kind: CanonicalId,
    pub from: CanonicalId,
    pub to: CanonicalId,
    #[serde(default)]
    pub conditions: Vec<ReasoningCondition>,
}

impl ReasoningEdge {
    fn validate(&self) -> Result<()> {
        if self.conditions.len() > MAX_REASONING_EDGE_CONDITIONS {
            return invalid("reasoning edge exceeds its condition bound");
        }
        let mut condition_ids = BTreeSet::new();
        for condition in &self.conditions {
            condition.validate()?;
            if !condition_ids.insert(&condition.id) {
                return invalid("reasoning condition ids must be unique within an edge");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningTree {
    pub contract_version: u16,
    pub id: CanonicalId,
    pub revision: u64,
    pub root: CanonicalId,
    #[serde(default)]
    pub recipes: Vec<ReasoningRecipe>,
    pub nodes: Vec<ReasoningNode>,
    #[serde(default)]
    pub edges: Vec<ReasoningEdge>,
}

impl ReasoningTree {
    pub fn validate(&self) -> Result<()> {
        validate_version(self.contract_version)?;
        if self.revision == 0 {
            return invalid("reasoning tree revision must be greater than zero");
        }
        if self.nodes.is_empty() || self.nodes.len() > MAX_REASONING_TREE_NODES {
            return invalid("reasoning tree node count is outside its bound");
        }
        if self.edges.len() > MAX_REASONING_TREE_EDGES {
            return invalid("reasoning tree exceeds its edge bound");
        }
        if self.recipes.len() > MAX_REASONING_TREE_RECIPES {
            return invalid("reasoning tree exceeds its recipe bound");
        }

        let mut recipes = BTreeMap::new();
        for recipe in &self.recipes {
            recipe.validate()?;
            if recipes.insert(&recipe.id, recipe).is_some() {
                return invalid("reasoning recipe ids must be unique");
            }
        }

        let mut nodes = BTreeMap::new();
        for node in &self.nodes {
            if node.verification_requirement.is_some() && !node.terminal {
                return invalid(
                    "only a terminal reasoning node may declare a verification requirement",
                );
            }
            if let Some(recipe_id) = &node.recipe_id {
                if !recipes.contains_key(recipe_id) {
                    return invalid("reasoning node references an unknown recipe");
                }
            }
            if nodes.insert(&node.id, node).is_some() {
                return invalid("reasoning node ids must be unique");
            }
        }
        if !nodes.contains_key(&self.root) {
            return invalid("reasoning tree root does not name a declared node");
        }

        let mut edge_ids = BTreeSet::new();
        let mut incoming = BTreeMap::<&CanonicalId, usize>::new();
        let mut outgoing = BTreeMap::<&CanonicalId, Vec<&CanonicalId>>::new();
        for edge in &self.edges {
            edge.validate()?;
            if !edge_ids.insert(&edge.id) {
                return invalid("reasoning edge ids must be unique");
            }
            if edge.from == edge.to {
                return invalid("reasoning edges must not be self-referential");
            }
            if !nodes.contains_key(&edge.from) || !nodes.contains_key(&edge.to) {
                return invalid("reasoning edge references an unknown node");
            }
            *incoming.entry(&edge.to).or_default() += 1;
            outgoing.entry(&edge.from).or_default().push(&edge.to);
        }

        if incoming.get(&self.root).copied().unwrap_or_default() != 0 {
            return invalid("reasoning tree root must not have an incoming edge");
        }
        for node in &self.nodes {
            let incoming_count = incoming.get(&node.id).copied().unwrap_or_default();
            if node.id != self.root && incoming_count != 1 {
                return invalid("every non-root reasoning node must have exactly one parent");
            }
            let outgoing_count = outgoing.get(&node.id).map_or(0, Vec::len);
            if node.terminal && outgoing_count != 0 {
                return invalid("terminal reasoning nodes must not have outgoing edges");
            }
            if !node.terminal && outgoing_count == 0 {
                return invalid("reasoning leaves must be explicitly terminal");
            }
        }

        let mut reached = BTreeSet::new();
        let mut pending = VecDeque::from([&self.root]);
        while let Some(node) = pending.pop_front() {
            if !reached.insert(node) {
                return invalid("reasoning tree contains a cycle");
            }
            if let Some(children) = outgoing.get(node) {
                pending.extend(children.iter().copied());
            }
        }
        if reached.len() != self.nodes.len() {
            return invalid("reasoning tree contains nodes unreachable from its root");
        }
        Ok(())
    }

    fn node(&self, id: &CanonicalId) -> Option<&ReasoningNode> {
        self.nodes.iter().find(|node| &node.id == id)
    }

    fn edge(&self, id: &CanonicalId) -> Option<&ReasoningEdge> {
        self.edges.iter().find(|edge| &edge.id == id)
    }

    fn recipe(&self, id: &CanonicalId) -> Option<&ReasoningRecipe> {
        self.recipes.iter().find(|recipe| &recipe.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningEvidence {
    pub kind: CanonicalId,
    pub source: String,
    pub content_sha256: String,
    pub observed_at: u64,
    pub summary: String,
}

impl ReasoningEvidence {
    pub fn validate(&self) -> Result<()> {
        validate_bounded_text(
            &self.source,
            "reasoning evidence source",
            MAX_REASONING_EVIDENCE_SOURCE_BYTES,
        )?;
        validate_sha256(&self.content_sha256, "reasoning evidence content_sha256")?;
        if self.observed_at == 0 {
            return invalid("reasoning evidence observed_at must be greater than zero");
        }
        validate_bounded_text(
            &self.summary,
            "reasoning evidence summary",
            MAX_REASONING_EVIDENCE_SUMMARY_BYTES,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningRecipeSelection {
    pub recipe_id: CanonicalId,
    pub recipe_revision: u64,
    pub parameters_sha256: String,
}

impl ReasoningRecipeSelection {
    fn validate(&self) -> Result<()> {
        if self.recipe_revision == 0 {
            return invalid("selected recipe revision must be greater than zero");
        }
        validate_sha256(&self.parameters_sha256, "selected recipe parameters_sha256")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningConditionEvaluation {
    pub condition_id: CanonicalId,
    pub satisfied: bool,
    #[serde(default)]
    pub evidence: Vec<ReasoningEvidence>,
}

impl ReasoningConditionEvaluation {
    fn validate(&self, decided_at: u64) -> Result<()> {
        validate_evidence(&self.evidence, decided_at)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningDecisionEvidence {
    pub id: CanonicalId,
    pub actor_kind: CanonicalId,
    pub actor_id: CanonicalId,
    pub cursor_id: CanonicalId,
    pub cursor_step: u64,
    pub read_manifest_sha256: String,
    pub selected_edge: CanonicalId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe: Option<ReasoningRecipeSelection>,
    pub input_sha256: String,
    pub decided_at: u64,
    pub condition_evaluations: Vec<ReasoningConditionEvaluation>,
    pub evidence: Vec<ReasoningEvidence>,
}

impl ReasoningDecisionEvidence {
    pub fn validate(&self) -> Result<()> {
        if self.decided_at == 0 {
            return invalid("reasoning decision decided_at must be greater than zero");
        }
        validate_sha256(
            &self.read_manifest_sha256,
            "reasoning decision read_manifest_sha256",
        )?;
        validate_sha256(&self.input_sha256, "reasoning decision input_sha256")?;
        if let Some(recipe) = &self.recipe {
            recipe.validate()?;
        }
        if self.condition_evaluations.len() > MAX_REASONING_EDGE_CONDITIONS {
            return invalid("reasoning decision exceeds its condition-evaluation bound");
        }
        for evaluation in &self.condition_evaluations {
            evaluation.validate(self.decided_at)?;
        }
        if self.evidence.is_empty() {
            return invalid("reasoning decision must cite operational evidence");
        }
        validate_evidence(&self.evidence, self.decided_at)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningVerificationStatus {
    Passed,
    Failed,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningVerificationResult {
    pub id: CanonicalId,
    pub requirement: CanonicalId,
    pub status: ReasoningVerificationStatus,
    pub checked_at: u64,
    pub evidence: Vec<ReasoningEvidence>,
}

impl ReasoningVerificationResult {
    pub fn validate(&self) -> Result<()> {
        if self.checked_at == 0 {
            return invalid("reasoning verification checked_at must be greater than zero");
        }
        if self.evidence.is_empty() {
            return invalid("reasoning verification must cite operational evidence");
        }
        validate_evidence(&self.evidence, self.checked_at)
    }
}

/// Public shape of the kernel read stamp. Runtime scopes remain strings
/// because their canonical `kind:value` coordinates are not URL path IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningReadStamp {
    pub contract_version: u16,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_revision: Option<u64>,
    pub catalog_revision: u64,
    pub commit_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accumulator_root: Option<String>,
    pub manifest_id: String,
}

impl ReasoningReadStamp {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != 1 {
            return invalid(format!(
                "unsupported data-runtime contract version {}",
                self.contract_version
            ));
        }
        validate_bounded_text(&self.scope, "reasoning read scope", 4_096)?;
        if self.schema_revision == Some(0) {
            return invalid("schema revision zero is not a valid installed revision");
        }
        match (self.commit_cursor, self.head_digest.as_deref()) {
            (0, None) => {}
            (0, Some(_)) => return invalid("an empty runtime cannot have a hash-chain head"),
            (_, Some(value)) => validate_sha256(value, "read stamp hash-chain head")?,
            (_, None) => return invalid("a non-empty runtime must carry its hash-chain head"),
        }
        if let Some(root) = &self.accumulator_root {
            validate_sha256(root, "read stamp accumulator root")?;
        }
        validate_sha256(&self.manifest_id, "read stamp manifest")?;
        if self.manifest_id != sha256_bytes(&self.manifest_bytes()) {
            return invalid("read stamp manifest digest does not match its fields");
        }
        Ok(())
    }

    fn manifest_bytes(&self) -> Vec<u8> {
        let mut output = b"rrflow-read-stamp-v1\0".to_vec();
        output.extend_from_slice(&self.contract_version.to_be_bytes());
        encode_text(&mut output, &self.scope);
        output.push(u8::from(self.schema_revision.is_some()));
        if let Some(revision) = self.schema_revision {
            output.extend_from_slice(&revision.to_be_bytes());
        }
        output.extend_from_slice(&self.catalog_revision.to_be_bytes());
        output.extend_from_slice(&self.commit_cursor.to_be_bytes());
        output.push(u8::from(self.head_digest.is_some()));
        if let Some(head) = &self.head_digest {
            encode_text(&mut output, head);
        }
        if let Some(root) = &self.accumulator_root {
            output.extend_from_slice(b"rrflow-read-stamp-accumulator-v1\0");
            encode_text(&mut output, root);
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningActiveCursor {
    pub contract_version: u16,
    pub id: CanonicalId,
    pub tree_id: CanonicalId,
    pub tree_revision: u64,
    pub node_id: CanonicalId,
    pub step: u64,
    pub read: ReasoningReadStamp,
}

impl ReasoningActiveCursor {
    pub fn validate(&self) -> Result<()> {
        validate_version(self.contract_version)?;
        self.read.validate()?;
        if self.tree_revision == 0 {
            return invalid("reasoning cursor tree revision must be greater than zero");
        }
        Ok(())
    }

    pub fn validate_for(&self, tree: &ReasoningTree) -> Result<()> {
        self.validate()?;
        if self.tree_id != tree.id || self.tree_revision != tree.revision {
            return invalid("reasoning cursor does not match the tree identity and revision");
        }
        if tree.node(&self.node_id).is_none() {
            return invalid("reasoning cursor references an unknown node");
        }
        if self.step == 0 && self.node_id != tree.root {
            return invalid("reasoning cursor step zero must point at the tree root");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningCursorAdvance {
    pub contract_version: u16,
    pub before: ReasoningActiveCursor,
    pub edge_id: CanonicalId,
    pub decision: ReasoningDecisionEvidence,
    #[serde(default)]
    pub verifications: Vec<ReasoningVerificationResult>,
    pub after: ReasoningActiveCursor,
}

impl ReasoningCursorAdvance {
    pub fn validate(&self, tree: &ReasoningTree) -> Result<()> {
        validate_version(self.contract_version)?;
        tree.validate()?;
        self.before.validate_for(tree)?;
        self.after.validate_for(tree)?;
        self.decision.validate()?;

        if self.before.id != self.after.id {
            return invalid("reasoning cursor identity changed during an advance");
        }
        if self.before.read != self.after.read {
            return invalid("reasoning cursor advance changed its decision read stamp");
        }
        let expected_step = self
            .before
            .step
            .checked_add(1)
            .ok_or_else(|| crate::ContractError("reasoning cursor step overflowed".into()))?;
        if self.after.step != expected_step {
            return invalid("reasoning cursor advance must increment step exactly once");
        }

        let source = tree
            .node(&self.before.node_id)
            .ok_or_else(|| crate::ContractError("reasoning cursor source node is absent".into()))?;
        if source.terminal {
            return invalid("reasoning cursor cannot advance from a terminal node");
        }
        let target = tree
            .node(&self.after.node_id)
            .ok_or_else(|| crate::ContractError("reasoning cursor target node is absent".into()))?;
        let edge = tree.edge(&self.edge_id).ok_or_else(|| {
            crate::ContractError("reasoning cursor advance references an unknown edge".into())
        })?;
        if edge.from != self.before.node_id || edge.to != self.after.node_id {
            return invalid("reasoning cursor advance does not follow the selected edge");
        }

        if self.decision.cursor_id != self.before.id
            || self.decision.cursor_step != self.before.step
            || self.decision.read_manifest_sha256 != self.before.read.manifest_id
            || self.decision.selected_edge != self.edge_id
        {
            return invalid("reasoning decision is not bound to the cursor and selected edge");
        }
        match (&source.recipe_id, &self.decision.recipe) {
            (None, None) => {}
            (Some(recipe_id), Some(selection)) if recipe_id == &selection.recipe_id => {
                let recipe = tree.recipe(recipe_id).ok_or_else(|| {
                    crate::ContractError("reasoning decision selected an unknown recipe".into())
                })?;
                if recipe.revision != selection.recipe_revision {
                    return invalid("reasoning decision selected the wrong recipe revision");
                }
            }
            _ => return invalid("reasoning decision recipe does not match its source node"),
        }

        let conditions = edge
            .conditions
            .iter()
            .map(|condition| (&condition.id, condition))
            .collect::<BTreeMap<_, _>>();
        let mut evaluated = BTreeSet::new();
        for evaluation in &self.decision.condition_evaluations {
            let condition = conditions.get(&evaluation.condition_id).ok_or_else(|| {
                crate::ContractError(
                    "reasoning decision evaluated a condition not on its edge".into(),
                )
            })?;
            if !evaluated.insert(&evaluation.condition_id) {
                return invalid("reasoning decision evaluated a condition more than once");
            }
            if !evaluation.satisfied {
                return invalid("reasoning cursor cannot advance across an unsatisfied condition");
            }
            if condition.requires_evidence() && evaluation.evidence.is_empty() {
                return invalid("reasoning condition evaluation is missing evidence");
            }
        }
        if evaluated.len() != edge.conditions.len() {
            return invalid("reasoning decision did not evaluate every edge condition");
        }

        let mut verification_ids = BTreeSet::new();
        let mut verification_requirements = BTreeMap::new();
        for verification in &self.verifications {
            verification.validate()?;
            if verification.checked_at < self.decision.decided_at {
                return invalid("reasoning verification predates its decision");
            }
            if !verification_ids.insert(&verification.id) {
                return invalid("reasoning verification result ids must be unique");
            }
            if verification_requirements
                .insert(&verification.requirement, verification)
                .is_some()
            {
                return invalid("reasoning verification requirements must be unique");
            }
        }

        let mut required = BTreeSet::new();
        for condition in &edge.conditions {
            if let ReasoningConditionPredicate::VerificationPassed { requirement } =
                &condition.predicate
            {
                required.insert(requirement);
            }
        }
        if let Some(requirement) = &target.verification_requirement {
            required.insert(requirement);
        }
        if verification_requirements.len() != required.len()
            || verification_requirements
                .keys()
                .any(|requirement| !required.contains(requirement))
        {
            return invalid("reasoning cursor advance has missing or unrelated verifications");
        }
        for requirement in required {
            let verification = verification_requirements.get(requirement).ok_or_else(|| {
                crate::ContractError(
                    "reasoning cursor advance is missing a required verification".into(),
                )
            })?;
            if verification.status != ReasoningVerificationStatus::Passed {
                return invalid("reasoning cursor advance requires passing verification");
            }
        }
        Ok(())
    }
}

fn validate_evidence(evidence: &[ReasoningEvidence], not_after: u64) -> Result<()> {
    if evidence.len() > MAX_REASONING_EVIDENCE_ITEMS {
        return invalid("reasoning evidence exceeds its item bound");
    }
    let mut identities = BTreeSet::new();
    for item in evidence {
        item.validate()?;
        if item.observed_at > not_after {
            return invalid("reasoning evidence was observed after the decision it supports");
        }
        if !identities.insert((
            &item.kind,
            item.source.as_str(),
            item.content_sha256.as_str(),
        )) {
            return invalid("reasoning evidence entries must be unique");
        }
    }
    Ok(())
}

fn validate_version(version: u16) -> Result<()> {
    if version != REASONING_TREE_CONTRACT_VERSION {
        return invalid(format!(
            "unsupported reasoning-tree contract version {version}"
        ));
    }
    Ok(())
}

fn validate_bounded_text(value: &str, field: &str, maximum: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > maximum || value.as_bytes().contains(&0) {
        return invalid(format!(
            "{field} must contain 1..={maximum} bytes and no NUL"
        ));
    }
    Ok(())
}

fn encode_text(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}
