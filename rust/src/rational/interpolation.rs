//! Shape-preserving rational cubic interpolation used by Let's Be Rational.
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::constants::{DBL_EPSILON, DBL_MAX, DBL_MIN};

const MINIMUM_CONTROL: f64 = -0.999_999_985_098_838_8;
const MAXIMUM_CONTROL: f64 = 2.0 / (DBL_EPSILON * DBL_EPSILON);

#[inline]
fn is_zero(x: f64) -> bool {
    x.abs() < DBL_MIN
}

#[inline]
pub(crate) fn rational_cubic_interpolation(
    x: f64,
    x_l: f64,
    x_r: f64,
    y_l: f64,
    y_r: f64,
    d_l: f64,
    d_r: f64,
    r: f64,
) -> f64 {
    let h = x_r - x_l;
    if h.abs() <= 0.0 {
        return 0.5 * (y_l + y_r);
    }
    let t = (x - x_l) / h;
    if !(r >= MAXIMUM_CONTROL) {
        let omt = 1.0 - t;
        let t2 = t * t;
        let omt2 = omt * omt;
        return (y_r * t2 * t
            + (r * y_r - h * d_r) * t2 * omt
            + (r * y_l + h * d_l) * t * omt2
            + y_l * omt2 * omt)
            / (1.0 + (r - 3.0) * t * omt);
    }
    y_r * t + y_l * (1.0 - t)
}

#[inline]
fn control_at_left(x_l: f64, x_r: f64, y_l: f64, y_r: f64, d_l: f64, d_r: f64, dd_l: f64) -> f64 {
    let h = x_r - x_l;
    let numerator = 0.5 * h * dd_l + (d_r - d_l);
    if is_zero(numerator) {
        return 0.0;
    }
    let denominator = (y_r - y_l) / h - d_l;
    if is_zero(denominator) {
        return if numerator > 0.0 {
            MAXIMUM_CONTROL
        } else {
            MINIMUM_CONTROL
        };
    }
    numerator / denominator
}

#[inline]
fn control_at_right(x_l: f64, x_r: f64, y_l: f64, y_r: f64, d_l: f64, d_r: f64, dd_r: f64) -> f64 {
    let h = x_r - x_l;
    let numerator = 0.5 * h * dd_r + (d_r - d_l);
    if is_zero(numerator) {
        return 0.0;
    }
    let denominator = d_r - (y_r - y_l) / h;
    if is_zero(denominator) {
        return if numerator > 0.0 {
            MAXIMUM_CONTROL
        } else {
            MINIMUM_CONTROL
        };
    }
    numerator / denominator
}

#[inline]
fn minimum_control(d_l: f64, d_r: f64, s: f64, prefer_shape: bool) -> f64 {
    let monotonic = d_l * s >= 0.0 && d_r * s >= 0.0;
    let convex = d_l <= s && s <= d_r;
    let concave = d_l >= s && s >= d_r;
    if !monotonic && !convex && !concave {
        return MINIMUM_CONTROL;
    }
    let dr_dl = d_r - d_l;
    let dr_s = d_r - s;
    let s_dl = s - d_l;
    let mut r1 = -DBL_MAX;
    let mut r2 = r1;
    if monotonic {
        if !is_zero(s) {
            r1 = (d_r + d_l) / s;
        } else if prefer_shape {
            r1 = MAXIMUM_CONTROL;
        }
    }
    if convex || concave {
        if !(is_zero(s_dl) || is_zero(dr_s)) {
            r2 = (dr_dl / dr_s).abs().max((dr_dl / s_dl).abs());
        } else if prefer_shape {
            r2 = MAXIMUM_CONTROL;
        }
    } else if monotonic && prefer_shape {
        r2 = MAXIMUM_CONTROL;
    }
    MINIMUM_CONTROL.max(r1.max(r2))
}

#[inline]
pub(crate) fn convex_control_at_left(
    x_l: f64,
    x_r: f64,
    y_l: f64,
    y_r: f64,
    d_l: f64,
    d_r: f64,
    dd_l: f64,
    prefer_shape: bool,
) -> f64 {
    control_at_left(x_l, x_r, y_l, y_r, d_l, d_r, dd_l).max(minimum_control(
        d_l,
        d_r,
        (y_r - y_l) / (x_r - x_l),
        prefer_shape,
    ))
}

#[inline]
pub(crate) fn convex_control_at_right(
    x_l: f64,
    x_r: f64,
    y_l: f64,
    y_r: f64,
    d_l: f64,
    d_r: f64,
    dd_r: f64,
    prefer_shape: bool,
) -> f64 {
    control_at_right(x_l, x_r, y_l, y_r, d_l, d_r, dd_r).max(minimum_control(
        d_l,
        d_r,
        (y_r - y_l) / (x_r - x_l),
        prefer_shape,
    ))
}
