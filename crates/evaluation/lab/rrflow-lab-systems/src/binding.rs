//! How a system under test is described: its kind, where it runs, its index and
//! search settings, and credentials named only by environment variable, so a
//! binding can be published with its results.
use rrflow_evaluation_core::{require, Availability, Id, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_LADDER_SETTINGS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemKind {
    Rrflow,
    Qdrant,
    Surrealdb,
    Milvus,
    Custom,
}

/// A reader must be able to see when a result crossed a network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemLocation {
    Loopback,
    Lan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "index", rename_all = "snake_case", deny_unknown_fields)]
pub enum IndexSpec {
    Exact,
    Hnsw { m: u32, ef_construction: u32 },
    TurboQuant { bits: String, seed: u64 },
    IvfRq { partitions: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchSetting {
    pub id: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ef: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_rerank: Option<u32>,
    /// How many partitions an IVF index probes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nprobes: Option<u32>,
}

/// Environment variable names only. The values never enter a file.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CredentialReferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_env: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SystemBinding {
    pub id: Id,
    pub label: String,
    pub kind: SystemKind,
    pub location: SystemLocation,
    pub endpoint: String,
    /// Pinned image or build identity, compared with what the server reports.
    pub artifact: String,
    /// What backs the data, because memory and durable disk are different claims.
    pub storage_profile: String,
    pub availability: Availability,
    pub index: IndexSpec,
    pub search_ladder: Vec<SearchSetting>,
    #[serde(default)]
    pub credentials: CredentialReferences,
    /// Commands that print one integer byte count on standard output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_probe: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_probe: Option<Vec<String>>,
    /// RRFlow `[storage]` settings the runner writes before the engine starts, so a run
    /// away from the installed defaults says so in the manifest it is bound to.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub engine_settings: BTreeMap<String, u64>,
    /// What a reader must know about this system's results, repeated in the report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limits: Vec<String>,
}

fn valid_env_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_endpoint(binding: &SystemBinding) -> Result<()> {
    let id = binding.id.as_str();
    let Some(address) = binding.endpoint.strip_prefix("http://") else {
        return Err(format!(
            "{id} endpoint must use http:// on a trusted network"
        ));
    };
    require(
        !address.contains('@'),
        &format!("{id} endpoint cannot carry credentials; use environment references"),
    )?;
    let host = address.split([':', '/']).next().unwrap_or_default();
    let loopback = matches!(host, "127.0.0.1" | "localhost" | "[::1]");
    match binding.location {
        SystemLocation::Loopback => require(
            loopback,
            &format!("{id} is declared loopback but its endpoint is not"),
        ),
        SystemLocation::Lan => require(
            !loopback,
            &format!("{id} is declared lan but its endpoint is loopback"),
        ),
    }
}

impl SystemBinding {
    /// Checks the binding for a test that asks for the top `k` results.
    pub fn validate(&self, k: u32) -> Result<()> {
        require(
            self.engine_settings.is_empty() || self.kind == SystemKind::Rrflow,
            &format!(
                "{} declares engine settings, which only RRFlow takes",
                self.id.as_str()
            ),
        )?;
        require(
            self.engine_settings.keys().all(|name| {
                !name.is_empty()
                    && name.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            }),
            &format!("{} has an invalid engine setting name", self.id.as_str()),
        )?;
        let id = self.id.as_str();
        require(
            !self.label.trim().is_empty() && self.label.len() <= 128,
            &format!("{id} label is invalid"),
        )?;
        require(
            !self.storage_profile.trim().is_empty() && !self.artifact.trim().is_empty(),
            &format!("{id} must declare its artifact and storage profile"),
        )?;
        validate_endpoint(self)?;
        for name in [
            &self.credentials.principal_env,
            &self.credentials.secret_env,
            &self.credentials.instance_env,
        ]
        .into_iter()
        .flatten()
        {
            require(
                valid_env_name(name),
                &format!("{id} credential reference must be an environment variable name"),
            )?;
        }
        if let Availability::Unavailable { reason }
        | Availability::Unimplemented { reason }
        | Availability::NotApplicable { reason } = &self.availability
        {
            return require(
                !reason.trim().is_empty(),
                &format!("{id} must say why it does not run"),
            );
        }
        require(
            !self.search_ladder.is_empty() && self.search_ladder.len() <= MAX_LADDER_SETTINGS,
            &format!("{id} search ladder size is outside bounds"),
        )?;
        let mut settings = BTreeSet::new();
        for setting in &self.search_ladder {
            require(
                settings.insert(&setting.id),
                &format!("{id} repeats a search setting"),
            )?;
            if let Some(ef) = setting.ef {
                require(
                    ef >= k,
                    &format!("{id} ef must be at least k, or some systems reject it"),
                )?;
            }
            if let Some(rerank) = setting.exact_rerank {
                require(
                    rerank >= k && setting.ef.is_none_or(|ef| rerank <= ef),
                    &format!("{id} rerank must be at least k and at most ef"),
                )?;
            }
            if matches!(
                self.index,
                IndexSpec::Hnsw { .. } | IndexSpec::TurboQuant { .. }
            ) {
                require(
                    setting.ef.is_some(),
                    &format!("{id} approximate search needs ef"),
                )?;
            }
            if let IndexSpec::IvfRq { partitions } = self.index {
                require(
                    setting
                        .nprobes
                        .is_some_and(|nprobes| (1..=partitions).contains(&nprobes)),
                    &format!("{id} IVF search needs nprobes within its partitions"),
                )?;
            }
        }
        match (&self.kind, &self.index) {
            (SystemKind::Custom, IndexSpec::IvfRq { partitions }) => require(
                *partitions > 0,
                &format!("{id} IVF index needs at least one partition"),
            ),
            (_, IndexSpec::IvfRq { .. }) => Err(format!(
                "{id}: an IVF index is available only through a custom system"
            )),
            (SystemKind::Rrflow, _) | (SystemKind::Custom, _) | (_, IndexSpec::Hnsw { .. }) => {
                Ok(())
            }
            (_, IndexSpec::Exact) => Ok(()),
            (_, IndexSpec::TurboQuant { .. }) => {
                Err(format!("{id}: TurboQuant is available only for RRFlow"))
            }
        }
    }
}
