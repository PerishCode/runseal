#!/usr/bin/env bash
set -euo pipefail

REPO="${ENVLOCK_INSTALL_REPO:-PerishCode/envlock}"
INSTALL_ROOT="${HOME}/.envlock"
BIN_DIR="${INSTALL_ROOT}/bin"
BIN_PATH="${BIN_DIR}/envlock"
LOCAL_BIN_DIR="${HOME}/.local/bin"
LINK_PATH="${LOCAL_BIN_DIR}/envlock"

usage() {
  cat <<'EOF'
Usage: install.sh [--version vX.Y.Z|X.Y.Z]

Install strategy:
  - binary: ~/.envlock/bin/envlock
  - symlink: ~/.local/bin/envlock -> ~/.envlock/bin/envlock
EOF
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "${os}:${arch}" in
    Linux:x86_64) echo "x86_64-unknown-linux-gnu" ;;
    Darwin:x86_64) echo "x86_64-apple-darwin" ;;
    Darwin:arm64|Darwin:aarch64) echo "aarch64-apple-darwin" ;;
    *)
      echo "unsupported platform: ${os}/${arch}" >&2
      exit 1
      ;;
  esac
}

fetch_latest_tag() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -nE 's/.*"tag_name":[[:space:]]*"([^"]+)".*/\1/p' \
    | head -n1
}

normalize_version() {
  local raw="$1"
  if [[ "${raw}" == v* ]]; then
    printf '%s\n' "${raw}"
  else
    printf 'v%s\n' "${raw}"
  fi
}

read_checksum() {
  local checksums_file="$1"
  local asset_name="$2"
  awk -v f="${asset_name}" '$2==f || $2=="*"f {print $1; exit}' "${checksums_file}"
}

verify_checksum() {
  local file="$1"
  local expected="$2"
  local actual=""
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "${file}" | awk '{print $1}')"
  else
    actual="$(shasum -a 256 "${file}" | awk '{print $1}')"
  fi
  if [[ "${actual}" != "${expected}" ]]; then
    echo "checksum mismatch: expected ${expected}, got ${actual}" >&2
    exit 1
  fi
}

VERSION=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_cmd curl
require_cmd tar
require_cmd mktemp

TARGET="$(detect_target)"
if [[ -z "${VERSION}" ]]; then
  VERSION="$(fetch_latest_tag)"
  if [[ -z "${VERSION}" ]]; then
    echo "failed to resolve latest release tag" >&2
    exit 1
  fi
else
  VERSION="$(normalize_version "${VERSION}")"
fi

ASSET="envlock-${VERSION}-${TARGET}.tar.gz"
if [[ -n "${ENVLOCK_INSTALL_BASE_URL:-}" ]]; then
  BASE_URL="${ENVLOCK_INSTALL_BASE_URL%/}/${VERSION}"
else
  BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
fi
ASSET_URL="${BASE_URL}/${ASSET}"
CHECKSUMS_URL="${BASE_URL}/checksums.txt"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

echo "Downloading ${ASSET_URL}"
curl -fsSL -o "${tmp_dir}/${ASSET}" "${ASSET_URL}"
curl -fsSL -o "${tmp_dir}/checksums.txt" "${CHECKSUMS_URL}"

expected="$(read_checksum "${tmp_dir}/checksums.txt" "${ASSET}")"
if [[ -z "${expected}" ]]; then
  echo "checksum not found for ${ASSET}" >&2
  exit 1
fi
verify_checksum "${tmp_dir}/${ASSET}" "${expected}"

mkdir -p "${BIN_DIR}" "${LOCAL_BIN_DIR}"
tar -xzf "${tmp_dir}/${ASSET}" -C "${tmp_dir}"
if [[ ! -f "${tmp_dir}/envlock" ]]; then
  echo "release archive missing envlock binary" >&2
  exit 1
fi

install -m 0755 "${tmp_dir}/envlock" "${BIN_PATH}"
ln -snf "${BIN_PATH}" "${LINK_PATH}"

echo "Installed envlock ${VERSION} to ${BIN_PATH}"
echo "Linked ${LINK_PATH} -> ${BIN_PATH}"
echo "Next: run envlock skill install --yes"
echo "Tip: set ENVLOCK_SKILL_INSTALL_HOME to override skill destination"
if [[ ":${PATH}:" != *":${LOCAL_BIN_DIR}:"* ]]; then
  echo "Note: ${LOCAL_BIN_DIR} is not in PATH."
  echo "Add it manually to your shell profile when convenient."
fi
