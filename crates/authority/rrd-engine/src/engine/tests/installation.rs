mod restore;

use super::*;

use crate::engine::clock::FixedClock;

use crate::{
    ClockAssessmentStatus, ClockObservationFailure, ClockSourceKind, ClockTrustLevel,
    InstallationPreview, InstallationVerificationReport, InstallationVerificationStatus,
};

use rrd_contract::{installation_plan_sha256, InstallationTargetKind, SessionLease};

use std::collections::BTreeMap;

use std::path::{Path, PathBuf};

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = std::fs::read_dir(current)
            .unwrap()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, output);
            } else {
                output.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

fn plan(project: &Path) -> InstallationPreview {
    RrdEngine::plan_installation(
        project,
        InstallationTargetKind::ExistingProject,
        &"1".repeat(64),
        "default",
        None,
        &test_executable(),
    )
    .unwrap()
}

fn plan_with_source_read(project: &Path, enabled: bool) -> InstallationPreview {
    RrdEngine::plan_installation_with_options(
        project,
        InstallationTargetKind::ExistingProject,
        &"1".repeat(64),
        "default",
        None,
        crate::InstallationPlanningOptions {
            attunement_follow: None,
            attunement_proposal: None,
            allow_project_source_read: enabled,
        },
        &test_executable(),
    )
    .unwrap()
}

fn test_executable() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

fn open_at(project: &Path, executable: &Path, at_unix_ms: u64) -> crate::Result<RrdEngine> {
    RrdEngine::open_installed_with_clock(project, executable, &FixedClock::at(at_unix_ms))
}

fn inspect_at(
    project: &Path,
    executable: &Path,
    at_unix_ms: u64,
) -> crate::Result<InstallationVerificationReport> {
    RrdEngine::inspect_installed_with_clock(project, executable, &FixedClock::at(at_unix_ms))
}

/// The lab measured search against this page cache. Everything that states a
/// default has to agree, and changing it should come with a new measurement.
#[test]
fn every_default_page_cache_is_the_one_search_was_measured_with() {
    const MEASURED: u64 = 64 * 1024 * 1024;
    let bundled = crate::load_estate_configuration(None).unwrap();
    assert_eq!(
        bundled.storage.page_cache_bytes, MEASURED,
        "the bundled configuration"
    );
    assert_eq!(
        rrd_contract::StorageLimits::default().page_cache_bytes,
        MEASURED,
        "the contract's default"
    );
    assert_eq!(
        rrd_store::DatabaseOptions::default().page_cache_bytes as u64,
        MEASURED,
        "the storage engine's default"
    );
    let installed = crate::engine::core::installed_storage_options(&bundled.storage).unwrap();
    assert_eq!(
        installed.page_cache_bytes as u64, MEASURED,
        "what a default install opens with"
    );
}

#[test]
fn a_configuration_still_chooses_its_own_page_cache() {
    let mut storage = crate::load_estate_configuration(None).unwrap().storage;
    for chosen in [0, 1024 * 1024, 1024 * 1024 * 1024] {
        storage.page_cache_bytes = chosen;
        let installed = crate::engine::core::installed_storage_options(&storage).unwrap();
        assert_eq!(installed.page_cache_bytes as u64, chosen);
    }
}

#[test]
fn installed_engine_retains_only_its_admitted_project_root_across_reopen() {
    use std::io::Read;
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let executable = test_executable();
    for (project, content) in [(first.path(), "first"), (second.path(), "second")] {
        std::fs::write(project.join("source.txt"), content).unwrap();
        let preview = RrdEngine::plan_installation(
            project,
            InstallationTargetKind::ExistingProject,
            &RrdEngine::generate_installation_identity().unwrap(),
            "default",
            None,
            &executable,
        )
        .unwrap();
        RrdEngine::apply_installation_at(
            project,
            InstallationTargetKind::ExistingProject,
            &preview,
            &preview.installation.plan_sha256,
            1_800_000_000_000,
            &executable,
        )
        .unwrap();
    }
    let first_engine = open_at(first.path(), &executable, 1_800_000_001_000).unwrap();
    let second_engine = open_at(second.path(), &executable, 1_800_000_001_000).unwrap();
    let read = |engine: &RrdEngine| {
        let root = engine
            .installed_project_root
            .as_ref()
            .expect("installed root is retained");
        assert!(root.join("../outside.txt").open_read().is_err());
        let mut content = String::new();
        root.join("source.txt")
            .open_read()
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        content
    };
    assert_eq!(read(&first_engine), "first");
    assert_eq!(read(&second_engine), "second");
    std::fs::write(first.path().join("source.txt"), "first edited").unwrap();
    assert_eq!(read(&first_engine), "first edited");
    assert_eq!(read(&second_engine), "second");
    drop(first_engine);
    let reopened = open_at(first.path(), &executable, 1_800_000_002_000).unwrap();
    assert_eq!(read(&reopened), "first edited");
    assert!(
        RrdEngine::rrflow_mx(CanonicalId::new("volatile").unwrap(), [0; 32])
            .installed_project_root
            .is_none()
    );
    let (_component_root, component) = isolated_engine();
    assert!(component.installed_project_root.is_none());
}

#[test]
fn independent_installations_do_not_reuse_content_identity() {
    for mode in [
        InstallationTargetKind::FreshProject,
        InstallationTargetKind::ExistingProject,
    ] {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        if mode == InstallationTargetKind::ExistingProject {
            for project in [first.path(), second.path()] {
                std::fs::write(project.join("README.md"), "identical source\n").unwrap();
            }
        }
        let executable = test_executable();
        let first_plan = RrdEngine::plan_installation(
            first.path(),
            mode,
            &RrdEngine::generate_installation_identity().unwrap(),
            "default",
            None,
            &executable,
        )
        .unwrap();
        let second_plan = RrdEngine::plan_installation(
            second.path(),
            mode,
            &RrdEngine::generate_installation_identity().unwrap(),
            "default",
            None,
            &executable,
        )
        .unwrap();
        assert_eq!(
            first_plan.installation.project_precondition_sha256,
            second_plan.installation.project_precondition_sha256
        );
        assert_ne!(
            first_plan.installation.target, second_plan.installation.target,
            "distinct projects with identical content must not share an estate identity"
        );
        let before = inventory(second.path());
        assert!(matches!(
            RrdEngine::apply_installation_at(
                second.path(),
                mode,
                &first_plan,
                &first_plan.installation.plan_sha256,
                1_800_000_000_000,
                &executable,
            ),
            Err(ServiceError::ProjectBindingMismatch)
        ));
        assert_eq!(
            inventory(second.path()),
            before,
            "foreign plan must have no effects"
        );
        for (project, preview) in [(first.path(), &first_plan), (second.path(), &second_plan)] {
            RrdEngine::apply_installation_at(
                project,
                mode,
                preview,
                &preview.installation.plan_sha256,
                1_800_000_000_000,
                &executable,
            )
            .unwrap();
            let reopened = open_at(project, &executable, 1_800_000_000_001).unwrap();
            assert_eq!(
                reopened.installed_estate_identity(),
                Some(&preview.installation.target)
            );
        }
        let first_key = RrdEngine::read_installed_api_key(first.path()).unwrap();
        let second_key = RrdEngine::read_installed_api_key(second.path()).unwrap();
        assert_ne!(first_key.credential(), second_key.credential());
        let engine = open_at(second.path(), &executable, 1_800_000_000_001).unwrap();
        let session = rrd_contract::CreateSession {
            limits: rrd_contract::SessionLimits {
                idle_timeout_ms: 30_000,
                absolute_timeout_ms: 60_000,
                max_open_transactions: 1,
            },
        };
        assert!(engine
            .create_authenticated_session(
                &first_key.principal_id,
                first_key.credential().as_bytes(),
                &session,
                &id("foreign-session"),
                1_800_000_000_002,
                "foreign-request",
                "foreign-operation",
            )
            .is_err());
        assert!(engine
            .create_authenticated_session(
                &second_key.principal_id,
                second_key.credential().as_bytes(),
                &session,
                &id("own-session"),
                1_800_000_000_003,
                "own-request",
                "own-operation",
            )
            .is_ok());
    }
}

#[test]
fn installation_identity_and_root_are_bound_before_effects() {
    let project = tempfile::tempdir().unwrap();
    let executable = test_executable();
    for invalid in [
        String::new(),
        "0".repeat(63),
        "A".repeat(64),
        "z".repeat(64),
    ] {
        assert!(RrdEngine::plan_installation(
            project.path(),
            InstallationTargetKind::FreshProject,
            &invalid,
            "default",
            None,
            &executable
        )
        .is_err());
        assert!(inventory(project.path()).is_empty());
    }
    let preview = plan(project.path());
    for change_identity in [true, false] {
        let mut forged = preview.clone();
        if change_identity {
            forged.installation.installation_identity = "2".repeat(64);
        } else {
            forged.installation.project_root_sha256 = "2".repeat(64);
        }
        forged.installation.plan_sha256 = installation_plan_sha256(&forged.installation).unwrap();
        assert!(RrdEngine::apply_installation_at(
            project.path(),
            InstallationTargetKind::ExistingProject,
            &forged,
            &forged.installation.plan_sha256,
            1_800_000_000_000,
            &executable
        )
        .is_err());
        assert!(inventory(project.path()).is_empty());
    }
}

#[test]
fn planning_is_byte_identical_and_has_no_project_effect() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "walking product\n").unwrap();
    let before = inventory(project.path());
    let first = plan(project.path());
    let second = plan(project.path());
    assert_eq!(
        first.installation.initial_grants,
        SecurityAction::ALL
            .into_iter()
            .filter(|action| *action != SecurityAction::ProjectSourceRead)
            .collect::<Vec<_>>()
    );
    assert_eq!(first.installation.managed_paths.len(), 13);
    assert!(serde_json::to_value(&first.installation)
        .unwrap()
        .get("project_root")
        .is_none());
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert!(serde_json::to_value(&first)
        .unwrap()
        .get("source_read_resource_scope")
        .is_none());
    assert_eq!(inventory(project.path()), before);
}

#[test]
fn source_read_is_an_explicit_canonical_installation_choice() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "walking product\n").unwrap();
    let default = plan_with_source_read(project.path(), false);
    let legacy_default_entrypoint = plan(project.path());
    let opted_in = plan_with_source_read(project.path(), true);
    assert_eq!(default, legacy_default_entrypoint);
    assert_eq!(
        serde_json::to_vec(&default).unwrap(),
        serde_json::to_vec(&legacy_default_entrypoint).unwrap()
    );
    assert_eq!(
        default.installation.initial_grants,
        rrd_contract::installation_initial_grants(false)
    );
    assert_eq!(
        opted_in.installation.initial_grants,
        rrd_contract::installation_initial_grants(true)
    );
    assert!(default.source_read_resource_scope.is_none());
    assert_eq!(
        opted_in.source_read_resource_scope.as_deref(),
        Some("instance_prefix")
    );
    assert_ne!(
        default.installation.plan_sha256,
        opted_in.installation.plan_sha256
    );
    let before = inventory(project.path());
    assert!(RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &opted_in,
        &"0".repeat(64),
        1_800_000_000_000,
        &test_executable()
    )
    .is_err());
    assert_eq!(inventory(project.path()), before);

    let mut forged = opted_in.clone();
    forged.installation.initial_grants = rrd_contract::installation_initial_grants(false);
    forged
        .installation
        .initial_grants
        .retain(|action| *action != SecurityAction::ServiceInspect);
    forged
        .installation
        .initial_grants
        .push(SecurityAction::ProjectSourceRead);
    forged.installation.initial_grants.sort();
    forged.installation.initial_grants.dedup();
    forged.installation.plan_sha256 = installation_plan_sha256(&forged.installation).unwrap();
    forged.validate().unwrap();
    assert!(RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &forged,
        &forged.installation.plan_sha256,
        1_800_000_000_000,
        &test_executable()
    )
    .is_err());
    assert_eq!(inventory(project.path()), before);

    let mut missing_scope = opted_in;
    missing_scope.source_read_resource_scope = None;
    assert!(missing_scope.validate().is_err());
}

#[test]
fn configuration_input_rejects_unknown_oversized_and_symbolic_sources() {
    let invalid = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        invalid.path(),
        "format_version = 1\nunknown_authority = true\n",
    )
    .unwrap();
    assert!(crate::load_estate_configuration(Some(invalid.path())).is_err());

    let oversized = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(oversized.path(), vec![b' '; 64 * 1024 + 1]).unwrap();
    assert!(crate::load_estate_configuration(Some(oversized.path())).is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("configuration.toml");
        let symbolic = directory.path().join("selected.toml");
        std::fs::write(&target, "format_version = 1\n").unwrap();
        symlink(target, &symbolic).unwrap();
        assert!(crate::load_estate_configuration(Some(&symbolic)).is_err());
    }
}

#[test]
fn exact_apply_reopens_and_quick_verification_changes_no_bytes() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "walking product\n").unwrap();
    let preview = plan(project.path());
    let executable = test_executable();
    let result = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();
    assert!(!result.idempotent_replay);
    assert_eq!(result.runtime_cursor, 2);
    assert!(project.path().join(".rrflow/config.toml").is_file());

    assert!(open_at(
        project.path(),
        &project.path().join("README.md"),
        1_800_000_000_001
    )
    .is_err());
    let engine = open_at(project.path(), &executable, 1_800_000_000_001).unwrap();
    assert!(engine.security_enforced().unwrap());
    assert_eq!(
        engine.installed_estate_identity(),
        Some(&preview.installation.target)
    );
    assert_eq!(
        engine.installed_deployment_profile(),
        Some(&preview.installation.deployment)
    );
    assert_eq!(
        engine.estate_configuration(),
        &preview.installation.configuration
    );
    assert_eq!(
        engine.readiness(1_800_000_000_001).unwrap().runtime_cursor,
        2
    );
    drop(engine);

    let before = inventory(project.path());
    let report = inspect_at(project.path(), &executable, 1_800_000_000_001).unwrap();
    assert_eq!(report.status, InstallationVerificationStatus::Passed);
    assert_eq!(report.clock.status, ClockAssessmentStatus::Current);
    assert_eq!(report.clock.anchor.source, ClockSourceKind::HostSystemTime);
    assert_eq!(report.clock.anchor.trust, ClockTrustLevel::Unverified);
    assert_eq!(
        report.clock.anchor.observed_at_unix_ms,
        result.applied_at_unix_ms
    );
    assert!(report.attunement_source_current);
    assert!(report.checks.iter().all(|check| check.passed));
    assert_eq!(inventory(project.path()), before);

    std::fs::write(
        project.path().join("README.md"),
        "walking product under active development\n",
    )
    .unwrap();
    let drift = inspect_at(project.path(), &executable, 1_800_000_000_002).unwrap();
    assert_eq!(drift.status, InstallationVerificationStatus::Passed);
    assert!(!drift.attunement_source_current);
    assert_ne!(
        drift.project_inventory_sha256,
        preview.installation.project_precondition_sha256
    );

    let replay = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_010,
        &executable,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.plan_sha256, result.plan_sha256);
}

#[test]
fn installed_open_and_verify_never_repair_a_torn_or_missing_active_wal() {
    use std::io::Write as _;

    for missing in [false, true] {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(project.path().join("README.md"), b"project-owned\n").unwrap();
        let preview = plan(project.path());
        let executable = test_executable();
        RrdEngine::apply_installation_at(
            project.path(),
            InstallationTargetKind::ExistingProject,
            &preview,
            &preview.installation.plan_sha256,
            1_800_000_000_000,
            &executable,
        )
        .unwrap();
        let storage = project.path().join(
            &preview
                .installation
                .managed_path(rrd_contract::InstallationManagedPathKind::StorageRoot)
                .relative_path,
        );
        let files = std::fs::read_dir(storage.join("wal"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1, "fixture must have one committed active WAL");
        if missing {
            std::fs::remove_file(&files[0]).unwrap();
        } else {
            let mut wal = std::fs::OpenOptions::new()
                .append(true)
                .open(&files[0])
                .unwrap();
            wal.write_all(b"incomplete-tail").unwrap();
            wal.sync_all().unwrap();
        }
        let damaged = inventory(project.path());
        assert!(open_at(project.path(), &executable, 1_800_000_000_001).is_err());
        assert_eq!(
            inventory(project.path()),
            damaged,
            "installed open repaired WAL"
        );
        assert!(inspect_at(project.path(), &executable, 1_800_000_000_001).is_err());
        assert_eq!(
            inventory(project.path()),
            damaged,
            "verification repaired WAL"
        );
    }
}

#[test]
fn installed_tail_repair_preserves_the_original_and_resumes_publication_boundaries() {
    use std::io::Write as _;
    for boundary in [
        "candidate-copied",
        "active-quarantined",
        "candidate-published",
        "receipt-committed",
    ] {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(project.path().join("README.md"), b"project-owned\n").unwrap();
        let preview = plan(project.path());
        let executable = test_executable();
        let now = crate::engine::clock::observe_required(
            &crate::engine::clock::HostSystemClock,
            "repair.test",
        )
        .unwrap()
        .observed_at_unix_ms;
        RrdEngine::apply_installation_at(
            project.path(),
            InstallationTargetKind::ExistingProject,
            &preview,
            &preview.installation.plan_sha256,
            now,
            &executable,
        )
        .unwrap();
        let locator = RrdEngine::read_project_locator(project.path()).unwrap();
        let active = project.path().join(&locator.storage_root);
        let wal = std::fs::read_dir(active.join("wal"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let mut file = std::fs::OpenOptions::new().append(true).open(&wal).unwrap();
        file.write_all(b"incomplete-tail").unwrap();
        file.sync_all().unwrap();
        drop(file);
        let damaged = inventory(&active);
        let before_plan = inventory(project.path());
        let repair = RrdEngine::plan_installed_tail_repair(project.path(), &executable).unwrap();
        assert_eq!(
            inventory(project.path()),
            before_plan,
            "repair planning changed bytes"
        );
        assert!(RrdEngine::apply_installed_tail_repair(
            project.path(),
            &executable,
            &repair,
            &"0".repeat(64)
        )
        .is_err());
        assert_eq!(
            inventory(project.path()),
            before_plan,
            "unapproved repair changed bytes"
        );
        let mut reached = false;
        let result = crate::engine::tail_repair::publication::apply_with_observer(
            project.path(),
            &executable,
            &repair,
            &mut |point| {
                if point == boundary {
                    reached = true;
                    Err(ServiceError::Storage("owned repair interruption".into()))
                } else {
                    Ok(())
                }
            },
        );
        assert!(
            reached && result.is_err(),
            "repair boundary not reached: {boundary}"
        );
        let quarantine = project.path().join(&repair.publication.quarantine_root);
        let preserved = if quarantine.exists() {
            &quarantine
        } else {
            &active
        };
        assert_eq!(
            inventory(preserved),
            damaged,
            "repair mutated its damaged input at {boundary}"
        );
        let receipt = RrdEngine::apply_installed_tail_repair(
            project.path(),
            &executable,
            &repair,
            &repair.plan_sha256,
        )
        .unwrap();
        assert_eq!(receipt.runtime_cursor, 2);
        assert_eq!(receipt.storage.discarded_bytes, 15);
        assert_eq!(
            inventory(&quarantine),
            damaged,
            "quarantined evidence changed"
        );
        let report = RrdEngine::inspect_installed(project.path(), &executable).unwrap();
        assert_eq!(report.status, InstallationVerificationStatus::Passed);
        let completed = inventory(project.path());
        let replay = RrdEngine::apply_installed_tail_repair(
            project.path(),
            &executable,
            &repair,
            &repair.plan_sha256,
        )
        .unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.plan_sha256, receipt.plan_sha256);
        assert_eq!(
            inventory(project.path()),
            completed,
            "completed retry wrote another repair"
        );
        assert_eq!(
            std::fs::read(project.path().join("README.md")).unwrap(),
            b"project-owned\n"
        );
    }
}

#[test]
fn unavailable_or_rolled_back_clock_blocks_effects_but_not_read_only_diagnosis() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "clock boundary\n").unwrap();
    let preview = plan(project.path());
    let executable = test_executable();
    let before = inventory(project.path());
    let unavailable = FixedClock::unavailable(ClockObservationFailure::BeforeUnixEpoch);
    let error = RrdEngine::apply_installation_with_test_clock(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        &executable,
        &unavailable,
    )
    .unwrap_err();
    assert!(matches!(error, ServiceError::ClockUnavailable(_)));
    assert_eq!(inventory(project.path()), before);

    let installed = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        10_000,
        &executable,
    )
    .unwrap();
    assert_eq!(installed.applied_at_unix_ms, 10_000);
    let installed_bytes = inventory(project.path());

    let replay_error = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        9_999,
        &executable,
    )
    .unwrap_err();
    assert!(matches!(
        replay_error,
        ServiceError::ClockRollback {
            observed_at_unix_ms: 9_999,
            anchor_unix_ms: 10_000,
            maximum_rollback_ms: 0
        }
    ));
    assert_eq!(inventory(project.path()), installed_bytes);

    let error = open_at(project.path(), &executable, 9_999)
        .err()
        .expect("rolled-back clock must reject installed open");
    assert!(matches!(
        error,
        ServiceError::ClockRollback {
            observed_at_unix_ms: 9_999,
            anchor_unix_ms: 10_000,
            maximum_rollback_ms: 0
        }
    ));
    let rolled_back = inspect_at(project.path(), &executable, 9_999).unwrap();
    assert_eq!(rolled_back.status, InstallationVerificationStatus::Failed);
    assert_eq!(
        rolled_back.clock.status,
        ClockAssessmentStatus::RollbackExceeded
    );
    assert_eq!(rolled_back.clock.rollback_ms, Some(1));
    assert!(
        !rolled_back
            .checks
            .iter()
            .find(|check| check.id.as_str() == "clock")
            .unwrap()
            .passed
    );
    assert_eq!(inventory(project.path()), installed_bytes);

    let unavailable = RrdEngine::inspect_installed_with_clock(
        project.path(),
        &executable,
        &FixedClock::unavailable(ClockObservationFailure::Unrepresentable),
    )
    .unwrap();
    assert_eq!(unavailable.status, InstallationVerificationStatus::Failed);
    assert_eq!(unavailable.clock.status, ClockAssessmentStatus::Unavailable);
    assert_eq!(
        unavailable.clock.failure,
        Some(ClockObservationFailure::Unrepresentable)
    );
    assert_eq!(inventory(project.path()), installed_bytes);
}

#[test]
fn changed_project_or_credential_permissions_fail_closed() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "before\n").unwrap();
    let preview = plan(project.path());
    std::fs::write(project.path().join("README.md"), "after\n").unwrap();
    assert!(RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &test_executable(),
    )
    .is_err());
    assert!(!project.path().join(".rrflow").exists());

    let clean = tempfile::tempdir().unwrap();
    let preview = plan(clean.path());
    let executable = test_executable();
    RrdEngine::apply_installation_at(
        clean.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let credential = clean.path().join(
            &preview
                .installation
                .managed_path(rrd_contract::InstallationManagedPathKind::OperatorCredential)
                .relative_path,
        );
        std::fs::set_permissions(&credential, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(RrdEngine::inspect_installed(clean.path(), &executable).is_err());
    }
}

#[test]
fn installed_secret_aliases_deny_open_verify_and_replay_without_effects() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let executable = test_executable();
    let preview = plan(project.path());
    let at = 1_800_000_000_000;
    let applied = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        at,
        &executable,
    )
    .unwrap();
    let before = inventory(project.path());
    for kind in [
        rrd_contract::InstallationManagedPathKind::TokenKey,
        rrd_contract::InstallationManagedPathKind::OperatorCredential,
    ] {
        let secret = project
            .path()
            .join(&preview.installation.managed_path(kind).relative_path);
        let alias = outside.path().join("foreign-alias");
        std::fs::hard_link(&secret, &alias).unwrap();
        assert!(open_at(project.path(), &executable, at + 1).is_err());
        assert!(inspect_at(project.path(), &executable, at + 1).is_err());
        assert!(RrdEngine::apply_installation_at(
            project.path(),
            InstallationTargetKind::ExistingProject,
            &preview,
            &preview.installation.plan_sha256,
            at + 1,
            &executable,
        )
        .is_err());
        assert_eq!(
            inventory(project.path()),
            before,
            "denial changed installed state"
        );
        assert_eq!(
            std::fs::read(&alias).unwrap(),
            std::fs::read(&secret).unwrap()
        );
        std::fs::remove_file(&alias).unwrap();
        let report = inspect_at(project.path(), &executable, at + 1).unwrap();
        assert_eq!(report.status, InstallationVerificationStatus::Passed);
        assert_eq!(report.runtime_cursor, applied.runtime_cursor);
        assert_eq!(
            report.control_journal_sequence,
            applied.control_journal_sequence
        );
    }
}

#[cfg(unix)]
#[test]
fn installed_open_and_inspection_reject_symbolic_object_paths() {
    use std::os::unix::fs::symlink;

    let project = tempfile::tempdir().unwrap();
    let executable = test_executable();
    let preview = plan(project.path());
    RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();
    let storage = project.path().join(
        &preview
            .installation
            .managed_path(rrd_contract::InstallationManagedPathKind::StorageRoot)
            .relative_path,
    );
    let staging = storage.join("immutable/staging");
    let redirected = storage.join("immutable/redirected-staging");
    std::fs::rename(&staging, &redirected).unwrap();
    symlink(&redirected, &staging).unwrap();

    let instance = preview.installation.target.instance_id.clone();
    assert!(RrdEngine::open_existing(&storage, instance, [0; 32]).is_err());
    assert!(RrdEngine::inspect_installed(project.path(), &executable).is_err());
}

#[test]
fn rehashed_profile_field_forgery_fails_before_any_project_effect() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "walking product\n").unwrap();
    let mut forged = plan(project.path());
    forged.installation.credential_bytes = 64;
    forged.installation.bootstrap.credential_entropy_bytes = 64;
    forged.installation.plan_sha256 = installation_plan_sha256(&forged.installation).unwrap();
    forged.validate().unwrap();

    assert!(RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &forged,
        &forged.installation.plan_sha256,
        1_800_000_000_000,
        &test_executable(),
    )
    .is_err());
    assert!(!project.path().join(".rrflow").exists());
}

#[test]
fn fresh_project_apply_and_exact_replay_are_supported() {
    let project = tempfile::tempdir().unwrap();
    let executable = test_executable();
    let preview = RrdEngine::plan_installation(
        project.path(),
        InstallationTargetKind::FreshProject,
        &RrdEngine::generate_installation_identity().unwrap(),
        "default",
        None,
        &executable,
    )
    .unwrap();

    let first = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::FreshProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();
    assert!(!first.idempotent_replay);

    let replay = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::FreshProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_001,
        &executable,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.plan_sha256, first.plan_sha256);
}

#[test]
fn explicit_configuration_is_sealed_and_apply_does_not_reread_it() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "configured product\n").unwrap();
    let input = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        input.path(),
        r#"format_version = 3

[storage]
recovery_reserve_bytes = 16777216
mutable_accounted_bytes = 134217728
mutable_versions = 1048576
projected_output_bytes = 134217728
source_page_bytes = 268435456
native_transactions = 64
native_transaction_bytes = 67108864
checkpoint_entries = 1024
active_read_views = 1024
pinned_read_bytes = 4294967296
page_cache_bytes = 67108864
wal_flush_target_bytes = 67108864
memtable_flush_target_versions = 524288
compaction_input_bytes = 1073741824
compaction_input_logical_bytes = 268435456
compaction_input_versions = 4194304
compaction_output_segments = 256
compaction_elapsed_ms = 30000
compaction_l0_segments = 64
retained_mutable_bytes = 536870912
retained_mutable_versions = 4194304
retained_mutable_generations = 64
reclamation_inventory_entries = 262144
reclamation_metadata_bytes = 67108864
reclamation_delete_files = 1024
reclamation_delete_bytes = 1073741824
reclamation_elapsed_ms = 30000
transaction_admission_requests = 64
transaction_admission_bytes = 67108864
transaction_admission_wait_ms = 30000
recovery_metadata_bytes = 67108864
recovery_files = 262144
recovery_read_bytes = 1099511627776
recovery_wal_bytes = 268435456
recovery_wal_frames = 1048576
recovery_elapsed_ms = 120000

[clock]
maximum_rollback_ms = 250

[reasoning]
max_run_elapsed_ms = 120000
max_steps = 32
max_step_elapsed_ms = 30000

[recall]
max_graph_depth = 2
max_items = 32
max_output_bytes = 131072
max_storage_keys = 5000

[query]
max_storage_keys = 5000
max_rows = 9
max_output_bytes = 131072
max_batch_rows = 8
max_memory_bytes = 16777216
max_spill_bytes = 33554432
max_elapsed_ms = 5000
"#,
    )
    .unwrap();
    let executable = test_executable();
    let preview = RrdEngine::plan_installation(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &RrdEngine::generate_installation_identity().unwrap(),
        "default",
        Some(input.path()),
        &executable,
    )
    .unwrap();
    assert_eq!(preview.installation.configuration.query.max_rows, 9);
    assert_eq!(
        preview.installation.configuration.clock.maximum_rollback_ms,
        250
    );
    std::fs::write(input.path(), "this no longer parses as TOML = [").unwrap();

    RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();
    let engine = open_at(project.path(), &executable, 1_799_999_999_750).unwrap();
    assert_eq!(engine.estate_configuration().query.max_rows, 9);
    drop(engine);
    let tolerated = inspect_at(project.path(), &executable, 1_799_999_999_750).unwrap();
    assert_eq!(
        tolerated.clock.status,
        ClockAssessmentStatus::RollbackWithinTolerance
    );
    assert_eq!(tolerated.clock.rollback_ms, Some(250));
    assert_eq!(tolerated.status, InstallationVerificationStatus::Passed);
    let mut locator = RrdEngine::read_project_locator(project.path()).unwrap();
    assert_eq!(locator.configuration_revision, 1);
    assert_eq!(
        locator.configuration_sha256,
        preview.installation.configuration.configuration_sha256
    );
    assert!(!toml::to_string(&locator).unwrap().contains("project_root"));

    locator.configuration_revision = 2;
    std::fs::write(
        project.path().join(".rrflow/config.toml"),
        toml::to_string(&locator).unwrap(),
    )
    .unwrap();
    assert!(open_at(project.path(), &executable, 1_800_000_000_001).is_err());
    assert!(inspect_at(project.path(), &executable, 1_800_000_000_001).is_err());
}

#[test]
fn installed_locator_relocates_but_never_walks_ancestors_or_pairs_foreign_state() {
    let container = tempfile::tempdir().unwrap();
    let original = container.path().join("original");
    let moved = container.path().join("moved");
    let foreign = container.path().join("foreign");
    std::fs::create_dir(&original).unwrap();
    std::fs::write(original.join("README.md"), "relocatable project\n").unwrap();
    let executable = test_executable();
    let preview = RrdEngine::plan_installation(
        &original,
        InstallationTargetKind::ExistingProject,
        &RrdEngine::generate_installation_identity().unwrap(),
        "default",
        None,
        &executable,
    )
    .unwrap();
    RrdEngine::apply_installation_at(
        &original,
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();

    std::fs::rename(&original, &moved).unwrap();
    let relocated = open_at(&moved, &executable, 1_800_000_000_001).unwrap();
    assert_eq!(
        relocated.installed_estate_identity(),
        Some(&preview.installation.target)
    );
    drop(relocated);

    let replay = RrdEngine::apply_installation_at(
        &moved,
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_001,
        &executable,
    )
    .unwrap();
    assert!(replay.idempotent_replay);

    let nested = moved.join("nested");
    std::fs::create_dir(&nested).unwrap();
    assert!(open_at(&nested, &executable, 1_800_000_000_001).is_err());

    std::fs::create_dir(&foreign).unwrap();
    std::fs::write(foreign.join("README.md"), "different project\n").unwrap();
    let foreign_preview = RrdEngine::plan_installation(
        &foreign,
        InstallationTargetKind::ExistingProject,
        &RrdEngine::generate_installation_identity().unwrap(),
        "default",
        None,
        &executable,
    )
    .unwrap();
    assert_ne!(
        foreign_preview.installation.target,
        preview.installation.target
    );
    RrdEngine::apply_installation_at(
        &foreign,
        InstallationTargetKind::ExistingProject,
        &foreign_preview,
        &foreign_preview.installation.plan_sha256,
        1_800_000_000_001,
        &executable,
    )
    .unwrap();
    std::fs::copy(
        foreign.join(".rrflow/config.toml"),
        moved.join(".rrflow/config.toml"),
    )
    .unwrap();
    assert!(open_at(&moved, &executable, 1_800_000_000_001).is_err());
}

/// A leftover `.rrflow` folder from earlier tools or runs, created with the
/// platform's ordinary inherited permissions and holding no locator.
fn leftover_rrflow(project: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    std::fs::create_dir_all(project.join(".rrflow/dev/old")).unwrap();
    std::fs::create_dir_all(project.join(".rrflow/rrd")).unwrap();
    std::fs::write(project.join(".rrflow/instance.toml"), "format = 1\n").unwrap();
    std::fs::write(project.join(".rrflow/dev/old/CURRENT"), "manifest-7\n").unwrap();
    std::fs::write(project.join(".rrflow/rrd/stale.bin"), [7u8; 64]).unwrap();
    std::fs::write(project.join("README.md"), "# project\n").unwrap();
    inventory(&project.join(".rrflow"))
}

#[test]
fn a_leftover_rrflow_folder_is_adopted_in_place_and_keeps_every_file() {
    let project = tempfile::tempdir().unwrap();
    let executable = test_executable();
    let leftover = leftover_rrflow(project.path());
    let before = inventory(project.path());

    // Planning reports the adoption and changes nothing.
    let preview = plan(project.path());
    assert_eq!(
        preview.installation.adopted_estate_entries.as_deref(),
        Some(
            &[
                "dev".to_owned(),
                "instance.toml".to_owned(),
                "rrd".to_owned()
            ][..]
        )
    );
    assert_eq!(inventory(project.path()), before);

    let result = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();
    assert!(!result.idempotent_replay);

    // Every leftover byte is retained below `.rrflow/previous/<plan id>/`.
    let retained = project
        .path()
        .join(".rrflow/previous")
        .join(preview.installation.id.as_str());
    assert_eq!(inventory(&retained), leftover);
    assert!(!project.path().join(".rrflow/instance.toml").exists());
    assert!(!project.path().join(".rrflow/dev").exists());
    assert!(!project.path().join(".rrflow/rrd/stale.bin").exists());
    crate::engine::token_key::validate_private_directory(&project.path().join(".rrflow")).unwrap();

    // The adopted estate opens, verifies and replays like any other.
    open_at(project.path(), &executable, 1_800_000_000_001).unwrap();
    let report = inspect_at(project.path(), &executable, 1_800_000_000_002).unwrap();
    assert_eq!(report.status, InstallationVerificationStatus::Passed);
    let replay = RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_003,
        &executable,
    )
    .unwrap();
    assert!(replay.idempotent_replay);
}

#[test]
fn a_changed_leftover_rrflow_folder_refuses_the_stale_plan_without_effects() {
    let project = tempfile::tempdir().unwrap();
    leftover_rrflow(project.path());
    let preview = plan(project.path());
    std::fs::write(
        project.path().join(".rrflow/late.txt"),
        "added after review\n",
    )
    .unwrap();
    let before = inventory(project.path());
    assert!(RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &test_executable(),
    )
    .is_err());
    assert_eq!(inventory(project.path()), before);
    assert!(!project.path().join(".rrflow/previous").exists());
}

#[test]
fn an_installed_estate_is_never_adopted() {
    let project = tempfile::tempdir().unwrap();
    let executable = test_executable();
    let preview = plan(project.path());
    RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        1_800_000_000_000,
        &executable,
    )
    .unwrap();
    let before = inventory(project.path());
    assert!(RrdEngine::plan_installation(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &RrdEngine::generate_installation_identity().unwrap(),
        "default",
        None,
        &executable,
    )
    .is_err());
    assert_eq!(inventory(project.path()), before);
}

#[test]
fn installed_backups_and_restore_staging_live_beside_the_storage_roots() {
    use crate::engine::estate_layout::{validate_recovery_estate, EstateLayout};
    use rrd_contract::{
        CreateInstanceBackup, CreateSession, ListInstanceBackups, RestoreInstanceBackup,
        SessionLimits,
    };

    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "walking product\n").unwrap();
    let preview = plan(project.path());
    let executable = test_executable();
    RrdEngine::apply_installation(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        &executable,
    )
    .unwrap();
    let instance = preview.installation.target.instance_id.clone();
    let storage_root_id = preview.installation.storage_root_id.clone();

    let engine = RrdEngine::open_installed(project.path(), &executable).unwrap();
    let api_key = RrdEngine::read_installed_api_key(project.path()).unwrap();
    let lease = engine
        .create_authenticated_session(
            &api_key.principal_id,
            api_key.credential().as_bytes(),
            &CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 60_000,
                    absolute_timeout_ms: 60_000,
                    max_open_transactions: 1,
                },
            },
            &id("installed-backup-session"),
            1_800_000_000_100,
            "request-installed-session",
            "operation-installed-session",
        )
        .unwrap();

    let backup = engine
        .create_instance_backup(
            &lease.session_id,
            &lease.token,
            &id("installed-backup-key"),
            &CreateInstanceBackup {
                label: "installed".into(),
                created_at_unix_ms: 1_800_000_000_200,
            },
            1_800_000_000_200,
            "request-installed-backup",
            "operation-installed-backup",
        )
        .unwrap();
    assert!(backup.backup.application_complete);

    let catalogue = engine
        .list_instance_backups(
            &lease.session_id,
            &lease.token,
            &ListInstanceBackups {
                verify_archives: true,
            },
            1_800_000_000_300,
            "request-installed-backup-list",
            "operation-installed-backup-list",
        )
        .unwrap();
    assert!(catalogue.archives_verified);
    assert_eq!(catalogue.backups.len(), 1);
    assert_eq!(
        catalogue.backups[0].backup_sha256,
        backup.backup.backup_sha256
    );

    let restore_id = CanonicalId::new("restore-one").unwrap();
    let restored = engine
        .restore_instance_backup(
            &lease.session_id,
            &lease.token,
            &id("installed-restore-key"),
            &RestoreInstanceBackup {
                backup_sha256: backup.backup.backup_sha256.clone(),
                restore_id: restore_id.clone(),
                restored_at_unix_ms: 1_800_000_000_400,
            },
            1_800_000_000_400,
            "request-installed-restore",
            "operation-installed-restore",
        )
        .unwrap();
    assert!(restored.reopened);
    drop(engine);

    let container = project.path().join(".rrflow/rrd");
    assert!(container
        .join("backups")
        .join(instance.as_str())
        .join("catalogue.json")
        .is_file());
    assert!(container
        .join("restores")
        .join(instance.as_str())
        .join(restore_id.as_str())
        .is_dir());

    let mut roots = std::fs::read_dir(container.join("roots"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    roots.sort();
    assert_eq!(
        roots,
        [storage_root_id.to_string()],
        "roots/ holds only the storage root the locator selects"
    );
    assert!(
        !inventory(project.path()).keys().any(|path| path
            .components()
            .any(|component| component.as_os_str() == "rrd-service")),
        "an installed estate has no rrd-service tree"
    );

    let layout =
        EstateLayout::new(&storage_root_id, &preview.installation.initial_principal_id).unwrap();
    validate_recovery_estate(
        &super::super::estate_layout::EstateFilesystem::admit(
            rrd_store::filesystem::AnchoredPath::open_root(project.path()).unwrap(),
            false,
        )
        .unwrap(),
        &layout,
    )
    .expect("install recovery accepts an estate that holds backups and restore staging");

    let reopened = RrdEngine::open_installed(project.path(), &executable).unwrap();
    assert_eq!(
        reopened
            .readiness(1_800_000_000_500)
            .unwrap()
            .runtime_cursor,
        2
    );
}

/// #247 characterized the pre-#236 installer: an existing `.rrflow` folder
/// with ordinary inherited permissions was rejected before preview, and the
/// storage error had to name the directory. #236 adopts that folder instead,
/// so the same shape now previews as an adoption with no entries and still
/// has no file effects. The in-place preservation oracle #247 asked for stays
/// below under POAM-006.
#[cfg(windows)]
#[test]
fn inherited_estate_preview_adopts_the_folder_without_effects() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".rrflow")).unwrap();
    let before = inventory(root.path());
    let preview = RrdEngine::plan_installation(
        root.path(),
        InstallationTargetKind::ExistingProject,
        &"1".repeat(64),
        "default",
        None,
        &test_executable(),
    )
    .unwrap();
    assert_eq!(
        preview.installation.adopted_estate_entries.as_deref(),
        Some(&[][..]),
        "an empty inherited .rrflow is adopted with no entries"
    );
    assert_eq!(inventory(root.path()), before);
}

#[cfg(windows)]
#[test]
#[ignore = "POAM-006: the installer still rejects an existing .rrflow with inherited permissions; #236 adopts it by relocation, this oracle asks for in-place preservation of bytes and ACLs, and no repair for that exists yet"]
fn existing_project_preserves_inherited_estate_and_opens_after_replay() {
    use std::process::Command;

    let root = tempfile::tempdir().unwrap();
    let estate = root.path().join(".rrflow");
    let old_storage = estate.join("rrd");
    std::fs::create_dir_all(&old_storage).unwrap();
    std::fs::write(root.path().join("source.txt"), b"existing project").unwrap();
    std::fs::write(estate.join("user-notes.txt"), b"leave notes unchanged").unwrap();
    std::fs::write(old_storage.join("legacy-state"), b"leave state unchanged").unwrap();
    let before = inventory(root.path());
    let acl = |path: &Path| {
        let result = Command::new("icacls").arg(path).output().unwrap();
        assert!(result.status.success());
        result.stdout
    };
    let estate_acl = acl(&estate);
    let storage_acl = acl(&old_storage);
    assert!(estate_acl.windows(3).any(|part| part == b"(I)"));
    assert!(storage_acl.windows(3).any(|part| part == b"(I)"));
    let preview = plan(root.path());
    assert_eq!(
        inventory(root.path()),
        before,
        "preview must not modify existing data"
    );
    let executable = test_executable();
    let apply = || {
        RrdEngine::apply_installation_at(
            root.path(),
            InstallationTargetKind::ExistingProject,
            &preview,
            &preview.installation.plan_sha256,
            1_800_000_000_000,
            &executable,
        )
        .unwrap()
    };
    apply();
    let engine = open_at(root.path(), &executable, 1_800_000_001_000).unwrap();
    assert_eq!(
        engine.installed_estate_identity(),
        Some(&preview.installation.target)
    );
    drop(engine);
    assert!(apply().idempotent_replay);
    for (path, bytes) in before {
        assert_eq!(std::fs::read(root.path().join(path)).unwrap(), bytes);
    }
    assert_eq!(acl(&estate), estate_acl);
    assert_eq!(acl(&old_storage), storage_acl);
}
