"""Pandas DataFrame helpers over the NumPy batch APIs.

These functions only extract columns, call the Rust-backed NumPy kernels, and
attach result columns. No pricing or root-finding logic lives here.

Install the optional extra to use this module::

    pip install 'iv-engine[pandas]'
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Union

import numpy as np

from iv_engine._core import (
    black_prices,
    black_scholes_prices,
    deltas,
    gammas,
    implied_volatilities,
    vegas,
)

if TYPE_CHECKING:
    import pandas as pd

FloatOrCol = Union[str, float, int]

__all__ = [
    "black_prices_frame",
    "black_scholes_prices_frame",
    "deltas_frame",
    "gammas_frame",
    "implied_volatilities_frame",
    "vegas_frame",
]


def _require_pandas() -> Any:
    try:
        import pandas as pd
    except ImportError as exc:  # pragma: no cover - exercised when pandas missing
        raise ImportError(
            "Pandas helpers require pandas. Install with: pip install 'iv-engine[pandas]'"
        ) from exc
    return pd


def _field(df: pd.DataFrame, spec: FloatOrCol, *, label: str) -> Any:
    """Return a column ndarray or a Python float for NumPy broadcast."""
    if isinstance(spec, str):
        if spec not in df.columns:
            raise KeyError(f"missing column {spec!r} for {label}")
        arr = np.ascontiguousarray(df[spec].to_numpy(dtype=np.float64, copy=False))
        if arr.ndim != 1:
            raise ValueError(f"column {spec!r} must be 1-D")
        return arr
    return float(spec)


def black_prices_frame(
    df: pd.DataFrame,
    *,
    forward: FloatOrCol = "forward",
    strike: FloatOrCol = "strike",
    maturity: FloatOrCol = "maturity",
    volatility: FloatOrCol = "volatility",
    is_call: bool = True,
    out: str = "price",
    parallel: bool = False,
) -> pd.DataFrame:
    """Add undiscounted Black-76 prices as column ``out``.

    Column specs may be a DataFrame column name or a scalar (broadcast).
    ``is_call`` is a single boolean for the whole frame.
    """
    _require_pandas()
    result = df.copy()
    prices = black_prices(
        _field(df, forward, label="forward"),
        _field(df, strike, label="strike"),
        _field(df, maturity, label="maturity"),
        _field(df, volatility, label="volatility"),
        is_call,
        parallel,
    )
    result[out] = prices
    return result


def black_scholes_prices_frame(
    df: pd.DataFrame,
    *,
    spot: FloatOrCol = "spot",
    strike: FloatOrCol = "strike",
    maturity: FloatOrCol = "maturity",
    rate: FloatOrCol = "rate",
    dividend: FloatOrCol = "dividend",
    volatility: FloatOrCol = "volatility",
    is_call: bool = True,
    out: str = "price",
    parallel: bool = False,
) -> pd.DataFrame:
    """Add discounted Black–Scholes–Merton prices as column ``out``."""
    _require_pandas()
    result = df.copy()
    prices = black_scholes_prices(
        _field(df, spot, label="spot"),
        _field(df, strike, label="strike"),
        _field(df, maturity, label="maturity"),
        _field(df, rate, label="rate"),
        _field(df, dividend, label="dividend"),
        _field(df, volatility, label="volatility"),
        is_call,
        parallel,
    )
    result[out] = prices
    return result


def implied_volatilities_frame(
    df: pd.DataFrame,
    *,
    price: FloatOrCol = "price",
    forward: FloatOrCol = "forward",
    strike: FloatOrCol = "strike",
    maturity: FloatOrCol = "maturity",
    is_call: bool = True,
    out: str = "implied_vol",
    parallel: bool = False,
) -> pd.DataFrame:
    """Add Black implied volatilities as column ``out``."""
    _require_pandas()
    result = df.copy()
    ivs = implied_volatilities(
        _field(df, price, label="price"),
        _field(df, forward, label="forward"),
        _field(df, strike, label="strike"),
        _field(df, maturity, label="maturity"),
        is_call,
        parallel,
    )
    result[out] = ivs
    return result


def deltas_frame(
    df: pd.DataFrame,
    *,
    forward: FloatOrCol = "forward",
    strike: FloatOrCol = "strike",
    maturity: FloatOrCol = "maturity",
    volatility: FloatOrCol = "volatility",
    is_call: bool = True,
    out: str = "delta",
    parallel: bool = False,
) -> pd.DataFrame:
    """Add Black-76 forward deltas as column ``out``."""
    _require_pandas()
    result = df.copy()
    result[out] = deltas(
        _field(df, forward, label="forward"),
        _field(df, strike, label="strike"),
        _field(df, maturity, label="maturity"),
        _field(df, volatility, label="volatility"),
        is_call,
        parallel,
    )
    return result


def gammas_frame(
    df: pd.DataFrame,
    *,
    forward: FloatOrCol = "forward",
    strike: FloatOrCol = "strike",
    maturity: FloatOrCol = "maturity",
    volatility: FloatOrCol = "volatility",
    out: str = "gamma",
    parallel: bool = False,
) -> pd.DataFrame:
    """Add Black-76 gammas as column ``out``."""
    _require_pandas()
    result = df.copy()
    result[out] = gammas(
        _field(df, forward, label="forward"),
        _field(df, strike, label="strike"),
        _field(df, maturity, label="maturity"),
        _field(df, volatility, label="volatility"),
        parallel,
    )
    return result


def vegas_frame(
    df: pd.DataFrame,
    *,
    forward: FloatOrCol = "forward",
    strike: FloatOrCol = "strike",
    maturity: FloatOrCol = "maturity",
    volatility: FloatOrCol = "volatility",
    out: str = "vega",
    parallel: bool = False,
) -> pd.DataFrame:
    """Add Black-76 vegas as column ``out``."""
    _require_pandas()
    result = df.copy()
    result[out] = vegas(
        _field(df, forward, label="forward"),
        _field(df, strike, label="strike"),
        _field(df, maturity, label="maturity"),
        _field(df, volatility, label="volatility"),
        parallel,
    )
    return result
