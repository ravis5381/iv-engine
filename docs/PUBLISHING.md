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

Wheels are produced with [maturin](https://www.maturin.rs/):

```bash
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin twine numpy
maturin build --release
# artifacts under python/target/wheels/ (or ../target/wheels depending on config)
twine upload target/wheels/*
```

For a source distribution that builds on the target machine:

```bash
maturin sdist
twine upload target/wheels/*.tar.gz
```

CI can publish on tagged releases using `PyO3/maturin-action` and a PyPI token.

### Optional extras

```text
pip install iv-engine           # NumPy required
pip install 'iv-engine[pandas]' # + Pandas helpers
```

## Versioning

Keep workspace `version` and `python/pyproject.toml` `version` in sync
(currently `0.1.0`). Record changes in [`CHANGELOG.md`](../CHANGELOG.md).
