//! The per-user instance registry, `rrflow instances` and `rrflow stop`.

use super::*;

struct Installed {
    project: PathBuf,
    instance_id: String,
    plan_file: PathBuf,
    plan_sha256: String,
}

impl Installed {
    fn project_arg(&self) -> &str {
        self.project.to_str().unwrap()
    }

    fn apply_args<'a>(&'a self, registry: &'a str) -> [&'a str; 11] {
        [
            "--instance-registry",
            registry,
            "install",
            "apply",
            "--mode",
            "existing",
            "--project",
            self.project_arg(),
            "--plan",
            self.plan_file.to_str().unwrap(),
            "--expect",
        ]
    }
}

/// Plans an installation of a new project below `root` without applying it.
fn plan_project(root: &Path, name: &str) -> Installed {
    let project = root.join(name);
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("README.md"), format!("{name}\n")).unwrap();
    let identity = invoke(&["install", "identity"]);
    assert_success(&identity, "allocate installation identity");
    let identity: serde_json::Value = serde_json::from_slice(&identity.stdout).unwrap();
    let plan = invoke(&[
        "install",
        "plan",
        "--identity",
        identity["installation_identity"].as_str().unwrap(),
        "--mode",
        "existing",
        "--project",
        project.to_str().unwrap(),
    ]);
    assert_success(&plan, "install plan");
    let plan_file = root.join(format!("{name}-install-plan.json"));
    std::fs::write(&plan_file, &plan.stdout).unwrap();
    let preview: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    Installed {
        project,
        instance_id: preview["installation"]["target"]["instance_id"]
            .as_str()
            .unwrap()
            .to_owned(),
        plan_file,
        plan_sha256: preview["installation"]["plan_sha256"]
            .as_str()
            .unwrap()
            .to_owned(),
    }
}

fn apply(installed: &Installed, registry: &str, json: bool) -> Output {
    let mut args = Vec::new();
    if json {
        args.push("--json");
    }
    args.extend(installed.apply_args(registry));
    args.push(&installed.plan_sha256);
    invoke(&args)
}

/// Proves that a server accepts connections and answers HTTP, without the
/// authenticated discovery `rrflow ready` runs.
fn answers_http(address: &str) -> bool {
    let mut stream = TcpStream::connect(address).expect("connect to the served address");
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    stream
        .write_all(b"GET /v1/health/ready HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    response.starts_with(b"HTTP/1.1 ")
}

/// The one registry entry of an instance, whatever its project root digest is.
fn entry_file(registry: &Path, instance_id: &str) -> PathBuf {
    let mut files: Vec<PathBuf> = std::fs::read_dir(registry)
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

fn canonical(project: &Path) -> String {
    std::fs::canonicalize(project)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

fn listed(registry: &str, captured: &mut Vec<u8>) -> serde_json::Value {
    let output = invoke(&["--json", "--instance-registry", registry, "instances"]);
    capture_output(&output, captured);
    assert_success(&output, "list instances");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn stop(project: &str, extra: &[&str], captured: &mut Vec<u8>) -> (Output, serde_json::Value) {
    let mut args = vec!["--json", "stop", "--project", project];
    args.extend_from_slice(extra);
    let output = invoke(&args);
    capture_output(&output, captured);
    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stop must answer with JSON: {error}: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output, report)
}

/// Install registers the instance, serve and a clean stop update it, and
/// `rrflow stop` ends a real `rrflow serve` the way an interrupt does, on
/// every platform.
#[test]
fn a_registered_instance_is_listed_served_and_stopped_cleanly() {
    let root = tempfile::tempdir().unwrap();
    let registry_path = root.path().join("state").join("instances");
    let registry = registry_path.to_str().unwrap();
    let installed = plan_project(root.path(), "project");
    let project = installed.project_arg();
    let record_path = installed.project.join(".rrflow").join("serving.json");
    let stop_request_path = installed.project.join(".rrflow").join("stop-request.json");
    let mut captured = Vec::new();
    let mut credential = None;

    assert!(!registry_path.exists());
    let empty = listed(registry, &mut captured);
    assert_eq!(empty["registry_present"], false);
    assert_eq!(empty["instances"], serde_json::json!([]));

    // Install writes the registry.
    let applied = apply(&installed, registry, false);
    capture_output(&applied, &mut captured);
    assert_success(&applied, "install apply");
    assert!(
        String::from_utf8_lossy(&applied.stdout)
            .contains(&format!("instance registry: {}", registry_path.display())),
        "install must name the registry it wrote"
    );
    let listing = listed(registry, &mut captured);
    assert_eq!(listing["registry_present"], true);
    assert_eq!(listing["unreadable"], serde_json::json!([]));
    assert_eq!(listing["instances"].as_array().unwrap().len(), 1);
    let instance = &listing["instances"][0];
    assert_eq!(instance["instance_id"], installed.instance_id.as_str());
    assert_eq!(instance["project_root"], canonical(&installed.project));
    assert_eq!(instance["product_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(instance["project_present"], true);
    assert_eq!(instance["locator_present"], true);
    assert_eq!(instance["serving_published"], false);
    assert_eq!(instance["serving_record_present"], false);
    assert_eq!(instance["endpoint"], serde_json::Value::Null);
    assert_eq!(instance["problems"], serde_json::json!([]));

    // Stopping a project that is not served is an answer, not a failure.
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop when nothing is served");
    assert_eq!(report["status"], "not_serving");
    assert_eq!(report["instance_id"], installed.instance_id.as_str());
    assert!(!stop_request_path.exists());

    // Serve registers the published record; the variable names the registry.
    let (mut served, address) =
        RunningEngine::start_with(&installed.project, &mut captured, |command| {
            command.env("RRFLOW_INSTANCE_REGISTRY", registry);
        });
    assert_eq!(
        served.announcement["instance_registry"]["status"],
        "recorded"
    );
    let listing = listed(registry, &mut captured);
    let instance = &listing["instances"][0];
    assert_eq!(instance["serving_published"], true);
    assert_eq!(instance["serving_record_present"], true);
    assert_eq!(instance["endpoint"], format!("http://{address}"));
    assert_eq!(instance["process_id"], served.child.id());
    let human = invoke(&["--instance-registry", registry, "instances"]);
    capture_output(&human, &mut captured);
    assert_success(&human, "list instances for a person");
    let human = String::from_utf8(human.stdout).unwrap();
    assert!(human.contains(&canonical(&installed.project)), "{human}");
    assert!(human.contains(&installed.instance_id), "{human}");
    assert!(human.contains(&format!("http://{address}")), "{human}");

    // A stop request ends the real server cleanly and waits until it has.
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop a served project");
    assert_eq!(report["status"], "stopped");
    assert_eq!(report["url"], format!("http://{address}"));
    assert_eq!(report["process_id"], served.child.id());
    assert_eq!(report["project_root"], canonical(&installed.project));
    assert!(
        !record_path.exists(),
        "a stopped server withdraws its serving record"
    );
    assert!(!stop_request_path.exists(), "the request is acknowledged");
    served.finish_stopped(&mut captured);
    let listing = listed(registry, &mut captured);
    let instance = &listing["instances"][0];
    assert_eq!(instance["serving_published"], false);
    assert_eq!(instance["serving_record_present"], false);
    assert_eq!(instance["endpoint"], serde_json::Value::Null);
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop after the server has stopped");
    assert_eq!(report["status"], "not_serving");

    // A server interrupted on its own closes its record with nothing asking,
    // and a stop that arrives afterwards finds nothing to stop.
    let (mut interrupted, _) =
        RunningEngine::start_with(&installed.project, &mut captured, |command| {
            command.env("RRFLOW_INSTANCE_REGISTRY", registry);
        });
    interrupted.stop(&mut captured);
    assert!(!record_path.exists());
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop after an interrupted server");
    assert_eq!(report["status"], "not_serving");

    // The engram is free again. A killed server never withdraws its record:
    // the stop is refused within its limit, and the record stays a candidate
    // that only `rrflow ready` can prove. The request stays as well. It names
    // the killed server's record, so it can stop nothing else.
    let (mut killed, replacement) =
        RunningEngine::start_with(&installed.project, &mut captured, |command| {
            command.env("RRFLOW_INSTANCE_REGISTRY", registry);
        });
    killed.terminate(&mut captured);
    assert!(record_path.exists());
    let (output, report) = stop(
        project,
        &["--wait-ms", "400", "--poll-interval-ms", "20"],
        &mut captured,
    );
    assert!(
        !output.status.success(),
        "a dead server acknowledged a stop"
    );
    assert_eq!(report["status"], "timed_out");
    assert_eq!(report["wait_limit_ms"], 400);
    assert!(stop_request_path.exists(), "a request is never taken back");
    let listing = listed(registry, &mut captured);
    let instance = &listing["instances"][0];
    assert_eq!(instance["serving_record_present"], true);
    assert_eq!(instance["endpoint"], format!("http://{replacement}"));

    // Replaying the install beside the record a killed server left succeeds,
    // and the registry keeps saying what the project shows.
    let replayed = apply(&installed, registry, true);
    capture_output(&replayed, &mut captured);
    assert_success(&replayed, "install replay beside a killed server's record");
    let result: serde_json::Value = serde_json::from_slice(&replayed.stdout).unwrap();
    assert_eq!(result["idempotent_replay"], true);
    assert_eq!(result["instance_registry"]["status"], "recorded");
    let listing = listed(registry, &mut captured);
    assert_eq!(listing["instances"][0]["serving_published"], true);

    // A stop still waiting on the killed server when another one starts is
    // told that the project is served again. It is never told that its stop
    // finished, and it does not stop the new server. The request left above
    // is discarded first, so that this stop's own publication can be seen.
    std::fs::remove_file(&stop_request_path).unwrap();
    let mut waiting = product_command();
    waiting.args([
        "--json",
        "stop",
        "--project",
        project,
        "--wait-ms",
        "120000",
    ]);
    let mut waiting =
        process::Process::start(&mut waiting, Duration::from_secs(180), OUTPUT_LIMIT, None)
            .expect("start a stop that waits on the killed server");
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    while !stop_request_path.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "the waiting stop never published its request"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let (mut successor, successor_address) =
        RunningEngine::start_with(&installed.project, &mut captured, |command| {
            command.env("RRFLOW_INSTANCE_REGISTRY", registry);
        });
    let output = waiting.finish().expect("reap the waiting stop");
    capture_output(&output, &mut captured);
    assert!(
        !output.status.success(),
        "a stop reported success while the project is served"
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "replaced");
    assert_eq!(report["url"], format!("http://{replacement}"));
    assert_eq!(
        report["replaced_by_url"],
        format!("http://{successor_address}")
    );
    assert_eq!(report["replaced_by_process_id"], successor.child.id());
    assert!(successor.child.running(), "the new server was not stopped");
    assert!(
        !stop_request_path.exists(),
        "a new server discards a request left for the one before it"
    );
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop the successor");
    assert_eq!(report["status"], "stopped");
    successor.finish_stopped(&mut captured);
    assert!(!record_path.exists());

    credential.replace(RrdEngine::read_installed_api_key(&installed.project).unwrap());

    // A removed project stays listed and is reported as missing.
    std::fs::remove_dir_all(&installed.project).unwrap();
    let listing = listed(registry, &mut captured);
    assert_eq!(listing["instances"].as_array().unwrap().len(), 1);
    let instance = &listing["instances"][0];
    assert_eq!(instance["instance_id"], installed.instance_id.as_str());
    assert_eq!(instance["project_present"], false);
    assert_eq!(
        instance["problems"],
        serde_json::json!(["the project folder no longer exists"])
    );

    let registered =
        std::fs::read_to_string(entry_file(&registry_path, &installed.instance_id)).unwrap();
    let credential = credential.unwrap();
    for (surface, text) in [
        ("command output", String::from_utf8_lossy(&captured)),
        ("the registry entry", registered.as_str().into()),
    ] {
        assert!(
            !text.contains(credential.credential()),
            "{surface} holds the installed credential"
        );
    }
}

/// The registry is a candidate list: when it cannot be written, install and
/// serve still succeed and say why it was not updated.
#[test]
fn install_and_serve_succeed_and_say_why_when_the_registry_cannot_be_written() {
    let root = tempfile::tempdir().unwrap();
    let blocker = root.path().join("not-a-directory");
    std::fs::write(&blocker, b"a file where a directory is needed").unwrap();
    let registry_path = blocker.join("instances");
    let registry = registry_path.to_str().unwrap();
    let installed = plan_project(root.path(), "project");
    let mut captured = Vec::new();

    let applied = apply(&installed, registry, false);
    capture_output(&applied, &mut captured);
    assert_success(&applied, "install apply without a writable registry");
    let text = String::from_utf8_lossy(&applied.stdout);
    assert!(text.contains("RRFlow installed:"), "{text}");
    assert!(text.contains("instance registry: not updated"), "{text}");

    // The JSON answer is the installation result with the registry outcome
    // beside it, where a client reads it; nothing is left for stderr to say.
    let replayed = apply(&installed, registry, true);
    capture_output(&replayed, &mut captured);
    assert_success(&replayed, "install replay without a writable registry");
    let result: serde_json::Value = serde_json::from_slice(&replayed.stdout).unwrap();
    assert_eq!(result["idempotent_replay"], true);
    assert_eq!(result["plan_sha256"], installed.plan_sha256.as_str());
    assert_eq!(result["instance_registry"]["status"], "not_updated");
    assert!(!result["instance_registry"]["reason"]
        .as_str()
        .unwrap()
        .is_empty());
    assert_eq!(String::from_utf8_lossy(&replayed.stderr), "");

    let (mut served, _) = RunningEngine::start_with(&installed.project, &mut captured, |command| {
        command.env("RRFLOW_INSTANCE_REGISTRY", registry);
    });
    let update = &served.announcement["instance_registry"];
    assert_eq!(update["status"], "not_updated");
    assert!(!update["reason"].as_str().unwrap().is_empty());
    let (output, report) = stop(installed.project_arg(), &[], &mut captured);
    assert_success(&output, "stop without a writable registry");
    assert_eq!(report["status"], "stopped");
    served.finish_stopped(&mut captured);

    let unreadable = invoke(&["--json", "--instance-registry", registry, "instances"]);
    assert!(
        !unreadable.status.success(),
        "an unreadable registry listed"
    );
    assert!(String::from_utf8_lossy(&unreadable.stderr).starts_with("error: "));
    assert_eq!(
        std::fs::read(&blocker).unwrap(),
        b"a file where a directory is needed"
    );
}

/// A registry lock some other process holds and never releases must not
/// hang install, serve, stop or the listing: each gives up at the lock wait,
/// succeeds, and says why the registry was not updated. Two processes that
/// wait for the lock to replace one entry each leave it whole.
#[test]
fn a_held_registry_lock_never_hangs_install_serve_stop_or_the_listing() {
    // Long enough that a waiting writer is still waiting when the test lets
    // the lock go; short enough that a writer meant to give up does.
    const WAITING: &str = "120000";
    const GIVING_UP: &str = "150";
    let root = tempfile::tempdir().unwrap();
    let registry_path = root.path().join("instances");
    let registry = registry_path.to_str().unwrap();
    let installed = plan_project(root.path(), "project");
    let project = installed.project_arg();
    let mut captured = Vec::new();
    let applied = apply(&installed, registry, true);
    capture_output(&applied, &mut captured);
    assert_success(&applied, "install apply");
    let result: serde_json::Value = serde_json::from_slice(&applied.stdout).unwrap();
    assert_eq!(result["instance_registry"]["status"], "recorded");
    assert_eq!(
        result["instance_registry"]["registry_root"],
        registry_path.to_str().unwrap()
    );
    let busy = |update: &serde_json::Value| {
        assert_eq!(update["status"], "not_updated", "{update}");
        let reason = update["reason"].as_str().unwrap();
        assert!(
            reason.contains("busy") && reason.contains("--instance-registry-lock-wait-ms"),
            "{reason}"
        );
    };
    let serve_with = |lock_wait: &'static str, captured: &mut Vec<u8>| {
        RunningEngine::start_with(&installed.project, captured, |command| {
            command
                .env("RRFLOW_INSTANCE_REGISTRY", registry)
                .env("RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS", lock_wait);
        })
    };
    let replay_args = |lock_wait: &'static str| {
        let mut args = vec!["--json", "--instance-registry-lock-wait-ms", lock_wait];
        args.extend(installed.apply_args(registry));
        args.push(&installed.plan_sha256);
        args
    };
    // A lock is held through a handle of its own, as another process holds it.
    let hold = || {
        let held = std::fs::File::open(registry_path.join(".lock")).unwrap();
        held.lock().unwrap();
        held
    };

    // A server registers while the lock is free and finds it held when it
    // closes. The stop is confirmed as soon as the serving record is
    // withdrawn: the registry never delays it.
    let (mut waiting_server, _) = serve_with(WAITING, &mut captured);
    assert_eq!(
        waiting_server.announcement["instance_registry"]["status"],
        "recorded"
    );
    let held = hold();
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop while the registry lock is held");
    assert_eq!(report["status"], "stopped");

    // Install gives up at its bound, as JSON and for a person.
    let args = replay_args(GIVING_UP);
    let replayed = invoke(&args);
    capture_output(&replayed, &mut captured);
    assert_success(&replayed, "install replay while the registry lock is held");
    let result: serde_json::Value = serde_json::from_slice(&replayed.stdout).unwrap();
    assert_eq!(result["idempotent_replay"], true);
    busy(&result["instance_registry"]);
    let replayed = invoke(&args[1..]);
    capture_output(&replayed, &mut captured);
    assert_success(&replayed, "install replay for a person");
    let text = String::from_utf8_lossy(&replayed.stdout).into_owned();
    assert!(
        text.contains("instance registry: not updated") && text.contains("busy"),
        "{text}"
    );

    // The listing reads without the lock and says so.
    let output = invoke(&[
        "--json",
        "--instance-registry",
        registry,
        "--instance-registry-lock-wait-ms",
        GIVING_UP,
        "instances",
    ]);
    capture_output(&output, &mut captured);
    assert_success(&output, "list instances while the registry lock is held");
    let listing: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        listing["read_without_lock"]
            .as_str()
            .is_some_and(|reason| reason.contains("lock")),
        "{listing}"
    );
    assert_eq!(listing["unreadable"], serde_json::json!([]));
    assert_eq!(listing["instances"].as_array().unwrap().len(), 1);
    assert_eq!(
        listing["instances"][0]["serving_published"], true,
        "the closed server has not been able to write its entry yet"
    );

    // Two processes now wait for the lock to replace the one entry: the
    // closed server, and an installer replaying the plan. Neither can finish
    // while the lock is held, and each leaves a whole entry once it is free.
    let args = replay_args(WAITING);
    let mut waiting_installer = product_command();
    waiting_installer.args(&args);
    let mut waiting_installer = process::Process::start(
        &mut waiting_installer,
        Duration::from_secs(180),
        OUTPUT_LIMIT,
        None,
    )
    .expect("start an installer that waits for the registry lock");
    assert!(waiting_server.child.running() && waiting_installer.running());
    std::fs::File::unlock(&held).unwrap();
    let output = waiting_installer
        .finish()
        .expect("reap the waiting installer");
    capture_output(&output, &mut captured);
    assert_success(&output, "an installer that waited for the registry lock");
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["instance_registry"]["status"], "recorded");
    let output = waiting_server
        .child
        .finish()
        .expect("capture the server that waited for the registry lock");
    waiting_server.finished = true;
    capture_output(&output, &mut captured);
    assert_success(&output, "a server that waited for the registry lock");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "",
        "a registry it could write is not a warning"
    );
    let listing = listed(registry, &mut captured);
    assert_eq!(listing["read_without_lock"], serde_json::Value::Null);
    assert_eq!(listing["unreadable"], serde_json::json!([]));
    assert_eq!(listing["instances"].as_array().unwrap().len(), 1);
    assert_eq!(
        listing["instances"][0]["serving_published"], false,
        "both writers wrote what the project showed when their turn came"
    );
    let entry: serde_json::Value = serde_json::from_slice(
        &std::fs::read(entry_file(&registry_path, &installed.instance_id)).unwrap(),
    )
    .unwrap();
    assert_eq!(entry["project_root"], canonical(&installed.project));

    // A closing server that gives up at its bound still exits cleanly and
    // says what it skipped.
    let (mut closing, _) = serve_with(GIVING_UP, &mut captured);
    assert_eq!(
        closing.announcement["instance_registry"]["status"],
        "recorded"
    );
    let held = hold();
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop a server whose registry stays locked");
    assert_eq!(report["status"], "stopped");
    let output = closing
        .child
        .finish()
        .expect("capture the stopped rrflow service");
    closing.finished = true;
    capture_output(&output, &mut captured);
    assert_success(&output, "serve closing while the registry lock is held");
    let warning = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        warning.contains("warning: instance registry: not updated") && warning.contains("busy"),
        "{warning}"
    );

    // A server that cannot register at all announces, accepts connections
    // and stops, with the lock held throughout.
    let (mut unregistered, address) = serve_with(GIVING_UP, &mut captured);
    busy(&unregistered.announcement["instance_registry"]);
    assert!(
        answers_http(&address),
        "a server that could not register answers nothing"
    );
    let (output, report) = stop(project, &[], &mut captured);
    assert_success(&output, "stop a server that could not register");
    assert_eq!(report["status"], "stopped");
    unregistered.finish_stopped(&mut captured);

    // Once the holder lets go the registry is written again.
    std::fs::File::unlock(&held).unwrap();
    let replayed = apply(&installed, registry, true);
    assert_success(&replayed, "install replay after the lock is released");
    let result: serde_json::Value = serde_json::from_slice(&replayed.stdout).unwrap();
    assert_eq!(result["instance_registry"]["status"], "recorded");
    let listing = listed(registry, &mut captured);
    assert_eq!(listing["read_without_lock"], serde_json::Value::Null);
    assert_eq!(listing["instances"][0]["serving_published"], false);
}

/// Several installers writing one registry at once each leave a whole entry.
#[test]
fn concurrent_installs_register_every_instance_in_one_registry() {
    const INSTALLERS: usize = 4;
    let root = tempfile::tempdir().unwrap();
    let registry_path = root.path().join("instances");
    let registry = registry_path.to_str().unwrap();
    let planned: Vec<Installed> = (0..INSTALLERS)
        .map(|index| plan_project(root.path(), &format!("project-{index}")))
        .collect();

    let mut running: Vec<process::Process> = planned
        .iter()
        .map(|installed| {
            let mut command = product_command();
            command
                .args(installed.apply_args(registry))
                .arg(&installed.plan_sha256);
            process::Process::start(&mut command, Duration::from_secs(120), OUTPUT_LIMIT, None)
                .expect("start a concurrent installer")
        })
        .collect();
    let mut captured = Vec::new();
    for process in &mut running {
        let output = process.finish().expect("reap a concurrent installer");
        capture_output(&output, &mut captured);
        assert_success(&output, "concurrent install apply");
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains(&format!("instance registry: {}", registry_path.display())),
            "every concurrent installer recorded its instance"
        );
    }

    let listing = listed(registry, &mut captured);
    assert_eq!(listing["unreadable"], serde_json::json!([]));
    assert_eq!(listing["read_without_lock"], serde_json::Value::Null);
    let mut registered: Vec<(String, String)> = listing["instances"]
        .as_array()
        .unwrap()
        .iter()
        .map(|instance| {
            assert_eq!(instance["locator_present"], true);
            assert_eq!(instance["problems"], serde_json::json!([]));
            (
                instance["instance_id"].as_str().unwrap().to_owned(),
                instance["project_root"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    registered.sort();
    let mut expected: Vec<(String, String)> = planned
        .iter()
        .map(|installed| (installed.instance_id.clone(), canonical(&installed.project)))
        .collect();
    expected.sort();
    assert_eq!(registered, expected);
    let mut names: Vec<String> = std::fs::read_dir(&registry_path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names.len(),
        INSTALLERS + 1,
        "one entry for each instance and the lock, and no file left pending: {names:?}"
    );
}
