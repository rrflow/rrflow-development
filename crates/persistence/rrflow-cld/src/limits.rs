use std::time::Instant;

/// Conservative admission estimate for one page, not an allocator-level
/// guarantee. Configuration validation must use this same formula, so that a
/// policy cannot admit a page its own budget would refuse.
pub fn working_set_reservation(
    input_bytes: usize,
    cells: usize,
    max_output_bytes: usize,
    columns: usize,
) -> Option<usize> {
    input_bytes
        .checked_mul(32)?
        .checked_add(cells.checked_mul(64)?)?
        .checked_add(max_output_bytes.checked_mul(4)?)?
        .checked_add(columns.checked_mul(4096)?)?
        .checked_add(65536)
}

/// The caller owns the public profile and supplies finite resource limits.
#[derive(Debug, Clone, Copy)]
pub struct WriteLimits {
    pub max_rows: usize,
    pub max_columns: usize,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub row_group_rows: usize,
    pub max_memory_bytes: usize,
    pub deadline: Instant,
}

/// Explicit writer choices. Callers must admit the work and its memory budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Uncompressed,
    Zstd { level: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Encoding {
    pub compression: Compression,
    pub dictionary: bool,
    pub statistics: bool,
}

impl Default for Encoding {
    fn default() -> Self {
        Self {
            compression: Compression::Uncompressed,
            dictionary: false,
            statistics: false,
        }
    }
}

#[derive(Debug)]
pub struct EncodedBatch {
    pub bytes: Vec<u8>,
    pub columns: usize,
    pub row_groups: usize,
    pub input_bytes: usize,
    pub reserved_working_bytes: usize,
    /// Largest writer estimate sampled after a write; not allocator peak RSS.
    pub observed_writer_bytes: usize,
}
