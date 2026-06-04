"""
Intent Broker reference service (capd/intentd/leasebroker).
"""

from __future__ import annotations

from dataclasses import dataclass
import queue
from typing import Any

from . import capabilities, config as cfg, crypto, logger as log


@dataclass
class CapabilityToken:
    capability: capabilities.Capability
    signature: crypto.Signature
    intent: dict[str, Any]
    issued_by: str = "intentd"

    def to_dict(self) -> dict[str, Any]:
        return {
            "capability": {
                "id": self.capability.cap_id,
                "type": self.capability.cap_type,
                "expires_at": self.capability.expires_at,
                "uses_left": self.capability.uses_left,
                "issued_at": self.capability.issued_at,
                "metadata": self.capability.metadata,
                "key": self.capability.key.hex(),
            },
            "signature": {
                "algorithm": self.signature.algorithm,
                "key_id": self.signature.key_id,
                "value": self.signature.value,
            },
            "intent": self.intent,
            "issued_by": self.issued_by,
        }


_event_queue: queue.Queue[CapabilityToken] | None = None


def _queue() -> queue.Queue[CapabilityToken]:
    global _event_queue
    if _event_queue is None:
        size = int(cfg.get_section("intent_broker").get("queue_size", 128))
        _event_queue = queue.Queue(maxsize=size)
    return _event_queue


def issue_capability(intent: dict[str, Any], cap_type: str,
                     ttl_ms: int | None = None,
                     uses: int | None = None,
                     metadata: dict[str, Any] | None = None) -> CapabilityToken:
    broker_cfg = cfg.get_section("intent_broker")
    ttl = int(ttl_ms or broker_cfg.get("default_ttl_ms", 5000))
    max_ttl = int(broker_cfg.get("max_ttl_ms", 60000))
    uses_val = int(uses or broker_cfg.get("default_uses", 1))
    ttl = min(ttl, max_ttl)
    cap = capabilities.create_capability(cap_type, ttl, uses_val, metadata=metadata)
    signature = crypto.sign(cap.to_payload())
    token = CapabilityToken(capability=cap, signature=signature, intent=intent)
    log.audit("broker", "Capability issued", {"cap_id": cap.cap_id, "type": cap.cap_type})
    return token


def request_capability(intent: dict[str, Any], cap_type: str,
                       ttl_ms: int | None = None,
                       uses: int | None = None,
                       metadata: dict[str, Any] | None = None,
                       publish: bool = True) -> CapabilityToken:
    token = issue_capability(intent, cap_type, ttl_ms=ttl_ms, uses=uses, metadata=metadata)
    if publish:
        publish_capability(token)
    return token


def publish_capability(token: CapabilityToken) -> None:
    try:
        _queue().put(token, block=False)
    except queue.Full:
        log.warn("broker", "Capability queue full; dropping token", {"cap_id": token.capability.cap_id})


def await_capability(timeout_s: float | None = None) -> CapabilityToken | None:
    try:
        return _queue().get(timeout=timeout_s)
    except queue.Empty:
        return None


def sweep_expired() -> int:
    removed = capabilities.prune_expired()
    if removed:
        log.audit("leasebroker", "Expired capabilities revoked", {"count": removed})
    return removed
