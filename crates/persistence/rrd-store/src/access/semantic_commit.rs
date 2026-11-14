//! One encoded semantic write plan for rrflowMX and rrflowKV.
//!
//! The plan is prepared against one captured storage snapshot. Preparation
//! validates logical state and encodes the complete semantic effect set,
//! including scope-bound versions, graph adjacency, index source deltas,
//! governed-function receipts, projection work, audit, and outcome state. It
//! cannot publish. The owning repository applies the plan to the same open
//! transaction and only that transaction's commit receipt makes its effects
//! authoritative.

use super::runtime_state::{
    checked_key, current_accumulator, get, get_json, read_sequence, read_stamp_with,
    validate_read_stamp, values_for_scope, AccessRead,
};
use super::{read_versioned, schema_at_read, RuntimeReadBudget, RuntimeVersionedSource};
use crate::keyspaces::{self, Space};
use crate::transaction::TransactionWriteSet;
use crate::{Error, FunctionInvocationReceiptRecord, Result, StorageTransaction};
use rrd_core::{
    projection_family, AuditEnvelope, ProjectionWork, ReadStamp, RuntimeChange, RuntimeCommit,
    RuntimeCommitOutcome, RuntimeDataSnapshot, RuntimeMutation, RuntimeRecord, RuntimeRef,
    RuntimeRelation, RuntimeSchemaRegistry,
};
use std::collections::{BTreeMap, BTreeSet};

mod constraints;

#[cfg(test)]
#[path = "semantic_commit/constraint_tests.rs"]
mod constraint_tests;

mod effects;
mod plan;

use effects::*;
pub use plan::*;

#[cfg(test)]
mod tests;
