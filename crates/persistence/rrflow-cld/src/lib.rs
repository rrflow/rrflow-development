//! Immutable columnar storage. Transaction admission, visibility and placement
//! belong to the caller. This crate has no dependency on the query planner.
pub mod columnar;
mod error;
pub mod formats;
mod sink;
pub use error::{Error, Result};
pub use rrflow_arrow::{record_batch_to_rows, rows_to_record_batch, QueryFieldTypes, QueryRow};

#[cfg(feature = "cld-parquet")]
mod interchange;
#[cfg(feature = "cld-parquet")]
pub use interchange::*;

mod limits;
pub use limits::*;
mod history;
pub use history::*;
