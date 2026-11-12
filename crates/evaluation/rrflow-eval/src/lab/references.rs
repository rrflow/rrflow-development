//! What others have published, beside what this lab measured. A published number is a claim
//! someone else made, so each row carries who measured it, what kind of source that is, and the
//! exact words it was read from. Nothing is ranked unless every condition matches.
use super::{require, Result};
use crate::storage::Id;
use rrflow_evaluation_core::report::{LabReport, SystemStatus};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const REFERENCE_SCHEMA_VERSION: u16 = 1;
pub const STALE_AFTER_DAYS: u64 = 60;
pub const ANSWERS_NOT_MEASURED: &str = "This lab has not measured answer quality. Measuring it takes a reader model and a judge model, and none has been approved.";

/// Within this of the target counts as the same recall, since results land near a target.
const RECALL_MATCH: f64 = 0.01;
const MAX_RESULTS_PER_SOURCE: usize = 256;
const MAX_TEXT_BYTES: usize = 2_048;
const DAY_SECONDS: u64 = 86_400;

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

impl Provenance {
    /// How a reader should hear it.
    pub fn words(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::PeerReviewed => "peer reviewed",
            Self::VendorOwn => "the vendor's own",
            Self::Competitor => "run by a competitor",
            Self::Authors => "the benchmark's or system's own authors",
        }
    }
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

impl HardwareClass {
    /// How a reader should hear it.
    pub fn words(self) -> &'static str {
        match self {
            Self::SingleCore => "one CPU core",
            Self::MultiCoreServer => "a multi-core server",
            Self::Gpu => "a GPU",
            Self::ManagedCloud => "a managed cloud service",
            Self::NotStated => "hardware not stated",
        }
    }
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

fn is_false(value: &bool) -> bool {
    !value
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
    /// True when the score or its benchmark is publicly contested. The limits say how.
    #[serde(default, skip_serializing_if = "is_false")]
    pub disputed: bool,
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

fn text(value: &str, label: &str) -> Result<()> {
    require(
        !value.trim().is_empty() && value.len() <= MAX_TEXT_BYTES,
        &format!("{label} must be nonempty and bounded"),
    )
}

fn secure_url(value: &str, label: &str) -> Result<()> {
    text(value, label)?;
    require(
        value.starts_with("https://"),
        &format!("{label} must be an https address"),
    )
}

/// The day number of a `YYYY-MM-DD` date, using the civil calendar.
fn day_number(date: &str) -> Option<i64> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year: i64 = date[..4].parse().ok()?;
    let month: i64 = date[5..7].parse().ok()?;
    let day: i64 = date[8..10].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let shifted = if month <= 2 { year - 1 } else { year };
    let era = shifted.div_euclid(400);
    let year_of_era = shifted.rem_euclid(400);
    let month_index = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

fn date(value: &str, label: &str) -> Result<()> {
    require(
        day_number(value).is_some(),
        &format!("{label} must be a YYYY-MM-DD date"),
    )
}

impl ReferenceOrigin {
    fn validate(&self, system: &str) -> Result<()> {
        secure_url(&self.url, &format!("{system}'s source"))?;
        text(&self.quote, &format!("{system}'s quote"))?;
        text(&self.measured_by, &format!("who measured {system}"))?;
        if let Some(published) = &self.published {
            date(published, &format!("{system}'s publication date"))?;
        }
        Ok(())
    }
}

impl SeekResult {
    fn validate(&self) -> Result<()> {
        text(&self.system, "a system's name")?;
        let name = &self.system;
        self.origin.validate(name)?;
        text(&self.dataset, &format!("{name}'s dataset"))?;
        text(&self.hardware, &format!("{name}'s hardware"))?;
        require(
            self.queries_per_second.is_finite() && self.queries_per_second > 0.0,
            &format!("{name}'s queries per second must be positive"),
        )?;
        require(
            self.recall.is_finite() && self.recall > 0.0 && self.recall <= 1.0,
            &format!("{name}'s recall must be a fraction, so 0.95 and not 95"),
        )?;
        require(
            self.latency_p99_ms
                .is_none_or(|latency| latency.is_finite() && latency > 0.0),
            &format!("{name}'s latency must be positive"),
        )?;
        require(
            self.k.is_none_or(|k| k > 0),
            &format!("{name}'s top-k must be positive"),
        )?;
        self.limits
            .iter()
            .try_for_each(|limit| text(limit, &format!("{name}'s limit")))
    }
}

impl AnswerResult {
    fn validate(&self) -> Result<()> {
        text(&self.system, "a system's name")?;
        let name = &self.system;
        self.origin.validate(name)?;
        text(&self.score_name, &format!("what {name}'s score means"))?;
        text(&self.benchmark, &format!("{name}'s benchmark"))?;
        require(
            self.score.is_finite() && self.score >= 0.0,
            &format!("{name}'s score must not be negative"),
        )?;
        self.limits
            .iter()
            .try_for_each(|limit| text(limit, &format!("{name}'s limit")))
    }
}

impl ReferenceFile {
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == REFERENCE_SCHEMA_VERSION,
            "unsupported reference schema version",
        )?;
        text(&self.source.title, "a source's title")?;
        secure_url(&self.source.url, "a source's address")?;
        date(&self.source.checked_on, "the day a source was checked")?;
        let rows = self.seek.len() + self.answers.len();
        require(
            (1..=MAX_RESULTS_PER_SOURCE).contains(&rows),
            "a reference source holds between one and 256 results",
        )?;
        self.seek.iter().try_for_each(SeekResult::validate)?;
        self.answers.iter().try_for_each(AnswerResult::validate)
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

/// Each way a published row differs from the test. Anything unstated counts, because a reader
/// cannot assume it matches.
fn differences(result: &SeekResult, conditions: Option<&TestConditions>) -> Vec<String> {
    let Some(conditions) = conditions else {
        return vec!["no test was named to set it beside".to_owned()];
    };
    let mut differs = Vec::new();
    match &conditions.dataset {
        Some(dataset) if *dataset == result.dataset => {}
        Some(dataset) => differs.push(format!(
            "its data is {}, and this test uses {dataset}",
            result.dataset
        )),
        None => differs.push(format!(
            "its data is {}, and this test's data is generated, so nothing published uses it",
            result.dataset
        )),
    }
    match result.k {
        Some(k) if k == conditions.k => {}
        Some(k) => differs.push(format!(
            "it asks for the top {k}, and this test the top {}",
            conditions.k
        )),
        None => differs.push(format!(
            "its top-k is not stated, and this test uses {}",
            conditions.k
        )),
    }
    if (result.recall - conditions.recall_target).abs() > RECALL_MATCH {
        differs.push(format!(
            "its recall is {}, and this test targets {}",
            result.recall, conditions.recall_target
        ));
    }
    if result.filtered {
        differs.push("it is a filtered search, and this sets it beside unfiltered ones".to_owned());
    }
    if result.hardware_class != conditions.hardware_class {
        differs.push(format!(
            "it ran on {}, and this test on {}",
            result.hardware_class.words(),
            conditions.hardware_class.words()
        ));
    }
    differs
}

fn is_stale(checked_on: &str, now_unix_s: u64) -> bool {
    let today = i64::try_from(now_unix_s / DAY_SECONDS).unwrap_or(i64::MAX);
    day_number(checked_on).is_none_or(|day| today - day > STALE_AFTER_DAYS as i64)
}

fn label_of<'a>(case: &'a BTreeMap<String, String>, key: &str) -> &'a str {
    case.get(key).map_or("", String::as_str)
}

/// Speed comes only from `qps_at_recall_target`, which the report holds only for error-free cases
/// that met the declared target. Its p99 is the one from the setting that reached that speed.
fn read_speeds(report: &LabReport, target: Option<f64>) -> (Vec<MeasuredSpeed>, Vec<WithoutSpeed>) {
    let target_words = target.map_or("the recall target".to_owned(), |value| {
        format!("recall {value}")
    });
    let mut measured = Vec::new();
    let mut without = Vec::new();
    for system in &report.systems {
        let label = || system.label.clone();
        if system.status != SystemStatus::Completed {
            without.push(WithoutSpeed {
                system: system.id.clone(),
                label: label(),
                filtered: false,
                reason: system
                    .reason
                    .clone()
                    .unwrap_or_else(|| "it did not finish".to_owned()),
            });
            continue;
        }
        let own: Vec<_> = report
            .observations
            .iter()
            .filter(|item| item.system == system.id)
            .collect();
        let mut speeds: Vec<MeasuredSpeed> = own
            .iter()
            .filter(|item| item.metric.as_str() == "qps_at_recall_target")
            .map(|item| {
                let filter = label_of(&item.case, "filter");
                let concurrency = label_of(&item.case, "concurrency");
                let setting = own.iter().find(|other| {
                    other.metric.as_str() == "queries_per_second"
                        && label_of(&other.case, "filter") == filter
                        && label_of(&other.case, "concurrency") == concurrency
                        && other.value == item.value
                });
                let latency = setting.and_then(|setting| {
                    own.iter().find(|other| {
                        other.metric.as_str() == "latency_p99_ms" && other.case == setting.case
                    })
                });
                MeasuredSpeed {
                    system: system.id.clone(),
                    label: label(),
                    filtered: filter != "none",
                    concurrency: concurrency.parse().unwrap_or(0),
                    queries_per_second: item.value,
                    latency_p99_ms: latency.map(|item| item.value),
                }
            })
            .collect();
        speeds.sort_by_key(|speed| (speed.filtered, speed.concurrency));
        let tried = |filtered: bool| {
            own.iter().any(|item| {
                let filter = label_of(&item.case, "filter");
                !filter.is_empty() && (filter != "none") == filtered
            })
        };
        for filtered in [false, true] {
            if tried(filtered) && !speeds.iter().any(|speed| speed.filtered == filtered) {
                without.push(WithoutSpeed {
                    system: system.id.clone(),
                    label: label(),
                    filtered,
                    reason: format!(
                        "it never reached {target_words} without errors in {} search",
                        if filtered { "filtered" } else { "unfiltered" }
                    ),
                });
            }
        }
        measured.extend(speeds);
    }
    (measured, without)
}

/// Sets a report, if there is one, beside what is published.
pub fn compare(
    files: &[ReferenceFile],
    conditions: Option<&TestConditions>,
    report: Option<&LabReport>,
    now_unix_s: u64,
) -> Result<Comparison> {
    let (measured, without_speed) = report.map_or((Vec::new(), Vec::new()), |report| {
        read_speeds(report, conditions.map(|item| item.recall_target))
    });
    let mut published_seek = Vec::new();
    let mut published_answers = Vec::new();
    for file in files {
        let stale = is_stale(&file.source.checked_on, now_unix_s);
        for result in &file.seek {
            let differs = differences(result, conditions);
            published_seek.push(ComparedSeek {
                source: file.source.id.clone(),
                source_title: file.source.title.clone(),
                checked_on: file.source.checked_on.clone(),
                stale,
                same_conditions: differs.is_empty(),
                differs,
                result: result.clone(),
            });
        }
        for result in &file.answers {
            published_answers.push(ListedAnswer {
                source: file.source.id.clone(),
                source_title: file.source.title.clone(),
                checked_on: file.source.checked_on.clone(),
                stale,
                result: result.clone(),
            });
        }
    }
    // By how much differs and never by speed, so the order cannot read as a ranking.
    published_seek.sort_by_key(|row| row.differs.len());
    Ok(Comparison {
        conditions: conditions.cloned(),
        measured,
        without_speed,
        published_seek,
        answers_measured: false,
        answers_note: ANSWERS_NOT_MEASURED.to_owned(),
        published_answers,
    })
}
