//! Running the server: its serving record, its one way to stop, and readiness.

use super::*;

pub(super) async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if std::env::var_os("RUST_LOG").is_some() {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init()?;
    }
    let args = parse_args(std::env::args().skip(1))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let engine = RrdEngine::open_installed(&args.project, &args.distribution_executable)?;
    // Issued while this engine holds the engram's writer lock, so a record
    // found at publication belongs to a process that no longer runs.
    let publisher = engine.serving_publisher()?;
    let tls = load_mtls(
        args.tls_certificate_file,
        args.tls_private_key_file,
        args.tls_client_ca_file,
    )?;
    let jwt = args.jwt_key_file.map(load_jwt_key).transpose()?;
    let server = match (tls, jwt) {
        (Some(tls), Some(jwt)) => RrdHttpServer::bind_mtls_with_jwt(engine, args.bind, tls, jwt)?,
        (Some(tls), None) => RrdHttpServer::bind_mtls(engine, args.bind, tls)?,
        (None, Some(jwt)) => RrdHttpServer::bind_with_jwt(engine, args.bind, jwt)?,
        (None, None) => RrdHttpServer::bind(engine, args.bind)?,
    };
    let scheme = if server.is_tls() { "https" } else { "http" };
    let address = server.local_addr();
    eprintln!("rrd-server: {scheme}://{address}");
    // A serving record names plain loopback HTTP, the one endpoint a local
    // client can prove with the installed credential. A listener of any other
    // kind publishes none, so no stop request can reach it.
    if server.is_tls() || !address.ip().is_loopback() {
        eprintln!(
            "rrd-server: no serving record is published for a listener that is not plain loopback HTTP; only an interrupt stops it"
        );
        if let Some(path) = &args.ready_file {
            write_readiness(path, scheme, address)?;
        }
        server
            .serve_until(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await?;
        return Ok(());
    }
    // Dropping the registration withdraws the record, so every return below
    // leaves none behind.
    let registration = std::sync::Arc::new(publisher.publish(address)?);
    let registry = InstanceRegistry::resolve(
        args.instance_registry.as_deref(),
        args.instance_registry_lock_wait_ms,
        &|name| std::env::var_os(name),
    )
    .and_then(|registry| {
        registry.record_serving(&args.project, true)?;
        Ok(registry)
    });
    report_registry(
        registry
            .as_ref()
            .map(InstanceRegistry::root)
            .map_err(ToString::to_string),
    );
    // Readiness is written after the record, so a supervisor that saw it can
    // ask this process to stop.
    if let Some(path) = &args.ready_file {
        write_readiness(path, scheme, address)?;
    }
    let end = serve_published(server, registration, args.stop_poll_interval).await;
    // The registry is a candidate list: an entry this process wrote is brought
    // in line with what the project now holds, after the record is closed so
    // that the registry never delays what a stop request is waiting for.
    if let Ok(registry) = &registry {
        if let Err(error) = registry.record_installed(&args.project) {
            report_registry(Err(error.to_string()));
        }
    }
    end.served?;
    end.closed?;
    Ok(())
}

/// The registry is a candidate list with no authority, so a registry that
/// cannot be found or written is reported and never stops the server.
fn report_registry(outcome: Result<&Path, String>) {
    match outcome {
        Ok(root) => eprintln!("rrd-server: instance registry: {}", root.display()),
        Err(reason) => eprintln!("rrd-server: instance registry: not updated ({reason})"),
    }
}

pub(super) fn write_readiness(path: &Path, scheme: &str, address: SocketAddr) -> io::Result<()> {
    use std::io::Write;

    let bytes = serde_json::to_vec(&serde_json::json!({
        "format": 1,
        "status": "ready",
        "url": format!("{scheme}://{address}"),
    }))?;
    let temporary = path.with_extension("ready.new");
    let _ = std::fs::remove_file(&temporary);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    std::fs::rename(&temporary, path)?;
    #[cfg(unix)]
    std::fs::File::open(path.parent().expect("readiness marker has a parent"))?.sync_all()?;
    Ok(())
}
