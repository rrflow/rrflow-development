//! Server process fixtures, permits and HTTP request helpers for the process tests.

use super::*;

pub(super) const MAX_CONCURRENT_SERVER_FIXTURES: usize = 4;

pub(super) const INTEGRATION_IO_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) const COMPONENT_TOKEN_KEY: [u8; 32] = [0x5a; 32];

pub(super) static ACTIVE_SERVER_FIXTURES: Mutex<usize> = Mutex::new(0);

pub(super) static SERVER_FIXTURE_AVAILABLE: Condvar = Condvar::new();

pub(super) struct ServerFixturePermit;

pub(super) fn acquire_server_fixture() -> ServerFixturePermit {
    let mut active = ACTIVE_SERVER_FIXTURES.lock().unwrap();
    while *active >= MAX_CONCURRENT_SERVER_FIXTURES {
        active = SERVER_FIXTURE_AVAILABLE.wait(active).unwrap();
    }
    *active += 1;
    ServerFixturePermit
}

impl Drop for ServerFixturePermit {
    fn drop(&mut self) {
        let mut active = ACTIVE_SERVER_FIXTURES.lock().unwrap();
        *active -= 1;
        SERVER_FIXTURE_AVAILABLE.notify_one();
    }
}

pub(super) fn engram_context(at: u64, operation: &str) -> rrd_engram::MutationContext {
    rrd_engram::MutationContext {
        at,
        actor: "rrd-engram".into(),
        request_id: format!("request-{operation}"),
        operation_id: CanonicalId::new(operation).unwrap(),
    }
}

pub(super) struct RunningServer {
    pub(super) address: SocketAddr,
    pub(super) shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    pub(super) thread: Option<JoinHandle<std::result::Result<(), HttpError>>>,
    pub(super) _fixture: ServerFixturePermit,
}

pub(super) struct RunningServerProcess {
    pub(super) address: SocketAddr,
    pub(super) child: Option<Child>,
    pub(super) project: PathBuf,
    pub(super) registry: PathBuf,
}

impl RunningServerProcess {
    /// Stops the child through the engine's one stop request, the way
    /// `rrflow stop` and the local-process driver do, and requires the clean
    /// shutdown: a successful exit with the serving record withdrawn.
    pub(super) fn stop(mut self) {
        let request = RrdEngine::request_serving_stop(&self.project)
            .unwrap()
            .expect("a running rrd-server child publishes a serving record");
        assert_eq!(
            request.record().process_id,
            self.child.as_ref().unwrap().id()
        );
        let deadline = Instant::now() + INTEGRATION_IO_TIMEOUT;
        let status = loop {
            if let Some(status) = self.child.as_mut().unwrap().try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "rrd-server child did not stop");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success());
        assert_eq!(
            request.observe().unwrap(),
            rrd_engine::ServingStopState::Stopped
        );
        assert_eq!(RrdEngine::read_serving_record(&self.project).unwrap(), None);
        assert!(!self.project.join(".rrflow/stop-request.json").exists());
        let listing = rrd_engine::InstanceRegistry::at(&self.registry)
            .list()
            .unwrap();
        assert!(
            !listing.instances[0].entry.serving_published,
            "a closed server brings its registry entry in line"
        );
        self.child = None;
    }
}

impl Drop for RunningServerProcess {
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        let _ = RrdEngine::request_serving_stop(&self.project);
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

pub(super) fn start_installed_process(
    server_binary: &Path,
    project: &Path,
    executable: &Path,
    temporary: &Path,
    name: &str,
) -> RunningServerProcess {
    let ready = temporary.join(format!("{name}.READY"));
    // The fixture names its own instance registry: a test must never write
    // the registry of the user who runs it.
    let registry = temporary.join("instance-registry");
    let mut child = Command::new(server_binary)
        .args([
            "--instance-registry",
            registry.to_str().unwrap(),
            "--project",
            project.to_str().unwrap(),
            "--distribution-executable",
            executable.to_str().unwrap(),
            "--bind",
            "127.0.0.1:0",
            "--ready-file",
            ready.to_str().unwrap(),
        ])
        .spawn()
        .unwrap();
    let deadline = Instant::now() + INTEGRATION_IO_TIMEOUT;
    while !ready.is_file() {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("rrd-server child exited before readiness: {status}");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("rrd-server child did not publish readiness");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let readiness: Value = serde_json::from_slice(&std::fs::read(&ready).unwrap()).unwrap();
    let address = readiness["url"]
        .as_str()
        .unwrap()
        .strip_prefix("http://")
        .unwrap()
        .parse()
        .unwrap();
    // Readiness is written after the serving record, so the record is there.
    let record = RrdEngine::read_serving_record(project)
        .unwrap()
        .expect("a ready rrd-server child has published its serving record");
    assert_eq!(record.address().unwrap(), address);
    assert_eq!(record.process_id, child.id());
    // A served instance is registered for this user, whoever serves it.
    let listing = rrd_engine::InstanceRegistry::at(&registry).list().unwrap();
    assert_eq!(listing.instances.len(), 1, "{listing:?}");
    assert_eq!(listing.instances[0].entry.instance_id, record.instance_id);
    assert!(listing.instances[0].entry.serving_published);
    assert_eq!(
        listing.instances[0].endpoint.as_deref(),
        Some(record.url.as_str())
    );
    RunningServerProcess {
        address,
        child: Some(child),
        project: project.to_owned(),
        registry,
    }
}

impl RunningServer {
    pub(super) fn stop(mut self) {
        let _ = self.shutdown.take().unwrap().send(());
        self.thread.take().unwrap().join().unwrap().unwrap();
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap().unwrap();
        }
    }
}

pub(super) fn start(root: &Path) -> RunningServer {
    start_configured(root, None)
}

pub(super) fn start_with_jwt(root: &Path, key: &[u8]) -> RunningServer {
    start_configured(
        root,
        Some(RrdJwtVerificationKey::new(key.to_vec()).unwrap()),
    )
}

pub(super) fn start_configured(
    root: &Path,
    jwt_verification_key: Option<RrdJwtVerificationKey>,
) -> RunningServer {
    let fixture = acquire_server_fixture();
    let engine = RrdEngine::open(
        root,
        CanonicalId::new("socket-test").unwrap(),
        COMPONENT_TOKEN_KEY,
    )
    .unwrap();
    let bind = "127.0.0.1:0".parse().unwrap();
    let server = match jwt_verification_key {
        Some(key) => RrdHttpServer::bind_with_jwt(engine, bind, key).unwrap(),
        None => RrdHttpServer::bind(engine, bind).unwrap(),
    };
    let address = server.local_addr();
    let (shutdown, receiver) = tokio::sync::oneshot::channel();
    let thread = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(server.serve_until(async move {
                let _ = receiver.await;
            }))
    });
    RunningServer {
        address,
        shutdown: Some(shutdown),
        thread: Some(thread),
        _fixture: fixture,
    }
}

pub(super) fn envelope(
    payload: Value,
    idempotency_key: Option<&str>,
    deadline: Option<u64>,
) -> Value {
    json!({
        "protocol": "rrd",
        "protocol_version": 1,
        "context": {
            "request_id": format!("request-{}", idempotency_key.unwrap_or("read")),
            "operation_id": format!("operation-{}", idempotency_key.unwrap_or("read")),
            "idempotency_key": idempotency_key,
            "deadline_unix_ms": deadline,
        },
        "resource": {
            "segments": [{"kind": "instance", "id": "socket-test"}],
        },
        "payload": payload,
    })
}

pub(super) fn http(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, Value) {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(INTEGRATION_IO_TIMEOUT))
        .unwrap();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    )
    .unwrap();
    for (name, value) in headers {
        write!(stream, "{name}: {value}\r\n").unwrap();
    }
    stream.write_all(b"\r\n").unwrap();
    stream.write_all(body).unwrap();
    stream.flush().unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap();
    let head = String::from_utf8_lossy(&response[..split]);
    let status = head
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let body = serde_json::from_slice(&response[split + 4..]).unwrap();
    (status, body)
}

pub(super) fn post(
    server: &RunningServer,
    path: &str,
    value: &Value,
    session: Option<(&str, &str)>,
) -> (u16, Value) {
    let body = serde_json::to_vec(value).unwrap();
    let mut headers = vec![("Content-Type", "application/json")];
    let authorization;
    if let Some((session_id, token)) = session {
        authorization = format!("Bearer {token}");
        headers.push(("X-RRD-Session", session_id));
        headers.push(("Authorization", &authorization));
    }
    http(server.address, "POST", path, &headers, &body)
}

pub(super) fn post_with_api_key(
    server: &RunningServer,
    path: &str,
    value: &Value,
    principal: &str,
    credential: &str,
) -> (u16, Value) {
    let body = serde_json::to_vec(value).unwrap();
    let authorization = format!("ApiKey {credential}");
    http(
        server.address,
        "POST",
        path,
        &[
            ("Content-Type", "application/json"),
            ("X-RRD-Principal", principal),
            ("Authorization", &authorization),
        ],
        &body,
    )
}

pub(super) fn assert_tree_excludes(root: &Path, secrets: &[&str]) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                let bytes = std::fs::read(entry.path()).unwrap();
                for secret in secrets {
                    assert!(
                        !bytes
                            .windows(secret.len())
                            .any(|window| window == secret.as_bytes()),
                        "raw credential reached {}",
                        entry.path().display()
                    );
                }
            }
        }
    }
}

pub(super) fn delete(
    server: &RunningServer,
    path: &str,
    value: &Value,
    session: (&str, &str),
) -> (u16, Value) {
    let body = serde_json::to_vec(value).unwrap();
    let authorization = format!("Bearer {}", session.1);
    http(
        server.address,
        "DELETE",
        path,
        &[
            ("Content-Type", "application/json"),
            ("X-RRD-Session", session.0),
            ("Authorization", &authorization),
        ],
        &body,
    )
}

pub(super) fn payload(response: &Value) -> &Value {
    &response["outcome"]["payload"]
}

pub(super) fn assert_direct_read_evidence(value: &Value, expected_path: ReadAccessPath) {
    let evidence: ReadEvidence = serde_json::from_value(value.clone()).unwrap();
    evidence.validate().unwrap();
    assert!(
        evidence.paths.iter().any(|path| path.path == expected_path),
        "missing direct read evidence for {expected_path:?}"
    );
}

pub(super) fn start_root() -> (tempfile::TempDir, PathBuf, RunningServer) {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("instance");
    let server = start(&root);
    (temporary, root, server)
}
