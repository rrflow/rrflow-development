//! Fusion constants and context candidate, text-source and graph-edge types.

use super::*;

pub(super) const RRF_RANK_CONSTANT: f64 = 60.0;
pub(super) const SEED_WEIGHT: f64 = 2.0;
pub(super) const TEXT_WEIGHT: f64 = 1.0;
pub(super) const VECTOR_WEIGHT: f64 = 1.0;
pub(super) const GRAPH_WEIGHT: f64 = 0.5;

#[derive(Debug)]
pub(super) struct ContextCandidate {
    pub(super) values: BTreeMap<String, QueryValue>,
    pub(super) evidence: Vec<ContextEvidence>,
}

#[derive(Debug)]
pub(super) struct TextSource {
    pub(super) graph_root: Option<RuntimeRef>,
    pub(super) evidence_source: String,
    pub(super) text: String,
    pub(super) values: BTreeMap<String, QueryValue>,
}

#[derive(Debug, Clone)]
pub(super) struct GraphEdge {
    pub(super) relation: RuntimeRef,
    pub(super) neighbor: RuntimeRef,
}

pub(super) type VectorContextOutcome = (Vec<(RuntimeRef, u64)>, bool, usize);
