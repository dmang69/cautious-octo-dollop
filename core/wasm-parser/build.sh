#!/bin/bash
# WASM Parser Build Script
# Compiles intent parser to WASM for sandboxed execution

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WASM_DIR="$SCRIPT_DIR"
OUTPUT_DIR="$PROJECT_ROOT/core/kernel-interface/src"

echo "🔨 Building WASM Intent Parser..."
echo "Working directory: $WASM_DIR"
echo "Output directory: $OUTPUT_DIR"

# Check for wasm32-wasip1 target
if ! rustup target list --installed | grep -q wasm32-wasip1; then
    echo "📥 Installing wasm32-wasip1 target..."
    rustup target add wasm32-wasip1
fi

# Build parser to WASM
cd "$WASM_DIR"

echo "🔧 Compiling parser.rs to WASM..."
cargo build --target wasm32-wasip1 --release

# Copy WASM binary to kernel-interface
WASM_FILE="$WASM_DIR/target/wasm32-wasip1/release/intent_parser.wasm"

if [ -f "$WASM_FILE" ]; then
    cp "$WASM_FILE" "$OUTPUT_DIR/parser.wasm"
    echo "✅ WASM parser built successfully: $OUTPUT_DIR/parser.wasm"
    
    # Show size
    SIZE=$(du -h "$OUTPUT_DIR/parser.wasm" | cut -f1)
    echo "📊 WASM binary size: $SIZE"
    
    # Optimize with wasm-opt if available
    if command -v wasm-opt &> /dev/null; then
        echo "🔧 Optimizing WASM binary..."
        wasm-opt -Oz "$OUTPUT_DIR/parser.wasm" -o "$OUTPUT_DIR/parser.opt.wasm"
        mv "$OUTPUT_DIR/parser.opt.wasm" "$OUTPUT_DIR/parser.wasm"
        
        OPT_SIZE=$(du -h "$OUTPUT_DIR/parser.wasm" | cut -f1)
        echo "✅ Optimized size: $OPT_SIZE"
    fi
else
    echo "❌ Error: WASM file not found at $WASM_FILE"
    exit 1
fi

echo "✅ WASM parser build complete!"
