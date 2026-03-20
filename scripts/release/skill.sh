#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_DIR="$REPO_DIR/app"
SKILLS_DIR="${REPO_DIR}/skills"

VERSION="${1:-}"
if [[ -z "${VERSION}" ]]; then
  VERSION="$(awk '
    BEGIN { in_package=0 }
    /^\[package\]/ { in_package=1; next }
    /^\[/ && in_package { exit }
    in_package && $0 ~ /^version[[:space:]]*=/ {
      gsub(/"/, "", $0)
      sub(/^version[[:space:]]*=[[:space:]]*/, "", $0)
      print $0
      exit
    }
  ' "${APP_DIR}/Cargo.toml")"
  VERSION="v${VERSION}"
fi

if [[ "${VERSION}" != v* ]]; then
  VERSION="v${VERSION}"
fi

if [[ ! -d "${SKILLS_DIR}" ]]; then
  echo "skills directory not found: ${SKILLS_DIR}" >&2
  exit 1
fi

mkdir -p "${REPO_DIR}/dist"
OUT="${REPO_DIR}/dist/skill-${VERSION}.zip"

python3 - <<'PY' "${SKILLS_DIR}" "${OUT}" "${VERSION}"
from pathlib import Path
import json
import sys
import zipfile

skills_dir = Path(sys.argv[1])
output = Path(sys.argv[2])
version = sys.argv[3]

manifest = {
    "version": version,
    "format": 1,
    "root": ".",
}

if output.exists():
    output.unlink()

with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as zf:
    zf.writestr("skill-manifest.json", json.dumps(manifest, indent=2) + "\n")
    for path in sorted(skills_dir.rglob("*")):
        if path.is_file():
            arcname = path.relative_to(skills_dir)
            zf.write(path, arcname.as_posix())
PY

echo "Built ${OUT}"
