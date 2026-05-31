# CRASS OS

CRASS OS is a universal, platform-agnostic operating system deployment and upgrade framework.
Its goal is to make installation, migration, and system upgrades simple across Windows, macOS, Linux, and legacy hardware.

## What is included

- `installer/crass_launch.py` — unified installer launcher for all supported platforms
- `installer/crass_launch.sh` — Linux/macOS installer shell script
- `installer/crass_launch.ps1` — Windows installer PowerShell wrapper
- `migration/crass_migrate.py` — migration engine for user files and settings
- `updater/crass_core_updater.py` — update engine for CRASS CORE
- `docs/crass_installation_spec.md` — deployment and upgrade specification

## Install Workflow

1. Download the CRASS LAUNCH package.
2. Run the installer for your platform.
3. Choose clean install, dual-boot, or migration mode.
4. CRASS CORE applies background updates automatically after install.

## Upgrade and Migration

- `CRASS MIGRATE` scans the existing system and transfers important data safely.
- `CRASS CORE` provides one-click rollback and silent rolling updates.
- The system is designed for incremental adoption across existing legacy and modern systems.

## Getting Started

```bash
python3 crass_os/installer/crass_launch.py --help
```

For Windows, run the PowerShell wrapper:

```powershell
.\crass_os\installer\crass_launch.ps1
```
