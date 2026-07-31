//! # iv-engine
//!
//! High-performance Black / Black–Scholes pricing and implied volatility via
//! Peter Jäckel's *Let's Be Rational* algorithm.
//!
//! ## Design
//!
//! This crate is the **source of truth** for all numerical routines. Python
//! bindings (added in a later phase) wrap these APIs and must not reimplement
//! pricing or root-finding logic.
//!
//! ## Public surface (planned)
//!
//! | Function                 | Phase | Status   |
//! |--------------------------|-------|----------|
//! | [`errors::IVError`]      | 1     | Ready    |
//! | [`constants`]            | 1     | Ready    |
//! | [`normal`] PDF / CDF     | 2     | Ready    |
//! | [`black_price`]          | 3     | Ready    |
//! | [`black_scholes_price`]  | 3     | Ready    |
//! | [`greeks`]               | 4     | Ready    |
//! | Implied volatility (LBR) | 5–6   | Ready    |
//! | [`vector`] batch APIs    | 7     | Ready    |
//! | [`parallel`] (feature)   | 8     | Ready    |
//! | Optional SIMD            | 9     | Planned  |
//!
//! ## Error handling
//!
//! All fallible public functions return [`Result<T, IVError>`](errors::IVError).
//! The library never panics on invalid market inputs.
//!
//! ## References
//!
//! - Jäckel, P. (2015). *Let's Be Rational*. Wilmott Magazine.
//! - Black, F. (1976). The pricing of commodity contracts.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod black;
pub mod black_scholes;
pub mod constants;
pub mod errors;
pub mod greeks;
pub mod normal;
pub mod parallel;
pub mod rational;
pub mod vector;

pub use black::{black_intrinsic, black_price, black_price_total_vol, log_moneyness};
pub use black_scholes::{black_scholes_forward, black_scholes_price};
pub use constants::{
    DBL_EPSILON, DBL_MAX, DBL_MIN, MIN_FORWARD, MIN_MATURITY, MIN_STRIKE, ONE_OVER_SQRT_TWO_PI,
    PRICE_ROUNDTRIP_ABS, PRICE_ROUNDTRIP_TOL, SQRT_PI_OVER_TWO, SQRT_TWO, SQRT_TWO_PI, TWO_PI,
};
pub use errors::IVError;
pub use greeks::{
    black_scholes_delta, black_scholes_gamma, black_scholes_theta, black_scholes_vanna,
    black_scholes_vega, black_scholes_vomma, delta, gamma, theta, vanna, vega, vomma,
};
pub use normal::{norm_cdf, norm_cdf_c, norm_pdf};
pub use parallel::{
    black_price_slice_par, black_scholes_price_slice_par, delta_slice_par, gamma_slice_par,
    implied_volatility_slice_par, normalised_implied_volatility_slice_par, rayon_enabled,
    vega_slice_par, MIN_PARALLEL,
};
pub use rational::{implied_volatility, normalised_implied_volatility};
pub use vector::{
    black_price_slice, black_scholes_price_slice, delta_slice, gamma_slice,
    implied_volatility_slice, normalised_implied_volatility_slice, vega_slice,
};
