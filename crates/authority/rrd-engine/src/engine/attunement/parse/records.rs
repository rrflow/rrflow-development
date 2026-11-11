//! Runtime records for parsed source. `source-parse` is the per-file state that
//! makes Parse resumable; its fact counts name the records a later edit retires.

use super::*;
use rrd_core::RuntimeRetirement;
use rrd_store::{RuntimeReadBudget, RuntimeVersionedSource};
use std::collections::{BTreeMap, BTreeSet};

const STATE: &str = "source-parse";
const SYMBOL: &str = "source-symbol";
const SPAN: &str = "source-span";
const CHUNK: &str = "source-chunk";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Status {
    Parsed,
    /// The parser refused this content; only a change to the file can help.
    Rejected,
    /// The worker died or timed out on this file; a later job tries again.
    Lost,
}

/// What is known about one file. Serialized as a `source-parse` record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct FileState {
    pub path: String,
    pub content_sha256: String,
    pub language: String,
    pub parser_revision: String,
    pub chunk_bytes: u64,
    /// The job that produced this state.
    pub job_id: String,
    pub status: Status,
    /// Why the parser refused or lost this file, when it did.
    pub reason: String,
    pub facts_sha256: String,
    pub symbols: u64,
    pub spans: u64,
    pub chunks: u64,
    pub nodes: u64,
    pub error_nodes: u64,
}

fn digest16(value: &str) -> String {
    digest::sha256_hex(value.as_bytes())[..16].to_owned()
}

fn fact_id(
    path: &str,
    content_sha256: &str,
    revision: &str,
    chunk_bytes: u64,
    index: u64,
) -> String {
    format!(
        "{}-{}-{}",
        digest::sha256_hex(
            format!("{path}\0{content_sha256}\0{revision}\0{chunk_bytes}").as_bytes()
        )[..32]
            .to_owned(),
        digest16(path),
        index
    )
}

fn state_id(path: &str) -> String {
    digest::sha256_hex(path.as_bytes())[..40].to_owned()
}

fn runtime_value(value: serde_json::Value) -> Result<RuntimeValue> {
    Ok(match value {
        serde_json::Value::String(text) => RuntimeValue::String(text),
        serde_json::Value::Number(number) => {
            RuntimeValue::Unsigned(number.as_u64().ok_or_else(invalid)?)
        }
        _ => return Err(invalid()),
    })
}

fn invalid() -> ServiceError {
    ServiceError::Storage("invalid parsed source record".into())
}

fn record(
    kind: &str,
    id: String,
    at: u64,
    properties: serde_json::Value,
) -> Result<RuntimeMutation> {
    let serde_json::Value::Object(map) = properties else {
        return Err(invalid());
    };
    Ok(RuntimeMutation::Record {
        record: RuntimeRecord {
            reference: RuntimeRef::new(kind, id).map_err(contract)?,
            valid_from: at,
            valid_to: None,
            properties: map
                .into_iter()
                .map(|(key, value)| Ok((key, runtime_value(value)?)))
                .collect::<Result<RuntimeProperties>>()?,
        },
    })
}

fn retire(kind: &str, id: String, at: u64) -> Result<RuntimeMutation> {
    Ok(RuntimeMutation::Retire {
        retirement: RuntimeRetirement {
            model: RuntimeLogicalModel::GraphNode,
            reference: RuntimeRef::new(kind, id).map_err(contract)?,
            effective_at: at,
        },
    })
}

fn state_record(state: &FileState, at: u64) -> Result<RuntimeMutation> {
    record(
        STATE,
        state_id(&state.path),
        at,
        serde_json::to_value(state).map_err(contract)?,
    )
}

/// Reads a `source-parse` record back into the state it was written from.
fn state_from(record: &RuntimeRecord) -> Result<FileState> {
    let plain = record
        .properties
        .iter()
        .map(|(key, value)| {
            Ok((
                key.clone(),
                match value {
                    RuntimeValue::String(text) => serde_json::json!(text),
                    RuntimeValue::Unsigned(number) => serde_json::json!(number),
                    _ => return Err(invalid()),
                },
            ))
        })
        .collect::<Result<serde_json::Map<_, _>>>()?;
    serde_json::from_value(serde_json::Value::Object(plain)).map_err(contract)
}

fn register(registry: &mut RuntimeSchemaRegistry) -> Result<bool> {
    use RuntimeValueType::{String as Text, Unsigned};
    let path = ("path", Text);
    let content = ("content_sha256", Text);
    let start = ("start_byte", Unsigned);
    let end = ("end_byte", Unsigned);
    let line = ("start_line", Unsigned);
    let tables: [(&str, Vec<(&str, RuntimeValueType)>); 4] = [
        (
            STATE,
            vec![
                path,
                content,
                ("language", Text),
                ("parser_revision", Text),
                ("chunk_bytes", Unsigned),
                ("job_id", Text),
                ("status", Text),
                ("reason", Text),
                ("facts_sha256", Text),
                ("symbols", Unsigned),
                ("spans", Unsigned),
                ("chunks", Unsigned),
                ("nodes", Unsigned),
                ("error_nodes", Unsigned),
            ],
        ),
        (
            SYMBOL,
            vec![
                path,
                content,
                ("language", Text),
                ("name", Text),
                ("symbol_kind", Text),
                ("role", Text),
                start,
                end,
                line,
                ("count", Unsigned),
            ],
        ),
        (
            SPAN,
            vec![
                path,
                content,
                ("language", Text),
                ("span_kind", Text),
                start,
                end,
                line,
                ("end_line", Unsigned),
            ],
        ),
        (
            CHUNK,
            vec![
                path,
                content,
                ("language", Text),
                ("text", Text),
                start,
                end,
                line,
                ("end_line", Unsigned),
            ],
        ),
    ];
    let mut changed = false;
    for (kind, fields) in tables {
        changed |= registry
            .define_record_table(
                RuntimeType::new(kind).map_err(contract)?,
                RuntimeLogicalModel::GraphNode,
                RuntimeRecordSchema {
                    properties: fields
                        .into_iter()
                        .map(|(name, kind)| (name.into(), RuntimePropertySchema::required(kind)))
                        .collect(),
                    allow_additional_properties: false,
                    unique_properties: BTreeSet::new(),
                },
            )
            .map_err(contract)?;
    }
    Ok(changed)
}

/// The latest non-retired `source-parse` state per path at `read`.
pub(super) fn load_states(
    engine: &RrdEngine,
    read: &ReadStamp,
    remaining: &mut u64,
) -> Result<BTreeMap<String, FileState>> {
    let source = RuntimeVersionedSource::Records {
        kind: Some(RuntimeType::new(STATE).map_err(contract)?),
    };
    let rows = engine.storage.runtime().read_versioned(
        read,
        &[source],
        RuntimeReadBudget::new(*remaining)?,
    )?;
    *remaining = remaining
        .checked_sub(rows.evidence.keys_examined)
        .ok_or_else(invalid)?;
    let mut latest = BTreeMap::<RuntimeRef, (u64, Option<RuntimeRecord>)>::new();
    for change in rows.changes {
        let (reference, value) = match change.mutation {
            RuntimeMutation::Record { record } => (record.reference.clone(), Some(record)),
            RuntimeMutation::Retire { retirement } => (retirement.reference, None),
            _ => continue,
        };
        let row = latest.entry(reference).or_insert((0, None));
        if change.cursor >= row.0 {
            *row = (change.cursor, value);
        }
    }
    let mut states = BTreeMap::new();
    for (_, record) in latest.into_values() {
        let Some(record) = record else { continue };
        let state = state_from(&record)?;
        states.insert(state.path.clone(), state);
    }
    Ok(states)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parsed_source_tables_are_registered_once() {
        let mut registry = RuntimeSchemaRegistry::empty(1, "parsed source");
        assert!(register(&mut registry).unwrap());
        assert!(
            !register(&mut registry).unwrap(),
            "a second registration changes nothing"
        );
        for kind in [STATE, SYMBOL, SPAN, CHUNK] {
            assert!(registry
                .records
                .contains_key(&RuntimeType::new(kind).unwrap()));
        }
    }

    #[test]
    fn a_fact_identity_changes_with_path_content_parser_chunk_size_and_index() {
        let base = fact_id("a.rs", "c1", "r1", 1_500, 0);
        assert_eq!(base, fact_id("a.rs", "c1", "r1", 1_500, 0));
        for other in [
            fact_id("b.rs", "c1", "r1", 1_500, 0),
            fact_id("a.rs", "c2", "r1", 1_500, 0),
            fact_id("a.rs", "c1", "r2", 1_500, 0),
            fact_id("a.rs", "c1", "r1", 1_000, 0),
            fact_id("a.rs", "c1", "r1", 1_500, 1),
        ] {
            assert_ne!(base, other);
        }
    }

    #[test]
    fn a_file_state_reads_back_from_its_record() {
        let state = FileState {
            path: "src/lib.rs".into(),
            content_sha256: "a".repeat(64),
            language: "rust".into(),
            parser_revision: "tree-sitter-rust".into(),
            chunk_bytes: 1_500,
            job_id: "job".into(),
            status: Status::Rejected,
            reason: "file_too_large".into(),
            facts_sha256: "0".repeat(64),
            symbols: 3,
            spans: 2,
            chunks: 1,
            nodes: 40,
            error_nodes: 0,
        };
        let RuntimeMutation::Record { record } = state_record(&state, 7).unwrap() else {
            panic!("a state is written as a record");
        };
        assert_eq!(record.reference.kind.as_str(), STATE);
        assert_eq!(state_from(&record).unwrap(), state);
    }
}
