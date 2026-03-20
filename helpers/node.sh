#!/usr/bin/env bash
set -euo pipefail

METHOD="${1:-}"
if [[ -z "$METHOD" ]]; then
  echo "usage: node.sh <install|list|which|remote list|snapshot|uninstall|help|example>" >&2
  exit 64
fi
shift

if [[ "$METHOD" == "remote" ]]; then
  SUBMETHOD="${1:-}"
  [[ -n "$SUBMETHOD" ]] || {
    echo "usage: node.sh remote list" >&2
    exit 64
  }
  shift
  case "$SUBMETHOD" in
    list) METHOD="remote-list" ;;
    *)
      echo "unsupported remote subcommand: $SUBMETHOD" >&2
      exit 64
      ;;
  esac
fi

NODE_VERSION_ARG=""

print_help() {
  cat <<'EOF'
usage: node.sh <install|list|which|remote list|snapshot|uninstall|help|example> [--node-version <version>]

commands:
  install   prepare an isolated Node toolchain home and emit a runseal patch
  list      list installed Node versions managed by the helper
  which     print resolved paths for one installed Node version
  remote list  list installable Node versions from the configured mirror
  snapshot  print a read-only snapshot of the current Node toolchain and recommended patch
  uninstall remove the managed Node helper home
  help      print this message and recommended runseal usage
  example   print a runseal profile example with all known dirty boundaries called out

environment:
  RUNSEAL_HELPER_NODE_HOME   node helper home (default: ~/.runseal/helpers/node)
  RUNSEAL_HELPER_NODE_MIRROR node distro mirror root (default: https://nodejs.org/dist)

recommended runseal usage:
  runseal helper :node remote list
  runseal helper :node install --node-version 24.12.0
  runseal helper :node list
  runseal helper :node which --node-version 24.12.0
  runseal helper :node uninstall --node-version 24.12.0

runtime entrypoint:
  use a runseal profile alias or profile path for day-to-day commands
  example: runseal :node24-profile npm i -g pnpm
EOF
}

print_example() {
  cat <<'EOF'
{
  "schema": "runseal.profile.v1",
  "meta": {
    "name": "node-dev",
    "description": "Node toolchain sealed through helper-managed versions/vX.Y.Z layout"
  },
  "injections": [
    {
      "type": "env",
      "ops": [
        { "op": "set", "key": "RUNSEAL_HELPER_NODE_HOME", "value": "~/.runseal/helpers/node" },
        { "op": "set", "key": "RUNSEAL_NODE_VERSION", "value": "24.12.0" },
        { "op": "set", "key": "RUNSEAL_NODE_BIN", "value": "~/.runseal/helpers/node/versions/v24.12.0/bin/node" },
        { "op": "set", "key": "RUNSEAL_COREPACK_SHIMS", "value": "~/.runseal/helpers/node/versions/v24.12.0/corepack-bin" },
        { "op": "set", "key": "COREPACK_HOME", "value": "~/.runseal/helpers/node/versions/v24.12.0/cache/corepack" },
        { "op": "set", "key": "NPM_CONFIG_CACHE", "value": "~/.runseal/helpers/node/versions/v24.12.0/cache/npm" },
        { "op": "set", "key": "NPM_CONFIG_PREFIX", "value": "~/.runseal/helpers/node/versions/v24.12.0" },
        { "op": "set", "key": "npm_config_prefix", "value": "~/.runseal/helpers/node/versions/v24.12.0" },
        { "op": "prepend", "key": "PATH", "value": "~/.runseal/helpers/node/versions/v24.12.0/node_modules/.bin", "separator": ":" },
        { "op": "prepend", "key": "PATH", "value": "~/.runseal/helpers/node/versions/v24.12.0/corepack-bin", "separator": ":" },
        { "op": "prepend", "key": "PATH", "value": "~/.runseal/helpers/node/versions/v24.12.0/bin", "separator": ":" }
      ]
    }
  ]
}

dirty boundaries to keep sealed:
- primary runtime entries in bin/
- corepack-managed shims in corepack-bin/
- node_modules/.bin entries
- npm cache + prefix
- corepack cache and prepared manager state
- helper-managed runtime directories

symlink recommendation:
- point project-local tool entrypoints at files under versions/vX.Y.Z/bin when you need stable workspace-local shims
- keep env injection as the default when PATH-based activation is acceptable
- treat corepack-bin/ as a visible managed shim layer, not as ambient global state
EOF
}

if [[ "$METHOD" == "help" ]]; then
  print_help
  exit 0
fi

if [[ "$METHOD" == "example" ]]; then
  print_example
  exit 0
fi

case "$METHOD" in
  install|list|which|remote-list|snapshot|uninstall|example)
    ;;
  *)
    echo "unsupported method: $METHOD (use install, list, which, remote list, snapshot, uninstall, or help)" >&2
    exit 64
    ;;
esac

while [[ $# -gt 0 ]]; do
  case "$1" in
    --node-version)
      [[ $# -ge 2 ]] || { echo "missing value for --node-version" >&2; exit 64; }
      NODE_VERSION_ARG="$2"
      shift 2
      ;;
    -h|--help)
      print_help
      exit 0
      ;;
    *)
      echo "unsupported option: $1" >&2
      exit 64
      ;;
  esac
done

NODE_HOME="${RUNSEAL_HELPER_NODE_HOME:-${RUNSEAL_HOME:-$HOME/.runseal}/helpers/node}"
NODE_MIRROR="${RUNSEAL_HELPER_NODE_MIRROR:-https://nodejs.org/dist}"

log_line() {
  local level="$1"
  shift
  [[ -n "${RUNSEAL_LOG_FILE:-}" ]] || return 0
  mkdir -p "$(dirname "$RUNSEAL_LOG_FILE")" 2>/dev/null || true
  printf '%s %s node %s\n' "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" "$level" "$*" >> "$RUNSEAL_LOG_FILE" 2>/dev/null || true
}

log_info() { log_line INFO "$*"; }
log_warn() { log_line WARN "$*"; }
log_error() { log_line ERROR "$*"; }

log_info "config node_home=$NODE_HOME node_mirror=$NODE_MIRROR"

node_home_error() {
  local action="$1"
  local detail="${2:-unknown error}"
  log_error "node_home action=$action node_home=$NODE_HOME detail=$detail"
  echo "failed to $action in node helper home $NODE_HOME: $detail" >&2
  exit 74
}

run_node_home_op() {
  local action="$1"
  shift
  local output
  if ! output="$($@ 2>&1)"; then
    node_home_error "$action" "$output"
  fi
}

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

shell_quote() {
  printf '%q' "$1"
}

normalize_version() {
  local raw="$1"
  raw="${raw#v}"
  printf '%s' "$raw"
}

requested_version() {
  if [[ -z "$NODE_VERSION_ARG" ]]; then
    return 1
  fi
  normalize_version "$NODE_VERSION_ARG"
}

version_dir() {
  local version="$1"
  printf '%s/versions/v%s' "$NODE_HOME" "$version"
}

installed_version_exists() {
  local version="$1"
  [[ -d "$(version_dir "$version")" ]]
}

bin_dir() {
  local version="$1"
  printf '%s/bin' "$(version_dir "$version")"
}

node_modules_dir() {
  local version="$1"
  printf '%s/node_modules' "$(version_dir "$version")"
}

runtime_dir() {
  local version="$1"
  printf '%s/runtime' "$(version_dir "$version")"
}

corepack_shims_dir() {
  local version="$1"
  printf '%s/corepack-bin' "$(version_dir "$version")"
}

global_modules_dir() {
  local version="$1"
  printf '%s/lib/node_modules' "$(version_dir "$version")"
}

cache_dir() {
  local version="$1"
  local name="${2:-npm}"
  printf '%s/cache/%s' "$(version_dir "$version")" "$name"
}

home_dir() {
  local version="$1"
  printf '%s/home/npm' "$(version_dir "$version")"
}

lock_dir() {
  local version="$1"
  printf '%s/.lock' "$(version_dir "$version")"
}

lock_pid_file() {
  local version="$1"
  printf '%s/pid' "$(lock_dir "$version")"
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
      echo "$tool binary not found (set RUNSEAL_${tool^^}_BIN or ensure $tool on PATH)" >&2
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
  local node_bin_dir=""
  mkdir -p "$NODE_HOME"
  if [[ -n "${NODE_BIN:-}" ]]; then
    node_bin_dir="$(dirname "$NODE_BIN")"
  fi
  if ! raw="$(cd "$NODE_HOME" && PATH="${node_bin_dir}${node_bin_dir:+:}$PATH" "$bin" --version 2>/dev/null)"; then
    return 1
  fi
  log_info "version bin=$bin raw=$raw"
  normalize_version "$raw"
}

resolve_node_tool() {
  NODE_VERSION="$(requested_version 2>/dev/null || true)"
  [[ -n "$NODE_VERSION" ]] || {
    echo "install requires --node-version" >&2
    exit 64
  }
}

resolve_manager_tools() {
  NPM_VERSION="$(resolve_tool_version "$NPM_BIN")" || { log_error "version tool=npm bin=$NPM_BIN result=failed"; echo "failed to resolve npm version" >&2; exit 4; }
  log_info "resolved node=$NODE_VERSION npm=$NPM_VERSION"
}

resolve_all_tools() {
  resolve_node_tool
  resolve_manager_tools
}

prepare_version_dirs() {
  local root
  root="$(version_dir "$NODE_VERSION")"
  run_node_home_op "prepare version directories" mkdir -p \
    "$(bin_dir "$NODE_VERSION")" \
    "$(corepack_shims_dir "$NODE_VERSION")" \
    "$(global_modules_dir "$NODE_VERSION")/.bin" \
    "$(cache_dir "$NODE_VERSION" npm)" \
    "$(cache_dir "$NODE_VERSION" corepack)" \
    "$(home_dir "$NODE_VERSION")"
  if [[ ! -e "$(node_modules_dir "$NODE_VERSION")" ]]; then
    run_node_home_op "link public node_modules" ln -sfn "lib/node_modules" "$(node_modules_dir "$NODE_VERSION")"
  fi
  log_info "dirs prepared node_home=$NODE_HOME version_root=$root"
}

write_wrapper() {
  local tool="$1"
  local path="$2"
  local real_bin="$3"
  [[ -n "$real_bin" ]] || return 0
  local version_root="$(version_dir "$NODE_VERSION")"
  local bin_root="$(bin_dir "$NODE_VERSION")"
  local node_modules_root="$(global_modules_dir "$NODE_VERSION")"

  run_node_home_op "prepare wrapper dir" mkdir -p "$(dirname "$path")"
  case "$tool" in
    npm)
      cat > "$path" <<EOF
#!/usr/bin/env bash
set -euo pipefail
export PATH="$bin_root:$node_modules_root/.bin${PATH:+:$PATH}"
export NPM_CONFIG_CACHE=$(shell_quote "$(cache_dir "$NODE_VERSION")")
export NPM_CONFIG_PREFIX=$(shell_quote "$version_root")
export npm_config_prefix=$(shell_quote "$version_root")
exec $(shell_quote "$real_bin") "\$@"
EOF
      ;;
    *)
      node_home_error "write wrapper" "unsupported tool $tool"
      ;;
  esac
  run_node_home_op "set wrapper executable" chmod 755 "$path"
}

link_versions() {
  local bin_root
  bin_root="$(bin_dir "$NODE_VERSION")"

  run_node_home_op "link node version" ln -sfn "$NODE_BIN" "$bin_root/node"
  write_wrapper npm "$bin_root/npm" "$NPM_BIN"
  log_info "version bins prepared version_root=$(version_dir "$NODE_VERSION") bin_root=$bin_root"
}

prepare_corepack_shims() {
  local shims_root shim_dir
  shim_dir="$(corepack_shims_dir "$NODE_VERSION")"
  shims_root="$(global_modules_dir "$NODE_VERSION")/corepack/shims"

  [[ -d "$shims_root" ]] || return 0
  run_node_home_op "prepare corepack shim dir" mkdir -p "$shim_dir"

  for name in pnpm pnpx yarn yarnpkg; do
    if [[ -e "$shims_root/$name" ]]; then
      cat > "$shim_dir/$name" <<EOF
#!/usr/bin/env bash
set -euo pipefail
exec $(shell_quote "$shims_root/$name") "\$@"
EOF
      run_node_home_op "set corepack shim executable $name" chmod 755 "$shim_dir/$name"
    fi
  done
  log_info "corepack shims prepared shim_dir=$shim_dir"
}

read_lock_pid() {
  local version="$1"
  local pid=""
  if ! pid="$(cat "$(lock_pid_file "$version")" 2>/dev/null)"; then
    return 1
  fi
  case "$pid" in
    ''|*[!0-9]*) return 1 ;;
  esac
  printf '%s' "$pid"
}

release_lock() {
  local version="$1"
  log_info "lock released dir=$(lock_dir "$version")"
  rm -f "$(lock_pid_file "$version")" 2>/dev/null || true
  rmdir "$(lock_dir "$version")" 2>/dev/null || true
}

recover_stale_lock() {
  local version="$1"
  local dir
  dir="$(lock_dir "$version")"
  local pid=""

  if pid="$(read_lock_pid "$version")"; then
    if kill -0 "$pid" 2>/dev/null; then
      log_warn "lock busy dir=$dir pid=$pid"
      return 1
    fi
    log_warn "lock stale dir=$dir pid=$pid"
    rm -rf "$dir"
    return 0
  fi

  sleep 1
  if pid="$(read_lock_pid "$version")"; then
    if kill -0 "$pid" 2>/dev/null; then
      log_warn "lock busy dir=$dir pid=$pid"
      return 1
    fi
    log_warn "lock stale dir=$dir pid=$pid"
    rm -rf "$dir"
    return 0
  fi

  if rmdir "$dir" 2>/dev/null; then
    log_warn "lock stale dir=$dir pid=missing"
    return 0
  fi
  log_warn "lock busy dir=$dir pid=unknown"
  return 1
}

acquire_lock() {
  local version="$1"
  local dir
  dir="$(lock_dir "$version")"
  local pid_file
  pid_file="$(lock_pid_file "$version")"
  local lock_output=""

  run_node_home_op "create version lock parent" mkdir -p "$(version_dir "$version")"

  if lock_output="$(mkdir "$dir" 2>&1)"; then
    :
  elif [[ ! -e "$dir" ]]; then
    node_home_error "create lock directory" "$lock_output"
  else
    recover_stale_lock "$version" || {
      log_warn "lock rejected dir=$dir"
      echo "node helper install is locked by another process" >&2
      exit 73
    }

    if lock_output="$(mkdir "$dir" 2>&1)"; then
      :
    elif [[ -e "$dir" ]]; then
      log_warn "lock rejected dir=$dir"
      echo "node helper install is locked by another process" >&2
      exit 73
    else
      node_home_error "create lock directory" "$lock_output"
    fi
  fi

  printf '%s\n' "$$" > "$pid_file"
  log_info "lock acquired dir=$dir pid=$$"
  trap 'release_lock "$NODE_VERSION"' EXIT
}

emit_patch() {
  local version="$1"
  local version_root="$(version_dir "$version")"
  local bin_root="$(bin_dir "$version")"
  local node_modules_root="$(node_modules_dir "$version")"

  cat <<EOF
{
  "schema": "runseal.patch.v1",
  "env": [
    { "op": "set", "key": "RUNSEAL_HELPER_NODE_HOME", "value": "$(json_escape "$NODE_HOME")" },
    { "op": "set", "key": "RUNSEAL_NODE_VERSION", "value": "$(json_escape "$version")" },
    { "op": "set", "key": "RUNSEAL_NODE_BIN", "value": "$(json_escape "$bin_root/node")" },
    { "op": "set", "key": "RUNSEAL_COREPACK_SHIMS", "value": "$(json_escape "$(corepack_shims_dir "$version")")" },
    { "op": "set", "key": "COREPACK_HOME", "value": "$(json_escape "$(cache_dir "$version" corepack)")" },
    { "op": "set", "key": "NPM_CONFIG_CACHE", "value": "$(json_escape "$(cache_dir "$version" npm)")" },
    { "op": "set", "key": "NPM_CONFIG_PREFIX", "value": "$(json_escape "$version_root")" },
    { "op": "set", "key": "npm_config_prefix", "value": "$(json_escape "$version_root")" },
    { "op": "prepend", "key": "PATH", "value": "$(json_escape "$node_modules_root/.bin")", "separator": ":" },
    { "op": "prepend", "key": "PATH", "value": "$(json_escape "$(corepack_shims_dir "$version")")", "separator": ":" },
    { "op": "prepend", "key": "PATH", "value": "$(json_escape "$bin_root")", "separator": ":" }
  ],
  "symlink": []
}
EOF
  log_info "patch emitted env_count=8 symlink_count=0 path_layers=3"
}

detect_platform() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) arch="x64" ;;
    arm64|aarch64) arch="arm64" ;;
    *)
      echo "unsupported architecture: $arch" >&2
      exit 2
      ;;
  esac
  case "$os" in
    linux|darwin) ;;
    *)
      echo "unsupported operating system: $os" >&2
      exit 2
      ;;
  esac
  printf '%s-%s' "$os" "$arch"
}

download_node_runtime() {
  local version="$1"
  local platform archive_name download_url temp_dir extracted_dir version_root
  platform="$(detect_platform)"
  archive_name="node-v${version}-${platform}.tar.xz"
  download_url="${NODE_MIRROR%/}/v${version}/${archive_name}"
  temp_dir="$(runtime_dir "$version")"
  version_root="$(version_dir "$version")"
  extracted_dir="$temp_dir/extracted"

run_node_home_op "prepare runtime temp dir" mkdir -p "$temp_dir"
  run_node_home_op "download node runtime" curl -fsSL "$download_url" -o "$temp_dir/$archive_name"
  run_node_home_op "extract node runtime" mkdir -p "$extracted_dir"
  run_node_home_op "extract node runtime" tar -xJf "$temp_dir/$archive_name" -C "$extracted_dir" --strip-components=1
  NODE_BIN="$extracted_dir/bin/node"
  NPM_BIN="$extracted_dir/bin/npm"
  NPM_VERSION="$(resolve_tool_version "$NPM_BIN")" || {
    log_error "version tool=npm bin=$NPM_BIN result=failed"
    echo "failed to resolve npm version from downloaded runtime" >&2
    exit 4
  }
  log_info "downloaded node runtime version=$version platform=$platform url=$download_url"
}

print_snapshot() {
  local version="$1"
  local version_root="$(version_dir "$version")"
  local bin_root="$(bin_dir "$version")"
  local node_modules_root="$(node_modules_dir "$version")"

  cat <<EOF
{
  "schema": "runseal.node.snapshot.v1",
  "resolved": {
    "node": { "bin": "$(json_escape "$NODE_BIN")", "version": "$(json_escape "$NODE_VERSION")", "wrapper": "$(json_escape "$bin_root/node")" },
    "npm": { "bin": "$(json_escape "$NPM_BIN")", "version": "$(json_escape "$NPM_VERSION")", "wrapper": "$(json_escape "$bin_root/npm")" }
  },
  "node_home": "$(json_escape "$NODE_HOME")",
  "version_root": "$(json_escape "$version_root")",
  "bin_dir": "$(json_escape "$bin_root")",
  "corepack_shims_dir": "$(json_escape "$(corepack_shims_dir "$version")")",
  "node_modules_dir": "$(json_escape "$node_modules_root")",
  "patch":
EOF
  emit_patch "$version"
  printf '\n'
}

print_installed_versions() {
  local versions_dir="$NODE_HOME/versions"
  [[ -d "$versions_dir" ]] || exit 0
  for dir in "$versions_dir"/v*; do
    [[ -d "$dir" ]] || continue
    printf '%s\t%s\n' "$(basename "$dir" | sed 's/^v//')" "$dir"
  done | sort -V
}

print_which() {
  local version="$1"
  local version_root="$(version_dir "$version")"
  [[ -d "$version_root" ]] || {
    echo "node helper version is not installed: $version" >&2
    exit 2
  }
  cat <<EOF
version=$version
version_root=$version_root
node=$(bin_dir "$version")/node
npm=$(bin_dir "$version")/npm
corepack_shims=$(corepack_shims_dir "$version")
node_modules=$(node_modules_dir "$version")
lock_dir=$(lock_dir "$version")
EOF
}

do_list() {
  log_info "method=list"
  print_installed_versions
}

do_which() {
  log_info "method=which"
  local version
  version="$(requested_version 2>/dev/null || true)"
  [[ -n "$version" ]] || {
    echo "which requires --node-version" >&2
    exit 64
  }
  print_which "$version"
}

do_remote_list() {
  log_info "method=remote-list mirror=$NODE_MIRROR"
  python3 - <<'PY' "$NODE_MIRROR"
import json, sys, urllib.request
mirror = sys.argv[1].rstrip('/')
with urllib.request.urlopen(mirror + '/index.json', timeout=30) as resp:
    data = json.load(resp)
for item in data:
    version = item.get('version', '')
    if version.startswith('v'):
        version = version[1:]
    if version:
        print(version)
PY
}

do_install() {
  log_info "method=install"
  resolve_node_tool
  acquire_lock "$NODE_VERSION"
  download_node_runtime "$NODE_VERSION"
  resolve_manager_tools
  prepare_version_dirs
  run_node_home_op "install downloaded runtime" cp -R "$(runtime_dir "$NODE_VERSION")/extracted/." "$(version_dir "$NODE_VERSION")"
  link_versions
  prepare_corepack_shims
  emit_patch "$NODE_VERSION"
}

do_snapshot() {
  log_info "method=snapshot"
  resolve_node_tool
  if installed_version_exists "$NODE_VERSION"; then
    NODE_BIN="$(bin_dir "$NODE_VERSION")/node"
    NPM_BIN="$(bin_dir "$NODE_VERSION")/npm"
    NPM_VERSION="$(resolve_tool_version "$NPM_BIN")"
  else
    download_node_runtime "$NODE_VERSION"
    resolve_manager_tools
  fi
  print_snapshot "$NODE_VERSION"
}

do_uninstall() {
  log_info "method=uninstall"
  local version
  version="$(requested_version 2>/dev/null || true)"
  local output
  if [[ -n "$version" ]]; then
    local version_root
    version_root="$(version_dir "$version")"
    [[ -e "$version_root" ]] || {
      echo "node helper version is already absent: $version_root" >&2
      exit 0
    }
    if ! output="$(rm -rf "$version_root" 2>&1)"; then
      node_home_error "remove version root" "$output"
    fi
    cat <<EOF
node helper uninstall ok
version=$version
version_root=$version_root
EOF
    return 0
  fi
  [[ -e "$NODE_HOME" ]] || {
    echo "node helper home is already absent: $NODE_HOME" >&2
    exit 0
  }
  if ! output="$(rm -rf "$NODE_HOME" 2>&1)"; then
    node_home_error "remove node helper home" "$output"
  fi
  cat <<EOF
node helper uninstall ok
node_home=$NODE_HOME
EOF
}

case "$METHOD" in
  install) do_install ;;
  list) do_list ;;
  which) do_which ;;
  remote-list) do_remote_list ;;
  snapshot) do_snapshot ;;
  uninstall) do_uninstall ;;
esac
