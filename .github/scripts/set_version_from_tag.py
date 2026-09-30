#!/usr/bin/env python3
"""Set workspace version from a release tag (e.g. 0.1.0). Used in release-pypi.yml."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FILES = ("pyproject.toml", "python/pyproject.toml", "Cargo.toml")


def main() -> None:
    if len(sys.argv) != 2:
        print("usage: set_version_from_tag.py VERSION", file=sys.stderr)
        sys.exit(2)
    version = sys.argv[1]
    if not re.fullmatch(r"\d+\.\d+\.\d+([a-zA-Z0-9.-]*)?", version):
        print(f"refusing unexpected version string: {version!r}", file=sys.stderr)
        sys.exit(1)

    for rel in FILES:
        path = ROOT / rel
        text = path.read_text(encoding="utf-8")
        new_text, n = re.subn(
            r'^version = "[^"]+"',
            f'version = "{version}"',
            text,
            count=1,
            flags=re.MULTILINE,
        )
        if n != 1:
            print(f"expected one version line in {rel}, replaced {n}", file=sys.stderr)
            sys.exit(1)
        path.write_text(new_text, encoding="utf-8")
        print(f"updated {rel} -> {version}")


if __name__ == "__main__":
    main()
