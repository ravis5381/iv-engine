# iv_engine (Python)

PyO3 / maturin bindings for the Rust `iv-engine` numerical core.

- **Phase 10** — scalar APIs
- **Phase 11** — NumPy batch APIs (`black_prices`, `implied_volatilities`, …)
- **Phase 12** — Pandas helpers (pending)

No pricing logic lives in Python.

## Install (editable)

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest numpy
maturin develop
pytest
```

## Scalar example

```python
import iv_engine as iv

price = iv.black_price(100.0, 100.0, 1.0, 0.2, True)
vol = iv.implied_volatility(price, 100.0, 100.0, 1.0, True)
assert abs(vol - 0.2) < 1e-12
```

## NumPy example

```python
import numpy as np
import iv_engine as iv

strikes = np.linspace(80.0, 120.0, 41)
prices = iv.black_prices(100.0, strikes, 1.0, 0.25)          # scalar F,T,σ broadcast
ivs = iv.implied_volatilities(prices, 100.0, strikes, 1.0)
assert np.allclose(ivs, 0.25, atol=1e-12)

# Optional Rayon path for large batches
_ = iv.black_prices(100.0, strikes, 1.0, 0.25, parallel=True)
```

Inputs may be Python `float` (length-1 broadcast) or contiguous 1-D `float64`
arrays. Per-element domain failures write `NaN` (same contract as Rust
`vector`). Length mismatches raise `IVError`.

## Errors

Domain / length failures raise `iv_engine.IVError` with
`args == (code, message)`.
