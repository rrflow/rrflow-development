//! Physical columnar adapters for projection and interchange.

use crate::columnar::{ColumnarColumn, ColumnarTable};
use crate::{Error, Result};
use arrow_array::{Array, ArrayRef, RecordBatch, StringArray, UInt64Array};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use std::sync::Arc;

#[cfg(feature = "cld-parquet")]
mod parquet;
#[cfg(feature = "cld-vortex")]
mod vortex;

#[cfg(feature = "cld-parquet")]
pub use self::parquet::ParquetReference;
#[cfg(feature = "cld-vortex")]
pub use self::vortex::VortexReference;

fn failed(module: &str, error: impl std::fmt::Display) -> Error {
    Error::Execution(format!("{module} reference module: {error}"))
}

fn schema_of(table: &ColumnarTable) -> SchemaRef {
    Arc::new(Schema::new(
        table
            .columns()
            .iter()
            .map(|column| {
                let data_type = match column {
                    ColumnarColumn::Utf8 { .. } => DataType::Utf8,
                    ColumnarColumn::U64 { .. } => DataType::UInt64,
                };
                Field::new(column.name(), data_type, true)
            })
            .collect::<Vec<_>>(),
    ))
}

fn to_batch(table: &ColumnarTable) -> Result<RecordBatch> {
    let arrays = table
        .columns()
        .iter()
        .map(|column| match column {
            ColumnarColumn::Utf8 { values, .. } => {
                Arc::new(StringArray::from_iter(values.iter().map(Option::as_deref))) as ArrayRef
            }
            ColumnarColumn::U64 { values, .. } => {
                Arc::new(UInt64Array::from(values.clone())) as ArrayRef
            }
        })
        .collect::<Vec<_>>();
    RecordBatch::try_new(schema_of(table), arrays).map_err(|error| failed("arrow", error))
}

/// Joins the batches a format returned into one table. `schema` names the
/// columns expected, so a result with no batch still has its columns.
fn from_batches(schema: &Schema, batches: &[RecordBatch]) -> Result<ColumnarTable> {
    let mut columns = schema
        .fields()
        .iter()
        .map(|field| match field.data_type() {
            DataType::Utf8 => Ok(ColumnarColumn::Utf8 {
                name: field.name().clone(),
                values: Vec::new(),
            }),
            DataType::UInt64 => Ok(ColumnarColumn::U64 {
                name: field.name().clone(),
                values: Vec::new(),
            }),
            other => Err(failed(
                "arrow",
                format!("column {} has unsupported type {other}", field.name()),
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    for batch in batches {
        for (index, column) in columns.iter_mut().enumerate() {
            // A format may hand a column back in another Arrow representation of
            // the same values (a view, a dictionary, a run). Bring it to the
            // representation the port names before reading it.
            let target = match column {
                ColumnarColumn::Utf8 { .. } => DataType::Utf8,
                ColumnarColumn::U64 { .. } => DataType::UInt64,
            };
            let array = arrow_cast::cast(batch.column(index), &target)
                .map_err(|error| failed("arrow", error))?;
            match column {
                ColumnarColumn::Utf8 { name, values } => {
                    let strings = array
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .ok_or_else(|| failed("arrow", format!("column {name} is not UTF-8")))?;
                    values.extend(strings.iter().map(|value| value.map(str::to_owned)));
                }
                ColumnarColumn::U64 { name, values } => {
                    let numbers = array
                        .as_any()
                        .downcast_ref::<UInt64Array>()
                        .ok_or_else(|| failed("arrow", format!("column {name} is not u64")))?;
                    values.extend(numbers.iter());
                }
            }
        }
    }
    ColumnarTable::new(columns)
}
