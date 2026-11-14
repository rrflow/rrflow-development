//! Liveness and readiness.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Liveness {
    pub observed_at_unix_ms: u64,
}

impl Liveness {
    pub fn validate(&self) -> Result<()> {
        if self.observed_at_unix_ms == 0 {
            return invalid("liveness observation time must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Readiness {
    pub observed_at_unix_ms: u64,
    pub claim_sequence: u64,
    pub runtime_cursor: u64,
    pub backend: CanonicalId,
}

impl Readiness {
    pub fn validate(&self) -> Result<()> {
        if self.observed_at_unix_ms == 0 {
            return invalid("readiness observation time must be greater than zero");
        }
        Ok(())
    }
}
