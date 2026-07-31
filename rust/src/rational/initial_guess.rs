//! Initial total-volatility guess for the Householder refiner.
//!
//! # Status
//!
//! Implemented in **Phase 5**.
//!
//! Combines asymptotic expansions (small-time, intrinsic limit, ATM limit)
//! with the rational interpolants in [`crate::rational::interpolation`] to
//! produce an initial guess accurate enough that a single Householder step
//! reaches machine precision on almost all inputs.

/// Development-phase marker (5). Removed once the module is populated.
pub const PHASE: u8 = 5;
