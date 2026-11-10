//! Optional bounded LFG v1 HTTP transport. The caller must authorize and
//! reserve the effect; constructing this value performs no external work.
//! The current LFG service has no authentication, so this adapter accepts
//! literal loopback only. That does not authenticate a local process.

use crate::lfg_observation::{decode_lfg_observation, ExpectedObservation, LfgObservation};
use crate::{EmbeddingExecutionControl, NetworkPolicy};
use rrd_core::digest::sha256_hex;
use serde::Serialize;
use std::io::{Read, Write};
use std::net::IpAddr;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct LfgTransportFailure {
    pub reason: &'static str,
    /// After dispatch, the remote process may still be processing the request.
    pub execution_may_continue: bool,
}

impl std::fmt::Display for LfgTransportFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.reason)
    }
}

impl std::error::Error for LfgTransportFailure {}

/// Digests identify the exact transport bytes, not producer identity or quality.
#[derive(Debug)]
pub struct LfgTransportObservation {
    pub observation: LfgObservation,
    pub request_sha256: String,
    pub response_sha256: String,
    pub elapsed_ms: u64,
}

fn failure(reason: &'static str, sent: bool) -> LfgTransportFailure {
    LfgTransportFailure {
        reason,
        execution_may_continue: sent,
    }
}

/// Explicitly configured wire limits and pinned analyzer configuration.
pub struct LfgTransportConfig<'a> {
    pub base_url: &'a str,
    pub assessment_sha256: &'a str,
    pub rules_sha256: &'a str,
    pub timeout: Duration,
    pub maximum_input_bytes: usize,
    pub maximum_request_bytes: usize,
    pub maximum_response_bytes: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WireRequest<'a> {
    schema_version: &'static str,
    request_id: &'a str,
    revision: u64,
    text: &'a str,
    input_sha256: &'a str,
    expected_assessment_sha256: &'a str,
    expected_rules_sha256: &'a str,
}

struct BoundedRequest {
    bytes: Vec<u8>,
    remaining: usize,
}

impl Write for BoundedRequest {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.remaining = self
            .remaining
            .checked_sub(data.len())
            .ok_or_else(|| std::io::Error::other("request too large"))?;
        self.bytes.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Prepared payload contains the supplied text in process memory. Never log it.
pub struct PreparedLfgObservation {
    url: String,
    request: Vec<u8>,
    request_id: String,
    revision: u64,
    input_sha256: String,
    assessment_sha256: String,
    rules_sha256: String,
    timeout: Duration,
    maximum_response_bytes: usize,
}

impl PreparedLfgObservation {
    pub fn new(
        config: &LfgTransportConfig<'_>,
        request_id: &str,
        revision: u64,
        text: &str,
    ) -> Result<Self, LfgTransportFailure> {
        let uri: ureq::http::Uri = config
            .base_url
            .parse()
            .map_err(|_| failure("invalid LFG endpoint", false))?;
        let host = uri
            .host()
            .ok_or_else(|| failure("invalid LFG endpoint", false))?;
        let loopback = host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.is_loopback());
        if uri.scheme_str() != Some("http")
            || !loopback
            || uri.port_u16().is_none()
            || uri.query().is_some()
            || config.base_url.contains('#')
            || uri.authority().is_some_and(|a| a.as_str().contains('@'))
            || !matches!(uri.path(), "" | "/")
        {
            return Err(failure(
                "LFG endpoint must be a literal loopback HTTP origin",
                false,
            ));
        }
        let valid_digest = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        if config.timeout.is_zero()
            || config.maximum_input_bytes == 0
            || config.maximum_request_bytes == 0
            || config.maximum_response_bytes == 0
            || u64::try_from(config.maximum_response_bytes)
                .ok()
                .and_then(|value| value.checked_add(1))
                .is_none()
            || !valid_digest(config.assessment_sha256)
            || !valid_digest(config.rules_sha256)
            || request_id.is_empty()
            || request_id.len() > config.maximum_request_bytes
            || !request_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || text.trim().is_empty()
            || text.len() > config.maximum_input_bytes
        {
            return Err(failure("invalid LFG input, identity, or limits", false));
        }
        let input_sha256 = sha256_hex(text.as_bytes());
        let mut request = BoundedRequest {
            bytes: Vec::new(),
            remaining: config.maximum_request_bytes,
        };
        serde_json::to_writer(
            &mut request,
            &WireRequest {
                schema_version: "lfg.observation-request.v1",
                request_id,
                revision,
                text,
                input_sha256: &input_sha256,
                expected_assessment_sha256: config.assessment_sha256,
                expected_rules_sha256: config.rules_sha256,
            },
        )
        .map_err(|_| failure("invalid or oversized LFG request", false))?;
        Ok(Self {
            url: format!(
                "{}/api/lfg/observations",
                config.base_url.trim_end_matches('/')
            ),
            request: request.bytes,
            request_id: request_id.into(),
            revision,
            input_sha256,
            assessment_sha256: config.assessment_sha256.into(),
            rules_sha256: config.rules_sha256.into(),
            timeout: config.timeout,
            maximum_response_bytes: config.maximum_response_bytes,
        })
    }

    pub fn execute(
        &self,
        network: NetworkPolicy,
        control: &EmbeddingExecutionControl,
    ) -> Result<LfgTransportObservation, LfgTransportFailure> {
        if network != NetworkPolicy::Allow {
            return Err(failure("network denied", false));
        }
        control
            .check()
            .map_err(|_| failure("LFG cancelled or deadline exceeded", false))?;
        let started = Instant::now();
        let deadline = started
            .checked_add(self.timeout)
            .ok_or_else(|| failure("invalid LFG deadline", false))?;
        let control = control.with_deadline(deadline);
        let remaining = control
            .deadline()
            .checked_duration_since(Instant::now())
            .ok_or_else(|| failure("LFG deadline exceeded", false))?;
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .proxy(None)
            .max_redirects(0)
            .http_status_as_error(false)
            .max_idle_connections(0)
            .timeout_global(Some(remaining))
            .build()
            .into();
        control
            .check()
            .map_err(|_| failure("LFG cancelled or deadline exceeded", false))?;
        let mut response = agent
            .post(&self.url)
            .header("Content-Type", "application/json")
            .send(self.request.as_slice())
            .map_err(|_| failure("LFG transport failed; remote outcome unknown", true))?;
        control
            .check()
            .map_err(|_| failure("LFG cancelled or deadline exceeded", true))?;
        if response.status().as_u16() != 200 {
            return Err(failure("LFG endpoint did not return success", true));
        }
        let limit = u64::try_from(self.maximum_response_bytes)
            .map_err(|_| failure("invalid LFG response limit", true))?;
        let read_limit = limit
            .checked_add(1)
            .ok_or_else(|| failure("invalid LFG response limit", true))?;
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(read_limit)
            .read_to_end(&mut bytes)
            .map_err(|_| failure("LFG response read failed; remote outcome unknown", true))?;
        control
            .check()
            .map_err(|_| failure("LFG cancelled or deadline exceeded", true))?;
        let expected = ExpectedObservation {
            request_id: &self.request_id,
            revision: self.revision,
            input_sha256: &self.input_sha256,
            assessment_sha256: &self.assessment_sha256,
            rules_sha256: &self.rules_sha256,
            maximum_response_bytes: self.maximum_response_bytes,
        };
        let observation = decode_lfg_observation(&bytes, &expected)
            .map_err(|_| failure("invalid or mismatched LFG response", true))?;
        control
            .check()
            .map_err(|_| failure("LFG cancelled or deadline exceeded", true))?;
        Ok(LfgTransportObservation {
            observation,
            request_sha256: sha256_hex(&self.request),
            response_sha256: sha256_hex(&bytes),
            elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }
}
