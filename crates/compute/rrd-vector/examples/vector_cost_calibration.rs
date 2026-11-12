//! Measures what each vector access path costs per candidate, in units of
//! one exact full-precision score, so the planner can be priced from
//! measurement instead of candidate counts.
//!
//! Run in release mode:
//! `cargo run --release -p rrd-vector --example vector_cost_calibration -- [vectors] [dims,...]`
//!
//! For each dimensionality it times an exact scan of every vector and a scan of
//! every packed code by each codec, returning `rerank` candidates. It prints one
//! JSON document. The ratio `scan_ns / exact_ns` is the per-candidate cost of
//! scanning that codec's codes relative to exact scoring.

use rrd_core::{
    ProjectionId, ReadStamp, RuntimeProperties, RuntimeRef, RuntimeVector, ScopeId,
    VectorCollectionAddress, VectorValue,
};
use rrd_vector::{
    search_exact_ref, ProductCompression, QuantizationMethod, QuantizedKernel, QuantizedSegment,
    QuantizedSegmentConfig, ScoreMetric, SearchMode, SearchRequest, TurboQuantBits,
    TurboQuantEstimator, TurboQuantSegment, TurboQuantSegmentConfig, VectorCandidate, VectorQuery,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::time::Instant;

const QUERIES: usize = 16;
const ROUNDS: usize = 5;
const TOP_K: usize = 10;
const RERANK: usize = 64;

#[derive(Clone, Copy)]
enum Codec {
    Scalar,
    Product(ProductCompression),
    Binary,
    Turbo(TurboQuantBits, TurboQuantEstimator),
}

impl Codec {
    fn label(self) -> String {
        match self {
            Self::Scalar => "scalar".into(),
            Self::Product(compression) => format!("product_{}x", compression.ratio()),
            Self::Binary => "binary".into(),
            Self::Turbo(bits, estimator) => format!(
                "turboquant_{}{}",
                match estimator {
                    TurboQuantEstimator::Mse => "",
                    TurboQuantEstimator::InnerProduct => "prod_",
                },
                match bits {
                    TurboQuantBits::Bits4 => "4bit",
                    TurboQuantBits::Bits2 => "2bit",
                    TurboQuantBits::Bits1_5 => "1_5bit",
                    TurboQuantBits::Bits1 => "1bit",
                }
            ),
        }
    }
}

enum Artifact {
    Quantized(Box<QuantizedSegment>),
    Turbo(Box<TurboQuantSegment>),
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let vectors: usize = arguments
        .next()
        .map(|value| value.parse().expect("vectors must be an integer"))
        .unwrap_or(20_000);
    let dimension_list: Vec<usize> = arguments
        .next()
        .map(|value| {
            value
                .split(',')
                .map(|dims| dims.parse().expect("dimensions must be integers"))
                .collect()
        })
        .unwrap_or_else(|| vec![128, 384, 768]);
    let codecs = [
        Codec::Scalar,
        Codec::Product(ProductCompression::X8),
        Codec::Product(ProductCompression::X32),
        Codec::Binary,
        Codec::Turbo(TurboQuantBits::Bits4, TurboQuantEstimator::Mse),
        Codec::Turbo(TurboQuantBits::Bits2, TurboQuantEstimator::Mse),
        Codec::Turbo(TurboQuantBits::Bits1, TurboQuantEstimator::Mse),
        Codec::Turbo(TurboQuantBits::Bits4, TurboQuantEstimator::InnerProduct),
    ];
    let scope = ScopeId::new("instance:vector-cost-calibration").unwrap();
    let mut profiles = Vec::new();
    for &dimensions in &dimension_list {
        let candidates = (0..vectors)
            .map(|row| candidate(&scope, row, dimensions))
            .collect::<Vec<_>>();
        let queries = (0..QUERIES)
            .map(|query| vector(1_000_003 + query as u64, dimensions))
            .collect::<Vec<_>>();
        let exact_ns = median_ns(|| {
            for query in &queries {
                let request = request(&scope, query, vectors, TOP_K, SearchMode::Exact);
                std::hint::black_box(search_exact_ref(&request, &candidates).unwrap());
            }
        }) / QUERIES as f64;
        let exact_per_candidate = exact_ns / vectors as f64;
        let mut rows = Vec::new();
        for codec in codecs {
            let artifact = build(codec, &scope, &candidates, dimensions, vectors);
            let scan_ns = median_ns(|| {
                for query in &queries {
                    let request = request(
                        &scope,
                        query,
                        vectors,
                        TOP_K,
                        SearchMode::RequireApproximate {
                            exact_rerank: RERANK,
                        },
                    );
                    let hits = match &artifact {
                        Artifact::Quantized(segment) => segment
                            .search_candidates_at(
                                &request,
                                RERANK,
                                vectors as u64,
                                QuantizedKernel::Auto,
                            )
                            .unwrap(),
                        Artifact::Turbo(segment) => segment
                            .search_candidates_at_with_kernel(
                                &request,
                                RERANK,
                                vectors as u64,
                                QuantizedKernel::Auto,
                            )
                            .unwrap(),
                    };
                    std::hint::black_box(hits);
                }
            }) / QUERIES as f64;
            let (packed, full) = match &artifact {
                Artifact::Quantized(segment) => (
                    segment.descriptor().packed_vector_bytes,
                    segment.descriptor().full_precision_vector_bytes,
                ),
                Artifact::Turbo(segment) => (
                    segment.descriptor().packed_vector_bytes,
                    segment.descriptor().full_precision_vector_bytes,
                ),
            };
            rows.push(json!({
                "codec": codec.label(),
                "packed_to_full_byte_ratio": packed as f64 / full as f64,
                "scan_ns_per_query": scan_ns,
                "scan_ns_per_candidate": scan_ns / vectors as f64,
                "relative_cost_per_candidate": scan_ns / exact_ns,
            }));
        }
        profiles.push(json!({
            "dimensions": dimensions,
            "exact_ns_per_query": exact_ns,
            "exact_ns_per_candidate": exact_per_candidate,
            "codecs": rows,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "rrflow.vector-cost-calibration.v1",
            "vectors": vectors,
            "queries": QUERIES,
            "rounds": ROUNDS,
            "top_k": TOP_K,
            "rerank": RERANK,
            "metric": "cosine",
            "kernel": "auto",
            "target_os": std::env::consts::OS,
            "target_arch": std::env::consts::ARCH,
            "release": !cfg!(debug_assertions),
            "profiles": profiles,
        }))
        .unwrap()
    );
}

/// Median wall time of `ROUNDS` runs after one warm-up run.
fn median_ns(mut run: impl FnMut()) -> f64 {
    run();
    let mut samples = (0..ROUNDS)
        .map(|_| {
            let started = Instant::now();
            run();
            started.elapsed().as_nanos() as f64
        })
        .collect::<Vec<_>>();
    samples.sort_by(f64::total_cmp);
    samples[ROUNDS / 2]
}

fn build(
    codec: Codec,
    scope: &ScopeId,
    candidates: &[VectorCandidate],
    dimensions: usize,
    vectors: usize,
) -> Artifact {
    let id = ProjectionId::new(format!("quant-calibration-{}", codec.label())).unwrap();
    match codec {
        Codec::Turbo(bits, estimator) => Artifact::Turbo(Box::new(
            TurboQuantSegment::build(
                TurboQuantSegmentConfig {
                    id,
                    scope: scope.clone(),
                    field: "body".into(),
                    dimensions,
                    metric: ScoreMetric::Cosine,
                    bits,
                    seed: 0x5eed,
                    embedding_model: None,
                    filter_properties: BTreeSet::new(),
                    estimator,
                },
                1,
                vectors as u64,
                candidates.to_vec(),
            )
            .unwrap(),
        )),
        Codec::Scalar | Codec::Product(_) | Codec::Binary => Artifact::Quantized(Box::new(
            QuantizedSegment::build(
                QuantizedSegmentConfig {
                    id,
                    scope: scope.clone(),
                    field: "body".into(),
                    dimensions,
                    metric: ScoreMetric::Cosine,
                    method: match codec {
                        Codec::Scalar => QuantizationMethod::Scalar,
                        Codec::Product(compression) => QuantizationMethod::Product { compression },
                        Codec::Binary => QuantizationMethod::Binary,
                        Codec::Turbo(..) => unreachable!(),
                    },
                    embedding_model: None,
                    filter_properties: BTreeSet::new(),
                },
                1,
                vectors as u64,
                candidates.to_vec(),
            )
            .unwrap(),
        )),
    }
}

fn request(
    scope: &ScopeId,
    query: &[f32],
    vectors: usize,
    top_k: usize,
    mode: SearchMode,
) -> SearchRequest {
    SearchRequest {
        read: ReadStamp::new(
            scope.clone(),
            None,
            0,
            vectors as u64,
            Some("11".repeat(32)),
        )
        .unwrap(),
        scope: scope.clone(),
        valid_at: 10,
        field: "body".into(),
        query: VectorQuery::Dense {
            values: query.to_vec(),
        },
        metric: ScoreMetric::Cosine,
        embedding_model: None,
        top_k,
        mode,
        filter: None,
    }
}

fn candidate(scope: &ScopeId, row: usize, dimensions: usize) -> VectorCandidate {
    VectorCandidate {
        scope: scope.clone(),
        source_cursor: row as u64 + 1,
        vector: RuntimeVector {
            reference: RuntimeRef::new("embedding", format!("v{row}")).unwrap(),
            subject: RuntimeRef::new("document", format!("v{row}")).unwrap(),
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
