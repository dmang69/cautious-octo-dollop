# Linux Platform — AI OS Context Manager

Installs and configures the AI Context Manager as a systemd service on Linux.

## Prerequisites

- Rust toolchain (`rustup`)
- `systemd` (most modern Linux distros)
- Root / sudo access

## Installation

```bash
sudo ./install.sh
```

## Service Management

```bash
sudo systemctl status ai-context-manager
sudo journalctl -u ai-context-manager -f
sudo systemctl restart ai-context-manager
```

## Uninstallation

```bash
sudo systemctl stop ai-context-manager
sudo systemctl disable ai-context-manager
sudo rm /etc/systemd/system/ai-context-manager.service
sudo systemctl daemon-reload
sudo rm /usr/local/bin/ai-runtime
```
