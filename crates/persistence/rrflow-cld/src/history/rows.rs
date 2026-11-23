use super::*;
use crate::{QueryFieldTypes, QueryRow};
use rrflow_kernel::{RuntimeMutation, RuntimeValue, RuntimeValueType};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

pub(super) struct PreparedRows {
    pub rows: Vec<QueryRow>,
    #[cfg(feature = "cld-parquet")]
    pub metadata: BTreeMap<String, String>,
    #[cfg(feature = "cld-vortex")]
    pub input_bytes: usize,
}

pub(super) fn prepare(
    stamp: &ReadStamp,
    page: &RuntimeChangePage,
    limits: WriteLimits,
) -> Result<PreparedRows> {
    if page.head_cursor != stamp.commit_cursor
        || page.requested_after > page.through_cursor
        || page.through_cursor > page.head_cursor
        || page.changes.len() > limits.max_rows
        || limits.max_input_bytes == 0
    {
        return Err(Error::Integrity(
            "cold history page coordinates or row limit are invalid".into(),
        ));
    }

    let mut remaining = limits.max_input_bytes;
    let _stamp_json = bounded_json(stamp, &mut remaining, limits.deadline)?;
    stamp
        .validate()
        .map_err(|error| Error::Integrity(error.to_string()))?;
    let row_headers = page
        .changes
        .len()
        .checked_mul(std::mem::size_of::<QueryRow>())
        .ok_or_else(|| Error::Budget("cold history row allocation overflows".into()))?;
    if row_headers > limits.max_memory_bytes {
        return Err(Error::Budget(
            "cold history row allocation exceeds memory budget".into(),
        ));
    }
    let mut rows = Vec::new();
    rows.try_reserve_exact(page.changes.len())
        .map_err(|_| Error::Budget("cold history row allocation failed".into()))?;
    let mut previous = page.requested_after;
    for change in &page.changes {
        if Instant::now() >= limits.deadline {
            return Err(Error::DeadlineExceeded(
                "cold history deadline exceeded".into(),
            ));
        }
        if change.scope != stamp.scope
            || change.cursor <= previous
            || change.cursor > page.through_cursor
        {
            return Err(Error::Integrity(
                "cold history change scope, cursor, or digest is invalid".into(),
            ));
        }
        previous = change.cursor;
        let encoded = bounded_json(change, &mut remaining, limits.deadline)?;
        if !change.verify_digest() {
            return Err(Error::Integrity(
                "cold history change digest is invalid".into(),
            ));
        }
        rows.push(change_row(change, encoded));
    }

    #[cfg(feature = "cld-parquet")]
    let metadata = BTreeMap::from([
        (
            "rrflow.cold.format_version".into(),
            COLD_HISTORY_FORMAT_VERSION.to_string(),
        ),
        ("rrflow.cold.read_stamp".into(), _stamp_json),
        (
            "rrflow.cold.requested_after".into(),
            page.requested_after.to_string(),
        ),
        (
            "rrflow.cold.through_cursor".into(),
            page.through_cursor.to_string(),
        ),
        (
            "rrflow.cold.head_cursor".into(),
            page.head_cursor.to_string(),
        ),
    ]);
    Ok(PreparedRows {
        rows,
        #[cfg(feature = "cld-parquet")]
        metadata,
        #[cfg(feature = "cld-vortex")]
        input_bytes: limits.max_input_bytes - remaining,
    })
}

pub(super) fn cold_schema() -> QueryFieldTypes {
    let mut fields = QueryFieldTypes::new();
    for name in [
        "scope",
        "commit_id",
        "family",
        "kind",
        "record_id",
        "subject",
        "predicate",
        "change_digest",
        "previous_digest",
        "change_json",
    ] {
        fields.insert(name.into(), vec![RuntimeValueType::String]);
    }
    for name in ["cursor", "commit_ordinal", "committed_at", "valid_at"] {
        fields.insert(name.into(), vec![RuntimeValueType::Unsigned]);
    }
    fields
}

fn change_row(change: &RuntimeChange, encoded: String) -> QueryRow {
    let (family, reference, subject, predicate, valid_at) = match &change.mutation {
        RuntimeMutation::Schema { .. } => ("schema", None, None, None, None),
        RuntimeMutation::Claim { claim } => (
            "claim",
            None,
            Some(claim.subject.as_str()),
            Some(claim.predicate.as_str()),
            Some(claim.valid_from),
        ),
        RuntimeMutation::Record { record } => (
            "record",
            Some(&record.reference),
            None,
            None,
            Some(record.valid_from),
        ),
        RuntimeMutation::Relation { relation } => (
            "relation",
            Some(&relation.reference),
            None,
            None,
            Some(relation.valid_from),
        ),
        RuntimeMutation::Event { event } => ("event", None, None, Some(event.kind.as_str()), None),
        RuntimeMutation::Vector { vector } => (
            "vector",
            Some(&vector.reference),
            None,
            None,
            Some(vector.valid_from),
        ),
        RuntimeMutation::SeriesSample { sample } => (
            "series_sample",
            Some(&sample.reference),
            None,
            None,
            Some(sample.observed_at),
        ),
        RuntimeMutation::Geo { geo } => (
            "geo",
            Some(&geo.reference),
            None,
            None,
            Some(geo.valid_from),
        ),
        RuntimeMutation::Object { object } => ("object", Some(&object.reference), None, None, None),
        RuntimeMutation::Retire { retirement } => (
            "retire",
            Some(&retirement.reference),
            None,
            None,
            Some(retirement.effective_at),
        ),
    };
    let text = |value: Option<&str>| {
        value.map_or(RuntimeValue::Null, |value| {
            RuntimeValue::String(value.into())
        })
    };
    let values = BTreeMap::from([
        (
            "scope".into(),
            RuntimeValue::String(change.scope.to_string()),
        ),
        ("cursor".into(), RuntimeValue::Unsigned(change.cursor)),
        (
            "commit_id".into(),
            RuntimeValue::String(change.commit_id.clone()),
        ),
        (
            "commit_ordinal".into(),
            RuntimeValue::Unsigned(change.commit_ordinal),
        ),
        ("committed_at".into(), RuntimeValue::Unsigned(change.at)),
        ("family".into(), RuntimeValue::String(family.into())),
        (
            "kind".into(),
            text(reference.map(|value| value.kind.as_str())),
        ),
        (
            "record_id".into(),
            text(reference.map(|value| value.id.as_str())),
        ),
        ("subject".into(), text(subject)),
        ("predicate".into(), text(predicate)),
        (
            "valid_at".into(),
            valid_at.map_or(RuntimeValue::Null, RuntimeValue::Unsigned),
        ),
        (
            "change_digest".into(),
            RuntimeValue::String(change.digest.clone()),
        ),
        (
            "previous_digest".into(),
            text(change.previous_digest.as_deref()),
        ),
        ("change_json".into(), RuntimeValue::String(encoded)),
    ]);
    QueryRow {
        identity: format!("change:{}", change.cursor),
        values,
    }
}

struct BoundedJson {
    bytes: Vec<u8>,
    limit: usize,
    deadline: Instant,
}

impl Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if Instant::now() >= self.deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "cold history JSON deadline exceeded",
            ));
        }
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("cold history JSON budget exceeded"));
        }
        self.bytes
            .try_reserve(bytes.len())
            .map_err(io::Error::other)?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn bounded_json<T: serde::Serialize>(
    value: &T,
    remaining: &mut usize,
    deadline: Instant,
) -> Result<String> {
    let mut output = BoundedJson {
        bytes: Vec::new(),
        limit: *remaining,
        deadline,
    };
    serde_json::to_writer(&mut output, value).map_err(|error| {
        if error.io_error_kind() == Some(io::ErrorKind::TimedOut) {
            Error::DeadlineExceeded(error.to_string())
        } else {
            Error::Budget(error.to_string())
        }
    })?;
    *remaining -= output.bytes.len();
    String::from_utf8(output.bytes).map_err(|error| Error::Integrity(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn cold_json_preserves_deadline_and_resource_identity() {
        let mut limit = 1024;
        assert!(matches!(
            bounded_json(&"payload", &mut limit, Instant::now()),
            Err(Error::DeadlineExceeded(_))
        ));
        let mut limit = 1;
        assert!(matches!(
            bounded_json(
                &"payload",
                &mut limit,
                Instant::now() + Duration::from_secs(30)
            ),
            Err(Error::Budget(_))
        ));
    }
}

pub(super) fn checked_change(
    row: QueryRow,
    expected: &ColdHistoryReadExpectation,
    previous: &mut u64,
) -> std::result::Result<RuntimeChange, ColdHistoryReadError> {
    let Some(RuntimeValue::String(json)) = row.values.get("change_json") else {
        return Err(Error::Integrity("cold history change payload is absent".into()).into());
    };
    let change: RuntimeChange = serde_json::from_str(json)?;
    if change.scope != expected.stamp.scope
        || change.cursor <= *previous
        || change.cursor > expected.through_cursor
        || !change.verify_digest()
        || change_row(&change, json.clone()) != row
    {
        return Err(
            Error::Integrity("cold history row does not match its typed change".into()).into(),
        );
    }
    *previous = change.cursor;
    Ok(change)
}
