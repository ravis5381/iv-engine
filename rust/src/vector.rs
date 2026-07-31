//! Vectorised batch APIs over contiguous slices.
//!
//! # Status
//!
//! Implemented in **Phase 7**.
//!
//! # Design
//!
//! Scalar kernels remain authoritative. Vector entry points accept
//! aligned `&[f64]` inputs of equal length and write into an `&mut [f64]`
//! output (or return `Vec<Result<…>>` where fallibility must be preserved
//! per element). No intermediate heap allocations in the happy path beyond
//! the caller-provided buffers.
//!
//! Example shape:
//!
//! ```ignore
//! pub fn implied_volatility_slice(
//!     prices: &[f64],
//!     forwards: &[f64],
//!     strikes: &[f64],
//!     maturities: &[f64],
//!     is_call: &[bool],
//!     out: &mut [f64],
//! ) -> Result<(), IVError>
//! ```

/// Development-phase marker (7). Removed once the module is populated.
pub const PHASE: u8 = 7;
