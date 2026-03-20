#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

usage() {
  cat <<'EOF'
Usage:
  accept.sh stable <tag>
  accept.sh beta <version>
EOF
}

mode="${1:-}"
input_version="${2:-}"

if [[ -z "$mode" || -z "$input_version" ]]; then
  usage >&2
  fail_release "missing mode or version"
fi

version="$(normalize_version_tag "$input_version")"
crate_version="$(parse_cargo_version)"

if [[ -z "$crate_version" ]]; then
  fail_release "failed to parse package.version from Cargo.toml"
fi

case "$mode" in
  stable)
    if [[ "$crate_version" == *-* ]]; then
      fail_release "stable releases require non-prerelease Cargo.toml version, got ${crate_version}"
    fi
    expected_tag="v${crate_version}"
    if [[ "$version" != "$expected_tag" ]]; then
      fail_release "tag/version mismatch: got ${version}, expected ${expected_tag}"
    fi
    ;;
  beta)
    if [[ ! "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+-beta\.[0-9]+$ ]]; then
      fail_release "version must match vX.Y.Z-beta.N"
    fi
    expected_tag="v${crate_version}"
    if [[ "$version" != "$expected_tag" ]]; then
      fail_release "beta version mismatch: got ${version}, expected ${expected_tag}"
    fi
    printf '%s\n' "$version"
    exit 0
    ;;
  *)
    usage >&2
    fail_release "unknown mode: ${mode}"
    ;;
esac

printf '%s\n' "$version"
