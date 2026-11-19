//! The instance commands: the per-user registry an install or a serve records
//! into, `rrflow instances`, and `rrflow stop`.

use super::*;

/// Whether this command recorded its project in the user's instance registry.
/// The registry is a candidate list, so a failure is reported and never
/// blocks the command.
#[derive(Debug, Serialize)]
pub(super) struct RegistryUpdate {
    pub(super) status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) registry_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reason: Option<String>,
    /// Other project roots registered under this instance id: a copied
    /// project folder keeps its original's identity.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) other_project_roots: Vec<String>,
}

impl RegistryUpdate {
    pub(super) fn recorded(&self) -> bool {
        self.reason.is_none()
    }

    pub(super) fn line(&self) -> String {
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

#[derive(Debug, Serialize)]
pub(super) struct StopReport {
    pub(super) status: &'static str,
    pub(super) project_root: String,
    pub(super) instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) process_id: Option<u32>,
    /// The record another process published while this stop waited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) replaced_by_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) replaced_by_process_id: Option<u32>,
    pub(super) waited_ms: u64,
    pub(super) wait_limit_ms: u64,
}

/// Record this project in the user's instance registry and describe the
/// outcome. The registry is a candidate list with no authority, so a registry
/// that is missing, unreadable or unwritable is reported and never blocks.
pub(super) fn update_registry(
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

pub(super) fn resolve_registry(
    options: RegistryOptions<'_>,
) -> Result<InstanceRegistry, ServiceError> {
    InstanceRegistry::resolve(options.root, options.lock_wait_ms, &|name| {
        std::env::var_os(name)
    })
}

/// Ask the process serving this project to stop and wait until the project
/// records that it has. The request is a file in the project's `.rrflow`
/// directory, so whoever may write that directory may stop its instance.
pub(super) fn stop(
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
pub(super) fn instances(
    json: bool,
    instance_registry: RegistryOptions<'_>,
) -> Result<Execution, BoxError> {
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

pub(super) fn instances_text(listing: &InstanceRegistryListing) -> String {
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
