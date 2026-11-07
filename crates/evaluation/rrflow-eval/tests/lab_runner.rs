#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn executable(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn runner_removes_only_the_container_it_created() {
    for system in ["qdrant", "surrealdb", "milvus"] {
        for mode in ["failure", "invalid", "success"] {
            let temp = tempfile::tempdir().unwrap();
            let bin = temp.path().join("bin");
            fs::create_dir(&bin).unwrap();
            let log = temp.path().join("docker.log");
            executable(
                &bin.join("docker"),
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$LAB_FAKE_LOG\"\nif [ \"$1\" = run ] && [ \"$2\" = -d ]; then\n  case \"$LAB_FAKE_CREATE\" in\n    success) printf '%064d\\n' 0; exit 0 ;;\n    invalid) echo lab-existing; exit 0 ;;\n    failure) echo lab-existing; echo 'name already in use' >&2; exit 125 ;;\n  esac\nfi\nexit 0\n",
            );
            executable(
                &bin.join("curl"),
                "#!/bin/sh\nif [ ! -e \"$LAB_FAKE_CURL_STATE\" ]; then touch \"$LAB_FAKE_CURL_STATE\"; exit 7; fi\necho OK\n",
            );
            executable(&bin.join("lab"), "#!/bin/sh\nexit 7\n");
            let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
            let output = Command::new("bash")
                .arg(root.join("eval/lab/run_system.sh"))
                .arg(system)
                .arg(temp.path().join("results"))
                .env("PATH", path)
                .env("TMPDIR", temp.path())
                .env("RRFLOW_LAB_BIN", bin.join("lab"))
                .env("LAB_FAKE_LOG", &log)
                .env("LAB_FAKE_CURL_STATE", temp.path().join("curl.state"))
                .env("LAB_FAKE_CREATE", mode)
                .output()
                .unwrap();
            assert!(
                !output.status.success(),
                "fixture must terminate with a failure"
            );
            let calls = fs::read_to_string(&log).unwrap();
            let removes: Vec<_> = calls
                .lines()
                .filter(|line| line.starts_with("rm "))
                .collect();
            if mode == "success" {
                assert_eq!(
                    removes,
                    [format!("rm -f -v {}", "0".repeat(64))],
                    "{system}: {calls}"
                );
            } else {
                assert!(
                    removes.is_empty(),
                    "{system} removed an unowned container: {calls}"
                );
            }
        }
    }
}
