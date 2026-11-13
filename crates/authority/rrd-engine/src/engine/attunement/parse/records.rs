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

/// What one Step learned about one file.
pub(super) enum Outcome<'a> {
    Parsed(&'a rrd_attunement::facts::SourceFacts),
    /// The parser refused this content, with the worker's code.
    Rejected(String),
    /// The worker died or ran out of time on this file.
    Lost(String),
}

/// `source[start..end]` as text, refusing a range the bytes do not hold.
fn slice(source: &[u8], start: u64, end: u64) -> Result<&str> {
    let (start, end) = (
        usize::try_from(start).map_err(|_| invalid())?,
        usize::try_from(end).map_err(|_| invalid())?,
    );
    if start > end {
        return Err(invalid());
    }
    let bytes = source.get(start..end).ok_or_else(invalid)?;
    std::str::from_utf8(bytes).map_err(|_| invalid())
}

fn fact_ids(state: &FileState) -> [(&'static str, BTreeSet<String>); 3] {
    let ids = |count: u64| {
        (0..count)
            .map(|index| {
                fact_id(
                    &state.path,
                    &state.content_sha256,
                    &state.parser_revision,
                    state.chunk_bytes,
                    index,
                )
            })
            .collect()
    };
    [
        (SYMBOL, ids(state.symbols)),
        (SPAN, ids(state.spans)),
        (CHUNK, ids(state.chunks)),
    ]
}

/// The mutations that make `next` the file's current state: retire every fact
/// the previous state named that the new one does not, write the new facts with
/// names and text sliced from `source`, the bytes the engine read and verified
/// itself, and write the state. A worker's facts never supply content.
pub(super) fn file_mutations(
    previous: Option<&FileState>,
    mut next: FileState,
    outcome: Outcome<'_>,
    source: &[u8],
    at: u64,
) -> Result<Vec<RuntimeMutation>> {
    let mut written = Vec::new();
    match outcome {
        Outcome::Parsed(facts) => {
            if facts.content_sha256 != next.content_sha256
                || facts.language != next.language
                || facts.parser_revision != next.parser_revision
                || digest::sha256_hex(source) != next.content_sha256
            {
                return Err(invalid());
            }
            let common = |path: &str| {
                serde_json::json!({
                    "path": path,
                    "content_sha256": next.content_sha256,
                    "language": next.language,
                })
            };
            let id = |index: usize| {
                fact_id(
                    &next.path,
                    &next.content_sha256,
                    &next.parser_revision,
                    next.chunk_bytes,
                    index as u64,
                )
            };
            for (index, symbol) in facts.symbols.iter().enumerate() {
                let mut properties = common(&next.path);
                properties["name"] = slice(source, symbol.name_start, symbol.name_end)?.into();
                properties["symbol_kind"] = symbol.kind.as_str().into();
                properties["role"] = match symbol.role {
                    rrd_attunement::facts::SymbolRole::Definition => "definition",
                    rrd_attunement::facts::SymbolRole::Reference => "reference",
                }
                .into();
                slice(source, symbol.start_byte, symbol.end_byte)?;
                properties["start_byte"] = symbol.start_byte.into();
                properties["end_byte"] = symbol.end_byte.into();
                properties["start_line"] = symbol.start_line.into();
                properties["count"] = symbol.count.into();
                written.push(record(SYMBOL, id(index), at, properties)?);
            }
            for (index, span) in facts.spans.iter().enumerate() {
                slice(source, span.start_byte, span.end_byte)?;
                let mut properties = common(&next.path);
                properties["span_kind"] = span.kind.as_str().into();
                properties["start_byte"] = span.start_byte.into();
                properties["end_byte"] = span.end_byte.into();
                properties["start_line"] = span.start_line.into();
                properties["end_line"] = span.end_line.into();
                written.push(record(SPAN, id(index), at, properties)?);
            }
            for (index, chunk) in facts.chunks.iter().enumerate() {
                if chunk.end_byte.saturating_sub(chunk.start_byte) > next.chunk_bytes {
                    return Err(invalid());
                }
                let mut properties = common(&next.path);
                properties["text"] = slice(source, chunk.start_byte, chunk.end_byte)?.into();
                properties["start_byte"] = chunk.start_byte.into();
                properties["end_byte"] = chunk.end_byte.into();
                properties["start_line"] = chunk.start_line.into();
                properties["end_line"] = chunk.end_line.into();
                written.push(record(CHUNK, id(index), at, properties)?);
            }
            next.status = Status::Parsed;
            next.reason = String::new();
            next.facts_sha256 = digest::sha256_hex(&serde_json::to_vec(facts).map_err(contract)?);
            next.symbols = facts.symbols.len() as u64;
            next.spans = facts.spans.len() as u64;
            next.chunks = facts.chunks.len() as u64;
            next.nodes = facts.nodes;
            next.error_nodes = facts.error_nodes;
        }
        Outcome::Rejected(reason) => {
            next.status = Status::Rejected;
            next.reason = reason;
            next.facts_sha256 = "0".repeat(64);
            (next.symbols, next.spans, next.chunks) = (0, 0, 0);
            (next.nodes, next.error_nodes) = (0, 0);
        }
        Outcome::Lost(reason) => {
            next.status = Status::Lost;
            next.reason = reason;
            next.facts_sha256 = "0".repeat(64);
            (next.symbols, next.spans, next.chunks) = (0, 0, 0);
            (next.nodes, next.error_nodes) = (0, 0);
        }
    }
    let mut mutations = Vec::new();
    if let Some(previous) = previous {
        let kept = fact_ids(&next);
        for ((kind, before), (_, after)) in fact_ids(previous).into_iter().zip(kept) {
            for id in before.difference(&after) {
                mutations.push(retire(kind, id.clone(), at)?);
            }
        }
    }
    mutations.extend(written);
    mutations.push(state_record(&next, at)?);
    Ok(mutations)
}

/// The parsed-source tables registered in the instance's schema, when they are
/// not there yet.
pub(super) fn schema_mutation(
    engine: &RrdEngine,
    scope: &ScopeId,
    read: &ReadStamp,
) -> Result<Option<RuntimeMutation>> {
    let current = engine.storage.runtime().schema(scope)?;
    if current.as_ref().map(|schema| schema.revision) != read.schema_revision {
        return Err(ServiceError::StorageConflict("Parse schema changed".into()));
    }
    let mut registry = current
        .clone()
        .unwrap_or_else(|| RuntimeSchemaRegistry::empty(1, "parsed source tables"));
    if !register(&mut registry)? {
        return Ok(None);
    }
    if let Some(old) = current {
        registry.revision = old.revision.checked_add(1).ok_or_else(invalid)?;
        registry.migration = "parsed source tables".into();
    }
    Ok(Some(RuntimeMutation::Schema { registry }))
}

/// The indexes parsed source is read through: exact lookups of a symbol by
/// name and of every fact of a file by path, and BM25 over chunk text that
/// indexes code identifiers whole and by their words. Each is maintained in
/// the commit that writes the facts, so a question reads current facts
/// without scanning them.
pub(super) fn index_definitions() -> Result<Vec<rrd_query::IndexDefinition>> {
    let record = |kind: &str| -> Result<rrd_query::Source> {
        Ok(rrd_query::Source::Record {
            kind: RuntimeType::new(kind).map_err(contract)?,
        })
    };
    let definition = |id: &str, kind: &str, field: &str, index: rrd_query::IndexKind| {
        Ok(rrd_query::IndexDefinition {
            id: rrd_core::ProjectionId::new(id).map_err(contract)?,
            source: record(kind)?,
            fields: vec![field.into()],
            unique: false,
            kind: index,
            filters: Vec::new(),
        })
    };
    let code = rrd_query::Bm25Config {
        identifiers: rrd_query::Bm25Identifiers::WholeAndParts,
        ..rrd_query::Bm25Config::default()
    };
    [
        definition(
            "source-symbol-name",
            SYMBOL,
            "name",
            rrd_query::IndexKind::Scalar,
        ),
        definition(
            "source-symbol-path",
            SYMBOL,
            "path",
            rrd_query::IndexKind::Scalar,
        ),
        definition(
            "source-span-path",
            SPAN,
            "path",
            rrd_query::IndexKind::Scalar,
        ),
        definition(
            "source-chunk-path",
            CHUNK,
            "path",
            rrd_query::IndexKind::Scalar,
        ),
        definition(
            "source-chunk-text",
            CHUNK,
            "text",
            rrd_query::IndexKind::Bm25 { config: code },
        ),
    ]
    .into_iter()
    .collect()
}

/// Defines any of [`index_definitions`] the scope does not have yet, once the
/// parsed-source tables exist. The store backfills facts already published.
pub(super) fn ensure_indexes(
    engine: &RrdEngine,
    scope: &ScopeId,
    context: &rrd_query::IndexMutationContext,
) -> Result<()> {
    let Some(schema) = engine.storage.runtime().schema(scope)? else {
        return Ok(());
    };
    if !schema
        .records
        .contains_key(&RuntimeType::new(CHUNK).map_err(contract)?)
    {
        return Ok(());
    }
    let repository = rrd_query::IndexCatalogueRepository::new(&engine.storage, scope.clone());
    let existing = repository
        .load()
        .map_err(|error| ServiceError::Query(error.to_string()))?;
    let missing = index_definitions()?
        .into_iter()
        .filter(|definition| !existing.entries.contains_key(&definition.id))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    let mut sources = Vec::new();
    for definition in &missing {
        if !sources.contains(&definition.source) {
            sources.push(definition.source.clone());
        }
    }
    let catalogue = rrd_query::Catalog::capture_for_sources(
        &engine.storage,
        scope,
        &sources,
        RuntimeReadBudget::new(engine.estate_configuration.query.max_storage_keys)?,
    )
    .map_err(|error| ServiceError::Query(error.to_string()))?;
    for definition in missing {
        repository
            .create(context, &catalogue, definition)
            .map_err(|error| ServiceError::Query(error.to_string()))?;
    }
    Ok(())
}

/// Retires a file that left the project or its parse bindings: its facts and
/// its state.
pub(super) fn retire_file(state: &FileState, at: u64) -> Result<Vec<RuntimeMutation>> {
    let mut mutations = Vec::new();
    for (kind, ids) in fact_ids(state) {
        for id in ids {
            mutations.push(retire(kind, id, at)?);
        }
    }
    mutations.push(retire(STATE, state_id(&state.path), at)?);
    Ok(mutations)
}

/// The state a mutation writes, when it writes one.
pub(super) fn state_of(mutation: &RuntimeMutation) -> Result<Option<FileState>> {
    match mutation {
        RuntimeMutation::Record { record } if record.reference.kind.as_str() == STATE => {
            Ok(Some(state_from(record)?))
        }
        _ => Ok(None),
    }
}

/// One digest of every file's current answer, recorded by the Parse checkpoint.
pub(super) fn summary_sha256(states: &BTreeMap<String, FileState>) -> Result<String> {
    let rows: Vec<_> = states
        .values()
        .map(|state| {
            (
                &state.path,
                &state.content_sha256,
                &state.status,
                &state.facts_sha256,
            )
        })
        .collect();
    Ok(digest::sha256_hex(
        &serde_json::to_vec(&rows).map_err(contract)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "fn alpha() {}\n\nfn beta() {\n    alpha();\n}\n";

    fn parsed(source: &str) -> (FileState, rrd_attunement::facts::SourceFacts) {
        let digest = digest::sha256_hex(source.as_bytes());
        let limits = rrd_attunement::parse::ParseLimits {
            input_bytes: 1 << 20,
            nodes: 1 << 20,
            depth: 256,
            output_bytes: 1 << 20,
            elapsed_micros: 5_000_000,
        };
        let facts = rrd_attunement::facts::source_facts(
            "src/lib.rs",
            &digest,
            "rust",
            source.as_bytes(),
            &limits,
            1_500,
            || false,
        )
        .unwrap();
        let state = FileState {
            path: "src/lib.rs".into(),
            content_sha256: digest,
            language: "rust".into(),
            parser_revision: facts.parser_revision.clone(),
            chunk_bytes: 1_500,
            job_id: "job".into(),
            status: Status::Lost,
            reason: String::new(),
            facts_sha256: String::new(),
            symbols: 0,
            spans: 0,
            chunks: 0,
            nodes: 0,
            error_nodes: 0,
        };
        (state, facts)
    }

    fn written<'a>(mutations: &'a [RuntimeMutation], kind: &str) -> Vec<&'a RuntimeRecord> {
        mutations
            .iter()
            .filter_map(|mutation| match mutation {
                RuntimeMutation::Record { record } if record.reference.kind.as_str() == kind => {
                    Some(record)
                }
                _ => None,
            })
            .collect()
    }

    fn retired(mutations: &[RuntimeMutation]) -> usize {
        mutations
            .iter()
            .filter(|mutation| matches!(mutation, RuntimeMutation::Retire { .. }))
            .count()
    }

    fn text(record: &RuntimeRecord, key: &str) -> String {
        match record.properties.get(key) {
            Some(RuntimeValue::String(value)) => value.clone(),
            other => panic!("{key} is {other:?}"),
        }
    }

    #[test]
    fn a_first_parse_writes_facts_sliced_from_the_verified_source() {
        let (state, facts) = parsed(SOURCE);
        let mutations =
            file_mutations(None, state, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
        let names: BTreeSet<_> = written(&mutations, SYMBOL)
            .into_iter()
            .map(|record| (text(record, "name"), text(record, "role")))
            .collect();
        assert!(names.contains(&("alpha".into(), "definition".into())));
        assert!(names.contains(&("beta".into(), "definition".into())));
        assert!(names.contains(&("alpha".into(), "reference".into())));
        let chunks = written(&mutations, CHUNK);
        assert_eq!(text(chunks[0], "text").trim_end(), SOURCE.trim_end());
        let states = written(&mutations, STATE);
        let state = state_from(states[0]).unwrap();
        assert_eq!(state.status, Status::Parsed);
        assert_eq!(state.symbols as usize, facts.symbols.len());
        assert_eq!(state.chunks as usize, chunks.len());
        assert_eq!(retired(&mutations), 0);
    }

    #[test]
    fn an_edit_retires_the_facts_of_the_old_content() {
        let (old, old_facts) = parsed(SOURCE);
        let first =
            file_mutations(None, old, Outcome::Parsed(&old_facts), SOURCE.as_bytes(), 9).unwrap();
        let previous = state_from(written(&first, STATE)[0]).unwrap();
        let edited = "fn gamma() {}\n";
        let (next, facts) = parsed(edited);
        let mutations = file_mutations(
            Some(&previous),
            next,
            Outcome::Parsed(&facts),
            edited.as_bytes(),
            10,
        )
        .unwrap();
        let before = (previous.symbols + previous.spans + previous.chunks) as usize;
        assert_eq!(retired(&mutations), before, "every old fact is retired");
        assert_eq!(text(written(&mutations, SYMBOL)[0], "name"), "gamma");
    }

    #[test]
    fn a_refusal_retires_old_facts_and_records_why() {
        let (old, facts) = parsed(SOURCE);
        let first =
            file_mutations(None, old, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
        let previous = state_from(written(&first, STATE)[0]).unwrap();
        let mutations = file_mutations(
            Some(&previous),
            previous.clone(),
            Outcome::Rejected("limit_exceeded".into()),
            SOURCE.as_bytes(),
            10,
        )
        .unwrap();
        let state = state_from(written(&mutations, STATE)[0]).unwrap();
        assert_eq!(
            (state.status, state.reason.as_str()),
            (Status::Rejected, "limit_exceeded")
        );
        assert_eq!((state.symbols, state.spans, state.chunks), (0, 0, 0));
        assert_eq!(
            retired(&mutations),
            (previous.symbols + previous.spans + previous.chunks) as usize
        );
    }

    #[test]
    fn facts_that_do_not_match_the_verified_bytes_are_refused() {
        let (state, facts) = parsed(SOURCE);
        let other = SOURCE.replace("alpha", "omega");
        assert!(file_mutations(
            None,
            state.clone(),
            Outcome::Parsed(&facts),
            other.as_bytes(),
            9
        )
        .is_err());
        let mut outside = facts.clone();
        outside.symbols[0].name_end = SOURCE.len() as u64 + 1;
        assert!(
            file_mutations(None, state, Outcome::Parsed(&outside), SOURCE.as_bytes(), 9).is_err()
        );
    }

    #[test]
    fn a_removed_file_retires_its_facts_and_state() {
        let (state, facts) = parsed(SOURCE);
        let first =
            file_mutations(None, state, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
        let written = state_from(written(&first, STATE)[0]).unwrap();
        let removed = retire_file(&written, 10).unwrap();
        let facts = (written.symbols + written.spans + written.chunks) as usize;
        assert_eq!(removed.len(), facts + 1, "every fact and the state");
        assert_eq!(retired(&removed), removed.len());
    }

    #[test]
    fn states_are_found_and_summarized_by_their_answer() {
        let (state, facts) = parsed(SOURCE);
        let mutations =
            file_mutations(None, state, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
        let found: Vec<_> = mutations
            .iter()
            .filter_map(|mutation| state_of(mutation).unwrap())
            .collect();
        assert_eq!(found.len(), 1, "only the state record yields a state");
        let mut states = BTreeMap::from([(found[0].path.clone(), found[0].clone())]);
        let parsed_summary = summary_sha256(&states).unwrap();
        states.get_mut("src/lib.rs").unwrap().status = Status::Lost;
        assert_ne!(parsed_summary, summary_sha256(&states).unwrap());
    }

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
