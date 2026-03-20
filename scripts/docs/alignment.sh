#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

failures=0

pass() {
  printf 'PASS %s\n' "$1"
}

fail() {
  printf 'FAIL %s\n' "$1" >&2
  failures=$((failures + 1))
}

check_contains() {
  local label="$1"
  local file="$2"
  local pattern="$3"
  if grep -Eq "$pattern" "$file"; then
    pass "$label"
  else
    fail "$label"
  fi
}

check_contains "readme_links_zh_readme" "$REPO_DIR/README.md" 'README\.zh-CN\.md'
check_contains "readme_has_docs_site_link" "$REPO_DIR/README.md" 'https://runseal\.ai/'

check_contains "readme_mentions_install" "$REPO_DIR/README.md" 'how-to/install'
check_contains "readme_mentions_use_profiles" "$REPO_DIR/README.md" 'how-to/use-profiles'
check_contains "readme_mentions_scoreboard" "$REPO_DIR/README.md" 'explanation/runseal-score/native'
check_contains "readme_zh_mentions_install" "$REPO_DIR/README.zh-CN.md" 'how-to/install'
check_contains "readme_zh_mentions_use_profiles" "$REPO_DIR/README.zh-CN.md" 'how-to/use-profiles'
check_contains "readme_zh_mentions_scoreboard" "$REPO_DIR/README.zh-CN.md" 'explanation/runseal-score/native'

while IFS= read -r file; do
  rel="${file#./}"
  if grep -Eq -- '--use|RUNSEAL_PROFILE_HOME' "$file"; then
    fail "deprecated_terms_outside_allowed:$rel"
  fi
done < <(cd "$REPO_DIR" && find . -type f -name '*.md')

if [ "$failures" -eq 0 ]; then
  pass "deprecated_terms_scope_clean"
fi

if [ "$failures" -gt 0 ]; then
  exit 1
fi
