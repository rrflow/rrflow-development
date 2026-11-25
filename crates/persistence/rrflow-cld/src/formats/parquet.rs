//! The Parquet reference module.

use super::{failed, from_batches, schema_of, to_batch};
use crate::columnar::{ColumnarFormat, ColumnarTable};
use crate::Result;
use arrow_schema::Schema;
use bytes::Bytes;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::{ArrowWriter, ProjectionMask};
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;

const MODULE: &str = "parquet";

/// Apache Parquet behind the columnar port, written and read through Arrow.
/// Zstandard level 3, the setting the cold tier has used.
#[derive(Debug, Default, Clone, Copy)]
pub struct ParquetReference;

impl ColumnarFormat for ParquetReference {
    fn id(&self) -> &'static str {
        MODULE
    }

    fn encode(&self, table: &ColumnarTable) -> Result<Vec<u8>> {
        let level = ZstdLevel::try_new(3).map_err(|error| failed(MODULE, error))?;
        let properties = WriterProperties::builder()
            .set_compression(Compression::ZSTD(level))
            .build();
        let mut artifact = Vec::new();
        let mut writer = ArrowWriter::try_new(&mut artifact, schema_of(table), Some(properties))
            .map_err(|error| failed(MODULE, error))?;
        writer
            .write(&to_batch(table)?)
            .map_err(|error| failed(MODULE, error))?;
        writer.close().map_err(|error| failed(MODULE, error))?;
        Ok(artifact)
    }

    fn decode(&self, artifact: &[u8]) -> Result<ColumnarTable> {
        read(artifact, None)
    }

    fn decode_columns(&self, artifact: &[u8], names: &[&str]) -> Result<ColumnarTable> {
        read(artifact, Some(names))
    }
}

fn read(artifact: &[u8], names: Option<&[&str]>) -> Result<ColumnarTable> {
    let mut builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::copy_from_slice(artifact))
        .map_err(|error| failed(MODULE, error))?;
    let stored = builder.schema().clone();
    let Some(names) = names else {
        return collect(&stored, builder);
    };
    // Parquet returns projected columns in stored order; the port returns them
    // in the order asked for, so read by mask and reorder.
    let indices = names
        .iter()
        .map(|name| {
            stored
                .index_of(name)
                .map_err(|_| failed(MODULE, format!("no column named {name}")))
        })
        .collect::<Result<Vec<_>>>()?;
    let mask = ProjectionMask::roots(builder.parquet_schema(), indices.iter().copied());
    builder = builder.with_projection(mask);
    let mut stored_order = indices;
    stored_order.sort_unstable();
    stored_order.dedup();
    let projected = stored
        .project(&stored_order)
        .map_err(|error| failed(MODULE, error))?;
    collect(&projected, builder)?.project(names)
}

fn collect(
    schema: &Schema,
    builder: ParquetRecordBatchReaderBuilder<Bytes>,
) -> Result<ColumnarTable> {
    let batches = builder
        .build()
        .map_err(|error| failed(MODULE, error))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| failed(MODULE, error))?;
    from_batches(schema, &batches)
}
