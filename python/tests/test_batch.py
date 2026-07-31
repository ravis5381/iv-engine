"""Batch APIs: NumPy ndarray and pandas Series (parallel by default)."""

from __future__ import annotations

import numpy as np
import pandas as pd
import pytest

import iv_engine as iv
from iv_engine.batch import (
    black_prices,
    deltas,
    gammas,
    implied_volatilities,
    vegas,
)


def test_series_roundtrip():
    df = pd.DataFrame(
        {
            "forward": np.full(5, 100.0),
            "strike": np.linspace(90.0, 110.0, 5),
            "maturity": np.full(5, 1.0),
            "volatility": np.full(5, 0.2),
        }
    )
    price = black_prices(
        forward=df["forward"],
        strike=df["strike"],
        maturity=df["maturity"],
        volatility=df["volatility"],
        is_call=True,
    )
    assert isinstance(price, pd.Series)
    assert price.name == "price"
    assert price.index.equals(df.index)

    ivs = implied_volatilities(
        price=price,
        forward=df["forward"],
        strike=df["strike"],
        maturity=df["maturity"],
        is_call=True,
    )
    np.testing.assert_allclose(ivs, 0.2, atol=1e-12)


def test_scalar_broadcast_series():
    strike = pd.Series([80.0, 100.0, 120.0], name="strike")
    mid = black_prices(
        forward=100.0,
        strike=strike,
        maturity=1.0,
        volatility=0.25,
        is_call=True,
        name="mid",
    )
    assert mid.name == "mid"
    np.testing.assert_allclose(
        mid.to_numpy(),
        iv.black_prices(100.0, strike.to_numpy(), 1.0, 0.25, True),
    )


def test_greeks_series():
    forward = pd.Series([100.0, 100.0])
    strike = pd.Series([100.0, 110.0])
    maturity = pd.Series([1.0, 1.0])
    volatility = pd.Series([0.2, 0.2])
    vega = vegas(forward, strike, maturity, volatility)
    gamma = gammas(forward, strike, maturity, volatility, name="g")
    delta = deltas(forward, strike, maturity, volatility, name="d")
    assert isinstance(vega, pd.Series)
    assert (gamma > 0).all()
    assert (vega > 0).all()
    assert delta.name == "d"


def test_numpy_still_returns_ndarray():
    strikes = np.linspace(90.0, 110.0, 5)
    prices = black_prices(100.0, strikes, 1.0, 0.2, True)
    assert isinstance(prices, np.ndarray)
    assert not isinstance(prices, pd.Series)


def test_index_mismatch():
    a = pd.Series([100.0, 100.0], index=[0, 1])
    b = pd.Series([100.0, 110.0], index=[1, 2])
    with pytest.raises(ValueError, match="same index"):
        black_prices(a, b, 1.0, 0.2)


def test_column_name_rejected():
    with pytest.raises(TypeError, match="column name"):
        black_prices("forward", 100.0, 1.0, 0.2)


def test_parallel_default_matches_explicit():
    strike = pd.Series(np.linspace(80.0, 120.0, 200))
    default = black_prices(100.0, strike, 1.0, 0.2)
    parallel = black_prices(100.0, strike, 1.0, 0.2, parallel=True)
    serial = black_prices(100.0, strike, 1.0, 0.2, parallel=False)
    np.testing.assert_array_equal(default.to_numpy(), parallel.to_numpy())
    np.testing.assert_array_equal(serial.to_numpy(), parallel.to_numpy())


def test_package_reexports():
    assert iv.black_prices is black_prices
    assert iv.vegas is vegas
    assert iv.implied_volatilities is implied_volatilities
