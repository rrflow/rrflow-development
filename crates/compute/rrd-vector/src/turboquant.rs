//! Deterministic TurboQuant codec for dense vector artifacts
//! (arXiv:2504.19874).
//!
//! Both estimators rotate with a seeded randomized orthogonal Hadamard
//! transform and normalize each vector's length onto fixed standard normal
//! Lloyd-Max codebooks, and both score queries asymmetrically at full
//! precision.
//!
//! - **MSE** (TurboQuant_mse) packs one codebook index per coordinate and
//!   rescales the reconstruction to the vector's length, a per-vector
//!   centroid-norm correction.
//! - **Inner product** (TurboQuant_prod, Algorithm 2) packs indices from a
//!   codebook one bit narrower, followed by the signs of the residual's QJL
//!   projection (`qjl`), and stores the residual's length. Its estimate of
//!   ⟨y, x⟩ is unbiased. The residual is taken in the rotated space: S·Πᵀ is
//!   again i.i.d. normal when S is, so projecting the rotated residual with an
//!   independent S is distributed as the paper's S·r.

use crate::contract::invalid;
use crate::qjl::QjlProjection;
use crate::{QuantizedKernel, ScoreMetric};
use rrd_core::Result;
use serde::{Deserialize, Serialize};

pub const TURBOQUANT_FORMAT_VERSION: u16 = 1;
const MAX_DIMENSIONS: usize = 1_048_576;
const ROTATION_ROUNDS: usize = 3;

const CENTROIDS_1: [f32; 2] = [-0.797_884_6, 0.797_884_6];
const CENTROIDS_2: [f32; 4] = [-1.510, -0.4528, 0.4528, 1.510];
/// The 8-level Lloyd-Max quantizer for the standard normal distribution: the
/// MSE stage of a 4-bit inner-product code.
const CENTROIDS_3: [f32; 8] = [-2.152, -1.344, -0.756, -0.2451, 0.2451, 0.756, 1.344, 2.152];
const CENTROIDS_4: [f32; 16] = [
    -2.733, -2.069, -1.618, -1.256, -0.9424, -0.6568, -0.3881, -0.1284, 0.1284, 0.3881, 0.6568,
    0.9424, 1.256, 1.618, 2.069, 2.733,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurboQuantBits {
    Bits4,
    Bits2,
    Bits1_5,
    Bits1,
}

/// How a TurboQuant generation estimates a query's inner product with a
/// stored vector.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurboQuantEstimator {
    /// TurboQuant_mse: the reconstruction, rescaled to the vector's length.
    #[default]
    Mse,
    /// TurboQuant_prod: an MSE stage one bit narrower plus the QJL signs of
    /// its residual, which together estimate inner products without bias.
    InnerProduct,
}

impl TurboQuantEstimator {
    pub fn is_mse(&self) -> bool {
        *self == Self::Mse
    }
}

impl TurboQuantBits {
    /// The MSE stage of an inner-product code at this total width: its bits
    /// per coordinate and codebook, one bit narrower, as Algorithm 2
    /// instantiates it. 1.5 bits has no such stage.
    pub(crate) const fn inner_product_stage(self) -> Option<(u8, &'static [f32])> {
        match self {
            Self::Bits4 => Some((3, &CENTROIDS_3)),
            Self::Bits2 => Some((1, &CENTROIDS_1)),
            Self::Bits1 => Some((0, &[])),
            Self::Bits1_5 => None,
        }
    }

    const fn stored_bits(self) -> u8 {
        match self {
            Self::Bits4 => 4,
            Self::Bits2 => 2,
            Self::Bits1_5 | Self::Bits1 => 1,
        }
    }

    const fn centroids(self) -> &'static [f32] {
        match self {
            Self::Bits4 => &CENTROIDS_4,
            Self::Bits2 => &CENTROIDS_2,
            Self::Bits1_5 | Self::Bits1 => &CENTROIDS_1,
        }
    }

    pub(crate) fn padded_dimensions(self, dimensions: usize) -> usize {
        match self {
            Self::Bits4 => dimensions.next_multiple_of(2),
            Self::Bits2 => dimensions.next_multiple_of(4),
            Self::Bits1_5 => dimensions
                .checked_mul(3)
                .and_then(|value| value.checked_add(1))
                .map(|value| value / 2)
                .unwrap_or(usize::MAX)
                .next_multiple_of(8),
            Self::Bits1 => dimensions.next_multiple_of(8),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurboQuantVector {
    pub format_version: u16,
    pub bits: TurboQuantBits,
    pub seed: u64,
    pub dimensions: usize,
    pub padded_dimensions: usize,
    pub original_norm: f32,
    pub centroid_norm: f32,
    pub packed: Vec<u8>,
}

impl TurboQuantVector {
    pub fn encode(values: &[f32], bits: TurboQuantBits, seed: u64) -> Result<Self> {
        validate_input(values)?;
        let padded_dimensions = bits.padded_dimensions(values.len());
        if padded_dimensions > MAX_DIMENSIONS.saturating_mul(2) {
            return invalid("TurboQuant padded dimensions exceed the codec bound");
        }
        let original_norm = l2_norm(values);
        let mut rotated = vec![0.0_f64; padded_dimensions];
        for (target, source) in rotated.iter_mut().zip(values) {
            *target = f64::from(*source);
        }
        Rotation::new(padded_dimensions, seed).apply(&mut rotated);
        if original_norm > 0.0 {
            let scale = (padded_dimensions as f64).sqrt() / f64::from(original_norm);
            for value in &mut rotated {
                *value *= scale;
            }
        }
        let indices = rotated
            .iter()
            .map(|value| nearest_centroid(bits.centroids(), *value))
            .collect::<Vec<_>>();
        let centroid_norm = indices
            .iter()
            .map(|index| f64::from(bits.centroids()[usize::from(*index)]).powi(2))
            .sum::<f64>()
            .sqrt() as f32;
        let vector = Self {
            format_version: TURBOQUANT_FORMAT_VERSION,
            bits,
            seed,
            dimensions: values.len(),
            padded_dimensions,
            original_norm,
            centroid_norm,
            packed: pack(&indices, bits.stored_bits()),
        };
        vector.validate()?;
        Ok(vector)
    }

    pub fn validate(&self) -> Result<()> {
        if self.format_version != TURBOQUANT_FORMAT_VERSION
            || self.dimensions == 0
            || self.dimensions > MAX_DIMENSIONS
            || self.padded_dimensions != self.bits.padded_dimensions(self.dimensions)
        {
            return invalid("TurboQuant vector format or dimensions are invalid");
        }
        if !self.original_norm.is_finite()
            || self.original_norm < 0.0
            || !self.centroid_norm.is_finite()
            || self.centroid_norm <= 0.0
        {
            return invalid("TurboQuant vector norms are invalid");
        }
        let expected = packed_len(self.padded_dimensions, self.bits.stored_bits());
        if self.packed.len() != expected {
            return invalid("TurboQuant packed byte length differs from its dimensions");
        }
        let levels = self.bits.centroids().len();
        if unpack(
            &self.packed,
            self.padded_dimensions,
            self.bits.stored_bits(),
        )
        .iter()
        .any(|index| usize::from(*index) >= levels)
        {
            return invalid("TurboQuant packed centroid index is out of range");
        }
        Ok(())
    }

    pub fn estimated_payload_bytes(&self) -> usize {
        self.packed.len()
            + std::mem::size_of::<u16>()
            + std::mem::size_of::<u64>()
            + 2 * std::mem::size_of::<usize>()
            + 2 * std::mem::size_of::<f32>()
    }

    pub fn packed_vector_bytes(&self) -> usize {
        self.packed.len()
    }

    pub fn score(&self, query: &[f32], metric: ScoreMetric) -> Result<f64> {
        self.score_with_kernel(query, metric, QuantizedKernel::Auto)
    }

    pub fn score_with_kernel(
        &self,
        query: &[f32],
        metric: ScoreMetric,
        kernel: QuantizedKernel,
    ) -> Result<f64> {
        self.validate()?;
        if query.len() != self.dimensions {
            return invalid("TurboQuant score requires finite vectors with matching dimensions");
        }
        TurboQuantQuery::mse(query, metric, self.bits, self.seed)?.score_mse(
            self.original_norm,
            self.centroid_norm,
            &self.packed,
            kernel,
        )
    }

    pub fn reconstruct(&self) -> Result<Vec<f32>> {
        self.validate()?;
        let scale = if self.original_norm == 0.0 {
            0.0
        } else {
            f64::from(self.original_norm) / f64::from(self.centroid_norm)
        };
        let mut reconstructed = self
            .decoded_centroids()
            .into_iter()
            .map(|value| value * scale)
            .collect::<Vec<_>>();
        Rotation::new(self.padded_dimensions, self.seed).apply_inverse(&mut reconstructed);
        Ok(reconstructed
            .into_iter()
            .take(self.dimensions)
            .map(|value| value as f32)
            .collect())
    }

    fn decoded_centroids(&self) -> Vec<f64> {
        let centroids = self.bits.centroids();
        unpack(
            &self.packed,
            self.padded_dimensions,
            self.bits.stored_bits(),
        )
        .into_iter()
        .map(|index| f64::from(centroids[usize::from(index)]))
        .collect()
    }
}

/// One vector under the inner-product estimator: its MSE-stage indices
/// followed by the QJL signs of its residual, packed in that order, with the
/// vector's length and the unit residual's.
#[derive(Debug, Clone, PartialEq)]
pub struct TurboQuantInnerProductCode {
    pub original_norm: f32,
    /// The MSE stage's centroid norm, for accounting; scoring does not use it.
    pub centroid_norm: f32,
    pub residual_norm: f32,
    pub packed: Vec<u8>,
}

impl TurboQuantInnerProductCode {
    /// Algorithm 2's quantizer. `projection` is S for this seed at the padded
    /// dimensions; a build materializes it once for every vector.
    pub fn encode(
        values: &[f32],
        bits: TurboQuantBits,
        seed: u64,
        projection: &QjlProjection,
    ) -> Result<Self> {
        validate_input(values)?;
        let Some((stage_bits, centroids)) = bits.inner_product_stage() else {
            return invalid("1.5-bit TurboQuant has no inner-product estimator");
        };
        let padded_dimensions = inner_product_padded_dimensions(values.len());
        if projection.dimensions() != padded_dimensions {
            return invalid("the QJL projection differs from the vector's padded dimensions");
        }
        let original_norm = l2_norm(values);
        let mut rotated = vec![0.0_f64; padded_dimensions];
        for (target, source) in rotated.iter_mut().zip(values) {
            *target = f64::from(*source);
        }
        Rotation::new(padded_dimensions, seed).apply(&mut rotated);
        if original_norm > 0.0 {
            let scale = (padded_dimensions as f64).sqrt() / f64::from(original_norm);
            for value in &mut rotated {
                *value *= scale;
            }
        }
        let indices = if stage_bits == 0 {
            Vec::new()
        } else {
            rotated
                .iter()
                .map(|value| nearest_centroid(centroids, *value))
                .collect::<Vec<_>>()
        };
        // The unit vector's residual after its MSE stage, in the rotated space.
        let root = (padded_dimensions as f64).sqrt();
        let residual = rotated
            .iter()
            .enumerate()
            .map(|(coordinate, value)| {
                let reconstructed = indices
                    .get(coordinate)
                    .map_or(0.0, |index| f64::from(centroids[usize::from(*index)]));
                (value - reconstructed) / root
            })
            .collect::<Vec<_>>();
        let residual_norm = residual
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt() as f32;
        let signs = projection
            .project(&residual)
            .into_iter()
            .map(|value| u8::from(value >= 0.0))
            .collect::<Vec<_>>();
        let centroid_norm = indices
            .iter()
            .map(|index| f64::from(centroids[usize::from(*index)]).powi(2))
            .sum::<f64>()
            .sqrt() as f32;
        let mut packed = pack(&indices, stage_bits);
        packed.extend(pack(&signs, 1));
        Ok(Self {
            original_norm,
            centroid_norm,
            residual_norm,
            packed,
        })
    }
}

/// Coordinates an inner-product code covers: whole bytes of indices and of
/// signs at every stage width.
pub(crate) fn inner_product_padded_dimensions(dimensions: usize) -> usize {
    dimensions.next_multiple_of(8)
}

/// The packed bytes of one inner-product code.
pub(crate) fn inner_product_packed_len(
    bits: TurboQuantBits,
    padded_dimensions: usize,
) -> Option<usize> {
    let (stage_bits, _) = bits.inner_product_stage()?;
    Some(packed_len(padded_dimensions, stage_bits) + packed_len(padded_dimensions, 1))
}

/// A query prepared once for scoring many stored vectors of one generation:
/// rotated once, and projected once with S for the inner-product estimator.
#[derive(Debug, Clone)]
pub struct TurboQuantQuery {
    metric: ScoreMetric,
    bits: TurboQuantBits,
    dimensions: usize,
    padded_dimensions: usize,
    query_norm: f32,
    query: Vec<f32>,
    rotation: Rotation,
    rotated: Vec<f64>,
    /// S·Πy for the inner-product estimator.
    projected: Option<Vec<f64>>,
}

impl TurboQuantQuery {
    pub fn mse(
        query: &[f32],
        metric: ScoreMetric,
        bits: TurboQuantBits,
        seed: u64,
    ) -> Result<Self> {
        Self::prepare(
            query,
            metric,
            bits,
            seed,
            bits.padded_dimensions(query.len()),
            None,
        )
    }

    pub fn inner_product(
        query: &[f32],
        metric: ScoreMetric,
        bits: TurboQuantBits,
        seed: u64,
        projection: &QjlProjection,
    ) -> Result<Self> {
        if bits.inner_product_stage().is_none() {
            return invalid("1.5-bit TurboQuant has no inner-product estimator");
        }
        if metric == ScoreMetric::Manhattan {
            return invalid(
                "the TurboQuant inner-product estimator scores dot, cosine and euclidean metrics",
            );
        }
        let padded_dimensions = inner_product_padded_dimensions(query.len());
        if projection.dimensions() != padded_dimensions {
            return invalid("the QJL projection differs from the query's padded dimensions");
        }
        Self::prepare(
            query,
            metric,
            bits,
            seed,
            padded_dimensions,
            Some(projection),
        )
    }

    fn prepare(
        query: &[f32],
        metric: ScoreMetric,
        bits: TurboQuantBits,
        seed: u64,
        padded_dimensions: usize,
        projection: Option<&QjlProjection>,
    ) -> Result<Self> {
        if query.is_empty()
            || query.len() > MAX_DIMENSIONS
            || query.iter().any(|value| !value.is_finite())
        {
            return invalid("TurboQuant score requires finite vectors with matching dimensions");
        }
        let query_norm = l2_norm(query);
        if metric == ScoreMetric::Cosine && query_norm == 0.0 {
            return invalid("cosine query must have non-zero norm");
        }
        let mut rotated = vec![0.0_f64; padded_dimensions];
        for (target, source) in rotated.iter_mut().zip(query) {
            *target = f64::from(*source);
        }
        let rotation = Rotation::new(padded_dimensions, seed);
        rotation.apply(&mut rotated);
        let projected = projection.map(|projection| projection.project(&rotated));
        Ok(Self {
            metric,
            bits,
            dimensions: query.len(),
            padded_dimensions,
            query_norm,
            query: query.to_vec(),
            rotation,
            rotated,
            projected,
        })
    }

    /// TurboQuant_mse's score of one stored vector.
    pub fn score_mse(
        &self,
        original_norm: f32,
        centroid_norm: f32,
        packed: &[u8],
        kernel: QuantizedKernel,
    ) -> Result<f64> {
        if self.projected.is_some()
            || self.padded_dimensions != self.bits.padded_dimensions(self.dimensions)
            || packed.len() != packed_len(self.padded_dimensions, self.bits.stored_bits())
        {
            return invalid("TurboQuant score requires finite vectors with matching dimensions");
        }
        let codebook = self.bits.centroids();
        let indices = unpack(packed, self.padded_dimensions, self.bits.stored_bits());
        if indices
            .iter()
            .any(|index| usize::from(*index) >= codebook.len())
        {
            return invalid("TurboQuant packed centroid index is out of range");
        }
        let centroids = indices
            .into_iter()
            .map(|index| f64::from(codebook[usize::from(index)]))
            .collect::<Vec<_>>();
        let raw_dot = dot(&self.rotated, &centroids, kernel);
        let scale = if original_norm == 0.0 {
            0.0
        } else {
            f64::from(original_norm) / f64::from(centroid_norm)
        };
        let dot = raw_dot * scale;
        match self.metric {
            ScoreMetric::Dot => Ok(dot),
            ScoreMetric::Cosine => {
                if original_norm == 0.0 {
                    Ok(0.0)
                } else {
                    Ok(raw_dot / (f64::from(self.query_norm) * f64::from(centroid_norm)))
                }
            }
            ScoreMetric::Euclidean => {
                let squared = f64::from(self.query_norm).powi(2) + f64::from(original_norm).powi(2)
                    - 2.0 * dot;
                Ok(-squared.max(0.0).sqrt())
            }
            ScoreMetric::Manhattan => {
                let mut reconstructed = centroids
                    .into_iter()
                    .map(|value| value * scale)
                    .collect::<Vec<_>>();
                self.rotation.apply_inverse(&mut reconstructed);
                Ok(-self
                    .query
                    .iter()
                    .map(|value| f64::from(*value))
                    .zip(reconstructed)
                    .take(self.dimensions)
                    .map(|(left, right)| (left - right).abs())
                    .sum::<f64>())
            }
        }
    }

    /// TurboQuant_prod's score of one stored vector: from the estimate
    /// ⟨y, x⟩ ≈ ‖x‖·(⟨Πy, c⟩/√d + √(π/2)/d · γ · ⟨S·Πy, signs⟩).
    pub fn score_inner_product(
        &self,
        original_norm: f32,
        residual_norm: f32,
        packed: &[u8],
        kernel: QuantizedKernel,
    ) -> Result<f64> {
        let (Some(projected), Some((stage_bits, codebook))) =
            (&self.projected, self.bits.inner_product_stage())
        else {
            return invalid("an inner-product score requires a query prepared with its projection");
        };
        let stage_len = packed_len(self.padded_dimensions, stage_bits);
        if packed.len() != stage_len + packed_len(self.padded_dimensions, 1) {
            return invalid("TurboQuant inner-product code length differs from its dimensions");
        }
        let root = (self.padded_dimensions as f64).sqrt();
        let stage = if stage_bits == 0 {
            0.0
        } else {
            let indices = unpack(&packed[..stage_len], self.padded_dimensions, stage_bits);
            if indices
                .iter()
                .any(|index| usize::from(*index) >= codebook.len())
            {
                return invalid("TurboQuant packed centroid index is out of range");
            }
            let centroids = indices
                .into_iter()
                .map(|index| f64::from(codebook[usize::from(index)]))
                .collect::<Vec<_>>();
            dot(&self.rotated, &centroids, kernel) / root
        };
        let signs = unpack(&packed[stage_len..], self.padded_dimensions, 1)
            .into_iter()
            .map(|sign| if sign == 1 { 1.0 } else { -1.0 })
            .collect::<Vec<_>>();
        let correction = (std::f64::consts::PI / 2.0).sqrt() / self.padded_dimensions as f64
            * f64::from(residual_norm)
            * dot(projected, &signs, kernel);
        let unit = stage + correction;
        let dot = f64::from(original_norm) * unit;
        match self.metric {
            ScoreMetric::Dot => Ok(dot),
            ScoreMetric::Cosine => {
                if original_norm == 0.0 {
                    Ok(0.0)
                } else {
                    Ok(unit / f64::from(self.query_norm))
                }
            }
            ScoreMetric::Euclidean => {
                let squared = f64::from(self.query_norm).powi(2) + f64::from(original_norm).powi(2)
                    - 2.0 * dot;
                Ok(-squared.max(0.0).sqrt())
            }
            ScoreMetric::Manhattan => invalid(
                "the TurboQuant inner-product estimator scores dot, cosine and euclidean metrics",
            ),
        }
    }
}

fn dot(left: &[f64], right: &[f64], kernel: QuantizedKernel) -> f64 {
    debug_assert_eq!(left.len(), right.len());
    match kernel {
        QuantizedKernel::Scalar => dot_scalar(left, right),
        QuantizedKernel::Auto => {
            #[cfg(target_arch = "x86_64")]
            if std::arch::is_x86_feature_detected!("avx2") {
                // SAFETY: AVX2 support was checked and both slices have equal
                // lengths.
                return unsafe { dot_avx2(left, right) };
            }
            dot_scalar(left, right)
        }
    }
}

fn dot_scalar(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn dot_avx2(left: &[f64], right: &[f64]) -> f64 {
    use std::arch::x86_64::*;
    let mut sum = _mm256_setzero_pd();
    let mut index = 0;
    while index + 4 <= left.len() {
        let left_values = _mm256_loadu_pd(left.as_ptr().add(index));
        let right_values = _mm256_loadu_pd(right.as_ptr().add(index));
        sum = _mm256_add_pd(sum, _mm256_mul_pd(left_values, right_values));
        index += 4;
    }
    let mut lanes = [0.0_f64; 4];
    _mm256_storeu_pd(lanes.as_mut_ptr(), sum);
    let mut result = lanes.into_iter().sum::<f64>();
    while index < left.len() {
        result += left[index] * right[index];
        index += 1;
    }
    result
}

fn validate_input(values: &[f32]) -> Result<()> {
    if values.is_empty() || values.len() > MAX_DIMENSIONS {
        return invalid("TurboQuant dimensions must be in 1..=1048576");
    }
    if values.iter().any(|value| !value.is_finite()) {
        return invalid("TurboQuant input must be finite");
    }
    Ok(())
}

fn l2_norm(values: &[f32]) -> f32 {
    values
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt() as f32
}

fn nearest_centroid(centroids: &[f32], value: f64) -> u8 {
    centroids
        .windows(2)
        .position(|pair| value <= f64::from((pair[0] + pair[1]) / 2.0))
        .unwrap_or(centroids.len() - 1) as u8
}

fn packed_len(dimensions: usize, bits: u8) -> usize {
    dimensions.saturating_mul(usize::from(bits)).div_ceil(8)
}

fn pack(indices: &[u8], bits: u8) -> Vec<u8> {
    let mut packed = vec![0_u8; packed_len(indices.len(), bits)];
    let mut bit_offset = 0usize;
    for index in indices {
        let byte = bit_offset / 8;
        let shift = bit_offset % 8;
        packed[byte] |= *index << shift;
        if shift + usize::from(bits) > 8 {
            packed[byte + 1] |= *index >> (8 - shift);
        }
        bit_offset += usize::from(bits);
    }
    packed
}

fn unpack(packed: &[u8], dimensions: usize, bits: u8) -> Vec<u8> {
    let mask = (1_u16 << bits) - 1;
    (0..dimensions)
        .map(|index| {
            let bit_offset = index * usize::from(bits);
            let byte = bit_offset / 8;
            let shift = bit_offset % 8;
            let lower = u16::from(packed[byte]);
            let upper = packed.get(byte + 1).copied().map(u16::from).unwrap_or(0);
            (((lower | (upper << 8)) >> shift) & mask) as u8
        })
        .collect()
}

#[derive(Debug, Clone)]
struct Rotation {
    permutations: [Vec<usize>; ROTATION_ROUNDS],
}

impl Rotation {
    fn new(dimensions: usize, seed: u64) -> Self {
        Self {
            permutations: std::array::from_fn(|round| {
                permutation(dimensions, mix(seed, round as u64 + 1))
            }),
        }
    }

    fn apply(&self, values: &mut [f64]) {
        hadamard_chunks(values);
        let mut scratch = vec![0.0; values.len()];
        for permutation in &self.permutations {
            for (target, source) in scratch.iter_mut().zip(permutation) {
                *target = values[*source];
            }
            values.copy_from_slice(&scratch);
            hadamard_chunks(values);
        }
    }

    fn apply_inverse(&self, values: &mut [f64]) {
        hadamard_chunks(values);
        let mut scratch = vec![0.0; values.len()];
        for permutation in self.permutations.iter().rev() {
            for (target, source) in permutation.iter().enumerate() {
                scratch[*source] = values[target];
            }
            values.copy_from_slice(&scratch);
            hadamard_chunks(values);
        }
    }
}

fn hadamard_chunks(values: &mut [f64]) {
    let mut remaining = values.len();
    let mut offset = 0usize;
    while remaining > 0 {
        let size = 1usize << remaining.ilog2();
        let chunk = &mut values[offset..offset + size];
        hadamard(chunk);
        let normalization = 1.0 / (size as f64).sqrt();
        for value in chunk {
            *value *= normalization;
        }
        offset += size;
        remaining -= size;
    }
}

fn hadamard(values: &mut [f64]) {
    let mut width = 1usize;
    while width < values.len() {
        for start in (0..values.len()).step_by(width * 2) {
            for offset in 0..width {
                let left = values[start + offset];
                let right = values[start + offset + width];
                values[start + offset] = left + right;
                values[start + offset + width] = left - right;
            }
        }
        width *= 2;
    }
}

fn permutation(dimensions: usize, seed: u64) -> Vec<usize> {
    let mut values = (0..dimensions).collect::<Vec<_>>();
    let mut state = seed;
    for upper in (1..dimensions).rev() {
        state = mix(state, upper as u64);
        let selected = (state as usize) % (upper + 1);
        values.swap(upper, selected);
    }
    values
}

fn mix(mut value: u64, salt: u64) -> u64 {
    value ^= salt.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_is_norm_preserving_and_invertible_for_irregular_dimensions() {
        for dimensions in [1, 3, 63, 384, 769] {
            let rotation = Rotation::new(dimensions, 42);
            let original = deterministic_vector(dimensions, dimensions as u64);
            let original = original.into_iter().map(f64::from).collect::<Vec<_>>();
            let mut rotated = original.clone();
            rotation.apply(&mut rotated);
            let before = original.iter().map(|value| value * value).sum::<f64>();
            let after = rotated.iter().map(|value| value * value).sum::<f64>();
            assert!((before - after).abs() < 1e-8 * before.max(1.0));
            rotation.apply_inverse(&mut rotated);
            for (expected, actual) in original.iter().zip(rotated) {
                assert!((expected - actual).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn all_bit_modes_are_deterministic_packed_and_score_asymmetrically() {
        let values = deterministic_vector(384, 7);
        let query = deterministic_vector(384, 9);
        for (bits, expected_bytes) in [
            (TurboQuantBits::Bits4, 192),
            (TurboQuantBits::Bits2, 96),
            (TurboQuantBits::Bits1_5, 72),
            (TurboQuantBits::Bits1, 48),
        ] {
            let encoded = TurboQuantVector::encode(&values, bits, 11).unwrap();
            assert_eq!(encoded.packed_vector_bytes(), expected_bytes);
            assert_eq!(
                encoded,
                TurboQuantVector::encode(&values, bits, 11).unwrap()
            );
            assert!(encoded.score(&query, ScoreMetric::Dot).unwrap().is_finite());
            assert!(encoded
                .score(&query, ScoreMetric::Cosine)
                .unwrap()
                .is_finite());
            assert!(encoded
                .score(&query, ScoreMetric::Euclidean)
                .unwrap()
                .is_finite());
            assert!(encoded
                .score(&query, ScoreMetric::Manhattan)
                .unwrap()
                .is_finite());
            for metric in [
                ScoreMetric::Dot,
                ScoreMetric::Cosine,
                ScoreMetric::Euclidean,
                ScoreMetric::Manhattan,
            ] {
                let scalar = encoded
                    .score_with_kernel(&query, metric, QuantizedKernel::Scalar)
                    .unwrap();
                let automatic = encoded
                    .score_with_kernel(&query, metric, QuantizedKernel::Auto)
                    .unwrap();
                assert!((scalar - automatic).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn four_bit_reconstruction_and_top_k_recall_are_oracle_gated() {
        let dimensions = 128;
        let corpus = (0..256)
            .map(|index| deterministic_vector(dimensions, index + 100))
            .collect::<Vec<_>>();
        let encoded = corpus
            .iter()
            .map(|vector| TurboQuantVector::encode(vector, TurboQuantBits::Bits4, 91).unwrap())
            .collect::<Vec<_>>();
        let mut recall_total = 0.0;
        for query_seed in 0..16 {
            let query = deterministic_vector(dimensions, query_seed + 10_000);
            let exact = top_k(&corpus, &query, 10, |vector, query| {
                vector
                    .iter()
                    .zip(query)
                    .map(|(left, right)| f64::from(*left) * f64::from(*right))
                    .sum()
            });
            let approximate = top_k(&encoded, &query, 10, |vector, query| {
                vector.score(query, ScoreMetric::Dot).unwrap()
            });
            let overlap = exact
                .iter()
                .filter(|identity| approximate.contains(identity))
                .count();
            recall_total += overlap as f64 / 10.0;
        }
        let mean_recall = recall_total / 16.0;
        assert!(mean_recall >= 0.80, "four-bit mean recall@10={mean_recall}");

        let reconstructed = encoded[0].reconstruct().unwrap();
        let original_norm = f64::from(l2_norm(&corpus[0]));
        let reconstructed_norm = f64::from(l2_norm(&reconstructed));
        assert!((original_norm - reconstructed_norm).abs() < 1e-4);
    }

    fn unit(values: Vec<f32>) -> Vec<f32> {
        let norm = l2_norm(&values);
        values.into_iter().map(|value| value / norm).collect()
    }

    fn true_dot(left: &[f32], right: &[f32]) -> f64 {
        left.iter()
            .zip(right)
            .map(|(left, right)| f64::from(*left) * f64::from(*right))
            .sum()
    }

    /// (estimate, truth) for every pair of 120 unit vectors of 128 dimensions
    /// and queries mixed from each vector and noise, so inner products spread
    /// from about 0 to 0.9, under several seeds.
    fn estimates(bits: TurboQuantBits, inner_product: bool) -> Vec<(f64, f64)> {
        let dimensions = 128;
        let corpus = (0..120)
            .map(|index| unit(deterministic_vector(dimensions, index + 500)))
            .collect::<Vec<_>>();
        let mut pairs = Vec::new();
        for seed in [3, 17, 29, 41] {
            let projection = QjlProjection::new(dimensions, seed).unwrap();
            for (row, vector) in corpus.iter().enumerate() {
                let noise = unit(deterministic_vector(dimensions, row as u64 + 90_000 + seed));
                for weight in [0.0_f32, 0.3, 0.6, 0.9] {
                    let query = unit(
                        vector
                            .iter()
                            .zip(&noise)
                            .map(|(signal, noise)| weight * signal + (1.0 - weight) * noise)
                            .collect(),
                    );
                    pairs.push((
                        estimate(vector, &query, bits, seed, &projection, inner_product),
                        true_dot(vector, &query),
                    ));
                }
            }
        }
        pairs
    }

    fn estimate(
        vector: &[f32],
        query: &[f32],
        bits: TurboQuantBits,
        seed: u64,
        projection: &QjlProjection,
        inner_product: bool,
    ) -> f64 {
        if inner_product {
            let code = TurboQuantInnerProductCode::encode(vector, bits, seed, projection).unwrap();
            TurboQuantQuery::inner_product(query, ScoreMetric::Dot, bits, seed, projection)
                .unwrap()
                .score_inner_product(
                    code.original_norm,
                    code.residual_norm,
                    &code.packed,
                    QuantizedKernel::Auto,
                )
                .unwrap()
        } else {
            let code = TurboQuantVector::encode(vector, bits, seed).unwrap();
            TurboQuantQuery::mse(query, ScoreMetric::Dot, bits, seed)
                .unwrap()
                .score_mse(
                    code.original_norm,
                    code.centroid_norm,
                    &code.packed,
                    QuantizedKernel::Auto,
                )
                .unwrap()
        }
    }

    /// Signed errors of an estimator over the spread of pairs.
    fn errors(bits: TurboQuantBits, inner_product: bool) -> Vec<f64> {
        estimates(bits, inner_product)
            .into_iter()
            .map(|(estimate, truth)| estimate - truth)
            .collect()
    }

    /// The least-squares slope of estimate on truth, and its standard error.
    fn slope(pairs: &[(f64, f64)]) -> (f64, f64) {
        let count = pairs.len() as f64;
        let mean_truth = pairs.iter().map(|(_, truth)| truth).sum::<f64>() / count;
        let mean_estimate = pairs.iter().map(|(estimate, _)| estimate).sum::<f64>() / count;
        let spread = pairs
            .iter()
            .map(|(_, truth)| (truth - mean_truth).powi(2))
            .sum::<f64>();
        let slope = pairs
            .iter()
            .map(|(estimate, truth)| (truth - mean_truth) * (estimate - mean_estimate))
            .sum::<f64>()
            / spread;
        let intercept = mean_estimate - slope * mean_truth;
        let residual = pairs
            .iter()
            .map(|(estimate, truth)| (estimate - intercept - slope * truth).powi(2))
            .sum::<f64>()
            / (count - 2.0);
        (slope, (residual / spread).sqrt())
    }

    #[test]
    fn the_mse_estimator_shrinks_inner_products_at_one_bit() {
        // The paper's motivation: an MSE quantizer's inner products are
        // biased by a multiplicative factor, about sqrt(2/pi) at one bit.
        let (one_bit, _) = slope(&estimates(TurboQuantBits::Bits1, false));
        assert!(
            (one_bit - (2.0 / std::f64::consts::PI).sqrt()).abs() < 0.05,
            "one-bit MSE slope {one_bit}"
        );
    }

    fn mean_and_standard_error(errors: &[f64]) -> (f64, f64) {
        let count = errors.len() as f64;
        let mean = errors.iter().sum::<f64>() / count;
        let variance = errors
            .iter()
            .map(|error| (error - mean).powi(2))
            .sum::<f64>()
            / (count - 1.0);
        (mean, (variance / count).sqrt())
    }

    #[test]
    fn the_inner_product_estimator_is_unbiased_and_within_its_distortion_bound() {
        // Theorem 2: D_prod <= sqrt(3)·pi²·|y|²/(d·4^b) for unit x, with b
        // the total bits per coordinate.
        for (bits, total) in [
            (TurboQuantBits::Bits4, 4),
            (TurboQuantBits::Bits2, 2),
            (TurboQuantBits::Bits1, 1),
        ] {
            let errors = errors(bits, true);
            let (mean, standard_error) = mean_and_standard_error(&errors);
            assert!(
                mean.abs() <= 4.0 * standard_error,
                "{bits:?}: mean signed error {mean} against standard error {standard_error}"
            );
            let (slope, slope_error) = slope(&estimates(bits, true));
            assert!(
                (slope - 1.0).abs() <= 4.0 * slope_error,
                "{bits:?}: slope {slope} against standard error {slope_error}"
            );
            let distortion =
                errors.iter().map(|error| error * error).sum::<f64>() / errors.len() as f64;
            let bound = 3f64.sqrt() * std::f64::consts::PI.powi(2) / (128.0 * 4f64.powi(total));
            assert!(
                distortion <= bound,
                "{bits:?}: distortion {distortion} above the paper's bound {bound}"
            );
        }
    }

    #[test]
    fn inner_product_codes_are_deterministic_and_refuse_what_they_cannot_encode() {
        let values = deterministic_vector(100, 7);
        let projection = QjlProjection::new(104, 11).unwrap();
        for (bits, bytes) in [
            (TurboQuantBits::Bits4, 52),
            (TurboQuantBits::Bits2, 26),
            (TurboQuantBits::Bits1, 13),
        ] {
            let code = TurboQuantInnerProductCode::encode(&values, bits, 11, &projection).unwrap();
            assert_eq!(code.packed.len(), bytes, "{bits:?}");
            assert_eq!(
                code,
                TurboQuantInnerProductCode::encode(&values, bits, 11, &projection).unwrap()
            );
            assert!(code.residual_norm > 0.0 && code.residual_norm <= 1.0);
            let query = deterministic_vector(100, 9);
            let prepared =
                TurboQuantQuery::inner_product(&query, ScoreMetric::Cosine, bits, 11, &projection)
                    .unwrap();
            for kernel in [QuantizedKernel::Scalar, QuantizedKernel::Auto] {
                assert!(prepared
                    .score_inner_product(
                        code.original_norm,
                        code.residual_norm,
                        &code.packed,
                        kernel
                    )
                    .unwrap()
                    .is_finite());
            }
            assert!(prepared
                .score_inner_product(
                    code.original_norm,
                    code.residual_norm,
                    &code.packed[1..],
                    QuantizedKernel::Auto
                )
                .is_err());
            assert!(prepared
                .score_mse(code.original_norm, 1.0, &code.packed, QuantizedKernel::Auto)
                .is_err());
        }
        assert!(TurboQuantInnerProductCode::encode(
            &values,
            TurboQuantBits::Bits1_5,
            11,
            &projection
        )
        .is_err());
        assert!(TurboQuantInnerProductCode::encode(
            &values,
            TurboQuantBits::Bits4,
            11,
            &QjlProjection::new(96, 11).unwrap()
        )
        .is_err());
        assert!(TurboQuantQuery::inner_product(
            &values,
            ScoreMetric::Manhattan,
            TurboQuantBits::Bits4,
            11,
            &projection
        )
        .is_err());
    }

    #[test]
    fn malformed_and_non_finite_inputs_fail_closed() {
        assert!(TurboQuantVector::encode(&[], TurboQuantBits::Bits4, 1).is_err());
        assert!(TurboQuantVector::encode(&[f32::NAN], TurboQuantBits::Bits4, 1).is_err());
        let mut encoded = TurboQuantVector::encode(&[1.0, 2.0], TurboQuantBits::Bits2, 1).unwrap();
        encoded.packed.clear();
        assert!(encoded.validate().is_err());
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

    fn top_k<T>(
        corpus: &[T],
        query: &[f32],
        limit: usize,
        score: impl Fn(&T, &[f32]) -> f64,
    ) -> Vec<usize> {
        let mut scores = corpus
            .iter()
            .enumerate()
            .map(|(identity, vector)| (identity, score(vector, query)))
            .collect::<Vec<_>>();
        scores.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        scores
            .into_iter()
            .take(limit)
            .map(|(identity, _)| identity)
            .collect()
    }
}
