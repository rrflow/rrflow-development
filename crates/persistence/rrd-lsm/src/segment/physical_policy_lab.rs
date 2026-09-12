//! Feature-gated C-06i measurements over rrflowKV's real segment-v4 encoder.
//!
//! Nothing in this module is a production storage policy. It exists to make
//! codec, persisted-filter, cache-admission, and value-placement decisions
//! reproducible before any candidate is allowed to change durable bytes.

#[cfg(test)]
use super::format::FOOTER_BYTES;
use super::format::{encode, parse_metadata, sha256_hex, PageKind};
use crate::{
    Database, DatabaseOptions, Durability, Error, Memtable, Mutation, Result,
    SegmentRowGroupBudget, WriteBatch, SEGMENT_FORMAT_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;

pub const PHYSICAL_POLICY_EVIDENCE_VERSION: u16 = 1;

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
const FILTER_MAGIC: &[u8; 8] = b"RRBF0001";
const FILTER_HASH_FUNCTIONS: usize = 7;
const FILTER_BITS_PER_KEY: usize = 10;
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
    pub passes_candidate_threshold: bool,
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
    pub persisted_filter_candidate: FilterObservation,
    pub cache_candidates: Vec<CachePolicyObservation>,
    pub value_placement_candidate: ValuePlacementObservation,
    pub families: Vec<FamilyObservation>,
    pub reopened_point_miss: ReopenedPointMissObservation,
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

#[derive(Debug, Clone)]
struct CandidateFilter {
    bits: Vec<u64>,
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
    let encoded = encode(&table, SegmentRowGroupBudget::default())?;
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
    let persisted_filter_candidate = observe_filters(&encoded.bytes, &metadata.row_groups, config)?;
    let cache_candidates = observe_caches(&pages, config.cache_bytes)?;
    let value_placement_candidate = observe_value_placement(&table)?;
    let families = observe_families(&pages)?;
    let reopened_point_miss = observe_reopened_database(root, config, &corpus)?;
    let decisions = decide_candidates(
        &codecs,
        &persisted_filter_candidate,
        &cache_candidates,
        &value_placement_candidate,
    );

    Ok(PhysicalPolicyTrial {
        evidence_version: PHYSICAL_POLICY_EVIDENCE_VERSION,
        evidence_scope: "c06i-candidate-screen-not-production-policy".into(),
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
        persisted_filter_candidate,
        cache_candidates,
        value_placement_candidate,
        families,
        reopened_point_miss,
        decisions,
        limitations: vec![
            "codec/filter/cache candidates are isolated measurements over real v4 page bodies; segment v4 still writes none of them".into(),
            "value placement is a byte model only and lacks pointer publication, recovery, snapshot, corruption, range-read, and garbage-collection proof".into(),
            "one machine and one deterministic corpus cannot establish release latency, all-workload policy, or competitor superiority".into(),
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
                    "segment v4 candidate screen expected current uncompressed pages".into(),
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
        let filter = CandidateFilter::from_keys(&keys);
        let persisted = filter.encode()?;
        let reopened = CandidateFilter::decode(&persisted)?;
        filters += 1;
        unique_keys = unique_keys.saturating_add(keys.len() as u64);
        serialized_bytes = serialized_bytes.saturating_add(persisted.len() as u64);
        for key in &keys {
            member_queries += 1;
            if !reopened.may_contain(key) {
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
            if reopened.may_contain(&absent) {
                false_positives += 1;
            }
        }
    }
    let false_positive_parts_per_million = false_positives
        .saturating_mul(1_000_000)
        .checked_div(absent_queries)
        .unwrap_or(0);
    Ok(FilterObservation {
        format: "candidate-rrbf0001-not-segment-v4".into(),
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
        passes_candidate_threshold: member_false_negatives == 0
            && false_positive_parts_per_million <= FILTER_FALSE_POSITIVE_LIMIT_PPM,
    })
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

impl CandidateFilter {
    fn from_keys(keys: &[Vec<u8>]) -> Self {
        let bit_count = keys
            .len()
            .saturating_mul(FILTER_BITS_PER_KEY)
            .max(u64::BITS as usize);
        let mut filter = Self {
            bits: vec![0; bit_count.div_ceil(u64::BITS as usize)],
        };
        for key in keys {
            filter.insert(key);
        }
        filter
    }

    fn insert(&mut self, key: &[u8]) {
        for position in filter_positions(key, self.bits.len() * u64::BITS as usize) {
            self.bits[position / u64::BITS as usize] |= 1u64 << (position % u64::BITS as usize);
        }
    }

    fn may_contain(&self, key: &[u8]) -> bool {
        filter_positions(key, self.bits.len() * u64::BITS as usize)
            .into_iter()
            .all(|position| {
                self.bits[position / u64::BITS as usize] & (1u64 << (position % u64::BITS as usize))
                    != 0
            })
    }

    fn encode(&self) -> Result<Vec<u8>> {
        let word_count = u32::try_from(self.bits.len())
            .map_err(|_| Error::InvalidSegment("candidate filter word count exceeds u32".into()))?;
        let mut output = Vec::with_capacity(16 + self.bits.len() * 8);
        output.extend_from_slice(FILTER_MAGIC);
        output.extend_from_slice(&word_count.to_le_bytes());
        output.push(FILTER_HASH_FUNCTIONS as u8);
        output.extend_from_slice(&[0; 3]);
        for word in &self.bits {
            output.extend_from_slice(&word.to_le_bytes());
        }
        Ok(output)
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 || bytes.get(..8) != Some(FILTER_MAGIC.as_slice()) {
            return Err(Error::InvalidSegment(
                "candidate filter framing is invalid".into(),
            ));
        }
        let word_count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        if bytes[12] as usize != FILTER_HASH_FUNCTIONS || bytes[13..16] != [0; 3] {
            return Err(Error::InvalidSegment(
                "candidate filter parameters are unsupported".into(),
            ));
        }
        let expected = 16usize
            .checked_add(word_count.checked_mul(8).ok_or_else(|| {
                Error::InvalidSegment("candidate filter byte count overflow".into())
            })?)
            .ok_or_else(|| Error::InvalidSegment("candidate filter length overflow".into()))?;
        if bytes.len() != expected || word_count == 0 {
            return Err(Error::InvalidSegment(
                "candidate filter length is invalid".into(),
            ));
        }
        let bits = bytes[16..]
            .as_chunks::<8>()
            .0
            .iter()
            .map(|chunk| u64::from_le_bytes(*chunk))
            .collect();
        Ok(Self { bits })
    }
}

fn filter_positions(key: &[u8], bit_count: usize) -> [usize; FILTER_HASH_FUNCTIONS] {
    let first = filter_hash(key, 0xcbf2_9ce4_8422_2325);
    let second = filter_hash(key, 0x9e37_79b9_7f4a_7c15) | 1;
    std::array::from_fn(|index| {
        first.wrapping_add((index as u64).wrapping_mul(second)) as usize % bit_count
    })
}

fn filter_hash(key: &[u8], seed: u64) -> u64 {
    let mut hash = seed;
    for byte in key {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    hash ^ (hash >> 33)
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
    let mut hit_samples_verified = 0u64;
    for (key, expected) in corpus.expected_latest.iter().take(128) {
        if database.get(key, snapshot)? != *expected {
            return Err(Error::InvalidSegment(
                "normal reopen returned a wrong present/tombstoned value".into(),
            ));
        }
        hit_samples_verified += 1;
    }
    let after = database.page_cache_stats();
    Ok(ReopenedPointMissObservation {
        integrated_rrflowkv: true,
        misses_verified: config.point_misses as u64,
        hit_samples_verified,
        elapsed_nanoseconds,
        segment_physical_bytes,
        filter_checks: after.filter_checks.saturating_sub(before.filter_checks),
        filter_negatives: after
            .filter_negatives
            .saturating_sub(before.filter_negatives),
        page_cache_hits: after.hits.saturating_sub(before.hits),
        page_cache_misses: after.misses.saturating_sub(before.misses),
        page_loads: after.loads.saturating_sub(before.loads),
        bytes_read: after.bytes_read.saturating_sub(before.bytes_read),
        bytes_decoded: after.bytes_decoded.saturating_sub(before.bytes_decoded),
        bytes_decompressed: after
            .bytes_decompressed
            .saturating_sub(before.bytes_decompressed),
        final_cache_resident_bytes: after.resident_bytes as u64,
    })
}

fn decide_candidates(
    codecs: &[CodecObservation],
    filter: &FilterObservation,
    caches: &[CachePolicyObservation],
    values: &ValuePlacementObservation,
) -> Vec<CandidateDecision> {
    let mut decisions = Vec::new();
    for codec in codecs.iter().filter(|codec| codec.codec != "none") {
        let passes = codec.round_trip_exact
            && codec.savings_basis_points >= ADAPTIVE_CODEC_MINIMUM_SAVINGS_BPS
            && codec.adaptive_selected_pages > 0;
        decisions.push(CandidateDecision {
            candidate: format!("adaptive-{}-page-codec", codec.codec),
            decision: if passes { "advance" } else { "reject" }.into(),
            reason: format!(
                "exact_round_trip={}, selected_pages={}, savings_basis_points={}, required_savings_basis_points={ADAPTIVE_CODEC_MINIMUM_SAVINGS_BPS}; production integration still requires a separate format plan",
                codec.round_trip_exact, codec.adaptive_selected_pages, codec.savings_basis_points
            ),
        });
    }
    decisions.push(CandidateDecision {
        candidate: "persisted-row-group-bloom".into(),
        decision: if filter.passes_candidate_threshold {
            "advance"
        } else {
            "reject"
        }
        .into(),
        reason: format!(
            "member_false_negatives={}, false_positive_ppm={}, limit_ppm={}, serialized_bytes={}",
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
    decisions.push(CandidateDecision {
        candidate: "value-separation".into(),
        decision: "reject-for-production-from-this-slice".into(),
        reason: format!(
            "{} reports {} modeled bytes avoided, but production_retainable={} and recovery/GC/snapshot proofs are absent",
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
    fn physical_policy_lab_is_deterministic_and_keeps_candidates_non_production() {
        let first_root = tempfile::tempdir().unwrap();
        let second_root = tempfile::tempdir().unwrap();
        let first = run_physical_policy_trial(first_root.path(), small_config()).unwrap();
        let second = run_physical_policy_trial(second_root.path(), small_config()).unwrap();

        assert_eq!(first.corpus_digest, second.corpus_digest);
        assert_eq!(first.operation_count, second.operation_count);
        assert_eq!(first.key_count, second.key_count);
        assert_eq!(first.page_count, second.page_count);
        assert_eq!(first.segment_physical_bytes, second.segment_physical_bytes);
        assert!(first.codecs.iter().all(|codec| codec.round_trip_exact));
        assert_eq!(first.persisted_filter_candidate.member_false_negatives, 0);
        assert!(first
            .cache_candidates
            .iter()
            .all(|cache| cache.semantic_identity_exact && cache.exact_capacity_respected));
        assert!(!first.value_placement_candidate.production_retainable);
        assert_eq!(first.reopened_point_miss.filter_negatives, 0);
        assert!(first.reopened_point_miss.filter_checks > 0);
        assert_eq!(first.segment_format_version, 4);
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
    fn candidate_filter_round_trip_has_no_member_false_negatives() {
        let keys = (0..128)
            .map(|index| format!("record/{index:08x}").into_bytes())
            .collect::<Vec<_>>();
        let filter = CandidateFilter::from_keys(&keys);
        let encoded = filter.encode().unwrap();
        let decoded = CandidateFilter::decode(&encoded).unwrap();
        assert!(keys.iter().all(|key| decoded.may_contain(key)));
        assert!(CandidateFilter::decode(&encoded[..encoded.len() - 1]).is_err());
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
        let encoded = encode(&table, SegmentRowGroupBudget::default()).unwrap();
        let mut cursor = Cursor::new(&encoded.bytes);
        let metadata =
            parse_metadata(&mut cursor, encoded.bytes.len() as u64, encoded.checksum).unwrap();
        let pages = extract_pages(&encoded.bytes, &metadata.row_groups).unwrap();
        let page_bytes: usize = pages.iter().map(|page| page.bytes.len()).sum();
        assert!(page_bytes + FOOTER_BYTES < encoded.bytes.len());
    }
}
