# iv_engine (Python)

PyO3 / maturin bindings for the Rust `iv-engine` numerical core.

**Phase 10** exposes scalar APIs. NumPy (Phase 11) and Pandas (Phase 12)
build on the same Rust kernels — no pricing logic lives in Python.

## Install (editable)

From the `python/` directory (requires a Rust toolchain):

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest
maturin develop
pytest
```

## Example

```python
import iv_engine as iv

price = iv.black_price(100.0, 100.0, 1.0, 0.2, True)
vol = iv.implied_volatility(price, 100.0, 100.0, 1.0, True)
assert abs(vol - 0.2) < 1e-12

try:
    iv.implied_volatility(-1.0, 100.0, 100.0, 1.0, True)
except iv.IVError as exc:
    code, message = exc.args
    assert code == "price_below_intrinsic"
```

## Errors

Domain failures raise `iv_engine.IVError` with `args == (code, message)`,
where `code` matches the Rust `IVError::code()` snake_case string.
