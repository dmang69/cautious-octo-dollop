#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/aios"
DATA_DIR="/var/lib/aios"
LOG_DIR="/var/log/aios"
SERVICE_FILE="systemd/ai-context-manager.service"
SERVICE_DEST="/etc/systemd/system/ai-context-manager.service"
BINARY="../../core/ai-runtime/target/release/ai-runtime"

echo "Installing AI OS Context Manager..."

# Build binary if not present
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
if [[ -d "../../models/pretrained" ]]; then
    install -m 644 ../../models/pretrained/*.onnx "$DATA_DIR/" 2>/dev/null || true
fi

# Create service user
id aios &>/dev/null || useradd --system --no-create-home --shell /sbin/nologin aios
chown -R aios:aios "$DATA_DIR" "$LOG_DIR"

# Install and enable systemd service
install -m 644 "$SERVICE_FILE" "$SERVICE_DEST"
systemctl daemon-reload
systemctl enable ai-context-manager.service
systemctl start ai-context-manager.service

echo "Installation complete. Status:"
systemctl status ai-context-manager.service --no-pager
