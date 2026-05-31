# CRASS OS Build & Deployment Framework

This document describes how to use the CRASS OS installer and upgrade framework.

## Prerequisites

- Python 3.11+
- PowerShell Core on Windows (or Windows PowerShell)
- `bash` on Linux/macOS

## Build / Run

### Linux / macOS

```bash
python3 crass_os/installer/crass_launch.py --mode install
```

### Windows

```powershell
.\crass_os\installer\crass_launch.ps1 -Mode install
```

## Components

- `installer/crass_launch.py` — cross-platform installer orchestration
- `migration/crass_migrate.py` — legacy system migration engine
- `updater/crass_core_updater.py` — automatic update engine

## Development

Use the scripts as stubs for the next phase of platform-specific packaging.

- Extend `crass_launch.py` into a packaged installer executable
- Add native `.dmg`, `.exe`, and USB boot image generation
- Integrate `crass_migrate.py` with user profile and credential vault migration
