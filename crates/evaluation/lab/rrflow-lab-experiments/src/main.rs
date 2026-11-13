//! Controlled paired evaluation runner. Provider output is evidence, not prose.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Trial {
    provider: String,
    repository: String,
    scenario: String,
    arm: String,
    success: bool,
    expected: String,
    attempts: u64,
    #[serde(default)]
    retries: u64,
    tool_calls: Option<u64>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    latency_ms: u64,
    exit_code: Option<i32>,
    raw_output: String,
    stderr: String,
}

/// Mechanical screening only: this is not an independent quality oracle or
/// proof that the named treatment was actually attached to the provider.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScreeningSummary {
    format_version: u16,
    evidence_class: String,
    trials: usize,
    successes: usize,
    paired_regressions: usize,
    providers: Vec<String>,
    repositories: Vec<String>,
    trials_detail: Vec<Trial>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("rrflow-lab-experiments: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("run") => run_trial(&args[2..]),
        Some("summarize") => summarize(
            Path::new(args.get(2).ok_or("result directory required")?),
            args.get(3).map(Path::new),
        ),
        Some("verify") => verify(Path::new(args.get(2).ok_or("summary file required")?)),
        _ => Err("usage: rrflow-lab-experiments run <provider> <repo> <scenario> <arm> <expected> <prompt-file> <output-file> | summarize <result-dir> [new-summary-file] | verify <summary-file>".into()),
    }
}

fn run_trial(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() != 7 {
        return Err("run expects seven arguments".into());
    }
    let provider = &args[0];
    let repo = PathBuf::from(&args[1]);
    let scenario = &args[2];
    let arm = &args[3];
    let expected = &args[4];
    if expected.trim().is_empty()
        || scenario.trim().is_empty()
        || !matches!(arm.as_str(), "baseline" | "runtime")
    {
        return Err(
            "screening requires a nonempty scenario and expected string and a valid arm".into(),
        );
    }
    let prompt = std::fs::read_to_string(&args[5])?;
    let output_file = PathBuf::from(&args[6]);
    let started = Instant::now();
    let output = match provider.as_str() {
        "codex" => Command::new("codex")
            .args([
                "exec",
                "--ephemeral",
                "--sandbox",
                "read-only",
                "--json",
                "--skip-git-repo-check",
                "--cd",
            ])
            .arg(&repo)
            .arg(&prompt)
            .output()?,
        "claude" => Command::new("claude")
            .args([
                "-p",
                "--permission-mode",
                "plan",
                "--allowedTools",
                "Read,Grep,Glob",
                "--output-format",
                "json",
                "--no-session-persistence",
            ])
            .arg(&prompt)
            .current_dir(&repo)
            .output()?,
        "gemini" => Command::new("gemini")
            .args([
                "--prompt",
                &prompt,
                "--approval-mode",
                "plan",
                "--output-format",
                "json",
                "--skip-trust",
            ])
            .current_dir(&repo)
            .output()?,
        other => return Err(format!("unsupported provider {other:?}").into()),
    };
    let latency_ms = started.elapsed().as_millis() as u64;
    let raw_output = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let (input_tokens, output_tokens, tool_calls) = metrics(provider, &raw_output);
    let trial = Trial {
        provider: provider.clone(),
        repository: repo.display().to_string(),
        scenario: scenario.clone(),
        arm: arm.clone(),
        success: output.status.success() && raw_output.contains(expected),
        expected: expected.clone(),
        attempts: 1,
        retries: 0,
        tool_calls,
        input_tokens,
        output_tokens,
        latency_ms,
        exit_code: output.status.code(),
        raw_output,
        stderr,
    };
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_file)?;
    file.write_all(&serde_json::to_vec_pretty(&trial)?)?;
    println!(
        "{} {} {} {} success={} latency={}ms",
        provider,
        scenario,
        arm,
        repo.display(),
        trial.success,
        trial.latency_ms
    );
    Ok(())
}

fn metrics(provider: &str, output: &str) -> (Option<u64>, Option<u64>, Option<u64>) {
    match provider {
        "claude" | "gemini" => {
            let value: Value = serde_json::from_str(output).unwrap_or(Value::Null);
            let usage = value.get("usage").or_else(|| {
                value
                    .get("stats")
                    .and_then(|v| v.get("models"))
                    .and_then(Value::as_object)
                    .and_then(|models| {
                        (models.len() == 1)
                            .then(|| models.values().next())
                            .flatten()
                    })
            });
            // Provider-native counters have different cache semantics; do not
            // add cache fields or infer a zero when the field is missing.
            let input = usage
                .and_then(|u| u.get("input_tokens").or_else(|| u.get("inputTokens")))
                .and_then(Value::as_u64);
            let output_tokens = usage
                .and_then(|u| u.get("output_tokens").or_else(|| u.get("outputTokens")))
                .and_then(Value::as_u64);
            // Model turns are not tool calls, and absence is not zero.
            (input, output_tokens, None)
        }
        "codex" => {
            let values: Vec<Value> = output
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            let usage = values.iter().find_map(|value| value.get("usage"));
            let input = usage
                .and_then(|u| u.get("input_tokens"))
                .and_then(Value::as_u64);
            let output_tokens = usage
                .and_then(|u| u.get("output_tokens"))
                .and_then(Value::as_u64);
            // A command-event count is not a complete provider tool count.
            (input, output_tokens, None)
        }
        _ => (None, None, None),
    }
}

const MAX_SCREENING_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_SCREENING_TRIALS: usize = 256;

fn read_screening_file(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_SCREENING_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SCREENING_FILE_BYTES {
        return Err("screening file exceeds 8 MiB".into());
    }
    Ok(bytes)
}

fn validate_trial(trial: &Trial) -> Result<(), Box<dyn std::error::Error>> {
    if trial.provider.trim().is_empty()
        || trial.repository.trim().is_empty()
        || trial.scenario.trim().is_empty()
        || !matches!(trial.arm.as_str(), "baseline" | "runtime")
        || trial.expected.trim().is_empty()
        || trial.attempts
            != trial
                .retries
                .checked_add(1)
                .ok_or("attempt count overflow")?
        || trial.success
            != (trial.exit_code == Some(0) && trial.raw_output.contains(&trial.expected))
    {
        return Err("invalid or inconsistent screening trial".into());
    }
    Ok(())
}

fn paired_regressions(trials: &[Trial]) -> Result<usize, Box<dyn std::error::Error>> {
    if trials.is_empty() {
        return Err("screening has no trials".into());
    }
    let mut pairs: BTreeMap<(&str, &str, &str), [Option<bool>; 2]> = BTreeMap::new();
    for trial in trials {
        validate_trial(trial)?;
        let pair = pairs
            .entry((&trial.provider, &trial.repository, &trial.scenario))
            .or_default();
        let arm = usize::from(trial.arm == "runtime");
        if pair[arm].replace(trial.success).is_some() {
            return Err("duplicate provider/repository/scenario arm".into());
        }
    }
    if pairs.values().any(|pair| pair.iter().any(Option::is_none)) {
        return Err("screening has an incomplete provider/repository/scenario pair".into());
    }
    Ok(pairs
        .values()
        .filter(|pair| pair[0] == Some(true) && pair[1] == Some(false))
        .count())
}

fn screening_summary(
    mut trials: Vec<Trial>,
) -> Result<ScreeningSummary, Box<dyn std::error::Error>> {
    if trials.len() < 8 || trials.len() > MAX_SCREENING_TRIALS {
        return Err("screening summary requires 8 through 256 trials".into());
    }
    trials.sort_by(|a, b| {
        (&a.provider, &a.repository, &a.scenario, &a.arm).cmp(&(
            &b.provider,
            &b.repository,
            &b.scenario,
            &b.arm,
        ))
    });
    let mut providers = BTreeSet::new();
    let mut repositories = BTreeSet::new();
    for trial in &trials {
        providers.insert(trial.provider.clone());
        repositories.insert(trial.repository.clone());
    }
    if providers.len() < 2 || repositories.len() < 2 {
        return Err("screening requires at least two distinct providers and repositories".into());
    }
    let paired_regressions = paired_regressions(&trials)?;
    Ok(ScreeningSummary {
        format_version: 1,
        evidence_class: "provider_screening_only".into(),
        trials: trials.len(),
        successes: trials.iter().filter(|trial| trial.success).count(),
        paired_regressions,
        providers: providers.into_iter().collect(),
        repositories: repositories.into_iter().collect(),
        trials_detail: trials,
    })
}

fn summarize(dir: &Path, output: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let mut trials = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            if trials.len() == MAX_SCREENING_TRIALS {
                return Err("screening exceeds 256 trials".into());
            }
            let trial: Trial = serde_json::from_slice(&read_screening_file(&path)?)?;
            validate_trial(&trial)?;
            trials.push(trial);
        }
    }
    trials.sort_by(|a, b| {
        (&a.provider, &a.repository, &a.scenario, &a.arm).cmp(&(
            &b.provider,
            &b.repository,
            &b.scenario,
            &b.arm,
        ))
    });
    let success = trials.iter().filter(|trial| trial.success).count();
    let regressions = paired_regressions(&trials)?;
    let latency = trials.iter().try_fold(0u64, |total, trial| {
        total
            .checked_add(trial.latency_ms)
            .ok_or("latency overflow")
    })?;
    let known_token_pairs = trials
        .iter()
        .filter(|trial| trial.input_tokens.zip(trial.output_tokens).is_some())
        .count();
    if let Some(path) = output {
        let report = screening_summary(trials.clone())?;
        let bytes = serde_json::to_vec_pretty(&report)?;
        if bytes.len() as u64 > MAX_SCREENING_FILE_BYTES {
            return Err("screening summary exceeds 8 MiB".into());
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(&bytes)?;
    }
    println!(
        "trials={} success={}/{} regressions={} total_latency_ms={} known_token_pairs={}/{}",
        trials.len(),
        success,
        trials.len(),
        regressions,
        latency,
        known_token_pairs,
        trials.len()
    );
    println!("screening only: treatment and answer correctness not verified; provider token counters are not aggregated");
    for trial in trials {
        println!(
            "{} {} {} success={} retries={} tools={:?} tokens={:?}+{:?} latency={}ms",
            trial.provider,
            trial.scenario,
            trial.arm,
            trial.success,
            trial.retries,
            trial.tool_calls,
            trial.input_tokens,
            trial.output_tokens,
            trial.latency_ms
        );
    }
    Ok(())
}

fn verify(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let report: ScreeningSummary = serde_json::from_slice(&read_screening_file(path)?)?;
    let expected = screening_summary(report.trials_detail)?;
    if report.format_version != expected.format_version
        || report.evidence_class != expected.evidence_class
        || report.trials != expected.trials
        || report.successes != expected.successes
        || report.paired_regressions != expected.paired_regressions
        || report.providers != expected.providers
        || report.repositories != expected.repositories
    {
        return Err("screening summary disagrees with its trial records".into());
    }
    println!("verified screening consistency: trials={} successes={} regressions={}; treatment and answer correctness not verified", expected.trials, expected.successes, expected.paired_regressions);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::metrics;

    #[test]
    fn unreported_provider_usage_and_tool_count_are_unknown() {
        assert_eq!(
            metrics("claude", r#"{"usage":{},"num_turns":4}"#),
            (None, None, None)
        );
        assert_eq!(metrics("codex", ""), (None, None, None));
        assert_eq!(
            metrics(
                "claude",
                r#"{"usage":{"input_tokens":10,"cache_read_input_tokens":7,"output_tokens":3}}"#
            ),
            (Some(10), Some(3), None)
        );
    }
}
