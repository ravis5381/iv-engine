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

PyPI renders [`python/README.md`](../python/README.md) as the project description
(`readme = "python/README.md"`). The Jupyter notebook lives at
[`python/examples/python_examples.ipynb`](../python/examples/python_examples.ipynb)
(included in sdists via `[tool.maturin] include`).

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

From the repository root:

```bash
source .venv/bin/activate   # optional
./scripts/build_dist.sh     # writes wheel + sdist to dist/
twine check dist/*          # validate only; no upload
```

Or manually (same output paths):

```bash
pip install -U maturin
maturin build --release --out dist
maturin sdist --out dist
```

**Upload** ( `--out` is for maturin only, not twine):

```bash
export TWINE_USERNAME=__token__
export TWINE_PASSWORD='pypi-...'   # one line each; full pypi- token

./scripts/upload_pypi.sh dist
# or: twine upload --non-interactive dist/*.whl dist/*.tar.gz
```

### Automated release (GitHub Actions)

Workflow: [`.github/workflows/release-pypi.yml`](../.github/workflows/release-pypi.yml)

**One-time setup**

1. Create a [PyPI API token](https://pypi.org/manage/account/token/) (scope: project `iv-engine` after the first upload, or entire account for the first release).
2. In GitHub repo **Settings → Secrets and variables → Actions** → **New repository secret**  
   - Name (exactly): `PYPI_API_TOKEN`  
   - Value: the full token string from PyPI, including the `pypi-` prefix (no quotes, no trailing newline).
3. **First release only:** token scope must be **Entire account** (or use “Upload packages” on a project token only *after* `iv-engine` exists on PyPI). A project-scoped token for a name that is not registered yet returns **403 Invalid auth**.
4. Optional: create a **pypi** [environment](https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment) on the repo so publishes require approval.

**403 on upload**

| Check | Fix |
|-------|-----|
| Secret name | Must be `PYPI_API_TOKEN`, not `PYPI_TOKEN` or similar |
| Token type | PyPI **API token** from pypi.org (not TestPyPI unless you change the upload URL) |
| First upload | Use account-wide token once, then rotate to project-scoped |
| Regenerate | Revoke old token, create new, update GitHub secret |

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
(currently `0.1.1`). Record changes in [`CHANGELOG.md`](../CHANGELOG.md).
