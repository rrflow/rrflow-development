use rrd_engine::{InstanceRegistry, RrdEngine};
use rrd_server::{serve_published, RrdHttpServer, RrdJwtVerificationKey, RrdMutualTlsServerConfig};
use rustls::RootCertStore;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

struct Args {
    project: PathBuf,
    distribution_executable: PathBuf,
    bind: SocketAddr,
    jwt_key_file: Option<PathBuf>,
    ready_file: Option<PathBuf>,
    stop_poll_interval: Duration,
    instance_registry: Option<PathBuf>,
    instance_registry_lock_wait_ms: Option<u64>,
    tls_certificate_file: Option<PathBuf>,
    tls_private_key_file: Option<PathBuf>,
    tls_client_ca_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("rrd-server: {error}");
        std::process::exit(2);
    }
}

mod cli;
mod process;

use cli::*;
use process::*;
