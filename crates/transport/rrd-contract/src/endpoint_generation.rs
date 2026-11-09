//! Explicit model generation settings; these do not attest provider support.
use crate::{invalid, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EndpointTokenParameter {
    MaxTokens,
    MaxCompletionTokens,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EndpointResponseFormat {
    Text,
    JsonObject,
}

/// No defaults: the installed binding's policy must name the supported wire
/// parameters. An omitted temperature is distinct from requesting zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointGenerationSettings {
    pub maximum_output_tokens: u32,
    pub token_parameter: EndpointTokenParameter,
    pub temperature_milli: Option<u16>,
    pub response_format: EndpointResponseFormat,
}

impl EndpointGenerationSettings {
    pub fn validate(&self) -> Result<()> {
        if self.maximum_output_tokens == 0
            || self.maximum_output_tokens > 65_536
            || self.temperature_milli.is_some_and(|value| value > 2_000)
        {
            return invalid("invalid model generation bounds");
        }
        Ok(())
    }
}
