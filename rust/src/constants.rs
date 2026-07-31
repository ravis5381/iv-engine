//! Mathematical and validation constants shared across the numerical core.
//!
//! # Design
//!
//! Constants fall into three groups:
//!
//! 1. **IEEE-754 / machine constants** — tolerances and overflow guards.
//! 2. **Normal-distribution constants** — √(2π) family used by Black and the
//!    asymptotic expansions in Let's Be Rational.
//! 3. **Input validation thresholds** — minimum admissible forward, strike,
//!    and maturity. Values at or below these thresholds are rejected via
//!    [`IVError`](crate::IVError).
//!
//! Algorithm-specific rational-approximation coefficients from Jäckel's
//! paper are **not** defined here; they live in [`crate::rational`] and are
//! introduced in Phase 5 so they stay co-located with the expansions that
//! use them.
//!
//! # Units
//!
//! Unless noted otherwise, times are in **years** (or any consistent model
//! time unit), and prices / forwards / strikes share the same currency unit.
//! Volatilities are **absolute** (e.g. `0.20` = 20%).

/// Machine epsilon for `f64` (`2⁻⁵² ≈ 2.220446049250313e-16`).
///
/// Used as the base unit for relative convergence tolerances. Prefer
/// multiples of this constant over hard-coded literals.
pub const DBL_EPSILON: f64 = f64::EPSILON;

/// Smallest positive normalized `f64` (`2⁻¹⁰²² ≈ 2.2250738585072014e-308`).
///
/// Guards against underflow when forming reciprocals or logarithms of
/// near-zero quantities.
pub const DBL_MIN: f64 = f64::MIN_POSITIVE;

/// Largest finite `f64` (`≈ 1.7976931348623157e+308`).
pub const DBL_MAX: f64 = f64::MAX;

/// `2π`.
pub const TWO_PI: f64 = 2.0 * core::f64::consts::PI;

/// `√2`.
pub const SQRT_TWO: f64 = core::f64::consts::SQRT_2;

/// `√(2π)`.
///
/// Appears in the Normal PDF: `φ(x) = exp(−x²/2) / √(2π)`.
///
/// Value is bit-identical to `(2.0 * π).sqrt()` on IEEE-754 `f64`.
pub const SQRT_TWO_PI: f64 = 2.506_628_274_631_000_2;

/// `1 / √(2π)`.
///
/// Prefactor of the Normal PDF; also used for Black vega
/// `ν = F · √T · φ(d₁)`.
///
/// Value is bit-identical to `1.0 / √(2π)` on IEEE-754 `f64`.
pub const ONE_OVER_SQRT_TWO_PI: f64 = 0.398_942_280_401_432_7;

/// `√(π/2)`.
///
/// Used by the normalised Black intrinsic asymptotics in Let's Be Rational
/// (the factor relating ATM normalised value to total volatility).
///
/// Value is bit-identical to `(π / 2.0).sqrt()` on IEEE-754 `f64`.
pub const SQRT_PI_OVER_TWO: f64 = 1.253_314_137_315_500_1;

/// `√(2/π)`.
///
/// Value is bit-identical to `(2.0 / π).sqrt()` on IEEE-754 `f64`.
pub const SQRT_TWO_OVER_PI: f64 = 0.797_884_560_802_865_4;

/// `ln(2)`.
pub const LN_TWO: f64 = core::f64::consts::LN_2;

/// Minimum admissible forward price.
///
/// Forwards at or below this value yield [`crate::IVError::NegativeForward`].
pub const MIN_FORWARD: f64 = 0.0;

/// Minimum admissible strike.
///
/// Strikes at or below this value yield [`crate::IVError::NegativeStrike`].
pub const MIN_STRIKE: f64 = 0.0;

/// Minimum admissible time to maturity.
///
/// Maturities at or below this value yield [`crate::IVError::NegativeTime`].
/// Exactly zero is rejected because the Black map is discontinuous there.
pub const MIN_MATURITY: f64 = 0.0;

/// Relative tolerance used for round-trip price checks in tests:
/// `|Black(IV(p)) − p| ≤ PRICE_ROUNDTRIP_TOL · max(1, |p|)`.
///
/// Set to `64 · ε` to absorb a handful of rounding errors through the
/// full price → IV → price pipeline while still demanding near
/// machine precision.
pub const PRICE_ROUNDTRIP_TOL: f64 = 64.0 * DBL_EPSILON;

/// Absolute floor applied together with [`PRICE_ROUNDTRIP_TOL`] so that
/// near-zero premiums are not held to an impossible relative standard.
pub const PRICE_ROUNDTRIP_ABS: f64 = 1e-14;

/// Maximum Householder / Newton iterations permitted in the IV refiner.
///
/// Let's Be Rational typically converges in ≤ 2 iterations; this budget is
/// a hard safety cap, not an expected iteration count.
pub const MAX_IV_ITERATIONS: u32 = 8;

/// Returns `true` if `x` is finite (`!NaN && !±∞`).
#[inline]
#[must_use]
pub fn is_finite(x: f64) -> bool {
    x.is_finite()
}

/// Returns `true` if `x` is strictly greater than `threshold`.
#[inline]
#[must_use]
pub fn is_positive_above(x: f64, threshold: f64) -> bool {
    x > threshold
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::assertions_on_constants,
    clippy::suboptimal_flops
)]
mod tests {
    use super::*;

    #[test]
    fn machine_constants_match_std() {
        assert_eq!(DBL_EPSILON, f64::EPSILON);
        assert_eq!(DBL_MIN, f64::MIN_POSITIVE);
        assert_eq!(DBL_MAX, f64::MAX);
    }

    #[test]
    fn two_pi_consistent() {
        assert!((TWO_PI - 2.0 * std::f64::consts::PI).abs() < DBL_EPSILON);
    }

    #[test]
    fn sqrt_two_consistent() {
        assert!((SQRT_TWO - std::f64::consts::SQRT_2).abs() < DBL_EPSILON);
    }

    #[test]
    fn sqrt_two_pi_accurate() {
        let expected = (2.0 * std::f64::consts::PI).sqrt();
        assert_eq!(SQRT_TWO_PI, expected);
    }

    #[test]
    fn one_over_sqrt_two_pi_is_reciprocal() {
        let expected = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
        assert_eq!(ONE_OVER_SQRT_TWO_PI, expected);
        let product = SQRT_TWO_PI * ONE_OVER_SQRT_TWO_PI;
        assert!((product - 1.0).abs() <= 4.0 * DBL_EPSILON);
    }

    #[test]
    fn sqrt_pi_over_two_accurate() {
        let expected = (std::f64::consts::PI / 2.0).sqrt();
        assert_eq!(SQRT_PI_OVER_TWO, expected);
    }

    #[test]
    fn sqrt_two_over_pi_accurate() {
        let expected = (2.0 / std::f64::consts::PI).sqrt();
        assert_eq!(SQRT_TWO_OVER_PI, expected);
    }

    #[test]
    fn ln_two_consistent() {
        assert!((LN_TWO - std::f64::consts::LN_2).abs() < DBL_EPSILON);
    }

    #[test]
    fn validation_thresholds_are_zero() {
        // Documented contract: strictly positive inputs required.
        assert_eq!(MIN_FORWARD, 0.0);
        assert_eq!(MIN_STRIKE, 0.0);
        assert_eq!(MIN_MATURITY, 0.0);
    }

    #[test]
    fn roundtrip_tolerance_sensible() {
        assert!(PRICE_ROUNDTRIP_TOL > DBL_EPSILON);
        assert!(PRICE_ROUNDTRIP_TOL < 1e-12);
        assert!(PRICE_ROUNDTRIP_ABS > 0.0);
        assert!(PRICE_ROUNDTRIP_ABS < 1e-10);
    }

    #[test]
    fn max_iv_iterations_positive() {
        assert!(MAX_IV_ITERATIONS >= 2);
    }

    #[test]
    fn is_finite_rejects_nan_and_inf() {
        assert!(is_finite(1.0));
        assert!(is_finite(-1e300));
        assert!(!is_finite(f64::NAN));
        assert!(!is_finite(f64::INFINITY));
        assert!(!is_finite(f64::NEG_INFINITY));
    }

    #[test]
    fn is_positive_above_boundary() {
        assert!(is_positive_above(1.0, 0.0));
        assert!(!is_positive_above(0.0, 0.0));
        assert!(!is_positive_above(-1.0, 0.0));
    }

    #[test]
    fn normal_pdf_at_zero_matches_one_over_sqrt_two_pi() {
        // φ(0) = 1/√(2π). Cross-check used by Phase 2 tests.
        let phi0 = (-0.0_f64).exp() / SQRT_TWO_PI;
        assert_eq!(ONE_OVER_SQRT_TWO_PI, phi0);
    }
}
