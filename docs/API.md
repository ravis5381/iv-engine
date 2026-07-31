# API overview

The Rust crate `iv-engine` is the numerical source of truth. Python wraps the
same functions; it never reimplements pricing or implied-volatility logic.

## Rust

| Area | Entry points |
|------|----------------|
| Normal | `norm_pdf`, `norm_cdf`, `norm_cdf_c` |
| Black-76 | `black_price`, `black_intrinsic`, `log_moneyness` |
| Black–Scholes | `black_scholes_price`, `black_scholes_forward` |
| IV (LBR) | `implied_volatility`, `normalised_implied_volatility` |
| Greeks | `delta`, `gamma`, `vega`, `theta`, `vomma`, `vanna` (+ `black_scholes_*`) |
| Batch | `*_slice` in `vector`; `*_slice_par` with feature `rayon` |
| SIMD | `norm_pdf_slice` with feature `simd` |

All fallible APIs return `Result<T, IVError>`. Batch kernels write `NaN` for
per-element domain failures and return `Err` only for length / broadcast
mismatches.

### Features

- `rayon` — parallel batch APIs (`MIN_PARALLEL` threshold applies)
- `simd` — portable `wide::f64x4` Normal PDF batches

### Examples

```bash
cargo run -p iv-engine --example atm_roundtrip
cargo run -p iv-engine --example strike_grid --features rayon
```

## Python

Install (editable):

```bash
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin pytest numpy pandas
maturin develop
```

| Layer | Names |
|-------|--------|
| Scalar | `black_price`, `implied_volatility`, Greeks, … |
| NumPy / Pandas | `black_prices`, `deltas`, `vegas`, … — ndarray or Series; **parallel by default** (`[pandas]` for Series) |

Errors raise `iv_engine.IVError` with `args == (code, message)`.

See also [`python/README.md`](../python/README.md) and
[`examples/python_roundtrip.py`](../examples/python_roundtrip.py).

## References

1. Jäckel, P. (2015). *Let's Be Rational*. Wilmott.
2. Black, F. (1976). The pricing of commodity contracts.
3. Black, F. & Scholes, M. (1973). The pricing of options and corporate liabilities.
