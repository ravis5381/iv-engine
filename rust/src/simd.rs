//! Optional SIMD accelerations for data-parallel kernels.
//!
//! # Design
//!
//! Let's Be Rational IV is intentionally **not** SIMD-vectorised: the
//! algorithm is heavily branched (region maps, Householder, asymptotic
//! expansions). Parallel batch IV belongs in [`crate::parallel`] (Rayon).
//!
//! What *does* vectorise cleanly is the Standard Normal density
//! `φ(x) = (1/√(2π)) exp(−x²/2)`. Enable the Cargo feature `simd` to evaluate
//! it with portable [`wide::f64x4`] lanes:
//!
//! ```bash
//! cargo test -p iv-engine --features simd
//! ```
//!
//! Without the feature, [`norm_pdf_slice`] falls back to a scalar loop over
//! [`crate::norm_pdf`] so call sites compile either way.
//!
//! Batch CDF / survival APIs ([`norm_cdf_slice`], [`norm_cdf_c_slice`]) always
//! call the scalar [`libm`](https://docs.rs/libm)-backed kernels lane-wise so
//! they stay **bit-identical** to [`crate::norm_cdf`] / [`crate::norm_cdf_c`]
//! (there is no portable SIMD `erfc` that matches libm).
//!
//! # Accuracy
//!
//! SIMD PDF results agree with the scalar PDF to roughly machine precision on
//! ordinary inputs; transcendental `exp` reductions may differ by a few ulps.
//! Tests enforce a tight absolute/relative tolerance, not bit equality.

use crate::errors::IVError;
#[cfg(not(feature = "simd"))]
use crate::normal::norm_pdf;
use crate::normal::{norm_cdf, norm_cdf_c};

/// SIMD lane count used by the `wide` PDF kernel (`f64x4`).
pub const SIMD_LANES: usize = 4;

/// Whether this build was compiled with the `simd` Cargo feature.
#[must_use]
pub const fn simd_enabled() -> bool {
    cfg!(feature = "simd")
}

#[inline]
const fn require_equal_len(xs: usize, out: usize) -> Result<(), IVError> {
    if xs != out {
        return Err(IVError::invalid(
            "input and output slices must have equal length",
        ));
    }
    Ok(())
}

/// Batch Standard Normal PDF into `out`.
///
/// With feature `simd`, interior chunks of [`SIMD_LANES`] are evaluated with
/// portable SIMD; the scalar remainder (and the entire buffer without the
/// feature) uses [`norm_pdf`].
///
/// # Errors
///
/// [`IVError::InvalidInput`] if `xs.len() != out.len()`.
pub fn norm_pdf_slice(xs: &[f64], out: &mut [f64]) -> Result<(), IVError> {
    require_equal_len(xs.len(), out.len())?;
    #[cfg(feature = "simd")]
    {
        simd_pdf::norm_pdf_slice_simd(xs, out);
    }
    #[cfg(not(feature = "simd"))]
    {
        for (x, slot) in xs.iter().zip(out.iter_mut()) {
            *slot = norm_pdf(*x);
        }
    }
    Ok(())
}

/// Batch Standard Normal CDF into `out` (scalar-accurate, lane-wise).
///
/// # Errors
///
/// [`IVError::InvalidInput`] if `xs.len() != out.len()`.
pub fn norm_cdf_slice(xs: &[f64], out: &mut [f64]) -> Result<(), IVError> {
    require_equal_len(xs.len(), out.len())?;
    for (x, slot) in xs.iter().zip(out.iter_mut()) {
        *slot = norm_cdf(*x);
    }
    Ok(())
}

/// Batch Standard Normal survival `1 − Φ(x)` into `out` (scalar-accurate).
///
/// # Errors
///
/// [`IVError::InvalidInput`] if `xs.len() != out.len()`.
pub fn norm_cdf_c_slice(xs: &[f64], out: &mut [f64]) -> Result<(), IVError> {
    require_equal_len(xs.len(), out.len())?;
    for (x, slot) in xs.iter().zip(out.iter_mut()) {
        *slot = norm_cdf_c(*x);
    }
    Ok(())
}

#[cfg(feature = "simd")]
mod simd_pdf {
    use super::SIMD_LANES;
    use crate::constants::ONE_OVER_SQRT_TWO_PI;
    use crate::normal::norm_pdf;
    use wide::f64x4;

    #[inline]
    pub(super) fn norm_pdf_slice_simd(xs: &[f64], out: &mut [f64]) {
        let n = xs.len();
        let mut i = 0usize;
        let inv_sqrt = f64x4::splat(ONE_OVER_SQRT_TWO_PI);
        let neg_half = f64x4::splat(-0.5);
        let zero = f64x4::ZERO;
        let nans = f64x4::splat(f64::NAN);

        while i + SIMD_LANES <= n {
            let x = f64x4::from([xs[i], xs[i + 1], xs[i + 2], xs[i + 3]]);
            let pdf = inv_sqrt * (neg_half * x * x).exp();
            // Match scalar contract: ±∞ → 0, NaN → NaN, finite → φ(x).
            let finite = x.is_finite().blend(pdf, zero);
            let result = x.is_nan().blend(nans, finite);
            let arr = result.to_array();
            out[i] = arr[0];
            out[i + 1] = arr[1];
            out[i + 2] = arr[2];
            out[i + 3] = arr[3];
            i += SIMD_LANES;
        }
        while i < n {
            out[i] = norm_pdf(xs[i]);
            i += 1;
        }
    }
}
