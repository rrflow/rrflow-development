//! Command definitions and execution.
//!
//! Execution is separated from `main` so that every command is exercised by
//! integration tests through the same path the operator uses, rather than
//! through a parallel test-only entry point.

use clap::{Parser, Subcommand};
use rrd_contract::{
    AssembleContext, BeginTransaction, CanonicalId, CloseSession, CommitTransaction, CorrelationId,
    CreateSession, DataReference, MemorySeatDefinition, MemoryWarp, PersistMemoryEstate,
    ProviderRepresentationDefinition, RequestContext, ResolveMemoryWarp, ResolveSeatIdentity,
    SessionLimits, MEMORY_SEAT_KIND,
};
use rrd_engine::{
    digest, Claim, Millis, Outcome, Predicate, Producer, Reader, RrdEngine, ScopeId, Subject,
    Trigger,
};

/// What a command produced for the operator and invocation audit.
pub struct Execution {
    pub text: String,
    pub detail: Option<String>,
    /// Process success is explicit so diagnostic commands can emit their full
    /// machine-readable report while still failing CI on a blocked topology.
    pub success: bool,
}

impl From<String> for Execution {
    fn from(text: String) -> Self {
        Execution {
            text,
            detail: None,
            success: true,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "rrflow",
    about = "RRFlow operator and development surface",
    long_about = "RRFlow operator and development surface. Context retrieval is composed by the engine."
)]
pub struct Cli {
    /// Database directory.
    #[arg(long, short = 'd', env = "RRFLOW_DB")]
    pub db: Option<std::path::PathBuf>,

    /// Emit JSON instead of rendered text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Identity recorded as the reader of any claim this command reads.
    #[arg(long, global = true, default_value = "operator:cli")]
    pub reader: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Inspect whether this checkout can run the canonical RRFlow development
    /// topology without split storage authority or manual hidden state.
    Dev {
        #[command(subcommand)]
        action: DevAction,
    },
    /// Record a claim.
    Assert {
        #[arg(long)]
        subject: String,
        #[arg(long)]
        predicate: String,
        #[arg(long)]
        object: String,
        /// Start of the valid-time interval. Defaults to now.
        #[arg(long)]
        valid_from: Option<Millis>,
        /// Actor recorded as the producer.
        #[arg(long, default_value = "operator:cli")]
        actor: String,
        /// Model the actor acted on behalf of.
        #[arg(long)]
        on_behalf_of: Option<String>,
    },
    /// Resolve the claim in force at an instant.
    AsOf {
        #[arg(long)]
        subject: String,
        #[arg(long)]
        predicate: String,
        /// Instant to resolve at. Defaults to now.
        #[arg(long)]
        at: Option<Millis>,
    },
    /// Every recorded version of a subject and predicate, newest first.
    History {
        #[arg(long)]
        subject: String,
        #[arg(long)]
        predicate: String,
    },
    /// Store counters and watermarks.
    Status,
    /// Derive removal candidates. Analysis only; nothing is removed.
    Gc {
        /// Inclusive lower bound of the interval considered.
        #[arg(long, default_value_t = 0)]
        since: Millis,
    },
    /// Recorded invocations, chronologically.
    Invocations {
        #[arg(long, default_value_t = 0)]
        since: Millis,
    },
    /// Execute one explicit, durably traced rrflowQL query. Raw query and
    /// parameter values are returned but never persisted as trace attributes.
    Query {
        /// Project root used to prove this database belongs to one instance.
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        /// One concrete runtime scope. Defaults to this engine's bound instance.
        #[arg(long)]
        scope: Option<String>,
        /// rrflowQL source text.
        #[arg(long)]
        ql: String,
        /// Scalar binder value as NAME=JSON. Repeatable.
        #[arg(long = "parameter")]
        parameters: Vec<String>,
        #[arg(long, default_value_t = 100_000)]
        max_scanned_changes: usize,
        #[arg(long, default_value_t = 10_000)]
        max_rows: usize,
        #[arg(long, default_value_t = 8 * 1024 * 1024)]
        max_output_bytes: usize,
        #[arg(long, default_value_t = 256)]
        max_batch_rows: usize,
    },
    /// Assemble one temporal, lexical, semantic, and graph context packet.
    Context {
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        /// Runtime scope. Defaults to this engine's bound instance.
        #[arg(long)]
        scope: Option<String>,
        #[arg(long, default_value = "")]
        query: String,
        /// Stable rrflow:// record coordinate. Cannot be combined with --seed.
        #[arg(long)]
        warp: Option<String>,
        /// Optional KIND:ID record anchor. Repeatable.
        #[arg(long = "seed")]
        seeds: Vec<String>,
        #[arg(long)]
        valid_at: Option<Millis>,
        #[arg(long, default_value_t = 2)]
        max_graph_depth: u8,
        #[arg(long, default_value_t = 32)]
        max_items: u64,
        #[arg(long, default_value_t = 256 * 1024)]
        max_output_bytes: u64,
        #[arg(long, default_value_t = 100_000)]
        max_scanned_changes: u64,
    },
    /// Persist or resolve a provider-independent RRFlow seat identity.
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },
    /// Content-authenticated logical archive and retained backup operations.
    Storage {
        #[command(subcommand)]
        action: StorageAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum IdentityAction {
    /// Persist a seat and one provider representation in the canonical log.
    Bind {
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        #[arg(long, default_value = "clyffy")]
        seat: String,
        #[arg(long, default_value = "Clyffy")]
        display_name: String,
        #[arg(
            long,
            default_value = "Persistent RRFlow reasoning and recall identity"
        )]
        purpose: String,
        /// Opaque provider name; no provider-specific contract is created.
        #[arg(long)]
        provider: String,
        /// Stable local ID for the provider identity record.
        #[arg(long)]
        provider_identity: String,
        /// Provider subject used only to derive a digest; plaintext is not stored.
        #[arg(long)]
        provider_subject: String,
        /// Stable ID for the provider-to-seat representation edge.
        #[arg(long)]
        representation: String,
    },
    /// Resolve the durable seat represented as self at one valid-time instant.
    Resolve {
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        #[arg(long, default_value = "clyffy")]
        seat: String,
        #[arg(long)]
        valid_at: Option<Millis>,
        #[arg(long, default_value_t = 100_000)]
        max_scanned_changes: u64,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DevAction {
    /// Emit every development-topology invariant and fail when a required
    /// invariant is blocked.
    Doctor {
        /// Workspace root to inspect.
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
    },
    /// Build and start one RRD authority, then report its client connection
    /// endpoint and development credential location.
    Up {
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        #[arg(long)]
        instance: Option<String>,
        #[arg(long, default_value = "127.0.0.1:9477")]
        rrd_bind: std::net::SocketAddr,
        /// Reuse companion binaries beside rrflow (or in RRFLOW_DEV_BIN_DIR).
        #[arg(long)]
        no_build: bool,
    },
    /// Probe the RRD daemon recorded by the recoverable supervisor manifest.
    Status {
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
    },
    /// Read a bounded tail of the retained RRD daemon log.
    Logs {
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        #[arg(long, default_value_t = 100)]
        lines: usize,
    },
    /// Request graceful RRD shutdown and wait for completion.
    Stop {
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        #[arg(long, default_value_t = 10_000)]
        timeout_ms: u64,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum StorageAction {
    /// Export a content-authenticated, backend-independent logical archive.
    ArchiveExport {
        #[arg(long)]
        archive: std::path::PathBuf,
    },
    /// Validate a logical archive without mutating a database.
    ArchiveInspect {
        #[arg(long)]
        archive: std::path::PathBuf,
    },
    /// Restore a logical archive into the absent `--db` root.
    ArchiveRestore {
        #[arg(long)]
        archive: std::path::PathBuf,
    },
    /// Create an authenticated catalogue entry and retained logical archive.
    BackupCreate {
        #[arg(long)]
        catalogue: std::path::PathBuf,
        #[arg(long)]
        label: String,
    },
    /// List a catalogue after authenticating it and all retained archives.
    BackupList {
        #[arg(long)]
        catalogue: std::path::PathBuf,
    },
    /// Restore one catalogued backup into the absent `--db` root.
    BackupRestore {
        #[arg(long)]
        catalogue: std::path::PathBuf,
        #[arg(long)]
        backup_id: String,
    },
}

impl Command {
    /// Stable name used in the invocation record.
    pub fn name(&self) -> &'static str {
        match self {
            Command::Dev {
                action: DevAction::Doctor { .. },
            } => "dev-doctor",
            Command::Dev {
                action: DevAction::Up { .. },
            } => "dev-up",
            Command::Dev {
                action: DevAction::Status { .. },
            } => "dev-status",
            Command::Dev {
                action: DevAction::Logs { .. },
            } => "dev-logs",
            Command::Dev {
                action: DevAction::Stop { .. },
            } => "dev-stop",
            Command::Assert { .. } => "assert",
            Command::AsOf { .. } => "as-of",
            Command::History { .. } => "history",
            Command::Status => "status",
            Command::Gc { .. } => "gc",
            Command::Invocations { .. } => "invocations",
            Command::Query { .. } => "query",
            Command::Context { .. } => "context",
            Command::Identity {
                action: IdentityAction::Bind { .. },
            } => "identity-bind",
            Command::Identity {
                action: IdentityAction::Resolve { .. },
            } => "identity-resolve",
            Command::Storage {
                action: StorageAction::ArchiveExport { .. },
            } => "storage-archive-export",
            Command::Storage {
                action: StorageAction::ArchiveInspect { .. },
            } => "storage-archive-inspect",
            Command::Storage {
                action: StorageAction::ArchiveRestore { .. },
            } => "storage-archive-restore",
            Command::Storage {
                action: StorageAction::BackupCreate { .. },
            } => "storage-backup-create",
            Command::Storage {
                action: StorageAction::BackupList { .. },
            } => "storage-backup-list",
            Command::Storage {
                action: StorageAction::BackupRestore { .. },
            } => "storage-backup-restore",
        }
    }

    pub fn trigger(&self) -> Trigger {
        Trigger::Manual
    }

    /// Arguments recorded alongside the invocation, so a recorded run can be
    /// reproduced.
    pub fn arguments(&self) -> Vec<String> {
        match self {
            Command::Assert {
                subject,
                predicate,
                object,
                valid_from,
                actor,
                on_behalf_of,
            } => {
                let mut a = vec![
                    format!("subject={subject}"),
                    format!("predicate={predicate}"),
                    format!("object={object}"),
                    format!("actor={actor}"),
                ];
                if let Some(v) = valid_from {
                    a.push(format!("valid_from={v}"));
                }
                if let Some(o) = on_behalf_of {
                    a.push(format!("on_behalf_of={o}"));
                }
                a
            }
            Command::AsOf {
                subject,
                predicate,
                at,
            } => {
                let mut a = vec![
                    format!("subject={subject}"),
                    format!("predicate={predicate}"),
                ];
                if let Some(t) = at {
                    a.push(format!("at={t}"));
                }
                a
            }
            Command::History { subject, predicate } => {
                vec![
                    format!("subject={subject}"),
                    format!("predicate={predicate}"),
                ]
            }
            Command::Status => Vec::new(),
            Command::Gc { since } => vec![format!("since={since}")],
            Command::Invocations { since } => vec![format!("since={since}")],
            Command::Query {
                root,
                scope,
                ql,
                parameters,
                max_scanned_changes,
                max_rows,
                max_output_bytes,
                max_batch_rows,
            } => vec![
                format!("root={}", root.display()),
                format!("scope={}", scope.as_deref().unwrap_or("bound-instance")),
                format!("query_digest={}", digest::sha256_hex(ql.as_bytes())),
                format!(
                    "parameter_input_digest={}",
                    digest::sha256_hex(parameters.join("\0").as_bytes())
                ),
                format!("parameter_count={}", parameters.len()),
                format!("max_scanned_changes={max_scanned_changes}"),
                format!("max_rows={max_rows}"),
                format!("max_output_bytes={max_output_bytes}"),
                format!("max_batch_rows={max_batch_rows}"),
            ],
            Command::Context {
                root,
                scope,
                query,
                warp,
                seeds,
                valid_at,
                max_graph_depth,
                max_items,
                max_output_bytes,
                max_scanned_changes,
            } => vec![
                format!("root={}", root.display()),
                format!("scope={}", scope.as_deref().unwrap_or("bound-instance")),
                format!("query_sha256={}", digest::sha256_hex(query.as_bytes())),
                format!("warp={}", warp.as_deref().unwrap_or("none")),
                format!("seed_count={}", seeds.len()),
                format!("valid_at={}", valid_at.unwrap_or(0)),
                format!("max_graph_depth={max_graph_depth}"),
                format!("max_items={max_items}"),
                format!("max_output_bytes={max_output_bytes}"),
                format!("max_scanned_changes={max_scanned_changes}"),
            ],
            Command::Identity {
                action:
                    IdentityAction::Bind {
                        root,
                        seat,
                        display_name,
                        purpose,
                        provider,
                        provider_identity,
                        provider_subject,
                        representation,
                    },
            } => vec![
                format!("root={}", root.display()),
                format!("seat={seat}"),
                format!("display_name={display_name}"),
                format!("purpose_sha256={}", digest::sha256_hex(purpose.as_bytes())),
                format!("provider={provider}"),
                format!("provider_identity={provider_identity}"),
                format!(
                    "provider_subject_sha256={}",
                    digest::sha256_hex(provider_subject.as_bytes())
                ),
                format!("representation={representation}"),
            ],
            Command::Identity {
                action:
                    IdentityAction::Resolve {
                        root,
                        seat,
                        valid_at,
                        max_scanned_changes,
                    },
            } => vec![
                format!("root={}", root.display()),
                format!("seat={seat}"),
                format!("valid_at={}", valid_at.unwrap_or(0)),
                format!("max_scanned_changes={max_scanned_changes}"),
            ],
            Command::Storage {
                action:
                    StorageAction::ArchiveExport { archive }
                    | StorageAction::ArchiveInspect { archive }
                    | StorageAction::ArchiveRestore { archive },
            } => {
                vec![format!("archive={}", archive.display())]
            }
            Command::Storage {
                action: StorageAction::BackupCreate { catalogue, label },
            } => vec![
                format!("catalogue={}", catalogue.display()),
                format!("label={label}"),
            ],
            Command::Storage {
                action: StorageAction::BackupList { catalogue },
            } => {
                vec![format!("catalogue={}", catalogue.display())]
            }
            Command::Storage {
                action:
                    StorageAction::BackupRestore {
                        catalogue,
                        backup_id,
                    },
            } => vec![
                format!("catalogue={}", catalogue.display()),
                format!("backup_id={backup_id}"),
            ],
            Command::Dev {
                action: DevAction::Doctor { root },
            } => vec![format!("root={}", root.display())],
            Command::Dev {
                action:
                    DevAction::Up {
                        root,
                        instance,
                        rrd_bind,
                        no_build,
                    },
            } => {
                let mut arguments = vec![
                    format!("root={}", root.display()),
                    format!("rrd_bind={rrd_bind}"),
                    format!("no_build={no_build}"),
                ];
                if let Some(instance) = instance {
                    arguments.push(format!("instance={instance}"));
                }
                arguments
            }
            Command::Dev {
                action: DevAction::Status { root },
            } => vec![format!("root={}", root.display())],
            Command::Dev {
                action: DevAction::Logs { root, lines },
            } => vec![format!("root={}", root.display()), format!("lines={lines}")],
            Command::Dev {
                action: DevAction::Stop { root, timeout_ms },
            } => vec![
                format!("root={}", root.display()),
                format!("timeout_ms={timeout_ms}"),
            ],
        }
    }
}

/// Executes topology and storage operations that own their open/restore
/// boundary instead of travelling through the invocation wrapper.
pub fn execute_offline(
    db: &std::path::Path,
    command: &Command,
    now: Millis,
    json: bool,
) -> Option<Result<Execution, Box<dyn std::error::Error>>> {
    if let Command::Dev { action } = command {
        return Some((|| match action {
            DevAction::Doctor { root } => {
                let report = crate::dev::doctor(root)?;
                let success = report.ready;
                let text = if json {
                    serde_json::to_string_pretty(&report)?
                } else {
                    report.render()
                };
                Ok(Execution {
                    text,
                    detail: Some(format!(
                        "{} passed, {} blocked, {} warnings",
                        report.passed, report.blocked, report.warnings
                    )),
                    success,
                })
            }
            DevAction::Up {
                root,
                instance,
                rrd_bind,
                no_build,
            } => {
                let report = crate::dev::supervisor::up(
                    crate::dev::supervisor::UpOptions {
                        root: root.clone(),
                        instance: instance.clone(),
                        rrd_bind: *rrd_bind,
                        no_build: *no_build,
                    },
                    now,
                )?;
                let text = if json {
                    serde_json::to_string_pretty(&report)?
                } else {
                    report.render()
                };
                Ok(text.into())
            }
            DevAction::Status { root } => {
                let report = crate::dev::supervisor::status(root)?;
                let success = report.rrd_ready;
                let text = if json {
                    serde_json::to_string_pretty(&report)?
                } else {
                    report.render()
                };
                Ok(Execution {
                    text,
                    detail: Some(format!("topology status: {}", report.status)),
                    success,
                })
            }
            DevAction::Logs { root, lines } => {
                Ok(crate::dev::supervisor::logs(root, *lines)?.into())
            }
            DevAction::Stop { root, timeout_ms } => {
                let report = crate::dev::supervisor::stop(
                    root,
                    std::time::Duration::from_millis(*timeout_ms),
                )?;
                let text = if json {
                    serde_json::to_string_pretty(&report)?
                } else {
                    report.render()
                };
                Ok(text.into())
            }
        })());
    }
    let action = match command {
        Command::Storage { action } => action,
        _ => return None,
    };
    Some((|| match action {
        StorageAction::ArchiveExport { archive } => {
            let engine = RrdEngine::open_project_store(db)?;
            let inventory = engine.export_logical_archive(archive)?;
            let text = if json {
                serde_json::to_string_pretty(&inventory)?
            } else {
                format!(
                    "logical archive: {} actions / {} claims / {} runtime mutations / sha256 {}\nArchive: {}",
                    inventory.action_count,
                    inventory.claim_sequence,
                    inventory.runtime_mutations,
                    inventory.archive_sha256,
                    archive.display()
                )
            };
            Ok(text.into())
        }
        StorageAction::ArchiveInspect { archive } => {
            let inventory = RrdEngine::inspect_logical_archive(archive)?;
            let text = if json {
                serde_json::to_string_pretty(&inventory)?
            } else {
                format!(
                    "logical archive verified: {} actions / claim sequence {} / runtime cursor {} / sha256 {}",
                    inventory.action_count,
                    inventory.claim_sequence,
                    inventory.runtime_cursor,
                    inventory.archive_sha256
                )
            };
            Ok(text.into())
        }
        StorageAction::ArchiveRestore { archive } => {
            let report = RrdEngine::restore_logical_archive(archive, db, now)?;
            let text = if json {
                serde_json::to_string_pretty(&report)?
            } else {
                format!(
                    "logical archive restored and reopened: {}\nTarget: {}",
                    report.inventory.archive_sha256,
                    report.target.display()
                )
            };
            Ok(text.into())
        }
        StorageAction::BackupCreate { catalogue, label } => {
            let engine = RrdEngine::open_project_store(db)?;
            let entry = engine.create_logical_backup(catalogue, label, now)?;
            let text = if json {
                serde_json::to_string_pretty(&entry)?
            } else {
                format!(
                    "backup {} retained as {}\nCoverage: claims and typed runtime included; object payloads referenced only; application-complete=false",
                    entry.backup_id, entry.archive_file
                )
            };
            Ok(text.into())
        }
        StorageAction::BackupList { catalogue } => {
            let verified = RrdEngine::verify_backup_catalogue(catalogue)?;
            let text = if json {
                serde_json::to_string_pretty(&verified)?
            } else if verified.backups.is_empty() {
                "backup catalogue is empty".into()
            } else {
                verified
                    .backups
                    .iter()
                    .map(|entry| {
                        format!(
                            "{} {} at {} (claims {}, runtime {}, application-complete={})",
                            entry.backup_id,
                            entry.label,
                            entry.created_at,
                            entry.archive.claim_sequence,
                            entry.archive.runtime_cursor,
                            entry.application_complete
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(text.into())
        }
        StorageAction::BackupRestore {
            catalogue,
            backup_id,
        } => {
            let report = RrdEngine::restore_catalogued_backup(catalogue, backup_id, db, now)?;
            let text = if json {
                serde_json::to_string_pretty(&report)?
            } else {
                format!(
                    "catalogued backup restored and reopened: {}\nTarget: {}",
                    backup_id,
                    report.target.display()
                )
            };
            Ok(text.into())
        }
    })())
}

/// Executes one command.
///
/// `now` is supplied rather than read here, so that tests are deterministic and
/// the clock enters at exactly one place (`main`).
pub fn execute(
    store: &RrdEngine,
    command: &Command,
    reader: &Reader,
    now: Millis,
    json: bool,
) -> Result<Execution, Box<dyn std::error::Error>> {
    match command {
        Command::Query {
            root,
            scope,
            ql,
            parameters,
            max_scanned_changes,
            max_rows,
            max_output_bytes,
            max_batch_rows,
        } => {
            verify_instance_store(store, root)?;
            let scope = ScopeId::new(
                scope
                    .clone()
                    .unwrap_or_else(|| format!("instance:{}", store.instance_id())),
            )?;
            let parameter_json = query_parameter_object(parameters)?;
            let parameters = rrd_engine::query_parameters_from_json(&parameter_json)?;
            let budget = rrd_engine::ExecutionBudget {
                max_scanned_changes: *max_scanned_changes,
                max_rows: *max_rows,
                max_output_bytes: *max_output_bytes,
                max_batch_rows: *max_batch_rows,
                ..rrd_engine::ExecutionBudget::default()
            };
            let result = store.execute_operator_query(
                scope,
                ql,
                &parameters,
                &budget,
                reader.as_str(),
                now,
            )?;
            let text = if json {
                serde_json::to_string_pretty(&result)?
            } else {
                let mut lines = vec![format!(
                    "plan {} read={} rows={} scanned={} validation={} validation_reads={} proof_nodes={} bytes={} truncated={}",
                    result.plan.digest,
                    result.execution.known_at_cursor,
                    result.execution.returned_rows,
                    result.execution.scanned_changes,
                    result.execution.stamp_validation,
                    result.execution.stamp_validation_max_changes,
                    result.execution.stamp_validation_proof_nodes,
                    result.execution.output_bytes,
                    result.execution.truncated,
                )];
                for row in result
                    .execution
                    .batches
                    .iter()
                    .flat_map(|batch| &batch.rows)
                {
                    lines.push(serde_json::to_string(row)?);
                }
                lines.join("\n")
            };
            return Ok(Execution {
                text,
                detail: Some(format!("plan={}", result.plan.digest)),
                success: true,
            });
        }

        Command::Context {
            root,
            scope,
            query,
            warp,
            seeds,
            valid_at,
            max_graph_depth,
            max_items,
            max_output_bytes,
            max_scanned_changes,
        } => {
            verify_instance_store(store, root)?;
            if warp.is_some() && !seeds.is_empty() {
                return Err("--warp cannot be combined with --seed".into());
            }
            let scope = scope
                .clone()
                .unwrap_or_else(|| format!("instance:{}", store.instance_id()));
            let session_key = CorrelationId::new(format!("cli-context-session-{now}"))?;
            let lease = store.create_session(
                &CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 30_000,
                        absolute_timeout_ms: 60_000,
                        max_open_transactions: 1,
                    },
                },
                &session_key,
                now,
                "cli-context-session",
                "cli-context-session",
            )?;
            let assembled = if let Some(uri) = warp {
                store
                    .resolve_memory_warp(
                        &lease.session_id,
                        &lease.token,
                        &ResolveMemoryWarp {
                            scope,
                            uri: uri.clone(),
                            query: query.clone(),
                            valid_at: valid_at.unwrap_or(now),
                            max_graph_depth: *max_graph_depth,
                            max_items: *max_items,
                            max_output_bytes: *max_output_bytes,
                            max_scanned_changes: *max_scanned_changes,
                        },
                        now,
                        "cli-context-warp",
                        "cli-context-warp",
                    )
                    .map(|resolved| {
                        let digest = resolved.context.packet_sha256.clone();
                        (serde_json::to_value(resolved), digest)
                    })
            } else {
                store
                    .assemble_context(
                        &lease.session_id,
                        &lease.token,
                        &AssembleContext {
                            scope,
                            query: query.clone(),
                            valid_at: valid_at.unwrap_or(now),
                            seeds: seeds
                                .iter()
                                .map(|seed| parse_data_reference(seed))
                                .collect::<Result<Vec<_>, _>>()?,
                            max_graph_depth: *max_graph_depth,
                            max_items: *max_items,
                            max_output_bytes: *max_output_bytes,
                            max_scanned_changes: *max_scanned_changes,
                        },
                        now,
                        "cli-context",
                        "cli-context",
                    )
                    .map(|packet| {
                        let digest = packet.packet_sha256.clone();
                        (serde_json::to_value(packet), digest)
                    })
            };
            let _ = store.close_session(
                &lease.session_id,
                &lease.token,
                &CloseSession {},
                &CorrelationId::new(format!("cli-context-close-{now}"))?,
                now,
                "cli-context-close",
                "cli-context-close",
            );
            let (value, digest) = assembled?;
            let text = if json {
                serde_json::to_string_pretty(&value?)?
            } else {
                serde_json::to_string(&value?)?
            };
            return Ok(Execution {
                text,
                detail: Some(format!("context={digest}")),
                success: true,
            });
        }

        Command::Identity {
            action:
                IdentityAction::Bind {
                    root,
                    seat,
                    display_name,
                    purpose,
                    provider,
                    provider_identity,
                    provider_subject,
                    representation,
                },
        } => {
            verify_instance_store(store, root)?;
            let session_key = CorrelationId::new(format!("cli-identity-bind-session-{now}"))?;
            let lease = store.create_session(
                &CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 30_000,
                        absolute_timeout_ms: 60_000,
                        max_open_transactions: 1,
                    },
                },
                &session_key,
                now,
                "cli-identity-bind-session",
                "cli-identity-bind-session",
            )?;
            let result = (|| {
                let seat_id = CanonicalId::new(seat.clone())?;
                let request = PersistMemoryEstate {
                    scope: format!("instance:{}", store.instance_id()),
                    seat: MemorySeatDefinition {
                        id: seat_id.clone(),
                        display_name: display_name.clone(),
                        purpose: purpose.clone(),
                    },
                    representations: vec![ProviderRepresentationDefinition {
                        id: CanonicalId::new(representation.clone())?,
                        provider_identity: CanonicalId::new(provider_identity.clone())?,
                        provider: CanonicalId::new(provider.clone())?,
                        subject_sha256: digest::sha256_hex(provider_subject.as_bytes()),
                    }],
                    valid_from: now,
                };
                let plan = store.plan_memory_estate(
                    &lease.session_id,
                    &lease.token,
                    &request,
                    now,
                    "cli-identity-plan",
                    "cli-identity-plan",
                )?;
                let begin_context = RequestContext {
                    request_id: CorrelationId::new(format!("cli-identity-begin-request-{now}"))?,
                    operation_id: CorrelationId::new(format!(
                        "cli-identity-begin-operation-{now}"
                    ))?,
                    idempotency_key: Some(CorrelationId::new(format!(
                        "cli-identity-begin-idempotency-{now}"
                    ))?),
                    deadline_unix_ms: None,
                };
                let transaction = store.begin_transaction(
                    &lease.session_id,
                    &lease.token,
                    &BeginTransaction {
                        scope: CanonicalId::new("data")?,
                        timeout_ms: 30_000,
                    },
                    &begin_context,
                    now,
                )?;
                let receipt = store.commit_transaction(
                    &lease.session_id,
                    &lease.token,
                    &transaction.transaction_id,
                    &CorrelationId::new(format!("cli-identity-commit-{now}"))?,
                    &CommitTransaction {
                        operation_sha256: plan.operation_sha256,
                        mutations: plan.mutations,
                    },
                    now,
                    "cli-identity-commit",
                    "cli-identity-commit",
                )?;
                let uri = MemoryWarp::new(
                    store.instance_id().clone(),
                    DataReference {
                        kind: CanonicalId::new(MEMORY_SEAT_KIND)?,
                        id: seat_id,
                    },
                )
                .uri();
                Ok::<_, Box<dyn std::error::Error>>((uri, receipt))
            })();
            close_cli_session(store, &lease, now, "identity-bind")?;
            let (uri, receipt) = result?;
            let text = if json {
                serde_json::to_string_pretty(&serde_json::json!({
                    "seat_uri": uri,
                    "receipt": receipt,
                }))?
            } else {
                format!(
                    "seat persisted: {uri}\nruntime cursor: {}",
                    receipt.last_runtime_cursor.unwrap_or(0)
                )
            };
            return Ok(Execution {
                text,
                detail: Some(format!("seat={uri}")),
                success: true,
            });
        }

        Command::Identity {
            action:
                IdentityAction::Resolve {
                    root,
                    seat,
                    valid_at,
                    max_scanned_changes,
                },
        } => {
            verify_instance_store(store, root)?;
            let session_key = CorrelationId::new(format!("cli-identity-resolve-session-{now}"))?;
            let lease = store.create_session(
                &CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 30_000,
                        absolute_timeout_ms: 60_000,
                        max_open_transactions: 1,
                    },
                },
                &session_key,
                now,
                "cli-identity-resolve-session",
                "cli-identity-resolve-session",
            )?;
            let identity = store.resolve_seat_identity(
                &lease.session_id,
                &lease.token,
                &ResolveSeatIdentity {
                    scope: format!("instance:{}", store.instance_id()),
                    seat_id: CanonicalId::new(seat.clone())?,
                    valid_at: valid_at.unwrap_or(now),
                    max_scanned_changes: *max_scanned_changes,
                },
                now,
                "cli-identity-resolve",
                "cli-identity-resolve",
            );
            close_cli_session(store, &lease, now, "identity-resolve")?;
            let identity = identity?;
            let text = if json {
                serde_json::to_string_pretty(&identity)?
            } else {
                format!(
                    "self: {} ({})\nrepresented by: {} provider identity(s)\nread cursor: {}",
                    identity.display_name,
                    identity.uri,
                    identity.representations.len(),
                    identity.read.runtime_cursor
                )
            };
            return Ok(Execution {
                text,
                detail: Some(format!("seat={}", identity.uri)),
                success: true,
            });
        }

        _ => {}
    }

    // Text-only commands, converted to an `Execution` in one place below.
    let text = (|| -> Result<String, Box<dyn std::error::Error>> {
        match command {
            Command::Query { .. }
            | Command::Context { .. }
            | Command::Identity { .. }
            | Command::Dev { .. }
            | Command::Storage { .. } => {
                unreachable!("handled above with an early return")
            }

            Command::Assert {
                subject,
                predicate,
                object,
                valid_from,
                actor,
                on_behalf_of,
            } => {
                let subject = Subject::new(subject.clone())?;
                let predicate = Predicate::new(predicate.clone())?;
                let claim = Claim::new(
                    subject,
                    predicate,
                    object.clone(),
                    valid_from.unwrap_or(now),
                    now,
                    Producer {
                        actor: actor.clone(),
                        on_behalf_of: on_behalf_of.clone(),
                        session: None,
                    },
                );
                let outcome = store.assert_claim(&claim)?;
                Ok(if json {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "sequence": outcome.last_sequence,
                        "claim": claim,
                    }))?
                } else {
                    format!("recorded at sequence {}", outcome.last_sequence)
                })
            }

            Command::AsOf {
                subject,
                predicate,
                at,
            } => {
                let subject = Subject::new(subject.clone())?;
                let predicate = Predicate::new(predicate.clone())?;
                let at = at.unwrap_or(now);
                let resolved = store.claim_as_of(&subject, &predicate, at)?;
                store.observe(reader, &subject, &predicate, now)?;
                Ok(match (&resolved, json) {
                    (_, true) => serde_json::to_string_pretty(&resolved)?,
                    (Some(claim), false) => format!(
                        "{} valid_from={} valid_to={} tx_time={} producer={}",
                        claim.object,
                        claim.valid_from,
                        claim
                            .valid_to
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "open".into()),
                        claim.tx_time,
                        claim.producer.actor,
                    ),
                    (None, false) => format!("no claim in force at {at}"),
                })
            }

            Command::History { subject, predicate } => {
                let subject = Subject::new(subject.clone())?;
                let predicate = Predicate::new(predicate.clone())?;
                let versions = store.claim_history(&subject, &predicate)?;
                store.observe(reader, &subject, &predicate, now)?;
                Ok(if json {
                    serde_json::to_string_pretty(&versions)?
                } else if versions.is_empty() {
                    "no versions recorded".to_string()
                } else {
                    versions
                        .iter()
                        .map(|c| {
                            format!(
                                "{:<20} valid=[{}, {}) tx={}",
                                c.object,
                                c.valid_from,
                                c.valid_to
                                    .map(|v| v.to_string())
                                    .unwrap_or_else(|| "open".into()),
                                c.tx_time
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
            }

            Command::Status => {
                let sequence = store.sequence()?;
                let invocations = store.invocation_count()?;
                let access = store.access_count()?;
                Ok(if json {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "storage_backend": store.backend_name(),
                        "claim_sequence": sequence,
                        "invocations": invocations,
                        "access_records_approximate": access,
                    }))?
                } else {
                    format!(
                        "storage backend     {}\n\
                     claim sequence      {sequence}\n\
                     invocations         {invocations}\n\
                     access records      {access} (approximate)",
                        store.backend_name()
                    )
                })
            }

            Command::Gc { since } => {
                let report = store.removal_report(*since, now)?;
                Ok(if json {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "since": report.since,
                        "evaluated_at": report.evaluated_at,
                        "candidates": report.candidates()
                            .map(|p| serde_json::json!({
                                "subject": p.subject.as_str(),
                                "predicate": p.predicate.as_str(),
                                "claim_count": p.claim_count,
                                "reason": p.reason(),
                            }))
                            .collect::<Vec<_>>(),
                        "retained": report.retained()
                            .map(|p| serde_json::json!({
                                "subject": p.subject.as_str(),
                                "predicate": p.predicate.as_str(),
                                "last_access": p.last_access,
                                "reason": p.reason(),
                            }))
                            .collect::<Vec<_>>(),
                    }))?
                } else {
                    report.render()
                })
            }

            Command::Invocations { since } => {
                let records = store.invocations_since(*since)?;
                Ok(if json {
                    serde_json::to_string_pretty(&records)?
                } else if records.is_empty() {
                    "no invocations recorded".to_string()
                } else {
                    records
                        .iter()
                        .map(|i| i.render())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
            }
        }
    })()?;
    Ok(text.into())
}

fn verify_instance_store(
    store: &RrdEngine,
    root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    store.verify_project_store(root)
}

fn close_cli_session(
    store: &RrdEngine,
    lease: &rrd_contract::SessionLease,
    now: Millis,
    operation: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    store.close_session(
        &lease.session_id,
        &lease.token,
        &CloseSession {},
        &CorrelationId::new(format!("cli-{operation}-close-{now}"))?,
        now,
        &format!("cli-{operation}-close"),
        &format!("cli-{operation}-close"),
    )?;
    Ok(())
}

fn parse_data_reference(value: &str) -> Result<DataReference, Box<dyn std::error::Error>> {
    let (kind, id) = value
        .split_once(':')
        .ok_or("context seed must use KIND:ID")?;
    Ok(DataReference {
        kind: CanonicalId::new(kind)?,
        id: CanonicalId::new(id)?,
    })
}

fn query_parameter_object(
    parameters: &[String],
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut object = serde_json::Map::new();
    for parameter in parameters {
        let (name, encoded) = parameter
            .split_once('=')
            .ok_or("query parameter must use NAME=JSON")?;
        if name.trim().is_empty() {
            return Err("query parameter name must not be empty".into());
        }
        if object.contains_key(name) {
            return Err(format!("query parameter {name:?} was supplied more than once").into());
        }
        object.insert(name.to_owned(), serde_json::from_str(encoded)?);
    }
    Ok(serde_json::Value::Object(object))
}

/// Maps an execution result onto the recorded outcome.
pub fn outcome_of(
    result: &Result<Execution, Box<dyn std::error::Error>>,
) -> (Outcome, Option<String>) {
    match result {
        Ok(execution) if execution.success => (Outcome::Ok, None),
        Ok(execution) => (
            Outcome::Error,
            execution
                .detail
                .clone()
                .or_else(|| Some("command reported an unsuccessful outcome".into())),
        ),
        Err(error) => (Outcome::Error, Some(error.to_string())),
    }
}
