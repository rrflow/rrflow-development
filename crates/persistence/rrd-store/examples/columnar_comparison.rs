//! The reference modules behind the columnar port, on one cold-history table.
//!
//! One table shaped like a cold-history page is put through every module:
//! encoded, decoded whole, and decoded for two columns out of ten. Each step
//! is timed over several trials and the median is reported, with the encoded
//! size. Every decode is checked against the table before its time counts.
//!
//! There is no native rrflowCLD arm yet: the cold tier has no format of its
//! own. This program is the harness that arm joins.
//!
//! This is a diagnostic from one machine. It states no verdict.
//!
//! ```text
//! cargo run --release -p rrd-store --features parquet-reference,vortex-reference \
//!   --example columnar_comparison -- --rows 100000 --trials 7 --output out.json
//! ```

use rrd_store::columnar::{ColumnarColumn, ColumnarFormat, ColumnarTable};
use rrd_store::columnar_reference::{ParquetReference, VortexReference};
use serde::Serialize;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

const FORMAT_VERSION: u16 = 1;
const PROJECTION: [&str; 2] = ["cursor", "subject"];

#[derive(Debug, Serialize)]
struct Arm {
    module: &'static str,
    encoded_bytes: usize,
    /// Encoded bytes divided by the table's raw value bytes.
    encoded_over_raw: f64,
    encode_ns: u64,
    decode_all_ns: u64,
    decode_two_columns_ns: u64,
}

#[derive(Debug, Serialize)]
struct Evidence {
    format_version: u16,
    kind: &'static str,
    architecture: &'static str,
    operating_system: &'static str,
    rows: usize,
    columns: usize,
    raw_value_bytes: usize,
    trials: usize,
    projection: [&'static str; 2],
    aggregation: &'static str,
    caveat: &'static str,
    arms: Vec<Arm>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("columnar comparison failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut rows = 100_000usize;
    let mut trials = 5usize;
    let mut output = None;
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("missing value after {}", arguments[index]))?;
        match arguments[index].as_str() {
            "--rows" => rows = positive(value, "rows")?,
            "--trials" => trials = positive(value, "trials")?,
            "--output" => output = Some(PathBuf::from(value)),
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 2;
    }

    let table = history(rows)?;
    let raw_value_bytes = raw_bytes(&table);
    let expected_projection = table
        .project(&PROJECTION)
        .map_err(|error| error.to_string())?;
    let modules: [&dyn ColumnarFormat; 2] = [&ParquetReference, &VortexReference];
    let mut arms = Vec::new();
    for module in modules {
        let mut encode = Vec::with_capacity(trials);
        let mut decode_all = Vec::with_capacity(trials);
        let mut decode_columns = Vec::with_capacity(trials);
        let mut encoded_bytes = 0;
        for _ in 0..trials {
            let started = Instant::now();
            let artifact = module.encode(&table).map_err(|error| error.to_string())?;
            encode.push(elapsed(started));
            encoded_bytes = artifact.len();

            let started = Instant::now();
            let whole = module
                .decode(&artifact)
                .map_err(|error| error.to_string())?;
            decode_all.push(elapsed(started));
            if whole != table {
                return Err(format!("{} decoded a different table", module.id()));
            }

            let started = Instant::now();
            let projected = module
                .decode_columns(&artifact, &PROJECTION)
                .map_err(|error| error.to_string())?;
            decode_columns.push(elapsed(started));
            if projected != expected_projection {
                return Err(format!("{} decoded different columns", module.id()));
            }
            black_box((whole, projected));
        }
        arms.push(Arm {
            module: module.id(),
            encoded_bytes,
            encoded_over_raw: encoded_bytes as f64 / raw_value_bytes as f64,
            encode_ns: median(encode),
            decode_all_ns: median(decode_all),
            decode_two_columns_ns: median(decode_columns),
        });
    }

    let evidence = Evidence {
        format_version: FORMAT_VERSION,
        kind: "columnar-reference-modules",
        architecture: std::env::consts::ARCH,
        operating_system: std::env::consts::OS,
        rows,
        columns: table.columns().len(),
        raw_value_bytes,
        trials,
        projection: PROJECTION,
        aggregation: "each time is the median across the trials",
        caveat: "one machine, in memory, no file I/O; times include conversion between the port's table and Arrow, which every module pays alike; there is no native rrflowCLD arm",
        arms,
    };
    let bytes = serde_json::to_vec_pretty(&evidence).map_err(|error| error.to_string())?;
    if let Some(output) = output {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(&output, &bytes).map_err(|error| error.to_string())?;
        eprintln!("wrote comparison evidence to {}", output.display());
    }
    println!(
        "{}",
        String::from_utf8(bytes).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn positive(value: &str, label: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(0) => Err(format!("{label} must be greater than zero")),
        Ok(parsed) => Ok(parsed),
        Err(error) => Err(format!("invalid {label}: {error}")),
    }
}

fn elapsed(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn median(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values[values.len() / 2]
}

fn raw_bytes(table: &ColumnarTable) -> usize {
    table
        .columns()
        .iter()
        .map(|column| match column {
            ColumnarColumn::Utf8 { values, .. } => {
                values.iter().flatten().map(String::len).sum::<usize>()
            }
            ColumnarColumn::U64 { values, .. } => values.iter().flatten().count() * 8,
        })
        .sum()
}

/// The columns a cold-history page stores, with values that repeat, grow and
/// vary the way commit history does. Deterministic, so every run and every
/// module sees the same table.
fn history(rows: usize) -> Result<ColumnarTable, String> {
    let utf8 = |name: &str, value: &dyn Fn(usize) -> Option<String>| ColumnarColumn::Utf8 {
        name: name.into(),
        values: (0..rows).map(value).collect(),
    };
    let u64s = |name: &str, value: &dyn Fn(usize) -> Option<u64>| ColumnarColumn::U64 {
        name: name.into(),
        values: (0..rows).map(value).collect(),
    };
    let mix = |row: usize| (row as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) >> 20;
    ColumnarTable::new(vec![
        utf8("scope", &|_| Some("instance:comparison".into())),
        utf8("commit_id", &|row| Some(format!("commit-{:012}", row / 8))),
        utf8("family", &|row| {
            Some(["record", "relation", "claim", "vector"][row % 4].into())
        }),
        utf8("kind", &|row| Some(format!("kind-{}", mix(row) % 24))),
        utf8("subject", &|row| {
            Some(format!("subject-{:08}", mix(row) % 50_000))
        }),
        utf8("predicate", &|row| {
            (row % 6 != 0).then(|| format!("predicate-{}", mix(row) % 64))
        }),
        utf8("change_digest", &|row| Some(format!("{:064x}", mix(row)))),
        utf8("change_json", &|row| {
            Some(format!(
                "{{\"subject\":\"subject-{:08}\",\"value\":{},\"note\":\"{}\"}}",
                mix(row) % 50_000,
                mix(row) % 1_000,
                "lorem ipsum ".repeat(row % 6)
            ))
        }),
        u64s("cursor", &|row| Some(row as u64 + 1)),
        u64s("committed_at", &|row| {
            Some(1_700_000_000_000 + row as u64 * 37)
        }),
    ])
    .map_err(|error| error.to_string())
}
