//! The vocabulary every RRFlow harness shares: bounded identifiers and
//! digests, declared availability, canonical JSON, and the test-agnostic
//! report. A vector comparison, a reasoning-engine comparison and a probe of
//! another system all write the same report, so a client renders any of them
//! without knowing which kind of test produced it.
//!
//! Results are diagnostic evidence. They configure nothing, and they are not
//! acceptance, parity or superiority claims.
pub mod report;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

pub type Result<T> = std::result::Result<T, String>;

pub fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// JSON with sorted object keys, so a digest does not depend on field order.
pub fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut canonical = serde_json::to_value(value).map_err(|error| error.to_string())?;
    canonical.sort_all_objects();
    let mut bytes = serde_json::to_vec_pretty(&canonical).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(!try_from, !into)]
pub struct Id(
    #[schemars(length(min = 1, max = 96), regex(pattern = "^[a-z0-9][a-z0-9_.-]*$"))] String,
);

impl TryFrom<String> for Id {
    type Error = String;
    fn try_from(value: String) -> Result<Self> {
        require(
            !value.is_empty()
                && value.len() <= 96
                && value.bytes().enumerate().all(|(i, b)| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || i > 0 && b"_.-".contains(&b)
                }),
            "invalid bounded identifier",
        )?;
        Ok(Self(value))
    }
}
impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}
impl Id {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(!try_from, !into)]
pub struct Digest(#[schemars(regex(pattern = "^[0-9a-f]{64}$"))] String);
impl TryFrom<String> for Digest {
    type Error = String;
    fn try_from(value: String) -> Result<Self> {
        require(
            value.len() == 64
                && value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "digest must be lowercase SHA-256",
        )?;
        Ok(Self(value))
    }
}
impl From<Digest> for String {
    fn from(value: Digest) -> Self {
        value.0
    }
}
impl Digest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LabMetric {
    Cosine,
    Dot,
    Euclidean,
}

/// A declared gap is reported as such. It can never read as a passing result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Availability {
    Available,
    Unavailable { reason: String },
    Unimplemented { reason: String },
    NotApplicable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_and_digests_accept_only_their_bounded_forms() {
        for valid in ["a", "vector-20k", "q1.f_2"] {
            assert_eq!(Id::try_from(valid.to_owned()).unwrap().as_str(), valid);
        }
        for invalid in ["", "-a", "A", "a b", &"a".repeat(97)] {
            assert!(Id::try_from(invalid.to_owned()).is_err(), "{invalid:?}");
        }
        assert!(Digest::try_from("ab".repeat(32)).is_ok());
        for invalid in ["AB".repeat(32), "ab".repeat(31), "zz".repeat(32)] {
            assert!(Digest::try_from(invalid.clone()).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn canonical_json_digest_does_not_depend_on_field_order() {
        let left = serde_json::json!({"b": 1, "a": {"d": 2, "c": 3}});
        let right = serde_json::json!({"a": {"c": 3, "d": 2}, "b": 1});
        assert_eq!(
            canonical_json(&left).unwrap(),
            canonical_json(&right).unwrap()
        );
        assert_eq!(sha256_hex(b"").len(), 64);
    }
}
