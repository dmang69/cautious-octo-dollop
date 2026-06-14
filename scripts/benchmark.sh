#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESULTS_DIR="$ROOT/benchmark-results/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "==> Running AI OS benchmarks — results in $RESULTS_DIR"

# ── Rust benchmarks ───────────────────────────────────────────────────────────
echo ""
echo "── Cargo benchmarks ──"
cargo bench --workspace 2>&1 | tee "$RESULTS_DIR/cargo-bench.txt"

# ── Inference latency ─────────────────────────────────────────────────────────
echo ""
echo "── Inference latency summary ──"
grep -E "inference_latency.*time:" "$RESULTS_DIR/cargo-bench.txt" || true

# ── Shell startup time ────────────────────────────────────────────────────────
SHELL_BIN="$ROOT/shell/tauri-app/src-tauri/target/release/ai-os-shell"
if [[ -f "$SHELL_BIN" ]]; then
  echo ""
  echo "── Shell startup time ──"
  for i in 1 2 3; do
    { time "$SHELL_BIN" --headless --exit-after-startup; } 2>&1 | grep real
  done | tee "$RESULTS_DIR/shell-startup.txt"
fi

echo ""
echo "==> Benchmarks complete. Results saved to $RESULTS_DIR"
