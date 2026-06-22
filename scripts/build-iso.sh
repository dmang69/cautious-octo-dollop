#!/usr/bin/env bash
# Build bootable IntentKernel.iso from staged media + live docs.
# Usage: ./scripts/build-iso.sh [--output PATH] [--iso-root PATH]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ISO_ROOT="${INTENTKERNEL_ISO_ROOT:-$HOME/IntentKernelISO}"
OUTPUT="${INTENTKERNEL_ISO_OUTPUT:-$ROOT/dist/IntentKernel.iso}"
STAGE="$ROOT/dist/IntentKernel"

usage() {
  echo "Usage: $0 [--output PATH] [--iso-root PATH]"
  echo "  Requires xorriso or grub-mkrescue. Run scripts/stage-iso.sh first."
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output) OUTPUT="$2"; shift 2 ;;
    --iso-root) ISO_ROOT="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

if [[ ! -d "$STAGE/bin" ]]; then
  echo "ERROR: staged tree missing at $STAGE — run ./scripts/stage-iso.sh first" >&2
  exit 1
fi

if [[ ! -d "$ISO_ROOT/boot/grub" ]]; then
  echo "ERROR: ISO root missing boot/grub at $ISO_ROOT" >&2
  exit 1
fi

WORK="$ROOT/dist/iso-work"
rm -rf "$WORK"
mkdir -p "$WORK"

echo "==> Assembling ISO tree in $WORK"
cp -a "$ISO_ROOT/boot" "$WORK/"
mkdir -p "$WORK/live"
if [[ -d "$ISO_ROOT/live/intentkernel" ]]; then
  cp -a "$ISO_ROOT/live/intentkernel" "$WORK/live/"
else
  mkdir -p "$WORK/live/intentkernel"
fi

# Prefer repo-managed autorun + grub when present.
if [[ -f "$ROOT/live/intentkernel/autorun.sh" ]]; then
  install -m 755 "$ROOT/live/intentkernel/autorun.sh" "$WORK/live/intentkernel/autorun.sh"
fi
if [[ -f "$ROOT/boot/grub/grub.cfg" ]]; then
  mkdir -p "$WORK/boot/grub"
  cp "$ROOT/boot/grub/grub.cfg" "$WORK/boot/grub/grub.cfg"
fi

cp -a "$STAGE" "$WORK/IntentKernel"

# Minimal El Torito boot image (GRUB rescue will replace when available).
if [[ ! -f "$WORK/boot/grub/eltorito.img" ]]; then
  mkdir -p "$WORK/boot/grub"
  # 512-byte placeholder; grub-mkrescue overwrites with a real boot image.
  dd if=/dev/zero of="$WORK/boot/grub/eltorito.img" bs=512 count=4 status=none 2>/dev/null || true
fi

mkdir -p "$(dirname "$OUTPUT")"
rm -f "$OUTPUT"

build_with_grub_mkrescue() {
  echo "==> Building ISO with grub-mkrescue -> $OUTPUT"
  grub-mkrescue -o "$OUTPUT" "$WORK" \
    --compress=xz \
    -V "IntentKernel" \
    2>/dev/null
}

build_with_xorriso() {
  echo "==> Building ISO with xorriso -> $OUTPUT"
  xorriso -as mkisofs \
    -iso-level 3 \
    -full-iso9660-filenames \
    -joliet \
    -rock \
    -volid "IntentKernel" \
    -output "$OUTPUT" \
    -graft-points \
    "$WORK" \
    -boot_image any partition_table=on \
    -boot_image any cat_path=/boot/grub/eltorito.img \
    -boot_image any boot_info_table=on \
    -boot_image any platform_id=0x00 \
    -boot_image any emul_type=no_emulation \
    -boot_image any load_size=2048
}

if command -v grub-mkrescue >/dev/null 2>&1; then
  if build_with_grub_mkrescue; then
    :
  elif command -v xorriso >/dev/null 2>&1; then
    echo "    grub-mkrescue failed; falling back to xorriso" >&2
    build_with_xorriso
  else
    echo "ERROR: grub-mkrescue failed and xorriso not installed" >&2
    exit 1
  fi
elif command -v xorriso >/dev/null 2>&1; then
  build_with_xorriso
else
  cat >&2 <<'EOF'
ERROR: ISO build tools not found.

Install one of:
  sudo apt install grub-pc-bin xorriso    # Debian/Ubuntu/WSL
  sudo dnf install grub2-tools xorriso    # Fedora

Staging still works without these tools:
  ./scripts/stage-iso.sh
EOF
  exit 1
fi

if [[ -f "$OUTPUT" ]]; then
  ls -lh "$OUTPUT"
  echo "✓ Bootable ISO: $OUTPUT"
else
  echo "ERROR: ISO was not created at $OUTPUT" >&2
  exit 1
fi