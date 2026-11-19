use super::*;
use crate::engine::clock::FixedClock;
use crate::engine::instance_registry::{HostPlatform, InstanceRegistry};
use rrd_contract::InstallationTargetKind;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const INSTALLED_AT: u64 = 1_800_000_000_000;

fn executable() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// Installs a project whose instance is derived from `identity`.
fn installed_project(identity: char) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "project").unwrap();
    let preview = RrdEngine::plan_installation(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &identity.to_string().repeat(64),
        "default",
        None,
        &executable(),
    )
    .unwrap();
    RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        INSTALLED_AT,
        &executable(),
    )
    .unwrap();
    project
}

fn open(project: &Path) -> RrdEngine {
    RrdEngine::open_installed_with_clock(
        project,
        &executable(),
        &FixedClock::at(INSTALLED_AT + 1_000),
    )
    .unwrap()
}

fn registry() -> (tempfile::TempDir, InstanceRegistry) {
    let home = tempfile::tempdir().unwrap();
    let registry = InstanceRegistry::at(home.path().join("state").join("instances"));
    (home, registry)
}

/// The one entry file of an instance, whatever its project root digest is.
fn entry_file(registry: &InstanceRegistry, instance_id: &CanonicalId) -> PathBuf {
    let mut files: Vec<PathBuf> = std::fs::read_dir(registry.root())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(&format!("{instance_id}."))
        })
        .collect();
    assert_eq!(files.len(), 1, "one entry for {instance_id}: {files:?}");
    files.remove(0)
}

fn file_names(registry: &InstanceRegistry) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(registry.root())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

fn canonical(project: &Path) -> String {
    std::fs::canonicalize(project)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

#[test]
fn the_registry_is_found_from_the_override_the_variable_or_the_users_state_directory() {
    let base = tempfile::tempdir().unwrap();
    let base = base.path();
    let with = |pairs: &[(&str, PathBuf)]| {
        let pairs: Vec<(String, OsString)> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone().into_os_string()))
            .collect();
        move |name: &str| {
            pairs
                .iter()
                .find(|(known, _)| known == name)
                .map(|(_, value)| value.clone())
        }
    };
    let resolve = |platform, flag: Option<&Path>, variables: &[(&str, PathBuf)]| {
        InstanceRegistry::resolve_for(platform, flag, None, &with(variables))
            .map(|registry| registry.root().to_owned())
    };
    let everything = [
        ("RRFLOW_INSTANCE_REGISTRY", base.join("named")),
        ("LOCALAPPDATA", base.join("local")),
        ("USERPROFILE", base.join("profile")),
        ("XDG_STATE_HOME", base.join("xdg")),
        ("HOME", base.join("home")),
    ];

    for platform in [
        HostPlatform::Windows,
        HostPlatform::MacOs,
        HostPlatform::Unix,
    ] {
        assert_eq!(
            resolve(platform, Some(&base.join("flag")), &everything).unwrap(),
            base.join("flag"),
            "the explicit override wins"
        );
        assert_eq!(
            resolve(platform, None, &everything).unwrap(),
            base.join("named"),
            "the variable wins over the platform directory"
        );
        assert!(
            resolve(platform, None, &[]).is_err(),
            "nothing names a state directory"
        );
    }

    let platform_only = &everything[1..];
    assert_eq!(
        resolve(HostPlatform::Windows, None, platform_only).unwrap(),
        base.join("local").join("rrflow").join("instances")
    );
    // Windows names its per-user state directory with LOCALAPPDATA and with
    // nothing else; a profile folder is not a substitute for it.
    let error = resolve(
        HostPlatform::Windows,
        None,
        &[("USERPROFILE", base.join("profile"))],
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("--instance-registry")
            && error.to_string().contains("RRFLOW_INSTANCE_REGISTRY"),
        "{error}"
    );
    assert_eq!(
        resolve(HostPlatform::MacOs, None, platform_only).unwrap(),
        base.join("home")
            .join("Library")
            .join("Application Support")
            .join("rrflow")
            .join("instances")
    );
    assert_eq!(
        resolve(HostPlatform::Unix, None, platform_only).unwrap(),
        base.join("xdg").join("rrflow").join("instances")
    );
    assert_eq!(
        resolve(HostPlatform::Unix, None, &[("HOME", base.join("home"))]).unwrap(),
        base.join("home")
            .join(".local")
            .join("state")
            .join("rrflow")
            .join("instances")
    );
    // A relative or empty base depends on the working directory; it is ignored.
    assert_eq!(
        resolve(
            HostPlatform::Unix,
            None,
            &[
                ("XDG_STATE_HOME", PathBuf::from("relative/state")),
                ("RRFLOW_INSTANCE_REGISTRY", PathBuf::new()),
                ("HOME", base.join("home")),
            ]
        )
        .unwrap(),
        base.join("home")
            .join(".local")
            .join("state")
            .join("rrflow")
            .join("instances")
    );
    // The product's own variable is refused when it is relative: it would
    // name a different registry in every folder a command runs in.
    for platform in [
        HostPlatform::Windows,
        HostPlatform::MacOs,
        HostPlatform::Unix,
    ] {
        let error = resolve(
            platform,
            None,
            &[
                (
                    "RRFLOW_INSTANCE_REGISTRY",
                    PathBuf::from("relative/registry"),
                ),
                ("LOCALAPPDATA", base.join("local")),
                ("HOME", base.join("home")),
            ],
        )
        .unwrap_err();
        assert!(matches!(error, ServiceError::Contract(_)), "{error:?}");
        assert!(
            error.to_string().contains("RRFLOW_INSTANCE_REGISTRY")
                && error.to_string().contains("absolute"),
            "{error}"
        );
    }
    // The explicit override is a path argument, resolved where it is given.
    assert_eq!(
        resolve(
            HostPlatform::Unix,
            Some(Path::new("relative/registry")),
            &[]
        )
        .unwrap(),
        std::env::current_dir()
            .unwrap()
            .join("relative")
            .join("registry")
    );
}

#[test]
fn the_lock_wait_is_a_named_default_the_flag_or_the_variable_overrides() {
    let home = tempfile::tempdir().unwrap();
    let root = home.path().join("instances");
    let wait = |flag: Option<u64>, variable: Option<&str>| {
        let variable = variable.map(OsString::from);
        InstanceRegistry::resolve_for(HostPlatform::Unix, Some(&root), flag, &|name| {
            (name == "RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS")
                .then(|| variable.clone())
                .flatten()
        })
        .map(|registry| registry.lock_wait())
    };
    assert_eq!(
        wait(None, None).unwrap(),
        Duration::from_millis(crate::engine::DEFAULT_INSTANCE_REGISTRY_LOCK_WAIT_MS)
    );
    assert_eq!(wait(None, Some("250")).unwrap(), Duration::from_millis(250));
    assert_eq!(
        wait(Some(7), Some("250")).unwrap(),
        Duration::from_millis(7)
    );
    assert_eq!(wait(Some(0), None).unwrap(), Duration::ZERO);
    for unusable in ["soon", "-1", "1.5"] {
        let error = wait(None, Some(unusable)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS"),
            "{error}"
        );
    }
}

#[test]
fn an_installed_project_is_registered_without_a_credential() {
    let project = installed_project('1');
    let (_home, registry) = registry();
    assert!(
        !registry.list().unwrap().registry_present,
        "a registry nothing has written lists nothing"
    );
    assert!(registry.list().unwrap().instances.is_empty());

    let registration = registry.record_installed(project.path()).unwrap();
    assert!(registration.other_project_roots.is_empty());
    let entry = registration.entry;

    let locator = RrdEngine::read_project_locator(project.path()).unwrap();
    assert_eq!(entry.instance_id, locator.identity.instance_id);
    assert_eq!(entry.project_root, canonical(project.path()));
    assert_eq!(entry.product_version, locator.product_version);
    assert!(!entry.serving_published);

    let file = entry_file(&registry, &entry.instance_id);
    let written: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    let mut fields: Vec<_> = written.as_object().unwrap().keys().cloned().collect();
    fields.sort();
    assert_eq!(
        fields,
        [
            "format_version",
            "instance_id",
            "product_version",
            "project_root",
            "serving_published"
        ],
        "the entry holds no credential or other field"
    );
    let credential = RrdEngine::read_installed_api_key(project.path()).unwrap();
    assert!(
        !String::from_utf8_lossy(&std::fs::read(&file).unwrap()).contains(credential.credential())
    );

    let listing = registry.list().unwrap();
    assert!(listing.registry_present);
    assert!(listing.unreadable.is_empty());
    assert_eq!(listing.instances.len(), 1);
    let status = &listing.instances[0];
    assert_eq!(status.entry, entry);
    assert!(status.project_present && status.locator_present);
    assert!(!status.serving_record_present);
    assert_eq!(status.endpoint, None);
    assert!(status.problems.is_empty(), "{:?}", status.problems);
}

#[test]
fn serving_is_registered_when_published_and_when_withdrawn() {
    let project = installed_project('1');
    let (_home, registry) = registry();
    let engine = open(project.path());
    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41310".parse().unwrap())
        .unwrap();

    assert!(
        registry
            .record_serving(project.path(), true)
            .unwrap()
            .entry
            .serving_published
    );
    let listing = registry.list().unwrap();
    let status = &listing.instances[0];
    assert!(status.entry.serving_published && status.serving_record_present);
    assert_eq!(status.endpoint.as_deref(), Some("http://127.0.0.1:41310"));
    assert_eq!(status.process_id, Some(std::process::id()));
    assert!(status.problems.is_empty(), "{:?}", status.problems);
    // Recording an installation reads the project, so it keeps what is published.
    assert!(
        registry
            .record_installed(project.path())
            .unwrap()
            .entry
            .serving_published
    );

    registration.withdraw().unwrap();
    assert!(
        !registry
            .record_installed(project.path())
            .unwrap()
            .entry
            .serving_published
    );
    let listing = registry.list().unwrap();
    assert_eq!(listing.instances.len(), 1, "one entry per instance");
    let status = &listing.instances[0];
    assert!(!status.entry.serving_published && !status.serving_record_present);
    assert_eq!(status.endpoint, None);
}

#[test]
fn a_missing_project_and_a_damaged_entry_are_reported_not_dropped() {
    let kept = installed_project('1');
    let removed = installed_project('2');
    let (_home, registry) = registry();
    let kept_entry = registry.record_installed(kept.path()).unwrap().entry;
    let removed_entry = registry.record_installed(removed.path()).unwrap().entry;
    assert_ne!(kept_entry.instance_id, removed_entry.instance_id);
    drop(removed);
    std::fs::write(registry.root().join("damaged.json"), b"{ not an entry").unwrap();
    std::fs::write(registry.root().join("notes.txt"), b"not an entry").unwrap();

    let listing = registry.list().unwrap();

    assert_eq!(listing.instances.len(), 2);
    let missing = listing
        .instances
        .iter()
        .find(|status| status.entry == removed_entry)
        .expect("the entry for the removed project is still listed");
    assert!(!missing.project_present && !missing.locator_present);
    assert_eq!(missing.problems, ["the project folder no longer exists"]);
    let present = listing
        .instances
        .iter()
        .find(|status| status.entry == kept_entry)
        .unwrap();
    assert!(present.project_present && present.problems.is_empty());
    let unreadable: Vec<_> = listing
        .unreadable
        .iter()
        .map(|entry| entry.file.as_str())
        .collect();
    assert_eq!(unreadable, ["damaged.json", "notes.txt"]);

    // A project whose engram was removed is reported as no longer installed.
    std::fs::remove_dir_all(kept.path().join(".rrflow")).unwrap();
    let listing = registry.list().unwrap();
    let uninstalled = listing
        .instances
        .iter()
        .find(|status| status.entry == kept_entry)
        .unwrap();
    assert!(uninstalled.project_present && !uninstalled.locator_present);
    assert_eq!(uninstalled.problems.len(), 1);
}

#[test]
fn a_registry_that_cannot_be_written_is_an_error_the_caller_can_report() {
    let project = installed_project('1');
    let home = tempfile::tempdir().unwrap();
    let blocker = home.path().join("not-a-directory");
    std::fs::write(&blocker, b"a file where a directory is needed").unwrap();
    let registry = InstanceRegistry::at(blocker.join("instances"));

    assert!(registry.record_installed(project.path()).is_err());
    assert!(registry.record_serving(project.path(), true).is_err());
    assert_eq!(
        std::fs::read(&blocker).unwrap(),
        b"a file where a directory is needed"
    );
}

#[test]
fn concurrent_writers_and_listers_never_corrupt_the_registry() {
    const WRITERS_PER_PROJECT: usize = 4;
    const ROUNDS: usize = 20;
    let projects = [installed_project('1'), installed_project('2')];
    let (_home, registry) = registry();
    let done = std::sync::atomic::AtomicBool::new(false);

    std::thread::scope(|scope| {
        let lister = scope.spawn(|| {
            let mut listings = 0_usize;
            while !done.load(std::sync::atomic::Ordering::Relaxed) {
                let listing = registry.list().unwrap();
                assert!(listing.unreadable.is_empty(), "{:?}", listing.unreadable);
                assert!(listing.instances.len() <= projects.len());
                listings += 1;
            }
            listings
        });
        let writers: Vec<_> = projects
            .iter()
            .flat_map(|project| (0..WRITERS_PER_PROJECT).map(move |writer| (project, writer)))
            .map(|(project, writer)| {
                let registry = &registry;
                scope.spawn(move || {
                    for round in 0..ROUNDS {
                        registry
                            .record_serving(project.path(), (round + writer) % 2 == 0)
                            .unwrap();
                    }
                })
            })
            .collect();
        for writer in writers {
            writer.join().unwrap();
        }
        done.store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(lister.join().unwrap() > 0);
    });

    let listing = registry.list().unwrap();
    assert!(listing.unreadable.is_empty(), "{:?}", listing.unreadable);
    let mut roots: Vec<_> = listing
        .instances
        .iter()
        .map(|status| status.entry.project_root.clone())
        .collect();
    roots.sort();
    let mut expected: Vec<_> = projects
        .iter()
        .map(|project| canonical(project.path()))
        .collect();
    expected.sort();
    assert_eq!(roots, expected);
    let names = file_names(&registry);
    assert_eq!(names.len(), 3, "two entries and the lock: {names:?}");
    assert_eq!(names[0], ".lock");
    assert!(names[1..].iter().all(|name| name.ends_with(".json")));
}

/// Holds the registry lock from another open file, the way another process
/// that has stopped making progress would. It is opened as every registry
/// writer opens it.
fn hold_lock(registry: &InstanceRegistry) -> std::fs::File {
    let path =
        rrd_store::filesystem::AnchoredPath::from_path(&registry.root().join(".lock")).unwrap();
    let lock = crate::engine::token_key::open_private_empty_anchored(&path, false).unwrap();
    lock.lock().unwrap();
    lock
}

#[test]
fn a_held_lock_never_hangs_a_writer_or_a_lister() {
    let project = installed_project('1');
    let (_home, registry) = registry();
    let entry = registry.record_installed(project.path()).unwrap().entry;
    let held = hold_lock(&registry);

    // A writer gives up at its bound and says what to change; it does not
    // wait for a holder that may never let go.
    for wait in [Duration::ZERO, Duration::from_millis(150)] {
        let bounded = registry.clone().with_lock_wait(wait);
        let started = Instant::now();
        let error = bounded.record_serving(project.path(), true).unwrap_err();
        assert!(started.elapsed() >= wait);
        assert!(
            matches!(error, ServiceError::StorageConflict(_)),
            "{error:?}"
        );
        let text = error.to_string();
        assert!(
            text.contains("busy")
                && text.contains("--instance-registry-lock-wait-ms")
                && text.contains("RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS"),
            "{text}"
        );

        // A lister reads the whole entries without the lock and says so.
        let listing = bounded.list().unwrap();
        assert!(
            listing
                .read_without_lock
                .as_deref()
                .is_some_and(|reason| reason.contains("lock")),
            "{:?}",
            listing.read_without_lock
        );
        assert!(listing.unreadable.is_empty(), "{:?}", listing.unreadable);
        assert_eq!(listing.instances.len(), 1);
        assert_eq!(listing.instances[0].entry, entry, "the entry is unchanged");
    }

    // A writer that is still waiting when the holder lets go gets the lock.
    let waiting = registry.clone().with_lock_wait(Duration::from_secs(60));
    std::thread::scope(|scope| {
        let writer = scope.spawn(|| waiting.record_serving(project.path(), true));
        std::fs::File::unlock(&held).unwrap();
        assert!(writer.join().unwrap().unwrap().entry.serving_published);
    });
    let listing = registry.list().unwrap();
    assert_eq!(listing.read_without_lock, None);
    assert!(listing.instances[0].entry.serving_published);
    assert_eq!(file_names(&registry).len(), 2, "one entry and the lock");
}

#[test]
fn a_writer_that_waited_records_what_the_project_shows_when_its_turn_comes() {
    let project = installed_project('1');
    let (_home, registry) = registry();
    let engine = open(project.path());
    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41311".parse().unwrap())
        .unwrap();
    assert!(
        registry
            .record_serving(project.path(), true)
            .unwrap()
            .entry
            .serving_published
    );
    let held = hold_lock(&registry);

    // An installer starts while the project is served and has to wait. The
    // server closes during the wait. What the installer saw before it waited
    // is no longer true, and it must not write that.
    let waiting = registry.clone().with_lock_wait(Duration::from_secs(60));
    std::thread::scope(|scope| {
        let writer = scope.spawn(|| waiting.record_installed(project.path()));
        registration.withdraw().unwrap();
        std::fs::File::unlock(&held).unwrap();
        assert!(!writer.join().unwrap().unwrap().entry.serving_published);
    });
    let listing = registry.list().unwrap();
    assert!(!listing.instances[0].entry.serving_published);
    assert!(!listing.instances[0].serving_record_present);
}

#[test]
fn listing_changes_nothing_and_needs_no_lock_file() {
    let project = installed_project('1');
    let (_home, registry) = registry();
    let entry = registry.record_installed(project.path()).unwrap().entry;
    // A registry directory somebody copied, or one on a volume this user can
    // only read, has entries and no lock file this user can create.
    std::fs::remove_file(registry.root().join(".lock")).unwrap();
    let before = file_names(&registry);
    assert_eq!(before.len(), 1);

    let listing = registry.list().unwrap();

    assert_eq!(listing.instances.len(), 1);
    assert_eq!(listing.instances[0].entry, entry);
    assert!(listing.unreadable.is_empty(), "{:?}", listing.unreadable);
    assert!(
        listing
            .read_without_lock
            .as_deref()
            .is_some_and(|reason| reason.contains("cannot be opened")),
        "{:?}",
        listing.read_without_lock
    );
    assert_eq!(file_names(&registry), before, "listing created no file");
}

#[test]
fn two_folders_installed_as_one_instance_are_both_registered_and_reported() {
    // A copied project folder keeps its original's identity. Planning two
    // folders with one identity gives exactly that state.
    let original = installed_project('1');
    let copy = installed_project('1');
    let (_home, registry) = registry();

    let first = registry.record_installed(original.path()).unwrap();
    assert!(first.other_project_roots.is_empty());
    let second = registry.record_installed(copy.path()).unwrap();
    assert_eq!(first.entry.instance_id, second.entry.instance_id);
    assert_eq!(
        second.other_project_roots,
        [canonical(original.path())],
        "the writer is told that its instance id is registered elsewhere"
    );
    // Recording one again replaces only its own entry.
    let again = registry.record_serving(original.path(), false).unwrap();
    assert_eq!(again.other_project_roots, [canonical(copy.path())]);

    let listing = registry.list().unwrap();
    assert!(listing.unreadable.is_empty(), "{:?}", listing.unreadable);
    let mut roots: Vec<_> = listing
        .instances
        .iter()
        .map(|status| status.entry.project_root.clone())
        .collect();
    roots.sort();
    let mut expected = vec![canonical(original.path()), canonical(copy.path())];
    expected.sort();
    assert_eq!(roots, expected, "neither folder replaced the other");
    for status in &listing.instances {
        let other = expected
            .iter()
            .find(|root| **root != status.entry.project_root)
            .unwrap();
        assert_eq!(status.problems.len(), 1, "{:?}", status.problems);
        assert!(
            status.problems[0].contains(other.as_str())
                && status.problems[0].contains("also installed"),
            "{:?}",
            status.problems
        );
    }

    // Once the copy is gone its entry says so and the original is clean.
    drop(copy);
    let listing = registry.list().unwrap();
    assert_eq!(listing.instances.len(), 2);
    for status in &listing.instances {
        if status.entry.project_root == canonical(original.path()) {
            assert!(status.problems.is_empty(), "{:?}", status.problems);
        } else {
            assert_eq!(status.problems, ["the project folder no longer exists"]);
        }
    }
}
