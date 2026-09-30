# Publishing

## Rust (`iv-engine` on crates.io)

1. Ensure `Cargo.toml` metadata is correct (version, license, repository, description).
2. From the workspace root:

```bash
cargo publish -p iv-engine --dry-run
cargo publish -p iv-engine
```

Do **not** publish `iv-engine-py` (`publish = false`); that crate is built
only as part of the Python wheel.

Optional features that consumers may enable:

```toml
iv-engine = { version = "0.1", features = ["rayon", "simd"] }
```

Docs.rs builds with `all-features` (configured in `rust/Cargo.toml`).

## Python (`iv-engine` on PyPI)

The installable package is defined by the **workspace-root** [`pyproject.toml`](../pyproject.toml)
so `rust/` and `python/` are both included in the source distribution.

PyPI metadata uses SPDX `license = "MIT"` with `license-files = ["LICENSE", "NOTICE"]`.
[`NOTICE`](../NOTICE) cites Jäckel's *Let's Be Rational* paper and preserves the
reference-software attribution notice from www.jaeckel.org/LetsBeRational.7z.

### Install from a git clone

```bash
# needs rustc / cargo (rustup)
pip install .
pip install '.[pandas]'
pip install -e '.[dev]'
```

### Build wheels locally

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install -U pip maturin twine
maturin build --release
# wheels under target/wheels/
twine upload target/wheels/*
```

Source distribution:

```bash
maturin sdist
twine upload target/wheels/*.tar.gz
```

CI can publish on tagged releases using `PyO3/maturin-action` and a PyPI token.

### Optional extras

```text
pip install iv-engine           # NumPy required
pip install 'iv-engine[pandas]' # + Pandas Series support
```

## Versioning

Keep workspace `version` and both `pyproject.toml` `version` fields in sync
(currently `0.1.0`). Record changes in [`CHANGELOG.md`](../CHANGELOG.md).
