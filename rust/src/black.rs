//! Undiscounted Black-76 option pricing.
//!
//! # Design
//!
//! The undiscounted Black-76 price is evaluated in the **total-volatility**
//! coordinates `s = σ√T` and log-moneyness `x = ln(F) − ln(K)`:
//!
//! ```text
//! d₁ = (x + s²/2) / s
//! d₂ = d₁ − s
//! Call = F · Φ(d₁) − K · Φ(d₂)
//! Put  = K · Φ(−d₂) − F · Φ(−d₁)
//! ```
//!
//! Deep ITM contracts are priced via **put–call parity** so that only the
//! OTM complementary CDF branch is evaluated:
//!
//! ```text
//! Call = (F − K) + Put_OTM    when F ≥ K
//! Put  = (K − F) + Call_OTM   when F ≤ K
//! ```
//!
//! This avoids subtracting two large nearly-equal terms when `Φ(d)` rounds
//! to one. Combined with [`norm_cdf`] / [`norm_cdf_c`], the kernel is stable
//! for deep ITM, deep OTM, ATM, tiny `σ`, and large `σ`.
//!
//! Zero volatility returns the undiscounted intrinsic value. Zero maturity
//! is rejected ([`IVError::NegativeTime`]) because the map is discontinuous
//! at expiry.
//!
//! # Complexity
//!
//! \(O(1)\): one log, a handful of arithmetic ops, and two Normal CDF calls.
//!
//! # References
//!
//! - Black, F. (1976). The pricing of commodity contracts.
//! - Jäckel, P. (2015). *Let's Be Rational* (normalised Black coordinates).

use crate::constants::{is_finite, MIN_FORWARD, MIN_MATURITY, MIN_STRIKE};
use crate::errors::IVError;
use crate::normal::{norm_cdf, norm_cdf_c};

/// Undiscounted intrinsic value `max(±(F − K), 0)`.
///
/// # Parameters
///
/// - `forward` — forward price `F` (same units as strike)
/// - `strike` — strike `K`
/// - `is_call` — `true` for a call, `false` for a put
///
/// # Note
///
/// Does **not** validate inputs; used internally after domain checks and in
/// tests. Prefer [`black_price`] with zero volatility for a validated API.
#[inline]
#[must_use]
pub fn black_intrinsic(forward: f64, strike: f64, is_call: bool) -> f64 {
    if is_call {
        (forward - strike).max(0.0)
    } else {
        (strike - forward).max(0.0)
    }
}

/// Undiscounted Black-76 option price.
///
/// # Formula
///
/// See the module-level documentation. The returned value is the
/// **forward** (undiscounted) premium; multiply by a discount factor for
/// present value when needed (see [`crate::black_scholes_price`]).
///
/// # Parameters
///
/// | Name         | Symbol | Units                          |
/// |--------------|--------|--------------------------------|
/// | `forward`    | `F`    | currency / index points        |
/// | `strike`     | `K`    | same as forward                |
/// | `maturity`   | `T`    | years (or consistent time)     |
/// | `volatility` | `σ`    | absolute (e.g. `0.2` = 20%)    |
/// | `is_call`    | —      | `true` = call, `false` = put   |
///
/// # Errors
///
/// - [`IVError::NegativeForward`] if `forward ≤ 0`
/// - [`IVError::NegativeStrike`] if `strike ≤ 0`
/// - [`IVError::NegativeTime`] if `maturity ≤ 0`
/// - [`IVError::InvalidInput`] if any input is non-finite or `volatility < 0`
///
/// # Edge cases
///
/// - `σ = 0` → intrinsic
/// - `F = K` (ATM) → both premiums equal and positive for `σ > 0`
/// - `σ → ∞` → call → `F`, put → `K`
/// - Deep ITM / OTM → stable via put–call parity + complementary CDF
///
/// # Examples
///
/// ```
/// use iv_engine::black_price;
///
/// let call = black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
/// let put = black_price(100.0, 100.0, 1.0, 0.2, false).unwrap();
/// assert!((call - put).abs() < 1e-12); // ATM: F − K = 0
/// assert!(call > 0.0);
/// ```
#[inline]
pub fn black_price(
    forward: f64,
    strike: f64,
    maturity: f64,
    volatility: f64,
    is_call: bool,
) -> Result<f64, IVError> {
    validate_black_inputs(forward, strike, maturity, volatility)?;
    let total_vol = volatility * maturity.sqrt();
    Ok(black_price_total_vol(forward, strike, total_vol, is_call))
}

/// Black-76 price given total volatility `s = σ√T` directly.
///
/// Useful for Let's Be Rational and for callers that already work in
/// normalised coordinates. Inputs must already be validated; `s` must be
/// finite and `≥ 0`.
///
/// # Panics
///
/// Never panics. Invalid `s` (negative / non-finite) yields `NaN`.
#[inline]
#[must_use]
pub fn black_price_total_vol(forward: f64, strike: f64, total_vol: f64, is_call: bool) -> f64 {
    if !(total_vol.is_finite() && total_vol >= 0.0 && forward.is_finite() && strike.is_finite()) {
        return f64::NAN;
    }
    if total_vol == 0.0 {
        return black_intrinsic(forward, strike, is_call);
    }

    let x = log_moneyness(forward, strike);
    // s²/2 via mul_add for a clean fused step.
    let half_s2 = 0.5 * total_vol * total_vol;
    let d1 = (x + half_s2) / total_vol;
    let d2 = d1 - total_vol;

    // Price the OTM option, then lift to ITM via put–call parity when needed.
    // Parity (undiscounted): Call − Put = F − K.
    if is_call {
        if x >= 0.0 {
            // ITM call = intrinsic + OTM put
            let put_otm = strike * norm_cdf(-d2) - forward * norm_cdf(-d1);
            (forward - strike) + put_otm
        } else {
            // OTM call
            forward * norm_cdf(d1) - strike * norm_cdf(d2)
        }
    } else if x <= 0.0 {
        // ITM put = intrinsic + OTM call
        let call_otm = forward * norm_cdf(d1) - strike * norm_cdf(d2);
        (strike - forward) + call_otm
    } else {
        // OTM put — use survival / left-tail CDFs
        strike * norm_cdf_c(d2) - forward * norm_cdf_c(d1)
    }
}

/// Log-moneyness `x = ln(F) − ln(K)`.
///
/// Preferencing `ln(F) − ln(K)` over `ln(F/K)` avoids overflow of the ratio
/// for extreme forwards/strikes while preserving accuracy near ATM.
#[inline]
#[must_use]
pub fn log_moneyness(forward: f64, strike: f64) -> f64 {
    forward.ln() - strike.ln()
}

/// Validate shared Black-76 market inputs.
#[inline]
pub(crate) fn validate_black_inputs(
    forward: f64,
    strike: f64,
    maturity: f64,
    volatility: f64,
) -> Result<(), IVError> {
    if !(is_finite(forward) && is_finite(strike) && is_finite(maturity) && is_finite(volatility)) {
        return Err(IVError::invalid("non-finite Black input"));
    }
    if forward <= MIN_FORWARD {
        return Err(IVError::NegativeForward { forward });
    }
    if strike <= MIN_STRIKE {
        return Err(IVError::NegativeStrike { strike });
    }
    if maturity <= MIN_MATURITY {
        return Err(IVError::NegativeTime { maturity });
    }
    if volatility < 0.0 {
        return Err(IVError::invalid("volatility must be non-negative"));
    }
    Ok(())
}

/// `d₁` and `d₂` for the Black formula (validated inputs).
///
/// Returns `(d1, d2, total_vol)`. When `total_vol == 0`, both `d`s are `±∞`
/// according to the sign of log-moneyness (or `NaN` if ATM and `s = 0`).
///
/// Reserved for analytical Greeks; kept `pub(crate)` so the pricing and
/// greeks modules share one definition.
#[inline]
pub(crate) fn black_d1_d2(
    forward: f64,
    strike: f64,
    maturity: f64,
    volatility: f64,
) -> Result<(f64, f64, f64), IVError> {
    validate_black_inputs(forward, strike, maturity, volatility)?;
    let s = volatility * maturity.sqrt();
    if s == 0.0 {
        let x = log_moneyness(forward, strike);
        let d = if x > 0.0 {
            f64::INFINITY
        } else if x < 0.0 {
            f64::NEG_INFINITY
        } else {
            f64::NAN
        };
        return Ok((d, d, 0.0));
    }
    let x = log_moneyness(forward, strike);
    let d1 = (x + 0.5 * s * s) / s;
    let d2 = d1 - s;
    Ok((d1, d2, s))
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::unreadable_literal,
    clippy::many_single_char_names
)]
mod tests {
    use super::*;
    use crate::constants::{DBL_EPSILON, PRICE_ROUNDTRIP_ABS, PRICE_ROUNDTRIP_TOL};

    fn approx_eq(a: f64, b: f64, tol: f64) {
        let err = (a - b).abs();
        assert!(err <= tol, "left={a} right={b} abs_err={err} tol={tol}");
    }

    #[test]
    fn rejects_bad_inputs() {
        assert!(matches!(
            black_price(0.0, 100.0, 1.0, 0.2, true),
            Err(IVError::NegativeForward { .. })
        ));
        assert!(matches!(
            black_price(100.0, -1.0, 1.0, 0.2, true),
            Err(IVError::NegativeStrike { .. })
        ));
        assert!(matches!(
            black_price(100.0, 100.0, 0.0, 0.2, true),
            Err(IVError::NegativeTime { .. })
        ));
        assert!(matches!(
            black_price(100.0, 100.0, 1.0, -0.1, true),
            Err(IVError::InvalidInput { .. })
        ));
        assert!(matches!(
            black_price(f64::NAN, 100.0, 1.0, 0.2, true),
            Err(IVError::InvalidInput { .. })
        ));
    }

    #[test]
    fn zero_vol_is_intrinsic() {
        let f = 110.0;
        let k = 100.0;
        approx_eq(
            black_price(f, k, 1.0, 0.0, true).unwrap(),
            10.0,
            DBL_EPSILON,
        );
        approx_eq(
            black_price(f, k, 1.0, 0.0, false).unwrap(),
            0.0,
            DBL_EPSILON,
        );
        approx_eq(
            black_price(90.0, k, 1.0, 0.0, false).unwrap(),
            10.0,
            DBL_EPSILON,
        );
    }

    #[test]
    fn put_call_parity() {
        let cases = [
            (100.0, 100.0, 1.0, 0.2),
            (100.0, 80.0, 0.5, 0.3),
            (100.0, 120.0, 2.0, 0.15),
            (50.0, 100.0, 1.0, 0.5),
            (200.0, 100.0, 0.25, 0.1),
        ];
        for &(f, k, t, v) in &cases {
            let c = black_price(f, k, t, v, true).unwrap();
            let p = black_price(f, k, t, v, false).unwrap();
            approx_eq(c - p, f - k, 1e-12);
        }
    }

    #[test]
    fn atm_call_equals_put() {
        let c = black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
        let p = black_price(100.0, 100.0, 1.0, 0.2, false).unwrap();
        approx_eq(c, p, 1e-14);
        // Rough ATM magnitude: ≈ F · σ · √(T / 2π) ≈ 7.966
        approx_eq(c, 7.965_567_455_405_863, 1e-12);
    }

    #[test]
    fn price_bounds() {
        let f = 100.0;
        let k = 100.0;
        let c = black_price(f, k, 1.0, 0.25, true).unwrap();
        let p = black_price(f, k, 1.0, 0.25, false).unwrap();
        assert!(c >= 0.0 && c <= f);
        assert!(p >= 0.0 && p <= k);
        assert!(c >= black_intrinsic(f, k, true) - 1e-15);
        assert!(p >= black_intrinsic(f, k, false) - 1e-15);
    }

    #[test]
    fn deep_itm_call_near_forward_minus_strike() {
        let f = 100.0;
        let k = 50.0;
        let c = black_price(f, k, 1.0, 0.1, true).unwrap();
        // Deep ITM, low vol: close to intrinsic, strictly above it.
        assert!(c > f - k);
        assert!(c - (f - k) < 1e-3);
        assert!(c < f);
    }

    #[test]
    fn deep_otm_call_small() {
        let c = black_price(100.0, 200.0, 0.25, 0.1, true).unwrap();
        assert!(c >= 0.0);
        assert!(c < 1e-6);
    }

    #[test]
    fn large_vol_call_approaches_forward() {
        let f = 100.0;
        let c = black_price(f, 100.0, 1.0, 50.0, true).unwrap();
        approx_eq(c, f, 1e-10);
        let p = black_price(f, 100.0, 1.0, 50.0, false).unwrap();
        approx_eq(p, 100.0, 1e-10);
    }

    #[test]
    fn tiny_vol_matches_intrinsic() {
        let f = 105.0;
        let k = 100.0;
        let c = black_price(f, k, 1.0, 1e-12, true).unwrap();
        approx_eq(c, 5.0, 1e-6);
    }

    #[test]
    fn known_reference_value() {
        // F=100, K=100, T=1, σ=0.2 call — matches high-precision / common
        // reference implementations to ~1e-12.
        let c = black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
        approx_eq(c, 7.965_567_455_405_863, 1e-12);
    }

    #[test]
    fn monotone_in_vol() {
        let mut prev = black_price(100.0, 100.0, 1.0, 0.0, true).unwrap();
        for i in 1..=50 {
            let v = f64::from(i) * 0.02;
            let px = black_price(100.0, 100.0, 1.0, v, true).unwrap();
            assert!(px >= prev - 1e-14, "vol monotone broken at {v}");
            prev = px;
        }
    }

    #[test]
    fn total_vol_api_matches() {
        let f = 100.0;
        let k = 90.0;
        let t = 0.75;
        let v = 0.22;
        let a = black_price(f, k, t, v, true).unwrap();
        let b = black_price_total_vol(f, k, v * t.sqrt(), true);
        approx_eq(a, b, 1e-15);
    }

    #[test]
    fn parity_tolerances_match_roundtrip_budget() {
        // Sanity: parity residual well inside the future IV round-trip budget.
        let c = black_price(123.4, 100.0, 1.25, 0.33, true).unwrap();
        let p = black_price(123.4, 100.0, 1.25, 0.33, false).unwrap();
        let err = (c - p - (123.4 - 100.0)).abs();
        let tol = PRICE_ROUNDTRIP_TOL * 123.4 + PRICE_ROUNDTRIP_ABS;
        assert!(err < tol);
    }

    #[test]
    fn d1_d2_consistency() {
        let (d1, d2, s) = black_d1_d2(100.0, 100.0, 1.0, 0.2).unwrap();
        approx_eq(d1 - d2, s, 1e-15);
        approx_eq(d1, 0.1, 1e-15); // (0 + 0.5*0.04)/0.2 = 0.1
    }

    #[test]
    fn d1_d2_zero_vol_signs() {
        let (d1, _, s) = black_d1_d2(110.0, 100.0, 1.0, 0.0).unwrap();
        assert_eq!(s, 0.0);
        assert!(d1.is_infinite() && d1.is_sign_positive());
        let (d1, _, _) = black_d1_d2(90.0, 100.0, 1.0, 0.0).unwrap();
        assert!(d1.is_infinite() && d1.is_sign_negative());
    }
}
