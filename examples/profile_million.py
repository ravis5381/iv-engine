#!/usr/bin/env python3
"""Profile iv_engine batch APIs on 1M rows (serial vs parallel).

Run from a venv with the package installed::

    cd python && maturin develop
    python ../examples/profile_million.py

Or via pytest (opt-in)::

    pytest tests/test_profile.py -m profile -s
"""

from __future__ import annotations

import time
from dataclasses import dataclass

import numpy as np

import iv_engine as iv

N = 1_000_000


@dataclass(frozen=True)
class BenchRow:
    name: str
    parallel: bool
    seconds: float
    rows_per_sec: float


def make_book(n: int = N, seed: int = 42) -> dict[str, np.ndarray]:
    rng = np.random.default_rng(seed)
    return {
        "forward": np.full(n, 100.0),
        "strike": 100.0 * np.exp(rng.normal(0.0, 0.15, size=n)),
        "maturity": rng.uniform(0.05, 2.0, size=n),
        "volatility": rng.uniform(0.10, 0.50, size=n),
    }


def time_call(fn, *, repeats: int = 1) -> float:
    fn()  # warm-up
    t0 = time.perf_counter()
    for _ in range(repeats):
        fn()
    return (time.perf_counter() - t0) / repeats


def format_table(rows: list[BenchRow]) -> str:
    header = f"{'operation':<28} {'mode':<10} {'seconds':>10} {'rows/s':>14}"
    lines = [header, "-" * len(header)]
    for r in rows:
        mode = "parallel" if r.parallel else "serial"
        lines.append(
            f"{r.name:<28} {mode:<10} {r.seconds:10.4f} {r.rows_per_sec:14,.0f}"
        )
    return "\n".join(lines)


def run_profile(n: int = N) -> list[BenchRow]:
    book = make_book(n)
    f, k, t, v = book["forward"], book["strike"], book["maturity"], book["volatility"]
    prices = iv.black_prices(f, k, t, v, True, parallel=True)

    results: list[BenchRow] = []

    def record(name: str, parallel: bool, fn) -> None:
        sec = time_call(fn, repeats=1)
        results.append(
            BenchRow(
                name=name,
                parallel=parallel,
                seconds=sec,
                rows_per_sec=(n / sec) if sec > 0 else float("inf"),
            )
        )

    ops = [
        ("black_prices", lambda p: iv.black_prices(f, k, t, v, True, parallel=p)),
        (
            "implied_volatilities",
            lambda p: iv.implied_volatilities(prices, f, k, t, True, parallel=p),
        ),
        ("deltas", lambda p: iv.deltas(f, k, t, v, True, parallel=p)),
        ("gammas", lambda p: iv.gammas(f, k, t, v, parallel=p)),
        ("vegas", lambda p: iv.vegas(f, k, t, v, parallel=p)),
        (
            "black_scholes_prices",
            lambda p: iv.black_scholes_prices(f, k, t, 0.05, 0.01, v, True, parallel=p),
        ),
    ]

    for name, fn in ops:
        for parallel in (False, True):
            record(name, parallel, lambda p=parallel, fn=fn: fn(p))

    return results


def main() -> None:
    rows = run_profile(N)
    print(f"iv_engine profile: n={N:,} rows")
    print(format_table(rows))


if __name__ == "__main__":
    main()
