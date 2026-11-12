//! Measures what scanning each vector codec costs per candidate, relative to
//! scoring one full-precision vector exactly, so the planner's costs can come
//! from measurement instead of candidate counts.
//!
//! `cargo run --release -p rrd-vector --example vector_cost_calibration -- [vectors] [dims,...]`
//!
//! For each dimensionality, each of `ROUNDS` rounds times an exact scan of
//! every vector and then a scan of every code by each codec that keeps
//! `RERANK` candidates. `relative_cost_per_candidate` is the median over rounds
//! of codec scan time over that round's exact scan time, with its range.

use rrd_core::{
    ProjectionId, ReadStamp, RuntimeProperties, RuntimeRef, RuntimeVector, ScopeId,
    VectorCollectionAddress, VectorValue,
};
use rrd_vector::{
    search_exact_ref, ProductCompression, QuantizationMethod, QuantizedKernel, QuantizedSegment,
    QuantizedSegmentConfig, ScoreMetric, SearchHit, SearchMode, SearchRequest, TurboQuantBits,
    TurboQuantEstimator, TurboQuantSegment, TurboQuantSegmentConfig, VectorCandidate, VectorQuery,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::time::Instant;

const QUERIES: usize = 16;
const ROUNDS: usize = 9;
const TOP_K: usize = 10;
const RERANK: usize = 64;

enum Artifact {
    Quantized(Box<QuantizedSegment>),
    Turbo(Box<TurboQuantSegment>),
}

impl Artifact {
    fn scan(&self, request: &SearchRequest, vectors: u64) -> Vec<SearchHit> {
        let kernel = QuantizedKernel::Auto;
        match self {
            Self::Quantized(segment) => {
                segment.search_candidates_at(request, RERANK, vectors, kernel)
            }
            Self::Turbo(segment) => {
                segment.search_candidates_at_with_kernel(request, RERANK, vectors, kernel)
            }
        }
        .unwrap()
    }

    fn byte_ratio(&self) -> f64 {
        let (packed, full) = match self {
            Self::Quantized(segment) => {
                let descriptor = segment.descriptor();
                (
                    descriptor.packed_vector_bytes,
                    descriptor.full_precision_vector_bytes,
                )
            }
            Self::Turbo(segment) => {
                let descriptor = segment.descriptor();
                (
                    descriptor.packed_vector_bytes,
                    descriptor.full_precision_vector_bytes,
                )
            }
        };
        packed as f64 / full as f64
    }
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let vectors: usize = arguments
        .next()
        .map_or(20_000, |value| value.parse().unwrap());
    let dimension_list: Vec<usize> = arguments.next().map_or_else(
        || vec![128, 384, 768],
        |value| value.split(',').map(|dims| dims.parse().unwrap()).collect(),
    );
    let quantized = [
        ("scalar", QuantizationMethod::Scalar),
        (
            "product_8x",
            QuantizationMethod::Product {
                compression: ProductCompression::X8,
            },
        ),
        (
            "product_32x",
            QuantizationMethod::Product {
                compression: ProductCompression::X32,
            },
        ),
        ("binary", QuantizationMethod::Binary),
    ];
    let turbo = [
        (
            "turboquant_4bit",
            TurboQuantBits::Bits4,
            TurboQuantEstimator::Mse,
        ),
        (
            "turboquant_2bit",
            TurboQuantBits::Bits2,
            TurboQuantEstimator::Mse,
        ),
        (
            "turboquant_1bit",
            TurboQuantBits::Bits1,
            TurboQuantEstimator::Mse,
        ),
        (
            "turboquant_prod_4bit",
            TurboQuantBits::Bits4,
            TurboQuantEstimator::InnerProduct,
        ),
    ];
    let scope = ScopeId::new("instance:vector-cost-calibration").unwrap();
    let mut profiles = Vec::new();
    for &dimensions in &dimension_list {
        let candidates = (0..vectors)
            .map(|row| candidate(&scope, row, dimensions))
            .collect::<Vec<_>>();
        let queries = (0..QUERIES)
            .map(|query| {
                let values = vector(1_000_003 + query as u64, dimensions);
                request(&scope, values, vectors as u64)
            })
            .collect::<Vec<_>>();
        let mut artifacts = Vec::new();
        for (label, method) in quantized {
            let config = QuantizedSegmentConfig {
                id: ProjectionId::new(format!("quant-calibration-{label}")).unwrap(),
                scope: scope.clone(),
                field: "body".into(),
                dimensions,
                metric: ScoreMetric::Cosine,
                method,
                embedding_model: None,
                filter_properties: BTreeSet::new(),
            };
            let segment =
                QuantizedSegment::build(config, 1, vectors as u64, candidates.clone()).unwrap();
            artifacts.push((label, Artifact::Quantized(Box::new(segment))));
        }
        for (label, bits, estimator) in turbo {
            let config = TurboQuantSegmentConfig {
                id: ProjectionId::new(format!("quant-calibration-{label}")).unwrap(),
                scope: scope.clone(),
                field: "body".into(),
                dimensions,
                metric: ScoreMetric::Cosine,
                bits,
                seed: 0x5eed,
                embedding_model: None,
                filter_properties: BTreeSet::new(),
                estimator,
            };
            let segment =
                TurboQuantSegment::build(config, 1, vectors as u64, candidates.clone()).unwrap();
            artifacts.push((label, Artifact::Turbo(Box::new(segment))));
        }
        let exact = || {
            time_ns(|| {
                for query in &queries {
                    let mut exact = query.clone();
                    exact.mode = SearchMode::Exact;
                    std::hint::black_box(search_exact_ref(&exact, &candidates).unwrap());
                }
            })
        };
        let scan = |artifact: &Artifact| {
            time_ns(|| {
                for query in &queries {
                    std::hint::black_box(artifact.scan(query, vectors as u64));
                }
            })
        };
        // One warm-up of each, then rounds that time exact and every codec back
        // to back, so drift in host load reaches both sides of each ratio.
        exact();
        for (_, artifact) in &artifacts {
            scan(artifact);
        }
        let mut exact_ns = Vec::new();
        let mut scan_ns = vec![Vec::new(); artifacts.len()];
        for _ in 0..ROUNDS {
            exact_ns.push(exact());
            for (samples, (_, artifact)) in scan_ns.iter_mut().zip(&artifacts) {
                samples.push(scan(artifact));
            }
        }
        let per_candidate = (QUERIES * vectors) as f64;
        let codecs = artifacts
            .iter()
            .zip(&scan_ns)
            .map(|((label, artifact), samples)| {
                let ratios = samples
                    .iter()
                    .zip(&exact_ns)
                    .map(|(scan, exact)| scan / exact)
                    .collect::<Vec<_>>();
                json!({
                    "codec": label,
                    "packed_to_full_byte_ratio": artifact.byte_ratio(),
                    "minimum_scan_ns_per_candidate": minimum(samples) / per_candidate,
                    "relative_cost_per_candidate": median(&ratios),
                    "relative_cost_range": [minimum(&ratios), maximum(&ratios)],
                })
            })
            .collect::<Vec<_>>();
        profiles.push(json!({
            "dimensions": dimensions,
            "minimum_exact_ns_per_candidate": minimum(&exact_ns) / per_candidate,
            "codecs": codecs,
        }));
    }
    let report = json!({
        "schema": "rrflow.vector-cost-calibration.v1",
        "vectors": vectors,
        "queries": QUERIES,
        "rounds": ROUNDS,
        "top_k": TOP_K,
        "rerank": RERANK,
        "metric": "cosine",
        "kernel": "auto",
        "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        "release": !cfg!(debug_assertions),
        "profiles": profiles,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn time_ns(run: impl FnOnce()) -> f64 {
    let started = Instant::now();
    run();
    started.elapsed().as_nanos() as f64
}

fn median(samples: &[f64]) -> f64 {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[sorted.len() / 2]
}

fn minimum(samples: &[f64]) -> f64 {
    samples.iter().copied().fold(f64::INFINITY, f64::min)
}

fn maximum(samples: &[f64]) -> f64 {
    samples.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn request(scope: &ScopeId, values: Vec<f32>, vectors: u64) -> SearchRequest {
    SearchRequest {
        read: ReadStamp::new(scope.clone(), None, 0, vectors, Some("11".repeat(32))).unwrap(),
        scope: scope.clone(),
        valid_at: 10,
        field: "body".into(),
        query: VectorQuery::Dense { values },
        metric: ScoreMetric::Cosine,
        embedding_model: None,
        top_k: TOP_K,
        mode: SearchMode::RequireApproximate {
            exact_rerank: RERANK,
        },
        filter: None,
    }
}

fn candidate(scope: &ScopeId, row: usize, dimensions: usize) -> VectorCandidate {
    let reference = |kind| RuntimeRef::new(kind, format!("v{row}")).unwrap();
    VectorCandidate {
        scope: scope.clone(),
        source_cursor: row as u64 + 1,
        vector: RuntimeVector {
            reference: reference("embedding"),
            subject: reference("document"),
            collection: VectorCollectionAddress {
                collection_id: "documents".into(),
                vector_name: "body".into(),
            },
            field: "body".into(),
            valid_from: 1,
            valid_to: None,
            value: VectorValue::Dense {
                values: vector(row as u64, dimensions),
            },
            provenance: None,
            properties: RuntimeProperties::new(),
        },
    }
}

/// Deterministic values in [-1, 1) from SplitMix64.
fn vector(seed: u64, dimensions: usize) -> Vec<f32> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
    (0..dimensions)
        .map(|_| {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            ((z >> 40) as f32 / (1u64 << 23) as f32) - 1.0
        })
        .collect()
}
