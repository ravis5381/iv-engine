# iv-engine

Implied volatility and Black / Black–Scholes pricing in Rust, with Python bindings.
The numerical core implements Peter Jäckel's [*Let's Be Rational*](http://www.jaeckel.org/LetsBeRational.pdf) implied-volatility method and is validated against his reference implementation.

- **Rust** — source of truth: pricing, Greeks, batch APIs, optional `rayon` / `simd`
- **Python** — PyO3 / maturin wrappers with NumPy (and optional Pandas); no math reimplemented in Python

Design target: stable round-trip recovery `Black(IV(price)) ≈ price` at machine precision on valid inputs.

## Install

### Python

From PyPI (when published):

```bash
pip install iv-engine
pip install 'iv-engine[pandas]'   # optional Series helpers
```

From a clone (requires [Rust](https://rustup.rs/) to build the extension):

```bash
pip install '.[pandas]'           # release install
pip install -e '.[dev]'           # editable + pytest
```

Usage, units, and the full API: **[python/README.md](python/README.md)**.

### Rust

Add to `Cargo.toml`:

```toml
iv-engine = { version = "0.1", features = ["rayon"] }  # rayon optional
```

## Quick start

**Rust**

```rust
use iv_engine::{black_price, implied_volatility};

let price = black_price(100.0, 100.0, 1.0, 0.2, true)?;
let vol = implied_volatility(price, 100.0, 100.0, 1.0, true)?;
```

**Python**

```python
import iv_engine as iv

price = iv.black_price(100.0, 100.0, 1.0, 0.2, True)
vol = iv.implied_volatility(price, 100.0, 100.0, 1.0, True)
```

## Development

```bash
# Rust
cargo test -p iv-engine --all-features
cargo clippy -p iv-engine --all-targets --all-features -- -D warnings

# Python (repo root)
pytest
```

**Benchmarks:** `cargo run -p iv-engine --release --example bench_throughput` — see [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

Optional crate features: `rayon` (parallel batch), `simd` (vectorised normal PDF). PyPI releases are automated on `v*` tags — see [docs/PUBLISHING.md](docs/PUBLISHING.md).

## Documentation

| Doc | Description |
|-----|-------------|
| [python/README.md](python/README.md) | Python install, units, examples |
| [docs/API.md](docs/API.md) | API overview |
| [docs/BENCHMARKS.md](docs/BENCHMARKS.md) | Timing / throughput |
| [docs/PUBLISHING.md](docs/PUBLISHING.md) | Release checklist |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guidelines |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

## References

1. Jäckel, P. (2015). [*Let's Be Rational*](http://www.jaeckel.org/LetsBeRational.pdf). Wilmott, 2015(75), 40–53.
2. Black, F. (1976). The pricing of commodity contracts. *Journal of Financial Economics*.
3. Black, F. & Scholes, M. (1973). The pricing of options and corporate liabilities. *JPE*.

## License

[MIT](LICENSE). Algorithm attribution and Jäckel reference notice: [NOTICE](NOTICE).
