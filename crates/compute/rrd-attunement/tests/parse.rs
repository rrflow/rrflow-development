use rrd_attunement::parse::{parse_source, ParseError, ParseLimits};
use rrd_core::digest::sha256_hex;

fn limits() -> ParseLimits {
    ParseLimits {
        input_bytes: 4096,
        nodes: 512,
        depth: 64,
        output_bytes: 65536,
        elapsed_micros: 1_000_000,
    }
}

#[test]
fn rust_symbol_ranges_are_exact_bytes_and_bound_to_source() {
    let source = "// π\nfn answer() -> u32 { 42 }\n".as_bytes();
    let digest = sha256_hex(source);
    let result = parse_source("source-a", &digest, "rust", source, &limits(), || false).unwrap();
    assert_eq!(result.format_version, 1);
    assert_eq!(result.source_id, "source-a");
    assert_eq!(result.content_sha256, digest);
    assert_eq!(result.language, "rust");
    assert!(!result.parser_revision.is_empty());
    let start = std::str::from_utf8(source).unwrap().find("answer").unwrap();
    assert!(result.nodes.iter().any(|node| {
        node.kind == "identifier"
            && node.start_byte == start as u64
            && node.end_byte == (start + "answer".len()) as u64
            && &source[node.start_byte as usize..node.end_byte as usize] == b"answer"
    }));
}

#[test]
fn explicitly_selected_python_and_go_produce_symbol_ranges() {
    for (language, source, name) in [
        ("python", "def lookup():\n    return 7\n", "lookup"),
        (
            "go",
            "package demo\nfunc Lookup() int { return 7 }\n",
            "Lookup",
        ),
    ] {
        let bytes = source.as_bytes();
        let result = parse_source(
            "source-d",
            &sha256_hex(bytes),
            language,
            bytes,
            &limits(),
            || false,
        )
        .unwrap();
        let start = source.find(name).unwrap();
        assert!(
            result.nodes.iter().any(|node| {
                node.start_byte == start as u64 && node.end_byte == (start + name.len()) as u64
            }),
            "missing symbol for {language}"
        );
        assert!(!result.nodes.iter().any(|node| node.error || node.missing));
    }
}

#[test]
fn incomplete_syntax_is_retained_and_unavailable_languages_are_explicit() {
    let source = b"fn broken( {";
    let result = parse_source(
        "source-b",
        &sha256_hex(source),
        "rust",
        source,
        &limits(),
        || false,
    )
    .unwrap();
    assert!(result.nodes.iter().any(|node| node.error || node.missing));
    assert!(matches!(
        parse_source(
            "source-b",
            &sha256_hex(source),
            "unknown",
            source,
            &limits(),
            || false
        ),
        Err(ParseError::UnsupportedLanguage)
    ));
}

#[test]
fn changed_bytes_and_exhausted_limits_never_produce_a_proposal() {
    let source = b"fn answer() {}";
    assert!(matches!(
        parse_source(
            "source-c",
            &"0".repeat(64),
            "rust",
            source,
            &limits(),
            || false
        ),
        Err(ParseError::SourceDigestMismatch)
    ));
    let mut constrained = limits();
    constrained.input_bytes = 1;
    assert!(matches!(
        parse_source(
            "source-c",
            &sha256_hex(source),
            "rust",
            source,
            &constrained,
            || false
        ),
        Err(ParseError::LimitExceeded)
    ));
    assert!(matches!(
        parse_source(
            "source-c",
            &sha256_hex(source),
            "rust",
            source,
            &limits(),
            || true
        ),
        Err(ParseError::Cancelled)
    ));
    constrained = limits();
    constrained.nodes = 1;
    assert!(matches!(
        parse_source(
            "source-c",
            &sha256_hex(source),
            "rust",
            source,
            &constrained,
            || false
        ),
        Err(ParseError::LimitExceeded)
    ));
    constrained = limits();
    constrained.output_bytes = 16;
    assert!(matches!(
        parse_source(
            "source-c",
            &sha256_hex(source),
            "rust",
            source,
            &constrained,
            || false
        ),
        Err(ParseError::LimitExceeded)
    ));
    constrained = limits();
    constrained.depth = 1;
    assert!(matches!(
        parse_source(
            "source-c",
            &sha256_hex(source),
            "rust",
            source,
            &constrained,
            || false
        ),
        Err(ParseError::LimitExceeded)
    ));
    let invalid_utf8 = [0xff];
    assert!(matches!(
        parse_source(
            "source-c",
            &sha256_hex(&invalid_utf8),
            "rust",
            &invalid_utf8,
            &limits(),
            || false
        ),
        Err(ParseError::InvalidEncoding)
    ));
}

#[test]
fn output_limit_counts_the_complete_serialized_proposal() {
    let source = b"fn answer() {}";
    let digest = sha256_hex(source);
    let expected = parse_source("source-e", &digest, "rust", source, &limits(), || false).unwrap();
    let exact = serde_json::to_vec(&expected).unwrap().len() as u64;
    let mut constrained = limits();
    constrained.output_bytes = exact;
    assert_eq!(
        parse_source("source-e", &digest, "rust", source, &constrained, || false).unwrap(),
        expected
    );
    constrained.output_bytes = exact - 1;
    assert!(matches!(
        parse_source("source-e", &digest, "rust", source, &constrained, || false),
        Err(ParseError::LimitExceeded)
    ));
}

#[test]
fn cancellation_during_parsing_does_not_return_a_partial_proposal() {
    let source = b"fn answer() { let value = 42; }";
    let calls = std::cell::Cell::new(0);
    let result = parse_source(
        "source-f",
        &sha256_hex(source),
        "rust",
        source,
        &limits(),
        || {
            calls.set(calls.get() + 1);
            calls.get() == 2
        },
    );
    assert!(matches!(result, Err(ParseError::Cancelled)));
}

#[test]
fn cancellation_callback_panic_does_not_unwind_through_native_parser() {
    let source = b"fn answer() {}";
    let calls = std::cell::Cell::new(0);
    let result = parse_source(
        "source-g",
        &sha256_hex(source),
        "rust",
        source,
        &limits(),
        || {
            calls.set(calls.get() + 1);
            assert_ne!(calls.get(), 2, "synthetic callback failure");
            false
        },
    );
    assert!(matches!(result, Err(ParseError::ParserUnavailable)));
}
