//! Another harness's results, restated in this lab's report so a client can show them
//! beside ours. They stay labelled as imported, because this lab did not measure them.
//! Only named result fields are read, so a source's connection settings never enter a report.
use super::driver::now_unix_ms;
use super::report::{
    ArtifactReference, DimensionDefinition, LabEvidenceClass, LabReport, MetricDefinition,
    MetricDirection, Observation, ReportSystem, SystemStatus, TestIdentity, Verdict,
    VerdictOutcome, ViewDefinition, REPORT_SCHEMA_VERSION,
};
use super::{require, sha256_hex, Result};
use crate::storage::{Digest, Id};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub const MAX_IMPORT_BYTES: u64 = 64 * 1024 * 1024;
const NORMAL_LABEL: &str = ":)";
const MAX_SETTING_BYTES: usize = 200;
/// A setting that names any of these is left out, whatever the source masked.
const PRIVATE_SETTING_WORDS: [&str; 6] = ["key", "token", "secret", "password", "url", "note"];

#[derive(Debug, Deserialize)]
struct VdbResult {
    run_id: String,
    results: Vec<VdbCase>,
}

#[derive(Debug, Deserialize)]
struct VdbCase {
    metrics: VdbMetrics,
    task_config: VdbTask,
    label: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct VdbMetrics {
    load_duration: f64,
    qps: f64,
    serial_latency_p50: f64,
    serial_latency_p95: f64,
    serial_latency_p99: f64,
    recall: f64,
    conc_num_list: Vec<u32>,
    conc_qps_list: Vec<f64>,
    conc_latency_p50_list: Vec<f64>,
    conc_latency_p95_list: Vec<f64>,
    conc_latency_p99_list: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct VdbTask {
    db: String,
    db_config: Value,
    db_case_config: Value,
    case_config: VdbCaseConfig,
}

#[derive(Debug, Deserialize)]
struct VdbCaseConfig {
    case_id: Value,
    #[serde(default)]
    custom_case: Value,
    #[serde(default)]
    k: Option<u32>,
}

impl VdbTask {
    fn config_text(&self, field: &str) -> Option<&str> {
        self.db_config
            .get(field)
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
    }

    fn system_id(&self) -> Result<Id> {
        let label = self.config_text("db_label").unwrap_or("default");
        let slug: String = format!("{}-{label}", self.db)
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        Id::try_from(format!("vdbbench-{}", slug.trim_matches('-')))
            .map_err(|_| format!("VectorDBBench system {} has no usable identifier", self.db))
    }

    fn case_label(&self) -> String {
        let case = &self.case_config;
        let id = match &case.case_id {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        };
        let dataset = case
            .custom_case
            .get("dataset_with_size_type")
            .and_then(Value::as_str);
        let filter = case
            .custom_case
            .get("label_percentage")
            .or_else(|| case.custom_case.get("filter_rate"))
            .filter(|value| !value.is_null());
        match (dataset, filter) {
            (Some(dataset), Some(filter)) => format!("{id}, {dataset}, filter {filter}"),
            (Some(dataset), None) => format!("{id}, {dataset}"),
            _ => id,
        }
    }

    fn setting_label(&self) -> String {
        let Some(settings) = self.db_case_config.as_object() else {
            return "default".into();
        };
        let mut parts: Vec<String> = settings
            .iter()
            .filter(|(name, value)| {
                !value.is_null()
                    && !PRIVATE_SETTING_WORDS
                        .iter()
                        .any(|word| name.to_ascii_lowercase().contains(word))
            })
            .map(|(name, value)| match value {
                Value::String(text) => format!("{name}={text}"),
                other => format!("{name}={other}"),
            })
            .collect();
        parts.sort();
        let mut label = parts.join(" ");
        if label.len() > MAX_SETTING_BYTES {
            let mut end = MAX_SETTING_BYTES;
            while !label.is_char_boundary(end) {
                end -= 1;
            }
            label.truncate(end);
        }
        if label.is_empty() {
            "default".into()
        } else {
            label
        }
    }
}

fn id(text: &str) -> Result<Id> {
    Id::try_from(text.to_string())
}

fn metric(
    id_text: &str,
    label: &str,
    unit: &str,
    better: MetricDirection,
) -> Result<MetricDefinition> {
    Ok(MetricDefinition {
        id: id(id_text)?,
        label: label.into(),
        unit: unit.into(),
        better,
    })
}

fn labels(entries: &[(&str, String)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), value.clone()))
        .collect()
}

fn push(
    list: &mut Vec<Observation>,
    system: &Id,
    metric_id: &str,
    case: &BTreeMap<String, String>,
    value: f64,
) -> Result<()> {
    list.push(Observation {
        system: system.clone(),
        metric: id(metric_id)?,
        case: case.clone(),
        value,
        // The source reports a single figure, not its samples.
        samples: 1,
        low: None,
        high: None,
    });
    Ok(())
}

fn case_observations(
    case: &VdbCase,
    system: &Id,
    recall_target: f64,
    list: &mut Vec<Observation>,
) -> Result<()> {
    let metrics = &case.metrics;
    let name = case.task_config.case_label();
    let setting = case.task_config.setting_label();
    let serial = labels(&[
        ("case", name.clone()),
        ("setting", setting.clone()),
        ("concurrency", "serial".to_string()),
    ]);
    push(list, system, "recall_at_k", &serial, metrics.recall)?;
    for (metric_id, seconds) in [
        ("latency_p50_ms", metrics.serial_latency_p50),
        ("latency_p95_ms", metrics.serial_latency_p95),
        ("latency_p99_ms", metrics.serial_latency_p99),
    ] {
        if seconds > 0.0 {
            push(list, system, metric_id, &serial, seconds * 1000.0)?;
        }
    }
    if metrics.load_duration > 0.0 {
        let load = labels(&[("case", name.clone()), ("setting", setting.clone())]);
        push(list, system, "load_seconds", &load, metrics.load_duration)?;
    }
    for (position, clients) in metrics.conc_num_list.iter().enumerate() {
        let case_labels = labels(&[
            ("case", name.clone()),
            ("setting", setting.clone()),
            ("concurrency", clients.to_string()),
        ]);
        for (metric_id, values, scale) in [
            ("queries_per_second", &metrics.conc_qps_list, 1.0),
            ("latency_p50_ms", &metrics.conc_latency_p50_list, 1000.0),
            ("latency_p95_ms", &metrics.conc_latency_p95_list, 1000.0),
            ("latency_p99_ms", &metrics.conc_latency_p99_list, 1000.0),
        ] {
            if let Some(value) = values.get(position) {
                push(list, system, metric_id, &case_labels, value * scale)?;
            }
        }
    }
    if metrics.recall >= recall_target && metrics.qps > 0.0 {
        let best = labels(&[("case", name), ("setting", setting)]);
        push(list, system, "qps_at_recall_target", &best, metrics.qps)?;
    }
    Ok(())
}

fn report_views() -> Result<Vec<ViewDefinition>> {
    Ok(vec![
        ViewDefinition::Bars {
            id: id("matched-quality")?,
            title: "Best speed at target recall, as VectorDBBench measured it".into(),
            metric: id("qps_at_recall_target")?,
            fixed: BTreeMap::new(),
        },
        ViewDefinition::Bars {
            id: id("load")?,
            title: "Time to load and optimize, as VectorDBBench measured it".into(),
            metric: id("load_seconds")?,
            fixed: BTreeMap::new(),
        },
        ViewDefinition::Table {
            id: id("all-cases")?,
            title: "Every imported case".into(),
            metrics: ["recall_at_k", "queries_per_second", "latency_p99_ms"]
                .into_iter()
                .map(id)
                .collect::<Result<_>>()?,
            rows: ["case", "setting", "concurrency"]
                .into_iter()
                .map(id)
                .collect::<Result<_>>()?,
            fixed: BTreeMap::new(),
        },
    ])
}

/// Restates one VectorDBBench result file. `source_path` is recorded, not read.
pub fn import_vectordbbench(
    bytes: &[u8],
    source_path: &str,
    harness_version: &str,
    recall_target: f64,
) -> Result<LabReport> {
    require(
        bytes.len() as u64 <= MAX_IMPORT_BYTES,
        "the file to import exceeds its size bound",
    )?;
    require(
        recall_target > 0.0 && recall_target <= 1.0,
        "recall target must be in (0, 1]",
    )?;
    require(
        !harness_version.trim().is_empty(),
        "name the harness version",
    )?;
    let source: VdbResult = serde_json::from_slice(bytes)
        .map_err(|error| format!("not a VectorDBBench result: {error}"))?;
    require(!source.results.is_empty(), "the file holds no results")?;

    let mut systems: BTreeMap<Id, ReportSystem> = BTreeMap::new();
    let mut failed: BTreeMap<Id, u64> = BTreeMap::new();
    let mut best: BTreeMap<Id, f64> = BTreeMap::new();
    let mut observations = Vec::new();
    let mut k_values = Vec::new();
    for case in &source.results {
        let task = &case.task_config;
        let system = task.system_id()?;
        systems
            .entry(system.clone())
            .or_insert_with(|| ReportSystem {
                id: system.clone(),
                label: format!(
                    "{} ({}), measured by VectorDBBench",
                    task.db,
                    task.config_text("db_label").unwrap_or("default")
                ),
                kind: "imported:vectordbbench".into(),
                status: SystemStatus::Failed,
                reason: Some("VectorDBBench reported no successful case".into()),
                observed_version: task.config_text("version").map(str::to_string),
                configuration: BTreeMap::from([(
                    "harness".to_string(),
                    format!("VectorDBBench {harness_version}"),
                )]),
            });
        if case.label != NORMAL_LABEL {
            *failed.entry(system).or_default() += 1;
            continue;
        }
        if let Some(entry) = systems.get_mut(&system) {
            entry.status = SystemStatus::Completed;
            entry.reason = None;
        }
        if let Some(k) = task.case_config.k {
            if !k_values.contains(&k) {
                k_values.push(k);
            }
        }
        let current = best.entry(system.clone()).or_insert(0.0);
        *current = current.max(case.metrics.recall);
        case_observations(case, &system, recall_target, &mut observations)?;
    }
    for (system, count) in &failed {
        if systems
            .get(system)
            .is_some_and(|entry| entry.status == SystemStatus::Completed)
        {
            push(
                &mut observations,
                system,
                "failed_cases",
                &BTreeMap::new(),
                *count as f64,
            )?;
        }
    }
    let verdicts = systems
        .values()
        .map(|system| {
            let recall = best.get(&system.id);
            Ok(Verdict {
                id: Id::try_from(format!("imported-recall-{}", system.id.as_str()))?,
                question: format!(
                    "Did VectorDBBench report recall of {recall_target} in any successful case?"
                ),
                system: Some(system.id.clone()),
                outcome: match recall {
                    Some(value) if *value >= recall_target => VerdictOutcome::Met,
                    Some(_) => VerdictOutcome::NotMet,
                    None => VerdictOutcome::NotRun,
                },
                basis: recall.map_or("no successful case".to_string(), |value| {
                    format!("best reported recall {value:.4}")
                }),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    use MetricDirection::{Higher, Lower};
    let digest = sha256_hex(bytes);
    let mut limits = vec![
        format!(
            "Imported from VectorDBBench {harness_version}. That harness measured these numbers with its own dataset, clients and settings, and this report only restates them."
        ),
        "Use them as a cross-check on direction. They do not form one ranking with a lab test, whose corpus and clients differ.".to_string(),
        "VectorDBBench takes recall from serial search and speed from its concurrent stage, so a recall and a speed in one row were measured separately.".to_string(),
    ];
    if !k_values.is_empty() {
        k_values.sort_unstable();
        limits.push(format!("Results use k of {k_values:?}."));
    }
    let report = LabReport {
        schema_version: REPORT_SCHEMA_VERSION,
        test: TestIdentity {
            id: Id::try_from(format!(
                "vdbbench-{}",
                source
                    .run_id
                    .to_ascii_lowercase()
                    .chars()
                    .take(32)
                    .collect::<String>()
            ))
            .map_err(|_| "the source's run id is not usable as an identifier".to_string())?,
            title: format!("VectorDBBench {harness_version} results, imported"),
            family: id("vector-search")?,
            revision: 1,
        },
        evidence_class: LabEvidenceClass::Imported,
        generated_by: format!("rrflow-lab {} import", env!("CARGO_PKG_VERSION")),
        generated_at_unix_ms: now_unix_ms(),
        environment: BTreeMap::new(),
        identities: BTreeMap::from([("source".to_string(), Digest::try_from(digest.clone())?)]),
        systems: systems.into_values().collect(),
        dimensions: [
            ("case", "VectorDBBench case"),
            ("setting", "System settings"),
            ("concurrency", "Concurrent clients"),
        ]
        .into_iter()
        .map(|(key, label)| {
            Ok(DimensionDefinition {
                id: id(key)?,
                label: label.into(),
            })
        })
        .collect::<Result<_>>()?,
        metrics: vec![
            metric("recall_at_k", "Recall at k", "fraction", Higher)?,
            metric("queries_per_second", "Queries per second", "1/s", Higher)?,
            metric(
                "qps_at_recall_target",
                "Queries per second at target recall",
                "1/s",
                Higher,
            )?,
            metric("latency_p50_ms", "Latency p50", "ms", Lower)?,
            metric("latency_p95_ms", "Latency p95", "ms", Lower)?,
            metric("latency_p99_ms", "Latency p99", "ms", Lower)?,
            metric("load_seconds", "Time to load and optimize", "s", Lower)?,
            metric(
                "failed_cases",
                "Cases VectorDBBench marked failed",
                "count",
                Lower,
            )?,
        ],
        observations,
        verdicts,
        views: report_views()?,
        limits,
        artifacts: vec![ArtifactReference {
            path: source_path.to_string(),
            sha256: Digest::try_from(digest)?,
            bytes: bytes.len() as u64,
            description: "the VectorDBBench result file these numbers came from".into(),
        }],
    };
    report.validate()?;
    Ok(report)
}
