//! Every system the lab knows, and what each can do. Adding a system adds one
//! entry here; the runner and the catalog read capabilities from it.
use crate::adapter::{ConnectionLimits, SystemAdapter};
use crate::adapters::{
    custom::CustomAdapter, milvus::MilvusAdapter, qdrant::QdrantAdapter, rrflow::RrflowAdapter,
    surrealdb::SurrealAdapter,
};
use crate::binding::{SystemBinding, SystemKind};
use rrflow_evaluation_core::Result;

/// What a system can be tested on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Collections of vectors, ingest, and exact or approximate search.
    VectorSearch,
}

type ConnectVector = fn(&SystemBinding, &ConnectionLimits) -> Result<Box<dyn SystemAdapter>>;

/// One system's entry: its kind, what it can do, and how to connect to it.
pub struct SystemEntry {
    pub kind: SystemKind,
    pub capabilities: &'static [Capability],
    connect_vector: ConnectVector,
}

pub const SYSTEMS: &[SystemEntry] = &[
    SystemEntry {
        kind: SystemKind::Rrflow,
        capabilities: &[Capability::VectorSearch],
        connect_vector: |binding, limits| Ok(Box::new(RrflowAdapter::connect(binding, limits)?)),
    },
    SystemEntry {
        kind: SystemKind::Qdrant,
        capabilities: &[Capability::VectorSearch],
        connect_vector: |binding, limits| Ok(Box::new(QdrantAdapter::new(binding, limits))),
    },
    SystemEntry {
        kind: SystemKind::Surrealdb,
        capabilities: &[Capability::VectorSearch],
        connect_vector: |binding, limits| Ok(Box::new(SurrealAdapter::new(binding, limits)?)),
    },
    SystemEntry {
        kind: SystemKind::Milvus,
        capabilities: &[Capability::VectorSearch],
        connect_vector: |binding, limits| Ok(Box::new(MilvusAdapter::new(binding, limits))),
    },
    SystemEntry {
        kind: SystemKind::Custom,
        capabilities: &[Capability::VectorSearch],
        connect_vector: |binding, limits| Ok(Box::new(CustomAdapter::new(binding, limits))),
    },
];

/// The registry entry for `kind`. Every kind has exactly one.
pub fn entry(kind: SystemKind) -> &'static SystemEntry {
    SYSTEMS
        .iter()
        .find(|entry| entry.kind == kind)
        .expect("every system kind has a registry entry")
}

/// Connects to `binding`'s system for vector search.
pub fn connect(
    binding: &SystemBinding,
    limits: &ConnectionLimits,
) -> Result<Box<dyn SystemAdapter>> {
    (entry(binding.kind).connect_vector)(binding, limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_system_kind_has_exactly_one_entry() {
        for kind in [
            SystemKind::Rrflow,
            SystemKind::Qdrant,
            SystemKind::Surrealdb,
            SystemKind::Milvus,
            SystemKind::Custom,
        ] {
            assert_eq!(SYSTEMS.iter().filter(|entry| entry.kind == kind).count(), 1);
        }
        assert_eq!(SYSTEMS.len(), 5);
    }
}
