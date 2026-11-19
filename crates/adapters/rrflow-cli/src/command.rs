//! The deliberately small primary RRFlow product surface.
//!
//! Data, reasoning, recall, backup, and administration operations are exposed
//! through authenticated generated clients after `serve`; the primary binary
//! does not reopen a raw database or create a second lifecycle authority.

use clap::{Parser, Subcommand, ValueEnum};

pub struct Execution {
    pub text: String,
    pub success: bool,
}

impl From<String> for Execution {
    fn from(text: String) -> Self {
        Self {
            text,
            success: true,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "rrflow",
    about = "Install, run, discover, and verify one RRFlow reasoning and recall engine",
    long_about = "RRFlow is an independently installable reasoning and recall engine. Install a sealed plan, run its authenticated service, then use generated SDK operations for governed effects."
)]
pub struct Cli {
    /// Emit machine-readable JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Directory of this user's instance registry, resolved against the
    /// working directory. Without it the absolute directory
    /// RRFLOW_INSTANCE_REGISTRY names is used, or else this user's state
    /// directory.
    #[arg(long, global = true, value_name = "DIR")]
    pub instance_registry: Option<std::path::PathBuf>,

    /// How long to wait for the instance registry's lock. Without it the
    /// milliseconds RRFLOW_INSTANCE_REGISTRY_LOCK_WAIT_MS names are used, or
    /// else 5000. A registry still locked after the wait is reported and
    /// never fails `install` or `serve`.
    #[arg(long, global = true, value_name = "MS")]
    pub instance_registry_lock_wait_ms: Option<u64>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Print the immutable RRFlow product and protocol version.
    Version,
    /// Preview or apply one explicit, content-bound installation plan.
    Install {
        #[command(subcommand)]
        action: InstallAction,
    },
    /// Start the installed engine's loopback HTTP service for SDK and UI clients.
    Serve {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long, default_value = "127.0.0.1:9477")]
        bind: std::net::SocketAddr,
        /// JSON mapping of endpoint credential references to explicit environment names.
        #[arg(long)]
        endpoint_credentials: Option<std::path::PathBuf>,
        /// How often to look for a stop request from `rrflow stop`.
        #[arg(
            long,
            value_name = "MS",
            default_value_t = rrd_engine::DEFAULT_STOP_POLL_INTERVAL_MS,
            value_parser = clap::value_parser!(u64).range(1..)
        )]
        stop_poll_interval_ms: u64,
        /// Debug-test surrogate for the distribution executable.
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
    /// Ask the instance serving a project to stop, and wait until it has.
    Stop {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        /// How long to wait for the serving process to withdraw its record.
        #[arg(long, value_name = "MS", default_value_t = rrd_engine::DEFAULT_STOP_WAIT_MS)]
        wait_ms: u64,
        /// How often to look at the project's serving record while waiting.
        #[arg(
            long,
            value_name = "MS",
            default_value_t = rrd_engine::DEFAULT_STOP_POLL_INTERVAL_MS,
            value_parser = clap::value_parser!(u64).range(1..)
        )]
        poll_interval_ms: u64,
    },
    /// List the projects this user installed or served, with what each project records now.
    Instances,
    /// Authenticate against a running engram and return UI discovery documents.
    Ready {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        /// Address to prove. Without it, the endpoint `rrflow serve` recorded
        /// for this project is proved.
        #[arg(long)]
        address: Option<std::net::SocketAddr>,
    },
    /// Perform bounded, read-only verification of an installed engram.
    Verify {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long, value_enum, default_value_t = VerifyLevel::Quick)]
        level: VerifyLevel,
        /// Debug-test surrogate for the distribution executable.
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
    /// Explicitly repair only a validated incomplete final WAL frame while stopped.
    Repair {
        #[command(subcommand)]
        action: RepairAction,
    },
    /// Preview or apply the publication of a verified restore while the engram is stopped.
    Restore {
        #[command(subcommand)]
        action: RestoreAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RepairAction {
    /// Inspect the installed engram and emit an exact plan without changing bytes.
    Plan {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
    /// Truncate only the incomplete tail approved by this exact plan digest.
    Apply {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long)]
        plan: std::path::PathBuf,
        #[arg(long)]
        expect: String,
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RestoreAction {
    /// Emit an exact, digest-bound publication plan for one staged restore and perform no writes.
    Plan {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        /// The restore id the `restore-create` operation staged.
        #[arg(long)]
        restore: String,
        /// The SHA-256 identity of the catalogued backup the restore was staged from.
        #[arg(long)]
        backup: String,
        /// Debug-test surrogate for the distribution executable.
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
    /// Publish only the supplied plan after its SHA-256 is explicitly accepted.
    Apply {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long)]
        plan: std::path::PathBuf,
        #[arg(long)]
        expect: String,
        /// Debug-test surrogate for the distribution executable.
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum InstallAction {
    /// Generate a new identity input without writing project or engine state.
    Identity,
    /// Emit the machine-readable attunement proposal shape without writes.
    AttunementSchema,
    /// Report present tracked Git submodule candidates without changing the project.
    Discover {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long, default_value = "default")]
        profile: String,
    },
    /// Emit an exact, deterministic preview and perform no project writes.
    Plan {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long, value_enum, default_value_t = InstallMode::Existing)]
        mode: InstallMode,
        /// Identity from `install identity`; retain it across previews for this installation.
        #[arg(long)]
        identity: String,
        #[arg(long, default_value = "default")]
        profile: String,
        /// Operator-authored engram ceilings sealed into the installation plan.
        #[arg(long)]
        configuration: Option<std::path::PathBuf>,
        /// Bounded JSON proposal for recognized attunement effects.
        #[arg(long)]
        attunement_proposal: Option<std::path::PathBuf>,
        /// Explicitly include the instance-scoped ProjectSourceRead grant in a new engram.
        #[arg(long, default_value_t = false)]
        allow_project_source_read: bool,
        /// Seal a policy under which `serve` keeps the project attuned: every
        /// interval it looks for changed files and, when it finds some, parses
        /// and indexes them as the installed operator.
        #[arg(long, value_name = "MS")]
        follow_interval_ms: Option<u64>,
        /// Debug-test surrogate for the distribution executable.
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
    /// Apply only the supplied plan after its SHA-256 is explicitly accepted.
    Apply {
        #[arg(long, default_value = ".")]
        project: std::path::PathBuf,
        #[arg(long, value_enum, default_value_t = InstallMode::Existing)]
        mode: InstallMode,
        #[arg(long)]
        plan: std::path::PathBuf,
        #[arg(long)]
        expect: String,
        /// Debug-test surrogate for the distribution executable.
        #[arg(long = "test-distribution-executable", hide = true)]
        test_distribution_executable: Option<std::path::PathBuf>,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    Fresh,
    Existing,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyLevel {
    Quick,
}
