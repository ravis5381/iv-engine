//! Householder correction primitives used by the LBR inverse.

/// LBR's third-order Householder correction factor.
#[inline]
pub(crate) fn householder_factor(newton: f64, halley: f64, hh3: f64) -> f64 {
    (1.0 + 0.5 * halley * newton) / (1.0 + newton * (halley + hh3 * newton / 6.0))
}
