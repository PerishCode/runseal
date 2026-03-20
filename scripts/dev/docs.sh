#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SESSION_NAME="${RUNSEAL_DEV_DOCS_SESSION:-runseal-docs-dev}"
SESSION_COMMAND="${RUNSEAL_DEV_DOCS_COMMAND:-pnpm run docs:dev}"
LOG_LINES="${RUNSEAL_DEV_DOCS_LOG_LINES:-200}"

usage() {
  cat <<EOF
usage: docs.sh <start|stop|restart|logs|status>

environment:
  RUNSEAL_DEV_DOCS_SESSION    tmux session name (default: ${SESSION_NAME})
  RUNSEAL_DEV_DOCS_COMMAND    command to run in session (default: ${SESSION_COMMAND})
  RUNSEAL_DEV_DOCS_LOG_LINES  log lines shown by logs (default: ${LOG_LINES})
EOF
}

require_tmux() {
  command -v tmux >/dev/null 2>&1 || {
    echo "tmux is required" >&2
    exit 127
  }
}

session_exists() {
  tmux has-session -t "$SESSION_NAME" 2>/dev/null
}

start_session() {
  if session_exists; then
    echo "session already running: $SESSION_NAME"
    return 0
  fi

  tmux new-session -d -s "$SESSION_NAME" -c "$REPO_DIR" "$SESSION_COMMAND"
  echo "started session: $SESSION_NAME"
}

stop_session() {
  if ! session_exists; then
    echo "session not running: $SESSION_NAME"
    return 0
  fi

  tmux kill-session -t "$SESSION_NAME"
  echo "stopped session: $SESSION_NAME"
}

show_logs() {
  if ! session_exists; then
    echo "session not running: $SESSION_NAME" >&2
    exit 1
  fi

  tmux capture-pane -p -S "-${LOG_LINES}" -t "$SESSION_NAME":0.0
}

show_status() {
  if ! session_exists; then
    echo "status: stopped"
    echo "session: $SESSION_NAME"
    return 0
  fi

  local pane_pid pane_cmd
  pane_pid="$(tmux display-message -p -t "$SESSION_NAME":0.0 '#{pane_pid}')"
  pane_cmd="$(tmux display-message -p -t "$SESSION_NAME":0.0 '#{pane_current_command}')"

  echo "status: running"
  echo "session: $SESSION_NAME"
  echo "configured_command: $SESSION_COMMAND"
  echo "pane_command: $pane_cmd"
  echo "pane_pid: $pane_pid"
}

main() {
  require_tmux

  case "${1:-}" in
    start)
      start_session
      ;;
    stop)
      stop_session
      ;;
    restart)
      stop_session
      start_session
      ;;
    logs)
      show_logs
      ;;
    status)
      show_status
      ;;
    -h|--help|help)
      usage
      ;;
    *)
      usage >&2
      exit 64
      ;;
  esac
}

main "$@"
