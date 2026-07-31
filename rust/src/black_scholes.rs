//! Discounted Black–Scholes–Merton pricing on a spot underlying.
//!
//! # Status
//!
//! Implemented in **Phase 3**.
//!
//! # Planned API
//!
//! ```ignore
//! pub fn black_scholes_price(
//!     spot: f64,
//!     strike: f64,
//!     maturity: f64,
//!     rate: f64,
//!     dividend: f64,
//!     volatility: f64,
//!     is_call: bool,
//! ) -> Result<f64, IVError>
//! ```
//!
//! Implemented as a thin wrapper around [`crate::black`]: map spot to the
//! forward `F = S · e^{(r−q)T}` and discount the Black price by `e^{−rT}`.
//!
//! # References
//!
//! Black, F. & Scholes, M. (1973). The pricing of options and corporate liabilities.

/// Development-phase marker (3). Removed once the module is populated.
pub const PHASE: u8 = 3;
