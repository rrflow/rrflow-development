//! Format-independent interchange between RRFlow planners and storage codecs.
use rrflow_core::{RuntimeValue, RuntimeValueType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod batch;
mod error;
pub use batch::*;
pub use error::{Error, Result};

pub type QueryFieldTypes = BTreeMap<String, Vec<RuntimeValueType>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryRow {
    pub identity: String,
    pub values: BTreeMap<String, RuntimeValue>,
}
