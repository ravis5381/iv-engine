# iv_engine (Python)

Python bindings for the Rust **iv-engine** numerical core (PyO3 / maturin).

All pricing, Greeks, and Let's Be Rational implied volatility run in Rust.
This package only wraps those kernels — it does not reimplement any math.

| Layer | What you get |
|-------|----------------|
| Scalar | Single-value floats |
| Batch | 1-D `float64` arrays or pandas `Series` (parallel by default) |

---

## Units (read this first)

Every pricing / IV / Greek call uses the same conventions. Apply them to every example below.

| Parameter | Symbol | Unit / convention | Example |
|-----------|--------|-------------------|---------|
| `forward` / `F` | *F* | Same currency units as the option price (undiscounted Black) | `100.0` |
| `spot` / `S` | *S* | Spot in the same units as strike | `100.0` |
| `strike` / `K` | *K* | Same units as *F* or *S* | `110.0` |
| `maturity` / `T` | *T* | **Years** (or your model time unit; must be &gt; 0) | `1.0` = 1y, `0.25` ≈ 3m |
| `volatility` / `σ` | *σ* | **Absolute** annual vol — **not** percent | `0.20` = 20%, **not** `20` |
| `total_vol` / `s` | *s* = σ√T | Dimensionless total volatility | `0.2` when σ=0.2, T=1 |
| `rate` / `r` | *r* | Continuous risk-free rate (decimal per year) | `0.05` = 5% |
| `dividend` / `q` | *q* | Continuous dividend / yield (decimal per year) | `0.02` = 2% |
| `price` / `β` | — | Undiscounted Black premium (same units as *F*); normalised price for LBR `normalised_*` | |
| `is_call` | — | `True` = call, `False` = put | |
| Greeks `vega` | — | Sensitivity **per 1.0 vol point** (not per 1%) | |
| Greeks `theta` | — | Calendar θ = −∂V/∂T (per year of calendar time) | |

---

## Install

### Editable (from this repository)

```bash
cd python
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install maturin pytest numpy pandas
maturin develop
pytest
```

### From PyPI (when published)

```bash
pip install iv-engine              # NumPy required
pip install 'iv-engine[pandas]'    # Series support for batch APIs
```

---

## Quick start

**Units:** `maturity` in years (`1.0`); `volatility` absolute (`0.20` = 20%); `forward`/`strike` in price units; returned `price` undiscounted Black; returned IV absolute.

```python
import iv_engine as iv

# Undiscounted Black-76 ATM call
price = iv.black_price(forward=100.0, strike=100.0, maturity=1.0, volatility=0.20, is_call=True)
print(price)  # ~7.9656

# Recover σ with Let's Be Rational
vol = iv.implied_volatility(price, 100.0, 100.0, 1.0, True)
print(vol)    # ~0.20
```

---

## 1. Standard Normal

Useful for diagnostics and custom formulas. Inputs may be infinite; `NaN` in → `NaN` out.

### `norm_pdf` / `norm_cdf` / `norm_cdf_c`

**Units:** `x` is a dimensionless standard-normal deviate (no maturity/vol). Output: density, probability in [0, 1], or survival probability.

```python
import math
import iv_engine as iv

# Density φ(x)
assert abs(iv.norm_pdf(0.0) - 1.0 / math.sqrt(2.0 * math.pi)) < 1e-15

# CDF Φ(x) and survival 1 − Φ(x) (use survival for right-tail accuracy)
assert abs(iv.norm_cdf(0.0) - 0.5) < 1e-15
assert iv.norm_cdf_c(6.0) > 0.0          # still positive in the far tail
assert abs(iv.norm_cdf(1.0) + iv.norm_cdf_c(1.0) - 1.0) < 1e-15
```

### `norm_pdfs` / `norm_cdfs` / `norm_cdf_cs`

**Units:** same as scalar — `x` dimensionless; batch over a 1-D `float64` array.

```python
import numpy as np
import iv_engine as iv

x = np.linspace(-3.0, 3.0, 61)
pdf = iv.norm_pdfs(x)
cdf = iv.norm_cdfs(x)
surv = iv.norm_cdf_cs(x)
```

---

## 2. Black-76 pricing (forward)

Undiscounted Black on a forward `F`.

### `black_price` / `black_intrinsic` / `log_moneyness` / `black_price_total_vol`

**Units:**
- `forward`, `strike` — same price units  
- `maturity` — years (`1.0` = 1y)  
- `volatility` — absolute (`0.25` = 25%)  
- `total_vol` `s` = σ√T — dimensionless  
- output price — undiscounted, same units as `forward`

```python
import iv_engine as iv

F, K, T, sigma = 100.0, 110.0, 1.0, 0.25

call = iv.black_price(F, K, T, sigma, is_call=True)
put  = iv.black_price(F, K, T, sigma, is_call=False)

# Undiscounted put–call parity: C − P = F − K
assert abs((call - put) - (F - K)) < 1e-12

# Intrinsic (zero-vol payoff)
assert iv.black_intrinsic(F, K, True) == max(F - K, 0.0)
assert iv.black_intrinsic(F, K, False) == max(K - F, 0.0)

# Log-moneyness x = ln(F) − ln(K)  (dimensionless)
x = iv.log_moneyness(F, K)

# Price from total volatility s = σ√T directly
s = sigma * T**0.5
call_tv = iv.black_price_total_vol(F, K, s, True)
assert abs(call_tv - call) < 1e-12
```

---

## 3. Black–Scholes–Merton (spot)

Discounted BS via forward mapping `F = S·e^{(r−q)T}` and `DF = e^{−rT}`.

### `black_scholes_forward` / `black_scholes_price`

**Units:**
- `spot`, `strike` — same price units  
- `maturity` — years  
- `rate`, `dividend` — continuous decimal rates per year (`0.05` = 5%)  
- `volatility` — absolute (`0.20` = 20%)  
- output price — **discounted** (present value)

```python
import iv_engine as iv

S, K, T = 100.0, 100.0, 1.0
r, q, sigma = 0.05, 0.02, 0.20

F = iv.black_scholes_forward(S, T, r, q)
px = iv.black_scholes_price(S, K, T, r, q, sigma, is_call=True)

# Same price via Black on the forward, then discount
undisc = iv.black_price(F, K, T, sigma, True)
import math
assert abs(px - math.exp(-r * T) * undisc) < 1e-12
```

---

## 4. Implied volatility (Let's Be Rational)

Given a market (undiscounted) Black price, recover σ.

### `implied_volatility`

**Units:**
- `price` — undiscounted Black premium (same units as `forward`)  
- `forward`, `strike` — price units  
- `maturity` — years  
- output — absolute volatility (`0.30` = 30%), **not** percent

```python
import iv_engine as iv

F, K, T, sigma_true = 100.0, 95.0, 0.5, 0.30
is_call = True

mid = iv.black_price(F, K, T, sigma_true, is_call)
iv_mid = iv.implied_volatility(mid, F, K, T, is_call)
assert abs(iv_mid - sigma_true) < 1e-12
```

### `normalised_implied_volatility`

**Units:**
- `beta` — normalised Black price β = b(x, s) (dimensionless LBR coordinate)  
- `x` — log-moneyness ln(F)−ln(K) (dimensionless)  
- output `s` — total volatility σ√T (dimensionless), **not** annual σ

```python
import iv_engine as iv

# ATM normalised call at s=0.2
beta = 0.07965567455405798
s = iv.normalised_implied_volatility(beta, x=0.0, is_call=True)
assert abs(s - 0.2) < 1e-12
```

### Round-trip helper

**Units:** same as `black_price` / `implied_volatility` — `maturity` in years; `volatility` absolute.

```python
import iv_engine as iv

def roundtrip(F, K, T, sigma, is_call=True):
    p = iv.black_price(F, K, T, sigma, is_call)
    return iv.implied_volatility(p, F, K, T, is_call)

assert abs(roundtrip(100, 100, 1.0, 0.2) - 0.2) < 1e-12
assert abs(roundtrip(100, 120, 2.0, 0.15, False) - 0.15) < 1e-12
```

---

## 5. Greeks

### Black-76 (forward)

### `delta` / `gamma` / `vega` / `theta` / `vomma` / `vanna`

**Units:**
- `forward`, `strike` — price units  
- `maturity` — years  
- `volatility` — absolute (`0.20` = 20%)  
- `vega` — per **1.0** vol point (not per 1%)  
- `theta` — calendar ∂V/∂t = −∂V/∂T (per year)  
- `gamma` / `vanna` / `vomma` — as usual for Black on the forward

```python
import iv_engine as iv

F, K, T, sigma = 100.0, 100.0, 1.0, 0.20

d_call = iv.delta(F, K, T, sigma, is_call=True)
d_put  = iv.delta(F, K, T, sigma, is_call=False)
g = iv.gamma(F, K, T, sigma)      # same for call and put
v = iv.vega(F, K, T, sigma)
th = iv.theta(F, K, T, sigma)
vo = iv.vomma(F, K, T, sigma)
va = iv.vanna(F, K, T, sigma)

assert 0.0 < d_call < 1.0
assert g > 0.0 and v > 0.0
```

### Black–Scholes (spot)

### `black_scholes_delta` / `gamma` / `vega` / `theta` / `vomma` / `vanna`

**Units:**
- `spot`, `strike` — price units  
- `maturity` — years  
- `rate`, `dividend` — continuous decimal per year  
- `volatility` — absolute  
- `vega` — per 1.0 vol point; `theta` — calendar (−∂V/∂T)

```python
import iv_engine as iv

S, K, T, r, q, sigma = 100.0, 100.0, 1.0, 0.05, 0.02, 0.20

print(iv.black_scholes_delta(S, K, T, r, q, sigma, True))
print(iv.black_scholes_gamma(S, K, T, r, q, sigma))
print(iv.black_scholes_vega(S, K, T, r, q, sigma))
print(iv.black_scholes_theta(S, K, T, r, q, sigma, True))
print(iv.black_scholes_vomma(S, K, T, r, q, sigma))
print(iv.black_scholes_vanna(S, K, T, r, q, sigma))
```

---

## 6. NumPy / pandas batch APIs

Plural names (`black_prices`, `deltas`, `vegas`, …) accept:

- a **1-D contiguous `float64`** array,
- a pandas **`Series`**, or
- a Python **`float`** (broadcast)

If any argument is a `Series`, the result is a `Series` with that index; otherwise an `ndarray`.
All inputs must share length `1` or `n`. Output length is `n`.

**Parallel is on by default** (`parallel=True`). Pass `parallel=False` to force serial.

### `black_prices` — strike grid

**Units:** `maturity` years; `volatility` absolute; `forward`/`strike` price units; output undiscounted Black prices.

```python
import numpy as np
import iv_engine as iv

strikes = np.linspace(80.0, 120.0, 41)
# F, T, σ broadcast as scalars
prices = iv.black_prices(100.0, strikes, 1.0, 0.25, is_call=True)
assert prices.shape == (41,)

# Mix array forwards with scalar strike
forwards = np.linspace(95.0, 105.0, 41)
prices2 = iv.black_prices(forwards, 100.0, 0.5, 0.20, is_call=False)
```

### `implied_volatilities` — round-trip

**Units:** input `price` undiscounted Black; `maturity` years; output absolute σ (`0.22` = 22%).

```python
import numpy as np
import iv_engine as iv

K = np.linspace(70.0, 130.0, 61)
F, T, sigma = 100.0, 1.0, 0.22
px = iv.black_prices(F, K, T, sigma, True)
ivs = iv.implied_volatilities(px, F, K, T, True)
assert np.allclose(ivs, sigma, atol=1e-12)
```

### `black_scholes_prices`

**Units:** `maturity` years; `rate`/`dividend` continuous decimal; `volatility` absolute; output **discounted** PV.

```python
import numpy as np
import iv_engine as iv

spots = np.full(10, 100.0)
strikes = np.linspace(95.0, 105.0, 10)
px = iv.black_scholes_prices(
    spots, strikes, 0.5, 0.05, 0.01, 0.20, is_call=True
)
```

### `deltas` / `gammas` / `vegas`

**Units:** `maturity` years; `volatility` absolute; `vega` per 1.0 vol point.

```python
import numpy as np
import iv_engine as iv

K = np.array([90.0, 100.0, 110.0])
d = iv.deltas(100.0, K, 1.0, 0.2, is_call=True)
g = iv.gammas(100.0, K, 1.0, 0.2)
v = iv.vegas(100.0, K, 1.0, 0.2)
```

### `norm_pdfs` / `norm_cdfs` / `normalised_implied_volatilities`

**Units:** Normal `x` dimensionless; `beta` normalised price; `x` log-moneyness; output `s` = σ√T (total vol), not annual σ.

```python
import numpy as np
import iv_engine as iv

x = np.linspace(-2.0, 2.0, 101)
_ = iv.norm_pdfs(x)
_ = iv.norm_cdfs(x)

betas = np.array([0.08, 0.1856830129966639])
xs = np.array([0.0, -0.5])
s = iv.normalised_implied_volatilities(betas, xs, is_call=True)
```

### Domain failures → `NaN`

**Units:** same as `black_prices`. Invalid rows become `NaN` (batch does not abort).

```python
import numpy as np
import iv_engine as iv

K = np.array([100.0, -1.0, 110.0])  # middle strike invalid
px = iv.black_prices(100.0, K, 1.0, 0.2, True)
assert np.isfinite(px[0]) and np.isnan(px[1]) and np.isfinite(px[2])
```

---

## 7. Parallel batches (default on)

Batch APIs use Rayon **by default**. Small inputs may still run serially inside Rust when below the parallel threshold. Results are bit-identical to `parallel=False`.

### `black_prices` — serial vs parallel

**Units:** same as `black_prices` — `maturity` years; `volatility` absolute.

```python
import numpy as np
import iv_engine as iv

K = np.linspace(50.0, 150.0, 10_000)
# parallel=True is the default
default = iv.black_prices(100.0, K, 1.0, 0.2, True)
serial = iv.black_prices(100.0, K, 1.0, 0.2, True, parallel=False)
assert np.array_equal(default, serial)
```

The same default applies to `implied_volatilities`, `black_scholes_prices`, `deltas`, `gammas`, and `vegas`.

---

## 8. Pandas Series (same function names)

Requires pandas (`pip install 'iv-engine[pandas]'` or install pandas in your venv).

Use the same batch names (`black_prices`, `vegas`, …) with **`df["col"]`**. Parallel is on by default.

### `black_prices` / `implied_volatilities`

**Units:**
- `maturity` — years (Series or scalar)  
- `volatility` — absolute (`0.20` = 20%)  
- `forward` / `strike` / `price` — price units (undiscounted)  
- returned Series — absolute σ for IV helpers

```python
import pandas as pd
import iv_engine as iv

df = pd.DataFrame({
    "forward": 100.0,
    "strike": [90.0, 100.0, 110.0],
    "maturity": 1.0,
    "volatility": 0.20,
})

df["mid"] = iv.black_prices(
    forward=df["forward"],
    strike=df["strike"],
    maturity=df["maturity"],   # or maturity=1.0
    volatility=df["volatility"],
    is_call=True,
    name="mid",                # optional Series.name
)

df["implied_vol"] = iv.implied_volatilities(
    price=df["mid"],
    forward=100.0,
    strike=df["strike"],
    maturity=1.0,
    is_call=True,
)
print(df[["strike", "mid", "implied_vol"]])
```

### `black_scholes_prices`

**Units:** `maturity` years; `rate`/`dividend` continuous decimal; `volatility` absolute; output discounted PV.

```python
import pandas as pd
import iv_engine as iv

book = pd.DataFrame({
    "spot": [100.0, 100.0, 50.0],
    "strike": [100.0, 110.0, 55.0],
    "maturity": [0.25, 1.0, 0.5],
    "rate": 0.05,
    "dividend": 0.01,
    "volatility": [0.2, 0.25, 0.35],
})

book["bs_price"] = iv.black_scholes_prices(
    spot=book["spot"],
    strike=book["strike"],
    maturity=book["maturity"],
    rate=book["rate"],
    dividend=book["dividend"],
    volatility=book["volatility"],
    is_call=True,
    name="bs_price",
)
```

### `deltas` / `gammas` / `vegas`

**Units:** `maturity` years; `volatility` absolute; `vega` per 1.0 vol point.

```python
import pandas as pd
import iv_engine as iv

df = pd.DataFrame({
    "forward": [100.0, 100.0],
    "strike": [100.0, 110.0],
    "maturity": [1.0, 1.0],
    "volatility": [0.2, 0.2],
})

df["delta"] = iv.deltas(df["forward"], df["strike"], df["maturity"], df["volatility"], is_call=True)
df["gamma"] = iv.gammas(df["forward"], df["strike"], df["maturity"], df["volatility"])
df["vega"] = iv.vegas(df["forward"], df["strike"], df["maturity"], df["volatility"])
print(df)
```

### Assign pipeline

**Units:** same throughout — `maturity` years; `volatility` / recovered `iv` absolute; `vega` per 1.0 vol point.

```python
import pandas as pd
import iv_engine as iv

df = pd.DataFrame({"strike": [80.0, 100.0, 120.0]})
df["price"] = iv.black_prices(100.0, df["strike"], 1.0, 0.25)
df["iv"] = iv.implied_volatilities(df["price"], 100.0, df["strike"], 1.0)
df["vega"] = iv.vegas(100.0, df["strike"], 1.0, df["iv"])
```

---

## 9. Errors

Domain and length problems raise `iv_engine.IVError`.

### `IVError` from `implied_volatility`

**Units:** same as IV API — here a negative `price` is invalid regardless of units.

```python
import iv_engine as iv

try:
    iv.implied_volatility(-1.0, 100.0, 100.0, 1.0, True)
except iv.IVError as exc:
    code, message = exc.args
    print(code)     # "price_below_intrinsic"
    print(message)  # human-readable detail
```

| Typical `code` | Meaning |
|----------------|---------|
| `price_below_intrinsic` | Price &lt; intrinsic |
| `price_above_maximum` | Price ≥ max (F call / K put) |
| `negative_time` | `maturity ≤ 0` |
| `negative_forward` / `negative_strike` | Non-positive F or K |
| `invalid_input` | Non-finite inputs, slice length mismatch, … |
| `no_convergence` | Extremely rare LBR failure |

Mismatched Series indexes raise **`ValueError`**. Passing a column name string by mistake raises **`TypeError`** (pass `df["col"]` instead).

### Length mismatch on `black_prices`

**Units:** N/A (shape error before any pricing).

```python
import numpy as np
import iv_engine as iv

try:
    iv.black_prices(
        np.array([100.0, 101.0]),
        np.array([100.0]),           # length 1 OK
        np.array([1.0, 1.0, 1.0]),   # length 3 → mismatch
        0.2,
        True,
    )
except iv.IVError as exc:
    assert exc.args[0] == "invalid_input"
```

---

## 10. API cheat sheet

### Scalar

| Function | Role |
|----------|------|
| `norm_pdf` / `norm_cdf` / `norm_cdf_c` | Normal density / CDF / survival |
| `black_price` / `black_price_total_vol` | Black-76 price |
| `black_intrinsic` / `log_moneyness` | Intrinsic & moneyness |
| `black_scholes_price` / `black_scholes_forward` | BS price & forward |
| `implied_volatility` / `normalised_implied_volatility` | LBR IV |
| `delta` `gamma` `vega` `theta` `vomma` `vanna` | Black-76 Greeks |
| `black_scholes_*` | Spot Greeks |

### NumPy / Pandas batch

| Function | Role | Series → |
|----------|------|----------|
| `black_prices` / `black_scholes_prices` | Batch prices | `name="price"` |
| `implied_volatilities` | Batch IV | `name="implied_vol"` |
| `deltas` / `gammas` / `vegas` | Batch Greeks | `delta` / `gamma` / `vega` |
| `norm_pdfs` / `norm_cdfs` / `norm_cdf_cs` | Batch Normal | (ndarray only) |
| `normalised_implied_volatilities` | Batch normalised IV | (ndarray only) |

Keyword: `parallel: bool = True` by default. Pass any pandas ``Series`` and get a ``Series`` back; otherwise an ndarray.

---

## Design notes

- **Rust is the source of truth.** Prefer these bindings over reimplementing Black or LBR in Python.
- **Black vs BS:** use `black_*` on forwards / futures; use `black_scholes_*` on spot with continuous `r` and `q`.
- **Arrays** must be contiguous `float64` (NumPy’s default for `linspace` / `astype(float)` is fine; use `np.ascontiguousarray` if you sliced oddly).

More context: repository [`docs/API.md`](../docs/API.md) and [`examples/python_roundtrip.py`](../examples/python_roundtrip.py).
