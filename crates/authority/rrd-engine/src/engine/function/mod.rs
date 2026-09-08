//! Governed function catalogue, execution, and transaction-binding behavior.
//!
//! This module preserves the current pre-release function behavior while
//! keeping catalogue persistence, runtime execution, and pre-commit bindings
//! structurally separate. Engine events, post-commit triggers, routines, and
//! skills are different capabilities and do not belong here.

mod catalogue;
mod execution;
mod javascript;
mod transaction_binding;
mod webassembly;
