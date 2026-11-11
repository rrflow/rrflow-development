//! A system or a test is a file. These tests hold the files to what the lab always ran, so moving
//! to them changes nothing, and to what must never get into a file.
use rrflow_eval::lab::catalog::{
    launch_arguments, launch_info, resolve_test, vectordbbench_arguments, Pod, SystemDefinition,
    TestDefinition,
};
use rrflow_eval::lab::driver::binding_sha256;
use rrflow_eval::lab::manifest::VectorComparisonManifest;
use rrflow_eval::storage::Id;
use std::collections::BTreeMap;
use std::path::PathBuf;

const QDRANT: &str =
    "qdrant/qdrant@sha256:9bfc89f9914fc14876a88c05270685921a55b8877b254a98bcc1cc2f0a424cd2";
const MILVUS: &str =
    "milvusdb/milvus@sha256:b9b79b4a284b41cd12d60abc418310f84621fa4c977d409b38f501519a7ccf8a";
const SURREAL: &str =
    "surrealdb/surrealdb@sha256:3db515b88e79737aa828c05fbf1515748004ba57495ff89a013851cc3f1ad4b9";

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../eval/lab")
}

fn read(path: PathBuf) -> String {
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn system(name: &str) -> SystemDefinition {
    serde_json::from_str(&read(lab().join("systems").join(format!("{name}.json")))).unwrap()
}

fn all_systems() -> BTreeMap<Id, SystemDefinition> {
    let mut systems = BTreeMap::new();
    for entry in std::fs::read_dir(lab().join("systems")).unwrap() {
        let path = entry.unwrap().path();
        let definition: SystemDefinition = serde_json::from_str(&read(path.clone())).unwrap();
        definition.validate().unwrap();
        assert_eq!(
            path.file_stem().unwrap().to_str().unwrap(),
            definition.binding.id.as_str(),
            "a system's file is named for it"
        );
        systems.insert(definition.binding.id.clone(), definition);
    }
    systems
}

fn no_env(_: &str) -> Option<String> {
    None
}

fn arguments(definition: &SystemDefinition) -> Vec<String> {
    launch_arguments(definition, "/run/data", "/lab", Some("5-9"), None, &no_env).unwrap()
}

fn words(text: &[&str]) -> Vec<String> {
    text.iter().map(|word| (*word).to_string()).collect()
}

#[test]
fn every_checked_in_manifest_is_what_its_files_generate() {
    let systems = all_systems();
    let mut seen = 0;
    for entry in std::fs::read_dir(lab().join("tests")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let test: TestDefinition = serde_json::from_str(&read(path)).unwrap();
        let generated = resolve_test(&test, &systems).unwrap();
        let checked_in: VectorComparisonManifest =
            serde_json::from_str(&read(lab().join(&name))).unwrap();
        assert_eq!(
            generated, checked_in,
            "{name} is stale; run rrflow-lab resolve-all"
        );
        for (left, right) in generated.systems.iter().zip(&checked_in.systems) {
            assert_eq!(
                binding_sha256(left),
                binding_sha256(right),
                "a changed binding would orphan the runs already made"
            );
        }
        seen += 1;
    }
    assert_eq!(seen, 4);
}

#[test]
fn each_system_starts_with_the_arguments_the_script_always_used() {
    assert_eq!(
        arguments(&system("qdrant")),
        words(&[
            "--name",
            "lab-qdrant",
            "--cpuset-cpus",
            "5-9",
            "-e",
            "QDRANT__SERVICE__HTTP_PORT=16333",
            "-p",
            "127.0.0.1:16333:16333",
            "-v",
            "/run/data:/qdrant/storage",
            QDRANT,
        ])
    );
    assert_eq!(
        arguments(&system("milvus")),
        words(&[
            "--name",
            "lab-milvus",
            "--cpuset-cpus",
            "5-9",
            "-e",
            "COMMON_STORAGETYPE=local",
            "-e",
            "DEPLOY_MODE=STANDALONE",
            "-e",
            "ETCD_DATA_DIR=/var/lib/milvus/etcd",
            "-e",
            "ETCD_USE_EMBED=true",
            "-p",
            "127.0.0.1:19530:19530",
            "-p",
            "127.0.0.1:19091:9091",
            "-v",
            "/run/data:/var/lib/milvus",
            MILVUS,
            "milvus",
            "run",
            "standalone",
        ])
    );
    assert_eq!(
        arguments(&system("surrealdb")),
        words(&[
            "--name",
            "lab-surrealdb",
            "--cpuset-cpus",
            "5-9",
            "-e",
            "SURREAL_BIND=0.0.0.0:18000",
            "-e",
            "SURREAL_CAPS_ALLOW_ALL=true",
            "-e",
            "SURREAL_PASS",
            "-e",
            "SURREAL_USER",
            "-p",
            "127.0.0.1:18000:18000",
            "--user",
            "root",
            "-v",
            "/run/data:/data",
            SURREAL,
            "start",
            "surrealkv:///data/lab",
        ]),
        "the password goes by name, so it is not on the host's process list"
    );
    assert_eq!(
        arguments(&system("custom-reference")),
        words(&["python3", "/lab/reference_custom_system.py", "18089"])
    );
    let python =
        |name: &str| (name == "RRFLOW_LAB_LANCE_PYTHON").then(|| "/venv/python".to_string());
    assert_eq!(
        launch_arguments(
            &system("lance-hnsw-sq"),
            "/run/data",
            "/lab",
            None,
            None,
            &python
        )
        .unwrap(),
        words(&["/venv/python", "/lab/lance_system.py", "18090", "/run/data"])
    );
    assert_eq!(arguments(&system("rrflow-exact")), Vec::<String>::new());
    assert_eq!(arguments(&system("rrflow-routed")), Vec::<String>::new());
}

#[test]
fn a_pod_member_publishes_nothing_and_keeps_its_data_in_the_run_volume() {
    let pod = Some(Pod {
        container: "lab-pod-7",
        volume: "lab-pod-7-qdrant-a1",
    });
    assert_eq!(
        launch_arguments(&system("qdrant"), "/run/data", "/lab", None, pod, &no_env).unwrap(),
        words(&[
            "--name",
            "lab-pod-7-lab-qdrant",
            "-e",
            "QDRANT__SERVICE__HTTP_PORT=16333",
            "--network",
            "container:lab-pod-7",
            "--mount",
            "type=volume,src=lab-pod-7-qdrant-a1,dst=/qdrant/storage",
            QDRANT,
        ]),
        "no host port, no host folder, and a name no other run uses"
    );
    let refused =
        launch_arguments(&system("milvus"), "/run/data", "/lab", None, pod, &no_env).unwrap_err();
    assert!(
        refused.contains("listens on 9091 but is reached on 19091"),
        "a pod maps no ports, so a remapped port is refused with its reason: {refused}"
    );
    assert_eq!(
        launch_arguments(
            &system("custom-reference"),
            "/run/data",
            "/lab",
            None,
            pod,
            &no_env
        )
        .unwrap(),
        words(&["python3", "/lab/reference_custom_system.py", "18089"]),
        "a process already runs inside the pod"
    );
}

#[test]
fn the_script_is_told_how_to_start_and_wait_without_any_secret() {
    let info: BTreeMap<String, String> = launch_info(&system("surrealdb")).into_iter().collect();
    assert_eq!(info["kind"], "container");
    assert_eq!(info["container"], "lab-surrealdb");
    assert_eq!(info["ports"], "18000");
    assert_eq!(info["secrets"], "SURREAL_PASS=SURREAL_LAB_SECRET");
    assert_eq!(info["host_env"], "SURREAL_LAB_USER=lab");
    assert_eq!(info["env_from_host"], "SURREAL_USER=SURREAL_LAB_USER");
    assert_eq!(info["ready_url"], "http://127.0.0.1:18000/version");

    let info: BTreeMap<String, String> = launch_info(&system("milvus")).into_iter().collect();
    assert_eq!(info["ports"], "19530 19091");
    assert_eq!(info["ready_contains"], "OK");
    assert_eq!(info["ready_seconds"], "180");

    let info: BTreeMap<String, String> = launch_info(&system("lance-ivf-rq")).into_iter().collect();
    assert_eq!(info["kind"], "process");
    assert_eq!(info["pid_file"], "shim.pid");
    assert_eq!(info["embedded_under_vectordbbench"], "true");

    let info: BTreeMap<String, String> =
        launch_info(&system("rrflow-routed")).into_iter().collect();
    assert_eq!(info["kind"], "none", "a declared gap starts nothing");
    let info: BTreeMap<String, String> = launch_info(&system("rrflow-exact")).into_iter().collect();
    assert_eq!(info["kind"], "rrflow_engine");
    assert_eq!(info["ports"], "19477");
}

#[test]
fn a_shim_that_needs_a_variable_says_so() {
    let refused = launch_arguments(
        &system("lance-hnsw-sq"),
        "/run/data",
        "/lab",
        None,
        None,
        &no_env,
    )
    .unwrap_err();
    assert!(refused.contains("RRFLOW_LAB_LANCE_PYTHON"), "{refused}");
}

#[test]
fn vectordbbench_arguments_come_from_the_systems_file() {
    assert_eq!(
        vectordbbench_arguments(&system("qdrant"), "/run/data", "128").unwrap(),
        words(&[
            "qdrantlocal",
            "--url",
            "http://127.0.0.1:16333",
            "--m",
            "16",
            "--ef-construct",
            "100",
            "--hnsw-ef",
            "128",
        ])
    );
    assert_eq!(
        vectordbbench_arguments(&system("lance-hnsw-sq"), "/run/data", "64").unwrap(),
        words(&[
            "lancedbivfhnswsq",
            "--uri",
            "/run/data/vdbbench",
            "--num-partitions",
            "1",
            "--m",
            "16",
            "--ef-construction",
            "100",
            "--ef",
            "64",
        ])
    );
    let refused = vectordbbench_arguments(&system("surrealdb"), "/run/data", "64").unwrap_err();
    assert!(refused.contains("no VectorDBBench client"), "{refused}");
}

fn changed(name: &str, change: impl Fn(&mut serde_json::Value)) -> SystemDefinition {
    let mut value: serde_json::Value =
        serde_json::from_str(&read(lab().join("systems").join(format!("{name}.json")))).unwrap();
    change(&mut value);
    serde_json::from_value(value).unwrap()
}

#[test]
fn a_file_cannot_hold_what_would_make_a_result_misleading_or_unsafe() {
    let cases: [(&str, SystemDefinition, &str); 6] = [
        (
            "a floating image",
            changed("qdrant", |value| {
                value["launch"]["image"] = "qdrant/qdrant:latest".into()
            }),
            "pinned by digest",
        ),
        (
            "a port beyond this machine",
            changed("qdrant", |value| {
                value["launch"]["ports"] = serde_json::json!(["0.0.0.0:16333:6333"])
            }),
            "loopback",
        ),
        (
            "a port with no host",
            changed("qdrant", |value| {
                value["launch"]["ports"] = serde_json::json!(["16333:6333"])
            }),
            "loopback",
        ),
        (
            "a password in a file",
            changed("surrealdb", |value| {
                value["launch"]["env"]["SURREAL_PASS"] = "hunter2".into()
            }),
            "generate it for the run",
        ),
        (
            "a readiness check elsewhere",
            changed("qdrant", |value| {
                value["launch"]["ready"]["url"] = "http://example.org/".into()
            }),
            "this machine",
        ),
        (
            "a runnable system that does not say how to start",
            changed("qdrant", |value| {
                value["launch"] = serde_json::json!({"kind": "none"})
            }),
            "how to start",
        ),
    ];
    for (name, definition, expected) in cases {
        let refused = definition.validate().unwrap_err();
        assert!(refused.contains(expected), "{name}: {refused}");
    }
    let mut value: serde_json::Value =
        serde_json::from_str(&read(lab().join("systems/qdrant.json"))).unwrap();
    value["launch"]["surprise"] = true.into();
    assert!(
        serde_json::from_value::<SystemDefinition>(value).is_err(),
        "an unknown field is refused, so a secret cannot be smuggled in"
    );
}

#[test]
fn a_test_can_only_change_what_it_owns_and_name_what_exists() {
    let systems = all_systems();
    let mut value: serde_json::Value =
        serde_json::from_str(&read(lab().join("tests/vector-small.json"))).unwrap();
    value["systems"][0]["artifact"] = "something else".into();
    assert!(
        serde_json::from_value::<TestDefinition>(value).is_err(),
        "a pin has one owner, the system's file"
    );

    let mut test: TestDefinition =
        serde_json::from_str(&read(lab().join("tests/vector-small.json"))).unwrap();
    test.systems[0].system = Id::try_from("nobody".to_string()).unwrap();
    let refused = resolve_test(&test, &systems).unwrap_err();
    assert!(
        refused.contains("eval/lab/systems/nobody.json"),
        "{refused}"
    );

    let mut twice: TestDefinition =
        serde_json::from_str(&read(lab().join("tests/vector-small.json"))).unwrap();
    twice.systems.push(twice.systems[0].clone());
    assert!(resolve_test(&twice, &systems)
        .unwrap_err()
        .contains("listed twice"));
}

#[test]
fn a_new_system_takes_one_file_and_no_code() {
    let mut systems = all_systems();
    let new_system: SystemDefinition = serde_json::from_str(
        r#"{
          "schema_version": 1,
          "binding": {
            "id": "brand-new", "label": "A brand new system", "kind": "custom", "location": "loopback",
            "endpoint": "http://127.0.0.1:18095", "artifact": "brand-new 1.0.0, behind its shim",
            "storage_profile": "local disk", "availability": {"state": "available"},
            "index": {"index": "exact"}, "search_ladder": [{"id": "default"}]
          },
          "launch": {
            "kind": "process", "command": ["python3", "{lab}/brand_new.py", "18095", "{data}"], "port": 18095,
            "ready": {"url": "http://127.0.0.1:18095/health", "seconds": 30}
          }
        }"#,
    )
    .unwrap();
    new_system.validate().unwrap();
    assert_eq!(
        launch_arguments(&new_system, "/run/data", "/lab", None, None, &no_env).unwrap(),
        words(&["python3", "/lab/brand_new.py", "18095", "/run/data"])
    );
    systems.insert(new_system.binding.id.clone(), new_system);
    let mut test: TestDefinition =
        serde_json::from_str(&read(lab().join("tests/vector-small.json"))).unwrap();
    test.systems
        .push(serde_json::from_str(r#"{"system": "brand-new"}"#).unwrap());
    let manifest = resolve_test(&test, &systems).unwrap();
    assert!(manifest
        .systems
        .iter()
        .any(|binding| binding.id.as_str() == "brand-new"));
    assert_eq!(
        manifest.test, test.test,
        "adding a system keeps the test, so runs already made still combine"
    );
}
