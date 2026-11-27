use super::*;

#[test]
fn installed_startup_keeps_source_admission_at_the_dependent_operation() {
    let valid_git = b"[core]\n\tbare = false\n";
    let (installed, engine, session) =
        Installed::with_source(InstallationTargetKind::ExistingProject, None, |root| {
            std::fs::create_dir(root.join(".git")).unwrap();
            std::fs::write(root.join(".git/config"), valid_git).unwrap();
        });
    let original = installed.read(&engine, &session, BASE + 200).job;
    drop(engine);

    // The project input becomes uninspectable; installed engine facts do not.
    let git_config = installed.root.path().join(".git/config");
    std::fs::write(&git_config, b"[core\n").unwrap();
    assert!(RrdEngine::inspect_installed_with_clock(
        installed.root.path(),
        &installed.executable,
        &FixedClock::at(BASE + 1_000),
    )
    .is_err());
    let engine = RrdEngine::open_installed_with_clock(
        installed.root.path(),
        &installed.executable,
        &FixedClock::at(BASE + 1_000),
    )
    .expect("opening verified engine state must not require a project source scan");
    let session = installed.session(&engine, "source-unavailable-session", BASE + 1_100);
    assert_eq!(
        installed.read(&engine, &session, BASE + 1_200).job,
        original
    );
    assert!(call(
        &engine,
        &session,
        &installed.start(30_000),
        "source-unavailable-start",
        BASE + 2_000,
    )
    .is_err());
    assert_eq!(
        installed.read(&engine, &session, BASE + 2_100).job,
        original
    );

    std::fs::write(&git_config, valid_git).unwrap();
    let started = call(
        &engine,
        &session,
        &installed.start(30_000),
        "source-restored-start",
        BASE + 3_000,
    )
    .unwrap();
    let started_at = started.status.job.updated_at_unix_ms;
    assert_eq!(started.status.job.state, State::Running);
    assert!(started.status.job.checkpoints.is_empty());
    drop(engine);

    let verified = RrdEngine::inspect_installed_with_clock(
        installed.root.path(),
        &installed.executable,
        &FixedClock::at(started_at + 1),
    )
    .unwrap();
    assert!(verified.attunement_source_current);
    assert_eq!(verified.checks.len(), 13);
    let source_check = verified
        .checks
        .iter()
        .find(|check| check.id.as_str() == "project-inventory")
        .unwrap();
    assert_eq!(
        source_check.evidence_sha256,
        verified.project_inventory_sha256
    );

    // Source availability never excuses a changed engine-owned bootstrap.
    std::fs::write(
        installed.root.path().join(".rrflow/bootstrap.md"),
        b"changed installed bootstrap\n",
    )
    .unwrap();
    assert!(matches!(
        RrdEngine::open_installed_with_clock(
            installed.root.path(),
            &installed.executable,
            &FixedClock::at(started_at + 1),
        ),
        Err(ServiceError::ProjectBindingMismatch)
    ));
}
