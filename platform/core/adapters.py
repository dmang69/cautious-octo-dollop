"""
Cross-platform adapter registry for IntentKernel MVP.
"""

from __future__ import annotations

from dataclasses import dataclass
import sys
from typing import Any


@dataclass
class Adapter:
    name: str
    description: str
    status: str
    supported_actions: list[str]

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "status": self.status,
            "supported_actions": self.supported_actions,
        }


ADAPTERS: dict[str, Adapter] = {
    "windows": Adapter(
        name="windows",
        description="Windows VBS service + micro-VM broker",
        status="planned",
        supported_actions=["network_request", "draw", "notify", "resource"],
    ),
    "linux": Adapter(
        name="linux",
        description="Linux LSM/eBPF broker and eventscope",
        status="mvp",
        supported_actions=["network_request", "draw", "notify", "resource"],
    ),
    "macos": Adapter(
        name="macos",
        description="macOS launchd broker + syscall interposer",
        status="planned",
        supported_actions=["network_request", "draw", "notify", "resource"],
    ),
    "android": Adapter(
        name="android",
        description="Privileged system service broker",
        status="planned",
        supported_actions=["network_request", "notify", "resource"],
    ),
    "ios": Adapter(
        name="ios",
        description="User-space broker + entitlement proxy",
        status="planned",
        supported_actions=["network_request", "notify", "resource"],
    ),
    "embedded": Adapter(
        name="embedded",
        description="Firmware supervisor + UCCS adapter",
        status="planned",
        supported_actions=["resource", "network_request"],
    ),
}


def detect_platform() -> str:
    if sys.platform.startswith("win"):
        return "windows"
    if sys.platform.startswith("linux"):
        return "linux"
    if sys.platform.startswith("darwin"):
        return "macos"
    return "unknown"


def get_adapter(name: str | None = None) -> Adapter | None:
    key = name or detect_platform()
    return ADAPTERS.get(key)


def list_adapters() -> list[dict[str, Any]]:
    return [adapter.to_dict() for adapter in ADAPTERS.values()]
