#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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
check_contains "readme_has_docs_site_link" "$REPO_DIR/README.md" 'https://perishcode\.github\.io/envlock/'
check_contains "docs_index_links_zh_cn" "$REPO_DIR/docs/index.md" '/zh-CN/'
check_contains "docs_zh_cn_index_links_root" "$REPO_DIR/docs/zh-CN/index.md" '\]\(/\)'

check_contains "readme_mentions_quick_reference" "$REPO_DIR/README.md" 'quick-reference'
check_contains "readme_mentions_ci_integration" "$REPO_DIR/README.md" 'ci-integration'
check_contains "readme_zh_mentions_quick_reference" "$REPO_DIR/README.zh-CN.md" 'quick-reference'
check_contains "readme_zh_mentions_ci_integration" "$REPO_DIR/README.zh-CN.md" 'ci-integration'

while IFS= read -r file; do
  rel="${file#./}"
  case "$rel" in
    docs/explanation/faq.md|docs/zh-CN/explanation/faq.md|docs/how-to/migrate-to-v0.2.md|docs/zh-CN/how-to/migrate-to-v0.2.md)
      continue
      ;;
  esac

  if grep -Eq -- '--use|ENVLOCK_PROFILE_HOME' "$file"; then
    fail "deprecated_terms_outside_allowed:$rel"
  fi
done < <(cd "$REPO_DIR" && find . -type f -name '*.md')

if [ "$failures" -eq 0 ]; then
  pass "deprecated_terms_scope_clean"
fi

if [ "$failures" -gt 0 ]; then
  exit 1
fi
