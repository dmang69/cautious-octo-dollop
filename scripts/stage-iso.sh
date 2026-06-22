#!/usr/bin/env bash
# Stage IntentKernel binaries and config for bootable ISO / offline media.
# Usage: ./scripts/stage-iso.sh [--no-build] [--iso-root PATH]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${INTENTKERNEL_VERSION:-$(jq -r .version releases/manifest.json 2>/dev/null || echo "1.0.0")}"
STAGE="$ROOT/dist/IntentKernel"
ISO_ROOT="${INTENTKERNEL_ISO_ROOT:-$HOME/IntentKernelISO}"
NO_BUILD=0

usage() {
  echo "Usage: $0 [--no-build] [--iso-root PATH]"
  echo "  Stages dist/IntentKernel/ and copies to \$INTENTKERNEL_ISO_ROOT/IntentKernel/"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-build) NO_BUILD=1; shift ;;
    --iso-root) ISO_ROOT="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

# Workspace binaries to ship on ISO media (leasebroker optional).
PACKAGES=(
  ai-runtime
  intent-verifier
  intentkernel-cli
  intentkernel-update
  ikd-verify
  capd
  intentd
  leasebroker
  intentkernel-sh
  eventscope
)

# Cargo package -> staged filename (when they differ).
declare -A BIN_NAMES=(
  [intentkernel-cli]=intentkernel
  [intentkernel-sh]=iksh
)

if [[ "$NO_BUILD" -eq 0 ]]; then
  echo "==> Building release workspace binaries..."
  BUILD_ARGS=()
  for pkg in "${PACKAGES[@]}"; do
    BUILD_ARGS+=(-p "$pkg")
  done
  cargo build --release "${BUILD_ARGS[@]}"

  if [[ -f "$ROOT/kernel/build_parser_wasm.sh" ]]; then
    echo "==> Building WASM intent parser..."
    bash "$ROOT/kernel/build_parser_wasm.sh"
  fi
else
  echo "==> Skipping cargo build (--no-build)"
fi

echo "==> Staging to $STAGE"
rm -rf "$STAGE"
mkdir -p "$STAGE/bin" "$STAGE/config" "$STAGE/share/"{wasm,proto,dashboard,brand}

stage_binary() {
  local name="$1"
  local src="$ROOT/target/release/$name"
  if [[ ! -f "$src" ]]; then
    echo "    WARN: missing binary $src (skipped)" >&2
    return 0
  fi
  install -m 755 "$src" "$STAGE/bin/$name"
  echo "    + bin/$name"
}

for pkg in "${PACKAGES[@]}"; do
  bin="${BIN_NAMES[$pkg]:-$pkg}"
  stage_binary "$bin"
done

# Config tree.
if [[ -d "$ROOT/config" ]]; then
  cp -a "$ROOT/config/." "$STAGE/config/"
  echo "    + config/"
fi

# Share assets.
if [[ -f "$ROOT/build/wasm/intent_parser.wasm" ]]; then
  cp "$ROOT/build/wasm/intent_parser.wasm" "$STAGE/share/wasm/"
  echo "    + share/wasm/intent_parser.wasm"
fi
if [[ -f "$ROOT/core/ai-runtime/proto/intentkernel.proto" ]]; then
  cp "$ROOT/core/ai-runtime/proto/intentkernel.proto" "$STAGE/share/proto/"
  echo "    + share/proto/intentkernel.proto"
fi
if [[ -f "$ROOT/share/dashboard/index.html" ]]; then
  cp "$ROOT/share/dashboard/index.html" "$STAGE/share/dashboard/"
  echo "    + share/dashboard/index.html"
fi
for logo in intent-kernel-logo.png intent-kernel-logo-dark.png; do
  if [[ -f "$ROOT/share/brand/$logo" ]]; then
    cp "$ROOT/share/brand/$logo" "$STAGE/share/brand/"
    echo "    + share/brand/$logo"
  fi
done

printf '%s\n' "$VERSION" >"$STAGE/VERSION"
echo "    + VERSION ($VERSION)"

echo "==> Copying staged tree to $ISO_ROOT/IntentKernel"
mkdir -p "$ISO_ROOT"
rm -rf "$ISO_ROOT/IntentKernel"
cp -a "$STAGE" "$ISO_ROOT/IntentKernel"

# Live boot helper + GRUB menu (version-controlled in repo).
if [[ -f "$ROOT/live/intentkernel/autorun.sh" ]]; then
  mkdir -p "$ISO_ROOT/live/intentkernel"
  install -m 755 "$ROOT/live/intentkernel/autorun.sh" "$ISO_ROOT/live/intentkernel/autorun.sh"
  echo "    + live/intentkernel/autorun.sh"
fi
if [[ -f "$ROOT/boot/grub/grub.cfg" ]]; then
  mkdir -p "$ISO_ROOT/boot/grub"
  cp "$ROOT/boot/grub/grub.cfg" "$ISO_ROOT/boot/grub/grub.cfg"
  echo "    + boot/grub/grub.cfg"
fi

echo ""
echo "✓ ISO media staged"
echo "  dist:     $STAGE"
echo "  iso root: $ISO_ROOT/IntentKernel"
echo ""
echo "Verify:"
echo "  INTENTKERNEL_ISO_ROOT=$ISO_ROOT $ROOT/target/release/ikd-verify --kernel-check --os linux"
echo "Build ISO:"
echo "  ./scripts/build-iso.sh"