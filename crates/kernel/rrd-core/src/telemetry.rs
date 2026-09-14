//! Closed, inert signal identity for RRFlow diagnostics.
//!
//! This module describes the diagnostic levels, durable trace projections, and
//! metric instruments shared by build manifests, generated contracts, SDKs,
//! and later runtime instrumentation. It does not collect or export telemetry,
//! read process state, or establish evidence of an engine effect.

use crate::{
    digest, Error, Result, TraceAttribute, TraceBoundary, TraceOperation,
    RUNTIME_TRACE_CONTRACT_VERSION,
};
use serde::{Deserialize, Serialize};

pub const TELEMETRY_CONTRACT_VERSION: u16 = 1;
pub const DEFAULT_METRIC_CARDINALITY_LIMIT: u32 = 2_000;

const FINGERPRINT_DOMAIN: &[u8] = b"rrflow.signal-catalogue.sha256";

/// Runtime-scoped diagnostic detail. These levels change observation cost,
/// never the engine's accepted operations, results, formats, or limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Off,
    Normal,
    Detailed,
    Profile,
}

impl DiagnosticLevel {
    pub const ALL: [Self; 4] = [Self::Off, Self::Normal, Self::Detailed, Self::Profile];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Normal => "normal",
            Self::Detailed => "detailed",
            Self::Profile => "profile",
        }
    }
}

/// Aggregation behavior of one metric instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    Counter,
    UpDownCounter,
    ObservableGauge,
    Histogram,
}

impl MetricKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::UpDownCounter => "up_down_counter",
            Self::ObservableGauge => "observable_gauge",
            Self::Histogram => "histogram",
        }
    }
}

/// UCUM/OpenTelemetry unit attached to one metric instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MetricUnit {
    #[serde(rename = "s")]
    Seconds,
    #[serde(rename = "By")]
    Bytes,
    #[serde(rename = "{operation}")]
    Operations,
    #[serde(rename = "{item}")]
    Items,
    #[serde(rename = "{request}")]
    Requests,
    #[serde(rename = "{row}")]
    Rows,
    #[serde(rename = "{step}")]
    Steps,
    #[serde(rename = "{candidate}")]
    Candidates,
    #[serde(rename = "{token}")]
    Tokens,
}

impl MetricUnit {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Seconds => "s",
            Self::Bytes => "By",
            Self::Operations => "{operation}",
            Self::Items => "{item}",
            Self::Requests => "{request}",
            Self::Rows => "{row}",
            Self::Steps => "{step}",
            Self::Candidates => "{candidate}",
            Self::Tokens => "{token}",
        }
    }
}

/// The complete metric-dimension vocabulary. Values supplied at runtime must
/// additionally come from the bounded catalogue owned by each instrumenting
/// site; this enum does not admit arbitrary labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MetricAttribute {
    #[serde(rename = "rrflow.boundary")]
    Boundary,
    #[serde(rename = "rrflow.operation")]
    Operation,
    #[serde(rename = "rrflow.outcome")]
    Outcome,
    #[serde(rename = "rrflow.storage.profile")]
    StorageProfile,
    #[serde(rename = "rrflow.access.path")]
    AccessPath,
    #[serde(rename = "rrflow.work.kind")]
    WorkKind,
    #[serde(rename = "rrflow.queue.kind")]
    QueueKind,
    #[serde(rename = "rrflow.cache.kind")]
    CacheKind,
    #[serde(rename = "error.type")]
    ErrorType,
}

impl MetricAttribute {
    pub const ALL: [Self; 9] = [
        Self::Boundary,
        Self::Operation,
        Self::Outcome,
        Self::StorageProfile,
        Self::AccessPath,
        Self::WorkKind,
        Self::QueueKind,
        Self::CacheKind,
        Self::ErrorType,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boundary => "rrflow.boundary",
            Self::Operation => "rrflow.operation",
            Self::Outcome => "rrflow.outcome",
            Self::StorageProfile => "rrflow.storage.profile",
            Self::AccessPath => "rrflow.access.path",
            Self::WorkKind => "rrflow.work.kind",
            Self::QueueKind => "rrflow.queue.kind",
            Self::CacheKind => "rrflow.cache.kind",
            Self::ErrorType => "error.type",
        }
    }
}

/// Closed metric instrument names. Exporters may translate presentation
/// syntax, but cannot add or rename instruments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MetricInstrument {
    #[serde(rename = "rrflow.operation.duration")]
    OperationDuration,
    #[serde(rename = "rrflow.operation.count")]
    OperationCount,
    #[serde(rename = "rrflow.operation.inflight")]
    OperationInflight,
    #[serde(rename = "rrflow.queue.depth")]
    QueueDepth,
    #[serde(rename = "rrflow.queue.wait")]
    QueueWait,
    #[serde(rename = "rrflow.kv.keys.examined")]
    KvKeysExamined,
    #[serde(rename = "rrflow.kv.pages.examined")]
    KvPagesExamined,
    #[serde(rename = "rrflow.kv.bytes")]
    KvBytes,
    #[serde(rename = "rrflow.kv.cache.requests")]
    KvCacheRequests,
    #[serde(rename = "rrflow.query.rows")]
    QueryRows,
    #[serde(rename = "rrflow.query.graph.steps")]
    QueryGraphSteps,
    #[serde(rename = "rrflow.query.candidates")]
    QueryCandidates,
    #[serde(rename = "rrflow.datafusion.memory.usage")]
    DataFusionMemoryUsage,
    #[serde(rename = "rrflow.datafusion.memory.peak")]
    DataFusionMemoryPeak,
    #[serde(rename = "rrflow.datafusion.spill")]
    DataFusionSpill,
    #[serde(rename = "rrflow.context.bytes")]
    ContextBytes,
    #[serde(rename = "rrflow.context.tokens")]
    ContextTokens,
    #[serde(rename = "rrflow.delivery.backlog")]
    DeliveryBacklog,
    #[serde(rename = "rrflow.process.memory")]
    ProcessMemory,
    #[serde(rename = "rrflow.process.cpu.time")]
    ProcessCpuTime,
    #[serde(rename = "rrflow.telemetry.dropped")]
    TelemetryDropped,
    #[serde(rename = "rrflow.telemetry.export.errors")]
    TelemetryExportErrors,
}

impl MetricInstrument {
    pub const ALL: [Self; 22] = [
        Self::OperationDuration,
        Self::OperationCount,
        Self::OperationInflight,
        Self::QueueDepth,
        Self::QueueWait,
        Self::KvKeysExamined,
        Self::KvPagesExamined,
        Self::KvBytes,
        Self::KvCacheRequests,
        Self::QueryRows,
        Self::QueryGraphSteps,
        Self::QueryCandidates,
        Self::DataFusionMemoryUsage,
        Self::DataFusionMemoryPeak,
        Self::DataFusionSpill,
        Self::ContextBytes,
        Self::ContextTokens,
        Self::DeliveryBacklog,
        Self::ProcessMemory,
        Self::ProcessCpuTime,
        Self::TelemetryDropped,
        Self::TelemetryExportErrors,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OperationDuration => "rrflow.operation.duration",
            Self::OperationCount => "rrflow.operation.count",
            Self::OperationInflight => "rrflow.operation.inflight",
            Self::QueueDepth => "rrflow.queue.depth",
            Self::QueueWait => "rrflow.queue.wait",
            Self::KvKeysExamined => "rrflow.kv.keys.examined",
            Self::KvPagesExamined => "rrflow.kv.pages.examined",
            Self::KvBytes => "rrflow.kv.bytes",
            Self::KvCacheRequests => "rrflow.kv.cache.requests",
            Self::QueryRows => "rrflow.query.rows",
            Self::QueryGraphSteps => "rrflow.query.graph.steps",
            Self::QueryCandidates => "rrflow.query.candidates",
            Self::DataFusionMemoryUsage => "rrflow.datafusion.memory.usage",
            Self::DataFusionMemoryPeak => "rrflow.datafusion.memory.peak",
            Self::DataFusionSpill => "rrflow.datafusion.spill",
            Self::ContextBytes => "rrflow.context.bytes",
            Self::ContextTokens => "rrflow.context.tokens",
            Self::DeliveryBacklog => "rrflow.delivery.backlog",
            Self::ProcessMemory => "rrflow.process.memory",
            Self::ProcessCpuTime => "rrflow.process.cpu.time",
            Self::TelemetryDropped => "rrflow.telemetry.dropped",
            Self::TelemetryExportErrors => "rrflow.telemetry.export.errors",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceOperationDescriptor {
    pub name: String,
    pub boundary: TraceBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceAttributeDescriptor {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricDescriptor {
    pub instrument: MetricInstrument,
    pub kind: MetricKind,
    pub unit: MetricUnit,
    pub description: String,
    /// Canonically ordered dimensions this instrument may carry. Each point
    /// uses only the subset applicable to that observation.
    pub attributes: Vec<MetricAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalCatalogue {
    pub contract_version: u16,
    pub runtime_trace_contract_version: u16,
    pub default_diagnostic_level: DiagnosticLevel,
    pub diagnostic_levels: Vec<DiagnosticLevel>,
    pub default_metric_cardinality_limit: u32,
    pub trace_operations: Vec<TraceOperationDescriptor>,
    pub trace_attributes: Vec<TraceAttributeDescriptor>,
    pub metric_attributes: Vec<MetricAttribute>,
    pub metric_instruments: Vec<MetricDescriptor>,
}

#[derive(Clone, Copy)]
struct MetricSpec {
    instrument: MetricInstrument,
    kind: MetricKind,
    unit: MetricUnit,
    description: &'static str,
    attributes: &'static [MetricAttribute],
}

use MetricAttribute as A;
use MetricInstrument as I;
use MetricKind as K;
use MetricUnit as U;

const OPERATION_ATTRIBUTES: &[A] = &[
    A::Boundary,
    A::Operation,
    A::Outcome,
    A::StorageProfile,
    A::AccessPath,
    A::ErrorType,
];
const QUEUE_DEPTH_ATTRIBUTES: &[A] = &[A::Boundary, A::QueueKind];
const QUEUE_WAIT_ATTRIBUTES: &[A] = &[
    A::Boundary,
    A::Operation,
    A::Outcome,
    A::QueueKind,
    A::ErrorType,
];
const KV_WORK_ATTRIBUTES: &[A] = &[
    A::Operation,
    A::Outcome,
    A::StorageProfile,
    A::AccessPath,
    A::ErrorType,
];
const KV_BYTE_ATTRIBUTES: &[A] = &[
    A::Operation,
    A::Outcome,
    A::StorageProfile,
    A::AccessPath,
    A::WorkKind,
    A::ErrorType,
];
const KV_CACHE_ATTRIBUTES: &[A] = &[
    A::Operation,
    A::Outcome,
    A::StorageProfile,
    A::AccessPath,
    A::CacheKind,
    A::ErrorType,
];
const QUERY_WORK_ATTRIBUTES: &[A] = &[
    A::Operation,
    A::Outcome,
    A::AccessPath,
    A::WorkKind,
    A::ErrorType,
];
const OPERATION_WORK_ATTRIBUTES: &[A] = &[A::Operation, A::Outcome, A::WorkKind, A::ErrorType];
const DELIVERY_ATTRIBUTES: &[A] = &[A::Boundary, A::QueueKind];
const PROCESS_ATTRIBUTES: &[A] = &[A::WorkKind];
const TELEMETRY_DROP_ATTRIBUTES: &[A] = &[A::Outcome, A::WorkKind, A::QueueKind, A::ErrorType];
const TELEMETRY_ERROR_ATTRIBUTES: &[A] = &[A::Outcome, A::WorkKind, A::ErrorType];

const METRIC_SPECS: [MetricSpec; 22] = [
    MetricSpec {
        instrument: I::OperationDuration,
        kind: K::Histogram,
        unit: U::Seconds,
        description: "Wall duration of one exact operation boundary, recorded once with terminal outcome.",
        attributes: OPERATION_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::OperationCount,
        kind: K::Counter,
        unit: U::Operations,
        description: "Accepted terminal operations; denials, cancellations, and errors remain distinct outcomes.",
        attributes: OPERATION_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::OperationInflight,
        kind: K::UpDownCounter,
        unit: U::Operations,
        description: "Currently admitted operations, including queued time until terminal release.",
        attributes: &[A::Boundary, A::Operation, A::StorageProfile, A::AccessPath],
    },
    MetricSpec {
        instrument: I::QueueDepth,
        kind: K::ObservableGauge,
        unit: U::Items,
        description: "Bounded executor, maintenance, delivery, model, spill, and telemetry queue occupancy.",
        attributes: QUEUE_DEPTH_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::QueueWait,
        kind: K::Histogram,
        unit: U::Seconds,
        description: "Admission-to-execution delay separate from compute time.",
        attributes: QUEUE_WAIT_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::KvKeysExamined,
        kind: K::Counter,
        unit: U::Items,
        description: "Physical KV keys examined by attributable point, range, and page operations.",
        attributes: KV_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::KvPagesExamined,
        kind: K::Counter,
        unit: U::Items,
        description: "Physical KV pages examined by attributable point, range, and page operations.",
        attributes: KV_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::KvBytes,
        kind: K::Counter,
        unit: U::Bytes,
        description: "Read, mapped, decoded, decompressed, borrowed, copied, allocated, WAL, flush, compaction, and spill byte work.",
        attributes: KV_BYTE_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::KvCacheRequests,
        kind: K::Counter,
        unit: U::Requests,
        description: "Hit, miss, admission rejection, and eviction outcomes for a bounded cache class.",
        attributes: KV_CACHE_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::QueryRows,
        kind: K::Counter,
        unit: U::Rows,
        description: "Rows examined and emitted by rrflowQL, native, and DataFusion execution.",
        attributes: QUERY_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::QueryGraphSteps,
        kind: K::Counter,
        unit: U::Steps,
        description: "Eligible graph edges and vertices traversed by query execution.",
        attributes: QUERY_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::QueryCandidates,
        kind: K::Counter,
        unit: U::Candidates,
        description: "Lexical, vector, fusion, and exact-rerank candidate work by bounded access path.",
        attributes: QUERY_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::DataFusionMemoryUsage,
        kind: K::ObservableGauge,
        unit: U::Bytes,
        description: "Current DataFusion pool reservation by bounded pool class.",
        attributes: &[A::Operation, A::WorkKind],
    },
    MetricSpec {
        instrument: I::DataFusionMemoryPeak,
        kind: K::Histogram,
        unit: U::Bytes,
        description: "Peak DataFusion pool reservation observed for one terminal query execution.",
        attributes: OPERATION_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::DataFusionSpill,
        kind: K::Counter,
        unit: U::Bytes,
        description: "DataFusion spill bytes read and written by bounded work kind.",
        attributes: OPERATION_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::ContextBytes,
        kind: K::Histogram,
        unit: U::Bytes,
        description: "Authorized context input, output, selected, skipped, truncated, and compacted byte volume.",
        attributes: OPERATION_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::ContextTokens,
        kind: K::Histogram,
        unit: U::Tokens,
        description: "Authorized context input, output, selected, skipped, truncated, and compacted token volume.",
        attributes: OPERATION_WORK_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::DeliveryBacklog,
        kind: K::ObservableGauge,
        unit: U::Items,
        description: "Unacknowledged bounded delivery items by stream class.",
        attributes: DELIVERY_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::ProcessMemory,
        kind: K::ObservableGauge,
        unit: U::Bytes,
        description: "Process RSS and allocator-owned bytes where the platform can identify them.",
        attributes: PROCESS_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::ProcessCpuTime,
        kind: K::Counter,
        unit: U::Seconds,
        description: "User and system CPU time for saturation and benchmark accounting.",
        attributes: PROCESS_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::TelemetryDropped,
        kind: K::Counter,
        unit: U::Items,
        description: "Diagnostic points dropped by bounded queue, sampling, or encoding policy.",
        attributes: TELEMETRY_DROP_ATTRIBUTES,
    },
    MetricSpec {
        instrument: I::TelemetryExportErrors,
        kind: K::Counter,
        unit: U::Items,
        description: "Diagnostic encode, export, and exporter-rejection failures.",
        attributes: TELEMETRY_ERROR_ATTRIBUTES,
    },
];

/// Materializes the canonical catalogue from its enum owners. The returned
/// value is owned so generators and tests can serialize or adversarially mutate
/// it without exposing mutable global state.
pub fn signal_catalogue() -> SignalCatalogue {
    SignalCatalogue {
        contract_version: TELEMETRY_CONTRACT_VERSION,
        runtime_trace_contract_version: RUNTIME_TRACE_CONTRACT_VERSION,
        default_diagnostic_level: DiagnosticLevel::Normal,
        diagnostic_levels: DiagnosticLevel::ALL.to_vec(),
        default_metric_cardinality_limit: DEFAULT_METRIC_CARDINALITY_LIMIT,
        trace_operations: TraceOperation::ALL
            .iter()
            .copied()
            .map(|operation| TraceOperationDescriptor {
                name: operation.as_str().into(),
                boundary: operation.boundary(),
            })
            .collect(),
        trace_attributes: TraceAttribute::ALL
            .iter()
            .copied()
            .map(|attribute| TraceAttributeDescriptor {
                name: attribute.as_str().into(),
            })
            .collect(),
        metric_attributes: MetricAttribute::ALL.to_vec(),
        metric_instruments: METRIC_SPECS
            .iter()
            .map(|spec| MetricDescriptor {
                instrument: spec.instrument,
                kind: spec.kind,
                unit: spec.unit,
                description: spec.description.into(),
                attributes: spec.attributes.to_vec(),
            })
            .collect(),
    }
}

pub fn signal_catalogue_sha256() -> Result<String> {
    signal_catalogue().sha256()
}

impl SignalCatalogue {
    /// Rejects semantic, ordering, or wire drift from the v1 catalogue.
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != TELEMETRY_CONTRACT_VERSION {
            return invalid(format!(
                "contract version {} is not {TELEMETRY_CONTRACT_VERSION}",
                self.contract_version
            ));
        }
        if self.runtime_trace_contract_version != RUNTIME_TRACE_CONTRACT_VERSION {
            return invalid(format!(
                "runtime trace contract version {} is not {RUNTIME_TRACE_CONTRACT_VERSION}",
                self.runtime_trace_contract_version
            ));
        }
        if self.default_diagnostic_level != DiagnosticLevel::Normal {
            return invalid("default diagnostic level is not normal");
        }
        if self.diagnostic_levels.as_slice() != DiagnosticLevel::ALL {
            return invalid("diagnostic levels are missing, duplicated, or out of canonical order");
        }
        if self.default_metric_cardinality_limit != DEFAULT_METRIC_CARDINALITY_LIMIT {
            return invalid(format!(
                "default metric cardinality limit {} is not {DEFAULT_METRIC_CARDINALITY_LIMIT}",
                self.default_metric_cardinality_limit
            ));
        }
        if self.trace_operations.len() != TraceOperation::ALL.len() {
            return invalid("trace operation catalogue length differs from TraceOperation::ALL");
        }
        for (descriptor, operation) in self.trace_operations.iter().zip(TraceOperation::ALL) {
            if descriptor.name != operation.as_str() || descriptor.boundary != operation.boundary()
            {
                return invalid(format!(
                    "trace operation descriptor differs from {}",
                    operation.as_str()
                ));
            }
        }
        if self.trace_attributes.len() != TraceAttribute::ALL.len() {
            return invalid("trace attribute catalogue length differs from TraceAttribute::ALL");
        }
        for (descriptor, attribute) in self.trace_attributes.iter().zip(TraceAttribute::ALL) {
            if descriptor.name != attribute.as_str() {
                return invalid(format!(
                    "trace attribute descriptor differs from {}",
                    attribute.as_str()
                ));
            }
        }
        if self.metric_attributes.as_slice() != MetricAttribute::ALL {
            return invalid("metric attributes are missing, duplicated, or out of canonical order");
        }
        if self.metric_instruments.len() != METRIC_SPECS.len()
            || self.metric_instruments.len() != MetricInstrument::ALL.len()
        {
            return invalid("metric instrument catalogue length is not canonical");
        }
        for ((descriptor, spec), instrument) in self
            .metric_instruments
            .iter()
            .zip(METRIC_SPECS)
            .zip(MetricInstrument::ALL)
        {
            if spec.instrument != instrument
                || descriptor.instrument != spec.instrument
                || descriptor.kind != spec.kind
                || descriptor.unit != spec.unit
                || descriptor.description != spec.description
                || descriptor.attributes.as_slice() != spec.attributes
            {
                return invalid(format!(
                    "metric descriptor differs from {}",
                    instrument.as_str()
                ));
            }
            if descriptor.attributes.is_empty() {
                return invalid(format!(
                    "metric descriptor {} has no bounded attributes",
                    instrument.as_str()
                ));
            }
        }
        Ok(())
    }

    /// Returns the deterministic content identity only for a valid canonical
    /// catalogue. Invalid projections cannot acquire an apparently trusted
    /// fingerprint.
    pub fn sha256(&self) -> Result<String> {
        self.validate()?;
        Ok(digest::sha256_hex(&catalogue_preimage(self)))
    }
}

fn catalogue_preimage(catalogue: &SignalCatalogue) -> Vec<u8> {
    let mut bytes = Vec::new();
    field(&mut bytes, b"fingerprint_domain");
    field(&mut bytes, FINGERPRINT_DOMAIN);
    field(&mut bytes, b"contract_version");
    number(&mut bytes, u64::from(catalogue.contract_version));
    field(&mut bytes, b"runtime_trace_contract_version");
    number(
        &mut bytes,
        u64::from(catalogue.runtime_trace_contract_version),
    );
    field(&mut bytes, b"default_diagnostic_level");
    field(
        &mut bytes,
        catalogue.default_diagnostic_level.as_str().as_bytes(),
    );
    field(&mut bytes, b"diagnostic_levels");
    collection_len(&mut bytes, catalogue.diagnostic_levels.len());
    for level in &catalogue.diagnostic_levels {
        field(&mut bytes, level.as_str().as_bytes());
    }
    field(&mut bytes, b"default_metric_cardinality_limit");
    number(
        &mut bytes,
        u64::from(catalogue.default_metric_cardinality_limit),
    );
    field(&mut bytes, b"trace_operations");
    collection_len(&mut bytes, catalogue.trace_operations.len());
    for operation in &catalogue.trace_operations {
        field(&mut bytes, operation.name.as_bytes());
        field(&mut bytes, operation.boundary.as_str().as_bytes());
    }
    field(&mut bytes, b"trace_attributes");
    collection_len(&mut bytes, catalogue.trace_attributes.len());
    for attribute in &catalogue.trace_attributes {
        field(&mut bytes, attribute.name.as_bytes());
    }
    field(&mut bytes, b"metric_attributes");
    collection_len(&mut bytes, catalogue.metric_attributes.len());
    for attribute in &catalogue.metric_attributes {
        field(&mut bytes, attribute.as_str().as_bytes());
    }
    field(&mut bytes, b"metric_instruments");
    collection_len(&mut bytes, catalogue.metric_instruments.len());
    for metric in &catalogue.metric_instruments {
        field(&mut bytes, metric.instrument.as_str().as_bytes());
        field(&mut bytes, metric.kind.as_str().as_bytes());
        field(&mut bytes, metric.unit.as_str().as_bytes());
        field(&mut bytes, metric.description.as_bytes());
        collection_len(&mut bytes, metric.attributes.len());
        for attribute in &metric.attributes {
            field(&mut bytes, attribute.as_str().as_bytes());
        }
    }
    bytes
}

fn field(output: &mut Vec<u8>, value: &[u8]) {
    number(output, value.len() as u64);
    output.extend_from_slice(value);
}

fn collection_len(output: &mut Vec<u8>, length: usize) {
    number(output, length as u64);
}

fn number(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn invalid<T>(reason: impl Into<String>) -> Result<T> {
    Err(Error::InvalidRuntime {
        reason: format!("invalid signal catalogue: {}", reason.into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_sha256(catalogue: &SignalCatalogue) -> String {
        digest::sha256_hex(&catalogue_preimage(catalogue))
    }

    #[test]
    fn fingerprint_preimage_is_sensitive_to_every_semantic_section() {
        let catalogue = signal_catalogue();
        let expected = raw_sha256(&catalogue);
        let assert_changed = |mutated: SignalCatalogue| {
            assert_ne!(raw_sha256(&mutated), expected);
        };

        let mut mutated = catalogue.clone();
        mutated.contract_version += 1;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.runtime_trace_contract_version += 1;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.default_diagnostic_level = DiagnosticLevel::Detailed;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.diagnostic_levels.swap(0, 1);
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.default_metric_cardinality_limit += 1;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.trace_operations[0].name.push_str(".changed");
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.trace_operations[0].boundary = TraceBoundary::Engine;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.trace_attributes[0].name.push_str("_changed");
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.metric_attributes.swap(0, 1);
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.metric_instruments[0].instrument = I::OperationCount;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.metric_instruments[0].kind = K::Counter;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.metric_instruments[0].unit = U::Bytes;
        assert_changed(mutated);
        let mut mutated = catalogue.clone();
        mutated.metric_instruments[0]
            .description
            .push_str(" Changed.");
        assert_changed(mutated);
        let mut mutated = catalogue;
        mutated.metric_instruments[0].attributes.swap(0, 1);
        assert_changed(mutated);
    }
}
