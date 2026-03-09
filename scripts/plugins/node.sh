#!/usr/bin/env bash
set -euo pipefail

METHOD="${1:-}"
if [[ -z "$METHOD" ]]; then
  echo "usage: node.sh <init|validate|preview|apply>" >&2
  exit 64
fi
shift

STATE_DIR_ARG=""
NODE_BIN_ARG=""
NPM_BIN_ARG=""
PNPM_BIN_ARG=""
YARN_BIN_ARG=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --state-dir)
      [[ $# -ge 2 ]] || { echo "missing value for --state-dir" >&2; exit 64; }
      STATE_DIR_ARG="$2"
      shift 2
      ;;
    --node-bin)
      [[ $# -ge 2 ]] || { echo "missing value for --node-bin" >&2; exit 64; }
      NODE_BIN_ARG="$2"
      shift 2
      ;;
    --npm-bin)
      [[ $# -ge 2 ]] || { echo "missing value for --npm-bin" >&2; exit 64; }
      NPM_BIN_ARG="$2"
      shift 2
      ;;
    --pnpm-bin)
      [[ $# -ge 2 ]] || { echo "missing value for --pnpm-bin" >&2; exit 64; }
      PNPM_BIN_ARG="$2"
      shift 2
      ;;
    --yarn-bin)
      [[ $# -ge 2 ]] || { echo "missing value for --yarn-bin" >&2; exit 64; }
      YARN_BIN_ARG="$2"
      shift 2
      ;;
    --force)
      shift
      ;;
    -h|--help)
      echo "usage: node.sh <init|validate|preview|apply> [--state-dir <path>] [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>]" >&2
      exit 0
      ;;
    *)
      echo "unsupported option: $1" >&2
      exit 64
      ;;
  esac
done

STATE_DIR="${STATE_DIR_ARG:-${ENVLOCK_PLUGIN_NODE_STATE_DIR:-${ENVLOCK_HOME:-$HOME/.envlock}/plugin-node}}"
NODE_BIN_OVERRIDE="${NODE_BIN_ARG:-${ENVLOCK_PLUGIN_NODE_BIN:-}}"
NPM_BIN_OVERRIDE="${NPM_BIN_ARG:-${ENVLOCK_PLUGIN_NPM_BIN:-}}"
PNPM_BIN_OVERRIDE="${PNPM_BIN_ARG:-${ENVLOCK_PLUGIN_PNPM_BIN:-}}"
YARN_BIN_OVERRIDE="${YARN_BIN_ARG:-${ENVLOCK_PLUGIN_YARN_BIN:-}}"

CURRENT_BIN_DIR="$STATE_DIR/current/bin"
LOCK_DIR="$STATE_DIR/locks/apply.lock"
STATE_FILE="$STATE_DIR/state.v2.json"

log_line() {
  local level="$1"
  shift
  [[ -n "${ENVLOCK_LOG_FILE:-}" ]] || return 0
  mkdir -p "$(dirname "$ENVLOCK_LOG_FILE")" 2>/dev/null || true
  printf '%s %s plugin.node %s\n' "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" "$level" "$*" >> "$ENVLOCK_LOG_FILE" 2>/dev/null || true
}

log_info() {
  log_line INFO "$*"
}

log_warn() {
  log_line WARN "$*"
}

log_error() {
  log_line ERROR "$*"
}

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

normalize_version() {
  local raw="$1"
  raw="${raw#v}"
  printf '%s' "$raw"
}

resolve_tool_bin() {
  local __resultvar="$1"
  local tool="$2"
  local override="$3"
  if [[ -n "$override" ]]; then
    log_info "resolve tool=$tool source=override bin=$override"
    if [[ -L "$override" && ! -e "$override" ]]; then
      log_error "resolve tool=$tool invalid_symlink=$override"
      echo "configured $tool binary symlink has invalid or looped target: $override" >&2
      return 2
    fi
    [[ -e "$override" ]] || {
      log_error "resolve tool=$tool missing_bin=$override"
      echo "configured $tool binary does not exist or cannot be read: $override" >&2
      return 2
    }
    [[ -x "$override" ]] || {
      log_error "resolve tool=$tool non_executable=$override"
      echo "configured $tool binary is not executable: $override" >&2
      return 2
    }
    printf -v "$__resultvar" '%s' "$override"
    return 0
  fi

  if command -v "$tool" >/dev/null 2>&1; then
    log_info "resolve tool=$tool source=path bin=$(command -v "$tool")"
    printf -v "$__resultvar" '%s' "$(command -v "$tool")"
    return 0
  fi

  log_warn "resolve tool=$tool source=path result=not_found"
  return 1
}

require_resolved_tool() {
  local __resultvar="$1"
  local tool="$2"
  local override="$3"
  local status=0

  resolve_tool_bin "$__resultvar" "$tool" "$override"
  status=$?
  if [[ "$status" -eq 0 ]]; then
    return 0
  fi

  case "$status" in
    1)
      log_error "resolve tool=$tool result=not_found"
      echo "$tool binary not found (set ENVLOCK_PLUGIN_${tool^^}_BIN or ensure $tool on PATH)" >&2
      exit 3
      ;;
    2)
      exit 2
      ;;
    *)
      exit "$status"
      ;;
  esac
}

resolve_tool_version() {
  local bin="$1"
  local raw
  if ! raw="$($bin --version 2>/dev/null)"; then
    return 1
  fi
  log_info "version bin=$bin raw=$raw"
  normalize_version "$raw"
}

tool_version_dir() {
  local tool="$1"
  local version="$2"
  printf '%s/versions/%s/v%s' "$STATE_DIR" "$tool" "$version"
}

tool_cache_dir() {
  local tool="$1"
  local version="$2"
  printf '%s/cache/%s/v%s' "$STATE_DIR" "$tool" "$version"
}

write_state() {
  local node_bin="$1" node_version="$2"
  local npm_bin="$3" npm_version="$4"
  local pnpm_bin="$5" pnpm_version="$6"
  local yarn_bin="$7" yarn_version="$8"

  mkdir -p "$(dirname "$STATE_FILE")"
  cat > "$STATE_FILE" <<EOF
{
  "schema": "envlock.plugin-node.state.v2",
  "resolved": {
    "node": { "bin": "$(json_escape "$node_bin")", "version": "$(json_escape "$node_version")" },
    "npm": { "bin": "$(json_escape "$npm_bin")", "version": "$(json_escape "$npm_version")" },
    "pnpm": { "bin": "$(json_escape "$pnpm_bin")", "version": "$(json_escape "$pnpm_version")" },
    "yarn": { "bin": "$(json_escape "$yarn_bin")", "version": "$(json_escape "$yarn_version")" }
  },
  "paths": {
    "current_bin": "$(json_escape "$CURRENT_BIN_DIR")"
  }
}
EOF
  log_info "state wrote file=$STATE_FILE"
}

ensure_layout() {
  mkdir -p "$CURRENT_BIN_DIR" "$STATE_DIR/locks"
  log_info "layout state_dir=$STATE_DIR current_bin=$CURRENT_BIN_DIR"
}

empty_patch() {
  cat <<'EOF'
{
  "schema": "envlock.patch.v1",
  "env": [],
  "symlink": []
}
EOF
}

emit_patch() {
  local node_bin="$1" node_version="$2"
  local npm_bin="$3" npm_version="$4"
  local pnpm_bin="$5" pnpm_version="$6"
  local yarn_bin="$7" yarn_version="$8"

  local current_node="$CURRENT_BIN_DIR/node"
  local current_npm="$CURRENT_BIN_DIR/npm"
  local current_pnpm="$CURRENT_BIN_DIR/pnpm"
  local current_yarn="$CURRENT_BIN_DIR/yarn"

  cat <<EOF
{
  "schema": "envlock.patch.v1",
  "env": [
    { "op": "set", "key": "ENVLOCK_NODE_BIN", "value": "$(json_escape "$current_node")" },
    { "op": "set", "key": "ENVLOCK_NODE_VERSION", "value": "$(json_escape "$node_version")" },
    { "op": "set", "key": "NPM_CONFIG_CACHE", "value": "$(json_escape "$(tool_cache_dir npm "$npm_version")")" },
    { "op": "set", "key": "NPM_CONFIG_PREFIX", "value": "$(json_escape "$(tool_version_dir npm "$npm_version")/global")" },
    { "op": "set", "key": "PNPM_HOME", "value": "$(json_escape "$CURRENT_BIN_DIR")" },
    { "op": "set", "key": "PNPM_STORE_PATH", "value": "$(json_escape "$(tool_cache_dir pnpm "$pnpm_version")/store")" },
    { "op": "set", "key": "YARN_CACHE_FOLDER", "value": "$(json_escape "$(tool_cache_dir yarn "$yarn_version")")" },
    { "op": "prepend_path", "key": "PATH", "value": "$(json_escape "$CURRENT_BIN_DIR")", "separator": ":" }
  ],
  "symlink": [
    { "op": "ensure", "source": "$(json_escape "$node_bin")", "target": "$(json_escape "$current_node")", "on_exist": "replace" },
    { "op": "ensure", "source": "$(json_escape "$npm_bin")", "target": "$(json_escape "$current_npm")", "on_exist": "replace" },
    { "op": "ensure", "source": "$(json_escape "$pnpm_bin")", "target": "$(json_escape "$current_pnpm")", "on_exist": "replace" },
    { "op": "ensure", "source": "$(json_escape "$yarn_bin")", "target": "$(json_escape "$current_yarn")", "on_exist": "replace" }
  ]
}
EOF
  log_info "patch emitted env_count=8 symlink_count=4"
}

resolve_all_tools() {
  require_resolved_tool NODE_BIN node "$NODE_BIN_OVERRIDE"
  [[ -x "$NODE_BIN" ]] || {
    echo "node binary is not executable: $NODE_BIN" >&2
    exit 2
  }
  NODE_VERSION="$(resolve_tool_version "$NODE_BIN")" || {
    log_error "version tool=node bin=$NODE_BIN result=failed"
    echo "failed to resolve node version from: $NODE_BIN" >&2
    exit 4
  }

  require_resolved_tool NPM_BIN npm "$NPM_BIN_OVERRIDE"
  require_resolved_tool PNPM_BIN pnpm "$PNPM_BIN_OVERRIDE"
  require_resolved_tool YARN_BIN yarn "$YARN_BIN_OVERRIDE"

  NPM_VERSION="$(resolve_tool_version "$NPM_BIN")" || { log_error "version tool=npm bin=$NPM_BIN result=failed"; echo "failed to resolve npm version" >&2; exit 4; }
  PNPM_VERSION="$(resolve_tool_version "$PNPM_BIN")" || { log_error "version tool=pnpm bin=$PNPM_BIN result=failed"; echo "failed to resolve pnpm version" >&2; exit 4; }
  YARN_VERSION="$(resolve_tool_version "$YARN_BIN")" || { log_error "version tool=yarn bin=$YARN_BIN result=failed"; echo "failed to resolve yarn version" >&2; exit 4; }
  log_info "resolved node=$NODE_VERSION npm=$NPM_VERSION pnpm=$PNPM_VERSION yarn=$YARN_VERSION"
}

prepare_version_dirs() {
  mkdir -p \
    "$(tool_version_dir node "$NODE_VERSION")/bin" \
    "$(tool_version_dir npm "$NPM_VERSION")/bin" \
    "$(tool_version_dir npm "$NPM_VERSION")/global" \
    "$(tool_version_dir pnpm "$PNPM_VERSION")/bin" \
    "$(tool_version_dir yarn "$YARN_VERSION")/bin" \
    "$(tool_cache_dir npm "$NPM_VERSION")" \
    "$(tool_cache_dir pnpm "$PNPM_VERSION")/store" \
    "$(tool_cache_dir yarn "$YARN_VERSION")"
  log_info "dirs prepared state_dir=$STATE_DIR"
}

link_versions() {
  ln -sfn "$NODE_BIN" "$(tool_version_dir node "$NODE_VERSION")/bin/node"
  ln -sfn "$NPM_BIN" "$(tool_version_dir npm "$NPM_VERSION")/bin/npm"
  ln -sfn "$PNPM_BIN" "$(tool_version_dir pnpm "$PNPM_VERSION")/bin/pnpm"
  ln -sfn "$YARN_BIN" "$(tool_version_dir yarn "$YARN_VERSION")/bin/yarn"

  ln -sfn "$(tool_version_dir node "$NODE_VERSION")/bin/node" "$CURRENT_BIN_DIR/node"
  ln -sfn "$(tool_version_dir npm "$NPM_VERSION")/bin/npm" "$CURRENT_BIN_DIR/npm"
  ln -sfn "$(tool_version_dir pnpm "$PNPM_VERSION")/bin/pnpm" "$CURRENT_BIN_DIR/pnpm"
  ln -sfn "$(tool_version_dir yarn "$YARN_VERSION")/bin/yarn" "$CURRENT_BIN_DIR/yarn"
  log_info "symlinks refreshed current_bin=$CURRENT_BIN_DIR"
}

do_init() {
  ensure_layout
  log_info "method=init"
  empty_patch
}

do_validate_or_preview() {
  ensure_layout
  log_info "method=$METHOD"
  resolve_all_tools
  prepare_version_dirs
  emit_patch "$NODE_BIN" "$NODE_VERSION" "$NPM_BIN" "$NPM_VERSION" "$PNPM_BIN" "$PNPM_VERSION" "$YARN_BIN" "$YARN_VERSION"
}

do_apply() {
  ensure_layout
  if ! mkdir "$LOCK_DIR" 2>/dev/null; then
    log_warn "lock rejected dir=$LOCK_DIR"
    echo "node plugin apply is locked by another process" >&2
    exit 73
  fi
  log_info "lock acquired dir=$LOCK_DIR"
  trap 'log_info "lock released dir=$LOCK_DIR"; rmdir "$LOCK_DIR" 2>/dev/null || true' EXIT

  resolve_all_tools
  prepare_version_dirs
  link_versions
  write_state "$NODE_BIN" "$NODE_VERSION" "$NPM_BIN" "$NPM_VERSION" "$PNPM_BIN" "$PNPM_VERSION" "$YARN_BIN" "$YARN_VERSION"
  emit_patch "$NODE_BIN" "$NODE_VERSION" "$NPM_BIN" "$NPM_VERSION" "$PNPM_BIN" "$PNPM_VERSION" "$YARN_BIN" "$YARN_VERSION"
}

case "$METHOD" in
  init)
    do_init
    ;;
  validate|preview)
    do_validate_or_preview
    ;;
  apply)
    do_apply
    ;;
  *)
    echo "unsupported method: $METHOD" >&2
    exit 64
    ;;
esac
