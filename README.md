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
| 9     | Optional SIMD                              | **Current**   |
| 10–12 | Python / NumPy / Pandas                    | Pending       |
| 13    | Docs, examples, CI, publishing             | Pending       |

## Goals

- Numerical stability and machine-precision recovery: `Black(IV(price)) ≈ price`
- Performance competitive with or faster than `py_vollib`
- Clean, idiomatic Rust with exhaustive tests
- Publishable Python package with zero-copy NumPy arrays

## Layout

```text
iv-engine/
├── Cargo.toml          # workspace
├── rust/               # numerical core (this crate)
├── python/             # PyO3 / maturin bindings (later)
├── examples/
└── docs/
```

## Building (Rust)

```bash
cd iv-engine
cargo test -p iv-engine
cargo test -p iv-engine --features rayon
cargo clippy -p iv-engine --all-targets --all-features -- -D warnings
cargo fmt --check
cargo bench -p iv-engine --bench normal
cargo bench -p iv-engine --bench black
cargo bench -p iv-engine --bench greeks
cargo bench -p iv-engine --bench implied_volatility
cargo bench -p iv-engine --bench vector
cargo bench -p iv-engine --features rayon --bench parallel
```

Optional features: `rayon` enables `*_slice_par` parallel batch APIs.

## References

1. Jäckel, P. (2015). *Let's Be Rational*. Wilmott, 2015(75), 40–53.
2. Black, F. (1976). The pricing of commodity contracts. *Journal of Financial Economics*.
3. Black, F. & Scholes, M. (1973). The pricing of options and corporate liabilities. *JPE*.

## License

MIT
