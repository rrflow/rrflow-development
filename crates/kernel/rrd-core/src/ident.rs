//! Validated identifiers.
//!
//! Identifiers are non-empty and exclude the NUL control byte. This is a
//! semantic text invariant, independent of any physical storage encoding.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

const FORBIDDEN_NUL: u8 = 0x00;

macro_rules! ident_type {
    ($name:ident, $label:literal) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                if value.is_empty() {
                    return Err(Error::EmptyIdentifier { kind: $label });
                }
                if value.as_bytes().contains(&FORBIDDEN_NUL) {
                    return Err(Error::SeparatorInIdentifier {
                        kind: $label,
                        value,
                    });
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;
            fn try_from(value: String) -> Result<Self> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({:?})", $label, self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

ident_type!(Subject, "subject");
ident_type!(Predicate, "predicate");
ident_type!(Reader, "reader");

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
