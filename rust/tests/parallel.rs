//! Phase 8: Rayon parallel batch APIs.

use iv_engine::{
    black_price_slice, black_price_slice_par, black_scholes_price_slice,
    black_scholes_price_slice_par, delta_slice, delta_slice_par, gamma_slice, gamma_slice_par,
    implied_volatility_slice, implied_volatility_slice_par, rayon_enabled, vanna_slice,
    vanna_slice_par, vega_slice, vega_slice_par, vomma_slice, vomma_slice_par, MIN_PARALLEL,
};

fn assert_slices_bit_equal(a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len());
    for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
        assert!(
            x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()),
            "mismatch at {i}: {x:?} vs {y:?}"
        );
    }
}

#[test]
fn small_batch_matches_serial() {
    let n = MIN_PARALLEL / 2;
    let forwards = vec![100.0_f64; n];
    let strikes: Vec<f64> = (0..n).map(|i| 90.0 + i as f64 * 0.1).collect();
    let mats = vec![1.0_f64; n];
    let vols = vec![0.2_f64; n];
    let mut serial = vec![0.0; n];
    let mut parallel = vec![0.0; n];
    black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut serial).unwrap();
    black_price_slice_par(&forwards, &strikes, &mats, &vols, true, &mut parallel).unwrap();
    assert_slices_bit_equal(&serial, &parallel);
}

#[test]
fn large_batch_matches_serial() {
    let n = MIN_PARALLEL * 4;
    let forwards = vec![100.0_f64; n];
    let strikes: Vec<f64> = (0..n)
        .map(|i| 80.0 + (i as f64) * 40.0 / n as f64)
        .collect();
    let mats = vec![1.0_f64; n];
    let vols = vec![0.25_f64; n];

    let mut prices_s = vec![0.0; n];
    let mut prices_p = vec![0.0; n];
    black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut prices_s).unwrap();
    black_price_slice_par(&forwards, &strikes, &mats, &vols, true, &mut prices_p).unwrap();
    assert_slices_bit_equal(&prices_s, &prices_p);

    let mut iv_s = vec![0.0; n];
    let mut iv_p = vec![0.0; n];
    implied_volatility_slice(&prices_s, &forwards, &strikes, &mats, true, &mut iv_s).unwrap();
    implied_volatility_slice_par(&prices_p, &forwards, &strikes, &mats, true, &mut iv_p).unwrap();
    assert_slices_bit_equal(&iv_s, &iv_p);

    let mut d_s = vec![0.0; n];
    let mut d_p = vec![0.0; n];
    let mut g_s = vec![0.0; n];
    let mut g_p = vec![0.0; n];
    let mut v_s = vec![0.0; n];
    let mut v_p = vec![0.0; n];
    let mut vo_s = vec![0.0; n];
    let mut vo_p = vec![0.0; n];
    let mut va_s = vec![0.0; n];
    let mut va_p = vec![0.0; n];
    delta_slice(&forwards, &strikes, &mats, &vols, true, &mut d_s).unwrap();
    delta_slice_par(&forwards, &strikes, &mats, &vols, true, &mut d_p).unwrap();
    gamma_slice(&forwards, &strikes, &mats, &vols, &mut g_s).unwrap();
    gamma_slice_par(&forwards, &strikes, &mats, &vols, &mut g_p).unwrap();
    vega_slice(&forwards, &strikes, &mats, &vols, &mut v_s).unwrap();
    vega_slice_par(&forwards, &strikes, &mats, &vols, &mut v_p).unwrap();
    vomma_slice(&forwards, &strikes, &mats, &vols, &mut vo_s).unwrap();
    vomma_slice_par(&forwards, &strikes, &mats, &vols, &mut vo_p).unwrap();
    vanna_slice(&forwards, &strikes, &mats, &vols, &mut va_s).unwrap();
    vanna_slice_par(&forwards, &strikes, &mats, &vols, &mut va_p).unwrap();
    assert_slices_bit_equal(&d_s, &d_p);
    assert_slices_bit_equal(&g_s, &g_p);
    assert_slices_bit_equal(&v_s, &v_p);
    assert_slices_bit_equal(&vo_s, &vo_p);
    assert_slices_bit_equal(&va_s, &va_p);
}

#[test]
fn black_scholes_par_matches_serial() {
    let n = MIN_PARALLEL + 8;
    let spots = vec![100.0_f64; n];
    let strikes: Vec<f64> = (0..n).map(|i| 95.0 + (i as f64) * 0.05).collect();
    let mats = vec![0.5_f64; n];
    let rates = [0.05_f64];
    let divs = [0.01_f64];
    let vols = [0.2_f64];
    let mut s = vec![0.0; n];
    let mut p = vec![0.0; n];
    black_scholes_price_slice(&spots, &strikes, &mats, &rates, &divs, &vols, true, &mut s).unwrap();
    black_scholes_price_slice_par(&spots, &strikes, &mats, &rates, &divs, &vols, true, &mut p)
        .unwrap();
    assert_slices_bit_equal(&s, &p);
}

#[test]
fn domain_nan_preserved_under_par() {
    let n = MIN_PARALLEL + 1;
    let forwards = vec![100.0_f64; n];
    let mut strikes = vec![100.0_f64; n];
    strikes[n / 2] = -1.0;
    let mats = vec![1.0_f64; n];
    let vols = vec![0.2_f64; n];
    let mut s = vec![0.0; n];
    let mut p = vec![0.0; n];
    black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut s).unwrap();
    black_price_slice_par(&forwards, &strikes, &mats, &vols, true, &mut p).unwrap();
    assert_slices_bit_equal(&s, &p);
    assert!(s[n / 2].is_nan());
}

#[test]
fn rayon_feature_flag_is_consistent() {
    // Integration tests run with whatever features the user selected.
    // This assertion documents the helper rather than requiring a specific build.
    let _ = rayon_enabled();
    assert!(MIN_PARALLEL >= 1);
}
