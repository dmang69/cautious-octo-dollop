"""
IntentKernel capability lifecycle — in-memory reference implementation.
"""

from __future__ import annotations

from dataclasses import dataclass, field
import hmac
import json
import secrets
import threading
import time
from typing import Any


@dataclass
class Capability:
    cap_id: str
    cap_type: str
    expires_at: float
    uses_left: int
    key: bytes
    issued_at: float
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_payload(self) -> bytes:
        payload = {
            "id": self.cap_id,
            "type": self.cap_type,
            "expires_at": round(self.expires_at, 6),
            "uses_left": self.uses_left,
            "issued_at": round(self.issued_at, 6),
            "key": self.key.hex(),
            "metadata": self.metadata,
        }
        return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


_cap_table: dict[str, Capability] = {}
_lock = threading.RLock()


def _now() -> float:
    return time.monotonic()


def create_capability(cap_type: str, ttl_ms: int, uses: int,
                      metadata: dict[str, Any] | None = None) -> Capability:
    issued_at = _now()
    expires_at = issued_at + (ttl_ms / 1000.0)
    cap_id = secrets.token_hex(8)
    cap = Capability(
        cap_id=cap_id,
        cap_type=cap_type,
        expires_at=expires_at,
        uses_left=max(1, int(uses)),
        key=secrets.token_bytes(32),
        issued_at=issued_at,
        metadata=metadata or {},
    )
    with _lock:
        _cap_table[cap_id] = cap
    return cap


def get_capability(cap_id: str) -> Capability | None:
    with _lock:
        return _cap_table.get(cap_id)


def is_expired(cap: Capability) -> bool:
    return _now() >= cap.expires_at


def validate_and_consume(presented: Capability) -> tuple[bool, str]:
    with _lock:
        stored = _cap_table.get(presented.cap_id)
        if stored is None:
            return False, "unknown_capability"
        if is_expired(stored):
            return False, "expired"
        if stored.uses_left <= 0:
            return False, "exhausted"
        if not hmac.compare_digest(stored.key, presented.key):
            return False, "invalid_key"
        stored.uses_left -= 1
        return True, "ok"


def revoke(cap_id: str) -> bool:
    with _lock:
        return _cap_table.pop(cap_id, None) is not None


def prune_expired() -> int:
    now = _now()
    removed = 0
    with _lock:
        for cap_id in list(_cap_table.keys()):
            if _cap_table[cap_id].expires_at <= now:
                _cap_table.pop(cap_id, None)
                removed += 1
    return removed


def snapshot() -> list[Capability]:
    with _lock:
        return list(_cap_table.values())
