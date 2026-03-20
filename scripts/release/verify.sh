#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$REPO_DIR"
pnpm install --frozen-lockfile
cargo fmt --check
cargo test --locked
pnpm run docs:build
bash scripts/docs/links.sh
bash scripts/docs/alignment.sh
bash scripts/docs/agent-meta.sh
bash scripts/docs/agent-routes.sh
