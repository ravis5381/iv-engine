//! Phase 6: validation against Peter Jäckel's Let's Be Rational C++ reference.
//!
//! Golden values were generated with the vollib/jaeckel.org reference sources
//! (`LetsBeRational.cpp`, compiled locally) for identical market inputs.
//!
//! Two complementary checks:
//! 1. **Reference IV match** — invert the reference Black price with our solver
//!    and compare σ to the reference implied volatility.
//! 2. **Round-trip** — `Black(IV(Black(σ))) ≈ Black(σ)` on a dense grid using
//!    *our* pricing kernel (machine-precision recovery contract).

#![allow(clippy::excessive_precision)] // C++ `%.17g` goldens; keep exact decimals.

use iv_engine::{
    black_price, implied_volatility, normalised_implied_volatility, PRICE_ROUNDTRIP_ABS,
    PRICE_ROUNDTRIP_TOL,
};

/// `(F, K, T, σ_true, is_call, ref_price, ref_iv)` from the C++ harness.
const GOLDEN_IV: &[(f64, f64, f64, f64, bool, f64, f64)] = &[
    (100.0, 100.0, 1.0, 0.2, true, 7.965_567_455_405_797_6, 0.2),
    (100.0, 100.0, 1.0, 0.2, false, 7.965_567_455_405_797_6, 0.2),
    (100.0, 90.0, 1.0, 0.25, true, 15.272_057_641_846_081, 0.25),
    (
        100.0,
        110.0,
        1.0,
        0.25,
        true,
        6.190_426_413_768_345_3,
        0.249_999_999_999_999_97,
    ),
    (
        100.0,
        80.0,
        0.5,
        0.35,
        true,
        22.206_096_552_663_876,
        0.350_000_000_000_000_03,
    ),
    (100.0, 120.0, 2.0, 0.15, true, 2.503_775_208_732_238_7, 0.15),
    // Deep ITM / tiny time value: reference returns 0 (price == intrinsic).
    (100.0, 50.0, 0.25, 0.10, true, 50.0, 0.0),
    (
        100.0,
        200.0,
        0.25,
        0.10,
        true,
        2.680_842_079_928_599_4e-44,
        0.10,
    ),
    (
        100.0,
        70.0,
        0.01,
        0.80,
        false,
        5.685_998_509_055_350_9e-6,
        0.800_000_000_000_000_3,
    ),
    (
        100.0,
        130.0,
        10.0,
        1.5,
        true,
        97.983_646_409_824_23,
        1.500_000_000_000_000_4,
    ),
    (
        100.0,
        100.0,
        5.0,
        0.02,
        false,
        1.783_975_450_293_204_4,
        0.02,
    ),
    (
        100.0,
        150.0,
        0.05,
        0.60,
        false,
        50.005_794_111_542_997,
        0.600_000_000_000_017_5,
    ),
    (100.0, 100.0, 1e-4, 0.5, true, 0.199_470_932_418_473_44, 0.5),
    // Near-intrinsic ITM: reference returns 0.
    (100.0, 99.0, 1.0, 1e-4, true, 1.0, 0.0),
    (50.0, 100.0, 1.0, 0.4, true, 0.470_086_504_172_107_79, 0.4),
    (
        200.0,
        100.0,
        0.75,
        0.3,
        false,
        0.042_932_450_599_144_932,
        0.3,
    ),
];

/// `(x, s_true, beta, ref_s)` for normalised call inversion.
const GOLDEN_NORM: &[(f64, f64, f64, f64)] = &[
    (0.0, 0.05, 0.019_945_036_390_476_085, 0.05),
    (0.0, 0.1, 0.039_877_611_676_744_931, 0.1),
    (0.0, 0.2, 0.079_655_674_554_057_976, 0.2),
    (0.0, 0.5, 0.197_412_651_365_847_4, 0.5),
    (0.0, 1.0, 0.382_924_922_548_026_29, 1.0),
    (0.0, 2.0, 0.682_689_492_137_086_1, 2.0),
    (-0.1, 0.05, 0.000_424_430_830_851_187_06, 0.05),
    (-0.1, 0.1, 0.008_324_939_376_702_142, 0.1),
    (-0.1, 0.2, 0.039_458_602_641_807_006, 0.2),
    (-0.1, 0.5, 0.151_493_280_813_103_01, 0.5),
    (-0.1, 1.0, 0.335_142_070_295_581_74, 1.0),
    (-0.1, 2.0, 0.634_127_122_705_938_96, 2.0),
    (-0.5, 0.2, 0.000_399_163_502_916_059_78, 0.2),
    (-0.5, 0.5, 0.040_840_564_840_954_753, 0.5),
    (-0.5, 1.0, 0.185_683_012_996_663_9, 1.0),
    (-0.5, 2.0, 0.466_646_228_919_351_37, 2.0),
    (-1.0, 0.5, 0.004_142_358_750_678_573_6, 0.5),
    (-1.0, 1.0, 0.076_991_023_141_674_15, 1.0),
    (-1.0, 2.0, 0.309_246_729_035_137_61, 2.0),
    (-2.0, 0.5, 3.472_476_082_547_400_6e-6, 0.5),
    (-2.0, 1.0, 0.007_697_375_453_146_090_3, 1.0),
    (-2.0, 2.0, 0.122_098_450_315_940_05, 2.0),
    (-5.0, 1.0, 4.755_776_371_084_642_9e-8, 1.0),
    (-5.0, 2.0, 0.002_649_866_673_786_449_2, 2.0),
];

#[test]
fn matches_reference_implied_volatility() {
    for &(f, k, t, _sig, is_call, ref_price, ref_iv) in GOLDEN_IV {
        // Skip denormal / exact-intrinsic cases where both sides are 0.
        if ref_price == 0.0 {
            continue;
        }
        let got = implied_volatility(ref_price, f, k, t, is_call);
        if ref_iv == 0.0 {
            // Intrinsic-locked: accept Ok(0) or a tiny residual σ that
            // re-prices within a few ulps of intrinsic.
            if let Ok(iv) = got {
                assert!(
                    iv.abs() < 1e-8,
                    "expected ~0 IV for intrinsic price, got {iv} (F={f} K={k})"
                );
            }
            // Err: price may sit on the intrinsic boundary within FP noise.
            continue;
        }
        let iv = got
            .unwrap_or_else(|e| panic!("IV failed for F={f} K={k} T={t} price={ref_price}: {e}"));
        let abs_err = (iv - ref_iv).abs();
        let rel_err = abs_err / ref_iv.abs().max(1e-300);
        assert!(
            abs_err < 1e-12 || rel_err < 1e-12,
            "ref IV mismatch F={f} K={k} T={t} call={is_call}: got={iv} ref={ref_iv} abs={abs_err} rel={rel_err}"
        );
    }
}

#[test]
fn matches_reference_normalised_implied_volatility() {
    for &(x, _s, beta, ref_s) in GOLDEN_NORM {
        if beta == 0.0 || ref_s == 0.0 {
            continue;
        }
        let got = normalised_implied_volatility(beta, x, true)
            .unwrap_or_else(|e| panic!("normalised IV failed x={x} beta={beta}: {e}"));
        let abs_err = (got - ref_s).abs();
        let rel_err = abs_err / ref_s.abs().max(1e-300);
        assert!(
            abs_err < 1e-12 || rel_err < 1e-12,
            "normalised mismatch x={x} beta={beta}: got={got} ref={ref_s} abs={abs_err} rel={rel_err}"
        );
    }
}

#[test]
fn dense_roundtrip_grid() {
    let forwards = [50.0_f64, 100.0, 150.0];
    let strikes = [60.0, 80.0, 100.0, 120.0, 140.0];
    let mats = [1.0 / 365.0, 0.05, 0.25, 1.0, 5.0];
    let vols = [0.05, 0.1, 0.2, 0.5, 1.0];
    let mut checked = 0u32;

    for &f in &forwards {
        for &k in &strikes {
            for &t in &mats {
                for &v in &vols {
                    for &is_call in &[true, false] {
                        let Ok(price) = black_price(f, k, t, v, is_call) else {
                            continue;
                        };
                        // Skip near-intrinsic / near-max where IV is discontinuous.
                        let intrinsic = if is_call {
                            (f - k).max(0.0)
                        } else {
                            (k - f).max(0.0)
                        };
                        let maximum = if is_call { f } else { k };
                        if price <= intrinsic + 1e-14 || price >= maximum - 1e-14 {
                            continue;
                        }
                        let iv = implied_volatility(price, f, k, t, is_call).unwrap_or_else(|e| {
                            panic!(
                                "IV failed F={f} K={k} T={t} σ={v} call={is_call} px={price}: {e}"
                            )
                        });
                        let repriced = black_price(f, k, t, iv, is_call).unwrap();
                        let tol = PRICE_ROUNDTRIP_ABS + PRICE_ROUNDTRIP_TOL * price.abs().max(1.0);
                        // Allow a slightly looser budget on extreme shorts / deep OTM.
                        let tol = tol.max(1e-12 * (1.0 + price));
                        assert!(
                            (repriced - price).abs() <= tol,
                            "round-trip F={f} K={k} T={t} σ={v} call={is_call}: px={price} iv={iv} re={repriced} tol={tol}"
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(
        checked > 200,
        "expected a dense grid, only checked {checked}"
    );
}

#[test]
fn atm_reference_price_matches_our_black() {
    // Cross-check that our Black ATM price agrees with the reference golden.
    let ours = black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
    let reference = 7.965_567_455_405_797_6;
    assert!(
        (ours - reference).abs() < 1e-12,
        "ATM Black drift vs reference: ours={ours} ref={reference}"
    );
}

/// Regression: when `s_true == sqrt(2|x|)` the guess sits on the `s_c`
/// bracket edge. Householder must early-exit on a zero step rather than
/// binary-nesting away from the root (C++ `fabs(ds) > DBL_EPSILON*s`).
#[test]
fn critical_branch_boundary_recovers_exactly() {
    for &(x, s_true, beta, ref_s) in &[
        (-0.5, 1.0, 0.185_683_012_996_663_9, 1.0),
        (-2.0, 2.0, 0.122_098_450_315_940_05, 2.0),
    ] {
        let got = normalised_implied_volatility(beta, x, true).unwrap();
        assert!(
            (got - ref_s).abs() < 1e-14,
            "s_c-boundary x={x} s={s_true}: got={got} ref={ref_s}"
        );
    }
}
