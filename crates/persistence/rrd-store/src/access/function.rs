//! Typed governed-function catalogue and invocation-receipt records.
//!
//! Executable content is stored once as a binary value addressed by SHA-256.
//! Definitions and transaction bindings are immutable, individually addressed
//! records. A catalogue revision contains only their identities and digests;
//! one compare-and-swap head selects the active membership. The public
//! function contract is decoded and validated by `RrdEngine`, while this
//! module owns the physical record closure shared by rrflowMX and rrflowKV.

use super::runtime_state::{checked_key, get, AccessRead};
use super::SemanticCommitPlan;
use crate::keyspaces;
use crate::{Error, Result, StorageTransaction};
use rrd_core::digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const FUNCTION_CATALOGUE_STATE_FORMAT_VERSION: u16 = 1;
pub const FUNCTION_INVOCATION_RECEIPT_FORMAT_VERSION: u16 = 1;
const FUNCTION_ARTIFACT_MAGIC: &[u8; 8] = b"RRFNART1";
const MAX_FUNCTION_ARTIFACT_BYTES: usize = 256 * 1024;
const MAX_FUNCTION_RECORD_JSON_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionArtifactMediaTypeRecord {
    JavaScriptUtf8,
    WebAssemblyBinary,
}

impl FunctionArtifactMediaTypeRecord {
    fn tag(self) -> u8 {
        match self {
            Self::JavaScriptUtf8 => 1,
            Self::WebAssemblyBinary => 2,
        }
    }

    fn from_tag(tag: u8) -> Result<Self> {
        match tag {
            1 => Ok(Self::JavaScriptUtf8),
            2 => Ok(Self::WebAssemblyBinary),
            _ => Err(function_error("function artifact media tag is unknown")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionArtifactRecord {
    pub content_sha256: String,
    pub media_type: FunctionArtifactMediaTypeRecord,
    pub content: Vec<u8>,
}

impl FunctionArtifactRecord {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.content_sha256, "function artifact")?;
        if self.content.is_empty() || self.content.len() > MAX_FUNCTION_ARTIFACT_BYTES {
            return Err(function_error(
                "function artifact byte length is outside the physical bound",
            ));
        }
        if digest::sha256_hex(&self.content) != self.content_sha256 {
            return Err(function_error(
                "function artifact content does not match its address",
            ));
        }
        Ok(())
    }

    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let length: u32 = self
            .content
            .len()
            .try_into()
            .map_err(|_| function_error("function artifact length exceeds u32"))?;
        let mut encoded = Vec::with_capacity(15 + self.content.len());
        encoded.extend_from_slice(FUNCTION_ARTIFACT_MAGIC);
        encoded.extend_from_slice(&FUNCTION_CATALOGUE_STATE_FORMAT_VERSION.to_be_bytes());
        encoded.push(self.media_type.tag());
        encoded.extend_from_slice(&length.to_be_bytes());
        encoded.extend_from_slice(&self.content);
        Ok(encoded)
    }

    pub(crate) fn decode(expected_sha256: &str, encoded: &[u8]) -> Result<Self> {
        if encoded.len() < 15 || &encoded[..8] != FUNCTION_ARTIFACT_MAGIC {
            return Err(function_error("function artifact record header is invalid"));
        }
        let version = u16::from_be_bytes([encoded[8], encoded[9]]);
        if version != FUNCTION_CATALOGUE_STATE_FORMAT_VERSION {
            return Err(function_error(
                "function artifact record version is unsupported",
            ));
        }
        let media_type = FunctionArtifactMediaTypeRecord::from_tag(encoded[10])?;
        let length = u32::from_be_bytes(encoded[11..15].try_into().expect("four-byte slice"));
        let content = encoded[15..].to_vec();
        if content.len() != length as usize {
            return Err(function_error("function artifact record length is invalid"));
        }
        let record = Self {
            content_sha256: expected_sha256.into(),
            media_type,
            content,
        };
        record.validate()?;
        Ok(record)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionDefinitionRecord {
    pub format_version: u16,
    pub function_id: String,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_sha256: Option<String>,
    pub artifact_sha256: String,
    pub runtime_profile: String,
    pub runtime_build_sha256: String,
    pub input_schema_sha256: String,
    pub output_schema_sha256: String,
    pub definition_sha256: String,
    pub canonical_definition_json: String,
}

impl FunctionDefinitionRecord {
    pub fn validate(&self) -> Result<()> {
        validate_record_version(self.format_version, "function definition")?;
        validate_coordinate(&self.function_id, "function identity")?;
        validate_revision(
            self.revision,
            self.predecessor_sha256.as_deref(),
            "definition",
        )?;
        validate_coordinate(&self.runtime_profile, "function runtime profile")?;
        for (name, value) in [
            ("function artifact", self.artifact_sha256.as_str()),
            ("function runtime build", self.runtime_build_sha256.as_str()),
            ("function input schema", self.input_schema_sha256.as_str()),
            ("function output schema", self.output_schema_sha256.as_str()),
            ("function definition", self.definition_sha256.as_str()),
        ] {
            validate_sha256(value, name)?;
        }
        validate_canonical_json(
            &self.canonical_definition_json,
            &self.definition_sha256,
            "function definition",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransactionFunctionBindingRecord {
    pub format_version: u16,
    pub binding_id: String,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_sha256: Option<String>,
    pub function_id: String,
    pub function_definition_sha256: String,
    pub binding_sha256: String,
    pub canonical_binding_json: String,
}

impl TransactionFunctionBindingRecord {
    pub fn validate(&self) -> Result<()> {
        validate_record_version(self.format_version, "transaction function binding")?;
        validate_coordinate(&self.binding_id, "transaction function binding identity")?;
        validate_coordinate(&self.function_id, "function identity")?;
        validate_revision(self.revision, self.predecessor_sha256.as_deref(), "binding")?;
        validate_sha256(
            &self.function_definition_sha256,
            "transaction function definition",
        )?;
        validate_sha256(&self.binding_sha256, "transaction function binding")?;
        validate_canonical_json(
            &self.canonical_binding_json,
            &self.binding_sha256,
            "transaction function binding",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionCatalogueMembershipRecord {
    pub format_version: u16,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_sha256: Option<String>,
    pub catalogue_sha256: String,
    pub artifact_sha256: BTreeSet<String>,
    pub function_definition_sha256: BTreeMap<String, String>,
    pub transaction_binding_sha256: BTreeMap<String, String>,
    pub published_at_unix_ms: u64,
    pub published_by: String,
    pub request_id: String,
    pub operation_id: String,
    pub membership_sha256: String,
}

impl FunctionCatalogueMembershipRecord {
    pub fn seal(mut self) -> Result<Self> {
        self.membership_sha256.clear();
        self.validate_components()?;
        self.membership_sha256 = digest::sha256_hex(&self.bytes_without_digest());
        Ok(self)
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_components()?;
        validate_sha256(&self.membership_sha256, "function catalogue membership")?;
        if digest::sha256_hex(&self.bytes_without_digest()) != self.membership_sha256 {
            return Err(function_error(
                "function catalogue membership digest does not match its fields",
            ));
        }
        Ok(())
    }

    fn validate_components(&self) -> Result<()> {
        validate_record_version(self.format_version, "function catalogue membership")?;
        validate_revision(
            self.revision,
            self.predecessor_sha256.as_deref(),
            "catalogue membership",
        )?;
        validate_sha256(&self.catalogue_sha256, "function catalogue")?;
        if self.published_at_unix_ms == 0 {
            return Err(function_error(
                "function catalogue publication time must be non-zero",
            ));
        }
        for (name, value) in [
            ("function catalogue publisher", self.published_by.as_str()),
            ("function catalogue request", self.request_id.as_str()),
            ("function catalogue operation", self.operation_id.as_str()),
        ] {
            validate_coordinate(value, name)?;
        }
        for artifact in &self.artifact_sha256 {
            validate_sha256(artifact, "function catalogue artifact membership")?;
        }
        for (identity, definition) in &self.function_definition_sha256 {
            validate_coordinate(identity, "function membership identity")?;
            validate_sha256(definition, "function membership definition")?;
        }
        for (identity, binding) in &self.transaction_binding_sha256 {
            validate_coordinate(identity, "function binding membership identity")?;
            validate_sha256(binding, "function binding membership")?;
        }
        Ok(())
    }

    fn bytes_without_digest(&self) -> Vec<u8> {
        let mut unsigned = self.clone();
        unsigned.membership_sha256.clear();
        serde_json::to_vec(&unsigned).expect("function catalogue membership serializes")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionCatalogueHeadRecord {
    pub format_version: u16,
    pub revision: u64,
    pub catalogue_sha256: String,
    pub membership_sha256: String,
}

impl FunctionCatalogueHeadRecord {
    pub fn validate(&self) -> Result<()> {
        validate_record_version(self.format_version, "function catalogue head")?;
        if self.revision == 0 {
            return Err(function_error(
                "function catalogue head revision must be non-zero",
            ));
        }
        validate_sha256(&self.catalogue_sha256, "function catalogue head catalogue")?;
        validate_sha256(
            &self.membership_sha256,
            "function catalogue head membership",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCataloguePublication {
    pub expected_revision: u64,
    pub membership: FunctionCatalogueMembershipRecord,
    pub artifacts: Vec<FunctionArtifactRecord>,
    pub definitions: Vec<FunctionDefinitionRecord>,
    pub bindings: Vec<TransactionFunctionBindingRecord>,
}

impl FunctionCataloguePublication {
    pub fn validate(&self) -> Result<()> {
        self.membership.validate()?;
        if self.membership.revision
            != self
                .expected_revision
                .checked_add(1)
                .ok_or(Error::SequenceOverflow)?
        {
            return Err(function_error(
                "function catalogue membership does not follow expected revision",
            ));
        }
        let mut artifacts = BTreeSet::new();
        for artifact in &self.artifacts {
            artifact.validate()?;
            if !artifacts.insert(artifact.content_sha256.clone()) {
                return Err(function_error("function publication repeats an artifact"));
            }
        }
        if artifacts != self.membership.artifact_sha256 {
            return Err(function_error(
                "function publication artifacts do not match membership",
            ));
        }
        let mut definitions = BTreeMap::new();
        for definition in &self.definitions {
            definition.validate()?;
            if definitions
                .insert(
                    definition.function_id.clone(),
                    definition.definition_sha256.clone(),
                )
                .is_some()
            {
                return Err(function_error("function publication repeats a definition"));
            }
            if !artifacts.contains(&definition.artifact_sha256) {
                return Err(function_error(
                    "function definition references an absent publication artifact",
                ));
            }
        }
        if definitions != self.membership.function_definition_sha256 {
            return Err(function_error(
                "function publication definitions do not match membership",
            ));
        }
        let mut bindings = BTreeMap::new();
        for binding in &self.bindings {
            binding.validate()?;
            if bindings
                .insert(binding.binding_id.clone(), binding.binding_sha256.clone())
                .is_some()
            {
                return Err(function_error("function publication repeats a binding"));
            }
            if definitions.get(&binding.function_id) != Some(&binding.function_definition_sha256) {
                return Err(function_error(
                    "function binding does not reference an exact member definition",
                ));
            }
        }
        if bindings != self.membership.transaction_binding_sha256 {
            return Err(function_error(
                "function publication bindings do not match membership",
            ));
        }
        Ok(())
    }

    pub fn head(&self) -> FunctionCatalogueHeadRecord {
        FunctionCatalogueHeadRecord {
            format_version: FUNCTION_CATALOGUE_STATE_FORMAT_VERSION,
            revision: self.membership.revision,
            catalogue_sha256: self.membership.catalogue_sha256.clone(),
            membership_sha256: self.membership.membership_sha256.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCatalogueSnapshot {
    pub head: FunctionCatalogueHeadRecord,
    pub membership: FunctionCatalogueMembershipRecord,
    pub artifacts: Vec<FunctionArtifactRecord>,
    pub definitions: Vec<FunctionDefinitionRecord>,
    pub bindings: Vec<TransactionFunctionBindingRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionInvocationReceiptRecord {
    pub format_version: u16,
    pub invocation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_commit_sha256: Option<String>,
    pub receipt_sha256: String,
    pub canonical_receipt_sha256: String,
    pub canonical_receipt_json: String,
}

impl FunctionInvocationReceiptRecord {
    pub fn validate(&self) -> Result<()> {
        if self.format_version != FUNCTION_INVOCATION_RECEIPT_FORMAT_VERSION {
            return Err(function_error(
                "function invocation receipt version is unsupported",
            ));
        }
        validate_coordinate(&self.invocation_id, "function invocation identity")?;
        if let Some(commit) = &self.runtime_commit_sha256 {
            validate_sha256(commit, "function invocation runtime commit")?;
        }
        validate_sha256(&self.receipt_sha256, "function invocation receipt")?;
        validate_sha256(
            &self.canonical_receipt_sha256,
            "canonical function invocation receipt",
        )?;
        validate_canonical_json(
            &self.canonical_receipt_json,
            &self.canonical_receipt_sha256,
            "function invocation receipt",
        )
    }
}

pub(crate) fn encode_function_invocation_receipts(
    reader: &(impl AccessRead + ?Sized),
    plan: &mut SemanticCommitPlan,
    instance: &str,
    runtime_commit_sha256: &str,
    receipts: &[FunctionInvocationReceiptRecord],
) -> Result<()> {
    validate_coordinate(instance, "function receipt instance")?;
    validate_sha256(runtime_commit_sha256, "function receipt runtime commit")?;
    let mut identities = BTreeSet::new();
    for receipt in receipts {
        receipt.validate()?;
        if receipt.runtime_commit_sha256.as_deref() != Some(runtime_commit_sha256) {
            return Err(function_error(
                "function receipt is not bound to the semantic commit",
            ));
        }
        if !identities.insert(receipt.invocation_id.as_str()) {
            return Err(function_error(
                "semantic commit repeats a function invocation receipt",
            ));
        }
        let key = keyspaces::function_invocation_receipt_key(instance, &receipt.invocation_id);
        let encoded = serde_json::to_vec(receipt)?;
        if let Some(existing) = get(reader, keyspaces::INVOCATIONS, &key)? {
            if existing != encoded {
                return Err(function_error(
                    "function invocation identity is already bound to another receipt",
                ));
            }
        } else {
            plan.put_bytes(keyspaces::INVOCATIONS, &key, encoded)?;
        }
    }
    Ok(())
}

pub(crate) fn put_standalone_function_receipt(
    transaction: &mut dyn StorageTransaction,
    instance: &str,
    receipt: &FunctionInvocationReceiptRecord,
) -> Result<bool> {
    validate_coordinate(instance, "function receipt instance")?;
    receipt.validate()?;
    if receipt.runtime_commit_sha256.is_some() {
        return Err(function_error(
            "standalone function receipt contains a runtime commit",
        ));
    }
    let key = keyspaces::function_invocation_receipt_key(instance, &receipt.invocation_id);
    let key = checked_key(keyspaces::INVOCATIONS, &key)?;
    let encoded = serde_json::to_vec(receipt)?;
    match transaction.get(&key)? {
        Some(existing) if existing == encoded => Ok(false),
        Some(_) => Err(function_error(
            "function invocation identity is already bound to another receipt",
        )),
        None => {
            transaction.put(key, encoded)?;
            Ok(true)
        }
    }
}

pub(crate) fn validate_coordinate(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 256
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(function_error(format!("{name} is invalid")));
    }
    Ok(())
}

pub(crate) fn validate_sha256(value: &str, name: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(function_error(format!("{name} is not lowercase SHA-256")));
    }
    Ok(())
}

fn validate_record_version(version: u16, name: &str) -> Result<()> {
    if version != FUNCTION_CATALOGUE_STATE_FORMAT_VERSION {
        Err(function_error(format!("{name} version is unsupported")))
    } else {
        Ok(())
    }
}

fn validate_revision(revision: u64, predecessor: Option<&str>, name: &str) -> Result<()> {
    match (revision, predecessor) {
        (0, _) => Err(function_error(format!("function {name} revision is zero"))),
        (1, None) => Ok(()),
        (1, Some(_)) => Err(function_error(format!(
            "first function {name} revision has a predecessor"
        ))),
        (_, None) => Err(function_error(format!(
            "later function {name} revision lacks a predecessor"
        ))),
        (_, Some(predecessor)) => validate_sha256(predecessor, "function predecessor"),
    }
}

fn validate_canonical_json(json: &str, expected_sha256: &str, name: &str) -> Result<()> {
    if json.is_empty() || json.len() > MAX_FUNCTION_RECORD_JSON_BYTES {
        return Err(function_error(format!(
            "{name} JSON length is outside its physical bound"
        )));
    }
    serde_json::from_str::<serde_json::Value>(json)?;
    if digest::sha256_hex(json.as_bytes()) != expected_sha256 {
        return Err(function_error(format!(
            "{name} JSON does not match its content digest"
        )));
    }
    Ok(())
}

fn function_error(message: impl Into<String>) -> Error {
    Error::FunctionConstraint(message.into())
}
