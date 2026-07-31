//! Integration tests for the Standard Normal PDF / CDF (Phase 2).

use iv_engine::normal::{norm_cdf, norm_cdf_c, norm_pdf};
use iv_engine::{DBL_EPSILON, ONE_OVER_SQRT_TWO_PI};

#[test]
fn public_reexports_usable_with_normal() {
    assert!((norm_pdf(0.0) - ONE_OVER_SQRT_TWO_PI).abs() < DBL_EPSILON);
}

#[test]
fn black_style_deep_otm_tail() {
    // Typical deep-OTM d₂ ≈ +6 … +10: survival must stay positive and
    // match the left-tail CDF.
    for &d in &[6.0, 7.0, 8.0, 9.0, 10.0] {
        let right = norm_cdf_c(d);
        let left = norm_cdf(-d);
        assert!(right > 0.0, "survival flushed at d={d}");
        assert!((right - left).abs() < 8.0 * DBL_EPSILON);
    }
}

#[test]
fn pdf_integrates_to_one_roughly() {
    // Crude trapezoidal check on [-8, 8]; not a proof, just a sanity net.
    let n = 80_000;
    let a = -8.0;
    let b = 8.0;
    let h = (b - a) / f64::from(n);
    let mut acc = 0.5 * (norm_pdf(a) + norm_pdf(b));
    for i in 1..n {
        let x = a + f64::from(i) * h;
        acc += norm_pdf(x);
    }
    let integral = acc * h;
    assert!(
        (integral - 1.0).abs() < 1e-6,
        "∫φ ≈ {integral}, expected ≈ 1"
    );
}

#[test]
fn cdf_bounds() {
    for &x in &[-100.0, -1.0, 0.0, 1.0, 100.0] {
        let p = norm_cdf(x);
        assert!((0.0..=1.0).contains(&p) || p.is_nan());
        let q = norm_cdf_c(x);
        assert!((0.0..=1.0).contains(&q) || q.is_nan());
    }
}
