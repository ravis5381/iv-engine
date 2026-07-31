//! Standard Normal density and cumulative distribution.
//!
//! # Status
//!
//! Implemented in **Phase 2**.
//!
//! # Planned API
//!
//! - `norm_pdf(x: f64) -> f64` — φ(x)
//! - `norm_cdf(x: f64) -> f64` — Φ(x), high-accuracy complementary formulation
//! - `norm_cdf_c(x: f64) -> f64` — 1 − Φ(x) without catastrophic cancellation
//!
//! # References
//!
//! Accurate Φ will follow a branch-wise rational / asymptotic scheme suitable
//! for deep OTM Black pricing (complementary tails). See also West, G.
//! (2005) and the complementary-error-function literature.

/// Development-phase marker (2). Removed once the module is populated.
pub const PHASE: u8 = 2;
