//! One canonical source representation. Snapshot membership/header, entry facts
//! and changes live in separate runtime records; the control head only binds
//! their authenticated commit identity, never a second copy of their contents.
use super::*;
use rrd_attunement::inventory::EntryKind;
use rrd_store::{RuntimeReadBudget, RuntimeVersionedSource};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const SNAPSHOT: &str = "source-tree-snapshot";
pub(super) const ENTRY: &str = "source-tree-entry";
pub(super) const CHANGES: &str = "source-tree-change-set";
const CONTAINS: &str = "source-tree-contains";

#[cfg(test)]
thread_local! {
    static OMIT_SOURCE_FILE_PROJECTION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) struct LegacySourceFileProjectionGuard {
    previous: bool,
}

#[cfg(test)]
impl LegacySourceFileProjectionGuard {
    pub(crate) fn omit_for_current_thread() -> Self {
        let previous = OMIT_SOURCE_FILE_PROJECTION.with(|enabled| enabled.replace(true));
        Self { previous }
    }
}

#[cfg(test)]
impl Drop for LegacySourceFileProjectionGuard {
    fn drop(&mut self) {
        OMIT_SOURCE_FILE_PROJECTION.with(|enabled| enabled.set(self.previous));
    }
}

fn source_file_projection_enabled() -> bool {
    #[cfg(test)]
    if OMIT_SOURCE_FILE_PROJECTION.with(std::cell::Cell::get) {
        return false;
    }
    true
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Head {
    pub format_version: u32,
    pub project_id: CanonicalId,
    pub proposal_sha256: String,
    pub read: ReadStamp,
}

pub(super) fn head_key(instance: &CanonicalId) -> String {
    format!("server/state/{instance}/inventory/head")
}

pub(super) fn load_prior(
    engine: &RrdEngine,
    plan: &AttunementPlan,
    max_bytes: u64,
    max_entries: u64,
    remaining_keys: &mut u64,
) -> Result<(Option<Vec<u8>>, Option<Proposal>)> {
    let loaded = load_accepted(
        engine,
        &plan.target.project_id,
        max_bytes,
        max_entries,
        remaining_keys,
    )?;
    Ok(match loaded {
        Some((bytes, _, proposal)) => (Some(bytes), Some(proposal)),
        None => (None, None),
    })
}

pub(super) fn load_accepted(
    engine: &RrdEngine,
    project_id: &CanonicalId,
    max_bytes: u64,
    max_entries: u64,
    remaining_keys: &mut u64,
) -> Result<Option<(Vec<u8>, Head, Proposal)>> {
    let Some(bytes) = engine.storage.control().get(&head_key(&engine.instance))? else {
        return Ok(None);
    };
    let head: Head = decode(&bytes)?;
    let scope = ScopeId::new(format!("instance:{}", engine.instance)).map_err(contract)?;
    if head.format_version != 1
        || &head.project_id != project_id
        || head.read.scope != scope
        || !is_digest(&head.proposal_sha256)
    {
        return Err(invalid());
    }
    let reference = RuntimeRef::new(SNAPSHOT, &head.proposal_sha256).map_err(contract)?;
    let mut records = read_records(
        engine,
        &head.read,
        std::slice::from_ref(&reference),
        remaining_keys,
    )?;
    let snapshot = records.remove(&reference).ok_or_else(invalid)?;
    let mut header = properties_json(&snapshot.properties)?;
    let membership = header
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("entry_ids")
        .ok_or_else(invalid)?;
    header
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("source_precondition_sha256")
        .ok_or_else(invalid)?;
    let members = membership.as_array().ok_or_else(invalid)?;
    if members.is_empty() || members.len() as u64 > max_entries {
        return Err(invalid());
    }
    let mut references = Vec::with_capacity(members.len() + 1);
    let mut unique = BTreeSet::new();
    for member in members {
        let identity = member
            .as_str()
            .filter(|s| is_digest(s))
            .ok_or_else(invalid)?;
        if !unique.insert(identity) {
            return Err(invalid());
        }
        references.push(RuntimeRef::new(ENTRY, identity).map_err(contract)?);
    }
    let changes_ref = RuntimeRef::new(CHANGES, &head.proposal_sha256).map_err(contract)?;
    references.push(changes_ref.clone());
    let rows = read_records(engine, &head.read, &references, remaining_keys)?;
    let entries = references[..members.len()]
        .iter()
        .map(|reference| {
            let record = rows.get(reference).ok_or_else(invalid)?;
            let mut value = properties_json(&record.properties)?;
            if value.get("identity").and_then(serde_json::Value::as_str)
                != Some(reference.id.as_str())
            {
                return Err(invalid());
            }
            strip_verified_source_file_projection(&mut value)?;
            Ok(value)
        })
        .collect::<Result<Vec<_>>>()?;
    let changes = properties_json(&rows.get(&changes_ref).ok_or_else(invalid)?.properties)?;
    let object = header.as_object_mut().ok_or_else(invalid)?;
    object.insert("entries".into(), entries.into());
    object.insert("changes".into(), changes);
    super::bounded(&header, max_bytes)?;
    let serialized = serde_json::to_vec(&header).map_err(contract)?;
    let proposal =
        Proposal::from_committed_bytes(&serialized, &head.proposal_sha256, max_bytes as usize)
            .map_err(contract)?;
    Ok(Some((bytes, head, proposal)))
}

fn read_records(
    engine: &RrdEngine,
    read: &ReadStamp,
    references: &[RuntimeRef],
    remaining: &mut u64,
) -> Result<BTreeMap<RuntimeRef, RuntimeRecord>> {
    let sources: Vec<_> = references
        .iter()
        .map(|reference| RuntimeVersionedSource::Identity {
            model: RuntimeLogicalModel::GraphNode,
            reference: reference.clone(),
        })
        .collect();
    let direct = engine.storage.runtime().read_versioned(
        read,
        &sources,
        RuntimeReadBudget::new(*remaining)?,
    )?;
    *remaining = remaining
        .checked_sub(direct.evidence.keys_examined)
        .ok_or_else(invalid)?;
    let admitted: BTreeSet<_> = references.iter().collect();
    let mut rows = BTreeMap::<RuntimeRef, (u64, RuntimeRecord)>::new();
    for change in direct.changes {
        if let RuntimeMutation::Record { record } = change.mutation {
            if !admitted.contains(&record.reference) {
                return Err(invalid());
            }
            let row = rows
                .entry(record.reference.clone())
                .or_insert_with(|| (0, record.clone()));
            if change.cursor >= row.0 {
                *row = (change.cursor, record);
            }
        }
    }
    Ok(rows
        .into_iter()
        .map(|(key, (_, value))| (key, value))
        .collect())
}

pub(super) fn mutations(
    engine: &RrdEngine,
    scope: &ScopeId,
    read: &ReadStamp,
    proposal: &Proposal,
    precondition: &str,
    at: u64,
) -> Result<Vec<RuntimeMutation>> {
    let current = engine.storage.runtime().schema(scope)?;
    if current.as_ref().map(|s| s.revision) != read.schema_revision {
        return Err(ServiceError::StorageConflict(
            "Inventory schema changed".into(),
        ));
    }
    let mut registry = current
        .clone()
        .unwrap_or_else(|| RuntimeSchemaRegistry::empty(1, "install source inventory tables"));
    let changed = register(&mut registry)?;
    let mut mutations = Vec::new();
    if changed {
        if let Some(old) = current {
            registry.revision = old.revision.checked_add(1).ok_or_else(invalid)?;
            registry.migration = "install source inventory tables".into();
        }
        mutations.push(RuntimeMutation::Schema { registry });
    }
    let value = serde_json::to_value(proposal).map_err(contract)?;
    let mut header = value.as_object().cloned().ok_or_else(invalid)?;
    header.remove("entries");
    header.remove("changes");
    header.insert(
        "entry_ids".into(),
        proposal
            .entries()
            .iter()
            .map(|e| serde_json::Value::String(e.identity.clone()))
            .collect(),
    );
    header.insert("source_precondition_sha256".into(), precondition.into());
    mutations.push(record(SNAPSHOT, proposal.proposal_sha256(), &header, at)?);
    mutations.push(record(
        CHANGES,
        proposal.proposal_sha256(),
        proposal.changes(),
        at,
    )?);
    for entry in proposal.entries() {
        let mut entry_record = record(ENTRY, &entry.identity, entry, at)?;
        if source_file_projection_enabled() {
            if let EntryKind::File {
                observation,
                content_sha256,
            } = &entry.kind
            {
                let RuntimeMutation::Record { record } = &mut entry_record else {
                    return Err(invalid());
                };
                record.properties.insert(
                    "source_file".into(),
                    RuntimeValue::Map(BTreeMap::from([
                        (
                            "content_sha256".into(),
                            RuntimeValue::String(content_sha256.clone()),
                        ),
                        ("bytes".into(), RuntimeValue::Unsigned(observation.bytes)),
                    ])),
                );
            }
        }
        mutations.push(entry_record);
        let (from, edge) = match &entry.parent {
            Some(parent) => (
                RuntimeRef::new(ENTRY, parent).map_err(contract)?,
                format!("entry-{}", entry.identity),
            ),
            None => (
                RuntimeRef::new(SNAPSHOT, proposal.proposal_sha256()).map_err(contract)?,
                format!("snapshot-{}", proposal.proposal_sha256()),
            ),
        };
        mutations.push(RuntimeMutation::Relation {
            relation: RuntimeRelation {
                reference: RuntimeRef::new(CONTAINS, edge).map_err(contract)?,
                from,
                to: RuntimeRef::new(ENTRY, &entry.identity).map_err(contract)?,
                valid_from: at,
                valid_to: None,
                properties: BTreeMap::new(),
            },
        });
    }
    for removed in &proposal.changes().removed {
        for (kind, id, model) in [
            (
                ENTRY,
                removed.identity.clone(),
                RuntimeLogicalModel::GraphNode,
            ),
            (
                CONTAINS,
                format!("entry-{}", removed.identity),
                RuntimeLogicalModel::GraphRelation,
            ),
        ] {
            mutations.push(RuntimeMutation::Retire {
                retirement: rrd_core::RuntimeRetirement {
                    model,
                    reference: RuntimeRef::new(kind, id).map_err(contract)?,
                    effective_at: at,
                },
            });
        }
    }
    Ok(mutations)
}

pub(super) struct PublicationParts {
    pub schema: Vec<RuntimeMutation>,
    pub groups: Vec<Vec<RuntimeMutation>>,
    pub complete: Vec<RuntimeMutation>,
}

/// Each entry and its parent edge form one source-observation group. The root
/// snapshot edge, complete manifests and retirements stay in final publication;
/// intermediate groups cannot retire a previously published source view.
pub(super) fn partition(mutations: Vec<RuntimeMutation>) -> Result<PublicationParts> {
    let mut parts = PublicationParts {
        schema: Vec::new(),
        groups: Vec::new(),
        complete: Vec::new(),
    };
    for mutation in mutations {
        match &mutation {
            RuntimeMutation::Schema { .. } => parts.schema.push(mutation),
            RuntimeMutation::Record { record } if record.reference.kind.as_str() == ENTRY => {
                parts.groups.push(vec![mutation])
            }
            RuntimeMutation::Relation { relation }
                if relation.reference.kind.as_str() == CONTAINS
                    && relation.from.kind.as_str() == ENTRY =>
            {
                let group = parts.groups.last_mut().ok_or_else(invalid)?;
                if !matches!(group.first(), Some(RuntimeMutation::Record { record }) if record.reference == relation.to)
                {
                    return Err(invalid());
                }
                group.push(mutation);
            }
            RuntimeMutation::Record { record }
                if [SNAPSHOT, CHANGES].contains(&record.reference.kind.as_str()) =>
            {
                parts.complete.push(mutation)
            }
            RuntimeMutation::Relation { relation }
                if relation.reference.kind.as_str() == CONTAINS
                    && relation.from.kind.as_str() == SNAPSHOT =>
            {
                parts.complete.push(mutation)
            }
            RuntimeMutation::Retire { .. } => parts.complete.push(mutation),
            _ => return Err(invalid()),
        }
    }
    if parts.complete.is_empty() {
        return Err(invalid());
    }
    Ok(parts)
}

fn record(kind: &str, id: &str, value: &impl Serialize, at: u64) -> Result<RuntimeMutation> {
    let value = serde_json::to_value(value).map_err(contract)?;
    let object = value.as_object().ok_or_else(invalid)?;
    let properties = object
        .iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(key, value)| Ok((key.clone(), to_runtime(value)?)))
        .collect::<Result<_>>()?;
    Ok(RuntimeMutation::Record {
        record: RuntimeRecord {
            reference: RuntimeRef::new(kind, id).map_err(contract)?,
            valid_from: at,
            valid_to: None,
            properties,
        },
    })
}

fn to_runtime(value: &serde_json::Value) -> Result<RuntimeValue> {
    Ok(match value {
        serde_json::Value::Null => RuntimeValue::Null,
        serde_json::Value::Bool(v) => RuntimeValue::Bool(*v),
        serde_json::Value::Number(v) => RuntimeValue::Unsigned(v.as_u64().ok_or_else(invalid)?),
        serde_json::Value::String(v) => RuntimeValue::String(v.clone()),
        serde_json::Value::Array(v) => {
            RuntimeValue::List(v.iter().map(to_runtime).collect::<Result<_>>()?)
        }
        serde_json::Value::Object(v) => RuntimeValue::Map(
            v.iter()
                .map(|(k, v)| Ok((k.clone(), to_runtime(v)?)))
                .collect::<Result<_>>()?,
        ),
    })
}

fn from_runtime(value: &RuntimeValue) -> Result<serde_json::Value> {
    Ok(match value {
        RuntimeValue::Null => serde_json::Value::Null,
        RuntimeValue::Bool(v) => (*v).into(),
        RuntimeValue::Unsigned(v) => (*v).into(),
        RuntimeValue::String(v) | RuntimeValue::Digest(v) => v.clone().into(),
        RuntimeValue::List(v) => v
            .iter()
            .map(from_runtime)
            .collect::<Result<Vec<_>>>()?
            .into(),
        RuntimeValue::Map(v) => v
            .iter()
            .map(|(k, v)| Ok((k.clone(), from_runtime(v)?)))
            .collect::<Result<serde_json::Map<_, _>>>()?
            .into(),
        _ => return Err(invalid()),
    })
}

fn properties_json(properties: &RuntimeProperties) -> Result<serde_json::Value> {
    from_runtime(&RuntimeValue::Map(properties.clone()))
}

/// Remove the collision-safe public projection only after confirming it is an
/// exact mechanical view of the unchanged canonical entry kind. Old records
/// without the optional projection remain reconstructible but cannot supply it
/// to a picker until an eligible inventory publication writes it.
fn strip_verified_source_file_projection(value: &mut serde_json::Value) -> Result<()> {
    let object = value.as_object_mut().ok_or_else(invalid)?;
    let projection = object.remove("source_file");
    let entry_kind: EntryKind =
        serde_json::from_value(object.get("kind").cloned().ok_or_else(invalid)?)
            .map_err(contract_json)?;
    match (entry_kind, projection) {
        (
            EntryKind::File {
                observation,
                content_sha256,
            },
            Some(projection),
        ) => {
            let expected = serde_json::json!({
                "content_sha256": content_sha256,
                "bytes": observation.bytes,
            });
            if projection != expected {
                return Err(invalid());
            }
        }
        (EntryKind::File { .. }, None) | (_, None) => {}
        (_, Some(_)) => return Err(invalid()),
    }
    Ok(())
}

fn register(registry: &mut RuntimeSchemaRegistry) -> Result<bool> {
    use RuntimeValueType::{Bool, List, Map, String as Text, Unsigned};
    let mut changed = false;
    let entry_optional = if source_file_projection_enabled() {
        vec![("parent", Text), ("source_file", Map)]
    } else {
        vec![("parent", Text)]
    };
    for (kind, fields, optional) in [
        (
            SNAPSHOT,
            vec![
                ("format_version", Unsigned),
                ("scope", Map),
                ("revisions", Map),
                ("limits", Map),
                ("complete", Bool),
                ("root_sha256", Text),
                ("accounting", Map),
                ("proposal_sha256", Text),
                ("entry_ids", List),
                ("source_precondition_sha256", Text),
            ],
            vec![("previous_sha256", Text)],
        ),
        (
            ENTRY,
            vec![
                ("path", Text),
                ("identity", Text),
                ("kind", Map),
                ("subtree_sha256", Text),
            ],
            entry_optional,
        ),
        (
            CHANGES,
            vec![
                ("added", List),
                ("modified", List),
                ("removed", List),
                ("rename_candidates", List),
            ],
            vec![],
        ),
    ] {
        let mut properties: BTreeMap<_, _> = fields
            .into_iter()
            .map(|(name, kind)| (name.into(), RuntimePropertySchema::required(kind)))
            .collect();
        properties.extend(
            optional
                .into_iter()
                .map(|(name, kind)| (name.into(), RuntimePropertySchema::optional(kind))),
        );
        changed |= registry
            .define_record_table(
                RuntimeType::new(kind).map_err(contract)?,
                RuntimeLogicalModel::GraphNode,
                RuntimeRecordSchema {
                    properties,
                    allow_additional_properties: false,
                    unique_properties: BTreeSet::new(),
                },
            )
            .map_err(contract)?;
    }
    changed |= registry
        .define_relation_table(
            RuntimeType::new(CONTAINS).map_err(contract)?,
            RuntimeRelationSchema {
                from: [SNAPSHOT, ENTRY]
                    .into_iter()
                    .map(|s| RuntimeType::new(s).map_err(contract))
                    .collect::<Result<_>>()?,
                to: BTreeSet::from([RuntimeType::new(ENTRY).map_err(contract)?]),
                properties: BTreeMap::new(),
                allow_additional_properties: false,
                unique_pair: false,
                max_outgoing: None,
                max_incoming: None,
            },
        )
        .map_err(contract)?;
    Ok(changed)
}

fn invalid() -> ServiceError {
    ServiceError::Storage("invalid committed source inventory".into())
}

pub(super) fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod source_file_projection_tests {
    use super::strip_verified_source_file_projection;
    use rrd_attunement::inventory::{Entry, EntryKind, FileObservation};

    fn file_entry() -> serde_json::Value {
        serde_json::json!({
            "identity": "1".repeat(64),
            "path": "alpha.txt",
            "kind": {
                "kind": "file",
                "observation": {
                    "bytes": 5,
                    "executable": false,
                    "revision": "fixture-revision",
                    "classification_sha256": "a".repeat(64)
                },
                "content_sha256": "b".repeat(64)
            },
            "subtree_sha256": "c".repeat(64)
        })
    }

    #[test]
    fn optional_source_file_is_removed_only_when_it_matches_canonical_file_kind() {
        let canonical = Entry {
            path: "alpha.txt".into(),
            identity: "1".repeat(64),
            parent: None,
            kind: EntryKind::File {
                observation: FileObservation {
                    bytes: 5,
                    executable: false,
                    revision: "fixture-revision".into(),
                    classification_sha256: "a".repeat(64),
                },
                content_sha256: "b".repeat(64),
            },
            subtree_sha256: "c".repeat(64),
        };
        let canonical_json = serde_json::to_value(&canonical).unwrap();
        let mut old = canonical_json.clone();
        strip_verified_source_file_projection(&mut old).unwrap();
        assert!(old.get("source_file").is_none());
        assert_eq!(old, canonical_json);

        let mut projected = canonical_json.clone();
        projected["source_file"] = serde_json::json!({
            "content_sha256": "b".repeat(64),
            "bytes": 5
        });
        strip_verified_source_file_projection(&mut projected).unwrap();
        assert!(projected.get("source_file").is_none());
        assert_eq!(projected, canonical_json);
    }

    #[test]
    fn inconsistent_or_non_file_source_projection_is_rejected() {
        for projection in [
            serde_json::json!({ "content_sha256": "d".repeat(64), "bytes": 5 }),
            serde_json::json!({ "content_sha256": "b".repeat(64), "bytes": 6 }),
            serde_json::json!({ "content_sha256": "b".repeat(64), "bytes": 5, "extra": true }),
        ] {
            let mut value = file_entry();
            value["source_file"] = projection;
            assert!(strip_verified_source_file_projection(&mut value).is_err());
        }

        let mut directory = file_entry();
        directory["kind"] = serde_json::json!({ "kind": "directory" });
        directory["source_file"] = serde_json::json!({
            "content_sha256": "b".repeat(64),
            "bytes": 5
        });
        assert!(strip_verified_source_file_projection(&mut directory).is_err());

        let mut old_directory = file_entry();
        old_directory["kind"] = serde_json::json!({ "kind": "directory" });
        strip_verified_source_file_projection(&mut old_directory).unwrap();
    }
}
