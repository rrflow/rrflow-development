//! Unambiguous, shared identities for derived result rows and context candidates.
//!
//! These are projections of typed engine identities, not storage addresses.
//! `%` and `:` are escaped in each component before joining. UTF-8 and all
//! other characters are preserved. The version is bound to persisted derived
//! artifacts; a change requires those artifacts to be rebuilt.

use crate::RuntimeRef;

/// Version of the derived identity encoding, not the storage key format.
pub const RESULT_IDENTITY_VERSION: u16 = 2;

/// The fixed engine-owned families distinguish otherwise identical tuples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultIdentityFamily {
    Record,
    Relation,
    Event,
    Series,
    Geo,
    Traversal,
}

impl ResultIdentityFamily {
    fn label(self) -> &'static str {
        match self {
            Self::Record => "record",
            Self::Relation => "relation",
            Self::Event => "event",
            Self::Series => "series",
            Self::Geo => "geo",
            Self::Traversal => "traversal",
        }
    }
}

fn append_component(out: &mut String, component: &str) {
    for character in component.chars() {
        match character {
            '%' => out.push_str("%25"),
            ':' => out.push_str("%3A"),
            other => out.push(other),
        }
    }
}

/// Encode a family and its ordered semantic components without collisions.
pub fn encode(family: ResultIdentityFamily, components: &[&str]) -> String {
    let mut out = String::from(family.label());
    for component in components {
        out.push(':');
        append_component(&mut out, component);
    }
    out
}

/// Encode a `(kind, id)` reference for a path, with no result-family prefix.
pub fn reference(reference: &RuntimeRef) -> String {
    let mut out = String::new();
    append_component(&mut out, reference.kind.as_str());
    out.push(':');
    append_component(&mut out, reference.id.as_str());
    out
}

/// Decode a canonical path reference. Reject malformed or noncanonical escapes.
pub fn parse_reference(value: &str) -> Option<RuntimeRef> {
    fn component(value: &str) -> Option<String> {
        let mut out = String::new();
        let mut chars = value.chars();
        while let Some(character) = chars.next() {
            match character {
                ':' => return None,
                '%' => match (chars.next()?, chars.next()?) {
                    ('2', '5') => out.push('%'),
                    ('3', 'A') => out.push(':'),
                    _ => return None,
                },
                other => out.push(other),
            }
        }
        Some(out)
    }
    let (kind, id) = value.split_once(':')?;
    RuntimeRef::parse(component(kind)?, component(id)?).ok()
}

/// Encode one typed reference in the given result family.
pub fn typed(family: ResultIdentityFamily, reference: &RuntimeRef) -> String {
    encode(family, &[reference.kind.as_str(), reference.id.as_str()])
}

/// A claim's existing context identity is opaque so its subject is not added
/// to logs or candidate IDs. Both inputs exclude NUL by their owning types.
pub fn claim(subject: &crate::Subject, predicate: &crate::Predicate) -> String {
    let mut bytes = b"rrd-context-claim-v1\0".to_vec();
    bytes.extend_from_slice(subject.as_str().as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(predicate.as_str().as_bytes());
    format!("claim:{}", crate::digest::sha256_hex(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_references_are_unambiguous_and_strictly_round_trip() {
        let references = [
            RuntimeRef::new("a", "b").unwrap(),
            RuntimeRef::new("a-b", "c").unwrap(),
            RuntimeRef::new("a", "b-c").unwrap(),
            RuntimeRef::event(crate::RuntimeType::new("observed").unwrap(), 7),
        ];
        let mut encoded = std::collections::BTreeSet::new();
        for value in &references {
            let path = reference(value);
            assert_eq!(parse_reference(&path), Some(value.clone()));
            assert!(encoded.insert(typed(ResultIdentityFamily::Record, value)));
        }
        assert_eq!(reference(&references[3]), "observed:cursor%3A7");
        for invalid in ["a:b:c", "a:%", "a:%3a", "a:%41", ":a", "a:"] {
            assert!(parse_reference(invalid).is_none(), "{invalid}");
        }
        // The encoding itself stays unambiguous for any text, even what no reference can hold.
        let mut components = std::collections::BTreeSet::new();
        for parts in [
            ["a:b", "c"],
            ["a", "b:c"],
            ["a%3Ab", "c"],
            ["α:β", "space % : 雪"],
        ] {
            let encoded = encode(ResultIdentityFamily::Record, &parts);
            assert!(components.insert(encoded.clone()), "{encoded}");
            let path = encoded.strip_prefix("record:").unwrap();
            assert_eq!(parse_reference(path), None, "{path}");
        }
    }
}
