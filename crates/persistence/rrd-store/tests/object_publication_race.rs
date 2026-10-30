use rrd_core::{digest, ObjectReference};
use rrd_store::{LocalObjectStore, ObjectStep};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
#[ignore = "child process for concurrent_object_publication_across_processes"]
fn concurrent_object_writer_child() {
    let root = std::env::var_os("RRFLOW_OBJECT_RACE_ROOT").unwrap();
    let gate = std::env::var_os("RRFLOW_OBJECT_RACE_GATE").unwrap();
    let actor = std::env::var("RRFLOW_OBJECT_RACE_ACTOR").unwrap();
    let payload = std::env::var("RRFLOW_OBJECT_RACE_PAYLOAD").unwrap();
    let gate = Path::new(&gate);
    let store = LocalObjectStore::open(root).unwrap();
    store
        .put_with_hook(payload.as_bytes(), |step| {
            if step == ObjectStep::AfterStageSync {
                std::fs::write(gate.join(&actor), b"staged")?;
                let deadline = Instant::now() + Duration::from_secs(10);
                while !gate.join("release").exists() {
                    if Instant::now() >= deadline {
                        return Err(rrd_store::Error::Object("process gate timed out".into()));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn concurrent_object_publication_across_processes() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("objects");
    let store = LocalObjectStore::open(&root).unwrap();
    let executable = std::env::current_exe().unwrap();

    for iteration in 0..8 {
        let gate = directory.path().join(format!("gate-{iteration}"));
        std::fs::create_dir(&gate).unwrap();
        let payload = format!("cross-process-object-{iteration}");
        let payloads = [
            payload.clone(),
            if iteration % 2 == 0 {
                payload
            } else {
                format!("cross-process-distinct-{iteration}")
            },
        ];
        let mut children = (0..2)
            .map(|actor| {
                Command::new(&executable)
                    .args([
                        "--exact",
                        "concurrent_object_writer_child",
                        "--ignored",
                        "--nocapture",
                    ])
                    .env("RRFLOW_OBJECT_RACE_ROOT", &root)
                    .env("RRFLOW_OBJECT_RACE_GATE", &gate)
                    .env("RRFLOW_OBJECT_RACE_ACTOR", actor.to_string())
                    .env("RRFLOW_OBJECT_RACE_PAYLOAD", &payloads[actor])
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !(gate.join("0").exists() && gate.join("1").exists()) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        let both_staged = gate.join("0").exists() && gate.join("1").exists();
        std::fs::write(gate.join("release"), b"release").unwrap();
        for child in &mut children {
            let deadline = Instant::now() + Duration::from_secs(15);
            while child.try_wait().unwrap().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
            }
            if child.try_wait().unwrap().is_none() {
                child.kill().unwrap();
            }
        }
        for child in children {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "child failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        assert!(
            both_staged,
            "both child processes must reach the synchronized stage"
        );

        for payload in payloads {
            let sha = digest::sha256_hex(payload.as_bytes());
            let verified = store.verify(&sha).unwrap();
            let reference = ObjectReference::for_bytes(
                "cross-process-object",
                None,
                "application/octet-stream",
                payload.as_bytes(),
                verified.receipt,
            )
            .unwrap();
            assert_eq!(store.get(&reference).unwrap(), payload.as_bytes());
        }
    }
    assert!(store
        .inventory(&BTreeSet::new())
        .unwrap()
        .staging_files
        .is_empty());
}
