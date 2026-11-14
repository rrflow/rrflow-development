//! Canonical semantic key shapes layered on the rrflowKV tuple codec.
//!
//! Callers name a semantic shape; they never concatenate physical delimiters.
//! The codec places every shape below the manifest-authenticated
//! `format / tenant / scope / family` prefix and this module freezes the first
//! component that subdivides a family.

use crate::error::{Error, Result};
use crate::key_codec::{
    CatalogueSubfamily, DecodedKeyPart, KeyAddress, KeyCodec, KeyFamily, KeyPart,
};
use rrd_core::{
    Millis, Predicate, ProjectionFamily, ProjectionId, ProjectionWork, Reader, RuntimeEvent,
    RuntimeRef, RuntimeRelation, RuntimeValue, ScopeId, Subject,
};

pub(crate) use crate::key_codec::APPLICATION_FORMAT as RRFLOW_KV_FORMAT;

mod claims;
mod indexes;
mod runtime;
mod space;
mod system;

pub(crate) use claims::*;
pub(crate) use indexes::*;
pub(crate) use runtime::*;
pub(crate) use space::*;
pub use system::*;

#[cfg(test)]
mod tests;
