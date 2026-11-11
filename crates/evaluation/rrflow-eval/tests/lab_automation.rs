//! A person should do three things and have the rest happen. These tests hold that automation to
//! the rules a hand-run process kept by care: a toggle cannot hide a system, tuning cannot pass as
//! the baseline, a folder cannot bring its own trust, and reading a status changes nothing.
use rrflow_eval::lab::catalog::{launch_arguments, launch_info, SystemDefinition};
use rrflow_eval::lab::definitions::content::{
    lab_files, pinned_keys, RrflowContent, RRFLOW_PRODUCT,
};
use rrflow_eval::lab::definitions::home::{DefinitionHome, PublishKind, PublishOptions};
use rrflow_eval::lab::definitions::pack::{read_pack, PackContent, TrustedKeys};
use rrflow_eval::lab::driver::{binding_sha256, test_sha256};
use rrflow_eval::lab::manifest::{Availability, IndexSpec, VectorComparisonManifest};
use rrflow_eval::lab::settings::{
    approve_all, build_plan, launch_digest, needs_approval, planned_files, Definitions, LabPlan,
    LabSettings, PlannedTest, SystemState, SystemTuning, TURNED_OFF_REASON,
};
use rrflow_eval::lab::Result;
use rrflow_eval::storage::Id;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NOW: u64 = 1_800_000_000;
const DAY: u64 = 86_400;
const PRODUCT: &str = "example-product";

/// Any product can supply this: the home knows nothing about what it carries.
struct Example;

impl PackContent for Example {
    fn product(&self) -> &str {
        PRODUCT
    }

    fn schema(&self) -> u16 {
        1
    }

    fn check(&self, files: &BTreeMap<String, String>) -> Result<()> {
        for (path, text) in files {
            serde_json::from_str::<serde_json::Value>(text)
                .map_err(|error| format!("{path}: {error}"))?;
        }
        Ok(())
    }

    fn notable_changes(
        &self,
        _before: &BTreeMap<String, String>,
        _after: &BTreeMap<String, String>,
    ) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

fn nobody() -> TrustedKeys {
    TrustedKeys { keys: Vec::new() }
}

fn home(root: &Path, name: &str) -> DefinitionHome {
    DefinitionHome::at(root.join(name), PRODUCT)
}

fn files(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
        .collect()
}

fn original() -> BTreeMap<String, String> {
    files(&[("a/one.json", "{\"n\":1}"), ("a/two.json", "{\"n\":2}")])
}

fn changed() -> BTreeMap<String, String> {
    files(&[("a/one.json", "{\"n\":1}"), ("a/two.json", "{\"n\":3}")])
}

fn options(now: u64) -> PublishOptions {
    PublishOptions {
        days: 90,
        now_unix_s: now,
    }
}

fn publish(
    home: &DefinitionHome,
    files: &BTreeMap<String, String>,
    now: u64,
) -> rrflow_eval::lab::definitions::home::PublishOutcome {
    home.publish(files, &Example, &nobody(), &options(now))
        .unwrap()
}

/// Everything under a folder, for proving nothing changed.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else {
                found.insert(path.clone(), std::fs::read(&path).unwrap());
            }
        }
    }
    found
}

#[test]
fn the_first_publish_makes_a_key_signs_and_installs() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    let outcome = publish(&publisher, &original(), NOW);
    assert_eq!(outcome.kind, PublishKind::Published);
    assert_eq!(outcome.sequence, 1);
    assert!(outcome.created_key);
    assert!(!outcome.key_pinned, "nothing pinned this new key");
    assert!(
        outcome.patch_pack.is_none(),
        "there is nothing to patch from"
    );

    let key = std::fs::read_to_string(publisher.signing_key_path()).unwrap();
    let full = std::fs::read(outcome.full_pack.as_ref().unwrap()).unwrap();
    assert!(
        !String::from_utf8_lossy(&full).contains(key.trim()),
        "the private key must not travel in a pack"
    );
    let keys = publisher.trusted_keys(&nobody()).unwrap();
    assert_eq!(keys.keys.len(), 1, "a publisher trusts its own key");
    assert_eq!(
        publisher.store().unwrap().current_files(&keys).unwrap(),
        original()
    );
}

#[test]
fn publishing_again_keeps_the_same_key() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    let first = publish(&publisher, &original(), NOW);
    let second = publish(&publisher, &changed(), NOW + 1);
    assert_eq!(first.key_id, second.key_id);
    assert!(!second.created_key);
}

#[test]
fn publishing_nothing_new_publishes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    publish(&publisher, &original(), NOW);
    let before = snapshot(&publisher.outbox());
    let again = publish(&publisher, &original(), NOW + DAY);
    assert_eq!(again.kind, PublishKind::Unchanged);
    assert_eq!(again.sequence, 1);
    assert_eq!(again.days_left, 89);
    assert!(again.full_pack.is_none());
    assert_eq!(snapshot(&publisher.outbox()), before);
}

#[test]
fn a_pack_near_expiry_is_published_again_when_nothing_changed() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    publish(&publisher, &original(), NOW);
    let later = publish(&publisher, &original(), NOW + 70 * DAY);
    assert_eq!(later.kind, PublishKind::Published);
    assert_eq!(later.sequence, 2);
    assert_eq!(later.days_left, 90);
}

#[test]
fn a_change_publishes_a_patch_and_a_full_pack_under_the_next_sequence() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    publish(&publisher, &original(), NOW);
    let outcome = publish(&publisher, &changed(), NOW + 1);
    assert_eq!(outcome.sequence, 2);
    assert_eq!(
        outcome.install.as_ref().unwrap().changed,
        vec!["a/two.json".to_owned()]
    );
    assert!(outcome.patch_bytes.unwrap() < outcome.full_bytes.unwrap());

    let keys = publisher.trusted_keys(&nobody()).unwrap();
    let full = read_pack(&std::fs::read(outcome.full_pack.unwrap()).unwrap(), &keys).unwrap();
    let patch = read_pack(&std::fs::read(outcome.patch_pack.unwrap()).unwrap(), &keys).unwrap();
    assert_eq!(full.carried.len(), 2);
    assert_eq!(patch.carried.len(), 1);
    assert_eq!(
        full.manifest_sha256, patch.manifest_sha256,
        "both are the same signed pack, so a machine ends in one state either way"
    );
}

/// Two machines: one publishes, and the other trusts it and takes what it sends.
fn two_machines(root: &Path) -> (DefinitionHome, DefinitionHome) {
    let publisher = home(root, "publisher");
    let machine = home(root, "machine");
    publish(&publisher, &original(), NOW);
    machine
        .add_trusted_keys(&publisher.trusted_keys(&nobody()).unwrap())
        .unwrap();
    (publisher, machine)
}

#[test]
fn an_update_takes_the_newest_pack_from_a_folder() {
    let root = tempfile::tempdir().unwrap();
    let (publisher, machine) = two_machines(root.path());
    publish(&publisher, &changed(), NOW + 1);
    let outcome = machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 2)
        .unwrap();
    assert_eq!(outcome.current_sequence, Some(2));
    assert_eq!(outcome.installed.unwrap().sequence, 2);
    assert!(outcome.ignored.is_empty(), "{:?}", outcome.ignored);
    let keys = machine.trusted_keys(&nobody()).unwrap();
    assert_eq!(
        machine.store().unwrap().current_files(&keys).unwrap(),
        changed()
    );
}

#[test]
fn an_update_uses_the_patch_when_it_applies() {
    let root = tempfile::tempdir().unwrap();
    let (publisher, machine) = two_machines(root.path());
    machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 1)
        .unwrap();
    let second = publish(&publisher, &changed(), NOW + 2);
    // With the full pack gone, only the patch can bring this machine forward.
    std::fs::remove_file(second.full_pack.unwrap()).unwrap();
    let outcome = machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 3)
        .unwrap();
    assert_eq!(outcome.current_sequence, Some(2));
}

#[test]
fn an_update_falls_back_to_the_full_pack_and_does_not_call_that_a_problem() {
    let root = tempfile::tempdir().unwrap();
    let (publisher, machine) = two_machines(root.path());
    // This machine never saw pack 1, so pack 2's patch cannot apply here.
    publish(&publisher, &changed(), NOW + 1);
    let outcome = machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 2)
        .unwrap();
    assert_eq!(outcome.current_sequence, Some(2));
    assert!(outcome.ignored.is_empty(), "{:?}", outcome.ignored);
}

#[test]
fn an_update_with_only_a_patch_it_cannot_use_says_a_full_pack_is_needed() {
    let root = tempfile::tempdir().unwrap();
    let (publisher, machine) = two_machines(root.path());
    let second = publish(&publisher, &changed(), NOW + 1);
    let only = root.path().join("only-patch");
    std::fs::create_dir_all(&only).unwrap();
    let patch = second.patch_pack.unwrap();
    std::fs::copy(&patch, only.join(patch.file_name().unwrap())).unwrap();
    let outcome = machine.update(&only, &Example, &nobody(), NOW + 2).unwrap();
    assert!(outcome.installed.is_none());
    assert_eq!(outcome.current_sequence, None);
    assert_eq!(outcome.ignored.len(), 1);
    assert!(
        outcome.ignored[0].reason.contains("full pack"),
        "{:?}",
        outcome.ignored
    );
}

#[test]
fn an_update_names_a_changed_pack_and_installs_the_newest_good_one() {
    let root = tempfile::tempdir().unwrap();
    let (publisher, machine) = two_machines(root.path());
    let second = publish(&publisher, &changed(), NOW + 1);
    let third = publish(&publisher, &original(), NOW + 2);
    for path in [third.full_pack.unwrap(), third.patch_pack.unwrap()] {
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            text.replacen("\\\"sequence\\\": 3", "\\\"sequence\\\": 9", 1),
        )
        .unwrap();
    }
    assert!(second.full_pack.is_some());
    let outcome = machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 3)
        .unwrap();
    assert_eq!(outcome.current_sequence, Some(2));
    assert_eq!(outcome.ignored.len(), 2, "{:?}", outcome.ignored);
    assert!(outcome
        .ignored
        .iter()
        .all(|ignored| ignored.reason.contains("signature")));
}

#[test]
fn an_update_with_nothing_newer_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let (publisher, machine) = two_machines(root.path());
    machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 1)
        .unwrap();
    let before = snapshot(machine.root());
    let outcome = machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 2)
        .unwrap();
    assert!(outcome.installed.is_none());
    assert_eq!(outcome.current_sequence, Some(1));
    assert!(outcome.ignored.is_empty(), "{:?}", outcome.ignored);
    assert_eq!(snapshot(machine.root()), before);
}

#[test]
fn a_folder_cannot_bring_its_own_trust() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    let machine = home(root.path(), "machine");
    publish(&publisher, &original(), NOW);
    // The outbox holds the publisher's public keys, and that alone must trust nothing.
    assert!(snapshot(&publisher.outbox())
        .keys()
        .any(|path| path.to_string_lossy().ends_with("trusted-keys.json")));
    let refused = machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 1)
        .unwrap();
    assert!(refused.installed.is_none());
    assert_eq!(refused.ignored.len(), 1, "{:?}", refused.ignored);
    assert!(
        refused.ignored[0].reason.contains("not trusted"),
        "{:?}",
        refused.ignored
    );

    let added = machine
        .add_trusted_keys(&publisher.trusted_keys(&nobody()).unwrap())
        .unwrap();
    assert_eq!(added.len(), 1);
    let accepted = machine
        .update(&publisher.outbox(), &Example, &nobody(), NOW + 2)
        .unwrap();
    assert_eq!(accepted.current_sequence, Some(1));
}

#[test]
fn a_key_cannot_go_by_another_keys_name() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    let machine = home(root.path(), "machine");
    publish(&publisher, &original(), NOW);
    let mut keys = publisher.trusted_keys(&nobody()).unwrap();
    keys.keys[0].id = "0000000000000000".to_owned();
    let error = machine.add_trusted_keys(&keys).unwrap_err();
    assert!(error.contains("different public key"), "{error}");
}

#[test]
fn a_key_pinned_in_the_build_is_trusted_with_no_file() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    let machine = home(root.path(), "machine");
    publish(&publisher, &original(), NOW);
    let pinned = publisher.trusted_keys(&nobody()).unwrap();
    let outcome = machine
        .update(&publisher.outbox(), &Example, &pinned, NOW + 1)
        .unwrap();
    assert_eq!(outcome.current_sequence, Some(1));
    let again = publisher
        .publish(&changed(), &Example, &pinned, &options(NOW + 2))
        .unwrap();
    assert!(again.key_pinned);
}

#[test]
fn an_update_from_a_folder_that_does_not_exist_says_so() {
    let root = tempfile::tempdir().unwrap();
    let machine = home(root.path(), "machine");
    let error = machine
        .update(&root.path().join("nowhere"), &Example, &nobody(), NOW)
        .unwrap_err();
    assert!(error.contains("nowhere"), "{error}");
}

#[test]
fn a_lost_trust_file_comes_back_from_the_key() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    let first = publish(&publisher, &original(), NOW);
    std::fs::remove_file(publisher.root().join("trusted-keys.json")).unwrap();
    let second = publish(&publisher, &changed(), NOW + 1);
    assert_eq!(second.key_id, first.key_id);
    assert_eq!(second.kind, PublishKind::Published);
}

#[test]
fn a_sequence_is_not_reused_when_the_outbox_is_emptied() {
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    publish(&publisher, &original(), NOW);
    std::fs::remove_dir_all(publisher.outbox()).unwrap();
    assert_eq!(publish(&publisher, &changed(), NOW + 1).sequence, 2);
}

#[test]
fn the_home_comes_from_the_override_and_then_the_users_own_folder() {
    let none = |_: &str| None;
    assert_eq!(
        DefinitionHome::resolve(Some(PathBuf::from("/chosen")), "acme", &none)
            .unwrap()
            .root(),
        Path::new("/chosen")
    );
    let error = DefinitionHome::resolve(None, "acme", &none).unwrap_err();
    assert!(error.contains("folder"), "{error}");
    #[cfg(windows)]
    let (name, expected) = (
        "LOCALAPPDATA",
        Path::new("/users/a").join("acme/definitions"),
    );
    #[cfg(not(windows))]
    let (name, expected) = (
        "HOME",
        Path::new("/users/a").join(".local/share/acme/definitions"),
    );
    let variable = |asked: &str| (asked == name).then(|| "/users/a".to_owned());
    let found = DefinitionHome::resolve(None, "acme", &variable).unwrap();
    assert_eq!(
        found.root().components().collect::<Vec<_>>(),
        expected.components().collect::<Vec<_>>()
    );
}

#[cfg(unix)]
#[test]
fn a_signing_key_others_can_read_is_refused() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = tempfile::tempdir().unwrap();
    let publisher = home(root.path(), "publisher");
    publish(&publisher, &original(), NOW);
    let key = publisher.signing_key_path();
    let mode = std::fs::metadata(&key).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600, "a new key is private");
    std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o644)).unwrap();
    let error = publisher
        .publish(&changed(), &Example, &nobody(), &options(NOW + 1))
        .unwrap_err();
    assert!(error.contains("others"), "{error}");
}

// What this machine chooses, against the lab's real definitions.

fn real_lab() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../eval/lab")
}

fn id(text: &str) -> Id {
    Id::try_from(text.to_owned()).unwrap()
}

fn real_definitions() -> Definitions {
    Definitions::read(&lab_files(&real_lab()).unwrap()).unwrap()
}

fn approved(definitions: &Definitions) -> LabSettings {
    let mut settings = LabSettings::default();
    approve_all(&mut settings, definitions).unwrap();
    settings
}

fn test_named<'a>(plan: &'a LabPlan, name: &str) -> &'a PlannedTest {
    plan.tests
        .iter()
        .find(|test| test.id.as_str() == name)
        .unwrap()
}

fn manifest_of<'a>(plan: &'a LabPlan, name: &str) -> &'a VectorComparisonManifest {
    test_named(plan, name).manifest.as_ref().unwrap()
}

fn state_of(plan: &LabPlan, test: &str, system: &str) -> SystemState {
    test_named(plan, test)
        .systems
        .iter()
        .find(|entry| entry.id.as_str() == system)
        .unwrap()
        .state
}

fn tuned(pairs: &[(&str, &str)]) -> SystemTuning {
    let mut tuning = SystemTuning::default();
    for (name, value) in pairs {
        tuning.set(name, value).unwrap();
    }
    tuning
}

#[test]
fn the_default_plan_equals_the_checked_in_manifests() {
    let definitions = real_definitions();
    let plan = build_plan(&definitions, &approved(&definitions)).unwrap();
    assert_eq!(plan.tests.len(), definitions.tests.len());
    for test in &plan.tests {
        assert_eq!(test.problem, None);
        let checked_in: VectorComparisonManifest = serde_json::from_str(
            &std::fs::read_to_string(real_lab().join(format!("{}.json", test.id.as_str())))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            test.manifest.as_ref().unwrap(),
            &checked_in,
            "{} must not change just by being planned",
            test.id.as_str()
        );
    }
}

#[test]
fn a_system_turned_off_stays_visible_and_cannot_start() {
    let definitions = real_definitions();
    let baseline = build_plan(&definitions, &approved(&definitions)).unwrap();
    let mut settings = approved(&definitions);
    settings.off.insert(id("qdrant"));
    let plan = build_plan(&definitions, &settings).unwrap();

    assert_eq!(
        state_of(&plan, "vector-small", "qdrant"),
        SystemState::TurnedOff
    );
    let manifest = manifest_of(&plan, "vector-small");
    assert_eq!(
        manifest.binding("qdrant").unwrap().availability,
        Availability::Unavailable {
            reason: TURNED_OFF_REASON.to_owned()
        },
        "a report shows it, with the reason"
    );
    assert_eq!(
        test_sha256(manifest).unwrap(),
        test_sha256(manifest_of(&baseline, "vector-small")).unwrap(),
        "turning a system off does not change the test"
    );

    let written = planned_files(&definitions, &settings, &plan, "vector-small").unwrap();
    let system: SystemDefinition = serde_json::from_str(&written["systems/qdrant.json"]).unwrap();
    assert!(launch_info(&system).contains(&("kind".to_owned(), "none".to_owned())));
    assert!(
        launch_arguments(&system, "/data", "/lab", None, None, &|_| None)
            .unwrap()
            .is_empty()
    );
    assert!(!written["run-list.txt"].lines().any(|line| line == "qdrant"));
    assert!(written["run-list.txt"].lines().any(|line| line == "milvus"));
}

#[test]
fn a_launch_is_held_until_approved_and_held_again_when_it_changes() {
    let mut definitions = real_definitions();
    let unapproved = build_plan(&definitions, &LabSettings::default()).unwrap();
    assert_eq!(
        state_of(&unapproved, "vector-small", "qdrant"),
        SystemState::NeedsApproval
    );
    let held = test_named(&unapproved, "vector-small")
        .systems
        .iter()
        .find(|entry| entry.id.as_str() == "qdrant")
        .unwrap();
    assert!(
        held.starts
            .as_deref()
            .unwrap_or_default()
            .contains("qdrant/qdrant@sha256:"),
        "a person is shown what approving allows: {held:?}"
    );
    assert!(!held.reason.as_deref().unwrap_or_default().is_empty());

    let mut settings = LabSettings::default();
    let system = &definitions.systems[&id("qdrant")];
    assert!(needs_approval(system));
    settings
        .approved_launches
        .insert(id("qdrant"), launch_digest(system).unwrap());
    let plan = build_plan(&definitions, &settings).unwrap();
    assert_eq!(
        state_of(&plan, "vector-small", "qdrant"),
        SystemState::WillRun
    );
    assert_eq!(
        state_of(&plan, "vector-small", "milvus"),
        SystemState::NeedsApproval,
        "approving one system approves only that one"
    );

    // A later pack starts something else, so the old approval does not cover it.
    let system = definitions.systems.get_mut(&id("qdrant")).unwrap();
    let mut value = serde_json::to_value(&system.launch).unwrap();
    value["image"] = serde_json::json!(format!("qdrant/other@sha256:{}", "a".repeat(64)));
    system.launch = serde_json::from_value(value).unwrap();
    let plan = build_plan(&definitions, &settings).unwrap();
    assert_eq!(
        state_of(&plan, "vector-small", "qdrant"),
        SystemState::NeedsApproval
    );
    let written = planned_files(&definitions, &settings, &plan, "vector-small").unwrap();
    let system: SystemDefinition = serde_json::from_str(&written["systems/qdrant.json"]).unwrap();
    assert!(
        launch_info(&system).contains(&("kind".to_owned(), "none".to_owned())),
        "a script given these files cannot start a held system"
    );
}

#[test]
fn rrflows_graph_arm_runs_like_any_other_and_says_what_it_needs() {
    let definitions = real_definitions();
    let system = &definitions.systems[&id("rrflow-hnsw")];
    assert!(
        !needs_approval(system),
        "it starts only the binary under test"
    );
    let plan = build_plan(&definitions, &LabSettings::default()).unwrap();
    assert_eq!(
        state_of(&plan, "vector-small", "rrflow-hnsw"),
        SystemState::WillRun
    );
    let binding = manifest_of(&plan, "vector-small")
        .binding("rrflow-hnsw")
        .unwrap();
    let qdrant = definitions.systems[&id("qdrant")].binding.clone();
    assert_eq!(
        binding.index, qdrant.index,
        "a graph measured beside Qdrant's is built with the same settings"
    );
    assert_eq!(
        binding
            .search_ladder
            .iter()
            .map(|setting| setting.ef)
            .collect::<Vec<_>>(),
        qdrant
            .search_ladder
            .iter()
            .map(|setting| setting.ef)
            .collect::<Vec<_>>(),
        "it searches at the same settings"
    );
    assert!(
        binding.limits.iter().any(|limit| limit.contains("pinned")),
        "a report must say it used the pinned tier: {:?}",
        binding.limits
    );
}

#[test]
fn rrflows_own_engine_needs_no_approval() {
    let definitions = real_definitions();
    assert!(!needs_approval(&definitions.systems[&id("rrflow-exact")]));
    let plan = build_plan(&definitions, &LabSettings::default()).unwrap();
    assert_eq!(
        state_of(&plan, "vector-small", "rrflow-exact"),
        SystemState::WillRun
    );
}

#[test]
fn a_declared_gap_stays_a_gap_whatever_this_machine_says() {
    let definitions = real_definitions();
    let mut settings = approved(&definitions);
    settings.off.remove(&id("rrflow-routed"));
    settings
        .tuning
        .insert(id("rrflow-routed"), tuned(&[("ef", "64")]));
    let plan = build_plan(&definitions, &settings).unwrap();
    assert_eq!(
        state_of(&plan, "vector-small", "rrflow-routed"),
        SystemState::NotAvailable
    );
    let entry = test_named(&plan, "vector-small")
        .systems
        .iter()
        .find(|entry| entry.id.as_str() == "rrflow-routed")
        .unwrap();
    assert!(entry.reason.as_deref().unwrap().contains("No operation"));
}

#[test]
fn tuning_changes_only_that_system_and_its_report_says_so() {
    let definitions = real_definitions();
    let baseline = build_plan(&definitions, &approved(&definitions)).unwrap();
    let mut settings = approved(&definitions);
    settings
        .tuning
        .insert(id("qdrant"), tuned(&[("m", "32"), ("ef", "24,48")]));
    let plan = build_plan(&definitions, &settings).unwrap();

    let before = manifest_of(&baseline, "vector-small");
    let after = manifest_of(&plan, "vector-small");
    assert_eq!(test_sha256(after).unwrap(), test_sha256(before).unwrap());
    let qdrant = after.binding("qdrant").unwrap();
    assert_ne!(
        binding_sha256(qdrant),
        binding_sha256(before.binding("qdrant").unwrap()),
        "a tuned run cannot be combined with an untuned one as if they were the same"
    );
    assert_eq!(
        qdrant.index,
        IndexSpec::Hnsw {
            m: 32,
            ef_construction: 100
        }
    );
    assert_eq!(
        qdrant
            .search_ladder
            .iter()
            .map(|setting| (setting.id.as_str().to_owned(), setting.ef))
            .collect::<Vec<_>>(),
        vec![
            ("ef-24".to_owned(), Some(24)),
            ("ef-48".to_owned(), Some(48))
        ]
    );
    assert!(
        qdrant
            .limits
            .iter()
            .any(|limit| limit.contains("m=32") && limit.contains("ef=24,48")),
        "{:?}",
        qdrant.limits
    );
    assert_eq!(
        binding_sha256(after.binding("milvus").unwrap()),
        binding_sha256(before.binding("milvus").unwrap())
    );
    let listed = test_named(&plan, "vector-small")
        .systems
        .iter()
        .find(|entry| entry.id.as_str() == "qdrant")
        .unwrap();
    assert_eq!(listed.tuned, vec!["m=32".to_owned(), "ef=24,48".to_owned()]);
}

#[test]
fn a_tuned_ladder_keeps_the_shape_its_index_needs() {
    let definitions = real_definitions();
    let mut settings = approved(&definitions);
    settings
        .tuning
        .insert(id("rrflow-turboquant"), tuned(&[("ef", "80")]));
    settings
        .tuning
        .insert(id("lance-ivf-rq"), tuned(&[("nprobes", "7")]));
    let plan = build_plan(&definitions, &settings).unwrap();
    let manifest = manifest_of(&plan, "vector-100k");

    let turbo = &manifest.binding("rrflow-turboquant").unwrap().search_ladder[0];
    assert_eq!(turbo.id.as_str(), "rerank-80");
    assert_eq!((turbo.ef, turbo.exact_rerank), (Some(80), Some(80)));
    let lance = &manifest.binding("lance-ivf-rq").unwrap().search_ladder[0];
    assert_eq!(lance.id.as_str(), "nprobes-7");
    assert_eq!((lance.nprobes, lance.exact_rerank), (Some(7), Some(100)));
}

#[test]
fn engine_settings_are_added_for_rrflow_and_refused_elsewhere() {
    let definitions = real_definitions();
    let mut settings = approved(&definitions);
    settings.tuning.insert(
        id("rrflow-exact"),
        tuned(&[("engine.source_page_bytes", "1048576")]),
    );
    let plan = build_plan(&definitions, &settings).unwrap();
    let binding = manifest_of(&plan, "vector-small")
        .binding("rrflow-exact")
        .unwrap();
    assert_eq!(binding.engine_settings["source_page_bytes"], 1_048_576);

    settings.tuning.clear();
    settings.tuning.insert(
        id("qdrant"),
        tuned(&[("engine.source_page_bytes", "1048576")]),
    );
    let plan = build_plan(&definitions, &settings).unwrap();
    let problem = test_named(&plan, "vector-small")
        .problem
        .as_deref()
        .unwrap();
    assert!(problem.contains("qdrant"), "{problem}");
}

#[test]
fn a_knob_the_index_does_not_have_is_a_problem_for_that_test() {
    let definitions = real_definitions();
    let mut settings = approved(&definitions);
    settings
        .tuning
        .insert(id("rrflow-exact"), tuned(&[("m", "32")]));
    let plan = build_plan(&definitions, &settings).unwrap();
    let test = test_named(&plan, "vector-small");
    let problem = test.problem.as_deref().unwrap();
    assert!(
        problem.contains("rrflow-exact") && problem.contains("m"),
        "{problem}"
    );
    assert!(
        test.manifest.is_none(),
        "a test with a problem has nothing to run"
    );
    assert!(planned_files(&definitions, &settings, &plan, "vector-small").is_err());
}

#[test]
fn tuning_a_test_cannot_accept_is_a_problem_and_not_a_crash() {
    let definitions = real_definitions();
    let mut settings = approved(&definitions);
    // Every test here has k of at least ten, and an ef below k is refused.
    settings.tuning.insert(id("qdrant"), tuned(&[("ef", "2")]));
    let plan = build_plan(&definitions, &settings).unwrap();
    let problem = test_named(&plan, "vector-small")
        .problem
        .as_deref()
        .unwrap();
    assert!(
        problem.contains("qdrant") && problem.contains("ef"),
        "{problem}"
    );
}

#[test]
fn tuning_refuses_a_name_or_value_it_cannot_read() {
    let mut tuning = SystemTuning::default();
    for (name, value) in [
        ("nonsense", "1"),
        ("m", "many"),
        ("ef", ""),
        ("ef", "8,x"),
        ("engine.", "1"),
        ("engine.Bad-Name", "1"),
        ("engine.source_page_bytes", "-1"),
    ] {
        assert!(tuning.set(name, value).is_err(), "{name}={value}");
    }
    assert_eq!(tuning, SystemTuning::default());
}

#[test]
fn a_test_turned_off_is_listed_and_not_planned() {
    let definitions = real_definitions();
    let mut settings = approved(&definitions);
    settings.off.insert(id("vector-100k"));
    let plan = build_plan(&definitions, &settings).unwrap();
    let test = test_named(&plan, "vector-100k");
    assert!(!test.on);
    assert!(test.manifest.is_none());
    assert!(test_named(&plan, "vector-small").on);
    assert!(planned_files(&definitions, &settings, &plan, "vector-100k").is_err());
}

#[test]
fn settings_survive_a_round_trip_and_refuse_what_they_do_not_know() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("settings.json");
    assert_eq!(LabSettings::load(&path).unwrap(), LabSettings::default());
    let mut settings = LabSettings::default();
    settings.off.insert(id("qdrant"));
    settings.tuning.insert(id("milvus"), tuned(&[("m", "32")]));
    settings.save(&path).unwrap();
    assert_eq!(LabSettings::load(&path).unwrap(), settings);

    std::fs::write(&path, r#"{"schema_version":1,"surprise":true}"#).unwrap();
    assert!(LabSettings::load(&path).is_err());
    std::fs::write(&path, r#"{"schema_version":99}"#).unwrap();
    assert!(LabSettings::load(&path).unwrap_err().contains("newer"));
}

#[test]
fn a_new_pack_keeps_this_machines_choices_and_only_a_changed_launch_is_held() {
    let definitions = real_definitions();
    let mut settings = approved(&definitions);
    settings.off.insert(id("milvus"));
    let mut next = definitions.clone();
    let system = next.systems.get_mut(&id("surrealdb-4")).unwrap();
    let mut value = serde_json::to_value(&system.launch).unwrap();
    value["image"] = serde_json::json!(format!("surrealdb/surrealdb@sha256:{}", "b".repeat(64)));
    system.launch = serde_json::from_value(value).unwrap();
    let plan = build_plan(&next, &settings).unwrap();
    assert_eq!(
        state_of(&plan, "vector-small", "milvus"),
        SystemState::TurnedOff
    );
    assert_eq!(
        state_of(&plan, "vector-small", "surrealdb-4"),
        SystemState::NeedsApproval
    );
    assert_eq!(
        state_of(&plan, "vector-small", "qdrant"),
        SystemState::WillRun
    );
}

#[test]
fn rrflows_pinned_keys_are_valid() {
    let pinned = pinned_keys().unwrap();
    for key in &pinned.keys {
        assert_eq!(
            key.checked_id().unwrap(),
            key.id,
            "a pinned key is named for itself"
        );
    }
    let root = tempfile::tempdir().unwrap();
    let machine = DefinitionHome::at(root.path().join("machine"), RRFLOW_PRODUCT);
    assert_eq!(machine.trusted_keys(&pinned).unwrap(), pinned);
    assert_eq!(RrflowContent.product(), RRFLOW_PRODUCT);
}

// The real binary, on a copy of the lab's real definitions.

struct Machine {
    home: PathBuf,
}

impl Machine {
    fn new(root: &Path, name: &str) -> Self {
        Self {
            home: root.join(name),
        }
    }

    fn run_in(&self, folder: &Path, arguments: &[&str]) -> (bool, String, String) {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rrflow-lab"))
            .args(arguments)
            .current_dir(folder)
            .env("RRFLOW_LAB_HOME", &self.home)
            .output()
            .unwrap();
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    fn ok(&self, folder: &Path, arguments: &[&str]) -> String {
        let (ok, out, error) = self.run_in(folder, arguments);
        assert!(ok, "{arguments:?}: {error}");
        out
    }
}

/// A checkout's `eval/lab`, with only what a pack carries.
fn checkout(root: &Path) -> PathBuf {
    let repository = root.join("repository");
    for (relative, text) in lab_files(&real_lab()).unwrap() {
        let target = repository.join("eval/lab").join(relative);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, text).unwrap();
    }
    repository
}

#[test]
fn publish_finds_the_lab_from_anywhere_in_a_checkout_and_says_what_happened() {
    let root = tempfile::tempdir().unwrap();
    let repository = checkout(root.path());
    let nested = repository.join("some/deep/folder");
    std::fs::create_dir_all(&nested).unwrap();
    let publisher = Machine::new(root.path(), "publisher");

    let first = publisher.ok(&nested, &["definitions", "publish"]);
    assert!(first.contains("pack 1"), "{first}");
    assert!(
        first.contains("signing key"),
        "it must say a key was made: {first}"
    );
    assert!(
        first.contains("Back"),
        "it must say to back the key up: {first}"
    );
    let key = std::fs::read_to_string(publisher.home.join("publisher/signing.key")).unwrap();
    assert!(!first.contains(key.trim()), "the private key was printed");
    assert!(
        !snapshot(&publisher.home.join("outbox"))
            .values()
            .any(|bytes| String::from_utf8_lossy(bytes).contains(key.trim())),
        "the private key reached the outbox"
    );

    let again = publisher.ok(&nested, &["definitions", "publish"]);
    assert!(again.contains("Nothing changed"), "{again}");
    assert!(!again.contains("signing key"), "{again}");

    let small = repository.join("eval/lab/tests/vector-small.json");
    let text = std::fs::read_to_string(&small).unwrap();
    std::fs::write(
        &small,
        text.replacen("\"revision\": 1", "\"revision\": 2", 1),
    )
    .unwrap();
    let second = publisher.ok(&nested, &["definitions", "publish"]);
    assert!(second.contains("pack 2"), "{second}");
    assert!(second.contains("tests/vector-small.json"), "{second}");
}

#[test]
fn an_update_says_only_what_would_really_start_and_in_words() {
    let root = tempfile::tempdir().unwrap();
    let repository = checkout(root.path());
    let publisher = Machine::new(root.path(), "publisher");
    let out = publisher.ok(&repository, &["definitions", "publish"]);
    assert!(
        out.contains("new system qdrant would start container qdrant/qdrant@sha256:"),
        "{out}"
    );
    assert!(
        !out.contains("rrflow-routed"),
        "a declared gap starts nothing: {out}"
    );
    assert!(
        !out.contains("rrflow-exact"),
        "the binary under test is not new: {out}"
    );
    assert!(
        !out.contains("\"kind\""),
        "a person should not read raw JSON: {out}"
    );
    assert!(out.contains("90 days"), "{out}");
    let again = publisher.ok(&repository, &["definitions", "publish"]);
    assert!(again.contains("more days"), "{again}");
    assert!(!again.contains("days more"), "{again}");

    // A launch that changes shows every detail, since any of them may be the difference.
    let path = repository.join("eval/lab/systems/qdrant.json");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replacen("\"seconds\": 60", "\"seconds\": 61", 1),
    )
    .unwrap();
    let out = publisher.ok(&repository, &["definitions", "publish"]);
    assert!(
        out.contains("qdrant now starts something different"),
        "{out}"
    );
    assert!(out.contains("\"seconds\":61"), "{out}");
}

#[test]
fn a_plan_says_what_a_run_still_needs() {
    let root = tempfile::tempdir().unwrap();
    let repository = checkout(root.path());
    let publisher = Machine::new(root.path(), "publisher");
    publisher.ok(&repository, &["definitions", "publish"]);
    let plan: serde_json::Value = serde_json::from_str(&publisher.ok(
        root.path(),
        &["plan", "--test", "vector-coco-i2i", "--json"],
    ))
    .unwrap();
    assert_eq!(plan["tests"][0]["needs"][0], "RRFLOW_LAB_CORPUS_DIR");
    let needs = |name: &str| {
        plan["tests"][0]["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == name)
            .unwrap()["needs"][0]
            .clone()
    };
    assert_eq!(needs("lance-hnsw-sq"), "RRFLOW_LAB_LANCE_PYTHON");
    assert_eq!(needs("rrflow-exact"), "RRFLOW_BIN");
}

#[test]
fn publish_outside_a_checkout_says_what_to_do() {
    let root = tempfile::tempdir().unwrap();
    let publisher = Machine::new(root.path(), "publisher");
    let (ok, _, error) = publisher.run_in(root.path(), &["definitions", "publish"]);
    assert!(!ok);
    assert!(error.contains("--lab"), "{error}");
}

#[test]
fn status_on_a_new_machine_says_what_to_do_next() {
    let root = tempfile::tempdir().unwrap();
    let machine = Machine::new(root.path(), "machine");
    let out = machine.ok(root.path(), &["definitions", "status"]);
    assert!(out.contains("No test definitions"), "{out}");
    assert!(out.contains("definitions publish"), "{out}");
    assert!(out.contains("definitions update"), "{out}");
    let (ok, _, error) = machine.run_in(root.path(), &["plan"]);
    assert!(!ok);
    assert!(error.contains("definitions update"), "{error}");
    let out = machine.ok(root.path(), &["settings", "--json"]);
    assert!(out.contains("schema_version"), "{out}");
    assert!(!machine.home.exists(), "reading must leave nothing behind");
}

#[test]
fn a_second_machine_updates_trusts_approves_and_plans() {
    let root = tempfile::tempdir().unwrap();
    let repository = checkout(root.path());
    let publisher = Machine::new(root.path(), "publisher");
    let machine = Machine::new(root.path(), "machine");
    publisher.ok(&repository, &["definitions", "publish"]);
    let outbox = publisher.home.join("outbox");
    let outbox_text = outbox.to_str().unwrap();

    // An unknown publisher: the folder is read and nothing from it is used.
    let refused = machine.ok(
        root.path(),
        &["definitions", "update", "--from", outbox_text],
    );
    assert!(refused.contains("not trusted"), "{refused}");
    assert!(refused.contains("definitions trust"), "{refused}");

    let trust = outbox.join("rrflow-trusted-keys.json");
    let out = machine.ok(
        root.path(),
        &["definitions", "trust", "--file", trust.to_str().unwrap()],
    );
    assert!(out.contains("Now trusting"), "{out}");
    let out = machine.ok(
        root.path(),
        &["definitions", "update", "--from", outbox_text],
    );
    assert!(out.contains("pack 1"), "{out}");
    assert!(out.contains("waiting for your approval"), "{out}");

    // The folder is remembered, so the next update needs no argument.
    let out = machine.ok(root.path(), &["definitions", "update"]);
    assert!(out.contains("Already up to date"), "{out}");

    let status = machine.ok(root.path(), &["definitions", "status"]);
    assert!(status.contains("pack 1"), "{status}");
    assert!(status.contains("settings approve"), "{status}");
    let facts: serde_json::Value =
        serde_json::from_str(&machine.ok(root.path(), &["definitions", "status", "--json"]))
            .unwrap();
    assert_eq!(facts["current_sequence"], 1);
    assert!(facts["waiting_for_approval"].as_array().unwrap().len() > 1);

    let plan = machine.ok(root.path(), &["plan", "--test", "vector-small"]);
    assert!(plan.contains("Waiting for approval"), "{plan}");
    assert!(
        plan.contains("qdrant/qdrant@sha256:"),
        "a person sees what would start: {plan}"
    );

    machine.ok(root.path(), &["settings", "approve", "qdrant"]);
    let plan: serde_json::Value = serde_json::from_str(
        &machine.ok(root.path(), &["plan", "--test", "vector-small", "--json"]),
    )
    .unwrap();
    let state = |name: &str| {
        plan["tests"][0]["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == name)
            .unwrap()["state"]
            .clone()
    };
    assert_eq!(state("qdrant"), "will_run");
    assert_eq!(state("milvus"), "needs_approval");

    machine.ok(root.path(), &["settings", "approve", "--all"]);
    let status = machine.ok(root.path(), &["definitions", "status"]);
    assert!(!status.contains("settings approve"), "{status}");
}

#[test]
fn toggles_and_tuning_work_with_no_file_to_edit() {
    let root = tempfile::tempdir().unwrap();
    let repository = checkout(root.path());
    let publisher = Machine::new(root.path(), "publisher");
    publisher.ok(&repository, &["definitions", "publish"]);
    let at = root.path();

    publisher.ok(at, &["settings", "off", "milvus"]);
    publisher.ok(at, &["settings", "off", "vector-100k"]);
    publisher.ok(at, &["settings", "tune", "qdrant", "m=32", "ef=24,48"]);
    let shown = publisher.ok(at, &["settings"]);
    assert!(shown.contains("milvus"), "{shown}");
    assert!(shown.contains("m=32"), "{shown}");
    let line = |name: &str| {
        shown
            .lines()
            .find(|line| line.split_whitespace().nth(1) == Some(name))
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned()
    };
    assert_eq!(
        line("custom-reference"),
        "on",
        "tests run it, as a plan says"
    );
    assert_eq!(line("rrflow-routed"), "n/a");
    assert_eq!(line("milvus"), "off");

    let plan = publisher.ok(at, &["plan", "--test", "vector-small"]);
    assert!(plan.contains("Turned off"), "{plan}");
    assert!(plan.contains("Tuned"), "{plan}");

    let (ok, _, error) = publisher.run_in(at, &["settings", "off", "nobody"]);
    assert!(!ok);
    assert!(error.contains("nobody"), "{error}");
    let (ok, _, error) = publisher.run_in(at, &["settings", "tune", "qdrant", "bogus=1"]);
    assert!(!ok);
    assert!(error.contains("bogus"), "{error}");
    let (ok, _, error) = publisher.run_in(at, &["settings", "tune", "nobody", "m=1"]);
    assert!(!ok);
    assert!(error.contains("nobody"), "{error}");

    publisher.ok(at, &["settings", "on", "milvus"]);
    publisher.ok(at, &["settings", "reset", "qdrant"]);
    let shown = publisher.ok(at, &["settings"]);
    assert!(!shown.contains("m=32"), "{shown}");
    publisher.ok(at, &["settings", "reset"]);
    let plan = publisher.ok(at, &["plan", "--test", "vector-100k"]);
    assert!(!plan.contains("Turned off"), "{plan}");
    assert!(
        publisher
            .ok(at, &["plan", "--test", "vector-small"])
            .contains("Will run"),
        "reset keeps what publishing approved"
    );
}

#[test]
fn reading_commands_change_nothing() {
    let root = tempfile::tempdir().unwrap();
    let repository = checkout(root.path());
    let publisher = Machine::new(root.path(), "publisher");
    publisher.ok(&repository, &["definitions", "publish"]);
    publisher.ok(root.path(), &["settings", "off", "milvus"]);
    let before = snapshot(&publisher.home);
    for arguments in [
        vec!["definitions", "status"],
        vec!["definitions", "status", "--json"],
        vec!["settings"],
        vec!["settings", "--json"],
        vec!["plan"],
        vec!["plan", "--json"],
        vec!["plan", "--test", "vector-small"],
    ] {
        publisher.ok(root.path(), &arguments);
    }
    assert_eq!(snapshot(&publisher.home), before);
}

#[test]
fn a_planned_folder_holds_what_a_run_reads_and_names_its_pack() {
    let root = tempfile::tempdir().unwrap();
    let repository = checkout(root.path());
    let publisher = Machine::new(root.path(), "publisher");
    publisher.ok(&repository, &["definitions", "publish"]);
    publisher.ok(root.path(), &["settings", "off", "milvus"]);
    let out = root.path().join("run/definitions");
    let out_text = out.to_str().unwrap();
    publisher.ok(
        root.path(),
        &["plan", "--test", "vector-small", "--out", out_text],
    );

    let list = std::fs::read_to_string(out.join("run-list.txt")).unwrap();
    assert!(list.lines().any(|line| line == "qdrant"), "{list}");
    assert!(!list.lines().any(|line| line == "milvus"), "{list}");
    assert!(!list.lines().any(|line| line == "rrflow-routed"), "{list}");

    let identity = std::fs::read_to_string(out.join("definitions.env")).unwrap();
    assert!(identity.contains("sequence=1"), "{identity}");
    assert!(identity.contains("manifest_sha256="), "{identity}");

    // The real commands a run uses read what was written.
    let manifest = out.join("manifests/vector-small.json");
    let said = publisher.ok(
        root.path(),
        &["validate", "--manifest", manifest.to_str().unwrap()],
    );
    assert!(said.contains("valid"), "{said}");
    let info = publisher.ok(
        root.path(),
        &[
            "launch-info",
            "--system",
            out.join("systems/milvus.json").to_str().unwrap(),
        ],
    );
    assert_eq!(
        info.trim(),
        "kind=none",
        "a turned-off system has nothing to start"
    );

    let (ok, _, error) = publisher.run_in(
        root.path(),
        &["plan", "--test", "vector-small", "--out", out_text],
    );
    assert!(!ok, "evidence is never silently replaced");
    assert!(error.contains("already exists"), "{error}");
    let (ok, _, error) = publisher.run_in(root.path(), &["plan", "--out", out_text]);
    assert!(!ok);
    assert!(error.contains("--test"), "{error}");
}
