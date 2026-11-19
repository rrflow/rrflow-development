use super::*;

#[test]
fn arguments_require_one_installed_project_and_distribution_executable() {
    assert!(parse_args(std::iter::empty()).is_err());
    assert!(parse_args(["--db".into(), "state".into()].into_iter()).is_err());
    assert!(parse_args(["initialize".into()].into_iter()).is_err());
    assert!(parse_args(["--root".into(), "project".into()].into_iter()).is_err());
    assert!(parse_args(["--project".into(), "project".into()].into_iter()).is_err());
    let args = parse_args(
        [
            "--project".into(),
            "project".into(),
            "--distribution-executable".into(),
            "rrflow".into(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(args.bind, "127.0.0.1:9477".parse().unwrap());
    assert_eq!(args.project, PathBuf::from("project"));
    assert_eq!(args.distribution_executable, PathBuf::from("rrflow"));
}

fn installed(extra: &[&str]) -> Result<Args, String> {
    parse_args(
        [
            "--project",
            "project",
            "--distribution-executable",
            "rrflow",
        ]
        .into_iter()
        .chain(extra.iter().copied())
        .map(String::from),
    )
}

#[test]
fn the_engine_stop_request_is_the_only_way_to_ask_the_server_to_stop() {
    // The server looks for the engine's stop request on a named default that
    // a flag overrides; it takes no stop files of its own.
    let args = installed(&[]).unwrap();
    assert_eq!(
        args.stop_poll_interval,
        Duration::from_millis(rrd_engine::DEFAULT_STOP_POLL_INTERVAL_MS)
    );
    let args = installed(&["--stop-poll-interval-ms", "25"]).unwrap();
    assert_eq!(args.stop_poll_interval, Duration::from_millis(25));
    assert!(installed(&["--stop-poll-interval-ms", "0"]).is_err());
    assert!(installed(&["--stop-poll-interval-ms", "soon"]).is_err());
    assert!(installed(&["--stop-poll-interval-ms"]).is_err());

    let root = std::env::current_dir().unwrap();
    let request = root.join("shutdown.request");
    let complete = root.join("shutdown.complete");
    for retired in [
        vec!["--shutdown-request-file", request.to_str().unwrap()],
        vec!["--shutdown-complete-file", complete.to_str().unwrap()],
        vec![
            "--shutdown-request-file",
            request.to_str().unwrap(),
            "--shutdown-complete-file",
            complete.to_str().unwrap(),
        ],
    ] {
        let error = installed(&retired).err().expect("a retired stop file flag");
        assert!(error.contains("unknown argument"), "{error}");
    }
}

#[test]
fn the_instance_registry_and_its_lock_wait_are_named_explicitly() {
    let args = installed(&[]).unwrap();
    assert_eq!(args.instance_registry, None);
    assert_eq!(args.instance_registry_lock_wait_ms, None);
    let args = installed(&[
        "--instance-registry",
        "registry",
        "--instance-registry-lock-wait-ms",
        "0",
    ])
    .unwrap();
    assert_eq!(args.instance_registry, Some(PathBuf::from("registry")));
    assert_eq!(args.instance_registry_lock_wait_ms, Some(0));
    assert!(installed(&["--instance-registry-lock-wait-ms", "-1"]).is_err());
}

#[test]
fn readiness_file_is_absolute() {
    let root = std::env::current_dir().unwrap();
    let ready = root.join("rrd.ready");
    let args = installed(&["--ready-file", ready.to_str().unwrap()]).unwrap();
    assert_eq!(args.ready_file, Some(ready));
    assert!(installed(&["--ready-file", "relative.ready"]).is_err());
}

#[test]
fn mutual_tls_files_are_an_all_or_nothing_contract() {
    let incomplete = [
        "--project".into(),
        "project".into(),
        "--distribution-executable".into(),
        "rrflow".into(),
        "--tls-cert".into(),
        "server.pem".into(),
    ];
    assert!(parse_args(incomplete.into_iter()).is_err());
}
