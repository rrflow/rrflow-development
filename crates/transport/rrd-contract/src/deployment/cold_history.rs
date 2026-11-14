//! The cold-history publication policy.

use super::*;

/// One admitted cold-history maintenance quantum. This is installed policy;
/// its presence does not assert that an archive generation is available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColdHistoryPolicy {
    /// Target period between server offers; missed or busy offers are skipped.
    pub interval_ms: u64,
    pub max_changes: u64,
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_memory_bytes: u64,
    pub max_elapsed_ms: u64,
    pub row_group_rows: u64,
    pub compression: ColdHistoryCompression,
    pub dictionary: bool,
    pub statistics: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ColdHistoryCompression {
    Uncompressed,
    Zstd { level: u8 },
}

/// Columns a cold page writes, counting the identity column.
pub(super) const COLD_HISTORY_COLUMNS: u64 = 15;

impl ColdHistoryPolicy {
    /// The encoder's admission estimate for one full page under this policy.
    /// A budget below it can refuse a page forever, which stops the tier at
    /// that change with no way forward, so `validate` refuses such a policy.
    pub fn minimum_working_memory_bytes(&self) -> u64 {
        self.max_input_bytes
            .saturating_mul(32)
            .saturating_add(
                self.max_changes
                    .saturating_mul(COLD_HISTORY_COLUMNS)
                    .saturating_mul(64),
            )
            .saturating_add(self.max_output_bytes.saturating_mul(4))
            .saturating_add(COLD_HISTORY_COLUMNS.saturating_mul(4096))
            .saturating_add(65_536)
    }

    pub fn validate(&self) -> Result<()> {
        validate_nonzero_max("cold history interval_ms", self.interval_ms, 3_600_000)?;
        validate_nonzero_max("cold history max_changes", self.max_changes, 65_536)?;
        validate_nonzero_max(
            "cold history max_input_bytes",
            self.max_input_bytes,
            1_073_741_824,
        )?;
        validate_nonzero_max(
            "cold history max_output_bytes",
            self.max_output_bytes,
            1_073_741_824,
        )?;
        validate_nonzero_max(
            "cold history max_memory_bytes",
            self.max_memory_bytes,
            4_294_967_296,
        )?;
        validate_nonzero_max(
            "cold history max_elapsed_ms",
            self.max_elapsed_ms,
            3_600_000,
        )?;
        validate_nonzero_max(
            "cold history row_group_rows",
            self.row_group_rows,
            self.max_changes,
        )?;
        if self.max_input_bytes > self.max_memory_bytes
            || self.max_output_bytes > self.max_memory_bytes
        {
            return invalid(
                "cold history input and output bounds must fit the working-memory budget",
            );
        }
        if self.max_memory_bytes < self.minimum_working_memory_bytes() {
            return invalid("cold history working-memory budget cannot encode one full input page");
        }
        if let ColdHistoryCompression::Zstd { level } = self.compression {
            if !(1..=22).contains(&level) {
                return invalid("cold history zstd level must be in 1..=22");
            }
        }
        Ok(())
    }
}
