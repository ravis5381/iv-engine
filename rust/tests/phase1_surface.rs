//! Integration tests for Phase 1: errors, constants, and crate surface.
//!
//! These tests exercise the public API from outside the crate to ensure
//! re-exports and documentation contracts hold.

use iv_engine::{
    constants::{
        is_finite, is_positive_above, DBL_EPSILON, MAX_IV_ITERATIONS, MIN_FORWARD, MIN_MATURITY,
        MIN_STRIKE, ONE_OVER_SQRT_TWO_PI, PRICE_ROUNDTRIP_ABS, PRICE_ROUNDTRIP_TOL,
        SQRT_PI_OVER_TWO, SQRT_TWO_PI, TWO_PI,
    },
    IVError,
};

#[test]
fn crate_reexports_iv_error() {
    let err = IVError::invalid("surface check");
    assert_eq!(err.code(), "invalid_input");
    assert!(!err.is_domain_error());
}

#[test]
fn crate_reexports_key_constants() {
    assert!(DBL_EPSILON > 0.0);
    assert!(SQRT_TWO_PI > 2.5 && SQRT_TWO_PI < 2.51);
    assert!(ONE_OVER_SQRT_TWO_PI > 0.39 && ONE_OVER_SQRT_TWO_PI < 0.40);
    assert!(SQRT_PI_OVER_TWO > 1.25 && SQRT_PI_OVER_TWO < 1.26);
    assert!((TWO_PI - 2.0 * std::f64::consts::PI).abs() < DBL_EPSILON);
}

#[test]
fn validation_thresholds_reject_non_positive() {
    assert!(!is_positive_above(MIN_FORWARD, MIN_FORWARD));
    assert!(!is_positive_above(MIN_STRIKE, MIN_STRIKE));
    assert!(!is_positive_above(MIN_MATURITY, MIN_MATURITY));
    assert!(is_positive_above(1e-300, MIN_FORWARD));
}

#[test]
fn roundtrip_tolerance_contract() {
    // Documented contract used by later Black(IV(p)) ≈ p tests.
    let price: f64 = 1.234_567_89;
    let tol = PRICE_ROUNDTRIP_TOL * price.max(1.0) + PRICE_ROUNDTRIP_ABS;
    assert!(tol > 0.0);
    assert!(tol < 1e-10);
}

#[test]
fn max_iterations_safety_cap() {
    assert!(MAX_IV_ITERATIONS >= 2);
    assert!(MAX_IV_ITERATIONS <= 64);
}

#[test]
fn error_display_is_stable_enough_for_ffi() {
    let cases = [
        IVError::PriceBelowIntrinsic {
            price: 1.0,
            intrinsic: 2.0,
        },
        IVError::PriceAboveMaximum {
            price: 3.0,
            maximum: 2.0,
        },
        IVError::NegativeTime { maturity: 0.0 },
        IVError::NegativeForward { forward: -1.0 },
        IVError::NegativeStrike { strike: -1.0 },
        IVError::NoConvergence {
            iterations: 8,
            residual: Some(1e-12),
        },
        IVError::invalid("non-finite volatility"),
    ];

    for err in cases {
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(!err.code().is_empty());
        // std::error::Error source is None for our leaf errors.
        assert!(std::error::Error::source(&err).is_none());
    }
}

#[test]
fn rejects_non_finite_via_helper() {
    assert!(is_finite(0.0));
    assert!(!is_finite(f64::NAN));
    assert!(!is_finite(f64::INFINITY));
}

#[test]
fn modules_are_linked_with_implemented_apis() {
    assert!(iv_engine::norm_pdf(0.0) > 0.0);
    assert!(iv_engine::black_price(100.0, 100.0, 1.0, 0.2, true).unwrap() > 0.0);
    assert!(iv_engine::black_scholes_price(100.0, 100.0, 1.0, 0.0, 0.0, 0.2, true).unwrap() > 0.0);
    assert!(iv_engine::delta(100.0, 100.0, 1.0, 0.2, true).unwrap() > 0.0);
    assert!(iv_engine::vega(100.0, 100.0, 1.0, 0.2).unwrap() > 0.0);
    let price = iv_engine::black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
    let iv = iv_engine::implied_volatility(price, 100.0, 100.0, 1.0, true).unwrap();
    assert!((iv - 0.2).abs() < 1e-12, "recovered {iv}");
    assert_eq!(iv_engine::vector::PHASE, 7);
    assert_eq!(iv_engine::parallel::PHASE, 8);
}
