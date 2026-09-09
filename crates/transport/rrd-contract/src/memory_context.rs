use crate::{invalid, validate_sha256, DataReference, QueryValue, ReadEvidence, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_CONTEXT_QUERY_BYTES: usize = 64 * 1024;
pub const MAX_CONTEXT_SEEDS: usize = 256;
pub const MAX_CONTEXT_ITEMS: u64 = 512;
pub const MAX_CONTEXT_OUTPUT_BYTES: u64 = 768 * 1024;
pub const MAX_CONTEXT_STORAGE_KEYS: u64 = 1_000_000;
pub const MAX_CONTEXT_GRAPH_DEPTH: u8 = 32;

/// One explicit, bounded context request over canonical semantic versions.
///
/// The caller supplies intent and optional anchors, not storage topology.
/// Record fields and graph relations are discovered by the engine from the
/// one catalogue/data snapshot captured for this request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssembleContext {
    pub scope: String,
    pub query: String,
    pub valid_at: u64,
    #[serde(default)]
    pub seeds: Vec<DataReference>,
    pub max_graph_depth: u8,
    pub max_items: u64,
    pub max_output_bytes: u64,
    pub max_storage_keys: u64,
}

impl AssembleContext {
    pub fn validate(&self) -> Result<()> {
        if self.scope.is_empty() || self.scope.len() > 256 || self.scope.as_bytes().contains(&0) {
            return invalid("context scope must be a non-empty bounded string without NUL");
        }
        if self.query.len() > MAX_CONTEXT_QUERY_BYTES || self.query.as_bytes().contains(&0) {
            return invalid("context query exceeds its byte bound or contains NUL");
        }
        if self.valid_at == 0 {
            return invalid("context valid_at must be greater than zero");
        }
        if self.seeds.is_empty() && self.query.trim().is_empty() {
            return invalid("context request requires a query or at least one seed");
        }
        if self.seeds.len() > MAX_CONTEXT_SEEDS {
            return invalid("context request exceeds its seed bound");
        }
        if self.max_items == 0 || self.max_items > MAX_CONTEXT_ITEMS {
            return invalid("context max_items exceeds its bound");
        }
        if self.max_output_bytes == 0 || self.max_output_bytes > MAX_CONTEXT_OUTPUT_BYTES {
            return invalid("context max_output_bytes exceeds its bound");
        }
        if self.max_storage_keys == 0 || self.max_storage_keys > MAX_CONTEXT_STORAGE_KEYS {
            return invalid("context max_storage_keys exceeds its bound");
        }
        if self.max_graph_depth > MAX_CONTEXT_GRAPH_DEPTH {
            return invalid("context graph depth exceeds 32");
        }
        let unique_seeds = self.seeds.iter().collect::<BTreeSet<_>>();
        if unique_seeds.len() != self.seeds.len() {
            return invalid("context seeds must be unique");
        }
        Ok(())
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ContextEvidenceKind {
    Seed,
    Text,
    Graph,
    Vector,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextEvidence {
    pub kind: ContextEvidenceKind,
    pub source: String,
    pub source_rank: u64,
    pub source_score: f64,
    pub contribution: f64,
    pub plan_sha256: String,
    pub evidence_sha256: String,
}

impl ContextEvidence {
    pub fn validate(&self) -> Result<()> {
        if self.source.is_empty() || self.source.len() > 512 || self.source.as_bytes().contains(&0)
        {
            return invalid("context evidence source is empty, oversized, or contains NUL");
        }
        if self.source_rank == 0
            || !self.source_score.is_finite()
            || !self.contribution.is_finite()
            || self.contribution <= 0.0
        {
            return invalid("context evidence rank and scores are invalid");
        }
        validate_sha256(&self.plan_sha256, "plan_sha256")?;
        validate_sha256(&self.evidence_sha256, "evidence_sha256")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextItem {
    pub identity: String,
    pub score: f64,
    pub values: BTreeMap<String, QueryValue>,
    pub evidence: Vec<ContextEvidence>,
}

impl ContextItem {
    pub fn validate(&self) -> Result<()> {
        if self.identity.is_empty()
            || self.identity.len() > 512
            || self.identity.as_bytes().contains(&0)
        {
            return invalid("context item identity is empty, oversized, or contains NUL");
        }
        if !self.score.is_finite() || self.score <= 0.0 {
            return invalid("context item score must be finite and positive");
        }
        if self.evidence.is_empty() {
            return invalid("context item must carry retrieval evidence");
        }
        for evidence in &self.evidence {
            evidence.validate()?;
        }
        let evidence_ids = self
            .evidence
            .iter()
            .map(|evidence| evidence.evidence_sha256.as_str())
            .collect::<BTreeSet<_>>();
        if evidence_ids.len() != self.evidence.len() {
            return invalid("context item evidence must be unique");
        }
        let expected_score = self
            .evidence
            .iter()
            .map(|evidence| evidence.contribution)
            .sum::<f64>();
        let tolerance = f64::EPSILON * expected_score.abs().max(1.0) * 8.0;
        if (self.score - expected_score).abs() > tolerance {
            return invalid("context item score differs from its evidence contributions");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextReadStamp {
    pub runtime_manifest_sha256: String,
    pub runtime_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_revision: Option<u64>,
    pub catalogue_revision: u64,
}

impl ContextReadStamp {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(
            &self.runtime_manifest_sha256,
            "read.runtime_manifest_sha256",
        )
    }
}

/// A canonical stage in the engine-owned context plan.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ContextPlanStageKind {
    Seed,
    Lexical,
    Semantic,
    Graph,
    Fusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContextPlanStageStatus {
    Selected,
    Skipped,
}

/// The physical access path actually selected for a context stage.
///
/// These names describe algorithms, never project-, provider-, or
/// deployment-specific resources. New physical implementations extend this
/// contract instead of being smuggled through free-form strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContextAccessPath {
    RequestSeed,
    SnapshotBm25,
    SnapshotVectorExact,
    SnapshotGraphBidirectionalBfs,
    ReciprocalRankFusion,
}

/// One selected or skipped decision in the context execution plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPlanStage {
    pub kind: ContextPlanStageKind,
    pub status: ContextPlanStageStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_path: Option<ContextAccessPath>,
    pub exact: bool,
    pub reason: String,
    pub decision_sha256: String,
}

impl ContextPlanStage {
    pub fn validate(&self) -> Result<()> {
        if self.reason.is_empty()
            || self.reason.len() > 1_024
            || self.reason.as_bytes().contains(&0)
        {
            return invalid("context plan stage reason is empty, oversized, or contains NUL");
        }
        match (self.status, self.access_path, self.exact) {
            (ContextPlanStageStatus::Selected, Some(_), _) => {}
            (ContextPlanStageStatus::Skipped, None, false) => {}
            _ => {
                return invalid(
                    "selected context stages require an access path; skipped stages must not claim one or exact execution",
                )
            }
        }
        validate_sha256(&self.decision_sha256, "decision_sha256")?;
        if self.decision_sha256 != context_plan_stage_sha256(self)? {
            return invalid("context plan stage digest does not match its decision");
        }
        Ok(())
    }
}

/// The complete, deterministic context execution plan at one read stamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPlanSnapshot {
    /// Digest of the complete request, including query, anchors, and all
    /// resource budgets.
    pub request_sha256: String,
    /// Revision of the security authority compiled before context planning.
    /// Zero denotes an explicitly unsecured loopback development engine.
    pub security_policy_revision: u64,
    /// Digest of the exact principal, action, resource, credential revision,
    /// and optional data policy compiled for this context read.
    pub authorization_sha256: String,
    pub read: ContextReadStamp,
    pub stages: Vec<ContextPlanStage>,
    pub plan_sha256: String,
}

impl ContextPlanSnapshot {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.request_sha256, "request_sha256")?;
        validate_sha256(&self.authorization_sha256, "authorization_sha256")?;
        self.read.validate()?;
        validate_sha256(&self.plan_sha256, "plan_sha256")?;
        if self.stages.len() != 5 {
            return invalid("context plan must contain every canonical stage exactly once");
        }
        let expected = [
            ContextPlanStageKind::Seed,
            ContextPlanStageKind::Lexical,
            ContextPlanStageKind::Semantic,
            ContextPlanStageKind::Graph,
            ContextPlanStageKind::Fusion,
        ];
        for (stage, expected_kind) in self.stages.iter().zip(expected) {
            stage.validate()?;
            if stage.kind != expected_kind {
                return invalid("context plan stages are missing, duplicated, or out of order");
            }
        }
        if self.stages[4].status != ContextPlanStageStatus::Selected {
            return invalid("context fusion stage must be selected");
        }
        if self.plan_sha256 != context_plan_sha256(self)? {
            return invalid("context plan digest does not match its content");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPacket {
    pub scope: String,
    pub valid_at: u64,
    pub query_sha256: String,
    pub read: ContextReadStamp,
    pub plan: ContextPlanSnapshot,
    pub selected_versions: u64,
    pub read_evidence: ReadEvidence,
    pub items: Vec<ContextItem>,
    pub output_bytes: u64,
    pub truncated: bool,
    pub packet_sha256: String,
}

impl ContextPacket {
    pub fn validate(&self) -> Result<()> {
        if self.scope.is_empty() || self.scope.len() > 256 || self.scope.as_bytes().contains(&0) {
            return invalid("context packet scope is invalid");
        }
        if self.valid_at == 0 {
            return invalid("context packet valid_at must be greater than zero");
        }
        validate_sha256(&self.query_sha256, "query_sha256")?;
        self.read.validate()?;
        self.plan.validate()?;
        self.read_evidence.validate()?;
        if self.plan.read != self.read {
            return invalid("context plan and packet read stamps differ");
        }
        validate_sha256(&self.packet_sha256, "packet_sha256")?;
        if self.items.len() > MAX_CONTEXT_ITEMS as usize
            || self.output_bytes > MAX_CONTEXT_OUTPUT_BYTES
        {
            return invalid("context packet exceeds its item or output bound");
        }
        for item in &self.items {
            item.validate()?;
        }
        if self.items.windows(2).any(|pair| {
            pair[0].score < pair[1].score
                || (pair[0].score == pair[1].score && pair[0].identity >= pair[1].identity)
        }) {
            return invalid("context packet items are not in deterministic score order");
        }
        let identities = self
            .items
            .iter()
            .map(|item| item.identity.as_str())
            .collect::<BTreeSet<_>>();
        if identities.len() != self.items.len() {
            return invalid("context packet item identities must be unique");
        }
        let encoded_bytes = self.items.iter().try_fold(0_u64, |total, item| {
            let encoded = serde_json::to_vec(item)
                .map_err(|error| crate::ContractError(error.to_string()))?;
            let length = u64::try_from(encoded.len())
                .map_err(|_| crate::ContractError("context item bytes exceed u64".into()))?;
            total
                .checked_add(length)
                .ok_or_else(|| crate::ContractError("context packet byte count overflowed".into()))
        })?;
        if self.output_bytes != encoded_bytes {
            return invalid("context packet output_bytes differs from its encoded items");
        }
        if self.packet_sha256 != context_packet_sha256(self)? {
            return invalid("context packet digest does not match its content");
        }
        Ok(())
    }
}

pub fn context_packet_sha256(packet: &ContextPacket) -> Result<String> {
    let bytes = serde_json::to_vec(&(
        &packet.scope,
        packet.valid_at,
        &packet.query_sha256,
        &packet.read,
        &packet.plan,
        packet.selected_versions,
        &packet.read_evidence,
        &packet.items,
        packet.output_bytes,
        packet.truncated,
    ))
    .map_err(|error| crate::ContractError(error.to_string()))?;
    Ok(crate::sha256_bytes(&bytes))
}

pub fn context_request_sha256(request: &AssembleContext) -> Result<String> {
    let bytes =
        serde_json::to_vec(request).map_err(|error| crate::ContractError(error.to_string()))?;
    Ok(crate::sha256_bytes(&bytes))
}

pub fn context_plan_stage_sha256(stage: &ContextPlanStage) -> Result<String> {
    let bytes = serde_json::to_vec(&(
        stage.kind,
        stage.status,
        stage.access_path,
        stage.exact,
        &stage.reason,
    ))
    .map_err(|error| crate::ContractError(error.to_string()))?;
    Ok(crate::sha256_bytes(&bytes))
}

pub fn context_plan_sha256(plan: &ContextPlanSnapshot) -> Result<String> {
    let bytes = serde_json::to_vec(&(
        &plan.request_sha256,
        plan.security_policy_revision,
        &plan.authorization_sha256,
        &plan.read,
        &plan.stages,
    ))
    .map_err(|error| crate::ContractError(error.to_string()))?;
    Ok(crate::sha256_bytes(&bytes))
}
