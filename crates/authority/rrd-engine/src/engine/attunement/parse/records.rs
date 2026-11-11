//! Runtime records for parsed source. `source-parse` is the per-file state that
//! makes Parse resumable; its fact counts name the records a later edit retires.

use super::*;
use rrd_attunement::facts::{SourceFacts, SymbolRole};
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
    Rejected,
}

/// What is known about one file. Serialized as a `source-parse` record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct FileState {
    pub path: String,
    pub content_sha256: String,
    pub language: String,
    pub parser_revision: String,
    pub chunk_bytes: u64,
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

/// A record whose commit did not have room is left pending for the next Step.
pub(super) enum Admission {
    Admitted(FileState),
    Full,
}

impl Admission {
    pub(super) fn admitted(self) -> Option<FileState> {
        match self {
            Self::Admitted(state) => Some(state),
            Self::Full => None,
        }
    }
}

fn digest16(value: &str) -> String {
    digest::sha256_hex(value.as_bytes())[..16].to_owned()
}

fn fact_id(path: &str, content_sha256: &str, revision: &str, chunk_bytes: u64, index: u64) -> String {
    format!(
        "{}-{}-{}",
        digest::sha256_hex(format!("{path}\0{content_sha256}\0{revision}\0{chunk_bytes}").as_bytes())
            [..32]
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

fn record(kind: &str, id: String, at: u64, properties: serde_json::Value) -> Result<RuntimeMutation> {
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
        let value = serde_json::to_value(&record.properties).map_err(contract)?;
        let plain = plain_properties(value)?;
        let state: FileState = serde_json::from_value(plain).map_err(contract)?;
        states.insert(state.path.clone(), state);
    }
    Ok(states)
}

/// Runtime values serialize tagged; `FileState` reads the bare values.
fn plain_properties(value: serde_json::Value) -> Result<serde_json::Value> {
    let serde_json::Value::Object(map) = value else {
        return Err(invalid());
    };
    Ok(serde_json::Value::Object(
        map.into_iter()
            .map(|(key, value)| {
                let serde_json::Value::Object(tagged) = value else {
                    return Err(invalid());
                };
                let inner = tagged.into_iter().next().ok_or_else(invalid)?.1;
                Ok((key, inner))
            })
            .collect::<Result<_>>()?,
    ))
}

/// One Step's mutations, bounded in JSON bytes.
pub(super) struct ParseCommit<'a> {
    engine: &'a RrdEngine,
    scope: ScopeId,
    read: ReadStamp,
    mutations: Vec<RuntimeMutation>,
    bytes: u64,
    limit: u64,
}

impl<'a> ParseCommit<'a> {
    pub(super) fn new(
        engine: &'a RrdEngine,
        scope: &ScopeId,
        read: &ReadStamp,
        limit: u64,
    ) -> Result<Self> {
        let current = engine.storage.runtime().schema(scope)?;
        if current.as_ref().map(|schema| schema.revision) != read.schema_revision {
            return Err(ServiceError::StorageConflict(
                "Parse schema changed".into(),
            ));
        }
        let mut registry = current
            .clone()
            .unwrap_or_else(|| RuntimeSchemaRegistry::empty(1, "install parsed source tables"));
        let mut mutations = Vec::new();
        if register(&mut registry)? {
            if let Some(old) = current {
                registry.revision = old.revision.checked_add(1).ok_or_else(invalid)?;
                registry.migration = "install parsed source tables".into();
            }
            mutations.push(RuntimeMutation::Schema { registry });
        }
        Ok(Self {
            engine,
            scope: scope.clone(),
            read: read.clone(),
            mutations,
            bytes: 0,
            limit,
        })
    }

    fn admit(&mut self, group: Vec<RuntimeMutation>) -> Result<bool> {
        let size = serde_json::to_vec(&group).map_err(contract)?.len() as u64;
        // One file always fits alone; its size is bounded by the policy.
        if !self.mutations.is_empty() && self.bytes.saturating_add(size) > self.limit {
            return Ok(false);
        }
        self.bytes = self.bytes.saturating_add(size);
        self.mutations.extend(group);
        Ok(true)
    }

    fn retirements(state: &FileState, at: u64) -> Result<Vec<RuntimeMutation>> {
        let mut group = Vec::new();
        if state.status != Status::Parsed {
            return Ok(group);
        }
        for (kind, count) in [
            (SYMBOL, state.symbols),
            (SPAN, state.spans),
            (CHUNK, state.chunks),
        ] {
            for index in 0..count {
                let id = fact_id(
                    &state.path,
                    &state.content_sha256,
                    &state.parser_revision,
                    state.chunk_bytes,
                    index,
                );
                group.push(retire(kind, id, at)?);
            }
        }
        Ok(group)
    }

    pub(super) fn retire_file(&mut self, state: &FileState, at: u64) -> Result<()> {
        let mut group = Self::retirements(state, at)?;
        group.push(retire(STATE, state_id(&state.path), at)?);
        self.admit(group)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn reject(
        &mut self,
        entry: &Entry,
        content_sha256: &str,
        language: &str,
        reason: &str,
        chunk_bytes: u64,
        previous: Option<&FileState>,
        at: u64,
    ) -> Result<FileState> {
        let state = FileState {
            path: entry.path.clone(),
            content_sha256: content_sha256.to_owned(),
            language: language.to_owned(),
            parser_revision: parser_revision(language).unwrap_or_default().to_owned(),
            chunk_bytes,
            status: Status::Rejected,
            reason: reason.to_owned(),
            facts_sha256: "0".repeat(64),
            symbols: 0,
            spans: 0,
            chunks: 0,
            nodes: 0,
            error_nodes: 0,
        };
        let mut group = previous.map_or_else(|| Ok(Vec::new()), |old| Self::retirements(old, at))?;
        group.push(state_record(&state, at)?);
        self.admit(group)?;
        Ok(state)
    }

    pub(super) fn publish(
        &mut self,
        entry: &Entry,
        source: &[u8],
        facts: &SourceFacts,
        chunk_bytes: u64,
        previous: Option<&FileState>,
        at: u64,
    ) -> Result<Admission> {
        let text = std::str::from_utf8(source).map_err(|_| invalid())?;
        let slice = |start: u64, end: u64| {
            text.get(start as usize..end as usize)
                .ok_or_else(|| ServiceError::Storage("parser range is not a text boundary".into()))
        };
        let id = |index: u64| {
            fact_id(
                &entry.path,
                &facts.content_sha256,
                &facts.parser_revision,
                chunk_bytes,
                index,
            )
        };
        let mut group =
            previous.map_or_else(|| Ok(Vec::new()), |old| Self::retirements(old, at))?;
        let common = serde_json::json!({
            "path": entry.path,
            "content_sha256": facts.content_sha256,
            "language": facts.language,
        });
        let with = |extra: serde_json::Value| {
            let mut value = common.clone();
            value
                .as_object_mut()
                .expect("object")
                .extend(extra.as_object().expect("object").clone());
            value
        };
        for (index, symbol) in facts.symbols.iter().enumerate() {
            group.push(record(
                SYMBOL,
                id(index as u64),
                at,
                with(serde_json::json!({
                    "name": slice(symbol.name_start, symbol.name_end)?,
                    "symbol_kind": symbol.kind,
                    "role": match symbol.role {
                        SymbolRole::Definition => "definition",
                        SymbolRole::Reference => "reference",
                    },
                    "start_byte": symbol.start_byte,
                    "end_byte": symbol.end_byte,
                    "start_line": symbol.start_line,
                    "count": symbol.count,
                })),
            )?);
        }
        for (index, span) in facts.spans.iter().enumerate() {
            group.push(record(
                SPAN,
                id(index as u64),
                at,
                with(serde_json::json!({
                    "span_kind": span.kind,
                    "start_byte": span.start_byte,
                    "end_byte": span.end_byte,
                    "start_line": span.start_line,
                    "end_line": span.end_line,
                })),
            )?);
        }
        for (index, chunk) in facts.chunks.iter().enumerate() {
            group.push(record(
                CHUNK,
                id(index as u64),
                at,
                with(serde_json::json!({
                    "text": slice(chunk.start_byte, chunk.end_byte)?,
                    "start_byte": chunk.start_byte,
                    "end_byte": chunk.end_byte,
                    "start_line": chunk.start_line,
                    "end_line": chunk.end_line,
                })),
            )?);
        }
        let state = FileState {
            path: entry.path.clone(),
            content_sha256: facts.content_sha256.clone(),
            language: facts.language.clone(),
            parser_revision: facts.parser_revision.clone(),
            chunk_bytes,
            status: Status::Parsed,
            reason: String::new(),
            facts_sha256: digest::sha256_hex(&serde_json::to_vec(facts).map_err(contract)?),
            symbols: facts.symbols.len() as u64,
            spans: facts.spans.len() as u64,
            chunks: facts.chunks.len() as u64,
            nodes: facts.nodes,
            error_nodes: facts.error_nodes,
        };
        group.push(state_record(&state, at)?);
        Ok(if self.admit(group)? {
            Admission::Admitted(state)
        } else {
            Admission::Full
        })
    }

    /// Commits the facts with the job transition: a lease renewal while files
    /// remain, or the Parse checkpoint once every file is decided.
    pub(super) fn commit(
        self,
        work: &Work<'_>,
        mut permit: TransactionPermit<'_>,
        states: BTreeMap<String, FileState>,
        finished: bool,
        check: &dyn Fn() -> Result<()>,
    ) -> Result<AttunementControlResult> {
        let engine = self.engine;
        let at = elapsed_time(work.now, work.started)?;
        let commit = RuntimeCommit {
            scope: self.scope,
            at,
            actor: format!("session:{}", work.session.as_str()),
            expected_cursor: self.read.commit_cursor,
            mutations: self.mutations,
        };
        let admission = engine
            .estate_configuration
            .storage
            .transaction_admission_bytes;
        permit.charge(&commit, admission)?;
        let output_sha256 = operation_digest(&(
            "attunement-parse-v1",
            &work.plan.plan_sha256,
            states.values().collect::<Vec<_>>(),
        ))?;
        check()?;
        let (_, _, accepted) = engine.storage.runtime().commit_with_control_builder(
            &commit,
            |read, outcome| -> Result<_> {
                check()?;
                if outcome.last_cursor != read.commit_cursor {
                    return Err(ServiceError::Storage("Parse cursor mismatch".into()));
                }
                let done = elapsed_time(work.now, work.started)?;
                let before = &work.before;
                if done <= before.updated_at_unix_ms {
                    return Err(contract("attunement transition time must advance"));
                }
                let mut next = before.clone();
                next.revision = next
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| contract("attunement revision overflow"))?;
                next.updated_at_unix_ms = done;
                let action = if finished {
                    let selected = work
                        .plan
                        .phases
                        .get(next.checkpoints.len())
                        .ok_or_else(|| contract("missing selected Parse step"))?;
                    let mut checkpoint = AttunementPhaseCheckpoint {
                        contract_version: ATTUNEMENT_CONTRACT_VERSION,
                        job_id: next.id.clone(),
                        plan_sha256: work.plan.plan_sha256.clone(),
                        phase: AttunementPhase::Parse,
                        sequence: selected.sequence,
                        configuration_sha256: selected.configuration_sha256.clone(),
                        input_sha256: next
                            .checkpoints
                            .last()
                            .map(|last| last.output_sha256.clone())
                            .ok_or_else(|| contract("Parse has no Inventory checkpoint"))?,
                        output_sha256: output_sha256.clone(),
                        coordinates: AttunementRuntimeCoordinates {
                            runtime_manifest_sha256: read.manifest_id.clone(),
                            runtime_cursor: read.commit_cursor,
                            schema_revision: read.schema_revision,
                            catalogue_revision: read.catalog_revision,
                        },
                        committed_at_unix_ms: done,
                        checkpoint_sha256: "0".repeat(64),
                    };
                    checkpoint.checkpoint_sha256 =
                        attunement_checkpoint_sha256(&checkpoint).map_err(contract)?;
                    next.checkpoints.push(checkpoint);
                    advance_after_checkpoint(&mut next, &work.plan);
                    "attunement.parse_checkpointed"
                } else {
                    let lease = next
                        .lease
                        .as_mut()
                        .ok_or_else(|| contract("Parse step has no lease"))?;
                    let held = lease.expires_at_unix_ms - lease.acquired_at_unix_ms;
                    lease.generation = next.revision;
                    lease.expires_at_unix_ms = done.saturating_add(held);
                    "attunement.parse_progressed"
                };
                before
                    .validate_transition(&next, &work.plan)
                    .map_err(contract)?;
                let accepted = status(next, &work.plan, done)?;
                let receipt = Receipt {
                    format_version: 1,
                    principal_id: work.principal.clone(),
                    operation_sha256: work.operation_sha256.clone(),
                    status: accepted.clone(),
                };
                let transition = |key: String, expected, value: Vec<u8>, action: &str| ControlTransition {
                    key,
                    expected,
                    replacement: Some(value),
                    at: done,
                    actor: format!("session:{}", work.session.as_str()),
                    action: action.into(),
                    request_id: work.context.request_id.as_str().into(),
                    operation_id: work.context.operation_id.as_str().into(),
                };
                let transitions = vec![
                    transition(
                        attunement_job_key(&engine.instance, &before.id),
                        Some(work.before_bytes.clone()),
                        encode(&accepted.job)?,
                        action,
                    ),
                    transition(
                        work.receipt_key.clone(),
                        None,
                        encode(&receipt)?,
                        "attunement.receipted",
                    ),
                ];
                permit.charge(&(&commit, &transitions), admission)?;
                check()?;
                Ok((transitions, accepted))
            },
        )?;
        Ok(AttunementControlResult {
            status: accepted,
            idempotent_replay: false,
        })
    }
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
