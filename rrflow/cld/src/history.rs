//! Authenticated, bounded generation codec used by the installed engine.
use crate::{Error, Result, WriteLimits};
use rrflow_core::{ReadStamp, RuntimeChange, RuntimeChangePage};
use std::time::Instant;

#[cfg(feature = "cld-parquet")]
mod parquet;
mod rows;
#[cfg(feature = "cld-vortex")]
mod vortex;

#[cfg(feature = "cld-parquet")]
pub use parquet::ParquetHistory;
#[cfg(feature = "cld-vortex")]
pub use vortex::VortexHistory;

#[derive(Debug)]
pub struct HistoryArtifact {
    pub bytes: Vec<u8>,
    pub row_groups: usize,
}

/// Durable codec identity. Existing receipts are decoded with their recorded format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryFormat {
    Parquet,
    Vortex,
}

impl HistoryFormat {
    pub fn name(self) -> &'static str {
        match self {
            Self::Parquet => "parquet",
            Self::Vortex => "vortex",
        }
    }
    pub fn artifact_name(self, digest: &str) -> String {
        format!("{digest}.{}", self.name())
    }
    pub fn reader(self) -> Result<Box<dyn ColdStorageBackend>> {
        match self {
            Self::Parquet => HistoryWriter::Parquet(crate::Encoding::default()).open(),
            Self::Vortex => HistoryWriter::Vortex.open(),
        }
    }
}

pub enum HistoryWriter {
    Parquet(crate::Encoding),
    Vortex,
}

impl HistoryWriter {
    pub fn open(self) -> Result<Box<dyn ColdStorageBackend>> {
        match self {
            #[cfg(feature = "cld-parquet")]
            Self::Parquet(settings) => Ok(Box::new(ParquetHistory(settings))),
            #[cfg(feature = "cld-vortex")]
            Self::Vortex => Ok(Box::new(VortexHistory)),
            #[cfg(not(feature = "cld-parquet"))]
            Self::Parquet(_) => Err(Error::Binding(
                "Parquet cold storage is disabled in this build".into(),
            )),
            #[cfg(not(feature = "cld-vortex"))]
            Self::Vortex => Err(Error::Binding(
                "Vortex cold storage is disabled in this build".into(),
            )),
        }
    }
}

pub trait ColdStorageBackend {
    fn format(&self) -> HistoryFormat;
    fn encode(
        &self,
        stamp: &ReadStamp,
        page: &RuntimeChangePage,
        limits: WriteLimits,
    ) -> Result<ColdHistoryPage>;
    fn decode(
        &self,
        bytes: Vec<u8>,
        expected: &ColdHistoryReadExpectation,
        limits: ColdHistoryReadLimits,
    ) -> std::result::Result<DecodedColdHistoryPage, ColdHistoryReadError>;
}

pub const COLD_HISTORY_FORMAT_VERSION: u16 = 1;

#[derive(Debug)]
pub struct ColdHistoryPage {
    pub requested_after: u64,
    pub through_cursor: u64,
    pub head_cursor: u64,
    pub source_manifest: String,
    pub artifact: HistoryArtifact,
}

#[derive(Debug, Clone)]
pub struct ColdHistoryReadExpectation {
    pub stamp: ReadStamp,
    pub requested_after: u64,
    pub through_cursor: u64,
    pub rows: u64,
    pub artifact_sha256: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ColdHistoryReadLimits {
    pub max_rows: usize,
    pub max_encoded_bytes: usize,
    pub max_decoded_bytes: usize,
    pub max_memory_bytes: usize,
    pub max_batch_rows: usize,
    pub deadline: Instant,
}

#[derive(Debug)]
pub struct DecodedColdHistoryPage {
    pub changes: Vec<RuntimeChange>,
    pub encoded_bytes: usize,
    pub row_groups: usize,
    pub declared_uncompressed_bytes: u64,
    pub observed_arrow_bytes: usize,
}

/// Failure identity from decoding an admitted cold-history artifact.
#[derive(Debug)]
pub enum ColdHistoryReadError {
    DeadlineExceeded,
    Codec(Error),
}

impl std::fmt::Display for ColdHistoryReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeadlineExceeded => formatter.write_str("cold history read deadline exceeded"),
            Self::Codec(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ColdHistoryReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DeadlineExceeded => None,
            Self::Codec(error) => Some(error),
        }
    }
}

impl From<Error> for ColdHistoryReadError {
    fn from(error: Error) -> Self {
        match error {
            Error::DeadlineExceeded(_) => Self::DeadlineExceeded,
            other => Self::Codec(other),
        }
    }
}

impl From<serde_json::Error> for ColdHistoryReadError {
    fn from(error: serde_json::Error) -> Self {
        Self::Codec(error.into())
    }
}

fn admit(
    bytes: &[u8],
    expected: &ColdHistoryReadExpectation,
    limits: ColdHistoryReadLimits,
) -> std::result::Result<(), ColdHistoryReadError> {
    if Instant::now() >= limits.deadline {
        return Err(ColdHistoryReadError::DeadlineExceeded);
    }
    if limits.max_rows == 0
        || limits.max_batch_rows == 0
        || limits.max_encoded_bytes == 0
        || limits.max_decoded_bytes == 0
        || limits.max_memory_bytes == 0
        || bytes.len() > limits.max_encoded_bytes
        || expected.rows > limits.max_rows as u64
        || expected.requested_after > expected.through_cursor
        || expected.through_cursor > expected.stamp.commit_cursor
    {
        return Err(Error::Budget("invalid or exceeded cold history read limits".into()).into());
    }
    expected
        .stamp
        .validate()
        .map_err(|error| Error::Integrity(error.to_string()))?;
    if rrflow_core::digest::sha256_hex(bytes) != expected.artifact_sha256 {
        return Err(Error::Integrity("cold history artifact digest mismatch".into()).into());
    }
    Ok(())
}
