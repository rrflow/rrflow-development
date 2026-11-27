//! Durable projection/vector/embedding execution evidence.
//!
//! High-volume physical counters are sampled around one complete logical
//! operation. Raw vectors, embedding inputs, and filter values never enter the
//! trace contract; their caller-owned requests are represented by digests.

use super::{DurableTraceSpan, TraceIdentity};
use rrflow_core::{
    digest, DataTransaction, Millis, ProjectionFamily, ReadStamp, RuntimeCommit,
    RuntimeCommitOutcome, RuntimeMutation, RuntimeProperties, RuntimeSchemaRegistry,
    RuntimeTraceEvent, RuntimeValue, TraceBoundary, TraceDataClass, TraceLink, TraceOutcome,
    RUNTIME_TRACE_EVENT_TYPE,
};
use rrflow_db::{RuntimeReadBudget, StorageEngine};
use rrflow_inference::{
    EmbeddingBackend, EmbeddingBackendDescriptor, EmbeddingCoordinator, EmbeddingJob,
    EmbeddingRequest, EmbeddingSourceReader, ExecutionTarget, PreparedEmbedding,
};
use rrflow_ql::Catalog;
use rrflow_vector::{
    AccessPathKind, PreparedVectorSearch, ScoreMetric, SearchExecution, SearchMode, SearchRequest,
    VectorQuery, VectorRuntime,
};
use serde::Serialize;

mod attributes;
mod execution;

use attributes::*;
pub use execution::*;
