#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/build/wasm"
TARGET=wasm32-unknown-unknown
rustup target add "$TARGET" 2>/dev/null || true
cd "$ROOT/kernel/parser-wasm"
cargo build --release --target "$TARGET"
mkdir -p "$OUT"
cp "$ROOT/target/$TARGET/release/parser_wasm.wasm" "$OUT/intent_parser.wasm"
echo "WASM: $OUT/intent_parser.wasm"