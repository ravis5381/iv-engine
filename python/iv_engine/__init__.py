"""iv_engine — Python bindings for the Rust numerical core.

All numerical work happens in Rust. This package only re-exports the
compiled extension [`iv_engine._core`]. NumPy and Pandas helpers arrive in
Phases 11–12.
"""

from __future__ import annotations

from iv_engine._core import (
    IVError,
    __version__,
    black_intrinsic,
    black_price,
    black_price_total_vol,
    black_scholes_delta,
    black_scholes_forward,
    black_scholes_gamma,
    black_scholes_price,
    black_scholes_theta,
    black_scholes_vanna,
    black_scholes_vega,
    black_scholes_vomma,
    delta,
    gamma,
    implied_volatility,
    log_moneyness,
    norm_cdf,
    norm_cdf_c,
    norm_pdf,
    normalised_implied_volatility,
    theta,
    vanna,
    vega,
    vomma,
)

__all__ = [
    "IVError",
    "__version__",
    "black_intrinsic",
    "black_price",
    "black_price_total_vol",
    "black_scholes_delta",
    "black_scholes_forward",
    "black_scholes_gamma",
    "black_scholes_price",
    "black_scholes_theta",
    "black_scholes_vanna",
    "black_scholes_vega",
    "black_scholes_vomma",
    "delta",
    "gamma",
    "implied_volatility",
    "log_moneyness",
    "norm_cdf",
    "norm_cdf_c",
    "norm_pdf",
    "normalised_implied_volatility",
    "theta",
    "vanna",
    "vega",
    "vomma",
]
