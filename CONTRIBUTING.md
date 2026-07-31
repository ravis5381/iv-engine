# Contributing

## Development

```bash
# Rust
cargo test -p iv-engine --all-features
cargo clippy -p iv-engine --all-targets --all-features -- -D warnings
cargo fmt --check

# Python
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin pytest numpy pandas
maturin develop
pytest
```

## Guidelines

- Numerical code belongs in the Rust crate; Python only wraps it.
- Do not approximate Let's Be Rational; prefer fidelity over micro-optimisations.
- Add tests for new public APIs (Rust unit/integration and/or Python pytest).
- Keep `README.md` phase table and `CHANGELOG.md` updated for user-visible changes.

## Pull requests

CI must pass (fmt, clippy, Rust tests with `--all-features`, Python pytest).
