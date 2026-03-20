#!/usr/bin/env bash
set -euo pipefail

DOCS_URL="https://runseal.ai/"
INSTALL_URL="https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh"
LATEST_RELEASE_API_URL="https://api.github.com/repos/PerishCode/runseal/releases/latest"

failures=0

pass() {
  printf 'PASS %s\n' "$1"
}

fail() {
  printf 'FAIL %s\n' "$1" >&2
  failures=$((failures + 1))
}

check_http() {
  local label="$1"
  local url="$2"
  if curl -fsSL -o /dev/null "$url"; then
    pass "$label"
  else
    fail "$label"
  fi
}

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL jq_missing: install jq (for example: brew install jq)" >&2
  exit 1
fi

check_http "docs_site_reachable" "$DOCS_URL"
check_http "install_script_reachable" "$INSTALL_URL"

release_json=""
if release_json="$(curl -fsSL "$LATEST_RELEASE_API_URL")"; then
  pass "latest_release_metadata_reachable"
else
  fail "latest_release_metadata_reachable"
fi

if [ -n "$release_json" ]; then
  if printf '%s' "$release_json" | jq -e 'any(.assets[]?; (.name | test("\\.tar\\.gz$")))' >/dev/null; then
    pass "latest_release_has_tar_gz_asset"
  else
    fail "latest_release_has_tar_gz_asset"
  fi

  if printf '%s' "$release_json" | jq -e 'any(.assets[]?; (.name | test("^skill-v[0-9]+\\.[0-9]+\\.[0-9]+\\.zip$")))' >/dev/null; then
    pass "latest_release_has_skill_zip_asset"
  else
    printf 'WARN latest_release_has_skill_zip_asset (not required before next tagged release)\n' >&2
  fi

  if printf '%s' "$release_json" | jq -e 'any(.assets[]?; .name == "checksums.txt")' >/dev/null; then
    pass "latest_release_has_checksums_asset"
  else
    fail "latest_release_has_checksums_asset"
  fi
fi

if [ "$failures" -gt 0 ]; then
  exit 1
fi
