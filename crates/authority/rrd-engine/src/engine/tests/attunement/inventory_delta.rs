use super::*;
use rrd_core::{ReadStamp, RuntimeCommit, RuntimeLogicalModel, RuntimeType};
use rrd_store::{RuntimeReadBudget, RuntimeVersionedSource};

fn first_inventory() -> (
    Installed,
    RrdEngine,
    SessionLease,
    AttunementJob,
    ScopeId,
    ReadStamp,
) {
    let proposal = crate::AttunementProposal {
        format_version: crate::ATTUNEMENT_PROPOSAL_FORMAT_VERSION,
        steps: [
            Phase::Connect,
            Phase::Inventory,
            Phase::Inventory,
            Phase::Inventory,
            Phase::Verify,
        ]
        .into_iter()
        .map(|phase| crate::AttunementProposalStep {
            phase,
            description: format!("{phase:?}"),
            estimated_items: 0,
            estimated_input_bytes: 0,
        })
        .collect(),
        estimated_min_duration_ms: 0,
        estimated_max_duration_ms: 30_000,
    };
    let (installed, engine, session) = Installed::with_source(
        InstallationTargetKind::ExistingProject,
        Some(&proposal),
        |root| {
            for name in ["alpha.txt", "beta.txt", "gamma.txt"] {
                std::fs::write(root.join(name), format!("source {name}\n")).unwrap();
            }
        },
    );
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "delta-start",
        BASE + 1_000,
    )
    .unwrap();
    let connected = call(
        &engine,
        &session,
        &step(&started.status.job),
        "delta-connect",
        started.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let first = call(
        &engine,
        &session,
        &step(&connected.status.job),
        "delta-first",
        connected.status.job.updated_at_unix_ms + 1,
    )
    .unwrap();
    let scope = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
    let read = engine.storage.runtime().read_stamp(&scope).unwrap();
    (installed, engine, session, first.status.job, scope, read)
}

#[test]
fn unchanged_inventory_publishes_only_completion_metadata() {
    let (installed, engine, session, mut job, scope, mut previous_read) = first_inventory();
    for key in ["delta-second", "delta-third"] {
        let next = call(
            &engine,
            &session,
            &step(&job),
            key,
            job.updated_at_unix_ms + 1,
        )
        .unwrap();
        let read = engine.storage.runtime().read_stamp(&scope).unwrap();
        assert_eq!(
            read.commit_cursor - previous_read.commit_cursor,
            3,
            "unchanged entries and containment must survive without new versions; only the snapshot, change set and root link are new"
        );
        assert_eq!(next.status.job.checkpoints.len(), job.checkpoints.len() + 1);
        assert_eq!(
            next.status
                .job
                .checkpoints
                .last()
                .unwrap()
                .coordinates
                .runtime_cursor,
            read.commit_cursor
        );
        previous_read = read;
        job = next.status.job;
    }
    assert_eq!(job.current_phase, Some(Phase::Verify));
    let reopen_at = job.updated_at_unix_ms + 1;
    drop(engine);
    let engine = installed.reopen(reopen_at);
    let session = installed.session(&engine, "delta-reopened", reopen_at + 1);
    assert_eq!(installed.read(&engine, &session, reopen_at + 2).job, job);
    assert_eq!(
        engine.storage.runtime().read_stamp(&scope).unwrap(),
        previous_read
    );
}

#[test]
fn inventory_reuse_observes_current_records_edges_and_retirements() {
    for changed_kind in ["record", "relation", "retirement"] {
        let (installed, engine, session, job, scope, before) = first_inventory();
        let facts = engine
            .storage
            .runtime()
            .read_versioned(
                &before,
                &[
                    RuntimeVersionedSource::Records {
                        kind: Some(RuntimeType::new("source-tree-entry").unwrap()),
                    },
                    RuntimeVersionedSource::Relations {
                        kind: Some(RuntimeType::new("source-tree-contains").unwrap()),
                    },
                ],
                RuntimeReadBudget::new(10_000).unwrap(),
            )
            .unwrap();
        let original = facts
            .changes
            .iter()
            .find_map(|change| match &change.mutation {
                RuntimeMutation::Record { record }
                    if record.properties.get("path")
                        == Some(&RuntimeValue::String("alpha.txt".into())) =>
                {
                    Some(record.clone())
                }
                _ => None,
            })
            .unwrap();
        let original_edge = facts
            .changes
            .iter()
            .find_map(|change| match &change.mutation {
                RuntimeMutation::Relation { relation } if relation.to == original.reference => {
                    Some(relation.clone())
                }
                _ => None,
            })
            .unwrap();
        let at = job.updated_at_unix_ms + 1;
        let mutation = match changed_kind {
            "record" => {
                let mut record = original.clone();
                record.valid_from = at;
                record.properties.insert(
                    "subtree_sha256".into(),
                    RuntimeValue::String("0".repeat(64)),
                );
                RuntimeMutation::Record { record }
            }
            "relation" => {
                let mut relation = original_edge.clone();
                relation.valid_from = at;
                relation.from = original.reference.clone();
                RuntimeMutation::Relation { relation }
            }
            "retirement" => RuntimeMutation::Retire {
                retirement: rrd_core::RuntimeRetirement {
                    model: RuntimeLogicalModel::GraphNode,
                    reference: original.reference.clone(),
                    effective_at: at,
                },
            },
            _ => unreachable!(),
        };
        engine
            .storage
            .runtime()
            .commit(&RuntimeCommit {
                scope: scope.clone(),
                at,
                actor: "test-fixture".into(),
                expected_cursor: before.commit_cursor,
                mutations: vec![mutation],
            })
            .unwrap();
        let intervening = engine.storage.runtime().read_stamp(&scope).unwrap();
        let repaired = call(&engine, &session, &step(&job), "delta-repair", at + 1).unwrap();
        let after = engine.storage.runtime().read_stamp(&scope).unwrap();
        assert_eq!(
            after.commit_cursor - intervening.commit_cursor,
            5,
            "{changed_kind}: republish the changed entry/edge group and completion metadata"
        );
        let current = engine
            .storage
            .runtime()
            .read_versioned(
                &after,
                &[
                    RuntimeVersionedSource::Identity {
                        model: RuntimeLogicalModel::GraphNode,
                        reference: original.reference.clone(),
                    },
                    RuntimeVersionedSource::Identity {
                        model: RuntimeLogicalModel::GraphRelation,
                        reference: original_edge.reference.clone(),
                    },
                ],
                RuntimeReadBudget::new(10_000).unwrap(),
            )
            .unwrap();
        let latest_record = current
            .changes
            .iter()
            .filter_map(|change| match &change.mutation {
                RuntimeMutation::Record { record } => Some((change.cursor, record)),
                _ => None,
            })
            .max_by_key(|(cursor, _)| *cursor)
            .unwrap()
            .1;
        assert_eq!(latest_record.properties, original.properties);
        let latest_edge = current
            .changes
            .iter()
            .filter_map(|change| match &change.mutation {
                RuntimeMutation::Relation { relation } => Some((change.cursor, relation)),
                _ => None,
            })
            .max_by_key(|(cursor, _)| *cursor)
            .unwrap()
            .1;
        assert_eq!(latest_edge.from, original_edge.from);
        assert_eq!(latest_edge.to, original_edge.to);
        // Reconstruct the new complete snapshot before another unchanged run.
        let repeated = call(
            &engine,
            &session,
            &step(&repaired.status.job),
            "delta-repaired-again",
            repaired.status.job.updated_at_unix_ms + 1,
        )
        .unwrap();
        assert_eq!(
            engine
                .storage
                .runtime()
                .read_stamp(&scope)
                .unwrap()
                .commit_cursor
                - after.commit_cursor,
            3
        );
        assert_eq!(
            installed
                .read(
                    &engine,
                    &session,
                    repeated.status.job.updated_at_unix_ms + 1
                )
                .job,
            repeated.status.job
        );
    }
}
