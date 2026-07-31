//! LBR transformed lower and upper maps used for initial guesses.
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

use super::inv_norm::{inverse_norm_cdf, lbr_norm_cdf};
use crate::constants::{SQRT_PI_OVER_TWO, TWO_PI};
use crate::normal::norm_pdf;

const SQRT_ONE_OVER_THREE: f64 = 0.577350269189625764509148780501957455647601751270;
const SQRT_THREE: f64 = 1.732050807568877293527446341505872366942805253810;
const TWO_PI_OVER_SQRT_TWENTY_SEVEN: f64 = 1.209199576156145233729385505094770488189377498728;

#[inline]
pub(crate) fn lower_map(x: f64, s: f64) -> (f64, f64, f64) {
    let ax = x.abs();
    let z = SQRT_ONE_OVER_THREE * ax / s;
    let y = z * z;
    let s2 = s * s;
    let phi = lbr_norm_cdf(-z);
    let fpp = core::f64::consts::PI / 6.0 * y / (s2 * s)
        * phi
        * (8.0 * SQRT_THREE * s * ax + (3.0 * s2 * (s2 - 8.0) - 8.0 * x * x) * phi / norm_pdf(z))
        * (2.0 * y + 0.25 * s2).exp();
    if s.abs() < f64::MIN_POSITIVE {
        return (0.0, 1.0, fpp);
    }
    let fp = TWO_PI * y * phi * phi * (y + 0.125 * s2).exp();
    let f = if x.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        TWO_PI_OVER_SQRT_TWENTY_SEVEN * ax * phi * phi * phi
    };
    (f, fp, fpp)
}

#[inline]
pub(crate) fn inverse_lower_map(x: f64, f: f64) -> f64 {
    if f.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        (x / (SQRT_THREE
            * inverse_norm_cdf((f / (TWO_PI_OVER_SQRT_TWENTY_SEVEN * x.abs())).powf(1.0 / 3.0))))
        .abs()
    }
}

#[inline]
pub(crate) fn upper_map(x: f64, s: f64) -> (f64, f64, f64) {
    let f = lbr_norm_cdf(-0.5 * s);
    if x.abs() < f64::MIN_POSITIVE {
        return (f, -0.5, 0.0);
    }
    let w = (x / s) * (x / s);
    (
        f,
        -0.5 * (0.5 * w).exp(),
        SQRT_PI_OVER_TWO * (w + 0.125 * s * s).exp() * w / s,
    )
}

#[inline]
pub(crate) fn inverse_upper_map(f: f64) -> f64 {
    -2.0 * inverse_norm_cdf(f)
}
