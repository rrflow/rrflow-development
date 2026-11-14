use crate::physical::graph::{AdjacencyExpansion, TraversalPlan};
use crate::{
    execute_snapshot, highlight_offsets, index::index_artifact_name, rows_to_arrow_snapshot,
    ArrowReadStamp, Bm25Artifact, Bm25Config, Bm25Offset, BoundFilter, Error, FusionAnalysis,
    FusionBudget, IndexDefinition, IndexKind, IndexMaintenanceEvidence, LogicalOperator,
    PhysicalOperator, PhysicalPlan, Result, INDEX_ARTIFACT_CONTRACT_VERSION,
};
use crate::{Join, Projection, Source};
use rrd_core::result_identity::{self, ResultIdentityFamily, RESULT_IDENTITY_VERSION};
use rrd_core::{
    resolve_as_of, Claim, GeoValue, RuntimeChange, RuntimeGeo, RuntimeMutation, RuntimeRecord,
    RuntimeRelation, RuntimeValue, SeriesValue,
};
use rrd_store::{RuntimeReadBudget, RuntimeReadEvidence, RuntimeVersionedSource, StorageEngine};
use serde::de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::collections::BTreeMap;
use std::fmt;

mod bm25;
mod budget;
mod read_path;
mod rows;
mod run;
mod shape;

pub(crate) use bm25::*;
pub use budget::*;
pub(crate) use rows::*;
pub use run::*;
use shape::*;
