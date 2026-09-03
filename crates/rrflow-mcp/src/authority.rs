use crate::config::RuntimeConfig;
use rrd_client::{ClientConfig, RequestOptions, RrdClient, Session};
use rrd_contract::{
    AssembleContext, CanonicalId, CapabilityStatus, CloseSession, CorrelationId, CreateSession,
    DataReference, SessionLease, SessionLimits,
};
use rrd_engine::{InstanceBinding, RrdEngine};
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_API_KEY_BYTES: u64 = 64 * 1024;
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) enum RuntimeAuthority {
    Embedded {
        engine: Box<RrdEngine>,
        session: SessionLease,
    },
    Daemon(Box<DaemonAuthority>),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct RuntimeAuthorityProfile {
    pub mode: &'static str,
    pub execution_authority: &'static str,
    pub storage_access: &'static str,
    pub caller_authentication: &'static str,
    pub context_engine: &'static str,
}

pub(crate) struct DaemonAuthority {
    runtime: tokio::runtime::Runtime,
    client: RrdClient,
    session: Session,
    principal: CanonicalId,
    api_key: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextInput {
    query: String,
    #[serde(default)]
    seeds: Vec<DataReference>,
    #[serde(default)]
    valid_at: Option<u64>,
    #[serde(default = "default_graph_depth")]
    max_graph_depth: u8,
    #[serde(default = "default_max_items")]
    max_items: u64,
    #[serde(default = "default_output_bytes")]
    max_output_bytes: u64,
    #[serde(default = "default_scanned_changes")]
    max_scanned_changes: u64,
}

impl RuntimeAuthority {
    pub(crate) fn open(config: RuntimeConfig) -> Result<Self, Box<dyn std::error::Error>> {
        match config {
            RuntimeConfig::Embedded {
                database,
                project_root,
            } => {
                let binding = InstanceBinding::discover(&project_root)?;
                binding.verify_store_path(&database)?;
                let engine = RrdEngine::open_bound(&binding)?;
                if engine.security_enforced()? {
                    return Err(
                        "embedded MCP cannot bypass initialized security; use daemon mode with an API key"
                            .into(),
                    );
                }
                let coordinate = next_coordinate();
                let session = engine.create_session(
                    &session_request(),
                    &CorrelationId::new(format!("session-{coordinate}"))?,
                    now(),
                    &format!("request-{coordinate}"),
                    &format!("operation-{coordinate}"),
                )?;
                Ok(Self::Embedded {
                    engine: Box::new(engine),
                    session,
                })
            }
            RuntimeConfig::Daemon {
                url,
                instance,
                principal,
                api_key_file,
            } => {
                let address = parse_loopback_url(&url)?;
                let api_key = read_api_key(&api_key_file)?;
                let scope = format!("instance:{instance}");
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                let client = RrdClient::connect_local(address, instance, ClientConfig::default())?;
                let capabilities = runtime.block_on(client.capabilities())?;
                let security_enforced = capabilities.capabilities.iter().any(|capability| {
                    capability.name.as_str() == "security-policy"
                        && matches!(
                            capability.status,
                            CapabilityStatus::Experimental | CapabilityStatus::Available
                        )
                });
                if !security_enforced {
                    return Err("daemon MCP requires an initialized RRD security policy".into());
                }
                let session = runtime.block_on(client.create_session(
                    principal.clone(),
                    &api_key,
                    session_request(),
                    request_options(true)?,
                ))?;
                Ok(Self::Daemon(Box::new(DaemonAuthority {
                    runtime,
                    client,
                    session,
                    principal,
                    api_key,
                    scope,
                })))
            }
        }
    }

    pub(crate) fn profile(&self) -> RuntimeAuthorityProfile {
        match self {
            Self::Embedded { .. } => RuntimeAuthorityProfile {
                mode: "embedded",
                execution_authority: "rrd_engine",
                storage_access: "exclusive_bound_engine",
                caller_authentication: "local_session",
                context_engine: "temporal_text_vector_graph",
            },
            Self::Daemon(_) => RuntimeAuthorityProfile {
                mode: "daemon",
                execution_authority: "rrd_server_via_rrd_client",
                storage_access: "none",
                caller_authentication: "principal_api_key_session",
                context_engine: "temporal_text_vector_graph",
            },
        }
    }

    pub(crate) fn assemble_context(&mut self, arguments: &Value) -> Result<String, String> {
        let input: ContextInput =
            serde_json::from_value(arguments.clone()).map_err(|error| error.to_string())?;
        let at = now();
        match self {
            Self::Embedded { engine, session } => {
                let coordinate = next_coordinate();
                let request = input.request(format!("instance:{}", engine.instance_id()), at);
                let packet = engine
                    .assemble_context(
                        &session.session_id,
                        &session.token,
                        &request,
                        at,
                        &format!("request-{coordinate}"),
                        &format!("operation-{coordinate}"),
                    )
                    .map_err(|error| error.to_string())?;
                serde_json::to_string(&packet).map_err(|error| error.to_string())
            }
            Self::Daemon(authority) => {
                let request = input.request(authority.scope.clone(), at);
                let options = request_options(false).map_err(|error| error.to_string())?;
                let first = authority
                    .runtime
                    .block_on(authority.client.assemble_context(
                        &authority.session,
                        request.clone(),
                        options.clone(),
                    ));
                let result = match first {
                    Err(error) if rrd_client::is_unauthenticated(&error) => {
                        authority
                            .refresh_session()
                            .map_err(|error| error.to_string())?;
                        authority
                            .runtime
                            .block_on(authority.client.assemble_context(
                                &authority.session,
                                request,
                                options,
                            ))
                    }
                    result => result,
                };
                let packet = result.map_err(|error| error.to_string())?;
                serde_json::to_string(&packet).map_err(|error| error.to_string())
            }
        }
    }

    pub(crate) fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Embedded { engine, session } => {
                let coordinate = next_coordinate();
                engine.close_session(
                    &session.session_id,
                    &session.token,
                    &CloseSession {},
                    &CorrelationId::new(format!("close-{coordinate}"))?,
                    now(),
                    &format!("request-{coordinate}"),
                    &format!("operation-{coordinate}"),
                )?;
            }
            Self::Daemon(authority) => {
                authority.runtime.block_on(authority.client.close_session(
                    &authority.session,
                    CloseSession {},
                    request_options(true)?,
                ))?;
            }
        }
        Ok(())
    }
}

impl ContextInput {
    fn request(self, scope: String, now: u64) -> AssembleContext {
        AssembleContext {
            scope,
            query: self.query,
            valid_at: self.valid_at.unwrap_or(now),
            seeds: self.seeds,
            max_graph_depth: self.max_graph_depth,
            max_items: self.max_items,
            max_output_bytes: self.max_output_bytes,
            max_scanned_changes: self.max_scanned_changes,
        }
    }
}

impl DaemonAuthority {
    fn refresh_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.session = self.runtime.block_on(self.client.create_session(
            self.principal.clone(),
            &self.api_key,
            session_request(),
            request_options(true)?,
        ))?;
        Ok(())
    }
}

fn session_request() -> CreateSession {
    CreateSession {
        limits: SessionLimits {
            idle_timeout_ms: 15 * 60 * 1_000,
            absolute_timeout_ms: 60 * 60 * 1_000,
            max_open_transactions: 1,
        },
    }
}

fn request_options(mutation: bool) -> rrd_client::Result<RequestOptions> {
    let coordinate = next_coordinate();
    if mutation {
        RequestOptions::mutation(
            &format!("request-{coordinate}"),
            &format!("operation-{coordinate}"),
            &format!("idempotency-{coordinate}"),
        )
    } else {
        RequestOptions::read(
            &format!("request-{coordinate}"),
            &format!("operation-{coordinate}"),
        )
    }
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

fn next_coordinate() -> String {
    let sequence = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("rrflow-mcp-{}-{sequence}", now())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the Unix epoch")
        .as_millis() as u64
}

fn parse_loopback_url(url: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let address: SocketAddr = url
        .strip_prefix("http://")
        .ok_or("daemon URL must use loopback http:// in the local profile")?
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("daemon cleartext URL must resolve to a loopback address".into());
    }
    Ok(address)
}

fn read_api_key(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    if !path.is_absolute() {
        return Err("--api-key-file must be an absolute operator-owned path".into());
    }
    let resolved = std::fs::canonicalize(path)?;
    let metadata = std::fs::metadata(&resolved)?;
    if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > MAX_API_KEY_BYTES
    {
        return Err("API key source must be a non-empty bounded regular file".into());
    }
    ensure_private(&resolved, &metadata)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(resolved)?
        .take(MAX_API_KEY_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_API_KEY_BYTES || !bytes.iter().all(|byte| byte.is_ascii_graphic()) {
        return Err("API key must be bounded visible ASCII without trailing whitespace".into());
    }
    String::from_utf8(bytes).map_err(Into::into)
}

#[cfg(unix)]
fn ensure_private(path: &Path, metadata: &std::fs::Metadata) -> std::io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mode = metadata.permissions().mode();
    let parent = std::fs::metadata(path.parent().unwrap_or(Path::new("/")))?;
    if mode & 0o077 != 0 || metadata.uid() != parent.uid() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "API key file must be owner-only and owned with its parent",
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_private(_path: &Path, _metadata: &std::fs::Metadata) -> std::io::Result<()> {
    Ok(())
}
