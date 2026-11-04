//! What others have published, beside what this lab measured. A published number is a claim
//! someone else made, so each row carries who measured it, what kind of source that is, and the
//! exact words it was read from. Nothing is ranked unless every condition matches.
use super::report::LabReport;
use super::Result;
use crate::storage::Id;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const REFERENCE_SCHEMA_VERSION: u16 = 1;
pub const STALE_AFTER_DAYS: u64 = 60;
pub const ANSWERS_NOT_MEASURED: &str = "This lab has not measured answer quality. Measuring it takes a reader model and a judge model, and none has been approved.";

const NOT_BUILT: &str = "not built yet";

/// Who stands behind a number, in the terms a reader needs to weigh it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Independent,
    PeerReviewed,
    VendorOwn,
    Competitor,
    Authors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HardwareClass {
    SingleCore,
    MultiCoreServer,
    Gpu,
    ManagedCloud,
    NotStated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceOrigin {
    pub url: String,
    /// The exact words or data row this was read from.
    pub quote: String,
    pub measured_by: String,
    pub provenance: Provenance,
    /// `YYYY-MM-DD`. Absent when the source gives no day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SeekResult {
    pub system: String,
    pub queries_per_second: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_p99_ms: Option<f64>,
    pub recall: f64,
    pub dataset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub k: Option<u32>,
    #[serde(default)]
    pub filtered: bool,
    pub hardware: String,
    pub hardware_class: HardwareClass,
    pub origin: ReferenceOrigin,
    /// What a reader must know before using this number.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnswerResult {
    pub system: String,
    pub score: f64,
    pub score_name: String,
    pub benchmark: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reader_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge_model: Option<String>,
    pub origin: ReferenceOrigin,
    /// What a reader must know before using this number, including any dispute.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSource {
    pub id: Id,
    pub title: String,
    pub url: String,
    /// `YYYY-MM-DD`, the day these rows were last read at their source.
    pub checked_on: String,
}

/// One source's published results. One file per source, so its update is a small patch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceFile {
    pub schema_version: u16,
    pub source: ReferenceSource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seek: Vec<SeekResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub answers: Vec<AnswerResult>,
}

impl ReferenceFile {
    pub fn validate(&self) -> Result<()> {
        Err(NOT_BUILT.into())
    }
}

/// What a published result must match to be set beside this test.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct TestConditions {
    pub test: Id,
    pub title: String,
    /// The name others publish this data under. Generated data has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<String>,
    pub k: u32,
    pub recall_target: f64,
    pub hardware_class: HardwareClass,
}

/// One system's speed at the recall target, from the report.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct MeasuredSpeed {
    pub system: Id,
    pub label: String,
    pub filtered: bool,
    pub concurrency: u32,
    pub queries_per_second: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_p99_ms: Option<f64>,
}

/// A system with no speed to show, and why, so a gap cannot read as a result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct WithoutSpeed {
    pub system: Id,
    pub label: String,
    pub filtered: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct ComparedSeek {
    pub source: Id,
    pub source_title: String,
    pub checked_on: String,
    /// True when this source has not been reread lately.
    pub stale: bool,
    /// True only when nothing below differs.
    pub same_conditions: bool,
    /// Each way this differs from the test, in words.
    pub differs: Vec<String>,
    pub result: SeekResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct ListedAnswer {
    pub source: Id,
    pub source_title: String,
    pub checked_on: String,
    pub stale: bool,
    pub result: AnswerResult,
}

/// There is deliberately no winner and no rank here. It sets numbers side by side and says what
/// differs, and a reader draws the conclusion.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Comparison {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<TestConditions>,
    pub measured: Vec<MeasuredSpeed>,
    pub without_speed: Vec<WithoutSpeed>,
    pub published_seek: Vec<ComparedSeek>,
    pub answers_measured: bool,
    pub answers_note: String,
    pub published_answers: Vec<ListedAnswer>,
}

/// Sets a report, if there is one, beside what is published.
pub fn compare(
    _files: &[ReferenceFile],
    _conditions: Option<&TestConditions>,
    _report: Option<&LabReport>,
    _now_unix_s: u64,
) -> Result<Comparison> {
    Err(NOT_BUILT.into())
}
