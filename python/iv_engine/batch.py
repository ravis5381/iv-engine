"""Batch APIs: NumPy ``ndarray`` or pandas ``Series``.

Functions like ``deltas``, ``vegas``, ``black_prices`` accept scalars,
1-D ``float64`` arrays, or pandas ``Series``. If any argument is a ``Series``,
the result is a ``Series`` with that index; otherwise an ``ndarray``.

Parallel Rayon evaluation is **on by default** (``parallel=True``). Pass
``parallel=False`` to force serial. Small batches may still run serially
inside Rust when below the parallel threshold.

::

    df["vega"] = vegas(df["forward"], df["strike"], df["maturity"], df["volatility"])
    prices = black_prices(100.0, strikes_array, 1.0, 0.2)

No pricing logic lives here — all numerics go to ``iv_engine._core``.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional, Sequence, Union

import numpy as np

from iv_engine._core import (
    black_prices as _black_prices,
    black_scholes_prices as _black_scholes_prices,
    deltas as _deltas,
    gammas as _gammas,
    implied_volatilities as _implied_volatilities,
    vegas as _vegas,
)

if TYPE_CHECKING:
    import pandas as pd

FloatOrArray = Union[float, int, "pd.Series", np.ndarray]

__all__ = [
    "black_prices",
    "black_scholes_prices",
    "deltas",
    "gammas",
    "implied_volatilities",
    "vegas",
]


def _require_pandas() -> Any:
    try:
        import pandas as pd
    except ImportError as exc:  # pragma: no cover
        raise ImportError(
            "pandas Series inputs require pandas. Install with: pip install 'iv-engine[pandas]'"
        ) from exc
    return pd


def _is_series(value: Any) -> bool:
    try:
        import pandas as pd
    except ImportError:
        return False
    return isinstance(value, pd.Series)


def _field(
    value: FloatOrArray,
    *,
    label: str,
) -> tuple[Any, Optional[Any]]:
    """Return ``(numpy_arg, index_or_None)``."""
    if isinstance(value, (float, int)) and not isinstance(value, bool):
        return float(value), None
    if isinstance(value, str):
        raise TypeError(
            f"{label} must be a pandas Series, ndarray, or scalar float; "
            f"got column name {value!r}. Pass the column explicitly, e.g. df[{value!r}]."
        )
    if _is_series(value):
        arr = np.ascontiguousarray(value.to_numpy(dtype=np.float64, copy=False))
        if arr.ndim != 1:
            raise ValueError(f"{label} Series must be 1-D")
        return arr, value.index
    arr = np.ascontiguousarray(np.asarray(value, dtype=np.float64))
    if arr.ndim != 1:
        raise ValueError(f"{label} must be 1-D")
    return arr, None


def _combine_index(indexes: Sequence[Optional[Any]], n: int) -> Any:
    pd = _require_pandas()
    chosen = None
    for idx in indexes:
        if idx is None:
            continue
        if chosen is None:
            chosen = idx
        elif not chosen.equals(idx):
            raise ValueError("all Series inputs must share the same index")
    if chosen is None:
        return pd.RangeIndex(n)
    if len(chosen) != n:
        raise ValueError(
            f"result length {n} does not match Series index length {len(chosen)}"
        )
    return chosen


def _series_from(values: np.ndarray, indexes: Sequence[Optional[Any]], *, name: str) -> Any:
    pd = _require_pandas()
    return pd.Series(values, index=_combine_index(indexes, len(values)), name=name)


def _pack(*labeled: tuple[str, FloatOrArray]) -> tuple[list[Any], list[Optional[Any]], bool]:
    args: list[Any] = []
    indexes: list[Optional[Any]] = []
    any_series = False
    for label, value in labeled:
        arr, idx = _field(value, label=label)
        args.append(arr)
        indexes.append(idx)
        any_series = any_series or idx is not None
    return args, indexes, any_series


def black_prices(
    forward: FloatOrArray,
    strike: FloatOrArray,
    maturity: FloatOrArray,
    volatility: FloatOrArray,
    is_call: bool = True,
    parallel: bool = True,
    *,
    name: str = "price",
) -> Any:
    """Undiscounted Black-76 prices (ndarray or Series). Parallel by default."""
    (f, k, t, v), indexes, as_series = _pack(
        ("forward", forward),
        ("strike", strike),
        ("maturity", maturity),
        ("volatility", volatility),
    )
    prices = _black_prices(f, k, t, v, is_call, parallel)
    if as_series:
        return _series_from(prices, indexes, name=name)
    return prices


def black_scholes_prices(
    spot: FloatOrArray,
    strike: FloatOrArray,
    maturity: FloatOrArray,
    rate: FloatOrArray,
    dividend: FloatOrArray,
    volatility: FloatOrArray,
    is_call: bool = True,
    parallel: bool = True,
    *,
    name: str = "price",
) -> Any:
    """Discounted Black–Scholes–Merton prices (ndarray or Series). Parallel by default."""
    (s, k, t, r, q, v), indexes, as_series = _pack(
        ("spot", spot),
        ("strike", strike),
        ("maturity", maturity),
        ("rate", rate),
        ("dividend", dividend),
        ("volatility", volatility),
    )
    prices = _black_scholes_prices(s, k, t, r, q, v, is_call, parallel)
    if as_series:
        return _series_from(prices, indexes, name=name)
    return prices


def implied_volatilities(
    price: FloatOrArray,
    forward: FloatOrArray,
    strike: FloatOrArray,
    maturity: FloatOrArray,
    is_call: bool = True,
    parallel: bool = True,
    *,
    name: str = "implied_vol",
) -> Any:
    """Black implied volatilities (ndarray or Series). Parallel by default."""
    (p, f, k, t), indexes, as_series = _pack(
        ("price", price),
        ("forward", forward),
        ("strike", strike),
        ("maturity", maturity),
    )
    ivs = _implied_volatilities(p, f, k, t, is_call, parallel)
    if as_series:
        return _series_from(ivs, indexes, name=name)
    return ivs


def deltas(
    forward: FloatOrArray,
    strike: FloatOrArray,
    maturity: FloatOrArray,
    volatility: FloatOrArray,
    is_call: bool = True,
    parallel: bool = True,
    *,
    name: str = "delta",
) -> Any:
    """Black-76 forward deltas (ndarray or Series). Parallel by default."""
    (f, k, t, v), indexes, as_series = _pack(
        ("forward", forward),
        ("strike", strike),
        ("maturity", maturity),
        ("volatility", volatility),
    )
    out = _deltas(f, k, t, v, is_call, parallel)
    if as_series:
        return _series_from(out, indexes, name=name)
    return out


def gammas(
    forward: FloatOrArray,
    strike: FloatOrArray,
    maturity: FloatOrArray,
    volatility: FloatOrArray,
    parallel: bool = True,
    *,
    name: str = "gamma",
) -> Any:
    """Black-76 gammas (ndarray or Series). Parallel by default."""
    (f, k, t, v), indexes, as_series = _pack(
        ("forward", forward),
        ("strike", strike),
        ("maturity", maturity),
        ("volatility", volatility),
    )
    out = _gammas(f, k, t, v, parallel)
    if as_series:
        return _series_from(out, indexes, name=name)
    return out


def vegas(
    forward: FloatOrArray,
    strike: FloatOrArray,
    maturity: FloatOrArray,
    volatility: FloatOrArray,
    parallel: bool = True,
    *,
    name: str = "vega",
) -> Any:
    """Black-76 vegas (ndarray or Series). Parallel by default."""
    (f, k, t, v), indexes, as_series = _pack(
        ("forward", forward),
        ("strike", strike),
        ("maturity", maturity),
        ("volatility", volatility),
    )
    out = _vegas(f, k, t, v, parallel)
    if as_series:
        return _series_from(out, indexes, name=name)
    return out
