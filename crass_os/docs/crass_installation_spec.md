# CRASS OS Installation & Upgrade Specification

## Overview

CRASS OS is designed to be installed from a single, unified distribution package. The installer, called **CRASS LAUNCH**, must provide a frictionless experience for:

- Windows users with `.exe`-style launchers
- macOS users with `.dmg` or signed installer packages
- Linux users with shell-driven package installation
- Legacy hardware through bootable USB creation

## Installer Requirements

The installer must:

- auto-detect the host platform
- detect A/B partitions and bootloaders
- offer dual-boot or full migration options
- preserve user files, profiles, and network settings
- support legacy hardware by creating bootable media

## Migration Requirements

The migration engine, **CRASS MIGRATE**, must transfer:

- user documents, media, and desktop files
- application preferences and system profiles
- network configurations and Wi-Fi credentials
- display settings and locale preferences

## Upgrade Requirements

The updater, **CRASS CORE**, must provide:

- silent rolling updates in the background
- one-click rollback to a previous working state
- security patch deployment within hours of release
- isolated update application to avoid OS corruption

## Platform-Specific Notes

- **Windows**: support UEFI and secure boot; fallback to legacy boot when needed.
- **macOS**: support Intel and Apple Silicon; preserve the native boot process.
- **Linux**: support Ubuntu, Fedora, Arch, Debian, and derivative distros.
- **Legacy Hardware**: provide one-click USB creation and a simple recovery path.

## User Experience

The installation flow should be:

1. Welcome & system scan
2. Installation mode selection
3. Migration preview and restore points
4. Progress dashboard with live status
5. Final reboot into CRASS OS

## Compatibility

CRASS OS should act as both a replacement and an upgrade path for existing systems, enabling users to migrate without requiring deep technical expertise.
