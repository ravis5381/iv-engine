# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] — 2026-09-30

### Added

- PyPI project URLs; `python/README.md` and Jupyter notebook linked from PyPI metadata
- Notebook at `python/examples/python_examples.ipynb` included in sdists
- PyPI release scripts (`build_dist.sh`, `upload_pypi.sh`, `check_pypi_token.sh`)

### Fixed

- GitHub Actions release: version sync without `maturin version`, publish job checkout, twine upload

## [0.1.0] — 2026-07-31

Initial public release.

### Added

- Rust core: Normal PDF/CDF, Black-76 / Black–Scholes pricing, analytical Greeks
- Let's Be Rational implied volatility (validated against C++ reference goldens)
- Slice batch APIs with length-1 broadcast; optional Rayon (`rayon`) and SIMD PDF (`simd`)
- Python bindings via PyO3 / maturin (scalar, NumPy, optional Pandas helpers)
- Examples, API docs, GitHub Actions CI

### Notes

- LBR IV is not SIMD-vectorised (branchy control flow); use Rayon for batch IV.
- Batch domain failures write `NaN`; length mismatches return / raise `IVError`.
