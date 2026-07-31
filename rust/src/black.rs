//! Undiscounted Black-76 option pricing.
//!
//! # Status
//!
//! Implemented in **Phase 3**.
//!
//! # Planned API
//!
//! ```ignore
//! pub fn black_price(
//!     forward: f64,
//!     strike: f64,
//!     maturity: f64,
//!     volatility: f64,
//!     is_call: bool,
//! ) -> Result<f64, IVError>
//! ```
//!
//! # Formula
//!
//! For total volatility `s = σ√T` and log-moneyness `x = ln(F/K)`:
//!
//! ```text
//! Call = F · Φ(d₁) − K · Φ(d₂)
//! Put  = K · Φ(−d₂) − F · Φ(−d₁)
//! d₁,₂ = (x ± s²/2) / s
//! ```
//!
//! Numerically stable formulations (put–call parity, complementary CDF)
//! will be used for deep ITM / OTM contracts.
//!
//! # References
//!
//! Black, F. (1976). The pricing of commodity contracts.

/// Development-phase marker (3). Removed once the module is populated.
pub const PHASE: u8 = 3;
