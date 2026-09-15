use super::*;
use crate::engine::clock::FixedClock;
use crate::{
    ClockAssessmentStatus, ClockObservationFailure, ClockSourceKind, ClockTrustLevel,
    InstallationPreview, InstallationVerificationReport, InstallationVerificationStatus,
};
use rrd_contract::{installation_plan_sha256, InstallationTargetKind};
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
        "default",
        None,
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

#[test]
fn planning_is_byte_identical_and_has_no_project_effect() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "walking product\n").unwrap();
    let before = inventory(project.path());
    let first = plan(project.path());
    let second = plan(project.path());
    assert_eq!(first.installation.initial_grants, SecurityAction::ALL);
    assert_eq!(first.installation.managed_paths.len(), 8);
    assert!(serde_json::to_value(&first.installation)
        .unwrap()
        .get("project_root")
        .is_none());
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert_eq!(inventory(project.path()), before);
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
        r#"format_version = 2

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

    let nested = moved.join("nested");
    std::fs::create_dir(&nested).unwrap();
    assert!(open_at(&nested, &executable, 1_800_000_000_001).is_err());

    std::fs::create_dir(&foreign).unwrap();
    std::fs::write(foreign.join("README.md"), "different project\n").unwrap();
    let foreign_preview = RrdEngine::plan_installation(
        &foreign,
        InstallationTargetKind::ExistingProject,
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

#[test]
fn pre_canonical_rrflow_state_blocks_a_parallel_installation_authority() {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join(".rrflow")).unwrap();
    std::fs::write(project.path().join(".rrflow/instance.toml"), "format = 1\n").unwrap();
    let before = inventory(project.path());
    assert!(RrdEngine::plan_installation(
        project.path(),
        InstallationTargetKind::ExistingProject,
        "default",
        None,
        &test_executable(),
    )
    .is_err());
    assert_eq!(inventory(project.path()), before);
}
