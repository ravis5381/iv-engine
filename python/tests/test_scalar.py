"""Phase 10: scalar Python bindings against the Rust core."""

from __future__ import annotations

import math

import pytest

import iv_engine as iv


def test_version():
    assert iv.__version__ == "0.1.1"


def test_norm_pdf_at_zero():
    expected = 1.0 / math.sqrt(2.0 * math.pi)
    assert abs(iv.norm_pdf(0.0) - expected) < 1e-15


def test_black_atm_roundtrip():
    price = iv.black_price(100.0, 100.0, 1.0, 0.2, True)
    assert price > 0.0
    vol = iv.implied_volatility(price, 100.0, 100.0, 1.0, True)
    assert abs(vol - 0.2) < 1e-12


def test_black_scholes_positive():
    px = iv.black_scholes_price(100.0, 100.0, 1.0, 0.05, 0.02, 0.2, True)
    assert px > 0.0


def test_greeks_smoke():
    assert iv.delta(100.0, 100.0, 1.0, 0.2, True) > 0.0
    assert iv.gamma(100.0, 100.0, 1.0, 0.2) > 0.0
    assert iv.vega(100.0, 100.0, 1.0, 0.2) > 0.0


def test_iv_error_code():
    with pytest.raises(iv.IVError) as excinfo:
        iv.implied_volatility(-1.0, 100.0, 100.0, 1.0, True)
    code, message = excinfo.value.args
    assert code == "price_below_intrinsic"
    assert "intrinsic" in message


def test_negative_strike():
    with pytest.raises(iv.IVError) as excinfo:
        iv.black_price(100.0, -1.0, 1.0, 0.2, True)
    assert excinfo.value.args[0] == "negative_strike"


def test_put_call_parity_iv():
    f, k, t, sig = 100.0, 110.0, 1.0, 0.25
    call = iv.black_price(f, k, t, sig, True)
    put = iv.black_price(f, k, t, sig, False)
    # Undiscounted parity: C - P = F - K
    assert abs((call - put) - (f - k)) < 1e-12
    assert abs(iv.implied_volatility(call, f, k, t, True) - sig) < 1e-12
    assert abs(iv.implied_volatility(put, f, k, t, False) - sig) < 1e-12
