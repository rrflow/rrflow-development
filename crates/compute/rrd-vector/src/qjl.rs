//! The Quantized Johnson-Lindenstrauss projection used by TurboQuant's
//! inner-product estimator (arXiv:2504.19874, Section 3.2). The paper uses a
//! dense matrix of independent standard normal entries. This implementation
//! deterministically approximates those draws from a seed using finite precision.
//! Cross-platform bitwise reproducibility requires native qualification.

use crate::contract::invalid;
use rrd_core::Result;

/// Padded dimensions the dense projection accepts at most: 4096 × 4096
/// entries, 64 MiB of `f32`.
pub const MAX_QJL_DIMENSIONS: usize = 4096;
/// Keeps S's draws apart from the rotation's, which use the same seed.
const QJL_SALT: u64 = 0x514a_4c5f_7475_7262;

/// S, materialized row by row.
#[derive(Debug, Clone)]
pub struct QjlProjection {
    dimensions: usize,
    seed: u64,
    entries: Vec<f32>,
}

impl QjlProjection {
    pub fn new(dimensions: usize, seed: u64) -> Result<Self> {
        if dimensions == 0 || dimensions > MAX_QJL_DIMENSIONS {
            return invalid(format!(
                "the TurboQuant inner-product projection accepts 1..={MAX_QJL_DIMENSIONS} padded dimensions"
            ));
        }
        let mut entries = Vec::with_capacity(dimensions * dimensions);
        for row in 0..dimensions {
            for column in 0..dimensions {
                entries.push(normal(seed, row, column) as f32);
            }
        }
        Ok(Self {
            dimensions,
            seed,
            entries,
        })
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    pub(crate) fn matches_seed(&self, seed: u64) -> bool {
        self.seed == seed
    }

    /// Project finite values of the configured dimension.
    pub fn project(&self, values: &[f64]) -> Result<Vec<f64>> {
        if values.len() != self.dimensions || values.iter().any(|value| !value.is_finite()) {
            return invalid("QJL projection requires finite values with matching dimensions");
        }
        let projected = self
            .entries
            .chunks_exact(self.dimensions)
            .map(|row| {
                row.iter()
                    .zip(values)
                    .map(|(entry, value)| f64::from(*entry) * value)
                    .sum::<f64>()
            })
            .collect::<Vec<_>>();
        if projected.iter().any(|value| !value.is_finite()) {
            return invalid("QJL projection produced non-finite values");
        }
        Ok(projected)
    }
}

/// Approximate a normal draw by the polar method using the entry's seeded state.
fn normal(seed: u64, row: usize, column: usize) -> f64 {
    let mut state = mix(mix(seed ^ QJL_SALT, row as u64 + 1), column as u64 + 1);
    let mut attempt = 0u64;
    loop {
        attempt += 1;
        state = mix(state, attempt);
        let u = unit(state >> 11) * 2.0 - 1.0;
        state = mix(state, attempt + 0x100);
        let v = unit(state >> 11) * 2.0 - 1.0;
        let s = u * u + v * v;
        if s > 0.0 && s < 1.0 {
            return u * (-2.0 * ln(s) / s).sqrt();
        }
    }
}

/// A 53-bit integer as a fraction in [0, 1).
fn unit(bits: u64) -> f64 {
    bits as f64 / (1u64 << 53) as f64
}

/// The natural logarithm of `x` in (0, 1), from IEEE arithmetic alone:
/// x = m · 2^e with m in [1, 2), and ln m = 2·atanh(t) for
/// t = (m − 1)/(m + 1) < 1/3, summed to double precision.
fn ln(x: f64) -> f64 {
    debug_assert!(x > 0.0 && x < 1.0);
    let bits = x.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mantissa = f64::from_bits((bits & ((1u64 << 52) - 1)) | (1023u64 << 52));
    let t = (mantissa - 1.0) / (mantissa + 1.0);
    let square = t * t;
    let mut term = t;
    let mut series = 0.0;
    for odd in (1..=41).step_by(2) {
        series += term / odd as f64;
        term *= square;
    }
    exponent as f64 * std::f64::consts::LN_2 + 2.0 * series
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
    fn the_logarithm_matches_the_platform_to_double_precision() {
        for x in [1e-300_f64, 1e-12, 0.001, 0.1, 0.25, 0.5, 0.75, 0.999_999] {
            let expected: f64 = x.ln();
            assert!(
                (ln(x) - expected).abs() <= 4.0 * f64::EPSILON * expected.abs().max(1.0),
                "ln({x}) = {} against {expected}",
                ln(x)
            );
        }
    }

    #[test]
    fn sampled_entries_have_expected_moments_and_repeat_for_the_same_seed() {
        let projection = QjlProjection::new(256, 7).unwrap();
        assert_eq!(
            projection.entries,
            QjlProjection::new(256, 7).unwrap().entries
        );
        assert_ne!(
            projection.entries,
            QjlProjection::new(256, 8).unwrap().entries
        );
        let count = projection.entries.len() as f64;
        let mean = projection
            .entries
            .iter()
            .map(|value| f64::from(*value))
            .sum::<f64>()
            / count;
        let variance = projection
            .entries
            .iter()
            .map(|value| (f64::from(*value) - mean).powi(2))
            .sum::<f64>()
            / count;
        let kurtosis = projection
            .entries
            .iter()
            .map(|value| (f64::from(*value) - mean).powi(4))
            .sum::<f64>()
            / count
            / variance.powi(2);
        // 65536 draws: the mean's standard error is 1/256.
        assert!(mean.abs() < 0.02, "mean {mean}");
        assert!((variance - 1.0).abs() < 0.02, "variance {variance}");
        assert!((kurtosis - 3.0).abs() < 0.1, "kurtosis {kurtosis}");
        let projected = projection.project(&vec![1.0; 256]).unwrap();
        assert_eq!(projected.len(), 256);
    }

    #[test]
    fn projection_rejects_mismatched_or_non_finite_inputs() {
        let projection = QjlProjection::new(8, 11).unwrap();
        assert!(projection.project(&[1.0; 7]).is_err());
        assert!(projection.project(&[1.0; 9]).is_err());
        assert!(projection.project(&[f64::NAN; 8]).is_err());
        assert!(projection.project(&[f64::INFINITY; 8]).is_err());
        let overflowing = projection.entries[..8]
            .iter()
            .map(|entry| if *entry >= 0.0 { f64::MAX } else { -f64::MAX })
            .collect::<Vec<_>>();
        assert!(projection.project(&overflowing).is_err());
        assert_eq!(projection.project(&[1.0; 8]).unwrap().len(), 8);
    }

    #[test]
    fn mismatched_projection_seed_is_rejected_by_encoder_and_query() {
        let projection = QjlProjection::new(8, 11).unwrap();
        let input = [1.0_f32; 8];
        assert!(crate::TurboQuantInnerProductCode::encode(
            &input,
            crate::TurboQuantBits::Bits4,
            12,
            &projection,
        )
        .is_err());
        assert!(crate::TurboQuantQuery::inner_product(
            &input,
            crate::ScoreMetric::Cosine,
            crate::TurboQuantBits::Bits4,
            12,
            &projection,
        )
        .is_err());
    }

    #[test]
    fn the_projection_is_bounded() {
        assert!(QjlProjection::new(0, 1).is_err());
        assert!(QjlProjection::new(MAX_QJL_DIMENSIONS + 1, 1).is_err());
    }
}
