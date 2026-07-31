//! Phase 9: optional SIMD / batch Normal kernels.

use iv_engine::{
    norm_cdf, norm_cdf_c, norm_cdf_c_slice, norm_cdf_slice, norm_pdf, norm_pdf_slice, simd_enabled,
    SIMD_LANES,
};

fn assert_close(got: f64, expected: f64, label: &str) {
    if expected.is_nan() {
        assert!(got.is_nan(), "{label}: expected NaN, got {got}");
        return;
    }
    if expected == 0.0 {
        assert!(got.abs() < 1e-15, "{label}: got={got} expected=0");
        return;
    }
    let abs = (got - expected).abs();
    let rel = abs / expected.abs();
    assert!(
        abs < 1e-14 || rel < 1e-14,
        "{label}: got={got} expected={expected} abs={abs} rel={rel}"
    );
}

#[test]
fn pdf_slice_matches_scalar() {
    let xs: Vec<f64> = (-40..=40).map(|i| f64::from(i) * 0.25).collect();
    let mut out = vec![0.0; xs.len()];
    norm_pdf_slice(&xs, &mut out).unwrap();
    for (i, &x) in xs.iter().enumerate() {
        assert_close(out[i], norm_pdf(x), &format!("pdf x={x}"));
    }
}

#[test]
fn pdf_slice_handles_specials_and_remainder() {
    // Length not a multiple of SIMD_LANES exercises the scalar tail.
    let xs = [
        0.0,
        1.0,
        -1.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        2.5,
    ];
    assert_eq!(xs.len() % SIMD_LANES, 3);
    let mut out = [0.0; 7];
    norm_pdf_slice(&xs, &mut out).unwrap();
    for (i, &x) in xs.iter().enumerate() {
        assert_close(out[i], norm_pdf(x), &format!("special[{i}]"));
    }
}

#[test]
fn cdf_slices_are_bit_identical() {
    let xs: Vec<f64> = (-20..=20)
        .map(|i| f64::from(i) * 0.5)
        .chain([f64::INFINITY, f64::NEG_INFINITY, f64::NAN])
        .collect();
    let mut cdf = vec![0.0; xs.len()];
    let mut cdf_c = vec![0.0; xs.len()];
    norm_cdf_slice(&xs, &mut cdf).unwrap();
    norm_cdf_c_slice(&xs, &mut cdf_c).unwrap();
    for (i, &x) in xs.iter().enumerate() {
        let e = norm_cdf(x);
        let ec = norm_cdf_c(x);
        assert!(
            cdf[i].to_bits() == e.to_bits() || (cdf[i].is_nan() && e.is_nan()),
            "cdf mismatch at {i}"
        );
        assert!(
            cdf_c[i].to_bits() == ec.to_bits() || (cdf_c[i].is_nan() && ec.is_nan()),
            "cdf_c mismatch at {i}"
        );
    }
}

#[test]
fn length_mismatch_errors() {
    let mut out = [0.0; 2];
    assert!(norm_pdf_slice(&[0.0], &mut out).is_err());
}

#[test]
fn simd_feature_helper() {
    let _ = simd_enabled();
    assert_eq!(SIMD_LANES, 4);
}
