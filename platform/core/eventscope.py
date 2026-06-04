"""
Event-scoped enforcement gate for IntentKernel MVP.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from . import capabilities, crypto, logger as log


ACTION_CAP_TYPES: dict[str, str] = {
    "draw": "display",
    "wait_event": "event",
    "get_resource": "resource",
    "put_resource": "resource",
    "network_request": "network",
    "schedule_notification": "notify",
    "invoke_capability": "invoke",
}


@dataclass
class AuthorizationResult:
    allowed: bool
    reason: str
    cap_type: str | None = None


def authorize(token: dict[str, Any], action: str, resource: dict[str, Any] | None = None) -> AuthorizationResult:
    if not isinstance(token, dict):
        return AuthorizationResult(False, "invalid_token")
    cap_data = token.get("capability")
    sig_data = token.get("signature")
    if not cap_data or not sig_data:
        return AuthorizationResult(False, "invalid_token")
    cap = capabilities.Capability(
        cap_id=cap_data["id"],
        cap_type=cap_data["type"],
        expires_at=cap_data["expires_at"],
        uses_left=cap_data["uses_left"],
        key=bytes.fromhex(cap_data["key"]),
        issued_at=cap_data["issued_at"],
        metadata=cap_data.get("metadata", {}),
    )
    expected_type = ACTION_CAP_TYPES.get(action)
    if expected_type and cap.cap_type != expected_type:
        return AuthorizationResult(False, "capability_type_mismatch", cap_type=cap.cap_type)
    if not crypto.verify(cap.to_payload(), sig_data):
        return AuthorizationResult(False, "signature_invalid", cap_type=cap.cap_type)
    ok, reason = capabilities.validate_and_consume(cap)
    if not ok:
        return AuthorizationResult(False, reason, cap_type=cap.cap_type)
    log.audit("eventscope", "Capability authorized", {"action": action, "cap_id": cap.cap_id, "resource": resource})
    return AuthorizationResult(True, "ok", cap_type=cap.cap_type)
