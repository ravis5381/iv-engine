# iv-engine

Production-quality implied volatility and option pricing library based on
Peter Jäckel's **Let's Be Rational** algorithm.

The Rust crate is the numerical source of truth. Python bindings (PyO3 /
maturin) wrap the same API with NumPy / Pandas vectorization — no pricing
logic is implemented in Python.

## Status

| Phase | Scope                                      | Status        |
|-------|--------------------------------------------|---------------|
| 1     | Workspace, errors, constants, module shells | Done          |
| 2     | Normal PDF / CDF                           | Done          |
| 3     | Black-76 / Black–Scholes pricing           | Done          |
| 4     | Greeks                                     | Done          |
| 5     | Let's Be Rational IV                       | Done          |
| 6     | Reference validation                       | Done          |
| 7     | Vector API                                 | Done          |
| 8     | Rayon parallelism                          | Done          |
| 9     | Optional SIMD                              | Done          |
| 10    | Python bindings (PyO3)                     | Done          |
| 11    | NumPy support                              | **Current**   |
| 12    | Pandas helpers                             | Pending       |
| 13    | Docs, examples, CI, publishing             | Pending       |

## Goals

- Numerical stability and machine-precision recovery: `Black(IV(price)) ≈ price`
- Performance competitive with or faster than `py_vollib`
- Clean, idiomatic Rust with exhaustive tests
- Publishable Python package with zero-copy NumPy arrays

## Layout

```text
iv-engine/
├── Cargo.toml          # workspace (rust + python)
├── rust/               # numerical core
├── python/             # PyO3 / maturin bindings
├── examples/
└── docs/
```

## Building (Rust)

```bash
cd iv-engine
cargo test -p iv-engine
cargo test -p iv-engine --features rayon
cargo test -p iv-engine --features simd
cargo test -p iv-engine --all-features
cargo clippy -p iv-engine --all-targets --all-features -- -D warnings
cargo fmt --check
cargo bench -p iv-engine --bench normal
cargo bench -p iv-engine --bench black
cargo bench -p iv-engine --bench greeks
cargo bench -p iv-engine --bench implied_volatility
cargo bench -p iv-engine --bench vector
cargo bench -p iv-engine --features rayon --bench parallel
cargo bench -p iv-engine --features simd --bench simd
```

Optional features:
- `rayon` — `*_slice_par` parallel batch APIs
- `simd` — portable `wide::f64x4` Normal PDF batches (`norm_pdf_slice`)

## Building (Python)

```bash
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop
pytest
```

See [`python/README.md`](python/README.md) for the scalar API (Phase 10).
NumPy array support is Phase 11.

## References

1. Jäckel, P. (2015). *Let's Be Rational*. Wilmott, 2015(75), 40–53.
2. Black, F. (1976). The pricing of commodity contracts. *Journal of Financial Economics*.
3. Black, F. & Scholes, M. (1973). The pricing of options and corporate liabilities. *JPE*.

## License

MIT
