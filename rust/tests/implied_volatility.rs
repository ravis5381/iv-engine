//! Integration tests for the Let's Be Rational surface.

use iv_engine::{
    black_price, implied_volatility, IVError, PRICE_ROUNDTRIP_ABS, PRICE_ROUNDTRIP_TOL,
};

#[test]
fn round_trips_black_prices() {
    for &(f, k, t, sigma, call) in &[
        (100.0, 100.0, 1.0, 0.2, true),
        (100.0, 80.0, 0.5, 0.35, true),
        (100.0, 120.0, 2.0, 0.15, true),
        (100.0, 80.0, 0.5, 0.35, false),
        (100.0, 50.0, 0.25, 0.10, true),
        (100.0, 200.0, 0.25, 0.10, true),
        (100.0, 70.0, 0.01, 0.80, false),
        (100.0, 130.0, 10.0, 1.50, true),
        (100.0, 100.0, 5.0, 0.02, false),
        (100.0, 150.0, 0.05, 0.60, false),
    ] {
        let price = black_price(f, k, t, sigma, call).unwrap();
        let recovered = implied_volatility(price, f, k, t, call).unwrap();
        let repriced = black_price(f, k, t, recovered, call).unwrap();
        let tolerance = PRICE_ROUNDTRIP_ABS + PRICE_ROUNDTRIP_TOL * price.abs().max(1.0);
        assert!(
            (repriced - price).abs() <= tolerance,
            "F={f} K={k} T={t} sigma={sigma} repriced={repriced} price={price}"
        );
    }
}

#[test]
fn put_call_parity_recovers_same_volatility() {
    let call = black_price(100.0, 110.0, 1.0, 0.3, true).unwrap();
    let put = black_price(100.0, 110.0, 1.0, 0.3, false).unwrap();
    let call_iv = implied_volatility(call, 100.0, 110.0, 1.0, true).unwrap();
    let put_iv = implied_volatility(put, 100.0, 110.0, 1.0, false).unwrap();
    assert!((call_iv - put_iv).abs() < 1e-12);
}

#[test]
fn rejects_price_and_market_domain_violations() {
    assert!(matches!(
        implied_volatility(-1.0, 100.0, 100.0, 1.0, true),
        Err(IVError::PriceBelowIntrinsic { .. })
    ));
    assert!(matches!(
        implied_volatility(100.0, 100.0, 100.0, 1.0, true),
        Err(IVError::PriceAboveMaximum { .. })
    ));
    assert!(matches!(
        implied_volatility(1.0, 0.0, 100.0, 1.0, true),
        Err(IVError::NegativeForward { .. })
    ));
    assert!(matches!(
        implied_volatility(1.0, 100.0, 0.0, 1.0, true),
        Err(IVError::NegativeStrike { .. })
    ));
    assert!(matches!(
        implied_volatility(1.0, 100.0, 100.0, 0.0, true),
        Err(IVError::NegativeTime { .. })
    ));
}
