//! Explicit, durably traced `rrflowQL -> RrdEngine -> execution` orchestration.
//!
//! Connectome's GET query lens remains read-only. Operator and MCP execution
//! use this boundary when the query itself should become optimization evidence.
//! The read stamp is captured before the first trace write, preventing
//! observability from changing the meaning of `KNOWN HEAD`.

use super::{DurableTraceSpan, TraceIdentity};
use rrflow_core::{
    digest, Millis, RuntimeProperties, RuntimeValue, ScopeId, TraceAttribute, TraceBoundary,
    TraceDataClass, TraceLink, TraceOutcome,
};
use rrflow_db::{PhysicalStoreEvidence, StorageEngine};
use rrflow_ql::{BoundQuery, PhysicalPlan, QueryExecution, StampedQueryPipeline};
pub use rrflow_ql::{ExecutionBudget, Parameters};
use rrflow_ql::{Query, Source, QUERY_CONTRACT_VERSION};
use serde::Serialize;
use serde_json::Value;

mod attributes;
mod execution;

use attributes::*;
pub use execution::*;

#[cfg(test)]
mod tests;
