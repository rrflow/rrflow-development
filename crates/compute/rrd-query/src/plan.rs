use crate::catalog::{normalize_version_sources, version_sources_for_sources};
use crate::{Catalog, Error, IndexKind, Result};
use crate::{
    ComparisonOperator, CursorExpr, Join, Projection, Query, Source, TimeExpr, ValueExpr,
    QUERY_CONTRACT_VERSION,
};
use rrd_core::{
    digest, ProjectionId, ProjectionState, ReadStamp, RuntimeSchemaRegistry, RuntimeValue,
    RuntimeValueType,
};
use rrd_store::RuntimeVersionedSource;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod binding;
mod commit_index;
mod fields;
mod planner;
mod sources;
mod types;

pub use binding::*;
pub use commit_index::*;
pub(crate) use fields::*;
pub use planner::*;
use sources::*;
pub use types::*;
