#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'EOF'
Usage: smoke.sh --version vX.Y.Z

Run install-run-uninstall smoke in a Linux container for a tagged release.
EOF
}

fail() {
  local reason="$1"
  printf 'FAIL release_smoke %s\n' "$reason" >&2
  exit 1
}

parse_args() {
  if [[ $# -ne 2 ]] || [[ "${1:-}" != "--version" ]]; then
    usage >&2
    fail "usage"
  fi

  local version="$2"
  if [[ ! "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    fail "invalid_version_format:${version}"
  fi

  printf '%s\n' "$version"
}

main() {
  local version
  version="$(parse_args "$@")"

  printf '==> release smoke for %s\n' "$version"

  if RUNSEAL_E2E_VERSION="$version" bash "$REPO_DIR/scripts/e2e/smoke.sh" smoke; then
    printf 'PASS release_smoke version=%s\n' "$version"
  else
    fail "smoke_failed:${version}"
  fi
}

main "$@"
