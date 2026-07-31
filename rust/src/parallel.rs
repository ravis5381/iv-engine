//! Optional Rayon-backed parallel batch evaluation.
//!
//! # Status
//!
//! Implemented in **Phase 8**.
//!
//! # Design
//!
//! Parallelism is feature-gated (`rayon`) so the default build stays
//! dependency-free. Parallel kernels chunk the same slice API as
//! [`crate::vector`] and must bit-match the serial results on identical
//! inputs (validated by tests).

/// Development-phase marker (8). Removed once the module is populated.
pub const PHASE: u8 = 8;
