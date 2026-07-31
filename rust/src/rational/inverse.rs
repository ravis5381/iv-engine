//! Normalised Black inverse using Jäckel's transformed rational guess.
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

use super::initial_guess::{inverse_lower_map, inverse_upper_map, lower_map, upper_map};
use super::interpolation::{
    convex_control_at_left, convex_control_at_right, rational_cubic_interpolation,
};
use super::normalised_black::{
    normalised_black, normalised_black_call, normalised_intrinsic, normalised_vega,
};
use super::refinement::householder_factor;
use crate::constants::{DBL_EPSILON, DBL_MAX, DBL_MIN};
use crate::errors::IVError;

const LBR_ITERATIONS: u32 = 2;

/// Middle-branch Householder refinement.
///
/// Matches the C++ loop condition `iterations < N && fabs(ds) > DBL_EPSILON * s`.
/// The early exit is essential: when the rational guess lands exactly on a
/// bracket edge (e.g. `s == s_c` for `β == b_c`), a further forced iteration
/// would treat the edge as "outside the open bracket" and binary-nest away
/// from the already-correct root.
#[inline]
fn refine(beta: f64, x: f64, mut s: f64, mut left: f64, mut right: f64, iterations: u32) -> f64 {
    let mut ds = -DBL_MAX;
    let mut previous = 0.0;
    let mut reversals = 0;
    let mut n = 0u32;
    while n < iterations && ds.abs() > DBL_EPSILON * s {
        if ds * previous < 0.0 {
            reversals += 1;
        }
        if n > 0 && (reversals == 3 || !(s > left && s < right)) {
            s = 0.5 * (left + right);
            if right - left <= DBL_EPSILON * s {
                break;
            }
            reversals = 0;
            ds = 0.0;
        }
        previous = ds;
        let b = normalised_black(x, s, 1.0);
        let bp = normalised_vega(x, s);
        if b > beta && s < right {
            right = s;
        } else if b < beta && s > left {
            left = s;
        }
        if bp <= 0.0 || !bp.is_finite() {
            ds = 0.5 * (left + right) - s;
        } else {
            let newton = (beta - b) / bp;
            let halley = (x / s).powi(2) / s - s / 4.0;
            let hh3 = halley * halley - 3.0 * (x / (s * s)).powi(2) - 0.25;
            ds = newton * householder_factor(newton, halley, hh3);
        }
        ds = ds.max(-0.5 * s);
        s += ds;
        n += 1;
    }
    s
}

/// Internal LBR inverse for an already-normalised, out-of-the-money price.
#[inline]
pub(crate) fn unchecked_normalised_implied_volatility(
    beta: f64,
    mut x: f64,
    q: f64,
    iterations: u32,
) -> f64 {
    let mut beta = beta;
    if q * x > 0.0 {
        beta = (beta - normalised_intrinsic(x, q)).max(0.0).abs();
        if q > 0.0 {
            x = -x;
        }
    } else if q < 0.0 {
        x = -x;
    }
    if beta <= 0.0 || beta < 0.0 {
        return 0.0;
    }
    let b_max = (0.5 * x).exp();
    if beta >= b_max {
        return DBL_MAX;
    }
    let s_c = (2.0 * x).abs().sqrt();
    let b_c = normalised_black_call(x, s_c);
    let v_c = normalised_vega(x, s_c);
    let (s, left, right) = if beta < b_c {
        let s_l = s_c - b_c / v_c;
        let b_l = normalised_black_call(x, s_l);
        if beta < b_l {
            let (f_l, fp_l, fpp_l) = lower_map(x, s_l);
            let r = convex_control_at_right(0.0, b_l, 0.0, f_l, 1.0, fp_l, fpp_l, true);
            let mut f = rational_cubic_interpolation(beta, 0.0, b_l, 0.0, f_l, 1.0, fp_l, r);
            if !(f > 0.0) {
                let t = beta / b_l;
                f = (f_l * t + b_l * (1.0 - t)) * t;
            }
            let s = inverse_lower_map(x, f);
            return refine_lower(beta, x, s, DBL_MIN, s_l, iterations);
        }
        let v_l = normalised_vega(x, s_l);
        let r = convex_control_at_right(b_l, b_c, s_l, s_c, 1.0 / v_l, 1.0 / v_c, 0.0, false);
        (
            rational_cubic_interpolation(beta, b_l, b_c, s_l, s_c, 1.0 / v_l, 1.0 / v_c, r),
            s_l,
            s_c,
        )
    } else {
        let s_h = if v_c > DBL_MIN {
            s_c + (b_max - b_c) / v_c
        } else {
            s_c
        };
        let b_h = normalised_black_call(x, s_h);
        if beta <= b_h {
            let v_h = normalised_vega(x, s_h);
            let r = convex_control_at_left(b_c, b_h, s_c, s_h, 1.0 / v_c, 1.0 / v_h, 0.0, false);
            (
                rational_cubic_interpolation(beta, b_c, b_h, s_c, s_h, 1.0 / v_c, 1.0 / v_h, r),
                s_c,
                s_h,
            )
        } else {
            let (f_h, fp_h, fpp_h) = upper_map(x, s_h);
            let mut f = if fpp_h > -DBL_MAX.sqrt() && fpp_h < DBL_MAX.sqrt() {
                let r = convex_control_at_left(b_h, b_max, f_h, 0.0, fp_h, -0.5, fpp_h, true);
                rational_cubic_interpolation(beta, b_h, b_max, f_h, 0.0, fp_h, -0.5, r)
            } else {
                0.0
            };
            if f <= 0.0 {
                let h = b_max - b_h;
                let t = (beta - b_h) / h;
                f = (f_h * (1.0 - t) + 0.5 * h * t) * (1.0 - t);
            }
            let s = inverse_upper_map(f);
            if beta > 0.5 * b_max {
                return refine_upper(beta, x, s, s_h, DBL_MAX, b_max, iterations);
            }
            (s, s_h, DBL_MAX)
        }
    };
    refine(beta, x, s, left, right, iterations)
}

#[inline]
fn refine_lower(
    beta: f64,
    x: f64,
    mut s: f64,
    mut left: f64,
    mut right: f64,
    iterations: u32,
) -> f64 {
    let mut ds = -DBL_MAX;
    let mut previous = 0.0;
    let mut reversals = 0;
    let mut n = 0u32;
    while n < iterations && ds.abs() > DBL_EPSILON * s {
        if ds * previous < 0.0 {
            reversals += 1;
        }
        if n > 0 && (reversals == 3 || !(s > left && s < right)) {
            s = 0.5 * (left + right);
            if right - left <= DBL_EPSILON * s {
                break;
            }
            reversals = 0;
            ds = 0.0;
        }
        previous = ds;
        let b = normalised_black_call(x, s);
        let bp = normalised_vega(x, s);
        if b > beta && s < right {
            right = s;
        } else if b < beta && s > left {
            left = s;
        }
        if b <= 0.0 || bp <= 0.0 {
            ds = 0.5 * (left + right) - s;
        } else {
            let lb = b.ln();
            let lbeta = beta.ln();
            let bpob = bp / b;
            let h = x / s;
            let bh = h * h / s - s / 4.0;
            let newton = (lbeta - lb) * lb / lbeta / bpob;
            let halley = bh - bpob * (1.0 + 2.0 / lb);
            let hh3 = bh * bh - 3.0 * (h / s).powi(2) - 0.25
                + 2.0 * bpob.powi(2) * (1.0 + 3.0 / lb * (1.0 + 1.0 / lb))
                - 3.0 * bh * bpob * (1.0 + 2.0 / lb);
            ds = newton * householder_factor(newton, halley, hh3);
        }
        ds = ds.max(-0.5 * s);
        s += ds;
        n += 1;
    }
    s
}

#[inline]
fn refine_upper(
    beta: f64,
    x: f64,
    mut s: f64,
    mut left: f64,
    mut right: f64,
    b_max: f64,
    iterations: u32,
) -> f64 {
    let mut ds = -DBL_MAX;
    let mut previous = 0.0;
    let mut reversals = 0;
    let mut n = 0u32;
    while n < iterations && ds.abs() > DBL_EPSILON * s {
        if ds * previous < 0.0 {
            reversals += 1;
        }
        if n > 0 && (reversals == 3 || !(s > left && s < right)) {
            s = 0.5 * (left + right);
            if right - left <= DBL_EPSILON * s {
                break;
            }
            reversals = 0;
            ds = 0.0;
        }
        previous = ds;
        let b = normalised_black_call(x, s);
        let bp = normalised_vega(x, s);
        if b > beta && s < right {
            right = s;
        } else if b < beta && s > left {
            left = s;
        }
        if b >= b_max || bp <= DBL_MIN {
            ds = 0.5 * (left + right) - s;
        } else {
            let mb = b_max - b;
            let g = ((b_max - beta) / mb).ln();
            let gp = bp / mb;
            let bh = (x / s).powi(2) / s - s / 4.0;
            let hh = bh * bh - 3.0 * (x / (s * s)).powi(2) - 0.25;
            let newton = -g / gp;
            let halley = bh + gp;
            let hh3 = hh + gp * (2.0 * gp + 3.0 * bh);
            ds = newton * householder_factor(newton, halley, hh3);
        }
        ds = ds.max(-0.5 * s);
        s += ds;
        n += 1;
    }
    s
}

/// Recovers Black implied volatility from an undiscounted option price.
pub fn implied_volatility(
    price: f64,
    forward: f64,
    strike: f64,
    maturity: f64,
    is_call: bool,
) -> Result<f64, IVError> {
    if !(price.is_finite() && forward.is_finite() && strike.is_finite() && maturity.is_finite()) {
        return Err(IVError::invalid("non-finite implied-volatility input"));
    }
    if forward <= 0.0 {
        return Err(IVError::NegativeForward { forward });
    }
    if strike <= 0.0 {
        return Err(IVError::NegativeStrike { strike });
    }
    if maturity <= 0.0 {
        return Err(IVError::NegativeTime { maturity });
    }
    let q = if is_call { 1.0 } else { -1.0 };
    let intrinsic = if is_call {
        (forward - strike).max(0.0)
    } else {
        (strike - forward).max(0.0)
    };
    if price < intrinsic {
        return Err(IVError::PriceBelowIntrinsic { price, intrinsic });
    }
    let maximum = if is_call { forward } else { strike };
    if price >= maximum {
        return Err(IVError::PriceAboveMaximum { price, maximum });
    }
    let s = unchecked_normalised_implied_volatility(
        price / (forward.sqrt() * strike.sqrt()),
        forward.ln() - strike.ln(),
        q,
        LBR_ITERATIONS,
    );
    if !s.is_finite() || s == DBL_MAX {
        return Err(IVError::NoConvergence {
            iterations: LBR_ITERATIONS,
            residual: None,
        });
    }
    Ok((s / maturity.sqrt()).max(0.0))
}

/// Recovers total volatility from a normalised Black price.
pub fn normalised_implied_volatility(beta: f64, x: f64, is_call: bool) -> Result<f64, IVError> {
    if !(beta.is_finite() && x.is_finite()) {
        return Err(IVError::invalid("non-finite normalised input"));
    }
    let q = if is_call { 1.0 } else { -1.0 };
    let intrinsic = normalised_intrinsic(x, q);
    if beta < intrinsic {
        return Err(IVError::PriceBelowIntrinsic {
            price: beta,
            intrinsic,
        });
    }
    let maximum = if is_call {
        (0.5 * x).exp()
    } else {
        (-0.5 * x).exp()
    };
    if beta >= maximum {
        return Err(IVError::PriceAboveMaximum {
            price: beta,
            maximum,
        });
    }
    Ok(unchecked_normalised_implied_volatility(
        beta,
        x,
        q,
        LBR_ITERATIONS,
    ))
}
