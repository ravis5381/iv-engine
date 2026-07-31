//! Normal-distribution routines used internally by Let's Be Rational.
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::constants::{DBL_EPSILON, DBL_MAX};
use crate::normal::norm_pdf;
use core::f64::consts::FRAC_1_SQRT_2;

/// LBR's lower-tail normal CDF, including its large-negative asymptotic path.
#[inline]
pub(crate) fn lbr_norm_cdf(z: f64) -> f64 {
    if z <= -10.0 {
        let mut sum = 1.0;
        if z >= -1.0 / DBL_EPSILON.sqrt() {
            let zsqr = z * z;
            let mut i = 1.0;
            let mut g = 1.0;
            let mut a = DBL_MAX;
            loop {
                let lasta = a;
                let x = (4.0 * i - 3.0) / zsqr;
                let y = x * ((4.0 * i - 1.0) / zsqr);
                a = g * (x - y);
                sum -= a;
                g *= y;
                i += 1.0;
                a = a.abs();
                if !(lasta > a && a >= (sum * DBL_EPSILON).abs()) {
                    break;
                }
            }
        }
        -norm_pdf(z) * sum / z
    } else {
        // This is the same `erfc_cody` call used by the reference. `libm`
        // provides a portable IEEE-754 complementary-error-function kernel.
        0.5 * libm::erfc(-z * FRAC_1_SQRT_2)
    }
}

/// AS241 inverse lower-tail normal CDF approximation from the LBR reference.
#[inline]
pub(crate) fn inverse_norm_cdf(u: f64) -> f64 {
    const A: [f64; 8] = [
        3.3871328727963666080E0,
        1.3314166789178437745E2,
        1.9715909503065514427E3,
        1.3731693765509461125E4,
        4.5921953931549871457E4,
        6.7265770927008700853E4,
        3.3430575583588128105E4,
        2.5090809287301226727E3,
    ];
    const B: [f64; 7] = [
        4.2313330701600911252E1,
        6.8718700749205790830E2,
        5.3941960214247511077E3,
        2.1213794301586595867E4,
        3.9307895800092710610E4,
        2.8729085735721942674E4,
        5.2264952788528545610E3,
    ];
    const C: [f64; 8] = [
        1.42343711074968357734E0,
        4.63033784615654529590E0,
        5.76949722146069140550E0,
        3.64784832476320460504E0,
        1.27045825245236838258E0,
        2.41780725177450611770E-1,
        2.27238449892691845833E-2,
        7.74545014278341407640E-4,
    ];
    const D: [f64; 7] = [
        2.05319162663775882187E0,
        1.67638483018380384940E0,
        6.89767334985100004550E-1,
        1.48103976427480074590E-1,
        1.51986665636164571966E-2,
        5.47593808499534494600E-4,
        1.05075007164441684324E-9,
    ];
    const E: [f64; 8] = [
        6.65790464350110377720E0,
        5.46378491116411436990E0,
        1.78482653991729133580E0,
        2.96560571828504891230E-1,
        2.65321895265761230930E-2,
        1.24266094738807843860E-3,
        2.71155556874348757815E-5,
        2.01033439929228813265E-7,
    ];
    const F: [f64; 7] = [
        5.99832206555887937690E-1,
        1.36929880922735805310E-1,
        1.48753612908506148525E-2,
        7.86869131145613259100E-4,
        1.84631831751005468180E-5,
        1.42151175831644588870E-7,
        2.04426310338993978564E-15,
    ];
    if u <= 0.0 {
        return u.ln();
    }
    if u >= 1.0 {
        return (1.0 - u).ln();
    }
    let q = u - 0.5;
    if q.abs() <= 0.425 {
        let r = 0.180625 - q * q;
        let num =
            ((((((A[7] * r + A[6]) * r + A[5]) * r + A[4]) * r + A[3]) * r + A[2]) * r + A[1]) * r
                + A[0];
        let den =
            ((((((B[6] * r + B[5]) * r + B[4]) * r + B[3]) * r + B[2]) * r + B[1]) * r + B[0]) * r
                + 1.0;
        return q * num / den;
    }
    let mut r = if q < 0.0 { u } else { 1.0 - u };
    r = (-r.ln()).sqrt();
    let ret = if r < 5.0 {
        let r = r - 1.6;
        (((((((C[7] * r + C[6]) * r + C[5]) * r + C[4]) * r + C[3]) * r + C[2]) * r + C[1]) * r
            + C[0])
            / (((((((D[6] * r + D[5]) * r + D[4]) * r + D[3]) * r + D[2]) * r + D[1]) * r + D[0])
                * r
                + 1.0)
    } else {
        let r = r - 5.0;
        (((((((E[7] * r + E[6]) * r + E[5]) * r + E[4]) * r + E[3]) * r + E[2]) * r + E[1]) * r
            + E[0])
            / (((((((F[6] * r + F[5]) * r + F[4]) * r + F[3]) * r + F[2]) * r + F[1]) * r + F[0])
                * r
                + 1.0)
    };
    if q < 0.0 {
        -ret
    } else {
        ret
    }
}
