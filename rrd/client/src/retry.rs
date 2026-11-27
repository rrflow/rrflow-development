//! Current bounded attempt/deadline configuration pending semantic certainty.

use crate::{Error, Result};
use rrd_contract::WebSocketLimits;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub request_timeout: Duration,
    pub max_attempts: u8,
    pub websocket_limits: WebSocketLimits,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(5),
            max_attempts: 2,
            websocket_limits: WebSocketLimits::default(),
        }
    }
}

pub(crate) fn request_timeout(
    configured: Duration,
    deadline_unix_ms: Option<u64>,
) -> Result<Duration> {
    let Some(deadline) = deadline_unix_ms else {
        return Ok(configured);
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::Contract(error.to_string()))?
        .as_millis();
    let now = u64::try_from(now).map_err(|_| Error::Contract("clock exceeds u64".into()))?;
    if deadline <= now {
        return Err(Error::Timeout);
    }
    Ok(configured.min(Duration::from_millis(deadline - now)))
}

pub(crate) fn validate_client_config(config: &ClientConfig) -> Result<()> {
    if config.request_timeout.is_zero() || config.max_attempts == 0 || config.max_attempts > 8 {
        return Err(Error::Contract(
            "request timeout must be nonzero and max_attempts must be in 1..=8".into(),
        ));
    }
    config
        .websocket_limits
        .validate()
        .map_err(|error| Error::Contract(error.to_string()))?;
    Ok(())
}
