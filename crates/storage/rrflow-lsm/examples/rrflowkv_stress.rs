#[path = "../tests/support/projected_read_model.rs"]
mod projected_read_model;

use projected_read_model::{run_seeded_scenarios, ScenarioConfig};
use std::ffi::OsString;
use std::path::PathBuf;

const USAGE: &str = "usage: rrflowkv_stress --seed <u64|0xhex> --cases <usize> --operations <usize> [--retained-root <path>]";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Arguments {
    seed: u64,
    cases: usize,
    operations: usize,
    retained_root: Option<PathBuf>,
}

impl Arguments {
    fn parse() -> Result<Self, String> {
        Self::parse_from(std::env::args_os().skip(1))
    }

    fn parse_from(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut arguments = arguments.into_iter();
        let mut seed = None;
        let mut cases = None;
        let mut operations = None;
        let mut retained_root = None;

        while let Some(flag) = arguments.next() {
            let value = arguments
                .next()
                .ok_or_else(|| format!("{} requires a value\n{USAGE}", flag.to_string_lossy()))?;
            match flag.to_str() {
                Some("--seed") => set_once(
                    &mut seed,
                    parse_seed(&value)?,
                    "--seed may be supplied only once",
                )?,
                Some("--cases") => set_once(
                    &mut cases,
                    parse_positive_usize("--cases", &value)?,
                    "--cases may be supplied only once",
                )?,
                Some("--operations") => set_once(
                    &mut operations,
                    parse_positive_usize("--operations", &value)?,
                    "--operations may be supplied only once",
                )?,
                Some("--retained-root") => {
                    if value.is_empty() {
                        return Err("--retained-root must not be empty".into());
                    }
                    set_once(
                        &mut retained_root,
                        PathBuf::from(value),
                        "--retained-root may be supplied only once",
                    )?;
                }
                _ => {
                    return Err(format!(
                        "unknown argument {}\n{USAGE}",
                        flag.to_string_lossy()
                    ));
                }
            }
        }

        Ok(Self {
            seed: seed.ok_or_else(|| format!("--seed is required\n{USAGE}"))?,
            cases: cases.ok_or_else(|| format!("--cases is required\n{USAGE}"))?,
            operations: operations.ok_or_else(|| format!("--operations is required\n{USAGE}"))?,
            retained_root,
        })
    }
}

fn main() -> std::process::ExitCode {
    match execute() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rrflowkv_stress: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<(), String> {
    let arguments = Arguments::parse()?;
    let report = run_seeded_scenarios(&ScenarioConfig {
        seed: arguments.seed,
        cases: arguments.cases,
        operations: arguments.operations,
        retained_root: arguments.retained_root,
    })?;

    println!(
        "rrflowkv_stress: ok seed={:#018x} cases={} operations_per_case={} operations={} writes={} mutations={} injected_failures={} flushes={} compactions={} reopens={} garbage_collections={} projected_reads={} emitted_rows={} emitted_batches={} cancellations={} resource_denials={} verification_rounds={}",
        arguments.seed,
        arguments.cases,
        arguments.operations,
        report.operations,
        report.writes,
        report.mutations,
        report.injected_failures,
        report.flushes,
        report.compactions,
        report.reopens,
        report.garbage_collections,
        report.projected_reads,
        report.emitted_rows,
        report.emitted_batches,
        report.cancellations,
        report.resource_denials,
        report.verification_rounds,
    );
    Ok(())
}

fn set_once<T>(slot: &mut Option<T>, value: T, duplicate: &'static str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(duplicate.into());
    }
    Ok(())
}

fn parse_seed(value: &OsString) -> Result<u64, String> {
    let text = value
        .to_str()
        .ok_or_else(|| "--seed must be valid UTF-8 digits".to_owned())?;
    let parsed = if let Some(hex) = text.strip_prefix("0x") {
        u64::from_str_radix(hex, 16)
    } else {
        text.parse()
    };
    parsed.map_err(|_| "--seed must be an unsigned 64-bit decimal or 0x-prefixed value".into())
}

fn parse_positive_usize(flag: &str, value: &OsString) -> Result<usize, String> {
    let text = value
        .to_str()
        .ok_or_else(|| format!("{flag} must be valid UTF-8 digits"))?;
    let parsed = text
        .parse::<usize>()
        .map_err(|_| format!("{flag} must be a positive integer"))?;
    if parsed == 0 {
        return Err(format!("{flag} must be greater than zero"));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_require_one_value_for_each_required_coordinate() {
        let parsed = Arguments::parse_from(
            [
                "--seed",
                "0xcafe",
                "--cases",
                "2",
                "--operations",
                "32",
                "--retained-root",
                "qualification",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap();

        assert_eq!(parsed.seed, 0xcafe);
        assert_eq!(parsed.cases, 2);
        assert_eq!(parsed.operations, 32);
        assert_eq!(parsed.retained_root, Some(PathBuf::from("qualification")));
    }

    #[test]
    fn arguments_reject_missing_duplicate_unknown_zero_and_overflow_values() {
        for arguments in [
            vec![],
            vec!["--seed", "1", "--cases", "1"],
            vec![
                "--seed",
                "1",
                "--seed",
                "2",
                "--cases",
                "1",
                "--operations",
                "1",
            ],
            vec!["--seed", "1", "--cases", "0", "--operations", "1"],
            vec![
                "--seed",
                "1",
                "--cases",
                "1",
                "--operations",
                "999999999999999999999999999999999999999",
            ],
            vec![
                "--seed",
                "1",
                "--cases",
                "1",
                "--unknown",
                "1",
                "--operations",
                "1",
            ],
        ] {
            assert!(
                Arguments::parse_from(arguments.into_iter().map(OsString::from)).is_err(),
                "arguments unexpectedly parsed"
            );
        }
    }
}
