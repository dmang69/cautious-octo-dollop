#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/aios"
DATA_DIR="/var/lib/aios"
LOG_DIR="/var/log/aios"
PLIST_SRC="launchd/com.aios.context-manager.plist"
PLIST_DEST="/Library/LaunchDaemons/com.aios.context-manager.plist"
BINARY="../../core/ai-runtime/target/release/ai-runtime"

echo "Installing AI OS Context Manager for macOS..."

if [[ ! -f "$BINARY" ]]; then
    echo "Building ai-runtime..."
    (cd ../../core/ai-runtime && cargo build --release)
fi

install -m 755 "$BINARY" "$INSTALL_DIR/ai-runtime"
mkdir -p "$CONFIG_DIR" "$DATA_DIR" "$LOG_DIR"

if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
    install -m 644 ../../core/ai-runtime/config.toml "$CONFIG_DIR/config.toml"
fi

# Copy ONNX models
install -m 644 ../../models/pretrained/*.onnx "$DATA_DIR/" 2>/dev/null || true

# Create service user
dscl . -read /Users/_aios &>/dev/null || \
    dscl . -create /Users/_aios UserShell /usr/bin/false

chown -R _aios "$DATA_DIR" "$LOG_DIR"

# Install and load launchd daemon
install -m 644 "$PLIST_SRC" "$PLIST_DEST"
launchctl load "$PLIST_DEST"

echo "Installation complete."
launchctl list | grep aios
