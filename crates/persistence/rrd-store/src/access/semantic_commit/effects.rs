//! Encoding each domain mutation's storage effects, including adjacency.

use super::*;

pub(super) fn encode_domain_effect(
    reader: &(impl AccessRead + ?Sized),
    plan: &mut SemanticCommitPlan,
    staged_relations: &mut BTreeMap<RuntimeRef, Option<RuntimeRelation>>,
    commit: &RuntimeCommit,
    mutation: &RuntimeMutation,
    change: &RuntimeChange,
) -> Result<()> {
    match mutation {
        RuntimeMutation::Schema { registry } => {
            plan.put_json(
                keyspaces::RUNTIME_SCHEMAS,
                &keyspaces::runtime_schema_key(&commit.scope),
                registry,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_SCHEMA_VERSIONS,
                &keyspaces::runtime_schema_version_key(&commit.scope, change.cursor),
                change,
            )
        }
        RuntimeMutation::Claim { claim } => plan.put_json(
            keyspaces::RUNTIME_CLAIM_VERSIONS,
            &keyspaces::runtime_claim_version_key(&commit.scope, claim, change.cursor),
            change,
        ),
        RuntimeMutation::Record { record } => {
            plan.put_json(
                keyspaces::RUNTIME_RECORDS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_RECORDS,
                    &commit.scope,
                    &record.reference,
                ),
                record,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_RECORD_VERSIONS,
                &keyspaces::runtime_version_key(
                    keyspaces::RUNTIME_RECORD_VERSIONS,
                    &commit.scope,
                    &record.reference,
                    record.valid_from,
                    change.cursor,
                ),
                change,
            )
        }
        RuntimeMutation::Relation { relation } => {
            if let Some(previous) =
                current_relation(reader, staged_relations, &commit.scope, &relation.reference)?
            {
                delete_current_adjacency(plan, &commit.scope, &previous)?;
            }
            plan.put_json(
                keyspaces::RUNTIME_RELATIONS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_RELATIONS,
                    &commit.scope,
                    &relation.reference,
                ),
                relation,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_RELATION_VERSIONS,
                &keyspaces::runtime_version_key(
                    keyspaces::RUNTIME_RELATION_VERSIONS,
                    &commit.scope,
                    &relation.reference,
                    relation.valid_from,
                    change.cursor,
                ),
                change,
            )?;
            put_adjacency(plan, &commit.scope, relation, relation.valid_from, change)?;
            staged_relations.insert(relation.reference.clone(), Some(relation.clone()));
            Ok(())
        }
        RuntimeMutation::Vector { vector } => {
            super::super::encode_vector_version(plan, &commit.scope, vector, change)
        }
        RuntimeMutation::Event { event } => {
            let reference = keyspaces::runtime_event_reference(event, change.cursor);
            plan.put_json(
                keyspaces::RUNTIME_EVENT_VERSIONS,
                &keyspaces::runtime_version_key(
                    keyspaces::RUNTIME_EVENT_VERSIONS,
                    &commit.scope,
                    &reference,
                    change.at,
                    change.cursor,
                ),
                change,
            )
        }
        RuntimeMutation::SeriesSample { sample } => {
            plan.put_json(
                keyspaces::RUNTIME_SERIES,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_SERIES,
                    &commit.scope,
                    &sample.reference,
                ),
                sample,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_SERIES_VERSIONS,
                &keyspaces::runtime_version_key(
                    keyspaces::RUNTIME_SERIES_VERSIONS,
                    &commit.scope,
                    &sample.reference,
                    sample.observed_at,
                    change.cursor,
                ),
                change,
            )
        }
        RuntimeMutation::Geo { geo } => {
            plan.put_json(
                keyspaces::RUNTIME_GEO,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_GEO,
                    &commit.scope,
                    &geo.reference,
                ),
                geo,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_GEO_VERSIONS,
                &keyspaces::runtime_version_key(
                    keyspaces::RUNTIME_GEO_VERSIONS,
                    &commit.scope,
                    &geo.reference,
                    geo.valid_from,
                    change.cursor,
                ),
                change,
            )
        }
        RuntimeMutation::Object { object } => {
            plan.put_json(
                keyspaces::RUNTIME_OBJECTS,
                &keyspaces::runtime_identity_key(
                    keyspaces::RUNTIME_OBJECTS,
                    &commit.scope,
                    &object.reference,
                ),
                object,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_OBJECT_VERSIONS,
                &keyspaces::runtime_version_key(
                    keyspaces::RUNTIME_OBJECT_VERSIONS,
                    &commit.scope,
                    &object.reference,
                    change.at,
                    change.cursor,
                ),
                change,
            )
        }
        RuntimeMutation::Retire { retirement } => {
            if retirement.model.is_record_like() {
                plan.put_json(
                    keyspaces::RUNTIME_RECORD_VERSIONS,
                    &keyspaces::runtime_version_key(
                        keyspaces::RUNTIME_RECORD_VERSIONS,
                        &commit.scope,
                        &retirement.reference,
                        retirement.effective_at,
                        change.cursor,
                    ),
                    change,
                )?;
                return plan.delete(
                    keyspaces::RUNTIME_RECORDS,
                    &keyspaces::runtime_identity_key(
                        keyspaces::RUNTIME_RECORDS,
                        &commit.scope,
                        &retirement.reference,
                    ),
                );
            }
            let current_space = match retirement.model {
                rrd_core::RuntimeLogicalModel::GraphRelation => {
                    if let Some(relation) = current_relation(
                        reader,
                        staged_relations,
                        &commit.scope,
                        &retirement.reference,
                    )? {
                        plan.put_json(
                            keyspaces::RUNTIME_RELATION_VERSIONS,
                            &keyspaces::runtime_version_key(
                                keyspaces::RUNTIME_RELATION_VERSIONS,
                                &commit.scope,
                                &retirement.reference,
                                retirement.effective_at,
                                change.cursor,
                            ),
                            change,
                        )?;
                        put_temporal_adjacency(
                            plan,
                            &commit.scope,
                            &relation,
                            retirement.effective_at,
                            change,
                        )?;
                        delete_current_adjacency(plan, &commit.scope, &relation)?;
                        staged_relations.insert(retirement.reference.clone(), None);
                    }
                    Some(keyspaces::RUNTIME_RELATIONS)
                }
                rrd_core::RuntimeLogicalModel::Vector => {
                    super::super::encode_vector_retirement(
                        plan,
                        &commit.scope,
                        &retirement.reference,
                        retirement.effective_at,
                        change,
                    )?;
                    None
                }
                rrd_core::RuntimeLogicalModel::Event
                | rrd_core::RuntimeLogicalModel::ReasoningEvent
                | rrd_core::RuntimeLogicalModel::LifecycleEvent => {
                    plan.put_json(
                        keyspaces::RUNTIME_EVENT_VERSIONS,
                        &keyspaces::runtime_version_key(
                            keyspaces::RUNTIME_EVENT_VERSIONS,
                            &commit.scope,
                            &retirement.reference,
                            retirement.effective_at,
                            change.cursor,
                        ),
                        change,
                    )?;
                    None
                }
                rrd_core::RuntimeLogicalModel::TimeSeries => {
                    plan.put_json(
                        keyspaces::RUNTIME_SERIES_VERSIONS,
                        &keyspaces::runtime_version_key(
                            keyspaces::RUNTIME_SERIES_VERSIONS,
                            &commit.scope,
                            &retirement.reference,
                            retirement.effective_at,
                            change.cursor,
                        ),
                        change,
                    )?;
                    Some(keyspaces::RUNTIME_SERIES)
                }
                rrd_core::RuntimeLogicalModel::Geo => {
                    plan.put_json(
                        keyspaces::RUNTIME_GEO_VERSIONS,
                        &keyspaces::runtime_version_key(
                            keyspaces::RUNTIME_GEO_VERSIONS,
                            &commit.scope,
                            &retirement.reference,
                            retirement.effective_at,
                            change.cursor,
                        ),
                        change,
                    )?;
                    Some(keyspaces::RUNTIME_GEO)
                }
                rrd_core::RuntimeLogicalModel::Object => {
                    plan.put_json(
                        keyspaces::RUNTIME_OBJECT_VERSIONS,
                        &keyspaces::runtime_version_key(
                            keyspaces::RUNTIME_OBJECT_VERSIONS,
                            &commit.scope,
                            &retirement.reference,
                            retirement.effective_at,
                            change.cursor,
                        ),
                        change,
                    )?;
                    Some(keyspaces::RUNTIME_OBJECTS)
                }
                rrd_core::RuntimeLogicalModel::ReasoningClaim
                | rrd_core::RuntimeLogicalModel::Document
                | rrd_core::RuntimeLogicalModel::Relational
                | rrd_core::RuntimeLogicalModel::GraphNode
                | rrd_core::RuntimeLogicalModel::KeyValue
                | rrd_core::RuntimeLogicalModel::ReasoningRecord
                | rrd_core::RuntimeLogicalModel::LifecycleRecord => None,
            };
            if let Some(space) = current_space {
                plan.delete(
                    space,
                    &keyspaces::runtime_identity_key(space, &commit.scope, &retirement.reference),
                )?;
            }
            Ok(())
        }
    }
}

pub(super) fn current_relation(
    reader: &(impl AccessRead + ?Sized),
    staged_relations: &mut BTreeMap<RuntimeRef, Option<RuntimeRelation>>,
    scope: &rrd_core::ScopeId,
    reference: &RuntimeRef,
) -> Result<Option<RuntimeRelation>> {
    if let Some(staged) = staged_relations.get(reference) {
        return Ok(staged.clone());
    }
    let current = get_json(
        reader,
        keyspaces::RUNTIME_RELATIONS,
        &keyspaces::runtime_identity_key(keyspaces::RUNTIME_RELATIONS, scope, reference),
    )?;
    staged_relations.insert(reference.clone(), current.clone());
    Ok(current)
}

pub(super) fn put_adjacency(
    plan: &mut SemanticCommitPlan,
    scope: &rrd_core::ScopeId,
    relation: &RuntimeRelation,
    effective_at: u64,
    change: &RuntimeChange,
) -> Result<()> {
    for (current_space, version_space) in [
        (
            keyspaces::RUNTIME_OUTGOING_EDGES,
            keyspaces::RUNTIME_OUTGOING_EDGE_VERSIONS,
        ),
        (
            keyspaces::RUNTIME_INCOMING_EDGES,
            keyspaces::RUNTIME_INCOMING_EDGE_VERSIONS,
        ),
    ] {
        plan.put_json(
            current_space,
            &keyspaces::runtime_adjacency_key(current_space, scope, relation),
            relation,
        )?;
        plan.put_json(
            version_space,
            &keyspaces::runtime_adjacency_version_key(
                version_space,
                scope,
                relation,
                effective_at,
                change.cursor,
            ),
            change,
        )?;
    }
    Ok(())
}

pub(super) fn put_temporal_adjacency(
    plan: &mut SemanticCommitPlan,
    scope: &rrd_core::ScopeId,
    relation: &RuntimeRelation,
    effective_at: u64,
    change: &RuntimeChange,
) -> Result<()> {
    for version_space in [
        keyspaces::RUNTIME_OUTGOING_EDGE_VERSIONS,
        keyspaces::RUNTIME_INCOMING_EDGE_VERSIONS,
    ] {
        plan.put_json(
            version_space,
            &keyspaces::runtime_adjacency_version_key(
                version_space,
                scope,
                relation,
                effective_at,
                change.cursor,
            ),
            change,
        )?;
    }
    Ok(())
}

pub(super) fn delete_current_adjacency(
    plan: &mut SemanticCommitPlan,
    scope: &rrd_core::ScopeId,
    relation: &RuntimeRelation,
) -> Result<()> {
    for space in [
        keyspaces::RUNTIME_OUTGOING_EDGES,
        keyspaces::RUNTIME_INCOMING_EDGES,
    ] {
        plan.delete(
            space,
            &keyspaces::runtime_adjacency_key(space, scope, relation),
        )?;
    }
    Ok(())
}
