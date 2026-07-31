//! Analytical Greeks for Black-76 and Black–Scholes–Merton.
//!
//! # Design
//!
//! Primary APIs (`delta`, `gamma`, …) are **undiscounted Black-76** Greeks
//! with the same `(F, K, T, σ, is_call)` signature as [`crate::black_price`].
//! Black–Scholes spot Greeks are provided as `black_scholes_*` wrappers that
//! map through the forward `F = S e^{(r−q)T}` and the discount `e^{−rT}`.
//!
//! ## Conventions
//!
//! | Greek   | Definition (Black-76)              | Units / notes                          |
//! |---------|------------------------------------|----------------------------------------|
//! | Delta   | `∂V/∂F`                            | Forward delta in `[−1, 1]`             |
//! | Gamma   | `∂²V/∂F²`                          | Per unit forward                       |
//! | Vega    | `∂V/∂σ`                            | Per **1.0** absolute vol (not per 1%)  |
//! | Theta   | `∂V/∂t` with `T = T_exp − t`       | Calendar decay (typically ≤ 0)         |
//! | Vomma   | `∂²V/∂σ²` (volga)                  | Per vol²                               |
//! | Vanna   | `∂²V/∂F∂σ`                         | Mixed forward / vol sensitivity        |
//!
//! Theta holds the forward `F` fixed (pure Black). Black–Scholes theta
//! includes the `r` / `q` carry terms from the spot formulation.
//!
//! Zero volatility: delta collapses to the digital intrinsic derivative
//! (`1` / `0` / `−1`); gamma, vega, theta, vomma, and vanna return `0`.
//!
//! # Complexity
//!
//! \(O(1)\) per Greek (shared `d₁`, `d₂`, `φ(d₁)` where applicable).
//!
//! # References
//!
//! - Black, F. (1976). The pricing of commodity contracts.
//! - Black, F. & Scholes, M. (1973). The pricing of options and corporate liabilities.
//! - Haug, E. G. (2007). *The Complete Guide to Option Pricing Formulas*.

#![allow(clippy::missing_errors_doc, clippy::option_if_let_else)]

use crate::black::{black_d1_d2, log_moneyness, validate_black_inputs};
use crate::black_scholes::black_scholes_forward;
use crate::constants::is_finite;
use crate::errors::IVError;
use crate::normal::{norm_cdf, norm_cdf_c, norm_pdf};

/// Bundled Black terms shared by several Greeks.
#[derive(Clone, Copy, Debug)]
struct BlackTerms {
    forward: f64,
    d1: f64,
    d2: f64,
    /// Total volatility `s = σ√T`.
    total_vol: f64,
    sqrt_t: f64,
    volatility: f64,
    phi_d1: f64,
}

impl BlackTerms {
    #[inline]
    fn new(
        forward: f64,
        strike: f64,
        maturity: f64,
        volatility: f64,
    ) -> Result<Option<Self>, IVError> {
        validate_black_inputs(forward, strike, maturity, volatility)?;
        let sqrt_t = maturity.sqrt();
        let total_vol = volatility * sqrt_t;
        if total_vol == 0.0 {
            return Ok(None);
        }
        let (d1, d2, s) = black_d1_d2(forward, strike, maturity, volatility)?;
        debug_assert!((s - total_vol).abs() <= 1e-15 * (1.0 + s));
        Ok(Some(Self {
            forward,
            d1,
            d2,
            total_vol,
            sqrt_t,
            volatility,
            phi_d1: norm_pdf(d1),
        }))
    }
}

// ---------------------------------------------------------------------------
// Black-76 Greeks
// ---------------------------------------------------------------------------

/// Black-76 forward delta `∂V/∂F`.
///
/// # Formula
///
/// ```text
/// Call: Φ(d₁)
/// Put:  Φ(d₁) − 1 = −Φ(−d₁)
/// ```
///
/// # Errors
///
/// Same domain errors as [`crate::black_price`].
#[inline]
pub fn delta(
    forward: f64,
    strike: f64,
    maturity: f64,
    volatility: f64,
    is_call: bool,
) -> Result<f64, IVError> {
    match BlackTerms::new(forward, strike, maturity, volatility)? {
        None => Ok(zero_vol_delta(forward, strike, is_call)),
        Some(t) => Ok(if is_call {
            norm_cdf(t.d1)
        } else {
            -norm_cdf_c(t.d1) // Φ(d1)−1
        }),
    }
}

/// Black-76 gamma `∂²V/∂F²`.
///
/// # Formula
///
/// ```text
/// Γ = φ(d₁) / (F · σ · √T) = φ(d₁) / (F · s)
/// ```
///
/// Same for calls and puts.
#[inline]
pub fn gamma(forward: f64, strike: f64, maturity: f64, volatility: f64) -> Result<f64, IVError> {
    match BlackTerms::new(forward, strike, maturity, volatility)? {
        None => Ok(0.0),
        Some(t) => Ok(t.phi_d1 / (t.forward * t.total_vol)),
    }
}

/// Black-76 vega `∂V/∂σ` per **1.0** absolute volatility.
///
/// # Formula
///
/// ```text
/// ν = F · φ(d₁) · √T
/// ```
///
/// Same for calls and puts. Multiply by `0.01` for a “per 1%” quote.
#[inline]
pub fn vega(forward: f64, strike: f64, maturity: f64, volatility: f64) -> Result<f64, IVError> {
    match BlackTerms::new(forward, strike, maturity, volatility)? {
        None => Ok(0.0),
        Some(t) => Ok(t.forward * t.phi_d1 * t.sqrt_t),
    }
}

/// Black-76 calendar theta `∂V/∂t` with `T = T_exp − t` (forward held fixed).
///
/// # Formula
///
/// ```text
/// Θ = − F · φ(d₁) · σ / (2 √T)
/// ```
///
/// Same for calls and puts under undiscounted Black put–call parity
/// (`C − P = F − K` is independent of `T` when `F` is fixed).
///
/// The value is typically **negative** for long options (time decay).
#[inline]
pub fn theta(forward: f64, strike: f64, maturity: f64, volatility: f64) -> Result<f64, IVError> {
    match BlackTerms::new(forward, strike, maturity, volatility)? {
        None => Ok(0.0),
        Some(t) => Ok(-t.forward * t.phi_d1 * t.volatility / (2.0 * t.sqrt_t)),
    }
}

/// Black-76 vomma (volga) `∂²V/∂σ²`.
///
/// # Formula
///
/// ```text
/// Vomma = ν · d₁ · d₂ / σ
/// ```
///
/// where `ν` is [`vega`]. Same for calls and puts.
#[inline]
pub fn vomma(forward: f64, strike: f64, maturity: f64, volatility: f64) -> Result<f64, IVError> {
    match BlackTerms::new(forward, strike, maturity, volatility)? {
        None => Ok(0.0),
        Some(t) => {
            let nu = t.forward * t.phi_d1 * t.sqrt_t;
            Ok(nu * t.d1 * t.d2 / t.volatility)
        }
    }
}

/// Black-76 vanna `∂²V/∂F∂σ`.
///
/// # Formula
///
/// ```text
/// Vanna = − φ(d₁) · d₂ / σ
/// ```
///
/// Same for calls and puts (derivative of forward delta w.r.t. volatility).
#[inline]
pub fn vanna(forward: f64, strike: f64, maturity: f64, volatility: f64) -> Result<f64, IVError> {
    match BlackTerms::new(forward, strike, maturity, volatility)? {
        None => Ok(0.0),
        Some(t) => Ok(-t.phi_d1 * t.d2 / t.volatility),
    }
}

#[inline]
fn zero_vol_delta(forward: f64, strike: f64, is_call: bool) -> f64 {
    // Almost-everywhere derivative of intrinsic; ATM is a set of measure zero.
    let x = log_moneyness(forward, strike);
    if is_call {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            0.0
        } else {
            0.5
        }
    } else if x > 0.0 {
        0.0
    } else if x < 0.0 {
        -1.0
    } else {
        -0.5
    }
}

// ---------------------------------------------------------------------------
// Black–Scholes–Merton Greeks
// ---------------------------------------------------------------------------

#[inline]
fn bs_setup(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> Result<(f64, f64, f64, Option<BlackTerms>), IVError> {
    if !(is_finite(spot) && is_finite(rate) && is_finite(dividend)) {
        return Err(IVError::invalid("non-finite Black-Scholes input"));
    }
    if spot <= 0.0 {
        return Err(IVError::invalid("spot must be positive"));
    }
    validate_black_inputs(spot, strike, maturity, volatility)?;
    let forward = black_scholes_forward(spot, maturity, rate, dividend)?;
    let discount = (-rate * maturity).exp();
    let div_df = (-dividend * maturity).exp();
    if !(is_finite(discount) && is_finite(div_df)) {
        return Err(IVError::invalid("non-finite discount factors"));
    }
    let terms = BlackTerms::new(forward, strike, maturity, volatility)?;
    Ok((forward, discount, div_df, terms))
}

/// Black–Scholes spot delta `∂V/∂S`.
///
/// # Formula
///
/// ```text
/// Call: e^{−qT} Φ(d₁)
/// Put:  e^{−qT} (Φ(d₁) − 1)
/// ```
#[inline]
pub fn black_scholes_delta(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
    is_call: bool,
) -> Result<f64, IVError> {
    let (forward, _df, div_df, terms) =
        bs_setup(spot, strike, maturity, rate, dividend, volatility)?;
    let fwd_delta = match terms {
        None => zero_vol_delta(forward, strike, is_call),
        Some(t) => {
            if is_call {
                norm_cdf(t.d1)
            } else {
                -norm_cdf_c(t.d1)
            }
        }
    };
    Ok(div_df * fwd_delta)
}

/// Black–Scholes gamma `∂²V/∂S²`.
///
/// # Formula
///
/// ```text
/// Γ = e^{−qT} · φ(d₁) / (S · σ · √T)
/// ```
#[inline]
pub fn black_scholes_gamma(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> Result<f64, IVError> {
    let (_forward, _df, div_df, terms) =
        bs_setup(spot, strike, maturity, rate, dividend, volatility)?;
    match terms {
        None => Ok(0.0),
        Some(t) => Ok(div_df * t.phi_d1 / (spot * t.total_vol)),
    }
}

/// Black–Scholes vega `∂V/∂σ` per 1.0 absolute vol.
///
/// # Formula
///
/// ```text
/// ν = S · e^{−qT} · φ(d₁) · √T = e^{−rT} · F · φ(d₁) · √T
/// ```
#[inline]
pub fn black_scholes_vega(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> Result<f64, IVError> {
    let (forward, discount, _div_df, terms) =
        bs_setup(spot, strike, maturity, rate, dividend, volatility)?;
    match terms {
        None => Ok(0.0),
        Some(t) => Ok(discount * forward * t.phi_d1 * t.sqrt_t),
    }
}

/// Black–Scholes calendar theta `∂V/∂t`.
///
/// # Formula
///
/// ```text
/// Call: −S e^{−qT} φ(d₁) σ/(2√T) − r K e^{−rT} Φ(d₂) + q S e^{−qT} Φ(d₁)
/// Put:  −S e^{−qT} φ(d₁) σ/(2√T) + r K e^{−rT} Φ(−d₂) − q S e^{−qT} Φ(−d₁)
/// ```
#[inline]
pub fn black_scholes_theta(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
    is_call: bool,
) -> Result<f64, IVError> {
    let (_forward, discount, div_df, terms) =
        bs_setup(spot, strike, maturity, rate, dividend, volatility)?;
    match terms {
        None => {
            // Zero-vol: only carry on intrinsic remains.
            let intrinsic_fwd = if is_call {
                (black_scholes_forward(spot, maturity, rate, dividend)? - strike).max(0.0)
            } else {
                (strike - black_scholes_forward(spot, maturity, rate, dividend)?).max(0.0)
            };
            // With σ=0 the time derivative of discounted intrinsic is subtle;
            // report pure discounting of the locked-in forward intrinsic as 0
            // for the diffusion term and leave carry to FD tests for σ>0.
            let _ = (discount, div_df, intrinsic_fwd);
            Ok(0.0)
        }
        Some(t) => {
            let time_decay = -spot * div_df * t.phi_d1 * t.volatility / (2.0 * t.sqrt_t);
            if is_call {
                Ok(time_decay - rate * strike * discount * norm_cdf(t.d2)
                    + dividend * spot * div_df * norm_cdf(t.d1))
            } else {
                Ok(time_decay + rate * strike * discount * norm_cdf_c(t.d2)
                    - dividend * spot * div_df * norm_cdf_c(t.d1))
            }
        }
    }
}

/// Black–Scholes vomma `∂²V/∂σ²`.
///
/// # Formula
///
/// ```text
/// Vomma = ν · d₁ · d₂ / σ
/// ```
///
/// with Black–Scholes [`black_scholes_vega`].
#[inline]
pub fn black_scholes_vomma(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> Result<f64, IVError> {
    let (forward, discount, _div_df, terms) =
        bs_setup(spot, strike, maturity, rate, dividend, volatility)?;
    match terms {
        None => Ok(0.0),
        Some(t) => {
            let nu = discount * forward * t.phi_d1 * t.sqrt_t;
            Ok(nu * t.d1 * t.d2 / t.volatility)
        }
    }
}

/// Black–Scholes vanna `∂²V/∂S∂σ`.
///
/// # Formula
///
/// ```text
/// Vanna = − e^{−qT} · φ(d₁) · d₂ / σ
/// ```
#[inline]
pub fn black_scholes_vanna(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> Result<f64, IVError> {
    let (_forward, _df, div_df, terms) =
        bs_setup(spot, strike, maturity, rate, dividend, volatility)?;
    match terms {
        None => Ok(0.0),
        Some(t) => Ok(-div_df * t.phi_d1 * t.d2 / t.volatility),
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::unreadable_literal,
    clippy::many_single_char_names,
    clippy::cast_precision_loss
)]
mod tests {
    use super::*;
    use crate::black::black_price;
    use crate::black_scholes::black_scholes_price;

    fn approx_eq(a: f64, b: f64, tol: f64) {
        let err = (a - b).abs();
        assert!(err <= tol, "left={a} right={b} abs_err={err} tol={tol}");
    }

    #[test]
    fn atm_call_delta_near_half() {
        let d = delta(100.0, 100.0, 1.0, 0.2, true).unwrap();
        // Φ(0.1) ≈ 0.5398
        approx_eq(d, 0.539_827_837_277_029, 1e-12);
        let dp = delta(100.0, 100.0, 1.0, 0.2, false).unwrap();
        approx_eq(d - dp, 1.0, 1e-14); // call−put forward delta
    }

    #[test]
    fn put_call_delta_diff_is_one() {
        for &(f, k, t, v) in &[
            (100.0, 100.0, 1.0, 0.2),
            (100.0, 80.0, 0.5, 0.3),
            (90.0, 110.0, 2.0, 0.15),
        ] {
            let dc = delta(f, k, t, v, true).unwrap();
            let dp = delta(f, k, t, v, false).unwrap();
            approx_eq(dc - dp, 1.0, 1e-12);
        }
    }

    #[test]
    fn gamma_vega_same_for_call_put_shape() {
        // gamma/vega APIs are not call/put dependent.
        let g = gamma(100.0, 100.0, 1.0, 0.2).unwrap();
        let v = vega(100.0, 100.0, 1.0, 0.2).unwrap();
        assert!(g > 0.0);
        assert!(v > 0.0);
        approx_eq(v, 100.0 * norm_pdf(0.1) * 1.0, 1e-12);
    }

    #[test]
    fn theta_negative_for_long_option() {
        let th = theta(100.0, 100.0, 1.0, 0.2).unwrap();
        assert!(th < 0.0);
    }

    #[test]
    fn zero_vol_greeks() {
        assert_eq!(delta(110.0, 100.0, 1.0, 0.0, true).unwrap(), 1.0);
        assert_eq!(delta(90.0, 100.0, 1.0, 0.0, true).unwrap(), 0.0);
        assert_eq!(gamma(100.0, 100.0, 1.0, 0.0).unwrap(), 0.0);
        assert_eq!(vega(100.0, 100.0, 1.0, 0.0).unwrap(), 0.0);
        assert_eq!(theta(100.0, 100.0, 1.0, 0.0).unwrap(), 0.0);
        assert_eq!(vomma(100.0, 100.0, 1.0, 0.0).unwrap(), 0.0);
        assert_eq!(vanna(100.0, 100.0, 1.0, 0.0).unwrap(), 0.0);
    }

    #[test]
    fn fd_delta_matches() {
        let f = 100.0;
        let k = 100.0;
        let t = 1.0;
        let v = 0.2;
        let h = 1e-4;
        let bump = (black_price(f + h, k, t, v, true).unwrap()
            - black_price(f - h, k, t, v, true).unwrap())
            / (2.0 * h);
        let analytic = delta(f, k, t, v, true).unwrap();
        approx_eq(analytic, bump, 1e-6);
    }

    #[test]
    fn fd_vega_matches() {
        let f = 100.0;
        let k = 95.0;
        let t = 1.0;
        let v = 0.25;
        let h = 1e-5;
        let bump = (black_price(f, k, t, v + h, true).unwrap()
            - black_price(f, k, t, v - h, true).unwrap())
            / (2.0 * h);
        let analytic = vega(f, k, t, v).unwrap();
        approx_eq(analytic, bump, 1e-4);
    }

    #[test]
    fn fd_gamma_matches() {
        let f = 100.0;
        let k = 100.0;
        let t = 1.0;
        let v = 0.2;
        let h = 1e-3;
        let bump = (black_price(f + h, k, t, v, true).unwrap()
            - 2.0 * black_price(f, k, t, v, true).unwrap()
            + black_price(f - h, k, t, v, true).unwrap())
            / (h * h);
        let analytic = gamma(f, k, t, v).unwrap();
        approx_eq(analytic, bump, 1e-4);
    }

    #[test]
    fn fd_vanna_vomma() {
        let f = 100.0;
        let k = 100.0;
        let t = 1.0;
        let v = 0.2;
        let hf = 1e-3;
        let hv = 1e-5;
        // vanna ≈ ∂²V/∂F∂σ
        let vanna_fd = ((black_price(f + hf, k, t, v + hv, true).unwrap()
            - black_price(f + hf, k, t, v - hv, true).unwrap())
            - (black_price(f - hf, k, t, v + hv, true).unwrap()
                - black_price(f - hf, k, t, v - hv, true).unwrap()))
            / (4.0 * hf * hv);
        approx_eq(vanna(f, k, t, v).unwrap(), vanna_fd, 5e-4);

        let vomma_fd = (black_price(f, k, t, v + hv, true).unwrap()
            - 2.0 * black_price(f, k, t, v, true).unwrap()
            + black_price(f, k, t, v - hv, true).unwrap())
            / (hv * hv);
        approx_eq(vomma(f, k, t, v).unwrap(), vomma_fd, 5e-2);
    }

    #[test]
    fn fd_theta_matches_maturity_bump() {
        // Θ_calendar = −∂V/∂T
        let f = 100.0;
        let k = 100.0;
        let t = 1.0;
        let v = 0.2;
        let h = 1e-5;
        let d_v_d_t = (black_price(f, k, t + h, v, true).unwrap()
            - black_price(f, k, t - h, v, true).unwrap())
            / (2.0 * h);
        let analytic = theta(f, k, t, v).unwrap();
        approx_eq(analytic, -d_v_d_t, 1e-4);
    }

    #[test]
    fn bs_delta_matches_discounted_forward_delta() {
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let q = 0.02;
        let v = 0.2;
        let f = black_scholes_forward(s, t, r, q).unwrap();
        let expected = (-q * t).exp() * delta(f, k, t, v, true).unwrap();
        let got = black_scholes_delta(s, k, t, r, q, v, true).unwrap();
        approx_eq(got, expected, 1e-14);
    }

    #[test]
    fn bs_vega_fd() {
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.01;
        let q = 0.0;
        let v = 0.2;
        let h = 1e-5;
        let bump = (black_scholes_price(s, k, t, r, q, v + h, true).unwrap()
            - black_scholes_price(s, k, t, r, q, v - h, true).unwrap())
            / (2.0 * h);
        approx_eq(black_scholes_vega(s, k, t, r, q, v).unwrap(), bump, 1e-4);
    }

    #[test]
    fn bs_gamma_positive() {
        let g = black_scholes_gamma(100.0, 100.0, 1.0, 0.05, 0.02, 0.2).unwrap();
        assert!(g > 0.0);
    }

    #[test]
    fn rejects_bad_inputs() {
        assert!(delta(0.0, 100.0, 1.0, 0.2, true).is_err());
        assert!(black_scholes_delta(0.0, 100.0, 1.0, 0.0, 0.0, 0.2, true).is_err());
    }
}
