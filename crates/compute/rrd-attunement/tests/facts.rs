use rrd_attunement::facts::{source_facts, SourceFacts, SymbolRole};
use rrd_attunement::parse::{ParseError, ParseLimits};
use rrd_core::digest::sha256_hex;

fn limits() -> ParseLimits {
    ParseLimits {
        input_bytes: 1 << 20,
        nodes: 1 << 20,
        depth: 256,
        output_bytes: 1 << 20,
        elapsed_micros: 5_000_000,
    }
}

fn facts(language: &str, source: &str) -> SourceFacts {
    let digest = sha256_hex(source.as_bytes());
    source_facts(
        "source-a",
        &digest,
        language,
        source.as_bytes(),
        &limits(),
        || false,
    )
    .unwrap()
}

fn text(source: &str, start: u64, end: u64) -> &str {
    &source[start as usize..end as usize]
}

fn defined(source: &str, facts: &SourceFacts) -> Vec<(String, String)> {
    let mut names: Vec<_> = facts
        .symbols
        .iter()
        .filter(|symbol| symbol.role == SymbolRole::Definition)
        .map(|symbol| {
            (
                text(source, symbol.name_start, symbol.name_end).to_string(),
                symbol.kind.clone(),
            )
        })
        .collect();
    names.sort();
    names
}

const RUST: &str = "use std::fmt;\n\n// one\npub struct Engine;\n\nimpl Engine {\n    fn start(&self) {\n        helper();\n        helper();\n    }\n}\n\nfn helper() {}\n";

#[test]
fn rust_definitions_are_named_from_exact_source_ranges() {
    let facts = facts("rust", RUST);
    assert_eq!(
        defined(RUST, &facts),
        [
            ("Engine".to_string(), "class".to_string()),
            ("helper".to_string(), "function".to_string()),
            ("start".to_string(), "method".to_string()),
        ]
    );
    assert_eq!(facts.content_sha256, sha256_hex(RUST.as_bytes()));
    assert_eq!(facts.error_nodes, 0);
}

#[test]
fn a_method_is_not_also_reported_as_a_function() {
    let facts = facts("rust", RUST);
    let start = facts
        .symbols
        .iter()
        .filter(|symbol| text(RUST, symbol.name_start, symbol.name_end) == "start")
        .count();
    assert_eq!(start, 1);
}

#[test]
fn references_to_one_name_collapse_with_their_count_and_first_line() {
    let facts = facts("rust", RUST);
    let helper = facts
        .symbols
        .iter()
        .find(|symbol| {
            symbol.role == SymbolRole::Reference
                && text(RUST, symbol.name_start, symbol.name_end) == "helper"
        })
        .expect("a reference to helper");
    assert_eq!(helper.count, 2);
    assert_eq!(helper.start_line, 7);
}

#[test]
fn python_and_go_definitions_come_from_their_tag_queries() {
    let python = "class Store:\n    def read(self):\n        return load()\n";
    let found = facts("python", python);
    let names: Vec<_> = defined(python, &found).into_iter().map(|d| d.0).collect();
    assert_eq!(names, ["Store", "read"]);
    let go = "package main\n\nfunc Serve() {\n\tlisten()\n}\n";
    let found = facts("go", go);
    let names: Vec<_> = defined(go, &found).into_iter().map(|d| d.0).collect();
    assert_eq!(names, ["Serve"]);
}

#[test]
fn syntax_errors_are_counted_not_hidden() {
    let source = "fn broken( {\n";
    assert!(facts("rust", source).error_nodes > 0);
}

#[test]
fn limits_digest_and_language_are_enforced() {
    let source = RUST.as_bytes();
    let digest = sha256_hex(source);
    let run = |digest: &str, language: &str, limits: &ParseLimits| {
        source_facts("a", digest, language, source, limits, || false).unwrap_err()
    };
    assert_eq!(
        run(&"0".repeat(64), "rust", &limits()),
        ParseError::SourceDigestMismatch
    );
    assert_eq!(
        run(&digest, "cobol", &limits()),
        ParseError::UnsupportedLanguage
    );
    let tight = ParseLimits {
        nodes: 3,
        ..limits()
    };
    assert_eq!(run(&digest, "rust", &tight), ParseError::LimitExceeded);
    let small = ParseLimits {
        output_bytes: 8,
        ..limits()
    };
    assert_eq!(run(&digest, "rust", &small), ParseError::LimitExceeded);
    assert_eq!(
        source_facts("a", &digest, "rust", source, &limits(), || true).unwrap_err(),
        ParseError::Cancelled
    );
}
