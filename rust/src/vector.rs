//! Vectorised batch APIs over contiguous slices.
//!
//! # Design
//!
//! Scalar kernels remain the source of truth. These entry points only
//! orchestrate length checks, optional length-1 broadcasting, and
//! element-wise dispatch — they never reimplement pricing or IV logic.
//!
//! ## Lengths and broadcasting
//!
//! Every input slice must have length `1` (broadcast) or `n`, where `n` is
//! the common batch length. The output slice must have length `n`. Empty
//! outputs (`n = 0`) are accepted and return immediately.
//!
//! ## Per-element failures
//!
//! Length / broadcast mismatches return [`IVError::InvalidInput`]. Domain
//! failures on individual elements write `f64::NAN` into `out` and
//! continue; the function still returns `Ok(())`. This matches the `NumPy`
//! convention that batch consumers expect and keeps the hot path
//! allocation-free.
//!
//! Callers that need structured per-element errors should invoke the
//! scalar APIs directly (or inspect `out.is_nan()`).
//!
//! ## Parallelism / SIMD
//!
//! These loops are sequential. Rayon ([`crate::parallel`]) parallelises the
//! same scalar kernels across threads. Portable SIMD ([`crate::simd`])
//! accelerates Normal PDF batches; LBR IV stays scalar by design.

use crate::black::black_price;
use crate::black_scholes::black_scholes_price;
use crate::errors::IVError;
use crate::greeks::{delta, gamma, vanna, vega, vomma};
use crate::rational::{implied_volatility, normalised_implied_volatility};

/// Resolve the common batch length from input slice lengths.
///
/// Each length must be `1` (broadcast) or equal to the unique non-1 length.
/// A batch is empty (`Ok(0)`) when every length is `0` or `1` and at least
/// one length is `0`.
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
#[inline]
pub fn batch_len(lengths: &[usize]) -> Result<usize, IVError> {
    resolve_batch_len(lengths)
}

/// Resolve the common batch length from input slice lengths.
///
/// Each length must be `1` (broadcast) or equal to the unique non-1 length.
/// A batch is empty (`Ok(0)`) when every length is `0` or `1` and at least
/// one length is `0`.
#[inline]
pub(crate) fn resolve_batch_len(lengths: &[usize]) -> Result<usize, IVError> {
    if lengths.contains(&0) {
        if lengths.iter().any(|&len| len > 1) {
            return Err(IVError::invalid(
                "empty input slice is only valid for an empty batch",
            ));
        }
        return Ok(0);
    }

    let mut n = None;
    for &len in lengths {
        if len == 1 {
            continue;
        }
        match n {
            None => n = Some(len),
            Some(existing) if existing == len => {}
            Some(_) => {
                return Err(IVError::invalid(
                    "slice length mismatch: inputs must be length 1 or n",
                ));
            }
        }
    }
    Ok(n.unwrap_or(1))
}

#[inline]
pub(crate) const fn require_out_len(out_len: usize, n: usize) -> Result<(), IVError> {
    if out_len != n {
        return Err(IVError::invalid(
            "output slice length must equal the resolved batch length",
        ));
    }
    Ok(())
}

#[inline]
pub(crate) fn at(slice: &[f64], i: usize) -> f64 {
    // SAFETY of indexing: callers only invoke with i < n and slice.len() is
    // either 1 or n (enforced by resolve_batch_len + require_out_len).
    if slice.len() == 1 {
        slice[0]
    } else {
        slice[i]
    }
}

#[inline]
pub(crate) fn write_or_nan(out: &mut f64, result: Result<f64, IVError>) {
    *out = result.unwrap_or(f64::NAN);
}

/// Batch undiscounted Black-76 prices.
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn black_price_slice(
    forwards: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    vols: &[f64],
    is_call: bool,
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            black_price(
                at(forwards, i),
                at(strikes, i),
                at(maturities, i),
                at(vols, i),
                is_call,
            ),
        );
    }
    Ok(())
}

/// Batch discounted Black–Scholes–Merton prices.
///
/// `rates` and `dividends` participate in the same length-1 broadcast rules.
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
#[allow(clippy::too_many_arguments)]
pub fn black_scholes_price_slice(
    spots: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    rates: &[f64],
    dividends: &[f64],
    vols: &[f64],
    is_call: bool,
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[
        spots.len(),
        strikes.len(),
        maturities.len(),
        rates.len(),
        dividends.len(),
        vols.len(),
    ])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            black_scholes_price(
                at(spots, i),
                at(strikes, i),
                at(maturities, i),
                at(rates, i),
                at(dividends, i),
                at(vols, i),
                is_call,
            ),
        );
    }
    Ok(())
}

/// Batch Black implied volatilities (Let's Be Rational).
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn implied_volatility_slice(
    prices: &[f64],
    forwards: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    is_call: bool,
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[
        prices.len(),
        forwards.len(),
        strikes.len(),
        maturities.len(),
    ])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            implied_volatility(
                at(prices, i),
                at(forwards, i),
                at(strikes, i),
                at(maturities, i),
                is_call,
            ),
        );
    }
    Ok(())
}

/// Batch normalised Black total-volatility inversions.
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn normalised_implied_volatility_slice(
    betas: &[f64],
    xs: &[f64],
    is_call: bool,
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[betas.len(), xs.len()])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            normalised_implied_volatility(at(betas, i), at(xs, i), is_call),
        );
    }
    Ok(())
}

/// Batch Black-76 forward deltas.
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn delta_slice(
    forwards: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    vols: &[f64],
    is_call: bool,
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            delta(
                at(forwards, i),
                at(strikes, i),
                at(maturities, i),
                at(vols, i),
                is_call,
            ),
        );
    }
    Ok(())
}

/// Batch Black-76 gammas (identical for calls and puts).
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn gamma_slice(
    forwards: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    vols: &[f64],
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            gamma(
                at(forwards, i),
                at(strikes, i),
                at(maturities, i),
                at(vols, i),
            ),
        );
    }
    Ok(())
}

/// Batch Black-76 vegas (per 1.0 volatility point).
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn vega_slice(
    forwards: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    vols: &[f64],
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            vega(
                at(forwards, i),
                at(strikes, i),
                at(maturities, i),
                at(vols, i),
            ),
        );
    }
    Ok(())
}

/// Batch Black-76 vommas (volga) `∂²V/∂σ²`.
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn vomma_slice(
    forwards: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    vols: &[f64],
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            vomma(
                at(forwards, i),
                at(strikes, i),
                at(maturities, i),
                at(vols, i),
            ),
        );
    }
    Ok(())
}

/// Batch Black-76 vannas `∂²V/∂F∂σ`.
///
/// # Errors
///
/// [`IVError::InvalidInput`] on length / broadcast mismatch.
pub fn vanna_slice(
    forwards: &[f64],
    strikes: &[f64],
    maturities: &[f64],
    vols: &[f64],
    out: &mut [f64],
) -> Result<(), IVError> {
    let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
    require_out_len(out.len(), n)?;
    for (i, slot) in out.iter_mut().enumerate() {
        write_or_nan(
            slot,
            vanna(
                at(forwards, i),
                at(strikes, i),
                at(maturities, i),
                at(vols, i),
            ),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_all_ones_is_one() {
        assert_eq!(resolve_batch_len(&[1, 1, 1]).unwrap(), 1);
    }

    #[test]
    fn resolve_broadcast() {
        assert_eq!(resolve_batch_len(&[1, 4, 1, 4]).unwrap(), 4);
    }

    #[test]
    fn resolve_mismatch_errs() {
        assert!(resolve_batch_len(&[2, 3]).is_err());
    }

    #[test]
    fn empty_batch() {
        assert_eq!(resolve_batch_len(&[0, 0]).unwrap(), 0);
        assert_eq!(resolve_batch_len(&[0, 1]).unwrap(), 0);
    }
}
