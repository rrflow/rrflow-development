//! Governed function catalogue, execution, and transaction-binding behavior.
//!
//! This module preserves the current pre-release function behavior while
//! keeping catalogue persistence, runtime execution, and pre-commit bindings
//! structurally separate. Engine events, post-commit triggers, routines, and
//! skills are different capabilities and do not belong here.

mod catalogue;
mod execution;
mod javascript;
pub(in crate::engine) mod runtime_profile;
mod transaction_binding;
mod webassembly;

pub(super) use execution::encode_receipt_record;
#[cfg(test)]
pub(in crate::engine) use execution::{reset_test_execution_count, test_execution_count};
