//! Bounded Parquet generation encoding and verified decoding.
use super::{
    rows::{checked_change, cold_schema, prepare},
    *,
};
use crate::{export_parquet_encoded, record_batch_to_rows, Encoding};
use ::parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use bytes::Bytes;

#[derive(Debug, Default)]
pub struct ParquetHistory(pub Encoding);

impl ColdStorageBackend for ParquetHistory {
    fn format(&self) -> HistoryFormat {
        HistoryFormat::Parquet
    }
    fn encode(
        &self,
        stamp: &ReadStamp,
        page: &RuntimeChangePage,
        limits: WriteLimits,
    ) -> Result<ColdHistoryPage> {
        let input = prepare(stamp, page, limits)?;
        let batch =
            export_parquet_encoded(&input.rows, &input.metadata, &cold_schema(), limits, self.0)?;
        Ok(ColdHistoryPage {
            requested_after: page.requested_after,
            through_cursor: page.through_cursor,
            head_cursor: page.head_cursor,
            source_manifest: stamp.manifest_id.clone(),
            artifact: HistoryArtifact {
                bytes: batch.bytes,
                row_groups: batch.row_groups,
            },
        })
    }
    fn decode(
        &self,
        bytes: Vec<u8>,
        expected: &ColdHistoryReadExpectation,
        limits: ColdHistoryReadLimits,
    ) -> std::result::Result<DecodedColdHistoryPage, ColdHistoryReadError> {
        decode_cold_history_page_with_clock(bytes, expected, limits, Instant::now)
    }
}

fn decode_cold_history_page_with_clock(
    bytes: Vec<u8>,
    expected: &ColdHistoryReadExpectation,
    limits: ColdHistoryReadLimits,
    mut now: impl FnMut() -> Instant,
) -> std::result::Result<DecodedColdHistoryPage, ColdHistoryReadError> {
    let mut check = || {
        if now() >= limits.deadline {
            Err(ColdHistoryReadError::DeadlineExceeded)
        } else {
            Ok(())
        }
    };
    check()?;
    admit(&bytes, expected, limits)?;
    let encoded_bytes = bytes.len();
    let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(bytes))
        .map_err(|error| Error::Integrity(format!("invalid cold history Parquet: {error}")))?;
    let metadata = builder.metadata();
    if metadata.file_metadata().num_rows() < 0
        || metadata.file_metadata().num_rows() as u64 != expected.rows
    {
        return Err(Error::Integrity("cold history row count differs from receipt".into()).into());
    }
    let entries = metadata
        .file_metadata()
        .key_value_metadata()
        .ok_or_else(|| Error::Integrity("cold history metadata is absent".into()))?;
    let metadata_value = |key: &str| -> Result<&str> {
        let mut matches = entries.iter().filter(|entry| entry.key == key);
        let value = matches
            .next()
            .and_then(|entry| entry.value.as_deref())
            .ok_or_else(|| Error::Integrity(format!("cold history metadata {key} is absent")))?;
        if matches.next().is_some() {
            return Err(Error::Integrity(format!(
                "duplicate cold history metadata {key}"
            )));
        }
        Ok(value)
    };
    if metadata_value("rrflow.cold.format_version")? != COLD_HISTORY_FORMAT_VERSION.to_string()
        || serde_json::from_str::<ReadStamp>(metadata_value("rrflow.cold.read_stamp")?)?
            != expected.stamp
        || metadata_value("rrflow.cold.requested_after")? != expected.requested_after.to_string()
        || metadata_value("rrflow.cold.through_cursor")? != expected.through_cursor.to_string()
        || metadata_value("rrflow.cold.head_cursor")? != expected.stamp.commit_cursor.to_string()
    {
        return Err(Error::Integrity("cold history metadata differs from receipt".into()).into());
    }
    let row_groups = metadata.row_groups().len();
    let mut uncompressed = 0u64;
    for group in metadata.row_groups() {
        for column in group.columns() {
            let size = u64::try_from(column.uncompressed_size())
                .map_err(|_| Error::Integrity("negative Parquet column size".into()))?;
            uncompressed = uncompressed
                .checked_add(size)
                .ok_or_else(|| Error::Budget("cold history decoded size overflow".into()))?;
        }
    }
    if uncompressed > limits.max_decoded_bytes as u64 {
        return Err(Error::Budget("cold history decoded-byte budget exceeded".into()).into());
    }
    // An admission estimate, not an allocator-level peak-RSS guarantee.
    let reserved = usize::try_from(uncompressed)
        .ok()
        .and_then(|size| size.checked_mul(32))
        .and_then(|size| size.checked_add(encoded_bytes))
        .ok_or_else(|| Error::Budget("cold history read reservation overflow".into()))?;
    if reserved > limits.max_memory_bytes {
        return Err(
            Error::Budget("cold history read reservation exceeds memory budget".into()).into(),
        );
    }
    let mut changes = Vec::new();
    changes
        .try_reserve_exact(expected.rows as usize)
        .map_err(|_| Error::Budget("cold history result allocation failed".into()))?;
    let mut observed_arrow_bytes = 0usize;
    let mut previous = expected.requested_after;
    let reader = builder
        .with_batch_size(limits.max_batch_rows)
        .build()
        .map_err(|error| Error::Integrity(format!("cold history reader failed: {error}")))?;
    for batch in reader {
        check()?;
        let batch = batch
            .map_err(|error| Error::Integrity(format!("cold history batch failed: {error}")))?;
        observed_arrow_bytes = observed_arrow_bytes
            .checked_add(batch.get_array_memory_size())
            .ok_or_else(|| Error::Budget("cold history observed bytes overflow".into()))?;
        if observed_arrow_bytes > limits.max_decoded_bytes {
            return Err(
                Error::Budget("cold history observed Arrow bytes exceed budget".into()).into(),
            );
        }
        for row in record_batch_to_rows(&batch).map_err(Error::from)? {
            check()?;
            let change = checked_change(row, expected, &mut previous)?;
            changes.push(change);
        }
    }
    if changes.len() as u64 != expected.rows {
        return Err(
            Error::Integrity("cold history decoded row count differs from receipt".into()).into(),
        );
    }
    check()?;
    Ok(DecodedColdHistoryPage {
        changes,
        encoded_bytes,
        row_groups,
        declared_uncompressed_bytes: uncompressed,
        observed_arrow_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rrflow_core::{RuntimeReadValidation, ScopeId};
    use std::time::Duration;
    #[test]
    fn empty_artifact_cannot_finish_after_deadline() {
        let stamp =
            ReadStamp::new(ScopeId::new("instance:cold").unwrap(), None, 0, 0, None).unwrap();
        let page = RuntimeChangePage {
            requested_after: 0,
            through_cursor: 0,
            head_cursor: 0,
            validation: RuntimeReadValidation::new("test", 0, 0),
            changes: Vec::new(),
        };
        let end = Instant::now() + Duration::from_secs(30);
        let encoded = ParquetHistory::default()
            .encode(
                &stamp,
                &page,
                WriteLimits {
                    max_rows: 1,
                    max_columns: 32,
                    max_input_bytes: 1024 * 1024,
                    max_output_bytes: 1024 * 1024,
                    row_group_rows: 1,
                    max_memory_bytes: 64 * 1024 * 1024,
                    deadline: end,
                },
            )
            .unwrap()
            .artifact
            .bytes;
        let expected = ColdHistoryReadExpectation {
            stamp,
            requested_after: 0,
            through_cursor: 0,
            rows: 0,
            artifact_sha256: rrflow_core::digest::sha256_hex(&encoded),
        };
        let limits = ColdHistoryReadLimits {
            max_rows: 1,
            max_encoded_bytes: 1024 * 1024,
            max_decoded_bytes: 1024 * 1024,
            max_memory_bytes: 64 * 1024 * 1024,
            max_batch_rows: 1,
            deadline: end,
        };
        let mut calls = 0;
        let result = decode_cold_history_page_with_clock(encoded, &expected, limits, || {
            calls += 1;
            if calls == 1 {
                end - Duration::from_secs(1)
            } else {
                end
            }
        });
        assert!(matches!(
            result,
            Err(ColdHistoryReadError::DeadlineExceeded)
        ));
        assert!(calls >= 2);
    }
}
