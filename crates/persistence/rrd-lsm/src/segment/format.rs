use crate::{Error, Memtable, Result, SegmentDescriptor, VersionedValue};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};

pub const SEGMENT_FORMAT_VERSION: u16 = 4;
pub const SEGMENT_MAGIC: &[u8; 8] = b"RRDSEG04";
pub const INDEX_MAGIC: &[u8; 8] = b"RRDIX004";
pub const PAGE_ALIGNMENT: usize = 64;
pub const SEGMENT_HEADER_BYTES: usize = 256;
pub const INDEX_HEADER_BYTES: usize = 32;
pub const ROW_GROUP_HEADER_BYTES: usize = 32;
pub const PAGE_DESCRIPTOR_BYTES: usize = 96;
pub const PAGES_PER_ROW_GROUP: usize = 6;
pub const FOOTER_BYTES: usize = 64;
pub const DEFAULT_ROW_GROUP_TARGET_BYTES: usize = 64 * 1024;
pub const DEFAULT_ROW_GROUP_MAX_ROWS: usize = 2 * 1024;
pub const MAX_SEGMENT_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_INDEX_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_KEY_BYTES: usize = 1024 * 1024;
pub const MAX_VALUE_BYTES: usize = 8 * 1024 * 1024;

/// The immutable v4 segment schema is intentionally narrower than the RRFlow
/// semantic model. Families such as graph adjacency, lexical postings, and
/// vectors remain transactionally encoded keys and values above this physical
/// layer; this digest identifies only their common MVCC storage columns.
pub const SEGMENT_SCHEMA_DIGEST: &str =
    "7f9ffd70086074178df404606f44560f7d04546ff21ae49e802935563e5c8861";
pub const SEGMENT_KEY_CODEC_DIGEST: &str =
    "542c8fdcc2419e2b8de3b54d85e4d1fc168d4c8705acceaa1b196883a20938da";
pub const SEGMENT_PAGE_FORMAT_DIGEST: &str =
    "4992b28a9b9c8087d0290a6d57f3037da50d95e653baa87484df61fe748be151";

/// Soft limits used when a mutable key/version table is transformed into
/// immutable Arrow-layout row groups. A complete version chain for one key is
/// indivisible and may therefore exceed either limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentRowGroupBudget {
    pub max_rows: usize,
    pub target_bytes: usize,
}

impl Default for SegmentRowGroupBudget {
    fn default() -> Self {
        Self {
            max_rows: DEFAULT_ROW_GROUP_MAX_ROWS,
            target_bytes: DEFAULT_ROW_GROUP_TARGET_BYTES,
        }
    }
}

impl SegmentRowGroupBudget {
    pub(crate) fn validate(self) -> Result<Self> {
        let target_bytes = u64::try_from(self.target_bytes).ok();
        if self.max_rows == 0
            || self.max_rows > u32::MAX as usize
            || self.target_bytes == 0
            || target_bytes.is_none_or(|bytes| bytes > MAX_SEGMENT_BYTES)
        {
            return Err(Error::InvalidConfiguration(
                "segment row-group target bytes must fit within the segment limit and max rows must fit u32; both must be non-zero".into(),
            ));
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum PageKind {
    KeyOffsets = 1,
    KeyData = 2,
    SequenceValues = 3,
    ValueValidity = 4,
    ValueOffsets = 5,
    ValueData = 6,
}

impl PageKind {
    pub(super) const ORDERED: [Self; PAGES_PER_ROW_GROUP] = [
        Self::KeyOffsets,
        Self::KeyData,
        Self::SequenceValues,
        Self::ValueValidity,
        Self::ValueOffsets,
        Self::ValueData,
    ];

    fn from_byte(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::KeyOffsets),
            2 => Ok(Self::KeyData),
            3 => Ok(Self::SequenceValues),
            4 => Ok(Self::ValueValidity),
            5 => Ok(Self::ValueOffsets),
            6 => Ok(Self::ValueData),
            other => invalid(format!("unknown v4 page kind {other}")),
        }
    }

    fn column(self) -> u8 {
        match self {
            Self::KeyOffsets | Self::KeyData => 1,
            Self::SequenceValues => 2,
            Self::ValueValidity | Self::ValueOffsets | Self::ValueData => 3,
        }
    }

    fn buffer_kind(self) -> u8 {
        match self {
            Self::KeyOffsets | Self::ValueOffsets => 2,
            Self::KeyData | Self::SequenceValues | Self::ValueData => 3,
            Self::ValueValidity => 1,
        }
    }

    fn logical_type(self) -> u8 {
        match self {
            Self::SequenceValues => 2,
            _ => 1,
        }
    }

    fn physical_type(self) -> u8 {
        match self {
            Self::KeyOffsets | Self::ValueOffsets => 2,
            Self::SequenceValues => 3,
            Self::KeyData | Self::ValueData => 4,
            Self::ValueValidity => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PageDescriptor {
    pub kind: PageKind,
    pub row_start: u64,
    pub row_count: u32,
    pub null_count: u32,
    pub offset: u64,
    pub physical_bytes: usize,
    pub logical_bytes: usize,
    pub statistic_min: u64,
    pub statistic_max: u64,
    pub digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PageStatistics {
    null_count: u32,
    minimum: u64,
    maximum: u64,
}

impl PageStatistics {
    const fn new(null_count: u32, minimum: u64, maximum: u64) -> Self {
        Self {
            null_count,
            minimum,
            maximum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RowGroupDescriptor {
    pub row_start: u64,
    pub row_count: u32,
    pub first_key: Vec<u8>,
    pub last_key: Vec<u8>,
    pub pages: [PageDescriptor; PAGES_PER_ROW_GROUP],
}

impl RowGroupDescriptor {
    pub(super) fn page(&self, kind: PageKind) -> &PageDescriptor {
        &self.pages[kind as usize - 1]
    }
}

pub(super) struct EncodedSegment {
    pub bytes: Vec<u8>,
    pub checksum: String,
}

pub(super) struct ParsedMetadata {
    pub descriptor: SegmentDescriptor,
    pub row_group_budget: SegmentRowGroupBudget,
    pub row_groups: Vec<RowGroupDescriptor>,
}

#[derive(Default)]
struct RowGroupBuilder {
    row_start: u64,
    keys: Vec<u8>,
    key_offsets: Vec<i64>,
    sequences: Vec<u64>,
    value_validity: Vec<u8>,
    values: Vec<u8>,
    value_offsets: Vec<i64>,
    first_key: Vec<u8>,
    last_key: Vec<u8>,
    null_count: u32,
    minimum_key_bytes: u64,
    maximum_key_bytes: u64,
    minimum_value_bytes: u64,
    maximum_value_bytes: u64,
}

impl RowGroupBuilder {
    fn new(row_start: u64) -> Self {
        Self {
            row_start,
            key_offsets: vec![0],
            value_offsets: vec![0],
            minimum_key_bytes: u64::MAX,
            minimum_value_bytes: u64::MAX,
            ..Self::default()
        }
    }

    fn rows(&self) -> usize {
        self.sequences.len()
    }

    fn estimated_bytes(&self) -> usize {
        self.keys
            .len()
            .saturating_add(self.values.len())
            .saturating_add(self.key_offsets.len().saturating_mul(8))
            .saturating_add(self.value_offsets.len().saturating_mul(8))
            .saturating_add(self.sequences.len().saturating_mul(8))
            .saturating_add(self.sequences.len().div_ceil(8))
    }

    fn append(&mut self, key: &[u8], version: &VersionedValue) -> Result<()> {
        validate_key_value(key, version.value.as_deref().unwrap_or_default())?;
        if self.first_key.is_empty() {
            self.first_key.extend_from_slice(key);
        }
        self.last_key.clear();
        self.last_key.extend_from_slice(key);
        self.keys.extend_from_slice(key);
        self.key_offsets
            .push(i64::try_from(self.keys.len()).map_err(|_| {
                Error::InvalidSegment("v4 key page length exceeds signed Arrow offsets".into())
            })?);
        self.sequences.push(version.sequence);
        let row = self.rows() - 1;
        let byte = row / 8;
        if self.value_validity.len() <= byte {
            self.value_validity.resize(byte + 1, 0);
        }
        if version.value.is_some() {
            self.value_validity[byte] |= 1 << (row % 8);
        } else {
            self.null_count = self.null_count.saturating_add(1);
        }
        let value = version.value.as_deref().unwrap_or_default();
        self.values.extend_from_slice(value);
        self.value_offsets
            .push(i64::try_from(self.values.len()).map_err(|_| {
                Error::InvalidSegment("v4 value page length exceeds signed Arrow offsets".into())
            })?);
        let key_bytes = key.len() as u64;
        let value_bytes = value.len() as u64;
        self.minimum_key_bytes = self.minimum_key_bytes.min(key_bytes);
        self.maximum_key_bytes = self.maximum_key_bytes.max(key_bytes);
        self.minimum_value_bytes = self.minimum_value_bytes.min(value_bytes);
        self.maximum_value_bytes = self.maximum_value_bytes.max(value_bytes);
        Ok(())
    }
}

pub(super) fn encode(
    table: &Memtable,
    row_group_budget: SegmentRowGroupBudget,
) -> Result<EncodedSegment> {
    let row_group_budget = row_group_budget.validate()?;
    if table.version_count() == 0 {
        return invalid("cannot write an empty segment");
    }
    let entries = u64::try_from(table.version_count())
        .map_err(|_| Error::InvalidSegment("v4 entry count exceeds u64".into()))?;
    let minimum_sequence = table
        .all_versions()
        .flat_map(|(_, versions)| versions.iter().map(|version| version.sequence))
        .min()
        .expect("non-empty table has a minimum sequence");
    let maximum_sequence = table
        .all_versions()
        .flat_map(|(_, versions)| versions.iter().map(|version| version.sequence))
        .max()
        .expect("non-empty table has a maximum sequence");

    let mut builders = Vec::new();
    let mut current = RowGroupBuilder::new(0);
    for (key, versions) in table.all_versions() {
        let key_bytes = versions.iter().try_fold(0usize, |total, version| {
            let value = version.value.as_deref().unwrap_or_default();
            validate_key_value(key, value)?;
            total
                .checked_add(key.len())
                .and_then(|total| total.checked_add(value.len()))
                .and_then(|total| total.checked_add(24))
                .ok_or_else(|| Error::InvalidSegment("v4 row-group estimate overflow".into()))
        })?;
        if current.rows() > 0
            && (current.rows().saturating_add(versions.len()) > row_group_budget.max_rows
                || current.estimated_bytes().saturating_add(key_bytes)
                    > row_group_budget.target_bytes)
        {
            let next = current
                .row_start
                .checked_add(current.rows() as u64)
                .ok_or_else(|| Error::InvalidSegment("v4 row offset overflow".into()))?;
            builders.push(current);
            current = RowGroupBuilder::new(next);
        }
        for version in versions {
            current.append(key, version)?;
        }
    }
    if current.rows() > 0 {
        builders.push(current);
    }

    let mut output = vec![0; SEGMENT_HEADER_BYTES];
    let mut row_groups = Vec::with_capacity(builders.len());
    for builder in builders {
        let row_count = u32::try_from(builder.rows())
            .map_err(|_| Error::InvalidSegment("v4 row-group count exceeds u32".into()))?;
        let sequence_min = *builder
            .sequences
            .iter()
            .min()
            .expect("non-empty row group has sequences");
        let sequence_max = *builder
            .sequences
            .iter()
            .max()
            .expect("non-empty row group has sequences");
        let key_offsets = encode_i64_values(&builder.key_offsets);
        let sequences = encode_u64_values(&builder.sequences);
        let value_offsets = encode_i64_values(&builder.value_offsets);
        let pages = [
            append_page(
                &mut output,
                PageKind::KeyOffsets,
                builder.row_start,
                row_count,
                &key_offsets,
                PageStatistics::new(0, builder.minimum_key_bytes, builder.maximum_key_bytes),
            )?,
            append_page(
                &mut output,
                PageKind::KeyData,
                builder.row_start,
                row_count,
                &builder.keys,
                PageStatistics::new(0, builder.minimum_key_bytes, builder.maximum_key_bytes),
            )?,
            append_page(
                &mut output,
                PageKind::SequenceValues,
                builder.row_start,
                row_count,
                &sequences,
                PageStatistics::new(0, sequence_min, sequence_max),
            )?,
            append_page(
                &mut output,
                PageKind::ValueValidity,
                builder.row_start,
                row_count,
                &builder.value_validity,
                PageStatistics::new(builder.null_count, 0, 1),
            )?,
            append_page(
                &mut output,
                PageKind::ValueOffsets,
                builder.row_start,
                row_count,
                &value_offsets,
                PageStatistics::new(
                    builder.null_count,
                    builder.minimum_value_bytes,
                    builder.maximum_value_bytes,
                ),
            )?,
            append_page(
                &mut output,
                PageKind::ValueData,
                builder.row_start,
                row_count,
                &builder.values,
                PageStatistics::new(
                    builder.null_count,
                    builder.minimum_value_bytes,
                    builder.maximum_value_bytes,
                ),
            )?,
        ];
        row_groups.push(RowGroupDescriptor {
            row_start: builder.row_start,
            row_count,
            first_key: builder.first_key,
            last_key: builder.last_key,
            pages,
        });
    }

    align(&mut output);
    let index_offset = output.len() as u64;
    output.extend_from_slice(INDEX_MAGIC);
    output.extend_from_slice(&(row_groups.len() as u32).to_le_bytes());
    output.extend_from_slice(&(PAGES_PER_ROW_GROUP as u16).to_le_bytes());
    output.extend_from_slice(&(ROW_GROUP_HEADER_BYTES as u16).to_le_bytes());
    output.extend_from_slice(&(PAGE_DESCRIPTOR_BYTES as u16).to_le_bytes());
    output.extend_from_slice(&[0; 14]);
    for group in &row_groups {
        output.extend_from_slice(&group.row_start.to_le_bytes());
        output.extend_from_slice(&group.row_count.to_le_bytes());
        output.extend_from_slice(&(group.first_key.len() as u32).to_le_bytes());
        output.extend_from_slice(&(group.last_key.len() as u32).to_le_bytes());
        output.extend_from_slice(&(PAGES_PER_ROW_GROUP as u16).to_le_bytes());
        output.extend_from_slice(&[0; 10]);
        output.extend_from_slice(&group.first_key);
        output.extend_from_slice(&group.last_key);
        for page in &group.pages {
            encode_page_descriptor(&mut output, page);
        }
    }
    let index_bytes = output.len() as u64 - index_offset;
    if index_bytes as usize > MAX_INDEX_BYTES {
        return invalid("v4 index exceeds its bounded contract");
    }
    if output.len() as u64 > MAX_SEGMENT_BYTES - FOOTER_BYTES as u64 {
        return invalid("encoded v4 segment exceeds the 1 GiB safety limit");
    }

    output[..8].copy_from_slice(SEGMENT_MAGIC);
    output[8..10].copy_from_slice(&SEGMENT_FORMAT_VERSION.to_le_bytes());
    output[10..12].copy_from_slice(&(SEGMENT_HEADER_BYTES as u16).to_le_bytes());
    output[12..16].copy_from_slice(&0u32.to_le_bytes());
    output[16..24].copy_from_slice(&entries.to_le_bytes());
    output[24..32].copy_from_slice(&minimum_sequence.to_le_bytes());
    output[32..40].copy_from_slice(&maximum_sequence.to_le_bytes());
    output[40..44].copy_from_slice(&(row_groups.len() as u32).to_le_bytes());
    output[44..46].copy_from_slice(&(PAGES_PER_ROW_GROUP as u16).to_le_bytes());
    output[46..48].copy_from_slice(&(PAGE_ALIGNMENT as u16).to_le_bytes());
    output[48..56].copy_from_slice(&index_offset.to_le_bytes());
    output[56..64].copy_from_slice(&index_bytes.to_le_bytes());
    output[64..96].copy_from_slice(&decode_fixed_digest(SEGMENT_SCHEMA_DIGEST));
    output[96..128].copy_from_slice(&decode_fixed_digest(SEGMENT_KEY_CODEC_DIGEST));
    output[128..160].copy_from_slice(&decode_fixed_digest(SEGMENT_PAGE_FORMAT_DIGEST));
    output[160..168].copy_from_slice(&(row_group_budget.target_bytes as u64).to_le_bytes());
    output[168..176].copy_from_slice(&(row_group_budget.max_rows as u64).to_le_bytes());

    let checksum = sha256_hex(&output);
    output.extend_from_slice(checksum.as_bytes());
    Ok(EncodedSegment {
        bytes: output,
        checksum,
    })
}

pub(super) fn parse_metadata(
    reader: &mut (impl Read + Seek),
    physical_bytes: u64,
    checksum: String,
) -> Result<ParsedMetadata> {
    if physical_bytes < (SEGMENT_HEADER_BYTES + INDEX_HEADER_BYTES + FOOTER_BYTES) as u64 {
        return invalid("v4 segment is shorter than its framing");
    }
    let mut header = [0; SEGMENT_HEADER_BYTES];
    reader.seek(SeekFrom::Start(0))?;
    reader.read_exact(&mut header)?;
    require_current_format(&header)?;
    if read_u16(&header, 10)? as usize != SEGMENT_HEADER_BYTES
        || read_u32(&header, 12)? != 0
        || read_u16(&header, 44)? as usize != PAGES_PER_ROW_GROUP
        || read_u16(&header, 46)? as usize != PAGE_ALIGNMENT
        || header[176..].iter().any(|byte| *byte != 0)
    {
        return invalid("v4 segment header contains unsupported framing or flags");
    }
    let entries = read_u64(&header, 16)?;
    let minimum_sequence = read_u64(&header, 24)?;
    let maximum_sequence = read_u64(&header, 32)?;
    let row_group_count = read_u32(&header, 40)? as usize;
    let index_offset = read_u64(&header, 48)?;
    let index_bytes = usize::try_from(read_u64(&header, 56)?)
        .map_err(|_| Error::InvalidSegment("v4 index length exceeds usize".into()))?;
    let row_group_budget = decode_row_group_budget(&header)?;
    let content_end = physical_bytes - FOOTER_BYTES as u64;
    if entries == 0
        || minimum_sequence == 0
        || minimum_sequence > maximum_sequence
        || row_group_count == 0
        || index_offset < SEGMENT_HEADER_BYTES as u64
        || index_offset % PAGE_ALIGNMENT as u64 != 0
        || !(INDEX_HEADER_BYTES..=MAX_INDEX_BYTES).contains(&index_bytes)
        || index_offset.checked_add(index_bytes as u64) != Some(content_end)
    {
        return invalid("v4 segment header contract is invalid");
    }
    if encode_digest(&header[64..96]) != SEGMENT_SCHEMA_DIGEST
        || encode_digest(&header[96..128]) != SEGMENT_KEY_CODEC_DIGEST
        || encode_digest(&header[128..160]) != SEGMENT_PAGE_FORMAT_DIGEST
    {
        return invalid("v4 segment schema, key codec, or page format digest is unknown");
    }

    reader.seek(SeekFrom::Start(index_offset))?;
    let mut index = vec![0; index_bytes];
    reader.read_exact(&mut index)?;
    if index.get(..8) != Some(INDEX_MAGIC.as_slice())
        || read_u32(&index, 8)? as usize != row_group_count
        || read_u16(&index, 12)? as usize != PAGES_PER_ROW_GROUP
        || read_u16(&index, 14)? as usize != ROW_GROUP_HEADER_BYTES
        || read_u16(&index, 16)? as usize != PAGE_DESCRIPTOR_BYTES
        || index[18..32].iter().any(|byte| *byte != 0)
    {
        return invalid("v4 segment index header is invalid");
    }

    let mut cursor = INDEX_HEADER_BYTES;
    let mut row_groups = Vec::with_capacity(row_group_count);
    let mut expected_row_start = 0u64;
    let mut prior_last_key: Option<Vec<u8>> = None;
    let mut prior_page_end = SEGMENT_HEADER_BYTES as u64;
    for _ in 0..row_group_count {
        let fixed = slice(
            &index,
            cursor,
            ROW_GROUP_HEADER_BYTES,
            "v4 row-group header",
        )?;
        let row_start = read_u64(fixed, 0)?;
        let row_count = read_u32(fixed, 8)?;
        let first_key_bytes = read_u32(fixed, 12)? as usize;
        let last_key_bytes = read_u32(fixed, 16)? as usize;
        if row_start != expected_row_start
            || row_count == 0
            || read_u16(fixed, 20)? as usize != PAGES_PER_ROW_GROUP
            || fixed[22..32].iter().any(|byte| *byte != 0)
            || first_key_bytes == 0
            || first_key_bytes > MAX_KEY_BYTES
            || last_key_bytes == 0
            || last_key_bytes > MAX_KEY_BYTES
        {
            return invalid("v4 row-group framing is invalid");
        }
        cursor += ROW_GROUP_HEADER_BYTES;
        let first_key = slice(&index, cursor, first_key_bytes, "v4 first key")?.to_vec();
        cursor += first_key_bytes;
        let last_key = slice(&index, cursor, last_key_bytes, "v4 last key")?.to_vec();
        cursor += last_key_bytes;
        if first_key > last_key
            || prior_last_key
                .as_ref()
                .is_some_and(|prior| prior >= &first_key)
        {
            return invalid("v4 row-group key ranges are not strictly ordered");
        }
        let mut pages = Vec::with_capacity(PAGES_PER_ROW_GROUP);
        for expected_kind in PageKind::ORDERED {
            let encoded = slice(&index, cursor, PAGE_DESCRIPTOR_BYTES, "v4 page descriptor")?;
            let page = decode_page_descriptor(encoded)?;
            if page.kind != expected_kind
                || page.row_start != row_start
                || page.row_count != row_count
                || page.offset % PAGE_ALIGNMENT as u64 != 0
                || page.offset < prior_page_end
                || page.offset.saturating_sub(prior_page_end) >= PAGE_ALIGNMENT as u64
                || page
                    .offset
                    .checked_add(page.physical_bytes as u64)
                    .is_none_or(|end| end > index_offset)
            {
                return invalid("v4 page descriptor violates ordering or bounds");
            }
            prior_page_end = page.offset + page.physical_bytes as u64;
            pages.push(page);
            cursor += PAGE_DESCRIPTOR_BYTES;
        }
        validate_page_shapes(&pages, row_count)?;
        expected_row_start = expected_row_start
            .checked_add(u64::from(row_count))
            .ok_or_else(|| Error::InvalidSegment("v4 row count overflow".into()))?;
        prior_last_key = Some(last_key.clone());
        row_groups.push(RowGroupDescriptor {
            row_start,
            row_count,
            first_key,
            last_key,
            pages: pages.try_into().expect("v4 page count was fixed"),
        });
    }
    if cursor != index.len() || expected_row_start != entries {
        return invalid("v4 index does not exactly describe all rows");
    }
    validate_zero_padding(reader, &row_groups, index_offset)?;
    let first_key = row_groups
        .first()
        .expect("validated v4 segment has a row group")
        .first_key
        .clone();
    let last_key = row_groups
        .last()
        .expect("validated v4 segment has a row group")
        .last_key
        .clone();
    Ok(ParsedMetadata {
        descriptor: SegmentDescriptor {
            id: checksum.clone(),
            level: 0,
            format_version: SEGMENT_FORMAT_VERSION,
            schema_digest: SEGMENT_SCHEMA_DIGEST.into(),
            key_codec_digest: SEGMENT_KEY_CODEC_DIGEST.into(),
            page_format_digest: SEGMENT_PAGE_FORMAT_DIGEST.into(),
            first_key,
            last_key,
            minimum_sequence,
            maximum_sequence,
            entries,
            bytes: physical_bytes,
            checksum,
        },
        row_group_budget,
        row_groups,
    })
}

fn decode_row_group_budget(header: &[u8]) -> Result<SegmentRowGroupBudget> {
    let target_bytes = usize::try_from(read_u64(header, 160)?)
        .map_err(|_| Error::InvalidSegment("v4 row-group byte target exceeds usize".into()))?;
    let max_rows = usize::try_from(read_u64(header, 168)?)
        .map_err(|_| Error::InvalidSegment("v4 row-group row limit exceeds usize".into()))?;
    if target_bytes == 0
        || target_bytes as u64 > MAX_SEGMENT_BYTES
        || max_rows == 0
        || max_rows > u32::MAX as usize
    {
        return invalid("v4 segment row-group budget is invalid");
    }
    Ok(SegmentRowGroupBudget {
        max_rows,
        target_bytes,
    })
}

pub(super) fn require_current_format(bytes: &[u8]) -> Result<()> {
    if let Some(version) = match bytes.get(..8) {
        Some(b"RRDSEG01") => Some(1),
        Some(b"RRDSEG02") => Some(2),
        Some(b"RRDSEG03") => Some(3),
        _ => None,
    } {
        return Err(Error::UnsupportedVersion {
            object: "segment",
            version,
        });
    }
    let version = bytes
        .get(8..10)
        .map(|field| u16::from_le_bytes(field.try_into().expect("fixed version field")))
        .ok_or_else(|| Error::InvalidSegment("segment has no complete format header".into()))?;
    if version != SEGMENT_FORMAT_VERSION {
        return Err(Error::UnsupportedVersion {
            object: "segment",
            version,
        });
    }
    if bytes.get(..8) != Some(SEGMENT_MAGIC.as_slice()) {
        return invalid("segment magic does not match its format version");
    }
    Ok(())
}

fn append_page(
    output: &mut Vec<u8>,
    kind: PageKind,
    row_start: u64,
    row_count: u32,
    bytes: &[u8],
    statistics: PageStatistics,
) -> Result<PageDescriptor> {
    align(output);
    let offset = output.len() as u64;
    output.extend_from_slice(bytes);
    Ok(PageDescriptor {
        kind,
        row_start,
        row_count,
        null_count: statistics.null_count,
        offset,
        physical_bytes: bytes.len(),
        logical_bytes: bytes.len(),
        statistic_min: statistics.minimum,
        statistic_max: statistics.maximum,
        digest: sha256(bytes),
    })
}

fn encode_page_descriptor(output: &mut Vec<u8>, page: &PageDescriptor) {
    output.push(page.kind as u8);
    output.push(page.kind.column());
    output.push(page.kind.buffer_kind());
    output.push(page.kind.logical_type());
    output.push(page.kind.physical_type());
    output.push(0); // plain encoding
    output.push(0); // no compression
    output.push(0);
    output.extend_from_slice(&page.row_start.to_le_bytes());
    output.extend_from_slice(&page.row_count.to_le_bytes());
    output.extend_from_slice(&page.null_count.to_le_bytes());
    output.extend_from_slice(&page.offset.to_le_bytes());
    output.extend_from_slice(&(page.physical_bytes as u64).to_le_bytes());
    output.extend_from_slice(&(page.logical_bytes as u64).to_le_bytes());
    output.extend_from_slice(&page.statistic_min.to_le_bytes());
    output.extend_from_slice(&page.statistic_max.to_le_bytes());
    output.extend_from_slice(&page.digest);
}

fn decode_page_descriptor(bytes: &[u8]) -> Result<PageDescriptor> {
    let kind = PageKind::from_byte(bytes[0])?;
    if bytes[1] != kind.column()
        || bytes[2] != kind.buffer_kind()
        || bytes[3] != kind.logical_type()
        || bytes[4] != kind.physical_type()
        || bytes[5] != 0
        || bytes[6] != 0
        || bytes[7] != 0
    {
        return invalid("v4 page type, encoding, or compression metadata is unknown");
    }
    let physical_bytes = usize::try_from(read_u64(bytes, 32)?)
        .map_err(|_| Error::InvalidSegment("v4 physical page length exceeds usize".into()))?;
    let logical_bytes = usize::try_from(read_u64(bytes, 40)?)
        .map_err(|_| Error::InvalidSegment("v4 logical page length exceeds usize".into()))?;
    if physical_bytes != logical_bytes {
        return invalid("v4 uncompressed page has different physical and logical lengths");
    }
    Ok(PageDescriptor {
        kind,
        row_start: read_u64(bytes, 8)?,
        row_count: read_u32(bytes, 16)?,
        null_count: read_u32(bytes, 20)?,
        offset: read_u64(bytes, 24)?,
        physical_bytes,
        logical_bytes,
        statistic_min: read_u64(bytes, 48)?,
        statistic_max: read_u64(bytes, 56)?,
        digest: bytes[64..96].try_into().expect("fixed page digest"),
    })
}

fn validate_page_shapes(pages: &[PageDescriptor], rows: u32) -> Result<()> {
    let rows = rows as usize;
    let expected_offsets = rows
        .checked_add(1)
        .and_then(|count| count.checked_mul(8))
        .ok_or_else(|| Error::InvalidSegment("v4 offset page length overflow".into()))?;
    if pages[0].logical_bytes != expected_offsets
        || pages[2].logical_bytes != rows.saturating_mul(8)
        || pages[3].logical_bytes != rows.div_ceil(8)
        || pages[4].logical_bytes != expected_offsets
        || pages.iter().any(|page| page.null_count > page.row_count)
        || pages[0].null_count != 0
        || pages[1].null_count != 0
        || pages[2].null_count != 0
        || pages[3].null_count != pages[4].null_count
        || pages[3].null_count != pages[5].null_count
    {
        return invalid("v4 Arrow-compatible page lengths or null counts are invalid");
    }
    Ok(())
}

fn validate_zero_padding(
    reader: &mut (impl Read + Seek),
    row_groups: &[RowGroupDescriptor],
    index_offset: u64,
) -> Result<()> {
    let mut expected = SEGMENT_HEADER_BYTES as u64;
    for page in row_groups.iter().flat_map(|group| group.pages.iter()) {
        validate_zero_range(reader, expected, page.offset)?;
        expected = page
            .offset
            .checked_add(page.physical_bytes as u64)
            .ok_or_else(|| Error::InvalidSegment("v4 page end overflow".into()))?;
    }
    validate_zero_range(reader, expected, index_offset)
}

fn validate_zero_range(reader: &mut (impl Read + Seek), start: u64, end: u64) -> Result<()> {
    let length = usize::try_from(
        end.checked_sub(start)
            .ok_or_else(|| Error::InvalidSegment("v4 padding range is inverted".into()))?,
    )
    .map_err(|_| Error::InvalidSegment("v4 padding range exceeds usize".into()))?;
    if length >= PAGE_ALIGNMENT {
        return invalid("v4 page padding exceeds the alignment contract");
    }
    if length == 0 {
        return Ok(());
    }
    let mut padding = [0; PAGE_ALIGNMENT - 1];
    reader.seek(SeekFrom::Start(start))?;
    reader.read_exact(&mut padding[..length])?;
    if padding[..length].iter().any(|byte| *byte != 0) {
        return invalid("v4 page alignment padding is non-zero");
    }
    Ok(())
}

fn validate_key_value(key: &[u8], value: &[u8]) -> Result<()> {
    if key.is_empty() || key.len() > MAX_KEY_BYTES || value.len() > MAX_VALUE_BYTES {
        return invalid("key or value exceeds the v4 segment contract");
    }
    Ok(())
}

fn encode_i64_values(values: &[i64]) -> Vec<u8> {
    let mut output = Vec::with_capacity(values.len() * 8);
    for value in values {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output
}

fn encode_u64_values(values: &[u64]) -> Vec<u8> {
    let mut output = Vec::with_capacity(values.len() * 8);
    for value in values {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output
}

fn align(output: &mut Vec<u8>) {
    let aligned = output.len().next_multiple_of(PAGE_ALIGNMENT);
    output.resize(aligned, 0);
}

fn slice<'a>(bytes: &'a [u8], offset: usize, length: usize, name: &str) -> Result<&'a [u8]> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| Error::InvalidSegment(format!("{name} range overflow")))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| Error::InvalidSegment(format!("truncated {name}")))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        slice(bytes, offset, 2, "u16 field")?.try_into().unwrap(),
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        slice(bytes, offset, 4, "u32 field")?.try_into().unwrap(),
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        slice(bytes, offset, 8, "u64 field")?.try_into().unwrap(),
    ))
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .try_into()
        .expect("SHA-256 output is 32 bytes")
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    encode_digest(&sha256(bytes))
}

fn encode_digest(digest: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = Vec::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(HEX[(byte >> 4) as usize]);
        encoded.push(HEX[(byte & 0x0f) as usize]);
    }
    String::from_utf8(encoded).expect("hex digest is ASCII")
}

fn decode_fixed_digest(encoded: &str) -> [u8; 32] {
    let mut digest = [0; 32];
    let (pairs, remainder) = encoded.as_bytes().as_chunks::<2>();
    debug_assert!(remainder.is_empty());
    for (index, pair) in pairs.iter().enumerate() {
        digest[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    digest
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("fixed RRFlow digest is lowercase hexadecimal"),
    }
}

fn invalid<T>(reason: impl Into<String>) -> Result<T> {
    Err(Error::InvalidSegment(reason.into()))
}
