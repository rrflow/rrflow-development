//! Validated identifiers.
//!
//! An identifier that can reach a public projection holds the canonical grammar
//! in its type, so a value that projection would refuse cannot be constructed.
//! The rest are non-empty and exclude the NUL control byte, because they hold a
//! deliberately namespaced form such as `instance:{id}`.

use crate::error::{Error, Result};
const FORBIDDEN_NUL: u8 = 0x00;

/// The longest canonical identifier, in bytes.
pub const MAX_CANONICAL_ID_BYTES: usize = 128;

/// Why `value` is not a canonical identifier, or `None` when it is.
///
/// This is the one grammar for an identifier that can reach a public projection,
/// a SPIFFE identity or a certificate. `rrd-contract` has no workspace
/// dependencies, so its `CanonicalId` keeps its own copy, and a test there proves
/// the two agree.
pub fn canonical_id_problem(value: &str) -> Option<&'static str> {
    if value.is_empty() || value.len() > MAX_CANONICAL_ID_BYTES {
        return Some("canonical id length must be in 1..=128 bytes");
    }
    let canonical = value.bytes().enumerate().all(|(index, byte)| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || (index > 0 && matches!(byte, b'-' | b'_' | b'.'))
    });
    (!canonical).then_some(
        "canonical id must start with lowercase ASCII or a digit and contain only lowercase ASCII, digits, '-', '_', or '.'",
    )
}

/// Refuses what no identifier can hold.
pub(crate) fn check_text(kind: &'static str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::EmptyIdentifier { kind });
    }
    if value.as_bytes().contains(&FORBIDDEN_NUL) {
        return Err(Error::SeparatorInIdentifier {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}

/// Holds an identifier to the one canonical grammar.
pub(crate) fn check_canonical(kind: &'static str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::EmptyIdentifier { kind });
    }
    match canonical_id_problem(value) {
        Some(problem) => Err(Error::NonCanonicalIdentifier {
            kind,
            value: value.to_owned(),
            problem,
        }),
        None => Ok(()),
    }
}

/// An identifier type whose values have passed `$check`, by every path that makes one.
macro_rules! text_ident {
    ($name:ident, $label:literal, $check:path) => {
        $crate::ident::text_ident!(@type $name, $label);

        impl $name {
            pub fn new(value: impl Into<String>) -> $crate::error::Result<Self> {
                let value = value.into();
                $check($label, &value)?;
                Ok(Self(value))
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::error::Error;

            fn try_from(value: String) -> $crate::error::Result<Self> {
                Self::new(value)
            }
        }
    };
    (@type $name:ident, $label:literal) => {
        #[derive(
            Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
        )]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({:?})", $label, self.0)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}
pub(crate) use text_ident;

text_ident!(Subject, "subject", check_canonical);
text_ident!(Predicate, "predicate", check_canonical);
// A principal in the access log, such as `agent:clyffy/worker`. It never reaches a public record.
text_ident!(Reader, "reader", check_text);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert!(Subject::new("").is_err());
        assert!(Predicate::new("").is_err());
    }

    #[test]
    fn rejects_separator_byte() {
        assert!(Subject::new("a\u{0}b").is_err());
        assert!(Predicate::new("a\u{0}b").is_err());
    }

    #[test]
    fn accepts_ordinary_values() {
        assert_eq!(Subject::new("wp3").unwrap().as_str(), "wp3");
        assert_eq!(Predicate::new("status").unwrap().as_str(), "status");
    }

    #[test]
    fn deserialize_enforces_invariant() {
        // The invariant must hold on the deserialization path too, not just `new`.
        let bad = serde_json::from_str::<Subject>("\"a\\u0000b\"");
        assert!(bad.is_err());
    }
}
