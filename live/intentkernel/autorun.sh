#!/usr/bin/env bash
# IntentKernel live-demo service launcher (ISO / offline media).
# Starts ai-runtime, intent-verifier, then intentd orchestrator.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ISO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
IK_ROOT="${INTENTKERNEL_ROOT:-${INTENTKERNEL_ISO_ROOT:+${INTENTKERNEL_ISO_ROOT}/IntentKernel}}"
IK_ROOT="${IK_ROOT:-$ISO_ROOT/IntentKernel}"
BIN="$IK_ROOT/bin"

log() { printf '[autorun] %s\n' "$*"; }

if [[ ! -d "$BIN" ]]; then
  log "ERROR: IntentKernel tree not found at $IK_ROOT"
  log "Set INTENTKERNEL_ROOT or run scripts/stage-iso.sh"
  exit 1
fi

export PATH="$BIN:$PATH"
export INTENTKERNEL_ROOT="$IK_ROOT"

PIDS=()
cleanup() {
  log "Shutting down services..."
  for pid in "${PIDS[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT INT TERM

log "ISO root: $ISO_ROOT"
log "Install root: $IK_ROOT"

for svc in ai-runtime intent-verifier; do
  if [[ ! -x "$BIN/$svc" ]]; then
    log "ERROR: missing $BIN/$svc"
    exit 1
  fi
done

log "Starting ai-runtime..."
"$BIN/ai-runtime" &
PIDS+=($!)

log "Starting intent-verifier..."
"$BIN/intent-verifier" &
PIDS+=($!)

sleep 2

if [[ -x "$BIN/intentd" ]]; then
  log "Starting intentd..."
  exec "$BIN/intentd" start --install-root "$IK_ROOT"
else
  log "intentd not present — ai-runtime and intent-verifier running (PIDs: ${PIDS[*]})"
  wait
fi