//! Engine-owned, derived cold-history publication. Source history remains
//! authoritative; publication never reclaims or replaces it.

use super::clock::{observe_required, HostSystemClock};
use super::*;
use rrd_contract::{
    ColdHistoryCompression, ColdHistoryFailure, ColdHistoryFailureCode,
    ColdHistoryGenerationSnapshot, ColdHistoryPolicy, ColdHistoryReadResult, ReadColdHistory,
};
use rrflow_cld::{
    ColdHistoryReadError, ColdHistoryReadExpectation, ColdHistoryReadLimits, ColdStorageBackend,
    Compression, Encoding, WriteLimits,
};
use rrflow_db::filesystem::{AnchoredPath, OpenOptions};
use std::io::{Read, Write};
use std::sync::TryLockError;

mod files;
mod maintain;
mod state;

use files::*;
use state::*;

/// Installation and reopen must admit the configured writer before serving.
pub(super) fn admit_backend(policy: &ColdHistoryPolicy) -> Result<()> {
    writer(policy).map(drop)
}

#[cfg(test)]
mod tests;
