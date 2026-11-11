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

fn facts(language: &str, source: &str, chunk_bytes: u64) -> SourceFacts {
    let digest = sha256_hex(source.as_bytes());
    source_facts(
        "source-a",
        &digest,
        language,
        source.as_bytes(),
        &limits(),
        chunk_bytes,
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
    let facts = facts("rust", RUST, 4096);
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
    let facts = facts("rust", RUST, 4096);
    let start = facts
        .symbols
        .iter()
        .filter(|symbol| text(RUST, symbol.name_start, symbol.name_end) == "start")
        .count();
    assert_eq!(start, 1);
}

#[test]
fn references_to_one_name_collapse_with_their_count_and_first_line() {
    let facts = facts("rust", RUST, 4096);
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
    let found = facts("python", python, 4096);
    let names: Vec<_> = defined(python, &found).into_iter().map(|d| d.0).collect();
    assert_eq!(names, ["Store", "read"]);
    let go = "package main\n\nfunc Serve() {\n\tlisten()\n}\n";
    let found = facts("go", go, 4096);
    let names: Vec<_> = defined(go, &found).into_iter().map(|d| d.0).collect();
    assert_eq!(names, ["Serve"]);
}

#[test]
fn top_level_items_become_spans_and_comments_do_not() {
    let facts = facts("rust", RUST, 4096);
    let kinds: Vec<_> = facts.spans.iter().map(|span| span.kind.as_str()).collect();
    assert_eq!(
        kinds,
        [
            "use_declaration",
            "struct_item",
            "impl_item",
            "function_item"
        ]
    );
    let helper = facts.spans.last().unwrap();
    assert_eq!(
        text(RUST, helper.start_byte, helper.end_byte),
        "fn helper() {}"
    );
    assert_eq!(helper.start_line, 12);
}

#[test]
fn a_small_file_is_one_chunk_covering_its_code() {
    let facts = facts("rust", RUST, 4096);
    assert_eq!(facts.chunks.len(), 1);
    let chunk = &facts.chunks[0];
    assert_eq!(
        text(RUST, chunk.start_byte, chunk.end_byte).trim(),
        RUST.trim()
    );
}

#[test]
fn chunks_stay_inside_the_bound_and_lose_no_code() {
    let mut source = String::new();
    for index in 0..40 {
        source.push_str(&format!("fn function_{index}() {{ let π = {index}; }}\n"));
    }
    let bound = 120;
    let facts = facts("rust", &source, bound);
    assert!(facts.chunks.len() > 1);
    let mut covered = String::new();
    for chunk in &facts.chunks {
        assert!(chunk.end_byte - chunk.start_byte <= bound);
        covered.push_str(text(&source, chunk.start_byte, chunk.end_byte));
    }
    let squeezed = |value: &str| value.split_whitespace().collect::<String>();
    assert_eq!(squeezed(&covered), squeezed(&source));
}

#[test]
fn one_oversized_token_is_split_on_character_boundaries() {
    let source = format!("const VALUE: &str = \"{}\";\n", "π".repeat(400));
    let bound = 100;
    let facts = facts("rust", &source, bound);
    assert!(facts.chunks.len() > 1);
    for chunk in &facts.chunks {
        assert!(chunk.end_byte - chunk.start_byte <= bound);
        assert!(source.is_char_boundary(chunk.start_byte as usize));
        assert!(source.is_char_boundary(chunk.end_byte as usize));
    }
}

#[test]
fn delimiters_the_grammar_hides_stay_in_the_chunks() {
    // tree-sitter-rust scans raw-string delimiters externally, so no child
    // node covers `r#"` or `"#`; an oversized literal must keep them anyway.
    let body = "line of text
"
    .repeat(20);
    let source = format!(
        "fn main() {{
    let text = r#\"{body}\"#;
}}
"
    );
    let bound = 64;
    let facts = facts("rust", &source, bound);
    let mut covered = String::new();
    for chunk in &facts.chunks {
        assert!(chunk.end_byte - chunk.start_byte <= bound);
        covered.push_str(text(&source, chunk.start_byte, chunk.end_byte));
    }
    let squeezed = |value: &str| value.split_whitespace().collect::<String>();
    assert_eq!(squeezed(&covered), squeezed(&source));
}

#[test]
fn syntax_errors_are_counted_not_hidden() {
    let source = "fn broken( {\n";
    assert!(facts("rust", source, 4096).error_nodes > 0);
}

#[test]
fn limits_digest_and_language_are_enforced() {
    let source = RUST.as_bytes();
    let digest = sha256_hex(source);
    let run = |digest: &str, language: &str, limits: &ParseLimits, chunk: u64| {
        source_facts("a", digest, language, source, limits, chunk, || false).unwrap_err()
    };
    assert_eq!(
        run(&"0".repeat(64), "rust", &limits(), 4096),
        ParseError::SourceDigestMismatch
    );
    assert_eq!(
        run(&digest, "cobol", &limits(), 4096),
        ParseError::UnsupportedLanguage
    );
    assert_eq!(
        run(&digest, "rust", &limits(), 0),
        ParseError::LimitExceeded
    );
    let tight = ParseLimits {
        nodes: 3,
        ..limits()
    };
    assert_eq!(
        run(&digest, "rust", &tight, 4096),
        ParseError::LimitExceeded
    );
    let small = ParseLimits {
        output_bytes: 8,
        ..limits()
    };
    assert_eq!(
        run(&digest, "rust", &small, 4096),
        ParseError::LimitExceeded
    );
    assert_eq!(
        source_facts("a", &digest, "rust", source, &limits(), 4096, || true).unwrap_err(),
        ParseError::Cancelled
    );
}
