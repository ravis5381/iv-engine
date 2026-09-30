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

### Automated release (GitHub Actions)

Workflow: [`.github/workflows/release-pypi.yml`](../.github/workflows/release-pypi.yml)

**One-time setup**

1. Create a [PyPI API token](https://pypi.org/manage/account/token/) (scope: project `iv-engine` after the first upload, or entire account for the first release).
2. In GitHub: **Settings → Secrets and variables → Actions** → New repository secret  
   `PYPI_API_TOKEN` = the token value.
3. Optional: create a **pypi** [environment](https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment) on the repo so publishes require approval.

**Release**

1. Bump `version` in root `pyproject.toml`, `python/pyproject.toml`, and workspace `Cargo.toml` (optional — the release workflow syncs them from the tag via `.github/scripts/set_version_from_tag.py`).
2. Commit, tag, and push:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds **manylinux** wheels (`x86_64`, `aarch64`), **Windows** (`x64`, `arm64`), **macOS** (`x86_64`, `aarch64`), an **sdist**, and uploads all artifacts to PyPI. Wheels use PyO3 **abi3** (Python ≥ 3.9).

Tags must look like `v0.1.0` (leading `v`); the published version is `0.1.0`.

### Optional extras

```text
pip install iv-engine           # NumPy required
pip install 'iv-engine[pandas]' # + Pandas Series support
```

## Versioning

Keep workspace `version` and both `pyproject.toml` `version` fields in sync
(currently `0.1.0`). Record changes in [`CHANGELOG.md`](../CHANGELOG.md).
