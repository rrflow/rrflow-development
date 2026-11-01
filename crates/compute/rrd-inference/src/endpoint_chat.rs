//! Optional, stateless transport for an explicitly prepared selector request.
//! This is not engine admission, a worker dispatcher, or a provider hard-stop.
use crate::{EmbeddingDispatchError, EmbeddingExecutionControl, NetworkPolicy};
use rrd_contract::{
    CanonicalId, CredentialReference, EndpointConfiguration, EndpointLocator,
    EndpointSelectionAllowance, EndpointSelectionChoice, EndpointTokenUsage, WorkloadClass,
    MAX_ROUTE_REQUEST_BYTES,
};
use rrd_core::digest::sha256_hex;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::IpAddr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChatResponseFormat {
    JsonObject,
}

/// Prompt construction and generation settings belong to the configured caller.
/// This narrow wire profile neither streams nor requests tools/multiple choices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointChatRequest {
    pub model: CanonicalId,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f64,
    pub response_format: ChatResponseFormat,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointChatFailure {
    pub reason: &'static str,
    /// Conservative after the HTTP call begins; never permission to retry.
    pub dispatch_may_have_occurred: bool,
    pub http_status: Option<u16>,
}
impl std::fmt::Display for EndpointChatFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason)
    }
}
impl std::error::Error for EndpointChatFailure {}
type Result<T> = std::result::Result<T, EndpointChatFailure>;
fn failure(reason: &'static str, sent: bool) -> EndpointChatFailure {
    EndpointChatFailure {
        reason,
        dispatch_may_have_occurred: sent,
        http_status: None,
    }
}

/// Provider envelopes may add metadata; the canonical retained usage is closed.
#[derive(Deserialize)]
struct ChatUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}
impl From<ChatUsage> for EndpointTokenUsage {
    fn from(value: ChatUsage) -> Self {
        Self {
            prompt_tokens: value.prompt_tokens,
            completion_tokens: value.completion_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EndpointChatObservation {
    pub choice: EndpointSelectionChoice,
    pub service_id: String,
    pub configuration_sha256: String,
    pub allowance_sha256: String,
    pub request_sha256: String,
    pub response_sha256: String,
    pub content_sha256: String,
    pub usage: Option<EndpointTokenUsage>,
    pub elapsed_ms: Option<u64>,
    pub http_status: Option<u16>,
    /// Exact bytes for an explicitly authorized evidence sink; not automatic logs.
    #[serde(skip)]
    pub response_bytes: Vec<u8>,
}

/// Validates configuration and prepares bytes without resolving credentials or
/// performing network work. Construction proves consistency, not authorization.
pub struct PreparedEndpointChat<'a> {
    configuration: &'a EndpointConfiguration,
    allowance: &'a EndpointSelectionAllowance,
    credential: Option<&'a CredentialReference>,
    service_id: &'a CanonicalId,
    url: String,
    request: Vec<u8>,
    timeout: Duration,
}

impl<'a> PreparedEndpointChat<'a> {
    pub fn new(
        configuration: &'a EndpointConfiguration,
        controller_binding: &CanonicalId,
        allowance: &'a EndpointSelectionAllowance,
        request: &EndpointChatRequest,
    ) -> Result<Self> {
        allowance
            .validate_for(configuration)
            .map_err(|_| failure("invalid selection configuration or allowance", false))?;
        let binding = configuration
            .profile
            .as_ref()
            .and_then(|p| p.bindings.iter().find(|b| &b.id == controller_binding))
            .ok_or_else(|| failure("controller binding is not configured", false))?;
        if !matches!(
            binding.class,
            WorkloadClass::LargeModelServing | WorkloadClass::AlwaysWarmSmallModels
        ) || binding.service_id != request.model
        {
            return Err(failure(
                "controller request differs from configured model binding",
                false,
            ));
        }
        if request.messages.is_empty()
            || request.messages.len() > 128
            || request.max_tokens == 0
            || request.max_tokens > 65_536
            || !request.temperature.is_finite()
            || !(0.0..=2.0).contains(&request.temperature)
        {
            return Err(failure("invalid chat generation bounds", false));
        }
        let EndpointLocator::Url { url } = &binding.endpoint else {
            return Err(failure("chat transport requires a URL binding", false));
        };
        let uri: ureq::http::Uri = url
            .parse()
            .map_err(|_| failure("invalid configured chat URL", false))?;
        let host = uri
            .host()
            .ok_or_else(|| failure("chat URL has no host", false))?;
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
            return Err(failure(
                "chat URL requires TLS or literal loopback without credentials/query",
                false,
            ));
        }
        let limit = binding.limits.max_input_bytes.min(MAX_ROUTE_REQUEST_BYTES);
        let request = encode_bounded(request, limit)?;
        Ok(Self {
            configuration,
            allowance,
            credential: binding.credential.as_ref(),
            service_id: &binding.service_id,
            url: format!("{}/chat/completions", url.trim_end_matches('/')),
            request,
            timeout: Duration::from_millis(binding.limits.timeout_ms),
        })
    }

    pub fn request_bytes(&self) -> &[u8] {
        &self.request
    }

    /// One call, no retry or redirect. The engine must reserve its durable
    /// attempt and authorize egress before invoking this adapter. The resolver
    /// must be bounded by the supplied control; its error text is not exposed.
    /// Cancellation is checked before/after blocking I/O, not a remote stop.
    pub fn execute(
        &self,
        network: NetworkPolicy,
        control: &EmbeddingExecutionControl,
        resolve: impl FnOnce(
            &CredentialReference,
            &EmbeddingExecutionControl,
        ) -> std::result::Result<String, String>,
    ) -> Result<EndpointChatObservation> {
        if network != NetworkPolicy::Allow {
            return Err(failure("network denied", false));
        }
        checkpoint(control, false)?;
        let started = Instant::now();
        let local_deadline = started
            .checked_add(self.timeout)
            .ok_or_else(|| failure("invalid chat deadline", false))?;
        let control = control.with_deadline(local_deadline);
        let authorization = if let Some(reference) = self.credential {
            let secret = resolve(reference, &control)
                .map_err(|_| failure("credential unavailable", false))?;
            if secret.is_empty() || secret.len() > 16_384 || secret.chars().any(char::is_whitespace)
            {
                return Err(failure("invalid credential", false));
            }
            let mut value = ureq::http::HeaderValue::from_str(&format!("Bearer {secret}"))
                .map_err(|_| failure("invalid credential", false))?;
            value.set_sensitive(true);
            Some(value)
        } else {
            None
        };
        checkpoint(&control, false)?;
        let remaining = control
            .deadline()
            .checked_duration_since(Instant::now())
            .ok_or_else(|| failure("chat deadline exceeded", false))?;
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .proxy(None)
            .max_redirects(0)
            .http_status_as_error(false)
            .max_idle_connections(0)
            .timeout_global(Some(remaining))
            .build()
            .into();
        let mut request = agent
            .post(&self.url)
            .header("Content-Type", "application/json");
        if let Some(value) = authorization {
            request = request.header("Authorization", value);
        }
        checkpoint(&control, false)?;
        let mut response = request
            .send(self.request.as_slice())
            .map_err(|_| failure("chat transport failed; remote outcome unknown", true))?;
        checkpoint(&control, true)?;
        let status = response.status().as_u16();
        if status != 200 {
            let mut error = failure("chat endpoint did not return success", true);
            error.http_status = Some(status);
            return Err(error);
        }
        let maximum = self.allowance.limits.maximum_response_bytes;
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(maximum + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failure("chat response read failed; remote outcome unknown", true))?;
        checkpoint(&control, true)?;
        if bytes.len() as u64 > maximum {
            return Err(failure("chat response exceeds byte limit", true));
        }
        let mut observation = self.decode_response(bytes)?;
        checkpoint(&control, true)?;
        observation.elapsed_ms =
            Some(u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX));
        observation.http_status = Some(status);
        Ok(observation)
    }

    /// Offline decoding of actual retained output. No usage or identity field
    /// is independently attested merely because the provider supplied it.
    pub fn decode_response(&self, bytes: Vec<u8>) -> Result<EndpointChatObservation> {
        if bytes.len() as u64 > self.allowance.limits.maximum_response_bytes {
            return Err(failure("chat response exceeds byte limit", true));
        }
        let response: WireResponse = serde_json::from_slice(&bytes)
            .map_err(|_| failure("invalid chat response JSON", true))?;
        if response.model != self.service_id.as_str() || response.choices.len() != 1 {
            return Err(failure("chat response model or choice count differs", true));
        }
        let item = &response.choices[0];
        if item.finish_reason != "stop"
            || item.message.role != "assistant"
            || item.message.refusal.is_some()
            || item.message.function_call.is_some()
            || item.message.reasoning.is_some()
            || item.message.reasoning_content.is_some()
            || item
                .message
                .tool_calls
                .as_ref()
                .is_some_and(|calls| !calls.is_empty())
        {
            return Err(failure(
                "chat response is incomplete or requests unsupported effects",
                true,
            ));
        }
        let content = item
            .message
            .content
            .as_ref()
            .ok_or_else(|| failure("chat response has no content", true))?;
        let choice =
            EndpointSelectionChoice::decode(content.as_bytes(), self.allowance, self.configuration)
                .map_err(|_| failure("invalid endpoint selection choice", true))?;
        let usage: Option<EndpointTokenUsage> = response.usage.map(Into::into);
        if let Some(usage) = &usage {
            usage
                .validate()
                .map_err(|_| failure("inconsistent reported usage", true))?;
        }
        Ok(EndpointChatObservation {
            choice,
            service_id: response.model,
            configuration_sha256: self.allowance.configuration_sha256.clone(),
            allowance_sha256: self
                .allowance
                .sha256(self.configuration)
                .map_err(|_| failure("invalid selection allowance", true))?,
            request_sha256: sha256_hex(&self.request),
            response_sha256: sha256_hex(&bytes),
            content_sha256: sha256_hex(content.as_bytes()),
            usage,
            elapsed_ms: None,
            http_status: None,
            response_bytes: bytes,
        })
    }
}

fn checkpoint(control: &EmbeddingExecutionControl, sent: bool) -> Result<()> {
    control.check().map_err(|error| {
        failure(
            match error {
                EmbeddingDispatchError::Cancelled { .. } => "chat cancelled",
                _ => "chat deadline exceeded",
            },
            sent,
        )
    })
}

fn encode_bounded(value: &impl Serialize, maximum: u64) -> Result<Vec<u8>> {
    struct Buffer {
        bytes: Vec<u8>,
        remaining: u64,
    }
    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.remaining = self
                .remaining
                .checked_sub(bytes.len() as u64)
                .ok_or_else(|| std::io::Error::other("request too large"))?;
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer {
        bytes: Vec::new(),
        remaining: maximum,
    };
    serde_json::to_writer(&mut buffer, value)
        .map_err(|_| failure("chat request exceeds byte limit or is invalid", false))?;
    Ok(buffer.bytes)
}

// Unknown provider envelope metadata is ignored within the raw byte cap. Known
// fields retain serde's duplicate-field checks; the actual choice is closed.
#[derive(Deserialize)]
struct WireResponse {
    model: String,
    choices: Vec<WireChoice>,
    usage: Option<ChatUsage>,
}
#[derive(Deserialize)]
struct WireChoice {
    finish_reason: String,
    message: WireMessage,
}
#[derive(Deserialize)]
struct WireMessage {
    role: String,
    content: Option<String>,
    refusal: Option<serde_json::Value>,
    function_call: Option<serde_json::Value>,
    reasoning: Option<serde_json::Value>,
    reasoning_content: Option<serde_json::Value>,
    tool_calls: Option<Vec<serde_json::Value>>,
}
