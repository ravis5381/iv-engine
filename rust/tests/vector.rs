//! Phase 7: vectorised batch APIs.

#![allow(clippy::excessive_precision)]

use iv_engine::{
    black_price, black_price_slice, black_scholes_price, black_scholes_price_slice, delta,
    delta_slice, gamma, gamma_slice, implied_volatility, implied_volatility_slice,
    normalised_implied_volatility, normalised_implied_volatility_slice, vega, vega_slice,
};

#[test]
fn black_price_slice_matches_scalar() {
    let forwards = [100.0_f64, 100.0, 90.0];
    let strikes = [100.0, 110.0, 100.0];
    let mats = [1.0_f64; 3];
    let vols = [0.2_f64, 0.25, 0.3];
    let mut out = [0.0; 3];
    black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut out).unwrap();
    for i in 0..3 {
        let expected = black_price(forwards[i], strikes[i], mats[i], vols[i], true).unwrap();
        assert!(
            (out[i] - expected).abs() < 1e-15,
            "i={i}: got={} expected={expected}",
            out[i]
        );
    }
}

#[test]
fn broadcast_forward_across_strikes() {
    let forward = [100.0_f64];
    let strikes = [80.0_f64, 100.0, 120.0];
    let mats = [1.0_f64];
    let vols = [0.2_f64];
    let mut out = [0.0; 3];
    black_price_slice(&forward, &strikes, &mats, &vols, true, &mut out).unwrap();
    for (i, &k) in strikes.iter().enumerate() {
        let expected = black_price(100.0, k, 1.0, 0.2, true).unwrap();
        assert!((out[i] - expected).abs() < 1e-15);
    }
}

#[test]
fn length_mismatch_errors() {
    let mut out = [0.0; 2];
    let err = black_price_slice(
        &[100.0, 101.0],
        &[100.0],
        &[1.0, 1.0, 1.0],
        &[0.2],
        true,
        &mut out,
    );
    assert!(err.is_err());
}

#[test]
fn domain_failure_writes_nan() {
    let mut out = [0.0; 2];
    // Second strike is invalid (≤ 0).
    black_price_slice(
        &[100.0, 100.0],
        &[100.0, -1.0],
        &[1.0, 1.0],
        &[0.2, 0.2],
        true,
        &mut out,
    )
    .unwrap();
    assert!(out[0].is_finite() && out[0] > 0.0);
    assert!(out[1].is_nan());
}

#[test]
fn implied_volatility_slice_roundtrip() {
    let forwards = [100.0_f64; 5];
    let strikes = [80.0_f64, 90.0, 100.0, 110.0, 120.0];
    let mats = [1.0_f64; 5];
    let vols = [0.25_f64; 5];
    let mut prices = [0.0; 5];
    black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut prices).unwrap();
    let mut ivs = [0.0; 5];
    implied_volatility_slice(&prices, &forwards, &strikes, &mats, true, &mut ivs).unwrap();
    for (i, &iv) in ivs.iter().enumerate() {
        assert!((iv - 0.25).abs() < 1e-12, "i={i}: iv={iv}");
        let scalar = implied_volatility(prices[i], forwards[i], strikes[i], mats[i], true).unwrap();
        assert!((iv - scalar).abs() < 1e-15);
    }
}

#[test]
fn normalised_and_greeks_slices_match_scalar() {
    let betas = [0.08_f64, 0.185_683_012_996_663_9];
    let xs = [0.0_f64, -0.5];
    let mut s_out = [0.0; 2];
    normalised_implied_volatility_slice(&betas, &xs, true, &mut s_out).unwrap();
    for i in 0..2 {
        let expected = normalised_implied_volatility(betas[i], xs[i], true).unwrap();
        assert!((s_out[i] - expected).abs() < 1e-15);
    }

    let f = [100.0_f64, 100.0];
    let k = [100.0_f64, 110.0];
    let t = [1.0_f64, 1.0];
    let v = [0.2_f64, 0.25];
    let mut d = [0.0; 2];
    let mut g = [0.0; 2];
    let mut vg = [0.0; 2];
    delta_slice(&f, &k, &t, &v, true, &mut d).unwrap();
    gamma_slice(&f, &k, &t, &v, &mut g).unwrap();
    vega_slice(&f, &k, &t, &v, &mut vg).unwrap();
    for i in 0..2 {
        assert!((d[i] - delta(f[i], k[i], t[i], v[i], true).unwrap()).abs() < 1e-15);
        assert!((g[i] - gamma(f[i], k[i], t[i], v[i]).unwrap()).abs() < 1e-15);
        assert!((vg[i] - vega(f[i], k[i], t[i], v[i]).unwrap()).abs() < 1e-15);
    }
}

#[test]
fn black_scholes_slice_matches_scalar() {
    let spots = [100.0_f64; 3];
    let strikes = [95.0_f64, 100.0, 105.0];
    let mats = [0.5_f64; 3];
    let rates = [0.05_f64];
    let divs = [0.02_f64];
    let vols = [0.2_f64];
    let mut out = [0.0; 3];
    black_scholes_price_slice(
        &spots, &strikes, &mats, &rates, &divs, &vols, true, &mut out,
    )
    .unwrap();
    for i in 0..3 {
        let expected =
            black_scholes_price(spots[i], strikes[i], mats[i], 0.05, 0.02, 0.2, true).unwrap();
        assert!((out[i] - expected).abs() < 1e-15);
    }
}

#[test]
fn empty_output_ok() {
    let mut out: [f64; 0] = [];
    black_price_slice(&[], &[], &[], &[], true, &mut out).unwrap();
}
