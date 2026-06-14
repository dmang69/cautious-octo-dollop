#!/usr/bin/env bash
# start_services.sh — Start all IntentKernel v1 daemons
#
# Usage:
#   cd platform
#   ./start_services.sh          # start all three daemons in background
#   ./start_services.sh --stop   # stop all running daemons

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_DIR="${HOME}/.intentos/pids"
mkdir -p "${PID_DIR}"

INTENTD_PID="${PID_DIR}/intentd.pid"
CAPD_PID="${PID_DIR}/capd.pid"
IP_DESC_PID="${PID_DIR}/ip_descramblerd.pid"

start_daemon() {
  local name="$1"
  local module="$2"
  local pidfile="$3"

  if [ -f "${pidfile}" ] && kill -0 "$(cat "${pidfile}")" 2>/dev/null; then
    echo "  ${name} already running (pid=$(cat "${pidfile}"))"
    return
  fi

  python -m "${module}" &
  echo $! > "${pidfile}"
  echo "  ${name} started (pid=$(cat "${pidfile}"))"
}

stop_daemon() {
  local name="$1"
  local pidfile="$2"

  if [ -f "${pidfile}" ]; then
    pid=$(cat "${pidfile}")
    if kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" && echo "  ${name} stopped (pid=${pid})"
    else
      echo "  ${name} not running"
    fi
    rm -f "${pidfile}"
  else
    echo "  ${name} — no PID file found"
  fi
}

if [ "${1:-}" = "--stop" ]; then
  echo ""
  echo "  Stopping IntentKernel services …"
  echo "  ─────────────────────────────────"
  (cd "${SCRIPT_DIR}" && stop_daemon intentd          "${INTENTD_PID}")
  (cd "${SCRIPT_DIR}" && stop_daemon capd              "${CAPD_PID}")
  (cd "${SCRIPT_DIR}" && stop_daemon ip-descramblerd   "${IP_DESC_PID}")
  echo ""
else
  echo ""
  echo "  Starting IntentKernel services …"
  echo "  ──────────────────────────────────────────────────"
  (cd "${SCRIPT_DIR}" && start_daemon intentd          intentd         "${INTENTD_PID}")
  (cd "${SCRIPT_DIR}" && start_daemon capd              capd            "${CAPD_PID}")
  (cd "${SCRIPT_DIR}" && start_daemon ip-descramblerd   ip_descramblerd "${IP_DESC_PID}")
  echo ""
  echo "  Services running:"
  echo "    intentd          → http://127.0.0.1:5001"
  echo "    capd             → http://127.0.0.1:5002"
  echo "    ip-descramblerd  → http://127.0.0.1:5003"
  echo ""
  echo "  Demo:"
  echo "    python demo/secure_curl.py http://example.com --verbose"
  echo ""
  echo "  Stop with:  ./start_services.sh --stop"
  echo ""
fi
