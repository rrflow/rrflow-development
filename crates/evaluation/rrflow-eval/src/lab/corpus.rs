//! A deterministic corpus and the exact oracle that grades every system.
//!
//! Generation uses only addition, multiplication and square root, which IEEE 754
//! fixes exactly. Trigonometric or logarithmic steps could differ between
//! platforms and change the digest.
use super::manifest::{CorpusSource, CorpusSpec, LabMetric};
use super::{sha256_hex, Result};
use std::cmp::Ordering;

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn uniform(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// The sum of twelve uniforms minus six has mean 0 and variance 1.
    fn normal(&mut self) -> f64 {
        let mut sum = -6.0;
        for _ in 0..12 {
            sum += self.uniform();
        }
        sum
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorSet {
    pub dimensions: usize,
    pub values: Vec<f32>,
    pub categories: Vec<u32>,
}

impl VectorSet {
    pub fn len(&self) -> usize {
        self.categories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }

    pub fn vector(&self, index: usize) -> &[f32] {
        &self.values[index * self.dimensions..(index + 1) * self.dimensions]
    }

    /// Point identities start at 1, because some systems reserve 0.
    pub fn point_id(index: usize) -> u64 {
        index as u64 + 1
    }

    pub fn digest(&self) -> String {
        let mut bytes = Vec::with_capacity(self.values.len() * 4 + self.categories.len() * 4);
        for value in &self.values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for category in &self.categories {
            bytes.extend_from_slice(&category.to_le_bytes());
        }
        sha256_hex(&bytes)
    }
}

fn sample(
    rng: &mut SplitMix64,
    centers: &[f64],
    spec: &CorpusSpec,
    spread: f64,
    count: usize,
) -> VectorSet {
    let dimensions = spec.dimensions as usize;
    let clusters = centers.len() / dimensions;
    let mut values = Vec::with_capacity(count * dimensions);
    let mut categories = Vec::with_capacity(count);
    let mut scratch = vec![0.0_f64; dimensions];
    for _ in 0..count {
        let cluster = (rng.next() % clusters as u64) as usize;
        let center = &centers[cluster * dimensions..(cluster + 1) * dimensions];
        for (slot, base) in scratch.iter_mut().zip(center) {
            *slot = base + spread * rng.normal();
        }
        if spec.metric == LabMetric::Cosine {
            let norm = scratch
                .iter()
                .map(|value| value * value)
                .sum::<f64>()
                .sqrt();
            if norm > 0.0 {
                scratch.iter_mut().for_each(|value| *value /= norm);
            }
        }
        values.extend(scratch.iter().map(|value| *value as f32));
        categories.push((rng.next() % u64::from(spec.categories)) as u32);
    }
    VectorSet {
        dimensions,
        values,
        categories,
    }
}

/// Points and queries come from one mixture but separate streams, so queries are
/// realistic neighbors of the data and never exact copies of it.
pub fn generate(spec: &CorpusSpec, queries: u32) -> (VectorSet, VectorSet) {
    let CorpusSource::GaussianMixture {
        seed,
        clusters,
        spread_milli,
    } = &spec.source;
    let dimensions = spec.dimensions as usize;
    let mut rng = SplitMix64(*seed);
    let mut centers = Vec::with_capacity(*clusters as usize * dimensions);
    for _ in 0..*clusters as usize * dimensions {
        centers.push(rng.normal());
    }
    let spread = f64::from(*spread_milli) / 1000.0;
    let mut points_rng = SplitMix64(seed ^ 0x0000_706F_696E_7473);
    let mut queries_rng = SplitMix64(seed ^ 0x0071_7565_7269_6573);
    (
        sample(
            &mut points_rng,
            &centers,
            spec,
            spread,
            spec.points as usize,
        ),
        sample(&mut queries_rng, &centers, spec, spread, queries as usize),
    )
}

fn score(metric: LabMetric, left: &[f32], right: &[f32]) -> f32 {
    match metric {
        LabMetric::Cosine | LabMetric::Dot => left.iter().zip(right).map(|(a, b)| a * b).sum(),
        LabMetric::Euclidean => -left
            .iter()
            .zip(right)
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>(),
    }
}

/// The exact answers, computed by the harness before any system is contacted.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundTruth {
    pub ids: Vec<Vec<u64>>,
}

impl GroundTruth {
    pub fn digest(&self) -> String {
        let mut bytes = Vec::new();
        for answer in &self.ids {
            bytes.extend_from_slice(&(answer.len() as u64).to_le_bytes());
            for id in answer {
                bytes.extend_from_slice(&id.to_le_bytes());
            }
        }
        sha256_hex(&bytes)
    }

    /// The share of exact answers returned. Duplicates in the answer count once.
    pub fn recall(&self, query: usize, answer: &[u64]) -> f64 {
        let truth = &self.ids[query];
        if truth.is_empty() {
            return 1.0;
        }
        let found = truth.iter().filter(|id| answer.contains(id)).count();
        found as f64 / truth.len() as f64
    }
}

fn answer_query(
    points: &VectorSet,
    query: &[f32],
    metric: LabMetric,
    k: usize,
    category: Option<u32>,
) -> Vec<u64> {
    let mut best: Vec<(f32, u64)> = Vec::with_capacity(k + 1);
    for index in 0..points.len() {
        if category.is_some_and(|wanted| points.categories[index] != wanted) {
            continue;
        }
        let candidate = (
            score(metric, query, points.vector(index)),
            VectorSet::point_id(index),
        );
        let position = best
            .binary_search_by(|entry| {
                candidate
                    .0
                    .partial_cmp(&entry.0)
                    .unwrap_or(Ordering::Equal)
                    .then(entry.1.cmp(&candidate.1))
            })
            .unwrap_or_else(|position| position);
        if position < k {
            best.insert(position, candidate);
            best.truncate(k);
        }
    }
    best.into_iter().map(|(_, id)| id).collect()
}

pub fn exact_answers(
    points: &VectorSet,
    queries: &VectorSet,
    metric: LabMetric,
    k: usize,
    filtered: bool,
    threads: usize,
) -> Result<GroundTruth> {
    let threads = threads.max(1);
    let mut ids = vec![Vec::new(); queries.len()];
    let chunk = queries.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        for (part, slots) in ids.chunks_mut(chunk).enumerate() {
            scope.spawn(move || {
                for (offset, slot) in slots.iter_mut().enumerate() {
                    let index = part * chunk + offset;
                    let category = filtered.then(|| queries.categories[index]);
                    *slot = answer_query(points, queries.vector(index), metric, k, category);
                }
            });
        }
    });
    Ok(GroundTruth { ids })
}
