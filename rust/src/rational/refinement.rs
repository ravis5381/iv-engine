//! Householder refinement of the implied total volatility.
//!
//! # Status
//!
//! Implemented in **Phase 5**.
//!
//! Uses the first three derivatives of the normalised Black function to
//! apply a cubic Householder update. Iteration count is capped by
//! [`crate::constants::MAX_IV_ITERATIONS`].

/// Development-phase marker (5). Removed once the module is populated.
pub const PHASE: u8 = 5;
