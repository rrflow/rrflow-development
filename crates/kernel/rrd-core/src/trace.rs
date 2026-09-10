//! Persisted, replayable runtime tracing contract.
//!
//! `tracing` spans remain cheap process diagnostics. This contract is the
//! durable causal evidence that Connectome may freeze, replay, compare, and
//! correlate with committed runtime truth. Callers provide clocks and IDs so
//! tests and replays remain deterministic.

use crate::{
    AuditDecision, Error, Millis, ProjectionStamp, ReadStamp, ReasoningActiveCursor, Result,
    RuntimeCommit, RuntimeEvent, RuntimeEventSchema, RuntimeLogicalModel, RuntimeMutation,
    RuntimeProperties, RuntimePropertySchema, RuntimeSchemaRegistry, RuntimeType, RuntimeValue,
    RuntimeValueType, ScopeId, SnapshotId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

pub const RUNTIME_TRACE_CONTRACT_VERSION: u16 = 1;
pub const RUNTIME_TRACE_EVENT_TYPE: &str = "runtime_trace";

const MAX_TRACE_NAME_BYTES: usize = 160;
const MAX_TRACE_LINKS: usize = 32;
const MAX_TRACE_ATTRIBUTES: usize = 64;
const MAX_TRACE_VALUE_DEPTH: usize = 8;
const MAX_TRACE_VALUE_NODES: usize = 1_024;
const MAX_TRACE_STRING_BYTES: usize = 8 * 1_024;

macro_rules! hex_ident {
    ($name:ident, $bytes:literal, $label:literal) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                validate_hex_identity($label, &value, $bytes)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(value: String) -> Result<Self> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

// W3C Trace Context widths: 16-byte trace ID and 8-byte parent/span ID.
hex_ident!(TraceId, 32, "trace id");
hex_ident!(SpanId, 16, "span id");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracePhase {
    Start,
    Annotation,
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceOutcome {
    Running,
    Ok,
    Error,
    Denied,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceDataClass {
    /// Digests, counts, timings, cursors, and other kernel control evidence.
    Control,
    /// Project/operator-authored knowledge safe under the instance policy.
    Operator,
    /// Prompt, document, model, or tool content requiring explicit retention.
    Content,
}

/// The fixed first-level operation boundary used by every RRFlow trace.
///
/// A boundary identifies the cooperating engine layer that performed the
/// work. It is not a product, process, storage authority, or lifecycle owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceBoundary {
    Ingress,
    Engine,
    Kv,
    Ql,
    Graph,
    Lexical,
    Vector,
    #[serde(rename = "datafusion")]
    DataFusion,
    Inference,
    Attunement,
    Routine,
    Adapter,
    Delivery,
}

impl TraceBoundary {
    pub const ALL: [Self; 13] = [
        Self::Ingress,
        Self::Engine,
        Self::Kv,
        Self::Ql,
        Self::Graph,
        Self::Lexical,
        Self::Vector,
        Self::DataFusion,
        Self::Inference,
        Self::Attunement,
        Self::Routine,
        Self::Adapter,
        Self::Delivery,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ingress => "ingress",
            Self::Engine => "engine",
            Self::Kv => "kv",
            Self::Ql => "ql",
            Self::Graph => "graph",
            Self::Lexical => "lexical",
            Self::Vector => "vector",
            Self::DataFusion => "datafusion",
            Self::Inference => "inference",
            Self::Attunement => "attunement",
            Self::Routine => "routine",
            Self::Adapter => "adapter",
            Self::Delivery => "delivery",
        }
    }
}

/// Closed low-cardinality operation catalogue for new instrumentation.
///
/// Dynamic record, route, provider, model, project, query, and error values
/// belong in typed links or bounded attributes. Adding an operation is an
/// explicit contract change; callers cannot manufacture operation names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraceOperation {
    IngressRequest,
    IngressFrame,
    EngineOperation,
    EngineAuthenticate,
    EngineAuthorize,
    EngineValidate,
    EngineCommit,
    EngineContextAssemble,
    EngineFunctionExecute,
    KvPointRead,
    KvRangeScan,
    KvWriteBatch,
    KvWalAppend,
    KvSnapshot,
    KvPageScan,
    KvFlush,
    KvCompact,
    KvRecover,
    QlParse,
    QlBind,
    QlPlan,
    QlExecute,
    GraphMaintain,
    GraphTraverse,
    LexicalMaintain,
    LexicalSearch,
    VectorMaintain,
    VectorSearch,
    VectorRerank,
    VectorProjection,
    DataFusionPlan,
    DataFusionScan,
    DataFusionExecute,
    DataFusionSpill,
    InferenceEmbed,
    InferenceRoute,
    AttunementJob,
    AttunementPhase,
    RoutineTrigger,
    RoutineActivation,
    RoutineStep,
    RoutineActivity,
    RoutineCompensation,
    AdapterResolve,
    AdapterInvoke,
    AdapterSynchronize,
    AdapterTransfer,
    DeliveryConnect,
    DeliveryPublish,
    DeliveryAcknowledge,
    DeliveryHeartbeat,
    DeliveryResume,
    DeliveryClose,
}

impl TraceOperation {
    pub const ALL: [Self; 53] = [
        Self::IngressRequest,
        Self::IngressFrame,
        Self::EngineOperation,
        Self::EngineAuthenticate,
        Self::EngineAuthorize,
        Self::EngineValidate,
        Self::EngineCommit,
        Self::EngineContextAssemble,
        Self::EngineFunctionExecute,
        Self::KvPointRead,
        Self::KvRangeScan,
        Self::KvWriteBatch,
        Self::KvWalAppend,
        Self::KvSnapshot,
        Self::KvPageScan,
        Self::KvFlush,
        Self::KvCompact,
        Self::KvRecover,
        Self::QlParse,
        Self::QlBind,
        Self::QlPlan,
        Self::QlExecute,
        Self::GraphMaintain,
        Self::GraphTraverse,
        Self::LexicalMaintain,
        Self::LexicalSearch,
        Self::VectorMaintain,
        Self::VectorSearch,
        Self::VectorRerank,
        Self::VectorProjection,
        Self::DataFusionPlan,
        Self::DataFusionScan,
        Self::DataFusionExecute,
        Self::DataFusionSpill,
        Self::InferenceEmbed,
        Self::InferenceRoute,
        Self::AttunementJob,
        Self::AttunementPhase,
        Self::RoutineTrigger,
        Self::RoutineActivation,
        Self::RoutineStep,
        Self::RoutineActivity,
        Self::RoutineCompensation,
        Self::AdapterResolve,
        Self::AdapterInvoke,
        Self::AdapterSynchronize,
        Self::AdapterTransfer,
        Self::DeliveryConnect,
        Self::DeliveryPublish,
        Self::DeliveryAcknowledge,
        Self::DeliveryHeartbeat,
        Self::DeliveryResume,
        Self::DeliveryClose,
    ];

    pub const fn boundary(self) -> TraceBoundary {
        match self {
            Self::IngressRequest | Self::IngressFrame => TraceBoundary::Ingress,
            Self::EngineOperation
            | Self::EngineAuthenticate
            | Self::EngineAuthorize
            | Self::EngineValidate
            | Self::EngineCommit
            | Self::EngineContextAssemble
            | Self::EngineFunctionExecute => TraceBoundary::Engine,
            Self::KvPointRead
            | Self::KvRangeScan
            | Self::KvWriteBatch
            | Self::KvWalAppend
            | Self::KvSnapshot
            | Self::KvPageScan
            | Self::KvFlush
            | Self::KvCompact
            | Self::KvRecover => TraceBoundary::Kv,
            Self::QlParse | Self::QlBind | Self::QlPlan | Self::QlExecute => TraceBoundary::Ql,
            Self::GraphMaintain | Self::GraphTraverse => TraceBoundary::Graph,
            Self::LexicalMaintain | Self::LexicalSearch => TraceBoundary::Lexical,
            Self::VectorMaintain
            | Self::VectorSearch
            | Self::VectorRerank
            | Self::VectorProjection => TraceBoundary::Vector,
            Self::DataFusionPlan
            | Self::DataFusionScan
            | Self::DataFusionExecute
            | Self::DataFusionSpill => TraceBoundary::DataFusion,
            Self::InferenceEmbed | Self::InferenceRoute => TraceBoundary::Inference,
            Self::AttunementJob | Self::AttunementPhase => TraceBoundary::Attunement,
            Self::RoutineTrigger
            | Self::RoutineActivation
            | Self::RoutineStep
            | Self::RoutineActivity
            | Self::RoutineCompensation => TraceBoundary::Routine,
            Self::AdapterResolve
            | Self::AdapterInvoke
            | Self::AdapterSynchronize
            | Self::AdapterTransfer => TraceBoundary::Adapter,
            Self::DeliveryConnect
            | Self::DeliveryPublish
            | Self::DeliveryAcknowledge
            | Self::DeliveryHeartbeat
            | Self::DeliveryResume
            | Self::DeliveryClose => TraceBoundary::Delivery,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IngressRequest => "rrflow.ingress.request",
            Self::IngressFrame => "rrflow.ingress.frame",
            Self::EngineOperation => "rrflow.engine.operation",
            Self::EngineAuthenticate => "rrflow.engine.authenticate",
            Self::EngineAuthorize => "rrflow.engine.authorize",
            Self::EngineValidate => "rrflow.engine.validate",
            Self::EngineCommit => "rrflow.engine.commit",
            Self::EngineContextAssemble => "rrflow.engine.context_assemble",
            Self::EngineFunctionExecute => "rrflow.engine.function_execute",
            Self::KvPointRead => "rrflow.kv.point_read",
            Self::KvRangeScan => "rrflow.kv.range_scan",
            Self::KvWriteBatch => "rrflow.kv.write_batch",
            Self::KvWalAppend => "rrflow.kv.wal_append",
            Self::KvSnapshot => "rrflow.kv.snapshot",
            Self::KvPageScan => "rrflow.kv.page_scan",
            Self::KvFlush => "rrflow.kv.flush",
            Self::KvCompact => "rrflow.kv.compact",
            Self::KvRecover => "rrflow.kv.recover",
            Self::QlParse => "rrflow.ql.parse",
            Self::QlBind => "rrflow.ql.bind",
            Self::QlPlan => "rrflow.ql.plan",
            Self::QlExecute => "rrflow.ql.execute",
            Self::GraphMaintain => "rrflow.graph.maintain",
            Self::GraphTraverse => "rrflow.graph.traverse",
            Self::LexicalMaintain => "rrflow.lexical.maintain",
            Self::LexicalSearch => "rrflow.lexical.search",
            Self::VectorMaintain => "rrflow.vector.maintain",
            Self::VectorSearch => "rrflow.vector.search",
            Self::VectorRerank => "rrflow.vector.rerank",
            Self::VectorProjection => "rrflow.vector.projection",
            Self::DataFusionPlan => "rrflow.datafusion.plan",
            Self::DataFusionScan => "rrflow.datafusion.scan",
            Self::DataFusionExecute => "rrflow.datafusion.execute",
            Self::DataFusionSpill => "rrflow.datafusion.spill",
            Self::InferenceEmbed => "rrflow.inference.embed",
            Self::InferenceRoute => "rrflow.inference.route",
            Self::AttunementJob => "rrflow.attunement.job",
            Self::AttunementPhase => "rrflow.attunement.phase",
            Self::RoutineTrigger => "rrflow.routine.trigger",
            Self::RoutineActivation => "rrflow.routine.activation",
            Self::RoutineStep => "rrflow.routine.step",
            Self::RoutineActivity => "rrflow.routine.activity",
            Self::RoutineCompensation => "rrflow.routine.compensation",
            Self::AdapterResolve => "rrflow.adapter.resolve",
            Self::AdapterInvoke => "rrflow.adapter.invoke",
            Self::AdapterSynchronize => "rrflow.adapter.synchronize",
            Self::AdapterTransfer => "rrflow.adapter.transfer",
            Self::DeliveryConnect => "rrflow.delivery.connect",
            Self::DeliveryPublish => "rrflow.delivery.publish",
            Self::DeliveryAcknowledge => "rrflow.delivery.acknowledge",
            Self::DeliveryHeartbeat => "rrflow.delivery.heartbeat",
            Self::DeliveryResume => "rrflow.delivery.resume",
            Self::DeliveryClose => "rrflow.delivery.close",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|operation| operation.as_str() == value)
    }
}

impl fmt::Display for TraceOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<TraceOperation> for String {
    fn from(value: TraceOperation) -> Self {
        value.as_str().into()
    }
}

/// Closed names for bounded attributes added by canonical instrumentation.
///
/// Stable identities and revisions belong in [`TraceLink`]. These names are
/// reserved for bounded measurements and decisions shared across engine
/// boundaries. Existing pre-convergence producers are admitted only through
/// the finite private inventory below and must converge with their behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraceAttribute {
    Stage,
    Attempt,
    Phase,
    Action,
    Selected,
    SkipReason,
    Exact,
    Fallback,
    Truncated,
    CacheStatus,
    InputItems,
    InputBytes,
    MaxStorageKeys,
    OutputItems,
    OutputBytes,
    SelectedVersions,
    KeysExamined,
    PagesExamined,
    RowsExamined,
    ReadPointReads,
    ReadRangeScans,
    ReadValuesDecoded,
    ReadStampValidation,
    ReadStampValidationChangeReads,
    ReadStampValidationProofNodes,
    GraphSteps,
    CandidatesExamined,
    MappedBytes,
    ReadBytes,
    DecodedBytes,
    DecompressedBytes,
    BorrowedBytes,
    CopiedBytes,
    AllocatedBytes,
    ElapsedMicros,
    MemoryPeakBytes,
    SpillBytes,
    InputTokens,
    OutputTokens,
    FailedStage,
    ErrorClass,
    ErrorDigest,
    Retryable,
    Rank,
    Score,
    RrfContribution,
    SelectedSourceCount,
    SkippedSourceCount,
    CompactedBytes,
    CompactedTokens,
    VerificationStatus,
    PropagationStatus,
    PropagationDigest,
    StartCursor,
}

impl TraceAttribute {
    pub const ALL: &'static [Self] = &[
        Self::Stage,
        Self::Attempt,
        Self::Phase,
        Self::Action,
        Self::Selected,
        Self::SkipReason,
        Self::Exact,
        Self::Fallback,
        Self::Truncated,
        Self::CacheStatus,
        Self::InputItems,
        Self::InputBytes,
        Self::MaxStorageKeys,
        Self::OutputItems,
        Self::OutputBytes,
        Self::SelectedVersions,
        Self::KeysExamined,
        Self::PagesExamined,
        Self::RowsExamined,
        Self::ReadPointReads,
        Self::ReadRangeScans,
        Self::ReadValuesDecoded,
        Self::ReadStampValidation,
        Self::ReadStampValidationChangeReads,
        Self::ReadStampValidationProofNodes,
        Self::GraphSteps,
        Self::CandidatesExamined,
        Self::MappedBytes,
        Self::ReadBytes,
        Self::DecodedBytes,
        Self::DecompressedBytes,
        Self::BorrowedBytes,
        Self::CopiedBytes,
        Self::AllocatedBytes,
        Self::ElapsedMicros,
        Self::MemoryPeakBytes,
        Self::SpillBytes,
        Self::InputTokens,
        Self::OutputTokens,
        Self::FailedStage,
        Self::ErrorClass,
        Self::ErrorDigest,
        Self::Retryable,
        Self::Rank,
        Self::Score,
        Self::RrfContribution,
        Self::SelectedSourceCount,
        Self::SkippedSourceCount,
        Self::CompactedBytes,
        Self::CompactedTokens,
        Self::VerificationStatus,
        Self::PropagationStatus,
        Self::PropagationDigest,
        Self::StartCursor,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stage => "stage",
            Self::Attempt => "attempt",
            Self::Phase => "phase",
            Self::Action => "action",
            Self::Selected => "selected",
            Self::SkipReason => "skip_reason",
            Self::Exact => "exact",
            Self::Fallback => "fallback",
            Self::Truncated => "truncated",
            Self::CacheStatus => "cache_status",
            Self::InputItems => "input_items",
            Self::InputBytes => "input_bytes",
            Self::MaxStorageKeys => "max_storage_keys",
            Self::OutputItems => "output_items",
            Self::OutputBytes => "output_bytes",
            Self::SelectedVersions => "selected_versions",
            Self::KeysExamined => "keys_examined",
            Self::PagesExamined => "pages_examined",
            Self::RowsExamined => "rows_examined",
            Self::ReadPointReads => "read_point_reads",
            Self::ReadRangeScans => "read_range_scans",
            Self::ReadValuesDecoded => "read_values_decoded",
            Self::ReadStampValidation => "read_stamp_validation",
            Self::ReadStampValidationChangeReads => "read_stamp_validation_change_reads",
            Self::ReadStampValidationProofNodes => "read_stamp_validation_proof_nodes",
            Self::GraphSteps => "graph_steps",
            Self::CandidatesExamined => "candidates_examined",
            Self::MappedBytes => "mapped_bytes",
            Self::ReadBytes => "read_bytes",
            Self::DecodedBytes => "decoded_bytes",
            Self::DecompressedBytes => "decompressed_bytes",
            Self::BorrowedBytes => "borrowed_bytes",
            Self::CopiedBytes => "copied_bytes",
            Self::AllocatedBytes => "allocated_bytes",
            Self::ElapsedMicros => "elapsed_micros",
            Self::MemoryPeakBytes => "memory_peak_bytes",
            Self::SpillBytes => "spill_bytes",
            Self::InputTokens => "input_tokens",
            Self::OutputTokens => "output_tokens",
            Self::FailedStage => "failed_stage",
            Self::ErrorClass => "error_class",
            Self::ErrorDigest => "error_digest",
            Self::Retryable => "retryable",
            Self::Rank => "rank",
            Self::Score => "score",
            Self::RrfContribution => "rrf_contribution",
            Self::SelectedSourceCount => "selected_source_count",
            Self::SkippedSourceCount => "skipped_source_count",
            Self::CompactedBytes => "compacted_bytes",
            Self::CompactedTokens => "compacted_tokens",
            Self::VerificationStatus => "verification_status",
            Self::PropagationStatus => "propagation_status",
            Self::PropagationDigest => "propagation_digest",
            Self::StartCursor => "start_cursor",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|attribute| attribute.as_str() == value)
    }
}

impl fmt::Display for TraceAttribute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<TraceAttribute> for String {
    fn from(value: TraceAttribute) -> Self {
        value.as_str().into()
    }
}

/// Relationship used when work is causally connected but is not a
/// synchronous child (for example a committed-event activation or retry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceCausalRelation {
    FollowsFrom,
    RetryOf,
    Resumes,
}

/// A typed causal coordinate. These links make a trace useful for correctness
/// and optimization rather than merely a wall-clock visualization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TraceLink {
    Request {
        request_id: String,
        operation_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_request_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        idempotency_digest: Option<String>,
    },
    ActorScope {
        actor: String,
        scope: ScopeId,
    },
    Authorization {
        decision: AuditDecision,
        policy_revision: u64,
        authorization_digest: String,
    },
    /// Correlates work with the exact generic reasoning cursor that selected
    /// it. The cursor owns no implicit lifecycle; its tree declares the route.
    ReasoningCursor {
        cursor: ReasoningActiveCursor,
    },
    RuntimeCursor {
        cursor: u64,
    },
    Read {
        stamp: ReadStamp,
    },
    Snapshot {
        snapshot_id: SnapshotId,
        cursor: u64,
    },
    Plan {
        plan_digest: String,
    },
    Projection {
        stamp: ProjectionStamp,
    },
    Source {
        source_kind: String,
        source_id: String,
        source_revision: String,
    },
    Commit {
        commit_id: String,
        first_cursor: u64,
        last_cursor: u64,
    },
    Resource {
        resource_kind: String,
        resource_id: String,
    },
    CausalSpan {
        trace_id: TraceId,
        span_id: SpanId,
        relation: TraceCausalRelation,
    },
    Output {
        output_digest: String,
        item_count: u64,
        byte_count: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeTraceEvent {
    pub contract_version: u16,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<SpanId>,
    pub phase: TracePhase,
    pub boundary: TraceBoundary,
    pub name: String,
    pub at: Millis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_micros: Option<u64>,
    pub outcome: TraceOutcome,
    pub data_class: TraceDataClass,
    #[serde(default)]
    pub links: Vec<TraceLink>,
    #[serde(default)]
    pub attributes: RuntimeProperties,
}

impl RuntimeTraceEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        trace_id: TraceId,
        span_id: SpanId,
        parent_span_id: Option<SpanId>,
        boundary: TraceBoundary,
        name: impl Into<String>,
        at: Millis,
        data_class: TraceDataClass,
        links: Vec<TraceLink>,
        attributes: RuntimeProperties,
    ) -> Result<Self> {
        Self::new(
            trace_id,
            span_id,
            parent_span_id,
            TracePhase::Start,
            boundary,
            name,
            at,
            None,
            TraceOutcome::Running,
            data_class,
            links,
            attributes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn annotation(
        trace_id: TraceId,
        span_id: SpanId,
        parent_span_id: Option<SpanId>,
        boundary: TraceBoundary,
        name: impl Into<String>,
        at: Millis,
        outcome: TraceOutcome,
        data_class: TraceDataClass,
        links: Vec<TraceLink>,
        attributes: RuntimeProperties,
    ) -> Result<Self> {
        Self::new(
            trace_id,
            span_id,
            parent_span_id,
            TracePhase::Annotation,
            boundary,
            name,
            at,
            None,
            outcome,
            data_class,
            links,
            attributes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finish(
        trace_id: TraceId,
        span_id: SpanId,
        parent_span_id: Option<SpanId>,
        boundary: TraceBoundary,
        name: impl Into<String>,
        at: Millis,
        duration_micros: u64,
        outcome: TraceOutcome,
        data_class: TraceDataClass,
        links: Vec<TraceLink>,
        attributes: RuntimeProperties,
    ) -> Result<Self> {
        Self::new(
            trace_id,
            span_id,
            parent_span_id,
            TracePhase::Finish,
            boundary,
            name,
            at,
            Some(duration_micros),
            outcome,
            data_class,
            links,
            attributes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        trace_id: TraceId,
        span_id: SpanId,
        parent_span_id: Option<SpanId>,
        phase: TracePhase,
        boundary: TraceBoundary,
        name: impl Into<String>,
        at: Millis,
        duration_micros: Option<u64>,
        outcome: TraceOutcome,
        data_class: TraceDataClass,
        links: Vec<TraceLink>,
        attributes: RuntimeProperties,
    ) -> Result<Self> {
        let event = Self {
            contract_version: RUNTIME_TRACE_CONTRACT_VERSION,
            trace_id,
            span_id,
            parent_span_id,
            phase,
            boundary,
            name: name.into(),
            at,
            duration_micros,
            outcome,
            data_class,
            links,
            attributes,
        };
        event.validate()?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<()> {
        if self.contract_version != RUNTIME_TRACE_CONTRACT_VERSION {
            return invalid(format!(
                "unsupported runtime trace contract version {}",
                self.contract_version
            ));
        }
        validate_operation_name(&self.name)?;
        let expected_boundary = TraceOperation::from_name(&self.name)
            .map(TraceOperation::boundary)
            .or_else(|| direct_convergence_operation_boundary(&self.name))
            .expect("validated trace operation must have one frozen boundary");
        if expected_boundary != self.boundary {
            return invalid("trace operation and boundary differ");
        }
        if self.parent_span_id.as_ref() == Some(&self.span_id) {
            return invalid("a trace span cannot parent itself");
        }
        match (self.phase, self.duration_micros, self.outcome) {
            (TracePhase::Start, None, TraceOutcome::Running) => {}
            (TracePhase::Annotation, None, _) => {}
            (TracePhase::Finish, Some(_), outcome) if outcome != TraceOutcome::Running => {}
            _ => {
                return invalid(
                    "trace phase, duration, and outcome do not form a valid lifecycle transition",
                );
            }
        }
        if self.links.len() > MAX_TRACE_LINKS {
            return invalid(format!(
                "runtime trace exceeds {MAX_TRACE_LINKS} causal links"
            ));
        }
        for link in &self.links {
            validate_link(link)?;
            if matches!(
                link,
                TraceLink::CausalSpan { trace_id, span_id, .. }
                    if trace_id == &self.trace_id && span_id == &self.span_id
            ) {
                return invalid("a trace span cannot causally link to itself");
            }
        }
        if self.attributes.len() > MAX_TRACE_ATTRIBUTES {
            return invalid(format!(
                "runtime trace exceeds {MAX_TRACE_ATTRIBUTES} attributes"
            ));
        }
        let mut nodes = 0;
        for (name, value) in &self.attributes {
            validate_attribute_name(name)?;
            if let Some(attribute) = TraceAttribute::from_name(name) {
                validate_canonical_attribute_value(attribute, value)?;
            }
            validate_value(value, 0, &mut nodes)?;
        }
        Ok(())
    }

    /// Converts the trace into the existing append-only runtime event model.
    /// Persistence therefore inherits the runtime cursor and hash-chain rather
    /// than creating an unrelated telemetry store.
    pub fn into_runtime_event(self) -> Result<RuntimeEvent> {
        self.validate()?;
        let mut properties = RuntimeProperties::from([
            (
                "contract_version".into(),
                RuntimeValue::Unsigned(u64::from(self.contract_version)),
            ),
            (
                "trace_id".into(),
                RuntimeValue::String(self.trace_id.to_string()),
            ),
            (
                "span_id".into(),
                RuntimeValue::String(self.span_id.to_string()),
            ),
            (
                "phase".into(),
                RuntimeValue::String(enum_name(self.phase).into()),
            ),
            (
                "boundary".into(),
                RuntimeValue::String(self.boundary.as_str().into()),
            ),
            ("name".into(), RuntimeValue::String(self.name)),
            ("at".into(), RuntimeValue::Unsigned(self.at)),
            (
                "outcome".into(),
                RuntimeValue::String(outcome_name(self.outcome).into()),
            ),
            (
                "data_class".into(),
                RuntimeValue::String(data_class_name(self.data_class).into()),
            ),
            (
                "links".into(),
                RuntimeValue::List(self.links.iter().map(link_value).collect()),
            ),
            ("attributes".into(), RuntimeValue::Map(self.attributes)),
        ]);
        if let Some(parent) = self.parent_span_id {
            properties.insert(
                "parent_span_id".into(),
                RuntimeValue::String(parent.to_string()),
            );
        }
        if let Some(duration) = self.duration_micros {
            properties.insert("duration_micros".into(), RuntimeValue::Unsigned(duration));
        }
        Ok(RuntimeEvent {
            kind: Self::event_type()?,
            subject: None,
            properties,
        })
    }

    pub fn event_type() -> Result<RuntimeType> {
        RuntimeType::new(RUNTIME_TRACE_EVENT_TYPE)
    }

    /// Strict schema entry an instance adds during bootstrap or migration.
    pub fn event_schema() -> RuntimeEventSchema {
        let mut properties = BTreeMap::new();
        for name in ["contract_version", "at"] {
            properties.insert(
                name.into(),
                RuntimePropertySchema::required(RuntimeValueType::Unsigned),
            );
        }
        for name in [
            "trace_id",
            "span_id",
            "phase",
            "boundary",
            "name",
            "outcome",
            "data_class",
        ] {
            properties.insert(
                name.into(),
                RuntimePropertySchema::required(RuntimeValueType::String),
            );
        }
        properties.insert(
            "parent_span_id".into(),
            RuntimePropertySchema::optional(RuntimeValueType::String),
        );
        properties.insert(
            "duration_micros".into(),
            RuntimePropertySchema::optional(RuntimeValueType::Unsigned),
        );
        properties.insert(
            "links".into(),
            RuntimePropertySchema::required(RuntimeValueType::List),
        );
        properties.insert(
            "attributes".into(),
            RuntimePropertySchema::required(RuntimeValueType::Map),
        );
        RuntimeEventSchema {
            properties,
            ..RuntimeEventSchema::default()
        }
    }

    /// Installs or repairs the canonical trace event declaration in a schema
    /// under construction. The caller owns schema revision allocation because
    /// that compare-and-swap must occur against its storage engine's exact
    /// read stamp.
    pub fn register_schema(registry: &mut RuntimeSchemaRegistry) -> Result<bool> {
        let kind = Self::event_type()?;
        let schema = Self::event_schema();
        registry.define_event_table(kind, RuntimeLogicalModel::Event, schema)
    }

    /// Prepares the one canonical runtime commit used by both local engines
    /// and consensus-backed trace writers. Schema installation/repair and the
    /// event share the caller's exact read cursor CAS.
    pub fn prepare_commit(
        &self,
        read: &ReadStamp,
        current_schema: Option<&RuntimeSchemaRegistry>,
        actor: &str,
    ) -> Result<RuntimeCommit> {
        self.validate()?;
        read.validate()?;
        if actor.trim().is_empty() {
            return invalid("runtime trace actor must not be empty");
        }
        if current_schema.map(|schema| schema.revision) != read.schema_revision {
            return invalid("runtime trace schema does not match its read stamp");
        }

        let mut mutations = Vec::new();
        let mut registry = current_schema
            .cloned()
            .unwrap_or_else(|| RuntimeSchemaRegistry::empty(1, "install runtime trace contract"));
        if Self::register_schema(&mut registry)? {
            if let Some(current) = current_schema {
                registry.revision =
                    current
                        .revision
                        .checked_add(1)
                        .ok_or_else(|| Error::InvalidRuntime {
                            reason:
                                "runtime schema revision overflow while installing trace contract"
                                    .into(),
                        })?;
                registry.migration = "install canonical runtime trace contract".into();
            }
            mutations.push(RuntimeMutation::Schema { registry });
        }
        mutations.push(RuntimeMutation::Event {
            event: self.clone().into_runtime_event()?,
        });
        let commit = RuntimeCommit {
            scope: read.scope.clone(),
            at: self.at,
            actor: actor.to_owned(),
            expected_cursor: read.commit_cursor,
            mutations,
        };
        commit.validate()?;
        Ok(commit)
    }
}

fn validate_hex_identity(kind: &'static str, value: &str, width: usize) -> Result<()> {
    if value.len() != width
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        || value.bytes().all(|byte| byte == b'0')
    {
        return invalid(format!(
            "{kind} must be {width} lowercase hexadecimal characters and not all zero"
        ));
    }
    Ok(())
}

fn validate_link(link: &TraceLink) -> Result<()> {
    match link {
        TraceLink::Request {
            request_id,
            operation_id,
            parent_request_id,
            idempotency_digest,
        } => {
            validate_bounded_text("trace request id", request_id, MAX_TRACE_STRING_BYTES)?;
            validate_bounded_text(
                "trace request operation",
                operation_id,
                MAX_TRACE_NAME_BYTES,
            )?;
            if let Some(parent_request_id) = parent_request_id {
                validate_bounded_text(
                    "trace parent request id",
                    parent_request_id,
                    MAX_TRACE_STRING_BYTES,
                )?;
            }
            if let Some(idempotency_digest) = idempotency_digest {
                validate_digest("trace idempotency", idempotency_digest)?;
            }
            Ok(())
        }
        TraceLink::ActorScope { actor, scope } => {
            validate_bounded_text("trace actor", actor, MAX_TRACE_STRING_BYTES)?;
            validate_bounded_text("trace scope", scope.as_str(), MAX_TRACE_STRING_BYTES)
        }
        TraceLink::Authorization {
            policy_revision,
            authorization_digest,
            ..
        } => {
            if *policy_revision == 0 {
                return invalid("trace authorization policy revision must be nonzero");
            }
            validate_digest("trace authorization", authorization_digest)
        }
        TraceLink::ReasoningCursor { cursor } => cursor.validate(),
        TraceLink::RuntimeCursor { .. } => Ok(()),
        TraceLink::Read { stamp } => stamp.validate(),
        TraceLink::Snapshot { snapshot_id, .. } => validate_bounded_text(
            "trace snapshot id",
            snapshot_id.as_str(),
            MAX_TRACE_STRING_BYTES,
        ),
        TraceLink::Plan { plan_digest } => validate_digest("trace plan", plan_digest),
        TraceLink::Projection { stamp } => stamp.validate(),
        TraceLink::Source {
            source_kind,
            source_id,
            source_revision,
        } => {
            validate_bounded_text("trace source kind", source_kind, MAX_TRACE_NAME_BYTES)?;
            validate_bounded_text("trace source id", source_id, MAX_TRACE_STRING_BYTES)?;
            validate_bounded_text(
                "trace source revision",
                source_revision,
                MAX_TRACE_STRING_BYTES,
            )
        }
        TraceLink::Commit {
            commit_id,
            first_cursor,
            last_cursor,
        } => {
            validate_digest("trace commit", commit_id)?;
            if *first_cursor == 0 || last_cursor < first_cursor {
                return invalid(
                    "trace commit cursors must be nonzero and ordered first through last",
                );
            }
            Ok(())
        }
        TraceLink::Resource {
            resource_kind,
            resource_id,
        } => {
            validate_bounded_text("trace resource kind", resource_kind, MAX_TRACE_NAME_BYTES)?;
            validate_bounded_text("trace resource id", resource_id, MAX_TRACE_STRING_BYTES)
        }
        TraceLink::CausalSpan { .. } => Ok(()),
        TraceLink::Output { output_digest, .. } => validate_digest("trace output", output_digest),
    }
}

fn validate_value(value: &RuntimeValue, depth: usize, nodes: &mut usize) -> Result<()> {
    *nodes = nodes.saturating_add(1);
    if *nodes > MAX_TRACE_VALUE_NODES || depth > MAX_TRACE_VALUE_DEPTH {
        return invalid("runtime trace attribute tree exceeds its structural budget");
    }
    match value {
        RuntimeValue::String(value)
        | RuntimeValue::Digest(value)
        | RuntimeValue::Decimal(value) => {
            if value.len() > MAX_TRACE_STRING_BYTES {
                return invalid(format!(
                    "runtime trace string exceeds {MAX_TRACE_STRING_BYTES} bytes"
                ));
            }
        }
        RuntimeValue::List(values) => {
            for value in values {
                validate_value(value, depth + 1, nodes)?;
            }
        }
        RuntimeValue::Map(values) => {
            for (name, value) in values {
                validate_bounded_text("trace map key", name, MAX_TRACE_NAME_BYTES)?;
                validate_value(value, depth + 1, nodes)?;
            }
        }
        RuntimeValue::Null
        | RuntimeValue::Bool(_)
        | RuntimeValue::Integer(_)
        | RuntimeValue::Unsigned(_) => {}
    }
    Ok(())
}

fn link_value(link: &TraceLink) -> RuntimeValue {
    let mut value = BTreeMap::new();
    match link {
        TraceLink::Request {
            request_id,
            operation_id,
            parent_request_id,
            idempotency_digest,
        } => {
            value.insert("kind".into(), RuntimeValue::String("request".into()));
            value.insert(
                "request_id".into(),
                RuntimeValue::String(request_id.clone()),
            );
            value.insert(
                "operation_id".into(),
                RuntimeValue::String(operation_id.clone()),
            );
            if let Some(parent_request_id) = parent_request_id {
                value.insert(
                    "parent_request_id".into(),
                    RuntimeValue::String(parent_request_id.clone()),
                );
            }
            if let Some(idempotency_digest) = idempotency_digest {
                value.insert(
                    "idempotency_digest".into(),
                    RuntimeValue::Digest(idempotency_digest.clone()),
                );
            }
        }
        TraceLink::ActorScope { actor, scope } => {
            value.insert("kind".into(), RuntimeValue::String("actor_scope".into()));
            value.insert("actor".into(), RuntimeValue::String(actor.clone()));
            value.insert("scope".into(), RuntimeValue::String(scope.to_string()));
        }
        TraceLink::Authorization {
            decision,
            policy_revision,
            authorization_digest,
        } => {
            value.insert("kind".into(), RuntimeValue::String("authorization".into()));
            value.insert(
                "decision".into(),
                RuntimeValue::String(audit_decision_name(*decision).into()),
            );
            value.insert(
                "policy_revision".into(),
                RuntimeValue::Unsigned(*policy_revision),
            );
            value.insert(
                "authorization_digest".into(),
                RuntimeValue::Digest(authorization_digest.clone()),
            );
        }
        TraceLink::ReasoningCursor { cursor } => {
            value.insert(
                "kind".into(),
                RuntimeValue::String("reasoning_cursor".into()),
            );
            value.insert(
                "cursor_id".into(),
                RuntimeValue::String(cursor.id.to_string()),
            );
            value.insert(
                "tree_id".into(),
                RuntimeValue::String(cursor.tree_id.to_string()),
            );
            value.insert(
                "tree_revision".into(),
                RuntimeValue::Unsigned(cursor.tree_revision),
            );
            value.insert(
                "node_id".into(),
                RuntimeValue::String(cursor.node_id.to_string()),
            );
            value.insert("step".into(), RuntimeValue::Unsigned(cursor.step));
            value.insert(
                "read_manifest_sha256".into(),
                RuntimeValue::Digest(cursor.read.manifest_id.clone()),
            );
        }
        TraceLink::RuntimeCursor { cursor } => {
            value.insert("kind".into(), RuntimeValue::String("runtime_cursor".into()));
            value.insert("cursor".into(), RuntimeValue::Unsigned(*cursor));
        }
        TraceLink::Read { stamp } => {
            value.insert("kind".into(), RuntimeValue::String("read".into()));
            value.insert(
                "contract_version".into(),
                RuntimeValue::Unsigned(u64::from(stamp.contract_version)),
            );
            value.insert(
                "scope".into(),
                RuntimeValue::String(stamp.scope.to_string()),
            );
            if let Some(schema_revision) = stamp.schema_revision {
                value.insert(
                    "schema_revision".into(),
                    RuntimeValue::Unsigned(schema_revision),
                );
            }
            value.insert(
                "catalog_revision".into(),
                RuntimeValue::Unsigned(stamp.catalog_revision),
            );
            value.insert(
                "commit_cursor".into(),
                RuntimeValue::Unsigned(stamp.commit_cursor),
            );
            if let Some(head_digest) = &stamp.head_digest {
                value.insert(
                    "head_digest".into(),
                    RuntimeValue::Digest(head_digest.clone()),
                );
            }
            value.insert(
                "manifest_id".into(),
                RuntimeValue::Digest(stamp.manifest_id.clone()),
            );
        }
        TraceLink::Snapshot {
            snapshot_id,
            cursor,
        } => {
            value.insert("kind".into(), RuntimeValue::String("snapshot".into()));
            value.insert(
                "snapshot_id".into(),
                RuntimeValue::String(snapshot_id.to_string()),
            );
            value.insert("cursor".into(), RuntimeValue::Unsigned(*cursor));
        }
        TraceLink::Plan { plan_digest } => {
            value.insert("kind".into(), RuntimeValue::String("plan".into()));
            value.insert(
                "plan_digest".into(),
                RuntimeValue::Digest(plan_digest.clone()),
            );
        }
        TraceLink::Projection { stamp } => {
            value.insert("kind".into(), RuntimeValue::String("projection".into()));
            value.insert(
                "contract_version".into(),
                RuntimeValue::Unsigned(u64::from(stamp.contract_version)),
            );
            value.insert("id".into(), RuntimeValue::String(stamp.id.to_string()));
            value.insert(
                "generation".into(),
                RuntimeValue::Unsigned(stamp.generation),
            );
            value.insert(
                "source_cursor".into(),
                RuntimeValue::Unsigned(stamp.source_cursor),
            );
            value.insert(
                "config_digest".into(),
                RuntimeValue::Digest(stamp.config_digest.clone()),
            );
            value.insert(
                "artifact_digest".into(),
                RuntimeValue::Digest(stamp.artifact_digest.clone()),
            );
            value.insert(
                "state".into(),
                RuntimeValue::String(projection_state_name(stamp.state).into()),
            );
        }
        TraceLink::Source {
            source_kind,
            source_id,
            source_revision,
        } => {
            value.insert("kind".into(), RuntimeValue::String("source".into()));
            value.insert(
                "source_kind".into(),
                RuntimeValue::String(source_kind.clone()),
            );
            value.insert("source_id".into(), RuntimeValue::String(source_id.clone()));
            value.insert(
                "source_revision".into(),
                RuntimeValue::String(source_revision.clone()),
            );
        }
        TraceLink::Commit {
            commit_id,
            first_cursor,
            last_cursor,
        } => {
            value.insert("kind".into(), RuntimeValue::String("commit".into()));
            value.insert("commit_id".into(), RuntimeValue::Digest(commit_id.clone()));
            value.insert("first_cursor".into(), RuntimeValue::Unsigned(*first_cursor));
            value.insert("last_cursor".into(), RuntimeValue::Unsigned(*last_cursor));
        }
        TraceLink::Resource {
            resource_kind,
            resource_id,
        } => {
            value.insert("kind".into(), RuntimeValue::String("resource".into()));
            value.insert(
                "resource_kind".into(),
                RuntimeValue::String(resource_kind.clone()),
            );
            value.insert(
                "resource_id".into(),
                RuntimeValue::String(resource_id.clone()),
            );
        }
        TraceLink::CausalSpan {
            trace_id,
            span_id,
            relation,
        } => {
            value.insert("kind".into(), RuntimeValue::String("causal_span".into()));
            value.insert(
                "trace_id".into(),
                RuntimeValue::String(trace_id.to_string()),
            );
            value.insert("span_id".into(), RuntimeValue::String(span_id.to_string()));
            value.insert(
                "relation".into(),
                RuntimeValue::String(causal_relation_name(*relation).into()),
            );
        }
        TraceLink::Output {
            output_digest,
            item_count,
            byte_count,
        } => {
            value.insert("kind".into(), RuntimeValue::String("output".into()));
            value.insert(
                "output_digest".into(),
                RuntimeValue::Digest(output_digest.clone()),
            );
            value.insert("item_count".into(), RuntimeValue::Unsigned(*item_count));
            value.insert("byte_count".into(), RuntimeValue::Unsigned(*byte_count));
        }
    }
    RuntimeValue::Map(value)
}

fn enum_name(value: TracePhase) -> &'static str {
    match value {
        TracePhase::Start => "start",
        TracePhase::Annotation => "annotation",
        TracePhase::Finish => "finish",
    }
}

fn outcome_name(value: TraceOutcome) -> &'static str {
    match value {
        TraceOutcome::Running => "running",
        TraceOutcome::Ok => "ok",
        TraceOutcome::Error => "error",
        TraceOutcome::Denied => "denied",
        TraceOutcome::Cancelled => "cancelled",
    }
}

fn data_class_name(value: TraceDataClass) -> &'static str {
    match value {
        TraceDataClass::Control => "control",
        TraceDataClass::Operator => "operator",
        TraceDataClass::Content => "content",
    }
}

fn audit_decision_name(value: AuditDecision) -> &'static str {
    match value {
        AuditDecision::Allow => "allow",
        AuditDecision::Deny => "deny",
    }
}

fn causal_relation_name(value: TraceCausalRelation) -> &'static str {
    match value {
        TraceCausalRelation::FollowsFrom => "follows_from",
        TraceCausalRelation::RetryOf => "retry_of",
        TraceCausalRelation::Resumes => "resumes",
    }
}

fn projection_state_name(value: crate::ProjectionState) -> &'static str {
    match value {
        crate::ProjectionState::Building => "building",
        crate::ProjectionState::Ready => "ready",
        crate::ProjectionState::Quarantined => "quarantined",
        crate::ProjectionState::Retiring => "retiring",
    }
}

fn validate_digest(kind: &'static str, value: &str) -> Result<()> {
    if value.len() != 64 || value.bytes().any(|byte| !byte.is_ascii_hexdigit()) {
        return invalid(format!("{kind} digest must be 64 hexadecimal characters"));
    }
    Ok(())
}

// These names are the exact observed pre-A-07.2 call-site inventory. They are
// accepted only so the owning C-through-I packages can replace the associated
// behavior and trace together. New instrumentation must use `TraceOperation`;
// this list can only shrink.
const DIRECT_CONVERGENCE_OPERATION_NAMES: &[&str] = &[
    "cluster.artifact_chunk",
    "cluster.artifact_transfer",
    "embedding.commit",
    "embedding.infer",
    "embedding.run",
    "object.replicate",
    "operator.knowledge.apply",
    "operator.knowledge.execute",
    "operator.knowledge.search",
    "operator.knowledge.sync",
    "rrflow.query.execute",
    "rrflow.query.parse_bind",
    "rrflow.query.plan",
    "rrflow.query.run",
    "rrflow.storage.runtime_read",
    "vector.execute",
    "vector.plan",
    "vector.projection.publish",
    "vector.quantization.activate",
    "vector.quantization.build",
    "vector.quantization.retire",
    "vector.search",
];

// Exact attribute names emitted by the reviewed trace producers that predate
// the canonical cross-boundary vocabulary. The behavior-owning C-through-I
// package must either map each value to `TraceAttribute`/`TraceLink` or remove
// it. This inventory can only shrink; it is not an extension namespace.
const DIRECT_CONVERGENCE_ATTRIBUTE_NAMES: &[&str] = &[
    "adapter_digest",
    "adapter_elapsed_micros",
    "adapter_implementation_digest",
    "applied_now",
    "approximation_requested",
    "artifact_digest",
    "automatic_compactions_delta",
    "automatic_flushes_delta",
    "backend",
    "backend_digest",
    "batch_count",
    "binding_digest",
    "cache_capacity_bytes",
    "cache_entries",
    "cache_evictions_delta",
    "cache_hits_delta",
    "cache_misses_delta",
    "cache_resident_bytes",
    "canonical_digest",
    "catalog_digest",
    "catalog_entry_digest",
    "catalog_revision",
    "commit_id",
    "compaction_debt_segments",
    "compaction_input_bytes_delta",
    "compaction_output_bytes_delta",
    "compaction_target_segment_bytes",
    "config_digest",
    "contract_version",
    "deterministic",
    "dimensions",
    "distinct_objects",
    "durability",
    "durable_sequence",
    "ef_search",
    "estimated_candidates",
    "estimated_cost",
    "evidence_policy",
    "exact_rerank",
    "execution_target",
    "expected_catalog_revision",
    "expected_length",
    "expected_stable_revision",
    "failed_compactions_delta",
    "failed_maintenance_flushes_delta",
    "fallback_reason_digest",
    "fallback_to_exact",
    "filter_applied_after_ann",
    "filter_checks_delta",
    "filter_count",
    "filter_negatives_delta",
    "filter_present",
    "filter_property_count",
    "filter_selectivity",
    "first_cursor",
    "generation",
    "hit_count",
    "hnsw_ef_search",
    "hnsw_max_scan_tuples",
    "idempotent_replay",
    "index_digest",
    "iterative_scan",
    "ivfflat_max_probes",
    "ivfflat_probes",
    "job_digest",
    "l0_compaction_trigger",
    "l0_segment_count",
    "last_cursor",
    "lifecycle_action",
    "lifecycle_revision",
    "lifecycle_state",
    "logical_operator_count",
    "logical_output_bytes",
    "maintenance_write_stalls_delta",
    "manifest_digest",
    "manifest_generation",
    "max_output_bytes",
    "max_rows",
    "media_type_digest",
    "memtable_bytes",
    "memtable_max_versions",
    "memtable_versions",
    "metric",
    "mode",
    "model_space_digest",
    "mutation_kind",
    "network_required",
    "next_offset",
    "object_backend",
    "object_digest",
    "object_length",
    "object_references",
    "object_sha256",
    "operation",
    "ordering_exact",
    "outcome_cursor",
    "output_dimensions",
    "output_kind",
    "oversized_batches_delta",
    "page_bytes_allocated_delta",
    "page_bytes_borrowed_delta",
    "page_bytes_copied_delta",
    "page_bytes_decoded_delta",
    "page_bytes_decompressed_delta",
    "page_bytes_read_delta",
    "parameter_count",
    "parameter_digest",
    "payload_digest",
    "peak_compaction_buffer_bytes",
    "physical_error_digest",
    "physical_evidence",
    "physical_evidence_consistent",
    "physical_operator_count",
    "physical_sequence",
    "placement_epoch",
    "plan_digest",
    "project_digest",
    "projection_id",
    "projection_kind",
    "query_bytes",
    "query_digest",
    "query_dimensions",
    "query_kind",
    "rebased_over_trace_events",
    "receipt_digest",
    "rejected_path_count",
    "rejected_paths",
    "rejected_paths_digest",
    "request_digest",
    "requested_dimensions",
    "requested_path",
    "required_source_cursor",
    "result_count",
    "returned_rows",
    "schema_revision",
    "segment_bytes",
    "segment_count",
    "segment_io_bounded_reads_delta",
    "segment_io_bounded_segments",
    "segment_io_bytes_read_delta",
    "segment_io_fallbacks_delta",
    "segment_io_last_fallback",
    "segment_io_max_request_bytes",
    "segment_io_mmap_reads_delta",
    "segment_io_mmap_segments",
    "segment_io_peak_request_bytes",
    "segment_io_requested_mode",
    "segment_io_uring_reads_delta",
    "segment_io_uring_segments",
    "selected_path",
    "selected_paths",
    "selected_projection",
    "shard",
    "snapshot_commit_index",
    "snapshot_digest",
    "snapshot_state_digest",
    "snapshot_term",
    "source_change_digest",
    "source_cursor",
    "source_digest",
    "source_family",
    "source_node",
    "source_revision_digest",
    "source_type",
    "stable_revision_present",
    "target_digest",
    "target_node",
    "top_k",
    "transferred_bytes",
    "transferred_objects",
    "verified_references",
    "wal_payload_bytes",
    "wal_payload_max_bytes",
    "work_id",
];

fn validate_operation_name(value: &str) -> Result<()> {
    validate_bounded_text("trace operation", value, MAX_TRACE_NAME_BYTES)?;
    if TraceOperation::from_name(value).is_some()
        || DIRECT_CONVERGENCE_OPERATION_NAMES.contains(&value)
    {
        return Ok(());
    }
    invalid(
        "trace operation is neither a canonical TraceOperation nor reviewed direct-convergence inventory",
    )
}

fn direct_convergence_operation_boundary(value: &str) -> Option<TraceBoundary> {
    match value {
        "embedding.commit" => Some(TraceBoundary::Engine),
        "embedding.infer" | "embedding.run" => Some(TraceBoundary::Inference),
        "rrflow.query.execute"
        | "rrflow.query.parse_bind"
        | "rrflow.query.plan"
        | "rrflow.query.run" => Some(TraceBoundary::Ql),
        "rrflow.storage.runtime_read" => Some(TraceBoundary::Kv),
        "vector.execute"
        | "vector.plan"
        | "vector.projection.publish"
        | "vector.quantization.activate"
        | "vector.quantization.build"
        | "vector.quantization.retire"
        | "vector.search" => Some(TraceBoundary::Vector),
        "cluster.artifact_chunk"
        | "cluster.artifact_transfer"
        | "object.replicate"
        | "operator.knowledge.apply"
        | "operator.knowledge.execute"
        | "operator.knowledge.search"
        | "operator.knowledge.sync" => Some(TraceBoundary::Adapter),
        _ => None,
    }
}

fn validate_attribute_name(value: &str) -> Result<()> {
    validate_bounded_text("trace attribute name", value, MAX_TRACE_NAME_BYTES)?;
    if TraceAttribute::from_name(value).is_some()
        || DIRECT_CONVERGENCE_ATTRIBUTE_NAMES.contains(&value)
    {
        return Ok(());
    }
    invalid(
        "trace attribute is neither a canonical TraceAttribute nor reviewed direct-convergence inventory",
    )
}

fn validate_canonical_attribute_value(
    attribute: TraceAttribute,
    value: &RuntimeValue,
) -> Result<()> {
    match attribute {
        TraceAttribute::Stage
        | TraceAttribute::Phase
        | TraceAttribute::Action
        | TraceAttribute::SkipReason
        | TraceAttribute::CacheStatus
        | TraceAttribute::FailedStage
        | TraceAttribute::ErrorClass
        | TraceAttribute::ReadStampValidation
        | TraceAttribute::VerificationStatus
        | TraceAttribute::PropagationStatus => {
            let RuntimeValue::String(value) = value else {
                return invalid(format!(
                    "trace attribute {} must be a low-cardinality string",
                    attribute.as_str()
                ));
            };
            validate_attribute_token(attribute, value)
        }
        TraceAttribute::Selected
        | TraceAttribute::Exact
        | TraceAttribute::Fallback
        | TraceAttribute::Truncated
        | TraceAttribute::Retryable => {
            if matches!(value, RuntimeValue::Bool(_)) {
                Ok(())
            } else {
                invalid(format!(
                    "trace attribute {} must be a boolean",
                    attribute.as_str()
                ))
            }
        }
        TraceAttribute::Attempt
        | TraceAttribute::InputItems
        | TraceAttribute::InputBytes
        | TraceAttribute::MaxStorageKeys
        | TraceAttribute::OutputItems
        | TraceAttribute::OutputBytes
        | TraceAttribute::SelectedVersions
        | TraceAttribute::KeysExamined
        | TraceAttribute::PagesExamined
        | TraceAttribute::RowsExamined
        | TraceAttribute::ReadPointReads
        | TraceAttribute::ReadRangeScans
        | TraceAttribute::ReadValuesDecoded
        | TraceAttribute::ReadStampValidationChangeReads
        | TraceAttribute::ReadStampValidationProofNodes
        | TraceAttribute::GraphSteps
        | TraceAttribute::CandidatesExamined
        | TraceAttribute::MappedBytes
        | TraceAttribute::ReadBytes
        | TraceAttribute::DecodedBytes
        | TraceAttribute::DecompressedBytes
        | TraceAttribute::BorrowedBytes
        | TraceAttribute::CopiedBytes
        | TraceAttribute::AllocatedBytes
        | TraceAttribute::ElapsedMicros
        | TraceAttribute::MemoryPeakBytes
        | TraceAttribute::SpillBytes
        | TraceAttribute::InputTokens
        | TraceAttribute::OutputTokens
        | TraceAttribute::Rank
        | TraceAttribute::SelectedSourceCount
        | TraceAttribute::SkippedSourceCount
        | TraceAttribute::CompactedBytes
        | TraceAttribute::CompactedTokens
        | TraceAttribute::StartCursor => {
            if matches!(value, RuntimeValue::Unsigned(_)) {
                Ok(())
            } else {
                invalid(format!(
                    "trace attribute {} must be an unsigned integer",
                    attribute.as_str()
                ))
            }
        }
        TraceAttribute::Score | TraceAttribute::RrfContribution => {
            let RuntimeValue::Decimal(value) = value else {
                return invalid(format!(
                    "trace attribute {} must be a finite decimal string",
                    attribute.as_str()
                ));
            };
            if value.parse::<f64>().is_ok_and(f64::is_finite) {
                Ok(())
            } else {
                invalid(format!(
                    "trace attribute {} must be a finite decimal string",
                    attribute.as_str()
                ))
            }
        }
        TraceAttribute::ErrorDigest | TraceAttribute::PropagationDigest => {
            let RuntimeValue::Digest(value) = value else {
                return invalid(format!(
                    "trace attribute {} must be a SHA-256 digest",
                    attribute.as_str()
                ));
            };
            validate_digest("trace attribute", value)
        }
    }
}

fn validate_attribute_token(attribute: TraceAttribute, value: &str) -> Result<()> {
    validate_bounded_text("trace attribute token", value, MAX_TRACE_NAME_BYTES)?;
    let mut bytes = value.bytes();
    if !bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || bytes.any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-'))
        })
    {
        return invalid(format!(
            "trace attribute {} must use a bounded lowercase token",
            attribute.as_str()
        ));
    }
    Ok(())
}

fn validate_bounded_text(kind: &'static str, value: &str, max: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > max || value.as_bytes().contains(&0) {
        return invalid(format!(
            "{kind} must be non-empty, contain no NUL, and fit within {max} bytes"
        ));
    }
    Ok(())
}

fn invalid<T>(reason: impl Into<String>) -> Result<T> {
    Err(Error::InvalidRuntime {
        reason: reason.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace_id() -> TraceId {
        TraceId::new("0123456789abcdef0123456789abcdef").unwrap()
    }

    fn span_id() -> SpanId {
        SpanId::new("0123456789abcdef").unwrap()
    }

    fn valid_canonical_attribute_value(attribute: TraceAttribute) -> RuntimeValue {
        match attribute {
            TraceAttribute::Stage
            | TraceAttribute::Phase
            | TraceAttribute::Action
            | TraceAttribute::SkipReason
            | TraceAttribute::CacheStatus
            | TraceAttribute::FailedStage
            | TraceAttribute::ErrorClass
            | TraceAttribute::ReadStampValidation
            | TraceAttribute::VerificationStatus
            | TraceAttribute::PropagationStatus => RuntimeValue::String("verified_value".into()),
            TraceAttribute::Selected
            | TraceAttribute::Exact
            | TraceAttribute::Fallback
            | TraceAttribute::Truncated
            | TraceAttribute::Retryable => RuntimeValue::Bool(true),
            TraceAttribute::Attempt
            | TraceAttribute::InputItems
            | TraceAttribute::InputBytes
            | TraceAttribute::MaxStorageKeys
            | TraceAttribute::OutputItems
            | TraceAttribute::OutputBytes
            | TraceAttribute::SelectedVersions
            | TraceAttribute::KeysExamined
            | TraceAttribute::PagesExamined
            | TraceAttribute::RowsExamined
            | TraceAttribute::ReadPointReads
            | TraceAttribute::ReadRangeScans
            | TraceAttribute::ReadValuesDecoded
            | TraceAttribute::ReadStampValidationChangeReads
            | TraceAttribute::ReadStampValidationProofNodes
            | TraceAttribute::GraphSteps
            | TraceAttribute::CandidatesExamined
            | TraceAttribute::MappedBytes
            | TraceAttribute::ReadBytes
            | TraceAttribute::DecodedBytes
            | TraceAttribute::DecompressedBytes
            | TraceAttribute::BorrowedBytes
            | TraceAttribute::CopiedBytes
            | TraceAttribute::AllocatedBytes
            | TraceAttribute::ElapsedMicros
            | TraceAttribute::MemoryPeakBytes
            | TraceAttribute::SpillBytes
            | TraceAttribute::InputTokens
            | TraceAttribute::OutputTokens
            | TraceAttribute::Rank
            | TraceAttribute::SelectedSourceCount
            | TraceAttribute::SkippedSourceCount
            | TraceAttribute::CompactedBytes
            | TraceAttribute::CompactedTokens
            | TraceAttribute::StartCursor => RuntimeValue::Unsigned(1),
            TraceAttribute::Score | TraceAttribute::RrfContribution => {
                RuntimeValue::Decimal("0.5".into())
            }
            TraceAttribute::ErrorDigest | TraceAttribute::PropagationDigest => {
                RuntimeValue::Digest("a".repeat(64))
            }
        }
    }

    #[test]
    fn w3c_id_widths_and_lifecycle_transitions_fail_closed() {
        assert!(TraceId::new("0".repeat(32)).is_err());
        assert!(TraceId::new("ABCDEF0123456789abcdef0123456789").is_err());
        assert!(SpanId::new("0123").is_err());

        let mut event = RuntimeTraceEvent::start(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Ql,
            TraceOperation::QlExecute,
            10,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::new(),
        )
        .unwrap();
        event.duration_micros = Some(1);
        assert!(event.validate().is_err());
    }

    #[test]
    fn external_source_is_linked_without_claiming_shared_transactions() {
        let event = RuntimeTraceEvent::finish(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Adapter,
            TraceOperation::AdapterInvoke,
            20,
            800,
            TraceOutcome::Ok,
            TraceDataClass::Operator,
            vec![TraceLink::Source {
                source_kind: "operator_knowledge".into(),
                source_id: "pgvector:rrflow".into(),
                source_revision: "postgres-lsn:0/16B6C50".into(),
            }],
            RuntimeProperties::from([(
                TraceAttribute::OutputItems.into(),
                RuntimeValue::Unsigned(7),
            )]),
        )
        .unwrap();

        let runtime = event.into_runtime_event().unwrap();
        assert_eq!(runtime.kind.as_str(), RUNTIME_TRACE_EVENT_TYPE);
        assert_eq!(
            runtime.properties["outcome"],
            RuntimeValue::String("ok".into())
        );
        assert_eq!(
            RuntimeTraceEvent::event_schema().properties["links"].value_type,
            RuntimeValueType::List
        );
    }

    #[test]
    fn schema_registration_is_strict_and_idempotent() {
        let mut registry = RuntimeSchemaRegistry::empty(1, "trace bootstrap");
        assert!(RuntimeTraceEvent::register_schema(&mut registry).unwrap());
        assert!(!RuntimeTraceEvent::register_schema(&mut registry).unwrap());
        assert_eq!(
            registry.events[&RuntimeTraceEvent::event_type().unwrap()],
            RuntimeTraceEvent::event_schema()
        );
    }

    #[test]
    fn trace_commit_preparation_atomically_installs_schema_and_binds_the_read() {
        let scope = crate::ScopeId::new("instance:trace-commit").unwrap();
        let read = ReadStamp::new(scope.clone(), None, 0, 0, None).unwrap();
        let event = RuntimeTraceEvent::start(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Adapter,
            TraceOperation::AdapterTransfer,
            10,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::new(),
        )
        .unwrap();
        let bootstrap = event
            .prepare_commit(&read, None, "cluster:transport")
            .unwrap();
        assert_eq!(bootstrap.scope, scope);
        assert_eq!(bootstrap.expected_cursor, 0);
        assert!(matches!(
            bootstrap.mutations[0],
            RuntimeMutation::Schema { .. }
        ));
        assert!(matches!(
            bootstrap.mutations[1],
            RuntimeMutation::Event { .. }
        ));

        let RuntimeMutation::Schema { registry } = &bootstrap.mutations[0] else {
            unreachable!()
        };
        let installed_read =
            ReadStamp::new(scope, Some(1), 0, 2, Some(bootstrap.digest())).unwrap();
        let event_only = event
            .prepare_commit(&installed_read, Some(registry), "cluster:transport")
            .unwrap();
        assert_eq!(event_only.mutations.len(), 1);
        assert!(matches!(
            event_only.mutations[0],
            RuntimeMutation::Event { .. }
        ));

        let stale = ReadStamp::new(
            installed_read.scope.clone(),
            Some(2),
            0,
            installed_read.commit_cursor,
            installed_read.head_digest.clone(),
        )
        .unwrap();
        assert!(event
            .prepare_commit(&stale, Some(registry), "cluster:transport")
            .is_err());
    }

    #[test]
    fn attributes_are_bounded_before_they_reach_the_runtime_log() {
        let event = RuntimeTraceEvent::annotation(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Inference,
            TraceOperation::InferenceRoute,
            10,
            TraceOutcome::Running,
            TraceDataClass::Content,
            Vec::new(),
            RuntimeProperties::from([(
                TraceAttribute::Stage.into(),
                RuntimeValue::String("x".repeat(MAX_TRACE_STRING_BYTES + 1)),
            )]),
        );
        assert!(event.is_err());
    }

    #[test]
    fn persisted_read_and_projection_links_retain_complete_reconstructable_stamps() {
        let read = ReadStamp::new(
            crate::ScopeId::new("instance:trace").unwrap(),
            Some(7),
            11,
            29,
            Some("a".repeat(64)),
        )
        .unwrap();
        let projection = ProjectionStamp {
            contract_version: crate::DATA_RUNTIME_CONTRACT_VERSION,
            id: crate::ProjectionId::new("vector:operator").unwrap(),
            generation: 3,
            source_cursor: 29,
            config_digest: "b".repeat(64),
            artifact_digest: "c".repeat(64),
            state: crate::ProjectionState::Ready,
        };
        projection.validate().unwrap();
        let runtime = RuntimeTraceEvent::annotation(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Vector,
            TraceOperation::VectorProjection,
            10,
            TraceOutcome::Ok,
            TraceDataClass::Control,
            vec![
                TraceLink::Read {
                    stamp: read.clone(),
                },
                TraceLink::Projection {
                    stamp: projection.clone(),
                },
            ],
            RuntimeProperties::new(),
        )
        .unwrap()
        .into_runtime_event()
        .unwrap();
        let RuntimeValue::List(links) = &runtime.properties["links"] else {
            panic!("links must remain a typed list")
        };
        let RuntimeValue::Map(read_link) = &links[0] else {
            panic!("read link must remain a typed map")
        };
        assert_eq!(
            read_link["contract_version"],
            RuntimeValue::Unsigned(u64::from(read.contract_version))
        );
        assert_eq!(
            read_link["schema_revision"],
            RuntimeValue::Unsigned(read.schema_revision.unwrap())
        );
        assert_eq!(
            read_link["catalog_revision"],
            RuntimeValue::Unsigned(read.catalog_revision)
        );
        assert_eq!(
            read_link["commit_cursor"],
            RuntimeValue::Unsigned(read.commit_cursor)
        );
        assert!(
            !read_link.contains_key("cursor"),
            "read links must not retain a compatibility alias for commit_cursor"
        );
        assert_eq!(
            read_link["head_digest"],
            RuntimeValue::Digest(read.head_digest.unwrap())
        );
        let RuntimeValue::Map(projection_link) = &links[1] else {
            panic!("projection link must remain a typed map")
        };
        assert_eq!(
            projection_link["config_digest"],
            RuntimeValue::Digest(projection.config_digest)
        );
        assert_eq!(
            projection_link["state"],
            RuntimeValue::String("ready".into())
        );
    }

    #[test]
    fn canonical_operations_are_closed_unique_and_cover_every_boundary() {
        let names = TraceOperation::ALL
            .iter()
            .map(|operation| operation.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), TraceOperation::ALL.len());
        for operation in TraceOperation::ALL {
            let expected_prefix = format!("rrflow.{}.", operation.boundary().as_str());
            assert!(operation.as_str().starts_with(&expected_prefix));
            assert_eq!(
                TraceOperation::from_name(operation.as_str()),
                Some(operation)
            );
            assert!(operation.as_str().bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_')
            }));
        }
        for boundary in TraceBoundary::ALL {
            assert!(TraceOperation::ALL
                .iter()
                .any(|operation| operation.boundary() == boundary));
        }

        let unknown = RuntimeTraceEvent::start(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Vector,
            "rrflow.vector.project_42",
            10,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::new(),
        );
        assert!(unknown.is_err());

        let mismatched = RuntimeTraceEvent::start(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Kv,
            TraceOperation::VectorSearch,
            10,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::new(),
        );
        assert!(mismatched.is_err());
    }

    #[test]
    fn canonical_attributes_and_direct_convergence_inventory_are_closed() {
        let names = TraceAttribute::ALL
            .iter()
            .map(|attribute| attribute.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), TraceAttribute::ALL.len());
        for attribute in TraceAttribute::ALL {
            assert_eq!(
                TraceAttribute::from_name(attribute.as_str()),
                Some(*attribute)
            );
            assert!(attribute.as_str().bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
            }));
            RuntimeTraceEvent::annotation(
                trace_id(),
                span_id(),
                None,
                TraceBoundary::Engine,
                TraceOperation::EngineOperation,
                10,
                TraceOutcome::Running,
                TraceDataClass::Control,
                Vec::new(),
                RuntimeProperties::from([(
                    attribute.to_string(),
                    valid_canonical_attribute_value(*attribute),
                )]),
            )
            .unwrap_or_else(|error| {
                panic!("canonical attribute {} failed: {error}", attribute.as_str())
            });
        }

        assert!(DIRECT_CONVERGENCE_OPERATION_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        assert!(DIRECT_CONVERGENCE_OPERATION_NAMES
            .iter()
            .all(|name| TraceOperation::from_name(name).is_none()
                && direct_convergence_operation_boundary(name).is_some()));
        assert!(DIRECT_CONVERGENCE_ATTRIBUTE_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        assert!(DIRECT_CONVERGENCE_ATTRIBUTE_NAMES
            .iter()
            .all(|name| TraceAttribute::from_name(name).is_none()));

        let unknown = RuntimeTraceEvent::annotation(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Engine,
            TraceOperation::EngineOperation,
            10,
            TraceOutcome::Running,
            TraceDataClass::Control,
            Vec::new(),
            RuntimeProperties::from([(
                "project_specific_counter".into(),
                RuntimeValue::Unsigned(1),
            )]),
        );
        assert!(unknown.is_err());

        for (attribute, wrong_value) in [
            (TraceAttribute::InputBytes, RuntimeValue::String("1".into())),
            (TraceAttribute::Selected, RuntimeValue::Unsigned(1)),
            (TraceAttribute::Score, RuntimeValue::Decimal("NaN".into())),
            (
                TraceAttribute::ErrorDigest,
                RuntimeValue::Digest("not-a-digest".into()),
            ),
            (
                TraceAttribute::ErrorClass,
                RuntimeValue::String("raw error text is forbidden".into()),
            ),
        ] {
            let invalid = RuntimeTraceEvent::annotation(
                trace_id(),
                span_id(),
                None,
                TraceBoundary::Engine,
                TraceOperation::EngineOperation,
                10,
                TraceOutcome::Running,
                TraceDataClass::Control,
                Vec::new(),
                RuntimeProperties::from([(attribute.to_string(), wrong_value)]),
            );
            assert!(
                invalid.is_err(),
                "{} accepted wrong value",
                attribute.as_str()
            );
        }
    }

    #[test]
    fn trace_wire_contract_rejects_old_and_extended_shapes() {
        let event = RuntimeTraceEvent::annotation(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Adapter,
            TraceOperation::AdapterInvoke,
            10,
            TraceOutcome::Ok,
            TraceDataClass::Control,
            vec![TraceLink::Source {
                source_kind: "project_database".into(),
                source_id: "adapter-source-1".into(),
                source_revision: "revision-1".into(),
            }],
            RuntimeProperties::new(),
        )
        .unwrap();

        let mut extended_event = serde_json::to_value(&event).unwrap();
        extended_event
            .as_object_mut()
            .unwrap()
            .insert("provider".into(), serde_json::json!("hidden-authority"));
        assert!(serde_json::from_value::<RuntimeTraceEvent>(extended_event).is_err());

        let mut old_boundary = serde_json::to_value(&event).unwrap();
        let old_boundary = old_boundary.as_object_mut().unwrap();
        let boundary = old_boundary.remove("boundary").unwrap();
        old_boundary.insert("domain".into(), boundary);
        assert!(
            serde_json::from_value::<RuntimeTraceEvent>(serde_json::Value::Object(
                old_boundary.clone()
            ))
            .is_err()
        );

        let mut extended_link = serde_json::to_value(&event).unwrap();
        extended_link["links"][0]["adapter"] = serde_json::json!("hidden-authority");
        assert!(serde_json::from_value::<RuntimeTraceEvent>(extended_link).is_err());

        let mut retired_link = serde_json::to_value(event).unwrap();
        retired_link["links"][0] = serde_json::json!({
            "kind": "provider",
            "provider": "claude",
            "invocation_id": "invocation-1"
        });
        assert!(serde_json::from_value::<RuntimeTraceEvent>(retired_link).is_err());
    }

    #[test]
    fn canonical_links_retain_correlation_authority_and_output_coordinates() {
        let scope = ScopeId::new("instance:trace-links").unwrap();
        let causal_span = SpanId::new("fedcba9876543210").unwrap();
        let links = vec![
            TraceLink::Request {
                request_id: "request-7".into(),
                operation_id: "query-execute".into(),
                parent_request_id: Some("request-6".into()),
                idempotency_digest: Some("1".repeat(64)),
            },
            TraceLink::ActorScope {
                actor: "seat:clyffy".into(),
                scope: scope.clone(),
            },
            TraceLink::Authorization {
                decision: AuditDecision::Allow,
                policy_revision: 9,
                authorization_digest: "2".repeat(64),
            },
            TraceLink::Source {
                source_kind: "project_tree".into(),
                source_id: "snapshot-7".into(),
                source_revision: "revision-3".into(),
            },
            TraceLink::Commit {
                commit_id: "3".repeat(64),
                first_cursor: 40,
                last_cursor: 43,
            },
            TraceLink::Resource {
                resource_kind: "instance".into(),
                resource_id: "rrflow-test".into(),
            },
            TraceLink::CausalSpan {
                trace_id: trace_id(),
                span_id: causal_span,
                relation: TraceCausalRelation::FollowsFrom,
            },
            TraceLink::Output {
                output_digest: "4".repeat(64),
                item_count: 7,
                byte_count: 512,
            },
        ];
        let runtime = RuntimeTraceEvent::annotation(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Engine,
            TraceOperation::EngineOperation,
            10,
            TraceOutcome::Ok,
            TraceDataClass::Control,
            links,
            RuntimeProperties::new(),
        )
        .unwrap()
        .into_runtime_event()
        .unwrap();
        assert_eq!(
            runtime.properties["boundary"],
            RuntimeValue::String("engine".into())
        );
        let RuntimeValue::List(links) = &runtime.properties["links"] else {
            panic!("links must remain a typed list")
        };
        let kinds = links
            .iter()
            .map(|link| {
                let RuntimeValue::Map(link) = link else {
                    panic!("link must remain a typed map")
                };
                let RuntimeValue::String(kind) = &link["kind"] else {
                    panic!("link kind must remain a string")
                };
                kind.as_str()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                "request",
                "actor_scope",
                "authorization",
                "source",
                "commit",
                "resource",
                "causal_span",
                "output",
            ]
        );

        let self_link = RuntimeTraceEvent::annotation(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Routine,
            TraceOperation::RoutineStep,
            11,
            TraceOutcome::Running,
            TraceDataClass::Control,
            vec![TraceLink::CausalSpan {
                trace_id: trace_id(),
                span_id: span_id(),
                relation: TraceCausalRelation::RetryOf,
            }],
            RuntimeProperties::new(),
        );
        assert!(self_link.is_err());

        let unstamped_authorization = RuntimeTraceEvent::annotation(
            trace_id(),
            span_id(),
            None,
            TraceBoundary::Engine,
            TraceOperation::EngineAuthorize,
            12,
            TraceOutcome::Denied,
            TraceDataClass::Control,
            vec![TraceLink::Authorization {
                decision: AuditDecision::Deny,
                policy_revision: 0,
                authorization_digest: "5".repeat(64),
            }],
            RuntimeProperties::new(),
        );
        assert!(unstamped_authorization.is_err());
    }
}
