#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

DOC_FILES=(
  "docs/how-to/install.md"
  "docs/how-to/update-and-uninstall.md"
  "docs/reference/release.md"
  "docs/explanation/support-policy.md"
)

fail() {
  local reason="$1"
  printf 'FAIL version_sync_check %s\n' "$reason" >&2
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

extract_versions() {
  local file="$1"
  awk '
    {
      line = $0
      while (match(line, /v[0-9]+\.[0-9]+\.[0-9]+/)) {
        print substr(line, RSTART, RLENGTH)
        line = substr(line, RSTART + RLENGTH)
      }
    }
  ' "$file" | sort -u
}

main() {
  local cargo_version expected_tag
  cargo_version="$(parse_cargo_version)"

  if [[ -z "$cargo_version" ]]; then
    fail "cargo_version_missing:Cargo.toml"
  fi

  expected_tag="v${cargo_version}"
  printf 'PASS version_sync_check cargo_version=%s\n' "$expected_tag"

  local rel file
  for rel in "${DOC_FILES[@]}"; do
    file="$REPO_DIR/$rel"
    if [[ ! -f "$file" ]]; then
      fail "missing_file:${rel}"
    fi

    local -a versions=()
    local version_ref
    while IFS= read -r version_ref; do
      [[ -n "$version_ref" ]] || continue
      versions+=("$version_ref")
    done < <(extract_versions "$file")
    if [[ ${#versions[@]} -eq 0 ]]; then
      fail "missing_version_reference:${rel}:expected:${expected_tag}"
    fi

    local found="${versions[*]}"
    if [[ ${#versions[@]} -ne 1 ]] || [[ "${versions[0]}" != "$expected_tag" ]]; then
      fail "version_mismatch:${rel}:expected:${expected_tag}:found:${found}"
    fi

    printf 'PASS version_sync_check file=%s version=%s\n' "$rel" "$expected_tag"
  done

  printf 'PASS version_sync_check summary files=%s version=%s\n' "${#DOC_FILES[@]}" "$expected_tag"
}

main "$@"
