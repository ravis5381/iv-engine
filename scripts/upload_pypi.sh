#!/usr/bin/env bash
# Upload ./dist (or given dir) to PyPI with twine. Requires:
#   export TWINE_USERNAME=__token__
#   export TWINE_PASSWORD='pypi-...'
set -euo pipefail
cd "$(dirname "$0")/.."

DIST_DIR="${1:-dist}"

if [[ "${TWINE_USERNAME:-}" != "__token__" ]]; then
  echo "error: export TWINE_USERNAME=__token__" >&2
  echo "  (do not merge exports on one line)" >&2
  exit 1
fi
if [[ -z "${TWINE_PASSWORD:-}" ]]; then
  echo "error: export TWINE_PASSWORD='pypi-...'" >&2
  exit 1
fi

mapfile -t files < <(
  find "$DIST_DIR" -type f \( -name '*.whl' -o -name '*.tar.gz' \) | sort
)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "error: no .whl or .tar.gz under ${DIST_DIR}/" >&2
  find "$DIST_DIR" -type f 2>/dev/null || true
  exit 1
fi

python3 -m pip install -q twine
echo "twine check (${#files[@]} files)"
twine check "${files[@]}"
echo "twine upload"
twine upload --non-interactive --skip-existing "${files[@]}"
