//! Lexical analysis for BM25: one owner for the configuration and the text
//! pipeline, used by the commit-time postings this store maintains and by the
//! query engine's artifacts and scoring, so a term is the same term at write
//! time and at read time.

use crate::{Error, Result};
use rrd_core::digest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

pub const MAX_TOKEN_CHARS: usize = 40;
const MAX_STOP_WORDS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bm25Analyzer {
    /// Split on non-alphanumeric characters and apply Unicode lowercase.
    UnicodeLowercase,
    /// Preserve the source case after tokenization.
    UnicodeCaseSensitive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bm25Tokenizer {
    /// Split at every non-alphanumeric Unicode scalar.
    #[default]
    UnicodeAlphanumeric,
    /// Split only at Unicode whitespace.
    Whitespace,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bm25Stemmer {
    #[default]
    None,
    /// A deterministic, bounded English suffix stemmer.
    English,
}

/// How a code identifier such as `read_stamp` or `RrdEngine` is indexed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bm25Identifiers {
    /// Only the tokenizer's pieces: `read_stamp` is `read` and `stamp`, and
    /// `RrdEngine` is the single token `rrdengine`.
    #[default]
    Pieces,
    /// The tokenizer's pieces, each run of letters and digits joined by `_`
    /// as one whole token, and the case-change parts of a mixed-case piece:
    /// `read_stamp` is `read`, `stamp` and `read_stamp`, and `RrdEngine` is
    /// `rrdengine`, `rrd` and `engine`. A question can then name an
    /// identifier whole or by its words.
    WholeAndParts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bm25Config {
    pub analyzer: Bm25Analyzer,
    #[serde(default, skip_serializing_if = "is_default_tokenizer")]
    pub tokenizer: Bm25Tokenizer,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ascii_folding: bool,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub stop_words: BTreeSet<String>,
    #[serde(
        default = "default_min_token_chars",
        skip_serializing_if = "is_default_min_token_chars"
    )]
    pub min_token_chars: u16,
    #[serde(
        default = "default_max_token_chars",
        skip_serializing_if = "is_default_max_token_chars"
    )]
    pub max_token_chars: u16,
    #[serde(default, skip_serializing_if = "is_default_stemmer")]
    pub stemmer: Bm25Stemmer,
    #[serde(default, skip_serializing_if = "is_default_identifiers")]
    pub identifiers: Bm25Identifiers,
    /// BM25 term-frequency saturation parameter in millionths.
    pub k1_micros: u32,
    /// BM25 document-length normalization parameter in millionths.
    pub b_micros: u32,
}

impl Default for Bm25Config {
    fn default() -> Self {
        Self {
            analyzer: Bm25Analyzer::UnicodeLowercase,
            tokenizer: Bm25Tokenizer::UnicodeAlphanumeric,
            ascii_folding: false,
            stop_words: BTreeSet::new(),
            min_token_chars: 1,
            max_token_chars: MAX_TOKEN_CHARS as u16,
            stemmer: Bm25Stemmer::None,
            identifiers: Bm25Identifiers::Pieces,
            k1_micros: 1_200_000,
            b_micros: 750_000,
        }
    }
}

impl Bm25Config {
    pub fn validate(&self) -> Result<()> {
        let invalid = |message: String| Err(Error::IndexConstraint(message));
        if self.k1_micros == 0 || self.k1_micros > 10_000_000 {
            return invalid("BM25 k1 must be in (0, 10]".into());
        }
        if self.b_micros > 1_000_000 {
            return invalid("BM25 b must be in [0, 1]".into());
        }
        if self.min_token_chars == 0
            || self.min_token_chars > self.max_token_chars
            || usize::from(self.max_token_chars) > MAX_TOKEN_CHARS
        {
            return invalid(format!(
                "BM25 token length must satisfy 1 <= min <= max <= {MAX_TOKEN_CHARS}"
            ));
        }
        if self.stop_words.len() > MAX_STOP_WORDS
            || self.stop_words.iter().any(|word| word.is_empty())
        {
            return invalid("BM25 stop-word configuration is invalid".into());
        }
        let mut normalization_config = self.clone();
        normalization_config.stop_words.clear();
        normalization_config.identifiers = Bm25Identifiers::Pieces;
        if self
            .stop_words
            .iter()
            .any(|word| analyze(&normalization_config, word).as_slice() != [word.as_str()])
        {
            return invalid(
                "BM25 stop words must already be unique normalized single tokens".into(),
            );
        }
        Ok(())
    }

    pub fn k1(&self) -> f64 {
        f64::from(self.k1_micros) / 1_000_000.0
    }

    pub fn b(&self) -> f64 {
        f64::from(self.b_micros) / 1_000_000.0
    }

    pub fn digest(&self) -> Result<String> {
        self.validate()?;
        Ok(digest::sha256_hex(&serde_json::to_vec(self)?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bm25Offset {
    pub token_ordinal: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedToken {
    pub term: String,
    pub offset: Bm25Offset,
}

/// The terms of `text`, in order, with repeats.
pub fn analyze(config: &Bm25Config, text: &str) -> Vec<String> {
    analyze_with_offsets(config, text)
        .unwrap_or_default()
        .into_iter()
        .map(|token| token.term)
        .collect()
}

/// BM25 with Robertson IDF and document-side term-frequency saturation, as
/// in Qdrant's sparse-vector reference: one term's contribution to a
/// document's score.
pub fn bm25_contribution(
    config: &Bm25Config,
    documents: f64,
    document_frequency: f64,
    term_frequency: f64,
    document_length: f64,
    average_length: f64,
) -> f64 {
    let idf = ((documents - document_frequency + 0.5) / (document_frequency + 0.5) + 1.0).ln();
    let normalization = if average_length == 0.0 {
        1.0
    } else {
        1.0 - config.b() + config.b() * document_length / average_length
    };
    idf * (term_frequency * (config.k1() + 1.0)) / (term_frequency + config.k1() * normalization)
}

pub fn analyze_with_offsets(config: &Bm25Config, text: &str) -> Result<Vec<AnalyzedToken>> {
    let mut spans = token_spans(config.tokenizer, text)
        .into_iter()
        .map(|(start, end)| (start, end, false))
        .collect::<Vec<_>>();
    if config.identifiers == Bm25Identifiers::WholeAndParts {
        spans.extend(
            identifier_spans(text)
                .into_iter()
                .map(|(start, end)| (start, end, true)),
        );
        spans.sort_unstable();
        spans.dedup_by_key(|span| (span.0, span.1));
    }
    let mut tokens = Vec::new();
    let push = |tokens: &mut Vec<AnalyzedToken>, start: usize, end: usize, raw: &str| {
        let Some(term) = normalize(config, raw) else {
            return Ok(());
        };
        let token_ordinal = u32::try_from(tokens.len())
            .map_err(|_| Error::IndexConstraint("BM25 token ordinal exceeds u32".into()))?;
        let byte = |offset: usize| {
            u32::try_from(offset)
                .map_err(|_| Error::IndexConstraint("BM25 text byte offset exceeds u32".into()))
        };
        tokens.push(AnalyzedToken {
            term,
            offset: Bm25Offset {
                token_ordinal,
                byte_start: byte(start)?,
                byte_end: byte(end)?,
            },
        });
        Ok::<_, Error>(())
    };
    for (start, end, whole) in spans {
        let raw = &text[start..end];
        push(&mut tokens, start, end, raw)?;
        if config.identifiers == Bm25Identifiers::WholeAndParts && !whole {
            let parts = case_parts(raw);
            if parts.len() > 1 {
                for (part_start, part_end) in parts {
                    push(
                        &mut tokens,
                        start + part_start,
                        start + part_end,
                        &raw[part_start..part_end],
                    )?;
                }
            }
        }
    }
    Ok(tokens)
}

fn normalize(config: &Bm25Config, raw: &str) -> Option<String> {
    let mut term = match config.analyzer {
        Bm25Analyzer::UnicodeLowercase => raw.chars().flat_map(char::to_lowercase).collect(),
        Bm25Analyzer::UnicodeCaseSensitive => raw.to_owned(),
    };
    if config.ascii_folding {
        term = term
            .nfkd()
            .filter(|character| !is_combining_mark(*character))
            .collect();
    }
    if config.stemmer == Bm25Stemmer::English {
        term = stem_english(&term);
    }
    let chars = term.chars().count();
    (chars >= usize::from(config.min_token_chars)
        && chars <= usize::from(config.max_token_chars)
        && !config.stop_words.contains(&term))
    .then_some(term)
}

fn token_spans(tokenizer: Bm25Tokenizer, text: &str) -> Vec<(usize, usize)> {
    let accepted = |character: char| match tokenizer {
        Bm25Tokenizer::UnicodeAlphanumeric => character.is_alphanumeric(),
        Bm25Tokenizer::Whitespace => !character.is_whitespace(),
    };
    runs(text, accepted)
}

/// Runs of letters, digits and `_` that join at least two alphanumeric runs
/// with `_`, trimmed of leading and trailing `_`.
fn identifier_spans(text: &str) -> Vec<(usize, usize)> {
    runs(text, |character| {
        character.is_alphanumeric() || character == '_'
    })
    .into_iter()
    .filter_map(|(start, end)| {
        let run = &text[start..end];
        let trimmed = run.trim_matches('_');
        if trimmed.is_empty() || !trimmed.contains('_') {
            return None;
        }
        let offset = run.len() - run.trim_start_matches('_').len();
        Some((start + offset, start + offset + trimmed.len()))
    })
    .collect()
}

fn runs(text: &str, accepted: impl Fn(char) -> bool) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (offset, character) in text.char_indices() {
        if accepted(character) {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            spans.push((begin, offset));
        }
    }
    if let Some(begin) = start {
        spans.push((begin, text.len()));
    }
    spans
}

/// Byte ranges of the case-change parts of one alphanumeric piece:
/// `RrdEngine` is `Rrd` and `Engine`, `HTTPServer` is `HTTP` and `Server`,
/// and `v2Plan` is `v2` and `Plan`.
fn case_parts(piece: &str) -> Vec<(usize, usize)> {
    let characters = piece.char_indices().collect::<Vec<_>>();
    let mut parts = Vec::new();
    let mut start = 0;
    for index in 1..characters.len() {
        let (offset, current) = characters[index];
        let previous = characters[index - 1].1;
        let next = characters.get(index + 1).map(|(_, character)| *character);
        let lower_to_upper =
            (previous.is_lowercase() || previous.is_numeric()) && current.is_uppercase();
        let acronym_end = previous.is_uppercase()
            && current.is_uppercase()
            && next.is_some_and(char::is_lowercase);
        if lower_to_upper || acronym_end {
            parts.push((start, offset));
            start = offset;
        }
    }
    parts.push((start, piece.len()));
    parts
}

fn stem_english(term: &str) -> String {
    if term.len() > 4 && term.ends_with("ies") {
        return format!("{}y", &term[..term.len() - 3]);
    }
    for suffix in ["ingly", "edly", "ing", "ed"] {
        if term.len() > suffix.len() + 2 && term.ends_with(suffix) {
            return term[..term.len() - suffix.len()].to_owned();
        }
    }
    if term.len() > 4
        && ["sses", "ches", "shes", "xes", "zes", "ses"]
            .iter()
            .any(|suffix| term.ends_with(suffix))
    {
        return term[..term.len() - 2].to_owned();
    }
    if term.len() > 3 && term.ends_with('s') {
        return term[..term.len() - 1].to_owned();
    }
    term.to_owned()
}

fn default_min_token_chars() -> u16 {
    1
}
fn default_max_token_chars() -> u16 {
    MAX_TOKEN_CHARS as u16
}
fn is_default_min_token_chars(value: &u16) -> bool {
    *value == default_min_token_chars()
}
fn is_default_max_token_chars(value: &u16) -> bool {
    *value == default_max_token_chars()
}
fn is_default_tokenizer(value: &Bm25Tokenizer) -> bool {
    *value == Bm25Tokenizer::default()
}
fn is_default_stemmer(value: &Bm25Stemmer) -> bool {
    *value == Bm25Stemmer::default()
}
fn is_default_identifiers(value: &Bm25Identifiers) -> bool {
    *value == Bm25Identifiers::default()
}
fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code() -> Bm25Config {
        Bm25Config {
            identifiers: Bm25Identifiers::WholeAndParts,
            ..Bm25Config::default()
        }
    }

    #[test]
    fn pieces_is_the_default_and_keeps_existing_terms() {
        assert_eq!(
            analyze(&Bm25Config::default(), "read_stamp RrdEngine"),
            ["read", "stamp", "rrdengine"]
        );
        let unchanged = serde_json::to_value(Bm25Config::default()).unwrap();
        assert!(unchanged.get("identifiers").is_none(), "{unchanged}");
    }

    #[test]
    fn identifiers_index_whole_and_by_their_words() {
        assert_eq!(
            analyze(&code(), "fn control_attunement(engine: &RrdEngine)"),
            [
                "fn",
                "control",
                "control_attunement",
                "attunement",
                "engine",
                "rrdengine",
                "rrd",
                "engine"
            ]
        );
        assert_eq!(
            analyze(&code(), "HTTPServer v2Plan"),
            ["httpserver", "http", "server", "v2plan", "v2", "plan"]
        );
        assert_eq!(analyze(&code(), "__init__"), ["init"]);
    }

    #[test]
    fn whole_identifier_offsets_cover_the_identifier() {
        let text = "x = read_stamp";
        let tokens = analyze_with_offsets(&code(), text).unwrap();
        let whole = tokens
            .iter()
            .find(|token| token.term == "read_stamp")
            .unwrap();
        assert_eq!(
            &text[whole.offset.byte_start as usize..whole.offset.byte_end as usize],
            "read_stamp"
        );
        let ordinals = tokens
            .iter()
            .map(|token| token.offset.token_ordinal)
            .collect::<Vec<_>>();
        assert_eq!(ordinals, (0..tokens.len() as u32).collect::<Vec<_>>());
    }
}
