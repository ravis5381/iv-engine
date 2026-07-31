//! Optional Rayon-backed parallel batch evaluation.
//!
//! # Design
//!
//! Parallel entry points mirror [`crate::vector`]: same length-1 broadcast
//! rules, same `f64::NAN` per-element failure contract, and **bit-identical**
//! results to the serial kernels on the same inputs.
//!
//! Enable with Cargo feature `rayon`:
//!
//! ```bash
//! cargo test -p iv-engine --features rayon
//! ```
//!
//! Without the feature, these functions fall back to the serial
//! [`crate::vector`] implementations so the default build stays free of the
//! Rayon dependency.
//!
//! Batches smaller than [`MIN_PARALLEL`] always use the serial path even when
//! Rayon is enabled — thread-pool overhead dominates for tiny jobs.

/// Minimum batch length before Rayon dispatch is worthwhile.
pub const MIN_PARALLEL: usize = 64;

/// Whether this build was compiled with the `rayon` Cargo feature.
#[must_use]
pub const fn rayon_enabled() -> bool {
    cfg!(feature = "rayon")
}

#[cfg(feature = "rayon")]
mod rayon_impl {
    use super::MIN_PARALLEL;
    use crate::black::black_price;
    use crate::black_scholes::black_scholes_price;
    use crate::errors::IVError;
    use crate::greeks::{delta, gamma, vanna, vega, vomma};
    use crate::rational::{implied_volatility, normalised_implied_volatility};
    use crate::vector::{
        at, black_price_slice as serial_black_price_slice,
        black_scholes_price_slice as serial_black_scholes_price_slice,
        delta_slice as serial_delta_slice, gamma_slice as serial_gamma_slice,
        implied_volatility_slice as serial_implied_volatility_slice,
        normalised_implied_volatility_slice as serial_normalised_implied_volatility_slice,
        require_out_len, resolve_batch_len, vanna_slice as serial_vanna_slice,
        vega_slice as serial_vega_slice, vomma_slice as serial_vomma_slice, write_or_nan,
    };
    use rayon::prelude::*;

    /// Parallel Black-76 prices (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn black_price_slice_par(
        forwards: &[f64],
        strikes: &[f64],
        maturities: &[f64],
        vols: &[f64],
        is_call: bool,
        out: &mut [f64],
    ) -> Result<(), IVError> {
        let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
        require_out_len(out.len(), n)?;
        if n < MIN_PARALLEL {
            return serial_black_price_slice(forwards, strikes, maturities, vols, is_call, out);
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
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
        });
        Ok(())
    }

    /// Parallel Black–Scholes–Merton prices (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    #[allow(clippy::too_many_arguments)]
    pub fn black_scholes_price_slice_par(
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
        if n < MIN_PARALLEL {
            return serial_black_scholes_price_slice(
                spots, strikes, maturities, rates, dividends, vols, is_call, out,
            );
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
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
        });
        Ok(())
    }

    /// Parallel Black implied volatilities (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn implied_volatility_slice_par(
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
        if n < MIN_PARALLEL {
            return serial_implied_volatility_slice(
                prices, forwards, strikes, maturities, is_call, out,
            );
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
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
        });
        Ok(())
    }

    /// Parallel normalised Black total-volatility inversions (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn normalised_implied_volatility_slice_par(
        betas: &[f64],
        xs: &[f64],
        is_call: bool,
        out: &mut [f64],
    ) -> Result<(), IVError> {
        let n = resolve_batch_len(&[betas.len(), xs.len()])?;
        require_out_len(out.len(), n)?;
        if n < MIN_PARALLEL {
            return serial_normalised_implied_volatility_slice(betas, xs, is_call, out);
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
            write_or_nan(
                slot,
                normalised_implied_volatility(at(betas, i), at(xs, i), is_call),
            );
        });
        Ok(())
    }

    /// Parallel Black-76 forward deltas (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn delta_slice_par(
        forwards: &[f64],
        strikes: &[f64],
        maturities: &[f64],
        vols: &[f64],
        is_call: bool,
        out: &mut [f64],
    ) -> Result<(), IVError> {
        let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
        require_out_len(out.len(), n)?;
        if n < MIN_PARALLEL {
            return serial_delta_slice(forwards, strikes, maturities, vols, is_call, out);
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
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
        });
        Ok(())
    }

    /// Parallel Black-76 gammas (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn gamma_slice_par(
        forwards: &[f64],
        strikes: &[f64],
        maturities: &[f64],
        vols: &[f64],
        out: &mut [f64],
    ) -> Result<(), IVError> {
        let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
        require_out_len(out.len(), n)?;
        if n < MIN_PARALLEL {
            return serial_gamma_slice(forwards, strikes, maturities, vols, out);
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
            write_or_nan(
                slot,
                gamma(
                    at(forwards, i),
                    at(strikes, i),
                    at(maturities, i),
                    at(vols, i),
                ),
            );
        });
        Ok(())
    }

    /// Parallel Black-76 vegas (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn vega_slice_par(
        forwards: &[f64],
        strikes: &[f64],
        maturities: &[f64],
        vols: &[f64],
        out: &mut [f64],
    ) -> Result<(), IVError> {
        let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
        require_out_len(out.len(), n)?;
        if n < MIN_PARALLEL {
            return serial_vega_slice(forwards, strikes, maturities, vols, out);
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
            write_or_nan(
                slot,
                vega(
                    at(forwards, i),
                    at(strikes, i),
                    at(maturities, i),
                    at(vols, i),
                ),
            );
        });
        Ok(())
    }

    /// Parallel Black-76 vommas (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn vomma_slice_par(
        forwards: &[f64],
        strikes: &[f64],
        maturities: &[f64],
        vols: &[f64],
        out: &mut [f64],
    ) -> Result<(), IVError> {
        let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
        require_out_len(out.len(), n)?;
        if n < MIN_PARALLEL {
            return serial_vomma_slice(forwards, strikes, maturities, vols, out);
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
            write_or_nan(
                slot,
                vomma(
                    at(forwards, i),
                    at(strikes, i),
                    at(maturities, i),
                    at(vols, i),
                ),
            );
        });
        Ok(())
    }

    /// Parallel Black-76 vannas (see module docs).
    ///
    /// # Errors
    ///
    /// [`IVError::InvalidInput`] on length / broadcast mismatch.
    pub fn vanna_slice_par(
        forwards: &[f64],
        strikes: &[f64],
        maturities: &[f64],
        vols: &[f64],
        out: &mut [f64],
    ) -> Result<(), IVError> {
        let n = resolve_batch_len(&[forwards.len(), strikes.len(), maturities.len(), vols.len()])?;
        require_out_len(out.len(), n)?;
        if n < MIN_PARALLEL {
            return serial_vanna_slice(forwards, strikes, maturities, vols, out);
        }
        out.par_iter_mut().enumerate().for_each(|(i, slot)| {
            write_or_nan(
                slot,
                vanna(
                    at(forwards, i),
                    at(strikes, i),
                    at(maturities, i),
                    at(vols, i),
                ),
            );
        });
        Ok(())
    }
}

#[cfg(not(feature = "rayon"))]
mod serial_fallback {
    pub use crate::vector::{
        black_price_slice as black_price_slice_par,
        black_scholes_price_slice as black_scholes_price_slice_par, delta_slice as delta_slice_par,
        gamma_slice as gamma_slice_par, implied_volatility_slice as implied_volatility_slice_par,
        normalised_implied_volatility_slice as normalised_implied_volatility_slice_par,
        vanna_slice as vanna_slice_par, vega_slice as vega_slice_par,
        vomma_slice as vomma_slice_par,
    };
}

#[cfg(feature = "rayon")]
pub use rayon_impl::{
    black_price_slice_par, black_scholes_price_slice_par, delta_slice_par, gamma_slice_par,
    implied_volatility_slice_par, normalised_implied_volatility_slice_par, vanna_slice_par,
    vega_slice_par, vomma_slice_par,
};

#[cfg(not(feature = "rayon"))]
pub use serial_fallback::{
    black_price_slice_par, black_scholes_price_slice_par, delta_slice_par, gamma_slice_par,
    implied_volatility_slice_par, normalised_implied_volatility_slice_par, vanna_slice_par,
    vega_slice_par, vomma_slice_par,
};
