//! Integration tests for analytical Greeks (Phase 4).

use iv_engine::{
    black_price, black_scholes_delta, black_scholes_gamma, black_scholes_price,
    black_scholes_theta, black_scholes_vanna, black_scholes_vega, black_scholes_vomma, delta,
    gamma, theta, vanna, vega, vomma,
};

#[test]
fn smoke_all_greeks_finite() {
    let f = 100.0;
    let k = 100.0;
    let t = 1.0;
    let v = 0.2;
    for g in [
        delta(f, k, t, v, true).unwrap(),
        delta(f, k, t, v, false).unwrap(),
        gamma(f, k, t, v).unwrap(),
        vega(f, k, t, v).unwrap(),
        theta(f, k, t, v).unwrap(),
        vomma(f, k, t, v).unwrap(),
        vanna(f, k, t, v).unwrap(),
    ] {
        assert!(g.is_finite(), "{g}");
    }
}

#[test]
fn call_put_vega_gamma_identical_via_parity() {
    // Vega/gamma are identity for call and put under Black parity.
    let f = 120.0;
    let k = 100.0;
    let t = 0.75;
    let v = 0.3;
    let vega_c = {
        let h = 1e-6;
        (black_price(f, k, t, v + h, true).unwrap() - black_price(f, k, t, v - h, true).unwrap())
            / (2.0 * h)
    };
    let vega_p = {
        let h = 1e-6;
        (black_price(f, k, t, v + h, false).unwrap() - black_price(f, k, t, v - h, false).unwrap())
            / (2.0 * h)
    };
    assert!((vega_c - vega_p).abs() < 1e-6);
    assert!((vega(f, k, t, v).unwrap() - vega_c).abs() < 1e-5);
}

#[test]
fn bs_greeks_smoke() {
    let s = 100.0;
    let k = 100.0;
    let t = 1.0;
    let r = 0.05;
    let q = 0.01;
    let v = 0.2;
    assert!(black_scholes_delta(s, k, t, r, q, v, true).unwrap() > 0.0);
    assert!(black_scholes_gamma(s, k, t, r, q, v).unwrap() > 0.0);
    assert!(black_scholes_vega(s, k, t, r, q, v).unwrap() > 0.0);
    assert!(black_scholes_theta(s, k, t, r, q, v, true)
        .unwrap()
        .is_finite());
    assert!(black_scholes_vomma(s, k, t, r, q, v).unwrap().is_finite());
    assert!(black_scholes_vanna(s, k, t, r, q, v).unwrap().is_finite());
    // Price still works alongside Greeks.
    assert!(black_scholes_price(s, k, t, r, q, v, true).unwrap() > 0.0);
}

#[test]
fn deep_itm_call_delta_near_one() {
    let d = delta(200.0, 100.0, 1.0, 0.1, true).unwrap();
    assert!(d > 0.999);
}

#[test]
fn deep_otm_call_delta_near_zero() {
    let d = delta(50.0, 100.0, 0.25, 0.1, true).unwrap();
    assert!(d < 1e-3);
}
