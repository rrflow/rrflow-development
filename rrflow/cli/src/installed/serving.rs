//! `rrflow serve`: publish the serving record, serve until an interrupt or a
//! stop request, and close the record on every path out.

use super::*;

#[derive(Debug, Serialize)]
pub(super) struct ServeAnnouncement {
    pub(super) status: &'static str,
    pub(super) url: String,
    pub(super) project_root: String,
    pub(super) instance_id: String,
    pub(super) product_version: String,
    pub(super) build: crate::build_identity::BuildReport,
    pub(super) readiness_path: &'static str,
    pub(super) capability_path: &'static str,
    pub(super) endpoint_catalogue_path: &'static str,
    pub(super) openapi_path: &'static str,
    pub(super) instance_registry: RegistryUpdate,
}

pub(super) fn serve(
    project: &Path,
    bind: std::net::SocketAddr,
    endpoint_credentials: Option<&Path>,
    stop_poll_interval: Duration,
    test_distribution_executable: Option<&Path>,
    json: bool,
    instance_registry: RegistryOptions<'_>,
) -> Result<Execution, BoxError> {
    let started = Instant::now();
    let executable = distribution_executable_path(test_distribution_executable)?;
    let locator = RrdEngine::read_project_locator(project)?;
    let project_root = std::fs::canonicalize(project)?.display().to_string();
    // Everything that can fail without the engine is done before a serving
    // record exists, so that failure leaves none behind.
    let build = crate::build_identity::report()?;
    let runtime = runtime()?;
    #[cfg(feature = "crash-test")]
    rrd_engine::crash_test::configure_from_stdin(&project.join(&locator.storage_root))?;
    let mut engine = RrdEngine::open_installed(project, &executable)
        .map_err(|error| engram_open_elsewhere(project, error))?;
    if let Some(path) = endpoint_credentials {
        engine.install_endpoint_credentials(endpoint_credentials::load(path)?)?;
    }
    // Refreshed while this engine holds the writer lock; the map is a
    // replaceable view, so its outcome is reported and never stops serving.
    let map = project_map_line(project);
    // Issued while this engine holds the engram's writer lock, so a record
    // found at publication belongs to a process that no longer runs.
    let publisher = engine.serving_publisher()?;
    let server = RrdHttpServer::bind(engine, bind)?;
    let address = server.local_addr();
    let instance_id = locator.identity.instance_id.to_string();
    let registration = Arc::new(publisher.publish(address)?);
    // The record is published. Every path from here reaches the one shutdown
    // below, which closes it; nothing returns in between.
    let registry = update_registry(instance_registry, "serve.listening", |registry| {
        registry.record_serving(project, true)
    });
    let registry_recorded = registry.recorded();
    let mut end = None;
    let served = (|| -> Result<(), BoxError> {
        // Listened for before the announcement is written, so an interrupt
        // sent the moment a supervisor reads it is already caught and ends
        // this server through the one shutdown below.
        let interrupt = {
            let _runtime = runtime.enter();
            rrd_server::Interrupt::listen()?
        };
        let registry_line = registry.line();
        let announcement = ServeAnnouncement {
            status: "listening",
            url: format!("http://{address}"),
            project_root,
            instance_id: instance_id.clone(),
            product_version: locator.product_version,
            build,
            readiness_path: "/v1/health/ready",
            capability_path: "/v1/capabilities",
            endpoint_catalogue_path: "/v1/schema/endpoints",
            openapi_path: "/v1/schema/openapi",
            instance_registry: registry,
        };
        let text = if json {
            // One line is a deliberate process-supervision contract: callers
            // can parse readiness without guessing where a pretty document
            // ends.
            serde_json::to_string(&announcement)?
        } else {
            format!(
                "RRFlow listening at {}\ninstance: {}\nUI discovery: {}/v1/schema/openapi\n{map}\n{registry_line}",
                announcement.url, announcement.instance_id, announcement.url
            )
        };
        let mut stdout = std::io::stdout();
        writeln!(stdout, "{text}")?;
        stdout.flush()?;
        tracing::info!(
            target: "rrflow::installed",
            operation = "serve.listening",
            instance_id = %instance_id,
            address = %address,
            serving_record = %registration.record().url,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "installed lifecycle operation completed"
        );
        // Ends on an interrupt or on a stop request addressed to this record,
        // and closes the record either way.
        let served = runtime.block_on(rrd_server::serve_published(
            server,
            Arc::clone(&registration),
            stop_poll_interval,
            interrupt,
        ));
        let result = served.served;
        end = Some((served.stop_requested, served.closed));
        result.map_err(Into::into)
    })();
    // One shutdown for an interrupt, a stop request and a failed announcement.
    // The server and its engine have gone with the block above; a path that
    // never served closes the record here.
    let (stop_requested, closed) = end.unwrap_or_else(|| (false, registration.close()));
    // The registry is brought in line with what the project now holds only
    // after the record is closed, so a registry that is slow or locked never
    // delays what a stop request is waiting for. The closing server has no
    // answer on stdout, so a registry it could not update is a warning line.
    if registry_recorded {
        let closing = update_registry(instance_registry, "serve.closed", |registry| {
            registry.record_installed(project)
        });
        if !closing.recorded() {
            eprintln!("warning: {}", closing.line());
        }
    }
    tracing::info!(
        target: "rrflow::installed",
        operation = "serve.closed",
        instance_id = %instance_id,
        address = %address,
        stop_requested,
        serving_record_withdrawn = closed.is_ok(),
        stop_request_removed = closed.as_ref().is_ok_and(|removed| *removed),
        elapsed_ms = started.elapsed().as_millis() as u64,
        "installed lifecycle operation completed"
    );
    served?;
    closed?;
    Ok(Execution {
        text: String::new(),
        success: true,
    })
}

/// Another process holds the engram, so this one fails before binding. Name
/// the endpoint it recorded, if any; only `rrflow ready` can prove it.
pub(super) fn engram_open_elsewhere(project: &Path, error: ServiceError) -> BoxError {
    if !matches!(error, ServiceError::EngramInUse(_)) {
        return error.into();
    }
    match RrdEngine::read_serving_record(project) {
        Ok(Some(record)) => format!(
            "this project's RRFlow engram is already open in another process; its served endpoint is recorded at {}. Run `rrflow ready --project` to prove it.",
            record.url
        )
        .into(),
        _ => "this project's RRFlow engram is already open in another process".into(),
    }
}
