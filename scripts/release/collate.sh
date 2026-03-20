#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

dist_dir="${1:-$REPO_DIR/dist}"

tmp_checksums="$(mktemp)"
find "$dist_dir" -name '*.txt' -type f ! -path "$dist_dir/checksums.txt" -exec cat {} + > "$tmp_checksums"
sort -k2 "$tmp_checksums" | uniq > "$dist_dir/checksums.txt"
rm -f "$tmp_checksums"
