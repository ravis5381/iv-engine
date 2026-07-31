//! Integration tests for Black-76 and Black–Scholes pricing (Phase 3).

use iv_engine::{
    black_intrinsic, black_price, black_price_total_vol, black_scholes_forward,
    black_scholes_price, IVError, PRICE_ROUNDTRIP_ABS, PRICE_ROUNDTRIP_TOL,
};

#[test]
fn public_api_smoke() {
    let c = black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
    assert!(c > 7.0 && c < 9.0);
    let bs = black_scholes_price(100.0, 100.0, 1.0, 0.05, 0.0, 0.2, true).unwrap();
    assert!(bs > 0.0);
    assert!(black_scholes_forward(100.0, 1.0, 0.05, 0.0).unwrap() > 100.0);
}

#[test]
fn grid_parity_and_bounds() {
    let forwards = [50.0, 100.0, 150.0];
    let strikes = [80.0, 100.0, 120.0];
    let mats = [1.0 / 365.0, 0.25, 1.0, 5.0];
    let vols = [1e-8, 0.05, 0.2, 1.0, 5.0];

    for &f in &forwards {
        for &k in &strikes {
            for &t in &mats {
                for &v in &vols {
                    let c = black_price(f, k, t, v, true).unwrap();
                    let p = black_price(f, k, t, v, false).unwrap();
                    let err = (c - p - (f - k)).abs();
                    let tol = 16.0 * f64::EPSILON * (1.0 + f + k).max(1.0);
                    assert!(
                        err <= tol,
                        "parity F={f} K={k} T={t} σ={v}: err={err} tol={tol}"
                    );
                    assert!((0.0..=f * (1.0 + 1e-12)).contains(&c) || c <= f + 1e-9);
                    assert!((0.0..=k * (1.0 + 1e-12)).contains(&p) || p <= k + 1e-9);
                    assert!(c + 1e-12 >= black_intrinsic(f, k, true));
                    assert!(p + 1e-12 >= black_intrinsic(f, k, false));
                }
            }
        }
    }
}

#[test]
fn total_vol_path_matches_sigma_t() {
    for &(f, k, t, v) in &[
        (100.0, 100.0, 1.0, 0.2),
        (100.0, 80.0, 0.5, 0.4),
        (90.0, 110.0, 2.0, 0.15),
    ] {
        let a = black_price(f, k, t, v, true).unwrap();
        let b = black_price_total_vol(f, k, v * t.sqrt(), true);
        assert!((a - b).abs() < 1e-14);
    }
}

#[test]
fn very_short_dated_otm_near_zero() {
    let c = black_price(100.0, 110.0, 1.0 / 365.0, 0.1, true).unwrap();
    assert!(c < 1e-8);
}

#[test]
fn error_variants_surface() {
    assert!(matches!(
        black_price(-1.0, 100.0, 1.0, 0.2, true),
        Err(IVError::NegativeForward { .. })
    ));
    assert!(matches!(
        black_scholes_price(100.0, 100.0, 0.0, 0.0, 0.0, 0.2, true),
        Err(IVError::NegativeTime { .. })
    ));
}

#[test]
fn roundtrip_tolerance_constants_available() {
    assert!(PRICE_ROUNDTRIP_TOL > 0.0);
    assert!(PRICE_ROUNDTRIP_ABS > 0.0);
}
