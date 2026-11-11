//! Bounded source facts: definitions and references from each grammar's tag
//! query, top-level spans, and AST chunks. Every fact is a byte range; the
//! engine slices names and text from bytes it read and verified itself, so a
//! worker's output can never supply source content.

use crate::parse::{parse_tree, ParseError, ParseLimits, Parsed};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Node, Query, QueryCursor};

pub const SOURCE_FACTS_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolRole {
    Definition,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSymbol {
    /// The tag's suffix, such as `function`, `class` or `call`.
    pub kind: String,
    pub role: SymbolRole,
    pub name_start: u64,
    pub name_end: u64,
    pub start_byte: u64,
    pub end_byte: u64,
    /// Zero-based line of the first occurrence.
    pub start_line: u64,
    /// Occurrences of this name and kind; always 1 for a definition.
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub kind: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub start_line: u64,
    pub end_line: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceChunk {
    pub start_byte: u64,
    pub end_byte: u64,
    pub start_line: u64,
    pub end_line: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFacts {
    pub format_version: u16,
    pub source_id: String,
    pub content_sha256: String,
    pub language: String,
    pub parser_revision: String,
    pub nodes: u64,
    pub error_nodes: u64,
    pub symbols: Vec<SourceSymbol>,
    pub spans: Vec<SourceSpan>,
    pub chunks: Vec<SourceChunk>,
}

fn tag_query(language: &str) -> Option<&'static str> {
    match language {
        "rust" => Some(tree_sitter_rust::TAGS_QUERY),
        "python" => Some(tree_sitter_python::TAGS_QUERY),
        "go" => Some(tree_sitter_go::TAGS_QUERY),
        _ => None,
    }
}

struct Budget<'a, C> {
    parsed: &'a Parsed,
    cancelled: &'a C,
}

impl<C: Fn() -> bool> Budget<'_, C> {
    fn check(&self) -> Result<(), ParseError> {
        if (self.cancelled)() {
            return Err(ParseError::Cancelled);
        }
        if self.parsed.started.elapsed() >= self.parsed.deadline {
            return Err(ParseError::LimitExceeded);
        }
        Ok(())
    }
}

/// Parses `source` and extracts its facts within `limits`. No chunk exceeds
/// `chunk_bytes`.
pub fn source_facts(
    source_id: &str,
    expected_sha256: &str,
    language: &str,
    source: &[u8],
    limits: &ParseLimits,
    chunk_bytes: u64,
    cancelled: impl Fn() -> bool,
) -> Result<SourceFacts, ParseError> {
    if chunk_bytes == 0 {
        return Err(ParseError::LimitExceeded);
    }
    let parsed = parse_tree(
        source_id,
        expected_sha256,
        language,
        source,
        limits,
        &cancelled,
    )?;
    let budget = Budget {
        parsed: &parsed,
        cancelled: &cancelled,
    };
    let root = parsed.tree.root_node();
    let (nodes, error_nodes) = count_nodes(root, limits, &budget)?;
    let lines = LineIndex::new(source);
    let facts = SourceFacts {
        format_version: SOURCE_FACTS_VERSION,
        source_id: source_id.into(),
        content_sha256: expected_sha256.into(),
        language: language.into(),
        parser_revision: parsed.revision.into(),
        nodes,
        error_nodes,
        symbols: symbols(&parsed.language, language, root, source, &budget)?,
        spans: spans(root, &lines),
        chunks: chunks(root, source, chunk_bytes, &lines, &budget)?,
    };
    let bytes = serde_json::to_vec(&facts).map_err(|_| ParseError::LimitExceeded)?;
    if bytes.len() as u64 > limits.output_bytes {
        return Err(ParseError::LimitExceeded);
    }
    Ok(facts)
}

fn count_nodes<C: Fn() -> bool>(
    root: Node<'_>,
    limits: &ParseLimits,
    budget: &Budget<'_, C>,
) -> Result<(u64, u64), ParseError> {
    let mut cursor = root.walk();
    let (mut nodes, mut errors, mut depth) = (0u64, 0u64, 1u64);
    loop {
        budget.check()?;
        nodes += 1;
        if nodes > limits.nodes || depth > limits.depth {
            return Err(ParseError::LimitExceeded);
        }
        let node = cursor.node();
        errors += u64::from(node.is_error() || node.is_missing());
        if cursor.goto_first_child() {
            depth += 1;
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return Ok((nodes, errors));
            }
            depth -= 1;
        }
    }
}

fn symbols<C: Fn() -> bool>(
    grammar: &Language,
    language: &str,
    root: Node<'_>,
    source: &[u8],
    budget: &Budget<'_, C>,
) -> Result<Vec<SourceSymbol>, ParseError> {
    let text = tag_query(language).ok_or(ParseError::UnsupportedLanguage)?;
    let query = Query::new(grammar, text).map_err(|_| ParseError::ParserUnavailable)?;
    let names = query.capture_names();
    let mut output: Vec<SourceSymbol> = Vec::new();
    let mut defined: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut referenced: BTreeMap<(&[u8], String), usize> = BTreeMap::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root, source);
    while let Some(found) = matches.next() {
        budget.check()?;
        let name = found
            .captures
            .iter()
            .find(|capture| names[capture.index as usize] == "name");
        let tag = found.captures.iter().find_map(|capture| {
            let (role, kind) = names[capture.index as usize].split_once('.')?;
            let role = match role {
                "definition" => SymbolRole::Definition,
                "reference" => SymbolRole::Reference,
                _ => return None,
            };
            Some((role, kind, capture.node))
        });
        let (Some(name), Some((role, kind, extent))) = (name, tag) else {
            continue;
        };
        let range = name.node.byte_range();
        let symbol = SourceSymbol {
            kind: kind.into(),
            role,
            name_start: range.start as u64,
            name_end: range.end as u64,
            start_byte: extent.start_byte() as u64,
            end_byte: extent.end_byte() as u64,
            start_line: name.node.start_position().row as u64,
            count: 1,
        };
        match role {
            // Grammars tag a method both as a method and as a function; the
            // more specific pattern comes first.
            SymbolRole::Definition => {
                if defined.insert((range.start, range.end)) {
                    output.push(symbol);
                }
            }
            SymbolRole::Reference => {
                let key = (&source[range.clone()], symbol.kind.clone());
                if let Some(index) = referenced.get(&key) {
                    output[*index].count += 1;
                } else {
                    referenced.insert(key, output.len());
                    output.push(symbol);
                }
            }
        }
    }
    output.sort_by_key(|symbol| (symbol.name_start, symbol.name_end));
    Ok(output)
}

struct LineIndex(Vec<usize>);

impl LineIndex {
    fn new(source: &[u8]) -> Self {
        Self(
            source
                .iter()
                .enumerate()
                .filter(|(_, byte)| **byte == b'\n')
                .map(|(offset, _)| offset)
                .collect(),
        )
    }

    fn line(&self, offset: usize) -> u64 {
        self.0.partition_point(|newline| *newline < offset) as u64
    }
}

fn spans(root: Node<'_>, lines: &LineIndex) -> Vec<SourceSpan> {
    let mut cursor = root.walk();
    root.named_children(&mut cursor)
        .filter(|node| !node.kind().contains("comment"))
        .map(|node| SourceSpan {
            kind: node.kind().into(),
            start_byte: node.start_byte() as u64,
            end_byte: node.end_byte() as u64,
            start_line: lines.line(node.start_byte()),
            end_line: lines.line(node.end_byte()),
        })
        .collect()
}

/// Splits text no syntax node divides further, on line then character edges.
fn split_text(
    source: &[u8],
    mut start: usize,
    end: usize,
    limit: usize,
    units: &mut Vec<(usize, usize)>,
) {
    while end - start > limit {
        let mut cut = start + limit;
        if let Some(newline) = source[start..cut].iter().rposition(|byte| *byte == b'\n') {
            cut = start + newline + 1;
        } else {
            // UTF-8 continuation bytes are 0b10xxxxxx; `start` is a boundary.
            while cut > start + 1 && source[cut] & 0xC0 == 0x80 {
                cut -= 1;
            }
        }
        units.push((start, cut));
        start = cut;
    }
    if end > start {
        units.push((start, end));
    }
}

enum Piece<'tree> {
    Node(Node<'tree>),
    Text(usize, usize),
}

fn has_code(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| !byte.is_ascii_whitespace())
}

fn chunks<C: Fn() -> bool>(
    root: Node<'_>,
    source: &[u8],
    chunk_bytes: u64,
    lines: &LineIndex,
    budget: &Budget<'_, C>,
) -> Result<Vec<SourceChunk>, ParseError> {
    let limit = usize::try_from(chunk_bytes).unwrap_or(usize::MAX);
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut stack = vec![Piece::Node(root)];
    while let Some(piece) = stack.pop() {
        budget.check()?;
        let node = match piece {
            Piece::Text(start, end) => {
                split_text(source, start, end, limit, &mut units);
                continue;
            }
            Piece::Node(node) => node,
        };
        let (start, end) = (node.start_byte(), node.end_byte());
        if end - start <= limit {
            if end > start {
                units.push((start, end));
            }
        } else if node.child_count() == 0 {
            split_text(source, start, end, limit, &mut units);
        } else {
            // A grammar can hide tokens, such as the delimiters of a Rust raw
            // string, that no child then covers; code between children is
            // kept as text so the chunks lose nothing but whitespace.
            let mut cursor = node.walk();
            let mut pieces = Vec::new();
            let mut at = start;
            for child in node.children(&mut cursor) {
                if has_code(&source[at..child.start_byte().max(at)]) {
                    pieces.push(Piece::Text(at, child.start_byte()));
                }
                at = at.max(child.end_byte());
                pieces.push(Piece::Node(child));
            }
            if has_code(&source[at..end.max(at)]) {
                pieces.push(Piece::Text(at, end));
            }
            stack.extend(pieces.into_iter().rev());
        }
    }
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in units {
        match merged.last_mut() {
            Some(current) if end - current.0 <= limit => current.1 = end,
            _ => merged.push((start, end)),
        }
    }
    Ok(merged
        .into_iter()
        .map(|(start, end)| SourceChunk {
            start_byte: start as u64,
            end_byte: end as u64,
            start_line: lines.line(start),
            end_line: lines.line(end),
        })
        .collect())
}
