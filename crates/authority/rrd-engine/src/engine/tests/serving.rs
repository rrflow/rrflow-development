use super::*;
use crate::engine::clock::FixedClock;
use crate::engine::ServingStopState;
use rrd_contract::InstallationTargetKind;
use serde_json::json;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

const INSTALLED_AT: u64 = 1_800_000_000_000;

fn installed_project() -> (tempfile::TempDir, PathBuf) {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("README.md"), "project").unwrap();
    let executable = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let preview = RrdEngine::plan_installation(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &"1".repeat(64),
        "default",
        None,
        &executable,
    )
    .unwrap();
    RrdEngine::apply_installation_at(
        project.path(),
        InstallationTargetKind::ExistingProject,
        &preview,
        &preview.installation.plan_sha256,
        INSTALLED_AT,
        &executable,
    )
    .unwrap();
    (project, executable)
}

fn open(project: &Path, executable: &Path) -> crate::engine::Result<RrdEngine> {
    RrdEngine::open_installed_with_clock(project, executable, &FixedClock::at(INSTALLED_AT + 1_000))
}

fn record_path(project: &Path) -> PathBuf {
    project.join(".rrflow/serving.json")
}

fn recorded_url(project: &Path) -> String {
    RrdEngine::read_serving_record(project)
        .unwrap()
        .expect("a served endpoint record")
        .url
}

#[test]
fn a_served_engram_records_its_endpoint_and_withdraws_it() {
    let (project, executable) = installed_project();
    assert_eq!(
        RrdEngine::read_serving_record(project.path()).unwrap(),
        None
    );
    let engine = open(project.path(), &executable).unwrap();
    let address: SocketAddr = "127.0.0.1:41234".parse().unwrap();

    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish(address)
        .unwrap();

    let record = RrdEngine::read_serving_record(project.path())
        .unwrap()
        .expect("a served endpoint record");
    assert_eq!(&record, registration.record());
    assert_eq!(record.url, "http://127.0.0.1:41234");
    assert_eq!(record.address().unwrap(), address);
    assert_eq!(&record.instance_id, engine.instance_id());
    assert_eq!(record.process_id, std::process::id());
    let written: serde_json::Value =
        serde_json::from_slice(&std::fs::read(record_path(project.path())).unwrap()).unwrap();
    let mut fields: Vec<_> = written.as_object().unwrap().keys().cloned().collect();
    fields.sort();
    assert_eq!(
        fields,
        [
            "format_version",
            "instance_id",
            "process_id",
            "product_version",
            "url"
        ],
        "the record holds no credential or other field"
    );
    let locator = std::fs::read_to_string(project.path().join(".rrflow/config.toml")).unwrap();
    assert!(!locator.contains("41234"), "the locator holds no endpoint");

    registration.withdraw().unwrap();
    assert!(!record_path(project.path()).exists());
    assert_eq!(
        RrdEngine::read_serving_record(project.path()).unwrap(),
        None
    );
}

#[test]
fn a_stale_record_is_replaced_and_withdrawal_leaves_a_newer_one() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let publisher = engine.serving_publisher().unwrap();

    // A killed process never withdraws. The next publication, made while the
    // writer lock is held, replaces its record.
    let stale = publisher
        .publish("127.0.0.1:41000".parse().unwrap())
        .unwrap();
    let current = publisher
        .publish("127.0.0.1:41001".parse().unwrap())
        .unwrap();
    assert_eq!(recorded_url(project.path()), "http://127.0.0.1:41001");

    // Withdrawal removes only the record this registration wrote.
    stale.withdraw().unwrap();
    assert_eq!(recorded_url(project.path()), "http://127.0.0.1:41001");
    current.withdraw().unwrap();
    assert!(!record_path(project.path()).exists());
}

#[test]
fn publication_refuses_an_object_it_cannot_remove() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    std::fs::create_dir(record_path(project.path())).unwrap();

    assert!(engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41002".parse().unwrap())
        .is_err());
    assert!(
        record_path(project.path()).is_dir(),
        "an object the engine did not write is not replaced"
    );
}

#[test]
fn a_record_naming_another_address_instance_or_shape_is_refused() {
    let (project, executable) = installed_project();
    let instance = open(project.path(), &executable)
        .unwrap()
        .instance_id()
        .to_string();
    let path = record_path(project.path());
    let write = |value: &serde_json::Value| {
        std::fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
    };
    let valid = json!({
        "format_version": 1,
        "instance_id": instance,
        "url": "http://127.0.0.1:9477",
        "process_id": 7,
        "product_version": "1.0.0",
    });
    write(&valid);
    assert_eq!(recorded_url(project.path()), "http://127.0.0.1:9477");

    for (field, value) in [
        ("url", json!("http://192.0.2.1:9477")),
        ("url", json!("https://127.0.0.1:9477")),
        ("url", json!("http://localhost:9477")),
        ("url", json!("http://127.0.0.1:9477/")),
        ("instance_id", json!("another-instance")),
        ("format_version", json!(2)),
        ("product_version", json!("")),
        ("credential", json!("a secret")),
    ] {
        let mut record = valid.clone();
        record[field] = value;
        write(&record);
        let error = RrdEngine::read_serving_record(project.path()).unwrap_err();
        assert!(
            matches!(error, ServiceError::Contract(_)),
            "{field}: {error:?}"
        );
    }

    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert!(RrdEngine::read_serving_record(project.path()).is_err());
}

#[test]
fn opening_an_engram_another_engine_holds_is_refused_as_in_use() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();

    let Err(error) = open(project.path(), &executable) else {
        panic!("a second engine opened an engram another engine holds");
    };
    assert!(matches!(error, ServiceError::EngramInUse(_)), "{error:?}");
    assert_eq!(
        error.kind(),
        crate::engine::ServiceErrorKind::FailedPrecondition
    );
    assert!(error
        .to_string()
        .contains("already open in another process"));

    drop(engine);
    open(project.path(), &executable).unwrap();
}

fn stop_request_path(project: &Path) -> PathBuf {
    project.join(".rrflow/stop-request.json")
}

#[test]
fn a_stop_request_reaches_the_published_record_and_is_acknowledged_last() {
    let (project, executable) = installed_project();
    assert!(
        RrdEngine::request_serving_stop(project.path())
            .unwrap()
            .is_none(),
        "nothing is served, so there is nothing to stop"
    );
    assert!(!stop_request_path(project.path()).exists());

    let engine = open(project.path(), &executable).unwrap();
    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41100".parse().unwrap())
        .unwrap();
    assert!(!registration.stop_requested().unwrap());
    assert!(!registration.acknowledge_stop().unwrap());

    let request = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .expect("a stop request for the published record");
    assert_eq!(request.record(), registration.record());
    let written: serde_json::Value =
        serde_json::from_slice(&std::fs::read(stop_request_path(project.path())).unwrap()).unwrap();
    let mut fields: Vec<_> = written.as_object().unwrap().keys().cloned().collect();
    fields.sort();
    assert_eq!(
        fields,
        ["format_version", "instance_id", "process_id", "url"],
        "the request holds no credential or other field"
    );
    // A second requester publishes the same request and waits on it too.
    let second = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .expect("the same stop request");
    assert!(registration.stop_requested().unwrap());
    assert_eq!(request.observe().unwrap(), ServingStopState::Pending);

    // The serving process closes its record: it withdraws the record, then
    // removes the request addressed to it.
    assert!(
        registration.close().unwrap(),
        "a request was there to remove"
    );
    assert!(!stop_request_path(project.path()).exists());
    assert!(!record_path(project.path()).exists());
    assert_eq!(request.observe().unwrap(), ServingStopState::Stopped);
    assert_eq!(second.observe().unwrap(), ServingStopState::Stopped);
    assert!(
        !stop_request_path(project.path()).exists(),
        "observing a finished stop publishes nothing"
    );
    assert!(RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .is_none());
}

#[test]
fn a_stop_request_stays_in_place_until_its_server_reads_it() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41101".parse().unwrap())
        .unwrap();
    let request = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .unwrap();
    let published = std::fs::read(stop_request_path(project.path())).unwrap();

    // Looking changes nothing, and a requester that gives up takes nothing
    // back: a stop that was asked for cannot be un-asked, and the request is
    // not one requester's to remove while others wait on it.
    assert_eq!(request.observe().unwrap(), ServingStopState::Pending);
    assert_eq!(request.observe().unwrap(), ServingStopState::Pending);
    drop(request);
    assert_eq!(
        std::fs::read(stop_request_path(project.path())).unwrap(),
        published
    );
    assert!(registration.stop_requested().unwrap());
    assert_eq!(recorded_url(project.path()), "http://127.0.0.1:41101");

    assert!(registration.close().unwrap());
    assert!(!stop_request_path(project.path()).exists());
}

#[test]
fn a_stop_request_for_an_earlier_record_does_not_stop_the_next_server() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let publisher = engine.serving_publisher().unwrap();

    // A killed process never acknowledges. The request left for it names its
    // record, and the next publication removes it.
    let killed = publisher
        .publish("127.0.0.1:41102".parse().unwrap())
        .unwrap();
    let stale = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .unwrap();
    assert!(killed.stop_requested().unwrap());
    let current = publisher
        .publish("127.0.0.1:41103".parse().unwrap())
        .unwrap();
    assert!(!stop_request_path(project.path()).exists());
    assert!(!current.stop_requested().unwrap());
    // The requester of the killed process is told that the project is served
    // again, never that its stop finished, and it does not ask the new
    // server to stop.
    assert_eq!(
        stale.observe().unwrap(),
        ServingStopState::Replaced(current.record().clone())
    );
    assert!(!stop_request_path(project.path()).exists());
    assert!(!current.stop_requested().unwrap());

    // A request that names another record, or is not a request, is ignored
    // and left for its owner.
    let mut other = serde_json::to_value(current.record()).unwrap();
    other.as_object_mut().unwrap().remove("product_version");
    other["process_id"] = json!(current.record().process_id.wrapping_add(1));
    for bytes in [
        serde_json::to_vec(&other).unwrap(),
        b"not a request".to_vec(),
    ] {
        std::fs::write(stop_request_path(project.path()), &bytes).unwrap();
        assert!(!current.stop_requested().unwrap());
        assert!(!current.acknowledge_stop().unwrap());
        assert_eq!(
            std::fs::read(stop_request_path(project.path())).unwrap(),
            bytes
        );
    }

    // A new request replaces what was left and reaches the current record.
    let request = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .unwrap();
    assert!(current.stop_requested().unwrap());
    assert!(!killed.stop_requested().unwrap());
    assert!(current.close().unwrap());
    assert_eq!(request.observe().unwrap(), ServingStopState::Stopped);
}

#[test]
fn one_requester_giving_up_does_not_finish_or_undo_anothers_stop() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41104".parse().unwrap())
        .unwrap();
    let patient = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .unwrap();
    let impatient = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .unwrap();

    // The impatient requester's wait ends before the server has looked. The
    // server still publishes its record, so nobody's stop has finished, and
    // the one request both of them share is still there for the server.
    assert_eq!(impatient.observe().unwrap(), ServingStopState::Pending);
    drop(impatient);
    assert_eq!(patient.observe().unwrap(), ServingStopState::Pending);
    assert!(stop_request_path(project.path()).exists());
    assert!(registration.stop_requested().unwrap());
    assert_eq!(recorded_url(project.path()), "http://127.0.0.1:41104");

    // Even with the request removed by something else, a published record is
    // never read as a finished stop.
    std::fs::remove_file(stop_request_path(project.path())).unwrap();
    assert_eq!(patient.observe().unwrap(), ServingStopState::Pending);

    registration.close().unwrap();
    assert_eq!(patient.observe().unwrap(), ServingStopState::Stopped);
}

#[test]
fn a_server_that_ended_before_it_saw_the_request_is_reported_stopped() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41105".parse().unwrap())
        .unwrap();

    // An interrupt closes the record in the moment between a requester
    // reading it and publishing its request, so nothing ever removes that
    // request. The requester's own bytes stand in for that late publication.
    let request = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .unwrap();
    let late = std::fs::read(stop_request_path(project.path())).unwrap();
    assert!(registration.close().unwrap());
    std::fs::write(stop_request_path(project.path()), &late).unwrap();

    assert_eq!(request.observe().unwrap(), ServingStopState::Stopped);
    assert!(
        !stop_request_path(project.path()).exists(),
        "a request nothing will read is not left for the next server"
    );
}

#[test]
fn dropping_a_registration_withdraws_its_record_and_no_newer_one() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let publisher = engine.serving_publisher().unwrap();

    // A serving process that fails after publication returns through a drop.
    let failed = publisher
        .publish("127.0.0.1:41106".parse().unwrap())
        .unwrap();
    assert!(record_path(project.path()).exists());
    drop(failed);
    assert!(!record_path(project.path()).exists());

    let earlier = publisher
        .publish("127.0.0.1:41107".parse().unwrap())
        .unwrap();
    let current = publisher
        .publish("127.0.0.1:41108".parse().unwrap())
        .unwrap();
    drop(earlier);
    assert_eq!(recorded_url(project.path()), "http://127.0.0.1:41108");
    drop(current);
    assert!(!record_path(project.path()).exists());
}

#[test]
fn an_unusable_stop_request_refuses_publication_and_is_named() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let publisher = engine.serving_publisher().unwrap();
    let pending_path = project.path().join(".rrflow/.stop-request.pending");
    let oversized = vec![b' '; 4 * 1024 + 1];

    for (path, name) in [
        (
            stop_request_path(project.path()),
            ".rrflow/stop-request.json",
        ),
        (pending_path, ".rrflow/.stop-request.pending"),
    ] {
        for directory in [false, true] {
            if directory {
                std::fs::create_dir(&path).unwrap();
            } else {
                std::fs::write(&path, &oversized).unwrap();
            }
            let error = publisher
                .publish("127.0.0.1:41109".parse().unwrap())
                .unwrap_err();
            let text = error.to_string();
            assert!(
                text.contains(name) && text.contains("stop request"),
                "the refusal names {name}: {text}"
            );
            assert!(
                !text.contains("served endpoint record"),
                "the refusal names the wrong file: {text}"
            );
            assert!(path.exists(), "an object the engine did not write stays");
            assert!(!record_path(project.path()).exists());
            if directory {
                std::fs::remove_dir(&path).unwrap();
            } else {
                std::fs::remove_file(&path).unwrap();
            }
        }
    }
    publisher
        .publish("127.0.0.1:41109".parse().unwrap())
        .unwrap();
}

#[test]
fn a_server_and_its_requesters_removing_the_same_files_together_never_fail() {
    const ROUNDS: usize = 40;
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let publisher = engine.serving_publisher().unwrap();
    let limit = std::time::Duration::from_secs(60);
    let interval = std::time::Duration::from_millis(1);

    // The withdrawn record tells the requesters that the stop has finished,
    // and they remove a request left behind. The server removes the request
    // it was sent at that same moment, while both requesters are reading the
    // record it withdraws. Whoever comes second finds a file gone, and a file
    // that is being removed can refuse a look. Neither is a failure for
    // anyone: the server closes, and every requester is told `Stopped`.
    for round in 0..ROUNDS {
        let registration = publisher
            .publish(format!("127.0.0.1:{}", 42000 + round).parse().unwrap())
            .unwrap();
        let requests = [(); 2].map(|()| {
            RrdEngine::request_serving_stop(project.path())
                .unwrap()
                .expect("a stop request for the published record")
        });
        assert!(registration.stop_requested().unwrap());
        let start = std::sync::Barrier::new(3);
        std::thread::scope(|scope| {
            let requesters = requests.each_ref().map(|request| {
                let start = &start;
                scope.spawn(move || {
                    start.wait();
                    request.wait(limit, interval)
                })
            });
            start.wait();
            registration.close().unwrap_or_else(|error| {
                panic!("round {round}: the server's close failed: {error}")
            });
            for requester in requesters {
                match requester.join().unwrap() {
                    Ok(state) => assert_eq!(state, ServingStopState::Stopped, "round {round}"),
                    Err(error) => panic!("round {round}: a requester failed: {error}"),
                }
            }
        });
        // Every removal has finished by now.
        assert!(!record_path(project.path()).exists());
        assert!(
            !stop_request_path(project.path()).exists(),
            "round {round}: a request was left behind"
        );
    }
}

#[test]
fn waiting_ends_at_its_limit_while_the_record_is_published() {
    let (project, executable) = installed_project();
    let engine = open(project.path(), &executable).unwrap();
    let registration = engine
        .serving_publisher()
        .unwrap()
        .publish("127.0.0.1:41110".parse().unwrap())
        .unwrap();
    let request = RrdEngine::request_serving_stop(project.path())
        .unwrap()
        .unwrap();
    let limit = std::time::Duration::from_millis(60);

    let started = std::time::Instant::now();
    assert_eq!(
        request
            .wait(limit, std::time::Duration::from_millis(5))
            .unwrap(),
        ServingStopState::Pending
    );
    assert!(started.elapsed() >= limit);
    // A zero limit is one look.
    assert_eq!(
        request
            .wait(
                std::time::Duration::ZERO,
                std::time::Duration::from_millis(5)
            )
            .unwrap(),
        ServingStopState::Pending
    );
    assert!(stop_request_path(project.path()).exists());

    registration.close().unwrap();
    assert_eq!(
        request
            .wait(limit, std::time::Duration::from_millis(5))
            .unwrap(),
        ServingStopState::Stopped
    );
}
