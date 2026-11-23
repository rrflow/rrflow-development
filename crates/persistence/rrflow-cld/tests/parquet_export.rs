#![cfg(feature = "cld-parquet")]
use bytes::Bytes;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use rrflow_cld::{export_parquet, record_batch_to_rows, QueryRow, WriteLimits};
use rrflow_kernel::RuntimeValue;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

fn limits() -> WriteLimits {
    WriteLimits {
        max_rows: 1024,
        max_columns: 64,
        max_input_bytes: 262144,
        max_output_bytes: 524288,
        row_group_rows: 64,
        max_memory_bytes: 67108864,
        deadline: Instant::now() + Duration::from_secs(30),
    }
}

fn rows() -> Vec<QueryRow> {
    (0..130)
        .map(|index| QueryRow {
            identity: format!("row-{index}"),
            values: BTreeMap::from([
                ("bool".into(), RuntimeValue::Bool(index % 2 == 0)),
                ("int".into(), RuntimeValue::Integer(-42)),
                ("uint".into(), RuntimeValue::Unsigned(u64::MAX)),
                (
                    "text".into(),
                    RuntimeValue::String("Unicode \u{03bb}".into()),
                ),
                ("null".into(), RuntimeValue::Null),
                ("digest".into(), RuntimeValue::Digest("a".repeat(64))),
                (
                    "decimal".into(),
                    RuntimeValue::Decimal("123456789.000001".into()),
                ),
                (
                    "mixed".into(),
                    if index % 2 == 0 {
                        RuntimeValue::Integer(1)
                    } else {
                        RuntimeValue::String("one".into())
                    },
                ),
                (
                    "nested".into(),
                    RuntimeValue::Map(BTreeMap::from([(
                        "list".into(),
                        RuntimeValue::List(vec![RuntimeValue::Null, RuntimeValue::Bool(true)]),
                    )])),
                ),
            ]),
        })
        .collect()
}

#[test]
fn parquet_round_trip_preserves_types_metadata_and_row_groups() {
    let rows = rows();
    let metadata = BTreeMap::from([(
        "rrflow.export.source".into(),
        "{\"scope\":\"instance:test\"}".into(),
    )]);
    let export = export_parquet(&rows, &metadata, limits()).unwrap();
    assert_eq!(export.row_groups, 3);
    let reader = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(export.bytes)).unwrap();
    let entries = reader
        .metadata()
        .file_metadata()
        .key_value_metadata()
        .unwrap();
    assert!(entries
        .iter()
        .any(|entry| entry.key == "rrflow.export.source"
            && entry.value.as_ref() == metadata.get(&entry.key)));
    let restored = reader
        .build()
        .unwrap()
        .flat_map(|batch| record_batch_to_rows(&batch.unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(restored, rows);
}

#[test]
fn empty_export_is_a_readable_file() {
    let export = export_parquet(&[], &BTreeMap::new(), limits()).unwrap();
    assert_eq!(export.row_groups, 0);
    assert_eq!(export.columns, 1);
    let reader = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(export.bytes)).unwrap();
    assert_eq!(reader.metadata().file_metadata().num_rows(), 0);
    assert_eq!(reader.build().unwrap().count(), 0);
}

#[test]
fn resource_denials_never_return_partial_artifacts() {
    let rows = rows();
    let base = limits();
    let complete = export_parquet(&rows, &BTreeMap::new(), base).unwrap();
    assert!(complete.observed_writer_bytes > 0);
    for denied in [
        WriteLimits {
            max_rows: 1,
            ..base
        },
        WriteLimits {
            max_columns: 1,
            ..base
        },
        WriteLimits {
            max_input_bytes: 12,
            ..base
        },
        WriteLimits {
            max_output_bytes: 12,
            ..base
        },
        WriteLimits {
            max_output_bytes: complete.bytes.len() - 1,
            ..base
        },
        WriteLimits {
            max_memory_bytes: 12,
            ..base
        },
        WriteLimits {
            row_group_rows: 0,
            ..base
        },
        WriteLimits {
            deadline: Instant::now(),
            ..base
        },
    ] {
        assert!(
            export_parquet(&rows, &BTreeMap::new(), denied).is_err(),
            "{denied:?}"
        );
    }
    let mut collision = rows;
    collision[0]
        .values
        .insert("__rrd_identity".into(), RuntimeValue::Null);
    assert!(export_parquet(&collision, &BTreeMap::new(), limits()).is_err());
}
