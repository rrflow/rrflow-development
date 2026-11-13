//! The outside systems, suites, datasets and studies the lab knows about, what was pinned,
//! and why each is here. `check_sources.py` reports newer releases; adopting one stays a
//! deliberate change to a manifest, so nothing here moves a pin by itself.
use rrflow_evaluation_core::Id;
use rrflow_evaluation_core::{require, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SOURCES_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    System,
    Harness,
    Dataset,
    TaskSuite,
    Study,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    /// A checked-in test or tool uses it.
    InUse,
    /// A named next step will use it.
    Planned,
    /// Read or noted, with no decision to use it yet.
    Watching,
    /// Considered and left out, with the reason in `purpose`.
    NotUsed,
}

/// How a check finds what is newest. `Manual` means a person has to look.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "from", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceTrack {
    GithubRelease { repository: String },
    GithubCommit { repository: String },
    Pypi { package: String },
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutsideSource {
    pub id: Id,
    pub title: String,
    pub kind: SourceKind,
    pub url: String,
    pub license: String,
    /// The version or revision last looked at. Empty only for a `Manual` source.
    pub pinned: String,
    pub checked_on: String,
    pub track: SourceTrack,
    pub status: SourceStatus,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceList {
    pub schema_version: u16,
    pub sources: Vec<OutsideSource>,
}

fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes
            .iter()
            .enumerate()
            .all(|(position, byte)| match position {
                4 | 7 => *byte == b'-',
                _ => byte.is_ascii_digit(),
            })
}

impl SourceList {
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == SOURCES_SCHEMA_VERSION,
            "unsupported source list schema version",
        )?;
        let mut seen = BTreeSet::new();
        for source in &self.sources {
            let id = source.id.as_str();
            require(seen.insert(&source.id), &format!("{id} is listed twice"))?;
            require(
                source.url.starts_with("https://"),
                &format!("{id} needs an https address"),
            )?;
            require(
                [&source.title, &source.license, &source.purpose]
                    .iter()
                    .all(|text| !text.trim().is_empty()),
                &format!("{id} must state its title, license and purpose"),
            )?;
            require(
                is_date(&source.checked_on),
                &format!("{id} must give the date it was checked as YYYY-MM-DD"),
            )?;
            require(
                source.track == SourceTrack::Manual || !source.pinned.trim().is_empty(),
                &format!("{id} is tracked, so it must say what was pinned"),
            )?;
        }
        Ok(())
    }
}
