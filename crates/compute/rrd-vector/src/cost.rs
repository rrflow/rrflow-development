//! What scanning each compressed access path costs per candidate, in
//! thousandths of one exact full-precision score.
//!
//! The planner compares paths in one unit: an exact scan of `n` candidates
//! costs `n`. A codec that scans every code, then exactly reranks
//! `exact_rerank` candidates and any newer overlay, costs
//! `ceil(n * per_mille / 1000) + exact_rerank + overlay`.
//!
//! The defaults are the worst run median, over bit widths, of
//! `examples/vector_cost_calibration.rs` in
//! `docs/evidence/e04-vector-cost-calibration-after-turboquant-scan-20000-windows.json`.
//! A deployment re-measures on its own hardware and replaces them.

use crate::{QuantizationMethod, TurboQuantEstimator};
use rrd_core::Result;
use serde::{Deserialize, Serialize};

/// The measured cost of one candidate at `dimensions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostPoint {
    pub dimensions: u32,
    pub per_mille: u32,
}

/// Per-candidate costs per codec, each a curve over dimensions, linear
/// between measured points and flat beyond them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorCostModel {
    pub turboquant_mse: Vec<CostPoint>,
    pub turboquant_inner_product: Vec<CostPoint>,
    pub scalar: Vec<CostPoint>,
    pub product: Vec<CostPoint>,
    pub binary: Vec<CostPoint>,
}

const fn points(values: [(u32, u32); 3]) -> [CostPoint; 3] {
    [
        CostPoint {
            dimensions: values[0].0,
            per_mille: values[0].1,
        },
        CostPoint {
            dimensions: values[1].0,
            per_mille: values[1].1,
        },
        CostPoint {
            dimensions: values[2].0,
            per_mille: values[2].1,
        },
    ]
}

impl Default for VectorCostModel {
    fn default() -> Self {
        Self {
            turboquant_mse: points([(128, 230), (384, 355), (768, 502)]).to_vec(),
            turboquant_inner_product: points([(128, 392), (384, 593), (768, 857)]).to_vec(),
            scalar: points([(128, 760), (384, 605), (768, 607)]).to_vec(),
            product: points([(128, 928), (384, 792), (768, 964)]).to_vec(),
            binary: points([(128, 779), (384, 594), (768, 530)]).to_vec(),
        }
    }
}

impl VectorCostModel {
    pub fn validate(&self) -> Result<()> {
        for curve in [
            &self.turboquant_mse,
            &self.turboquant_inner_product,
            &self.scalar,
            &self.product,
            &self.binary,
        ] {
            if curve.is_empty()
                || curve
                    .windows(2)
                    .any(|pair| pair[0].dimensions >= pair[1].dimensions)
                || curve
                    .iter()
                    .any(|point| point.dimensions == 0 || !(1..=100_000).contains(&point.per_mille))
            {
                return Err(rrd_core::Error::InvalidRuntime {
                    reason: "a vector cost curve needs ascending dimensions and per-mille costs in 1..=100000"
                        .into(),
                });
            }
        }
        Ok(())
    }

    pub fn turboquant(&self, estimator: TurboQuantEstimator) -> &[CostPoint] {
        match estimator {
            TurboQuantEstimator::Mse => &self.turboquant_mse,
            TurboQuantEstimator::InnerProduct => &self.turboquant_inner_product,
        }
    }

    pub fn quantized(&self, method: QuantizationMethod) -> &[CostPoint] {
        match method {
            QuantizationMethod::Scalar => &self.scalar,
            QuantizationMethod::Product { .. } => &self.product,
            QuantizationMethod::Binary => &self.binary,
        }
    }
}

/// The cost of scanning `candidates` codes at `dimensions` and exactly
/// reranking `exact_rerank` of them, in exact-score units.
pub(crate) fn codec_scan_cost(
    curve: &[CostPoint],
    dimensions: usize,
    candidates: usize,
    exact_rerank: usize,
) -> u64 {
    let per_mille = per_mille_at(curve, dimensions);
    let scan = (candidates as u128 * u128::from(per_mille)).div_ceil(1000);
    u64::try_from(scan.max(1))
        .unwrap_or(u64::MAX)
        .saturating_add(exact_rerank as u64)
}

fn per_mille_at(curve: &[CostPoint], dimensions: usize) -> u32 {
    let dimensions = u32::try_from(dimensions).unwrap_or(u32::MAX);
    let (first, last) = (curve[0], curve[curve.len() - 1]);
    if dimensions <= first.dimensions {
        return first.per_mille;
    }
    if dimensions >= last.dimensions {
        return last.per_mille;
    }
    let upper = curve
        .iter()
        .position(|point| point.dimensions >= dimensions)
        .expect("a dimension inside the curve has an upper point");
    let (low, high) = (curve[upper - 1], curve[upper]);
    let span = i64::from(high.dimensions - low.dimensions);
    let offset = i64::from(dimensions - low.dimensions);
    let delta = i64::from(high.per_mille) - i64::from(low.per_mille);
    (i64::from(low.per_mille) + delta * offset / span) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_is_valid_and_interpolates_between_measured_points() {
        let model = VectorCostModel::default();
        model.validate().unwrap();
        let curve = model.turboquant(TurboQuantEstimator::Mse);
        assert_eq!(per_mille_at(curve, 64), 230);
        assert_eq!(per_mille_at(curve, 128), 230);
        assert_eq!(per_mille_at(curve, 256), 292);
        assert_eq!(per_mille_at(curve, 768), 502);
        assert_eq!(per_mille_at(curve, 4096), 502);
        // 2,044 codes at 0.23 each, then 32 exact reranks.
        assert_eq!(codec_scan_cost(curve, 64, 2_044, 32), 471 + 32);
    }

    #[test]
    fn malformed_curves_are_refused() {
        let mut model = VectorCostModel::default();
        model.binary.reverse();
        assert!(model.validate().is_err());
        let mut model = VectorCostModel::default();
        model.scalar.clear();
        assert!(model.validate().is_err());
        let mut model = VectorCostModel::default();
        model.product[0].per_mille = 0;
        assert!(model.validate().is_err());
    }
}
