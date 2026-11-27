use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigurationAction {
    /// Preview a local user's exact configuration change without creating state.
    Plan {
        /// An explicit private user-configuration directory, outside project engrams.
        #[arg(long)]
        root: PathBuf,
        /// Stable source identity; retain it when binding projects or updating policies.
        #[arg(long)]
        source: String,
        #[arg(long)]
        expected_revision: u64,
        /// JSON ConfigurationChange with a valid interval and typed settings.
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value_t = 256)]
        max_history_reads: u64,
    },
    /// Apply only the approved plan, under this operating-system user's authority.
    Apply {
        #[arg(long)]
        plan: PathBuf,
        #[arg(long)]
        expect: String,
    },
    /// Read a user configuration revision without writing or refreshing timestamps.
    Read {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        source: String,
        #[arg(long)]
        valid_at_unix_ms: Option<u64>,
        #[arg(long)]
        recorded_revision: Option<u64>,
        #[arg(long, default_value_t = 256)]
        max_history_reads: u64,
    },
}
