//! Discounted Black–Scholes–Merton pricing on a spot underlying.
//!
//! # Design
//!
//! Black–Scholes is implemented as a thin wrapper around
//! [`crate::black_price`]:
//!
//! ```text
//! F  = S · exp((r − q) · T)
//! DF = exp(−r · T)
//! BS = DF · Black(F, K, T, σ)
//! ```
//!
//! All numerical work (ITM/OTM stability, CDF evaluation) lives in the
//! Black-76 kernel; this module only maps spot / rates into forward space
//! and applies the discount factor.
//!
//! # Parameters / units
//!
//! | Name         | Symbol | Units / convention                |
//! |--------------|--------|-----------------------------------|
//! | `spot`       | `S`    | currency / index points           |
//! | `strike`     | `K`    | same as spot                      |
//! | `maturity`   | `T`    | years                             |
//! | `rate`       | `r`    | continuously compounded risk-free |
//! | `dividend`   | `q`    | continuous dividend / borrow yield|
//! | `volatility` | `σ`    | absolute (e.g. `0.2` = 20%)       |
//!
//! # References
//!
//! Black, F. & Scholes, M. (1973). The pricing of options and corporate liabilities.

use crate::black::{black_price, validate_black_inputs};
use crate::constants::is_finite;
use crate::errors::IVError;

/// Discounted Black–Scholes–Merton option price.
///
/// # Errors
///
/// - [`IVError::InvalidInput`] if `spot ≤ 0`, any input is non-finite, or the
///   implied forward / discount factor is non-finite
/// - Propagates [`IVError`] from [`black_price`] for strike / maturity /
///   volatility domain violations
///
/// # Edge cases
///
/// - `σ = 0` → discounted intrinsic `e^{−rT} · max(±(F − K), 0)`
/// - `q = r` → forward equals spot
/// - Large `|r|` / `|q|` that overflow `exp` → [`IVError::InvalidInput`]
///
/// # Examples
///
/// ```
/// use iv_engine::black_scholes_price;
///
/// let px = black_scholes_price(100.0, 100.0, 1.0, 0.05, 0.02, 0.2, true).unwrap();
/// assert!(px > 0.0);
/// ```
#[inline]
pub fn black_scholes_price(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
    is_call: bool,
) -> Result<f64, IVError> {
    if !(is_finite(spot) && is_finite(rate) && is_finite(dividend)) {
        return Err(IVError::invalid("non-finite Black-Scholes input"));
    }
    if spot <= 0.0 {
        return Err(IVError::invalid("spot must be positive"));
    }

    // Re-use Black validators for K, T, σ (also rejects non-finite strike etc.).
    // Forward is checked after the exponential map.
    validate_black_inputs(spot, strike, maturity, volatility)?;

    let forward = spot * ((rate - dividend) * maturity).exp();
    if !is_finite(forward) || forward <= 0.0 {
        return Err(IVError::invalid(
            "non-finite or non-positive forward from spot/rates",
        ));
    }

    let discount = (-rate * maturity).exp();
    if !is_finite(discount) {
        return Err(IVError::invalid("non-finite discount factor"));
    }

    let undiscounted = black_price(forward, strike, maturity, volatility, is_call)?;
    Ok(discount * undiscounted)
}

/// Forward price `F = S · exp((r − q) · T)` from Black–Scholes inputs.
///
/// # Errors
///
/// Returns [`IVError::InvalidInput`] if inputs are non-finite, `spot ≤ 0`,
/// `maturity ≤ 0`, or the exponential overflows.
#[inline]
pub fn black_scholes_forward(
    spot: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
) -> Result<f64, IVError> {
    if !(is_finite(spot) && is_finite(maturity) && is_finite(rate) && is_finite(dividend)) {
        return Err(IVError::invalid("non-finite forward inputs"));
    }
    if spot <= 0.0 {
        return Err(IVError::invalid("spot must be positive"));
    }
    if maturity <= 0.0 {
        return Err(IVError::NegativeTime { maturity });
    }
    let forward = spot * ((rate - dividend) * maturity).exp();
    if !is_finite(forward) || forward <= 0.0 {
        return Err(IVError::invalid("non-finite or non-positive forward"));
    }
    Ok(forward)
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::unreadable_literal,
    clippy::many_single_char_names
)]
mod tests {
    use super::*;
    use crate::black::black_price;
    use crate::constants::DBL_EPSILON;

    fn approx_eq(a: f64, b: f64, tol: f64) {
        let err = (a - b).abs();
        assert!(err <= tol, "left={a} right={b} abs_err={err} tol={tol}");
    }

    #[test]
    fn reduces_to_discounted_black() {
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let q = 0.02;
        let v = 0.2;
        let f = black_scholes_forward(s, t, r, q).unwrap();
        let df = (-r * t).exp();
        let expected = df * black_price(f, k, t, v, true).unwrap();
        let got = black_scholes_price(s, k, t, r, q, v, true).unwrap();
        approx_eq(got, expected, 1e-14);
    }

    #[test]
    fn put_call_parity_discounted() {
        let s = 100.0;
        let k = 95.0;
        let t = 1.0;
        let r = 0.03;
        let q = 0.01;
        let v = 0.25;
        let c = black_scholes_price(s, k, t, r, q, v, true).unwrap();
        let p = black_scholes_price(s, k, t, r, q, v, false).unwrap();
        let f = black_scholes_forward(s, t, r, q).unwrap();
        let df = (-r * t).exp();
        approx_eq(c - p, df * (f - k), 1e-12);
    }

    #[test]
    fn zero_rates_match_black() {
        let c_bs = black_scholes_price(100.0, 100.0, 1.0, 0.0, 0.0, 0.2, true).unwrap();
        let c_b = black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
        approx_eq(c_bs, c_b, 1e-15);
    }

    #[test]
    fn rejects_non_positive_spot() {
        assert!(matches!(
            black_scholes_price(0.0, 100.0, 1.0, 0.0, 0.0, 0.2, true),
            Err(IVError::InvalidInput { .. })
        ));
    }

    #[test]
    fn zero_vol_discounted_intrinsic() {
        let s = 110.0;
        let k = 100.0;
        let t = 1.0;
        let r: f64 = 0.05;
        let q = 0.0;
        let f = s * (r * t).exp();
        let df = (-r * t).exp();
        let expected = df * (f - k);
        let got = black_scholes_price(s, k, t, r, q, 0.0, true).unwrap();
        approx_eq(got, expected, 1e-12);
    }

    #[test]
    fn forward_helper() {
        let f = black_scholes_forward(100.0, 1.0, 0.05, 0.02).unwrap();
        approx_eq(f, 100.0 * (0.03_f64).exp(), 4.0 * DBL_EPSILON);
    }
}
