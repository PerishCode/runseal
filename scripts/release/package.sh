#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./common.sh
source "$SCRIPT_DIR/common.sh"

version="${1:-}"
target="${2:-}"
shift 2 || true
tools=("$@")

if [[ -z "$version" || -z "$target" || ${#tools[@]} -eq 0 ]]; then
  fail_release "usage: package.sh <version> <target> <tool> [tool ...]"
fi

cd "$REPO_DIR"
mkdir -p dist

for tool in "${tools[@]}"; do
  cargo build --locked --release --target "$target" --bin "$tool"
  cp "target/${target}/release/${tool}" "dist/${tool}"
  tar -C dist -czf "dist/${tool}-${version}-${target}.tar.gz" "$tool"
  rm -f "dist/${tool}"
done
