#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

fail() {
  local reason="$1"
  printf 'FAIL release_ready %s\n' "$reason" >&2
  exit 1
}

parse_cargo_version() {
  awk '
    BEGIN { in_package=0 }
    /^\[package\]/ { in_package=1; next }
    /^\[/ && in_package { exit }
    in_package && $0 ~ /^version[[:space:]]*=/ {
      gsub(/"/, "", $0)
      sub(/^version[[:space:]]*=[[:space:]]*/, "", $0)
      print $0
      exit
    }
  ' "$REPO_DIR/Cargo.toml"
}

main() {
  local version tag
  version="$(parse_cargo_version)"

  if [[ -z "$version" ]]; then
    fail "cargo_version_missing:Cargo.toml"
  fi

  tag="v${version}"
  printf 'PASS release_ready parsed_version=%s\n' "$tag"

  if bash "$REPO_DIR/scripts/version-sync-check.sh"; then
    printf 'PASS release_ready version_sync_check\n'
  else
    fail "version_sync_failed:run_scripts/version-sync-check.sh:expected:${tag}"
  fi

  if bash "$REPO_DIR/scripts/converge-check.sh"; then
    printf 'PASS release_ready converge_check\n'
  else
    fail "converge_check_failed:run_scripts/converge-check.sh"
  fi

  if bash "$REPO_DIR/scripts/release-smoke.sh" --version "$tag"; then
    printf 'PASS release_ready release_smoke version=%s\n' "$tag"
  else
    fail "release_smoke_failed:run_scripts/release-smoke.sh_--version_${tag}"
  fi

  printf 'PASS release_ready summary version=%s checks=3\n' "$tag"
}

main "$@"
