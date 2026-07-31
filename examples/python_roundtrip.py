#!/usr/bin/env python3
"""Scalar + NumPy + Pandas round-trip demo.

Run from a venv with the package installed::

    cd python && maturin develop && python ../examples/python_roundtrip.py
"""

from __future__ import annotations

import numpy as np
import pandas as pd

import iv_engine as iv


def main() -> None:
    # Scalar
    price = iv.black_price(100.0, 100.0, 1.0, 0.2, True)
    vol = iv.implied_volatility(price, 100.0, 100.0, 1.0, True)
    print(f"scalar: price={price:.12f}  iv={vol:.12f}")

    # NumPy
    strikes = np.linspace(80.0, 120.0, 9)
    prices = iv.black_prices(100.0, strikes, 1.0, 0.25)
    ivs = iv.implied_volatilities(prices, 100.0, strikes, 1.0)
    print(f"numpy:  max |iv-0.25| = {np.max(np.abs(ivs - 0.25)):.3e}")

    # Pandas
    df = pd.DataFrame({"strike": strikes})
    priced = iv.black_prices_frame(
        df, forward=100.0, maturity=1.0, volatility=0.25, is_call=True
    )
    recovered = iv.implied_volatilities_frame(
        priced, forward=100.0, maturity=1.0, is_call=True
    )
    print(recovered[["strike", "price", "implied_vol"]].to_string(index=False))


if __name__ == "__main__":
    main()
