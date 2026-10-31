use super::*;
use rrd_contract::{
    AttunementJobPage, AttunementSourceVerification, ListAttunementJobs,
    ATTUNEMENT_CONTRACT_VERSION,
};

fn request(after_job_id: Option<CanonicalId>, limit: u16) -> ListAttunementJobs {
    ListAttunementJobs {
        contract_version: ATTUNEMENT_CONTRACT_VERSION,
        after_job_id,
        limit,
    }
}

fn list(
    engine: &RrdEngine,
    session: &SessionLease,
    request: &ListAttunementJobs,
    at: u64,
) -> crate::Result<AttunementJobPage> {
    engine.list_attunement_jobs(
        &session.session_id,
        &session.token,
        request,
        at,
        "request-discovery",
        "operation-discovery",
    )
}

fn replan(
    installed: &Installed,
    engine: &RrdEngine,
    session: &SessionLease,
    job: &CanonicalId,
    generation: u64,
) -> CanonicalId {
    std::fs::write(
        installed.root.path().join("README.md"),
        format!("ordinary edit {generation}\n"),
    )
    .unwrap();
    let at = BASE + generation * 1_000;
    let preview = engine
        .preview_attunement_replan(
            &session.session_id,
            &session.token,
            job,
            1,
            at,
            "request-discovery-replan",
            "operation-discovery-replan",
        )
        .unwrap();
    assert_eq!(preview.generation, generation);
    engine
        .accept_attunement_replan(
            &session.session_id,
            &session.token,
            &preview,
            &preview.proposal_sha256,
            &context(&format!("discovery-replan-{generation}")),
            at + 1,
        )
        .unwrap()
        .successor
        .job
        .id
}

#[test]
fn discovers_initial_job_without_an_id_and_preserves_it_after_edit_and_reopen() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let request = request(None, 1);
    let first = list(&engine, &session, &request, BASE + 1_000).unwrap();
    assert_eq!(first.jobs.len(), 1);
    assert_eq!(first.jobs[0].job_id, installed.job);
    assert_eq!(first.jobs[0].generation, 0);
    assert_eq!(
        first.jobs[0].plan_sha256,
        installed.preview.attunement.plan_sha256
    );
    assert_eq!(
        first.jobs[0].source_sha256,
        installed.preview.attunement.source_sha256
    );
    assert_eq!(first.jobs[0].state, State::Pending);
    assert_eq!(first.next_after_job_id, None);
    assert_eq!(
        first.source_verification,
        AttunementSourceVerification::NotChecked
    );
    let before = installed.read(&engine, &session, BASE + 1_100).job;
    std::fs::write(
        installed.root.path().join("README.md"),
        "edited, not replanned\n",
    )
    .unwrap();
    assert_eq!(
        list(&engine, &session, &request, BASE + 1_200)
            .unwrap()
            .jobs,
        first.jobs
    );
    assert_eq!(installed.read(&engine, &session, BASE + 1_300).job, before);
    drop(engine);
    let engine = installed.reopen(BASE + 2_000);
    let session = installed.session(&engine, "discovery-reopened", BASE + 2_100);
    let after = list(&engine, &session, &request, BASE + 2_200).unwrap();
    assert_eq!(after.jobs, first.jobs);
    assert_eq!(after.target, first.target);
    assert_eq!(after.source_verification, first.source_verification);
}

#[test]
fn pages_committed_successors_and_retains_previous_pages_after_restart() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let second = replan(&installed, &engine, &session, &installed.job, 1);
    let third = replan(&installed, &engine, &session, &second, 2);
    let first = list(&engine, &session, &request(None, 2), BASE + 3_000).unwrap();
    assert_eq!(
        first
            .jobs
            .iter()
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>(),
        vec![installed.job.clone(), second.clone()]
    );
    assert_eq!(first.jobs[0].state, State::CancelRequested);
    assert_eq!(
        first.jobs[1].predecessor_job_id,
        Some(installed.job.clone())
    );
    assert_eq!(first.next_after_job_id, Some(second.clone()));
    let remainder = list(
        &engine,
        &session,
        &request(first.next_after_job_id.clone(), 2),
        BASE + 3_100,
    )
    .unwrap();
    assert_eq!(remainder.jobs.len(), 1);
    assert_eq!(remainder.jobs[0].job_id, third);
    assert_eq!(remainder.jobs[0].generation, 2);
    assert_eq!(remainder.jobs[0].predecessor_job_id, Some(second));
    assert_eq!(remainder.next_after_job_id, None);
    let exhausted = list(&engine, &session, &request(Some(third), 1), BASE + 3_200).unwrap();
    assert!(exhausted.jobs.is_empty());
    assert_eq!(exhausted.next_after_job_id, None);
    drop(engine);
    let engine = installed.reopen(BASE + 4_000);
    let session = installed.session(&engine, "lineage-reopened", BASE + 4_100);
    assert_eq!(
        list(&engine, &session, &request(None, 2), BASE + 4_200)
            .unwrap()
            .jobs,
        first.jobs
    );
    assert_eq!(
        list(
            &engine,
            &session,
            &request(first.next_after_job_id, 2),
            BASE + 4_300
        )
        .unwrap()
        .jobs,
        remainder.jobs
    );
}

#[test]
fn rejects_missing_foreign_unauthenticated_and_oversized_discovery() {
    let (_installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let (foreign, _other, _) = Installed::new(InstallationTargetKind::ExistingProject);
    for cursor in [CanonicalId::new("job-absent").unwrap(), foreign.job] {
        assert!(list(&engine, &session, &request(Some(cursor), 1), BASE + 1_000).is_err());
    }
    for limit in [0, rrd_contract::MAX_ATTUNEMENT_JOB_PAGE + 1] {
        assert!(list(&engine, &session, &request(None, limit), BASE + 1_100).is_err());
    }
    assert!(engine
        .list_attunement_jobs(
            &session.session_id,
            &id("invalid-token"),
            &request(None, 1),
            BASE + 1_200,
            "request-denied-list",
            "operation-denied-list"
        )
        .is_err());
}

#[test]
fn rejects_a_page_if_control_changes_during_collection() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    let request = request(None, 1);
    let collected = engine
        .collect_attunement_jobs(&request, BASE + 1_000)
        .unwrap();
    call(
        &engine,
        &session,
        &installed.start(30_000),
        "discovery-concurrent-start",
        BASE + 1_100,
    )
    .unwrap();
    assert!(matches!(
        engine.finish_attunement_discovery(&request, collected),
        Err(ServiceError::StorageConflict(_))
    ));
    let fresh = list(&engine, &session, &request, BASE + 1_200).unwrap();
    assert_eq!(fresh.jobs[0].state, State::Running);
    assert_eq!(fresh.jobs[0].revision, 2);
}

#[test]
fn corrupt_successor_links_do_not_return_partial_pages() {
    let (installed, engine, session) = Installed::new(InstallationTargetKind::ExistingProject);
    replan(&installed, &engine, &session, &installed.job, 1);
    let key = format!(
        "{}/successor",
        crate::engine::installation::attunement_job_key(engine.instance_id(), &installed.job)
    );
    let expected = engine.storage.control().get(&key).unwrap();
    let mut link: crate::AttunementReplanPreview =
        serde_json::from_slice(expected.as_ref().unwrap()).unwrap();
    link.generation += 1;
    engine
        .storage
        .control()
        .commit(&ControlTransition {
            key,
            expected,
            replacement: Some(serde_json::to_vec(&link).unwrap()),
            at: BASE + 2_000,
            actor: "test-fixture".into(),
            action: "attunement.fixture_corrupt_link".into(),
            request_id: "request-discovery-corrupt".into(),
            operation_id: "operation-discovery-corrupt".into(),
        })
        .unwrap();
    assert!(list(&engine, &session, &request(None, 1), BASE + 2_100).is_err());
    assert!(list(
        &engine,
        &session,
        &request(Some(installed.job), 1),
        BASE + 2_200
    )
    .is_err());
}
