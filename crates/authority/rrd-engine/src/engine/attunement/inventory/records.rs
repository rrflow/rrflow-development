//! One canonical source representation. Snapshot membership/header, entry facts
//! and changes live in separate runtime records; the control head only binds
//! their authenticated commit identity, never a second copy of their contents.
use super::*;
use rrd_store::{RuntimeReadBudget, RuntimeVersionedSource};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const SNAPSHOT: &str = "source-tree-snapshot";
pub(super) const ENTRY: &str = "source-tree-entry";
pub(super) const CHANGES: &str = "source-tree-change-set";
const CONTAINS: &str = "source-tree-contains";

#[derive(Serialize, Deserialize)]
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
) -> Result<(Option<Vec<u8>>, Option<Proposal>)> {
    let Some(bytes) = engine.storage.control().get(&head_key(&engine.instance))? else {
        return Ok((None, None));
    };
    let head: Head = decode(&bytes)?;
    let scope = ScopeId::new(format!("instance:{}", engine.instance)).map_err(contract)?;
    if head.format_version != 1
        || head.project_id != plan.target.project_id
        || head.read.scope != scope
        || !is_digest(&head.proposal_sha256)
    {
        return Err(invalid());
    }
    let mut keys = engine.estate_configuration.query.max_storage_keys;
    let reference = RuntimeRef::new(SNAPSHOT, &head.proposal_sha256).map_err(contract)?;
    let mut records = read_records(
        engine,
        &head.read,
        std::slice::from_ref(&reference),
        &mut keys,
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
    let rows = read_records(engine, &head.read, &references, &mut keys)?;
    let entries = references[..members.len()]
        .iter()
        .map(|reference| {
            let record = rows.get(reference).ok_or_else(invalid)?;
            let value = properties_json(&record.properties)?;
            if value.get("identity").and_then(serde_json::Value::as_str)
                != Some(reference.id.as_str())
            {
                return Err(invalid());
            }
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
    Ok((Some(bytes), Some(proposal)))
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
        mutations.push(record(ENTRY, &entry.identity, entry, at)?);
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
    if parts.groups.is_empty() || parts.complete.is_empty() {
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

fn register(registry: &mut RuntimeSchemaRegistry) -> Result<bool> {
    use RuntimeValueType::{Bool, List, Map, String as Text, Unsigned};
    let mut changed = false;
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
            vec![("parent", Text)],
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
