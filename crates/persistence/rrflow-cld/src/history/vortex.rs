//! Vortex history files keep authenticated coordinates in a native metadata segment.
//! Generations contain bounded input chunks. Reads select bounded row ranges.
use super::{
    rows::{checked_change, cold_schema, prepare},
    *,
};
use crate::sink::{BoundedOutput, SINK_DEADLINE, SINK_NO_FAILURE};
use crate::{record_batch_to_rows, rows_to_record_batch, working_set_reservation};
use ::vortex::arrow::convert::from_arrow_batch;
use ::vortex::file::{OpenOptionsSessionExt, WriteOptionsSessionExt};
use ::vortex::io::runtime::{single::SingleThreadRuntime, BlockingRuntime};
use ::vortex::io::session::RuntimeSessionExt;
use ::vortex::session::VortexSession;
use ::vortex::VortexSessionDefault;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

const METADATA: &str = "rrflow.history";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    version: u16,
    stamp: ReadStamp,
    after: u64,
    through: u64,
    rows_per_chunk: usize,
    arrow_bytes: usize,
}

#[derive(Debug)]
pub struct VortexHistory;

fn deadline(end: Instant) -> Result<()> {
    if Instant::now() >= end {
        return Err(Error::DeadlineExceeded(
            "Vortex history deadline exceeded".into(),
        ));
    }
    Ok(())
}

impl ColdStorageBackend for VortexHistory {
    fn format(&self) -> HistoryFormat {
        HistoryFormat::Vortex
    }
    fn encode(
        &self,
        stamp: &ReadStamp,
        page: &RuntimeChangePage,
        limits: WriteLimits,
    ) -> Result<ColdHistoryPage> {
        deadline(limits.deadline)?;
        let fields = cold_schema();
        if limits.row_group_rows == 0
            || limits.max_columns < fields.len()
            || limits.max_rows == 0
            || limits.max_output_bytes == 0
        {
            return Err(Error::Budget("invalid Vortex page limits".into()));
        }
        let prepared = prepare(stamp, page, limits)?;
        let cells = prepared
            .rows
            .len()
            .checked_mul(fields.len() + 1)
            .ok_or_else(|| Error::Budget("Vortex page size overflow".into()))?;
        let reserve = working_set_reservation(
            prepared.input_bytes,
            cells,
            limits.max_output_bytes,
            fields.len() + 1,
        );
        if reserve.is_none_or(|bytes| bytes > limits.max_memory_bytes) {
            return Err(Error::Budget(
                "Vortex page exceeds its memory reservation".into(),
            ));
        }
        let batch = rows_to_record_batch(&prepared.rows, &fields)?;
        let arrow_bytes = batch.get_array_memory_size();
        let header = Header {
            version: COLD_HISTORY_FORMAT_VERSION,
            stamp: stamp.clone(),
            after: page.requested_after,
            through: page.through_cursor,
            rows_per_chunk: limits.row_group_rows,
            arrow_bytes,
        };
        let runtime = SingleThreadRuntime::default();
        let session = VortexSession::default().with_handle(runtime.handle());
        let failure = Arc::new(AtomicU8::new(SINK_NO_FAILURE));
        let mut output = BoundedOutput {
            bytes: Vec::new(),
            limit: limits.max_output_bytes,
            deadline: limits.deadline,
            failure: Arc::clone(&failure),
        };
        let writer_error =
            |error: ::vortex::error::VortexError| match failure.load(Ordering::Relaxed) {
                SINK_DEADLINE => Error::DeadlineExceeded(error.to_string()),
                SINK_NO_FAILURE => Error::Execution(error.to_string()),
                _ => Error::Budget(error.to_string()),
            };
        let empty = from_arrow_batch(&batch.slice(0, 0), false).map_err(writer_error)?;
        let mut writer = session
            .write_options()
            .with_metadata_segment(METADATA, serde_json::to_vec(&header)?)
            .blocking(&runtime)
            .writer(&mut output, empty.dtype().clone());
        for start in (0..batch.num_rows()).step_by(limits.row_group_rows) {
            deadline(limits.deadline)?;
            let length = limits.row_group_rows.min(batch.num_rows() - start);
            let chunk =
                from_arrow_batch(&batch.slice(start, length), false).map_err(writer_error)?;
            writer.push(chunk).map_err(writer_error)?;
            if writer.buffered_bytes().saturating_add(arrow_bytes as u64)
                > limits.max_memory_bytes as u64
            {
                return Err(Error::Budget("Vortex writer buffers exceed budget".into()));
            }
        }
        writer.finish().map_err(writer_error)?;
        deadline(limits.deadline)?;
        Ok(ColdHistoryPage {
            requested_after: page.requested_after,
            through_cursor: page.through_cursor,
            head_cursor: page.head_cursor,
            source_manifest: stamp.manifest_id.clone(),
            artifact: HistoryArtifact {
                bytes: output.bytes,
                row_groups: batch.num_rows().div_ceil(limits.row_group_rows),
            },
        })
    }

    fn decode(
        &self,
        bytes: Vec<u8>,
        expected: &ColdHistoryReadExpectation,
        limits: ColdHistoryReadLimits,
    ) -> std::result::Result<DecodedColdHistoryPage, ColdHistoryReadError> {
        admit(&bytes, expected, limits)?;
        let encoded_bytes = bytes.len();
        let runtime = SingleThreadRuntime::default();
        let session = VortexSession::default().with_handle(runtime.handle());
        let invalid = |error: ::vortex::error::VortexError| {
            Error::Integrity(format!("invalid Vortex history: {error}"))
        };
        let file = session
            .open_options()
            .include_metadata()
            .open_buffer(bytes)
            .map_err(invalid)?;
        let header: Header = serde_json::from_slice(
            file.metadata_segment(METADATA)
                .ok_or_else(|| Error::Integrity("missing Vortex history metadata".into()))?,
        )?;
        if file.row_count() != expected.rows
            || header.version != COLD_HISTORY_FORMAT_VERSION
            || header.stamp != expected.stamp
            || header.after != expected.requested_after
            || header.through != expected.through_cursor
            || header.rows_per_chunk == 0
        {
            return Err(
                Error::Integrity("Vortex generation differs from its receipt".into()).into(),
            );
        }
        let reservation = header
            .arrow_bytes
            .checked_mul(32)
            .and_then(|bytes| bytes.checked_add(encoded_bytes));
        if header.arrow_bytes > limits.max_decoded_bytes
            || reservation.is_none_or(|bytes| bytes > limits.max_memory_bytes)
        {
            return Err(Error::Budget("Vortex generation exceeds decode budget".into()).into());
        }
        let schema = rows_to_record_batch(&[], &cold_schema())
            .map_err(Error::from)?
            .schema();
        let rows = usize::try_from(expected.rows)
            .map_err(|_| Error::Budget("row count exceeds address space".into()))?;
        let mut changes = Vec::new();
        changes
            .try_reserve_exact(rows)
            .map_err(|_| Error::Budget("cold history row allocation failed".into()))?;
        let mut previous = expected.requested_after;
        let mut observed = 0usize;
        for start in (0..rows).step_by(limits.max_batch_rows) {
            deadline(limits.deadline)?;
            let end = start.saturating_add(limits.max_batch_rows).min(rows);
            let scan = file
                .scan()
                .map_err(invalid)?
                .with_row_range(start as u64..end as u64);
            let reader = scan
                .into_record_batch_reader(Arc::clone(&schema), &runtime)
                .map_err(invalid)?;
            for batch in reader {
                deadline(limits.deadline)?;
                let batch = batch.map_err(|error| Error::Integrity(error.to_string()))?;
                observed = observed
                    .checked_add(batch.get_array_memory_size())
                    .ok_or_else(|| Error::Budget("Vortex decoded size overflow".into()))?;
                if observed > limits.max_decoded_bytes || batch.num_rows() > end - start {
                    return Err(Error::Budget("Vortex batch exceeds decode limits".into()).into());
                }
                for row in record_batch_to_rows(&batch).map_err(Error::from)? {
                    deadline(limits.deadline)?;
                    changes.push(checked_change(row, expected, &mut previous)?);
                }
            }
            if changes.len() != end {
                return Err(
                    Error::Integrity("Vortex range returned a different row count".into()).into(),
                );
            }
        }
        deadline(limits.deadline)?;
        Ok(DecodedColdHistoryPage {
            changes,
            encoded_bytes,
            row_groups: rows.div_ceil(header.rows_per_chunk),
            declared_uncompressed_bytes: header.arrow_bytes as u64,
            observed_arrow_bytes: observed,
        })
    }
}
