#!/usr/bin/env bash
set -euo pipefail

IMAGE="${RUNSEAL_E2E_IMAGE:-ubuntu:24.04}"
PLATFORM="${RUNSEAL_E2E_PLATFORM:-linux/amd64}"
CPU_LIMIT="${RUNSEAL_E2E_CPUS:-1}"
MEM_LIMIT="${RUNSEAL_E2E_MEMORY:-1g}"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'EOF'
Usage: smoke.sh <command>

Commands:
  smoke     Run one-shot install/run/uninstall smoke in Linux container
  exec ...  Run one-shot custom shell command in Linux container

Environment overrides:
  RUNSEAL_E2E_IMAGE     Container image (default: ubuntu:24.04)
  RUNSEAL_E2E_PLATFORM  Docker platform (default: linux/amd64)
  RUNSEAL_E2E_CPUS      CPU limit (default: 1)
  RUNSEAL_E2E_MEMORY    Memory limit (default: 1g)
  RUNSEAL_E2E_VERSION   Optional release version passed to install.sh
EOF
}

ensure_docker() {
  if ! command -v docker >/dev/null 2>&1; then
    echo "missing required command: docker" >&2
    exit 1
  fi
}

docker_shell() {
  ensure_docker
  docker run --rm \
    --platform "${PLATFORM}" \
    --cpus "${CPU_LIMIT}" \
    --memory "${MEM_LIMIT}" \
    -v "${REPO_DIR}:/workspace/runseal" \
    -w /workspace/runseal \
    "${IMAGE}" \
    sh -lc "$*"
}

run_smoke() {
  local install_version_arg=""
  if [[ -n "${RUNSEAL_E2E_VERSION:-}" ]]; then
    install_version_arg="--version ${RUNSEAL_E2E_VERSION}"
  fi

  docker_shell "set -eu
apt-get update >/dev/null
apt-get install -y --no-install-recommends bash curl ca-certificates tar coreutils >/dev/null

rm -rf /root/.runseal /root/.local/bin/runseal /tmp/runseal-profile.json /tmp/runseal-preview.txt
bash /workspace/runseal/scripts/manage/install.sh ${install_version_arg}

cat > /tmp/runseal-profile.json <<'JSON'
{\"injections\":[{\"type\":\"env\",\"vars\":{\"RUNSEAL_E2E\":\"ok\"}}]}
JSON

/root/.local/bin/runseal --output json -p /tmp/runseal-profile.json > /tmp/runseal-out.json
grep -q '\"RUNSEAL_E2E\": \"ok\"' /tmp/runseal-out.json

if /root/.local/bin/runseal preview --help >/dev/null 2>&1; then
  /root/.local/bin/runseal preview --profile /tmp/runseal-profile.json --output text > /tmp/runseal-preview.txt
  grep -q 'RUNSEAL_E2E' /tmp/runseal-preview.txt
fi

bash /workspace/runseal/scripts/manage/uninstall.sh
test ! -e /root/.local/bin/runseal
echo 'smoke passed in Linux container'"
}

run_exec() {
  if [[ $# -eq 0 ]]; then
    echo "exec requires a command string" >&2
    exit 1
  fi
  docker_shell "$*"
}

main() {
  local cmd="${1:-}"
  case "${cmd}" in
    smoke)
      run_smoke
      ;;
    exec)
      shift
      run_exec "$@"
      ;;
    -h|--help|help|"")
      usage
      ;;
    *)
      echo "unknown command: ${cmd}" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"
