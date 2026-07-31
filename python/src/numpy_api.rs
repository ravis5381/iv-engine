//! NumPy array bindings over Rust [`iv_engine::vector`] / [`iv_engine::parallel`].
//!
//! Inputs are contiguous 1-D `float64` arrays (or Python scalars, treated as
//! length-1 broadcasts). Outputs are newly allocated `numpy.ndarray` buffers
//! written in place by the Rust kernels — no pricing logic here.

use numpy::{PyArray1, PyArrayMethods, PyReadonlyArray1};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;

use crate::map_err;

/// Borrow a contiguous `f64` slice, or wrap a Python float as a one-element view.
enum F64In<'py> {
    Scalar([f64; 1]),
    Array(PyReadonlyArray1<'py, f64>),
}

impl<'py> F64In<'py> {
    fn from_obj(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(x) = obj.extract::<f64>() {
            return Ok(Self::Scalar([x]));
        }
        let arr: PyReadonlyArray1<'py, f64> = obj
            .extract()
            .map_err(|_| PyTypeError::new_err("expected a float or 1-D float64 numpy.ndarray"))?;
        // Ensure we can view as a contiguous slice (zero-copy).
        let _ = arr
            .as_slice()
            .map_err(|_| PyTypeError::new_err("numpy array must be a contiguous float64 buffer"))?;
        Ok(Self::Array(arr))
    }

    fn as_slice(&self) -> &[f64] {
        match self {
            Self::Scalar(s) => s.as_slice(),
            Self::Array(a) => a.as_slice().expect("checked contiguous"),
        }
    }
}

fn alloc_out<'py>(py: Python<'py>, n: usize) -> Bound<'py, PyArray1<f64>> {
    PyArray1::<f64>::zeros_bound(py, [n], false)
}

fn write_out<'py, F>(py: Python<'py>, n: usize, fill: F) -> PyResult<Bound<'py, PyArray1<f64>>>
where
    F: FnOnce(&mut [f64]) -> Result<(), iv_engine::IVError>,
{
    let out = alloc_out(py, n);
    // Exclusive write into a freshly allocated NumPy buffer.
    #[allow(unsafe_code)]
    let slice = unsafe { out.as_slice_mut()? };
    fill(slice).map_err(map_err)?;
    Ok(out)
}

/// Batch Standard Normal PDF.
#[pyfunction]
#[pyo3(name = "norm_pdfs")]
fn norm_pdfs<'py>(py: Python<'py>, x: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let xs = F64In::from_obj(x)?;
    let slice = xs.as_slice();
    write_out(py, slice.len(), |out| iv_engine::norm_pdf_slice(slice, out))
}

/// Batch Standard Normal CDF.
#[pyfunction]
#[pyo3(name = "norm_cdfs")]
fn norm_cdfs<'py>(py: Python<'py>, x: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let xs = F64In::from_obj(x)?;
    let slice = xs.as_slice();
    write_out(py, slice.len(), |out| iv_engine::norm_cdf_slice(slice, out))
}

/// Batch Standard Normal survival `1 − Φ(x)`.
#[pyfunction]
#[pyo3(name = "norm_cdf_cs")]
fn norm_cdf_cs<'py>(py: Python<'py>, x: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let xs = F64In::from_obj(x)?;
    let slice = xs.as_slice();
    write_out(py, slice.len(), |out| {
        iv_engine::norm_cdf_c_slice(slice, out)
    })
}

/// Batch undiscounted Black-76 prices.
#[pyfunction]
#[pyo3(signature = (forwards, strikes, maturities, vols, is_call=true, parallel=false))]
fn black_prices<'py>(
    py: Python<'py>,
    forwards: &Bound<'py, PyAny>,
    strikes: &Bound<'py, PyAny>,
    maturities: &Bound<'py, PyAny>,
    vols: &Bound<'py, PyAny>,
    is_call: bool,
    parallel: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let f = F64In::from_obj(forwards)?;
    let k = F64In::from_obj(strikes)?;
    let t = F64In::from_obj(maturities)?;
    let v = F64In::from_obj(vols)?;
    let fs = f.as_slice();
    let ks = k.as_slice();
    let ts = t.as_slice();
    let vs = v.as_slice();
    let n = iv_engine::batch_len(&[fs.len(), ks.len(), ts.len(), vs.len()]).map_err(map_err)?;
    write_out(py, n, |out| {
        if parallel {
            iv_engine::black_price_slice_par(fs, ks, ts, vs, is_call, out)
        } else {
            iv_engine::black_price_slice(fs, ks, ts, vs, is_call, out)
        }
    })
}

/// Batch discounted Black–Scholes–Merton prices.
#[pyfunction]
#[pyo3(signature = (spots, strikes, maturities, rates, dividends, vols, is_call=true, parallel=false))]
#[allow(clippy::too_many_arguments)]
fn black_scholes_prices<'py>(
    py: Python<'py>,
    spots: &Bound<'py, PyAny>,
    strikes: &Bound<'py, PyAny>,
    maturities: &Bound<'py, PyAny>,
    rates: &Bound<'py, PyAny>,
    dividends: &Bound<'py, PyAny>,
    vols: &Bound<'py, PyAny>,
    is_call: bool,
    parallel: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let s = F64In::from_obj(spots)?;
    let k = F64In::from_obj(strikes)?;
    let t = F64In::from_obj(maturities)?;
    let r = F64In::from_obj(rates)?;
    let q = F64In::from_obj(dividends)?;
    let v = F64In::from_obj(vols)?;
    let ss = s.as_slice();
    let ks = k.as_slice();
    let ts = t.as_slice();
    let rs = r.as_slice();
    let qs = q.as_slice();
    let vs = v.as_slice();
    let n = iv_engine::batch_len(&[ss.len(), ks.len(), ts.len(), rs.len(), qs.len(), vs.len()])
        .map_err(map_err)?;
    write_out(py, n, |out| {
        if parallel {
            iv_engine::black_scholes_price_slice_par(ss, ks, ts, rs, qs, vs, is_call, out)
        } else {
            iv_engine::black_scholes_price_slice(ss, ks, ts, rs, qs, vs, is_call, out)
        }
    })
}

/// Batch Black implied volatilities.
#[pyfunction]
#[pyo3(signature = (prices, forwards, strikes, maturities, is_call=true, parallel=false))]
fn implied_volatilities<'py>(
    py: Python<'py>,
    prices: &Bound<'py, PyAny>,
    forwards: &Bound<'py, PyAny>,
    strikes: &Bound<'py, PyAny>,
    maturities: &Bound<'py, PyAny>,
    is_call: bool,
    parallel: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let p = F64In::from_obj(prices)?;
    let f = F64In::from_obj(forwards)?;
    let k = F64In::from_obj(strikes)?;
    let t = F64In::from_obj(maturities)?;
    let ps = p.as_slice();
    let fs = f.as_slice();
    let ks = k.as_slice();
    let ts = t.as_slice();
    let n = iv_engine::batch_len(&[ps.len(), fs.len(), ks.len(), ts.len()]).map_err(map_err)?;
    write_out(py, n, |out| {
        if parallel {
            iv_engine::implied_volatility_slice_par(ps, fs, ks, ts, is_call, out)
        } else {
            iv_engine::implied_volatility_slice(ps, fs, ks, ts, is_call, out)
        }
    })
}

/// Batch normalised Black total-volatility inversions.
#[pyfunction]
#[pyo3(signature = (betas, xs, is_call=true, parallel=false))]
fn normalised_implied_volatilities<'py>(
    py: Python<'py>,
    betas: &Bound<'py, PyAny>,
    xs: &Bound<'py, PyAny>,
    is_call: bool,
    parallel: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let b = F64In::from_obj(betas)?;
    let x = F64In::from_obj(xs)?;
    let bs = b.as_slice();
    let xss = x.as_slice();
    let n = iv_engine::batch_len(&[bs.len(), xss.len()]).map_err(map_err)?;
    write_out(py, n, |out| {
        if parallel {
            iv_engine::normalised_implied_volatility_slice_par(bs, xss, is_call, out)
        } else {
            iv_engine::normalised_implied_volatility_slice(bs, xss, is_call, out)
        }
    })
}

/// Batch Black-76 forward deltas.
#[pyfunction]
#[pyo3(signature = (forwards, strikes, maturities, vols, is_call=true, parallel=false))]
fn deltas<'py>(
    py: Python<'py>,
    forwards: &Bound<'py, PyAny>,
    strikes: &Bound<'py, PyAny>,
    maturities: &Bound<'py, PyAny>,
    vols: &Bound<'py, PyAny>,
    is_call: bool,
    parallel: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let f = F64In::from_obj(forwards)?;
    let k = F64In::from_obj(strikes)?;
    let t = F64In::from_obj(maturities)?;
    let v = F64In::from_obj(vols)?;
    let fs = f.as_slice();
    let ks = k.as_slice();
    let ts = t.as_slice();
    let vs = v.as_slice();
    let n = iv_engine::batch_len(&[fs.len(), ks.len(), ts.len(), vs.len()]).map_err(map_err)?;
    write_out(py, n, |out| {
        if parallel {
            iv_engine::delta_slice_par(fs, ks, ts, vs, is_call, out)
        } else {
            iv_engine::delta_slice(fs, ks, ts, vs, is_call, out)
        }
    })
}

/// Batch Black-76 gammas.
#[pyfunction]
#[pyo3(signature = (forwards, strikes, maturities, vols, parallel=false))]
fn gammas<'py>(
    py: Python<'py>,
    forwards: &Bound<'py, PyAny>,
    strikes: &Bound<'py, PyAny>,
    maturities: &Bound<'py, PyAny>,
    vols: &Bound<'py, PyAny>,
    parallel: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let f = F64In::from_obj(forwards)?;
    let k = F64In::from_obj(strikes)?;
    let t = F64In::from_obj(maturities)?;
    let v = F64In::from_obj(vols)?;
    let fs = f.as_slice();
    let ks = k.as_slice();
    let ts = t.as_slice();
    let vs = v.as_slice();
    let n = iv_engine::batch_len(&[fs.len(), ks.len(), ts.len(), vs.len()]).map_err(map_err)?;
    write_out(py, n, |out| {
        if parallel {
            iv_engine::gamma_slice_par(fs, ks, ts, vs, out)
        } else {
            iv_engine::gamma_slice(fs, ks, ts, vs, out)
        }
    })
}

/// Batch Black-76 vegas.
#[pyfunction]
#[pyo3(signature = (forwards, strikes, maturities, vols, parallel=false))]
fn vegas<'py>(
    py: Python<'py>,
    forwards: &Bound<'py, PyAny>,
    strikes: &Bound<'py, PyAny>,
    maturities: &Bound<'py, PyAny>,
    vols: &Bound<'py, PyAny>,
    parallel: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let f = F64In::from_obj(forwards)?;
    let k = F64In::from_obj(strikes)?;
    let t = F64In::from_obj(maturities)?;
    let v = F64In::from_obj(vols)?;
    let fs = f.as_slice();
    let ks = k.as_slice();
    let ts = t.as_slice();
    let vs = v.as_slice();
    let n = iv_engine::batch_len(&[fs.len(), ks.len(), ts.len(), vs.len()]).map_err(map_err)?;
    write_out(py, n, |out| {
        if parallel {
            iv_engine::vega_slice_par(fs, ks, ts, vs, out)
        } else {
            iv_engine::vega_slice(fs, ks, ts, vs, out)
        }
    })
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(norm_pdfs, m)?)?;
    m.add_function(wrap_pyfunction!(norm_cdfs, m)?)?;
    m.add_function(wrap_pyfunction!(norm_cdf_cs, m)?)?;
    m.add_function(wrap_pyfunction!(black_prices, m)?)?;
    m.add_function(wrap_pyfunction!(black_scholes_prices, m)?)?;
    m.add_function(wrap_pyfunction!(implied_volatilities, m)?)?;
    m.add_function(wrap_pyfunction!(normalised_implied_volatilities, m)?)?;
    m.add_function(wrap_pyfunction!(deltas, m)?)?;
    m.add_function(wrap_pyfunction!(gammas, m)?)?;
    m.add_function(wrap_pyfunction!(vegas, m)?)?;
    Ok(())
}
