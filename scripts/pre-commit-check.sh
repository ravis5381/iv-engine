#!/usr/bin/env bash
# Same checks as .pre-commit-config.yaml / CI Rust job (no tests).
set -euo pipefail
cd "$(dirname "$0")/.."

export RUSTFLAGS=-Dwarnings
export CARGO_TERM_COLOR=always

echo "== cargo fmt =="
cargo fmt --all -- --check

echo "== cargo clippy iv-engine =="
cargo clippy -p iv-engine --all-targets --all-features -- -D warnings

echo "== cargo clippy iv-engine-py =="
cargo clippy -p iv-engine-py -- -D warnings

echo "pre-commit checks passed"
