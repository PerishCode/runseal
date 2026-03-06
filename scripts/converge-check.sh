#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_step() {
  local label="$1"
  shift
  printf '==> %s\n' "$label"
  "$@"
}

cd "$REPO_DIR"

run_step "verify doc alignment" ./scripts/verify-doc-alignment.sh
run_step "verify doc links" ./scripts/verify-doc-links.sh
run_step "build docs" pnpm run docs:build
run_step "verify agent meta" ./scripts/verify-agent-meta.sh
run_step "verify agent routes" ./scripts/check-agent-routes.sh
run_step "run cargo tests" cargo test --locked
run_step "verify public surface" ./scripts/verify-public-surface.sh

printf 'PASS converge_check\n'
