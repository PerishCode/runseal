#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

output="${1:-}"
shift || true

if [[ -z "$output" || $# -eq 0 ]]; then
  fail_release "usage: checksum.sh <output> <file> [file ...]"
fi

write_checksums "$output" "$@"
