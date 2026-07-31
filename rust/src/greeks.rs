//! Analytical Greeks for Black-76 and Black–Scholes.
//!
//! # Status
//!
//! Implemented in **Phase 4**.
//!
//! # Planned API
//!
//! Scalar first derivatives and selected second derivatives:
//!
//! - `delta`, `gamma`, `theta`, `vega`, `vomma` (volga), `vanna`
//!
//! All Greeks share the same domain validation as pricing and return
//! [`IVError`](crate::IVError) on invalid inputs. Conventions (which rate
//! enters theta, whether vega is per 1% or per 1.0 vol point) will be
//! documented on each function.

/// Development-phase marker (4). Removed once the module is populated.
pub const PHASE: u8 = 4;
