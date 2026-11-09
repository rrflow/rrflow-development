//! Process-local RRFlowCLD trigger observations.
//!
//! RRFlowCLD is a lifecycle trigger surface: canonical vector/query/storage
//! authority remains in engine state, rrflowKV, and immutable object receipts.

use crate::VectorResidencySource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RrflowCldSnapshot {
    pub trigger_count: u64,
    pub cold_source_hits: u64,
    pub cold_mapped_hits: u64,
    pub cold_owned_hits: u64,
    pub cache_bypass_hits: u64,
    pub last_trigger_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RrflowCldLifecycleState {
    trigger_count: u64,
    cold_source_hits: u64,
    cold_mapped_hits: u64,
    cold_owned_hits: u64,
    cache_bypass_hits: u64,
    last_trigger_at_unix_ms: Option<u64>,
}

impl RrflowCldLifecycleState {
    pub fn record_vector_residency_source(&mut self, source: VectorResidencySource, at: u64) {
        if !source.is_cold_source() {
            return;
        }
        self.trigger_count = self.trigger_count.saturating_add(1);
        self.cold_source_hits = self.cold_source_hits.saturating_add(1);
        match source {
            VectorResidencySource::ColdMapped => {
                self.cold_mapped_hits = self.cold_mapped_hits.saturating_add(1);
            }
            VectorResidencySource::ColdOwned => {
                self.cold_owned_hits = self.cold_owned_hits.saturating_add(1);
            }
            VectorResidencySource::CacheBypassMapped | VectorResidencySource::CacheBypassOwned => {
                self.cache_bypass_hits = self.cache_bypass_hits.saturating_add(1);
            }
            VectorResidencySource::PinnedHit
            | VectorResidencySource::CachedHit
            | VectorResidencySource::PinnedLoad
            | VectorResidencySource::CachedLoad => {}
        }
        self.last_trigger_at_unix_ms = Some(at);
    }

    pub fn snapshot(&self) -> RrflowCldSnapshot {
        RrflowCldSnapshot {
            trigger_count: self.trigger_count,
            cold_source_hits: self.cold_source_hits,
            cold_mapped_hits: self.cold_mapped_hits,
            cold_owned_hits: self.cold_owned_hits,
            cache_bypass_hits: self.cache_bypass_hits,
            last_trigger_at_unix_ms: self.last_trigger_at_unix_ms,
        }
    }
}
