use crate::contract::invalid;
use crate::exact::validate_candidate_versions;
use crate::qjl::{QjlProjection, MAX_QJL_DIMENSIONS};
use crate::turboquant::{inner_product_packed_len, inner_product_padded_dimensions};
use crate::{
    AccessPathKind, CandidatePath, EmbeddingModelBinding, QuantizedKernel,
    QuantizedMemoryPlacement, ScoreMetric, SearchHit, SearchRequest, TurboQuantBits,
    TurboQuantEstimator, TurboQuantInnerProductCode, TurboQuantQuery, TurboQuantVector,
    VectorCandidate, VectorQuery,
};
use memmap2::{Mmap, MmapOptions};
use rrd_core::{
    digest, ProjectionId, ProjectionStamp, ProjectionState, Result, RuntimeProperties, RuntimeRef,
    ScopeId, VectorCollectionAddress, VectorValue, DATA_RUNTIME_CONTRACT_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, OnceLock};

pub const TURBOQUANT_SEGMENT_FORMAT_VERSION: u16 = 1;
const TURBOQUANT_SEGMENT_MAGIC: &[u8; 8] = b"RRDTQ001";
const HEADER_PREFIX_BYTES: usize = 8 + 2 + 4 + 8 + 64;
const MAX_TURBOQUANT_SEGMENT_BYTES: usize = 1 << 30;
const MAX_TURBOQUANT_CANDIDATES: usize = 10_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurboQuantSegmentConfig {
    pub id: ProjectionId,
    pub scope: ScopeId,
    pub field: String,
    pub dimensions: usize,
    pub metric: ScoreMetric,
    pub bits: TurboQuantBits,
    pub seed: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<EmbeddingModelBinding>,
    #[serde(default)]
    pub filter_properties: BTreeSet<String>,
    /// Absent for MSE, so an MSE configuration, and its digest, read as they
    /// did before the inner-product estimator existed.
    #[serde(default, skip_serializing_if = "TurboQuantEstimator::is_mse")]
    pub estimator: TurboQuantEstimator,
}

impl TurboQuantSegmentConfig {
    pub fn validate(&self) -> Result<()> {
        if self.field.trim().is_empty() || self.field.as_bytes().contains(&0) {
            return invalid("TurboQuant segment field must be non-empty and contain no NUL bytes");
        }
        if self.dimensions == 0 || self.dimensions > 1_048_576 {
            return invalid("TurboQuant segment dimensions must be in 1..=1048576");
        }
        if self
            .filter_properties
            .iter()
            .any(|property| property.trim().is_empty() || property.as_bytes().contains(&0))
        {
            return invalid("TurboQuant filter properties must be valid names");
        }
        if let Some(model) = &self.embedding_model {
            model.validate()?;
        }
        if self.estimator == TurboQuantEstimator::InnerProduct {
            if self.bits == TurboQuantBits::Bits1_5 {
                return invalid("1.5-bit TurboQuant has no inner-product estimator");
            }
            if self.metric == ScoreMetric::Manhattan {
                return invalid(
                    "the TurboQuant inner-product estimator scores dot, cosine and euclidean metrics",
                );
            }
            if inner_product_padded_dimensions(self.dimensions) > MAX_QJL_DIMENSIONS {
                return invalid(format!(
                    "the TurboQuant inner-product estimator accepts at most {MAX_QJL_DIMENSIONS} padded dimensions"
                ));
            }
        }
        Ok(())
    }

    /// S for this configuration, which an inner-product build and search use.
    fn projection(&self) -> Result<Option<QjlProjection>> {
        match self.estimator {
            TurboQuantEstimator::Mse => Ok(None),
            TurboQuantEstimator::InnerProduct => {
                QjlProjection::new(inner_product_padded_dimensions(self.dimensions), self.seed)
                    .map(Some)
            }
        }
    }

    /// The digest a generation's stamp carries as its configuration digest.
    pub fn digest(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_vec(self)
            .map(|bytes| digest::sha256_hex(&bytes))
            .map_err(|error| rrd_core::Error::InvalidRuntime {
                reason: format!("TurboQuant configuration cannot be encoded: {error}"),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurboQuantDescriptor {
    pub stamp: ProjectionStamp,
    pub scope: ScopeId,
    pub field: String,
    pub dimensions: usize,
    pub metric: ScoreMetric,
    pub bits: TurboQuantBits,
    pub seed: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<EmbeddingModelBinding>,
    #[serde(default)]
    pub filter_properties: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "TurboQuantEstimator::is_mse")]
    pub estimator: TurboQuantEstimator,
    pub minimum_cursor: u64,
    pub candidate_versions: usize,
    pub packed_vector_bytes: usize,
    pub full_precision_vector_bytes: usize,
}

impl TurboQuantDescriptor {
    pub fn validate(&self) -> Result<()> {
        self.stamp.validate()?;
        if self.minimum_cursor != 0
            || self.minimum_cursor > self.stamp.source_cursor
            || self.dimensions == 0
            || self.candidate_versions > MAX_TURBOQUANT_CANDIDATES
            || self.packed_vector_bytes == 0
            || self.full_precision_vector_bytes
                != self
                    .candidate_versions
                    .saturating_mul(self.dimensions)
                    .saturating_mul(std::mem::size_of::<f32>())
        {
            return invalid("TurboQuant descriptor coverage or byte accounting is invalid");
        }
        if let Some(model) = &self.embedding_model {
            model.validate()?;
        }
        Ok(())
    }

    pub fn candidate_path(&self, estimated_cost: u64) -> CandidatePath {
        CandidatePath {
            stamp: self.stamp.clone(),
            kind: AccessPathKind::TurboQuant,
            field: self.field.clone(),
            dimensions: self.dimensions,
            metric: self.metric,
            embedding_model: self.embedding_model.clone(),
            filter_properties: self.filter_properties.clone(),
            estimated_candidates: self.candidate_versions as u64,
            estimated_cost,
            overlay_source_cursor: None,
            overlay_candidates: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredCandidate {
    reference: RuntimeRef,
    subject: RuntimeRef,
    collection: VectorCollectionAddress,
    source_cursor: u64,
    valid_from: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    valid_to: Option<u64>,
    #[serde(default)]
    properties: RuntimeProperties,
    original_norm: f32,
    centroid_norm: f32,
    /// The unit residual's length, under the inner-product estimator only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    residual_norm: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SegmentHeader {
    config: TurboQuantSegmentConfig,
    generation: u64,
    source_cursor: u64,
    minimum_cursor: u64,
    packed_bytes_per_vector: usize,
    candidates: Vec<StoredCandidate>,
}

#[derive(Debug, Clone)]
enum TurboQuantStorage {
    Owned(Arc<[u8]>),
    Mapped(Arc<Mmap>),
}

impl TurboQuantStorage {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Mapped(bytes) => bytes,
        }
    }

    fn placement(&self) -> QuantizedMemoryPlacement {
        match self {
            Self::Owned(_) => QuantizedMemoryPlacement::Owned,
            Self::Mapped(_) => QuantizedMemoryPlacement::Mapped,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurboQuantSegment {
    descriptor: TurboQuantDescriptor,
    candidates: Vec<StoredCandidate>,
    packed_bytes_per_vector: usize,
    payload_offset: usize,
    payload_len: usize,
    storage: TurboQuantStorage,
    /// S for an inner-product generation, materialized on its first search.
    projection: Arc<OnceLock<QjlProjection>>,
}

impl PartialEq for TurboQuantSegment {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor && self.as_bytes() == other.as_bytes()
    }
}

impl TurboQuantSegment {
    pub fn build(
        config: TurboQuantSegmentConfig,
        generation: u64,
        source_cursor: u64,
        candidates: impl IntoIterator<Item = VectorCandidate>,
    ) -> Result<Self> {
        config.validate()?;
        if generation == 0 {
            return invalid("TurboQuant segment generation must be greater than zero");
        }
        let mut candidates = candidates.into_iter().collect::<Vec<_>>();
        if candidates.is_empty() || candidates.len() > MAX_TURBOQUANT_CANDIDATES {
            return invalid("TurboQuant segment candidate count is outside bounds");
        }
        validate_candidate_versions(&candidates)?;
        sort_candidates(&mut candidates);
        let projection = config.projection()?;
        let mut versions = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            if candidate.source_cursor > source_cursor {
                return invalid("TurboQuant candidate violates configuration or coverage");
            }
            versions.push(encode_candidate(&config, projection.as_ref(), candidate)?);
        }
        Self::assemble(config, generation, source_cursor, versions, projection)
    }

    /// The next generation of `previous`, from the identities that changed
    /// since it.
    ///
    /// `replaced` names every identity whose history in this source changed
    /// after `previous`. `candidates` holds each such identity's complete
    /// history at the new read, limited to this source. Every other identity
    /// keeps its stored versions and their codes. The codec encodes each
    /// vector on its own, with the configuration's seed and fixed codebook, so
    /// re-encoding a kept version would give the same bytes. The source cursor
    /// is the newest version the generation holds, as for a full build, and
    /// the result is byte for byte the segment `build` makes from the same
    /// versions.
    pub fn extend(
        previous: &Self,
        config: TurboQuantSegmentConfig,
        generation: u64,
        replaced: &BTreeSet<RuntimeRef>,
        candidates: impl IntoIterator<Item = VectorCandidate>,
    ) -> Result<Self> {
        config.validate()?;
        if config.digest()? != previous.descriptor.stamp.config_digest {
            return invalid("a TurboQuant generation extends only one with its configuration");
        }
        if generation <= previous.descriptor.stamp.generation {
            return invalid("a TurboQuant generation extends only an earlier generation");
        }
        let mut candidates = candidates.into_iter().collect::<Vec<_>>();
        validate_candidate_versions(&candidates)?;
        if candidates
            .iter()
            .any(|candidate| !replaced.contains(&candidate.vector.reference))
        {
            return invalid("an unchanged identity keeps its stored TurboQuant versions");
        }
        sort_candidates(&mut candidates);
        let payload = previous.payload();
        let width = previous.packed_bytes_per_vector;
        let mut versions = previous
            .candidates
            .iter()
            .zip(payload.chunks_exact(width))
            .filter(|(stored, _)| !replaced.contains(&stored.reference))
            .map(|(stored, packed)| (stored.clone(), packed.to_vec()))
            .collect::<Vec<_>>();
        let projection = config.projection()?;
        for candidate in candidates {
            versions.push(encode_candidate(&config, projection.as_ref(), candidate)?);
        }
        versions.sort_by(|(left, _), (right, _)| {
            left.reference
                .cmp(&right.reference)
                .then_with(|| left.source_cursor.cmp(&right.source_cursor))
        });
        if versions.is_empty() || versions.len() > MAX_TURBOQUANT_CANDIDATES {
            return invalid("TurboQuant segment candidate count is outside bounds");
        }
        let source_cursor = versions
            .iter()
            .map(|(stored, _)| stored.source_cursor)
            .max()
            .unwrap_or_default();
        Self::assemble(config, generation, source_cursor, versions, projection)
    }

    /// The artifact for `versions`, already in reference and cursor order,
    /// each with its packed codes.
    fn assemble(
        config: TurboQuantSegmentConfig,
        generation: u64,
        source_cursor: u64,
        versions: Vec<(StoredCandidate, Vec<u8>)>,
        projection: Option<QjlProjection>,
    ) -> Result<Self> {
        let mut stored = Vec::with_capacity(versions.len());
        let mut payload = Vec::new();
        let mut packed_bytes_per_vector = None;
        for (candidate, packed) in versions {
            match packed_bytes_per_vector {
                Some(expected) if expected != packed.len() => {
                    return invalid("TurboQuant vectors produced inconsistent packed lengths");
                }
                None => packed_bytes_per_vector = Some(packed.len()),
                _ => {}
            }
            payload.extend_from_slice(&packed);
            stored.push(candidate);
        }
        let header = SegmentHeader {
            config,
            generation,
            source_cursor,
            minimum_cursor: 0,
            packed_bytes_per_vector: packed_bytes_per_vector.unwrap_or_default(),
            candidates: stored,
        };
        let header_bytes = encode_json(&header)?;
        let artifact_digest = artifact_digest(&header_bytes, &payload);
        let bytes = encode_artifact(&header_bytes, &payload, &artifact_digest)?;
        let segment = Self::decode(TurboQuantStorage::Owned(Arc::from(bytes)))?;
        // The build already holds S; the segment keeps it for its searches.
        if let Some(projection) = projection {
            let _ = segment.projection.set(projection);
        }
        Ok(segment)
    }

    fn payload(&self) -> &[u8] {
        &self.as_bytes()[self.payload_offset..self.payload_offset + self.payload_len]
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::decode(TurboQuantStorage::Owned(Arc::from(bytes)))
    }

    pub fn open_mmap(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|error| {
            runtime_error(format!(
                "cannot open TurboQuant artifact {}: {error}",
                path.display()
            ))
        })?;
        Self::open_mmap_file(file)
    }

    /// Maps the already admitted immutable file without reopening a pathname.
    pub fn open_mmap_file(file: File) -> Result<Self> {
        let length = file
            .metadata()
            .map_err(|error| runtime_error(format!("cannot stat TurboQuant artifact: {error}")))?
            .len();
        if length == 0 || length > MAX_TURBOQUANT_SEGMENT_BYTES as u64 {
            return invalid("TurboQuant artifact file length is outside safety bounds");
        }
        // SAFETY: the file is opened read-only and its bounded non-zero length
        // was checked. Published artifact files are immutable.
        let mmap = unsafe { MmapOptions::new().map(&file) }
            .map_err(|error| runtime_error(format!("cannot map TurboQuant artifact: {error}")))?;
        Self::decode(TurboQuantStorage::Mapped(Arc::new(mmap)))
    }

    fn decode(storage: TurboQuantStorage) -> Result<Self> {
        let bytes = storage.as_bytes();
        if bytes.len() < HEADER_PREFIX_BYTES || bytes.len() > MAX_TURBOQUANT_SEGMENT_BYTES {
            return invalid("TurboQuant artifact byte length is outside bounds");
        }
        if &bytes[..8] != TURBOQUANT_SEGMENT_MAGIC {
            return invalid("TurboQuant artifact magic is invalid");
        }
        let version = u16::from_le_bytes(bytes[8..10].try_into().expect("fixed version slice"));
        if version != TURBOQUANT_SEGMENT_FORMAT_VERSION {
            return invalid("TurboQuant artifact format version is unsupported");
        }
        let header_len = u32::from_le_bytes(bytes[10..14].try_into().expect("fixed header slice"));
        let payload_len =
            u64::from_le_bytes(bytes[14..22].try_into().expect("fixed payload slice"));
        let header_len =
            usize::try_from(header_len).map_err(|_| rrd_core::Error::InvalidRuntime {
                reason: "TurboQuant header length overflows this host".into(),
            })?;
        let payload_len =
            usize::try_from(payload_len).map_err(|_| rrd_core::Error::InvalidRuntime {
                reason: "TurboQuant payload length overflows this host".into(),
            })?;
        let expected_len = HEADER_PREFIX_BYTES
            .checked_add(header_len)
            .and_then(|value| value.checked_add(payload_len))
            .ok_or_else(|| rrd_core::Error::InvalidRuntime {
                reason: "TurboQuant artifact length overflows".into(),
            })?;
        if bytes.len() != expected_len {
            return invalid("TurboQuant artifact framing length is inconsistent");
        }
        let advertised =
            std::str::from_utf8(&bytes[22..86]).map_err(|_| rrd_core::Error::InvalidRuntime {
                reason: "TurboQuant digest is not ASCII".into(),
            })?;
        let header_bytes = &bytes[HEADER_PREFIX_BYTES..HEADER_PREFIX_BYTES + header_len];
        let payload = &bytes[HEADER_PREFIX_BYTES + header_len..];
        let actual = artifact_digest(header_bytes, payload);
        if advertised != actual {
            return invalid("TurboQuant artifact digest does not match its bytes");
        }
        let header: SegmentHeader = serde_json::from_slice(header_bytes).map_err(|error| {
            rrd_core::Error::InvalidRuntime {
                reason: format!("TurboQuant header cannot be decoded: {error}"),
            }
        })?;
        if encode_json(&header)? != header_bytes {
            return invalid("TurboQuant header is not in canonical encoding");
        }
        header.config.validate()?;
        if header.generation == 0
            || header.minimum_cursor != 0
            || header.candidates.is_empty()
            || header.candidates.len() > MAX_TURBOQUANT_CANDIDATES
            || header.packed_bytes_per_vector == 0
            || payload_len
                != header
                    .candidates
                    .len()
                    .saturating_mul(header.packed_bytes_per_vector)
        {
            return invalid("TurboQuant header coverage or payload length is invalid");
        }
        let inner_product = header.config.estimator == TurboQuantEstimator::InnerProduct;
        if inner_product
            && Some(header.packed_bytes_per_vector)
                != inner_product_packed_len(
                    header.config.bits,
                    inner_product_padded_dimensions(header.config.dimensions),
                )
        {
            return invalid("TurboQuant inner-product code width differs from its configuration");
        }
        let mut versions = BTreeSet::new();
        for candidate in &header.candidates {
            let estimator_norms = if inner_product {
                candidate.centroid_norm >= 0.0
                    && candidate
                        .residual_norm
                        .is_some_and(|residual| residual.is_finite() && residual >= 0.0)
            } else {
                candidate.centroid_norm > 0.0 && candidate.residual_norm.is_none()
            };
            if candidate.source_cursor == 0
                || candidate.source_cursor > header.source_cursor
                || !candidate.original_norm.is_finite()
                || candidate.original_norm < 0.0
                || !candidate.centroid_norm.is_finite()
                || !estimator_norms
                || !versions.insert((candidate.reference.clone(), candidate.source_cursor))
            {
                return invalid("TurboQuant candidate metadata is invalid");
            }
        }
        let descriptor = TurboQuantDescriptor {
            stamp: ProjectionStamp {
                contract_version: DATA_RUNTIME_CONTRACT_VERSION,
                id: header.config.id.clone(),
                generation: header.generation,
                source_cursor: header.source_cursor,
                config_digest: header.config.digest()?,
                artifact_digest: actual,
                state: ProjectionState::Ready,
            },
            scope: header.config.scope,
            field: header.config.field,
            dimensions: header.config.dimensions,
            metric: header.config.metric,
            bits: header.config.bits,
            seed: header.config.seed,
            embedding_model: header.config.embedding_model,
            filter_properties: header.config.filter_properties,
            estimator: header.config.estimator,
            minimum_cursor: header.minimum_cursor,
            candidate_versions: header.candidates.len(),
            packed_vector_bytes: payload_len,
            full_precision_vector_bytes: header
                .candidates
                .len()
                .saturating_mul(header.config.dimensions)
                .saturating_mul(std::mem::size_of::<f32>()),
        };
        descriptor.validate()?;
        Ok(Self {
            descriptor,
            candidates: header.candidates,
            packed_bytes_per_vector: header.packed_bytes_per_vector,
            payload_offset: HEADER_PREFIX_BYTES + header_len,
            payload_len,
            storage,
            projection: Arc::new(OnceLock::new()),
        })
    }

    pub fn descriptor(&self) -> &TurboQuantDescriptor {
        &self.descriptor
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.storage.as_bytes()
    }

    pub fn memory_placement(&self) -> QuantizedMemoryPlacement {
        self.storage.placement()
    }

    pub fn search_candidates_at(
        &self,
        request: &SearchRequest,
        candidate_limit: usize,
        required_source_cursor: u64,
    ) -> Result<Vec<SearchHit>> {
        self.search_candidates_at_with_kernel(
            request,
            candidate_limit,
            required_source_cursor,
            QuantizedKernel::Auto,
        )
    }

    pub fn search_candidates_at_with_kernel(
        &self,
        request: &SearchRequest,
        candidate_limit: usize,
        required_source_cursor: u64,
        kernel: QuantizedKernel,
    ) -> Result<Vec<SearchHit>> {
        request.validate()?;
        self.descriptor.validate()?;
        let VectorQuery::Dense { values } = &request.query else {
            return invalid("TurboQuant artifact requires a dense query");
        };
        let required_properties = request
            .filter
            .as_ref()
            .map(|filter| {
                filter
                    .referenced_properties()
                    .into_iter()
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if candidate_limit < request.top_k
            || required_source_cursor > request.read.commit_cursor
            || self.descriptor.stamp.state != ProjectionState::Ready
            || self.descriptor.scope != request.scope
            || self.descriptor.field != request.field
            || self.descriptor.metric != request.metric
            || self.descriptor.embedding_model != request.embedding_model
            || self.descriptor.dimensions != values.len()
            || self.descriptor.stamp.source_cursor < required_source_cursor
            || !required_properties.is_subset(&self.descriptor.filter_properties)
        {
            return invalid("TurboQuant artifact does not satisfy the search contract");
        }
        let mut latest = BTreeMap::<&RuntimeRef, usize>::new();
        for (index, candidate) in self.candidates.iter().enumerate() {
            if candidate.source_cursor > request.read.commit_cursor
                || candidate.valid_from > request.valid_at
            {
                continue;
            }
            if latest.get(&candidate.reference).is_none_or(|current| {
                self.candidates[*current].source_cursor < candidate.source_cursor
            }) {
                latest.insert(&candidate.reference, index);
            }
        }
        let payload = &self.as_bytes()[self.payload_offset..self.payload_offset + self.payload_len];
        let query = match self.descriptor.estimator {
            TurboQuantEstimator::Mse => TurboQuantQuery::mse(
                values,
                request.metric,
                self.descriptor.bits,
                self.descriptor.seed,
            )?,
            TurboQuantEstimator::InnerProduct => TurboQuantQuery::inner_product(
                values,
                request.metric,
                self.descriptor.bits,
                self.descriptor.seed,
                self.projection()?,
            )?,
        };
        let mut hits = Vec::new();
        for index in latest.into_values() {
            let candidate = &self.candidates[index];
            if candidate
                .valid_to
                .is_some_and(|valid_to| request.valid_at >= valid_to)
                || request
                    .filter
                    .as_ref()
                    .is_some_and(|filter| !filter.matches(&candidate.properties))
            {
                continue;
            }
            let start = index * self.packed_bytes_per_vector;
            let packed = &payload[start..start + self.packed_bytes_per_vector];
            let score = match candidate.residual_norm {
                None => query.score_mse(
                    candidate.original_norm,
                    candidate.centroid_norm,
                    packed,
                    kernel,
                )?,
                Some(residual_norm) => query.score_inner_product(
                    candidate.original_norm,
                    residual_norm,
                    packed,
                    kernel,
                )?,
            };
            hits.push(SearchHit {
                reference: candidate.reference.clone(),
                subject: candidate.subject.clone(),
                source_cursor: candidate.source_cursor,
                score,
            });
        }
        hits.sort_by(SearchHit::compare_best_first);
        hits.truncate(candidate_limit);
        Ok(hits)
    }

    /// S for this inner-product generation, materialized once and kept.
    fn projection(&self) -> Result<&QjlProjection> {
        if let Some(projection) = self.projection.get() {
            return Ok(projection);
        }
        let projection = QjlProjection::new(
            inner_product_padded_dimensions(self.descriptor.dimensions),
            self.descriptor.seed,
        )?;
        // A concurrent search may have set it first; both drew the same S.
        let _ = self.projection.set(projection);
        self.projection
            .get()
            .map_or_else(|| invalid("the QJL projection could not be kept"), Ok)
    }
}

fn sort_candidates(candidates: &mut [VectorCandidate]) {
    candidates.sort_by(|left, right| {
        left.vector
            .reference
            .cmp(&right.vector.reference)
            .then_with(|| left.source_cursor.cmp(&right.source_cursor))
    });
}

/// One version as a generation stores it, with its packed codes.
fn encode_candidate(
    config: &TurboQuantSegmentConfig,
    projection: Option<&QjlProjection>,
    candidate: VectorCandidate,
) -> Result<(StoredCandidate, Vec<u8>)> {
    if candidate.scope != config.scope
        || candidate.vector.field != config.field
        || candidate.vector.value.dimensions() != config.dimensions
        || !candidate.matches_model(config.embedding_model.as_ref())
    {
        return invalid("TurboQuant candidate violates configuration or coverage");
    }
    let VectorValue::Dense { values } = &candidate.vector.value else {
        return invalid("TurboQuant artifacts currently require dense vectors");
    };
    let (original_norm, centroid_norm, residual_norm, packed) = match projection {
        None => {
            let encoded = TurboQuantVector::encode(values, config.bits, config.seed)?;
            (
                encoded.original_norm,
                encoded.centroid_norm,
                None,
                encoded.packed,
            )
        }
        Some(projection) => {
            let encoded =
                TurboQuantInnerProductCode::encode(values, config.bits, config.seed, projection)?;
            (
                encoded.original_norm,
                encoded.centroid_norm,
                Some(encoded.residual_norm),
                encoded.packed,
            )
        }
    };
    Ok((
        StoredCandidate {
            reference: candidate.vector.reference,
            subject: candidate.vector.subject,
            collection: candidate.vector.collection,
            source_cursor: candidate.source_cursor,
            valid_from: candidate.vector.valid_from,
            valid_to: candidate.vector.valid_to,
            properties: candidate.vector.properties,
            original_norm,
            centroid_norm,
            residual_norm,
        },
        packed,
    ))
}

fn runtime_error(reason: impl Into<String>) -> rrd_core::Error {
    rrd_core::Error::InvalidRuntime {
        reason: reason.into(),
    }
}

fn encode_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| rrd_core::Error::InvalidRuntime {
        reason: format!("TurboQuant artifact cannot be encoded: {error}"),
    })
}

fn artifact_digest(header: &[u8], payload: &[u8]) -> String {
    let mut bytes = b"rrflow-turboquant-segment-v1\0".to_vec();
    bytes.extend_from_slice(header);
    bytes.extend_from_slice(payload);
    digest::sha256_hex(&bytes)
}

fn encode_artifact(header: &[u8], payload: &[u8], artifact_digest: &str) -> Result<Vec<u8>> {
    let header_len = u32::try_from(header.len()).map_err(|_| rrd_core::Error::InvalidRuntime {
        reason: "TurboQuant header exceeds u32".into(),
    })?;
    let payload_len =
        u64::try_from(payload.len()).map_err(|_| rrd_core::Error::InvalidRuntime {
            reason: "TurboQuant payload exceeds u64".into(),
        })?;
    let mut bytes = Vec::with_capacity(HEADER_PREFIX_BYTES + header.len() + payload.len());
    bytes.extend_from_slice(TURBOQUANT_SEGMENT_MAGIC);
    bytes.extend_from_slice(&TURBOQUANT_SEGMENT_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&header_len.to_le_bytes());
    bytes.extend_from_slice(&payload_len.to_le_bytes());
    bytes.extend_from_slice(artifact_digest.as_bytes());
    bytes.extend_from_slice(header);
    bytes.extend_from_slice(payload);
    if bytes.len() > MAX_TURBOQUANT_SEGMENT_BYTES {
        return invalid("TurboQuant artifact exceeds the 1 GiB safety limit");
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FilterCondition, FilterExpression, FilterOperator, SearchMode};
    use rrd_core::{ReadStamp, RuntimeValue, RuntimeVector, VectorValue};

    fn config(scope: &ScopeId) -> TurboQuantSegmentConfig {
        TurboQuantSegmentConfig {
            id: ProjectionId::new("vector:turbo-body").unwrap(),
            scope: scope.clone(),
            field: "body".into(),
            dimensions: 64,
            metric: ScoreMetric::Cosine,
            bits: TurboQuantBits::Bits4,
            seed: 91,
            embedding_model: None,
            filter_properties: BTreeSet::from(["tenant".into()]),
            estimator: TurboQuantEstimator::Mse,
        }
    }

    fn candidate(scope: &ScopeId, cursor: u64, id: &str, tenant: &str) -> VectorCandidate {
        VectorCandidate {
            scope: scope.clone(),
            source_cursor: cursor,
            vector: RuntimeVector {
                reference: RuntimeRef::new("embedding", id).unwrap(),
                subject: RuntimeRef::new("document", id).unwrap(),
                collection: VectorCollectionAddress {
                    collection_id: "documents".into(),
                    vector_name: "body".into(),
                },
                field: "body".into(),
                valid_from: 1,
                valid_to: None,
                value: VectorValue::Dense {
                    values: deterministic_vector(64, cursor + 10),
                },
                provenance: None,
                properties: RuntimeProperties::from([(
                    "tenant".into(),
                    RuntimeValue::String(tenant.into()),
                )]),
            },
        }
    }

    /// `id`'s version at `cursor`, ending at `valid_to` when it was superseded
    /// or retired.
    fn version(scope: &ScopeId, cursor: u64, id: &str, valid_to: Option<u64>) -> VectorCandidate {
        let mut candidate = candidate(scope, cursor, id, "a");
        candidate.vector.valid_to = valid_to;
        candidate
    }

    fn refs(ids: &[&str]) -> BTreeSet<RuntimeRef> {
        ids.iter()
            .map(|id| RuntimeRef::new("embedding", *id).unwrap())
            .collect()
    }

    /// MSE artifacts keep the bytes they had before the inner-product
    /// estimator existed: these digests and scores come from the codec at
    /// E-04c (`69c36845`), on the same versions.
    #[test]
    fn mse_artifacts_keep_their_bytes_and_scores() {
        let scope = ScopeId::new("instance:probe").unwrap();
        let candidates = (1..=40u64)
            .map(|cursor| VectorCandidate {
                scope: scope.clone(),
                source_cursor: cursor,
                vector: RuntimeVector {
                    reference: RuntimeRef::new("embedding", format!("p{cursor}")).unwrap(),
                    subject: RuntimeRef::new("document", format!("p{cursor}")).unwrap(),
                    collection: VectorCollectionAddress {
                        collection_id: "documents".into(),
                        vector_name: "body".into(),
                    },
                    field: "body".into(),
                    valid_from: 1,
                    valid_to: None,
                    value: VectorValue::Dense {
                        values: (0..48)
                            .map(|dimension| {
                                (((cursor * 37 + dimension * 11) % 101) as f32) / 50.0 - 1.0
                            })
                            .collect(),
                    },
                    provenance: None,
                    properties: RuntimeProperties::new(),
                },
            })
            .collect::<Vec<_>>();
        for (bits, digest, top) in [
            (
                TurboQuantBits::Bits4,
                "e29b417205f42a2ee95e932201784ceb9cccfb17c003b11d8b537c7cba20cc52",
                ("p35", 1.205_116_994_953_018_3),
            ),
            (
                TurboQuantBits::Bits2,
                "b1d05c52291448dc675e394ec7cc618b0546fcf3c3586f8033cde196733f8286",
                ("p5", 1.626_582_098_861_267_1),
            ),
            (
                TurboQuantBits::Bits1_5,
                "e1c691b939bd8e872d20f1096a3526861d0cc013352e0a83e5cdf592ba978e1a",
                ("p5", 1.633_507_625_007_59),
            ),
            (
                TurboQuantBits::Bits1,
                "32e3b64ef9cec42a5c4c287bfc6f939f44a615b9a545b3a846ced11cdfea9157",
                ("p5", 1.519_326_381_657_862_8),
            ),
        ] {
            let segment = TurboQuantSegment::build(
                TurboQuantSegmentConfig {
                    id: ProjectionId::new("probe").unwrap(),
                    scope: scope.clone(),
                    field: "body".into(),
                    dimensions: 48,
                    metric: ScoreMetric::Dot,
                    bits,
                    seed: 5,
                    embedding_model: None,
                    filter_properties: BTreeSet::new(),
                    estimator: TurboQuantEstimator::Mse,
                },
                1,
                40,
                candidates.clone(),
            )
            .unwrap();
            assert_eq!(
                segment.descriptor().stamp.artifact_digest,
                digest,
                "{bits:?}"
            );
            let hits = segment
                .search_candidates_at(
                    &SearchRequest {
                        scope: scope.clone(),
                        read: ReadStamp::new(scope.clone(), None, 0, 40, Some("11".repeat(32)))
                            .unwrap(),
                        valid_at: 2,
                        field: "body".into(),
                        metric: ScoreMetric::Dot,
                        embedding_model: None,
                        query: VectorQuery::Dense {
                            values: (0..48)
                                .map(|dimension| dimension as f32 / 48.0 - 0.5)
                                .collect(),
                        },
                        filter: None,
                        top_k: 5,
                        mode: SearchMode::RequireApproximate { exact_rerank: 40 },
                    },
                    40,
                    40,
                )
                .unwrap();
            assert_eq!(hits[0].reference.id.as_str(), top.0, "{bits:?}");
            assert!(
                (hits[0].score - top.1).abs() < 1e-9,
                "{bits:?}: {}",
                hits[0].score
            );
        }
    }

    #[test]
    fn an_inner_product_configuration_refuses_what_its_estimator_cannot_serve() {
        let scope = ScopeId::new("instance:turbo-inner-product-configuration").unwrap();
        let inner_product = |change: &dyn Fn(&mut TurboQuantSegmentConfig)| {
            let mut config = config(&scope);
            config.estimator = TurboQuantEstimator::InnerProduct;
            change(&mut config);
            config.validate()
        };
        assert!(inner_product(&|_| {}).is_ok());
        assert!(inner_product(&|config| config.bits = TurboQuantBits::Bits1_5).is_err());
        assert!(inner_product(&|config| config.metric = ScoreMetric::Manhattan).is_err());
        assert!(inner_product(&|config| config.dimensions = MAX_QJL_DIMENSIONS).is_ok());
        assert!(inner_product(&|config| config.dimensions = MAX_QJL_DIMENSIONS + 1).is_err());
        // The MSE estimator keeps every width and metric.
        let mut mse = config(&scope);
        mse.bits = TurboQuantBits::Bits1_5;
        mse.metric = ScoreMetric::Manhattan;
        assert!(mse.validate().is_ok());
    }

    #[test]
    fn extending_a_generation_equals_a_full_build() {
        let scope = ScopeId::new("instance:turbo-extend").unwrap();
        let first = (1..=6)
            .map(|cursor| version(&scope, cursor, &format!("point-{cursor}"), None))
            .collect::<Vec<_>>();
        let base = TurboQuantSegment::build(config(&scope), 1, 6, first.clone()).unwrap();

        // Since the base: point-2 is updated, point-3 is retired, point-4
        // leaves this source (its history here is unchanged), and point-7 is
        // new. Each changed identity arrives with its complete history.
        let histories = vec![
            version(&scope, 2, "point-2", Some(7)),
            version(&scope, 7, "point-2", None),
            version(&scope, 3, "point-3", Some(8)),
            version(&scope, 4, "point-4", None),
            version(&scope, 9, "point-7", None),
        ];
        let replaced = refs(&["point-2", "point-3", "point-4", "point-7"]);
        let extended =
            TurboQuantSegment::extend(&base, config(&scope), 2, &replaced, histories.clone())
                .unwrap();

        let unchanged = first
            .into_iter()
            .filter(|candidate| !replaced.contains(&candidate.vector.reference));
        let full =
            TurboQuantSegment::build(config(&scope), 2, 9, unchanged.chain(histories)).unwrap();
        assert_eq!(extended.descriptor(), full.descriptor());
        assert_eq!(extended.as_bytes(), full.as_bytes());
        assert_eq!(extended.descriptor().candidate_versions, 8);
        assert_eq!(extended.descriptor().stamp.source_cursor, 9);

        // Extending again from the extended generation stays exact.
        let again = TurboQuantSegment::extend(
            &extended,
            config(&scope),
            3,
            &refs(&["point-1"]),
            [
                version(&scope, 10, "point-1", None),
                version(&scope, 1, "point-1", Some(10)),
            ],
        )
        .unwrap();
        let reopened = TurboQuantSegment::from_bytes(again.as_bytes()).unwrap();
        assert_eq!(reopened.descriptor(), again.descriptor());
        assert_eq!(again.descriptor().candidate_versions, 9);
    }

    #[test]
    fn extending_refuses_what_a_full_build_would_not_make() {
        let scope = ScopeId::new("instance:turbo-extend-refusals").unwrap();
        let base = TurboQuantSegment::build(
            config(&scope),
            2,
            2,
            [version(&scope, 1, "a", None), version(&scope, 2, "b", None)],
        )
        .unwrap();
        let changed = refs(&["b"]);
        let fresh = || [version(&scope, 3, "b", None)];

        let mut seeded = config(&scope);
        seeded.seed += 1;
        assert!(TurboQuantSegment::extend(&base, seeded, 3, &changed, fresh()).is_err());
        let mut narrower = config(&scope);
        narrower.bits = TurboQuantBits::Bits2;
        assert!(TurboQuantSegment::extend(&base, narrower, 3, &changed, fresh()).is_err());
        assert!(
            TurboQuantSegment::extend(&base, config(&scope), 2, &changed, fresh()).is_err(),
            "the next generation is later than its base"
        );
        assert!(
            TurboQuantSegment::extend(
                &base,
                config(&scope),
                3,
                &changed,
                [version(&scope, 3, "a", None)]
            )
            .is_err(),
            "an unchanged identity keeps its stored versions"
        );
        assert!(
            TurboQuantSegment::extend(&base, config(&scope), 3, &refs(&["a", "b"]), []).is_err(),
            "a generation holds at least one version"
        );
        let other = ScopeId::new("instance:turbo-extend-other").unwrap();
        assert!(TurboQuantSegment::extend(
            &base,
            config(&scope),
            3,
            &changed,
            [version(&other, 3, "b", None)]
        )
        .is_err());
        assert!(TurboQuantSegment::extend(&base, config(&scope), 3, &changed, fresh()).is_ok());
    }

    #[test]
    fn artifact_reader_rejects_candidate_without_collection_address() {
        let scope = ScopeId::new("instance:turbo-required-address").unwrap();
        let segment =
            TurboQuantSegment::build(config(&scope), 1, 1, [candidate(&scope, 1, "a", "a")])
                .unwrap();
        let header_len = u32::from_le_bytes(
            segment.as_bytes()[10..14]
                .try_into()
                .expect("fixed header length"),
        ) as usize;
        let header_end = HEADER_PREFIX_BYTES + header_len;
        let mut header: serde_json::Value =
            serde_json::from_slice(&segment.as_bytes()[HEADER_PREFIX_BYTES..header_end]).unwrap();
        header["candidates"][0]
            .as_object_mut()
            .unwrap()
            .remove("collection");
        let header = serde_json::to_vec(&header).unwrap();
        let payload = &segment.as_bytes()[header_end..];
        let checksum = artifact_digest(&header, payload);
        let bytes = encode_artifact(&header, payload, &checksum).unwrap();

        let error = TurboQuantSegment::from_bytes(&bytes).unwrap_err();
        assert!(error.to_string().contains("missing field `collection`"));
    }

    #[test]
    fn binary_artifact_round_trips_filters_and_accounts_for_compression() {
        let scope = ScopeId::new("instance:turbo-segment").unwrap();
        let candidates = (1..=32)
            .map(|cursor| {
                candidate(
                    &scope,
                    cursor,
                    &format!("point-{cursor:02}"),
                    if cursor % 2 == 0 { "a" } else { "b" },
                )
            })
            .collect::<Vec<_>>();
        let segment = TurboQuantSegment::build(config(&scope), 1, 32, candidates).unwrap();
        assert_eq!(segment.descriptor.packed_vector_bytes, 32 * 32);
        assert_eq!(segment.descriptor.full_precision_vector_bytes, 32 * 64 * 4);
        let reopened = TurboQuantSegment::from_bytes(segment.as_bytes()).unwrap();
        assert_eq!(reopened.descriptor(), segment.descriptor());

        let request = SearchRequest {
            scope: scope.clone(),
            read: ReadStamp::new(scope, None, 0, 32, Some("11".repeat(32))).unwrap(),
            valid_at: 100,
            field: "body".into(),
            query: VectorQuery::Dense {
                values: deterministic_vector(64, 99),
            },
            metric: ScoreMetric::Cosine,
            embedding_model: None,
            top_k: 4,
            mode: SearchMode::RequireApproximate { exact_rerank: 8 },
            filter: Some(FilterExpression::Condition {
                condition: FilterCondition {
                    property: "tenant".into(),
                    operator: FilterOperator::Equals {
                        value: RuntimeValue::String("a".into()),
                    },
                },
            }),
        };
        let hits = reopened.search_candidates_at(&request, 8, 32).unwrap();
        assert_eq!(hits.len(), 8);
        assert!(hits.iter().all(|hit| {
            hit.reference
                .id
                .as_str()
                .chars()
                .last()
                .and_then(|character| character.to_digit(10))
                .is_some_and(|digit| digit % 2 == 0)
        }));
    }

    #[test]
    fn artifact_corruption_and_stale_reads_fail_closed() {
        let scope = ScopeId::new("instance:turbo-corrupt").unwrap();
        let segment =
            TurboQuantSegment::build(config(&scope), 1, 1, [candidate(&scope, 1, "point", "a")])
                .unwrap();
        let mut corrupt = segment.as_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 0x80;
        assert!(TurboQuantSegment::from_bytes(&corrupt).is_err());

        let request = SearchRequest {
            scope: scope.clone(),
            read: ReadStamp::new(scope, None, 0, 2, Some("11".repeat(32))).unwrap(),
            valid_at: 100,
            field: "body".into(),
            query: VectorQuery::Dense {
                values: deterministic_vector(64, 99),
            },
            metric: ScoreMetric::Cosine,
            embedding_model: None,
            top_k: 1,
            mode: SearchMode::RequireApproximate { exact_rerank: 1 },
            filter: None,
        };
        assert!(segment.search_candidates_at(&request, 1, 2).is_err());
    }

    fn deterministic_vector(dimensions: usize, seed: u64) -> Vec<f32> {
        let mut state = seed;
        (0..dimensions)
            .map(|index| {
                state = mix(state, index as u64 + 1);
                let unit = (state >> 40) as f32 / ((1_u32 << 24) - 1) as f32;
                unit * 2.0 - 1.0
            })
            .collect()
    }

    fn mix(mut value: u64, salt: u64) -> u64 {
        value ^= salt.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        value ^= value >> 30;
        value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value ^= value >> 27;
        value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}
