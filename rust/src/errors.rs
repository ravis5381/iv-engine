//! Error types for the iv-engine numerical core.
//!
//! # Design
//!
//! Every fallible public API returns [`Result<T, IVError>`]. Invalid market
//! data (negative strike, price below intrinsic, etc.) is reported as an
//! error — the library **never panics** on domain inputs.
//!
//! Errors are cheap to construct (`Copy` where possible) so hot paths can
//! validate without allocation.
//!
//! # Variants
//!
//! | Variant                          | Typical cause                                      |
//! |----------------------------------|----------------------------------------------------|
//! | [`IVError::PriceBelowIntrinsic`] | Option price &lt; discounted intrinsic value       |
//! | [`IVError::PriceAboveMaximum`]   | Option price &gt; undiscounted forward bound       |
//! | [`IVError::NegativeTime`]        | Maturity / time-to-expiry ≤ 0                      |
//! | [`IVError::NegativeForward`]     | Forward price ≤ 0                                  |
//! | [`IVError::NegativeStrike`]      | Strike ≤ 0                                         |
//! | [`IVError::NoConvergence`]       | Root finder failed to meet tolerance               |
//! | [`IVError::InvalidInput`]        | Non-finite values, empty slices, length mismatch   |

use core::fmt;

/// Error type returned by all fallible iv-engine APIs.
///
/// Implements [`std::error::Error`] and [`Display`](fmt::Display) for
/// ergonomic propagation into Python (`PyErr`) and application layers.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum IVError {
    /// Market price is strictly below the model intrinsic value.
    ///
    /// For undiscounted Black-76, a call intrinsic is `max(F − K, 0)` and a
    /// put intrinsic is `max(K − F, 0)`. Prices below intrinsic have no
    /// real implied volatility.
    PriceBelowIntrinsic {
        /// Observed option premium.
        price: f64,
        /// Model intrinsic value for the given `(F, K, is_call)`.
        intrinsic: f64,
    },

    /// Market price exceeds the theoretical maximum for the contract.
    ///
    /// For undiscounted Black-76 the call maximum is the forward `F` and the
    /// put maximum is the strike `K`. Prices above these bounds are
    /// arbitrageable and have no finite implied volatility.
    PriceAboveMaximum {
        /// Observed option premium.
        price: f64,
        /// Theoretical upper bound.
        maximum: f64,
    },

    /// Time to maturity is negative or zero.
    ///
    /// Zero maturity is excluded because the Black mapping is discontinuous
    /// at `T = 0` (price collapses to intrinsic). Callers that need the
    /// expiry payoff should handle `T = 0` outside this library.
    NegativeTime {
        /// Provided time to maturity (years, or model time unit).
        maturity: f64,
    },

    /// Forward price is negative or zero.
    ///
    /// Black / Black–Scholes log-moneyness `ln(F/K)` is undefined for
    /// non-positive forwards.
    NegativeForward {
        /// Provided forward price.
        forward: f64,
    },

    /// Strike is negative or zero.
    NegativeStrike {
        /// Provided strike.
        strike: f64,
    },

    /// Iterative refinement failed to converge within the iteration budget.
    ///
    /// This should be extremely rare for Let's Be Rational on valid inputs;
    /// if it occurs, treat it as a numerical defect and report a bug.
    NoConvergence {
        /// Iterations performed before giving up.
        iterations: u32,
        /// Absolute residual `|model_price − market_price|` at termination,
        /// when available.
        residual: Option<f64>,
    },

    /// Catch-all for inputs that are non-finite, inconsistent, or otherwise
    /// outside the documented domain (e.g. empty vector slices, mismatched
    /// lengths, negative prices that are not covered by a more specific
    /// variant).
    InvalidInput {
        /// Human-readable reason (static string — no heap allocation).
        reason: &'static str,
    },
}

impl IVError {
    /// Returns a short `snake_case` machine-readable code for logging / FFI.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PriceBelowIntrinsic { .. } => "price_below_intrinsic",
            Self::PriceAboveMaximum { .. } => "price_above_maximum",
            Self::NegativeTime { .. } => "negative_time",
            Self::NegativeForward { .. } => "negative_forward",
            Self::NegativeStrike { .. } => "negative_strike",
            Self::NoConvergence { .. } => "no_convergence",
            Self::InvalidInput { .. } => "invalid_input",
        }
    }

    /// Convenience constructor for [`IVError::InvalidInput`].
    #[must_use]
    pub const fn invalid(reason: &'static str) -> Self {
        Self::InvalidInput { reason }
    }

    /// Returns `true` when the error indicates an arbitrage / no-IV domain
    /// violation rather than a programming mistake.
    #[must_use]
    pub const fn is_domain_error(self) -> bool {
        matches!(
            self,
            Self::PriceBelowIntrinsic { .. }
                | Self::PriceAboveMaximum { .. }
                | Self::NegativeTime { .. }
                | Self::NegativeForward { .. }
                | Self::NegativeStrike { .. }
        )
    }
}

impl fmt::Display for IVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PriceBelowIntrinsic { price, intrinsic } => write!(
                f,
                "option price {price} is below intrinsic value {intrinsic}"
            ),
            Self::PriceAboveMaximum { price, maximum } => {
                write!(f, "option price {price} exceeds maximum value {maximum}")
            }
            Self::NegativeTime { maturity } => {
                write!(f, "maturity must be positive, got {maturity}")
            }
            Self::NegativeForward { forward } => {
                write!(f, "forward must be positive, got {forward}")
            }
            Self::NegativeStrike { strike } => {
                write!(f, "strike must be positive, got {strike}")
            }
            Self::NoConvergence {
                iterations,
                residual,
            } => match residual {
                Some(r) => write!(
                    f,
                    "implied volatility failed to converge after {iterations} iterations (residual {r})"
                ),
                None => write!(
                    f,
                    "implied volatility failed to converge after {iterations} iterations"
                ),
            },
            Self::InvalidInput { reason } => write!(f, "invalid input: {reason}"),
        }
    }
}

impl std::error::Error for IVError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_price_below_intrinsic() {
        let err = IVError::PriceBelowIntrinsic {
            price: 0.5,
            intrinsic: 1.0,
        };
        let msg = err.to_string();
        assert!(msg.contains("0.5"));
        assert!(msg.contains('1'));
        assert!(msg.contains("below intrinsic"));
    }

    #[test]
    fn display_price_above_maximum() {
        let err = IVError::PriceAboveMaximum {
            price: 110.0,
            maximum: 100.0,
        };
        let msg = err.to_string();
        assert!(msg.contains("110"));
        assert!(msg.contains("100"));
        assert!(msg.contains("exceeds maximum"));
    }

    #[test]
    fn display_negative_time() {
        let err = IVError::NegativeTime { maturity: -0.1 };
        assert!(err.to_string().contains("maturity must be positive"));
        assert!(err.to_string().contains("-0.1"));
    }

    #[test]
    fn display_negative_forward() {
        let err = IVError::NegativeForward { forward: 0.0 };
        assert!(err.to_string().contains("forward must be positive"));
    }

    #[test]
    fn display_negative_strike() {
        let err = IVError::NegativeStrike { strike: -5.0 };
        assert!(err.to_string().contains("strike must be positive"));
    }

    #[test]
    fn display_no_convergence_with_residual() {
        let err = IVError::NoConvergence {
            iterations: 4,
            residual: Some(1e-8),
        };
        let msg = err.to_string();
        assert!(msg.contains("4 iterations"));
        assert!(msg.contains("residual"));
    }

    #[test]
    fn display_no_convergence_without_residual() {
        let err = IVError::NoConvergence {
            iterations: 2,
            residual: None,
        };
        let msg = err.to_string();
        assert!(msg.contains("2 iterations"));
        assert!(!msg.contains("residual"));
    }

    #[test]
    fn display_invalid_input() {
        let err = IVError::invalid("non-finite price");
        assert_eq!(err.to_string(), "invalid input: non-finite price");
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(
            IVError::PriceBelowIntrinsic {
                price: 0.0,
                intrinsic: 1.0
            }
            .code(),
            "price_below_intrinsic"
        );
        assert_eq!(
            IVError::PriceAboveMaximum {
                price: 2.0,
                maximum: 1.0
            }
            .code(),
            "price_above_maximum"
        );
        assert_eq!(
            IVError::NegativeTime { maturity: -1.0 }.code(),
            "negative_time"
        );
        assert_eq!(
            IVError::NegativeForward { forward: -1.0 }.code(),
            "negative_forward"
        );
        assert_eq!(
            IVError::NegativeStrike { strike: -1.0 }.code(),
            "negative_strike"
        );
        assert_eq!(
            IVError::NoConvergence {
                iterations: 1,
                residual: None
            }
            .code(),
            "no_convergence"
        );
        assert_eq!(IVError::invalid("x").code(), "invalid_input");
    }

    #[test]
    fn domain_error_classification() {
        assert!(IVError::NegativeStrike { strike: -1.0 }.is_domain_error());
        assert!(IVError::PriceBelowIntrinsic {
            price: 0.0,
            intrinsic: 1.0
        }
        .is_domain_error());
        assert!(!IVError::NoConvergence {
            iterations: 1,
            residual: None
        }
        .is_domain_error());
        assert!(!IVError::invalid("bad").is_domain_error());
    }

    #[test]
    fn error_trait_object_safe() {
        let err: Box<dyn std::error::Error> = Box::new(IVError::invalid("test"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn copy_and_eq() {
        let a = IVError::NegativeTime { maturity: -1.0 };
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, IVError::NegativeTime { maturity: -2.0 });
    }
}
