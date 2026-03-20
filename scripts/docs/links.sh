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

normalized_repo_path() {
  local absolute_path="$1"
  local normalized_dir repo_path

  normalized_dir="$(cd "$(dirname "$absolute_path")" && pwd)"
  if [ "$normalized_dir" = "$REPO_DIR" ]; then
    repo_path="$(basename "$absolute_path")"
  else
    repo_path="${normalized_dir#$REPO_DIR/}/$(basename "$absolute_path")"
  fi

  printf '%s\n' "$repo_path"
}

resolve_relative_link() {
  local source_file="$1"
  local link_path="$2"
  local source_dir candidate

  source_dir="$(cd "$(dirname "$source_file")" && pwd)"
  candidate="$source_dir/$link_path"

  if [ -f "$candidate" ]; then
    printf '%s\n' "$candidate"
    return 0
  fi

  if [ -d "$candidate" ] && [ -f "$candidate/index.md" ]; then
    printf '%s\n' "$candidate/index.md"
    return 0
  fi

  case "$link_path" in
    */)
      if [ -f "$candidate/index.md" ]; then
        printf '%s\n' "$candidate/index.md"
        return 0
      fi
      ;;
    *)
      if [ -f "$candidate.md" ]; then
        printf '%s\n' "$candidate.md"
        return 0
      fi
      if [ -f "$candidate/index.md" ]; then
        printf '%s\n' "$candidate/index.md"
        return 0
      fi
      ;;
  esac

  return 1
}

resolve_root_docs_link() {
  local link_path="$1"
  local docs_target candidate

  docs_target="${link_path#/}"
  candidate="$REPO_DIR/docs/$docs_target"

  if [ -z "$docs_target" ]; then
    if [ -f "$REPO_DIR/docs/index.md" ]; then
      printf '%s\n' "$REPO_DIR/docs/index.md"
      return 0
    fi
    return 1
  fi

  if [ -f "$candidate" ]; then
    printf '%s\n' "$candidate"
    return 0
  fi

  if [ -d "$candidate" ] && [ -f "$candidate/index.md" ]; then
    printf '%s\n' "$candidate/index.md"
    return 0
  fi

  case "$docs_target" in
    */)
      if [ -f "$candidate/index.md" ]; then
        printf '%s\n' "$candidate/index.md"
        return 0
      fi
      ;;
    *)
      if [ -f "$candidate.md" ]; then
        printf '%s\n' "$candidate.md"
        return 0
      fi
      if [ -f "$candidate/index.md" ]; then
        printf '%s\n' "$candidate/index.md"
        return 0
      fi
      ;;
  esac

  return 1
}

check_link() {
  local source_file="$1"
  local line_no="$2"
  local raw_target="$3"
  local target path resolved source_rel

  target="${raw_target#${raw_target%%[![:space:]]*}}"
  target="${target%${target##*[![:space:]]}}"
  target="${target%%[[:space:]]*}"

  case "$target" in
    "") return 0 ;;
    '<'*)
      if [[ "$target" == *'>' ]]; then
        target="${target#<}"
        target="${target%>}"
      fi
      ;;
  esac

  case "$target" in
    ""|\#*|http://*|https://*|mailto:*)
      return 0
      ;;
  esac

  if [[ "$target" =~ ^[A-Za-z][A-Za-z0-9+.-]*: ]]; then
    return 0
  fi

  path="${target%%#*}"
  path="${path%%\?*}"

  if [ -z "$path" ]; then
    return 0
  fi

  source_rel="${source_file#$REPO_DIR/}"

  if [[ "$path" == /* ]]; then
    if resolved="$(resolve_root_docs_link "$path")"; then
      pass "doc_link:$source_rel:$line_no:$target -> $(normalized_repo_path "$resolved")"
    else
      fail "doc_link:$source_rel:$line_no:$target"
    fi
    return
  fi

  if resolved="$(resolve_relative_link "$source_file" "$path")"; then
    pass "doc_link:$source_rel:$line_no:$target -> $(normalized_repo_path "$resolved")"
  else
    fail "doc_link:$source_rel:$line_no:$target"
  fi
}

scan_file() {
  local source_file="$1"
  local source_rel entry line_no raw_target

  source_rel="${source_file#$REPO_DIR/}"
  pass "scan_file:$source_rel"

  while IFS= read -r entry; do
    line_no="${entry%%:*}"
    raw_target="${entry#*:}"
    check_link "$source_file" "$line_no" "$raw_target"
  done < <(perl -ne 'while (/!?\[[^\[\]]*\]\(([^)]+)\)/g) { print "$.:$1\n"; }' "$source_file")
}

files=(
  "$REPO_DIR/README.md"
  "$REPO_DIR/README.zh-CN.md"
)

while IFS= read -r md_file; do
  files+=("$md_file")
done < <(find "$REPO_DIR/docs" -type f -name '*.md' | sort)

for file in "${files[@]}"; do
  scan_file "$file"
done

if [ "$failures" -gt 0 ]; then
  exit 1
fi

pass "doc_link_integrity"
