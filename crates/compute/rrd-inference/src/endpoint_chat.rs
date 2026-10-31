//! Optional, stateless transport for explicitly prepared model requests.
//! This is not engine admission, a worker dispatcher, or a provider hard-stop.
use crate::{EmbeddingDispatchError, EmbeddingExecutionControl, NetworkPolicy};
use rrd_contract::{
    CanonicalId, CredentialReference, EndpointConfiguration, EndpointGenerationSettings,
    EndpointLocator, EndpointResponseFormat, EndpointSelectionAllowance, EndpointSelectionChoice,
    EndpointTokenParameter, EndpointTokenUsage, WorkloadClass, MAX_ROUTE_REQUEST_BYTES,
    MAX_ROUTE_RESPONSE_BYTES,
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

/// Consistent wire input, not an engine authorization or a project effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelChatRequest {
    pub model: CanonicalId,
    pub messages: Vec<ChatMessage>,
    pub settings: EndpointGenerationSettings,
    /// Exact configured value; the engine must verify support and selection.
    pub reasoning_effort: Option<String>,
}

#[derive(Serialize)]
struct ModelWireRequest<'a> {
    model: &'a CanonicalId,
    messages: &'a [ChatMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    response_format: EndpointResponseFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointChatFailure {
    pub reason: &'static str,
    pub outcome: EndpointChatOutcome,
    pub http_status: Option<u16>,
}

/// Transport observation, not attested model identity or permission to retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointChatOutcome {
    NotDispatched,
    Unknown,
    /// A bounded, matching, terminal generation response was decoded. Its
    /// content can still be rejected and must never bypass engine admission.
    ResponseComplete,
}

impl EndpointChatFailure {
    pub fn dispatch_may_have_occurred(&self) -> bool {
        self.outcome != EndpointChatOutcome::NotDispatched
    }

    pub fn execution_may_continue(&self) -> bool {
        self.outcome == EndpointChatOutcome::Unknown
    }
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
        outcome: if sent {
            EndpointChatOutcome::Unknown
        } else {
            EndpointChatOutcome::NotDispatched
        },
        http_status: None,
    }
}

fn completed_failure(reason: &'static str) -> EndpointChatFailure {
    EndpointChatFailure {
        reason,
        outcome: EndpointChatOutcome::ResponseComplete,
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

/// A bounded provider report. Content is a draft and usage/identity are not
/// independently attested. No correctness, mutation or training claim follows.
#[derive(Debug, Serialize)]
pub struct ModelChatObservation {
    pub content: String,
    pub service_id: String,
    pub configuration_sha256: String,
    pub request_sha256: String,
    pub response_sha256: String,
    pub content_sha256: String,
    pub usage: Option<EndpointTokenUsage>,
    pub elapsed_ms: Option<u64>,
    pub http_status: Option<u16>,
    #[serde(skip)]
    pub response_bytes: Vec<u8>,
}

/// Validates configuration and prepares bytes without resolving credentials or
/// performing network work. Construction proves consistency, not authorization.
pub struct PreparedModelChat<'a> {
    configuration_sha256: String,
    maximum_response_bytes: u64,
    response_format: EndpointResponseFormat,
    credential: Option<&'a CredentialReference>,
    service_id: &'a CanonicalId,
    url: String,
    request: Vec<u8>,
    timeout: Duration,
}

impl<'a> PreparedModelChat<'a> {
    pub fn new(
        configuration: &'a EndpointConfiguration,
        binding_id: &CanonicalId,
        request: &ModelChatRequest,
        maximum_response_bytes: u64,
    ) -> Result<Self> {
        request
            .settings
            .validate()
            .map_err(|_| failure("invalid chat generation bounds", false))?;
        if request.reasoning_effort.as_ref().is_some_and(|value| {
            value.is_empty()
                || value.len() as u64 > MAX_ROUTE_REQUEST_BYTES
                || value.trim() != value
                || value.chars().any(char::is_control)
        }) {
            return Err(failure("invalid configured reasoning effort", false));
        }
        let settings = &request.settings;
        let wire = ModelWireRequest {
            model: &request.model,
            messages: &request.messages,
            max_tokens: (settings.token_parameter == EndpointTokenParameter::MaxTokens)
                .then_some(settings.maximum_output_tokens),
            max_completion_tokens: (settings.token_parameter
                == EndpointTokenParameter::MaxCompletionTokens)
                .then_some(settings.maximum_output_tokens),
            temperature: settings
                .temperature_milli
                .map(|value| f64::from(value) / 1000.0),
            response_format: settings.response_format,
            reasoning_effort: request.reasoning_effort.as_deref(),
        };
        Self::prepare(
            configuration,
            binding_id,
            &request.model,
            &request.messages,
            &wire,
            maximum_response_bytes,
            settings.response_format,
        )
    }

    fn prepare(
        configuration: &'a EndpointConfiguration,
        binding_id: &CanonicalId,
        model: &CanonicalId,
        messages: &[ChatMessage],
        wire: &impl Serialize,
        maximum_response_bytes: u64,
        response_format: EndpointResponseFormat,
    ) -> Result<Self> {
        let configuration_sha256 = configuration
            .sha256()
            .map_err(|_| failure("invalid endpoint configuration", false))?;
        if maximum_response_bytes == 0 || maximum_response_bytes > MAX_ROUTE_RESPONSE_BYTES {
            return Err(failure("invalid chat response bound", false));
        }
        let binding = configuration
            .profile
            .as_ref()
            .and_then(|p| p.bindings.iter().find(|b| &b.id == binding_id))
            .ok_or_else(|| failure("model binding is not configured", false))?;
        if !matches!(
            binding.class,
            WorkloadClass::LargeModelServing | WorkloadClass::AlwaysWarmSmallModels
        ) || &binding.service_id != model
        {
            return Err(failure(
                "request differs from configured model binding",
                false,
            ));
        }
        if messages.is_empty() || messages.len() > 128 {
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
        let request = encode_bounded(wire, limit)?;
        Ok(Self {
            configuration_sha256,
            maximum_response_bytes,
            response_format,
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
    ) -> Result<ModelChatObservation> {
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
        let maximum = self.maximum_response_bytes;
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(maximum + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failure("chat response read failed; remote outcome unknown", true))?;
        if bytes.len() as u64 > maximum {
            return Err(failure("chat response exceeds byte limit", true));
        }
        // Decode the bounded body before the final cancellation check so a
        // completed response is not confused with an unknown remote outcome.
        // Cancellation still forbids returning its content as accepted output.
        let mut observation = self.decode_response(bytes).map_err(|mut error| {
            error.http_status = Some(status);
            error
        })?;
        completed_checkpoint(&control)?;
        observation.elapsed_ms =
            Some(u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX));
        observation.http_status = Some(status);
        Ok(observation)
    }

    /// Offline decoding of actual retained output. No usage or identity field
    /// is independently attested merely because the provider supplied it.
    pub fn decode_response(&self, bytes: Vec<u8>) -> Result<ModelChatObservation> {
        if bytes.len() as u64 > self.maximum_response_bytes {
            return Err(failure("chat response exceeds byte limit", true));
        }
        let response: WireResponse = serde_json::from_slice(&bytes)
            .map_err(|_| failure("invalid chat response JSON", true))?;
        if response.model != self.service_id.as_str() || response.choices.len() != 1 {
            return Err(failure("chat response model or choice count differs", true));
        }
        let item = &response.choices[0];
        if item.message.role != "assistant"
            || !matches!(
                item.finish_reason.as_str(),
                "stop" | "length" | "content_filter"
            )
            || item.message.function_call.is_some()
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
        if item.finish_reason != "stop"
            || item.message.refusal.is_some()
            || item.message.reasoning.is_some()
            || item.message.reasoning_content.is_some()
        {
            return Err(completed_failure(
                "chat response is incomplete or requests unsupported effects",
            ));
        }
        let content = item
            .message
            .content
            .as_ref()
            .ok_or_else(|| completed_failure("chat response has no content"))?;
        if self.response_format == EndpointResponseFormat::JsonObject
            && !serde_json::from_str::<serde_json::Value>(content)
                .is_ok_and(|value| value.is_object())
        {
            return Err(completed_failure(
                "chat content is not the requested JSON object",
            ));
        }
        let usage: Option<EndpointTokenUsage> = response.usage.map(Into::into);
        if let Some(usage) = &usage {
            usage
                .validate()
                .map_err(|_| completed_failure("inconsistent reported usage"))?;
        }
        Ok(ModelChatObservation {
            content: content.clone(),
            service_id: response.model,
            configuration_sha256: self.configuration_sha256.clone(),
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

/// Strict selector decoder over the same transport. Generation text never
/// bypasses the configured allowance or becomes an eligible selection by itself.
pub struct PreparedEndpointChat<'a> {
    configuration: &'a EndpointConfiguration,
    allowance: &'a EndpointSelectionAllowance,
    transport: PreparedModelChat<'a>,
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
        if request.max_tokens == 0
            || request.max_tokens > 65_536
            || !request.temperature.is_finite()
            || !(0.0..=2.0).contains(&request.temperature)
        {
            return Err(failure("invalid chat generation bounds", false));
        }
        // Serialize the existing selector wire unchanged, including field order.
        let transport = PreparedModelChat::prepare(
            configuration,
            controller_binding,
            &request.model,
            &request.messages,
            request,
            allowance.limits.maximum_response_bytes,
            EndpointResponseFormat::JsonObject,
        )?;
        Ok(Self {
            configuration,
            allowance,
            transport,
        })
    }

    pub fn request_bytes(&self) -> &[u8] {
        self.transport.request_bytes()
    }

    pub fn execute(
        &self,
        network: NetworkPolicy,
        control: &EmbeddingExecutionControl,
        resolve: impl FnOnce(
            &CredentialReference,
            &EmbeddingExecutionControl,
        ) -> std::result::Result<String, String>,
    ) -> Result<EndpointChatObservation> {
        let started = Instant::now();
        let deadline = started
            .checked_add(self.transport.timeout)
            .ok_or_else(|| failure("invalid chat deadline", false))?;
        let control = control.with_deadline(deadline);
        let observed = self.transport.execute(network, &control, resolve)?;
        let mut selection = self.selection(observed)?;
        completed_checkpoint(&control)?;
        selection.elapsed_ms =
            Some(u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX));
        Ok(selection)
    }

    pub fn decode_response(&self, bytes: Vec<u8>) -> Result<EndpointChatObservation> {
        self.selection(self.transport.decode_response(bytes)?)
    }

    fn selection(&self, observed: ModelChatObservation) -> Result<EndpointChatObservation> {
        let choice = EndpointSelectionChoice::decode(
            observed.content.as_bytes(),
            self.allowance,
            self.configuration,
        )
        .map_err(|_| completed_failure("invalid endpoint selection choice"))?;
        Ok(EndpointChatObservation {
            choice,
            service_id: observed.service_id,
            configuration_sha256: observed.configuration_sha256,
            allowance_sha256: self
                .allowance
                .sha256(self.configuration)
                .map_err(|_| completed_failure("invalid selection allowance"))?,
            request_sha256: observed.request_sha256,
            response_sha256: observed.response_sha256,
            content_sha256: observed.content_sha256,
            usage: observed.usage,
            elapsed_ms: observed.elapsed_ms,
            http_status: observed.http_status,
            response_bytes: observed.response_bytes,
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

fn completed_checkpoint(control: &EmbeddingExecutionControl) -> Result<()> {
    checkpoint(control, true).map_err(|error| completed_failure(error.reason))
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
