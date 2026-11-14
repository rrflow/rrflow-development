//! Bounded installation preconditions and read-only Inventory preparation.
//!
//! Version-three preconditions bind admitted policy/Git inputs and exclude safety paths
//! before content reads. Enumeration admits entries before retaining them, and
//! hashing reads at most the admitted file length. Metadata drift fails closed;
//! this is not an atomic
//! filesystem snapshot or evidence for metadata-only content reuse.

use super::policy::CompiledPolicy;
use super::DefaultInstallProfile;
use crate::engine::{Result, ServiceError};
use rrd_core::digest::Sha256 as IncrementalSha256;
use rrd_store::filesystem::{AnchoredPath, SourceKind, SourceLeaf, SourceObservation};

use std::fs::Metadata;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

mod git_inputs;
mod proposal;
pub(in crate::engine) use proposal::prepare_project_inventory;

use git_inputs::GitInputs;

mod entries;
mod scan;
mod selected;
mod source;
mod walk;

pub(in crate::engine) use entries::*;
pub(in crate::engine) use scan::*;
pub(in crate::engine) use selected::*;
pub(in crate::engine) use source::*;
use walk::*;

#[cfg(test)]
mod tests;
