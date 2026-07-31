"""Profiling: 1M-row batch timings (opt-in).

Skipped in normal test runs. Execute with::

    pytest tests/test_profile.py -m profile -s

Or::

    python ../examples/profile_million.py
"""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
import pytest

import iv_engine as iv

# Import shared runner from examples/
_EXAMPLES = Path(__file__).resolve().parents[2] / "examples"
sys.path.insert(0, str(_EXAMPLES))

from profile_million import (  # noqa: E402
    N,
    format_table,
    run_profile,
)


@pytest.mark.profile
def test_profile_million_rows():
    """Time Black price, IV, and Greeks on 1M rows (serial vs parallel)."""
    results = run_profile(N)

    print(f"\niv_engine profile: n={N:,} rows\n{format_table(results)}\n")

    # Correctness smoke: ATM round-trip (deep OTM rows can pin to intrinsic)
    n_check = 10_000
    f = np.full(n_check, 100.0)
    k = np.full(n_check, 100.0)
    t = np.full(n_check, 1.0)
    v = np.full(n_check, 0.25)
    prices = iv.black_prices(f, k, t, v, True)
    ivs = iv.implied_volatilities(prices, f, k, t, True)
    np.testing.assert_allclose(ivs, v, rtol=0, atol=1e-12)

    slowest = max(r.seconds for r in results)
    assert slowest < 120.0, f"slowest op took {slowest:.1f}s"
