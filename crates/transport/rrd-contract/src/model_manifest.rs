//! Closed, provider-neutral identity and admission contracts for router models.
//!
//! A manifest describes immutable model, tokenizer, runtime, and constrained-
//! decoding grammar artifacts without naming a filesystem path, download URL,
//! provider, or secret. A runtime handshake independently reports what it is
//! about to load. `rrd-inference` must validate both declarations and the
//! supplied bytes before constructing a model runtime.

use crate::{
    canonical_json, invalid, route_step_decision_schema_sha256, sha256_bytes, validate_sha256,
    CanonicalId, Result, RouteDecisionKind, RouterBackendDescriptor, RouterBackendLimits,
    ROUTER_CONTRACT_VERSION,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const ROUTER_MODEL_MANIFEST_CONTRACT_VERSION: u16 = 1;
pub const MAX_ROUTER_ARTIFACT_MEDIA_TYPE_BYTES: usize = 128;
pub const MAX_ROUTER_MODEL_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024 * 1024;
pub const MAX_ROUTER_TOKENIZER_ARTIFACT_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_ROUTER_RUNTIME_ARTIFACT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const MAX_ROUTER_GRAMMAR_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_ROUTER_RESIDENT_BYTES: u64 = 128 * 1024 * 1024 * 1024;
pub const MAX_ROUTER_CONTEXT_TOKENS: u64 = 4_194_304;
pub const MAX_ROUTER_OUTPUT_TOKENS: u64 = 262_144;
pub const MAX_ROUTER_THREADS: u32 = 1_024;

/// Content identity for one immutable artifact.
///
/// `format` identifies the bytes' encoding, while `media_type` identifies the
/// transport representation. Locating and acquiring these bytes belongs to an
/// installed adapter and is deliberately absent from the canonical manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterArtifactDescriptor {
    pub media_type: String,
    pub format: CanonicalId,
    pub format_revision: u64,
    pub encoded_bytes: u64,
    pub sha256: String,
}

impl RouterArtifactDescriptor {
    fn validate(&self, label: &str, maximum_bytes: u64) -> Result<()> {
        validate_media_type(&self.media_type, label)?;
        if self.format_revision == 0 {
            return invalid(format!("{label} format revision must be greater than zero"));
        }
        validate_limit(
            self.encoded_bytes,
            maximum_bytes,
            &format!("{label} encoded bytes"),
        )?;
        validate_sha256(&self.sha256, &format!("{label} sha256"))
    }
}

/// Hard admission limits declared by a model installation.
///
/// The limits are data, not baked-in assumptions about a particular local
/// model. An installation may choose any smaller values within these protocol
/// ceilings. Input plus generated output must fit the declared context window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterModelLimits {
    pub maximum_model_bytes: u64,
    pub maximum_tokenizer_bytes: u64,
    pub maximum_runtime_bytes: u64,
    pub maximum_grammar_bytes: u64,
    pub maximum_resident_bytes: u64,
    pub maximum_context_tokens: u64,
    pub maximum_input_tokens: u64,
    pub maximum_output_tokens: u64,
    pub maximum_threads: u32,
}

impl RouterModelLimits {
    pub fn validate(&self) -> Result<()> {
        validate_limit(
            self.maximum_model_bytes,
            MAX_ROUTER_MODEL_ARTIFACT_BYTES,
            "router maximum_model_bytes",
        )?;
        validate_limit(
            self.maximum_tokenizer_bytes,
            MAX_ROUTER_TOKENIZER_ARTIFACT_BYTES,
            "router maximum_tokenizer_bytes",
        )?;
        validate_limit(
            self.maximum_runtime_bytes,
            MAX_ROUTER_RUNTIME_ARTIFACT_BYTES,
            "router maximum_runtime_bytes",
        )?;
        validate_limit(
            self.maximum_grammar_bytes,
            MAX_ROUTER_GRAMMAR_ARTIFACT_BYTES,
            "router maximum_grammar_bytes",
        )?;
        validate_limit(
            self.maximum_resident_bytes,
            MAX_ROUTER_RESIDENT_BYTES,
            "router maximum_resident_bytes",
        )?;
        validate_limit(
            self.maximum_context_tokens,
            MAX_ROUTER_CONTEXT_TOKENS,
            "router maximum_context_tokens",
        )?;
        validate_limit(
            self.maximum_input_tokens,
            MAX_ROUTER_CONTEXT_TOKENS,
            "router maximum_input_tokens",
        )?;
        validate_limit(
            self.maximum_output_tokens,
            MAX_ROUTER_OUTPUT_TOKENS,
            "router maximum_output_tokens",
        )?;
        validate_limit(
            u64::from(self.maximum_threads),
            u64::from(MAX_ROUTER_THREADS),
            "router maximum_threads",
        )?;
        let requested_tokens = self
            .maximum_input_tokens
            .checked_add(self.maximum_output_tokens)
            .ok_or_else(|| {
                crate::ContractError("router input/output token limit overflowed u64".into())
            })?;
        if requested_tokens > self.maximum_context_tokens {
            return invalid("router input and output token limits exceed the context window");
        }
        Ok(())
    }
}

/// Exact executable runtime and ABI selected for this model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterRuntimeBinding {
    pub id: CanonicalId,
    pub revision: u64,
    pub abi: CanonicalId,
    pub abi_revision: u64,
    pub model_format: CanonicalId,
    pub model_format_revision: u64,
    pub tokenizer_format: CanonicalId,
    pub tokenizer_format_revision: u64,
    pub device_class: CanonicalId,
    pub artifact: RouterArtifactDescriptor,
    pub configuration_sha256: String,
}

impl RouterRuntimeBinding {
    fn validate(&self, limits: &RouterModelLimits) -> Result<()> {
        if self.revision == 0
            || self.abi_revision == 0
            || self.model_format_revision == 0
            || self.tokenizer_format_revision == 0
        {
            return invalid("router runtime revisions must be greater than zero");
        }
        self.artifact
            .validate("router runtime artifact", limits.maximum_runtime_bytes)?;
        validate_sha256(
            &self.configuration_sha256,
            "router runtime configuration_sha256",
        )
    }
}

/// Model quantization is explicit and extensible without granting it storage
/// or index authority. `None` means the model bytes are not quantized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RouterQuantizationBinding {
    None,
    Quantized {
        scheme: CanonicalId,
        revision: u64,
        configuration_sha256: String,
    },
}

impl RouterQuantizationBinding {
    fn validate(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Quantized {
                revision,
                configuration_sha256,
                ..
            } => {
                if *revision == 0 {
                    return invalid("router quantization revision must be greater than zero");
                }
                validate_sha256(
                    configuration_sha256,
                    "router quantization configuration_sha256",
                )
            }
        }
    }
}

/// Exact constrained-decoding grammar derived from the routing output schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterGrammarBinding {
    pub revision: u64,
    pub source_schema_sha256: String,
    pub artifact: RouterArtifactDescriptor,
}

impl RouterGrammarBinding {
    fn validate(&self, limits: &RouterModelLimits) -> Result<()> {
        if self.revision == 0 {
            return invalid("router grammar revision must be greater than zero");
        }
        validate_sha256(
            &self.source_schema_sha256,
            "router grammar source_schema_sha256",
        )?;
        self.artifact
            .validate("router grammar artifact", limits.maximum_grammar_bytes)
    }
}

/// Canonical identity and limits for one route-proposal model installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterModelManifest {
    pub contract_version: u16,
    pub id: CanonicalId,
    pub revision: u64,
    pub router_contract_version: u16,
    pub model: RouterArtifactDescriptor,
    pub tokenizer: RouterArtifactDescriptor,
    pub routing_schema_sha256: String,
    pub decisions: BTreeSet<RouteDecisionKind>,
    pub backend_limits: RouterBackendLimits,
    pub model_limits: RouterModelLimits,
    pub runtime: RouterRuntimeBinding,
    pub quantization: RouterQuantizationBinding,
    pub grammar: RouterGrammarBinding,
    pub manifest_sha256: String,
}

impl RouterModelManifest {
    pub fn validate(&self) -> Result<()> {
        validate_manifest_contract_version(self.contract_version)?;
        if self.revision == 0 {
            return invalid("router model manifest revision must be greater than zero");
        }
        if self.router_contract_version != ROUTER_CONTRACT_VERSION {
            return invalid(format!(
                "unsupported manifest router contract version {}; expected {ROUTER_CONTRACT_VERSION}",
                self.router_contract_version
            ));
        }
        if self.decisions.is_empty() {
            return invalid("router model manifest must declare at least one routing decision");
        }
        self.backend_limits.validate()?;
        self.model_limits.validate()?;
        self.model.validate(
            "router model artifact",
            self.model_limits.maximum_model_bytes,
        )?;
        self.tokenizer.validate(
            "router tokenizer artifact",
            self.model_limits.maximum_tokenizer_bytes,
        )?;
        validate_sha256(
            &self.routing_schema_sha256,
            "router manifest routing_schema_sha256",
        )?;
        let expected_schema = route_step_decision_schema_sha256()?;
        if self.routing_schema_sha256 != expected_schema {
            return invalid("router manifest routing schema differs from the contract schema");
        }
        self.runtime.validate(&self.model_limits)?;
        if self.runtime.model_format != self.model.format
            || self.runtime.model_format_revision != self.model.format_revision
        {
            return invalid("router runtime model format differs from the model artifact");
        }
        if self.runtime.tokenizer_format != self.tokenizer.format
            || self.runtime.tokenizer_format_revision != self.tokenizer.format_revision
        {
            return invalid("router runtime tokenizer format differs from the tokenizer artifact");
        }
        self.quantization.validate()?;
        self.grammar.validate(&self.model_limits)?;
        if self.grammar.source_schema_sha256 != self.routing_schema_sha256 {
            return invalid("router grammar source schema differs from the routing schema");
        }
        validate_sha256(&self.manifest_sha256, "router manifest_sha256")?;
        if self.manifest_sha256 != router_model_manifest_sha256(self)? {
            return invalid("router model manifest digest does not match its content");
        }
        Ok(())
    }

    pub fn validate_for_backend(&self, backend: &RouterBackendDescriptor) -> Result<()> {
        self.validate()?;
        backend.validate()?;
        if backend.model_manifest_id != self.id
            || backend.model_manifest_revision != self.revision
            || backend.model_manifest_sha256 != self.manifest_sha256
        {
            return invalid("router backend binds a different model manifest");
        }
        if backend.decisions != self.decisions {
            return invalid("router backend decisions differ from the model manifest");
        }
        if backend.limits != self.backend_limits {
            return invalid("router backend limits differ from the model manifest");
        }
        Ok(())
    }
}

/// Independent runtime observation made immediately before model loading.
///
/// Repetition here is intentional: comparing a separately produced runtime
/// declaration to the signed/installed manifest detects drift instead of
/// trusting the manifest as a report of current process state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouterModelHandshake {
    pub contract_version: u16,
    pub manifest_id: CanonicalId,
    pub manifest_revision: u64,
    pub manifest_sha256: String,
    pub backend_id: CanonicalId,
    pub backend_revision: u64,
    pub backend_descriptor_sha256: String,
    pub model: RouterArtifactDescriptor,
    pub tokenizer: RouterArtifactDescriptor,
    pub routing_schema_sha256: String,
    pub decisions: BTreeSet<RouteDecisionKind>,
    pub backend_limits: RouterBackendLimits,
    pub model_limits: RouterModelLimits,
    pub runtime: RouterRuntimeBinding,
    pub quantization: RouterQuantizationBinding,
    pub grammar: RouterGrammarBinding,
    pub handshake_sha256: String,
}

impl RouterModelHandshake {
    pub fn validate_for(
        &self,
        manifest: &RouterModelManifest,
        backend: &RouterBackendDescriptor,
    ) -> Result<()> {
        manifest.validate_for_backend(backend)?;
        validate_manifest_contract_version(self.contract_version)?;
        if self.manifest_revision == 0 || self.backend_revision == 0 {
            return invalid("router handshake revisions must be greater than zero");
        }
        validate_sha256(&self.manifest_sha256, "router handshake manifest_sha256")?;
        validate_sha256(
            &self.backend_descriptor_sha256,
            "router handshake backend_descriptor_sha256",
        )?;
        validate_sha256(
            &self.routing_schema_sha256,
            "router handshake routing_schema_sha256",
        )?;
        validate_sha256(&self.handshake_sha256, "router handshake_sha256")?;
        if self.handshake_sha256 != router_model_handshake_sha256(self)? {
            return invalid("router model handshake digest does not match its content");
        }
        if self.manifest_id != manifest.id
            || self.manifest_revision != manifest.revision
            || self.manifest_sha256 != manifest.manifest_sha256
        {
            return invalid("router runtime observed a different model manifest");
        }
        if self.backend_id != backend.id
            || self.backend_revision != backend.revision
            || self.backend_descriptor_sha256 != backend.descriptor_sha256
        {
            return invalid("router runtime observed a different backend descriptor");
        }
        if self.model != manifest.model {
            return invalid("router runtime model declaration differs from the manifest");
        }
        if self.tokenizer != manifest.tokenizer {
            return invalid("router runtime tokenizer declaration differs from the manifest");
        }
        if self.routing_schema_sha256 != manifest.routing_schema_sha256 {
            return invalid("router runtime routing schema differs from the manifest");
        }
        if self.decisions != manifest.decisions {
            return invalid("router runtime decisions differ from the manifest");
        }
        if self.backend_limits != manifest.backend_limits {
            return invalid("router runtime backend limits differ from the manifest");
        }
        if self.model_limits != manifest.model_limits {
            return invalid("router runtime model limits differ from the manifest");
        }
        if self.runtime != manifest.runtime {
            return invalid("router runtime binding differs from the manifest");
        }
        if self.quantization != manifest.quantization {
            return invalid("router runtime quantization differs from the manifest");
        }
        if self.grammar != manifest.grammar {
            return invalid("router runtime grammar differs from the manifest");
        }
        Ok(())
    }
}

pub fn router_model_manifest_sha256(manifest: &RouterModelManifest) -> Result<String> {
    digest_json(
        b"rrflow-router-model-manifest-v1",
        &(
            manifest.contract_version,
            &manifest.id,
            manifest.revision,
            manifest.router_contract_version,
            &manifest.model,
            &manifest.tokenizer,
            &manifest.routing_schema_sha256,
            &manifest.decisions,
            &manifest.backend_limits,
            &manifest.model_limits,
            &manifest.runtime,
            &manifest.quantization,
            &manifest.grammar,
        ),
    )
}

pub fn router_model_handshake_sha256(handshake: &RouterModelHandshake) -> Result<String> {
    digest_json(
        b"rrflow-router-model-handshake-v1",
        &(
            handshake.contract_version,
            &handshake.manifest_id,
            handshake.manifest_revision,
            &handshake.manifest_sha256,
            &handshake.backend_id,
            handshake.backend_revision,
            &handshake.backend_descriptor_sha256,
            &handshake.model,
            &handshake.tokenizer,
            &handshake.routing_schema_sha256,
            &handshake.decisions,
            &handshake.backend_limits,
            &handshake.model_limits,
            &handshake.runtime,
            &handshake.quantization,
            &handshake.grammar,
        ),
    )
}

fn validate_manifest_contract_version(version: u16) -> Result<()> {
    if version != ROUTER_MODEL_MANIFEST_CONTRACT_VERSION {
        return invalid(format!(
            "unsupported router model manifest contract version {version}; expected {ROUTER_MODEL_MANIFEST_CONTRACT_VERSION}"
        ));
    }
    Ok(())
}

fn validate_media_type(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > MAX_ROUTER_ARTIFACT_MEDIA_TYPE_BYTES
        || value
            .bytes()
            .any(|byte| byte != b'/' && !is_media_type_token(byte))
        || value.matches('/').count() != 1
        || value.starts_with('/')
        || value.ends_with('/')
    {
        return invalid(format!(
            "{label} media_type must be one bounded lowercase canonical media type"
        ));
    }
    Ok(())
}

fn is_media_type_token(byte: u8) -> bool {
    byte.is_ascii_lowercase()
        || byte.is_ascii_digit()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn validate_limit(value: u64, maximum: u64, field: &str) -> Result<()> {
    if value == 0 || value > maximum {
        return invalid(format!("{field} must be in 1..={maximum}"));
    }
    Ok(())
}

fn digest_json<T: Serialize>(domain: &[u8], value: &T) -> Result<String> {
    let value = serde_json::to_value(value).map_err(|error| {
        crate::ContractError(format!(
            "router model digest value encoding failed: {error}"
        ))
    })?;
    let encoded = serde_json::to_vec(&canonical_json(value)).map_err(|error| {
        crate::ContractError(format!("router model digest encoding failed: {error}"))
    })?;
    let mut bytes = Vec::with_capacity(domain.len() + 1 + encoded.len());
    bytes.extend_from_slice(domain);
    bytes.push(0);
    bytes.extend_from_slice(&encoded);
    Ok(sha256_bytes(&bytes))
}
