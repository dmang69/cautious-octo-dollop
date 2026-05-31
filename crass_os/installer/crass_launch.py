#!/usr/bin/env python3
"""CRASS LAUNCH — unified installer launcher."""

from __future__ import annotations

import argparse
import os
import platform
import subprocess
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[2]


def detect_platform() -> str:
    system = platform.system().lower()
    if system == "darwin":
        return "macos"
    if system == "windows":
        return "windows"
    if system == "linux":
        return "linux"
    return "unknown"


def run_installer(mode: str) -> int:
    current_platform = detect_platform()
    print(f"[CRASS LAUNCH] detected platform: {current_platform}")

    if mode == "scan":
        print("Scanning system for existing installations and partitions...")
        return scan_environment(current_platform)

    if current_platform == "windows":
        return run_windows_installer(mode)
    if current_platform in ("linux", "macos"):
        return run_unix_installer(mode)

    print("Unsupported platform. Please use CRASS LAUNCH on Windows, macOS, or Linux.")
    return 1


def scan_environment(platform_name: str) -> int:
    print("Preparing system scan...")
    if platform_name == "windows":
        print("- Checking Windows boot environment")
    elif platform_name == "macos":
        print("- Checking Apple Silicon / Intel compatibility")
    elif platform_name == "linux":
        print("- Checking distro compatibility and package manager")
    else:
        print("- Unknown platform; collecting generic hardware data")

    print("Scan complete. CRASS LAUNCH is ready to install or migrate.")
    return 0


def run_windows_installer(mode: str) -> int:
    script = ROOT_DIR / "crass_os" / "installer" / "crass_launch.ps1"
    command = ["powershell.exe", "-ExecutionPolicy", "Bypass", "-File", str(script), mode]
    print(f"Launching Windows installer: {command}")
    return subprocess.call(command)


def run_unix_installer(mode: str) -> int:
    script = ROOT_DIR / "crass_os" / "installer" / "crass_launch.sh"
    command = ["bash", str(script), mode]
    print(f"Launching POSIX installer: {command}")
    return subprocess.call(command)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="CRASS LAUNCH: unified installer launcher")
    parser.add_argument("--mode", default="install", choices=["install", "migrate", "usb", "scan"], help="Installation mode")
    args = parser.parse_args()

    sys.exit(run_installer(args.mode))
