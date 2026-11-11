//! Bounded interchange encoding of already authorized query rows.
//! This codec neither reads storage nor grants access to a source.

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    time::Instant,
};

use parquet::{
    arrow::ArrowWriter,
    basic::{Compression, ZstdLevel},
    file::metadata::KeyValue,
    file::properties::{EnabledStatistics, WriterProperties},
};

use crate::{rows_to_record_batch, Error, QueryFieldTypes, QueryRow, Result};

/// Conservative admission estimate for one page, not an allocator-level
/// guarantee. Configuration validation must use this same formula, so that a
/// policy cannot admit a page its own budget would refuse.
pub fn parquet_working_set_reservation(
    input_bytes: usize,
    cells: usize,
    max_output_bytes: usize,
    columns: usize,
) -> Option<usize> {
    input_bytes
        .checked_mul(32)?
        .checked_add(cells.checked_mul(64)?)?
        .checked_add(max_output_bytes.checked_mul(4)?)?
        .checked_add(columns.checked_mul(4096)?)?
        .checked_add(65536)
}

/// The caller owns the public profile and supplies finite resource limits.
#[derive(Debug, Clone, Copy)]
pub struct ParquetExportLimits {
    pub max_rows: usize,
    pub max_columns: usize,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub row_group_rows: usize,
    pub max_memory_bytes: usize,
    pub deadline: Instant,
}

/// Explicit writer choices. Callers must admit the work and its memory budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParquetCompression {
    Uncompressed,
    Zstd { level: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParquetEncoding {
    pub compression: ParquetCompression,
    pub dictionary: bool,
    pub statistics: bool,
}

impl Default for ParquetEncoding {
    fn default() -> Self {
        Self {
            compression: ParquetCompression::Uncompressed,
            dictionary: false,
            statistics: false,
        }
    }
}

#[derive(Debug)]
pub struct ParquetExport {
    pub bytes: Vec<u8>,
    pub columns: usize,
    pub row_groups: usize,
    pub input_bytes: usize,
    pub reserved_working_bytes: usize,
    /// Largest writer estimate sampled after a write; not allocator peak RSS.
    pub observed_writer_bytes: usize,
}

struct BoundedOutput {
    bytes: Vec<u8>,
    limit: usize,
    deadline: Instant,
}

impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if Instant::now() >= self.deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Parquet deadline exceeded",
            ));
        }
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("Parquet byte budget exceeded"));
        }
        self.bytes
            .try_reserve_exact(bytes.len())
            .map_err(io::Error::other)?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn check_deadline(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        return Err(Error::Budget("Parquet deadline exceeded".into()));
    }
    Ok(())
}

/// Produce a complete uncompressed Parquet file, including its footer, or no
/// artifact. Field metadata preserves RRFlow's typed runtime values.
pub fn export_parquet(
    rows: &[QueryRow],
    metadata: &BTreeMap<String, String>,
    limits: ParquetExportLimits,
) -> Result<ParquetExport> {
    export_parquet_encoded(
        rows,
        metadata,
        &QueryFieldTypes::new(),
        limits,
        ParquetEncoding::default(),
    )
}

/// Encode admitted rows with a declared schema and explicit physical policy.
/// The result is derived bytes; publication and authorization belong to the caller.
pub fn export_parquet_encoded(
    rows: &[QueryRow],
    metadata: &BTreeMap<String, String>,
    declared: &QueryFieldTypes,
    limits: ParquetExportLimits,
    encoding: ParquetEncoding,
) -> Result<ParquetExport> {
    check_deadline(limits.deadline)?;
    if limits.max_rows == 0
        || limits.max_columns == 0
        || limits.row_group_rows == 0
        || limits.max_input_bytes == 0
        || limits.max_output_bytes == 0
        || limits.max_memory_bytes == 0
        || rows.len() > limits.max_rows
    {
        return Err(Error::Budget("invalid or exceeded Parquet limits".into()));
    }
    let mut fields: BTreeSet<&str> = declared.keys().map(String::as_str).collect();
    if fields.contains("__rrd_identity") || fields.len() > limits.max_columns {
        return Err(Error::Budget("invalid declared Parquet columns".into()));
    }
    let mut cells = 0usize;
    for row in rows {
        for field in row.values.keys() {
            if field == "__rrd_identity" {
                return Err(Error::Binding("reserved Parquet identity column".into()));
            }
            fields.insert(field.as_str());
            if fields.len() > limits.max_columns {
                return Err(Error::Budget("Parquet column budget exceeded".into()));
            }
        }
        cells = cells.saturating_add(row.values.len()).saturating_add(1);
    }
    // Count serialized source bytes through a bounded sink before Arrow allocates.
    let mut input = BoundedOutput {
        bytes: Vec::new(),
        limit: limits.max_input_bytes,
        deadline: limits.deadline,
    };
    serde_json::to_writer(&mut input, &(rows, metadata))
        .map_err(|error| Error::Budget(error.to_string()))?;
    let input_bytes = input.bytes.len();
    drop(input);
    let columns = fields.len() + 1;
    let reservation =
        parquet_working_set_reservation(input_bytes, cells, limits.max_output_bytes, columns)
            .ok_or_else(|| Error::Budget("Parquet memory reservation overflow".into()))?;
    if reservation > limits.max_memory_bytes {
        return Err(Error::Budget(
            "Parquet working-set reservation exceeds budget".into(),
        ));
    }
    check_deadline(limits.deadline)?;
    let batch = rows_to_record_batch(rows, declared)?;
    let arrow_bytes = batch.get_array_memory_size();
    let compression = match encoding.compression {
        ParquetCompression::Uncompressed => Compression::UNCOMPRESSED,
        ParquetCompression::Zstd { level } => Compression::ZSTD(
            ZstdLevel::try_new(level).map_err(|error| Error::Binding(error.to_string()))?,
        ),
    };
    let properties = WriterProperties::builder()
        .set_compression(compression)
        .set_dictionary_enabled(encoding.dictionary)
        .set_statistics_enabled(if encoding.statistics {
            EnabledStatistics::Page
        } else {
            EnabledStatistics::None
        })
        // Explicit flush below bounds groups and permits sampling the buffered
        // writer before a flush discards its memory estimate.
        .set_max_row_group_row_count(None)
        .set_write_batch_size(limits.row_group_rows)
        .set_key_value_metadata(Some(
            metadata
                .iter()
                .map(|(key, value)| KeyValue {
                    key: key.clone(),
                    value: Some(value.clone()),
                })
                .collect(),
        ))
        .build();
    let sink = BoundedOutput {
        bytes: Vec::new(),
        limit: limits.max_output_bytes,
        deadline: limits.deadline,
    };
    let mut writer = ArrowWriter::try_new(sink, batch.schema(), Some(properties))
        .map_err(|error| Error::Execution(error.to_string()))?;
    let mut observed_writer_bytes = 0;
    for start in (0..rows.len()).step_by(limits.row_group_rows) {
        check_deadline(limits.deadline)?;
        writer
            .write(&batch.slice(start, limits.row_group_rows.min(rows.len() - start)))
            .map_err(|error| Error::Budget(error.to_string()))?;
        observed_writer_bytes = observed_writer_bytes.max(writer.memory_size());
        if arrow_bytes.saturating_add(writer.memory_size()) > limits.max_memory_bytes {
            return Err(Error::Budget(
                "Parquet measured buffers exceed budget".into(),
            ));
        }
        writer
            .flush()
            .map_err(|error| Error::Budget(error.to_string()))?;
    }
    let row_groups = writer.flushed_row_groups().len();
    let output = writer
        .into_inner()
        .map_err(|error| Error::Budget(error.to_string()))?;
    check_deadline(limits.deadline)?;
    Ok(ParquetExport {
        bytes: output.bytes,
        columns,
        row_groups,
        input_bytes,
        reserved_working_bytes: reservation,
        observed_writer_bytes,
    })
}
