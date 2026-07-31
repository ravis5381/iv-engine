//! Normalised Black pricing kernel from Let's Be Rational.
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

use super::erfcx::erfcx_cody;
use super::inv_norm::lbr_norm_cdf;
use crate::constants::{DBL_EPSILON, DBL_MIN, ONE_OVER_SQRT_TWO_PI, SQRT_PI_OVER_TWO};
use core::f64::consts::FRAC_1_SQRT_2;

const ASYMPTOTIC_THRESHOLD: f64 = -10.0;

#[inline]
fn fourth_root_dbl_epsilon() -> f64 {
    DBL_EPSILON.sqrt().sqrt()
}

#[inline]
fn small_t_threshold() -> f64 {
    let sqrt_dbl_epsilon = DBL_EPSILON.sqrt();
    let fourth_root_dbl_epsilon = sqrt_dbl_epsilon.sqrt();
    let eighth_root_dbl_epsilon = fourth_root_dbl_epsilon.sqrt();
    let sixteenth_root_dbl_epsilon = eighth_root_dbl_epsilon.sqrt();
    2.0 * sixteenth_root_dbl_epsilon
}

#[inline]
pub(crate) fn normalised_intrinsic(x: f64, q: f64) -> f64 {
    if q * x <= 0.0 {
        return 0.0;
    }
    let x2 = x * x;
    if x2 < 98.0 * fourth_root_dbl_epsilon() {
        return ((if q < 0.0 { -1.0 } else { 1.0 })
            * x
            * (1.0
                + x2 * (1.0 / 24.0
                    + x2 * (1.0 / 1920.0 + x2 * (1.0 / 322560.0 + x2 / 92897280.0)))))
            .max(0.0)
            .abs();
    }
    let b_max = (0.5 * x).exp();
    ((if q < 0.0 { -1.0 } else { 1.0 }) * (b_max - 1.0 / b_max))
        .max(0.0)
        .abs()
}

#[inline]
fn erfcx_black(h: f64, t: f64) -> f64 {
    let b = 0.5
        * (-0.5 * (h * h + t * t)).exp()
        * (erfcx_cody(-FRAC_1_SQRT_2 * (h + t)) - erfcx_cody(-FRAC_1_SQRT_2 * (h - t)));
    b.max(0.0).abs()
}

#[inline]
fn small_t_black(h: f64, t: f64) -> f64 {
    let a = 1.0 + h * SQRT_PI_OVER_TWO * erfcx_cody(-FRAC_1_SQRT_2 * h);
    let w = t * t;
    let h2 = h * h;
    let p7 = (-89055.0
        + 135135.0 * a
        + h2 * (-82845.0
            + 270270.0 * a
            + h2 * (-20370.0
                + 135135.0 * a
                + h2 * (-1926.0
                    + 25740.0 * a
                    + h2 * (-75.0 + 2145.0 * a + h2 * (-1.0 + 78.0 * a + a * h2))))))
        * w
        / 6227020800.0;
    let p6 = (-6555.0
        + 10395.0 * a
        + h2 * (-4680.0
            + 17325.0 * a
            + h2 * (-840.0
                + 6930.0 * a
                + h2 * (-52.0 + 990.0 * a + h2 * (-1.0 + 55.0 * a + a * h2)))))
        / 39916800.0
        + p7;
    let p5 = (-561.0
        + 945.0 * a
        + h2 * (-285.0 + 1260.0 * a + h2 * (-33.0 + 378.0 * a + h2 * (-1.0 + 36.0 * a + a * h2))))
        / 362880.0
        + w * p6;
    let p4 = (-57.0 + 105.0 * a + h2 * (-18.0 + 105.0 * a + h2 * (-1.0 + 21.0 * a + a * h2)))
        / 5040.0
        + w * p5;
    let p3 = (-7.0 + 15.0 * a + h2 * (-1.0 + 10.0 * a + a * h2)) / 120.0 + w * p4;
    let expansion = 2.0 * t * (a + w * ((-1.0 + 3.0 * a + a * h2) / 6.0 + w * p3));
    let b = ONE_OVER_SQRT_TWO_PI * (-0.5 * (h * h + t * t)).exp() * expansion;
    b.max(0.0).abs()
}

#[inline]
fn asymptotic_black(h: f64, t: f64) -> f64 {
    let e = (t / h) * (t / h);
    let r = (h + t) * (h - t);
    let q = (h / r) * (h / r);
    #[inline]
    fn polynomial(e: f64, coefficients: &[f64]) -> f64 {
        coefficients
            .iter()
            .rev()
            .fold(0.0, |value, &coefficient| value * e + coefficient)
    }
    let polynomials: [&[f64]; 17] = [
        &[-6.0, -2.0],
        &[10.0, 20.0, 2.0],
        &[-14.0, -70.0, -42.0, -2.0],
        &[18.0, 168.0, 252.0, 72.0, 2.0],
        &[-22.0, -330.0, -924.0, -660.0, -110.0, -2.0],
        &[26.0, 572.0, 2574.0, 3432.0, 1430.0, 156.0, 2.0],
        &[
            -30.0, -910.0, -6006.0, -12870.0, -10010.0, -2730.0, -210.0, -2.0,
        ],
        &[
            34.0, 1360.0, 12376.0, 38896.0, 48620.0, 24752.0, 4760.0, 272.0, 2.0,
        ],
        &[
            -38.0, -1938.0, -23256.0, -100776.0, -184756.0, -151164.0, -54264.0, -7752.0, -342.0,
            -2.0,
        ],
        &[
            42.0, 2660.0, 40698.0, 232560.0, 587860.0, 705432.0, 406980.0, 108528.0, 11970.0,
            420.0, 2.0,
        ],
        &[
            -46.0, -3542.0, -67298.0, -490314.0, -1634380.0, -2704156.0, -2288132.0, -980628.0,
            -201894.0, -17710.0, -506.0, -2.0,
        ],
        &[
            50.0, 4600.0, 106260.0, 961400.0, 4085950.0, 8914800.0, 10400600.0, 6537520.0,
            2163150.0, 354200.0, 25300.0, 600.0, 2.0,
        ],
        &[
            -54.0,
            -5850.0,
            -161460.0,
            -1776060.0,
            -9373650.0,
            -26075790.0,
            -40116600.0,
            -34767720.0,
            -16872570.0,
            -4440150.0,
            -592020.0,
            -35100.0,
            -702.0,
            -2.0,
        ],
        &[
            58.0,
            7308.0,
            237510.0,
            3121560.0,
            20030010.0,
            69194580.0,
            135727830.0,
            155117520.0,
            103791870.0,
            40060020.0,
            8584290.0,
            950040.0,
            47502.0,
            812.0,
            2.0,
        ],
        &[
            -62.0,
            -8990.0,
            -339822.0,
            -5259150.0,
            -40320150.0,
            -169344630.0,
            -412506150.0,
            -601080390.0,
            -530365050.0,
            -282241050.0,
            -88704330.0,
            -15777450.0,
            -1472560.0,
            -62930.0,
            -930.0,
            -2.0,
        ],
        &[
            66.0,
            10912.0,
            474672.0,
            8544096.0,
            77134200.0,
            387073440.0,
            1146332880.0,
            2074316640.0,
            2333606220.0,
            1637618420.0,
            709634640.0,
            185122080.0,
            27768320.0,
            2215136.0,
            81840.0,
            1056.0,
            2.0,
        ],
        &[
            -70.0,
            -13090.0,
            -649264.0,
            -13449040.0,
            -141214920.0,
            -834451800.0,
            -2952675600.0,
            -6495886320.0,
            -9075135300.0,
            -8119857900.0,
            -4639918200.0,
            -1668903600.0,
            -367158792.0,
            -47071640.0,
            -3246320.0,
            -104720.0,
            -1190.0,
            -2.0,
        ],
    ];
    let mut nested = polynomial(e, polynomials[16]);
    for index in (1..16).rev() {
        nested = polynomial(e, polynomials[index]) + (2 * index + 3) as f64 * q * nested;
    }
    let sum = 2.0 + q * (polynomial(e, polynomials[0]) + 3.0 * q * nested);
    let b = ONE_OVER_SQRT_TWO_PI * (-0.5 * (h * h + t * t)).exp() * (t / r) * sum;
    b.max(0.0).abs()
}

#[inline]
fn norm_cdf_black(x: f64, s: f64) -> f64 {
    let h = x / s;
    let t = 0.5 * s;
    let b_max = (0.5 * x).exp();
    (lbr_norm_cdf(h + t) * b_max - lbr_norm_cdf(h - t) / b_max)
        .max(0.0)
        .abs()
}

/// LBR normalised call Black value `b(x, s)`.
#[inline]
pub(crate) fn normalised_black_call(x: f64, s: f64) -> f64 {
    if x > 0.0 {
        return normalised_intrinsic(x, 1.0) + normalised_black_call(-x, s);
    }
    let ax = x.abs();
    if s <= ax * 0.0 {
        return normalised_intrinsic(x, 1.0);
    }
    if x < s * ASYMPTOTIC_THRESHOLD
        && 0.5 * s * s + x < s * (small_t_threshold() + ASYMPTOTIC_THRESHOLD)
    {
        return asymptotic_black(x / s, 0.5 * s);
    }
    if 0.5 * s < small_t_threshold() {
        return small_t_black(x / s, 0.5 * s);
    }
    if x + 0.5 * s * s > s * 0.85 {
        return norm_cdf_black(x, s);
    }
    erfcx_black(x / s, 0.5 * s)
}

/// Derivative of normalised Black with respect to total volatility.
#[inline]
pub(crate) fn normalised_vega(x: f64, s: f64) -> f64 {
    let ax = x.abs();
    if ax <= 0.0 {
        ONE_OVER_SQRT_TWO_PI * (-0.125 * s * s).exp()
    } else if s <= 0.0 || s <= ax * DBL_MIN.sqrt() {
        0.0
    } else {
        ONE_OVER_SQRT_TWO_PI * (-0.5 * ((x / s).powi(2) + (0.5 * s).powi(2))).exp()
    }
}

/// LBR normalised Black value for `q = ±1`.
#[inline]
pub(crate) fn normalised_black(x: f64, s: f64, q: f64) -> f64 {
    normalised_black_call(if q < 0.0 { -x } else { x }, s)
}
