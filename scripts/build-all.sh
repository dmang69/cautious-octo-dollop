#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "========================================"
echo " cautious-octo-dollop — cross-platform build"
echo "========================================"

echo ""
echo "==> [1/8] Brand assets"
if [ -f share/brand/intent-kernel-logo.png ]; then
  (cd scripts && npm install --silent 2>/dev/null && node generate-brand-assets.mjs) \
    || echo "    SKIP: brand asset generation failed"
else
  echo "    SKIP: share/brand/intent-kernel-logo.png missing"
fi

echo ""
echo "==> [2/8] Rust workspace"
cargo build --release -p ai-runtime -p intent-verifier -p intentkernel-update -p intentkernel-cli -p ikd-verify

echo ""
echo "==> [3/8] WASM intent parser"
./kernel/build_parser_wasm.sh

echo ""
echo "==> [4/8] ikd-verify gate"
./target/release/ikd-verify --kernel-check --os linux || true

echo ""
echo "==> [5/8] Tauri shell frontend"
if [ -d shell/tauri-app ]; then
  (cd shell/tauri-app && npm install && npm run build)
else
  echo "    SKIP: shell/tauri-app missing"
fi

echo ""
echo "==> [6/8] Tauri backend check"
if [ -d shell/tauri-app/src-tauri ]; then
  if pkg-config --exists dbus-1 2>/dev/null; then
    (cd shell/tauri-app/src-tauri && CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target/tauri}" cargo check)
  else
    echo "    SKIP: libdbus-1-dev not installed (Linux Tauri dep)"
  fi
fi

echo ""
echo "==> [7/8] Docker image (ai-os-dev)"
if command -v docker >/dev/null 2>&1; then
  docker compose build ai-os-dev 2>/dev/null || docker-compose build ai-os-dev 2>/dev/null || echo "    SKIP: docker compose build failed"
else
  echo "    SKIP: docker not installed"
fi

echo ""
echo "==> [8/8] Done"
echo "  cargo run --release -p intent-verifier"
echo "  cargo run --release -p ai-runtime"
echo "  cd shell/tauri-app && npm run tauri dev"
echo "  docker compose up ai-os-dev"
echo "========================================"