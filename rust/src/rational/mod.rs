//! Let's Be Rational implied-volatility solver.
//!
//! # Status
//!
//! Implemented in **Phase 5**, validated against the reference in **Phase 6**.
//!
//! # Algorithm overview
//!
//! Peter Jäckel's *Let's Be Rational* recovers Black implied volatility to
//! near machine precision without a classical Newton loop on the raw price:
//!
//! 1. Normalise the market price to the dimensionless *normalised Black*
//!    value.
//! 2. Obtain a high-quality initial guess via asymptotic expansions and
//!    rational interpolants (branches for small / large moneyness and
//!    intrinsic regime).
//! 3. Apply a Householder (third-order) refinement that typically converges
//!    in one or two iterations.
//!
//! Submodules mirror the paper's logical stages:
//!
//! | Module              | Role                                      |
//! |---------------------|-------------------------------------------|
//! | [`interpolation`]   | Rational interpolants / branch polynomials |
//! | [`initial_guess`]   | Asymptotic + interpolated σ₀               |
//! | [`inverse`]         | Normalised-price → total-volatility map    |
//! | [`refinement`]      | Householder corrector                      |
//!
//! # References
//!
//! Jäckel, P. (2015). *Let's Be Rational*. Wilmott, 2015(75), 40–53.
//! <http://www.jaeckel.org/LetsBeRational.pdf>

pub mod initial_guess;
pub mod interpolation;
pub mod inverse;
pub mod refinement;

/// Development-phase marker (5). Removed once the module is populated.
pub const PHASE: u8 = 5;
