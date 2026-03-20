#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_DIR="$REPO_DIR/app"

fail_release() {
  local message="$1"
  printf '%s\n' "$message" >&2
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
  ' "$APP_DIR/Cargo.toml"
}

normalize_version_tag() {
  local version="$1"
  if [[ "$version" != v* ]]; then
    version="v${version}"
  fi
  printf '%s\n' "$version"
}

write_checksums() {
  local output="$1"
  shift

  if [[ $# -eq 0 ]]; then
    fail_release "no files provided for checksum output"
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$@" > "$output"
  else
    shasum -a 256 "$@" > "$output"
  fi
}
