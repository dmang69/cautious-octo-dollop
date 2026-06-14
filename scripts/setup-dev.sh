#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "==> Setting up AI OS development environment"

# ── Rust ─────────────────────────────────────────────────────────────────────
if ! command -v rustup &>/dev/null; then
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
fi
rustup update stable
rustup component add rustfmt clippy

# ── Node.js ──────────────────────────────────────────────────────────────────
if ! command -v node &>/dev/null; then
  echo "Node.js not found. Please install Node.js 20+ from https://nodejs.org"
  exit 1
fi
node_version=$(node --version | sed 's/v//' | cut -d. -f1)
if [[ "$node_version" -lt 20 ]]; then
  echo "Node.js 20+ required (found v$node_version)"
  exit 1
fi

# ── ONNX Runtime ─────────────────────────────────────────────────────────────
ORT_VERSION="1.16.0"
ORT_DIR="$ROOT/.cache/onnxruntime-${ORT_VERSION}"
if [[ ! -d "$ORT_DIR" ]]; then
  echo "Downloading ONNX Runtime ${ORT_VERSION}..."
  ARCH="x64"
  OS_NAME="linux"
  if [[ "$OSTYPE" == "darwin"* ]]; then OS_NAME="osx"; fi
  URL="https://github.com/microsoft/onnxruntime/releases/download/v${ORT_VERSION}/onnxruntime-${OS_NAME}-${ARCH}-${ORT_VERSION}.tgz"
  mkdir -p "$ORT_DIR"
  curl -L "$URL" | tar -xzf - -C "$ORT_DIR" --strip-components=1
fi
export ORT_LIB_LOCATION="$ORT_DIR/lib"
echo "export ORT_LIB_LOCATION=\"$ORT_LIB_LOCATION\"" >> "$HOME/.bashrc"

# ── Node modules ─────────────────────────────────────────────────────────────
echo "Installing shell dependencies..."
(cd "$ROOT/shell/tauri-app" && npm install)

# ── Python (models) ──────────────────────────────────────────────────────────
if command -v pip3 &>/dev/null; then
  pip3 install -r "$ROOT/models/scheduler/requirements.txt"
fi

echo ""
echo "==> Development environment ready."
echo "    ORT_LIB_LOCATION=$ORT_LIB_LOCATION"
echo "    Run './scripts/build-all.sh' to build all components."
