use super::*;

#[test]
fn selected_supported_effects_finish_replay_and_reopen_without_fixed_verify() {
    for phases in [
        &[Phase::Connect][..],
        &[Phase::Connect, Phase::Inventory][..],
    ] {
        let proposal = crate::AttunementProposal {
            format_version: crate::ATTUNEMENT_PROPOSAL_FORMAT_VERSION,
            steps: phases
                .iter()
                .copied()
                .map(|phase| crate::AttunementProposalStep {
                    phase,
                    description: format!("selected {phase:?}"),
                    estimated_items: 0,
                    estimated_input_bytes: 0,
                    parse_policy: None,
                })
                .collect(),
            estimated_min_duration_ms: 0,
            estimated_max_duration_ms: 30_000,
        };
        let (installed, engine, session) = Installed::with_source(
            InstallationTargetKind::ExistingProject,
            Some(&proposal),
            |root| std::fs::write(root.join("source.txt"), "source for inventory\n").unwrap(),
        );
        let selected: Vec<_> = installed
            .preview
            .attunement
            .phases
            .iter()
            .map(|p| p.phase)
            .collect();
        assert_eq!(selected, phases);
        let started = call(
            &engine,
            &session,
            &installed.start(30_000),
            "adaptive-start",
            BASE + 1_000,
        )
        .unwrap();
        let mut job = started.status.job;
        let mut last_command = None;
        for (index, phase) in phases.iter().enumerate() {
            assert_eq!(job.current_phase, Some(*phase));
            let command = step(&job);
            let key = format!("adaptive-step-{index}");
            let result = call(
                &engine,
                &session,
                &command,
                &key,
                job.updated_at_unix_ms + 1,
            )
            .unwrap();
            job = result.status.job;
            last_command = Some((command, key));
        }
        assert_eq!(job.state, State::Succeeded);
        assert_eq!(job.checkpoints.len(), phases.len());
        assert_eq!(job.plan_sha256, installed.preview.attunement.plan_sha256);
        let scope = ScopeId::new(format!("instance:{}", engine.instance_id())).unwrap();
        let stamp = engine.storage.runtime().read_stamp(&scope).unwrap();
        assert_eq!(
            job.checkpoints.last().unwrap().coordinates.runtime_cursor,
            stamp.commit_cursor
        );
        let (command, key) = last_command.unwrap();
        let replay = call(
            &engine,
            &session,
            &command,
            &key,
            job.updated_at_unix_ms + 1,
        )
        .unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.status.job, job);
        assert_eq!(engine.storage.runtime().read_stamp(&scope).unwrap(), stamp);
        let finished_at = job.updated_at_unix_ms;
        drop(engine);
        let engine = installed.reopen(finished_at + 2);
        let session = installed.session(&engine, "adaptive-reopen", finished_at + 3);
        assert_eq!(installed.read(&engine, &session, finished_at + 4).job, job);
        if phases.contains(&Phase::Inventory) {
            let changes = engine
                .storage
                .runtime()
                .read_changes(&stamp, 0, 512)
                .unwrap();
            assert!(changes.changes.iter().any(|change| matches!(&change.mutation,
                RuntimeMutation::Record { record } if record.reference.kind.as_str() == "source-tree-snapshot")));
        }
        std::fs::write(installed.root.path().join("source.txt"), "ordinary edit\n").unwrap();
        let preview = engine
            .preview_attunement_replan(
                &session.session_id,
                &session.token,
                &installed.job,
                job.revision,
                finished_at + 5,
                "adaptive-replan-preview",
                "adaptive-replan-preview",
            )
            .unwrap();
        assert_ne!(
            preview.plan.source_sha256,
            installed.preview.attunement.source_sha256
        );
        std::fs::write(
            installed.root.path().join("source.txt"),
            "changed after preview\n",
        )
        .unwrap();
        assert!(matches!(
            engine.accept_attunement_replan(
                &session.session_id,
                &session.token,
                &preview,
                &preview.proposal_sha256,
                &context("adaptive-stale-accept"),
                finished_at + 6,
            ),
            Err(ServiceError::AttunementSourceChanged)
        ));
        assert_eq!(installed.read(&engine, &session, finished_at + 7).job, job);
        let preview = engine
            .preview_attunement_replan(
                &session.session_id,
                &session.token,
                &installed.job,
                job.revision,
                finished_at + 8,
                "adaptive-fresh-preview",
                "adaptive-fresh-preview",
            )
            .unwrap();
        let accepted = engine
            .accept_attunement_replan(
                &session.session_id,
                &session.token,
                &preview,
                &preview.proposal_sha256,
                &context("adaptive-replan-accept"),
                finished_at + 9,
            )
            .unwrap();
        assert_eq!(accepted.predecessor.job, job);
        assert_eq!(accepted.successor.job.state, State::Pending);
        assert_ne!(accepted.plan.plan_sha256, job.plan_sha256);
        let successor = &accepted.successor.job;
        let started = call(
            &engine,
            &session,
            &AttunementCommand::Start {
                job_id: successor.id.clone(),
                plan_sha256: successor.plan_sha256.clone(),
                expected_revision: successor.revision,
                lease_ms: 30_000,
            },
            "adaptive-successor-start",
            successor.updated_at_unix_ms + 1,
        )
        .unwrap();
        let mut successor = started.status.job;
        for (index, phase) in phases.iter().enumerate() {
            assert_eq!(successor.current_phase, Some(*phase));
            successor = call(
                &engine,
                &session,
                &step(&successor),
                &format!("adaptive-successor-step-{index}"),
                successor.updated_at_unix_ms + 1,
            )
            .unwrap()
            .status
            .job;
        }
        assert_eq!(successor.state, State::Succeeded);
        assert_eq!(successor.checkpoints.len(), phases.len());
        let at = successor.updated_at_unix_ms;
        drop(engine);
        let engine = installed.reopen(at + 1);
        let session = installed.session(&engine, "adaptive-replan-reopen", at + 2);
        assert_eq!(installed.read(&engine, &session, at + 3).job, job);
        assert_eq!(
            engine
                .read_attunement(
                    &session.session_id,
                    &session.token,
                    &successor.id,
                    at + 3,
                    "adaptive-successor-read",
                    "adaptive-successor-read",
                )
                .unwrap()
                .job,
            successor
        );
        let replay = engine
            .accept_attunement_replan(
                &session.session_id,
                &session.token,
                &preview,
                &preview.proposal_sha256,
                &context("adaptive-replan-accept"),
                at + 4,
            )
            .unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.predecessor.job, job);
        assert_eq!(replay.successor.job, accepted.successor.job);
    }
}
