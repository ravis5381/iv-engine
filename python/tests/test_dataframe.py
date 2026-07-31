"""Phase 12: Pandas DataFrame helpers."""

from __future__ import annotations

import numpy as np
import pandas as pd
import pytest

import iv_engine as iv
from iv_engine.dataframe import (
    black_prices_frame,
    deltas_frame,
    gammas_frame,
    implied_volatilities_frame,
    vegas_frame,
)


def test_black_prices_frame_roundtrip():
    df = pd.DataFrame(
        {
            "forward": np.full(5, 100.0),
            "strike": np.linspace(90.0, 110.0, 5),
            "maturity": np.full(5, 1.0),
            "volatility": np.full(5, 0.2),
        }
    )
    priced = black_prices_frame(df, is_call=True)
    assert "price" in priced.columns
    assert len(priced) == 5
    # Original frame unchanged.
    assert "price" not in df.columns

    with_iv = implied_volatilities_frame(priced, is_call=True)
    np.testing.assert_allclose(with_iv["implied_vol"], 0.2, atol=1e-12)


def test_scalar_column_broadcast():
    df = pd.DataFrame({"strike": [80.0, 100.0, 120.0]})
    priced = black_prices_frame(
        df,
        forward=100.0,
        strike="strike",
        maturity=1.0,
        volatility=0.25,
        is_call=True,
        out="mid",
    )
    assert list(priced["mid"]) == list(
        iv.black_prices(100.0, priced["strike"].to_numpy(), 1.0, 0.25, True)
    )


def test_greeks_frames():
    df = pd.DataFrame(
        {
            "forward": [100.0, 100.0],
            "strike": [100.0, 110.0],
            "maturity": [1.0, 1.0],
            "volatility": [0.2, 0.2],
        }
    )
    out = deltas_frame(gammas_frame(vegas_frame(df), out="g"), out="d")
    assert {"vega", "g", "d"} <= set(out.columns)
    assert (out["g"] > 0).all()
    assert (out["vega"] > 0).all()


def test_missing_column():
    df = pd.DataFrame({"strike": [100.0]})
    with pytest.raises(KeyError, match="forward"):
        black_prices_frame(df)


def test_package_reexports():
    assert iv.black_prices_frame is black_prices_frame
    assert iv.implied_volatilities_frame is implied_volatilities_frame
