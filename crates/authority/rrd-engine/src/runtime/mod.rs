//! Engine-internal execution primitives.
//!
//! Public product operations belong on [`crate::RrdEngine`]. This module only
//! contains reusable operators that do not own a second lifecycle, policy, or
//! persistence authority.

pub mod cluster_transfer;
pub mod data_plane;
pub mod instance;
pub mod operator_knowledge;
pub mod query;
mod read_evidence;
pub mod trace;
pub mod vector_catalog;
pub mod vector_residency;
pub use cluster_transfer::{
    execute_traced_artifact_transfer, record_artifact_transfer_observation,
    DurableArtifactTransferObserver,
};
pub use data_plane::{
    execute_traced_embedding, execute_traced_vector_search, TracedEmbeddingExecution,
    TracedVectorSearch,
};
pub use instance::{
    InstanceBinding, InstanceManifest, InstanceMode, ProjectAuthorityBinding, INSTANCE_FILE,
    INSTANCE_FORMAT, PROJECT_AUTHORITY_FORMAT, STORE_DIR,
};
pub use operator_knowledge::{
    execute_traced_operator_search, execute_traced_operator_sync, TracedOperatorSearch,
    TracedOperatorSync,
};
pub use query::{
    execute_traced_query, query_parameters_from_json, ExecutionBudget, Parameters,
    TracedQueryExecution,
};
pub(crate) use read_evidence::{merge_read_evidence, public_read_evidence};
pub use trace::{
    install_runtime_trace_contract, record_runtime_trace, DurableTraceSpan, TraceIdentity,
};
pub(crate) use vector_catalog::vector_artifact_catalog_entries_from_changes;
pub use vector_catalog::{
    build_traced_quantization_artifact, publish_traced_vector_artifact,
    publish_traced_vector_artifact_with_evidence, quantization_artifact_catalogue,
    reopen_vector_runtime, reopen_vector_runtime_metadata, transition_traced_quantization_artifact,
    vector_artifact_catalog_entries, QuantizationArtifactPublication,
    QuantizationArtifactTransition, VectorArtifactBinding, VectorArtifactPublication,
    VectorArtifactResidencyKey, VectorRuntimeManifest,
};
pub use vector_residency::{
    VectorResidencyAcquisition, VectorResidencyError, VectorResidencyLimits,
    VectorResidencyManager, VectorResidencySnapshot, VectorResidencySource,
    DEFAULT_VECTOR_CACHED_BYTES, DEFAULT_VECTOR_PINNED_BYTES,
};
