//! The reference modules behind the columnar port.
//!
//! A reference module is only useful if it is the same contract with a
//! different format, so every check here puts one table through every module
//! and compares what each answers, line for line, with the table itself.

#![cfg(all(feature = "parquet-reference", feature = "vortex-reference"))]

use rrd_store::columnar::{ColumnarColumn, ColumnarFormat, ColumnarTable};
use rrd_store::columnar_reference::{ParquetReference, VortexReference};

fn modules() -> Vec<Box<dyn ColumnarFormat>> {
    vec![Box::new(ParquetReference), Box::new(VortexReference)]
}

/// A page shaped like cold history: text identities, a JSON payload, cursors.
fn history(rows: usize) -> ColumnarTable {
    let utf8 = |name: &str, value: &dyn Fn(usize) -> Option<String>| ColumnarColumn::Utf8 {
        name: name.into(),
        values: (0..rows).map(value).collect(),
    };
    let u64s = |name: &str, value: &dyn Fn(usize) -> Option<u64>| ColumnarColumn::U64 {
        name: name.into(),
        values: (0..rows).map(value).collect(),
    };
    ColumnarTable::new(vec![
        utf8("scope", &|_| Some("instance:lab".into())),
        utf8("family", &|row| {
            Some(["record", "relation", "claim"][row % 3].into())
        }),
        utf8("subject", &|row| Some(format!("subject-{:05}", row / 4))),
        // Absent for every fifth row, empty for every seventh, non-ASCII for every eleventh.
        utf8("predicate", &|row| match row {
            row if row % 5 == 0 => None,
            row if row % 7 == 0 => Some(String::new()),
            row if row % 11 == 0 => Some("größe/サイズ".into()),
            row => Some(format!("predicate-{}", row % 13)),
        }),
        utf8("change_json", &|row| {
            Some(format!(
                "{{\"row\":{row},\"note\":\"{}\"}}",
                "x".repeat(row % 40)
            ))
        }),
        u64s("cursor", &|row| Some(row as u64 + 1)),
        u64s("committed_at", &|row| {
            Some(1_700_000_000_000 + row as u64 * 37)
        }),
        // The extremes, and absent for every ninth row.
        u64s("valid_at", &|row| match row % 9 {
            0 => None,
            1 => Some(u64::MAX),
            2 => Some(0),
            _ => Some(row as u64),
        }),
    ])
    .unwrap()
}

fn digest(table: &ColumnarTable) -> Vec<String> {
    let mut lines = vec![format!("{} rows", table.rows())];
    for column in table.columns() {
        lines.push(match column {
            ColumnarColumn::Utf8 { name, values } => format!("{name}: {values:?}"),
            ColumnarColumn::U64 { name, values } => format!("{name}: {values:?}"),
        });
    }
    lines
}

/// Everything a module answers about one table, as text.
fn transcript(module: &dyn ColumnarFormat, table: &ColumnarTable) -> Vec<String> {
    let artifact = module.encode(table).unwrap();
    let mut log = Vec::new();
    log.push("whole".to_string());
    log.extend(digest(&module.decode(&artifact).unwrap()));
    for names in [
        &["cursor"][..],
        &["valid_at", "subject"],
        &["change_json", "cursor", "predicate"],
    ] {
        log.push(format!("columns {names:?}"));
        log.extend(digest(&module.decode_columns(&artifact, names).unwrap()));
    }
    log.push(format!(
        "unknown column refused: {}",
        module.decode_columns(&artifact, &["missing"]).is_err()
    ));
    log.push(format!(
        "damaged artifact refused: {}",
        module.decode(&artifact[..artifact.len() / 2]).is_err()
    ));
    log
}

/// What the table itself says, in the transcript's form: the reference answer.
fn expected(table: &ColumnarTable) -> Vec<String> {
    let mut log = vec!["whole".to_string()];
    log.extend(digest(table));
    for names in [
        &["cursor"][..],
        &["valid_at", "subject"],
        &["change_json", "cursor", "predicate"],
    ] {
        log.push(format!("columns {names:?}"));
        log.extend(digest(&table.project(names).unwrap()));
    }
    log.push("unknown column refused: true".into());
    log.push("damaged artifact refused: true".into());
    log
}

#[test]
fn every_module_answers_exactly_what_the_table_holds() {
    for rows in [1, 2, 257, 5_000] {
        let table = history(rows);
        let reference = expected(&table);
        for module in modules() {
            assert_eq!(
                transcript(module.as_ref(), &table),
                reference,
                "the {} module departs from the table at {rows} rows",
                module.id()
            );
        }
    }
}

#[test]
fn an_empty_table_keeps_its_columns_in_every_module() {
    let table = history(0);
    for module in modules() {
        let artifact = module.encode(&table).unwrap();
        assert_eq!(
            digest(&module.decode(&artifact).unwrap()),
            digest(&table),
            "{}",
            module.id()
        );
        assert_eq!(
            digest(&module.decode_columns(&artifact, &["cursor"]).unwrap()),
            digest(&table.project(&["cursor"]).unwrap()),
            "{}",
            module.id()
        );
    }
}

#[test]
fn the_modules_are_distinct_formats_with_distinct_names() {
    let table = history(64);
    let artifacts = modules()
        .iter()
        .map(|module| (module.id(), module.encode(&table).unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(artifacts[0].0, "parquet");
    assert_eq!(artifacts[1].0, "vortex");
    assert_eq!(&artifacts[0].1[..4], b"PAR1");
    assert_eq!(&artifacts[1].1[..4], b"VTXF");
    // A module refuses the other's artifact instead of misreading it.
    assert!(ParquetReference.decode(&artifacts[1].1).is_err());
    assert!(VortexReference.decode(&artifacts[0].1).is_err());
}
