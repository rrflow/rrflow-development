//! Persistent BM25 postings maintained in the same transaction as the records
//! they index (E-03).
//!
//! A BM25 commit binding keeps, for every current record of its kind, one
//! posting per distinct term of the bound text field, the term's document
//! frequency, and the index's document and token totals. Postings describe the
//! current records only, like scalar entries: a lookup is exact at the head
//! read stamp for a valid time at or after the index's watermark, the latest
//! valid time at which the set of visible indexed versions changed (a version
//! became valid, was replaced or was retired), and reports itself not
//! applicable otherwise so the caller can read history. A current version with
//! an explicit end of validity would change the set later, so it raises the
//! watermark past every valid time until the index is rebuilt. Scalar
//! bindings keep the same watermark in the same statistics record.

use super::super::keyspaces::{self, Space};
use super::runtime_state::{checked_key, get_json, scan_space_bounded, AccessRead};
use crate::lexical::{analyze, bm25_contribution, Bm25Config};
use crate::{Error, Result, StorageTransaction};
use rrd_core::{ProjectionId, RuntimeRecord, RuntimeRef, RuntimeValue, ScopeId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Removable postings in one index; beyond this a rebuild is refused.
const MAX_INDEX_POSTINGS: usize = 10_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Posting {
    pub record: RuntimeRef,
    pub term_frequency: u32,
    pub document_tokens: u32,
    pub valid_from: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexStatistics {
    /// Documents with postings; zero for a scalar index.
    pub documents: u64,
    /// Tokens across those documents; zero for a scalar index.
    pub tokens: u64,
    /// The latest valid time at which the visible indexed set changed; a
    /// lookup for an earlier valid time needs history.
    pub watermark: u64,
}

/// The watermark a newly current version imposes: its valid time, or the
/// end of time when it carries an explicit end of validity.
pub(crate) fn addition_watermark(record: &RuntimeRecord) -> u64 {
    if record.valid_to.is_some() {
        u64::MAX
    } else {
        record.valid_from
    }
}

/// Raises the watermark of any commit-bound index.
pub(crate) fn raise_watermark(
    reader: &(impl AccessRead + ?Sized),
    put: &mut dyn FnMut(Space, Vec<u8>, Option<Vec<u8>>) -> Result<()>,
    scope: &ScopeId,
    index: &ProjectionId,
    at: u64,
) -> Result<()> {
    let key = keyspaces::runtime_term_statistics_key(scope, index);
    let current: IndexStatistics =
        get_json(reader, keyspaces::RUNTIME_TERM_STATISTICS, &key)?.unwrap_or_default();
    if at <= current.watermark {
        return Ok(());
    }
    put(
        keyspaces::RUNTIME_TERM_STATISTICS,
        key,
        Some(serde_json::to_vec(&IndexStatistics {
            watermark: at,
            ..current
        })?),
    )
}

/// The watermark of a commit-bound index.
pub(crate) fn watermark(
    reader: &(impl AccessRead + ?Sized),
    scope: &ScopeId,
    index: &ProjectionId,
) -> Result<u64> {
    Ok(get_json::<IndexStatistics>(
        reader,
        keyspaces::RUNTIME_TERM_STATISTICS,
        &keyspaces::runtime_term_statistics_key(scope, index),
    )?
    .unwrap_or_default()
    .watermark)
}

/// What one document contributes: its distinct terms with frequencies.
struct Document {
    terms: BTreeMap<String, u32>,
    tokens: u32,
}

fn document(config: &Bm25Config, text: &str) -> Result<Document> {
    let mut terms = BTreeMap::<String, u32>::new();
    let mut tokens = 0_u32;
    for term in analyze(config, text) {
        tokens = tokens
            .checked_add(1)
            .ok_or_else(|| Error::IndexConstraint("BM25 document token count overflow".into()))?;
        *terms.entry(term).or_default() += 1;
    }
    Ok(Document { terms, tokens })
}

fn text_of(record: &RuntimeRecord, field: &str) -> Option<String> {
    match record.properties.get(field) {
        Some(RuntimeValue::String(text)) => Some(text.clone()),
        _ => None,
    }
}

/// The writes one batch of document changes makes to one index, gathered so
/// shared keys (document frequencies, totals) are read and written once.
pub(crate) struct LexicalChanges<'a> {
    scope: &'a ScopeId,
    index: &'a ProjectionId,
    config: &'a Bm25Config,
    field: &'a str,
    postings: BTreeMap<(String, RuntimeRef), Option<Posting>>,
    frequencies: BTreeMap<String, i64>,
    documents: i64,
    tokens: i64,
    watermark: u64,
}

impl<'a> LexicalChanges<'a> {
    pub(crate) fn new(
        scope: &'a ScopeId,
        index: &'a ProjectionId,
        config: &'a Bm25Config,
        field: &'a str,
    ) -> Self {
        Self {
            scope,
            index,
            config,
            field,
            postings: BTreeMap::new(),
            frequencies: BTreeMap::new(),
            documents: 0,
            tokens: 0,
            watermark: 0,
        }
    }

    /// Removes the postings of a record version that is no longer current.
    /// `removed_at` is the valid time it stopped being visible: its
    /// retirement, or the valid time of the version that replaced it.
    pub(crate) fn remove(&mut self, record: &RuntimeRecord, removed_at: Option<u64>) -> Result<()> {
        if let Some(at) = removed_at {
            self.watermark = self.watermark.max(at);
        }
        let Some(text) = text_of(record, self.field) else {
            return Ok(());
        };
        let document = document(self.config, &text)?;
        self.documents -= 1;
        self.tokens -= i64::from(document.tokens);
        for term in document.terms.into_keys() {
            *self.frequencies.entry(term.clone()).or_default() -= 1;
            self.postings.insert((term, record.reference.clone()), None);
        }
        Ok(())
    }

    /// Adds the postings of the current version of a record.
    pub(crate) fn add(&mut self, record: &RuntimeRecord) -> Result<()> {
        self.watermark = self.watermark.max(addition_watermark(record));
        let Some(text) = text_of(record, self.field) else {
            return Ok(());
        };
        let document = document(self.config, &text)?;
        self.documents += 1;
        self.tokens += i64::from(document.tokens);
        for (term, term_frequency) in document.terms {
            *self.frequencies.entry(term.clone()).or_default() += 1;
            self.postings.insert(
                (term, record.reference.clone()),
                Some(Posting {
                    record: record.reference.clone(),
                    term_frequency,
                    document_tokens: document.tokens,
                    valid_from: record.valid_from,
                    valid_to: record.valid_to,
                }),
            );
        }
        Ok(())
    }

    /// Writes the gathered changes: postings, the document frequency of every
    /// term they touch, and the index totals.
    pub(crate) fn write(
        self,
        reader: &(impl AccessRead + ?Sized),
        put: &mut dyn FnMut(Space, Vec<u8>, Option<Vec<u8>>) -> Result<()>,
    ) -> Result<()> {
        for ((term, reference), posting) in &self.postings {
            let key = keyspaces::runtime_term_posting_key(self.scope, self.index, term, reference);
            put(
                keyspaces::RUNTIME_TERM_POSTINGS,
                key,
                posting.as_ref().map(serde_json::to_vec).transpose()?,
            )?;
        }
        for (term, delta) in &self.frequencies {
            if *delta == 0 {
                continue;
            }
            let key = keyspaces::runtime_term_dictionary_key(self.scope, self.index, term);
            let current: u64 =
                get_json(reader, keyspaces::RUNTIME_TERM_DICTIONARY, &key)?.unwrap_or(0);
            let next = apply(current, *delta, "BM25 document frequency")?;
            put(
                keyspaces::RUNTIME_TERM_DICTIONARY,
                key,
                (next > 0).then(|| next.to_string().into_bytes()),
            )?;
        }
        if self.documents != 0 || self.tokens != 0 || self.watermark != 0 {
            let key = keyspaces::runtime_term_statistics_key(self.scope, self.index);
            let current: IndexStatistics =
                get_json(reader, keyspaces::RUNTIME_TERM_STATISTICS, &key)?.unwrap_or_default();
            let next = IndexStatistics {
                documents: apply(current.documents, self.documents, "BM25 document count")?,
                tokens: apply(current.tokens, self.tokens, "BM25 token count")?,
                watermark: current.watermark.max(self.watermark),
            };
            put(
                keyspaces::RUNTIME_TERM_STATISTICS,
                key,
                Some(serde_json::to_vec(&next)?),
            )?;
        }
        Ok(())
    }
}

fn apply(current: u64, delta: i64, what: &str) -> Result<u64> {
    let next = i128::from(current) + i128::from(delta);
    u64::try_from(next)
        .map_err(|_| Error::IndexConstraint(format!("{what} would leave its range ({next})")))
}

/// Removes every posting, frequency and total of one index.
pub(crate) fn clear(
    transaction: &mut dyn StorageTransaction,
    scope: &ScopeId,
    index: &ProjectionId,
) -> Result<()> {
    for (space, prefix) in [
        (
            keyspaces::RUNTIME_TERM_POSTINGS,
            keyspaces::runtime_term_posting_prefix(scope, index, None),
        ),
        (
            keyspaces::RUNTIME_TERM_DICTIONARY,
            keyspaces::runtime_term_dictionary_prefix(scope, index),
        ),
    ] {
        let entries = scan_space_bounded(&*transaction, space, &prefix, MAX_INDEX_POSTINGS + 1)?;
        if entries.len() > MAX_INDEX_POSTINGS {
            return Err(Error::IndexConstraint(format!(
                "index {index} exceeds {MAX_INDEX_POSTINGS} removable postings"
            )));
        }
        for (key, _) in entries {
            transaction.delete(checked_key(space, &key)?)?;
        }
    }
    transaction.delete(checked_key(
        keyspaces::RUNTIME_TERM_STATISTICS,
        &keyspaces::runtime_term_statistics_key(scope, index),
    )?)
}

/// One scored document of a search.
#[derive(Debug, Clone, PartialEq)]
pub struct LexicalHit {
    pub record: RuntimeRef,
    pub score: f64,
    pub matched_terms: Vec<String>,
}

/// A search over current postings, or the reason it cannot be answered from
/// them.
#[derive(Debug, Clone, PartialEq)]
pub enum LexicalSearch {
    Hits {
        hits: Vec<LexicalHit>,
        /// Documents scored before the top-k cut.
        candidates: u64,
        /// Index keys read: totals, frequencies and postings.
        keys_read: u64,
        documents: u64,
    },
    /// The request needs history the current postings do not hold.
    NotApplicable(String),
}

/// Exact BM25 top-k over the current postings of one index. `max_keys`
/// bounds the keys read; reaching it refuses the search rather than return a
/// partial ranking.
#[allow(clippy::too_many_arguments)]
pub(crate) fn search(
    reader: &(impl AccessRead + ?Sized),
    scope: &ScopeId,
    index: &ProjectionId,
    config: &Bm25Config,
    query: &str,
    valid_at: u64,
    top_k: usize,
    max_keys: u64,
) -> Result<LexicalSearch> {
    if top_k == 0 {
        return Err(Error::IndexConstraint(
            "BM25 top_k must be greater than zero".into(),
        ));
    }
    let mut keys_read = 1_u64;
    let statistics: IndexStatistics = get_json(
        reader,
        keyspaces::RUNTIME_TERM_STATISTICS,
        &keyspaces::runtime_term_statistics_key(scope, index),
    )?
    .unwrap_or_default();
    if valid_at < statistics.watermark {
        return Ok(LexicalSearch::NotApplicable(format!(
            "valid time {valid_at} precedes the watermark {} of index {index}",
            statistics.watermark
        )));
    }
    let terms = analyze(config, query).into_iter().collect::<BTreeSet<_>>();
    if terms.is_empty() || statistics.documents == 0 {
        return Ok(LexicalSearch::Hits {
            hits: Vec::new(),
            candidates: 0,
            keys_read,
            documents: statistics.documents,
        });
    }
    let documents = statistics.documents as f64;
    let average = statistics.tokens as f64 / documents;
    let mut scores = BTreeMap::<RuntimeRef, (f64, Vec<String>)>::new();
    for term in terms {
        let frequency: u64 = get_json(
            reader,
            keyspaces::RUNTIME_TERM_DICTIONARY,
            &keyspaces::runtime_term_dictionary_key(scope, index, &term),
        )?
        .unwrap_or(0);
        keys_read += 1;
        if frequency == 0 {
            continue;
        }
        let remaining = max_keys.saturating_sub(keys_read);
        if frequency > remaining {
            return Err(Error::IndexConstraint(format!(
                "BM25 search over index {index} would read more than {max_keys} keys (term {term:?} has {frequency} postings)"
            )));
        }
        let prefix = keyspaces::runtime_term_posting_prefix(scope, index, Some(&term));
        let limit = usize::try_from(frequency + 1)
            .map_err(|_| Error::IndexConstraint("posting count exceeds usize".into()))?;
        let postings =
            scan_space_bounded(reader, keyspaces::RUNTIME_TERM_POSTINGS, &prefix, limit)?;
        keys_read += postings.len() as u64;
        if postings.len() as u64 != frequency {
            return Err(Error::IndexConstraint(format!(
                "index {index} term {term:?} has {} postings where its frequency is {frequency}",
                postings.len()
            )));
        }
        for (_, bytes) in postings {
            let posting: Posting = serde_json::from_slice(&bytes)?;
            if posting.valid_from > valid_at
                || posting
                    .valid_to
                    .is_some_and(|valid_to| valid_to <= valid_at)
            {
                continue;
            }
            let contribution = bm25_contribution(
                config,
                documents,
                frequency as f64,
                f64::from(posting.term_frequency),
                f64::from(posting.document_tokens),
                average,
            );
            let score = scores.entry(posting.record).or_default();
            score.0 += contribution;
            score.1.push(term.clone());
        }
    }
    let candidates = scores.len() as u64;
    let mut hits = scores
        .into_iter()
        .map(|(record, (score, matched_terms))| LexicalHit {
            record,
            score,
            matched_terms,
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.record.cmp(&right.record))
    });
    hits.truncate(top_k);
    Ok(LexicalSearch::Hits {
        hits,
        candidates,
        keys_read,
        documents: statistics.documents,
    })
}

/// The identities a scalar index lookup selected, or why current entries
/// cannot answer it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexLookup {
    Records {
        references: Vec<RuntimeRef>,
        /// Index entries read.
        keys_read: u64,
    },
    NotApplicable(String),
}
