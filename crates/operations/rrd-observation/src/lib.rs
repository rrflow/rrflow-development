//! A single bounded HTTP exchange. No storage, policy authority, retry or task execution.
use rrd_contract::{
    EndpointBinding, EndpointLocator, ObservationDeliveryDisposition, ObservationExportPolicy,
    MAX_OBSERVATION_EVENT_BYTES,
};
use rrd_core::digest::sha256_hex;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// The RROFLOW v1 receiver retains these as reported references, not source attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineReferences {
    pub version: u16,
    pub instance_id: String,
    pub scope: String,
    pub operation: String,
    pub response_sha256: String,
    pub task_id: Option<String>,
    pub attempt_id: Option<String>,
    pub job_id: Option<String>,
    pub plan_sha256: Option<String>,
    pub source_sha256: Option<String>,
    pub configuration_sha256: Option<String>,
}

fn id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.:/".contains(&b))
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, b)| {
            if [8, 13, 18, 23].contains(&index) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}

impl EngineReferences {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != 1
            || !id(&self.instance_id)
            || !id(&self.scope)
            || !id(&self.operation)
            || !digest(&self.response_sha256)
            || [&self.task_id, &self.attempt_id, &self.job_id]
                .iter()
                .any(|value| value.as_deref().is_some_and(|value| !id(value)))
            || [
                &self.plan_sha256,
                &self.source_sha256,
                &self.configuration_sha256,
            ]
            .iter()
            .any(|value| value.as_deref().is_some_and(|value| !digest(value)))
            || (self.task_id.is_none() && self.job_id.is_none())
            || (self.attempt_id.is_some() && self.task_id.is_none())
        {
            return Err("invalid engine observation references");
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct Event<'a> {
    schema: &'static str,
    event_id: &'a str,
    capture_id: &'a str,
    sequence: u64,
    agent: &'static str,
    session_id: &'a str,
    operation_id: &'a str,
    observed_at: String,
    kind: &'static str,
    engine: &'a EngineReferences,
}

/// Encode only the closed metadata projection; no arbitrary payload is accepted.
pub fn encode_event(
    event_id: &str,
    capture_id: &str,
    delivery_id: &str,
    observed_at_unix_ms: u64,
    references: &EngineReferences,
) -> Result<Vec<u8>, &'static str> {
    references.validate()?;
    if !uuid(event_id) || !uuid(capture_id) || !id(delivery_id) || observed_at_unix_ms == 0 {
        return Err("invalid observation envelope identity or time");
    }
    let time = i64::try_from(observed_at_unix_ms).map_err(|_| "invalid observation time")?;
    let at = chrono::DateTime::from_timestamp_millis(time).ok_or("invalid observation time")?;
    let event = Event {
        schema: "rroflow.capture/v1",
        event_id,
        capture_id,
        sequence: 0,
        agent: "rrflow",
        session_id: capture_id,
        operation_id: delivery_id,
        observed_at: at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        kind: "engine.observed",
        engine: references,
    };
    #[derive(Serialize)]
    struct Batch<'a> {
        events: [Event<'a>; 1],
    }
    let bytes = serde_json::to_vec(&Batch { events: [event] })
        .map_err(|_| "observation encoding failed")?;
    if bytes.len() > MAX_OBSERVATION_EVENT_BYTES {
        return Err("observation exceeds byte limit");
    }
    Ok(bytes)
}

/// Facts returned to the engine. The adapter does not accept a delivery receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportObservation {
    pub disposition: ObservationDeliveryDisposition,
    pub request_may_have_been_sent: bool,
    pub http_status: Option<u16>,
    pub reply_sha256: Option<String>,
}
impl TransportObservation {
    fn failure(disposition: ObservationDeliveryDisposition, sent: bool) -> Self {
        Self {
            disposition,
            request_may_have_been_sent: sent,
            http_status: None,
            reply_sha256: None,
        }
    }
}

pub struct PreparedExport<'a> {
    url: String,
    payload: &'a [u8],
    expected_event_id: &'a str,
    policy: &'a ObservationExportPolicy,
    timeout: Duration,
}

impl<'a> PreparedExport<'a> {
    /// Pure preparation is not permission to contact the destination.
    pub fn new(
        binding: &EndpointBinding,
        policy: &'a ObservationExportPolicy,
        payload: &'a [u8],
        expected_event_id: &'a str,
    ) -> Result<Self, &'static str> {
        policy
            .validate_for(std::slice::from_ref(binding))
            .map_err(|_| "invalid observation binding")?;
        if !policy.allow_network || !policy.allow_metadata_export {
            return Err("observation export is disabled");
        }
        if payload.is_empty()
            || payload.len() > MAX_OBSERVATION_EVENT_BYTES
            || payload.len() as u64 > binding.limits.max_input_bytes
        {
            return Err("observation payload exceeds configured limit");
        }
        let EndpointLocator::Url { url } = &binding.endpoint else {
            return Err("observation transport requires a URL binding");
        };
        let uri: ureq::http::Uri = url.parse().map_err(|_| "invalid observation URL")?;
        let host = uri.host().ok_or("observation URL has no host")?;
        let loopback = host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.is_loopback());
        if uri.query().is_some()
            || url.contains('#')
            || uri.authority().is_some_and(|a| a.as_str().contains('@'))
            || !(uri.scheme_str() == Some("https") || uri.scheme_str() == Some("http") && loopback)
        {
            return Err(
                "observation URL requires TLS or literal loopback without credentials/query",
            );
        }
        Ok(Self {
            url: format!("{}/v1/events", url.trim_end_matches('/')),
            payload,
            expected_event_id,
            policy,
            timeout: Duration::from_millis(binding.limits.timeout_ms),
        })
    }

    /// Call only after the engine commits and authorizes its dispatch fence.
    /// One exchange, no redirects/proxies/retry. In-flight revocation cannot
    /// undo a remote write; the engine must separately validate the returned facts.
    pub fn execute(&self, credential: &str, deadline: Instant) -> TransportObservation {
        use ObservationDeliveryDisposition as D;
        if credential.is_empty()
            || credential.len() > 16_384
            || credential
                .bytes()
                .any(|c| c.is_ascii_whitespace() || c.is_ascii_control())
        {
            return TransportObservation::failure(D::CredentialUnavailable, false);
        }
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return TransportObservation::failure(D::DeadlineExceeded, false);
        };
        let Ok(mut authorization) =
            ureq::http::HeaderValue::from_str(&format!("Bearer {credential}"))
        else {
            return TransportObservation::failure(D::CredentialUnavailable, false);
        };
        authorization.set_sensitive(true);
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .proxy(None)
            .max_redirects(0)
            .http_status_as_error(false)
            .max_idle_connections(0)
            .timeout_global(Some(remaining.min(self.timeout)))
            .build()
            .into();
        let response = agent
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Authorization", authorization)
            .send(self.payload);
        let Ok(mut response) = response else {
            return TransportObservation::failure(D::TransportUnavailable, true);
        };
        let status = response.status().as_u16();
        let mut observation = TransportObservation::failure(D::ReplyRejected, true);
        observation.http_status = Some(status);
        if Instant::now() >= deadline {
            observation.disposition = D::DeadlineExceeded;
            return observation;
        }
        if status != 200 {
            return observation;
        }
        let mut bytes = Vec::new();
        if response
            .body_mut()
            .as_reader()
            .take(self.policy.max_response_bytes + 1)
            .read_to_end(&mut bytes)
            .is_err()
        {
            observation.disposition = D::TransportUnavailable;
            return observation;
        }
        if bytes.len() as u64 > self.policy.max_response_bytes {
            observation.disposition = D::ReplyTooLarge;
            return observation;
        }
        if Instant::now() >= deadline {
            observation.disposition = D::DeadlineExceeded;
            return observation;
        }
        observation.reply_sha256 = Some(sha256_hex(&bytes));
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Acknowledgement {
            devspace: String,
            accepted_ids: Vec<String>,
        }
        if let Ok(reply) = serde_json::from_slice::<Acknowledgement>(&bytes) {
            if reply.devspace == self.policy.expected_devspace
                && reply.accepted_ids == [self.expected_event_id]
            {
                observation.disposition = D::Acknowledged;
            }
        }
        observation
    }
}
