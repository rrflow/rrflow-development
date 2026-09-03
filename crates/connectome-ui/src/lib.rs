//! Client-only HTTP gateway and embedded frontend for Connectome.
//!
//! RRD is the sole data authority. This crate owns presentation and a bounded
//! authenticated session, but it never opens an RRD store or composes physical
//! query, vector, estate, cluster, or engine crates.

use rrd_client::{ClientConfig, RequestOptions, RrdClient, Session};
use rrd_contract::{
    AssembleContext, CanonicalId, ContextPacket, CreateSession, DataReference, DiagnosticSnapshot,
    ReadDiagnosticSnapshot, ServiceCapabilities, SessionLimits,
};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

const INDEX: &str = include_str!("../static/index.html");
const CSS: &str = include_str!("../static/app.css");
const JS: &str = include_str!("../static/app.js");
const REQUEST_BODY_LIMIT: u64 = 1024 * 1024;
const SESSION_RENEWAL_MARGIN_MS: u64 = 30_000;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Connection and presentation settings. The API key is deliberately never
/// included in a debug representation or HTTP response.
pub struct ConnectomeConfig {
    pub rrd_address: SocketAddr,
    pub instance: CanonicalId,
    pub principal: CanonicalId,
    pub api_key: String,
    pub scope: String,
    pub bind: SocketAddr,
    pub shutdown: Option<ShutdownFiles>,
}

#[derive(Debug, Clone)]
pub struct ShutdownFiles {
    pub request: PathBuf,
    pub complete: PathBuf,
}

impl ConnectomeConfig {
    pub fn validate(&self) -> Result<()> {
        if !self.rrd_address.ip().is_loopback() {
            return Err("Connectome local HTTP mode requires a loopback RRD address".into());
        }
        if self.api_key.is_empty() || self.api_key.as_bytes().contains(&0) {
            return Err("RRD API key is empty or contains NUL".into());
        }
        let request = diagnostic_request(&self.scope, now_unix_ms()?);
        request.validate()?;
        if let Some(shutdown) = &self.shutdown {
            if !shutdown.request.is_absolute()
                || !shutdown.complete.is_absolute()
                || shutdown.request == shutdown.complete
            {
                return Err("shutdown control files must be distinct absolute paths".into());
            }
        }
        Ok(())
    }
}

/// A synchronous presentation adapter around the supported asynchronous Rust
/// client. Each outward operation is a typed RRD request under one renewable
/// authenticated session.
pub struct ConnectomeBackend {
    runtime: tokio::runtime::Runtime,
    client: RrdClient,
    session: Mutex<Session>,
    principal: CanonicalId,
    api_key: String,
    scope: String,
    sequence: AtomicU64,
}

impl ConnectomeBackend {
    pub fn connect(config: &ConnectomeConfig) -> Result<Self> {
        config.validate()?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        let client = RrdClient::connect_local(
            config.rrd_address,
            config.instance.clone(),
            ClientConfig {
                request_timeout: Duration::from_secs(10),
                max_attempts: 2,
            },
        )?;
        runtime.block_on(client.capabilities())?;
        let session = runtime.block_on(client.create_session(
            config.principal.clone(),
            &config.api_key,
            session_request(),
            RequestOptions::mutation(
                "connectome-session-request-1",
                "connectome-session-operation-1",
                "connectome-session-key-1",
            )?,
        ))?;
        Ok(Self {
            runtime,
            client,
            session: Mutex::new(session),
            principal: config.principal.clone(),
            api_key: config.api_key.clone(),
            scope: config.scope.clone(),
            sequence: AtomicU64::new(1),
        })
    }

    pub fn service_capabilities(&self) -> Result<ServiceCapabilities> {
        Ok(self.runtime.block_on(self.client.capabilities())?)
    }

    pub fn diagnostic_snapshot(&self) -> Result<DiagnosticSnapshot> {
        let session = self.current_session()?;
        let now = now_unix_ms()?;
        let (request, operation) = self.read_options("diagnostic-snapshot")?;
        Ok(self.runtime.block_on(self.client.read_diagnostic_snapshot(
            &session,
            diagnostic_request(&self.scope, now),
            RequestOptions::read(&request, &operation)?,
        ))?)
    }

    pub fn assemble_context(&self, request: ContextRequest) -> Result<ContextPacket> {
        request.validate()?;
        let session = self.current_session()?;
        let now = now_unix_ms()?;
        let (request_id, operation_id) = self.read_options("context")?;
        Ok(self.runtime.block_on(self.client.assemble_context(
            &session,
            AssembleContext {
                scope: self.scope.clone(),
                query: request.query,
                valid_at: request.valid_at.unwrap_or(now),
                seeds: request.seeds,
                max_graph_depth: request.max_graph_depth,
                max_items: request.max_items,
                max_output_bytes: request.max_output_bytes,
                max_scanned_changes: request.max_scanned_changes,
            },
            RequestOptions::read(&request_id, &operation_id)?,
        ))?)
    }

    fn current_session(&self) -> Result<Session> {
        let now = now_unix_ms()?;
        let mut guard = self
            .session
            .lock()
            .map_err(|_| "Connectome session lock is poisoned")?;
        if now.saturating_add(SESSION_RENEWAL_MARGIN_MS) >= guard.lease.idle_expires_at_unix_ms
            || now.saturating_add(SESSION_RENEWAL_MARGIN_MS)
                >= guard.lease.absolute_expires_at_unix_ms
        {
            let ordinal = self.next_sequence()?;
            *guard = self.runtime.block_on(self.client.create_session(
                self.principal.clone(),
                &self.api_key,
                session_request(),
                RequestOptions::mutation(
                    &format!("connectome-session-request-{ordinal}"),
                    &format!("connectome-session-operation-{ordinal}"),
                    &format!("connectome-session-key-{ordinal}"),
                )?,
            ))?;
        }
        Ok(guard.clone())
    }

    fn read_options(&self, name: &str) -> Result<(String, String)> {
        let ordinal = self.next_sequence()?;
        Ok((
            format!("connectome-{name}-request-{ordinal}"),
            format!("connectome-{name}-operation-{ordinal}"),
        ))
    }

    fn next_sequence(&self) -> Result<u64> {
        self.sequence
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| "Connectome request sequence exhausted".into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextRequest {
    pub query: String,
    #[serde(default)]
    pub seeds: Vec<DataReference>,
    #[serde(default)]
    pub valid_at: Option<u64>,
    #[serde(default = "default_graph_depth")]
    pub max_graph_depth: u8,
    #[serde(default = "default_max_items")]
    pub max_items: u64,
    #[serde(default = "default_output_bytes")]
    pub max_output_bytes: u64,
    #[serde(default = "default_scanned_changes")]
    pub max_scanned_changes: u64,
}

impl ContextRequest {
    fn validate(&self) -> Result<()> {
        AssembleContext {
            scope: "validation".into(),
            query: self.query.clone(),
            valid_at: self.valid_at.unwrap_or(1),
            seeds: self.seeds.clone(),
            max_graph_depth: self.max_graph_depth,
            max_items: self.max_items,
            max_output_bytes: self.max_output_bytes,
            max_scanned_changes: self.max_scanned_changes,
        }
        .validate()
        .map_err(Into::into)
    }
}

#[derive(Serialize)]
struct CapabilityView {
    service: ServiceCapabilities,
}

const fn default_graph_depth() -> u8 {
    2
}

const fn default_max_items() -> u64 {
    32
}

const fn default_output_bytes() -> u64 {
    256 * 1024
}

const fn default_scanned_changes() -> u64 {
    100_000
}

pub fn serve(config: ConnectomeConfig) -> Result<()> {
    config.validate()?;
    let server = Server::http(config.bind)?;
    let backend = ConnectomeBackend::connect(&config)?;
    eprintln!(
        "connectome: http://{} -> rrd://{} instance={} scope={} principal={}",
        config.bind, config.rrd_address, config.instance, config.scope, config.principal
    );
    loop {
        if config
            .shutdown
            .as_ref()
            .is_some_and(|shutdown| shutdown.request.is_file())
        {
            break;
        }
        if let Some(request) = server.recv_timeout(Duration::from_millis(100))? {
            respond(request, &backend);
        }
    }
    if let Some(shutdown) = config.shutdown {
        write_shutdown_completion(&shutdown.complete)?;
    }
    Ok(())
}

fn write_shutdown_completion(path: &Path) -> std::io::Result<()> {
    use std::io::Write;

    let temporary = path.with_extension("complete.new");
    let _ = std::fs::remove_file(&temporary);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(b"connectome-graceful-shutdown-v1\n")?;
    file.sync_all()?;
    std::fs::rename(&temporary, path)?;
    #[cfg(unix)]
    std::fs::File::open(path.parent().expect("shutdown marker has a parent"))?.sync_all()?;
    Ok(())
}

fn respond(mut request: Request, backend: &ConnectomeBackend) {
    let path = request.url().split('?').next().unwrap_or(request.url());
    let response = match (request.method(), path) {
        (&Method::Get | &Method::Head, "/" | "/index.html") => {
            text_response(StatusCode(200), "text/html; charset=utf-8", INDEX)
        }
        (&Method::Get | &Method::Head, "/app.css") => {
            text_response(StatusCode(200), "text/css; charset=utf-8", CSS)
        }
        (&Method::Get | &Method::Head, "/app.js") => {
            text_response(StatusCode(200), "text/javascript; charset=utf-8", JS)
        }
        (&Method::Get | &Method::Head, "/api/snapshot") => match backend.diagnostic_snapshot() {
            Ok(snapshot) => json_response(StatusCode(200), &snapshot),
            Err(error) => gateway_error(error),
        },
        (&Method::Get | &Method::Head, "/api/runtime/capabilities") => {
            match backend.service_capabilities() {
                Ok(service) => json_response(StatusCode(200), &CapabilityView { service }),
                Err(error) => gateway_error(error),
            }
        }
        (&Method::Post, "/api/context") => {
            match read_json::<ContextRequest>(&mut request)
                .and_then(|value| backend.assemble_context(value))
            {
                Ok(result) => json_response(StatusCode(200), &result),
                Err(error) => request_error(error),
            }
        }
        (&Method::Post, "/api/flights" | "/api/demos/prompt-strength" | "/api/cluster/samples") => {
            json_response(
                StatusCode(501),
                &serde_json::json!({
                    "error": "the retired embedded diagnostics mutation has no governed RRD operation",
                    "required_action": "add an exact rrd-contract/rrd-engine operation before exposing this mutation"
                }),
            )
        }
        (
            &Method::Get | &Method::Head,
            "/api/flights" | "/api/cluster/history" | "/api/runtime/traces" | "/api/route",
        ) => json_response(
            StatusCode(501),
            &serde_json::json!({
                "error": "this legacy projection is not part of the authoritative diagnostic snapshot",
                "available": ["/api/snapshot", "/api/runtime/capabilities", "/api/context"]
            }),
        ),
        (&Method::Get | &Method::Head, _) => {
            json_response(StatusCode(404), &serde_json::json!({"error": "not found"}))
        }
        _ => json_response(
            StatusCode(405),
            &serde_json::json!({"error": "method not allowed"}),
        ),
    };
    let _ = request.respond(response);
}

fn diagnostic_request(scope: &str, now: u64) -> ReadDiagnosticSnapshot {
    ReadDiagnosticSnapshot {
        scope: scope.to_owned(),
        graph_valid_at_unix_ms: now,
        graph_known_at_cursor: None,
        graph_compare_cursor: 0,
        runtime_max_scanned_changes: 1_000_000,
        changes_after_cursor: 0,
        change_limit: 4_096,
        audit_after_sequence: 0,
        audit_limit: 1_024,
    }
}

fn session_request() -> CreateSession {
    CreateSession {
        limits: SessionLimits {
            idle_timeout_ms: 900_000,
            absolute_timeout_ms: 3_600_000,
            max_open_transactions: 4,
        },
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(request: &mut Request) -> Result<T> {
    let mut bytes = Vec::new();
    request
        .as_reader()
        .take(REQUEST_BODY_LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > REQUEST_BODY_LIMIT {
        return Err("request body exceeds one MiB".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn request_error(
    error: Box<dyn std::error::Error + Send + Sync>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    json_response(
        StatusCode(400),
        &serde_json::json!({"error": error.to_string()}),
    )
}

fn gateway_error(
    error: Box<dyn std::error::Error + Send + Sync>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    json_response(
        StatusCode(502),
        &serde_json::json!({"error": error.to_string()}),
    )
}

fn text_response(
    status: StatusCode,
    content_type: &str,
    body: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(body.as_bytes().to_vec())
        .with_status_code(status)
        .with_header(Header::from_bytes("Content-Type", content_type).expect("valid header"))
        .with_header(Header::from_bytes("Cache-Control", "no-store").expect("valid header"))
        .with_header(Header::from_bytes("X-Content-Type-Options", "nosniff").expect("valid header"))
        .with_header(Header::from_bytes("Referrer-Policy", "no-referrer").expect("valid header"))
}

fn json_response<T: Serialize>(status: StatusCode, body: &T) -> Response<std::io::Cursor<Vec<u8>>> {
    match serde_json::to_vec(body) {
        Ok(body) => Response::from_data(body)
            .with_status_code(status)
            .with_header(
                Header::from_bytes("Content-Type", "application/json; charset=utf-8")
                    .expect("valid header"),
            )
            .with_header(Header::from_bytes("Cache-Control", "no-store").expect("valid header"))
            .with_header(
                Header::from_bytes("X-Content-Type-Options", "nosniff").expect("valid header"),
            )
            .with_header(
                Header::from_bytes("Referrer-Policy", "no-referrer").expect("valid header"),
            ),
        Err(error) => text_response(
            StatusCode(500),
            "application/json; charset=utf-8",
            &format!("{error}"),
        ),
    }
}

fn now_unix_ms() -> Result<u64> {
    Ok(u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_loopback_rrd_and_empty_credentials() {
        let mut config = ConnectomeConfig {
            rrd_address: "10.0.0.1:9477".parse().unwrap(),
            instance: CanonicalId::new("test-instance").unwrap(),
            principal: CanonicalId::new("connectome").unwrap(),
            api_key: "secret".into(),
            scope: "instance:test-instance".into(),
            bind: "127.0.0.1:4387".parse().unwrap(),
            shutdown: None,
        };
        assert!(config
            .validate()
            .unwrap_err()
            .to_string()
            .contains("loopback"));
        config.rrd_address = "127.0.0.1:9477".parse().unwrap();
        config.api_key.clear();
        assert!(config
            .validate()
            .unwrap_err()
            .to_string()
            .contains("API key"));
    }
}
