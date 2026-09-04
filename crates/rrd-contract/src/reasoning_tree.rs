//! Versioned public schema for generic RRFlow reasoning trees.
//!
//! Public identifiers remain transport-safe [`CanonicalId`] values. Semantic
//! validation is delegated inward to `rrd-core`; this module does not create a
//! second lifecycle or cursor authority.

use crate::{CanonicalId, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use rrd_core::{
    MAX_REASONING_EDGE_CONDITIONS, MAX_REASONING_EVIDENCE_ITEMS,
    MAX_REASONING_EVIDENCE_SOURCE_BYTES, MAX_REASONING_EVIDENCE_SUMMARY_BYTES,
    MAX_REASONING_TREE_EDGES, MAX_REASONING_TREE_NODES, MAX_REASONING_TREE_RECIPES,
    REASONING_TREE_CONTRACT_VERSION,
};

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
        self.to_kernel()?.validate().map_err(kernel_error)
    }

    fn to_kernel(&self) -> Result<rrd_core::ReasoningTree> {
        Ok(rrd_core::ReasoningTree {
            contract_version: self.contract_version,
            id: runtime_id(&self.id)?,
            revision: self.revision,
            root: runtime_id(&self.root)?,
            recipes: self
                .recipes
                .iter()
                .map(ReasoningRecipe::to_kernel)
                .collect::<Result<_>>()?,
            nodes: self
                .nodes
                .iter()
                .map(ReasoningNode::to_kernel)
                .collect::<Result<_>>()?,
            edges: self
                .edges
                .iter()
                .map(ReasoningEdge::to_kernel)
                .collect::<Result<_>>()?,
        })
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningRecipeSelection {
    pub recipe_id: CanonicalId,
    pub recipe_revision: u64,
    pub parameters_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasoningConditionEvaluation {
    pub condition_id: CanonicalId,
    pub satisfied: bool,
    #[serde(default)]
    pub evidence: Vec<ReasoningEvidence>,
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

/// Public shape of the kernel read stamp. The scope is a string because
/// canonical runtime scopes use `kind:value` coordinates rather than URL path
/// segments. Validation reconstructs the kernel stamp and verifies its digest.
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
    fn to_kernel(&self) -> Result<rrd_core::ReadStamp> {
        let stamp = rrd_core::ReadStamp {
            contract_version: self.contract_version,
            scope: rrd_core::ScopeId::new(&self.scope).map_err(kernel_error)?,
            schema_revision: self.schema_revision,
            catalog_revision: self.catalog_revision,
            commit_cursor: self.commit_cursor,
            head_digest: self.head_digest.clone(),
            accumulator_root: self.accumulator_root.clone(),
            manifest_id: self.manifest_id.clone(),
        };
        stamp.validate().map_err(kernel_error)?;
        Ok(stamp)
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
        let tree = tree.to_kernel()?;
        self.to_kernel()?.validate(&tree).map_err(kernel_error)
    }

    fn to_kernel(&self) -> Result<rrd_core::ReasoningCursorAdvance> {
        Ok(rrd_core::ReasoningCursorAdvance {
            contract_version: self.contract_version,
            before: self.before.to_kernel()?,
            edge_id: runtime_id(&self.edge_id)?,
            decision: self.decision.to_kernel()?,
            verifications: self
                .verifications
                .iter()
                .map(ReasoningVerificationResult::to_kernel)
                .collect::<Result<_>>()?,
            after: self.after.to_kernel()?,
        })
    }
}

impl ReasoningRecipe {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningRecipe> {
        Ok(rrd_core::ReasoningRecipe {
            id: runtime_id(&self.id)?,
            revision: self.revision,
            kind: runtime_type(&self.kind)?,
            instruction_sha256: self.instruction_sha256.clone(),
            parameter_schema_sha256: self.parameter_schema_sha256.clone(),
            output_schema_sha256: self.output_schema_sha256.clone(),
        })
    }
}

impl ReasoningNode {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningNode> {
        Ok(rrd_core::ReasoningNode {
            id: runtime_id(&self.id)?,
            kind: runtime_type(&self.kind)?,
            recipe_id: self.recipe_id.as_ref().map(runtime_id).transpose()?,
            terminal: self.terminal,
            verification_requirement: self
                .verification_requirement
                .as_ref()
                .map(runtime_id)
                .transpose()?,
        })
    }
}

impl ReasoningConditionPredicate {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningConditionPredicate> {
        Ok(match self {
            Self::Always => rrd_core::ReasoningConditionPredicate::Always,
            Self::Snapshot {
                evaluator,
                expression_sha256,
            } => rrd_core::ReasoningConditionPredicate::Snapshot {
                evaluator: runtime_type(evaluator)?,
                expression_sha256: expression_sha256.clone(),
            },
            Self::VerificationPassed { requirement } => {
                rrd_core::ReasoningConditionPredicate::VerificationPassed {
                    requirement: runtime_id(requirement)?,
                }
            }
        })
    }
}

impl ReasoningCondition {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningCondition> {
        Ok(rrd_core::ReasoningCondition {
            id: runtime_id(&self.id)?,
            predicate: self.predicate.to_kernel()?,
        })
    }
}

impl ReasoningEdge {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningEdge> {
        Ok(rrd_core::ReasoningEdge {
            id: runtime_id(&self.id)?,
            kind: runtime_type(&self.kind)?,
            from: runtime_id(&self.from)?,
            to: runtime_id(&self.to)?,
            conditions: self
                .conditions
                .iter()
                .map(ReasoningCondition::to_kernel)
                .collect::<Result<_>>()?,
        })
    }
}

impl ReasoningEvidence {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningEvidence> {
        Ok(rrd_core::ReasoningEvidence {
            kind: runtime_type(&self.kind)?,
            source: self.source.clone(),
            content_sha256: self.content_sha256.clone(),
            observed_at: self.observed_at,
            summary: self.summary.clone(),
        })
    }
}

impl ReasoningRecipeSelection {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningRecipeSelection> {
        Ok(rrd_core::ReasoningRecipeSelection {
            recipe_id: runtime_id(&self.recipe_id)?,
            recipe_revision: self.recipe_revision,
            parameters_sha256: self.parameters_sha256.clone(),
        })
    }
}

impl ReasoningConditionEvaluation {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningConditionEvaluation> {
        Ok(rrd_core::ReasoningConditionEvaluation {
            condition_id: runtime_id(&self.condition_id)?,
            satisfied: self.satisfied,
            evidence: self
                .evidence
                .iter()
                .map(ReasoningEvidence::to_kernel)
                .collect::<Result<_>>()?,
        })
    }
}

impl ReasoningDecisionEvidence {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningDecisionEvidence> {
        Ok(rrd_core::ReasoningDecisionEvidence {
            id: runtime_id(&self.id)?,
            actor_kind: runtime_type(&self.actor_kind)?,
            actor_id: runtime_id(&self.actor_id)?,
            cursor_id: runtime_id(&self.cursor_id)?,
            cursor_step: self.cursor_step,
            read_manifest_sha256: self.read_manifest_sha256.clone(),
            selected_edge: runtime_id(&self.selected_edge)?,
            recipe: self
                .recipe
                .as_ref()
                .map(ReasoningRecipeSelection::to_kernel)
                .transpose()?,
            input_sha256: self.input_sha256.clone(),
            decided_at: self.decided_at,
            condition_evaluations: self
                .condition_evaluations
                .iter()
                .map(ReasoningConditionEvaluation::to_kernel)
                .collect::<Result<_>>()?,
            evidence: self
                .evidence
                .iter()
                .map(ReasoningEvidence::to_kernel)
                .collect::<Result<_>>()?,
        })
    }
}

impl From<ReasoningVerificationStatus> for rrd_core::ReasoningVerificationStatus {
    fn from(status: ReasoningVerificationStatus) -> Self {
        match status {
            ReasoningVerificationStatus::Passed => Self::Passed,
            ReasoningVerificationStatus::Failed => Self::Failed,
            ReasoningVerificationStatus::Inconclusive => Self::Inconclusive,
        }
    }
}

impl ReasoningVerificationResult {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningVerificationResult> {
        Ok(rrd_core::ReasoningVerificationResult {
            id: runtime_id(&self.id)?,
            requirement: runtime_id(&self.requirement)?,
            status: self.status.into(),
            checked_at: self.checked_at,
            evidence: self
                .evidence
                .iter()
                .map(ReasoningEvidence::to_kernel)
                .collect::<Result<_>>()?,
        })
    }
}

impl ReasoningActiveCursor {
    fn to_kernel(&self) -> Result<rrd_core::ReasoningActiveCursor> {
        Ok(rrd_core::ReasoningActiveCursor {
            contract_version: self.contract_version,
            id: runtime_id(&self.id)?,
            tree_id: runtime_id(&self.tree_id)?,
            tree_revision: self.tree_revision,
            node_id: runtime_id(&self.node_id)?,
            step: self.step,
            read: self.read.to_kernel()?,
        })
    }
}

fn runtime_id(id: &CanonicalId) -> Result<rrd_core::RuntimeId> {
    rrd_core::RuntimeId::new(id.as_str()).map_err(kernel_error)
}

fn runtime_type(kind: &CanonicalId) -> Result<rrd_core::RuntimeType> {
    rrd_core::RuntimeType::new(kind.as_str()).map_err(kernel_error)
}

fn kernel_error(error: rrd_core::Error) -> crate::ContractError {
    crate::ContractError(error.to_string())
}
