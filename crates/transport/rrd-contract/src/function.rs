use crate::{invalid, validate_sha256, CanonicalId, QueryValue, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const FUNCTION_CONTRACT_VERSION: u16 = 1;
pub const MAX_FUNCTIONS_PER_CATALOGUE: usize = 64;
pub const MAX_TRANSACTION_FUNCTION_BINDINGS_PER_CATALOGUE: usize = 128;
pub const MAX_FUNCTION_SOURCE_BYTES: usize = 64 * 1024;
pub const MAX_FUNCTION_WASM_BYTES: usize = 256 * 1024;
/// Maximum decoded executable content in one catalogue replacement.
///
/// This aggregate bound is intentionally lower than the sum of every
/// per-artifact maximum: one replacement must remain one atomic rrflowKV
/// batch after typed definitions, bindings, membership, keys, and framing are
/// added.
pub const MAX_FUNCTION_CATALOGUE_ARTIFACT_BYTES: usize = 3 * 1024 * 1024;
/// Maximum canonical JSON bytes accepted for one function catalogue.
///
/// The public representation includes base64 artifacts. The physical store
/// performs a second exact current-format admission check; this conservative
/// public bound leaves room for immutable-record envelopes and escaped
/// canonical definition/binding JSON without exposing the WAL format here.
pub const MAX_FUNCTION_CATALOGUE_ENCODED_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_FUNCTION_INPUT_BYTES: usize = 256 * 1024;
pub const MAX_FUNCTION_OUTPUT_BYTES: usize = 256 * 1024;
pub const MAX_FUNCTION_VALUE_DEPTH: usize = 64;
pub const MAX_FUNCTION_VALUE_ITEMS: usize = 8_192;
pub const MAX_FUNCTION_SCHEMA_DEPTH: usize = 32;
pub const MAX_FUNCTION_SCHEMA_ITEMS: usize = 1_024;
pub const MAX_FUNCTION_JSON_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MIN_FUNCTION_MEMORY_BYTES: u64 = 64 * 1024;
const MAX_FUNCTION_MEMORY_BYTES: u64 = 64 * 1024 * 1024;
const MIN_FUNCTION_STACK_BYTES: u32 = 16 * 1024;
const MAX_FUNCTION_STACK_BYTES: u32 = 1024 * 1024;
const MAX_FUNCTION_INTERRUPTS: u32 = 1_000_000;
const MAX_FUNCTION_FUEL: u64 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FunctionRuntimeKind {
    JavaScriptEs2020,
    WebAssemblyV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FunctionArtifactMediaType {
    JavaScriptUtf8,
    WebAssemblyBinary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionArtifact {
    pub content_sha256: String,
    pub media_type: FunctionArtifactMediaType,
    pub byte_length: u32,
    pub content_base64: String,
}

impl FunctionArtifact {
    pub fn decoded_content(&self) -> Result<Vec<u8>> {
        validate_sha256(&self.content_sha256, "function artifact content_sha256")?;
        let content = STANDARD
            .decode(&self.content_base64)
            .map_err(|_| crate::ContractError("function artifact content is not base64".into()))?;
        let expected_length: usize = self
            .byte_length
            .try_into()
            .map_err(|_| crate::ContractError("function artifact length exceeds usize".into()))?;
        if content.len() != expected_length || content.is_empty() {
            return invalid("function artifact byte_length does not match non-empty content");
        }
        let maximum = match self.media_type {
            FunctionArtifactMediaType::JavaScriptUtf8 => MAX_FUNCTION_SOURCE_BYTES,
            FunctionArtifactMediaType::WebAssemblyBinary => MAX_FUNCTION_WASM_BYTES,
        };
        if content.len() > maximum {
            return invalid(format!(
                "function artifact exceeds its {maximum}-byte media bound"
            ));
        }
        if sha256(&content) != self.content_sha256 {
            return invalid("function artifact digest does not match decoded content");
        }
        match self.media_type {
            FunctionArtifactMediaType::JavaScriptUtf8 => {
                let source = std::str::from_utf8(&content).map_err(|_| {
                    crate::ContractError("JavaScript function artifact is not UTF-8".into())
                })?;
                if source.trim().is_empty() {
                    return invalid("JavaScript function artifact is empty");
                }
            }
            FunctionArtifactMediaType::WebAssemblyBinary => {
                if !content.starts_with(b"\0asm") {
                    return invalid("WebAssembly function artifact lacks the module magic");
                }
            }
        }
        Ok(content)
    }

    pub fn validate(&self) -> Result<()> {
        self.decoded_content().map(|_| ())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebAssemblyAbi {
    /// The module exports `memory`, `rrd_alloc(i32) -> i32`, and
    /// `rrd_run(i32, i32) -> i64`. The host writes canonical JSON input at
    /// the allocated pointer. The result packs an output pointer in the high
    /// 32 bits and a UTF-8 JSON byte length in the low 32 bits.
    JsonV1,
}

/// An executable reference. Artifact bytes live once in the catalogue's
/// content-addressed artifact set and never inside a definition or membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "runtime", rename_all = "snake_case", deny_unknown_fields)]
pub enum FunctionRuntime {
    JavaScriptEs2020 {
        artifact_sha256: String,
        runtime_profile: CanonicalId,
        runtime_build_sha256: String,
    },
    WebAssemblyV1 {
        artifact_sha256: String,
        runtime_profile: CanonicalId,
        runtime_build_sha256: String,
        abi: WebAssemblyAbi,
    },
}

impl FunctionRuntime {
    pub fn kind(&self) -> FunctionRuntimeKind {
        match self {
            Self::JavaScriptEs2020 { .. } => FunctionRuntimeKind::JavaScriptEs2020,
            Self::WebAssemblyV1 { .. } => FunctionRuntimeKind::WebAssemblyV1,
        }
    }

    pub fn artifact_sha256(&self) -> &str {
        match self {
            Self::JavaScriptEs2020 {
                artifact_sha256, ..
            }
            | Self::WebAssemblyV1 {
                artifact_sha256, ..
            } => artifact_sha256,
        }
    }

    pub fn runtime_profile(&self) -> &CanonicalId {
        match self {
            Self::JavaScriptEs2020 {
                runtime_profile, ..
            }
            | Self::WebAssemblyV1 {
                runtime_profile, ..
            } => runtime_profile,
        }
    }

    pub fn runtime_build_sha256(&self) -> &str {
        match self {
            Self::JavaScriptEs2020 {
                runtime_build_sha256,
                ..
            }
            | Self::WebAssemblyV1 {
                runtime_build_sha256,
                ..
            } => runtime_build_sha256,
        }
    }

    pub fn expected_media_type(&self) -> FunctionArtifactMediaType {
        match self {
            Self::JavaScriptEs2020 { .. } => FunctionArtifactMediaType::JavaScriptUtf8,
            Self::WebAssemblyV1 { .. } => FunctionArtifactMediaType::WebAssemblyBinary,
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_sha256(self.artifact_sha256(), "function artifact_sha256")?;
        validate_sha256(self.runtime_build_sha256(), "function runtime_build_sha256")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "shape", rename_all = "snake_case", deny_unknown_fields)]
pub enum FunctionValueShape {
    CanonicalValueV1,
    Null,
    Boolean,
    Integer,
    Unsigned,
    Decimal,
    String {
        max_bytes: u32,
    },
    Digest,
    List {
        items: Box<FunctionValueShape>,
        max_items: u32,
    },
    Object {
        #[serde(default)]
        properties: BTreeMap<String, FunctionValueShape>,
        #[serde(default)]
        required: BTreeSet<String>,
        allow_additional_properties: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionValueSchema {
    pub schema_id: CanonicalId,
    pub revision: u64,
    pub shape: FunctionValueShape,
}

impl FunctionValueSchema {
    pub fn validate(&self) -> Result<()> {
        if self.revision == 0 {
            return invalid("function value schema revision must be non-zero");
        }
        let mut items = 0_usize;
        validate_schema_shape(&self.shape, 0, &mut items)
    }

    pub fn validate_value(&self, value: &QueryValue) -> Result<()> {
        self.validate()?;
        validate_function_value(value)?;
        validate_value_shape(&self.shape, value, self.schema_id.as_str())
    }

    pub fn sha256(&self) -> String {
        sha256(&serde_json::to_vec(self).expect("function schema serializes"))
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FunctionCapability {
    /// Allows a transaction-function result object to be lowered into one
    /// typed append-only event in the same authoritative data transaction.
    EmitEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionLimits {
    pub max_input_bytes: u32,
    pub max_output_bytes: u32,
    pub memory_bytes: u64,
    pub stack_bytes: u32,
    /// QuickJS interrupt-callback budget. The runtime-build identity pins
    /// callback scheduling; exhaustion is an uncatchable terminal failure.
    pub max_interrupts: u32,
    /// Wasm instruction fuel. Wasmi traps deterministically at exhaustion.
    pub max_fuel: u64,
}

impl FunctionLimits {
    pub fn validate(&self) -> Result<()> {
        if self.max_input_bytes == 0
            || self.max_input_bytes as usize > MAX_FUNCTION_INPUT_BYTES
            || self.max_output_bytes == 0
            || self.max_output_bytes as usize > MAX_FUNCTION_OUTPUT_BYTES
        {
            return invalid("function input/output byte limits are outside the v1 bounds");
        }
        if !(MIN_FUNCTION_MEMORY_BYTES..=MAX_FUNCTION_MEMORY_BYTES).contains(&self.memory_bytes)
            || !(MIN_FUNCTION_STACK_BYTES..=MAX_FUNCTION_STACK_BYTES).contains(&self.stack_bytes)
            || u64::from(self.stack_bytes) >= self.memory_bytes
        {
            return invalid("function memory/stack limits are outside the v1 bounds");
        }
        if self.max_interrupts == 0
            || self.max_interrupts > MAX_FUNCTION_INTERRUPTS
            || self.max_fuel == 0
            || self.max_fuel > MAX_FUNCTION_FUEL
        {
            return invalid("function interrupt/fuel limits are outside the v1 bounds");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionDefinition {
    pub function_id: CanonicalId,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_sha256: Option<String>,
    pub runtime: FunctionRuntime,
    pub input_schema: FunctionValueSchema,
    pub input_schema_sha256: String,
    pub output_schema: FunctionValueSchema,
    pub output_schema_sha256: String,
    pub limits: FunctionLimits,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub capabilities: BTreeSet<FunctionCapability>,
}

impl FunctionDefinition {
    pub fn validate(&self) -> Result<()> {
        validate_revision_lineage(
            self.revision,
            self.predecessor_sha256.as_deref(),
            "function definition",
        )?;
        self.runtime.validate()?;
        self.input_schema.validate()?;
        self.output_schema.validate()?;
        validate_sha256(&self.input_schema_sha256, "function input_schema_sha256")?;
        validate_sha256(&self.output_schema_sha256, "function output_schema_sha256")?;
        if self.input_schema.sha256() != self.input_schema_sha256
            || self.output_schema.sha256() != self.output_schema_sha256
        {
            return invalid("function schema digest does not match its typed schema");
        }
        self.limits.validate()
    }

    pub fn sha256(&self) -> String {
        sha256(&serde_json::to_vec(self).expect("function definition serializes"))
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TransactionMutationKind {
    AssertClaim,
    PutSchema,
    PutRecord,
    PutRelation,
    AppendEvent,
    PutVector,
    AppendSeriesSample,
    PutGeo,
    PublishObjectReference,
    RetireData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "effect", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransactionFunctionEffect {
    RequireTrue,
    AppendEvent { kind: CanonicalId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransactionFunctionBinding {
    pub binding_id: CanonicalId,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_sha256: Option<String>,
    pub function_id: CanonicalId,
    pub function_definition_sha256: String,
    pub mutation: TransactionMutationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<CanonicalId>,
    pub effect: TransactionFunctionEffect,
    pub max_attempts: u8,
}

impl TransactionFunctionBinding {
    pub fn validate(&self, functions: &BTreeMap<CanonicalId, FunctionDefinition>) -> Result<()> {
        validate_revision_lineage(
            self.revision,
            self.predecessor_sha256.as_deref(),
            "transaction function binding",
        )?;
        validate_sha256(
            &self.function_definition_sha256,
            "transaction function definition",
        )?;
        if self.max_attempts != 1 {
            return invalid("v1 transaction function bindings require max_attempts=1");
        }
        if self.kind.is_some()
            && matches!(
                self.mutation,
                TransactionMutationKind::AssertClaim
                    | TransactionMutationKind::PutSchema
                    | TransactionMutationKind::RetireData
            )
        {
            return invalid(
                "transaction function binding kind cannot narrow a mutation without a kind coordinate",
            );
        }
        let function = functions.get(&self.function_id).ok_or_else(|| {
            crate::ContractError(
                "transaction function binding references an unknown function".into(),
            )
        })?;
        if function.sha256() != self.function_definition_sha256 {
            return invalid("transaction function binding does not pin its exact definition");
        }
        if matches!(self.effect, TransactionFunctionEffect::AppendEvent { .. })
            && !function
                .capabilities
                .contains(&FunctionCapability::EmitEvent)
        {
            return invalid(
                "append-event transaction function bindings require the emit_event capability",
            );
        }
        Ok(())
    }

    pub fn sha256(&self) -> String {
        sha256(&serde_json::to_vec(self).expect("transaction function binding serializes"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionCatalogue {
    pub contract_version: u16,
    pub revision: u64,
    pub artifacts: BTreeMap<String, FunctionArtifact>,
    pub functions: BTreeMap<CanonicalId, FunctionDefinition>,
    pub transaction_bindings: BTreeMap<CanonicalId, TransactionFunctionBinding>,
}

impl FunctionCatalogue {
    pub fn empty() -> Self {
        Self {
            contract_version: FUNCTION_CONTRACT_VERSION,
            revision: 0,
            artifacts: BTreeMap::new(),
            functions: BTreeMap::new(),
            transaction_bindings: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.contract_version != FUNCTION_CONTRACT_VERSION {
            return invalid("unsupported function contract version");
        }
        if self.revision == 0
            && (!self.artifacts.is_empty()
                || !self.functions.is_empty()
                || !self.transaction_bindings.is_empty())
        {
            return invalid("function catalogue revision zero must be empty");
        }
        if self.artifacts.len() > MAX_FUNCTIONS_PER_CATALOGUE
            || self.functions.len() > MAX_FUNCTIONS_PER_CATALOGUE
            || self.transaction_bindings.len() > MAX_TRANSACTION_FUNCTION_BINDINGS_PER_CATALOGUE
        {
            return invalid("function catalogue exceeds the v1 entry bound");
        }
        let mut artifact_bytes = 0_usize;
        for (content_sha256, artifact) in &self.artifacts {
            if content_sha256 != &artifact.content_sha256 {
                return invalid("function artifact map key does not match content_sha256");
            }
            artifact.validate()?;
            artifact_bytes = artifact_bytes
                .checked_add(artifact.byte_length as usize)
                .ok_or_else(|| crate::ContractError("function artifact bytes overflow".into()))?;
        }
        if artifact_bytes > MAX_FUNCTION_CATALOGUE_ARTIFACT_BYTES {
            return invalid("function catalogue artifact bytes exceed the physical v1 bound");
        }
        let mut referenced_artifacts = BTreeSet::new();
        for (function_id, function) in &self.functions {
            if function_id != &function.function_id {
                return invalid("function catalogue key does not match function_id");
            }
            function.validate()?;
            let artifact = self
                .artifacts
                .get(function.runtime.artifact_sha256())
                .ok_or_else(|| {
                    crate::ContractError(
                        "function definition references an unknown artifact".into(),
                    )
                })?;
            if artifact.media_type != function.runtime.expected_media_type() {
                return invalid("function artifact media type does not match its runtime");
            }
            referenced_artifacts.insert(function.runtime.artifact_sha256());
        }
        if referenced_artifacts.len() != self.artifacts.len() {
            return invalid("function catalogue contains an unreferenced artifact");
        }
        for (binding_id, binding) in &self.transaction_bindings {
            if binding_id != &binding.binding_id {
                return invalid("transaction function catalogue key does not match binding_id");
            }
            binding.validate(&self.functions)?;
        }
        let encoded_bytes = serde_json::to_vec(self)
            .map_err(|error| crate::ContractError(error.to_string()))?
            .len();
        if encoded_bytes > MAX_FUNCTION_CATALOGUE_ENCODED_BYTES {
            return invalid(format!(
                "function catalogue is {encoded_bytes} encoded bytes; maximum is {MAX_FUNCTION_CATALOGUE_ENCODED_BYTES}"
            ));
        }
        Ok(())
    }

    pub fn sha256(&self) -> String {
        sha256(&serde_json::to_vec(self).expect("function catalogue serializes"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaceFunctionCatalogue {
    pub expected_revision: u64,
    pub catalogue: FunctionCatalogue,
}

impl ReplaceFunctionCatalogue {
    pub fn validate(&self) -> Result<()> {
        self.catalogue.validate()?;
        let next_revision = self
            .expected_revision
            .checked_add(1)
            .ok_or_else(|| crate::ContractError("function catalogue revision overflow".into()))?;
        if self.catalogue.revision != next_revision {
            return invalid("replacement catalogue revision must follow expected_revision");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListFunctionCatalogue {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecuteFunction {
    pub function_id: CanonicalId,
    pub input: QueryValue,
}

impl ExecuteFunction {
    pub fn validate(&self) -> Result<()> {
        validate_function_value(&self.input)?;
        let bytes = serde_json::to_vec(&self.input)
            .map_err(|error| crate::ContractError(error.to_string()))?;
        if bytes.len() > MAX_FUNCTION_INPUT_BYTES {
            return invalid("function input exceeds the public v1 bound");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "proposal", rename_all = "snake_case", deny_unknown_fields)]
pub enum FunctionInvocationProposal {
    ReturnValue,
    RequireTrue,
    AppendEvent {
        kind: CanonicalId,
        properties_sha256: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionInvocationReceipt {
    pub contract_version: u16,
    pub invocation_id: CanonicalId,
    pub catalogue_revision: u64,
    pub catalogue_sha256: String,
    pub function_id: CanonicalId,
    pub function_definition_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<CanonicalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_index: Option<u32>,
    pub runtime: FunctionRuntimeKind,
    pub artifact_sha256: String,
    pub runtime_profile: CanonicalId,
    pub runtime_build_sha256: String,
    pub input_schema_sha256: String,
    pub output_schema_sha256: String,
    pub input_sha256: String,
    pub output_sha256: String,
    pub output: QueryValue,
    pub proposal: FunctionInvocationProposal,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_commit_sha256: Option<String>,
    pub limits: FunctionLimits,
    pub attempts: u8,
    pub interrupts_consumed: u32,
    pub fuel_consumed: u64,
    pub receipt_sha256: String,
}

impl FunctionInvocationReceipt {
    pub fn seal(mut self) -> Result<Self> {
        self.receipt_sha256.clear();
        self.validate_components()?;
        self.receipt_sha256 = sha256(&self.bytes_without_digest());
        Ok(self)
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_components()?;
        validate_sha256(&self.receipt_sha256, "function receipt_sha256")?;
        if sha256(&self.bytes_without_digest()) != self.receipt_sha256 {
            return invalid("function receipt digest does not match its fields");
        }
        Ok(())
    }

    fn validate_components(&self) -> Result<()> {
        if self.contract_version != FUNCTION_CONTRACT_VERSION
            || self.catalogue_revision == 0
            || self.attempts != 1
        {
            return invalid("function invocation coordinates are invalid");
        }
        for (field, value) in [
            ("catalogue_sha256", self.catalogue_sha256.as_str()),
            (
                "function_definition_sha256",
                self.function_definition_sha256.as_str(),
            ),
            ("artifact_sha256", self.artifact_sha256.as_str()),
            ("runtime_build_sha256", self.runtime_build_sha256.as_str()),
            ("input_schema_sha256", self.input_schema_sha256.as_str()),
            ("output_schema_sha256", self.output_schema_sha256.as_str()),
            ("input_sha256", self.input_sha256.as_str()),
            ("output_sha256", self.output_sha256.as_str()),
        ] {
            validate_sha256(value, &format!("function {field}"))?;
        }
        if let Some(commit_sha256) = &self.runtime_commit_sha256 {
            validate_sha256(commit_sha256, "function runtime_commit_sha256")?;
        }
        self.limits.validate()?;
        if self.interrupts_consumed > self.limits.max_interrupts
            || self.fuel_consumed > self.limits.max_fuel
        {
            return invalid("function receipt consumption exceeds its admitted budget");
        }
        validate_function_value(&self.output)?;
        let output = serde_json::to_vec(&self.output)
            .map_err(|error| crate::ContractError(error.to_string()))?;
        if output.len() > self.limits.max_output_bytes as usize
            || sha256(&output) != self.output_sha256
        {
            return invalid("function receipt output bytes or digest are invalid");
        }
        let transaction_coordinates = self.binding_id.is_some()
            && self.mutation_index.is_some()
            && self.runtime_commit_sha256.is_some();
        match &self.proposal {
            FunctionInvocationProposal::ReturnValue
                if self.binding_id.is_some()
                    || self.mutation_index.is_some()
                    || self.runtime_commit_sha256.is_some() =>
            {
                return invalid("standalone function receipt has transaction coordinates");
            }
            FunctionInvocationProposal::RequireTrue
                if !transaction_coordinates || self.output != QueryValue::Bool(true) =>
            {
                return invalid("require_true receipt is not an accepted transaction proposal");
            }
            FunctionInvocationProposal::AppendEvent {
                properties_sha256, ..
            } => {
                if !transaction_coordinates || !matches!(self.output, QueryValue::Map(_)) {
                    return invalid("append_event receipt is not a complete transaction proposal");
                }
                validate_sha256(properties_sha256, "function append_event properties_sha256")?;
                if properties_sha256 != &self.output_sha256 {
                    return invalid("append_event proposal digest does not match function output");
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn bytes_without_digest(&self) -> Vec<u8> {
        let mut unsigned = self.clone();
        unsigned.receipt_sha256.clear();
        serde_json::to_vec(&unsigned).expect("function receipt fields serialize")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionExecutionResult {
    pub receipt: FunctionInvocationReceipt,
}

impl FunctionExecutionResult {
    pub fn validate(&self) -> Result<()> {
        self.receipt.validate()?;
        if !matches!(
            self.receipt.proposal,
            FunctionInvocationProposal::ReturnValue
        ) {
            return invalid("standalone function execution returned a transaction proposal");
        }
        Ok(())
    }
}

pub fn validate_function_value(value: &QueryValue) -> Result<()> {
    let mut stack = vec![(value, 0_usize)];
    let mut items = 0_usize;
    while let Some((value, depth)) = stack.pop() {
        if depth > MAX_FUNCTION_VALUE_DEPTH {
            return invalid("function value exceeds the v1 nesting-depth bound");
        }
        items = items
            .checked_add(1)
            .ok_or_else(|| crate::ContractError("function value item count overflow".into()))?;
        if items > MAX_FUNCTION_VALUE_ITEMS {
            return invalid("function value exceeds the v1 item-count bound");
        }
        match value {
            QueryValue::List(values) => {
                stack.extend(values.iter().map(|value| (value, depth + 1)));
            }
            QueryValue::Map(values) => {
                stack.extend(values.values().map(|value| (value, depth + 1)));
            }
            QueryValue::Null
            | QueryValue::Bool(_)
            | QueryValue::Integer(_)
            | QueryValue::Unsigned(_)
            | QueryValue::Decimal(_)
            | QueryValue::String(_)
            | QueryValue::Digest(_) => {}
        }
    }
    Ok(())
}

fn validate_revision_lineage(revision: u64, predecessor: Option<&str>, kind: &str) -> Result<()> {
    match (revision, predecessor) {
        (0, _) => invalid(format!("{kind} revision must be non-zero")),
        (1, None) => Ok(()),
        (1, Some(_)) => invalid(format!("first {kind} revision cannot have a predecessor")),
        (_, None) => invalid(format!("later {kind} revision requires a predecessor")),
        (_, Some(predecessor)) => validate_sha256(predecessor, &format!("{kind} predecessor")),
    }
}

fn validate_schema_shape(
    shape: &FunctionValueShape,
    depth: usize,
    items: &mut usize,
) -> Result<()> {
    if depth > MAX_FUNCTION_SCHEMA_DEPTH {
        return invalid("function schema exceeds its nesting-depth bound");
    }
    *items = items
        .checked_add(1)
        .ok_or_else(|| crate::ContractError("function schema item count overflow".into()))?;
    if *items > MAX_FUNCTION_SCHEMA_ITEMS {
        return invalid("function schema exceeds its item-count bound");
    }
    match shape {
        FunctionValueShape::String { max_bytes } if *max_bytes == 0 => {
            invalid("function string schema max_bytes must be non-zero")
        }
        FunctionValueShape::List {
            items: item,
            max_items,
        } => {
            if *max_items == 0 || *max_items as usize > MAX_FUNCTION_VALUE_ITEMS {
                return invalid("function list schema max_items is outside the v1 bound");
            }
            validate_schema_shape(item, depth + 1, items)
        }
        FunctionValueShape::Object {
            properties,
            required,
            ..
        } => {
            if properties.len() > MAX_FUNCTION_SCHEMA_ITEMS
                || !required.iter().all(|name| properties.contains_key(name))
            {
                return invalid("function object schema properties or required set are invalid");
            }
            for (name, property) in properties {
                if name.is_empty() || name.len() > 256 || name.as_bytes().contains(&0) {
                    return invalid("function object schema property name is invalid");
                }
                validate_schema_shape(property, depth + 1, items)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_value_shape(
    shape: &FunctionValueShape,
    value: &QueryValue,
    schema: &str,
) -> Result<()> {
    let valid = match (shape, value) {
        (FunctionValueShape::CanonicalValueV1, _) => true,
        (FunctionValueShape::Null, QueryValue::Null) => true,
        (FunctionValueShape::Boolean, QueryValue::Bool(_)) => true,
        (FunctionValueShape::Integer, QueryValue::Integer(_)) => true,
        (FunctionValueShape::Unsigned, QueryValue::Unsigned(_)) => true,
        (FunctionValueShape::Decimal, QueryValue::Decimal(_)) => true,
        (FunctionValueShape::String { max_bytes }, QueryValue::String(value)) => {
            value.len() <= *max_bytes as usize
        }
        (FunctionValueShape::Digest, QueryValue::Digest(value)) => {
            validate_sha256(value, "function schema digest value").is_ok()
        }
        (FunctionValueShape::List { items, max_items }, QueryValue::List(values)) => {
            values.len() <= *max_items as usize
                && values
                    .iter()
                    .all(|value| validate_value_shape(items, value, schema).is_ok())
        }
        (
            FunctionValueShape::Object {
                properties,
                required,
                allow_additional_properties,
            },
            QueryValue::Map(values),
        ) => {
            required.iter().all(|name| values.contains_key(name))
                && (*allow_additional_properties
                    || values.keys().all(|name| properties.contains_key(name)))
                && values.iter().all(|(name, value)| {
                    properties
                        .get(name)
                        .is_none_or(|shape| validate_value_shape(shape, value, schema).is_ok())
                })
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        invalid(format!("function value does not satisfy schema {schema}"))
    }
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to string cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> FunctionLimits {
        FunctionLimits {
            max_input_bytes: 4_096,
            max_output_bytes: 4_096,
            memory_bytes: 1024 * 1024,
            stack_bytes: 64 * 1024,
            max_interrupts: 1_000,
            max_fuel: 100_000,
        }
    }

    fn schema(id: &str, shape: FunctionValueShape) -> FunctionValueSchema {
        FunctionValueSchema {
            schema_id: CanonicalId::new(id).unwrap(),
            revision: 1,
            shape,
        }
    }

    fn javascript(id: &str, source: &str) -> (FunctionArtifact, FunctionDefinition) {
        let content_sha256 = sha256(source.as_bytes());
        let artifact = FunctionArtifact {
            content_sha256: content_sha256.clone(),
            media_type: FunctionArtifactMediaType::JavaScriptUtf8,
            byte_length: source.len() as u32,
            content_base64: STANDARD.encode(source),
        };
        let input_schema = schema("canonical-input", FunctionValueShape::CanonicalValueV1);
        let output_schema = schema("canonical-output", FunctionValueShape::CanonicalValueV1);
        let definition = FunctionDefinition {
            function_id: CanonicalId::new(id).unwrap(),
            revision: 1,
            predecessor_sha256: None,
            runtime: FunctionRuntime::JavaScriptEs2020 {
                artifact_sha256: content_sha256,
                runtime_profile: CanonicalId::new("javascript-es2020-json-v1").unwrap(),
                runtime_build_sha256: "1".repeat(64),
            },
            input_schema_sha256: input_schema.sha256(),
            input_schema,
            output_schema_sha256: output_schema.sha256(),
            output_schema,
            limits: limits(),
            capabilities: BTreeSet::new(),
        };
        (artifact, definition)
    }

    fn maximum_webassembly(id: &str, marker: u32) -> (FunctionArtifact, FunctionDefinition) {
        let mut content = vec![0_u8; MAX_FUNCTION_WASM_BYTES];
        content[..8].copy_from_slice(b"\0asm\x01\0\0\0");
        content[8..12].copy_from_slice(&marker.to_be_bytes());
        let content_sha256 = sha256(&content);
        let artifact = FunctionArtifact {
            content_sha256: content_sha256.clone(),
            media_type: FunctionArtifactMediaType::WebAssemblyBinary,
            byte_length: content.len().try_into().unwrap(),
            content_base64: STANDARD.encode(content),
        };
        let input_schema = schema(&format!("{id}-input"), FunctionValueShape::CanonicalValueV1);
        let output_schema = schema(
            &format!("{id}-output"),
            FunctionValueShape::CanonicalValueV1,
        );
        let definition = FunctionDefinition {
            function_id: CanonicalId::new(id).unwrap(),
            revision: 1,
            predecessor_sha256: None,
            runtime: FunctionRuntime::WebAssemblyV1 {
                artifact_sha256: content_sha256,
                runtime_profile: CanonicalId::new("webassembly-json-v1").unwrap(),
                runtime_build_sha256: "1".repeat(64),
                abi: WebAssemblyAbi::JsonV1,
            },
            input_schema_sha256: input_schema.sha256(),
            input_schema,
            output_schema_sha256: output_schema.sha256(),
            output_schema,
            limits: limits(),
            capabilities: BTreeSet::new(),
        };
        (artifact, definition)
    }

    #[test]
    fn catalogue_rejects_digest_identity_capability_and_retry_substitution() {
        let (artifact, function) = javascript("validator", "(input) => input.allowed === true");
        let function_id = function.function_id.clone();
        let binding_id = CanonicalId::new("validate-record").unwrap();
        let mut catalogue = FunctionCatalogue {
            contract_version: FUNCTION_CONTRACT_VERSION,
            revision: 1,
            artifacts: BTreeMap::from([(artifact.content_sha256.clone(), artifact)]),
            functions: BTreeMap::from([(function_id.clone(), function.clone())]),
            transaction_bindings: BTreeMap::from([(
                binding_id.clone(),
                TransactionFunctionBinding {
                    binding_id,
                    revision: 1,
                    predecessor_sha256: None,
                    function_id,
                    function_definition_sha256: function.sha256(),
                    mutation: TransactionMutationKind::PutRecord,
                    kind: Some(CanonicalId::new("person").unwrap()),
                    effect: TransactionFunctionEffect::RequireTrue,
                    max_attempts: 1,
                },
            )]),
        };
        catalogue.validate().unwrap();
        catalogue
            .artifacts
            .values_mut()
            .next()
            .unwrap()
            .content_sha256 = "0".repeat(64);
        assert!(catalogue.validate().is_err());

        let (artifact, mut event_function) = javascript("event-writer", "() => ({ ok: true })");
        let event_function_id = event_function.function_id.clone();
        let event_binding_id = CanonicalId::new("write-event").unwrap();
        let mut event_binding = TransactionFunctionBinding {
            binding_id: event_binding_id.clone(),
            revision: 1,
            predecessor_sha256: None,
            function_id: event_function_id.clone(),
            function_definition_sha256: event_function.sha256(),
            mutation: TransactionMutationKind::AppendEvent,
            kind: None,
            effect: TransactionFunctionEffect::AppendEvent {
                kind: CanonicalId::new("derived-event").unwrap(),
            },
            max_attempts: 1,
        };
        let mut event_catalogue = FunctionCatalogue {
            contract_version: FUNCTION_CONTRACT_VERSION,
            revision: 1,
            artifacts: BTreeMap::from([(artifact.content_sha256.clone(), artifact)]),
            functions: BTreeMap::from([(event_function_id.clone(), event_function.clone())]),
            transaction_bindings: BTreeMap::from([(
                event_binding_id.clone(),
                event_binding.clone(),
            )]),
        };
        assert!(event_catalogue.validate().is_err());
        event_function
            .capabilities
            .insert(FunctionCapability::EmitEvent);
        event_binding.function_definition_sha256 = event_function.sha256();
        event_catalogue
            .functions
            .insert(event_function_id, event_function);
        event_catalogue
            .transaction_bindings
            .insert(event_binding_id.clone(), event_binding);
        event_catalogue.validate().unwrap();
        event_catalogue
            .transaction_bindings
            .get_mut(&event_binding_id)
            .unwrap()
            .max_attempts = 2;
        assert!(event_catalogue.validate().is_err());

        assert!(ReplaceFunctionCatalogue {
            expected_revision: u64::MAX,
            catalogue: FunctionCatalogue {
                contract_version: FUNCTION_CONTRACT_VERSION,
                revision: u64::MAX,
                artifacts: BTreeMap::new(),
                functions: BTreeMap::new(),
                transaction_bindings: BTreeMap::new(),
            },
        }
        .validate()
        .is_err());
    }

    #[test]
    fn catalogue_encoded_limit_admits_the_largest_max_artifact_prefix_and_rejects_the_next() {
        let mut candidate = FunctionCatalogue {
            contract_version: FUNCTION_CONTRACT_VERSION,
            revision: 1,
            artifacts: BTreeMap::new(),
            functions: BTreeMap::new(),
            transaction_bindings: BTreeMap::new(),
        };
        let mut accepted = None;
        let rejection = loop {
            let index = candidate.functions.len();
            let id = format!("maximum-wasm-{index:02}");
            let (artifact, definition) = maximum_webassembly(&id, index as u32);
            candidate
                .artifacts
                .insert(artifact.content_sha256.clone(), artifact);
            candidate
                .functions
                .insert(definition.function_id.clone(), definition);
            match candidate.validate() {
                Ok(()) => accepted = Some(candidate.clone()),
                Err(error) => break error,
            }
        };
        let accepted = accepted.expect("at least one maximum-size artifact is admitted");
        let encoded = serde_json::to_vec(&accepted).unwrap();
        assert!(encoded.len() <= MAX_FUNCTION_CATALOGUE_ENCODED_BYTES);
        assert!(
            encoded.len() > MAX_FUNCTION_CATALOGUE_ENCODED_BYTES - 2 * MAX_FUNCTION_WASM_BYTES,
            "the positive fixture must exercise the upper catalogue range"
        );
        assert!(
            accepted
                .artifacts
                .values()
                .map(|artifact| artifact.byte_length as usize)
                .sum::<usize>()
                <= MAX_FUNCTION_CATALOGUE_ARTIFACT_BYTES
        );
        assert!(rejection.to_string().contains("encoded bytes"));
    }

    #[test]
    fn artifact_digest_covers_decoded_bytes_and_schema_is_enforced() {
        let module = b"\0asm\x01\0\0\0";
        let mut artifact = FunctionArtifact {
            content_sha256: sha256(module),
            media_type: FunctionArtifactMediaType::WebAssemblyBinary,
            byte_length: module.len() as u32,
            content_base64: STANDARD.encode(module),
        };
        artifact.validate().unwrap();
        artifact.content_base64.push('A');
        assert!(artifact.validate().is_err());

        let boolean = schema("boolean-result", FunctionValueShape::Boolean);
        boolean.validate_value(&QueryValue::Bool(true)).unwrap();
        assert!(boolean
            .validate_value(&QueryValue::String("true".into()))
            .is_err());
    }

    #[test]
    fn invocation_receipt_binds_proposal_coordinates_budgets_and_output() {
        let output = QueryValue::Bool(true);
        let output_sha256 = sha256(&serde_json::to_vec(&output).unwrap());
        let receipt = FunctionInvocationReceipt {
            contract_version: FUNCTION_CONTRACT_VERSION,
            invocation_id: CanonicalId::new("function-receipt-one").unwrap(),
            catalogue_revision: 1,
            catalogue_sha256: "1".repeat(64),
            function_id: CanonicalId::new("validator").unwrap(),
            function_definition_sha256: "2".repeat(64),
            binding_id: None,
            mutation_index: None,
            runtime: FunctionRuntimeKind::JavaScriptEs2020,
            artifact_sha256: "3".repeat(64),
            runtime_profile: CanonicalId::new("javascript-es2020-json-v1").unwrap(),
            runtime_build_sha256: "4".repeat(64),
            input_schema_sha256: "5".repeat(64),
            output_schema_sha256: "6".repeat(64),
            input_sha256: "7".repeat(64),
            output_sha256,
            output,
            proposal: FunctionInvocationProposal::ReturnValue,
            runtime_commit_sha256: None,
            limits: limits(),
            attempts: 1,
            interrupts_consumed: 1,
            fuel_consumed: 0,
            receipt_sha256: String::new(),
        }
        .seal()
        .unwrap();
        receipt.validate().unwrap();

        let mut tampered = receipt.clone();
        tampered.runtime_build_sha256 = "8".repeat(64);
        assert!(tampered.validate().is_err());

        let mut unbound_proposal = receipt;
        unbound_proposal.receipt_sha256.clear();
        unbound_proposal.proposal = FunctionInvocationProposal::RequireTrue;
        assert!(unbound_proposal.seal().is_err());
    }

    #[test]
    fn function_values_fail_closed_on_depth_and_item_amplification() {
        let mut nested = QueryValue::Null;
        for _ in 0..=MAX_FUNCTION_VALUE_DEPTH {
            nested = QueryValue::List(vec![nested]);
        }
        assert!(validate_function_value(&nested).is_err());
        assert!(validate_function_value(&QueryValue::List(vec![
            QueryValue::Null;
            MAX_FUNCTION_VALUE_ITEMS
        ]))
        .is_err());
    }
}
