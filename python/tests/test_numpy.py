"""Phase 11: NumPy batch APIs."""

from __future__ import annotations

import numpy as np
import pytest

import iv_engine as iv


def test_norm_pdfs_matches_scalar():
    xs = np.linspace(-3.0, 3.0, 21)
    got = iv.norm_pdfs(xs)
    expected = np.array([iv.norm_pdf(float(x)) for x in xs])
    np.testing.assert_allclose(got, expected, rtol=0, atol=1e-15)


def test_scalar_broadcast_strikes():
    strikes = np.array([80.0, 100.0, 120.0])
    prices = iv.black_prices(100.0, strikes, 1.0, 0.2, is_call=True)
    assert prices.shape == (3,)
    for i, k in enumerate(strikes):
        assert abs(prices[i] - iv.black_price(100.0, float(k), 1.0, 0.2, True)) < 1e-15


def test_implied_volatilities_roundtrip():
    strikes = np.linspace(80.0, 120.0, 41)
    forwards = 100.0
    mats = 1.0
    vols = 0.25
    prices = iv.black_prices(forwards, strikes, mats, vols, True)
    ivs = iv.implied_volatilities(prices, forwards, strikes, mats, True)
    np.testing.assert_allclose(ivs, 0.25, rtol=0, atol=1e-12)


def test_parallel_matches_serial():
    n = 128
    strikes = np.linspace(70.0, 130.0, n)
    serial = iv.black_prices(100.0, strikes, 1.0, 0.2, True, parallel=False)
    parallel = iv.black_prices(100.0, strikes, 1.0, 0.2, True, parallel=True)
    np.testing.assert_array_equal(serial, parallel)


def test_domain_nan_in_batch():
    strikes = np.array([100.0, -1.0, 110.0])
    prices = iv.black_prices(100.0, strikes, 1.0, 0.2, True)
    assert np.isfinite(prices[0])
    assert np.isnan(prices[1])
    assert np.isfinite(prices[2])


def test_length_mismatch_raises():
    with pytest.raises(iv.IVError) as excinfo:
        iv.black_prices(
            np.array([100.0, 101.0]),
            np.array([100.0]),
            np.array([1.0, 1.0, 1.0]),
            0.2,
            True,
        )
    assert excinfo.value.args[0] == "invalid_input"


def test_greeks_batch():
    strikes = np.array([90.0, 100.0, 110.0])
    d = iv.deltas(100.0, strikes, 1.0, 0.2, True)
    g = iv.gammas(100.0, strikes, 1.0, 0.2)
    v = iv.vegas(100.0, strikes, 1.0, 0.2)
    vo = iv.vommas(100.0, strikes, 1.0, 0.2)
    va = iv.vannas(100.0, strikes, 1.0, 0.2)
    assert d.shape == g.shape == v.shape == vo.shape == va.shape == (3,)
    assert np.all(g > 0.0)
    assert np.all(v > 0.0)
    for i, k in enumerate(strikes):
        assert abs(vo[i] - iv.vomma(100.0, float(k), 1.0, 0.2)) < 1e-14
        assert abs(va[i] - iv.vanna(100.0, float(k), 1.0, 0.2)) < 1e-14


def test_black_scholes_prices_batch():
    spots = np.full(5, 100.0)
    strikes = np.linspace(95.0, 105.0, 5)
    px = iv.black_scholes_prices(spots, strikes, 0.5, 0.05, 0.02, 0.2, True)
    assert px.shape == (5,)
    assert np.all(px > 0.0)
