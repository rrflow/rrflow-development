//! Transaction-bound scalar, unique, and search-source index effects.
//!
//! The query catalogue remains the definition authority. This module stores a
//! digest-bound physical projection of the definition fields required during a
//! semantic commit, so rrflowMX and rrflowKV can derive the same old/new
//! effects without depending on rrflowQL or replaying the runtime log.

use super::super::keyspaces::{self, Space};
use super::runtime_state::{
    checked_key, get, get_json, read_sequence, scan_space_bounded, scan_space_bounded_from,
};
use super::semantic_commit::SemanticCommitPlan;
use crate::{Error, Result, StorageTransaction};
use rrd_core::{
    digest, ProjectionId, RuntimeCommit, RuntimeMutation, RuntimeRecord, RuntimeRef,
    RuntimeSchemaRegistry, RuntimeType, RuntimeValue, RuntimeValueType, ScopeId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod backfill;
mod binding;
mod effects;
mod lookup;
mod unique;

use backfill::*;
pub use binding::*;
pub(crate) use effects::*;
pub(crate) use lookup::*;
use unique::*;
