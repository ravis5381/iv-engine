# Benchmarks

Timings depend on CPU, power settings, and Rust release optimizations. Treat numbers as relative guides on your machine.

## Quick table (Rust)

From the repo root:

```bash
cargo run -p iv-engine --release --example bench_throughput
cargo run -p iv-engine --release --features rayon --example bench_throughput
```

Prints nanoseconds per scalar call and rows/s for batch implied vol (10k / 100k / 1M) plus 1M-row pricing/Greeks.

## Criterion (detailed micro-benchmarks)

```bash
cargo bench -p iv-engine --bench implied_volatility
cargo bench -p iv-engine --bench black
cargo bench -p iv-engine --bench greeks
cargo bench -p iv-engine --bench vector
cargo bench -p iv-engine --features rayon --bench parallel
cargo bench -p iv-engine --features simd --bench simd
```

HTML reports are written under `target/criterion/` after a full run.

## Python (1M rows)

Install the extension, then:

```bash
pip install -e '.[dev]'
python examples/profile_million.py
# or: pytest python/tests/test_profile.py -m profile -s
```

Compares serial vs `parallel=True` for batch `black_prices`, `implied_volatilities`, Greeks, and Black–Scholes.

## Reference snapshot (one dev machine, release)

Re-run the commands above on your hardware; numbers below are illustrative only.

| Operation | Throughput |
|-----------|------------|
| `black_price` ATM (scalar) | ~15 ns (~64M/s) |
| `implied_volatility` ATM (scalar) | ~160–220 ns (~4–6M/s) |
| `implied_volatility_slice` (serial) | ~5M rows/s at 10k–1M |
| `black_price_slice` 1M | ~32M rows/s |
| `vega_slice` 1M | ~118M rows/s |

With `--features rayon`, `bench_throughput` also prints parallel IV throughput (typically several× on multi-core CPUs).
