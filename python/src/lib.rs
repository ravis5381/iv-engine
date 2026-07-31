//! `PyO3` bindings for the `iv-engine` numerical core.
//!
//! # Design
//!
//! This crate contains **no pricing or root-finding logic**. Every public
//! Python function delegates to the corresponding Rust API in `iv-engine`.
//! `NumPy` / Pandas vectorization arrives in later phases; Phase 10 exposes
//! scalar callables only.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::useless_conversion)] // PyO3 `PyResult` / `map_err` false positives
#![allow(missing_docs)]

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(iv_engine, IVError, PyException);

fn map_err(err: iv_engine::IVError) -> PyErr {
    IVError::new_err((err.code(), err.to_string()))
}

// --- Normal -----------------------------------------------------------------

#[pyfunction]
fn norm_pdf(x: f64) -> f64 {
    iv_engine::norm_pdf(x)
}

#[pyfunction]
fn norm_cdf(x: f64) -> f64 {
    iv_engine::norm_cdf(x)
}

#[pyfunction]
fn norm_cdf_c(x: f64) -> f64 {
    iv_engine::norm_cdf_c(x)
}

// --- Black-76 ---------------------------------------------------------------

#[pyfunction]
fn black_intrinsic(forward: f64, strike: f64, is_call: bool) -> f64 {
    iv_engine::black_intrinsic(forward, strike, is_call)
}

#[pyfunction]
fn log_moneyness(forward: f64, strike: f64) -> f64 {
    iv_engine::log_moneyness(forward, strike)
}

#[pyfunction]
fn black_price(
    forward: f64,
    strike: f64,
    maturity: f64,
    volatility: f64,
    is_call: bool,
) -> PyResult<f64> {
    iv_engine::black_price(forward, strike, maturity, volatility, is_call).map_err(map_err)
}

#[pyfunction]
fn black_price_total_vol(forward: f64, strike: f64, total_vol: f64, is_call: bool) -> f64 {
    iv_engine::black_price_total_vol(forward, strike, total_vol, is_call)
}

// --- Black–Scholes ----------------------------------------------------------

#[pyfunction]
fn black_scholes_forward(spot: f64, maturity: f64, rate: f64, dividend: f64) -> PyResult<f64> {
    iv_engine::black_scholes_forward(spot, maturity, rate, dividend).map_err(map_err)
}

#[pyfunction]
fn black_scholes_price(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
    is_call: bool,
) -> PyResult<f64> {
    iv_engine::black_scholes_price(spot, strike, maturity, rate, dividend, volatility, is_call)
        .map_err(map_err)
}

// --- Implied volatility -----------------------------------------------------

#[pyfunction]
fn implied_volatility(
    price: f64,
    forward: f64,
    strike: f64,
    maturity: f64,
    is_call: bool,
) -> PyResult<f64> {
    iv_engine::implied_volatility(price, forward, strike, maturity, is_call).map_err(map_err)
}

#[pyfunction]
fn normalised_implied_volatility(beta: f64, x: f64, is_call: bool) -> PyResult<f64> {
    iv_engine::normalised_implied_volatility(beta, x, is_call).map_err(map_err)
}

// --- Black-76 Greeks --------------------------------------------------------

#[pyfunction]
fn delta(
    forward: f64,
    strike: f64,
    maturity: f64,
    volatility: f64,
    is_call: bool,
) -> PyResult<f64> {
    iv_engine::delta(forward, strike, maturity, volatility, is_call).map_err(map_err)
}

#[pyfunction]
fn gamma(forward: f64, strike: f64, maturity: f64, volatility: f64) -> PyResult<f64> {
    iv_engine::gamma(forward, strike, maturity, volatility).map_err(map_err)
}

#[pyfunction]
fn vega(forward: f64, strike: f64, maturity: f64, volatility: f64) -> PyResult<f64> {
    iv_engine::vega(forward, strike, maturity, volatility).map_err(map_err)
}

#[pyfunction]
fn theta(forward: f64, strike: f64, maturity: f64, volatility: f64) -> PyResult<f64> {
    iv_engine::theta(forward, strike, maturity, volatility).map_err(map_err)
}

#[pyfunction]
fn vomma(forward: f64, strike: f64, maturity: f64, volatility: f64) -> PyResult<f64> {
    iv_engine::vomma(forward, strike, maturity, volatility).map_err(map_err)
}

#[pyfunction]
fn vanna(forward: f64, strike: f64, maturity: f64, volatility: f64) -> PyResult<f64> {
    iv_engine::vanna(forward, strike, maturity, volatility).map_err(map_err)
}

// --- Black–Scholes Greeks ---------------------------------------------------

#[pyfunction]
fn black_scholes_delta(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
    is_call: bool,
) -> PyResult<f64> {
    iv_engine::black_scholes_delta(spot, strike, maturity, rate, dividend, volatility, is_call)
        .map_err(map_err)
}

#[pyfunction]
fn black_scholes_gamma(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> PyResult<f64> {
    iv_engine::black_scholes_gamma(spot, strike, maturity, rate, dividend, volatility)
        .map_err(map_err)
}

#[pyfunction]
fn black_scholes_vega(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> PyResult<f64> {
    iv_engine::black_scholes_vega(spot, strike, maturity, rate, dividend, volatility)
        .map_err(map_err)
}

#[pyfunction]
fn black_scholes_theta(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
    is_call: bool,
) -> PyResult<f64> {
    iv_engine::black_scholes_theta(spot, strike, maturity, rate, dividend, volatility, is_call)
        .map_err(map_err)
}

#[pyfunction]
fn black_scholes_vomma(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> PyResult<f64> {
    iv_engine::black_scholes_vomma(spot, strike, maturity, rate, dividend, volatility)
        .map_err(map_err)
}

#[pyfunction]
fn black_scholes_vanna(
    spot: f64,
    strike: f64,
    maturity: f64,
    rate: f64,
    dividend: f64,
    volatility: f64,
) -> PyResult<f64> {
    iv_engine::black_scholes_vanna(spot, strike, maturity, rate, dividend, volatility)
        .map_err(map_err)
}

/// Python module `iv_engine._core` — compiled extension.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("IVError", m.py().get_type_bound::<IVError>())?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_function(wrap_pyfunction!(norm_pdf, m)?)?;
    m.add_function(wrap_pyfunction!(norm_cdf, m)?)?;
    m.add_function(wrap_pyfunction!(norm_cdf_c, m)?)?;

    m.add_function(wrap_pyfunction!(black_intrinsic, m)?)?;
    m.add_function(wrap_pyfunction!(log_moneyness, m)?)?;
    m.add_function(wrap_pyfunction!(black_price, m)?)?;
    m.add_function(wrap_pyfunction!(black_price_total_vol, m)?)?;

    m.add_function(wrap_pyfunction!(black_scholes_forward, m)?)?;
    m.add_function(wrap_pyfunction!(black_scholes_price, m)?)?;

    m.add_function(wrap_pyfunction!(implied_volatility, m)?)?;
    m.add_function(wrap_pyfunction!(normalised_implied_volatility, m)?)?;

    m.add_function(wrap_pyfunction!(delta, m)?)?;
    m.add_function(wrap_pyfunction!(gamma, m)?)?;
    m.add_function(wrap_pyfunction!(vega, m)?)?;
    m.add_function(wrap_pyfunction!(theta, m)?)?;
    m.add_function(wrap_pyfunction!(vomma, m)?)?;
    m.add_function(wrap_pyfunction!(vanna, m)?)?;

    m.add_function(wrap_pyfunction!(black_scholes_delta, m)?)?;
    m.add_function(wrap_pyfunction!(black_scholes_gamma, m)?)?;
    m.add_function(wrap_pyfunction!(black_scholes_vega, m)?)?;
    m.add_function(wrap_pyfunction!(black_scholes_theta, m)?)?;
    m.add_function(wrap_pyfunction!(black_scholes_vomma, m)?)?;
    m.add_function(wrap_pyfunction!(black_scholes_vanna, m)?)?;

    Ok(())
}
