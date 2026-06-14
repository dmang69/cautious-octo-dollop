#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "==> Building all AI OS components from $ROOT"

# ── Core (Rust) ──────────────────────────────────────────────────────────────
echo ""
echo "── Building core/kernel-interface ──"
cargo build --release --manifest-path "$ROOT/core/kernel-interface/Cargo.toml"

echo ""
echo "── Building core/ipc ──"
cargo build --release --manifest-path "$ROOT/core/ipc/Cargo.toml"

echo ""
echo "── Building core/ai-runtime ──"
cargo build --release --manifest-path "$ROOT/core/ai-runtime/Cargo.toml"

# ── Shell (Tauri + React) ────────────────────────────────────────────────────
echo ""
echo "── Building shell/tauri-app ──"
(
  cd "$ROOT/shell/tauri-app"
  npm ci --prefer-offline
  npm run tauri:build
)

echo ""
echo "==> All components built successfully."
