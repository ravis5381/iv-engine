#!/usr/bin/env bash
# Build wheel + sdist into ./dist (repo root). Requires Rust + Python.
set -euo pipefail
cd "$(dirname "$0")/.."

python3 -m pip install -q -U maturin
rm -rf dist
mkdir -p dist
maturin build --release --out dist
maturin sdist --out dist
echo "Built:"
ls -la dist/
