#!/usr/bin/env bash
# Upload wheels/sdist from ./dist (or given directory) to PyPI via twine.
#
#   export TWINE_USERNAME=__token__
#   export TWINE_PASSWORD='pypi-...'
#   ./scripts/upload_pypi.sh
#   ./scripts/upload_pypi.sh dist
#
# TestPyPI:
#   TWINE_REPOSITORY_URL=https://test.pypi.org/legacy/ ./scripts/upload_pypi.sh dist

set -euo pipefail
cd "$(dirname "$0")/.."

DIST_DIR="${1:-dist}"

if [[ "${TWINE_USERNAME:-}" != "__token__" ]]; then
  echo "error: export TWINE_USERNAME=__token__" >&2
  echo "  (use a separate line for TWINE_PASSWORD)" >&2
  exit 1
fi
if [[ -z "${TWINE_PASSWORD:-}" ]]; then
  echo "error: export TWINE_PASSWORD='pypi-...'" >&2
  exit 1
fi

files=()
while IFS= read -r path; do
  files+=("$path")
done < <(find "$DIST_DIR" -type f \( -name '*.whl' -o -name '*.tar.gz' \) | sort)

if [[ ${#files[@]} -eq 0 ]]; then
  echo "error: no .whl or .tar.gz under ${DIST_DIR}/" >&2
  echo "  run: ./scripts/build_dist.sh" >&2
  find "$DIST_DIR" -type f 2>/dev/null || true
  exit 1
fi

python3 -m pip install -q twine

echo "twine check (${#files[@]} files)"
twine check "${files[@]}"

echo "twine upload -> ${TWINE_REPOSITORY_URL:-https://upload.pypi.org/legacy/}"
twine upload --non-interactive --skip-existing "${files[@]}"
