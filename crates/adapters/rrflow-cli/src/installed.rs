//! Thin installed-product adapter over the engine, server, and generated SDK.
//!
//! This module owns no lifecycle state. Plans, installed records, credentials,
//! storage, and verification remain engine-owned; the CLI only translates
//! explicit operator arguments and renders typed results.

use crate::command::{
    Command, Execution, InstallAction, InstallMode, RepairAction, RestoreAction, VerifyLevel,
};
use rrd_client::{ClientConfig, RequestOptions, RrdClient};
use rrd_contract::{
    CanonicalId, CloseSession, CreateSession, InstallationTargetKind, RestorePublicationPlan,
    SessionLimits, PROTOCOL, PROTOCOL_VERSION,
};
use rrd_engine::{
    digest, InstallationPreview, InstallationVerificationStatus, InstanceRegistration,
    InstanceRegistry, InstanceRegistryListing, RrdEngine, ServiceError, ServingStopState,
};
use rrd_server::RrdHttpServer;
use serde::Serialize;
use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_INSTALLATION_PLAN_BYTES: u64 = 8 * 1024 * 1024;

type BoxError = Box<dyn Error>;
mod endpoint_credentials;

#[derive(Debug, Serialize)]
struct VersionReport {
    product: &'static str,
    product_version: &'static str,
    protocol: &'static str,
    protocol_version: u16,
    installation_plan_contract_version: u16,
    release_stage: &'static str,
    build: crate::build_identity::BuildReport,
}

#[derive(Debug, Serialize)]
struct ServeAnnouncement {
    status: &'static str,
    url: String,
    project_root: String,
    instance_id: String,
    product_version: String,
    build: crate::build_identity::BuildReport,
    readiness_path: &'static str,
    capability_path: &'static str,
    endpoint_catalogue_path: &'static str,
    openapi_path: &'static str,
    instance_registry: RegistryUpdate,
}

/// Where this user's instance registry is and how long to wait for its lock,
/// as the command line named them. What is not named comes from the
/// environment or the engine's defaults.
#[derive(Debug, Clone, Copy)]
pub struct RegistryOptions<'a> {
    pub root: Option<&'a Path>,
    pub lock_wait_ms: Option<u64>,
}

/// Whether this command recorded its project in the user's instance registry.
/// The registry is a candidate list, so a failure is reported and never
/// blocks the command.
#[derive(Debug, Serialize)]
struct RegistryUpdate {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    registry_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    /// Other project roots registered under this instance id: a copied
    /// project folder keeps its original's identity.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    other_project_roots: Vec<String>,
}

impl RegistryUpdate {
    fn recorded(&self) -> bool {
        self.reason.is_none()
    }

    fn line(&self) -> String {
        let line = match (&self.reason, &self.registry_root) {
            (None, Some(root)) => format!("instance registry: {root}"),
            (Some(reason), Some(root)) => {
                format!("instance registry: not updated at {root} ({reason})")
            }
            (Some(reason), None) => format!("instance registry: not updated ({reason})"),
            (None, None) => "instance registry: recorded".into(),
        };
        if self.other_project_roots.is_empty() {
            line
        } else {
            format!(
                "{line}\ninstance registry: this instance id is also registered for {}",
                self.other_project_roots.join(", ")
            )
        }
    }
}

/// The `install apply` answer: the engine's installation result, whose own
/// members `result_sha256` binds, and beside them what this adapter did with
/// the instance registry.
#[derive(Debug, Serialize)]
struct InstallApplyReport<'a> {
    #[serde(flatten)]
    installation: &'a rrd_contract::InstallationResult,
    instance_registry: &'a RegistryUpdate,
}

#[derive(Debug, Serialize)]
struct StopReport {
    status: &'static str,
    project_root: String,
    instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    process_id: Option<u32>,
    /// The record another process published while this stop waited.
    #[serde(skip_serializing_if = "Option::is_none")]
    replaced_by_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replaced_by_process_id: Option<u32>,
    waited_ms: u64,
    wait_limit_ms: u64,
}

#[derive(Debug, Serialize)]
struct UiDiscoveryReport {
    status: &'static str,
    url: String,
    project_root: String,
    openapi_sha256: String,
    authentication: AuthenticatedReady,
    readiness: rrd_contract::Readiness,
    capabilities: rrd_contract::ServiceCapabilities,
    endpoint_catalogue: rrd_contract::EndpointCatalogue,
    openapi: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct AuthenticatedReady {
    principal_id: String,
    session_created: bool,
    session_closed: bool,
}

pub fn execute(
    command: &Command,
    json: bool,
    instance_registry: RegistryOptions<'_>,
) -> Result<Execution, BoxError> {
    match command {
        Command::Version => version(json),
        Command::Install { action } => install(action, json, instance_registry),
        Command::Serve {
            project,
            bind,
            endpoint_credentials,
            stop_poll_interval_ms,
            test_distribution_executable,
        } => serve(
            project,
            *bind,
            endpoint_credentials.as_deref(),
            Duration::from_millis(*stop_poll_interval_ms),
            test_distribution_executable.as_deref(),
            json,
            instance_registry,
        ),
        Command::Stop {
            project,
            wait_ms,
            poll_interval_ms,
        } => stop(
            project,
            Duration::from_millis(*wait_ms),
            Duration::from_millis(*poll_interval_ms),
            json,
        ),
        Command::Instances => instances(json, instance_registry),
        Command::Ready { project, address } => ready(project, *address, json),
        Command::Verify {
            project,
            level,
            test_distribution_executable,
        } => verify(
            project,
            *level,
            test_distribution_executable.as_deref(),
            json,
        ),
        Command::Restore { action } => restore(action, json),
        Command::Repair { action } => repair(action),
    }
}

fn repair(action: &RepairAction) -> Result<Execution, BoxError> {
    match action {
        RepairAction::Plan {
            project,
            test_distribution_executable,
        } => {
            let executable = distribution_executable_path(test_distribution_executable.as_deref())?;
            let plan = RrdEngine::plan_installed_tail_repair(project, &executable)?;
            Ok(serde_json::to_string_pretty(&plan)?.into())
        }
        RepairAction::Apply {
            project,
            plan,
            expect,
            test_distribution_executable,
        } => {
            let executable = distribution_executable_path(test_distribution_executable.as_deref())?;
            if std::fs::symlink_metadata(plan)?.file_type().is_symlink() {
                return Err("repair plan must not be a symlink".into());
            }
            let file = std::fs::File::open(plan)?;
            let metadata = file.metadata()?;
            if !metadata.is_file() || metadata.len() > MAX_INSTALLATION_PLAN_BYTES {
                return Err("repair plan must be a bounded regular file".into());
            }
            let mut bytes = Vec::with_capacity(metadata.len() as usize);
            file.take(MAX_INSTALLATION_PLAN_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 != metadata.len() {
                return Err("repair plan changed while reading".into());
            }
            let plan: rrd_engine::InstalledTailRepairPlan = serde_json::from_slice(&bytes)?;
            let result =
                RrdEngine::apply_installed_tail_repair(project, &executable, &plan, expect)?;
            tracing::info!(target: "rrflow::installed", operation = "repair.apply",
                plan_sha256 = %result.plan_sha256, discarded_bytes = result.storage.discarded_bytes,
                idempotent_replay = result.idempotent_replay, "installed tail repair completed");
            Ok(serde_json::to_string_pretty(&result)?.into())
        }
    }
}

fn restore(action: &RestoreAction, json: bool) -> Result<Execution, BoxError> {
    match action {
        RestoreAction::Plan {
            project,
            restore,
            backup,
            test_distribution_executable,
        } => {
            let started = Instant::now();
            let executable = distribution_executable_path(test_distribution_executable.as_deref())?;
            let restore_id = CanonicalId::new(restore)?;
            let plan =
                RrdEngine::plan_restore_publication(project, &restore_id, backup, &executable)?;
            tracing::info!(
                target: "rrflow::installed",
                operation = "restore.plan",
                plan_sha256 = %plan.plan_sha256,
                restore_id = %plan.restore_id,
                backup_sha256 = %plan.backup_sha256,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "installed lifecycle operation completed"
            );
            // Like an installation plan, a publication plan is the exact JSON
            // document `restore apply` accepts, whatever the rendering flag.
            Ok(serde_json::to_string_pretty(&plan)?.into())
        }
        RestoreAction::Apply {
            project,
            plan,
            expect,
            test_distribution_executable,
        } => {
            let started = Instant::now();
            let executable = distribution_executable_path(test_distribution_executable.as_deref())?;
            let plan = read_publication_plan(plan)?;
            let result = RrdEngine::apply_restore_publication(
                project,
                &plan,
                expect,
                wall_clock_millis()?,
                &executable,
            )?;
            tracing::info!(
                target: "rrflow::installed",
                operation = "restore.apply",
                plan_sha256 = %result.plan_sha256,
                runtime_cursor = result.runtime_cursor,
                control_journal_sequence = result.control_journal_sequence,
                idempotent_replay = result.idempotent_replay,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "installed lifecycle operation completed"
            );
            let text = render(
                &result,
                json,
                format!(
                    "RRFlow restore published: {}\npublished root: {}\nquarantined root: {}\nruntime cursor: {}\ncontrol journal: {}\nidempotent replay: {}",
                    result.plan_sha256,
                    result.published_root,
                    result.quarantine_root,
                    result.runtime_cursor,
                    result.control_journal_sequence,
                    result.idempotent_replay,
                ),
            )?;
            Ok(Execution {
                text,
                success: true,
            })
        }
    }
}

fn version(json: bool) -> Result<Execution, BoxError> {
    let report = VersionReport {
        product: "RRFlow",
        product_version: env!("CARGO_PKG_VERSION"),
        protocol: PROTOCOL,
        protocol_version: PROTOCOL_VERSION,
        installation_plan_contract_version: rrd_contract::INSTALLATION_PLAN_CONTRACT_VERSION,
        release_stage: "pre-release",
        build: crate::build_identity::report()?,
    };
    Ok(render(
        &report,
        json,
        format!(
            "RRFlow {} (pre-release; {} protocol v{})",
            report.product_version, report.protocol, report.protocol_version
        ),
    )?
    .into())
}

fn install(
    action: &InstallAction,
    json: bool,
    instance_registry: RegistryOptions<'_>,
) -> Result<Execution, BoxError> {
    match action {
        InstallAction::Identity => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "installation_identity": RrdEngine::generate_installation_identity()?,
        }))?
        .into()),
        InstallAction::AttunementSchema => {
            Ok(serde_json::to_string_pretty(&rrd_engine::attunement_proposal_schema()?)?.into())
        }
        InstallAction::Discover { project, profile } => {
            let discovery = RrdEngine::discover_nested_repositories(project, profile)?;
            Ok(serde_json::to_string_pretty(&discovery)?.into())
        }
        InstallAction::Plan {
            project,
            mode,
            identity,
            profile,
            configuration,
            attunement_proposal,
            allow_project_source_read,
            follow_interval_ms,
            test_distribution_executable,
        } => {
            let started = Instant::now();
            let executable = distribution_executable_path(test_distribution_executable.as_deref())?;
            let proposal = attunement_proposal
                .as_deref()
                .map(rrd_engine::load_attunement_proposal)
                .transpose()?;
            let preview = RrdEngine::plan_installation_with_options(
                project,
                target_kind(*mode),
                identity,
                profile,
                configuration.as_deref(),
                rrd_engine::InstallationPlanningOptions {
                    attunement_proposal: proposal,
                    allow_project_source_read: *allow_project_source_read,
                    attunement_follow: follow_interval_ms
                        .map(|interval_ms| rrd_contract::AttunementFollowPolicy { interval_ms }),
                },
                &executable,
            )?;
            tracing::info!(
                target: "rrflow::installed",
                operation = "install.plan",
                plan_sha256 = %preview.installation.plan_sha256,
                profile_sha256 = %preview.installation.profile_sha256,
                action_count = preview.installation.actions.len(),
                managed_path_count = preview.installation.managed_paths.len(),
                elapsed_ms = started.elapsed().as_millis() as u64,
                "installed lifecycle operation completed"
            );
            // A plan is an executable machine contract even when the operator
            // did not request global JSON rendering, so stdout is always the
            // exact JSON document accepted by `install apply`.
            Ok(serde_json::to_string_pretty(&preview)?.into())
        }
        InstallAction::Apply {
            project,
            mode,
            plan,
            expect,
            test_distribution_executable,
        } => {
            let started = Instant::now();
            let executable = distribution_executable_path(test_distribution_executable.as_deref())?;
            let preview = read_installation_plan(plan)?;
            let result = RrdEngine::apply_installation(
                project,
                target_kind(*mode),
                &preview,
                expect,
                &executable,
            )?;
            tracing::info!(
                target: "rrflow::installed",
                operation = "install.apply",
                plan_sha256 = %result.plan_sha256,
                runtime_cursor = result.runtime_cursor,
                control_journal_sequence = result.control_journal_sequence,
                idempotent_replay = result.idempotent_replay,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "installed lifecycle operation completed"
            );
            let map = project_map_line(project);
            let registry = update_registry(instance_registry, "install.apply", |registry| {
                registry.record_installed(project)
            });
            let text = render(
                &InstallApplyReport {
                    installation: &result,
                    instance_registry: &registry,
                },
                json,
                format!(
                    "RRFlow installed: {}\ninstance: {}\nruntime cursor: {}\ncontrol journal: {}\nidempotent replay: {}\n{map}\n{}",
                    result.plan_sha256,
                    preview.installation.target.instance_id,
                    result.runtime_cursor,
                    result.control_journal_sequence,
                    result.idempotent_replay,
                    registry.line(),
                ),
            )?;
            Ok(Execution {
                text,
                success: true,
            })
        }
    }
}

/// Refresh the project's map and describe the outcome in one line. A map is
/// a replaceable projection, so a failure is reported and never blocks.
fn project_map_line(project: &Path) -> String {
    match RrdEngine::refresh_project_map(project) {
        Ok(map) => format!(
            "project map: {} ({} files{})",
            map.path.display(),
            map.files,
            if map.changed { ", updated" } else { "" }
        ),
        Err(error) => {
            tracing::warn!(
                target: "rrflow::installed",
                operation = "project_map.refresh",
                error = %error,
                "project map was not refreshed"
            );
            format!("project map: not refreshed ({error})")
        }
    }
}

/// Record this project in the user's instance registry and describe the
/// outcome. The registry is a candidate list with no authority, so a registry
/// that is missing, unreadable or unwritable is reported and never blocks.
fn update_registry(
    options: RegistryOptions<'_>,
    operation: &'static str,
    update: impl FnOnce(&InstanceRegistry) -> Result<InstanceRegistration, ServiceError>,
) -> RegistryUpdate {
    let registry = match resolve_registry(options) {
        Ok(registry) => registry,
        Err(error) => {
            tracing::warn!(
                target: "rrflow::installed",
                operation,
                error = %error,
                "instance registry was not found"
            );
            return RegistryUpdate {
                status: "not_updated",
                registry_root: None,
                reason: Some(error.to_string()),
                other_project_roots: Vec::new(),
            };
        }
    };
    let registry_root = Some(registry.root().display().to_string());
    match update(&registry) {
        Ok(registration) => RegistryUpdate {
            status: "recorded",
            registry_root,
            reason: None,
            other_project_roots: registration.other_project_roots,
        },
        Err(error) => {
            tracing::warn!(
                target: "rrflow::installed",
                operation,
                error = %error,
                "instance registry was not updated"
            );
            RegistryUpdate {
                status: "not_updated",
                registry_root,
                reason: Some(error.to_string()),
                other_project_roots: Vec::new(),
            }
        }
    }
}

fn resolve_registry(options: RegistryOptions<'_>) -> Result<InstanceRegistry, ServiceError> {
    InstanceRegistry::resolve(options.root, options.lock_wait_ms, &|name| {
        std::env::var_os(name)
    })
}

fn serve(
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
fn engram_open_elsewhere(project: &Path, error: ServiceError) -> BoxError {
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

/// Ask the process serving this project to stop and wait until the project
/// records that it has. The request is a file in the project's `.rrflow`
/// directory, so whoever may write that directory may stop its instance.
fn stop(
    project: &Path,
    wait_limit: Duration,
    poll_interval: Duration,
    json: bool,
) -> Result<Execution, BoxError> {
    let started = Instant::now();
    let locator = RrdEngine::read_project_locator(project)?;
    let project_root = std::fs::canonicalize(project)?.display().to_string();
    let mut report = StopReport {
        status: "not_serving",
        project_root,
        instance_id: locator.identity.instance_id.to_string(),
        url: None,
        process_id: None,
        replaced_by_url: None,
        replaced_by_process_id: None,
        waited_ms: 0,
        wait_limit_ms: wait_limit.as_millis() as u64,
    };
    if let Some(request) = RrdEngine::request_serving_stop(project)? {
        report.url = Some(request.record().url.clone());
        report.process_id = Some(request.record().process_id);
        // The serving record decides, never the request file: the engine
        // reports the stop as finished only once the record is withdrawn.
        report.status = match request.wait(wait_limit, poll_interval)? {
            ServingStopState::Stopped => "stopped",
            ServingStopState::Replaced(record) => {
                report.replaced_by_url = Some(record.url);
                report.replaced_by_process_id = Some(record.process_id);
                "replaced"
            }
            ServingStopState::Pending => "timed_out",
        };
    }
    report.waited_ms = started.elapsed().as_millis() as u64;
    tracing::info!(
        target: "rrflow::installed",
        operation = "stop",
        instance_id = %report.instance_id,
        status = report.status,
        elapsed_ms = report.waited_ms,
        "installed lifecycle operation completed"
    );
    let endpoint = |url: &Option<String>, process: Option<u32>| {
        format!(
            "{} (process {})",
            url.as_deref().unwrap_or("an unrecorded endpoint"),
            process.map_or_else(|| "unknown".into(), |process| process.to_string()),
        )
    };
    let recorded = endpoint(&report.url, report.process_id);
    let human = match report.status {
        "stopped" => format!(
            "RRFlow stopped: {recorded}\ninstance: {}\nwaited: {} ms",
            report.instance_id, report.waited_ms
        ),
        "not_serving" => format!(
            "RRFlow is not serving this project: no serving record is published\ninstance: {}",
            report.instance_id
        ),
        "replaced" => format!(
            "RRFlow at {recorded} no longer serves this project, but another process now does: {}\ninstance: {}\nRun `rrflow stop --project` again to stop that one.",
            endpoint(&report.replaced_by_url, report.replaced_by_process_id),
            report.instance_id
        ),
        _ => format!(
            "RRFlow stop was not confirmed within {} ms: {recorded} still publishes its serving record. The request stays in place: a live server stops when it reads it, and the next `rrflow serve` discards it.\ninstance: {}\nRun `rrflow ready --project` to prove whether that process is live.",
            report.wait_limit_ms, report.instance_id
        ),
    };
    Ok(Execution {
        text: render(&report, json, human)?,
        success: matches!(report.status, "stopped" | "not_serving"),
    })
}

/// List this user's registered instances with what each project records now.
/// The list holds candidates; `rrflow ready` proves one.
fn instances(json: bool, instance_registry: RegistryOptions<'_>) -> Result<Execution, BoxError> {
    let listing = resolve_registry(instance_registry)?.list()?;
    tracing::info!(
        target: "rrflow::installed",
        operation = "instances",
        instance_count = listing.instances.len(),
        unreadable_count = listing.unreadable.len(),
        "installed lifecycle operation completed"
    );
    let human = instances_text(&listing);
    Ok(render(&listing, json, human)?.into())
}

fn instances_text(listing: &InstanceRegistryListing) -> String {
    let mut text = if listing.instances.is_empty() {
        format!(
            "RRFlow instances: none registered (registry: {})",
            listing.registry_root
        )
    } else {
        format!(
            "RRFlow instances: {} (registry: {})",
            listing.instances.len(),
            listing.registry_root
        )
    };
    if let Some(reason) = &listing.read_without_lock {
        text.push_str(&format!("\nregistry read without its lock: {reason}"));
    }
    for instance in &listing.instances {
        let serving = match (&instance.endpoint, instance.process_id) {
            (Some(url), Some(process)) => format!("{url} (process {process})"),
            (Some(url), None) => url.clone(),
            (None, _) if instance.serving_record_present => "present but unusable".into(),
            (None, _) => "none".into(),
        };
        text.push_str(&format!(
            "\n{}\n  project: {}{}\n  locator: {}\n  serving record: {serving}",
            instance.entry.instance_id,
            instance.entry.project_root,
            if instance.project_present {
                ""
            } else {
                " (missing)"
            },
            if instance.locator_present {
                "present"
            } else {
                "absent"
            },
        ));
        for problem in &instance.problems {
            text.push_str(&format!("\n  problem: {problem}"));
        }
    }
    for entry in &listing.unreadable {
        text.push_str(&format!(
            "\nunreadable registry entry {}: {}",
            entry.file, entry.reason
        ));
    }
    text
}

fn ready(
    project: &Path,
    address: Option<std::net::SocketAddr>,
    json: bool,
) -> Result<Execution, BoxError> {
    let started = Instant::now();
    let now = wall_clock_millis()?;
    let invocation = format!("{now}-{}", std::process::id());
    let locator = RrdEngine::read_project_locator(project)?;
    let (address, recorded) = match address {
        Some(address) => (address, false),
        None => {
            let record = RrdEngine::read_serving_record(project)?.ok_or(
                "no served endpoint is recorded for this project; start it with `rrflow serve`",
            )?;
            (record.address()?, true)
        }
    };
    let project_root = std::fs::canonicalize(project)?.display().to_string();
    let credential = RrdEngine::read_installed_api_key(project)?;
    let client = RrdClient::connect_local(
        address,
        locator.identity.instance_id,
        ClientConfig::default(),
    )?;
    let proof = runtime()?.block_on(async {
        let session = client
            .create_session(
                credential.principal_id.clone(),
                credential.credential(),
                CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 30_000,
                        absolute_timeout_ms: 60_000,
                        max_open_transactions: 1,
                    },
                },
                RequestOptions::mutation(
                    &format!("ready-session-request-{invocation}"),
                    &format!("ready-session-operation-{invocation}"),
                    &format!("ready-session-idempotency-{invocation}"),
                )?,
            )
            .await?;
        let discovery = async {
            Ok::<_, rrd_client::Error>((
                client.readiness().await?,
                client.capabilities().await?,
                client.endpoint_catalogue().await?,
                client.openapi_document().await?,
            ))
        }
        .await;
        let closed = client
            .close_session(
                &session,
                CloseSession {},
                RequestOptions::mutation(
                    &format!("ready-close-request-{invocation}"),
                    &format!("ready-close-operation-{invocation}"),
                    &format!("ready-close-idempotency-{invocation}"),
                )?,
            )
            .await;
        let discovery = discovery?;
        closed?;
        Ok::<_, rrd_client::Error>(discovery)
    });
    let (readiness, capabilities, endpoint_catalogue, openapi) =
        proof.map_err(|error| -> BoxError {
            if recorded {
                format!("the served endpoint recorded at http://{address} is not ready: {error}")
                    .into()
            } else {
                error.into()
            }
        })?;
    let openapi_sha256 = digest::sha256_hex(&serde_json::to_vec(&openapi)?);
    let report = UiDiscoveryReport {
        status: "ready",
        url: format!("http://{address}"),
        project_root,
        openapi_sha256,
        authentication: AuthenticatedReady {
            principal_id: credential.principal_id.to_string(),
            session_created: true,
            session_closed: true,
        },
        readiness,
        capabilities,
        endpoint_catalogue,
        openapi,
    };
    tracing::info!(
        target: "rrflow::installed",
        operation = "ready",
        instance_id = %report.capabilities.instance.id,
        runtime_cursor = report.readiness.runtime_cursor,
        endpoint_count = report.endpoint_catalogue.endpoints.len(),
        websocket_endpoint_count = report.endpoint_catalogue.websocket_endpoints.len(),
        openapi_sha256 = %report.openapi_sha256,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "installed lifecycle operation completed"
    );
    let human = format!(
        "RRFlow ready at {}\ninstance: {}\nruntime cursor: {}\nHTTP operations: {}\nWebSocket operations: {}\nOpenAPI: {}",
        report.url,
        report.capabilities.instance.id,
        report.readiness.runtime_cursor,
        report.endpoint_catalogue.endpoints.len(),
        report.endpoint_catalogue.websocket_endpoints.len(),
        report.openapi_sha256,
    );
    Ok(Execution {
        text: render(&report, json, human)?,
        success: true,
    })
}

fn verify(
    project: &Path,
    level: VerifyLevel,
    distribution_executable: Option<&Path>,
    json: bool,
) -> Result<Execution, BoxError> {
    let started = Instant::now();
    let executable = distribution_executable_path(distribution_executable)?;
    let report = match level {
        VerifyLevel::Quick => RrdEngine::inspect_installed(project, &executable)?,
    };
    tracing::info!(
        target: "rrflow::installed",
        operation = "verify.quick",
        instance_id = %report.instance_id,
        plan_sha256 = %report.plan_sha256,
        runtime_cursor = report.runtime_cursor,
        control_journal_sequence = report.control_journal_sequence,
        attunement_source_current = report.attunement_source_current,
        clock_status = ?report.clock.status,
        clock_observed_at_unix_ms = report
            .clock
            .observation
            .as_ref()
            .map(|observation| observation.observed_at_unix_ms),
        clock_anchor_unix_ms = report.clock.anchor.observed_at_unix_ms,
        clock_rollback_ms = report.clock.rollback_ms,
        clock_maximum_rollback_ms = report.clock.maximum_rollback_ms,
        physical_file_count = report.physical.physical.files.len(),
        physical_bytes = report.physical.physical.total_bytes,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "installed lifecycle operation completed"
    );
    let passed_checks = report.checks.iter().filter(|check| check.passed).count();
    let success = report.status == InstallationVerificationStatus::Passed;
    let human = format!(
        "RRFlow installation verification: {:?}\ninstance: {}\nplan: {}\nattunement source current: {}\nclock: {:?}\nclock anchor: {}\nclock rollback: {:?} ms\nruntime cursor: {}\ncontrol journal: {}\nchecks: {}/{} passed",
        report.status,
        report.instance_id,
        report.plan_sha256,
        report.attunement_source_current,
        report.clock.status,
        report.clock.anchor.observed_at_unix_ms,
        report.clock.rollback_ms,
        report.runtime_cursor,
        report.control_journal_sequence,
        passed_checks,
        report.checks.len(),
    );
    Ok(Execution {
        text: render(&report, json, human)?,
        success,
    })
}

fn wall_clock_millis() -> Result<u64, BoxError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| std::io::Error::other("host clock is before the Unix epoch"))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|_| std::io::Error::other("host clock is outside RRFlow's range"))?;
    if millis == 0 {
        return Err(std::io::Error::other("host clock is at the Unix epoch").into());
    }
    Ok(millis)
}

fn read_installation_plan(path: &Path) -> Result<InstallationPreview, BoxError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_INSTALLATION_PLAN_BYTES
    {
        return Err("installation plan must be a bounded regular file, not a symlink".into());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    std::fs::File::open(path)?
        .take(MAX_INSTALLATION_PLAN_BYTES + 1)
        .read_to_end(&mut bytes)?;
    let preview: InstallationPreview = serde_json::from_slice(&bytes)?;
    preview.validate()?;
    Ok(preview)
}

fn read_publication_plan(path: &Path) -> Result<RestorePublicationPlan, BoxError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_INSTALLATION_PLAN_BYTES
    {
        return Err(
            "restore publication plan must be a bounded regular file, not a symlink".into(),
        );
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    std::fs::File::open(path)?
        .take(MAX_INSTALLATION_PLAN_BYTES + 1)
        .read_to_end(&mut bytes)?;
    let plan: RestorePublicationPlan = serde_json::from_slice(&bytes)?;
    plan.validate()?;
    Ok(plan)
}

fn target_kind(mode: InstallMode) -> InstallationTargetKind {
    match mode {
        InstallMode::Fresh => InstallationTargetKind::FreshProject,
        InstallMode::Existing => InstallationTargetKind::ExistingProject,
    }
}

fn distribution_executable_path(path: Option<&Path>) -> Result<PathBuf, BoxError> {
    if let Some(path) = path {
        if !cfg!(debug_assertions) {
            return Err("the test distribution surrogate is disabled in release builds".into());
        }
        return Ok(path.to_owned());
    }
    std::env::current_exe().map_err(Into::into)
}

fn runtime() -> Result<tokio::runtime::Runtime, BoxError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?)
}

fn render<T: Serialize>(value: &T, json: bool, human: String) -> Result<String, BoxError> {
    if json {
        Ok(serde_json::to_string_pretty(value)?)
    } else {
        Ok(human)
    }
}
