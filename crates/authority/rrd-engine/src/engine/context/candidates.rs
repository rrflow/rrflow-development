//! Plan stages, graph expansion and candidate identity and text.

use super::*;

pub(super) fn context_plan_stage(
    kind: ContextPlanStageKind,
    status: ContextPlanStageStatus,
    access_path: Option<ContextAccessPath>,
    exact: bool,
    reason: String,
) -> Result<ContextPlanStage> {
    let mut stage = ContextPlanStage {
        kind,
        status,
        access_path,
        exact,
        reason,
        decision_sha256: String::new(),
    };
    stage.decision_sha256 = context_plan_stage_sha256(&stage)
        .map_err(|error| ServiceError::Contract(error.to_string()))?;
    Ok(stage)
}

pub(super) fn add_graph_context(
    request: &AssembleContext,
    read: &ReadStamp,
    snapshot: &RuntimeDataSnapshot,
    records: &BTreeMap<RuntimeRef, RuntimeRecord>,
    roots: &[RuntimeRef],
    candidates: &mut BTreeMap<String, ContextCandidate>,
) -> Result<bool> {
    let mut adjacency = BTreeMap::<RuntimeRef, Vec<GraphEdge>>::new();
    for relation in &snapshot.relations {
        adjacency
            .entry(relation.value.from.clone())
            .or_default()
            .push(GraphEdge {
                relation: relation.value.reference.clone(),
                neighbor: relation.value.to.clone(),
            });
        adjacency
            .entry(relation.value.to.clone())
            .or_default()
            .push(GraphEdge {
                relation: relation.value.reference.clone(),
                neighbor: relation.value.from.clone(),
            });
    }
    for edges in adjacency.values_mut() {
        edges.sort_by(|left, right| {
            left.relation
                .cmp(&right.relation)
                .then_with(|| left.neighbor.cmp(&right.neighbor))
        });
        edges.dedup_by(|left, right| {
            left.relation == right.relation && left.neighbor == right.neighbor
        });
    }

    let graph_plan = context_digest(&(
        "rrd-context-graph-bfs-v1",
        read.manifest_id.as_str(),
        request.max_graph_depth,
        "both",
    ))?;
    let max_steps = request.max_items.saturating_mul(16).max(1);
    let mut steps = 0_u64;
    let mut evidence_rank = 0_u64;
    let mut truncated = false;
    for root in roots {
        let mut visited = BTreeSet::from([root.clone()]);
        let mut queue = VecDeque::from([(root.clone(), 0_u8)]);
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= request.max_graph_depth {
                continue;
            }
            if steps >= max_steps {
                truncated = adjacency
                    .get(&current)
                    .is_some_and(|edges| !edges.is_empty());
                if truncated {
                    break;
                }
                continue;
            }
            for edge in adjacency.get(&current).into_iter().flatten() {
                if steps >= max_steps {
                    truncated = true;
                    break;
                }
                steps += 1;
                if !visited.insert(edge.neighbor.clone()) {
                    continue;
                }
                let next_depth = depth + 1;
                queue.push_back((edge.neighbor.clone(), next_depth));
                let Some(record) = records.get(&edge.neighbor) else {
                    continue;
                };
                evidence_rank = evidence_rank.checked_add(1).ok_or_else(|| {
                    ServiceError::Contract("context graph rank overflowed u64".into())
                })?;
                let evidence = ContextEvidence {
                    kind: ContextEvidenceKind::Graph,
                    source: rrd_core::result_identity::typed(
                        rrd_core::result_identity::ResultIdentityFamily::Relation,
                        &edge.relation,
                    ),
                    source_rank: evidence_rank,
                    source_score: 1.0 / f64::from(next_depth),
                    contribution: reciprocal_rank(GRAPH_WEIGHT, evidence_rank),
                    plan_sha256: graph_plan.clone(),
                    evidence_sha256: context_digest(&(
                        "rrd-context-graph-evidence-v1",
                        read.manifest_id.as_str(),
                        &root,
                        &current,
                        &edge.neighbor,
                        &edge.relation,
                        next_depth,
                    ))?,
                };
                add_record_candidate(candidates, &edge.neighbor, record, evidence)?;
            }
            if truncated {
                break;
            }
        }
        if truncated {
            break;
        }
    }
    Ok(truncated)
}

pub(super) fn add_record_candidate(
    candidates: &mut BTreeMap<String, ContextCandidate>,
    reference: &RuntimeRef,
    record: &RuntimeRecord,
    evidence: ContextEvidence,
) -> Result<()> {
    add_candidate(
        candidates,
        record_identity(reference),
        public_properties(&record.properties)?,
        evidence,
    );
    Ok(())
}

pub(super) fn add_candidate(
    candidates: &mut BTreeMap<String, ContextCandidate>,
    identity: String,
    values: BTreeMap<String, QueryValue>,
    evidence: ContextEvidence,
) {
    let candidate = candidates
        .entry(identity)
        .or_insert_with(|| ContextCandidate {
            values: BTreeMap::new(),
            evidence: Vec::new(),
        });
    if candidate.values.is_empty() {
        candidate.values = values;
    }
    if !candidate
        .evidence
        .iter()
        .any(|existing| existing.evidence_sha256 == evidence.evidence_sha256)
    {
        candidate.evidence.push(evidence);
    }
}

pub(super) fn record_identity(reference: &RuntimeRef) -> String {
    rrd_core::result_identity::typed(
        rrd_core::result_identity::ResultIdentityFamily::Record,
        reference,
    )
}

pub(super) fn claim_identity(claim: &Claim) -> String {
    rrd_core::result_identity::claim(&claim.subject, &claim.predicate)
}

pub(super) fn claim_text(claim: &Claim) -> String {
    let mut output = String::new();
    for value in [
        claim.subject.as_str(),
        claim.predicate.as_str(),
        claim.object.as_str(),
        claim.producer.actor.as_str(),
    ] {
        append_text(&mut output, value);
    }
    if let Some(value) = &claim.producer.on_behalf_of {
        append_text(&mut output, value);
    }
    output
}

pub(super) fn claim_values(claim: &Claim) -> BTreeMap<String, QueryValue> {
    BTreeMap::from([
        ("kind".into(), QueryValue::String("reasoning_claim".into())),
        (
            "subject".into(),
            QueryValue::String(claim.subject.as_str().into()),
        ),
        (
            "predicate".into(),
            QueryValue::String(claim.predicate.as_str().into()),
        ),
        ("object".into(), QueryValue::String(claim.object.clone())),
        ("valid_from".into(), QueryValue::Unsigned(claim.valid_from)),
        (
            "valid_to".into(),
            claim
                .valid_to
                .map(QueryValue::Unsigned)
                .unwrap_or(QueryValue::Null),
        ),
        ("tx_time".into(), QueryValue::Unsigned(claim.tx_time)),
        (
            "actor".into(),
            QueryValue::String(claim.producer.actor.clone()),
        ),
        (
            "confidence".into(),
            claim
                .confidence
                .map(|value| QueryValue::Decimal(value.to_string()))
                .unwrap_or(QueryValue::Null),
        ),
    ])
}
