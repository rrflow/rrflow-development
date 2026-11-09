//! Feature-gated C-06j measurements over rrflowKV's real segment-v6 encoder
//! and immutable-page reader.
//!
//! Nothing in this module owns production storage policy. It verifies the
//! canonical persisted filter, integrated adaptive-LZ4 behavior, and selected
//! scan-resistant cache policy, then keeps unintegrated codec, cache, and
//! value-placement candidates reproducible before another candidate changes
//! durable bytes.

#[cfg(test)]
use super::format::RowGroupFilter;
#[cfg(test)]
use super::format::FOOTER_BYTES;
use super::format::{
    encode, parse_metadata, sha256_hex, PageKind, FILTER_BITS_PER_KEY, FILTER_HASH_FUNCTIONS,
};
use crate::{
    Database, DatabaseOptions, Durability, Error, Memtable, Mutation, PageCachePolicy,
    ProjectedReadBudget, ProjectedReadProjection, ProjectedReadRange, ProjectedReadRequest, Result,
    SegmentCompressionPolicy, SegmentRowGroupBudget, WriteBatch, SEGMENT_FORMAT_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;

pub const PHYSICAL_POLICY_EVIDENCE_VERSION: u16 = 4;

const FAMILY_PREFIXES: [(&str, &[u8]); 8] = [
    ("audit", b"audit/"),
    ("edge-in", b"edge/in/"),
    ("edge-out", b"edge/out/"),
    ("record", b"record/"),
    ("runtime", b"runtime/"),
    ("scalar", b"scalar/"),
    ("term", b"term/"),
    ("vector", b"vector/"),
];
const FILTER_FALSE_POSITIVE_LIMIT_PPM: u64 = 20_000;
const ADAPTIVE_CODEC_MINIMUM_SAVINGS_BPS: u64 = 1_250;
const ADAPTIVE_CODEC_FRAME_BYTES: usize = 8;
const VALUE_POINTER_BYTES: usize = 16;
const MAX_RECORDS_PER_FAMILY: usize = 16_384;
const MAX_VERSIONS_PER_KEY: usize = 4;
const MAX_VALUE_BYTES: usize = 1024 * 1024;
const MAX_POINT_MISSES: usize = 1_000_000;
const MAX_CACHE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalPolicyConfig {
    pub seed: u64,
    pub records_per_family: usize,
    pub versions_per_key: usize,
    pub value_bytes: usize,
    pub point_misses: usize,
    pub cache_bytes: usize,
}

impl PhysicalPolicyConfig {
    pub fn validate(self) -> Result<Self> {
        if self.records_per_family == 0
            || self.records_per_family > MAX_RECORDS_PER_FAMILY
            || self.versions_per_key == 0
            || self.versions_per_key > MAX_VERSIONS_PER_KEY
            || self.value_bytes == 0
            || self.value_bytes > MAX_VALUE_BYTES
            || self.point_misses == 0
            || self.point_misses > MAX_POINT_MISSES
            || self.cache_bytes == 0
            || self.cache_bytes > MAX_CACHE_BYTES
        {
            return Err(Error::InvalidConfiguration(format!(
                "physical-policy lab requires records_per_family=1..={MAX_RECORDS_PER_FAMILY}, versions_per_key=1..={MAX_VERSIONS_PER_KEY}, value_bytes=1..={MAX_VALUE_BYTES}, point_misses=1..={MAX_POINT_MISSES}, and cache_bytes=1..={MAX_CACHE_BYTES}"
            )));
        }
        let operations = self
            .records_per_family
            .checked_mul(FAMILY_PREFIXES.len())
            .and_then(|value| value.checked_mul(self.versions_per_key))
            .ok_or_else(|| {
                Error::InvalidConfiguration("physical-policy operation count overflow".into())
            })?;
        let estimated_payload = operations
            .checked_mul(self.value_bytes.saturating_add(64))
            .ok_or_else(|| {
                Error::InvalidConfiguration("physical-policy payload estimate overflow".into())
            })?;
        if estimated_payload > crate::WAL_MAX_PAYLOAD_BYTES {
            return Err(Error::InvalidConfiguration(format!(
                "physical-policy corpus estimate {estimated_payload} exceeds one bounded WAL payload of {} bytes",
                crate::WAL_MAX_PAYLOAD_BYTES
            )));
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDecision {
    pub candidate: String,
    pub decision: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodecObservation {
    pub codec: String,
    pub pages: u64,
    pub logical_bytes: u64,
    pub codec_bytes: u64,
    pub adaptive_stored_bytes: u64,
    pub adaptive_selected_pages: u64,
    pub savings_basis_points: u64,
    pub encode_nanoseconds: u64,
    pub decode_nanoseconds: u64,
    pub round_trip_exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterObservation {
    pub format: String,
    pub filters: u64,
    pub unique_keys: u64,
    pub serialized_bytes: u64,
    pub bits_per_key: u64,
    pub hash_functions: u64,
    pub member_queries: u64,
    pub member_false_negatives: u64,
    pub absent_queries: u64,
    pub false_positives: u64,
    pub false_positive_parts_per_million: u64,
    pub passes_policy_threshold: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachePolicyObservation {
    pub policy: String,
    pub capacity_bytes: u64,
    pub peak_resident_bytes: u64,
    pub final_resident_bytes: u64,
    pub hits: u64,
    pub misses: u64,
    pub admissions: u64,
    pub evictions: u64,
    pub post_scan_hot_hits: u64,
    pub post_scan_hot_requests: u64,
    pub semantic_identity_exact: bool,
    pub exact_capacity_respected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValuePlacementObservation {
    pub evidence_kind: String,
    pub non_null_versions: u64,
    pub inline_value_bytes: u64,
    pub latest_live_value_bytes: u64,
    pub obsolete_value_bytes: u64,
    pub separated_pointer_bytes: u64,
    pub append_only_value_log_bytes: u64,
    pub modeled_compaction_bytes_avoided: u64,
    pub production_retainable: bool,
    pub missing_proofs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyObservation {
    pub family: String,
    pub rows: u64,
    pub page_count: u64,
    pub logical_page_bytes: u64,
    pub lz4_adaptive_bytes: u64,
    pub zstd_adaptive_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReopenedPointMissObservation {
    pub integrated_rrflowkv: bool,
    pub open_none_policy_segment_count: u64,
    pub open_adaptive_lz4_policy_segment_count: u64,
    pub open_raw_page_count: u64,
    pub open_compressed_page_count: u64,
    pub open_stored_page_bytes: u64,
    pub open_logical_page_bytes: u64,
    pub open_persisted_filter_count: u64,
    pub open_persisted_filter_bytes: u64,
    pub open_semantic_page_operations: u64,
    pub open_semantic_page_bytes: u64,
    pub misses_verified: u64,
    pub hit_samples_verified: u64,
    pub elapsed_nanoseconds: u64,
    pub segment_physical_bytes: u64,
    pub filter_checks: u64,
    pub filter_negatives: u64,
    pub page_cache_hits: u64,
    pub page_cache_misses: u64,
    pub page_loads: u64,
    pub bytes_read: u64,
    pub bytes_decoded: u64,
    pub bytes_decompressed: u64,
    pub final_cache_resident_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegratedCachePolicyObservation {
    pub policy: String,
    pub protected_capacity_basis_points: u64,
    pub snapshot_sequence: u64,
    pub manifest_digest: String,
    pub semantic_digest: String,
    pub projected_row_digest: String,
    pub hot_family_count: u64,
    pub hot_requests_per_pass: u64,
    pub projected_rows: u64,
    pub capacity_bytes: u64,
    pub final_resident_bytes: u64,
    pub probationary_resident_bytes: u64,
    pub protected_resident_bytes: u64,
    pub final_entries: u64,
    pub probationary_entries: u64,
    pub protected_entries: u64,
    pub warm_hits: u64,
    pub warm_misses: u64,
    pub warm_loads: u64,
    pub scan_hits: u64,
    pub scan_misses: u64,
    pub scan_loads: u64,
    pub post_scan_hot_hits: u64,
    pub post_scan_hot_misses: u64,
    pub post_scan_hot_loads: u64,
    pub admissions: u64,
    pub admission_rejections: u64,
    pub promotions: u64,
    pub demotions: u64,
    pub same_scope_hits: u64,
    pub evictions: u64,
    pub duplicate_loads: u64,
    pub bytes_read: u64,
    pub bytes_decoded: u64,
    pub bytes_decompressed: u64,
    pub semantic_identity_exact: bool,
    pub exact_capacity_respected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalPolicyTrial {
    pub evidence_version: u16,
    pub evidence_scope: String,
    pub config: PhysicalPolicyConfig,
    pub corpus_digest: String,
    pub operation_count: u64,
    pub key_count: u64,
    pub row_group_count: u64,
    pub page_count: u64,
    pub segment_format_version: u16,
    pub segment_physical_bytes: u64,
    pub logical_page_bytes: u64,
    pub elapsed_nanoseconds: u64,
    pub peak_rss_bytes: Option<u64>,
    pub codecs: Vec<CodecObservation>,
    pub persisted_row_group_filter: FilterObservation,
    pub cache_candidates: Vec<CachePolicyObservation>,
    pub value_placement_candidate: ValuePlacementObservation,
    pub families: Vec<FamilyObservation>,
    pub reopened_point_miss: ReopenedPointMissObservation,
    pub integrated_cache_policies: Vec<IntegratedCachePolicyObservation>,
    pub decisions: Vec<CandidateDecision>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone)]
struct Corpus {
    mutations: Vec<Mutation>,
    expected_latest: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    digest: String,
}

#[derive(Debug, Clone)]
struct PageSample {
    id: usize,
    row_group: usize,
    row_count: u64,
    family: String,
    kind: &'static str,
    bytes: Vec<u8>,
}

#[derive(Debug, Default)]
struct CodecAccumulator {
    pages: u64,
    logical_bytes: u64,
    codec_bytes: u64,
    adaptive_stored_bytes: u64,
    selected_pages: u64,
    encode_nanoseconds: u64,
    decode_nanoseconds: u64,
}

#[derive(Debug, Clone, Copy)]
struct CacheAccess {
    page_id: usize,
    bytes: usize,
    post_scan_hot: bool,
}

#[derive(Debug, Clone, Copy)]
struct CacheEntry {
    bytes: usize,
    stamp: u64,
}

#[derive(Debug)]
struct ExactLru {
    capacity: usize,
    resident: usize,
    peak: usize,
    clock: u64,
    entries: BTreeMap<usize, CacheEntry>,
    observation: CachePolicyObservation,
}

#[derive(Debug)]
struct SegmentedLru {
    capacity: usize,
    protected_capacity: usize,
    probation_capacity: usize,
    protected_resident: usize,
    probation_resident: usize,
    peak: usize,
    clock: u64,
    protected: BTreeMap<usize, CacheEntry>,
    probation: BTreeMap<usize, CacheEntry>,
    observation: CachePolicyObservation,
}

pub fn run_physical_policy_trial(
    root: &Path,
    config: PhysicalPolicyConfig,
) -> Result<PhysicalPolicyTrial> {
    let config = config.validate()?;
    let started = Instant::now();
    let corpus = build_corpus(config)?;
    let operation_count = corpus.mutations.len() as u64;
    let mut table = Memtable::default();
    let first_sequence = 1;
    let last_sequence = operation_count;
    table.apply_owned_write_batch(
        WriteBatch::new(corpus.mutations.clone())?,
        first_sequence,
        last_sequence,
    )?;
    let encoded = encode(
        &table,
        integrated_row_group_budget(config)?,
        SegmentCompressionPolicy::None,
    )?;
    let segment_physical_bytes = encoded.bytes.len() as u64;
    let mut cursor = Cursor::new(&encoded.bytes);
    let metadata = parse_metadata(
        &mut cursor,
        segment_physical_bytes,
        encoded.checksum.clone(),
    )?;
    let pages = extract_pages(&encoded.bytes, &metadata.row_groups)?;
    let logical_page_bytes = pages.iter().try_fold(0u64, |total, page| {
        total
            .checked_add(page.bytes.len() as u64)
            .ok_or_else(|| Error::InvalidSegment("physical-policy page byte total overflow".into()))
    })?;

    let codecs = observe_codecs(&pages)?;
    let persisted_row_group_filter = observe_filters(&encoded.bytes, &metadata.row_groups, config)?;
    let cache_candidates = observe_caches(&pages, config.cache_bytes)?;
    let value_placement_candidate = observe_value_placement(&table)?;
    let families = observe_families(&pages)?;
    let reopened_point_miss = observe_reopened_database(root, config, &corpus)?;
    validate_persisted_filter_integration(
        metadata.row_groups.len(),
        &persisted_row_group_filter,
        &reopened_point_miss,
    )?;
    let integrated_cache_policies = observe_integrated_cache_policies(root, config, &corpus)?;
    validate_integrated_cache_policies(&integrated_cache_policies)?;
    let decisions = decide_candidates(
        &codecs,
        &persisted_row_group_filter,
        &cache_candidates,
        &integrated_cache_policies,
        &value_placement_candidate,
    );

    Ok(PhysicalPolicyTrial {
        evidence_version: PHYSICAL_POLICY_EVIDENCE_VERSION,
        evidence_scope: "c06j-scan-resistant-cache-integration".into(),
        config,
        corpus_digest: corpus.digest,
        operation_count,
        key_count: table.key_count() as u64,
        row_group_count: metadata.row_groups.len() as u64,
        page_count: pages.len() as u64,
        segment_format_version: SEGMENT_FORMAT_VERSION,
        segment_physical_bytes,
        logical_page_bytes,
        elapsed_nanoseconds: started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
        peak_rss_bytes: peak_rss_bytes(),
        codecs,
        persisted_row_group_filter,
        cache_candidates,
        value_placement_candidate,
        families,
        reopened_point_miss,
        integrated_cache_policies,
        decisions,
        limitations: vec![
            "adaptive LZ4 and the exact-byte scan-resistant page cache are integrated; Zstandard and trace-only cache candidates remain unintegrated".into(),
            "value placement is a byte model only and lacks pointer publication, recovery, snapshot, corruption, range-read, and garbage-collection proof".into(),
            "the integrated cache comparison is single-threaded and records duplicate loads without proving load coalescing, lock scalability, cancellation fairness, or sustained concurrency".into(),
            "one machine and one deterministic corpus cannot establish release latency, every-workload policy, network behavior, or competitor superiority".into(),
            "operating-system device cache and competing host load are recorded but not controlled by this executable".into(),
        ],
    })
}

fn build_corpus(config: PhysicalPolicyConfig) -> Result<Corpus> {
    let mut mutations = Vec::with_capacity(
        config.records_per_family * FAMILY_PREFIXES.len() * config.versions_per_key,
    );
    let mut expected_latest = BTreeMap::new();
    let mut digest_input = Vec::new();
    for (family_index, (_, prefix)) in FAMILY_PREFIXES.iter().enumerate() {
        for ordinal in 0..config.records_per_family {
            let key = corpus_key(prefix, ordinal, false);
            for version in 0..config.versions_per_key {
                let delete = version + 1 == config.versions_per_key && ordinal % 97 == 0;
                let mutation = if delete {
                    expected_latest.insert(key.clone(), None);
                    Mutation::Delete { key: key.clone() }
                } else {
                    let value = corpus_value(
                        config.seed,
                        family_index,
                        ordinal,
                        version,
                        config.value_bytes,
                    );
                    expected_latest.insert(key.clone(), Some(value.clone()));
                    Mutation::Put {
                        key: key.clone(),
                        value,
                    }
                };
                digest_mutation(&mut digest_input, &mutation)?;
                mutations.push(mutation);
            }
        }
    }
    Ok(Corpus {
        mutations,
        expected_latest,
        digest: sha256_hex(&digest_input),
    })
}

fn corpus_key(prefix: &[u8], ordinal: usize, missing: bool) -> Vec<u8> {
    let numeric = ordinal
        .saturating_mul(2)
        .saturating_add(usize::from(missing));
    let mut key = Vec::with_capacity(prefix.len() + 8);
    key.extend_from_slice(prefix);
    key.extend_from_slice(format!("{numeric:08x}").as_bytes());
    key
}

fn corpus_value(seed: u64, family: usize, ordinal: usize, version: usize, bytes: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes);
    let structured =
        format!("family={family}|record={ordinal:08x}|version={version}|estate=rrflow|");
    let mut state = seed
        ^ (family as u64).rotate_left(11)
        ^ (ordinal as u64).rotate_left(29)
        ^ (version as u64).rotate_left(47);
    while output.len() < bytes {
        match family {
            0 | 3 => output.extend_from_slice(structured.as_bytes()),
            1 | 2 => output.extend_from_slice(b"from~edge~to|valid_from|valid_to|"),
            4 => output.extend_from_slice(b"state=ready|branch=open|lease=held|"),
            5 => output.extend_from_slice(&(ordinal as u64).to_be_bytes()),
            6 => {
                output.extend_from_slice(&(ordinal as u32).to_le_bytes());
                output.extend_from_slice(&(version as u32 + 1).to_le_bytes());
            }
            7 => {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                output.extend_from_slice(&state.to_le_bytes());
            }
            _ => unreachable!(),
        }
    }
    output.truncate(bytes);
    output
}

fn digest_mutation(output: &mut Vec<u8>, mutation: &Mutation) -> Result<()> {
    let (tag, key, value) = match mutation {
        Mutation::Put { key, value } => (1u8, key.as_slice(), Some(value.as_slice())),
        Mutation::Delete { key } => (2u8, key.as_slice(), None),
    };
    output.push(tag);
    output.extend_from_slice(
        &u64::try_from(key.len())
            .map_err(|_| Error::InvalidConfiguration("corpus key length exceeds u64".into()))?
            .to_le_bytes(),
    );
    output.extend_from_slice(key);
    if let Some(value) = value {
        output.extend_from_slice(
            &u64::try_from(value.len())
                .map_err(|_| Error::InvalidConfiguration("corpus value length exceeds u64".into()))?
                .to_le_bytes(),
        );
        output.extend_from_slice(value);
    } else {
        output.extend_from_slice(&0u64.to_le_bytes());
    }
    Ok(())
}

fn extract_pages(
    encoded: &[u8],
    row_groups: &[super::format::RowGroupDescriptor],
) -> Result<Vec<PageSample>> {
    let mut pages = Vec::with_capacity(row_groups.len() * super::format::PAGES_PER_ROW_GROUP);
    for (row_group, group) in row_groups.iter().enumerate() {
        let family = family_for_range(&group.first_key, &group.last_key);
        for descriptor in &group.pages {
            let start = usize::try_from(descriptor.offset)
                .map_err(|_| Error::InvalidSegment("page offset exceeds usize".into()))?;
            let end = start
                .checked_add(descriptor.physical_bytes)
                .ok_or_else(|| Error::InvalidSegment("page range overflow".into()))?;
            let bytes = encoded
                .get(start..end)
                .ok_or_else(|| Error::InvalidSegment("page range escaped encoded segment".into()))?
                .to_vec();
            if descriptor.physical_bytes != descriptor.logical_bytes {
                return Err(Error::InvalidSegment(
                    "the none-policy segment-v6 candidate corpus unexpectedly contained compression"
                        .into(),
                ));
            }
            pages.push(PageSample {
                id: row_group * super::format::PAGES_PER_ROW_GROUP + descriptor.kind as usize,
                row_group,
                row_count: u64::from(group.row_count),
                family: family.clone(),
                kind: page_kind_name(descriptor.kind),
                bytes,
            });
        }
    }
    Ok(pages)
}

fn page_kind_name(kind: PageKind) -> &'static str {
    match kind {
        PageKind::KeyOffsets => "key-offsets",
        PageKind::KeyData => "key-data",
        PageKind::SequenceValues => "sequence-values",
        PageKind::ValueValidity => "value-validity",
        PageKind::ValueOffsets => "value-offsets",
        PageKind::ValueData => "value-data",
    }
}

fn family_for_range(first: &[u8], last: &[u8]) -> String {
    let first_family = family_name(first);
    let last_family = family_name(last);
    if first_family == last_family {
        first_family.into()
    } else {
        "mixed".into()
    }
}

fn family_name(key: &[u8]) -> &'static str {
    FAMILY_PREFIXES
        .iter()
        .find_map(|(name, prefix)| key.starts_with(prefix).then_some(*name))
        .unwrap_or("unknown")
}

fn observe_codecs(pages: &[PageSample]) -> Result<Vec<CodecObservation>> {
    ["none", "lz4", "zstd"]
        .into_iter()
        .map(|codec| observe_codec(codec, pages))
        .collect()
}

fn observe_codec(codec: &str, pages: &[PageSample]) -> Result<CodecObservation> {
    let mut accumulator = CodecAccumulator::default();
    for page in pages {
        accumulator.pages += 1;
        accumulator.logical_bytes = accumulator
            .logical_bytes
            .checked_add(page.bytes.len() as u64)
            .ok_or_else(|| Error::InvalidSegment("codec logical byte total overflow".into()))?;
        let encode_started = Instant::now();
        let encoded = match codec {
            "none" => page.bytes.clone(),
            "lz4" => lz4_flex::block::compress_prepend_size(&page.bytes),
            "zstd" => zstd::bulk::compress(&page.bytes, 1)
                .map_err(|error| Error::InvalidSegment(format!("zstd encode failed: {error}")))?,
            _ => {
                return Err(Error::InvalidConfiguration(format!(
                    "unknown lab codec {codec}"
                )))
            }
        };
        accumulator.encode_nanoseconds = accumulator.encode_nanoseconds.saturating_add(
            encode_started
                .elapsed()
                .as_nanos()
                .try_into()
                .unwrap_or(u64::MAX),
        );
        black_box(&encoded);
        let decode_started = Instant::now();
        let decoded = match codec {
            "none" => encoded.clone(),
            "lz4" => lz4_flex::block::decompress_size_prepended(&encoded).map_err(|error| {
                Error::InvalidSegment(format!("lz4 candidate decode failed: {error}"))
            })?,
            "zstd" => zstd::bulk::decompress(&encoded, page.bytes.len()).map_err(|error| {
                Error::InvalidSegment(format!("zstd candidate decode failed: {error}"))
            })?,
            _ => unreachable!(),
        };
        accumulator.decode_nanoseconds = accumulator.decode_nanoseconds.saturating_add(
            decode_started
                .elapsed()
                .as_nanos()
                .try_into()
                .unwrap_or(u64::MAX),
        );
        black_box(&decoded);
        if decoded != page.bytes {
            return Err(Error::InvalidSegment(format!(
                "{codec} candidate changed {} page {}",
                page.family, page.kind
            )));
        }
        accumulator.codec_bytes = accumulator
            .codec_bytes
            .checked_add(encoded.len() as u64)
            .ok_or_else(|| Error::InvalidSegment("codec byte total overflow".into()))?;
        let adaptive = if codec != "none" && codec_saves_enough(page.bytes.len(), encoded.len()) {
            accumulator.selected_pages += 1;
            encoded.len().saturating_add(ADAPTIVE_CODEC_FRAME_BYTES)
        } else {
            page.bytes.len()
        };
        accumulator.adaptive_stored_bytes = accumulator
            .adaptive_stored_bytes
            .checked_add(adaptive as u64)
            .ok_or_else(|| Error::InvalidSegment("adaptive codec byte total overflow".into()))?;
    }
    let savings_basis_points =
        savings_basis_points(accumulator.logical_bytes, accumulator.adaptive_stored_bytes);
    Ok(CodecObservation {
        codec: codec.into(),
        pages: accumulator.pages,
        logical_bytes: accumulator.logical_bytes,
        codec_bytes: accumulator.codec_bytes,
        adaptive_stored_bytes: accumulator.adaptive_stored_bytes,
        adaptive_selected_pages: accumulator.selected_pages,
        savings_basis_points,
        encode_nanoseconds: accumulator.encode_nanoseconds,
        decode_nanoseconds: accumulator.decode_nanoseconds,
        round_trip_exact: true,
    })
}

fn codec_saves_enough(logical: usize, encoded: usize) -> bool {
    let Some(stored) = encoded.checked_add(ADAPTIVE_CODEC_FRAME_BYTES) else {
        return false;
    };
    stored < logical
        && savings_basis_points(logical as u64, stored as u64) >= ADAPTIVE_CODEC_MINIMUM_SAVINGS_BPS
}

fn savings_basis_points(logical: u64, stored: u64) -> u64 {
    if logical == 0 || stored >= logical {
        0
    } else {
        logical.saturating_sub(stored).saturating_mul(10_000) / logical
    }
}

fn observe_filters(
    encoded: &[u8],
    row_groups: &[super::format::RowGroupDescriptor],
    config: PhysicalPolicyConfig,
) -> Result<FilterObservation> {
    let mut filters = 0u64;
    let mut unique_keys = 0u64;
    let mut serialized_bytes = 0u64;
    let mut member_queries = 0u64;
    let mut member_false_negatives = 0u64;
    let mut absent_queries = 0u64;
    let mut false_positives = 0u64;
    for group in row_groups {
        let keys = keys_for_group(encoded, group)?;
        if keys.len() as u32 != group.unique_key_count {
            return Err(Error::InvalidSegment(
                "persisted filter unique-key count disagrees with key pages".into(),
            ));
        }
        let filter = &group.filter;
        filters += 1;
        unique_keys = unique_keys.saturating_add(keys.len() as u64);
        serialized_bytes = serialized_bytes.saturating_add(filter.byte_len() as u64);
        for key in &keys {
            member_queries += 1;
            if !filter.may_contain(key) {
                member_false_negatives += 1;
            }
        }
        let probes = config.point_misses.div_ceil(row_groups.len()).max(1);
        for probe in 0..probes {
            let mut absent = keys[probe % keys.len()].clone();
            absent.extend_from_slice(b"/absent/");
            absent.extend_from_slice(&(probe as u64).to_le_bytes());
            if keys.binary_search(&absent).is_ok() {
                return Err(Error::InvalidSegment(
                    "generated filter probe unexpectedly exists".into(),
                ));
            }
            absent_queries += 1;
            if filter.may_contain(&absent) {
                false_positives += 1;
            }
        }
    }
    let false_positive_parts_per_million = false_positives
        .saturating_mul(1_000_000)
        .checked_div(absent_queries)
        .unwrap_or(0);
    Ok(FilterObservation {
        format: "segment-v6-row-group-bloom-v1".into(),
        filters,
        unique_keys,
        serialized_bytes,
        bits_per_key: FILTER_BITS_PER_KEY as u64,
        hash_functions: FILTER_HASH_FUNCTIONS as u64,
        member_queries,
        member_false_negatives,
        absent_queries,
        false_positives,
        false_positive_parts_per_million,
        passes_policy_threshold: member_false_negatives == 0
            && false_positive_parts_per_million <= FILTER_FALSE_POSITIVE_LIMIT_PPM,
    })
}

fn validate_persisted_filter_integration(
    row_group_count: usize,
    filter: &FilterObservation,
    reopened: &ReopenedPointMissObservation,
) -> Result<()> {
    let row_group_count = u64::try_from(row_group_count)
        .map_err(|_| Error::InvalidSegment("row-group count exceeds u64".into()))?;
    require_integration(
        filter.filters == row_group_count,
        "candidate and segment row-group counts differ",
    )?;
    require_integration(
        filter.member_false_negatives == 0,
        "persisted filter produced a member false negative",
    )?;
    require_integration(
        filter.passes_policy_threshold,
        "persisted filter exceeded its accepted false-positive threshold",
    )?;
    require_integration(
        reopened.open_persisted_filter_count == filter.filters,
        "reopen filter count differs from the candidate observation",
    )?;
    require_integration(
        reopened.open_persisted_filter_bytes == filter.serialized_bytes,
        "reopen filter bytes differ from the candidate observation",
    )?;
    require_integration(
        reopened.open_semantic_page_operations == 0 && reopened.open_semantic_page_bytes == 0,
        "metadata-only reopen performed semantic page work",
    )?;
    require_integration(
        reopened.open_none_policy_segment_count == 0
            && reopened.open_adaptive_lz4_policy_segment_count > 0,
        "reopen did not authenticate only adaptive-LZ4 segments",
    )?;
    require_integration(
        reopened.open_raw_page_count > 0 && reopened.open_compressed_page_count > 0,
        "corpus did not exercise both raw and compressed production pages",
    )?;
    require_integration(
        reopened.open_stored_page_bytes < reopened.open_logical_page_bytes,
        "adaptive production pages did not reduce stored bytes",
    )?;
    require_integration(
        reopened.bytes_decompressed > 0,
        "present-key reads did not exercise production decompression",
    )?;
    require_integration(
        reopened.filter_checks > 0 && reopened.filter_negatives > 0,
        "point-miss workload did not exercise persisted filter pruning",
    )?;
    require_integration(
        reopened.page_loads < reopened.filter_checks,
        "point-miss workload did not avoid semantic page loads",
    )?;
    Ok(())
}

fn require_integration(condition: bool, invariant: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Error::InvalidSegment(format!(
            "adaptive page-compression integration evidence failed: {invariant}"
        )))
    }
}

fn keys_for_group(
    encoded: &[u8],
    group: &super::format::RowGroupDescriptor,
) -> Result<Vec<Vec<u8>>> {
    let offsets_page = group.page(PageKind::KeyOffsets);
    let data_page = group.page(PageKind::KeyData);
    let offsets = page_slice(encoded, offsets_page)?;
    let data = page_slice(encoded, data_page)?;
    let expected_offsets = (group.row_count as usize)
        .checked_add(1)
        .and_then(|count| count.checked_mul(std::mem::size_of::<i64>()))
        .ok_or_else(|| Error::InvalidSegment("key-offset shape overflow".into()))?;
    if offsets.len() != expected_offsets {
        return Err(Error::InvalidSegment(
            "candidate filter saw an invalid key-offset page".into(),
        ));
    }
    let mut keys = BTreeSet::new();
    for row in 0..group.row_count as usize {
        let start = decode_offset(offsets, row)?;
        let end = decode_offset(offsets, row + 1)?;
        if start > end || end > data.len() {
            return Err(Error::InvalidSegment(
                "candidate filter key offsets escaped key data".into(),
            ));
        }
        keys.insert(data[start..end].to_vec());
    }
    if keys.is_empty() {
        return Err(Error::InvalidSegment(
            "candidate filter received an empty row group".into(),
        ));
    }
    Ok(keys.into_iter().collect())
}

fn page_slice<'a>(
    encoded: &'a [u8],
    descriptor: &super::format::PageDescriptor,
) -> Result<&'a [u8]> {
    let start = usize::try_from(descriptor.offset)
        .map_err(|_| Error::InvalidSegment("page offset exceeds usize".into()))?;
    let end = start
        .checked_add(descriptor.physical_bytes)
        .ok_or_else(|| Error::InvalidSegment("page offset overflow".into()))?;
    encoded
        .get(start..end)
        .ok_or_else(|| Error::InvalidSegment("page slice is outside segment".into()))
}

fn decode_offset(bytes: &[u8], index: usize) -> Result<usize> {
    let start = index
        .checked_mul(std::mem::size_of::<i64>())
        .ok_or_else(|| Error::InvalidSegment("offset index overflow".into()))?;
    let end = start + std::mem::size_of::<i64>();
    let raw: [u8; 8] = bytes
        .get(start..end)
        .ok_or_else(|| Error::InvalidSegment("offset page ended early".into()))?
        .try_into()
        .expect("fixed offset width was checked");
    usize::try_from(i64::from_le_bytes(raw))
        .map_err(|_| Error::InvalidSegment("offset is negative or exceeds usize".into()))
}

fn observe_caches(pages: &[PageSample], capacity: usize) -> Result<Vec<CachePolicyObservation>> {
    let trace = cache_trace(pages)?;
    let mut lru = ExactLru::new(capacity);
    let mut segmented = SegmentedLru::new(capacity);
    for access in trace {
        lru.access(access)?;
        segmented.access(access)?;
    }
    Ok(vec![lru.finish(), segmented.finish()?])
}

fn cache_trace(pages: &[PageSample]) -> Result<Vec<CacheAccess>> {
    if pages.is_empty() {
        return Err(Error::InvalidSegment("cache trace has no pages".into()));
    }
    let hot = pages.iter().take(12).collect::<Vec<_>>();
    let mut trace = Vec::new();
    for _ in 0..8 {
        for page in &hot {
            trace.push(CacheAccess {
                page_id: page.id,
                bytes: page.bytes.len().max(1),
                post_scan_hot: false,
            });
        }
    }
    for page in pages {
        trace.push(CacheAccess {
            page_id: page.id,
            bytes: page.bytes.len().max(1),
            post_scan_hot: false,
        });
    }
    for _ in 0..8 {
        for page in &hot {
            trace.push(CacheAccess {
                page_id: page.id,
                bytes: page.bytes.len().max(1),
                post_scan_hot: true,
            });
        }
    }
    Ok(trace)
}

impl ExactLru {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            resident: 0,
            peak: 0,
            clock: 0,
            entries: BTreeMap::new(),
            observation: CachePolicyObservation {
                policy: "exact-byte-lru".into(),
                capacity_bytes: capacity as u64,
                peak_resident_bytes: 0,
                final_resident_bytes: 0,
                hits: 0,
                misses: 0,
                admissions: 0,
                evictions: 0,
                post_scan_hot_hits: 0,
                post_scan_hot_requests: 0,
                semantic_identity_exact: true,
                exact_capacity_respected: true,
            },
        }
    }

    fn access(&mut self, access: CacheAccess) -> Result<()> {
        self.clock = self.clock.saturating_add(1);
        if access.post_scan_hot {
            self.observation.post_scan_hot_requests += 1;
        }
        if let Some(entry) = self.entries.get_mut(&access.page_id) {
            if entry.bytes != access.bytes {
                return Err(Error::InvalidSegment(
                    "cache trace changed one page's physical identity".into(),
                ));
            }
            entry.stamp = self.clock;
            self.observation.hits += 1;
            if access.post_scan_hot {
                self.observation.post_scan_hot_hits += 1;
            }
            return Ok(());
        }
        self.observation.misses += 1;
        if access.bytes > self.capacity {
            return Ok(());
        }
        while self.resident.saturating_add(access.bytes) > self.capacity {
            let (&victim, entry) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.stamp)
                .ok_or_else(|| Error::InvalidSegment("LRU cannot select an eviction".into()))?;
            let bytes = entry.bytes;
            self.entries.remove(&victim);
            self.resident -= bytes;
            self.observation.evictions += 1;
        }
        self.entries.insert(
            access.page_id,
            CacheEntry {
                bytes: access.bytes,
                stamp: self.clock,
            },
        );
        self.resident += access.bytes;
        self.peak = self.peak.max(self.resident);
        self.observation.admissions += 1;
        if self.resident > self.capacity {
            return Err(Error::InvalidSegment("LRU exceeded exact capacity".into()));
        }
        Ok(())
    }

    fn finish(mut self) -> CachePolicyObservation {
        self.observation.peak_resident_bytes = self.peak as u64;
        self.observation.final_resident_bytes = self.resident as u64;
        self.observation
    }
}

impl SegmentedLru {
    fn new(capacity: usize) -> Self {
        let probation_capacity = (capacity / 5).max(1).min(capacity);
        let protected_capacity = capacity.saturating_sub(probation_capacity);
        Self {
            capacity,
            protected_capacity,
            probation_capacity,
            protected_resident: 0,
            probation_resident: 0,
            peak: 0,
            clock: 0,
            protected: BTreeMap::new(),
            probation: BTreeMap::new(),
            observation: CachePolicyObservation {
                policy: "exact-byte-segmented-lru-candidate".into(),
                capacity_bytes: capacity as u64,
                peak_resident_bytes: 0,
                final_resident_bytes: 0,
                hits: 0,
                misses: 0,
                admissions: 0,
                evictions: 0,
                post_scan_hot_hits: 0,
                post_scan_hot_requests: 0,
                semantic_identity_exact: true,
                exact_capacity_respected: true,
            },
        }
    }

    fn access(&mut self, access: CacheAccess) -> Result<()> {
        self.clock = self.clock.saturating_add(1);
        if access.post_scan_hot {
            self.observation.post_scan_hot_requests += 1;
        }
        if let Some(entry) = self.protected.get_mut(&access.page_id) {
            validate_cache_identity(*entry, access)?;
            entry.stamp = self.clock;
            self.hit(access);
            return Ok(());
        }
        if let Some(mut entry) = self.probation.remove(&access.page_id) {
            validate_cache_identity(entry, access)?;
            self.probation_resident -= entry.bytes;
            entry.stamp = self.clock;
            self.protected_resident += entry.bytes;
            self.protected.insert(access.page_id, entry);
            self.hit(access);
            self.rebalance_protected()?;
            self.check_capacity()?;
            return Ok(());
        }
        self.observation.misses += 1;
        if access.bytes > self.probation_capacity {
            return Ok(());
        }
        while self.probation_resident.saturating_add(access.bytes) > self.probation_capacity {
            self.evict_probation()?;
        }
        self.probation.insert(
            access.page_id,
            CacheEntry {
                bytes: access.bytes,
                stamp: self.clock,
            },
        );
        self.probation_resident += access.bytes;
        self.observation.admissions += 1;
        self.update_peak();
        self.check_capacity()
    }

    fn hit(&mut self, access: CacheAccess) {
        self.observation.hits += 1;
        if access.post_scan_hot {
            self.observation.post_scan_hot_hits += 1;
        }
    }

    fn rebalance_protected(&mut self) -> Result<()> {
        while self.protected_resident > self.protected_capacity {
            let (&victim, entry) = self
                .protected
                .iter()
                .min_by_key(|(_, entry)| entry.stamp)
                .ok_or_else(|| {
                    Error::InvalidSegment("segmented LRU has no protected victim".into())
                })?;
            let entry = *entry;
            self.protected.remove(&victim);
            self.protected_resident -= entry.bytes;
            while self.probation_resident.saturating_add(entry.bytes) > self.probation_capacity {
                self.evict_probation()?;
            }
            if entry.bytes <= self.probation_capacity {
                self.probation.insert(victim, entry);
                self.probation_resident += entry.bytes;
            } else {
                self.observation.evictions += 1;
            }
        }
        self.update_peak();
        Ok(())
    }

    fn evict_probation(&mut self) -> Result<()> {
        let (&victim, entry) = self
            .probation
            .iter()
            .min_by_key(|(_, entry)| entry.stamp)
            .ok_or_else(|| Error::InvalidSegment("segmented LRU has no probation victim".into()))?;
        let bytes = entry.bytes;
        self.probation.remove(&victim);
        self.probation_resident -= bytes;
        self.observation.evictions += 1;
        Ok(())
    }

    fn update_peak(&mut self) {
        self.peak = self.peak.max(
            self.protected_resident
                .saturating_add(self.probation_resident),
        );
    }

    fn check_capacity(&self) -> Result<()> {
        let resident = self
            .protected_resident
            .saturating_add(self.probation_resident);
        if resident > self.capacity
            || self.protected_resident > self.protected_capacity
            || self.probation_resident > self.probation_capacity
        {
            return Err(Error::InvalidSegment(
                "segmented LRU exceeded exact byte partition".into(),
            ));
        }
        Ok(())
    }

    fn finish(mut self) -> Result<CachePolicyObservation> {
        self.check_capacity()?;
        self.observation.peak_resident_bytes = self.peak as u64;
        self.observation.final_resident_bytes =
            self.protected_resident
                .saturating_add(self.probation_resident) as u64;
        Ok(self.observation)
    }
}

fn validate_cache_identity(entry: CacheEntry, access: CacheAccess) -> Result<()> {
    if entry.bytes != access.bytes {
        return Err(Error::InvalidSegment(
            "cache trace changed one page's physical identity".into(),
        ));
    }
    Ok(())
}

fn observe_value_placement(table: &Memtable) -> Result<ValuePlacementObservation> {
    let mut non_null_versions = 0u64;
    let mut inline_value_bytes = 0u64;
    let mut latest_live_value_bytes = 0u64;
    for (_, versions) in table.all_versions() {
        for version in versions {
            if let Some(value) = &version.value {
                non_null_versions += 1;
                inline_value_bytes = inline_value_bytes
                    .checked_add(value.len() as u64)
                    .ok_or_else(|| Error::InvalidSegment("value byte total overflow".into()))?;
            }
        }
        if let Some(value) = versions.last().and_then(|version| version.value.as_ref()) {
            latest_live_value_bytes = latest_live_value_bytes
                .checked_add(value.len() as u64)
                .ok_or_else(|| Error::InvalidSegment("live value byte total overflow".into()))?;
        }
    }
    let separated_pointer_bytes = non_null_versions
        .checked_mul(VALUE_POINTER_BYTES as u64)
        .ok_or_else(|| Error::InvalidSegment("modeled pointer byte total overflow".into()))?;
    Ok(ValuePlacementObservation {
        evidence_kind: "analytical-model-only".into(),
        non_null_versions,
        inline_value_bytes,
        latest_live_value_bytes,
        obsolete_value_bytes: inline_value_bytes.saturating_sub(latest_live_value_bytes),
        separated_pointer_bytes,
        append_only_value_log_bytes: inline_value_bytes,
        modeled_compaction_bytes_avoided: inline_value_bytes
            .saturating_sub(separated_pointer_bytes),
        production_retainable: false,
        missing_proofs: vec![
            "authenticated pointer encoding".into(),
            "atomic WAL and manifest publication".into(),
            "snapshot and backup closure".into(),
            "value-log garbage collection".into(),
            "range-read amplification".into(),
            "corruption and crash recovery".into(),
        ],
    })
}

fn observe_families(pages: &[PageSample]) -> Result<Vec<FamilyObservation>> {
    let mut grouped: BTreeMap<String, Vec<PageSample>> = BTreeMap::new();
    for page in pages {
        grouped
            .entry(page.family.clone())
            .or_default()
            .push(page.clone());
    }
    grouped
        .into_iter()
        .map(|(family, samples)| {
            let mut row_groups = BTreeMap::new();
            for sample in &samples {
                row_groups.insert(sample.row_group, sample.row_count);
            }
            let none = observe_codec("none", &samples)?;
            let lz4 = observe_codec("lz4", &samples)?;
            let zstd = observe_codec("zstd", &samples)?;
            Ok(FamilyObservation {
                family,
                rows: row_groups.values().copied().sum(),
                page_count: samples.len() as u64,
                logical_page_bytes: none.logical_bytes,
                lz4_adaptive_bytes: lz4.adaptive_stored_bytes,
                zstd_adaptive_bytes: zstd.adaptive_stored_bytes,
            })
        })
        .collect()
}

fn observe_reopened_database(
    root: &Path,
    config: PhysicalPolicyConfig,
    corpus: &Corpus,
) -> Result<ReopenedPointMissObservation> {
    let options = DatabaseOptions {
        page_cache_bytes: config.cache_bytes,
        segment_row_group_budget: integrated_row_group_budget(config)?,
        ..DatabaseOptions::default()
    };
    let mut database = Database::create_with_options(root, options)?;
    database.write_owned(
        WriteBatch::new(corpus.mutations.clone())?,
        Durability::Authoritative,
    )?;
    database.flush_memtable(1)?;
    let snapshot = database.snapshot();
    let segment_physical_bytes =
        database
            .manifest()
            .segments
            .iter()
            .try_fold(0u64, |total, segment| {
                total.checked_add(segment.bytes).ok_or_else(|| {
                    Error::InvalidSegment("manifest segment byte total overflow".into())
                })
            })?;
    drop(database);

    let database = Database::open_with_options(root, options)?;
    if database.snapshot() != snapshot {
        return Err(Error::InvalidSegment(
            "normal reopen changed the candidate corpus snapshot".into(),
        ));
    }
    let open = database.segment_open_evidence().clone();
    let before = database.page_cache_stats();
    let started = Instant::now();
    for probe in 0..config.point_misses {
        let family = probe % FAMILY_PREFIXES.len();
        let ordinal = (probe / FAMILY_PREFIXES.len()) % config.records_per_family;
        let key = corpus_key(FAMILY_PREFIXES[family].1, ordinal, true);
        if database.get(&key, snapshot)?.is_some() {
            return Err(Error::InvalidSegment(
                "normal reopen returned a value for an absent corpus key".into(),
            ));
        }
    }
    let elapsed_nanoseconds = started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX);
    let after_misses = database.page_cache_stats();
    let mut hit_samples_verified = 0u64;
    for (key, expected) in corpus.expected_latest.iter().take(128) {
        if database.get(key, snapshot)? != *expected {
            return Err(Error::InvalidSegment(
                "normal reopen returned a wrong present/tombstoned value".into(),
            ));
        }
        hit_samples_verified += 1;
    }
    let after_reads = database.page_cache_stats();
    Ok(ReopenedPointMissObservation {
        integrated_rrflowkv: true,
        open_none_policy_segment_count: open.none_policy_segment_count,
        open_adaptive_lz4_policy_segment_count: open.adaptive_lz4_policy_segment_count,
        open_raw_page_count: open.raw_page_count,
        open_compressed_page_count: open.compressed_page_count,
        open_stored_page_bytes: open.stored_page_bytes,
        open_logical_page_bytes: open.logical_page_bytes,
        open_persisted_filter_count: open.persisted_filter_count,
        open_persisted_filter_bytes: open.persisted_filter_bytes,
        open_semantic_page_operations: open.semantic_page_operations,
        open_semantic_page_bytes: open.semantic_page_bytes,
        misses_verified: config.point_misses as u64,
        hit_samples_verified,
        elapsed_nanoseconds,
        segment_physical_bytes,
        filter_checks: after_misses
            .filter_checks
            .saturating_sub(before.filter_checks),
        filter_negatives: after_misses
            .filter_negatives
            .saturating_sub(before.filter_negatives),
        page_cache_hits: after_reads.hits.saturating_sub(before.hits),
        page_cache_misses: after_reads.misses.saturating_sub(before.misses),
        page_loads: after_reads.loads.saturating_sub(before.loads),
        bytes_read: after_reads.bytes_read.saturating_sub(before.bytes_read),
        bytes_decoded: after_reads
            .bytes_decoded
            .saturating_sub(before.bytes_decoded),
        bytes_decompressed: after_reads
            .bytes_decompressed
            .saturating_sub(before.bytes_decompressed),
        final_cache_resident_bytes: after_reads.resident_bytes as u64,
    })
}

fn integrated_row_group_budget(config: PhysicalPolicyConfig) -> Result<SegmentRowGroupBudget> {
    let max_rows = config
        .versions_per_key
        .checked_mul(4)
        .ok_or_else(|| Error::InvalidConfiguration("cache-lab row bound overflow".into()))?;
    let target_bytes = config
        .value_bytes
        .checked_mul(max_rows)
        .and_then(|bytes| bytes.checked_add(1_024))
        .ok_or_else(|| Error::InvalidConfiguration("cache-lab byte target overflow".into()))?;
    Ok(SegmentRowGroupBudget {
        max_rows,
        target_bytes,
    })
}

fn observe_integrated_cache_policies(
    root: &Path,
    config: PhysicalPolicyConfig,
    corpus: &Corpus,
) -> Result<Vec<IntegratedCachePolicyObservation>> {
    let policies = [
        PageCachePolicy::ExactLru,
        PageCachePolicy::ScanResistantLru {
            protected_capacity_basis_points:
                crate::DEFAULT_PAGE_CACHE_PROTECTED_CAPACITY_BASIS_POINTS,
        },
    ];
    policies
        .into_iter()
        .map(|policy| observe_integrated_cache_policy(root, config, corpus, policy))
        .collect()
}

fn observe_integrated_cache_policy(
    root: &Path,
    config: PhysicalPolicyConfig,
    corpus: &Corpus,
    policy: PageCachePolicy,
) -> Result<IntegratedCachePolicyObservation> {
    let database = Database::open_with_options(
        root,
        DatabaseOptions {
            page_cache_bytes: config.cache_bytes,
            page_cache_policy: policy,
            segment_row_group_budget: integrated_row_group_budget(config)?,
            ..DatabaseOptions::default()
        },
    )?;
    let snapshot = database.snapshot();
    let manifest_digest =
        sha256_hex(&serde_json::to_vec(database.manifest()).map_err(|error| {
            Error::InvalidSegment(format!("cache-lab manifest encoding failed: {error}"))
        })?);
    let hot_keys = FAMILY_PREFIXES
        .iter()
        .map(|(_, prefix)| corpus_key(prefix, 1, false))
        .collect::<Vec<_>>();
    let expected_hot_values = hot_keys
        .iter()
        .map(|key| {
            corpus
                .expected_latest
                .get(key)
                .and_then(Clone::clone)
                .ok_or_else(|| {
                    Error::InvalidSegment(
                        "cache-lab hot key is not present in the expected corpus".into(),
                    )
                })
        })
        .collect::<Result<Vec<_>>>()?;
    let expected_rows = corpus
        .expected_latest
        .iter()
        .filter_map(|(key, value)| {
            value
                .as_ref()
                .map(|value| (key.clone(), Some(value.clone())))
        })
        .collect::<Vec<_>>();
    let expected_row_digest = digest_projected_rows(&expected_rows)?;

    let before = database.page_cache_stats();
    let first_hot_values = read_integrated_hot_values(&database, snapshot, &hot_keys)?;
    let second_hot_values = read_integrated_hot_values(&database, snapshot, &hot_keys)?;
    let after_warm = database.page_cache_stats();

    let mut stream = database.begin_projected_read(ProjectedReadRequest {
        ranges: vec![ProjectedReadRange::all()],
        projection: ProjectedReadProjection::KeyValue,
        snapshot,
        budget: ProjectedReadBudget {
            max_batch_rows: 257,
            ..ProjectedReadBudget::default()
        },
    })?;
    let mut rows = Vec::with_capacity(expected_rows.len());
    while let Some(batch) = stream.next_batch()? {
        for row in 0..batch.len() {
            rows.push((
                batch
                    .key(row)
                    .ok_or_else(|| {
                        Error::InvalidSegment("cache-lab projected key is absent".into())
                    })?
                    .to_vec(),
                batch.value(row).map(<[u8]>::to_vec),
            ));
        }
    }
    drop(stream);
    let after_scan = database.page_cache_stats();
    let post_scan_hot_values = read_integrated_hot_values(&database, snapshot, &hot_keys)?;
    let after_post_scan = database.page_cache_stats();
    let projected_row_digest = digest_projected_rows(&rows)?;
    let semantic_identity_exact = first_hot_values == expected_hot_values
        && second_hot_values == expected_hot_values
        && post_scan_hot_values == expected_hot_values
        && rows == expected_rows
        && projected_row_digest == expected_row_digest;
    let classified_bytes = after_post_scan
        .probationary_resident_bytes
        .checked_add(after_post_scan.protected_resident_bytes)
        .ok_or_else(|| Error::InvalidSegment("cache-lab region byte total overflow".into()))?;
    let classified_entries = after_post_scan
        .probationary_entries
        .checked_add(after_post_scan.protected_entries)
        .ok_or_else(|| Error::InvalidSegment("cache-lab region entry total overflow".into()))?;
    let exact_capacity_respected = after_post_scan.resident_bytes <= config.cache_bytes
        && match policy {
            PageCachePolicy::ExactLru => classified_bytes == 0 && classified_entries == 0,
            PageCachePolicy::ScanResistantLru { .. } => {
                classified_bytes == after_post_scan.resident_bytes
                    && classified_entries == after_post_scan.entries
            }
        };
    let semantic_digest = digest_cache_semantics(&rows, &post_scan_hot_values)?;

    Ok(IntegratedCachePolicyObservation {
        policy: policy.kind().into(),
        protected_capacity_basis_points: u64::from(policy.protected_capacity_basis_points()),
        snapshot_sequence: snapshot.sequence,
        manifest_digest,
        semantic_digest,
        projected_row_digest,
        hot_family_count: FAMILY_PREFIXES.len() as u64,
        hot_requests_per_pass: hot_keys.len() as u64,
        projected_rows: rows.len() as u64,
        capacity_bytes: after_post_scan.capacity_bytes as u64,
        final_resident_bytes: after_post_scan.resident_bytes as u64,
        probationary_resident_bytes: after_post_scan.probationary_resident_bytes as u64,
        protected_resident_bytes: after_post_scan.protected_resident_bytes as u64,
        final_entries: after_post_scan.entries as u64,
        probationary_entries: after_post_scan.probationary_entries as u64,
        protected_entries: after_post_scan.protected_entries as u64,
        warm_hits: counter_delta(after_warm.hits, before.hits, "warm hits")?,
        warm_misses: counter_delta(after_warm.misses, before.misses, "warm misses")?,
        warm_loads: counter_delta(after_warm.loads, before.loads, "warm loads")?,
        scan_hits: counter_delta(after_scan.hits, after_warm.hits, "scan hits")?,
        scan_misses: counter_delta(after_scan.misses, after_warm.misses, "scan misses")?,
        scan_loads: counter_delta(after_scan.loads, after_warm.loads, "scan loads")?,
        post_scan_hot_hits: counter_delta(
            after_post_scan.hits,
            after_scan.hits,
            "post-scan hot hits",
        )?,
        post_scan_hot_misses: counter_delta(
            after_post_scan.misses,
            after_scan.misses,
            "post-scan hot misses",
        )?,
        post_scan_hot_loads: counter_delta(
            after_post_scan.loads,
            after_scan.loads,
            "post-scan hot loads",
        )?,
        admissions: counter_delta(after_post_scan.admissions, before.admissions, "admissions")?,
        admission_rejections: counter_delta(
            after_post_scan.admission_rejections,
            before.admission_rejections,
            "admission rejections",
        )?,
        promotions: counter_delta(after_post_scan.promotions, before.promotions, "promotions")?,
        demotions: counter_delta(after_post_scan.demotions, before.demotions, "demotions")?,
        same_scope_hits: counter_delta(
            after_post_scan.same_scope_hits,
            before.same_scope_hits,
            "same-scope hits",
        )?,
        evictions: counter_delta(after_post_scan.evictions, before.evictions, "evictions")?,
        duplicate_loads: counter_delta(
            after_post_scan.duplicate_loads,
            before.duplicate_loads,
            "duplicate loads",
        )?,
        bytes_read: counter_delta(after_post_scan.bytes_read, before.bytes_read, "bytes read")?,
        bytes_decoded: counter_delta(
            after_post_scan.bytes_decoded,
            before.bytes_decoded,
            "bytes decoded",
        )?,
        bytes_decompressed: counter_delta(
            after_post_scan.bytes_decompressed,
            before.bytes_decompressed,
            "bytes decompressed",
        )?,
        semantic_identity_exact,
        exact_capacity_respected,
    })
}

fn read_integrated_hot_values(
    database: &Database,
    snapshot: crate::Snapshot,
    hot_keys: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>> {
    hot_keys
        .iter()
        .map(|key| {
            database.get(key, snapshot)?.ok_or_else(|| {
                Error::InvalidSegment("cache-lab present hot key was not visible".into())
            })
        })
        .collect()
}

fn digest_projected_rows(rows: &[(Vec<u8>, Option<Vec<u8>>)]) -> Result<String> {
    let mut encoded = Vec::new();
    for (key, value) in rows {
        encoded.extend_from_slice(
            &u64::try_from(key.len())
                .map_err(|_| Error::InvalidSegment("cache-lab key length exceeds u64".into()))?
                .to_le_bytes(),
        );
        encoded.extend_from_slice(key);
        match value {
            Some(value) => {
                encoded.push(1);
                encoded.extend_from_slice(
                    &u64::try_from(value.len())
                        .map_err(|_| {
                            Error::InvalidSegment("cache-lab value length exceeds u64".into())
                        })?
                        .to_le_bytes(),
                );
                encoded.extend_from_slice(value);
            }
            None => {
                encoded.push(0);
                encoded.extend_from_slice(&0u64.to_le_bytes());
            }
        }
    }
    Ok(sha256_hex(&encoded))
}

fn digest_cache_semantics(
    rows: &[(Vec<u8>, Option<Vec<u8>>)],
    hot_values: &[Vec<u8>],
) -> Result<String> {
    let mut encoded = digest_projected_rows(rows)?.into_bytes();
    for value in hot_values {
        encoded.extend_from_slice(
            &u64::try_from(value.len())
                .map_err(|_| Error::InvalidSegment("cache-lab hot value exceeds u64".into()))?
                .to_le_bytes(),
        );
        encoded.extend_from_slice(value);
    }
    Ok(sha256_hex(&encoded))
}

fn counter_delta(after: u64, before: u64, name: &str) -> Result<u64> {
    after
        .checked_sub(before)
        .ok_or_else(|| Error::InvalidSegment(format!("cache-lab {name} counter moved backwards")))
}

fn validate_integrated_cache_policies(
    observations: &[IntegratedCachePolicyObservation],
) -> Result<()> {
    let exact = observations
        .iter()
        .find(|observation| observation.policy == PageCachePolicy::ExactLru.kind())
        .ok_or_else(|| Error::InvalidSegment("integrated exact-LRU evidence is absent".into()))?;
    let selected_policy = PageCachePolicy::default();
    let scan_resistant = observations
        .iter()
        .find(|observation| observation.policy == selected_policy.kind())
        .ok_or_else(|| {
            Error::InvalidSegment("integrated scan-resistant evidence is absent".into())
        })?;
    if !exact.semantic_identity_exact
        || !scan_resistant.semantic_identity_exact
        || !exact.exact_capacity_respected
        || !scan_resistant.exact_capacity_respected
        || exact.snapshot_sequence != scan_resistant.snapshot_sequence
        || exact.manifest_digest != scan_resistant.manifest_digest
        || exact.semantic_digest != scan_resistant.semantic_digest
        || exact.projected_row_digest != scan_resistant.projected_row_digest
        || exact.post_scan_hot_loads == 0
        || scan_resistant.post_scan_hot_loads >= exact.post_scan_hot_loads
        || scan_resistant.promotions == 0
        || scan_resistant.same_scope_hits == 0
        || scan_resistant.protected_entries == 0
    {
        return Err(Error::InvalidSegment(format!(
            "integrated scan-resistant cache policy failed: exact_post_scan_loads={}, scan_resistant_post_scan_loads={}, scan_warm_hits={}, scan_warm_loads={}, scan_scan_hits={}, scan_scan_loads={}, promotions={}, same_scope_hits={}, protected_entries={}",
            exact.post_scan_hot_loads,
            scan_resistant.post_scan_hot_loads,
            scan_resistant.warm_hits,
            scan_resistant.warm_loads,
            scan_resistant.scan_hits,
            scan_resistant.scan_loads,
            scan_resistant.promotions,
            scan_resistant.same_scope_hits,
            scan_resistant.protected_entries
        )));
    }
    Ok(())
}

fn decide_candidates(
    codecs: &[CodecObservation],
    filter: &FilterObservation,
    caches: &[CachePolicyObservation],
    integrated_caches: &[IntegratedCachePolicyObservation],
    values: &ValuePlacementObservation,
) -> Vec<CandidateDecision> {
    let mut decisions = Vec::new();
    for codec in codecs.iter().filter(|codec| codec.codec != "none") {
        let passes = codec.round_trip_exact
            && codec.savings_basis_points >= ADAPTIVE_CODEC_MINIMUM_SAVINGS_BPS
            && codec.adaptive_selected_pages > 0;
        let integrated = codec.codec == "lz4";
        decisions.push(CandidateDecision {
            candidate: format!("adaptive-{}-page-codec", codec.codec),
            decision: match (passes, integrated) {
                (true, true) => "integrated",
                (true, false) => "advance",
                (false, _) => "reject",
            }
            .into(),
            reason: if integrated {
                format!(
                    "segment_v6=true, exact_round_trip={}, selected_pages={}, savings_basis_points={}, required_savings_basis_points={ADAPTIVE_CODEC_MINIMUM_SAVINGS_BPS}; authenticated adaptive LZ4 is integrated in the normal rrflowKV flush, compaction, reopen, and read path",
                    codec.round_trip_exact, codec.adaptive_selected_pages, codec.savings_basis_points
                )
            } else {
                format!(
                    "exact_round_trip={}, selected_pages={}, savings_basis_points={}, required_savings_basis_points={ADAPTIVE_CODEC_MINIMUM_SAVINGS_BPS}; this candidate remains laboratory-only and requires a separate production-format plan",
                    codec.round_trip_exact, codec.adaptive_selected_pages, codec.savings_basis_points
                )
            },
        });
    }
    decisions.push(CandidateDecision {
        candidate: "persisted-row-group-bloom".into(),
        decision: if filter.passes_policy_threshold {
            "integrated"
        } else {
            "integrated-policy-failed"
        }
        .into(),
        reason: format!(
            "segment_v6=true, member_false_negatives={}, false_positive_ppm={}, limit_ppm={}, serialized_bytes={}",
            filter.member_false_negatives,
            filter.false_positive_parts_per_million,
            FILTER_FALSE_POSITIVE_LIMIT_PPM,
            filter.serialized_bytes
        ),
    });
    let baseline = caches.iter().find(|cache| cache.policy == "exact-byte-lru");
    let candidate = caches
        .iter()
        .find(|cache| cache.policy == "exact-byte-segmented-lru-candidate");
    let cache_advances = baseline
        .zip(candidate)
        .is_some_and(|(baseline, candidate)| {
            baseline.semantic_identity_exact
                && candidate.semantic_identity_exact
                && candidate.exact_capacity_respected
                && candidate.post_scan_hot_hits > baseline.post_scan_hot_hits
        });
    decisions.push(CandidateDecision {
        candidate: "exact-byte-segmented-lru".into(),
        decision: if cache_advances { "advance" } else { "reject" }.into(),
        reason: match (baseline, candidate) {
            (Some(baseline), Some(candidate)) => format!(
                "post_scan_hot_hits={} versus LRU {}; both must preserve identity and exact capacity",
                candidate.post_scan_hot_hits, baseline.post_scan_hot_hits
            ),
            _ => "required cache observations are absent".into(),
        },
    });
    let exact_integrated = integrated_caches
        .iter()
        .find(|cache| cache.policy == PageCachePolicy::ExactLru.kind());
    let selected_policy = PageCachePolicy::default();
    let scan_resistant_integrated = integrated_caches
        .iter()
        .find(|cache| cache.policy == selected_policy.kind());
    let cache_integrated =
        exact_integrated
            .zip(scan_resistant_integrated)
            .is_some_and(|(exact, scan_resistant)| {
                exact.semantic_identity_exact
                    && scan_resistant.semantic_identity_exact
                    && exact.exact_capacity_respected
                    && scan_resistant.exact_capacity_respected
                    && exact.semantic_digest == scan_resistant.semantic_digest
                    && exact.post_scan_hot_loads > 0
                    && scan_resistant.post_scan_hot_loads < exact.post_scan_hot_loads
                    && scan_resistant.promotions > 0
                    && scan_resistant.same_scope_hits > 0
            });
    decisions.push(CandidateDecision {
        candidate: "scan-resistant-page-cache".into(),
        decision: if cache_integrated {
            "integrated"
        } else {
            "integrated-policy-failed"
        }
        .into(),
        reason: match (exact_integrated, scan_resistant_integrated) {
            (Some(exact), Some(scan_resistant)) => format!(
                "real_rrflowkv_reader=true, semantic_identity_exact={}, exact_capacity_respected={}, post_scan_hot_loads={} versus exact-LRU {}, promotions={}, same_scope_hits={}, duplicate_loads={}",
                scan_resistant.semantic_identity_exact
                    && exact.semantic_identity_exact
                    && scan_resistant.semantic_digest == exact.semantic_digest,
                scan_resistant.exact_capacity_respected && exact.exact_capacity_respected,
                scan_resistant.post_scan_hot_loads,
                exact.post_scan_hot_loads,
                scan_resistant.promotions,
                scan_resistant.same_scope_hits,
                scan_resistant.duplicate_loads
            ),
            _ => "required integrated cache observations are absent".into(),
        },
    });
    decisions.push(CandidateDecision {
        candidate: "semantic-family-cache-partitions".into(),
        decision: if cache_integrated {
            "reject"
        } else {
            "unresolved"
        }
        .into(),
        reason: "the real mixed audit/edge/record/runtime/scalar/term/vector corpus is served by one family-neutral policy, so semantic quotas would duplicate key knowledge and can strand capacity".into(),
    });
    decisions.push(CandidateDecision {
        candidate: "value-separation".into(),
        decision: "reject".into(),
        reason: format!(
            "{} reports {} modeled bytes avoided, but production_retainable={} and authenticated pointer publication, recovery, GC, snapshot, corruption, and range-read proofs are absent",
            values.evidence_kind,
            values.modeled_compaction_bytes_avoided,
            values.production_retainable
        ),
    });
    decisions
}

fn peak_rss_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let kib = status
            .lines()
            .find_map(|line| line.strip_prefix("VmHWM:"))?
            .split_whitespace()
            .next()?
            .parse::<u64>()
            .ok()?;
        kib.checked_mul(1024)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> PhysicalPolicyConfig {
        PhysicalPolicyConfig {
            seed: 0xca7c_4f10_0000_0001,
            records_per_family: 32,
            versions_per_key: 2,
            value_bytes: 128,
            point_misses: 512,
            cache_bytes: 32 * 1024,
        }
    }

    #[test]
    fn physical_policy_lab_proves_adaptive_v6_and_keeps_other_candidates_non_production() {
        let first_root = tempfile::tempdir().unwrap();
        let second_root = tempfile::tempdir().unwrap();
        let first = run_physical_policy_trial(first_root.path(), small_config()).unwrap();
        let second = run_physical_policy_trial(second_root.path(), small_config()).unwrap();

        assert_eq!(first.corpus_digest, second.corpus_digest);
        assert_eq!(first.operation_count, second.operation_count);
        assert_eq!(first.key_count, second.key_count);
        assert_eq!(first.page_count, second.page_count);
        assert_eq!(first.segment_physical_bytes, second.segment_physical_bytes);
        assert_eq!(
            first.integrated_cache_policies,
            second.integrated_cache_policies
        );
        assert!(first.codecs.iter().all(|codec| codec.round_trip_exact));
        assert_eq!(first.persisted_row_group_filter.member_false_negatives, 0);
        assert!(first.persisted_row_group_filter.passes_policy_threshold);
        assert!(first
            .cache_candidates
            .iter()
            .all(|cache| cache.semantic_identity_exact && cache.exact_capacity_respected));
        let exact_cache = first
            .integrated_cache_policies
            .iter()
            .find(|cache| cache.policy == PageCachePolicy::ExactLru.kind())
            .unwrap();
        let scan_resistant_cache = first
            .integrated_cache_policies
            .iter()
            .find(|cache| cache.policy == PageCachePolicy::default().kind())
            .unwrap();
        assert_eq!(
            exact_cache.semantic_digest,
            scan_resistant_cache.semantic_digest
        );
        assert!(exact_cache.post_scan_hot_loads > 0);
        assert!(scan_resistant_cache.post_scan_hot_loads < exact_cache.post_scan_hot_loads);
        assert!(scan_resistant_cache.same_scope_hits > 0);
        assert!(scan_resistant_cache.promotions > 0);
        assert!(scan_resistant_cache.protected_entries > 0);
        assert!(!first.value_placement_candidate.production_retainable);
        assert!(first.reopened_point_miss.open_persisted_filter_count > 0);
        assert!(first.reopened_point_miss.open_persisted_filter_bytes > 0);
        assert_eq!(first.reopened_point_miss.open_semantic_page_operations, 0);
        assert_eq!(first.reopened_point_miss.open_semantic_page_bytes, 0);
        assert_eq!(first.reopened_point_miss.open_none_policy_segment_count, 0);
        assert!(
            first
                .reopened_point_miss
                .open_adaptive_lz4_policy_segment_count
                > 0
        );
        assert!(first.reopened_point_miss.open_raw_page_count > 0);
        assert!(first.reopened_point_miss.open_compressed_page_count > 0);
        assert!(
            first.reopened_point_miss.open_stored_page_bytes
                < first.reopened_point_miss.open_logical_page_bytes
        );
        assert!(first.reopened_point_miss.bytes_decompressed > 0);
        assert!(first.reopened_point_miss.filter_negatives > 0);
        assert!(first.reopened_point_miss.filter_checks > 0);
        assert!(first.reopened_point_miss.page_loads < first.reopened_point_miss.filter_checks);
        assert_eq!(first.segment_format_version, 6);
        let lz4 = first
            .decisions
            .iter()
            .find(|decision| decision.candidate == "adaptive-lz4-page-codec")
            .unwrap();
        assert_eq!(lz4.decision, "integrated");
        assert!(lz4.reason.contains("normal rrflowKV"));
        let zstd = first
            .decisions
            .iter()
            .find(|decision| decision.candidate == "adaptive-zstd-page-codec")
            .unwrap();
        assert_eq!(zstd.decision, "advance");
        assert!(zstd.reason.contains("laboratory-only"));

        let mut invalid_reopen = first.reopened_point_miss.clone();
        invalid_reopen.open_semantic_page_operations = 1;
        assert!(validate_persisted_filter_integration(
            first.row_group_count as usize,
            &first.persisted_row_group_filter,
            &invalid_reopen,
        )
        .is_err());
    }

    #[test]
    fn physical_policy_lab_denies_unbounded_or_zero_configuration() {
        let mut config = small_config();
        config.records_per_family = 0;
        assert!(matches!(
            config.validate(),
            Err(Error::InvalidConfiguration(_))
        ));
        config = small_config();
        config.value_bytes = MAX_VALUE_BYTES + 1;
        assert!(matches!(
            config.validate(),
            Err(Error::InvalidConfiguration(_))
        ));
        config = small_config();
        config.cache_bytes = MAX_CACHE_BYTES + 1;
        assert!(matches!(
            config.validate(),
            Err(Error::InvalidConfiguration(_))
        ));
    }

    #[test]
    fn canonical_filter_has_no_member_false_negatives() {
        let keys = (0..128)
            .map(|index| format!("record/{index:08x}").into_bytes())
            .collect::<Vec<_>>();
        let mut filter = RowGroupFilter::new(keys.len() as u32).unwrap();
        for key in &keys {
            filter.insert(key);
        }
        assert!(keys.iter().all(|key| filter.may_contain(key)));
        assert_eq!(
            RowGroupFilter::from_words(keys.len() as u32, filter.words().to_vec()).unwrap(),
            filter
        );
        assert!(RowGroupFilter::from_words(keys.len() as u32, Vec::new()).is_err());
    }

    #[test]
    fn cache_candidates_preserve_identity_and_exact_capacity() {
        let pages = (0..64)
            .map(|id| PageSample {
                id,
                row_group: id / super::super::format::PAGES_PER_ROW_GROUP,
                row_count: 1,
                family: "record".into(),
                kind: "value-data",
                bytes: vec![id as u8; 1024],
            })
            .collect::<Vec<_>>();
        let observations = observe_caches(&pages, 16 * 1024).unwrap();
        assert_eq!(observations.len(), 2);
        assert!(observations
            .iter()
            .all(|observation| observation.semantic_identity_exact
                && observation.exact_capacity_respected
                && observation.peak_resident_bytes <= observation.capacity_bytes));
    }

    #[test]
    fn encoded_segment_footer_is_not_mistaken_for_page_data() {
        let corpus = build_corpus(small_config()).unwrap();
        let mut table = Memtable::default();
        let operations = corpus.mutations.len() as u64;
        table
            .apply_owned_write_batch(WriteBatch::new(corpus.mutations).unwrap(), 1, operations)
            .unwrap();
        let encoded = encode(
            &table,
            SegmentRowGroupBudget::default(),
            SegmentCompressionPolicy::None,
        )
        .unwrap();
        let mut cursor = Cursor::new(&encoded.bytes);
        let metadata =
            parse_metadata(&mut cursor, encoded.bytes.len() as u64, encoded.checksum).unwrap();
        let pages = extract_pages(&encoded.bytes, &metadata.row_groups).unwrap();
        let page_bytes: usize = pages.iter().map(|page| page.bytes.len()).sum();
        assert!(page_bytes + FOOTER_BYTES < encoded.bytes.len());
    }
}
