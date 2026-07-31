//! Standard Normal density and cumulative distribution functions.
//!
//! # Design
//!
//! The cumulative distribution is evaluated via the complementary error
//! function:
//!
//! ```text
//! Φ(x)     = ½ · erfc(−x / √2)
//! 1 − Φ(x) = ½ · erfc( x / √2)
//! ```
//!
//! This is the numerically stable formulation required for deep OTM / ITM
//! Black pricing: the survival function [`norm_cdf_c`] never forms
//! `1 − Φ(x)` by subtraction, so there is no catastrophic cancellation in
//! the far right tail.
//!
//! `erfc` is provided by the [`libm`] crate because `f64::erfc` is still
//! unstable on current stable Rust. Results are portable across targets.
//!
//! The density uses the direct formula with the shared constant
//! [`crate::ONE_OVER_SQRT_TWO_PI`]; extreme `|x|` underflows to `0.0`
//! naturally.
//!
//! # Non-finite inputs
//!
//! | Input | [`norm_pdf`] | [`norm_cdf`] | [`norm_cdf_c`] |
//! |-------|--------------|--------------|----------------|
//! | `NaN` | `NaN`        | `NaN`        | `NaN`          |
//! | `+∞`  | `0.0`        | `1.0`        | `0.0`          |
//! | `−∞`  | `0.0`        | `0.0`        | `1.0`          |
//!
//! These functions do **not** return [`IVError`](crate::IVError); domain
//! validation of market inputs belongs at the pricing / IV API boundary.
//!
//! # Complexity
//!
//! Each call is \(O(1)\) with a small fixed number of floating-point
//! operations plus one `exp` (PDF) or one `erfc` (CDF / survival).
//!
//! # References
//!
//! - Abramowitz, M. & Stegun, I. A. (1964). *Handbook of Mathematical
//!   Functions*, §7.1 (relationship of Φ to erf / erfc).
//! - West, G. (2005). Better approximations to cumulative normal functions.
//! - Cody, W. J. (1969). Rational Chebyshev approximations for the error
//!   function (underlying many `erfc` implementations, including libm).

use core::f64::consts::FRAC_1_SQRT_2;

use crate::constants::ONE_OVER_SQRT_TWO_PI;

/// Standard Normal probability density function φ(x).
///
/// # Formula
///
/// ```text
/// φ(x) = (1 / √(2π)) · exp(−x² / 2)
/// ```
///
/// # Parameters
///
/// - `x` — point of evaluation (dimensionless; typically a Black `d₁` / `d₂`).
///
/// # Returns
///
/// Density value in `[0, 1/√(2π)]`. Underflows to `0.0` for `|x| ≳ 38.6`.
/// Returns `NaN` if `x` is `NaN`.
///
/// # Edge cases
///
/// - `φ(0) = 1/√(2π)`
/// - `φ(±∞) = 0`
/// - Even function: `φ(−x) = φ(x)` for finite `x`
///
/// # Examples
///
/// ```
/// use iv_engine::normal::norm_pdf;
/// use iv_engine::ONE_OVER_SQRT_TWO_PI;
///
/// let phi0 = norm_pdf(0.0);
/// assert!((phi0 - ONE_OVER_SQRT_TWO_PI).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn norm_pdf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if !x.is_finite() {
        return 0.0;
    }
    // Direct formula; exp underflows to 0 for |x| ≳ 38.6.
    ONE_OVER_SQRT_TWO_PI * (-0.5 * x * x).exp()
}

/// Standard Normal cumulative distribution function Φ(x).
///
/// # Formula
///
/// ```text
/// Φ(x) = ½ · erfc(−x / √2) = ∫_{−∞}^{x} φ(t) dt
/// ```
///
/// # Parameters
///
/// - `x` — upper integration limit (dimensionless).
///
/// # Returns
///
/// Probability in `[0, 1]`. Returns `NaN` if `x` is `NaN`.
///
/// # Edge cases
///
/// - `Φ(0) = ½`
/// - `Φ(+∞) = 1`, `Φ(−∞) = 0`
/// - Symmetry: `Φ(−x) = 1 − Φ(x)` (use [`norm_cdf_c`] for the right-hand
///   side when `x > 0` to avoid cancellation)
///
/// # Examples
///
/// ```
/// use iv_engine::normal::norm_cdf;
///
/// assert!((norm_cdf(0.0) - 0.5).abs() < 1e-15);
/// assert!(norm_cdf(8.0) > 1.0 - 1e-15);
/// assert!(norm_cdf(-8.0) < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn norm_cdf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if !x.is_finite() {
        return if x.is_sign_positive() { 1.0 } else { 0.0 };
    }
    // Φ(x) = ½ erfc(−x/√2). libm::erfc is accurate into the far tails.
    0.5 * libm::erfc(-x * FRAC_1_SQRT_2)
}

/// Standard Normal survival function 1 − Φ(x).
///
/// Computed as `½ · erfc(x / √2)` — **not** as `1.0 - norm_cdf(x)` — so the
/// right tail remains accurate for large positive `x` (deep OTM calls /
/// deep ITM puts in the Black formula).
///
/// # Parameters
///
/// - `x` — point of evaluation (dimensionless).
///
/// # Returns
///
/// Probability in `[0, 1]`. Returns `NaN` if `x` is `NaN`.
///
/// # Edge cases
///
/// - `1 − Φ(0) = ½`
/// - `1 − Φ(+∞) = 0`, `1 − Φ(−∞) = 1`
/// - Identity: `norm_cdf(x) + norm_cdf_c(x) = 1` for finite `x` (within a
///   few ulps)
///
/// # Examples
///
/// ```
/// use iv_engine::normal::{norm_cdf, norm_cdf_c};
///
/// let x = 6.0;
/// // Direct survival is tiny but non-zero; 1 - Φ(x) would flush to 0 in f64
/// // if Φ(x) rounds to 1 too early — erfc avoids that regime carefully.
/// assert!(norm_cdf_c(x) > 0.0);
/// assert!((norm_cdf(x) + norm_cdf_c(x) - 1.0).abs() < 1e-15);
/// ```
#[inline]
#[must_use]
pub fn norm_cdf_c(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if !x.is_finite() {
        return if x.is_sign_positive() { 0.0 } else { 1.0 };
    }
    0.5 * libm::erfc(x * FRAC_1_SQRT_2)
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::while_float,
    clippy::cast_precision_loss,
    clippy::unreadable_literal,
    clippy::inconsistent_digit_grouping
)]
mod tests {
    use super::*;
    use crate::constants::{DBL_EPSILON, ONE_OVER_SQRT_TWO_PI};

    #[test]
    fn pdf_at_zero() {
        assert_eq!(norm_pdf(0.0), ONE_OVER_SQRT_TWO_PI);
    }

    #[test]
    fn pdf_is_even() {
        for &x in &[0.5, 1.0, 2.5, 10.0, 30.0] {
            assert_eq!(norm_pdf(x), norm_pdf(-x));
        }
    }

    #[test]
    fn pdf_non_negative_and_max_at_zero() {
        let peak = norm_pdf(0.0);
        for i in -100..=100 {
            let x = f64::from(i) * 0.1;
            let p = norm_pdf(x);
            assert!(p >= 0.0);
            assert!(p <= peak + DBL_EPSILON);
        }
    }

    #[test]
    fn pdf_underflow_and_nonfinite() {
        assert_eq!(norm_pdf(100.0), 0.0);
        assert_eq!(norm_pdf(-100.0), 0.0);
        assert_eq!(norm_pdf(f64::INFINITY), 0.0);
        assert_eq!(norm_pdf(f64::NEG_INFINITY), 0.0);
        assert!(norm_pdf(f64::NAN).is_nan());
    }

    #[test]
    fn cdf_at_zero() {
        assert!((norm_cdf(0.0) - 0.5).abs() < 2.0 * DBL_EPSILON);
    }

    #[test]
    fn cdf_c_at_zero() {
        assert!((norm_cdf_c(0.0) - 0.5).abs() < 2.0 * DBL_EPSILON);
    }

    #[test]
    fn cdf_symmetry() {
        for &x in &[0.25, 0.5, 1.0, 1.5, 2.0, 3.0, 5.0, 8.0] {
            let sum = norm_cdf(x) + norm_cdf(-x);
            assert!(
                (sum - 1.0).abs() < 4.0 * DBL_EPSILON,
                "symmetry failed at {x}: sum={sum}"
            );
        }
    }

    #[test]
    fn cdf_plus_survival_is_one() {
        for &x in &[
            -10.0, -5.0, -2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0,
        ] {
            let sum = norm_cdf(x) + norm_cdf_c(x);
            assert!(
                (sum - 1.0).abs() < 4.0 * DBL_EPSILON,
                "cdf+cdf_c failed at {x}: sum={sum}"
            );
        }
    }

    #[test]
    fn cdf_known_values() {
        // Reference values: Φ(x) to ~1e-16 from high-precision sources /
        // erfc identity. Tolerances allow a few ulps of libm error.
        let cases = [
            (1.0, 0.841_344_746_068_542_9),
            (-1.0, 0.158_655_253_931_457_05),
            (2.0, 0.977_249_868_051_820_8),
            (-2.0, 0.022_750_131_948_179_195),
            (3.0, 0.998_650_101_968_369_9),
        ];
        for &(x, expected) in &cases {
            let got = norm_cdf(x);
            let err = (got - expected).abs();
            assert!(
                err < 1e-14,
                "Φ({x}): got {got}, expected {expected}, err {err}"
            );
        }
    }

    #[test]
    fn survival_matches_left_cdf() {
        for &x in &[0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0] {
            assert!(
                (norm_cdf_c(x) - norm_cdf(-x)).abs() < 4.0 * DBL_EPSILON,
                "survival asymmetry at {x}"
            );
        }
    }

    #[test]
    fn cdf_monotone() {
        let mut prev = norm_cdf(-20.0);
        for i in -39..=40 {
            let x = f64::from(i) * 0.5;
            let cur = norm_cdf(x);
            assert!(cur >= prev - 1e-15, "monotone broken near {x}");
            prev = cur;
        }
    }

    #[test]
    fn cdf_limits() {
        assert_eq!(norm_cdf(f64::INFINITY), 1.0);
        assert_eq!(norm_cdf(f64::NEG_INFINITY), 0.0);
        assert_eq!(norm_cdf_c(f64::INFINITY), 0.0);
        assert_eq!(norm_cdf_c(f64::NEG_INFINITY), 1.0);
        assert!(norm_cdf(f64::NAN).is_nan());
        assert!(norm_cdf_c(f64::NAN).is_nan());
    }

    #[test]
    fn far_right_survival_is_positive() {
        // Φ(8) rounds extremely close to 1; the complementary form must
        // still return a positive tail probability.
        let s = norm_cdf_c(8.0);
        assert!(s > 0.0);
        assert!(s < 1e-15);
        // Cross-check against known asymptotic order: φ(x)/x.
        let asym = norm_pdf(8.0) / 8.0;
        assert!((s / asym - 1.0).abs() < 0.02);
    }

    #[test]
    fn numerical_derivative_of_cdf_matches_pdf() {
        let h = 1e-6;
        for &x in &[-2.0, -0.5, 0.0, 0.5, 1.0, 2.0] {
            let deriv = (norm_cdf(x + h) - norm_cdf(x - h)) / (2.0 * h);
            let pdf = norm_pdf(x);
            let rel = (deriv - pdf).abs() / pdf.max(1e-300);
            assert!(rel < 1e-6, "deriv mismatch at {x}: {deriv} vs {pdf}");
        }
    }

    #[test]
    fn randomized_cdf_survival_consistency() {
        // Deterministic LCG — no extra test dependency.
        let mut state: u64 = 0x00C0_FFEE;
        for _ in 0..2_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            // 53-bit mantissa extract → Uniform(0, 1), then map to (-20, 20).
            let u = f64::from_bits(0x3FF0_0000_0000_0000 | (state >> 12)) - 1.0;
            let x = (u - 0.5) * 40.0;
            let sum = norm_cdf(x) + norm_cdf_c(x);
            assert!(
                (sum - 1.0).abs() < 8.0 * DBL_EPSILON,
                "random consistency failed at {x}"
            );
            assert!(norm_pdf(x) >= 0.0);
        }
    }
}
