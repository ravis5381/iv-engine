#!/usr/bin/env bash
# Verify PyPI API token without publishing a real release.
#
#   export TWINE_USERNAME=__token__
#   export TWINE_PASSWORD='pypi-...'
#   ./scripts/check_pypi_token.sh

set -euo pipefail

UPLOAD_URL="${PYPI_UPLOAD_URL:-https://upload.pypi.org/legacy/}"

if [[ "${TWINE_USERNAME:-}" != "__token__" ]]; then
  echo "error: export TWINE_USERNAME=__token__" >&2
  exit 1
fi
if [[ -z "${TWINE_PASSWORD:-}" ]]; then
  echo "error: export TWINE_PASSWORD='pypi-...'" >&2
  exit 1
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
printf '' >"${tmpdir}/not-a-release.txt"

echo "Probing ${UPLOAD_URL} (dummy upload; nothing published)..."

http_code=$(
  curl -sS -o "${tmpdir}/body.html" -w '%{http_code}' \
    -u "${TWINE_USERNAME}:${TWINE_PASSWORD}" \
    -F ':action=file_upload' \
    -F 'protocol_version=1' \
    -F 'name=iv-engine' \
    -F 'version=0.0.0.token-check-do-not-publish' \
    -F "content@${tmpdir}/not-a-release.txt;type=text/plain;filename=not-a-release.txt" \
    "${UPLOAD_URL}" || true
)

case "${http_code}" in
  403)
    echo "FAIL: PyPI rejected credentials (HTTP 403)." >&2
    echo "  Use a new account-scoped token from pypi.org for the first upload." >&2
    exit 1
    ;;
  400 | 200)
    echo "OK: Token accepted (HTTP ${http_code}; dummy upload rejected as expected)."
    ;;
  *)
    echo "Unexpected HTTP ${http_code}." >&2
    head -c 500 "${tmpdir}/body.html" >&2 || true
    echo >&2
    exit 1
    ;;
esac
