# macOS Platform — AI OS Context Manager

Installs the AI Context Manager as a launchd daemon on macOS.

## Prerequisites

- Xcode Command Line Tools (`xcode-select --install`)
- Rust toolchain (`rustup`)
- Administrator rights (`sudo`)

## Installation

```bash
sudo ./install.sh
```

## Service Management

```bash
sudo launchctl list | grep aios
sudo launchctl stop com.aios.context-manager
sudo launchctl start com.aios.context-manager
tail -f /var/log/aios/ai-runtime.log
```

## Uninstallation

```bash
sudo launchctl unload /Library/LaunchDaemons/com.aios.context-manager.plist
sudo rm /Library/LaunchDaemons/com.aios.context-manager.plist
sudo rm /usr/local/bin/ai-runtime
```
